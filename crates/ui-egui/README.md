# rustmix-ui

UI-022 candidate: measured Settings/Playlist dimensions, right-aligned Overlay labels, native geometry gates and one-pending-callback queue completion diagnostics. Local format/source guards and review pass; hosted build/tests/native visual acceptance pending. No new dependency/schema or runtime capability; GPU/cadence/full parity remain open. See ADR0022.

UI-021: explicit dirty-channel choices, cached isolated engine preview, reserved form labels and full-width Playlist rows implemented. Exact be79b2e screenshots verify selected visual corrections; hosted check/Clippy/all tests including115 UI tests/build pass. Main native reaches stage51 then fails GPU shutdown; later workflows not run. No local compile/test/native execution. Full parity/runtime acceptance remains open; see ADR0021 and ui021-visual-review.md.

UI-020 (2026-10-20): toolbar/monitor/source/mixer proportions, bounded light forms and the
responsive Shortcuts bank were compared using exact-code native PNGs and bounds. Code bec97a8
passed hosted check/Clippy/all tests including106 UI tests and build, but native GPU shutdown
drain failed. Program/Properties/thumbnail/Add textures reuse allocations at unchanged dimensions;
actual texture-delta tests pass, but GPU reliability remains unproven. No local build/test/launch.
See docs/ui020-visual-review.md and ADR 0020; full reference/workflow acceptance remains open.

Current top-button behavior supersedes the manual workflow below: Fullscreen sends
Program directly to a verified extended display. The arrow selects the monitor;
F11 and dragging are unnecessary. Startup never opens Program output. The optional
windowed Program monitor remains available. Physical two-display qualification is
pending; see ADR 0017.

Separate Program monitor candidate: the arrow beside the application's Fullscreen
button now opens a real native Program window. Open it windowed, drag its title bar
to the extended monitor, then select Fullscreen Program output or press F11 in that
window. Escape and its native Close button close only the Program window. The main
application's fullscreen state and Program/Preview selections stay independent.
The window presents the same last validated 640×360 Program texture, including
transitions, overlays and FTB, fitted to 16:9 over black. It emits no audio and is
not an external-device, encoder or network output. A bounded window size preference
is stored outside project JSON; startup never automatically opens an output.

Prepared regressions exercise real menu widgets and deferred viewport callbacks,
Program texture identity, transitions/FTB, child F11/Escape, unsupported embedding,
and preference validation. Hosted `--ui-output-smoke` / `--ui-output-smoke-reopen`
add native child fullscreen/windowed state, GPU screenshot samples against Program,
FTB-black capture, Preview isolation, close/reopen and project save/reopen checks.
Local formatting/source checks only; compilation, tests and native screenshot
review remain pending hosted Windows acceptance. Physical multi-monitor hotplug,
color management, display latency and endurance are not qualified.

Task 1.3 candidate: the External gear opens four independently staged output
configurations. Apply selected output dispatches the atomic
`output.external.apply` engine command and keeps the other drafts; Cancel closes
and discards only unapplied edits. Source, rate, size, device/connector, audio
device/bus, alpha request and audio delay are project configuration only. External
start stays disabled because no device or transport backend exists. Widget coverage
and native `reference-output-settings.png` plus save/reopen assertions are prepared;
they require the hosted Windows workflow and have not been run locally.

Task 1.1 port: the switcher's overlay buttons are a live 4×2 grid
(`switcher.overlay.{1..8}`), the source-tile strip assigns all eight channels, the Overlays panel
and Overlay Settings dialog cover channels 1–8, and default F-keys run F1–F8. Stinger channels are
configured on the engine but composite no clip yet. Hosted native acceptance passed in run
39129117131. See ADR 0012.

Task 1.2 port: the source-tile grid's category filter runs through the engine
`ProjectState::visible_inputs` query over the typed `InputCategory` contract; the text matcher
stays in the UI and the chips, context menu and Settings editor are unchanged. See ADR 0013.

UI-013 source candidate: schema 4 categories/theme, atomic Settings, slot-defined T-Bar,
real Input Search, temporary audio-pane visibility and worker snapshots. Unsupported scaffolds
are explicitly unavailable. Hosted compile/tests/native captures passed in run 39129117131; see ADR 0011.

UI-012: `controls` renders the engine-backed named playlist editor and staged keyboard mapping
settings. Slot menus dispatch persisted effect/duration updates. Playlist Start/Stop/Next/Previous
use engine commands; empty starts are disabled. Exact modifier/repeat/focus guards protect keys.
The Program producer returns state plus control generation; painting consumes feedback before new
actions and never repeatedly submits stale state. Native PNG acceptance now includes real playlist
widget operations, remapped input selection and persisted configuration on reopen. Basic/Advanced
is a separate native UI preference, independent of engine state and multiview. Run 39111788801
(`a01f498`) passed all 39 UI tests, five native processes and required capture/reopen gates.
Basic, workspace, playlist, keyboard and reopened captures were reviewed. Cadence qualification
failed; no new device/output support or complete reference parity is claimed.

Native egui frontend with dockable monitors, inputs, simulated audio and Program overlay editing. See docs/ui-architecture.md. All engine changes use typed commands. File dialogs/IO are off the UI thread. UI-only layout and dialog actions use the coordinator dispatcher.

UI-009 allocates a fixed monitor-bank rectangle and clips child layouts to prevent per-frame height feedback. Input/mixer space is reserved when resizing or loading old panel state. The manual T-Bar is horizontal. Monitor transport widgets dispatch typed engine commands; pictures, countdowns, progress and timecode use the same source clock. Input effects and custom PNGs reach Program before transition/FTB. Unsupported output start paths report unavailable rather than counting fictitious frames/bytes.

The `dialogs.rs` module owns the source library, input properties and preferences pages. Add Input disables unsupported types and invalid source names; General edits are staged until Apply, while the Audio page dispatches simulated routing commands immediately. Settings stages display, appearance, shortcuts and workspace layout changes until Apply. Source deletion uses an explicit confirmation and the engine's selected-input guard. The background switcher is disabled while editing in a dialog.

The opt-in hosted smoke harness exercises native frames, egui input handling, state readback, save/reopen and wgpu screenshots. Run it only on the authorized GitHub build runner. Never use cua-driver for this task.

UI-004 through UI-006 bound all four dialog windows (`Add Input`, `Input Properties`, `Settings`, and `Overlay Settings`) to the viewport with separate navigation/page scrollers, left-aligned procedural category badges, and a fixed footer outside them. `Overlay Settings` stages per-slot edits and previews normalized 16:9 placement via `overlay_panel_rect`. Audio Mixer input strips and `Input Properties → General` provide direct shortcuts to `InputPage::Audio`, and `Multiview` supports `Split: Monitors + Grid` and `Full Source Grid` presets.


UI-010: PNG references and processing parameters belong to engine state. Input Settings keeps a draft/cache until OK; Cancel leaves Program untouched. Source pickers run off the UI thread, one capacity-32 PNG worker decodes bounded images, and stale project/editor results are ignored. Cached rasters fit 640x360, with Contain/Cover/Stretch and alpha over black. Missing PNGs remain visible and can be relinked through Change. Ordered Add Input and duplication preserve source/effect state. Hosted functional PNG acceptance passed, including run 39111788801.

UI-011 measured / cadence qualification failed: the bounded Program worker runs independently of painting, with replaceable state/frame slots and cached active surfaces. Thumbnail refresh is limited to 15 fps. The native profiler measures produced 50/59.94/60 fps cadence at the actual 640x360 reference raster, separately from UI FPS, and the hosted script samples working set/private memory/CPU. Full-HD/GPU/device/encoder/endurance qualification remains open.
