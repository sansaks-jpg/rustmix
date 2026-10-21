# rustmix-ui

UI-035: `output_dialogs` owns destination/format/status inspection;
`settings_pages` owns specific Settings summaries. Input processing resets only
the current draft group. Test-only `ui_review` captures actual egui meshes and
textures without a native app build. See [UI-035](../../docs/ui035-reference-uiux.md)
and ADR 0035 for scope and results.

Final local UI-035 pass: all 177 UI tests pass with 94 actual-egui software
PNG/geometry pairs inspected; Clippy/format/inventory pass. Across core/platform
retries with the generated fixture, 132 more tests pass; one existing optional
release benchmark stays unrun. The initial filesystem/fixture/history failures
and exact commands remain in the UI-035 ledger. No application .exe or native
GPU/DPI acceptance is claimed.

UI-032 (2026-10-21), source candidate: `add_input` owns explicit source-page
presentation, separate PNG drafts, category retirement, original semantic icons
and engine/readiness queries. `add_smoke` prepares six mandatory offline native
page captures. Eight new regressions are written; hosted compile/Clippy/test/
native acceptance is pending. No engine/platform/schema/dependency changes.
See [UI-032](../../docs/ui032-add-input-changes.md) and ADR0032.

UI-028 through UI-031 (2026-10-21): Video-first Input Select with genuine bounded
recent files and validation; topmost Escape/popup/repeat safety; responsive
default widths preserving real resizing; Input Settings expanding fields,
aligned labels/flags/colour/fit/metadata and left source caption.
Source 56048cb/run 39556368477: locked check/Clippy, 274 tests (150 UI), release, all15 native processes and packaging pass.
Changes stay in ui-egui; no dependency, backend, project/dock
schema or engine-command change. Widget paint/key/edge-drag and native source
workflows verified; exact selected PNG/JSON inspected. Only formatting/diff/
inventory/script checks locally, no local Rust compile/test/build/launch.
Full UI/workflow correspondence, original source-icon semantics, remaining
source/property/preview dimensions, GPU reliability, cadence and physical/
DPI/endurance qualification remain open. Light theme is deprioritized.
See ADRs0028-0031 and UI-028/029/030/031 reviews for exact scope and every failed candidate.

UI-024 through UI-027: shared original dialog captions/fixed footers, aligned
Display form, fitted top-aligned monitor pictures/transport, initial 85/15 Production
dock, retained Add Input categories and real quick media routes. Unavailable output
actions use neutral disabled controls; Close remains usable. Existing typed commands,
saved layouts and Program/Preview isolation are retained; no dependency/schema/backend
change. ba3b209/run 39540613350 passes 261 tests (137 UI), check/Clippy/build and all
13 native workflows; exact images/JSON reviewed. Only formatting/diff/inventory/script
checks ran locally. Light theme is deprioritized; full correspondence, GPU reliability,
cadence and hardware/endurance remain open. See docs/ui027-native-acceptance.md.

UI-023: flat controls, left monitor titles, centered/bounded effect controls and the
selected-format Recording page with 12 px form gutter are tested for source 126291d.
Hosted run 39452377970 passes check/Clippy/all 251 tests (127 UI), release build and 13
native processes; exact workspace/Recording PNGs and geometry inspected. Start remains
disabled and configuration unsaved. No local compile/test/native run, dependency,
schema or engine change. Full visual equivalence, Light icon contrast and the hosted
slowdown remain open. See ADR 0023 and docs/ui023-visual-review.md.

UI-022: measured Settings/Playlist dimensions, right-aligned Overlay glyphs, native geometry gates and one-pending-callback diagnostics verified in7aeac9c/run39434584384. All121 UI tests/other suites/check/Clippy/build and13 native processes pass; exact screenshots independently reviewed. Full UI parity, cadence/endurance and intermittent GPU failure remain open. No local compile/test/native run. UI023 candidate remains isolated; see ADR0022/ui022-visual-review.md.

UI-021: explicit dirty-channel choices, cached isolated engine preview, reserved form labels and full-width Playlist rows implemented. Exact be79b2e screenshots verify selected visual corrections; hosted check/Clippy/all tests including115 UI tests/build pass. Main native reaches stage51 then fails GPU shutdown; later workflows not run. No local compile/test/native execution. Full parity/runtime acceptance remains open; see ADR0021 and ui021-visual-review.md.

UI-020 (2026-10-20): toolbar/monitor/source/mixer proportions, bounded light forms and the
responsive Shortcuts bank were compared using exact-code native PNGs and bounds. Code bec97a8
passed hosted check/Clippy/all tests including106 UI tests and build, but native GPU shutdown
drain failed. Program/Properties/thumbnail/Add textures reuse allocations at unchanged dimensions;
actual texture-delta tests pass, but GPU reliability remains unproven. No local build/test/launch.
See docs/ui020-visual-review.md and ADR 0020; full reference/workflow acceptance remains open.

Current top-button behavior supersedes the manual workflow below: Fullscreen routes
Program directly to the selected extended display. The arrow selects the monitor;
F11 and dragging are unnecessary. Startup never opens Program output. The optional
windowed Program monitor remains available. Physical two-display qualification is
pending; see ADR 0017.

Separate Program monitor (native acceptance passed in run 39270445070): the arrow beside the
application's Fullscreen button opens a real native Program window presenting the same last
validated 640×360 Program texture, including transitions, overlays and FTB, fitted to 16:9 over
black. The top Fullscreen button routes Program directly to the selected extended display; Escape and
the window's native Close button close only the Program window, and the main application's
fullscreen state and Program/Preview selections stay independent. The window itself emits no audio
and is not an external-device, encoder or network output; local audio monitoring is a separate,
explicitly started session. Physical two-display/hotplug/color/latency qualification is pending;
see ADR 0017. Deferred Program screenshots are sequence-tagged so stale readbacks cannot satisfy a
later acceptance request.

Regressions exercise real menu widgets and deferred viewport callbacks, Program texture identity,
transitions/FTB, child F11/Escape, unsupported embedding and preference validation. Hosted
`--ui-output-smoke` / `--ui-output-smoke-reopen` add native child fullscreen/windowed state, GPU
screenshot samples against Program, FTB-black capture, Preview isolation, close/reopen and project
save/reopen checks; run 39270445070 passed them. Run 39272249323 validated stale-readback rejection
and the bounded native-smoke stage guards. Physical multi-monitor hotplug, color management, display
latency and endurance are not qualified.

2026-10-19 overlay captions: the operator monitor no longer paints its own caption text. Enabled
overlay titles and subtitles are rasterized into the Program surface by the engine compositor
(`engine::overlay_text`, ADR 0019), so the monitor, the separate Program window, fullscreen output
and captures show identical words. The live stage of `--ui-output-smoke` sets deterministic caption
text, maps every title/subtitle ink pixel of the Program raster into the captured Program viewport
and requires it to be bright there, writing the counts into `program-output-live.json`; hosted
acceptance is pending for this commit. Non-claims: bitmap typography, ASCII-only glyphs, no
styling/animation/ticker/data binding, no per-source overlay and no GPU/hardware/encoder
qualification.

2026-10-19 local MP4 video: `video_player.rs` spawns one bounded owner decoder thread per committed
video input (global maximum eight live workers) and publishes immutable frames at exact 100 ns PTS
into the engine `VideoFeed`; pause/loop/seek/restart follow the engine transport clock, and the UI
thread only snapshots feeds into the existing decoded-media cache. `video_controls.rs` appends the
`Video file` Add Input category (index 20 of 21), validates the MP4 on a bounded probe worker and
commits the descriptor only after that probe. `video_smoke.rs` provides `--ui-video-smoke` /
`--ui-video-smoke-reopen`: real dialog add, probe format 160×90 at 30/1, Program quadrant pixels,
paused-picture freeze/resume motion, three 1600×900 GPU captures, schema-8 save and second-process
decode. Run 39273839106 passed all steps. Non-claims: video-only MP4 (the audio track is not
decoded), no A/V synchronization, cameras/capture cards/network/browser inputs absent, no GPU or
hardware decode, no frame-accurate seek UI, no native relink acceptance stage (relinking clears the
previous file's decoded frame and is unit-tested), and only the generated 160×90 fixture is
qualified.

2026-10-19 audio session: `audio_controls.rs` owns the Audio I/O operator dialog and
`audio_session.rs` runs an explicitly started local monitoring session (WAV clips and configured
microphones through the 48 kHz mixer and WASAPI output), reporting actual Master block peak/RMS and
counted underruns/drops; the main mixer's animated meters remain simulations. `audio_smoke.rs`
provides `--ui-audio-smoke` / `--ui-audio-smoke-reopen`, which validate a generated one-second
44.1 kHz mono PCM16 WAV through the add dialog, assert monitoring stays off, save and reopen; run
39270445070 passed. No device/callback/hardware acceptance, compressed audio, mix-minus hardware
test or A/V synchronization is claimed.

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

Native egui frontend with dockable monitors, inputs, audio controls with an explicitly started
monitoring session, Program overlay editing and an experimental local MP4 video path. See docs/ui-architecture.md. All engine changes use typed commands. File dialogs/IO are off the UI thread. UI-only layout and dialog actions use the coordinator dispatcher.

UI-009 allocates a fixed monitor-bank rectangle and clips child layouts to prevent per-frame height feedback. Input/mixer space is reserved when resizing or loading old panel state. The manual T-Bar is horizontal. Monitor transport widgets dispatch typed engine commands; pictures, countdowns, progress and timecode use the same source clock. Input effects and custom PNGs reach Program before transition/FTB. Unsupported output start paths report unavailable rather than counting fictitious frames/bytes.

The `dialogs.rs` module owns the source library, input properties and preferences pages. Add Input disables unsupported types and invalid source names; General edits are staged until Apply, while the Audio page dispatches routing commands immediately. Settings stages display, appearance, shortcuts and workspace layout changes until Apply. Source deletion uses an explicit confirmation and the engine's selected-input guard. The background switcher is disabled while editing in a dialog.

The opt-in hosted smoke harness exercises native frames, egui input handling, state readback, save/reopen and wgpu screenshots. Run it only on the authorized GitHub build runner. Never use cua-driver for this task.

UI-004 through UI-006 bound all four dialog windows (`Add Input`, `Input Properties`, `Settings`, and `Overlay Settings`) to the viewport with separate navigation/page scrollers, left-aligned procedural category badges, and a fixed footer outside them. `Overlay Settings` stages per-slot edits and previews normalized 16:9 placement via `overlay_panel_rect`. Audio Mixer input strips and `Input Properties → General` provide direct shortcuts to `InputPage::Audio`, and `Multiview` supports `Split: Monitors + Grid` and `Full Source Grid` presets.


UI-010: PNG references and processing parameters belong to engine state. Input Settings keeps a draft/cache until OK; Cancel leaves Program untouched. Source pickers run off the UI thread, one capacity-32 PNG worker decodes bounded images, and stale project/editor results are ignored. Cached rasters fit 640x360, with Contain/Cover/Stretch and alpha over black. Missing PNGs remain visible and can be relinked through Change. Ordered Add Input and duplication preserve source/effect state. Hosted functional PNG acceptance passed, including run 39111788801.

UI-011 measured / cadence qualification failed: the bounded Program worker runs independently of painting, with replaceable state/frame slots and cached active surfaces. Thumbnail refresh is limited to 15 fps. The native profiler measures produced 50/59.94/60 fps cadence at the actual 640x360 reference raster, separately from UI FPS, and the hosted script samples working set/private memory/CPU. Full-HD/GPU/device/encoder/endurance qualification remains open.
UI-034 candidate (2026-10-21): New Preset appears on normal startup/New, with a
staged landscape/portrait/rate/scan form and explicit unsaved replacement policy.
Portrait Program/Preview/tiles/output-window/snapshots preserve aspect at the same
bounded reference pixel count. Camera uses actual mode discovery; Browser exposes
portrait viewport presets, two owner permits and fixed length-delimited discovery.
`--ui-preset-smoke` is a mandatory new hosted process. No local compile/native run,
full-HD/interlaced output, 60fps or physical-device qualification is claimed.
See [UI-034](../../docs/ui034-add-input-and-new-preset.md).
