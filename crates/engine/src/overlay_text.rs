//! Program overlay caption raster.
//!
//! Overlay channels have always carried a title and a subtitle, but only the
//! operator's Program monitor painted them. Program output, the native Program
//! window, fullscreen and any future recorder showed the panel geometry without
//! any text, so the picture was not the same picture everywhere. This module
//! rasterizes the captions directly into the composited Program surface.
//!
//! The glyphs are an original 5x8 bitmap font authored for this project: no
//! third-party font file, font crate or lockfile change is involved, the output
//! is deterministic on every machine, and a missing glyph degrades to a blank
//! advance instead of an error. A scalable vector font remains the planned
//! upgrade once a reviewed font rasterizer can be added to the lockfile.
//!
//! Placement mirrors the operator painter: panel rectangles come from the same
//! fractions, and the title/subtitle use the same relative offsets and colours.

use crate::{OverlayState, Position};

/// Panel width as a fraction of the Program surface.
pub(crate) const PANEL_WIDTH: f32 = 0.56;
/// Panel height as a fraction of the Program surface.
pub(crate) const PANEL_HEIGHT: f32 = 0.095;
/// Vertical step between stacked panels of the same position.
const PANEL_OFFSET: f32 = 0.105;
/// Bitmap glyph height in pixels at scale 1.
const GLYPH_HEIGHT: i64 = 8;
/// Bitmap glyph advance in pixels at scale 1 (five columns plus one of spacing).
const GLYPH_ADVANCE: i64 = 6;
/// Upper bound on rasterized characters per caption line.
const MAX_CHARS: usize = 96;

/// Panel origin fractions shared by the geometry pass and the caption pass.
pub(crate) fn panel_fractions(index: usize, position: Position) -> (f32, f32) {
    let offset = index as f32 * PANEL_OFFSET;
    match position {
        Position::LowerLeft => (0.05, 0.78 - offset),
        Position::LowerRight => (0.38, 0.78 - offset),
        Position::TopLeft => (0.05, 0.08 + offset),
    }
}

/// Rasterizes every enabled caption into `pixels`.
///
/// The function never panics and never indexes unchecked: out-of-range
/// coordinates, unknown characters and truncated surfaces are skipped, so a
/// malformed overlay cannot interrupt Program output.
pub(crate) fn draw_captions(pixels: &mut [[u8; 4]], w: u32, h: u32, overlays: &[OverlayState]) {
    if w == 0 || h == 0 {
        return;
    }
    for (index, overlay) in overlays.iter().enumerate() {
        if !overlay.enabled {
            continue;
        }
        let (left, top) = panel_fractions(index, overlay.position);
        let panel_x = (left * w as f32).round() as i64;
        let panel_y = (top * h as f32).round() as i64;
        let inset = (0.02 * w as f32).round() as i64;
        let right = panel_x + (0.52 * w as f32).round() as i64;
        let title_top = panel_y + (0.006 * h as f32).round() as i64;
        let subtitle_top = panel_y + (0.052 * h as f32).round() as i64;
        draw_line(
            pixels,
            w,
            h,
            panel_x + inset,
            title_top,
            right,
            &overlay.title,
            line_scale(h, 0.038),
            [255, 255, 255],
        );
        draw_line(
            pixels,
            w,
            h,
            panel_x + inset,
            subtitle_top,
            right,
            &overlay.subtitle,
            line_scale(h, 0.026),
            [190, 190, 190],
        );
    }
}

/// Integer scale for a target pixel height, clamped to the authored range.
fn line_scale(h: u32, fraction: f32) -> i64 {
    ((h as f32 * fraction) / GLYPH_HEIGHT as f32)
        .round()
        .clamp(1.0, 8.0) as i64
}

#[allow(clippy::too_many_arguments)]
fn draw_line(
    pixels: &mut [[u8; 4]],
    w: u32,
    h: u32,
    x: i64,
    top: i64,
    right: i64,
    text: &str,
    scale: i64,
    color: [u8; 3],
) {
    let [red, green, blue] = color;
    let mut pen = x;
    for character in text.chars().take(MAX_CHARS) {
        if pen + GLYPH_ADVANCE * scale > right {
            break;
        }
        if let Some(rows) = glyph(character) {
            for (row, bits) in rows.iter().enumerate() {
                for column in 0..5i64 {
                    if bits & (1 << (4 - column)) == 0 {
                        continue;
                    }
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let px = pen + column * scale + dx;
                            let py = top + row as i64 * scale + dy;
                            let (Ok(px), Ok(py)) = (u32::try_from(px), u32::try_from(py)) else {
                                continue;
                            };
                            if px >= w || py >= h {
                                continue;
                            }
                            let index = py as usize * w as usize + px as usize;
                            if let Some(pixel) = pixels.get_mut(index) {
                                *pixel = [red, green, blue, 255];
                            }
                        }
                    }
                }
            }
        }
        pen += GLYPH_ADVANCE * scale;
    }
}

/// Bitmap rows for one printable ASCII character.
///
/// Each row uses bits 4..0 for the five columns, most significant bit leftmost;
/// row 0 is the cap height and rows 6/7 carry the baseline and descenders.
fn glyph(character: char) -> Option<[u8; 8]> {
    let code = character as u32;
    if !(32..127).contains(&code) {
        return None;
    }
    GLYPHS.get((code - 32) as usize).copied()
}

/// Original 5x8 bitmap font covering U+0020..U+007E.
#[rustfmt::skip]
const GLYPHS: [[u8; 8]; 95] = [
        [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000], // ...../...../...../...../...../...../...../.....
        [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100, 0b00000], // ..#../..#../..#../..#../..#../...../..#../.....
        [0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000], // .#.#./.#.#./...../...../...../...../...../.....
        [0b01010, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b01010, 0b00000], // .#.#./.#.#./#####/.#.#./#####/.#.#./.#.#./.....
        [0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100, 0b00000], // ..#../.####/#.#../.###./..#.#/####./..#../.....
        [0b11001, 0b11001, 0b00010, 0b00100, 0b01000, 0b10011, 0b10011, 0b00000], // ##..#/##..#/...#./..#../.#.../#..##/#..##/.....
        [0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101, 0b00000], // .##../#..#./#.#../.#.../#.#.#/#..#./.##.#/.....
        [0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000], // ..#../..#../...../...../...../...../...../.....
        [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010, 0b00000], // ...#./..#../.#.../.#.../.#.../..#../...#./.....
        [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000, 0b00000], // .#.../..#../...#./...#./...#./..#../.#.../.....
        [0b00000, 0b10101, 0b01110, 0b11111, 0b01110, 0b10101, 0b00000, 0b00000], // ...../#.#.#/.###./#####/.###./#.#.#/...../.....
        [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000, 0b00000], // ...../..#../..#../#####/..#../..#../...../.....
        [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b01000], // ...../...../...../...../...../...../..#../.#...
        [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000, 0b00000], // ...../...../...../#####/...../...../...../.....
        [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00000], // ...../...../...../...../...../...../..#../.....
        [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000, 0b00000], // ....#/...#./...#./..#../.#.../.#.../#..../.....
        [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110, 0b00000], // .###./#...#/#..##/#.#.#/##..#/#...#/.###./.....
        [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110, 0b00000], // ..#../.##../..#../..#../..#../..#../.###./.....
        [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111, 0b00000], // .###./#...#/....#/...#./..#../.#.../#####/.....
        [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110, 0b00000], // #####/...#./..#../...#./....#/#...#/.###./.....
        [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010, 0b00000], // ...#./..##./.#.#./#..#./#####/...#./...#./.....
        [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110, 0b00000], // #####/#..../####./....#/....#/#...#/.###./.....
        [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110, 0b00000], // ..##./.#.../#..../####./#...#/#...#/.###./.....
        [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00000], // #####/....#/...#./..#../.#.../.#.../.#.../.....
        [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110, 0b00000], // .###./#...#/#...#/.###./#...#/#...#/.###./.....
        [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100, 0b00000], // .###./#...#/#...#/.####/....#/...#./.##../.....
        [0b00000, 0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b00000, 0b00000], // ...../...../..#../...../...../..#../...../.....
        [0b00000, 0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b01000, 0b00000], // ...../...../..#../...../...../..#../.#.../.....
        [0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010, 0b00000], // ...#./..#../.#.../#..../.#.../..#../...#./.....
        [0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000], // ...../...../#####/...../#####/...../...../.....
        [0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000, 0b00000], // .#.../..#../...#./....#/...#./..#../.#.../.....
        [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100, 0b00000], // .###./#...#/....#/...#./..#../...../..#../.....
        [0b01110, 0b10001, 0b10111, 0b10101, 0b10111, 0b10000, 0b01110, 0b00000], // .###./#...#/#.###/#.#.#/#.###/#..../.###./.....
        [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001, 0b00000], // .###./#...#/#...#/#####/#...#/#...#/#...#/.....
        [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110, 0b00000], // ####./#...#/#...#/####./#...#/#...#/####./.....
        [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110, 0b00000], // .###./#...#/#..../#..../#..../#...#/.###./.....
        [0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100, 0b00000], // ###../#..#./#...#/#...#/#...#/#..#./###../.....
        [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111, 0b00000], // #####/#..../#..../####./#..../#..../#####/.....
        [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b00000], // #####/#..../#..../####./#..../#..../#..../.....
        [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110, 0b00000], // .###./#...#/#..../#.###/#...#/#...#/.###./.....
        [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001, 0b00000], // #...#/#...#/#...#/#####/#...#/#...#/#...#/.....
        [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110, 0b00000], // .###./..#../..#../..#../..#../..#../.###./.....
        [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100, 0b00000], // ..###/...#./...#./...#./...#./#..#./.##../.....
        [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001, 0b00000], // #...#/#..#./#.#../##.../#.#../#..#./#...#/.....
        [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111, 0b00000], // #..../#..../#..../#..../#..../#..../#####/.....
        [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001, 0b00000], // #...#/##.##/#.#.#/#.#.#/#...#/#...#/#...#/.....
        [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001, 0b00000], // #...#/##..#/#.#.#/#..##/#...#/#...#/#...#/.....
        [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110, 0b00000], // .###./#...#/#...#/#...#/#...#/#...#/.###./.....
        [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000, 0b00000], // ####./#...#/#...#/####./#..../#..../#..../.....
        [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101, 0b00000], // .###./#...#/#...#/#...#/#.#.#/#..#./.##.#/.....
        [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001, 0b00000], // ####./#...#/#...#/####./#.#../#..#./#...#/.....
        [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110, 0b00000], // .####/#..../#..../.###./....#/....#/####./.....
        [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000], // #####/..#../..#../..#../..#../..#../..#../.....
        [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110, 0b00000], // #...#/#...#/#...#/#...#/#...#/#...#/.###./.....
        [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100, 0b00000], // #...#/#...#/#...#/#...#/#...#/.#.#./..#../.....
        [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001, 0b00000], // #...#/#...#/#...#/#.#.#/#.#.#/##.##/#...#/.....
        [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001, 0b00000], // #...#/#...#/.#.#./..#../.#.#./#...#/#...#/.....
        [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000], // #...#/#...#/.#.#./..#../..#../..#../..#../.....
        [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111, 0b00000], // #####/....#/...#./..#../.#.../#..../#####/.....
        [0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01100, 0b00000], // .##../..#../..#../..#../..#../..#../.##../.....
        [0b10000, 0b01000, 0b01000, 0b00100, 0b00010, 0b00010, 0b00001, 0b00000], // #..../.#.../.#.../..#../...#./...#./....#/.....
        [0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01100, 0b00000], // .##../..#../..#../..#../..#../..#../.##../.....
        [0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000], // ..#../.#.#./#...#/...../...../...../...../.....
        [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111], // ...../...../...../...../...../...../...../#####
        [0b01000, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000], // .#.../..#../...../...../...../...../...../.....
        [0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111, 0b00000], // ...../...../.###./....#/.####/#...#/.####/.....
        [0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110, 0b00000], // #..../#..../####./#...#/#...#/#...#/####./.....
        [0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10000, 0b01110, 0b00000], // ...../...../.###./#..../#..../#..../.###./.....
        [0b00001, 0b00001, 0b01111, 0b10001, 0b10001, 0b10001, 0b01111, 0b00000], // ....#/....#/.####/#...#/#...#/#...#/.####/.....
        [0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110, 0b00000], // ...../...../.###./#...#/#####/#..../.###./.....
        [0b00110, 0b01000, 0b11110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000], // ..##./.#.../####./..#../..#../..#../..#../.....
        [0b00000, 0b00000, 0b01111, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110], // ...../...../.####/#...#/#...#/.####/....#/.###.
        [0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b00000], // #..../#..../####./#...#/#...#/#...#/#...#/.....
        [0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110, 0b00000], // ..#../...../.##../..#../..#../..#../.###./.....
        [0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100], // ...#./...../..##./...#./...#./...#./#..#./.##..
        [0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b00000], // #..../#..../#..#./#.#../##.../#.#../#..#./.....
        [0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110, 0b00000], // .##../..#../..#../..#../..#../..#../.###./.....
        [0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10001, 0b10001, 0b00000], // ...../...../##.#./#.#.#/#.#.#/#...#/#...#/.....
        [0b00000, 0b00000, 0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b00000], // ...../...../####./#...#/#...#/#...#/#...#/.....
        [0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110, 0b00000], // ...../...../.###./#...#/#...#/#...#/.###./.....
        [0b00000, 0b00000, 0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000], // ...../...../####./#...#/#...#/####./#..../#....
        [0b00000, 0b00000, 0b01111, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001], // ...../...../.####/#...#/#...#/.####/....#/....#
        [0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000, 0b00000], // ...../...../#.##./##..#/#..../#..../#..../.....
        [0b00000, 0b00000, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110, 0b00000], // ...../...../.####/#..../.###./....#/####./.....
        [0b01000, 0b01000, 0b11110, 0b01000, 0b01000, 0b01000, 0b00110, 0b00000], // .#.../.#.../####./.#.../.#.../.#.../..##./.....
        [0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10001, 0b01111, 0b00000], // ...../...../#...#/#...#/#...#/#...#/.####/.....
        [0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100, 0b00000], // ...../...../#...#/#...#/#...#/.#.#./..#../.....
        [0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010, 0b00000], // ...../...../#...#/#...#/#.#.#/#.#.#/.#.#./.....
        [0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b00000], // ...../...../#...#/.#.#./..#../.#.#./#...#/.....
        [0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110], // ...../...../#...#/#...#/#...#/.####/....#/.###.
        [0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111, 0b00000], // ...../...../#####/...#./..#../.#.../#####/.....
        [0b00011, 0b00100, 0b00100, 0b01000, 0b00100, 0b00100, 0b00011, 0b00000], // ...##/..#../..#../.#.../..#../..#../...##/.....
        [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000], // ..#../..#../..#../..#../..#../..#../..#../.....
        [0b11000, 0b00100, 0b00100, 0b00010, 0b00100, 0b00100, 0b11000, 0b00000], // ##.../..#../..#../...#./..#../..#../##.../.....
        [0b00000, 0b00000, 0b01001, 0b10101, 0b10010, 0b00000, 0b00000, 0b00000], // ...../...../.#..#/#.#.#/#..#./...../...../.....
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_covers_printable_ascii_with_original_bitmaps() {
        assert_eq!(GLYPHS.len(), 95);
        assert!(glyph(' ').is_some_and(|rows| rows.iter().all(|row| *row == 0)));
        for code in 33..127u32 {
            let Some(character) = char::from_u32(code) else {
                continue;
            };
            assert!(
                glyph(character).is_some_and(|rows| rows.iter().any(|row| *row != 0)),
                "glyph {character:?} must contain ink"
            );
        }
        assert!(glyph('é').is_none());
        assert!(glyph('\u{7f}').is_none());
    }

    #[test]
    fn every_glyph_stays_inside_five_columns_and_eight_rows() {
        for rows in GLYPHS {
            for row in rows {
                assert!(row & !0b1_1111 == 0, "row {row:#07b} uses reserved bits");
            }
        }
    }

    #[test]
    fn disabled_captions_draw_nothing() {
        let mut pixels = vec![[9, 9, 9, 255]; 64 * 36];
        let overlays = [OverlayState {
            enabled: false,
            title: "LIVE".into(),
            subtitle: "CAM 1".into(),
            position: Position::LowerLeft,
        }];
        draw_captions(&mut pixels, 64, 36, &overlays);
        assert!(pixels.iter().all(|pixel| *pixel == [9, 9, 9, 255]));
    }

    #[test]
    fn captions_paint_inside_the_panel_only() {
        let (w, h) = (640u32, 360u32);
        let mut pixels = vec![[9, 9, 9, 255]; (w * h) as usize];
        let overlays = [OverlayState {
            enabled: true,
            title: "LIVE".into(),
            subtitle: "CAM 1".into(),
            position: Position::LowerLeft,
        }];
        draw_captions(&mut pixels, w, h, &overlays);
        let (left, top) = panel_fractions(0, Position::LowerLeft);
        let right = left + PANEL_WIDTH;
        let bottom = top + PANEL_HEIGHT;
        let white = pixels
            .iter()
            .filter(|pixel| **pixel == [255, 255, 255, 255])
            .count();
        let grey = pixels
            .iter()
            .filter(|pixel| **pixel == [190, 190, 190, 255])
            .count();
        assert!(white >= 20, "title ink expected, found {white}");
        assert!(grey >= 10, "subtitle ink expected, found {grey}");
        for y in 0..h {
            for x in 0..w {
                let index = y as usize * w as usize + x as usize;
                let painted = pixels
                    .get(index)
                    .is_some_and(|pixel| *pixel != [9, 9, 9, 255]);
                if painted {
                    // The geometry pass compares fractions, so the oracle uses
                    // the same rule rather than a rounded integer rectangle.
                    let u = x as f32 / w as f32;
                    let v = y as f32 / h as f32;
                    assert!(
                        u >= left && u < right && v >= top && v < bottom,
                        "caption pixel ({x},{y}) escaped the panel"
                    );
                }
            }
        }
    }

    #[test]
    fn unknown_characters_advance_without_ink() {
        let (w, h) = (64u32, 16u32);
        let mut pixels = vec![[0, 0, 0, 255]; (w * h) as usize];
        let overlays = [OverlayState {
            enabled: true,
            title: "éé".into(),
            subtitle: String::new(),
            position: Position::TopLeft,
        }];
        draw_captions(&mut pixels, w, h, &overlays);
        assert!(pixels.iter().all(|pixel| *pixel == [0, 0, 0, 255]));
    }
}
