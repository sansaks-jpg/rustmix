# rustmix-ui

Native egui frontend with dockable monitors, inputs, simulated audio and Program overlay editing. See docs/ui-architecture.md. All engine changes use typed commands. File dialogs/IO are off the UI thread. UI-only layout and dialog actions use the coordinator dispatcher.

The `dialogs.rs` module owns the source library, input properties and preferences pages. Add Input disables unsupported types and invalid source names; General edits are staged until Apply, while the Audio page dispatches simulated routing commands immediately. Settings stages display, appearance, shortcuts and workspace layout changes until Apply. Source deletion uses an explicit confirmation and the engine's selected-input guard. The background switcher is disabled while editing in a dialog.

The opt-in hosted smoke harness exercises native frames, egui input handling, state readback, save/reopen and wgpu screenshots. Run it only on the authorized GitHub build runner. Never use cua-driver for this task.

UI-004 bounds the three dialog windows to the viewport. Each window has separate navigation/page scrollers and a footer outside them. Add Input caches one original procedural sample independent of the current source bank. Monitor Edit dispatches the existing Properties action; all UI removal paths dispatch RequestRemove before the engine command. Filters have text/count/reset feedback; offline tiles retain their identity and show black fallback explicitly. Current-frame automation bounds are cleared before drawing, preventing stale hidden-widget bounds. The new source and tests have not yet been compiled or launched on the hosted runner.
