//! Safe timestamped video handoff shared by independent Program/UI consumers.
//! One replaceable frame is retained per source; snapshots never remove it.
//! Only short nonblocking try-locks protect metadata/Arc publication. No render,
//! decode or I/O work runs under the feed lock, and poisoned state is rejected.

use crate::InputId;
use std::{
    fmt,
    sync::{Arc, RwLock, TryLockError},
};

pub const MAX_VIDEO_SOURCE_WIDTH: u32 = 4096;
pub const MAX_VIDEO_SOURCE_HEIGHT: u32 = 2160;
const MAX_HEALTH_REASON_BYTES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoFeedError {
    Invalid(&'static str),
    Busy,
    Poisoned,
    WrongInput { expected: InputId, actual: InputId },
    WrongGeneration { expected: u64, actual: u64 },
    StaleSequence { previous: u64, next: u64 },
    TimestampRegression,
}

impl fmt::Display for VideoFeedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => formatter.write_str(message),
            Self::Busy => {
                formatter.write_str("Video source state is busy; retain the last known frame")
            }
            Self::Poisoned => formatter
                .write_str("Video source state failed; isolate this source and retain Program"),
            Self::WrongInput { .. } => {
                formatter.write_str("Video frame belongs to a different input")
            }
            Self::WrongGeneration { .. } => {
                formatter.write_str("Video frame belongs to an obsolete source generation")
            }
            Self::StaleSequence { .. } => formatter
                .write_str("Video frame sequence is not newer than the last accepted frame"),
            Self::TimestampRegression => formatter
                .write_str("Video timestamps moved backwards without a discontinuity marker"),
        }
    }
}

impl std::error::Error for VideoFeedError {}

type Result<T> = std::result::Result<T, VideoFeedError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoFrameMetadata {
    pub input_id: InputId,
    pub generation: u64,
    pub sequence: u64,
    pub width: u32,
    pub height: u32,
    /// Signed source PTS, including valid negative decoder preroll.
    pub timestamp_100ns: i64,
    pub duration_100ns: Option<i64>,
    pub discontinuity: bool,
}

impl VideoFrameMetadata {
    fn pixel_count(self) -> Result<usize> {
        if self.input_id.0 == 0 {
            return Err(VideoFeedError::Invalid("Video input ID must be nonzero"));
        }
        if self.width == 0
            || self.height == 0
            || self.width > MAX_VIDEO_SOURCE_WIDTH
            || self.height > MAX_VIDEO_SOURCE_HEIGHT
        {
            return Err(VideoFeedError::Invalid(
                "Video frame raster must be 1–4096 by 1–2160 pixels",
            ));
        }
        if let Some(duration) = self.duration_100ns
            && (duration <= 0 || self.timestamp_100ns.checked_add(duration).is_none())
        {
            return Err(VideoFeedError::Invalid(
                "Video frame duration or end timestamp is invalid",
            ));
        }
        usize::try_from(self.width)
            .ok()
            .zip(usize::try_from(self.height).ok())
            .and_then(|(width, height)| width.checked_mul(height))
            .filter(|count| count.checked_mul(4).is_some())
            .ok_or(VideoFeedError::Invalid(
                "Video frame allocation size overflow",
            ))
    }
}

/// Immutable validated frame. Metadata cannot change after construction and no
/// mutable pixel view is exposed; Arc clones share an owned RGBA allocation.
#[derive(Debug, Clone)]
pub struct VideoFrame {
    metadata: VideoFrameMetadata,
    pixels: Arc<Vec<[u8; 4]>>,
}

impl VideoFrame {
    pub fn new(metadata: VideoFrameMetadata, pixels: Arc<Vec<[u8; 4]>>) -> Result<Self> {
        if pixels.len() != metadata.pixel_count()? {
            return Err(VideoFeedError::Invalid(
                "Video RGBA pixel count does not match its raster",
            ));
        }
        let maximum = u64::from(MAX_VIDEO_SOURCE_WIDTH) * u64::from(MAX_VIDEO_SOURCE_HEIGHT);
        if u64::try_from(pixels.capacity())
            .ok()
            .is_none_or(|capacity| capacity > maximum)
        {
            return Err(VideoFeedError::Invalid(
                "Video pixel allocation exceeds the source raster bound",
            ));
        }
        Ok(Self { metadata, pixels })
    }

    pub fn metadata(&self) -> VideoFrameMetadata {
        self.metadata
    }

    pub fn pixels(&self) -> &[[u8; 4]] {
        self.pixels.as_slice()
    }

    pub fn shared_pixels(&self) -> Arc<Vec<[u8; 4]>> {
        Arc::clone(&self.pixels)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoSourceHealth {
    Created,
    Probing,
    Starting,
    Running,
    Degraded { reason: String },
    Offline { reason: String },
    Stopped,
    Failed { reason: String },
}

impl VideoSourceHealth {
    fn validate(&self) -> Result<()> {
        let reason = match self {
            Self::Degraded { reason } | Self::Offline { reason } | Self::Failed { reason } => {
                reason
            }
            _ => return Ok(()),
        };
        if reason.len() > MAX_HEALTH_REASON_BYTES
            || reason.trim().is_empty()
            || reason.chars().any(char::is_control)
        {
            return Err(VideoFeedError::Invalid(
                "Video health reason must be non-empty text of at most 256 bytes",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VideoFeedSnapshot {
    pub input_id: InputId,
    pub generation: u64,
    pub revision: u64,
    pub health: VideoSourceHealth,
    pub frame: Option<Arc<VideoFrame>>,
}

/// A single-generation feed. Reopening/relinking/reusing an input constructs a
/// new feed; old workers cannot publish into it using their previous identity.
#[derive(Debug)]
pub struct VideoFeed {
    input_id: InputId,
    generation: u64,
    state: RwLock<VideoFeedSnapshot>,
}

impl VideoFeed {
    pub fn new(input_id: InputId, generation: u64) -> Result<Self> {
        if input_id.0 == 0 {
            return Err(VideoFeedError::Invalid("Video input ID must be nonzero"));
        }
        Ok(Self {
            input_id,
            generation,
            state: RwLock::new(VideoFeedSnapshot {
                input_id,
                generation,
                revision: 0,
                health: VideoSourceHealth::Created,
                frame: None,
            }),
        })
    }

    /// Snapshotting never steals a frame from another consumer. A contended or
    /// poisoned lock returns explicit state; callers can hold their previous Arc.
    pub fn snapshot(&self) -> Result<VideoFeedSnapshot> {
        match self.state.try_read() {
            Ok(state) => Ok(state.clone()),
            Err(TryLockError::WouldBlock) => Err(VideoFeedError::Busy),
            Err(TryLockError::Poisoned(_)) => Err(VideoFeedError::Poisoned),
        }
    }

    /// Publish one complete frame. No publication waits for a consumer, and
    /// only the latest accepted frame remains in the feed. Keep a rejected Busy
    /// publication on the worker for a bounded retry or count the dropped frame.
    pub fn publish(&self, frame: VideoFrame) -> Result<()> {
        let metadata = frame.metadata;
        if metadata.input_id != self.input_id {
            return Err(VideoFeedError::WrongInput {
                expected: self.input_id,
                actual: metadata.input_id,
            });
        }
        if metadata.generation != self.generation {
            return Err(VideoFeedError::WrongGeneration {
                expected: self.generation,
                actual: metadata.generation,
            });
        }
        // Any allocation for the shared frame happens before acquiring state.
        let frame = Arc::new(frame);
        let mut state = match self.state.try_write() {
            Ok(state) => state,
            Err(TryLockError::WouldBlock) => return Err(VideoFeedError::Busy),
            Err(TryLockError::Poisoned(_)) => return Err(VideoFeedError::Poisoned),
        };
        if let Some(previous) = &state.frame {
            let previous = previous.metadata;
            if metadata.sequence <= previous.sequence {
                return Err(VideoFeedError::StaleSequence {
                    previous: previous.sequence,
                    next: metadata.sequence,
                });
            }
            if metadata.timestamp_100ns < previous.timestamp_100ns && !metadata.discontinuity {
                return Err(VideoFeedError::TimestampRegression);
            }
        }
        let revision = state
            .revision
            .checked_add(1)
            .ok_or(VideoFeedError::Invalid(
                "Video source state revision exhausted",
            ))?;
        let retired_frame = state.frame.replace(frame);
        let retired_health = std::mem::replace(&mut state.health, VideoSourceHealth::Running);
        state.revision = revision;
        drop(state);
        // Dropping the feed's last Arc can free an entire raster; retire it only
        // after releasing state so consumers never wait for buffer destruction.
        drop(retired_frame);
        drop(retired_health);
        Ok(())
    }

    /// Health changes retain the last frame (freeze policy); the operator sees
    /// the warning independently. Running requires an actual accepted frame.
    pub fn set_health(&self, health: VideoSourceHealth) -> Result<()> {
        health.validate()?;
        let mut state = match self.state.try_write() {
            Ok(state) => state,
            Err(TryLockError::WouldBlock) => return Err(VideoFeedError::Busy),
            Err(TryLockError::Poisoned(_)) => return Err(VideoFeedError::Poisoned),
        };
        if health == VideoSourceHealth::Running && state.frame.is_none() {
            return Err(VideoFeedError::Invalid(
                "Video source cannot run before its first decoded frame",
            ));
        }
        let revision = state
            .revision
            .checked_add(1)
            .ok_or(VideoFeedError::Invalid(
                "Video source state revision exhausted",
            ))?;
        let retired_health = std::mem::replace(&mut state.health, health);
        state.revision = revision;
        drop(state);
        drop(retired_health);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::panic)]
    use super::*;

    fn frame() -> VideoFrame {
        VideoFrame::new(
            VideoFrameMetadata {
                input_id: InputId(1),
                generation: 0,
                sequence: 0,
                width: 1,
                height: 1,
                timestamp_100ns: 0,
                duration_100ns: None,
                discontinuity: true,
            },
            Arc::new(vec![[0, 0, 0, 255]]),
        )
        .unwrap()
    }

    #[test]
    fn concurrent_access_never_blocks_or_recovers_poisoned_state() {
        let feed = VideoFeed::new(InputId(1), 0).unwrap();
        let guard = feed.state.write().unwrap();
        assert!(matches!(feed.snapshot(), Err(VideoFeedError::Busy)));
        assert_eq!(feed.publish(frame()), Err(VideoFeedError::Busy));
        assert_eq!(
            feed.set_health(VideoSourceHealth::Starting),
            Err(VideoFeedError::Busy)
        );
        drop(guard);
        assert!(feed.snapshot().is_ok());
        let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = feed.state.write().unwrap();
            panic!("Synthetic source-state failure");
        }));
        assert!(poisoned.is_err());
        assert!(matches!(feed.snapshot(), Err(VideoFeedError::Poisoned)));
        assert_eq!(feed.publish(frame()), Err(VideoFeedError::Poisoned));
        assert_eq!(
            feed.set_health(VideoSourceHealth::Stopped),
            Err(VideoFeedError::Poisoned)
        );
    }

    #[test]
    fn revision_exhaustion_preserves_the_accepted_frame_and_health() {
        let feed = VideoFeed::new(InputId(1), 0).unwrap();
        feed.state.write().unwrap().revision = u64::MAX;
        assert!(feed.publish(frame()).is_err());
        assert!(feed.set_health(VideoSourceHealth::Starting).is_err());
        let snapshot = feed.snapshot().unwrap();
        assert!(snapshot.frame.is_none());
        assert_eq!(snapshot.health, VideoSourceHealth::Created);
        assert_eq!(snapshot.revision, u64::MAX);
    }
}
