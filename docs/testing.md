# Verification

## Environment policy

All compilation, Rust tests and executable generation run on GitHub-hosted Windows (`windows-2022`, MSVC x64, Rust 1.94.1). Local formatting and source review are permitted. Do not build the executable on the owner's laptop. Do not use cua-driver.

The workflow runs formatting, locked workspace check, warnings-as-errors clippy, all-target tests and release build. It then launches the real native app in opt-in offline test mode, injects switching keys and a pointer event into the Master fader, captures the actual wgpu window surface, saves a project, closes, starts a second native process and verifies the reopened project.

Expected end state: Program 3, Preview 2, Overlay 1 enabled, Master below -1 dB, three dock tabs plus the resizable monitor bank persisted. Screenshots at initial and expanded workspace sizes are kept in `rustmix-ui-evidence`; screenshot dimensions must be read from the files rather than inferred from their names because runner desktop bounds/DPI can constrain the window.

Core tests cover Preview isolation, atomic CUT, crossfade cut point, reversible manual transition, cancellation, active source protection, invalid/NaN commands, rational rates, offline black, FTB/restore, simulated routing, project roundtrip, future/corrupt files, backup recovery and independent pixel blend vectors. Widget tests exercise actual egui button hit testing and keyboard shortcuts, multiple layouts and serialized docking. A drag test presses the tab, moves within it to initialize the drag, then moves to the central destination docking overlay and releases; jumping directly to the target does not initialize this library’s drag payload. Geometry assertions reject collapsed or overlapping 16:9 monitors at 1100×700, 1600×900 and 1920×1080 and require monitor growth when enlarging the window. A T-Bar regression holds the endpoint and releases it, asserting that the source exchange occurs exactly once.

Native smoke is a functional and visual check, not a long-duration/performance qualification. A missing adapter or screenshot is a failure, not a passing skipped GPU test. No external stream, camera, audio device or recording is opened. No local executable is required for inspection: retrieve only screenshot/JSON evidence from Actions.

Not yet verified: audio samples, capture devices, monitor hotplug, independent output clock, full disk interruption, GPU loss recovery, color-managed GPU parity, accessibility navigation audit, real keyboard device delivery, 30-minute/8-hour soak, production media playback and a signed installer. A headless widget test does not prove desktop event delivery; the native harness does not prove behavior on a physical operator workstation.

Current run evidence and exact outcomes belong in [ui-progress.md](ui-progress.md).

## Pending UI-004 acceptance checks

Three added widget regressions cover: fixed footer geometry and sidebar navigation at 734×466, 1100×700, 1600×900 and 1920×1080; monitor Edit/filter reset/offline visibility/removal cancellation with Preview/Program isolation; and generator samples after source removal plus Settings draft cancellation. The 734×466 case represents reduced logical space only and does not establish physical DPI behavior. These new Rust tests have not been run locally or remotely in this task.

The hosted native harness now opens Add Input, Input Properties and Settings at each of the three supported physical-size requests, checks actual footer/body bounds, and captures nine `add-input-*`, `input-properties-*`, and `settings-*` PNG files plus matching `*-elements.json`. The PowerShell evidence gate requires every file before starting the saved-project reopen check. Existing switching, fader and save/reopen assertions remain. These captures are prepared, not generated; inspect actual PNG dimensions and pixels after an authorized CI run.
