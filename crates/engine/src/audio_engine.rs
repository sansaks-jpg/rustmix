//! Bounded, sample-based stereo mixing. This module does not open an audio device.
//!
//! Prepare blocks, batches and parameter snapshots on a control/producer thread.
//! `render` uses only fixed buffers and existing source state: no allocation,
//! locking, I/O, logging or arbitrary callbacks. Its fixed 256-frame quantum is
//! 48 kHz. A device callback with another quantum needs a preallocated adapter.
//! Missing blocks produce counted silence; unknown source IDs are not routed.

use crate::{InputId, Result, invalid};

pub const SAMPLE_RATE: u32 = 48_000;
pub const BLOCK_FRAMES: usize = 256;
/// The reference routing matrix: Master, buses A–G and Headphones.
pub const BUS_COUNT: usize = 9;
/// Labels for the nine-bus matrix, in mix index order.
pub const BUS_LABELS: [&str; BUS_COUNT] =
    ["Master", "A", "B", "C", "D", "E", "F", "G", "Headphones"];
pub const GUEST_RETURN_COUNT: usize = 2;
pub const MAX_AUDIO_SOURCES: usize = 32;
/// Five milliseconds at 48 kHz, applied independently of block boundaries.
pub const RAMP_FRAMES: usize = 240;
/// Bounded float headroom; values beyond full scale are counted and clipped.
pub const MAX_SAMPLE_MAGNITUDE: f32 = 16.0;
pub type StereoFrame = [f32; 2];

#[derive(Debug, Clone)]
pub struct AudioBlock {
    samples: [StereoFrame; BLOCK_FRAMES],
}
impl AudioBlock {
    /// Control-side validation. No implicit resampling or partial-block padding.
    pub fn new(sample_rate: u32, samples: &[StereoFrame]) -> Result<Self> {
        if sample_rate != SAMPLE_RATE {
            return Err(invalid("Audio mixer requires explicit 48000 Hz samples"));
        }
        if samples.len() != BLOCK_FRAMES {
            return Err(invalid("Audio mixer requires exactly 256 stereo frames"));
        }
        if samples
            .iter()
            .flatten()
            .any(|sample| !sample.is_finite() || sample.abs() > MAX_SAMPLE_MAGNITUDE)
        {
            return Err(invalid("Audio samples must be finite and within +/-16"));
        }
        let mut block = Self {
            samples: [[0.0; 2]; BLOCK_FRAMES],
        };
        block.samples.copy_from_slice(samples);
        Ok(block)
    }
    pub fn samples(&self) -> &[StereoFrame; BLOCK_FRAMES] {
        &self.samples
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SourceBlock<'a> {
    pub id: InputId,
    pub block: &'a AudioBlock,
}

/// Validated borrowed input list. Construction belongs outside the callback.
pub struct AudioInputBatch<'a> {
    sources: &'a [SourceBlock<'a>],
}
impl<'a> AudioInputBatch<'a> {
    pub fn new(sources: &'a [SourceBlock<'a>]) -> Result<Self> {
        if sources.len() > MAX_AUDIO_SOURCES {
            return Err(invalid("Audio source block count exceeds 32"));
        }
        for (index, source) in sources.iter().enumerate() {
            if source.id.0 == 0 || sources.iter().take(index).any(|old| old.id == source.id) {
                return Err(invalid("Audio source IDs must be nonzero and unique"));
            }
        }
        Ok(Self { sources })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBus {
    Master,
    BusA,
    BusB,
    BusC,
    BusD,
    BusE,
    BusF,
    BusG,
    Headphones,
}
impl AudioBus {
    pub fn index(self) -> usize {
        match self {
            Self::Master => 0,
            Self::BusA => 1,
            Self::BusB => 2,
            Self::BusC => 3,
            Self::BusD => 4,
            Self::BusE => 5,
            Self::BusF => 6,
            Self::BusG => 7,
            Self::Headphones => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceConfig {
    pub id: InputId,
    pub gain_db: f32,
    /// Stereo balance: -1 silences right, +1 silences left, centre is unity.
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    /// Independent sends to Master and buses A–G plus Headphones; mute and
    /// solo gate all sends.
    pub sends: [bool; BUS_COUNT],
}
impl SourceConfig {
    pub fn new(id: InputId) -> Self {
        Self {
            id,
            gain_db: 0.0,
            pan: 0.0,
            muted: false,
            solo: false,
            sends: [true, false, false, false, false, false, false, false, false],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BusConfig {
    pub gain_db: f32,
    pub muted: bool,
    pub solo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuestReturn {
    pub bus: AudioBus,
    /// This input is omitted before summation, gain and clipping for this return.
    pub exclude: InputId,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MixerConfig {
    pub sources: Vec<SourceConfig>,
    pub buses: [BusConfig; BUS_COUNT],
    pub guest_returns: [Option<GuestReturn>; GUEST_RETURN_COUNT],
}
impl MixerConfig {
    pub fn validate(&self) -> Result<()> {
        if self.sources.len() > MAX_AUDIO_SOURCES {
            return Err(invalid("Audio mixer supports at most 32 sources"));
        }
        for (index, source) in self.sources.iter().enumerate() {
            if source.id.0 == 0
                || self
                    .sources
                    .iter()
                    .take(index)
                    .any(|old| old.id == source.id)
            {
                return Err(invalid("Audio source IDs must be nonzero and unique"));
            }
            valid_gain(source.gain_db)?;
            if !source.pan.is_finite() || !(-1.0..=1.0).contains(&source.pan) {
                return Err(invalid("Audio pan must be finite and between -1 and 1"));
            }
        }
        for bus in &self.buses {
            valid_gain(bus.gain_db)?;
        }
        for destination in self.guest_returns.iter().flatten() {
            if !self
                .sources
                .iter()
                .any(|source| source.id == destination.exclude)
            {
                return Err(invalid(
                    "Guest return exclusion must identify a configured source",
                ));
            }
        }
        Ok(())
    }
}

fn valid_gain(gain: f32) -> Result<()> {
    if !gain.is_finite() || !(-90.0..=12.0).contains(&gain) {
        Err(invalid(
            "Audio gain must be finite and between -90 and +12 dB",
        ))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct Ramp {
    current: f32,
    target: f32,
    remaining: usize,
}
impl Ramp {
    fn new(value: f32) -> Self {
        Self {
            current: value,
            target: value,
            remaining: 0,
        }
    }
    fn set(&mut self, target: f32) {
        if self.target != target {
            self.target = target;
            self.remaining = RAMP_FRAMES;
        }
    }
    fn next(&mut self) -> f32 {
        if self.remaining > 0 {
            self.current += (self.target - self.current) / self.remaining as f32;
            self.remaining -= 1;
            if self.remaining == 0 {
                self.current = self.target;
            }
        }
        self.current
    }
}

fn source_gains(source: &SourceConfig, solo: bool) -> StereoFrame {
    if source.muted || (solo && !source.solo) {
        return [0.0; 2];
    }
    let gain = 10.0_f32.powf(source.gain_db / 20.0);
    [
        gain * (1.0 - source.pan.max(0.0)),
        gain * (1.0 + source.pan.min(0.0)),
    ]
}
fn bus_gain(bus: &BusConfig, solo: bool) -> f32 {
    if bus.muted || (solo && !bus.solo) {
        0.0
    } else {
        10.0_f32.powf(bus.gain_db / 20.0)
    }
}

#[derive(Debug, Clone)]
struct SourceRuntime {
    id: InputId,
    gain: [Ramp; 2],
    sends: [Ramp; BUS_COUNT],
}
impl SourceRuntime {
    fn new(source: &SourceConfig, solo: bool) -> Self {
        Self {
            id: source.id,
            gain: source_gains(source, solo).map(Ramp::new),
            sends: source
                .sends
                .map(|enabled| Ramp::new(if enabled { 1.0 } else { 0.0 })),
        }
    }
    fn apply(&mut self, source: &SourceConfig, solo: bool) {
        for (ramp, target) in self.gain.iter_mut().zip(source_gains(source, solo)) {
            ramp.set(target);
        }
        for (ramp, enabled) in self.sends.iter_mut().zip(source.sends) {
            ramp.set(if enabled { 1.0 } else { 0.0 });
        }
    }
}

/// Meters measure post-gain, pre-clamp samples. RMS is per-channel block RMS,
/// not loudness or true peak. Hard-clipped output samples are counted per channel.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AudioMeter {
    pub peak: StereoFrame,
    pub rms: StereoFrame,
    pub clipped_samples: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AudioMetrics {
    pub buses: [AudioMeter; BUS_COUNT],
    pub guest_returns: [AudioMeter; GUEST_RETURN_COUNT],
    pub underrun_sources: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AudioTotals {
    pub rendered_frames: u64,
    pub underrun_blocks: u64,
    pub clipped_samples: u64,
}

/// Caller-owned, reusable output. Render always overwrites every sample/meter.
pub struct AudioOutput {
    pub buses: [[StereoFrame; BLOCK_FRAMES]; BUS_COUNT],
    pub guest_returns: [[StereoFrame; BLOCK_FRAMES]; GUEST_RETURN_COUNT],
    pub metrics: AudioMetrics,
}
impl Default for AudioOutput {
    fn default() -> Self {
        Self {
            buses: [[[0.0; 2]; BLOCK_FRAMES]; BUS_COUNT],
            guest_returns: [[[0.0; 2]; BLOCK_FRAMES]; GUEST_RETURN_COUNT],
            metrics: AudioMetrics::default(),
        }
    }
}

pub struct AudioMixer {
    sources: Vec<SourceRuntime>,
    buses: [Ramp; BUS_COUNT],
    guest_returns: [Option<GuestReturn>; GUEST_RETURN_COUNT],
    totals: AudioTotals,
}
impl AudioMixer {
    /// Control-side construction. Initial parameters apply without a startup ramp.
    pub fn new(config: MixerConfig) -> Result<Self> {
        config.validate()?;
        let mut sources = Vec::new();
        sources
            .try_reserve_exact(config.sources.len())
            .map_err(|_| invalid("Audio source state allocation failed"))?;
        let solo = config.sources.iter().any(|source| source.solo);
        for source in &config.sources {
            sources.push(SourceRuntime::new(source, solo));
        }
        let bus_solo = config.buses.iter().any(|bus| bus.solo);
        Ok(Self {
            sources,
            buses: config.buses.map(|bus| Ramp::new(bus_gain(&bus, bus_solo))),
            guest_returns: config.guest_returns,
            totals: AudioTotals::default(),
        })
    }

    /// Control-side transaction; validation/allocation finish before state changes.
    /// Existing source gain/pan/mute/solo/send and bus changes ramp for 240 samples.
    /// Fade a source to mute before removing it. Topology/guest-exclusion changes
    /// are immediate, and require an external safe handoff between render calls.
    pub fn apply_config(&mut self, config: &MixerConfig) -> Result<()> {
        config.validate()?;
        let solo = config.sources.iter().any(|source| source.solo);
        let mut sources = Vec::new();
        sources
            .try_reserve_exact(config.sources.len())
            .map_err(|_| invalid("Audio source state allocation failed"))?;
        for source in &config.sources {
            let mut runtime = self
                .sources
                .iter()
                .find(|old| old.id == source.id)
                .cloned()
                .unwrap_or_else(|| {
                    let mut runtime = SourceRuntime::new(source, solo);
                    runtime.gain = [Ramp::new(0.0); 2];
                    runtime
                });
            runtime.apply(source, solo);
            sources.push(runtime);
        }
        let bus_solo = config.buses.iter().any(|bus| bus.solo);
        for (ramp, bus) in self.buses.iter_mut().zip(&config.buses) {
            ramp.set(bus_gain(bus, bus_solo));
        }
        self.sources = sources;
        self.guest_returns = config.guest_returns;
        Ok(())
    }
    pub fn totals(&self) -> AudioTotals {
        self.totals
    }

    /// Bounded render path: no allocation, locks, I/O, logging or fallible parsing.
    /// Counter saturation at u64::MAX is intentional and never affects samples.
    pub fn render(&mut self, input: &AudioInputBatch<'_>, output: &mut AudioOutput) {
        *output = AudioOutput::default();
        for source in &mut self.sources {
            let block = input
                .sources
                .iter()
                .find(|block| block.id == source.id)
                .map(|block| block.block);
            if block.is_none() {
                output.metrics.underrun_sources = output.metrics.underrun_sources.saturating_add(1);
            }
            for index in 0..BLOCK_FRAMES {
                let [left, right] = block
                    .and_then(|block| block.samples.get(index))
                    .copied()
                    .unwrap_or([0.0; 2]);
                let [gain_left, gain_right] = &mut source.gain;
                let sample = [left * gain_left.next(), right * gain_right.next()];
                for (bus_index, (bus_frames, send)) in
                    output.buses.iter_mut().zip(&mut source.sends).enumerate()
                {
                    let send = send.next();
                    if let Some(frame) = bus_frames.get_mut(index) {
                        for (sum, sample) in frame.iter_mut().zip(sample) {
                            *sum += sample * send;
                        }
                    }
                    for (guest_frames, destination) in
                        output.guest_returns.iter_mut().zip(self.guest_returns)
                    {
                        if destination.is_some_and(|destination| {
                            destination.bus.index() == bus_index && destination.exclude != source.id
                        }) && let Some(frame) = guest_frames.get_mut(index)
                        {
                            for (sum, sample) in frame.iter_mut().zip(sample) {
                                *sum += sample * send;
                            }
                        }
                    }
                }
            }
        }
        for (bus_index, (bus_frames, gain)) in
            output.buses.iter_mut().zip(&mut self.buses).enumerate()
        {
            for (index, frame) in bus_frames.iter_mut().enumerate() {
                let gain = gain.next();
                for sample in frame {
                    *sample *= gain;
                }
                for (guest_frames, destination) in
                    output.guest_returns.iter_mut().zip(self.guest_returns)
                {
                    if destination.is_some_and(|destination| destination.bus.index() == bus_index)
                        && let Some(frame) = guest_frames.get_mut(index)
                    {
                        for sample in frame {
                            *sample *= gain;
                        }
                    }
                }
            }
        }
        for (meter, frames) in output.metrics.buses.iter_mut().zip(&mut output.buses) {
            *meter = measure_and_clip(frames);
        }
        for (meter, frames) in output
            .metrics
            .guest_returns
            .iter_mut()
            .zip(&mut output.guest_returns)
        {
            *meter = measure_and_clip(frames);
        }
        self.totals.rendered_frames = self
            .totals
            .rendered_frames
            .saturating_add(BLOCK_FRAMES as u64);
        if output.metrics.underrun_sources > 0 {
            self.totals.underrun_blocks = self.totals.underrun_blocks.saturating_add(1);
        }
        for meter in output
            .metrics
            .buses
            .iter()
            .chain(&output.metrics.guest_returns)
        {
            self.totals.clipped_samples = self
                .totals
                .clipped_samples
                .saturating_add(meter.clipped_samples);
        }
    }
}

fn measure_and_clip(frames: &mut [StereoFrame; BLOCK_FRAMES]) -> AudioMeter {
    let mut meter = AudioMeter::default();
    let mut squares = [0.0_f64; 2];
    for frame in frames {
        for ((sample, peak), square) in frame.iter_mut().zip(&mut meter.peak).zip(&mut squares) {
            *peak = peak.max(sample.abs());
            *square += f64::from(*sample).powi(2);
            if sample.abs() > 1.0 {
                meter.clipped_samples = meter.clipped_samples.saturating_add(1);
            }
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
    for (rms, square) in meter.rms.iter_mut().zip(squares) {
        *rms = (square / BLOCK_FRAMES as f64).sqrt() as f32;
    }
    meter
}
