//! Bounded source owners for MP4, cameras, Photos and Browser frames.
//! No embedded MP4/browser/camera audio, GPU decode or A/V synchronization claim.
//! Open/read/seek are synchronous Media Foundation calls on their owner worker.
//! Stop/Drop request cancellation without blocking the UI; a blocked MF call
//! can delay termination indefinitely. Its global worker permit is retained
//! until actual termination, preventing unbounded abandoned decoder workers.

use eframe::egui;
#[cfg(any(windows, test))]
use rustmix_engine::video_source::{VideoFeedError, VideoFrame, VideoFrameMetadata};
use rustmix_engine::{
    InputId, VideoSource,
    video_source::{VideoFeed, VideoSourceHealth},
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
#[cfg(windows)]
use std::time::Instant;
#[cfg(any(windows, test))]
use std::{thread, time::Duration};

/// Bounded number of decoder owner threads, including abandoned ones that have
/// not finished yet. Only a Windows build can start an owner.
#[cfg_attr(not(windows), allow(dead_code))]
pub const MAX_VIDEO_PLAYERS: usize = 8;
#[cfg(windows)]
static LIVE_WORKERS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoControl {
    pub paused: bool,
    pub looping: bool,
    pub restart_generation: u64,
}
impl Default for VideoControl {
    fn default() -> Self {
        Self {
            paused: false,
            looping: true,
            restart_generation: 0,
        }
    }
}
struct Shared {
    feed: Arc<VideoFeed>,
    control: Mutex<VideoControl>,
    stop: AtomicBool,
    finished: AtomicBool,
    failure: Mutex<Option<String>>,
    repaint: egui::Context,
}
impl Shared {
    fn record_failure(&self, message: &str) {
        let reason = safe_reason(message);
        if let Ok(mut failure) = self.failure.lock() {
            *failure = Some(reason.clone());
        }
        let _ = self.feed.set_health(VideoSourceHealth::Failed { reason });
        self.stop.store(true, Ordering::Release);
        self.repaint.request_repaint();
    }
}

pub struct VideoPlayer {
    shared: Arc<Shared>,
}
impl VideoPlayer {
    /// Pure descriptor validation and thread creation only; no file/device I/O
    /// occurs on the UI thread. Each generation owns its own immutable feed ID.
    pub fn spawn_source(
        input_id: InputId,
        generation: u64,
        source: VideoSource,
        repaint: egui::Context,
    ) -> Result<Self, String> {
        #[cfg(windows)]
        {
            source.validate().map_err(|e| e.to_string())?;
            let feed =
                Arc::new(VideoFeed::new(input_id, generation).map_err(|error| error.to_string())?);
            let permit = Permit::acquire()?;
            let shared = Arc::new(Shared {
                feed,
                control: Mutex::new(VideoControl::default()),
                stop: AtomicBool::new(false),
                finished: AtomicBool::new(false),
                failure: Mutex::new(None),
                repaint,
            });
            let worker = Arc::clone(&shared);
            thread::Builder::new()
                .name(format!("rustmix-video-{}", input_id.0))
                .spawn(move || {
                    let result =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match &source {
                            VideoSource::File { path } => {
                                native::run(&worker, input_id, generation, path, None, false)
                            }
                            VideoSource::Camera { device_id, format } => native::run(
                                &worker,
                                input_id,
                                generation,
                                device_id,
                                format.as_ref(),
                                true,
                            ),
                            _ => source_workers::run(&worker, input_id, generation, &source),
                        }));
                    let failure = match result {
                        Ok(Ok(())) => None,
                        Ok(Err(error)) => Some(error),
                        Err(_) => {
                            Some("Video owner worker stopped after an internal failure".into())
                        }
                    };
                    if let Some(reason) = failure {
                        worker.record_failure(&reason);
                    }
                    let terminal = match worker.failure.lock() {
                        Ok(failure) => match failure.as_ref() {
                            Some(reason) => VideoSourceHealth::Failed {
                                reason: reason.clone(),
                            },
                            None => VideoSourceHealth::Stopped,
                        },
                        Err(_) => VideoSourceHealth::Failed {
                            reason: "Video worker status handoff failed".into(),
                        },
                    };
                    if let Err(error) = set_health_bounded(&worker.feed, terminal) {
                        worker.record_failure(&error);
                    }
                    drop(permit);
                    worker.finished.store(true, Ordering::Release);
                    worker.repaint.request_repaint();
                })
                .map_err(|_| "Could not create a video owner worker".to_owned())?;
            Ok(Self { shared })
        }
        #[cfg(not(windows))]
        {
            let _ = (input_id, generation, source, repaint);
            Err("Local MP4 video decoding requires Windows Media Foundation".into())
        }
    }
    pub fn feed(&self) -> Arc<VideoFeed> {
        Arc::clone(&self.shared.feed)
    }
    pub fn update(&self, paused: bool, looping: bool, restart_generation: u64) {
        if self.shared.finished.load(Ordering::Acquire) {
            return;
        }
        if let Ok(mut control) = self.shared.control.lock() {
            *control = VideoControl {
                paused,
                looping,
                restart_generation,
            };
        } else {
            self.shared
                .record_failure("Video control handoff failed; recreate this source");
        }
    }
    pub fn stop(&self) {
        self.shared.stop.store(true, Ordering::Release);
    }
    pub fn is_finished(&self) -> bool {
        self.shared.finished.load(Ordering::Acquire)
    }
    /// Separate error channel remains readable if a feed lock becomes poisoned.
    pub fn error(&self) -> Option<String> {
        match self.shared.failure.lock() {
            Ok(failure) => failure.clone(),
            Err(_) => Some("Video status handoff failed".into()),
        }
    }
}
impl Drop for VideoPlayer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn safe_reason(message: &str) -> String {
    let mut reason = String::with_capacity(256);
    for character in message.chars().filter(|character| !character.is_control()) {
        if reason.len().saturating_add(character.len_utf8()) > 256 {
            break;
        }
        reason.push(character);
    }
    if reason.trim().is_empty() {
        "Video source unavailable".into()
    } else {
        reason
    }
}

#[cfg(windows)]
struct Permit;
#[cfg(windows)]
impl Permit {
    fn acquire() -> Result<Self, String> {
        LIVE_WORKERS
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count
                    .checked_add(1)
                    .filter(|count| *count <= MAX_VIDEO_PLAYERS)
            })
            .map_err(|_| {
                "At most eight video owner workers may run; wait for stopped readers to terminate"
                    .to_owned()
            })?;
        Ok(Self)
    }
}
#[cfg(windows)]
impl Drop for Permit {
    fn drop(&mut self) {
        let _ = LIVE_WORKERS.fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
            count.checked_sub(1)
        });
    }
}
#[cfg(any(windows, test))]
const STATUS_LIMIT: u32 = 120;
#[cfg(any(windows, test))]
fn set_health_bounded(feed: &VideoFeed, health: VideoSourceHealth) -> Result<(), String> {
    for _ in 0..8 {
        match feed.set_health(health.clone()) {
            Ok(()) => return Ok(()),
            Err(VideoFeedError::Busy) => thread::sleep(Duration::from_millis(2)),
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Video health feed remained busy after a bounded handoff".into())
}

#[cfg(any(windows, test))]
#[derive(Default)]
struct Pacer {
    anchor_pts: Option<i64>,
    anchor_ns: u64,
    last_pts: Option<i64>,
    paused_since: Option<u64>,
    next_anchor: Option<u64>,
}
#[cfg(any(windows, test))]
impl Pacer {
    fn reset(&mut self, next_anchor: Option<u64>) {
        *self = Self {
            next_anchor,
            ..Self::default()
        };
    }
    fn pause(&mut self, paused: bool, now: u64) -> Result<(), String> {
        if paused {
            if self.paused_since.is_none() {
                self.paused_since = Some(now);
            }
        } else if let Some(start) = self.paused_since.take() {
            let elapsed = now
                .checked_sub(start)
                .ok_or("Video monotonic clock regressed")?;
            if self.anchor_pts.is_some() {
                self.anchor_ns = self
                    .anchor_ns
                    .checked_add(elapsed)
                    .ok_or("Video pause timeline overflow")?;
            } else if let Some(anchor) = &mut self.next_anchor {
                *anchor = anchor
                    .checked_add(elapsed)
                    .ok_or("Video loop pause timeline overflow")?;
            }
        }
        Ok(())
    }
    fn observe(&mut self, pts: i64, discontinuity: bool, now: u64) -> Result<(), String> {
        if self.last_pts.is_some_and(|last| pts < last) && !discontinuity {
            return Err("Video presentation timestamp regressed without discontinuity".into());
        }
        if self.anchor_pts.is_none() || discontinuity {
            self.anchor_pts = Some(pts);
            self.anchor_ns = self.next_anchor.take().unwrap_or(now);
            if self.paused_since.is_some() {
                self.paused_since = Some(now);
            }
        }
        self.last_pts = Some(pts);
        Ok(())
    }
    fn deadline(&self, pts: i64) -> Result<u64, String> {
        let anchor = self
            .anchor_pts
            .ok_or("Video presentation anchor is missing")?;
        let relative = pts
            .checked_sub(anchor)
            .ok_or("Video presentation timestamp overflow")?;
        let nanos = u64::try_from(
            i128::from(relative)
                .checked_mul(100)
                .ok_or("Video presentation time overflow")?,
        )
        .map_err(|_| "Video presentation time is negative or too large")?;
        self.anchor_ns
            .checked_add(nanos)
            .ok_or_else(|| "Video presentation deadline overflow".into())
    }
}

#[cfg(any(windows, test))]
enum ReadEvent {
    Frame {
        frame: VideoFrame,
        nominal_duration_100ns: Option<i64>,
    },
    NoSample,
    StreamTick,
    End,
}
#[cfg(any(windows, test))]
trait FrameReader {
    fn read(&mut self) -> Result<ReadEvent, String>;
    fn seek_zero(&mut self) -> Result<(), String>;
}
#[cfg(any(windows, test))]
struct Step {
    wait: Duration,
    changed: bool,
}
#[cfg(any(windows, test))]
impl Step {
    fn wait(nanos: u64) -> Self {
        Self {
            wait: Duration::from_nanos(nanos.clamp(100_000, 10_000_000)),
            changed: false,
        }
    }
    fn changed() -> Self {
        Self {
            wait: Duration::from_millis(1),
            changed: true,
        }
    }
}

#[cfg(any(windows, test))]
struct Playback<R> {
    reader: R,
    feed: Arc<VideoFeed>,
    pacer: Pacer,
    pending: Option<VideoFrame>,
    pending_observed: bool,
    force_discontinuity: bool,
    restart_generation: Option<u64>,
    poster: bool,
    ended: bool,
    status_reads: u32,
    empty_loops: u32,
    publish_busy: u32,
    last_due: Option<u64>,
    last_duration_ns: u64,
}
#[cfg(any(windows, test))]
impl<R: FrameReader> Playback<R> {
    fn new(reader: R, feed: Arc<VideoFeed>) -> Self {
        Self {
            reader,
            feed,
            pacer: Pacer::default(),
            pending: None,
            pending_observed: false,
            force_discontinuity: true,
            restart_generation: None,
            poster: true,
            ended: false,
            status_reads: 0,
            empty_loops: 0,
            publish_busy: 0,
            last_due: None,
            last_duration_ns: 33_333_300,
        }
    }
    fn seek_cycle(&mut self, now: u64, explicit: bool) -> Result<(), String> {
        self.reader.seek_zero()?;
        self.pending = None;
        self.pending_observed = false;
        self.force_discontinuity = true;
        let boundary = if explicit {
            now
        } else {
            self.last_due
                .and_then(|deadline| deadline.checked_add(self.last_duration_ns))
                .ok_or("Video loop deadline overflow")?
                .max(now)
        };
        self.pacer.reset(Some(boundary));
        self.ended = false;
        self.poster = explicit;
        self.status_reads = 0;
        self.publish_busy = 0;
        self.empty_loops = self
            .empty_loops
            .checked_add(1)
            .ok_or("Video loop counter overflow")?;
        if self.empty_loops > 2 {
            return Err("Video ended repeatedly without a decoded frame".into());
        }
        Ok(())
    }
    fn step(&mut self, control: VideoControl, now: u64) -> Result<Step, String> {
        if let Some(generation) = self.restart_generation
            && generation != control.restart_generation
        {
            self.seek_cycle(now, true)?;
        }
        self.restart_generation = Some(control.restart_generation);
        self.pacer.pause(control.paused, now)?;
        if control.paused && !self.poster {
            return Ok(Step::wait(10_000_000));
        }
        if self.ended {
            if control.looping {
                self.seek_cycle(now, false)?;
            } else {
                return Ok(Step::wait(10_000_000));
            }
        }
        if self.pending.is_none() {
            match self.reader.read()? {
                ReadEvent::Frame {
                    frame,
                    nominal_duration_100ns,
                } => {
                    let mut metadata = frame.metadata();
                    if metadata.timestamp_100ns < 0 {
                        self.status_reads = self
                            .status_reads
                            .checked_add(1)
                            .ok_or("Video preroll counter overflow")?;
                        if self.status_reads > STATUS_LIMIT {
                            return Err("Video preroll exceeded its bounded read limit".into());
                        }
                        return Ok(Step::wait(1_000_000));
                    }
                    let duration = metadata
                        .duration_100ns
                        .or(nominal_duration_100ns)
                        .unwrap_or(333_333);
                    self.last_duration_ns = u64::try_from(
                        i128::from(duration)
                            .checked_mul(100)
                            .ok_or("Video sample duration overflow")?,
                    )
                    .ok()
                    .filter(|duration| *duration > 0)
                    .ok_or("Video sample duration is invalid")?;
                    self.status_reads = 0;
                    if self.force_discontinuity {
                        // Decoder preroll can consume its seek marker before
                        // the first nonnegative frame. Preserve the logical
                        // source discontinuity through that preroll shedding.
                        metadata.discontinuity = true;
                        self.pending = Some(
                            VideoFrame::new(metadata, frame.shared_pixels())
                                .map_err(|error| error.to_string())?,
                        );
                    } else {
                        self.pending = Some(frame);
                    }
                    self.pending_observed = false;
                    // A synchronous read may block. Re-enter the owner loop
                    // before publication to recheck cancellation, controls and
                    // the actual monotonic time after that read has finished.
                    return Ok(Step::wait(100_000));
                }
                ReadEvent::NoSample | ReadEvent::StreamTick => {
                    self.status_reads = self
                        .status_reads
                        .checked_add(1)
                        .ok_or("Video status read counter overflow")?;
                    if self.status_reads > STATUS_LIMIT {
                        return Err("Video decoder returned too many status-only reads".into());
                    }
                    set_health_bounded(&self.feed,VideoSourceHealth::Degraded { reason:"Decoder reported a sample gap; holding the last valid frame. MP4 audio is not decoded.".into() })?;
                    return Ok(Step::wait(10_000_000));
                }
                ReadEvent::End => {
                    if self.last_due.is_none() {
                        return Err(
                            "Video reached end of stream without a valid nonnegative frame".into(),
                        );
                    }
                    self.ended = true;
                    if control.looping {
                        self.seek_cycle(now, false)?;
                    } else {
                        set_health_bounded(&self.feed, VideoSourceHealth::Stopped)?;
                    }
                    return Ok(Step::changed());
                }
            }
        }
        let Some(frame) = self.pending.as_ref() else {
            return Ok(Step::wait(1_000_000));
        };
        if !self.pending_observed {
            let metadata = frame.metadata();
            self.pacer
                .observe(metadata.timestamp_100ns, metadata.discontinuity, now)?;
            self.pending_observed = true;
        }
        let due = self.pacer.deadline(frame.metadata().timestamp_100ns)?;
        if now < due && !(self.poster && control.paused) {
            return Ok(Step::wait(due - now));
        }
        match self.feed.publish(frame.clone()) {
            Ok(()) => {
                self.last_due = Some(due);
                self.pending = None;
                self.poster = false;
                self.force_discontinuity = false;
                self.empty_loops = 0;
                self.publish_busy = 0;
                if now.saturating_sub(due) > 250_000_000 {
                    set_health_bounded(&self.feed,VideoSourceHealth::Degraded { reason:"Video decoding is late; timestamps are preserved. MP4 audio is not decoded.".into() })?;
                }
                Ok(Step::changed())
            }
            Err(VideoFeedError::Busy) => {
                self.publish_busy = self
                    .publish_busy
                    .checked_add(1)
                    .ok_or("Video publication retry counter overflow")?;
                if self.publish_busy > 8 {
                    Err("Video frame feed remained busy after a bounded handoff".into())
                } else {
                    Ok(Step::wait(2_000_000))
                }
            }
            Err(error) => Err(error.to_string()),
        }
    }
}

pub(crate) fn discover_cameras() -> Result<Vec<(String, String)>, String> {
    #[cfg(windows)]
    {
        rustmix_platform::media_foundation::camera_devices()
            .map(|items| items.into_iter().map(|d| (d.id, d.name)).collect())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        Err("Camera capture requires Windows".into())
    }
}

#[cfg(windows)]
fn platform_mode(
    mode: &rustmix_engine::video_format::CameraFormat,
) -> rustmix_platform::media_foundation::CameraMode {
    rustmix_platform::media_foundation::CameraMode {
        width: mode.width,
        height: mode.height,
        numerator: mode.rate.numerator,
        denominator: mode.rate.denominator,
        interlace: match mode.scan {
            rustmix_engine::ScanMode::Progressive => 2,
            rustmix_engine::ScanMode::InterlacedUpperFirst => 3,
            rustmix_engine::ScanMode::InterlacedLowerFirst => 4,
        },
        subtype: mode.subtype.clone(),
    }
}
pub(crate) fn discover_camera_modes(
    device: &str,
) -> Result<Vec<rustmix_engine::video_format::CameraFormat>, String> {
    #[cfg(windows)]
    {
        rustmix_platform::media_foundation::VideoFile::camera_modes(device)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|mode| {
                let format = rustmix_engine::video_format::CameraFormat {
                    width: mode.width,
                    height: mode.height,
                    rate: rustmix_engine::Rate::new(mode.numerator, mode.denominator)
                        .map_err(|e| e.to_string())?,
                    scan: match mode.interlace {
                        2 => rustmix_engine::ScanMode::Progressive,
                        3 => rustmix_engine::ScanMode::InterlacedUpperFirst,
                        4 => rustmix_engine::ScanMode::InterlacedLowerFirst,
                        _ => return Err("Unsupported camera scan".into()),
                    },
                    subtype: mode.subtype,
                };
                format.validate().map_err(|e| e.to_string())?;
                Ok(format)
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        let _ = device;
        Err("Camera capture requires Windows".into())
    }
}

pub(crate) fn probe_source(source: &VideoSource) -> Result<(), String> {
    source.validate().map_err(|e| e.to_string())?;
    #[cfg(windows)]
    {
        match source {
            VideoSource::Photos { paths, .. } => {
                for path in paths {
                    source_workers::photo(std::path::Path::new(path))?;
                }
                Ok(())
            }
            VideoSource::Browser { url, width, height } => {
                source_workers::probe_browser(url, *width, *height)
            }
            VideoSource::Camera { device_id, format } => {
                use rustmix_platform::media_foundation::{VideoFile, VideoRead};
                let mode = format.as_ref().map(platform_mode);
                let mut camera = VideoFile::open_camera_mode(device_id, mode.as_ref())
                    .map_err(|e| e.to_string())?;
                for _ in 0..120 {
                    match camera.read_frame().map_err(|e| e.to_string())? {
                        VideoRead::Frame(_) => return Ok(()),
                        VideoRead::EndOfStream => {
                            return Err("Camera ended before its first frame".into());
                        }
                        _ => thread::sleep(Duration::from_millis(10)),
                    }
                }
                Err("Camera did not deliver a frame".into())
            }
            _ => Err("Use the Video page to check MP4".into()),
        }
    }
    #[cfg(not(windows))]
    {
        Err("These source workers require Windows".into())
    }
}

#[cfg(windows)]
#[path = "source_workers.rs"]
mod source_workers;

#[cfg(windows)]
mod native {
    use super::*;
    use rustmix_platform::media_foundation::{VideoFile, VideoRead};
    struct NativeReader {
        file: VideoFile,
        input_id: InputId,
        generation: u64,
        camera: bool,
    }
    impl FrameReader for NativeReader {
        fn read(&mut self) -> Result<ReadEvent, String> {
            match self.file.read_frame().map_err(|error| error.to_string())? {
                VideoRead::Frame(decoded) => {
                    let nominal_duration_100ns = if decoded.format.frame_rate_numerator > 0
                        && decoded.format.frame_rate_denominator > 0
                    {
                        i128::from(decoded.format.frame_rate_denominator)
                            .checked_mul(10_000_000)
                            .and_then(|ticks| {
                                ticks.checked_div(i128::from(decoded.format.frame_rate_numerator))
                            })
                            .and_then(|ticks| i64::try_from(ticks).ok())
                            .filter(|ticks| *ticks > 0)
                    } else {
                        None
                    };
                    let metadata = VideoFrameMetadata {
                        input_id: self.input_id,
                        generation: self.generation,
                        sequence: decoded.sequence,
                        width: decoded.width,
                        height: decoded.height,
                        timestamp_100ns: decoded.timestamp_100ns,
                        duration_100ns: decoded.duration_100ns,
                        discontinuity: decoded.discontinuity || decoded.media_type_changed,
                    };
                    let frame = VideoFrame::new(metadata, Arc::new(decoded.pixels))
                        .map_err(|error| error.to_string())?;
                    Ok(ReadEvent::Frame {
                        frame,
                        nominal_duration_100ns,
                    })
                }
                VideoRead::NoSample { .. } => Ok(ReadEvent::NoSample),
                VideoRead::StreamTick { .. } => Ok(ReadEvent::StreamTick),
                VideoRead::EndOfStream => Ok(ReadEvent::End),
            }
        }
        fn seek_zero(&mut self) -> Result<(), String> {
            if self.camera {
                Ok(())
            } else {
                self.file.seek_100ns(0).map_err(|error| error.to_string())
            }
        }
    }
    pub(super) fn run(
        shared: &Shared,
        input_id: InputId,
        generation: u64,
        path: &str,
        format: Option<&rustmix_engine::video_format::CameraFormat>,
        camera: bool,
    ) -> Result<(), String> {
        if shared.stop.load(Ordering::Acquire) {
            return Ok(());
        }
        set_health_bounded(&shared.feed, VideoSourceHealth::Probing)?;
        let file = if camera {
            let mode = format.map(platform_mode);
            VideoFile::open_camera_mode(path, mode.as_ref())
        } else {
            VideoFile::open(std::path::Path::new(path))
        }
        .map_err(|error| error.to_string())?;
        if shared.stop.load(Ordering::Acquire) {
            return Ok(());
        }
        set_health_bounded(&shared.feed, VideoSourceHealth::Starting)?;
        let reader = NativeReader {
            file,
            input_id,
            generation,
            camera,
        };
        let mut playback = Playback::new(reader, Arc::clone(&shared.feed));
        let origin = Instant::now();
        while !shared.stop.load(Ordering::Acquire) {
            let control = *shared
                .control
                .lock()
                .map_err(|_| "Video control handoff failed")?;
            let now = u64::try_from(origin.elapsed().as_nanos())
                .map_err(|_| "Video monotonic session time overflow")?;
            let step = playback.step(control, now)?;
            if step.changed {
                shared.repaint.request_repaint();
            }
            thread::sleep(step.wait);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]
    use super::*;
    use std::sync::atomic::AtomicUsize;

    fn frame(sequence: u64, pts: i64, discontinuity: bool) -> VideoFrame {
        VideoFrame::new(
            VideoFrameMetadata {
                input_id: InputId(1),
                generation: 9,
                sequence,
                width: 1,
                height: 1,
                timestamp_100ns: pts,
                duration_100ns: Some(1_000_000),
                discontinuity,
            },
            Arc::new(vec![[sequence as u8, 0, 0, 255]]),
        )
        .unwrap()
    }
    struct FakeReader {
        timestamps: Vec<i64>,
        position: usize,
        sequence: u64,
        discontinuity: bool,
        seeks: Arc<AtomicUsize>,
    }
    impl FrameReader for FakeReader {
        fn read(&mut self) -> Result<ReadEvent, String> {
            let Some(pts) = self.timestamps.get(self.position).copied() else {
                return Ok(ReadEvent::End);
            };
            let frame = frame(self.sequence, pts, self.discontinuity);
            self.position += 1;
            self.sequence += 1;
            self.discontinuity = false;
            Ok(ReadEvent::Frame {
                frame,
                nominal_duration_100ns: Some(1_000_000),
            })
        }
        fn seek_zero(&mut self) -> Result<(), String> {
            self.position = 0;
            self.discontinuity = true;
            self.seeks.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }
    fn playback(timestamps: Vec<i64>) -> (Playback<FakeReader>, Arc<VideoFeed>, Arc<AtomicUsize>) {
        let feed = Arc::new(VideoFeed::new(InputId(1), 9).unwrap());
        let seeks = Arc::new(AtomicUsize::new(0));
        let reader = FakeReader {
            timestamps,
            position: 0,
            sequence: 0,
            discontinuity: true,
            seeks: Arc::clone(&seeks),
        };
        (Playback::new(reader, Arc::clone(&feed)), feed, seeks)
    }
    #[test]
    fn exact_pts_pacing_pause_resume_and_discontinuity_never_accumulate_float_time() {
        let mut pacer = Pacer::default();
        pacer.observe(1_000_000, false, 0).unwrap();
        pacer.observe(1_333_667, false, 1).unwrap();
        assert_eq!(pacer.deadline(1_333_667).unwrap(), 33_366_700);
        pacer.pause(true, 10_000_000).unwrap();
        pacer.pause(false, 110_000_000).unwrap();
        assert_eq!(pacer.deadline(1_333_667).unwrap(), 133_366_700);
        assert!(pacer.observe(1_000_000, false, 120_000_000).is_err());
        pacer.observe(0, true, 200_000_000).unwrap();
        assert_eq!(pacer.deadline(0).unwrap(), 200_000_000);
        assert!(pacer.deadline(i64::MAX).is_err());
    }
    #[test]
    fn pending_frame_stays_private_until_due_and_pause_preserves_its_deadline() {
        let (mut playback, feed, _) = playback(vec![0, 1_000_000]);
        let mut control = VideoControl::default();
        playback.step(control, 0).unwrap();
        playback.step(control, 0).unwrap();
        playback.step(control, 1_000_000).unwrap();
        assert_eq!(
            feed.snapshot().unwrap().frame.unwrap().metadata().sequence,
            0
        );
        control.paused = true;
        playback.step(control, 20_000_000).unwrap();
        control.paused = false;
        playback.step(control, 120_000_000).unwrap();
        assert_eq!(
            feed.snapshot().unwrap().frame.unwrap().metadata().sequence,
            0
        );
        playback.step(control, 200_000_000).unwrap();
        assert_eq!(
            feed.snapshot().unwrap().frame.unwrap().metadata().sequence,
            1
        );
    }
    #[test]
    fn loop_holds_final_sample_duration_and_restart_routes_a_real_seek() {
        let (mut playback, feed, seeks) = playback(vec![0, 1_000_000]);
        let mut control = VideoControl::default();
        playback.step(control, 0).unwrap();
        playback.step(control, 0).unwrap();
        playback.step(control, 1_000_000).unwrap();
        playback.step(control, 100_000_000).unwrap();
        playback.step(control, 101_000_000).unwrap(); // EOS seeks for next cycle
        playback.step(control, 102_000_000).unwrap(); // next cycle frame remains pending
        assert_eq!(seeks.load(Ordering::Relaxed), 1);
        assert_eq!(
            feed.snapshot().unwrap().frame.unwrap().metadata().sequence,
            1
        );
        playback.step(control, 200_000_000).unwrap();
        assert_eq!(
            feed.snapshot()
                .unwrap()
                .frame
                .unwrap()
                .metadata()
                .timestamp_100ns,
            0
        );
        control.restart_generation = 1;
        playback.step(control, 210_000_000).unwrap();
        playback.step(control, 210_000_000).unwrap();
        assert_eq!(seeks.load(Ordering::Relaxed), 2);
        assert_eq!(
            feed.snapshot()
                .unwrap()
                .frame
                .unwrap()
                .metadata()
                .timestamp_100ns,
            0
        );
    }
    #[test]
    fn paused_start_decodes_one_poster_and_preroll_is_bounded_before_publication() {
        let (mut playback, feed, _) = playback(vec![-10_000, 0, 1_000_000]);
        let control = VideoControl {
            paused: true,
            ..VideoControl::default()
        };
        playback.step(control, 0).unwrap();
        assert!(feed.snapshot().unwrap().frame.is_none());
        playback.step(control, 1_000_000).unwrap();
        playback.step(control, 1_000_000).unwrap();
        let sequence = feed.snapshot().unwrap().frame.unwrap().metadata().sequence;
        playback.step(control, 100_000_000).unwrap();
        assert_eq!(
            feed.snapshot().unwrap().frame.unwrap().metadata().sequence,
            sequence
        );
        assert_eq!(playback.reader.position, 2);
    }
    #[test]
    fn status_only_reads_are_bounded_and_keep_the_last_valid_frame() {
        struct StatusReader {
            calls: u32,
        }
        impl FrameReader for StatusReader {
            fn read(&mut self) -> Result<ReadEvent, String> {
                self.calls += 1;
                Ok(if self.calls.is_multiple_of(2) {
                    ReadEvent::NoSample
                } else {
                    ReadEvent::StreamTick
                })
            }
            fn seek_zero(&mut self) -> Result<(), String> {
                Ok(())
            }
        }
        let feed = Arc::new(VideoFeed::new(InputId(1), 9).unwrap());
        feed.publish(frame(0, 0, true)).unwrap();
        let before = feed.snapshot().unwrap().frame.unwrap();
        let mut playback = Playback::new(StatusReader { calls: 0 }, Arc::clone(&feed));
        let step = playback.step(VideoControl::default(), 0).unwrap();
        assert_eq!(step.wait, Duration::from_millis(10));
        assert!(!step.changed);
        for index in 1..STATUS_LIMIT {
            playback
                .step(VideoControl::default(), u64::from(index) * 10_000_000)
                .unwrap();
        }
        assert!(
            playback
                .step(
                    VideoControl::default(),
                    u64::from(STATUS_LIMIT) * 10_000_000
                )
                .is_err()
        );
        let after = feed.snapshot().unwrap();
        assert!(Arc::ptr_eq(&before, &after.frame.unwrap()));
        assert!(matches!(after.health, VideoSourceHealth::Degraded { .. }));
    }
    #[test]
    fn preroll_after_seek_keeps_discontinuity_until_a_publishable_frame() {
        let (mut playback, feed, _) = playback(vec![-10_000, 0, 1_000_000]);
        let mut control = VideoControl::default();
        for _ in 0..3 {
            playback.step(control, 0).unwrap();
        }
        playback.step(control, 1_000_000).unwrap();
        playback.step(control, 100_000_000).unwrap();
        assert_eq!(
            feed.snapshot()
                .unwrap()
                .frame
                .unwrap()
                .metadata()
                .timestamp_100ns,
            1_000_000
        );
        control.restart_generation = 1;
        for _ in 0..3 {
            playback.step(control, 110_000_000).unwrap();
        }
        let frame = feed.snapshot().unwrap().frame.unwrap();
        assert_eq!(frame.metadata().timestamp_100ns, 0);
        assert!(frame.metadata().discontinuity);
    }
    #[test]
    fn wrong_generation_is_rejected_and_error_text_is_safe_for_feed_health() {
        let feed = Arc::new(VideoFeed::new(InputId(1), 10).unwrap());
        let reader = FakeReader {
            timestamps: vec![0],
            position: 0,
            sequence: 0,
            discontinuity: true,
            seeks: Arc::new(AtomicUsize::new(0)),
        };
        let mut playback = Playback::new(reader, Arc::clone(&feed));
        playback.step(VideoControl::default(), 0).unwrap();
        assert!(playback.step(VideoControl::default(), 0).is_err());
        assert!(feed.snapshot().unwrap().frame.is_none());
        let reason = safe_reason(&format!("\n\r{}", "é😀".repeat(100)));
        assert!(reason.len() <= 256 && !reason.chars().any(char::is_control));
        assert!(!safe_reason("\n\r").is_empty());
    }
}
