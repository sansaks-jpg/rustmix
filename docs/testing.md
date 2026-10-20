# Verification

UI-022 candidate: prepared Settings widget/Cancel/Apply, actual Overlay glyph alignment, Playlist dimensions/full-row interactions, bounded completion token and native geometry rejection regressions. Harness requires callback evidence while retaining strict shutdown/timeouts. Local fmt/diff/repository/reference/PowerShell syntax checks pass; no local compile/test/native. Exact Windows acceptance and screenshot comparison pending. See ADR0022.

UI-021 verification: Windows2022 run39422425429/be79b2e passed check/Clippy/all suites including115 UI tests and build. Exact native Overlay/dirty-channel/Playlist captures were reviewed; main completed stage51 then GPU drain timed out and Queue::drop panicked (exit-1073741819). Later native workflows did not run. f6db67c/run39420668981 passed all13 native processes but had a rejected label overlap. Preserve bounded adapter/stage diagnostics and strict gates. Local fmt/diff/repository/reference/script syntax checks pass; no local compile/test/native run. See ADR0021 and ui021-visual-review.md.

## UI-020 verification candidate (2026-10-20)

Local verification is restricted to formatting, diff inspection and repository/reference inventory
guards. The new toolbar regression renders Advanced/Basic at 1100x700, 1600x900 and 1920x1200,
checks visible targets for clipping/overlap and clicks Pause Inputs without exchanging Preview/Program.
The existing native monitor-geometry gate now checks toolbar target bounds/overlap too. Mixer widget
regressions exercise vertical strip geometry under a horizontal parent and the real mute widget's
typed command without switching buses. Source-dialog regressions cover the changed staged controls.
Hosted Windows2022 run39413208502 (bec97a8) passed locked check, warnings-as-errors Clippy,
all suites including106 UI tests, and release build. Main native generated all workspace/dialog
captures, pause/stability and smoke-ok evidence; exact screenshots/bounds were inspected. Its
process exited0 but the10s GPU drain proof failed, so full native acceptance FAILED and later
reopen/PNG/reference/output/audio/video/perf workflows did not run. Same-size texture reuse is
covered by actual Context delta tests but is not a proven hang fix. Code61f4286/run39365300339
is the last fully green baseline; its cadence targets still fail. Full reference/physical-device
qualification remains open. No local executable build/test/launch was performed. See ADR0020 and
ui020-visual-review.md for measured corrections and unresolved visual differences.

## Task 1.3 prepared acceptance (2026-10-19)

The external-output source change requires engine tests for atomic complete-slot Apply, rejected
slot/source/config without mutation, Program/Preview isolation, valid/invalid fractional rates,
raster/bus/delay/device/connector bounds, input removal, schema 1–5 migration and schema-6
round-trip/corruption. Widget tests must exercise the real gear, independent per-slot drafts,
Apply/Cancel, invalid configuration feedback and bounded footers. The hosted native harness must
capture `reference-output-settings.png` with `reference-output-elements.json`, then verify configured
slots after save/reopen. These new tests and
native checks are **prepared, not yet executed**; previous CI results do not verify this source.

Local checks are limited to formatting, source/diff inspection and the existing repository /
reference inventory guards. Compilation and native acceptance retain the environment policy below.

## Current UI-013 verification candidate

No Rust compilation, Rust tests, release build or native launch has been run locally. UI-012 baseline was accepted in run39111788801; new UI-013 category/search/theme/atomic-settings/T-Bar/snapshot tests and seven native processes require a new exact-code run.
Permitted local checks: `cargo fmt --all -- --check`, `./scripts/check-repository.ps1` (metadata,
dependency layering and asset inventory only), and `git diff --check` / source review.

Required exact-patch hosted Windows gates remain the existing workflow: locked workspace check,
warnings-as-errors Clippy, all-target unit/integration/widget tests, release build and native smoke.
New test files `engine/tests/production.rs` and `engine/tests/transitions.rs` cover slot pinning,
independent pixel vectors, playlist clock/options/protection/offline behavior, configuration
migration/IO and key validation. UI widget tests cover playlist editing and footer bounds,
modifier/repeat guards and shortcut draft cancellation/invalid application. The producer test
advances a playlist without a UI and checks generation acknowledgment.

Native `--ui-input-smoke` now adds production configuration through typed commands and actual
playlist widgets, starts/stops the playlist, uses the custom X key, saves `input-playlist.png`,
and requires schema-3 configuration in `--ui-input-smoke-reopen`. The cadence sampler must be
reviewed again because producer state acknowledgment changes the hot path. No timing improvement
or real-device behavior is claimed from prepared tests or local metadata checks.

## Earlier verification

UI-009 adds engine coverage for independent pause/resume/restart/loop, global pause with live transitions, late-created source origins, malformed surfaces and processed bus blending. Widget coverage checks 90 idle frames at each of four resize steps, horizontal T-Bar endpoint hold/release, actual monitor pause/restart clicks, frozen Program pixels, custom image resizing/sharpen and unavailable outputs. All 32 tests and native acceptance passed on GitHub-hosted Windows in [run 39089212707](https://github.com/sansaks-jpg/rustmix/actions/runs/39089212707), code `3a36cf2`. Screenshots were reviewed at 1100x700, 1600x900 and 1920x1080, including the final Shortcuts/Streaming availability states. No local Rust test/build was performed; physical hardware and endurance qualification remain open.

The final consistency regression also clicks the unavailable Merge/Record/Stream shortcut tiles and Streaming Start, requiring unchanged engine revision and no transition/record/stream activity. Native screenshot review covers the resulting disabled states and labels.

## Environment policy

All compilation, Rust tests and executable generation run on GitHub-hosted Windows (`windows-2022`, MSVC x64, Rust 1.94.1). Local formatting and source review are permitted. Do not build the executable on the owner's laptop. Do not use cua-driver.

The workflow runs formatting, locked workspace check, warnings-as-errors clippy, all-target tests and release build. It then launches the real native app in opt-in offline test mode, injects switching keys and a pointer event into the Master fader, captures the actual wgpu window surface, saves a project, closes, starts a second native process and verifies the reopened project.

Expected end state: Program 3, Preview 2, Overlay 1 enabled, Master below -1 dB, three dock tabs plus the resizable monitor bank persisted. Screenshots at initial and expanded workspace sizes are kept in `rustmix-ui-evidence`; screenshot dimensions must be read from the files rather than inferred from their names because runner desktop bounds/DPI can constrain the window.

Core tests cover Preview isolation, atomic CUT, crossfade cut point, reversible manual transition, cancellation, active source protection, invalid/NaN commands, rational rates, offline black, FTB/restore, simulated routing, project roundtrip, future/corrupt files, backup recovery and independent pixel blend vectors. Widget tests exercise actual egui button hit testing and keyboard shortcuts, multiple layouts and serialized docking. A drag test presses the tab, moves within it to initialize the drag, then moves to the central destination docking overlay and releases; jumping directly to the target does not initialize this library’s drag payload. Geometry assertions reject collapsed or overlapping 16:9 monitors at 1100×700, 1600×900 and 1920×1080 and require monitor growth when enlarging the window. A T-Bar regression holds the endpoint and releases it, asserting that the source exchange occurs exactly once.

Native smoke is a functional and visual check, not a long-duration/performance qualification. A missing adapter or screenshot is a failure, not a passing skipped GPU test. No external stream, camera, audio device or recording is opened. No local executable is required for inspection: retrieve only screenshot/JSON evidence from Actions.

Not yet verified: audio samples, capture devices, monitor hotplug, independent output clock, full disk interruption, GPU loss recovery, color-managed GPU parity, accessibility navigation audit, real keyboard device delivery, 30-minute/8-hour soak, production media playback and a signed installer. A headless widget test does not prove desktop event delivery; the native harness does not prove behavior on a physical operator workstation.

Current run evidence and exact outcomes belong in [ui-progress.md](ui-progress.md).

## Complete reference inventory versus acceptance

`scripts/check-reference-coverage.ps1` validates the committed [audit data](reference-audit.json):
30 areas, 40 input types, 20 effects, 16 overlay/stinger channels, 9 audio destinations, 12 UI
workflows and all 777 unique public function names in 29 groups. It checks the pinned function-name
hash, actual engine transition enum, overlay/audio bank sizes and evidence paths. The normal
repository guard checks inventory consistency; a green result is not reference feature acceptance.
`./scripts/check-reference-coverage.ps1 -RequireFullMatch` is the separate strict gate and must fail
while the ledger contains missing/partial behavior or lacks protocol/hardware/cadence qualification.
Current full match is false. Every future matched claim still requires actual scope-specific runtime
evidence; this static checker cannot establish behavior or replace hardware/AV/endurance tests.

Basic-mode regression checks reduced controls, real Quick Play, preservation of live project state,
restoring Advanced and independent multiview selection at three viewport sizes. The native input
workflow must capture `input-basic.png` after clicking Basic and preserve buses/revision when
returning to Advanced. Its deadline is 150 seconds for the added playlist/keyboard/Basic stages;
the outer process timeout is 600 seconds (raised on 2026-10-18 from 270 after the hosted runner measured 329 smoke frames in 240 seconds). This correction passed hosted run 39111788801; the Basic capture was visually reviewed. Global-mode restore has source coverage only; second-process tests verify project configuration.

## UI-004 / UI-005 / UI-006 dialog and workflow checks

Widget regressions cover: fixed footer geometry and sidebar navigation at 734×466, 1100×700, 1600×900 and 1920×1080 across `Add Input`, `Input Properties`, `Settings` and `Overlay Settings`; monitor gear / filter reset / offline visibility / removal cancellation with Preview/Program isolation; generator samples after source removal plus Settings draft cancellation; horizontal T-Bar endpoint hold/release; and `overlay_settings_stage_slot_edits_and_apply_only_on_apply` (staged per-slot edits, slot switching, Cancel discard, Apply commit, bottom-bar `output.overlay.settings` gear, Audio strip `audio.input.1.settings` routing shortcut, `dialog.properties.audioSettings`, and `Multiview` Split/Grid layout presets).

The hosted native harness opens `Add Input`, `Input Properties` and `Settings` at 1100×700, 1600×900 and 1920×1080 plus `Overlay Settings` at 1600×900 (`overlay-settings-1600.png` and `overlay-settings-1600-elements.json`), checks actual footer/body bounds, and verifies every artifact in `scripts/ci-native-smoke.ps1` before running the saved-project reopen check.

## UI-010 input acceptance

The new suite covers schema migration, invalid/cyclic processing graphs, insertion and safe duplication, persisted layer cleanup, preserving concurrent mixer edits, bounded PNG decoding, missing/corrupt/oversized files, real widget OK/Cancel, source relink, late results, capacity failures, image fit/alpha and processed PNG PIP. Native `--ui-input-smoke` imports PNG via the actual Add Input confirm control, verifies Cancel and OK, duplicates the input, checks Program pixels and saves; `--ui-input-smoke-reopen` verifies the same state/pixels in a second process. PNG file-picker results/cancellation are injected at the worker boundary in widget tests; the OS file-dialog UI is not automated. Hosted functional acceptance passed, including the current run 39111788801.

## UI-011 resource and cadence measurements (targets failed)

Clock tests cover exact progressive 24/25/30/50/60 and fractional 24000/1001, 30000/1001, 60000/1001 deadlines over an hour, late-wake skipping and bounded queues. Producer tests cover progress without a UI consumer and static-buffer reuse; source buffers retain capacity across renders. All native launches emit per-process `*-memory.json` and stderr files. `--ui-perf` samples moving 640x360 Program at 50/59.94/60 with seven inputs; `program-cadence.json` records actual frame production and skipped deadlines. A short sample meeting its target is not hardware, full-HD, audio/video sync or broadcast output qualification. A sample missing its target remains an explicit failed performance qualification.
