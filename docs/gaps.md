# Known gaps

UI-022 outcome:7aeac9c/run39434584384 passes check/Clippy/all tests (121 UI), build and complete native workflows. Exact Settings/Playlist/Overlay captures accept selected dimensions/alignment, not full equivalence. Earlier GPU drain failure remains unexplained;50/59.94/60cadence targets still fail. Header/footer placement/typography and advanced backends remain open. UI023 Recording source is isolated and untested. See ADR0022/ui022-visual-review.md.

UI-021 outcome: be79b2e passes hosted check/Clippy/all tests (115 UI) and build, with corrected Overlay labels/contained preview/full-width Playlist rows verified in exact screenshots. Main native GPU shutdown fails (Wait timeout, Queue::drop panic, exit-1073741819); later workflows do not run. Full parity, Settings proportions, Playlist dimensions, advanced overlays and cadence remain open. See ADR0021 and ui021-visual-review.md.

2026-10-20 UI-020: code bec97a8 passed hosted check/Clippy/all tests (106 UI) and build, with
all main native workspace/dialog captures inspected against supplied references. Main native
shutdown failed its GPU drain proof; later workflow stages did not run. Same-size texture reuse
is not a proven GPU hang fix. Full UI equivalence remains open: Overlay Settings draft/preview,
Settings proportions, Playlist controls, record/stream layout and disabled action affordance.
See ADR 0020 and ui020-visual-review.md for exact evidence and remaining measurements.

Task 1.3 continuation (2026-10-19): four external-output configuration slots and staged editing
are implemented in source, with schema 6 and atomic source/config changes. New regression tests
are prepared, not executed in this session. G05 remains open: configuration is not an SDI,
virtual-camera or network output backend, and External Start stays unavailable. Device port is a
hardware connector placeholder (Default or a numbered connector), not a network port. No new
hardware/cadence qualification is claimed. See [ADR 0014](adr/0014-external-output-configuration.md).

The [complete reference audit](reference-audit.md) enumerates all observed areas, input/effect/
overlay/audio contracts, UI workflows and public function names. Full-match acceptance is false:
major media/audio/output/API backends are absent, and hosted cadence/hardware qualification
does not pass. The Basic-to-Multiview alias is corrected and functionally verified in hosted run 39111788801; complete interface parity is still partial.

UI-012 (2026-10-18), experimental functional acceptance: the fixed-key limitation in G07 is
addressed by persisted remapping, and UI-owned playlist/slot state moves into the headless engine.
Four independent transition slots, twelve CPU effects, named playlists, worker acknowledgment and
Basic/Advanced passed hosted run 39111788801: 75 tests, native capture and save/reopen.
G01 still needs production-clock/output qualification; hosted 50/59.94/60 cadence targets failed.
G02–G05, hardware/API/accessibility/endurance and full parity remain open.
See [reference comparison](reference-feature-comparison.md) for capabilities not implemented here.

Task 1.1 port (2026-10-18): the schema-5 sixteen-channel overlay/stinger bank,
`overlay.stinger.*` commands, exactly-once cut-point Program/Preview exchange and the channels
1-8 UI are implemented and covered by engine/UI tests; hosted native acceptance passed in run
39129117131. Stinger clip compositing, `display_underneath_overlays` rasterisation and
per-channel effects/timing remain open (G20).

Task 1.2 port (2026-10-18): the typed `InputCategory` contract (`All`, six preset colours,
`Custom(1..=16)` clamping, filter-only `Search`) and the `ProjectState::visible_inputs` engine
query are implemented over the existing schema-5 definition ids, with the tile grid filtering
through the engine; local headless verification passed and hosted acceptance for this commit is
pending. No control endpoint exists, so no wire ordinals and no vendor category identity are
claimed (see the HTTP/API row in the reference audit); the text matcher stays in the UI (ADR 0013).

UI-009 audit (2026-10-18): the UI-008 GPU/output labels were not evidence of media backends. Composition/effects are CPU reference code, wgpu presents UI textures, recording/stream/external start paths were counter simulations, and real video decode/audio/hardware output remain absent. UI-009 prevents false live output state and repairs existing transport/composition/layout paths. It does not close G01–G05 or establish production readiness.

2026-10-17 · UI-001. No feature is production-ready.

| ID | Priority | Gap / next action |
|---|---|---|
| G01 | P0 for broadcast | Program is a native UI demo, without a dedicated real-time output scheduler. Introduce immutable frame metadata and an independent Program consumer before live claims. |
| G02 | P0 for broadcast | No actual audio graph/device output. Add 48 kHz synthetic stereo samples, Master/Aux routing and impulse/mix-minus tests. |
| G03 | P1 | No custom GPU compositor or CPU/GPU golden parity. Current CPU demo uploads to egui's wgpu surface. |
| G04 | P1 | UI-010 bounded PNG ingest/persistence/relink/source edits passed hosted acceptance, including run39111788801. Local video decode, camera/device capture and network ingest remain absent. |
| G05 | P1 | No encoders, recoverable recording or network streams. Controls remain disabled. |
| G06 | P1 | Crash recovery is manual checkpoint/backup loading; abrupt power-loss durability and filesystem fault injection need dedicated tests. |
| G07 | P2 | Validated custom key/modifier remapping is implemented and verified in UI-012. Full per-widget automation metadata, command JSON schemas and accessibility review remain incomplete. |
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
| G18 | Closed in UI-006 | Historical UI-006 label adjustment. Current tile control uses Audio with selected color for unmuted simulation; this is not real device audio or live-output status. |
| G19 | Closed in UI-006 | Top toolbar `Open ▼`, `Save As ▼`, and `Settings ▼` carets now expose real actions (open, reload last saved, recover checkpoint, write immediate recovery checkpoint, and jump directly to individual Settings pages or Overlay Settings). |
| G20 | P1 | Task 1.1 stinger channels (9-16) are configuration and Program/Preview bus exchange only: no stinger clip raster, no `display_underneath_overlays` compositing, no per-channel effects/alpha/timing, and no dedicated stinger UI (configured through `overlay.stinger.*` commands). Clip rendering depends on the media layer (G04). |

Do not claim 1080p/4K processing based on the project-format selector; actual demo rasters are capped and only displayed in the native UI.

UI-009 acceptance: [run 39089212707](https://github.com/sansaks-jpg/rustmix/actions/runs/39089212707), code `3a36cf2`, passed all 32 tests, native layout/pause/switch/save/reopen and review of 14 actual PNG captures. The reported idle-layout drift, vertical T-Bar and advancing paused-source progress are closed for the tested prototype. Real GPU media composition, decoding/capture/audio/output backends and hardware/endurance readiness remain open as detailed in the engine audit.
