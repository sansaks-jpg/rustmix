# ADR 0009: engine-owned transition slots, playlists and keyboard bindings

Date: 2026-10-18 · Task: UI-012 · Status: experimental functional acceptance in Windows run 39111788801 (`a01f498`); full reference/performance qualification remains open.

The supplied `rustmix-reference/vmix` project describes four configurable transition slots,
named playlists over input references, per-item timing/effects, manual/loop options and configurable
shortcuts. RustMix previously stored transition names and playlist timers in UI fields. Some names
were unavailable, playlist options had no behavior, and neither configuration survived project IO.

Implement these generic workflows independently in the existing headless engine. Do not import
reference source, binary metadata, assets, protocol emulation, SDKs or dependencies. Consulted
behavioral summaries: `FEATURE-MAP.md` sections 4/13, `SPEC.md` sections 3.4/3.10 and `UI-UX.md`
section 12.9. The local playlist screenshot was inspected read-only and remains outside this repo.
The public [playlist guide](https://hwww.vmix.com/help29/playlist.html) clarifies that Clear Overlays
applies at start; this takes precedence over the local summary's suggestion of clearing every item.

## Decisions

- Four typed slots each own their effect and 100–10000 ms duration. A running take pins both.
  Cut is immediate; Fade, horizontal/vertical Wipe and Slide (each normal/reverse), Barn Door
  and Roller Door use original CPU reference algorithms. Merge/3D/stinger effects are not aliases.
- Store 1–16 named playlists, each containing at most 128 validated input references. Each item
  has an explicit hold of 100 ms to 24 hours and an independent transition. No inferred file
  duration or seek behavior is advertised. Hold begins after the incoming transition completes.
- Start respects Begin from selected; Clear Overlays applies only on start. Next/Previous use
  item transitions; loop wraps only when enabled. Manual mode suppresses automatic advancement.
  A late wake performs at most one automatic advance, with no catch-up queue.
- Stop leaves an in-flight take intact. Explicit manual switching/cancellation stops automation.
  Offline targets stop auto progression with an alarm and leave both buses untouched. Active
  playlists and their input references are protected against removal/editing. Inactive references
  are pruned when a source is removed. A missing PNG still follows the existing black/relink policy.
- Persist keyboard bindings with exact Ctrl/Alt/Shift modifiers and supported logical keys.
  Reject duplicate combinations, missing explicit targets, invalid slots and reserved file keys.
  Default numeric bindings are intentionally positional; custom input bindings use stable IDs.
  Ignore held-key repeats and suppress production shortcuts during text entry, dialogs or UI lock.
- Schema 3 stores slots, playlists and bindings. Schemas 1/2 migrate with idle production defaults
  and preserve their old global transition duration. Original backup bytes stay unchanged.
  Playback, running playlist, FTB and active transition are transient and never auto-resume on load.
- The Program producer returns the engine state associated with each frame and the control-update
  generation. Only explicit commands/media changes submit snapshots. The UI rejects old-generation
  frames and reads producer state before dispatching new actions. Repainting cannot rewind an
  autonomous playlist by repeatedly submitting an old UI copy.

## Verification and limits

Passing tests cover exact effect endpoints/directional pixels, slot pinning, playlist clocks,
loop/manual/selected start, empty/offline/invalid targets, protection/cleanup, atomic invalid edits,
save/migration/reopen, key conflicts/modifiers/repeats, widget editing and bounded dialog footers.
The producer test exercises progression without a UI consumer and generation acknowledgment.
Hosted PNG acceptance now configures a playlist through real widgets, starts/stops it, exercises
a remapped key, captures the dialog and checks production configuration in a second process.

Local verification is restricted to formatting, source/diff inspection and the repository inventory
guard by `docs/testing.md`. Compilation, Clippy, Rust tests, native screenshots and performance
measurements require the authorized hosted Windows workflow. No new production readiness,
hardware support, codec, audio, recording, streaming, full-HD or GPU composition claim is made.
