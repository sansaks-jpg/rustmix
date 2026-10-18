# Known gaps

UI-009 audit (2026-10-18): the UI-008 GPU/output labels were not evidence of media backends. Composition/effects are CPU reference code, wgpu presents UI textures, recording/stream/external start paths were counter simulations, and real video decode/audio/hardware output remain absent. UI-009 prevents false live output state and repairs existing transport/composition/layout paths. It does not close G01–G05 or establish production readiness.

2026-10-17 · UI-001. No feature is production-ready.

| ID | Priority | Gap / next action |
|---|---|---|
| G01 | P0 for broadcast | Program is a native UI demo, without a dedicated real-time output scheduler. Introduce immutable frame metadata and an independent Program consumer before live claims. |
| G02 | P0 for broadcast | No actual audio graph/device output. Add 48 kHz synthetic stereo samples, Master/Aux routing and impulse/mix-minus tests. |
| G03 | P1 | No custom GPU compositor or CPU/GPU golden parity. Current CPU demo uploads to egui's wgpu surface. |
| G04 | P1 | UI-010 implements bounded PNG ingest, persistence, relink and source edits (hosted acceptance pending). Local video decode, camera/device capture and network ingest remain absent. |
| G05 | P1 | No encoders, recoverable recording or network streams. Controls remain disabled. |
| G06 | P1 | Crash recovery is manual checkpoint/backup loading; abrupt power-loss durability and filesystem fault injection need dedicated tests. |
| G07 | P2 | Shortcut enable/disable exists; custom key remapping does not. Full per-widget automation metadata, command JSON schemas and accessibility review remain incomplete. |
| G08 | P2 | Minimum 1100×700; dense panels use scrolling. Broader high-DPI/multi-monitor/physical-device QA remains needed. |
| G09 | P2 | FTB/active transitions reset at load. Demo countdown loops every minute; no editable production countdown. |
| G10 | P1 | Unsigned portable prototype only. No installer, SBOM standard, signing, upgrade/rollback or release audit. |
| G11 | Closed for UI-005; P2 residual | Bounded dialog layout, navigation, footer visibility, monitor header controls, input/tile states and the UI-005 main-window fidelity passed hosted compilation/widget execution and native screenshot review at 1100×700, 1600×900 and 1920×1080 (hosted Windows run 39046367904, commit `a19fc03`, all steps green). Logical 734×466 widget coverage is prepared; physical 150% DPI and physical-device screenshot review remain unverified. |
| G12 | Closed in UI-006 | Bounded `Overlay Settings` dialog (`dialog.overlay.*`) implemented for slots 1–4 with staged per-slot edits, live 16:9 normalized preview (`overlay_panel_rect`), and access from the Overlays panel, bottom `Overlay ⚙` button (`output.overlay.settings`), `☰` menu, and `Settings ▼` caret. Full-frame PIP/chroma overlay types and per-slot transition durations remain future engine work. |
| G13 | Closed in UI-006 | Per-input audio routing (`InputPage::Audio`) is now directly reachable from Audio Mixer input strip `⚙` buttons (`audio.input.<id>.settings`), `"Audio Settings…"` on `Input Properties → General` (`dialog.properties.audioSettings`), and source tile context menus. |
| G14 | Closed in UI-006 | Dialog sidebar buttons (`sidebar_button`) now render left-aligned labels with 16×16 colored badges and original procedural `egui::Painter` vector glyphs (`Grid`, `Geometry`, `Colour`, `Title`/`Overlay`, `Workspace`, etc.), eliminating centered-text overlap. |
| G15 | Closed in UI-006 | `egui_dock::Style` now uses broadcast-dark tab bar surfaces, `TEAL` active/focused outlines, and high-contrast active tab typography without altering tab geometry. |
| G16 | Closed in UI-006 | `Multiview` now offers two bounded layout presets (`Split: Monitors + Grid` and `Full Source Grid`) via `multiview.preset.split` and `multiview.preset.grid`. |
| G17 | Closed in UI-006 | All eleven private reference screenshots (`Interface.png`, `Add Input.png`, `Input Settings.png`, `Settings.png`, `Overlay Settings.png`, `External.png`, `Multicoder.png`, `Playlist.png`, `Recording Setup.png`, `Shortcut.png`, `Streaming Setup.png`) were reviewed read-only and documented in [ui-reference.md](ui-reference.md). Unimplemented output/encoder dialogs remain disabled by design. |
| G18 | Closed in UI-006 | Source-tile audio button displays plain `"Audio"` when unmuted and red `"Muted"` (`RED` fill) when muted, so unmuted tiles do not appear falsely armed. |
| G19 | Closed in UI-006 | Top toolbar `Open ▼`, `Save As ▼`, and `Settings ▼` carets now expose real actions (open, reload last saved, recover checkpoint, write immediate recovery checkpoint, and jump directly to individual Settings pages or Overlay Settings). |

Do not claim 1080p/4K processing based on the project-format selector; actual demo rasters are capped and only displayed in the native UI.

UI-009 acceptance: [run 39089212707](https://github.com/sansaks-jpg/rustmix/actions/runs/39089212707), code `3a36cf2`, passed all 32 tests, native layout/pause/switch/save/reopen and review of 14 actual PNG captures. The reported idle-layout drift, vertical T-Bar and advancing paused-source progress are closed for the tested prototype. Real GPU media composition, decoding/capture/audio/output backends and hardware/endurance readiness remain open as detailed in the engine audit.
