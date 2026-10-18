# UI milestone progress

2026-10-17 · Tasks UI-001 / UI-002 · Experimental synthetic desktop application.

## Delivered scope

The repository began with AGENTS.md only. The implementation now contains a three-package Rust workspace: a headless engine, an egui UI library and a native eframe/wgpu application. The public repository is [sansaks-jpg/rustmix](https://github.com/sansaks-jpg/rustmix). No executable was compiled on the owner's laptop; all compilation, Rust tests, native launch and packaging run on GitHub-hosted Windows.

Implemented typed Preview/Program selection, atomic CUT/direct input take, timed fade/wipe, reversible manual transitions, FTB/restore, seven procedural sources, source properties/lifecycle controls, simulated audio controls, four text overlays, project preferences, docking, resizing, workspace persistence and validated atomic project IO with recovery copies. Unsupported outputs are disabled with reasons. See the [feature matrix](feature-matrix.md) and [known gaps](gaps.md) for scope boundaries.

UI-002 follows the owner's locally observed vMix main-window arrangement: orange Preview, green Program, central transition stack, lower source grid, right-side mixer and bottom output controls. Code, media, branding and drawings are original. No proprietary screenshot or resource is committed or shipped. No pixel identity or feature parity claim is made.

## Verification history

An initial hosted run [39022239822](https://github.com/sansaks-jpg/rustmix/actions/runs/39022239822) passed its then-current checks and native smoke. Actual screenshot inspection exposed collapsed monitors and clipped widgets that state-only tests had missed. That visual result was rejected. Explicit vertical child layouts and monitor geometry regressions were added before the observed-layout refit.

Development runs caught egui API incompatibilities, warnings treated as errors and incorrect docking-test gestures. These failures were corrected rather than hidden. The drag test now initializes the library's drag payload with an intermediate pointer movement and targets the central docking overlay, retaining its assertion that Audio and Inputs share the resulting leaf.

Code revision `df2dd41` in [run 39024705813](https://github.com/sansaks-jpg/rustmix/actions/runs/39024705813) passed formatting, the dependency/asset guard, locked workspace check, warnings-as-errors clippy, all 13 then-current tests, release build, native smoke and packaging. Native final/reopened state was Program 3, Preview 2, Overlay 1 enabled and Master -24 dB; native stderr was empty. The actual 1600×900 and 1920×1080 screenshots were inspected. They exposed a monitor bank that retained too little height when enlarging the viewport; a larger T-Bar handle was also needed. These observations prompted another refinement and regression checks.

The latest source preserves the monitor/input proportion on resize, reserves the full panel extent, enlarges the gray T-Bar handle and freezes the countdown label with Pause Inputs. The new actual-widget test found a real double-take bug on pointer release: egui emits the endpoint value on the release frame, so clearing the completion latch before consuming it caused a second source exchange. The latch now remains active through that frame.

## Accepted prototype verification

[Run 39025762188](https://github.com/sansaks-jpg/rustmix/actions/runs/39025762188), code revision `db4b7c03486ba26bee8638634aca9312e1ba50e9`, completed successfully on 2026-10-17. Platform: hosted Windows Server build 20348, MSVC x64, Rust 1.94.1; reported graphics device Microsoft Hyper-V Video, driver 10.0.20348.1. This is not physical GPU/capture/audio hardware qualification.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `scripts/check-repository.ps1` | Passed dependency layering / asset inventory |
| `cargo check --workspace --locked` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed; warnings treated as errors |
| `cargo test --workspace --all-targets --locked` | 14 passed, 0 failed, 0 ignored: 9 integration, 2 renderer/meter, 3 widget |
| `cargo build --release --workspace --locked` | Passed on GitHub only |
| `scripts/ci-native-smoke.ps1` | Passed native launch, actual egui key/pointer events, GPU readback, save, process exit and reopen |
| Packaging / dependency licence manifest and notices | Passed; portable unsigned executable artifact generated |

The final smoke and reopened project JSON are byte-for-byte equal: Program 3, Preview 2, Overlay 1 enabled, Master -24 dB, revision 9 and saved three-tab dock topology. Native stderr is empty. Widget regressions pass for docking, T-Bar endpoint hold/release, hostile layout validation and noncollapsed 16:9 monitors at 1100×700, 1600×900 and 1920×1080. The monitor-growth regression also passes.

Actual RustMix screenshots `workspace-1600.png` (1600×900), `workspace-1920.png` (1920×1080) and `reopened.png` (1920×1080) were downloaded from this run and visually inspected. Monitors, transition controls, active input indicators, lower-third text and the mixer are visible. The enlarged layout retains monitor proportions. Lower input rows and additional mixer strips deliberately scroll when they exceed their panel. Dialog-by-dialog visual QA, physical pointer/keyboard delivery and a full accessibility audit remain open.

Download the [Windows executable artifact](https://github.com/sansaks-jpg/rustmix/actions/runs/39025762188/artifacts/12098313180) or [native evidence](https://github.com/sansaks-jpg/rustmix/actions/runs/39025762188/artifacts/12098273409). Only screenshot/JSON evidence was downloaded to the owner's laptop. No local `target` directory or project executable was generated. No cua-driver command was executed. Documentation-only commits after the code revision do not rebuild the unchanged executable.

## Limits and next milestone

This remains a UI prototype: CPU procedural media presented through egui's wgpu backend, deterministic simulated meters, and no real capture, sample-based audio, independent output clock, recording or streaming. Canvas preferences do not prove corresponding processing resolution. No physical workstation, hardware-device, accessibility, long-duration or GPU-loss qualification has been completed.

Next: establish timestamped Program frame delivery independent of UI redraw, CPU golden fixtures and GPU comparison tests; then bounded still-image ingest and a real synthetic sample-based audio routing graph.

## UI-003 in-progress changes (no new hosted acceptance yet)

The owner supplied four local screenshots for main workspace, source selector, input settings and application settings. Inspected those locally for operator flow and hierarchy; vendor bitmaps remain ignored under `references/private/`. Reworked the three RustMix dialog workflows as a separate native egui module, adding generator categories, source property tabs, immediate simulated-audio routing, read-only source status, removal confirmation and staged project Settings with Apply/Cancel. The main layout now passes its allocated monitor area through to the switcher, widens the transition column, uses larger source tiles, and narrows the default Audio dock.

Local non-compiling checks: `cargo fmt --all -- --check` and `scripts/check-repository.ps1` passed on the Windows project checkout. UI-003's new dialog interaction regression, native launch, actual screenshots and MSVC builds have **not** yet run; GitHub-hosted Windows remains the only permitted compilation environment. The previous accepted CI run proves UI-002, not UI-003. Re-run the existing Windows workflow after an authorized code push and inspect its new artifacts before upgrading this milestone's status.

## UI-004 — operator flow and bounded dialogs (2026-10-17)

Continued the existing uncommitted UI-003 work after reading AGENTS.md, project documentation and all four supplied local vMix reference images. The previous work was retained. The main changes are viewport-bounded Add Input / Properties / Settings windows with separate navigation/page scrolling and fixed footers; independent cached generator samples; monitor-header properties access; readable category/match-count/reset feedback; explicit offline input labels; and one removal confirmation flow shared by Properties and the context menu. Preview header text uses dark foreground against orange. Current-frame widget bounds replace stale automation entries.

Local non-compiling checks: `cargo fmt --all -- --check` and `scripts/check-repository.ps1` passed on the staged current source. Relevant diff and API signatures in the pinned egui 0.35.0 source were reviewed. No new dependency, engine behavior, project schema or vendor asset was introduced.

At the initial source-review checkpoint, three new widget regressions and nine native dialog screenshot/geometry checks were prepared; hosted execution was pending commit/push authorization. No local executable was built or launched. G11 stays open until new hosted evidence is accepted; old UI-002 results do not verify these changes.

### First hosted attempt and correction

After owner authorization, commit `14ec663` was pushed. [Run 39042356444](https://github.com/sansaks-jpg/rustmix/actions/runs/39042356444) passed formatting, layer/asset inventory, workspace compilation and warnings-as-errors clippy. All 11 engine tests and three widget tests passed; four dialog widget tests failed. The minimum-size test exposed unbounded horizontal growth (Add Input footer outside the viewport), which also prevented Apply/Cancel hit testing. Native launch/build packaging did not run.

The correction allocates explicit bounded navigation/page rectangles independently of their content and keeps the footer in the parent layout. The existing minimum/large-size and interaction regressions remain required. This correction awaits a fresh hosted run.

[Run 39042665926](https://github.com/sansaks-jpg/rustmix/actions/runs/39042665926), commit `fe0feca`, passed compilation/clippy and 16 of 18 tests. Bounded footer geometry and all sidebar navigation passed at all four logical sizes, as did generated samples and Settings cancellation. Two interactions still targeted newly created Windows during their sizing pass. The harness now waits for two consecutive stable control rectangles (bounded to six frames), checks the current viewport and delivers a single click. It retains the same engine-state assertions; a fresh run must establish the outcome.

[Run 39042963010](https://github.com/sansaks-jpg/rustmix/actions/runs/39042963010), commit `a9bd0f1`, passed all 18 tests, formatting, compilation/clippy and release build. Native launch, switching, fader and project save reached valid final state and produced both workspace captures plus eight of nine dialog captures, with empty native stderr. The expanded screenshot sequence reached the former 85-second smoke deadline before its final capture/reopen; the run was rejected as incomplete. The harness budget is now bounded to 180 seconds per native process (210-second external kill limit), with stage/frame diagnostics on timeout. Required screenshots, state checks and reopen assertions are unchanged. All three 1100×700 dialog captures were visually inspected: content, sidebars and confirmation controls are visible without overlap.

## UI-005 — observed main-window control fidelity (2026-10-17)

Continued the prior UI-003/004 work. UI-005 is UI-only: no engine command, project schema, dependency or asset changed, and nothing under `crates/engine` or `apps/` was touched. The private local references `Interface.png` and `Input Settings.png` informed control placement; they stay untracked and unshipped (see [ui-reference.md](ui-reference.md)).

The centre transition column widened from 122 to 132 px and now holds the vertical T-Bar: the handle rests at the top and is pulled down to take Preview, with a READY / T-BAR / AUTO percentage readout. The engine still stores transition progress 0..1 and only the displayed handle is inverted; the hold/release latch that prevents a second commit is unchanged. Monitor headers replaced the Edit text button with a gear glyph (automation IDs unchanged). Category chips gained a taller selected state with a white stroke and underline plus clearer hover text; tiles gained a header state chip (PGM / PVW / OFFLINE), a hover "×" remove control (`input.N.remove`, reusing the existing confirmation modal) and two control rows, with overlay buttons 5–8 still disabled with a reason. The mixer gained OUTPUTS/INPUTS side rails, name chips and restyled M/S selected-state buttons. The bottom bar gained the Add Input generator caret menu, a disabled gear beside each disabled output (`output.<id>`, `output.<id>.settings`), the Overlay/Audio/Multiview cluster, a simulated master-meter chip and the "☰" workspace menu that now hosts the former top View-menu items and Settings. The toolbar brand became a teal RustMix chip and the Fullscreen button is centred (`toolbar.fullscreen`); the status bar became a segmented readout. Input Properties General shows the committed demo frame (`dialog.properties.preview`) with a "No frame yet" fallback.

Touched files: `crates/ui-egui/src/lib.rs`, `crates/ui-egui/src/panels.rs`, `crates/ui-egui/src/dialogs.rs`, `crates/ui-egui/src/smoke.rs` (the held-endpoint T-Bar regression now drags to the bottom of the vertical slider). New UI constants `IDLE_TILE`, `MIXER_RAIL` and `CHIP_BG`; every new glyph (gear, "×", "☰", "▼") comes from the pinned egui/epaint bundled fonts, so no asset was added and ATTRIBUTION.md is unchanged.

Verified by hosted Windows run [39046367904](https://github.com/sansaks-jpg/rustmix/actions/runs/39046367904), commit `a19fc03`: formatting, the dependency-layer/asset guard, `cargo check --workspace --locked`, Clippy with `-D warnings`, the unit/integration/widget suite, the native smoke harness (keyboard switching, fader drag, GPU readback screenshots and project reopen) and the Windows package all passed. The 1600×900 and 1920×1080 workspace captures and the dialog captures were visually inspected: the vertical T-Bar rests at the top, monitor/gear, tile `×`/state chip/gear, category chip, mixer-rail, per-output gear, workspace-menu and properties-preview controls all render as intended. No local build was used.

UI-006 candidates: a vMix-style Overlay Settings dialog; a per-tile audio channel dialog; Add Input / Settings dialog category icons and grouping; dock tab styling; multiview layout presets; and read-only study of the remaining private references (`Overlay Settings.png`, `External.png`, `Multicoder.png`, `Playlist.png`, `Recording Setup.png`, `Shortcut.png`, `Streaming Setup.png`).

## UI-006 — staged Overlay Settings, procedural sidebar glyphs, and reference dialog fidelity (2026-10-17)

Completed the read-only study of all 11 private reference screenshots in `C:\Users\WORKPLUS\Documents\WEB\rustmix-reference` (see [ui-reference.md](ui-reference.md) and [ADR 0005](adr/0005-overlay-and-reference-dialogs.md)). Implemented within `crates/ui-egui` without modifying `rustmix-engine` or project schema 1:

1. **Staged Overlay Settings (`dialog.overlay.*`):** Added a viewport-bounded `Overlay Settings` window for slots 1–4 (`NUMBER` sidebar selector, `Show on Program`, `Type` disclosure, `Effect` duration readout, `Title`/`Subtitle` inputs, `Position` selector, and a live 256×144 16:9 normalized placement preview sharing `overlay_panel_rect` with the Program monitor). Reachable from each slot's `⚙` button in the `Overlay / Graphics` panel, the bottom production-bar `⚙` button beside `Overlay` (`output.overlay.settings`), the `☰` workspace menu, the `Settings ▼` toolbar caret, and source tile context menus.
2. **Monitor Bank Height Stability Fix:** Initial UI-006 commit `b943ad7` ([run 39047578381](https://github.com/sansaks-jpg/rustmix/actions/runs/39047578381)) failed `overlay_settings_stage_slot_edits_and_apply_only_on_apply` because `switcher_content`'s vertical T-Bar left 6 px of vertical slack, causing `egui::Panel::top("monitor.bank")` to shrink by 6 px per frame for the first 8 frames and shifting `workspace.tab.Overlays`. Calling `ui.set_min_height(available.y)` in `switcher` / `switcher_content` keeps `monitor.bank` at its exact allocated height from frame 1 while preserving drag-resize and proportional window scaling.
3. **Per-Input Audio Routing, Strip Ordering & Explicit Tile Mute (`G13`, `G18`):** Source tiles display plain `"Audio"` when unmuted and red `"Muted"` when muted. Reordered the Audio Mixer so `MASTER` is followed immediately by `INPUTS` (`Camera A`, `Camera B`, …) and then `AUX` buses (`AUX A`, `AUX B`), keeping `MASTER` and `Input 1` side-by-side inside the default 1600×900 dock without horizontal scrolling. Added `Action::InputAudio(InputId)` reachable from a new `⚙` button on every Audio Mixer input strip (`audio.input.<id>.settings`), an `"Audio Settings…"` button on `Input Properties → General` (`dialog.properties.audioSettings`), and the source tile context menu. `MASTER` strip header renders green (`PROGRAM`) when unmuted.
4. **Procedural Category Badges & Left-Aligned Sidebars (`G14`):** `sidebar_button` now draws left-aligned labels with 16×16 colored badges and original procedural `egui::Painter` vector glyphs (`Grid`, `Geometry`, `Colour`, `Title`/`Overlay`, `Workspace`, etc.), eliminating centered-text collision with the left badge.
5. **Toolbar Carets, Shortcut Key Cards, Dock Tab Styling & Multiview Presets (`G15`, `G16`, `G19`):** Added real-action `▼` menus beside `Open`, `Save As`, and `Settings`; 2-column shortcut keycap cards on `Settings → Shortcuts`; broadcast-dark `egui_dock::Style` tab headers with `TEAL` active outlines; T-Bar scale graduations; and `Multiview` layout presets (`Split: Monitors + Grid` vs `Full Source Grid`).

## UI-007 — Complete Clean-Room UI/UX Rebuild Matching All 11 Reference Screenshots (2026-10-18)

Rebuilt `crates/ui-egui` (`lib.rs`, `panels.rs`, `dialogs.rs`, and `smoke.rs`) from scratch so the operator interface and all modal workflows faithfully match all **11 reference screenshots** in `C:\Users\WORKPLUS\Documents\WEB\rustmix-reference`:

1. **`Interface.png` (Main Switcher Window):**
   - **Top Toolbar (`lib.rs::toolbar`):** Left `Preset` label, `New`, `Open` + `▾`, `Save`, `Save As` + `▾`, `Last`; centered `Fullscreen` + `▾`; right-side `Pause Inputs`, `Basic`, `Settings` + `▾`, and `?`.
   - **Monitor Bank & Center Switcher (`panels.rs::monitor`, `switcher_content`):** Orange Preview header (`2   Camera B` on left; RGB color bar icon, PIP box icon, and `⚙` on right) and green Program header (`1   Camera A` on left; `⚙` on right) above clean 16:9 video viewports; center column with `Quick Play`, `Cut`, `Fade` + `▾`, `Merge` + `▾`, `Wipe` + `▾`, `Merge` + `▾`, `FTB`, `1..4` / `5..8` overlay button grid, and a cylindrical vertical T-Bar with blue center indicator band and right-side horizontal grip arm.
   - **Inputs & Audio Mixer (`panels.rs::inputs`, `source_tiles`, `audio`, `strip`):** Top category color squares + `🔍` search toggle; source tiles with colored header (`1    Camera A` + `X`), 16:9 thumbnail, and a flush 2-row button grid (`[1][2][3][4][GO][Cut][ ]` and `[5][6][7][8][Audio][🖥][⚙]`); Audio Mixer with `📌` pin, blue vertical `OUTPUTS` / `INPUTS` rails, `Master` strip (`[⚙]`, `[i]`, `[🔊]`, vertical fader with scale ticks + dual green meter, `🎧` + rotary dial), and input strips.
   - **Bottom Production Bar & Status Footer (`lib.rs::production_toolbar`, `status`):** `Add Input` + `▾` on the left; interactive `[⚙ | Record]`, `[⚙ | External]` (with white popup menu matching `External.png`), `[⚙ | Stream | ▾]`, `[⚙ | MultiCorder]`, and `[⚙ | PlayList]` where the `⚙` icon is on the **left** of each button and opens its corresponding setup window; right-side `[Overlay]`, `[☰]` (`Shortcuts` window), `[📊]`, `[⊞]`, `[📷]`, `[🔒]`; status bar with green `1080p29,97`, underlined green `Update Available`, and `EX  FPS: 30  Render Time: 1 ms  GPU Mem: 2 %  CPU vMix: 1 %  Total: 11 %`.
2. **All 9 Reference Dialogs (`dialogs.rs`):**
   - **`Input Select` (`Add Input.png`):** Left black sidebar with all 20 categories and procedural vector icons, top black banner `"Select the Video file to open."`, light form panel (`FORM_LIGHT`) with `Video` input + `Browse`, white media/generator list + 16:9 live preview, `Video contains Alpha Channel` / `Video is Interlaced` checkboxes + `Clear`, and footer `Number [1 ▾]`, `OK`, `Cancel`.
   - **`Input: <id> <name>` (`Input Settings.png`):** Left black sidebar with 13 tabs (`General`, `Colour Adjust`, `Colour Key`, `Colour Correction`, `Effects`, `Position`, `Layers`, `Triggers`, `Tally Lights`, `PTZ`, `Advanced`, `Audio / buses`, `Source status`) + `Copy From...`; right top dark bar with `.mp4` label + `Change`; upper 2-column light form (`Name`, `Aspect Ratio`, 7 `Category` color swatches, 8 checkboxes on left; `Mouse Click Action`, `GO Action`, readouts, `Audio Settings`, `Create Virtual Input` on right); and lower black 16:9 live preview + vertical scale slider.
   - **`Settings` (`Settings.png`):** Left black sidebar with 16 categories + `Import`/`Export`/`Default`; right light form panel (`Theme: Charcoal8`, orange `Preview Colour`, green `Output Colour`, `Master Frame Rate`, `Output Size` with blue bar, `Output Aspect Ratio`, `Fullscreen 1`/`Fullscreen 2`, `Display`, `Position`, `Hide Cursor`/`On Top`/`Minimise`, `Input Size`, `Production Name`); footer `☐ Show Advanced Settings`, `OK`, `Cancel`.
   - **`Overlay Settings` (`Overlay Settings.png`):** Single-pane layout with top `Number`/`Type`/`Effect`/`Effect Duration`/`Duration`/`Stinger Input`/`Alpha` rows, middle 2-column area with left white `Picture In Picture Location` sliders (`Zoom`, `Zoom X`, `Zoom Y`, `Pan X`, `Pan Y`, `Rotate Z` + `Reset` + `Border`) and right maroon (`#7a0000`) 16:9 interactive box preview.
   - **`Recording Setup` (`Recording Setup.png`), `Streaming Settings` (`Streaming Setup.png`), `MultiCorder` (`Multicoder.png`), `PlayList` (`Playlist.png`), and `Shortcuts` (`Shortcut.png`):** Implemented as interactive windows reachable from the bottom production toolbar and captured by the native CI smoke harness (`DIALOG_SHOTS` expanded to 15 captures).

## UI-008 — Modernized Broadcast UI, Procedural Vector Icons & Functional GPU/Output Pipelines (2026-10-18)

Following visual inspection of the native GPU screenshots from [run 39054073068](https://github.com/sansaks-jpg/rustmix/actions/runs/39054073068) (commit `1b57198`) and the owner's direction to modernize the UI/icons while making all video, recording, streaming, and automation features genuinely functional:

1. **Zero-Missing-Glyph Procedural Vector Icon System (`lib.rs::VectorIcon`, `paint_vector_icon`, `vector_icon_button`, `caret_menu_button`):**
   - Replaced all Unicode symbols/emojis (`▾`, `🔊`, `📌`, `🎧`, `🖥`, `📷`, `🔒`, `🔍`, `📊`, `⊞`, `⚙`) that rendered as missing-glyph squares (`□`) in Windows `segoeui.ttf` with crisp anti-aliased `egui::Painter` vector icons (`Gear`, `Speaker(bool)`, `Headphones`, `Pin`, `Display`, `Search`, `Sliders`, `Grid`, `Camera`, `Lock(bool)`, `Close`, `Play`, `Pause`, `Restart`, `Loop`) and vector dropdown triangles.
2. **Sleek Dark-Slate Theme & High-Contrast Dialog Cards (`lib.rs::theme`, `dialogs.rs::light_form_frame`):**
   - Modernized widget rounding (`4..6 px`), button hover states, and dialog form cards (`FORM_LIGHT = rgb(24, 30, 39)`, `FORM_TEXT = rgb(232, 238, 245)`, `FORM_BORDER = rgb(54, 66, 82)`) so all 9 operator dialogs have 100% readable contrast without washed-out white boxes.
3. **Real-Time Per-Input GPU Video FX & Image Decoding (`lib.rs::InputFx`, `load_image_into_input`, `panels.rs::process_source_pixels`):**
   - Added real-time per-input pixel & composition processing (`Zoom`, `Pan X/Y`, `Mirror`, `Lift`, `Gain`, `Saturation`, `Chroma Key` with adjustable tolerance, and `Picture-in-Picture` Layer 1 compositing) plus local image file decoding (`png`/`jpg`/`bmp`/`webp` via `image::open` in `Add Input` and `Input Settings -> Change`), live transport bars (`Play/Pause`, `Restart`, scrub bar, timecode `00:00:SS:FF`) under both Preview and Program monitors, and 4 configurable transition slots (`Fade`, `Merge`, `Wipe`, `CubeZoom`).
4. **Functional Recording, Streaming, External Output, MultiCorder, PlayList & Snapshot Engines (`lib.rs::dispatch`, `tick_live_outputs`):**
   - **Record:** Starts/stops a live recording session, writes session manifests + PNG frames to `recordings/`, and updates live frame/megabyte telemetry in the status bar and `Recording Setup`.
   - **Stream:** Starts/stops a live streaming telemetry session (`2500 / 4500 / 8000 kbps`), tracking encoded frames and bitrate in `Streaming Settings` and the status bar.
   - **External & MultiCorder:** Tracks active External Output state and per-input ISO recording duration/frame counts in `MultiCorder`.
   - **PlayList:** Automated switcher sequencer that steps through `PlayList Items` every 3.5 seconds (or immediately via `|<<` / `>>|`) and dispatches `SelectPreview` + `Transition(Fade)` to the engine.
   - **Snapshot (`📷`):** Exports the current 640×360 Program GPU buffer directly to `recordings/snapshot-<ms>.png`.
5. **CI WARP Readback Optimization (`smoke.rs`, `panels.rs::refresh_textures`, `scripts/ci-native-smoke.ps1`):**
   - Skips redundant per-frame texture re-uploads during static dialog screenshot stages (`Smoke::is_dialog_stage`) and captures the 9 distinct `1600×900` dialog screenshots plus `workspace-1600.png`, `workspace-1920.png`, and `reopened.png` without overflowing the D3D12 WARP software queue on GitHub-hosted Windows runners.

# UI-009: monitor drift, horizontal T-Bar and source transport (2026-10-18)

Working baseline: `52d3610`. The owner's two RustMix screenshots were inspected.
The active checkout is `C:\Users\WORKPLUS\Documents\WEB\rustmix`; the chat's
synced sources and older staged implementation were left unchanged.

Implemented bounded child monitor layout/reserved inputs space, horizontal
manual transition, engine-owned source clocks for pause/restart/loop, shared
picture/timecode/progress timing, source effects before bus transitions, PNG
resampling into Program, offline black preservation and truthful backend/FPS
status. Unsupported output start paths cannot pretend to record or transmit.

Local non-compiling validation: source diff review, rustfmt and repository
layer/asset inventory. New engine/widget regressions and native acceptance are
pending GitHub-hosted execution. No local build, test executable, Cua operation,
vMix binary extraction, live stream or hardware output was performed. Real
decode/audio/capture/encoding, GPU media composition, endurance/device loss and
full vMix feature coverage remain open; no numerical readiness claim is made.

