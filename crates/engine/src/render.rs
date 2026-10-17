//! Bounded deterministic SDR, full-range RGBA8 demo renderer.
//! Blending is in display-referred sRGB for this UI prototype, not a broadcast color pipeline.
use crate::{
    InputKind, InputState, Position, ProjectState, Result, TransitionKind, TransitionState, invalid,
};
fn pixel(input: &InputState, x: u32, y: u32, w: u32, h: u32, ms: u64) -> [u8; 4] {
    if !input.online {
        return [0, 0, 0, 255];
    }
    let u = x as f32 / w.max(1) as f32;
    let v = y as f32 / h.max(1) as f32;
    let phase = (ms % 10000) as f32 / 10000.0;
    let c = input.color;
    let mix = |k: f32| {
        [
            (c.first().copied().unwrap_or(0) as f32 * k) as u8,
            (c.get(1).copied().unwrap_or(0) as f32 * k) as u8,
            (c.get(2).copied().unwrap_or(0) as f32 * k) as u8,
            255,
        ]
    };
    match input.kind {
        InputKind::Bars => {
            let colors = [
                [190, 190, 190, 255],
                [190, 190, 0, 255],
                [0, 190, 190, 255],
                [0, 190, 0, 255],
                [190, 0, 190, 255],
                [190, 0, 0, 255],
                [0, 0, 190, 255],
            ];
            if v > 0.78 {
                let l = (((u + phase) % 1.0) * 255.0) as u8;
                [l, l, l, 255]
            } else {
                colors
                    .get((u * 7.0) as usize)
                    .copied()
                    .unwrap_or([0, 0, 0, 255])
            }
        }
        InputKind::Grid => {
            let grid = ((u * 16.0 + phase * 2.0).fract() < 0.018) || ((v * 9.0).fract() < 0.025);
            let ring = ((u - 0.5).powi(2) + (v - 0.5).powi(2) * 0.32).sqrt();
            if (ring - (0.12 + phase * 0.25)).abs() < 0.003 {
                [91, 218, 230, 255]
            } else if grid {
                [40, 76, 90, 255]
            } else {
                mix(0.12 + (1.0 - v) * 0.12)
            }
        }
        InputKind::Geometry => {
            let cx = 0.25 + phase * 0.5;
            let cy = 0.5 + (phase * std::f32::consts::TAU).sin() * 0.15;
            if (u - cx).powi(2) + (v - cy).powi(2) * 0.32 < 0.018 {
                [235, 163, 81, 255]
            } else if (u + v * 0.4 + phase).fract() < 0.08 {
                [70, 51, 44, 255]
            } else {
                [24, 28, 38, 255]
            }
        }
        InputKind::Graphics => {
            if v > 0.62 && v < 0.82 && u > 0.07 && u < 0.88 {
                if u < 0.09 {
                    [40, 193, 207, 255]
                } else {
                    [20, 47, 66, 255]
                }
            } else {
                [15, 23, 32, 255]
            }
        }
        InputKind::Studio => {
            if v > 0.76 {
                let k = ((u - 0.5).abs() + (v - 0.76) * 0.4).min(1.0);
                mix(0.1 + k * 0.3)
            } else if u > 0.1 && u < 0.9 && v > 0.15 && v < 0.62 {
                if (u * 12.0 + phase).fract() < 0.03 {
                    [34, 125, 158, 255]
                } else {
                    mix(0.5 + u * 0.2)
                }
            } else {
                [17, 30, 43, 255]
            }
        }
        InputKind::Color => mix(0.25 + 0.75 * u),
        InputKind::Countdown => {
            if v > 0.82 && u < 1.0 - (ms % 60000) as f32 / 60000.0 {
                [35, 174, 157, 255]
            } else {
                [17, 28, 34, 255]
            }
        }
    }
}
fn dimensions(w: u32, h: u32) -> Result<usize> {
    if w == 0 || h == 0 || w > 1920 || h > 1080 {
        return Err(invalid("Demo render dimensions must fit 1920 × 1080"));
    }
    w.checked_mul(h)
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| invalid("Render allocation overflow"))
}
pub fn source(input: &InputState, w: u32, h: u32, ms: u64) -> Result<Vec<[u8; 4]>> {
    let size = dimensions(w, h)?;
    let mut out = Vec::with_capacity(size);
    for y in 0..h {
        for x in 0..w {
            out.push(pixel(input, x, y, w, h, ms));
        }
    }
    Ok(out)
}
pub fn blend(a: [u8; 4], b: [u8; 4], p: f32) -> [u8; 4] {
    let p = if p.is_finite() {
        p.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut out = [0, 0, 0, 255];
    for ((c, a), b) in out.iter_mut().take(3).zip(a).zip(b) {
        *c = (a as f32 * (1.0 - p) + b as f32 * p).round() as u8;
    }
    out
}
pub fn program(
    p: &ProjectState,
    t: Option<&TransitionState>,
    black: f32,
    w: u32,
    h: u32,
    ms: u64,
) -> Result<Vec<[u8; 4]>> {
    let size = dimensions(w, h)?;
    let mut out = Vec::with_capacity(size);
    let from = p
        .input(t.map_or(p.program, |t| t.from))
        .ok_or_else(|| invalid("Program source missing"))?;
    let target = t.and_then(|t| p.input(t.to));
    for y in 0..h {
        for x in 0..w {
            let mut color = pixel(from, x, y, w, h, ms);
            if let (Some(t), Some(to)) = (t, target) {
                let b = pixel(to, x, y, w, h, ms);
                color = match t.kind {
                    TransitionKind::Fade => blend(color, b, t.progress),
                    TransitionKind::Wipe => {
                        if (x as f32 / w as f32) < t.progress {
                            b
                        } else {
                            color
                        }
                    }
                };
            }
            for (i, o) in p.overlays.iter().enumerate() {
                if o.enabled {
                    let u = x as f32 / w as f32;
                    let v = y as f32 / h as f32;
                    let offset = i as f32 * 0.105;
                    let (left, top) = match o.position {
                        Position::LowerLeft => (0.05, 0.78 - offset),
                        Position::LowerRight => (0.38, 0.78 - offset),
                        Position::TopLeft => (0.05, 0.08 + offset),
                    };
                    if u >= left && u < left + 0.56 && v >= top && v < top + 0.095 {
                        color = if u < left + 0.008 {
                            [39, 192, 207, 255]
                        } else {
                            blend(color, [14, 29, 42, 255], 0.94)
                        };
                    }
                }
            }
            out.push(blend(color, [0, 0, 0, 255], black));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_blend_vectors() {
        assert_eq!(
            blend([20, 80, 180, 255], [220, 40, 20, 255], 0.5),
            [120, 60, 100, 255]
        );
        assert_eq!(
            blend([20, 80, 180, 255], [220, 40, 20, 255], 1.0),
            [220, 40, 20, 255]
        );
    }
    #[test]
    fn simulated_meters_are_deterministic() {
        let p = ProjectState::demo();
        assert_eq!(
            crate::audio::bus_peaks(&p, 0, 1234),
            crate::audio::bus_peaks(&p, 0, 1234)
        );
    }
}
