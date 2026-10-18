# ADR 0012: sixteen-channel production overlay bank and stinger exchange

2026-10-18 · reference Task 1.1 port (production overlay model) · Implemented in source;
local headless verification only, hosted acceptance pending.

The reference task list defines sixteen production overlay channels: standard channels `1–8`
and stinger channels `9–16`, where each stinger carries its source input, a cut point and a
display-underneath flag. RustMix previously stored four title panels (`overlays: [OverlayState; 4]`)
with no per-channel split and no stinger concept. The port must extend the model without
changing existing operator behavior or losing authored panels.

Decision, implemented in `crates/engine/src/overlays.rs` and the existing crates:

- Schema 5 stores `OverlayBank { standard: [OverlayState; 8], stingers: [StingerChannel; 8] }`
  under the existing `overlays` field name. Standard channels keep exactly the previous
  title-panel shape (`enabled`/`title`/`subtitle`/`position`); stinger channels carry
  `stinger_input: Option<InputId>`, `cut_point_ms: u32` (bounded to 60 000 ms) and
  `display_underneath_overlays: bool`.
- Migration reads schema 1–4 files as JSON values: the legacy four-element `overlays` array is
  expanded into the bank, preserving every authored title/subtitle/position/enabled flag.
  Channels 5–8 keep their fresh defaults and stingers stay idle. Save-time rollback copies keep
  the original bytes, so an older application can still read `bak1`.
- Commands: `overlay.set` now addresses standard channels 0–7 (`overlay.stinger.set`,
  `overlay.stinger.trigger`, `overlay.stinger.stop` are the new ids). Validation reports
  operator-readable disabled reasons: missing channel, missing/unconfigured stinger input,
  out-of-range cut point. Failed commands leave the project untouched because `execute` works on
  a clone and validates before commit.
- Stinger behavior is a bus exchange, like a Cut: at the cut point the captured Preview input
  becomes Program and the captured Program input becomes Preview, exactly once per trigger, with
  a zero cut point exchanging at the trigger command itself. A re-trigger replaces the in-flight
  stinger; the stinger stays trackable until `StopStinger`, because when a played clip ends is not
  observable without a media layer. `Engine::tick` performs the exchange before transition
  completion, so ordering stays deterministic.
- Rendering paints the eight standard channels only. Stinger configuration and exchange change
  Program routing but no pixels; `display_underneath_overlays` is stored configuration without a
  clip raster. This is deliberately explicit rather than a fake stinger layer.
- UI: the switcher's second overlay row (5–8) is live, the Overlays panel and Overlay Settings
  dialog cover channels 1–8, default F-key bindings are F1–F8 (ToggleOverlay 0–7), and the
  Overlay Settings dialog states that stingers are configured but do not composite a clip yet.
  New automation ids `switcher.overlay.{1..8}` exist for the center buttons.

Verification (local, headless): `cargo fmt --all -- --check` clean,
`cargo clippy --workspace --all-targets --locked -- -D warnings` clean,
`cargo test --workspace` green (engine 56: 13 unit + 7 overlays + 7 categories + 8 inputs +
8 production + 2 transitions + 11 workflow; UI 45), and `scripts/check-repository.ps1` passes.
New engine tests cover channel classification, cut-point timing, zero-cut exchange, re-trigger,
explicit stop, atomic rejection paths, the 4→5 migration with preserved panels and original
rollback bytes, F5 toggling channel 5 and the paint/no-paint split. The reference-coverage guard
now reads the two channel constants instead of the removed fixed-size array.

Remaining gaps: stinger clip compositing, stinger timing raster, per-channel effects/alpha/
transforms, source-based standard channels and any vendor pixel/protocol parity remain
unimplemented and are listed as such in `docs/reference-audit.md` and `docs/feature-matrix.md`.
