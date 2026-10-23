#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use rustmix_engine::{InputId, audio_engine::*};

fn mixer(sources: Vec<SourceConfig>) -> AudioMixer {
    AudioMixer::new(MixerConfig {
        sources,
        ..MixerConfig::default()
    })
    .unwrap()
}

fn block(value: StereoFrame) -> AudioBlock {
    AudioBlock::new(SAMPLE_RATE, &[value; BLOCK_FRAMES]).unwrap()
}

fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.00001, "{a} != {b}");
}

#[test]
fn impulses_route_to_independent_buses_without_crosstalk() {
    let mut source = SourceConfig::new(InputId(1));
    source.sends = [false, true, false, false, false, false, false, false, false];
    let mut mixer = mixer(vec![source]);
    let mut samples = [[0.0; 2]; BLOCK_FRAMES];
    samples[0] = [0.5, -0.25];
    let block = AudioBlock::new(SAMPLE_RATE, &samples).unwrap();
    let sources = [SourceBlock {
        id: InputId(1),
        block: &block,
    }];
    let batch = AudioInputBatch::new(&sources).unwrap();
    let mut output = AudioOutput::default();
    mixer.render(&batch, &mut output);
    assert!(output.buses[0].iter().all(|frame| *frame == [0.0; 2]));
    assert_eq!(output.buses[1], samples);
    assert!(output.buses[2].iter().all(|frame| *frame == [0.0; 2]));
    close(output.metrics.buses[1].peak[0], 0.5);
    close(
        output.metrics.buses[1].rms[1],
        0.25 / (BLOCK_FRAMES as f32).sqrt(),
    );
    assert_eq!(output.metrics.underrun_sources, 0);
}

#[test]
fn parameter_changes_ramp_mute_pan_and_routing_without_a_step() {
    let source = SourceConfig::new(InputId(1));
    let mut config = MixerConfig {
        sources: vec![source],
        ..MixerConfig::default()
    };
    let mut mixer = AudioMixer::new(config.clone()).unwrap();
    let block = block([0.5, 0.5]);
    let sources = [SourceBlock {
        id: InputId(1),
        block: &block,
    }];
    let batch = AudioInputBatch::new(&sources).unwrap();
    let mut output = AudioOutput::default();
    config.sources[0].muted = true;
    mixer.apply_config(&config).unwrap();
    mixer.render(&batch, &mut output);
    assert!(output.buses[0][0][0] < 0.5 && output.buses[0][0][0] > 0.49);
    assert!(
        output.buses[0]
            .windows(2)
            .all(|pair| pair[1][0] <= pair[0][0])
    );
    assert_eq!(output.buses[0][RAMP_FRAMES - 1], [0.0; 2]);
    config.sources[0].muted = false;
    config.sources[0].pan = -1.0;
    config.sources[0].sends = [false, false, true, false, false, false, false, false, false];
    mixer.apply_config(&config).unwrap();
    mixer.render(&batch, &mut output);
    assert_eq!(output.buses[2][BLOCK_FRAMES - 1], [0.5, 0.0]);
    assert_eq!(output.buses[0][BLOCK_FRAMES - 1], [0.0; 2]);
}

#[test]
fn two_guest_returns_exclude_their_own_source_and_respect_independent_bus_sends() {
    let mut first = SourceConfig::new(InputId(1));
    first.sends = [true, true, false, false, false, false, false, false, false];
    let mut second = SourceConfig::new(InputId(2));
    second.sends = [true, true, false, false, false, false, false, false, false];
    let config = MixerConfig {
        sources: vec![first, second],
        guest_returns: [
            Some(GuestReturn {
                bus: AudioBus::BusA,
                exclude: InputId(1),
            }),
            Some(GuestReturn {
                bus: AudioBus::Master,
                exclude: InputId(2),
            }),
        ],
        ..MixerConfig::default()
    };
    let mut mixer = AudioMixer::new(config).unwrap();
    let first = block([0.2, -0.1]);
    let second = block([0.3, 0.4]);
    let sources = [
        SourceBlock {
            id: InputId(1),
            block: &first,
        },
        SourceBlock {
            id: InputId(2),
            block: &second,
        },
    ];
    let mut output = AudioOutput::default();
    mixer.render(&AudioInputBatch::new(&sources).unwrap(), &mut output);
    close(output.buses[0][0][0], 0.5);
    assert_eq!(output.guest_returns[0][0], [0.3, 0.4]);
    assert_eq!(output.guest_returns[1][0], [0.2, -0.1]);
    assert!(output.buses[2].iter().all(|frame| *frame == [0.0; 2]));
}

#[test]
fn solo_mute_clipping_and_underrun_have_explicit_sample_results_and_metrics() {
    let mut first = SourceConfig::new(InputId(1));
    first.solo = true;
    first.gain_db = 12.0;
    let mut second = SourceConfig::new(InputId(2));
    second.sends = [false, true, false, false, false, false, false, false, false];
    let mut mixer = mixer(vec![first, second]);
    let block = block([0.75, -0.75]);
    let sources = [SourceBlock {
        id: InputId(1),
        block: &block,
    }];
    let batch = AudioInputBatch::new(&sources).unwrap();
    let mut output = AudioOutput::default();
    mixer.render(&batch, &mut output);
    assert_eq!(output.buses[0][0], [1.0, -1.0]);
    assert_eq!(
        output.metrics.buses[0].clipped_samples,
        (BLOCK_FRAMES * 2) as u64
    );
    assert!(output.metrics.buses[0].peak[0] > 2.9);
    assert_eq!(output.metrics.underrun_sources, 1);
    assert!(output.buses[1].iter().all(|frame| *frame == [0.0; 2]));
    mixer.render(&AudioInputBatch::new(&[]).unwrap(), &mut output);
    assert!(
        output
            .buses
            .iter()
            .flatten()
            .all(|frame| *frame == [0.0; 2])
    );
    assert_eq!(output.metrics.underrun_sources, 2);
    assert_eq!(mixer.totals().rendered_frames, (BLOCK_FRAMES * 2) as u64);
}

#[test]
fn invalid_samples_and_configurations_are_rejected_before_render() {
    assert!(AudioBlock::new(44100, &[[0.0; 2]; BLOCK_FRAMES]).is_err());
    assert!(AudioBlock::new(SAMPLE_RATE, &[]).is_err());
    assert!(AudioBlock::new(SAMPLE_RATE, &[[f32::NAN, 0.0]; BLOCK_FRAMES]).is_err());
    assert!(AudioBlock::new(SAMPLE_RATE, &[[f32::INFINITY, 0.0]; BLOCK_FRAMES]).is_err());
    let source = SourceConfig::new(InputId(1));
    let mut config = MixerConfig {
        sources: vec![source.clone()],
        ..MixerConfig::default()
    };
    let mut mixer = AudioMixer::new(config.clone()).unwrap();
    config.sources[0].gain_db = f32::NAN;
    assert!(mixer.apply_config(&config).is_err());
    config.sources = vec![source.clone(), source];
    assert!(mixer.apply_config(&config).is_err());
    let block = block([0.25, 0.25]);
    let sources = [SourceBlock {
        id: InputId(1),
        block: &block,
    }];
    let mut output = AudioOutput::default();
    mixer.render(&AudioInputBatch::new(&sources).unwrap(), &mut output);
    assert_eq!(output.buses[0][0], [0.25, 0.25]);
    let duplicates = [sources[0], sources[0]];
    assert!(AudioInputBatch::new(&duplicates).is_err());
}

#[test]
fn all_parameter_limits_and_guest_exclusions_are_validated_before_commit() {
    let valid = MixerConfig {
        sources: vec![SourceConfig::new(InputId(1))],
        ..MixerConfig::default()
    };
    let mut malformed = Vec::new();
    for gain in [f32::NAN, f32::INFINITY, -91.0, 12.1] {
        let mut config = valid.clone();
        config.sources[0].gain_db = gain;
        malformed.push(config);
    }
    for pan in [f32::NAN, -1.1, 1.1] {
        let mut config = valid.clone();
        config.sources[0].pan = pan;
        malformed.push(config);
    }
    let mut invalid_bus = valid.clone();
    invalid_bus.buses[0].gain_db = f32::INFINITY;
    malformed.push(invalid_bus);
    let mut unknown_guest = valid.clone();
    unknown_guest.guest_returns[0] = Some(GuestReturn {
        bus: AudioBus::Master,
        exclude: InputId(99),
    });
    malformed.push(unknown_guest);
    let mut excess = valid.clone();
    excess.sources = (1..=MAX_AUDIO_SOURCES + 1)
        .map(|id| SourceConfig::new(InputId(id as u32)))
        .collect();
    malformed.push(excess);
    let mut mixer = AudioMixer::new(valid).unwrap();
    for config in malformed {
        assert!(mixer.apply_config(&config).is_err());
    }
    let block = block([0.25; 2]);
    let sources = [SourceBlock {
        id: InputId(1),
        block: &block,
    }];
    let mut output = AudioOutput::default();
    mixer.render(&AudioInputBatch::new(&sources).unwrap(), &mut output);
    assert_eq!(output.buses[0][0], [0.25; 2]);
    assert!(
        AudioBlock::new(
            SAMPLE_RATE,
            &[[MAX_SAMPLE_MAGNITUDE + 1.0, 0.0]; BLOCK_FRAMES]
        )
        .is_err()
    );
}

#[test]
fn newly_added_source_fades_in_and_a_silent_block_clears_old_output() {
    let mut mixer = AudioMixer::new(MixerConfig::default()).unwrap();
    mixer
        .apply_config(&MixerConfig {
            sources: vec![SourceConfig::new(InputId(1))],
            ..MixerConfig::default()
        })
        .unwrap();
    let block = block([0.5; 2]);
    let sources = [SourceBlock {
        id: InputId(1),
        block: &block,
    }];
    let mut output = AudioOutput::default();
    mixer.render(&AudioInputBatch::new(&sources).unwrap(), &mut output);
    assert!(output.buses[0][0][0] > 0.0 && output.buses[0][0][0] < 0.01);
    assert_eq!(output.buses[0][RAMP_FRAMES - 1], [0.5; 2]);
    mixer.render(&AudioInputBatch::new(&[]).unwrap(), &mut output);
    assert!(
        output
            .buses
            .iter()
            .flatten()
            .all(|sample| *sample == [0.0; 2])
    );
    assert_eq!(output.metrics.buses[0].peak, [0.0; 2]);
    assert_eq!(output.metrics.underrun_sources, 1);
}

#[test]
fn bus_solo_and_mute_affect_only_the_selected_destinations() {
    let mut source = SourceConfig::new(InputId(1));
    source.sends = [true; BUS_COUNT];
    let mut config = MixerConfig {
        sources: vec![source],
        ..MixerConfig::default()
    };
    config.buses[1].solo = true;
    config.buses[1].gain_db = -6.0;
    let mut mixer = AudioMixer::new(config.clone()).unwrap();
    let block = block([1.0, 1.0]);
    let sources = [SourceBlock {
        id: InputId(1),
        block: &block,
    }];
    let batch = AudioInputBatch::new(&sources).unwrap();
    let mut output = AudioOutput::default();
    mixer.render(&batch, &mut output);
    assert_eq!(output.buses[0][0], [0.0; 2]);
    close(output.buses[1][0][0], 10.0_f32.powf(-6.0 / 20.0));
    assert_eq!(output.buses[2][0], [0.0; 2]);
    config.buses[1].muted = true;
    mixer.apply_config(&config).unwrap();
    mixer.render(&batch, &mut output);
    assert_eq!(output.buses[1][BLOCK_FRAMES - 1], [0.0; 2]);
}

#[test]
fn per_source_taps_measure_post_gain_pre_clamp_levels_per_input() {
    let mut loud = SourceConfig::new(InputId(1));
    loud.gain_db = 12.0;
    loud.sends = [false, true, false, false, false, false, false, false, false];
    let mut quiet = SourceConfig::new(InputId(2));
    quiet.gain_db = -6.0;
    let mut muted = SourceConfig::new(InputId(3));
    muted.muted = true;
    let mut mixer = mixer(vec![loud, quiet, muted]);
    let mut loud_samples = [[0.0; 2]; BLOCK_FRAMES];
    loud_samples[0] = [0.75, -0.75];
    let loud_block = AudioBlock::new(SAMPLE_RATE, &loud_samples).unwrap();
    let quiet_block = block([0.5, -0.25]);
    let sources = [
        SourceBlock {
            id: InputId(1),
            block: &loud_block,
        },
        SourceBlock {
            id: InputId(2),
            block: &quiet_block,
        },
    ];
    let batch = AudioInputBatch::new(&sources).unwrap();
    let mut output = AudioOutput::default();
    mixer.render(&batch, &mut output);
    let loud_gain = 10.0_f32.powf(12.0 / 20.0);
    let quiet_gain = 10.0_f32.powf(-6.0 / 20.0);

    // The loud tap measures post-gain, pre-clamp: 0.75 at +12 dB is ~2.99,
    // far beyond full scale, and nothing clamps it at the tap.
    let loud_tap = output.metrics.sources[0].unwrap();
    assert_eq!(loud_tap.id, InputId(1));
    close(loud_tap.meter.peak[0], 0.75 * loud_gain);
    close(loud_tap.meter.peak[1], 0.75 * loud_gain);
    close(
        loud_tap.meter.rms[0],
        0.75 * loud_gain / (BLOCK_FRAMES as f32).sqrt(),
    );
    close(
        loud_tap.meter.rms[1],
        0.75 * loud_gain / (BLOCK_FRAMES as f32).sqrt(),
    );
    assert_eq!(loud_tap.meter.clipped_samples, 2);

    // The quiet tap is a constant block, so per-channel RMS equals its peak.
    let quiet_tap = output.metrics.sources[1].unwrap();
    assert_eq!(quiet_tap.id, InputId(2));
    close(quiet_tap.meter.peak[0], 0.5 * quiet_gain);
    close(quiet_tap.meter.peak[1], 0.25 * quiet_gain);
    close(quiet_tap.meter.rms[0], 0.5 * quiet_gain);
    close(quiet_tap.meter.rms[1], 0.25 * quiet_gain);
    assert_eq!(quiet_tap.meter.clipped_samples, 0);

    // Mute gates the tap to silence before bus sends are even considered.
    let muted_tap = output.metrics.sources[2].unwrap();
    assert_eq!(muted_tap.id, InputId(3));
    assert_eq!(muted_tap.meter.peak, [0.0; 2]);
    assert_eq!(muted_tap.meter.rms, [0.0; 2]);
    assert_eq!(muted_tap.meter.clipped_samples, 0);

    // Slots beyond the configured source count stay empty.
    assert!(output.metrics.sources[3..].iter().all(Option::is_none));

    // Cross-check the taps against the buses: Bus A carries only the loud
    // source and Master only the quiet one, both at unity send and bus gain,
    // so each pre-clamp bus meter peak must equal its source tap peak.
    close(output.metrics.buses[1].peak[0], loud_tap.meter.peak[0]);
    close(output.metrics.buses[1].peak[1], loud_tap.meter.peak[1]);
    close(output.metrics.buses[0].peak[0], quiet_tap.meter.peak[0]);
    close(output.metrics.buses[0].peak[1], quiet_tap.meter.peak[1]);
    assert!(output.buses[2].iter().all(|frame| *frame == [0.0; 2]));

    // A missing block is counted silence: taps stay identified but read zero.
    mixer.render(&AudioInputBatch::new(&[]).unwrap(), &mut output);
    for (index, id) in [(0, InputId(1)), (1, InputId(2)), (2, InputId(3))] {
        let tap = output.metrics.sources[index].unwrap();
        assert_eq!(tap.id, id);
        assert_eq!(tap.meter.peak, [0.0; 2]);
        assert_eq!(tap.meter.rms, [0.0; 2]);
        assert_eq!(tap.meter.clipped_samples, 0);
    }
    assert_eq!(output.metrics.underrun_sources, 3);
}
