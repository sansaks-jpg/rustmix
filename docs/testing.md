# Verification

UI-041 (2026-10-24) — reference-verification sweep of the UI-040 alignment
work (commit `101c41f` plus its in-flight fixes); runs under the UI-036 owner
instruction. Method: four reference areas were freshly re-derived from the
installed vMix 29.0.0.48 (SHA-256 in the UI-039 gaps entry) — overlays,
audio, transitions, transport — and thirteen implementation units were
checked against that evidence. Verdicts: ten correct, three discrepancies;
all three discrepancies were confirmed and fixed in this session's working
tree (uncommitted changes preserved per the owner branch policy); zero
contested, zero not fixed. The re-derived evidence and its per-answer
strength are recorded in [reference-notes](reference-notes.md); the outcome
table below cites what this session actually read and ran.

Discrepancies confirmed and fixed (fix sites read and tests located this
session; re-covered by the gate run below):

- **overlay-xml-state-shape** — the state document rendered all sixteen
  channels as self-closing empty elements, with a comment claiming the
  occupied-channel evidence "is not in the repository". The fresh
  re-derivation produced the official DeveloperAPI.html sample shape:
  an occupied channel carries the bound input's NUMBER as element text
  (direct evidence). Fixed: `document.rs` renders
  `<overlay number="N">input</overlay>` for occupied standard channels using
  the same numbering as `<input number=…>`/`<preview>`
  (crates/automation/src/document.rs:158-186); empty channels and stinger
  channels 9–16 keep the self-closing form; a binding that no longer
  resolves renders empty rather than a wrong number. New api_contract test
  `occupied_overlay_channels_carry_the_bound_input_number_as_element_text`
  (crates/automation/tests/api_contract.rs:40); suite 14→15. Gap notes
  updated everywhere the old claim lived: the feature-matrix limitation,
  crates/automation/README.md, reference-audit.md:69 (area 4) and
  reference-audit.json `areas[3]` (JSON re-validated this session:
  30 areas, area-4 gap text present).
- **docs-overlay-rows** — the channel 2–8 rows of reference-audit.md and the
  feature-matrix Graphics/Overlays limitation still carried the stale
  "no source overlays" wording that `101c41f` had already obsoleted.
  Fixed: rows now carry the exact `101c41f` channel-1 wording (verified
  this session at reference-audit.md:171-178; `grep -c "no source overlay"`
  over docs/feature-matrix.md, docs/reference-audit.md and
  docs/reference-audit.json returns 0 hits in each).
- **live-pip-clock** — the Program composite sampled bound overlay inputs at
  clock 0, so a playing input's picture-in-picture never animated on
  Program. Fixed at both ends: `render::program_from_sources_clocked`
  resolves the PiP clock per bound input (crates/engine/src/render.rs:637-688)
  with `program_from_sources_at` delegating through a constant resolver
  (render.rs:616-627), and the UI composites through it with each input's own
  `engine.playback(id).position_ms()` — the same clock that generates that
  input's full-frame surface — plus a `pip_clocks` field in the composite
  cache key so an advancing transport invalidates the cached composite
  (crates/ui-egui/src/program.rs:208-220, 457-470). New tests
  `clocked_pips_follow_each_bound_inputs_own_clock`
  (crates/engine/tests/overlays.rs:278; suite 14→15) and
  `a_bound_overlay_pip_advances_with_the_input_transport_on_program`
  (crates/ui-egui/src/program.rs:627; suite 188→189). The settings-dialog
  draft visualization deliberately keeps clock 0: it is the documented
  no-live-mutation draft on a synthetic background, not Program output.

Units verified correct (no change needed): input-backed-overlay-channels
(channel-geometry PiP under the caption text), overlay-control-functions
(wire mapping incl. Off-as-clear and the 9–16 stinger reason),
ui-overlay-input-assignment, per-source-audio-meters (post-gain pre-clamp
taps), nine-bus-routing (Master + A–G + Headphones),
ui-meter-strips, transition-set-mapping (member-by-member 20-effect set
comparison against the re-extracted enum), playpause-control-function
(toggle via the public per-input clock), category-catalogue-rename
(Aqua/Blue; Azure and MidnightBlue do not exist in the reference binary),
and control-api-coverage-sweep (every dispatch arm against the five
families).

Workspace gate — all four commands run and passing in this session on the
integrated tree (owner laptop, Windows x64, GNU toolchain, Rust 1.94.1;
TMP/TEMP/TMPDIR redirected into `<workspace>/target/tmp` per the UI-040
environment note, no retry needed):

- `cargo fmt --all -- --check` — pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — pass,
  no warnings.
- `cargo test --workspace --all-targets --locked` — **363 passed, 0 failed,
  1 ignored** (the pre-existing opt-in `render_perf` release benchmark),
  0 filtered; 22 test binaries. Per suite: engine lib 42; engine
  integration audio_samples 9, audio_sources 4, categories 8, inputs 10,
  outputs 10, overlays 15, presets 6, production 8, source_capabilities 3,
  transitions 8, video_sources 7, wav 7, workflow 11, render_perf 0
  (+1 ignored) — engine total 148; platform lib 5, video_buffers 4,
  video_decode 2 (generated MP4 fixture present in `artifacts/`) —
  platform total 11; automation api_contract 15; UI 189.
- `cargo build -p rustmix --locked` — pass.
- Inventory guards re-run after the doc edits: `pwsh -File
  scripts/check-reference-coverage.ps1` and `pwsh -File
  scripts/check-repository.ps1` — both pass ("Reference inventory verified:
  30 areas, 40 input types, 20 effects, 16 overlay/stinger channels,
  9 audio destinations, 12 UI workflows, 777 unique shortcut function names
  in 29 groups … Full reference match: False", the expected state while
  protocol/hardware/cadence qualification is unqualified).

Not covered by this entry: unchanged from UI-040 — no physical
camera/microphone/display/audio device, GPU drain/teardown workflows, the
Full HD benchmark of the eight per-pixel effects, hosted CI execution,
long-duration/soak qualification, and the deferred families in
[gaps](gaps.md). Reference behavior that remains unevidenced (overlay
9–16 ≡ stinger numbering, transition ordinals, meterF scale, PiP motion
during ordinary transitions, PlayPause on transport-less inputs) is listed
in [reference-notes](reference-notes.md).

UI-040 (2026-10-24) — reference-alignment follow-up; runs under the UI-036 owner
instruction (local build/manual laptop acceptance precedes push; GitHub compiles
and packages without running tests). Everything below was executed in this
session on the owner laptop (Windows x64, GNU toolchain, Rust 1.94.1), except
where explicitly marked as reported by an implementation agent.

Workspace gate — all four commands run and passing on the integrated tree
(implementation changes from three agents plus the docs edits):

- `cargo fmt --all -- --check` — pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — pass, no
  warnings.
- `cargo test --workspace --all-targets --locked` — **360 passed, 0 failed,
  1 ignored**, 0 filtered. Per suite: engine lib 42; engine integration
  audio_samples 9, audio_sources 4, categories 8, inputs 10, outputs 10,
  overlays 14, presets 6, production 8, source_capabilities 3, transitions 8,
  video_sources 7, wav 7, workflow 11, render_perf 0 (+1 opt-in ignored) —
  engine total 147; platform lib 5, video_buffers 4, video_decode 2 (generated
  MP4 fixture present in `artifacts/`) — platform total 11; automation
  api_contract 14; UI 188. The one ignored test is the pre-existing opt-in
  `render_perf` release benchmark, unchanged by this work.
- `cargo build -p rustmix --locked` — pass.

New coverage introduced by the implementation run and re-verified inside the
gate run above:

- Engine overlays: 14 integration tests (the file grew 9→14 vs HEAD, verified
  by `git show HEAD:crates/engine/tests/overlays.rs | grep -c '#\[test\]'`) —
  overlay binding command semantics, project validation, pixel-level
  composite-under-caption with offline/parked inputs, save/load plus
  legacy-file load, and input-removal cleanup; 3 further unit tests live in
  `crates/engine/src/overlays.rs` (engine lib 42, 39 at HEAD).
- Engine transitions: 8 tests including `golden_frames_pin_every_effect_curve`,
  pinning all 20 effects' motion at 25/50/75% with FNV-1a frame hashes; the
  temporary hash-extraction probe was removed before landing (reported by the
  engine agent; the landed file contains only the pinned table).
- Engine audio_samples: 9 tests including
  `per_source_taps_measure_post_gain_pre_clamp_levels_per_input` (per-source
  impulse peak/RMS, mute-to-silence, pre-clamp behaviour, per-render slot
  reset, underrun silence).
- Automation api_contract: 14 tests (was 9) — Off clears the overlay channel,
  `OverlayInput3Out` is unsupported and leaves the binding intact,
  `OverlayInput9`/`16` return the stinger-range reason, outside-vocabulary
  spellings stay UnknownFunction, an unconfigured `Stinger1` is rejected with
  "Configure a stinger input", and PlayPause covers playing, paused and
  no-transport inputs.
- UI suite: 188 tests including meter-strip geometry from injected peaks,
  stable `audio.{id}.meter` IDs for every visible strip, the overlay-channel
  input selector dispatch test and the disabled-selector reason test.

Hosted/native evidence reported by the UI agent (not re-executed in this docs
session): `cargo build -p rustmix --release --locked` success,
`target/release/rustmix.exe --ui-audio-smoke` exit 0 with the new stage-14
`assert_mixer_meters` check, artifacts `audio-source.png` and
`audio-source-elements.json` (contain `audio.bus.0.meter` and
`audio.input.8.meter` with real rects), and `artifacts/overlay-panel.png`
captured from a release-build instance showing the channel-01 input row.

Environment note: the tempfile-based tests previously hit `PermissionDenied
(os error 5)` on `C:\Users\...\AppData\Local\Temp` in this sandbox; for this
run TMP/TEMP/TMPDIR were redirected into `<workspace>/target/tmp` before
starting, and no such failure occurred — the 360-test result above is a clean
single pass, not a retry across redirects.

Reference inventory guards after the docs/inventory edits: `pwsh -File
scripts/check-reference-coverage.ps1` and `pwsh -File
scripts/check-repository.ps1` — both run and passing, reporting
"Reference inventory verified: 30 areas, 40 input types, 20 effects,
16 overlay/stinger channels, 9 audio destinations, 12 UI workflows, 777 unique
shortcut function names in 29 groups. Actual engine scope: 20 effects, 8
title-panel channels and 8 modelled stinger channels, 9 configured audio buses
(device qualification separate). Full reference match: False." The strict
`-RequireFullMatch` mode continues to fail while protocol/hardware/cadence
qualification is unqualified, which is the expected state. (The checker script
requires PowerShell 7 — `SHA256.HashData` does not exist in Windows PowerShell
5.1, which fails on its own .NET call before reading any data.)

Not covered by this entry: any physical camera/microphone/display/audio device,
GPU drain/teardown workflows, the Full HD benchmark of the eight per-pixel
effects (still opt-in, still unrun for them), hosted CI execution (the Windows
workflow packages without running tests), long-duration/soak qualification, and
the deferred families listed in [gaps](gaps.md) and the
[feature matrix](feature-matrix.md).

UI-036 current owner instruction supersedes the older UI-035 build restriction:
local build/manual laptop acceptance precedes push; GitHub compiles/packages
without automated test execution. See [current audit](core-engine-audit.md) for
exact checks, failed intermediate runs, measurements and scope boundaries.


UI-035 owner exception (2026-10-21): local Rust tests are explicitly authorized;
do not build/package the RustMix application `.exe`. Use selected library and
integration test runners, Clippy checks and test-only software captures of actual
egui output. Native Windows/wgpu screenshots are not claimed. See
[commands, results and failures](ui035-reference-uiux.md).

Final UI-035: 177 UI tests pass, zero failed/ignored/filtered, with 94 software
PNG/JSON pairs inspected at 1100×700 and 1600×900 using local Windows typography.
132 core/platform tests pass across the sequential and fixture-backed retries;
one existing opt-in release benchmark is unrun. Clippy/fmt/inventory/diff pass.
Earlier failed runs are preserved in the linked ledger. No application build or
native GPU/DPI/hardware acceptance.

UI-032 Add Input candidate (2026-10-21): eight new regression tests and the
mandatory independent `--ui-add-smoke` process are prepared. The latter requires
six new native page PNG/JSON pairs, real sidebar/Cancel input, PNG draft retirement,
disabled invalid/unavailable OK and unchanged project/output state, with its own
180s deadline and the existing strict stage/readback/GPU-drain gates. No previous
gate is relaxed. Formatting, inventory, diff and PowerShell syntax checks pass
locally; new compile/Clippy/Rust tests/build/native captures are pending on hosted
Windows. See [exact scope and acceptance](ui032-add-input-changes.md).

UI-028 through UI-031 (2026-10-21): Video-first Input Select with genuine bounded
recent files and validation; topmost Escape/popup/repeat safety; responsive
default widths preserving real resizing; Input Settings expanding fields,
aligned labels/flags/colour/fit/metadata and left source caption.
Source 56048cb/run 39556368477: locked check/Clippy, 274 tests (150 UI), release, all15 native processes and packaging pass.
Real key/click/edge-drag and painted-glyph regressions cover
validation/history/clear/cancel, popup/held/modifier Escape, manual-transition
preservation, default/custom widths and General typing/Escape/OK/alignment.
Exact selected native PNG/JSON, original CRC32, raster/restoration, all mode
drains/stderr/traces inspected. Core/dialog/layout checks are independently
bounded mandatory processes; no single-session/endurance or GPU-fix claim.
Local checks only fmt/diff/inventory/script syntax, no local compile/tests/build/
native launch. Prior failed tests and runtime candidates remain in the ledger.
Full UI/workflow correspondence, original source-icon semantics, remaining
source/property/preview dimensions, GPU reliability, cadence and physical/
DPI/endurance qualification remain open. Light theme is deprioritized.
See ADRs0028-0031 and UI-028/029/030/031 reviews for exact scope and every failed candidate.

UI-024 through UI-027 source ba3b209/run 39540613350 passes locked workspace check,
warnings-as-errors Clippy, all 261 tests across 17 suites (137 UI, none ignored/filtered),
release build, 13 native processes and packaging. Exact PNG/JSON, all 13 drain proofs,
stderr, restored 1600x900 and next input viewport inspected. Native step 104 s; high-rate
Program samples still miss 50/59.94/60 fps. Earlier acb00d7/e44050b native failures remain
recorded; a passing later run does not establish a driver fix. Local verification was
formatting, diff, repository/reference inventory and PowerShell syntax only. No local
Rust compile/test/build/native launch. Full UI/DPI/hardware/endurance acceptance remains
open; Light theme is deprioritized. See ui027-native-acceptance.md.

UI-023 source 126291d/run 39452377970 passes locked workspace check, warnings-as-errors
Clippy, all 251 tests including 127 UI, release build, all 13 native processes and packaging.
Exact workspace/Recording PNG/JSON were inspected. New regressions check painted title/
take alignment, all effect/caret bounds,12 px form gutter, recording navigation, disabled
controls and unchanged project/output state through 1920x1200. All 13 GPU drains pass;
stderr is empty. Native step675 s and profiler UI about1.5fps remain a hosted performance
gap; cadence28.8106/36.9233/33.4939fps misses50/59.94/60 targets at 640x360. No physical,
endurance, full-equivalence or causal GPU-fix claim. Local checks remain formatting,
diff and inventory only. Earlier compile/test failures and cancelled/same-source repeat
runs are retained in ADR 0023/ui023-visual-review.md.

UI-022 verified on hosted Windows2022 run39434584384/7aeac9c: cargo check --workspace --locked, cargo clippy --workspace --all-targets --locked -- -D warnings, cargo test --workspace --all-targets --locked (121 UI plus other suites), release build and all13 native processes/package pass. Exact Settings/Playlist/Overlay PNG/JSON inspected. Callback evidence bounded and real; cadence43.1605/43.1390/42.4105fps misses50/59.94/60 targets. Intermittent prior GPU failure remains unresolved. Local checks only fmt/diff/repository/reference/PowerShell syntax; no local compile/test/native. See ADR0022/ui022-visual-review.md.

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

## Audio, Program viewport and local MP4 video acceptance (2026-10-19)

Hosted Windows [run 39270445070](https://github.com/sansaks-jpg/rustmix/actions/runs/39270445070)
(commit `3ed91a6`) accepted the independent Program-viewport delivery path and the native audio
descriptor workflow. Hosted Windows
[run 39273839106](https://github.com/sansaks-jpg/rustmix/actions/runs/39273839106) (commit
`1df8c47`) accepted the local MP4 video source. Both runs were fully green on `windows-2022` with
Rust 1.94.1.

### Video fixture and native video smoke

The video input is generated by `scripts/ci-video-fixture.ps1`, which runs before the workspace
tests in CI. It is an **external developer fixture generator/oracle only**: the script pins the Gyan
FFmpeg 9.0.2 essentials Windows archive and verifies its SHA256 before executing it, and both the
GPLv3 developer tool and its libraries are excluded from RustMix packages and evidence uploads.
The RustMix application never links, bundles or invokes FFmpeg. The generated artifact is original procedural
content: 160×90, 30/1, 60 frames of red/green/blue/white quadrants plus a moving black square,
encoded as H.264 Baseline yuv420p (limited BT.709, GOP 30, no B frames, no audio). The script
records provenance, hashes and the exact command in `artifacts/video-fixture.json`, then exports
`RUSTMIX_TEST_VIDEO`. Local `cargo test` runs without that export, so `crates/platform/tests/video_decode.rs`
falls back to the script's own default output location, `artifacts/video-fixture.mp4` (the
`RUSTMIX_TEST_VIDEO` environment variable always takes precedence). A missing fixture is a hard
failure, never a skipped test.

`--ui-video-smoke` drives the real Add Input dialog, waits for the off-thread probe and rejects any
raster other than 160×90 or any frame-rate ratio other than 30/1 (Media Foundation may report an
unreduced 60/2 for the same stream). It then takes the input to Program, requires the four
quadrant sample points to match within tolerance, requests three 1600×900 GPU screenshots
(`video-source.png`, `video-program.png`, `video-resumed.png`), pauses through the engine transport
clock and requires the Program picture to freeze, resumes and requires a pixel change, saves, and
closes. `--ui-video-smoke-reopen` starts a second process from `video.rustmix.json`, checks schema 8
and the persisted `VideoSource::File` descriptor, requires exactly one owner worker and re-verifies
decoded Program pixels before writing `video-reopen-ok.json`. The PowerShell harness additionally
requires byte-identical `video-workflow-ok.json` / `video-reopen-ok.json`.

### Audio descriptor smoke

`--ui-audio-smoke` generates a one-second 44.1 kHz mono PCM16 RIFF/WAVE fixture in Rust, validates
it through the real Add Input dialog, opens and closes the Audio I/O dialog in its devices-closed
state, takes the audio-only input to Program and asserts the Program slate stays black, saves, and
closes. It fails if an audio session or monitoring starts. `--ui-audio-smoke-reopen` verifies the
persisted `AudioSource::WavFile` descriptor and that reopening does not arm monitoring. Evidence:
`audio-fixture.wav`, `audio-input.png`, `audio-io.png`, `audio-source.png`, the three element JSONs,
`audio-validation.json`, and byte-identical `audio-workflow-ok.json` / `audio-workflow-reopen-ok.json`.

### Program viewport capture retries and stage guards

`--ui-output-smoke` exercises the second native Program viewport, windowed/fullscreen child state,
Preview isolation, FTB-black capture and save/reopen. Every deferred screenshot request carries a
monotonic acceptance sequence; the capacity-one mailbox drains obsolete captures and accepts only
the frame matching the current request, so a retried stage can never be satisfied by a stale
readback. The output, audio and video native workflows each use a 360 s in-process workflow timeout
and a 240-frame per-stage guard that reports the stage that stopped advancing; the external process
kill remains at 600 s.

What this proves: descriptor storage, probe validation, real decoder negotiation, delivered
Program pixels from the generated fixture, transport freeze/resume behavior, native viewport
samples and save/reopen equality on the hosted runner. What it does **not** prove: audio-device or
callback behavior, physical extended-display behavior/hotplug, A/V synchronization, broad codec
compatibility, hardware/GPU decode performance, endurance or production output.

### Overlay caption verification (2026-10-19)

Overlay titles and subtitles are rasterized into the Program surface by the engine compositor
(`engine::overlay_text`, [ADR 0019](adr/0019-overlay-captions-in-program-surface.md)); the operator
monitor no longer paints its own copy. Unit tests in `crates/engine/src/overlay_text.rs` and
`crates/engine/src/render.rs` assert glyph coverage for printable ASCII, ink containment inside the
panel rectangle for lower-left, lower-right and top-left positions, title and subtitle ink presence,
fade-to-black covering the ink, and disabled overlays leaving source pixels untouched. The live
stage of `--ui-output-smoke` sets deterministic caption text (`PROGRAM LIVE` / `CAPTION CHECK`),
then maps every title/subtitle ink pixel of the Program raster into the captured native Program
viewport and requires it to be bright there (`program-output-live.json` records
`caption_title_ink_pixels`, `caption_title_bright_pixels`, `caption_subtitle_ink_pixels` and
`caption_subtitle_bright_pixels`). Because the overlay fill darkens its panel region, a bright pixel
inside the panel can only come from caption ink. This does not qualify typography (bitmap font,
printable ASCII only), styling, animation, tickers, data binding, per-source overlays, GPU/hardware
composition or any physical output.

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

For UI-035, the owner's later explicit authorization permits local Rust test
runners and Clippy while preserving the application-executable build ban. The
hosted-only policy below records the prior/default workflow; it does not prohibit
the specifically authorized local tests for this task.

All compilation, Rust tests and executable generation run on GitHub-hosted Windows (`windows-2022`, MSVC x64, Rust 1.94.1). Local formatting and source review are permitted. Do not build the executable on the owner's laptop. Do not use cua-driver.

The workflow runs formatting, locked workspace check, warnings-as-errors clippy, all-target tests and release build. It then launches the real native app in opt-in offline test mode, injects switching keys and a pointer event into the Master fader, captures the actual wgpu window surface, saves a project, closes, starts a second native process and verifies the reopened project.

Expected end state: Program 3, Preview 2, Overlay 1 enabled, Master below -1 dB, three dock tabs plus the resizable monitor bank persisted. Screenshots at initial and expanded workspace sizes are kept in `rustmix-ui-evidence`; screenshot dimensions must be read from the files rather than inferred from their names because runner desktop bounds/DPI can constrain the window.

Core tests cover Preview isolation, atomic CUT, crossfade cut point, reversible manual transition, cancellation, active source protection, invalid/NaN commands, rational rates, offline black, FTB/restore, simulated routing, project roundtrip, future/corrupt files, backup recovery and independent pixel blend vectors. Widget tests exercise actual egui button hit testing and keyboard shortcuts, multiple layouts and serialized docking. A drag test presses the tab, moves within it to initialize the drag, then moves to the central destination docking overlay and releases; jumping directly to the target does not initialize this library’s drag payload. Geometry assertions reject collapsed or overlapping 16:9 monitors at 1100×700, 1600×900 and 1920×1080 and require monitor growth when enlarging the window. A T-Bar regression holds the endpoint and releases it, asserting that the source exchange occurs exactly once.

Native smoke is a functional and visual check, not a long-duration/performance qualification. A missing adapter or screenshot is a failure, not a passing skipped GPU test. No external stream, camera, audio device or recording is opened. No local executable is required for inspection: retrieve only screenshot/JSON evidence from Actions.

Not yet verified: audio device/callback and hardware acceptance, capture devices, monitor hotplug, independent output clock, full disk interruption, GPU loss recovery, color-managed GPU parity, accessibility navigation audit, real keyboard device delivery, 30-minute/8-hour soak, broad production media playback (only generated fixtures qualify), A/V synchronization and a signed installer. A headless widget test does not prove desktop event delivery; the native harness does not prove behavior on a physical operator workstation. Audio mixer/WAV decoding and local MP4 video decode now have hosted descriptor/pixel evidence, but no physical audio device or display was opened.

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
UI-034 candidate (2026-10-21): prepared preset/scan/migration/camera atomicity,
portrait Browser and real New/Cancel/unsaved-widget regressions. HTTP discovery
regression uses a complete keepalive response whose next read times out. Hosted
`--ui-preset-smoke` exercises normal cold initialization, actual New/size/rate/OK
clicks, Cancel isolation, portrait-60p reference texture, save/load and three GPU
captures. Source smoke now uses a 1080x1920 loopback Browser viewport. Existing
native modes, bounded deadlines, adapter evidence and GPU drain gates remain.
Local allowed formatting/diff/inventory/script-syntax checks only; new Rust/native
checks have not run. Old f6ecbdd/run39603431352 failed Browser discovery and GPU
teardown. See [UI-034](ui034-add-input-and-new-preset.md) for limits.
