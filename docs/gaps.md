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
| G11 | P1 for UI-003/004 | Bounded dialog layout, navigation, footer visibility, monitor edit controls and input states await hosted compilation/widget execution and review of all nine dialog screenshots at 1100×700, 1600×900 and 1920×1080. Logical 734×466 widget coverage is prepared; physical 150% DPI remains unverified. |

Do not claim 1080p/4K processing based on the project-format selector; actual demo rasters are capped and only displayed in the native UI.
