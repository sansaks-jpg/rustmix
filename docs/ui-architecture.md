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

DockState holds Switcher, Inputs, Audio and Overlays. Panel tabs drag between split areas or floating egui windows. Production / Audio focus reset presets. Project saves include the serialized layout; eframe separately saves window geometry and its panel snapshot. Imported layouts are checked for bounds, valid tree children, active indices and exactly one of each panel before use. Invalid layout falls back visibly without replacing engine source identities.

Demo source IDs remain stable across rename/save/reopen. Program/Preview are different state fields. Input changes cannot select Program implicitly. Remove rejects currently selected IDs. Offline state retains identity/configuration and renders black. Renderer failures retain the last valid monitor texture and set a warning.

Audio simulation is a pure function of time and channel ID. Input mute gates every send, input solo excludes nonsolo inputs, bus gains/mute/solo gate bus meters. It is not an audio callback, sample mixer, limiter, compliant loudness meter or audio-follow-video implementation.

Project IO uses validated schema 1 JSON, a 1 MiB bound, finite numeric validation, 2–32 inputs, short strings and known formats/rates. Save writes/syncs a same-directory tempfile, validates a JSON roundtrip, journals and atomically replaces. Two previous generations are retained. Invalid existing files are not overwritten. Paths stay out of operator logs. Bounded background jobs retain the current project on errors.

Automation: command buttons, source thumbnail selection, overlay toggles and audio faders publish stable IDs/bounds to an internal registry. The opt-in `--ui-smoke` native harness injects egui events, checks engine state and requests real GPU screenshots. JSON exposes the active revision and resulting source IDs. There is no general-purpose remote UI endpoint yet; full per-widget coverage remains a gap.

Measured UI callback FPS and CPU image generation time are distinct from output-clock statistics. Canvas preferences do not change the capped monitor raster. Real-time video/audio scheduling, a custom GPU compositor and device-loss continuity are not implemented.
