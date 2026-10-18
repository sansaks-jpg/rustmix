# rustmix-ui

Native egui frontend with dockable monitors, inputs, simulated audio and Program overlay editing. See docs/ui-architecture.md. All engine changes use typed commands. File dialogs/IO are off the UI thread. UI-only layout and dialog actions use the coordinator dispatcher.

UI-009 allocates a fixed monitor-bank rectangle and clips child layouts to prevent per-frame height feedback. Input/mixer space is reserved when resizing or loading old panel state. The manual T-Bar is horizontal. Monitor transport widgets dispatch typed engine commands; pictures, countdowns, progress and timecode use the same source clock. Input effects and custom PNGs reach Program before transition/FTB. Unsupported output start paths report unavailable rather than counting fictitious frames/bytes.

The `dialogs.rs` module owns the source library, input properties and preferences pages. Add Input disables unsupported types and invalid source names; General edits are staged until Apply, while the Audio page dispatches simulated routing commands immediately. Settings stages display, appearance, shortcuts and workspace layout changes until Apply. Source deletion uses an explicit confirmation and the engine's selected-input guard. The background switcher is disabled while editing in a dialog.

The opt-in hosted smoke harness exercises native frames, egui input handling, state readback, save/reopen and wgpu screenshots. Run it only on the authorized GitHub build runner. Never use cua-driver for this task.

UI-004 through UI-006 bound all four dialog windows (`Add Input`, `Input Properties`, `Settings`, and `Overlay Settings`) to the viewport with separate navigation/page scrollers, left-aligned procedural category badges, and a fixed footer outside them. `Overlay Settings` stages per-slot edits and previews normalized 16:9 placement via `overlay_panel_rect`. Audio Mixer input strips and `Input Properties → General` provide direct shortcuts to `InputPage::Audio`, and `Multiview` supports `Split: Monitors + Grid` and `Full Source Grid` presets.


UI-010: PNG references and processing parameters belong to engine state. Input Settings keeps a draft/cache until OK; Cancel leaves Program untouched. Source pickers run off the UI thread, one capacity-32 PNG worker decodes bounded images, and stale project/editor results are ignored. Cached rasters fit 640x360, with Contain/Cover/Stretch and alpha over black. Missing PNGs remain visible and can be relinked through Change. Ordered Add Input and duplication preserve source/effect state. Hosted verification is pending.
