//! Bounded deterministic SDR, full-range RGBA8 demo renderer.
//! Blending is in display-referred sRGB for this UI prototype, not a broadcast color pipeline.
use crate::{
    InputKind, InputState, ProjectState, Result, TransitionKind, TransitionState, invalid,
    overlay_text,
};
fn pixel(input: &InputState, x: u32, y: u32, w: u32, h: u32, ms: u64) -> [u8; 4] {
    if !input.online
        || input.config.media_path.is_some()
        || input.config.audio_source.is_some()
        || input.config.video_source.is_some()
    {
        // Still-image, audio-only and video-file inputs are decoded outside this
        // reference generator; until a valid frame exists they must show black
        // rather than a plausible-looking procedural picture.
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
    if w == 0 || h == 0 || w > 1920 || h > 1920 || u64::from(w) * u64::from(h) > 1920 * 1080 {
        return Err(invalid(
            "Program render dimensions must fit a full-HD pixel budget",
        ));
    }
    w.checked_mul(h)
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| invalid("Render allocation overflow"))
}
pub fn source(input: &InputState, w: u32, h: u32, ms: u64) -> Result<Vec<[u8; 4]>> {
    let mut out = Vec::new();
    source_into(input, w, h, ms, &mut out)?;
    Ok(out)
}
pub fn source_into(
    input: &InputState,
    w: u32,
    h: u32,
    ms: u64,
    out: &mut Vec<[u8; 4]>,
) -> Result<()> {
    let size = dimensions(w, h)?;
    out.clear();
    out.reserve(size);
    for y in 0..h {
        for x in 0..w {
            out.push(pixel(input, x, y, w, h, ms));
        }
    }
    Ok(())
}

/// Two-pass box filter from a full-resolution Program frame to the operator
/// monitor. Ranges and contiguous row slices avoid per-sample division.
pub fn downscale(src: &[[u8; 4]], sw: u32, sh: u32, dw: u32, dh: u32) -> Result<Vec<[u8; 4]>> {
    let source_size = dimensions(sw, sh)?;
    let target_size = dimensions(dw, dh)?;
    if src.len() != source_size {
        return Err(invalid("Downscale source length does not match its raster"));
    }
    if (sw, sh) == (dw, dh) {
        return Ok(src.to_vec());
    }
    let mut x_ranges = Vec::with_capacity(dw as usize);
    let mut x_inverse = Vec::with_capacity(dw as usize);
    for dx in 0..dw {
        let x0 = dx * sw / dw;
        let x1 = ((dx + 1) * sw / dw).max(x0 + 1).min(sw);
        x_ranges.push((x0, x1));
        x_inverse.push(reciprocal(x1.saturating_sub(x0).max(1)));
    }
    let mut y_ranges = Vec::with_capacity(dh as usize);
    let mut y_inverse = Vec::with_capacity(dh as usize);
    for dy in 0..dh {
        let y0 = dy * sh / dh;
        let y1 = ((dy + 1) * sh / dh).max(y0 + 1).min(sh);
        y_ranges.push((y0, y1));
        y_inverse.push(reciprocal(y1.saturating_sub(y0).max(1)));
    }
    let mut row_sums = vec![[0_u32; 4]; dw as usize];
    let mut out = Vec::with_capacity(target_size);
    for (dy, &(y0, y1)) in y_ranges.iter().enumerate() {
        let inverse_y = y_inverse.get(dy).copied().unwrap_or(1 << 32);
        row_sums.fill([0_u32; 4]);
        for y in y0..y1 {
            let start = (y as usize)
                .checked_mul(sw as usize)
                .ok_or_else(|| invalid("Downscale row offset exceeds its raster"))?;
            let end = start
                .checked_add(sw as usize)
                .ok_or_else(|| invalid("Downscale row end exceeds its raster"))?;
            let row = src
                .get(start..end)
                .ok_or_else(|| invalid("Downscale source row is missing"))?;
            for (dx, &(x0, x1)) in x_ranges.iter().enumerate() {
                let segment = row
                    .get(x0 as usize..x1 as usize)
                    .ok_or_else(|| invalid("Downscale source column is missing"))?;
                let sums = row_sums
                    .get_mut(dx)
                    .ok_or_else(|| invalid("Downscale row accumulator is missing"))?;
                for pixel in segment {
                    let [red, green, blue, alpha] = *pixel;
                    let [sr, sg, sb, sa] = sums;
                    *sr = sr.saturating_add(u32::from(red));
                    *sg = sg.saturating_add(u32::from(green));
                    *sb = sb.saturating_add(u32::from(blue));
                    *sa = sa.saturating_add(u32::from(alpha));
                }
            }
        }
        for (dx, sums) in row_sums.iter().enumerate() {
            let inverse_x = x_inverse.get(dx).copied().unwrap_or(1 << 32);
            let [red, green, blue, alpha] = *sums;
            out.push([
                scaled_average(scaled_average(red, inverse_x), inverse_y) as u8,
                scaled_average(scaled_average(green, inverse_x), inverse_y) as u8,
                scaled_average(scaled_average(blue, inverse_x), inverse_y) as u8,
                scaled_average(scaled_average(alpha, inverse_x), inverse_y) as u8,
            ]);
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
    let from = p
        .input(t.map_or(p.program, |t| t.from))
        .ok_or_else(|| invalid("Program source missing"))?;
    let target = t.and_then(|t| p.input(t.to));
    let from = source(from, w, h, ms)?;
    let target = target.map(|input| source(input, w, h, ms)).transpose()?;
    program_from_sources(p, t, black, w, h, &from, target.as_deref())
}

/// Paint only the enabled title-panel rectangles. The previous per-pixel loop
/// checked every overlay on every pixel, which dominated full-HD composition.
fn paint_overlay_panels(out: &mut [[u8; 4]], w: u32, h: u32, overlays: &[crate::OverlayState]) {
    for (index, overlay) in overlays.iter().enumerate() {
        if !overlay.enabled {
            continue;
        }
        let (left, top) = overlay_text::panel_fractions(index, overlay.position);
        let x0 = ((left * w as f32).floor() as i64).clamp(0, i64::from(w)) as u32;
        let x1 = (((left + overlay_text::PANEL_WIDTH) * w as f32).ceil() as i64)
            .clamp(0, i64::from(w)) as u32;
        let y0 = ((top * h as f32).floor() as i64).clamp(0, i64::from(h)) as u32;
        let y1 = (((top + overlay_text::PANEL_HEIGHT) * h as f32).ceil() as i64)
            .clamp(0, i64::from(h)) as u32;
        for y in y0..y1 {
            for x in x0..x1 {
                let u = x as f32 / w as f32;
                let v = y as f32 / h as f32;
                if u >= left
                    && u < left + overlay_text::PANEL_WIDTH
                    && v >= top
                    && v < top + overlay_text::PANEL_HEIGHT
                {
                    let offset = (y as usize)
                        .saturating_mul(w as usize)
                        .saturating_add(x as usize);
                    if let Some(pixel) = out.get_mut(offset) {
                        *pixel = if u < left + 0.008 {
                            [39, 192, 207, 255]
                        } else {
                            blend(*pixel, [14, 29, 42, 255], 0.94)
                        };
                    }
                }
            }
        }
    }
}

/// Composite already processed sources, with independent source clocks/effects.
/// Titles/backgrounds and fade-to-black are applied after source processing.
pub fn program_from_sources(
    p: &ProjectState,
    t: Option<&TransitionState>,
    black: f32,
    w: u32,
    h: u32,
    from: &[[u8; 4]],
    target: Option<&[[u8; 4]]>,
) -> Result<Vec<[u8; 4]>> {
    let size = dimensions(w, h)?;
    if from.len() != size
        || target.is_some_and(|pixels| pixels.len() != size)
        || (t.is_some() && target.is_none())
        || !black.is_finite()
        || !(0.0..=1.0).contains(&black)
        || t.is_some_and(|t| !t.progress.is_finite() || !(0.0..=1.0).contains(&t.progress))
    {
        return Err(invalid("Invalid source surface or compositor state"));
    }
    if t.is_none() {
        let mut out = from.to_vec();
        finish_program(&mut out, p, w, h, black);
        return Ok(out);
    }
    if let Some(t) = t
        && t.kind == TransitionKind::Cut
    {
        let surface = if t.progress >= 1.0 {
            target.ok_or_else(|| invalid("Target surface missing"))?
        } else {
            from
        };
        let mut out = surface.to_vec();
        finish_program(&mut out, p, w, h, black);
        return Ok(out);
    }
    if let Some(t) = t
        && t.kind == TransitionKind::Fade
    {
        let to = target.ok_or_else(|| invalid("Target surface missing"))?;
        let mut out = fade_surfaces(from, to, t.progress);
        finish_program(&mut out, p, w, h, black);
        return Ok(out);
    }
    let mut out = Vec::with_capacity(size);
    for y in 0..h {
        for x in 0..w {
            let index = (y * w + x) as usize;
            let mut color = from
                .get(index)
                .copied()
                .ok_or_else(|| invalid("Source surface truncated"))?;
            if let (Some(t), Some(to)) = (t, target) {
                let b = to
                    .get(index)
                    .copied()
                    .ok_or_else(|| invalid("Target surface truncated"))?;
                color = match t.kind {
                    TransitionKind::Cut => {
                        if t.progress >= 1.0 {
                            b
                        } else {
                            color
                        }
                    }
                    TransitionKind::Fade => blend(color, b, t.progress),
                    TransitionKind::Wipe => {
                        if (x as f32 / w as f32) < t.progress {
                            b
                        } else {
                            color
                        }
                    }
                    TransitionKind::WipeReverse => {
                        if x as f32 / w as f32 >= 1.0 - t.progress {
                            b
                        } else {
                            color
                        }
                    }
                    TransitionKind::VerticalWipe => {
                        if (y as f32 / h as f32) < t.progress {
                            b
                        } else {
                            color
                        }
                    }
                    TransitionKind::VerticalWipeReverse => {
                        if y as f32 / h as f32 >= 1.0 - t.progress {
                            b
                        } else {
                            color
                        }
                    }
                    TransitionKind::BarnDoor => {
                        if ((2 * x + 1) as f32 / w as f32 - 1.0).abs() < t.progress {
                            b
                        } else {
                            color
                        }
                    }
                    TransitionKind::RollerDoor => {
                        if ((2 * y + 1) as f32 / h as f32 - 1.0).abs() < t.progress {
                            b
                        } else {
                            color
                        }
                    }
                    TransitionKind::Slide
                    | TransitionKind::SlideReverse
                    | TransitionKind::VerticalSlide
                    | TransitionKind::VerticalSlideReverse => {
                        let vertical = matches!(
                            t.kind,
                            TransitionKind::VerticalSlide | TransitionKind::VerticalSlideReverse
                        );
                        let reverse = matches!(
                            t.kind,
                            TransitionKind::SlideReverse | TransitionKind::VerticalSlideReverse
                        );
                        let extent = if vertical { h } else { w };
                        let coordinate = if vertical { y } else { x };
                        let shift = (extent as f32 * t.progress).round() as u32;
                        let boundary = extent.saturating_sub(shift);
                        let (surface, offset) = if reverse {
                            if coordinate < shift {
                                (to, coordinate + boundary)
                            } else {
                                (from, coordinate - shift)
                            }
                        } else if coordinate < boundary {
                            (from, coordinate + shift)
                        } else {
                            (to, coordinate - boundary)
                        };
                        let (sx, sy) = if vertical { (x, offset) } else { (offset, y) };
                        let sample = u64::from(sy)
                            .checked_mul(u64::from(w))
                            .and_then(|n| n.checked_add(u64::from(sx)))
                            .and_then(|n| usize::try_from(n).ok())
                            .ok_or_else(|| invalid("Slide pixel offset exceeds surface"))?;
                        surface
                            .get(sample)
                            .copied()
                            .ok_or_else(|| invalid("Slide surface truncated"))?
                    }
                };
            }
            out.push(color);
        }
    }
    finish_program(&mut out, p, w, h, black);
    Ok(out)
}

fn finish_program(out: &mut [[u8; 4]], p: &ProjectState, w: u32, h: u32, black: f32) {
    paint_overlay_panels(out, w, h, &p.overlays.standard);
    // Captions belong to the overlay layer: they are rasterized after the panel
    // geometry and before any fade-to-black so Program output, the native window
    // and future recorders all show the same picture as the operator monitor.
    overlay_text::draw_captions(out, w, h, &p.overlays.standard);
    if black > 0.0 {
        fade_to_black(out, black);
    }
}

#[inline]
fn reciprocal(count: u32) -> u64 {
    (1_u64 << 32) / u64::from(count.max(1))
}

#[inline]
fn scaled_average(value: u32, reciprocal: u64) -> u32 {
    ((u64::from(value) * reciprocal + (1_u64 << 31)) >> 32) as u32
}

#[inline]
fn fade_channel(a: u8, b: u8, keep: u32, alpha: u32) -> u8 {
    ((u32::from(a) * keep + u32::from(b) * alpha + 127) / 255) as u8
}

fn fade_table(keep: u32, alpha: u32) -> [u8; 65536] {
    let mut table = [0_u8; 65536];
    for a in 0..=255_u8 {
        for b in 0..=255_u8 {
            let index = usize::from(a) * 256 + usize::from(b);
            if let Some(cell) = table.get_mut(index) {
                *cell = fade_channel(a, b, keep, alpha);
            }
        }
    }
    table
}

#[inline]
fn table_lookup(table: &[u8; 65536], a: u8, b: u8) -> u8 {
    table
        .get(usize::from(a) * 256 + usize::from(b))
        .copied()
        .unwrap_or(0)
}

fn fade_to_black(out: &mut [[u8; 4]], black: f32) {
    let alpha = (black.clamp(0.0, 1.0) * 255.0).round() as u32;
    let keep = 255_u32.saturating_sub(alpha);
    let mut table = [0_u8; 256];
    for (a, cell) in table.iter_mut().enumerate() {
        *cell = fade_channel(u8::try_from(a).unwrap_or(0), 0, keep, alpha);
    }
    for pixel in out {
        let [red, green, blue, _] = *pixel;
        let lookup = |value: u8| table.get(usize::from(value)).copied().unwrap_or(0);
        *pixel = [lookup(red), lookup(green), lookup(blue), 255];
    }
}

fn fade_surfaces(from: &[[u8; 4]], to: &[[u8; 4]], progress: f32) -> Vec<[u8; 4]> {
    if progress <= 0.0 {
        return from.to_vec();
    }
    if progress >= 1.0 {
        return to.to_vec();
    }
    let alpha = (progress * 255.0).round() as u32;
    let keep = 255_u32.saturating_sub(alpha);
    let table = fade_table(keep, alpha);
    let mut out = vec![[0_u8; 4]; from.len().min(to.len())];
    for (destination, (a, b)) in out.iter_mut().zip(from.iter().zip(to)) {
        let [ar, ag, ab, _] = *a;
        let [br, bg, bb, _] = *b;
        *destination = [
            table_lookup(&table, ar, br),
            table_lookup(&table, ag, bg),
            table_lookup(&table, ab, bb),
            255,
        ];
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    #[test]
    fn external_media_inputs_render_black_in_the_reference_generator() {
        let demo = ProjectState::demo();
        let Some(mut input) = demo.input(crate::InputId(1)).cloned() else {
            return;
        };
        input.kind = InputKind::Color;
        input.online = true;
        input.config = crate::InputConfig::default();
        let procedural = source(&input, 2, 2, 0);
        assert!(
            procedural
                .as_ref()
                .is_ok_and(|pixels| pixels.iter().all(|pixel| pixel.get(3) == Some(&255))),
            "procedural colour input still renders opaque pixels"
        );
        for config in [
            crate::InputConfig {
                media_path: Some("still.png".into()),
                ..crate::InputConfig::default()
            },
            crate::InputConfig {
                audio_source: Some(crate::AudioSource::WavFile {
                    path: "clip.wav".into(),
                }),
                ..crate::InputConfig::default()
            },
            crate::InputConfig {
                video_source: Some(crate::VideoSource::File {
                    path: "clip.mp4".into(),
                }),
                ..crate::InputConfig::default()
            },
        ] {
            input.config = config;
            assert!(
                source(&input, 2, 2, 0)
                    .is_ok_and(|pixels| pixels.iter().all(|pixel| *pixel == [0, 0, 0, 255])),
                "external media must never fall back to a procedural picture"
            );
        }
    }

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
    #[test]
    fn processed_sources_keep_their_pixels_and_black_is_applied_last() {
        let p = ProjectState::demo();
        let from = [[200, 20, 80, 255]; 4];
        let to = [[10, 180, 40, 255]; 4];
        let mut t = TransitionState {
            from: p.program,
            to: p.preview,
            kind: TransitionKind::Fade,
            start_ms: 0,
            duration_ms: 1000,
            progress: 0.5,
            manual: true,
        };
        assert_eq!(
            program_from_sources(&p, Some(&t), 0.0, 2, 2, &from, Some(&to)).ok(),
            Some(vec![[105, 100, 60, 255]; 4])
        );
        assert_eq!(
            program_from_sources(&p, Some(&t), 1.0, 2, 2, &from, Some(&to)).ok(),
            Some(vec![[0, 0, 0, 255]; 4])
        );
        t.kind = TransitionKind::Wipe;
        assert_eq!(
            program_from_sources(&p, Some(&t), 0.0, 2, 2, &from, Some(&to)).ok(),
            Some(vec![
                [10, 180, 40, 255],
                [200, 20, 80, 255],
                [10, 180, 40, 255],
                [200, 20, 80, 255]
            ])
        );
        assert!(program_from_sources(&p, Some(&t), 0.0, 2, 2, &from, None).is_err());
        assert!(program_from_sources(&p, None, f32::NAN, 2, 2, &from, None).is_err());
        assert!(program_from_sources(&p, None, 0.0, 3, 2, &from, None).is_err());
    }

    #[test]
    fn program_captions_reach_the_composited_surface_and_fade_with_black() {
        let (w, h) = (640u32, 360u32);
        let from = vec![[10, 10, 10, 255]; (w * h) as usize];
        let mut p = ProjectState::demo();
        let Some(plain) = program_from_sources(&p, None, 0.0, w, h, &from, None).ok() else {
            return;
        };
        assert_eq!(plain, from, "disabled overlays must retain source pixels");
        assert!(
            !p.overlays.standard.is_empty(),
            "the demo project must expose overlay channels"
        );
        for position in [
            crate::Position::LowerLeft,
            crate::Position::LowerRight,
            crate::Position::TopLeft,
        ] {
            if let Some(overlay) = p.overlays.standard.first_mut() {
                overlay.enabled = true;
                overlay.position = position;
                overlay.title = "LIVE".into();
                overlay.subtitle = "CAM 1".into();
            }
            let Some(painted) = program_from_sources(&p, None, 0.0, w, h, &from, None).ok() else {
                return;
            };
            let white = painted
                .iter()
                .filter(|pixel| **pixel == [255, 255, 255, 255])
                .count();
            let grey = painted
                .iter()
                .filter(|pixel| **pixel == [190, 190, 190, 255])
                .count();
            assert!(
                white >= 20,
                "{position:?}: title ink expected, found {white}"
            );
            assert!(
                grey >= 8,
                "{position:?}: subtitle ink expected, found {grey}"
            );
            let (left, top) = overlay_text::panel_fractions(0, position);
            let right = left + overlay_text::PANEL_WIDTH;
            let bottom = top + overlay_text::PANEL_HEIGHT;
            for (index, pixel) in painted.iter().enumerate() {
                if *pixel == from.get(index).copied().unwrap_or([10, 10, 10, 255]) {
                    continue;
                }
                let x = index as u32 % w;
                let y = index as u32 / w;
                // Compare with the same float rule the geometry pass uses, so a
                // rounded integer rectangle cannot report a false escape on the
                // panel's right or bottom boundary pixel.
                let u = x as f32 / w as f32;
                let v = y as f32 / h as f32;
                assert!(
                    u >= left && u < right && v >= top && v < bottom,
                    "{position:?}: caption pixel ({x},{y}) escaped its panel"
                );
            }
            let Some(faded) = program_from_sources(&p, None, 1.0, w, h, &from, None).ok() else {
                return;
            };
            assert!(
                faded.iter().all(|pixel| *pixel == [0, 0, 0, 255]),
                "{position:?}: fade-to-black must cover caption ink"
            );
        }
    }

    #[test]
    fn full_hd_downscale_preserves_colour_and_bounds_its_inputs() {
        let source = [[40, 120, 200, 255]; 32 * 18];
        let scaled = downscale(&source, 32, 18, 16, 9).unwrap();
        assert_eq!(scaled.len(), 16 * 9);
        assert!(scaled.iter().all(|pixel| *pixel == [40, 120, 200, 255]));
        let portrait = downscale(&source, 32, 18, 9, 16).unwrap();
        assert_eq!(portrait.len(), 9 * 16);
        assert!(downscale(&source, 32, 18, 16, 9).is_ok());
        assert!(downscale(&source[..10], 32, 18, 16, 9).is_err());
        assert!(downscale(&source, 0, 18, 16, 9).is_err());
        assert_eq!(downscale(&source, 32, 18, 32, 18).unwrap(), source.to_vec());
    }

    #[test]
    fn program_bytes_fit_full_hd_in_either_orientation() {
        assert!(dimensions(1920, 1080).is_ok());
        assert!(dimensions(1080, 1920).is_ok());
        assert!(dimensions(3840, 2160).is_err());
        assert!(dimensions(1920, 1081).is_err());
    }
}
