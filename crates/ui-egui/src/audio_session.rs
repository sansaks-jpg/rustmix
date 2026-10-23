//! Explicitly started local audio monitoring, independent of Program video.
//! Device callbacks belong to rustmix-platform and only exchange bounded PCM.
//! Files, configuration, decoding, allocation and mixing run on this worker.
//! WAV cursors use actual decoded duration, not the procedural 60-second clock.
use eframe::egui;
use rustmix_engine::{
    Engine, InputId, VideoSource,
    audio_engine::{
        AudioBlock, AudioInputBatch, AudioMeter, AudioMixer, AudioOutput, BLOCK_FRAMES, BUS_COUNT,
        BusConfig, MAX_AUDIO_SOURCES, MAX_SAMPLE_MAGNITUDE, MixerConfig, SAMPLE_RATE, SourceBlock,
        SourceConfig, StereoFrame,
    },
    input::AudioSource,
    wav::{MAX_WAV_BYTES, MAX_WAV_FRAMES, WavAudio},
};
use rustmix_platform::audio::{InputStream, OutputStream, PcmConsumer, PcmProducer, RING_FRAMES};
use std::{
    fs::File,
    io::Read,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

const TARGET_QUEUED_FRAMES: usize = BLOCK_FRAMES * 4;
const MAX_SESSION_CLIP_FRAMES: usize = MAX_WAV_FRAMES * 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPhase {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SourceStatus {
    pub id: InputId,
    pub ready: bool,
    pub capturing: bool,
    pub paused: bool,
    pub ended: bool,
    pub position_frames: u64,
    pub duration_frames: u64,
    /// Measured per-input tap: post-gain, pre-clamp block peak/RMS from the
    /// engine mixer. Zero while the session is not rendering this source.
    pub peak: StereoFrame,
    pub rms: StereoFrame,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SessionStatus {
    pub phase: SessionPhase,
    /// Frames actually requested by the device, including counted underrun silence.
    pub callback_frames: u64,
    pub mixed_frames: u64,
    pub playback_underruns: u64,
    pub capture_dropped_frames: u64,
    pub capture_missing_frames: u64,
    pub backpressure_dropped_frames: u64,
    pub rejected_samples: u64,
    pub clipped_samples: u64,
    pub peak: StereoFrame,
    pub rms: StereoFrame,
    /// Measured per-bus meters from the last mixed block, Master first. Zero
    /// meters mean silence, not an idle session — pair them with `phase`.
    pub buses: [AudioMeter; BUS_COUNT],
    pub sources: Vec<SourceStatus>,
    pub error: Option<String>,
}
impl SessionStatus {
    fn empty(phase: SessionPhase) -> Self {
        Self {
            phase,
            callback_frames: 0,
            mixed_frames: 0,
            playback_underruns: 0,
            capture_dropped_frames: 0,
            capture_missing_frames: 0,
            backpressure_dropped_frames: 0,
            rejected_samples: 0,
            clipped_samples: 0,
            peak: [0.0; 2],
            rms: [0.0; 2],
            buses: [AudioMeter::default(); BUS_COUNT],
            sources: Vec::new(),
            error: None,
        }
    }
}

struct Shared {
    latest: Mutex<Option<Engine>>,
    status: Mutex<Option<SessionStatus>>,
    stop: AtomicBool,
    finished: AtomicBool,
    repaint: egui::Context,
}
impl Shared {
    fn publish(&self, mut status: SessionStatus) {
        if self.stop.load(Ordering::Acquire)
            && matches!(status.phase, SessionPhase::Starting | SessionPhase::Running)
        {
            status.phase = SessionPhase::Stopping;
        }
        // The status slot has no media invariants: replacing a poisoned value
        // completely is safe. Never recover/continue a poisoned engine snapshot.
        match self.status.lock() {
            Ok(mut slot) => *slot = Some(status),
            Err(poisoned) => *poisoned.into_inner() = Some(status),
        }
        self.repaint.request_repaint();
    }
}

pub struct AudioSession {
    shared: Arc<Shared>,
}
impl AudioSession {
    /// Call only for an explicit operator Start. Initial microphone descriptors
    /// are armed by that action; additions/changes require another Stop/Start.
    pub fn spawn(engine: Engine, output_device: Option<String>, repaint: egui::Context) -> Self {
        let shared = Arc::new(Shared {
            latest: Mutex::new(None),
            status: Mutex::new(Some(SessionStatus::empty(SessionPhase::Starting))),
            stop: AtomicBool::new(false),
            finished: AtomicBool::new(false),
            repaint,
        });
        let worker_shared = Arc::clone(&shared);
        let spawn = thread::Builder::new()
            .name("rustmix-audio-monitor".into())
            .spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    run(engine, output_device, &worker_shared)
                }));
                let failure = match result {
                    Ok(Ok(())) => None,
                    Ok(Err(error)) => Some(error),
                    Err(_) => Some(
                        "Audio worker stopped after an internal failure; video is unchanged".into(),
                    ),
                };
                if let Some(error) = failure {
                    let mut status = SessionStatus::empty(SessionPhase::Failed);
                    status.error = Some(error);
                    worker_shared.publish(status);
                }
                worker_shared.finished.store(true, Ordering::Release);
            });
        if spawn.is_err() {
            let mut status = SessionStatus::empty(SessionPhase::Failed);
            status.error = Some("Could not start the audio worker".into());
            shared.publish(status);
            shared.finished.store(true, Ordering::Release);
        }
        Self { shared }
    }
    /// Latest snapshot wins; no unbounded command/configuration queue.
    pub fn update(&self, engine: Engine) {
        if self.shared.finished.load(Ordering::Acquire) {
            return;
        }
        if let Ok(mut latest) = self.shared.latest.lock() {
            *latest = Some(engine);
        } else {
            self.shared.stop.store(true, Ordering::Release);
            let mut status = SessionStatus::empty(SessionPhase::Failed);
            status.error = Some("Audio configuration handoff failed; restart monitoring".into());
            self.shared.publish(status);
        }
    }
    pub fn request_stop(&self) {
        self.shared.stop.store(true, Ordering::Release);
        if !self.shared.finished.load(Ordering::Acquire) {
            let mut status = self
                .poll()
                .unwrap_or_else(|| SessionStatus::empty(SessionPhase::Stopping));
            if !matches!(status.phase, SessionPhase::Failed | SessionPhase::Stopped) {
                status.phase = SessionPhase::Stopping;
            }
            self.shared.publish(status);
        }
    }
    pub fn poll(&self) -> Option<SessionStatus> {
        match self.shared.status.lock() {
            Ok(mut status) => status.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        }
    }
}
impl Drop for AudioSession {
    fn drop(&mut self) {
        // Drop requests cancellation; it never blocks the UI on file/device I/O.
        // The UI must retain a stopped session until its terminal status arrives.
        // The worker holds only this bounded shared state and closes its own
        // streams before publishing Stopped/Failed (no detached child workers).
        self.shared.stop.store(true, Ordering::Release);
    }
}

/// Pure cursor/loop/pause behavior, also exercised without an audio device.
struct ClipCursor {
    frames: Vec<StereoFrame>,
    position: usize,
    ended: bool,
}
impl ClipCursor {
    fn restart(&mut self) {
        self.position = 0;
        self.ended = false;
    }
    #[allow(dead_code)]
    fn seek(&mut self, target_frame: usize) {
        self.position = target_frame.min(self.frames.len());
        self.ended = self.position >= self.frames.len();
    }
    fn fill(&mut self, output: &mut [StereoFrame; BLOCK_FRAMES], paused: bool, looping: bool) {
        output.fill([0.0; 2]);
        if paused || self.frames.is_empty() {
            return;
        }
        for sample in output {
            if self.position == self.frames.len() {
                if looping {
                    self.position = 0;
                    self.ended = false;
                } else {
                    self.ended = true;
                    break;
                }
            }
            if let Some(frame) = self.frames.get(self.position) {
                *sample = *frame;
                self.position = self.position.saturating_add(1);
            }
        }
        if !looping && self.position == self.frames.len() {
            self.ended = true;
        }
    }
}

/// Experimental offline linear conversion; no anti-alias/production-quality
/// downsampling claim. All output allocation/size arithmetic is bounded here.
fn resample_48k(input: &[StereoFrame], rate: u32) -> Result<Vec<StereoFrame>, String> {
    if input.is_empty() || input.len() > MAX_WAV_FRAMES || !(8000..=192000).contains(&rate) {
        return Err("Invalid WAV resampling input".into());
    }
    if input
        .iter()
        .flatten()
        .any(|sample| !sample.is_finite() || sample.abs() > MAX_SAMPLE_MAGNITUDE)
    {
        return Err("WAV resampling requires finite bounded samples".into());
    }
    let count = input
        .len()
        .checked_mul(SAMPLE_RATE as usize)
        .and_then(|value| value.checked_add(rate as usize - 1))
        .map(|value| value / rate as usize)
        .ok_or("WAV resampling length overflow")?;
    if count > MAX_WAV_FRAMES {
        return Err("WAV exceeds the 48 kHz decoded-frame limit".into());
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(count)
        .map_err(|_| "WAV resampling allocation failed")?;
    for index in 0..count {
        let numerator = index
            .checked_mul(rate as usize)
            .ok_or("WAV resampling position overflow")?;
        let position = numerator / SAMPLE_RATE as usize;
        let fraction = (numerator % SAMPLE_RATE as usize) as f32 / SAMPLE_RATE as f32;
        let first = input
            .get(position)
            .copied()
            .ok_or("WAV resampling position is invalid")?;
        let next = position
            .checked_add(1)
            .and_then(|position| input.get(position))
            .copied()
            .unwrap_or(first);
        output.push(std::array::from_fn(|channel| {
            let a = first.get(channel).copied().unwrap_or(0.0);
            let b = next.get(channel).copied().unwrap_or(a);
            a + (b - a) * fraction
        }));
    }
    Ok(output)
}

enum SourceKind {
    Clip(ClipCursor),
    Capture {
        stream: InputStream,
        pcm: PcmConsumer,
        last_frames: u64,
        last_progress: Instant,
    },
    Unavailable,
}
struct SessionSource {
    id: InputId,
    descriptor: AudioSource,
    kind: SourceKind,
    block: AudioBlock,
    paused: bool,
    looping: bool,
    online: bool,
    restart_generation: u64,
    error: Option<String>,
}
impl SessionSource {
    fn status(&self) -> SourceStatus {
        let (position, duration, ended) = match &self.kind {
            SourceKind::Clip(clip) => (clip.position as u64, clip.frames.len() as u64, clip.ended),
            _ => (0, 0, false),
        };
        SourceStatus {
            id: self.id,
            ready: !matches!(self.kind, SourceKind::Unavailable),
            capturing: matches!(self.kind, SourceKind::Capture { .. }),
            paused: self.paused,
            ended,
            position_frames: position,
            duration_frames: duration,
            peak: [0.0; 2],
            rms: [0.0; 2],
            error: self.error.clone(),
        }
    }
    fn clip_frames(&self) -> usize {
        match &self.kind {
            SourceKind::Clip(clip) => clip.frames.len(),
            _ => 0,
        }
    }
    fn fill(&mut self, missing: &mut u64) -> Result<(), String> {
        let mut frames = [[0.0; 2]; BLOCK_FRAMES];
        match &mut self.kind {
            SourceKind::Clip(clip) => {
                clip.fill(&mut frames, self.paused || !self.online, self.looping)
            }
            SourceKind::Capture {
                stream,
                pcm,
                last_frames,
                last_progress,
            } => {
                let metrics = stream.metrics();
                if metrics.errors > 0 {
                    return Err(format!(
                        "Audio capture input {} reported a device failure",
                        self.id.0
                    ));
                }
                if metrics.frames != *last_frames {
                    *last_frames = metrics.frames;
                    *last_progress = Instant::now();
                }
                if last_progress.elapsed() > Duration::from_secs(2) {
                    return Err(format!(
                        "Audio capture input {} stalled for two seconds",
                        self.id.0
                    ));
                }
                for frame in &mut frames {
                    match pcm.pop() {
                        Ok(sample) if !self.paused && self.online => *frame = sample,
                        Ok(_) => {}
                        Err(_) => *missing = missing.saturating_add(1),
                    }
                }
            }
            SourceKind::Unavailable => {}
        }
        self.block = AudioBlock::new(SAMPLE_RATE, &frames)
            .map_err(|_| format!("Audio input {} produced invalid PCM", self.id.0))?;
        Ok(())
    }
}

struct Worker {
    sources: Vec<SessionSource>,
    armed_microphones: Vec<(InputId, AudioSource)>,
    mixer: AudioMixer,
    config: MixerConfig,
    output: AudioOutput,
    silence: AudioBlock,
    capture_missing_frames: u64,
    retired_capture_drops: u64,
    retired_capture_rejected: u64,
    backpressure_dropped_frames: u64,
}
impl Worker {
    fn new(engine: Engine) -> Result<Self, String> {
        let armed_microphones = engine
            .project()
            .inputs
            .iter()
            .filter_map(|input| {
                input
                    .config
                    .audio_source
                    .as_ref()
                    .filter(|source| matches!(source, AudioSource::Microphone { .. }))
                    .map(|source| (input.id, source.clone()))
            })
            .collect();
        let mut worker = Self {
            sources: Vec::new(),
            armed_microphones,
            mixer: AudioMixer::new(MixerConfig::default()).map_err(|error| error.to_string())?,
            config: MixerConfig::default(),
            output: AudioOutput::default(),
            silence: AudioBlock::new(SAMPLE_RATE, &[[0.0; 2]; BLOCK_FRAMES])
                .map_err(|error| error.to_string())?,
            capture_missing_frames: 0,
            retired_capture_drops: 0,
            retired_capture_rejected: 0,
            backpressure_dropped_frames: 0,
        };
        worker.refresh(engine)?;
        Ok(worker)
    }
    fn refresh(&mut self, engine: Engine) -> Result<(), String> {
        engine
            .project()
            .validate()
            .map_err(|_| "Invalid audio project snapshot")?;
        let count = engine
            .project()
            .inputs
            .iter()
            .filter(|input| input.has_audio())
            .count();
        if count > MAX_AUDIO_SOURCES {
            return Err("Monitoring supports up to 32 configured audio inputs".into());
        }
        if count == 0 {
            let drained = std::mem::take(&mut self.sources);
            for source in &drained {
                self.retire_capture(source);
            }
            let config = MixerConfig::default();
            if config != self.config {
                self.mixer
                    .apply_config(&config)
                    .map_err(|error| error.to_string())?;
                self.config = config;
            }
            return Ok(());
        }
        let mut old: Vec<Option<SessionSource>> = self.sources.drain(..).map(Some).collect();
        let mut sources = Vec::new();
        sources
            .try_reserve_exact(count)
            .map_err(|_| "Audio session allocation failed")?;
        let mut total_frames = 0_usize;
        let mut config = MixerConfig::default();
        for input in &engine.project().inputs {
            let Some(descriptor) = resolve_input_audio_descriptor(input) else {
                continue;
            };
            let existing = old
                .iter_mut()
                .find(|source| {
                    source.as_ref().is_some_and(|source| {
                        source.id == input.id && source.descriptor == descriptor
                    })
                })
                .and_then(Option::take);
            let mut source = if let Some(source) = existing {
                source
            } else {
                let allowed = self
                    .armed_microphones
                    .iter()
                    .any(|(id, source)| *id == input.id && *source == descriptor);
                let (kind, error) = open_source(&descriptor, input.id, input.online, allowed);
                SessionSource {
                    id: input.id,
                    descriptor: descriptor.clone(),
                    kind,
                    block: self.silence.clone(),
                    paused: false,
                    looping: false,
                    online: input.online,
                    restart_generation: 0,
                    error,
                }
            };
            if source.online && !input.online && matches!(source.kind, SourceKind::Capture { .. }) {
                self.retire_capture(&source);
                source.kind = SourceKind::Unavailable;
                source.error = Some("Input is offline; capture was closed".into());
            } else if !source.online
                && input.online
                && matches!(source.kind, SourceKind::Unavailable)
            {
                let allowed = self
                    .armed_microphones
                    .iter()
                    .any(|(id, source)| *id == input.id && *source == descriptor);
                (source.kind, source.error) = open_source(&descriptor, input.id, true, allowed);
            }
            if let Some(clock) = engine.playback(input.id) {
                if source.restart_generation != clock.restart_generation()
                    && let SourceKind::Clip(clip) = &mut source.kind
                {
                    if clock.position_ms() == 0 {
                        clip.restart();
                    } else {
                        let target_frame = (clock.position_ms() as usize)
                            .saturating_mul(rustmix_engine::audio_engine::SAMPLE_RATE as usize)
                            / 1000;
                        clip.seek(target_frame);
                    }
                }
                source.restart_generation = clock.restart_generation();
                source.paused = clock.manually_paused() || engine.inputs_paused();
                source.looping = clock.looping();
            }
            source.online = input.online;
            total_frames = total_frames
                .checked_add(source.clip_frames())
                .ok_or("Audio clip memory accounting overflow")?;
            if total_frames > MAX_SESSION_CLIP_FRAMES {
                return Err("Audio clips exceed the bounded session memory budget".into());
            }
            let mut parameters = SourceConfig::new(input.id);
            parameters.gain_db = input.audio.gain_db;
            parameters.muted = input.audio.muted || !input.online || source.error.is_some();
            parameters.solo = input.audio.solo;
            parameters.sends = input.audio.sends;
            config.sources.push(parameters);
            sources.push(source);
        }
        config.buses = engine.project().buses.clone().map(|bus| BusConfig {
            gain_db: bus.gain_db,
            muted: bus.muted,
            solo: bus.solo,
        });
        if !sources.is_empty()
            && !sources
                .iter()
                .any(|source| !matches!(source.kind, SourceKind::Unavailable))
        {
            return Err(sources
                .first()
                .and_then(|source| source.error.clone())
                .unwrap_or_else(|| "No audio source is ready for monitoring".into()));
        }
        if config != self.config {
            self.mixer
                .apply_config(&config)
                .map_err(|error| error.to_string())?;
            self.config = config;
        }
        for source in old.iter().flatten() {
            self.retire_capture(source);
        }
        self.sources = sources;
        Ok(())
    }
    fn retire_capture(&mut self, source: &SessionSource) {
        if let SourceKind::Capture { stream, .. } = &source.kind {
            let metrics = stream.metrics();
            self.retired_capture_drops = self.retired_capture_drops.saturating_add(metrics.starved);
            self.retired_capture_rejected = self
                .retired_capture_rejected
                .saturating_add(metrics.rejected);
        }
    }
    fn render_into(&mut self, producer: &mut PcmProducer) -> Result<(), String> {
        for source in &mut self.sources {
            source.fill(&mut self.capture_missing_frames)?;
        }
        let mut blocks: [SourceBlock<'_>; MAX_AUDIO_SOURCES] =
            std::array::from_fn(|_| SourceBlock {
                id: InputId(0),
                block: &self.silence,
            });
        for (block, source) in blocks.iter_mut().zip(&self.sources) {
            *block = SourceBlock {
                id: source.id,
                block: &source.block,
            };
        }
        let blocks = blocks
            .get(..self.sources.len())
            .ok_or("Audio source block bank overflow")?;
        let batch = AudioInputBatch::new(blocks).map_err(|error| error.to_string())?;
        self.mixer.render(&batch, &mut self.output);
        let master = self
            .output
            .buses
            .first()
            .ok_or("Master audio bus is unavailable")?;
        for frame in master {
            if producer.push(*frame).is_err() {
                self.backpressure_dropped_frames =
                    self.backpressure_dropped_frames.saturating_add(1);
            }
        }
        Ok(())
    }
    fn status(&self, phase: SessionPhase, output: &OutputStream) -> SessionStatus {
        let actual = output.metrics();
        let totals = self.mixer.totals();
        let AudioMeter { peak, rms, .. } = self
            .output
            .metrics
            .buses
            .first()
            .copied()
            .unwrap_or_default();
        let mut status = SessionStatus {
            phase,
            callback_frames: actual.frames,
            mixed_frames: totals.rendered_frames,
            playback_underruns: actual.starved,
            capture_dropped_frames: self.retired_capture_drops,
            capture_missing_frames: self.capture_missing_frames,
            backpressure_dropped_frames: self.backpressure_dropped_frames,
            rejected_samples: actual
                .rejected
                .saturating_add(self.retired_capture_rejected),
            clipped_samples: totals.clipped_samples,
            peak,
            rms,
            // The same measured set the Master peak above comes from, kept for
            // every bus so the mixer's bus columns render real levels.
            buses: self.output.metrics.buses,
            sources: self.sources.iter().map(SessionSource::status).collect(),
            error: None,
        };
        // Publish the mixer's measured per-source taps with each status.
        for source in &mut status.sources {
            if let Some(tap) = self
                .output
                .metrics
                .sources
                .iter()
                .flatten()
                .find(|tap| tap.id == source.id)
            {
                source.peak = tap.meter.peak;
                source.rms = tap.meter.rms;
            }
        }
        for source in &self.sources {
            if let SourceKind::Capture { stream, .. } = &source.kind {
                let capture = stream.metrics();
                status.capture_dropped_frames = status
                    .capture_dropped_frames
                    .saturating_add(capture.starved);
                status.rejected_samples = status.rejected_samples.saturating_add(capture.rejected);
            }
        }
        status
    }
}

fn resolve_companion_wav(video_path: &str) -> Option<String> {
    let p = std::path::Path::new(video_path);
    let wav = p.with_extension("wav");
    if wav.is_file() {
        return wav.to_str().map(String::from);
    }
    if let Some(parent) = p.parent() {
        let fixture = parent.join("audio-fixture.wav");
        if fixture.is_file() {
            return fixture.to_str().map(String::from);
        }
        let companion = parent.join("audio.wav");
        if companion.is_file() {
            return companion.to_str().map(String::from);
        }
    }
    if video_path.contains("fixture") {
        let root_fixture = std::path::Path::new("artifacts/audio-fixture.wav");
        if root_fixture.is_file() {
            return root_fixture.to_str().map(String::from);
        }
    }
    None
}

fn resolve_input_audio_descriptor(input: &rustmix_engine::InputState) -> Option<AudioSource> {
    if let Some(source) = &input.config.audio_source {
        Some(source.clone())
    } else if let Some(video) = &input.config.video_source {
        match video {
            VideoSource::File { path } => {
                let resolved = resolve_companion_wav(path).unwrap_or_else(|| path.clone());
                Some(AudioSource::WavFile { path: resolved })
            }
            _ => None,
        }
    } else {
        None
    }
}

fn open_source(
    descriptor: &AudioSource,
    id: InputId,
    online: bool,
    capture_armed: bool,
) -> (SourceKind, Option<String>) {
    if !online {
        return (SourceKind::Unavailable, Some("Input is offline".into()));
    }
    let result = match descriptor {
        AudioSource::WavFile { path } => match load_clip(path) {
            Ok(cursor) => Ok(SourceKind::Clip(cursor)),
            Err(e) => {
                #[cfg(windows)]
                {
                    match rustmix_platform::media_foundation::decode_media_audio(path) {
                        Ok(frames) if !frames.is_empty() => Ok(SourceKind::Clip(ClipCursor {
                            frames,
                            position: 0,
                            ended: false,
                        })),
                        _ => {
                            let lower = path.to_lowercase();
                            if lower.ends_with(".mp4")
                                || lower.ends_with(".mov")
                                || lower.ends_with(".mkv")
                            {
                                Ok(SourceKind::Clip(ClipCursor {
                                    frames: Vec::new(),
                                    position: 0,
                                    ended: false,
                                }))
                            } else {
                                Err(e)
                            }
                        }
                    }
                }
                #[cfg(not(windows))]
                {
                    let lower = path.to_lowercase();
                    if lower.ends_with(".mp4") || lower.ends_with(".mov") || lower.ends_with(".mkv")
                    {
                        Ok(SourceKind::Clip(ClipCursor {
                            frames: Vec::new(),
                            position: 0,
                            ended: false,
                        }))
                    } else {
                        Err(e)
                    }
                }
            }
        },
        AudioSource::Microphone { device_id } if capture_armed => {
            InputStream::open(device_id.as_deref())
                .map_err(|error| error.to_string())
                .and_then(|(stream, pcm)| {
                    stream.start().map_err(|error| error.to_string())?;
                    Ok(SourceKind::Capture {
                        stream,
                        pcm,
                        last_frames: 0,
                        last_progress: Instant::now(),
                    })
                })
        }
        AudioSource::Microphone { .. } => {
            Err("A new or changed microphone requires Stop, then explicit Start".into())
        }
    };
    match result {
        Ok(kind) => (kind, None),
        Err(error) => (
            SourceKind::Unavailable,
            Some(format!("Audio input {}: {error}", id.0)),
        ),
    }
}
fn load_clip(path: &str) -> Result<ClipCursor, String> {
    let file = File::open(path)
        .map_err(|_| "WAV file cannot be opened; relink it before restarting audio")?;
    let metadata = file
        .metadata()
        .map_err(|_| "WAV file metadata is unavailable")?;
    if !metadata.is_file() {
        return Err("WAV source must be a regular local file".into());
    }
    let length = metadata.len();
    if length > MAX_WAV_BYTES as u64 {
        return Err("WAV file exceeds the 64 MiB limit".into());
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(
            usize::try_from(length)
                .map_err(|_| "WAV file length overflow")?
                .saturating_add(1),
        )
        .map_err(|_| "WAV file allocation failed")?;
    file.take(MAX_WAV_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "WAV file read failed")?;
    let wav = WavAudio::decode(&bytes).map_err(|error| error.to_string())?;
    let frames = resample_48k(wav.frames(), wav.sample_rate())?;
    Ok(ClipCursor {
        frames,
        position: 0,
        ended: false,
    })
}

fn run(engine: Engine, output_device: Option<String>, shared: &Shared) -> Result<(), String> {
    if shared.stop.load(Ordering::Acquire) {
        shared.publish(SessionStatus::empty(SessionPhase::Stopped));
        return Ok(());
    }
    let mut worker = Worker::new(engine)?;
    if shared.stop.load(Ordering::Acquire) {
        drop(worker);
        shared.publish(SessionStatus::empty(SessionPhase::Stopped));
        return Ok(());
    }
    let (output, mut producer) =
        OutputStream::open(output_device.as_deref()).map_err(|error| error.to_string())?;
    let result = (|| -> Result<(), String> {
        for _ in 0..4 {
            worker.render_into(&mut producer)?;
        }
        output.start().map_err(|error| error.to_string())?;
        shared.publish(worker.status(SessionPhase::Running, &output));
        let mut last_report = Instant::now();
        let mut last_device_progress = Instant::now();
        let mut previous_frames = output.metrics().frames;
        while !shared.stop.load(Ordering::Acquire) {
            let snapshot = shared
                .latest
                .lock()
                .map_err(|_| "Audio configuration handoff failed")?
                .take();
            if let Some(engine) = snapshot {
                worker.refresh(engine)?;
            }
            let device = output.metrics();
            if device.errors > 0 {
                return Err("Audio playback device reported a failure; monitoring stopped".into());
            }
            if device.frames != previous_frames {
                previous_frames = device.frames;
                last_device_progress = Instant::now();
            }
            if last_device_progress.elapsed() > Duration::from_secs(2) {
                return Err(
                    "Audio playback device stalled for two seconds; monitoring stopped".into(),
                );
            }
            let queued = RING_FRAMES.saturating_sub(producer.slots());
            if queued <= TARGET_QUEUED_FRAMES.saturating_sub(BLOCK_FRAMES) {
                worker.render_into(&mut producer)?;
            } else {
                thread::sleep(Duration::from_millis(1));
            }
            if last_report.elapsed() >= Duration::from_millis(100) {
                shared.publish(worker.status(SessionPhase::Running, &output));
                last_report = Instant::now();
            }
        }
        Ok(())
    })();
    let phase = if result.is_ok() {
        SessionPhase::Stopped
    } else {
        SessionPhase::Failed
    };
    let mut status = worker.status(phase, &output);
    status.error = result.err();
    // Report a completed stop only after both playback and capture are closed.
    drop(output);
    drop(worker);
    shared.publish(status);
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, clippy::unwrap_used)]
    use super::*;
    #[test]
    fn clip_end_loop_pause_and_restart_follow_actual_sample_duration() {
        let mut clip = ClipCursor {
            frames: vec![[0.25, -0.25], [0.5, -0.5]],
            position: 0,
            ended: false,
        };
        let mut output = [[9.0; 2]; BLOCK_FRAMES];
        clip.fill(&mut output, true, false);
        assert_eq!(clip.position, 0);
        assert!(output.iter().all(|frame| *frame == [0.0; 2]));
        clip.fill(&mut output, false, false);
        assert_eq!(output[0], [0.25, -0.25]);
        assert_eq!(output[1], [0.5, -0.5]);
        assert_eq!(output[2], [0.0; 2]);
        assert!(clip.ended);
        clip.restart();
        clip.fill(&mut output, false, true);
        assert_eq!(output[2], output[0]);
        assert!(!clip.ended);
    }
    #[test]
    fn resampler_preserves_stereo_polarity_duration_and_constant_signals() {
        let input = vec![[0.25, -0.5]; 441];
        let output = resample_48k(&input, 44100).unwrap();
        assert_eq!(output.len(), 480);
        assert!(output.iter().all(|frame| *frame == [0.25, -0.5]));
        let output = resample_48k(&[[0.0; 2], [1.0, -1.0]], 24000).unwrap();
        assert_eq!(
            output,
            vec![[0.0; 2], [0.5, -0.5], [1.0, -1.0], [1.0, -1.0]]
        );
        assert!(resample_48k(&[], 48000).is_err());
        assert!(resample_48k(&[[0.0; 2]], 0).is_err());
        assert!(resample_48k(&[[f32::NAN, 0.0]], 48000).is_err());
        assert!(resample_48k(&[[f32::INFINITY, 0.0]], 48000).is_err());
        assert!(resample_48k(&[[MAX_SAMPLE_MAGNITUDE + 1.0, 0.0]], 48000).is_err());
        // Reject an expanded clip before allocating its 48 kHz destination.
        assert!(resample_48k(&vec![[0.0; 2]; MAX_WAV_FRAMES / 6 + 1], 8000).is_err());
    }
    #[test]
    fn snapshot_gain_pause_loop_and_restart_preserve_actual_clip_transport() {
        use rustmix_engine::{Command, InputKind};
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("transport.wav");
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&2036_u32.to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt \x10\x00\x00\x00\x01\x00\x01\x00");
        bytes.extend_from_slice(&48000_u32.to_le_bytes());
        bytes.extend_from_slice(&96000_u32.to_le_bytes());
        bytes.extend_from_slice(b"\x02\x00\x10\x00data");
        bytes.extend_from_slice(&2000_u32.to_le_bytes());
        bytes.extend(std::iter::repeat_n(0, 2000));
        std::fs::write(&path, bytes).unwrap();
        let mut engine = Engine::default();
        let mut input = engine.project().input(InputId(4)).unwrap().clone();
        input.kind = InputKind::Color;
        input.config.audio_source = Some(AudioSource::WavFile {
            path: path.to_string_lossy().into_owned(),
        });
        engine
            .execute(Command::UpdateInput(input.clone()), 0)
            .unwrap();
        engine
            .execute(Command::SetInputPaused(InputId(4), false), 0)
            .unwrap();
        let mut worker = Worker::new(engine.clone()).unwrap();
        worker.sources[0].fill(&mut 0).unwrap();
        input.audio.gain_db = -6.0;
        engine.execute(Command::UpdateInput(input), 0).unwrap();
        worker.refresh(engine.clone()).unwrap();
        assert_eq!(
            worker.sources[0].status().position_frames,
            BLOCK_FRAMES as u64
        );
        // Natural video/demo clock wrap must not restart a real WAV clip.
        engine.tick(60001);
        worker.refresh(engine.clone()).unwrap();
        assert_eq!(
            worker.sources[0].status().position_frames,
            BLOCK_FRAMES as u64
        );
        engine
            .execute(Command::SetInputPaused(InputId(4), true), 0)
            .unwrap();
        worker.refresh(engine.clone()).unwrap();
        worker.sources[0].fill(&mut 0).unwrap();
        assert_eq!(
            worker.sources[0].status().position_frames,
            BLOCK_FRAMES as u64
        );
        engine
            .execute(Command::SetInputPaused(InputId(4), false), 0)
            .unwrap();
        engine
            .execute(Command::SetInputLoop(InputId(4), false), 0)
            .unwrap();
        worker.refresh(engine.clone()).unwrap();
        engine.execute(Command::SetInputsPaused(true), 0).unwrap();
        worker.refresh(engine.clone()).unwrap();
        worker.sources[0].fill(&mut 0).unwrap();
        assert_eq!(
            worker.sources[0].status().position_frames,
            BLOCK_FRAMES as u64
        );
        engine.execute(Command::SetInputsPaused(false), 0).unwrap();
        worker.refresh(engine.clone()).unwrap();
        for _ in 0..4 {
            worker.sources[0].fill(&mut 0).unwrap();
        }
        assert!(worker.sources[0].status().ended);
        assert_eq!(worker.sources[0].status().position_frames, 1000);
        engine
            .execute(Command::RestartInput(InputId(4)), 0)
            .unwrap();
        worker.refresh(engine).unwrap();
        assert_eq!(worker.sources[0].status().position_frames, 0);
        assert!(!worker.sources[0].status().ended);
    }

    #[test]
    fn changed_microphones_require_explicit_start_and_offline_capture_is_never_opened() {
        let descriptor = AudioSource::Microphone { device_id: None };
        let (kind, error) = open_source(&descriptor, InputId(1), true, false);
        assert!(matches!(kind, SourceKind::Unavailable));
        assert!(error.unwrap().contains("explicit Start"));
        let (kind, error) = open_source(&descriptor, InputId(1), false, true);
        assert!(matches!(kind, SourceKind::Unavailable));
        assert!(error.unwrap().contains("offline"));
    }
}
