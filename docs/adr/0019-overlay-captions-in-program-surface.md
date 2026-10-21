# ADR 0019: overlay captions rasterized into the Program surface

2026-10-19 · implementation complete; hosted unit and native acceptance pending for this change.

Overlay channels have carried a title and a subtitle since the overlay bank was added, but the
engine compositor only drew the panel geometry (a cyan left edge plus a translucent dark fill). The
text was painted exclusively by the operator's own Program monitor with `egui::Painter`, so the
separate Program window, fullscreen output, GPU captures and any future recorder showed a lower
third with no words in it. "Correct picture everywhere" is a first-order rule in `AGENTS.md`, and a
monitor-only caption also made the operator monitor the only place where the composed picture was
complete.

Decision:

- The engine compositor rasterizes enabled caption text into the Program surface itself, after the
  panel geometry and **before** fade-to-black. Ordering matters: FTB is a program-wide black, so
  caption ink must be under it, exactly like the panel fill.
- The glyphs are an original 5x8 bitmap font authored in this repository
  (`crates/engine/src/overlay_text.rs`), covering U+0020..U+007E. This avoids a third-party font
  file, a font-rasterizer dependency and the lockfile regeneration that a new dependency would
  require under this repository's owner policy. Rendering is deterministic on every machine and
  every supported Program size.
- Placement, scale and colours mirror the removed operator painter: panel origin and size come from
  `overlay_text::panel_fractions` / `PANEL_WIDTH` / `PANEL_HEIGHT`, the title uses
  `0.02w` inset, `0.006h` top offset, `0.038h` target height and white; the subtitle uses
  `0.052h` top offset, `0.026h` target height and grey 190. The bitmap line scale is
  `clamp(round(target_height / 8), 1, 8)`, so the same caption reads correctly at 640×360 today and
  at larger Program rasters later.
- Failure behaviour is bounded and never interrupts Program output: unknown characters advance
  blank, characters beyond 96 per line are dropped, text stops at the panel's 0.52w text edge, and
  every write goes through `get_mut` with explicit bounds checks. A malformed overlay cannot panic.
- The duplicated `egui::Painter` caption block in the operator monitor is removed. The monitor now
  shows exactly the composed Program raster, so monitor, Program window, fullscreen and captures
  are the same picture.
- Acceptance: engine unit tests assert glyph coverage, ink containment inside the panel rectangle
  for all three positions, subtitle ink, fade-to-black covering the ink, and that disabled overlays
  leave source pixels untouched. `--ui-output-smoke` sets deterministic caption text in its live
  stage, maps every title/subtitle ink pixel of the Program raster into the captured native Program
  viewport and requires them to be bright there, recording the counts in
  `artifacts/program-output-live.json`. Because the overlay fill darkens its panel region, a bright
  pixel inside the panel can only come from caption ink.

Non-claims: this is a bitmap font, not a scalable/typeset title engine. There is no font choice,
bold/italic, outline, shadow, background box, alpha ramp, animation, ticker, data binding or
per-source overlay yet; characters outside printable ASCII are skipped; the glyphs are original but
intentionally simple, so typographic quality is below the operator UI's own font rendering. Captions
are only composited into the experimental 640×360 CPU Program raster; no GPU, hardware, encoder,
physical-output or endurance qualification is implied. Stinger channels still composite no clip,
and the synthetic `Graphics` input's decorative text remains an operator-monitor painter, so that
one input is still not identical in Program output.