# Known gaps

2026-10-17 · UI-001. No feature is production-ready.

| ID | Priority | Gap / next action |
|---|---|---|
| G01 | P0 for broadcast | Program is a native UI demo, without a dedicated real-time output scheduler. Introduce immutable frame metadata and an independent Program consumer before live claims. |
| G02 | P0 for broadcast | No actual audio graph/device output. Add 48 kHz synthetic stereo samples, Master/Aux routing and impulse/mix-minus tests. |
| G03 | P1 | No custom GPU compositor or CPU/GPU golden parity. Current CPU demo uploads to egui's wgpu surface. |
| G04 | P1 | No file/image/camera inputs or real capture lifecycle. Add bounded still-image ingest first. |
| G05 | P1 | No encoders, recoverable recording or network streams. Controls remain disabled. |
| G06 | P1 | Crash recovery is manual checkpoint/backup loading; abrupt power-loss durability and filesystem fault injection need dedicated tests. |
| G07 | P2 | Shortcut enable/disable exists; custom key remapping does not. Full per-widget automation metadata, command JSON schemas and accessibility review remain incomplete. |
| G08 | P2 | Minimum 1100×700; dense panels use scrolling. Broader high-DPI/multi-monitor/physical-device QA remains needed. |
| G09 | P2 | FTB/active transitions reset at load. Demo countdown loops every minute; no editable production countdown. |
| G10 | P1 | Unsigned portable prototype only. No installer, SBOM standard, signing, upgrade/rollback or release audit. |
| G11 | Closed for UI-005; P2 residual | Bounded dialog layout, navigation, footer visibility, monitor header controls, input/tile states and the UI-005 main-window fidelity passed hosted compilation/widget execution and native screenshot review at 1100×700, 1600×900 and 1920×1080 (hosted Windows run 39046367904, commit `a19fc03`, all steps green). Logical 734×466 widget coverage is prepared; physical 150% DPI and physical-device screenshot review remain unverified. |
| G12 | P1 for UI-006 | No Overlay Settings dialog: slots can only be toggled and reuse the source name. UI-005 evidence is accepted, so add a bounded vMix-style per-slot dialog (slot, enable, content, position, effect/duration presentation) with staged edits, reusing the ADR 0003 window bounds and only existing overlay commands. |
| G13 | P2 | No per-tile audio channel dialog: the tile Audio control only toggles the simulated mute. Expose gain/mute/sends per input in a bounded dialog; keep the simulation disclosure. |
| G14 | P2 | Add Input / Settings dialogs list text rows; the observed references group source categories with icons. Add original category icons and grouping without copying vendor art. |
| G15 | P2 | Dock tabs keep stock egui styling. Closer observed tab treatment is undecided and unmeasured. |
| G16 | P2 | Multiview has one fixed tile layout; add bounded layout presets (grid / single-source) without inventing output claims. |
| G17 | P2 | `Overlay Settings.png` and the not-yet-studied private references `External.png`, `Multicoder.png`, `Playlist.png`, `Recording Setup.png`, `Shortcut.png` and `Streaming Setup.png` still need read-only UI-006 review for generic workflow facts; never commit, copy or trace them. `Playlist.png` was reviewed on 2026-10-17 and confirmed the mixer rail order (`OUTPUTS` left of the bus strips, `INPUTS` right of them), the per-output gear grouping and the tile `×` header control already implemented in UI-005. |
| G18 | P2 | Source-tile "Audio" control shows its selected state whenever the input is unmuted, so every tile looks armed. Present mute state explicitly (plain label when live, muted style plus label when muted) in UI-006. |
| G19 | P2 | Top file/stream toolbar groups have no dropdown menus (the observed workflow shows carets beside Preset/New/Open/Save/Save As/Last/Stream). Add only menus with real actions; do not add decorative carets. |

Do not claim 1080p/4K processing based on the project-format selector; actual demo rasters are capped and only displayed in the native UI.
