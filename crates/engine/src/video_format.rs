//! Validated project/capture standards. Rates are frames, never field counts.
use crate::{Engine, InputId, InputKind, Preferences, ProjectState, Rate, Result, invalid};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanMode {
    #[default]
    Progressive,
    InterlacedUpperFirst,
    InterlacedLowerFirst,
}
impl ScanMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Progressive => "p",
            Self::InterlacedUpperFirst => "i · top first",
            Self::InterlacedLowerFirst => "i · bottom first",
        }
    }
    pub fn rate_label(self, rate: Rate) -> String {
        if self == Self::Progressive {
            format!("{}p", rate.label())
        } else {
            format!(
                "{:.2}i · {} fps · {}",
                2.0 * f64::from(rate.numerator) / f64::from(rate.denominator.max(1)),
                rate.label(),
                self.label().trim_start_matches("i · ")
            )
        }
    }
}

pub const CANVAS_SIZES: [(u32, u32); 8] = [
    (1280, 720),
    (1920, 1080),
    (1080, 1920),
    (720, 1280),
    (2560, 1440),
    (1440, 2560),
    (3840, 2160),
    (2160, 3840),
];

/// A native camera mode including codec identity; never an assumed project rate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraFormat {
    pub width: u32,
    pub height: u32,
    pub rate: Rate,
    pub scan: ScanMode,
    pub subtype: String,
}
impl CameraFormat {
    pub fn validate(&self) -> Result<()> {
        if self.width == 0
            || self.height == 0
            || self.width > 4096
            || self.height > 2160
            || Rate::new(self.rate.numerator, self.rate.denominator)? != self.rate
            || self.subtype.len() != 36
            || !self.subtype.bytes().enumerate().all(|(i, b)| {
                if [8, 13, 18, 23].contains(&i) {
                    b == b'-'
                } else {
                    b.is_ascii_hexdigit()
                }
            })
        {
            return Err(invalid(
                "Camera format must be a valid native size, rational rate, scan and pixel format",
            ));
        }
        Ok(())
    }
    pub fn label(&self) -> String {
        format!(
            "{}×{} · {}",
            self.width,
            self.height,
            self.scan.rate_label(self.rate)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoStandard {
    pub width: u32,
    pub height: u32,
    pub rate: Rate,
    pub scan: ScanMode,
}
impl Default for VideoStandard {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            rate: Rate {
                numerator: 30000,
                denominator: 1001,
            },
            scan: ScanMode::Progressive,
        }
    }
}
impl VideoStandard {
    pub fn validate(self) -> Result<()> {
        if !CANVAS_SIZES.contains(&(self.width, self.height)) {
            return Err(invalid(
                "Choose a supported landscape or portrait canvas size",
            ));
        }
        if Rate::new(self.rate.numerator, self.rate.denominator)? != self.rate {
            return Err(invalid("Frame rate must be reduced"));
        }
        if self.scan != ScanMode::Progressive
            && u64::from(self.rate.numerator) > 60 * u64::from(self.rate.denominator)
        {
            return Err(invalid(
                "Interlaced timing is limited to 60 frames / 120 fields per second",
            ));
        }
        Ok(())
    }
    /// Keep both orientations at the same 230400-pixel reference processing budget.
    pub fn monitor_size(self) -> [u32; 2] {
        if self.height > self.width {
            [360, 640]
        } else {
            [640, 360]
        }
    }
    /// Full-resolution processing target: full HD in the project orientation.
    /// Larger canvases are fitted down; smaller ones are never upscaled.
    pub fn program_size(self) -> [u32; 2] {
        const MAX_WIDE: (u32, u32) = (1920, 1080);
        const MAX_TALL: (u32, u32) = (1080, 1920);
        let (max_w, max_h) = if self.height > self.width {
            MAX_TALL
        } else {
            MAX_WIDE
        };
        if self.width <= max_w && self.height <= max_h {
            return [self.width, self.height];
        }
        let wide = u64::from(self.width) * u64::from(max_h);
        let tall = u64::from(self.height) * u64::from(max_w);
        if wide >= tall {
            let height = u64::from(self.height) * u64::from(max_w) / u64::from(self.width);
            [max_w, u32::try_from(height.max(2)).unwrap_or(max_h)]
        } else {
            let width = u64::from(self.width) * u64::from(max_h) / u64::from(self.height);
            [u32::try_from(width.max(2)).unwrap_or(max_w), max_h]
        }
    }
}
pub fn rate_options() -> Vec<(Rate, ScanMode)> {
    let progressive = [
        (24000, 1001),
        (24, 1),
        (25, 1),
        (30000, 1001),
        (30, 1),
        (50, 1),
        (60000, 1001),
        (60, 1),
    ]
    .into_iter()
    .map(|(numerator, denominator)| {
        (
            Rate {
                numerator,
                denominator,
            },
            ScanMode::Progressive,
        )
    });
    let interlaced =
        [(25, 1), (30000, 1001), (30, 1)]
            .into_iter()
            .flat_map(|(numerator, denominator)| {
                [
                    ScanMode::InterlacedUpperFirst,
                    ScanMode::InterlacedLowerFirst,
                ]
                .map(|scan| {
                    (
                        Rate {
                            numerator,
                            denominator,
                        },
                        scan,
                    )
                })
            });
    progressive.chain(interlaced).collect()
}
impl Preferences {
    pub fn video_standard(&self) -> VideoStandard {
        VideoStandard {
            width: self.width,
            height: self.height,
            rate: self.rate,
            scan: self.scan,
        }
    }
}
impl Engine {
    /// Pure, validated new-session construction; no media, devices or network I/O.
    pub fn new_preset(format: VideoStandard, now: u64) -> Result<Self> {
        format.validate()?;
        let mut project = ProjectState::demo();
        project.name = "Untitled production".into();
        project.inputs.truncate(2);
        for (index, input) in project.inputs.iter_mut().enumerate() {
            input.name = format!("Black {}", index + 1);
            input.kind = InputKind::Color;
            input.color = [0; 3];
            input.audio.muted = true;
        }
        project.program = InputId(1);
        project.preview = InputId(2);
        project.preferences.width = format.width;
        project.preferences.height = format.height;
        project.preferences.rate = format.rate;
        project.preferences.scan = format.scan;
        Self::from_project_at(project, now)
    }
}
