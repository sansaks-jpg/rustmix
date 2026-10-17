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

Project IO uses validated schema 1 JSON, a 1 MiB bound, finite numeric validation, 2–32 inputs, short strings and known formats/rates. Save writes/syncs a same-directory tempfile, validates a JSON roundtrip, journals and atomically replaces. Two previous generations are retained. Invalid existing files are not overwritten. Paths stay out of operator logs. Bounded background jobs retain the current project on errors.

Automation: command buttons, source thumbnail selection, overlay toggles and audio faders publish stable IDs/bounds to an internal registry. The opt-in `--ui-smoke` native harness injects egui events, checks engine state and requests real GPU screenshots. JSON exposes the active revision and resulting source IDs. There is no general-purpose remote UI endpoint yet; full per-widget coverage remains a gap.

Measured UI callback FPS and CPU image generation time are distinct from output-clock statistics. Canvas preferences do not change the capped monitor raster. Real-time video/audio scheduling, a custom GPU compositor and device-loss continuity are not implemented.

The main layout follows observed generic broadcast placement from the owner's installed vMix 29, with original RustMix controls/media and native Windows system typography. Four lower-third slots work; per-input number buttons set the source name as a title. Slots 5–8 and unsupported output/merge/playlist features are disabled. Direct thumbnail Cut is an atomic engine command and rejects taking the already active Program source. See ADR 0002.

The monitor bank gives switcher widgets the panel's actual available height (without consuming it in advance). Its persisted egui PanelState height scales proportionally on viewport resize, retaining the operator's chosen monitor/input proportion within bounded minimum and maximum heights. T-Bar completion remains latched through the pointer-release frame, because egui can emit a final changed slider value on release. Otherwise an endpoint held after completion could take Preview a second time.

UI-003 (pending hosted verification) adds distinct Add Input / Input Properties / Settings windows with native sidebar navigation. Generator selection displays representative synthetic source frames; unsupported source types remain visibly disabled. Source General edits stage until Apply; the audio page issues `audio.input.set` immediately and discloses simulation. Properties shows live source status and requires confirmation for removal. Settings stages `Preferences`, project-name and layout-preset changes until Apply; checkpoint recovery remains an immediate, separately confirmed action. Background switching controls and shortcuts are disabled while a dialog is open. Button IDs `dialog.add.confirm`, `dialog.properties.apply`, and `dialog.settings.apply` support synthetic widget tests. Original local vendor screenshots were only viewed in gitignored `references/private/`, never incorporated in distributable media.

## UI-004 operator refinements (source only)

The existing dialogs use a shared viewport-bounded Window builder. Navigation and page content each own a bounded ScrollArea; Apply/Cancel remain sibling controls below both. A single cached 320×180 sample texture is generated on source-type changes and does not depend on a matching source in the project. No sample is registered in the engine or sent to Preview/Program.

Monitor header Edit actions open the current input's properties. Source filters expose category and match count alongside the color strip; offline thumbnails show explicit text and black fallback. Both Properties and the source context menu dispatch RequestRemove, which checks engine eligibility and opens a blocking egui Modal before removal. Engine commands and project schema are unchanged. Automation rectangles are rebuilt every frame rather than retaining hidden controls. See ADR 0003; compile, widget execution and native render evidence remain pending.
