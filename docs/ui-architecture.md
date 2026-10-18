# Native UI architecture

`apps/rustmix` hosts eframe's native event loop and wgpu renderer. `crates/ui-egui` renders the interface and coordinates file jobs. `crates/engine` owns all switcher state; it cannot import egui, wgpu, dialogs or windows.

```text
Widget / keyboard → UI Action → Command → enabled + candidate validation
                                              ↓
                                     Engine commit + revision
                                              ↓
              CPU procedural frames ← project + pinned transition + monotonic time
                        ↓
               egui textures → native wgpu surface
```

`Engine::execute` handles typed commands and `Command::definition` supplies stable version-1 IDs, label/category, shortcut, local-demo scope and live/idempotent flags. `enabled` returns a human-readable reason. Results include command, new revision and affected input IDs. Domain commands are reusable headlessly; no HTTP/MCP/network endpoint is exposed. An authenticated transport and full machine-readable argument schemas are future work.

UI-only actions (open dialogs, fullscreen, layout) are handled by one dispatcher in the UI coordinator. Source properties and audio strip edits clone typed values and request engine validation. Unavailable output controls are disabled with explanations. Actual state, rather than button activity, drives monitor and source labels.

UI-002 has a resizable monitor bank above DockState. DockState holds three tabs: Inputs, Audio and Overlays. Inputs/Overlays initially share the lower-left region; Audio occupies the lower-right region. Panel tabs drag between split areas or floating egui windows. Production / Audio focus reset presets. Project saves include the serialized layout; eframe separately saves window geometry and its panel snapshot. Imported layouts are checked for bounds, valid tree children, active indices and exactly one of each of the three dock panels before use. Invalid layout falls back visibly without replacing engine source identities.

Demo source IDs remain stable across rename/save/reopen. Program/Preview are different state fields. Input changes cannot select Program implicitly. Remove rejects currently selected IDs. Offline state retains identity/configuration and renders black. Renderer failures retain the last valid monitor texture and set a warning.

Audio simulation is a pure function of time and channel ID. Input mute gates every send, input solo excludes nonsolo inputs, bus gains/mute/solo gate bus meters. It is not an audio callback, sample mixer, limiter, compliant loudness meter or audio-follow-video implementation.

Project IO now uses schema4 JSON with schema1/2/3 migration (UI-013), a 1 MiB bound, finite numeric validation, 2–32 inputs, short strings and known formats/rates. Save writes/syncs a same-directory tempfile, validates a JSON roundtrip, journals and atomically replaces. Two previous generations are retained. Invalid existing files are not overwritten. Paths stay out of operator logs. Bounded background jobs retain the current project on errors.

Automation: command buttons, source thumbnail selection, overlay toggles and audio faders publish stable IDs/bounds to an internal registry. The opt-in `--ui-smoke` native harness injects egui events, checks engine state and requests real GPU screenshots. JSON exposes the active revision and resulting source IDs. There is no general-purpose remote UI endpoint yet; full per-widget coverage remains a gap.

Measured UI callback FPS and CPU image generation time are distinct from output-clock statistics. Canvas preferences do not change the capped monitor raster. Real-time video/audio scheduling, a custom GPU compositor and device-loss continuity are not implemented.

The main layout follows observed generic broadcast placement from the owner's installed vMix 29, with original RustMix controls/media and native Windows system typography. Four lower-third slots work; per-input number buttons set the source name as a title. Slots 5–8 and unsupported output/Merge features are unavailable. Named engine-owned playlists and four saved transition slots are implemented; see ADR 0009. Direct thumbnail Cut is an atomic engine command and rejects taking the already active Program source. See ADR 0002.

The monitor bank gives switcher widgets the panel's actual available height (without consuming it in advance). Its persisted egui PanelState height scales proportionally on viewport resize, retaining the operator's chosen monitor/input proportion within bounded minimum and maximum heights. T-Bar completion remains latched through the pointer-release frame, because egui can emit a final changed slider value on release. Otherwise an endpoint held after completion could take Preview a second time.

UI-003 (historical design, subsequently covered by hosted acceptance) adds distinct Add Input / Input Properties / Settings windows with native sidebar navigation. Generator selection displays representative synthetic source frames; unsupported source types remain visibly disabled. Source General edits stage until Apply; the audio page stages simulated routing until Apply and Cancel discards the draft. Properties shows live source status and requires confirmation for removal. Settings stages `Preferences`, project-name and layout-preset changes until Apply; checkpoint recovery remains an immediate, separately confirmed action. Background switching controls and shortcuts are disabled while a dialog is open. Button IDs `dialog.add.confirm`, `dialog.properties.apply`, and `dialog.settings.apply` support synthetic widget tests. Original local vendor screenshots were only viewed in gitignored `references/private/`, never incorporated in distributable media.

## UI-004 operator refinements (source only)

The existing dialogs use a shared viewport-bounded Window builder. Navigation and page content each own a bounded ScrollArea; Apply/Cancel remain sibling controls below both. A single cached 320×180 sample texture is generated on source-type changes and does not depend on a matching source in the project. No sample is registered in the engine or sent to Preview/Program.

Monitor header Edit actions open the current input's properties. Source filters expose category and match count alongside the color strip; offline thumbnails show explicit text and black fallback. Both Properties and the source context menu dispatch RequestRemove, which checks engine eligibility and opens a blocking egui Modal before removal. Engine commands and project schema are unchanged. Automation rectangles are rebuilt every frame rather than retaining hidden controls. See ADR 0003; compile, widget execution and native render evidence remain pending.

## UI-005 main-window fidelity (source only)

The centre transition column widened to 132 px. Its T-Bar is a vertical egui slider whose displayed handle is `1.0 − progress`: the handle rests at the top when engine progress is 0 and reaches the bottom endpoint at progress 1, so the operator pulls it down to take Preview. The engine keeps its 0..1 manual progress and unchanged `SetManualProgress` validation; inversion is a display concern, and changing the domain semantics for a visual preference would ripple through tested transition rules for no operator-visible benefit. The readout below the handle shows READY / T-BAR / AUTO with the percentage, and the existing completion latch still spans the pointer-release frame because egui can emit the final slider value on release.

Monitor headers now use a compact gear glyph instead of the Edit text button; `monitor.preview.properties` and `monitor.program.properties` are unchanged. Category chips render the selected state as a taller chip with a 1.5 px white stroke and a white underline bar, and the compact "All" button is a selected-state button with filter/clear hover text.

Each source tile header carries the source number and name, a PGM / PVW / OFFLINE state chip and a hover "×" remove control (`input.<n>.remove`) that dispatches the same `RequestRemove` confirmation as the context menu. Tile control row 1 is overlay buttons 1–4 plus PVW and Cut; row 2 is overlay buttons 5–8 (disabled with a reason) plus Audio and the properties gear (`input.<n>.properties` unchanged). The tile keeps its context menu. Vertical spacing inside the tile was tightened and the tile height budget increased so both rows fit without clipping.

The audio mixer separates its OUTPUTS and INPUTS groups with 20 px blue side rails whose rotated labels are drawn through epaint `TextShape`; the artwork and text are RustMix's own. Strips have a filled name chip and restyled M/S selected-state buttons, and `audio.<name>.gain` remains the automation ID. In the bottom production bar, each disabled output (Record, External, Stream, MultiCorder, PlayList) now owns a disabled gear button with an explicit reason and publishes `output.<id>` and `output.<id>.settings`. The right cluster holds the Overlay, Audio and Multiview focus buttons, a small simulated master-meter chip whose hover text states it is a simulated signal with no audio device, and the "☰" workspace menu. That menu now holds the layout presets, Multiview, the panel focuses and Settings, so the toolbar has no View menu. The toolbar keeps the brand chip (About), New/Open/Save/Save As/Last with separators at 24 px heights, a centred Fullscreen button (`toolbar.fullscreen`), the SYNTHETIC/OFFLINE chip, the Pause Inputs selected-state button, Settings (`toolbar.settings`) and "?" (About). The status bar is a segmented readout — project target height/rate (integer rates printed without decimals), UI FPS, demo CPU time, input count, engine revision and the notice, separated by rules — and each segment's hover text distinguishes demo measurements from encoder or broadcast performance. Input Properties General shows the current committed demo frame under `dialog.properties.preview`, with a black "No frame yet" placeholder when no texture exists.

No new asset was added: the gear, "×", "☰" and "▼" glyphs come from the pinned egui/epaint bundled fonts, so ATTRIBUTION.md is unchanged. Verified by hosted Windows run 39046367904, commit `a19fc03`, whose `cargo check`, Clippy `-D warnings`, widget tests and native screenshots cover this structure; no local build was used.

## UI-006 Overlay Settings, procedural badges and layout stability

`Overlay Settings` (`dialog.overlay.*`) reuses the viewport-bounded `operator_window` and two-column `dialog_columns` layout from ADR 0003. `RustMixApp::overlay_settings` holds `Option<(usize, OverlayState)>`, staging edits to a single overlay slot until `Apply` dispatches `Command::SetOverlay(slot, draft)`. Switching slots in the left `NUMBER` sidebar reloads the newly selected slot from committed engine state, and `Cancel` or window close discards the draft. Both the Program monitor and the dialog's 256×144 16:9 preview call `overlay_panel_rect(frame, position, slot)` so preview geometry matches the CPU compositor (`56% × 9.5%` panel, `10.5%` vertical slot stride).

The UI-009 monitor bank allocates/clips a bounded child rectangle to avoid persisted-size feedback; only the bounded switcher content consumes that height. PanelState scales proportionally on viewport resize within explicit bounds. The older minimum-height approach is superseded by ADR0006. `sidebar_button` allocates an explicit `152×29` interactive rect, draws a 16×16 colored badge with procedural `egui::Painter` vector strokes, and left-aligns the label at `x + 29` to prevent badge/text overlap. See [ADR 0005](adr/0005-overlay-and-reference-dialogs.md).


UI-012: Basic/Advanced is an independent persisted native UI presentation flag. Basic shows both monitors, Quick Play and reduced toolbars/input controls; it temporarily supersedes a remembered multiview presentation without changing the engine or that choice. Hosted run 39111788801 passed 75 tests and native project/configuration reopen, with reviewed Basic/playlist/keyboard captures. CPU cadence targets failed and complete reference acceptance remains false. The full boundary is recorded in docs/reference-audit.md and ADR 0010.
