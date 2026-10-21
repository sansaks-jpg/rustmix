//! Persisted source descriptors and validated CPU reference processing controls.
use crate::{InputId, Result, invalid};
use serde::{Deserialize, Serialize};

/// Persisted audio-only source identity. Opening files and devices belongs to
/// the audio worker; the engine validates descriptors without performing I/O.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSource {
    WavFile {
        path: String,
    },
    /// `None` requests the system default microphone. A selected endpoint keeps
    /// its stable device ID rather than a possibly duplicated display name.
    Microphone {
        device_id: Option<String>,
    },
}

impl AudioSource {
    pub fn label(&self) -> &'static str {
        match self {
            Self::WavFile { .. } => "WAV file",
            Self::Microphone { .. } => "Microphone",
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::WavFile { path } => {
                if !valid_audio_identity(path) {
                    return Err(invalid(
                        "Audio file source must reference a local WAV path of at most 4096 bytes",
                    ));
                }
                // URLs, UNC shares and alternate data streams are not local
                // WAV descriptors. A normal Windows drive prefix is allowed.
                let mut colons = path.match_indices(':');
                let colon_is_drive = match colons.next() {
                    None => true,
                    Some((1, _)) => path.as_bytes().first().is_some_and(u8::is_ascii_alphabetic),
                    Some(_) => false,
                } && colons.next().is_none();
                if path.starts_with("//")
                    || path.starts_with(r"\\")
                    || !colon_is_drive
                    || !std::path::Path::new(path)
                        .extension()
                        .and_then(|extension| extension.to_str())
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"))
                {
                    return Err(invalid(
                        "Audio file source must reference a local WAV path of at most 4096 bytes",
                    ));
                }
            }
            Self::Microphone {
                device_id: Some(device_id),
            } if !valid_audio_identity(device_id) => {
                return Err(invalid(
                    "Microphone device ID must be non-empty text of at most 4096 bytes",
                ));
            }
            Self::Microphone { .. } => {}
        }
        Ok(())
    }
}

fn valid_audio_identity(value: &str) -> bool {
    value.len() <= 4096 && !value.trim().is_empty() && !value.chars().any(char::is_control)
}

/// Persisted video identity only; platform workers open and decode the file.
/// A descriptor never establishes that its container's codec is supported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoSource {
    File {
        path: String,
    },
    Photos {
        paths: Vec<String>,
        interval_ms: u32,
    },
    Browser {
        url: String,
        width: u32,
        height: u32,
    },
    Camera {
        device_id: String,
    },
}

impl VideoSource {
    pub fn label(&self) -> &'static str {
        match self {
            Self::File { .. } => "Video file",
            Self::Photos { .. } => "Photos",
            Self::Browser { .. } => "Web Browser",
            Self::Camera { .. } => "Camera",
        }
    }

    pub fn validate(&self) -> Result<()> {
        let path = match self {
            Self::Camera { device_id } => {
                return if valid_audio_identity(device_id) {
                    Ok(())
                } else {
                    Err(invalid("Choose a valid camera device ID"))
                };
            }
            Self::Browser { url, width, height } => {
                let authority = url
                    .split_once("://")
                    .map(|(_, rest)| rest.split('/').next().unwrap_or(""));
                return if valid_audio_identity(url)
                    && !url.chars().any(char::is_whitespace)
                    && (url.starts_with("https://") || url.starts_with("http://"))
                    && authority.is_some_and(|a| !a.is_empty() && !a.contains('@'))
                    && (64..=1920).contains(width)
                    && (64..=1080).contains(height)
                {
                    Ok(())
                } else {
                    Err(invalid(
                        "Browser needs an HTTP(S) URL without credentials and a 64–1920 by 64–1080 viewport",
                    ))
                };
            }
            Self::Photos { paths, interval_ms } => {
                return if !paths.is_empty()
                    && paths.len() <= 256
                    && (500..=3_600_000).contains(interval_ms)
                    && paths.iter().all(|p| {
                        valid_audio_identity(p)
                            && !p.starts_with("//")
                            && !p.starts_with(r"\\")
                            && std::path::Path::new(p)
                                .extension()
                                .and_then(|e| e.to_str())
                                .is_some_and(|e| {
                                    ["png", "jpg", "jpeg", "bmp"]
                                        .iter()
                                        .any(|x| e.eq_ignore_ascii_case(x))
                                })
                    }) {
                    Ok(())
                } else {
                    Err(invalid(
                        "Photos needs 1–256 local PNG/JPEG/BMP files and an interval of 0.5–3600 seconds",
                    ))
                };
            }
            Self::File { path } => path,
        };
        if !valid_audio_identity(path) {
            return Err(invalid(
                "Video file source must reference a local MP4 path of at most 4096 bytes",
            ));
        }
        let mut colons = path.match_indices(':');
        let colon_is_drive = match colons.next() {
            None => true,
            Some((1, _)) => path.as_bytes().first().is_some_and(u8::is_ascii_alphabetic),
            Some(_) => false,
        } && colons.next().is_none();
        if path.starts_with("//")
            || path.starts_with(r"\\")
            || !colon_is_drive
            || !std::path::Path::new(path)
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("mp4"))
        {
            return Err(invalid(
                "Video file source must reference a local MP4 path of at most 4096 bytes",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFit {
    #[default]
    Contain,
    Cover,
    Stretch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub mirror: bool,
    pub sharpen: bool,
    pub lift: f32,
    pub gain: f32,
    pub saturation: f32,
    pub chroma_key: bool,
    pub chroma_tolerance: f32,
    pub pip_source: Option<InputId>,
    /// External PNG reference. Decoded pixels are a UI cache, never project JSON.
    pub media_path: Option<String>,
    /// Audio-only source; a consumer must display its explicit slate rather
    /// than presenting the input's procedural kind as decoded video.
    pub audio_source: Option<AudioSource>,
    /// External video descriptor. Timestamped decoded frames live in the
    /// bounded runtime feed, never inside the saved project.
    pub video_source: Option<VideoSource>,
    pub image_fit: ImageFit,
}
impl Default for InputConfig {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            mirror: false,
            sharpen: false,
            lift: 0.0,
            gain: 1.0,
            saturation: 1.0,
            chroma_key: false,
            chroma_tolerance: 0.28,
            pip_source: None,
            media_path: None,
            audio_source: None,
            video_source: None,
            image_fit: ImageFit::Contain,
        }
    }
}
impl InputConfig {
    pub(crate) fn validate(&self) -> Result<()> {
        if (self.media_path.is_some()
            && (self.audio_source.is_some() || self.video_source.is_some()))
            || (self.audio_source.is_some() && self.video_source.is_some())
        {
            return Err(invalid(
                "Assign only one PNG, audio or video source to an input",
            ));
        }
        if let Some(source) = &self.audio_source {
            source.validate()?;
        }
        if let Some(source) = &self.video_source {
            source.validate()?;
        }
        for (value, low, high) in [
            (self.zoom, 0.25, 4.0),
            (self.pan_x, -1.0, 1.0),
            (self.pan_y, -1.0, 1.0),
            (self.lift, -1.0, 1.0),
            (self.gain, 0.0, 2.0),
            (self.saturation, 0.0, 2.0),
            (self.chroma_tolerance, 0.05, 0.85),
        ] {
            if !value.is_finite() || !(low..=high).contains(&value) {
                return Err(invalid(
                    "Input processing value is outside its supported range",
                ));
            }
        }
        if let Some(path) = &self.media_path
            && (path.trim().is_empty()
                || path.len() > 4096
                || path.chars().any(char::is_control)
                || !std::path::Path::new(path)
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.eq_ignore_ascii_case("png")))
        {
            return Err(invalid(
                "Image source must reference a PNG path of at most 4096 bytes",
            ));
        }
        Ok(())
    }
}
