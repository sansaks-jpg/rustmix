//! Control-thread device ownership; callbacks only transfer bounded PCM frames.
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub const SAMPLE_RATE: u32 = 48_000;
pub const RING_FRAMES: usize = 4_096;
pub type StereoFrame = [f32; 2];
pub type PcmProducer = rtrb::Producer<StereoFrame>;
pub type PcmConsumer = rtrb::Consumer<StereoFrame>;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("{0}")]
    Unavailable(String),
}
pub type Result<T> = std::result::Result<T, AudioError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Capture,
    Playback,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDevice {
    pub id: String,
    pub label: String,
    pub direction: Direction,
}

#[derive(Default)]
struct Counters {
    frames: AtomicU64,
    starved: AtomicU64,
    rejected: AtomicU64,
    errors: AtomicU64,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AudioMetrics {
    pub frames: u64,
    /// Missing playback frames or capture frames lost because the ring was full.
    pub starved: u64,
    pub rejected: u64,
    pub errors: u64,
}
impl Counters {
    fn snapshot(&self) -> AudioMetrics {
        AudioMetrics {
            frames: self.frames.load(Ordering::Relaxed),
            starved: self.starved.load(Ordering::Relaxed),
            rejected: self.rejected.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
        }
    }
}

#[cfg(any(windows, test))]
fn safe_sample(value: f32, counters: &Counters) -> f32 {
    if value.is_finite() {
        value.clamp(-1.0, 1.0)
    } else {
        counters.rejected.fetch_add(1, Ordering::Relaxed);
        0.0
    }
}

// These exact routines are used by native callbacks and by device-free tests.
// Fixed ring capacity, no locks, allocation, logging or OS access.
#[cfg(any(windows, test))]
fn playback(data: &mut [f32], consumer: &mut PcmConsumer, counters: &Counters) {
    let mut frames = data.chunks_exact_mut(2);
    for frame in &mut frames {
        let incoming = consumer.pop().unwrap_or_else(|_| {
            counters.starved.fetch_add(1, Ordering::Relaxed);
            [0.0; 2]
        });
        for (out, value) in frame.iter_mut().zip(incoming) {
            *out = safe_sample(value, counters);
        }
        counters.frames.fetch_add(1, Ordering::Relaxed);
    }
    for sample in frames.into_remainder() {
        *sample = 0.0;
        counters.rejected.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(any(windows, test))]
fn playback_i16(data: &mut [i16], consumer: &mut PcmConsumer, counters: &Counters) {
    let mut frames = data.chunks_exact_mut(2);
    for frame in &mut frames {
        let incoming = consumer.pop().unwrap_or_else(|_| {
            counters.starved.fetch_add(1, Ordering::Relaxed);
            [0.0; 2]
        });
        for (out, value) in frame.iter_mut().zip(incoming) {
            let s = safe_sample(value, counters);
            *out = (s * 32767.0) as i16;
        }
        counters.frames.fetch_add(1, Ordering::Relaxed);
    }
    for sample in frames.into_remainder() {
        *sample = 0;
        counters.rejected.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(any(windows, test))]
fn capture(data: &[f32], channels: usize, producer: &mut PcmProducer, counters: &Counters) {
    if !(1..=2).contains(&channels) {
        counters.rejected.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let mut frames = data.chunks_exact(channels);
    for frame in &mut frames {
        let left = frame.first().copied().unwrap_or(0.0);
        let right = frame.get(1).copied().unwrap_or(left);
        if producer
            .push([safe_sample(left, counters), safe_sample(right, counters)])
            .is_err()
        {
            counters.starved.fetch_add(1, Ordering::Relaxed);
        }
        counters.frames.fetch_add(1, Ordering::Relaxed);
    }
    if !frames.remainder().is_empty() {
        counters.rejected.fetch_add(1, Ordering::Relaxed);
    }
}

pub struct OutputStream {
    #[cfg(windows)]
    stream: cpal::Stream,
    counters: Arc<Counters>,
}
pub struct InputStream {
    #[cfg(windows)]
    stream: cpal::Stream,
    counters: Arc<Counters>,
}

impl OutputStream {
    /// Build a paused 48 kHz f32 stereo stream on the control thread.
    pub fn open(device_id: Option<&str>) -> Result<(Self, PcmProducer)> {
        #[cfg(windows)]
        {
            use cpal::traits::DeviceTrait;
            let device = native::find(device_id, Direction::Playback)?;
            let (config, format) = native::playback_config(&device)?;
            let (producer, mut consumer) = rtrb::RingBuffer::new(RING_FRAMES);
            let counters = Arc::new(Counters::default());
            let callback = Arc::clone(&counters);
            let faults = Arc::clone(&counters);
            let stream = match format {
                cpal::SampleFormat::I16 => device.build_output_stream(
                    config,
                    move |data: &mut [i16], _| playback_i16(data, &mut consumer, &callback),
                    move |_| {
                        faults.errors.fetch_add(1, Ordering::Relaxed);
                    },
                    Some(std::time::Duration::from_secs(2)),
                ),
                _ => device.build_output_stream(
                    config,
                    move |data: &mut [f32], _| playback(data, &mut consumer, &callback),
                    move |_| {
                        faults.errors.fetch_add(1, Ordering::Relaxed);
                    },
                    Some(std::time::Duration::from_secs(2)),
                ),
            }
            .map_err(native::error)?;
            Ok((Self { stream, counters }, producer))
        }
        #[cfg(not(windows))]
        {
            let _ = device_id;
            Err(unsupported())
        }
    }
    pub fn start(&self) -> Result<()> {
        #[cfg(windows)]
        {
            use cpal::traits::StreamTrait;
            self.stream.play().map_err(native::error)
        }
        #[cfg(not(windows))]
        Err(unsupported())
    }
    pub fn metrics(&self) -> AudioMetrics {
        self.counters.snapshot()
    }
}
impl InputStream {
    /// Build paused mono/stereo capture. Starting requires an operator action.
    pub fn open(device_id: Option<&str>) -> Result<(Self, PcmConsumer)> {
        #[cfg(windows)]
        {
            use cpal::traits::DeviceTrait;
            let device = native::find(device_id, Direction::Capture)?;
            let config = native::capture_config(&device)?;
            let channels = usize::from(config.channels);
            let (mut producer, consumer) = rtrb::RingBuffer::new(RING_FRAMES);
            let counters = Arc::new(Counters::default());
            let callback = Arc::clone(&counters);
            let faults = Arc::clone(&counters);
            let stream = device
                .build_input_stream(
                    config,
                    move |data: &[f32], _| capture(data, channels, &mut producer, &callback),
                    move |_| {
                        faults.errors.fetch_add(1, Ordering::Relaxed);
                    },
                    Some(std::time::Duration::from_secs(2)),
                )
                .map_err(native::error)?;
            Ok((Self { stream, counters }, consumer))
        }
        #[cfg(not(windows))]
        {
            let _ = device_id;
            Err(unsupported())
        }
    }
    pub fn start(&self) -> Result<()> {
        #[cfg(windows)]
        {
            use cpal::traits::StreamTrait;
            self.stream.play().map_err(native::error)
        }
        #[cfg(not(windows))]
        Err(unsupported())
    }
    pub fn metrics(&self) -> AudioMetrics {
        self.counters.snapshot()
    }
}

pub fn devices(direction: Direction) -> Result<Vec<AudioDevice>> {
    #[cfg(windows)]
    {
        native::devices(direction)
    }
    #[cfg(not(windows))]
    {
        let _ = direction;
        Err(unsupported())
    }
}
#[cfg(not(windows))]
fn unsupported() -> AudioError {
    AudioError::Unavailable("The WASAPI adapter requires Windows".into())
}

#[cfg(windows)]
mod native {
    use super::*;
    use cpal::traits::{DeviceTrait, HostTrait};
    const MAX_DEVICES: usize = 64;
    pub(super) fn error(error: cpal::Error) -> AudioError {
        AudioError::Unavailable(format!("Audio device operation failed: {error}"))
    }
    pub(super) fn devices(direction: Direction) -> Result<Vec<AudioDevice>> {
        let host = cpal::default_host();
        let all = host.devices().map_err(error)?;
        let mut result = Vec::new();
        for device in all.take(MAX_DEVICES) {
            let supported = match direction {
                Direction::Capture => device.supports_input(),
                Direction::Playback => device.supports_output(),
            };
            if supported {
                result.push(AudioDevice {
                    id: device.id().map_err(error)?.to_string(),
                    label: device.to_string(),
                    direction,
                });
            }
        }
        Ok(result)
    }
    pub(super) fn find(id: Option<&str>, direction: Direction) -> Result<cpal::Device> {
        let host = cpal::default_host();
        if let Some(id) = id {
            if id.is_empty() || id.len() > 4096 || id.chars().any(char::is_control) {
                return Err(AudioError::Unavailable(
                    "Invalid audio device identity".into(),
                ));
            }
            for device in host.devices().map_err(error)?.take(MAX_DEVICES) {
                if device
                    .id()
                    .is_ok_and(|candidate| candidate.to_string() == id)
                {
                    return Ok(device);
                }
            }
            return Err(AudioError::Unavailable(
                "Selected audio device is disconnected; select a device explicitly".into(),
            ));
        }
        match direction {
            Direction::Capture => host.default_input_device(),
            Direction::Playback => host.default_output_device(),
        }
        .ok_or_else(|| AudioError::Unavailable("No default audio device is available".into()))
    }
    fn supported(range: &cpal::SupportedStreamConfigRange, channels: u16) -> bool {
        range.channels() == channels
            && range.sample_format() == cpal::SampleFormat::F32
            && range.min_sample_rate() <= SAMPLE_RATE
            && range.max_sample_rate() >= SAMPLE_RATE
    }
    pub(super) fn playback_config(
        device: &cpal::Device,
    ) -> Result<(cpal::StreamConfig, cpal::SampleFormat)> {
        if let Ok(configs) = device.supported_output_configs() {
            let list: Vec<_> = configs.take(256).collect();
            for range in &list {
                if range.channels() == 2
                    && range.min_sample_rate() <= SAMPLE_RATE
                    && range.max_sample_rate() >= SAMPLE_RATE
                    && let Some(config) = range.try_with_sample_rate(SAMPLE_RATE)
                {
                    return Ok((config.config(), config.sample_format()));
                }
            }
            for range in &list {
                if range.channels() == 2 {
                    let rate = SAMPLE_RATE.clamp(range.min_sample_rate(), range.max_sample_rate());
                    if let Some(config) = range.try_with_sample_rate(rate) {
                        return Ok((config.config(), config.sample_format()));
                    }
                }
            }
        }
        let default_config = device.default_output_config().map_err(error)?;
        Ok((default_config.config(), default_config.sample_format()))
    }
    pub(super) fn capture_config(device: &cpal::Device) -> Result<cpal::StreamConfig> {
        if let Ok(configs) = device.supported_input_configs() {
            let list: Vec<_> = configs.take(256).collect();
            for range in &list {
                if (supported(range, 1) || supported(range, 2))
                    && let Some(config) = range.try_with_sample_rate(SAMPLE_RATE)
                {
                    return Ok(config.config());
                }
            }
            for range in &list {
                if range.channels() <= 2 {
                    let rate = SAMPLE_RATE.clamp(range.min_sample_rate(), range.max_sample_rate());
                    if let Some(config) = range.try_with_sample_rate(rate) {
                        return Ok(config.config());
                    }
                }
            }
        }
        let default_config = device.default_input_config().map_err(error)?;
        Ok(default_config.config())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    #[test]
    fn real_pcm_reaches_playback_with_explicit_underrun_silence() {
        let (mut producer, mut consumer) = rtrb::RingBuffer::new(4);
        producer.push([0.5, -0.25]).unwrap();
        producer.push([2.0, f32::NAN]).unwrap();
        let counters = Counters::default();
        let mut samples = [9.0; 7];
        playback(&mut samples, &mut consumer, &counters);
        assert_eq!(samples, [0.5, -0.25, 1.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(
            counters.snapshot(),
            AudioMetrics {
                frames: 3,
                starved: 1,
                rejected: 2,
                errors: 0
            }
        );
    }
    #[test]
    fn capture_duplicates_mono_and_counts_overflow_without_overwriting() {
        let (mut producer, mut consumer) = rtrb::RingBuffer::new(2);
        let counters = Counters::default();
        capture(&[0.25, -0.5, 0.75], 1, &mut producer, &counters);
        assert_eq!(consumer.pop().unwrap(), [0.25; 2]);
        assert_eq!(consumer.pop().unwrap(), [-0.5; 2]);
        assert!(consumer.pop().is_err());
        assert_eq!(counters.snapshot().frames, 3);
        assert_eq!(counters.snapshot().starved, 1);
        capture(&[0.0], 0, &mut producer, &counters);
        assert_eq!(counters.snapshot().rejected, 1);
    }
    #[test]
    fn stereo_capture_preserves_channels_and_sanitizes_bad_samples() {
        let (mut producer, mut consumer) = rtrb::RingBuffer::new(4);
        let counters = Counters::default();
        capture(
            &[0.25, -0.75, f32::INFINITY, 0.5, 0.25],
            2,
            &mut producer,
            &counters,
        );
        assert_eq!(consumer.pop().unwrap(), [0.25, -0.75]);
        assert_eq!(consumer.pop().unwrap(), [0.0, 0.5]);
        assert_eq!(counters.snapshot().rejected, 2);
    }
}
