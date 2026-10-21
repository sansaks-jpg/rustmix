#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use rustmix_engine::*;

#[test]
fn blank_template_keeps_full_hd_fractional_timing_and_inactive_transports() {
    let format = VideoStandard::default();
    assert_eq!(format.canvas_label(), "Full HD (1920×1080)");
    assert_eq!(format.rate, Rate::new(30000, 1001).unwrap());
    assert_eq!(format.monitor_size(), [640, 360]);
    assert_eq!(format.program_size(), [1920, 1080]);
    assert_eq!(
        Engine::default().project().preferences.video_standard(),
        format
    );
    let mut engine = Engine::new_preset(format, 1000).unwrap();
    let before = engine.project().clone();
    engine.tick(15_000);
    engine.tick(120_000);
    for input in &engine.project().inputs {
        assert!(!input.has_playback());
        assert!(!input.has_audio());
        assert_eq!(engine.playback(input.id).unwrap().position_ms(), 0);
        for command in [
            Command::RestartInput(input.id),
            Command::SetInputPaused(input.id, false),
            Command::SetInputLoop(input.id, true),
        ] {
            assert!(engine.enabled(&command).is_err());
        }
    }
    assert_eq!(*engine.project(), before);
}

#[test]
fn committed_source_capabilities_override_names_colours_and_fallback_kinds() {
    let mut input = ProjectState::demo().inputs[0].clone();
    input.name = "Black / blank".into();
    input.color = [0; 3];
    assert!(
        input.has_playback(),
        "A black picture can still be moving video"
    );
    assert!(
        !input.has_audio(),
        "Procedural pictures do not produce samples"
    );
    input.config.media_path = Some("still.png".into());
    assert!(
        !input.has_playback(),
        "A still overrides the fallback generator"
    );
    input.config.media_path = None;
    input.kind = InputKind::Color;
    for (source, playback) in [
        (
            VideoSource::File {
                path: "movie.mp4".into(),
            },
            true,
        ),
        (
            VideoSource::Photos {
                paths: vec!["still.png".into()],
                interval_ms: 1000,
            },
            true,
        ),
        (
            VideoSource::Camera {
                device_id: "test-only".into(),
                format: None,
            },
            false,
        ),
        (
            VideoSource::Browser {
                url: "http://127.0.0.1/".into(),
                width: 1920,
                height: 1080,
            },
            false,
        ),
    ] {
        input.config.video_source = Some(source);
        assert_eq!(input.has_playback(), playback);
        assert!(
            input.has_audio(),
            "Video sources participate in audio routing, mixing and metering"
        );
    }
    input.config.video_source = None;
    input.config.audio_source = Some(AudioSource::WavFile {
        path: "sound.wav".into(),
    });
    assert!(input.has_audio() && input.has_playback());
    input.config.audio_source = Some(AudioSource::Microphone { device_id: None });
    assert!(input.has_audio() && !input.has_playback());
}

#[test]
fn offline_and_static_sources_do_not_accumulate_idle_time_on_reactivation() {
    let mut engine = Engine::new_preset(VideoStandard::default(), 0).unwrap();
    engine.tick(50_000);
    let mut input = engine.project().inputs[0].clone();
    input.kind = InputKind::Grid;
    engine
        .execute(Command::UpdateInput(input.clone()), 50_000)
        .unwrap();
    engine.tick(50_100);
    assert_eq!(engine.playback(input.id).unwrap().position_ms(), 100);
    input.online = false;
    engine
        .execute(Command::UpdateInput(input.clone()), 50_100)
        .unwrap();
    engine.tick(90_000);
    assert_eq!(engine.playback(input.id).unwrap().position_ms(), 100);
    input.online = true;
    engine
        .execute(Command::UpdateInput(input.clone()), 90_000)
        .unwrap();
    engine.tick(90_100);
    assert_eq!(engine.playback(input.id).unwrap().position_ms(), 200);
    assert_ne!(engine.project().program, engine.project().preview);
}
