//! Persisted source descriptors and validated CPU reference processing controls.
use crate::{InputId, Result, invalid};
use serde::{Deserialize, Serialize};

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
            image_fit: ImageFit::Contain,
        }
    }
}
impl InputConfig {
    pub(crate) fn validate(&self) -> Result<()> {
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
