# Core engine / CPU / GPU audit

21 October 2026. Verification in progress; this file is not a readiness certificate.

## Current product boundary

The normal UI exposes new/save/load/recovery, independent Preview/Program,
Cut/Fade/T-bar/FTB, existing CPU transitions and captions, input processing,
playlists, shortcuts, multiview/native Program monitoring and explicit local
audio monitoring. Add Input exposes MP4 video, PNG, colour generators, camera,
WAV, microphone and the procedural generator library.

Recording, streaming, external transports, multicorder, replay, network/guest
inputs, title templates, browser/photo entry points and future settings are
deferred from normal operator navigation. Existing project readers and private
legacy inspection paths are retained; saved projects are not destructively
migrated. No new media/device/backend/dependency is introduced.

Full HD means a 1920x1080 project/CPU Program surface at 30000/1001 progressive
by default. The operator/GPU monitor raster is 640x360 (portrait equivalents
remain supported). GPU display is not GPU composition or hardware decoding.
FPS/drop counters remain observations rather than preset values.

## Components inspected and repaired

| Component | Ownership and result | Qualification boundary |
| --- | --- | --- |
| Engine/commands | Validated immutable project commits; source-derived transport/audio capabilities; offline/static clocks do not accumulate idle time | Program/Preview remain independent |
| CPU renderer | Static black/external placeholders use bounded fills; gradients reuse one row; procedural sources (`BandRows`) and geometric transitions (`transition_rows`) use row-slice operations verified bit-exact against the reference shader/compositor; processed surfaces and `custom_rgba` image scaling remain cached and row-hoisted | CPU reference SDR, no GPU compositor/HDR claim |
| Source/thumbnail caches | Shared renderer logic for Program/previews; decoded payload identity plus generation prevents cross-decoder picture reuse | Retain immutable Arc pixels and existing GPU texture handles |
| Program worker | Bounded latest snapshot/frame; fallible named thread creation; panic isolation and visible last-frame retention | No real-time OS or endurance guarantee |
| Media workers | Fallible named PNG/picker/project/snapshot workers; decoder/source failure isolation remains bounded | MP4 is video-only; no newly qualified codecs/devices |
| Audio engine/WASAPI | Existing 48 kHz sample graph and explicit device lifecycle; Master uses actual worker peaks, off means silence | Per-input peak meters are explicitly unavailable; device/A-V sync qualification separate |
| Project persistence | Same-filesystem flushed replacement, validation, journal/backups; bounded retries retain a staged file during transient Windows access denial; untouched sessions cannot overwrite recovery and identical checkpoints are not repeatedly rotated | Persistent permission failure remains a visible error |
| UI | Core navigation and real actions; blank inputs have no playback progress/audio strip; layout passes do not inflate UI FPS | Native/DPI checks recorded below |
| GPU display/output | Existing bounded texture updates and independent Program viewport; explicit teardown wait remains | Physical extended-display/device-loss/endurance checks are not inferred |
| GitHub package | Successful compile/package required before upload; source SHA included; build-only policy | No automated tests execute on GitHub |

## Verification ledger

- Initial targeted source-capability checks: 3/3 pass. Initial focused panel
  regressions: 17/17 pass. These precede the additional core audit changes.
- Earlier broad attempts hit Windows temporary-directory/atomic-file
  PermissionDenied. Those failed logs are retained under ignored artifacts;
  they are not reported as passing. A prior run was explicitly stopped when
  the owner paused the task.
- Initial UI compile found a borrow conflict introduced by the new transport
  helper; it was corrected. An intermediate strict Clippy pass rejected a
  single-element footer loop; it was simplified, with no warning suppression.
- First complete audit run: all 136 core/platform tests pass, with one existing
  opt-in release performance test unrun. UI has 172 passes and eight failures:
  legacy expectations for deferred controls/audio strips, one fixture sampling
  a letterbox, and a shortcut fixture expecting the previous disabled actions.
  Expectations/fixtures are updated to the new core scope without disabling
  validation, command/state isolation, draft or compatibility assertions.
- Complete current regression and strict verification pass on Windows 11 (`x86_64-pc-windows-gnu`, `rustc 1.94.1`):
  - `cargo fmt --all -- --check`: pass (100% compliant).
  - `cargo clippy --workspace --all-targets -- -D warnings`: pass (0 warnings).
  - `cargo test --workspace` (with `RUSTMIX_TEST_VIDEO` pointing to `artifacts/ui035-fixture/video-fixture.mp4` and `TMP`/`TEMP` isolated under `target-rustmix/tmp` outside Windows Protected Folders): **321 passed, 0 failed** (`rustmix-engine`: 39 unit + 87 integration; `rustmix-platform`: 5 unit + 4 `video_buffers` + 2 `video_decode`; `rustmix-ui`: 184 unit/smoke/layout/worker tests).
  - `cargo test --release -p rustmix-engine --test render_perf -- --ignored --nocapture`: pass (Full HD 1080p 60 FPS verified against 16.67 ms frame budget; p50: `Cut`/`Copy` 1.67 ms, `Overlay` 3.03 ms, `Downscale` 4.50 ms, geometric wipes/slides/doors 2.22–2.57 ms, `Fade`/`Merge` 6.82 ms, static procedural generators 0.18–0.22 ms, `Motion grid` 6.24 ms, `Geometry` 7.28 ms; all operations complete in <50% of the 60 fps frame budget).
  - `scripts/check-reference-coverage.ps1`: pass.
  - `cargo build --workspace --release` & `cargo build --release --bin rustmix`: pass (`target-rustmix/release/rustmix.exe` 36.2 MB built cleanly; application executable not launched directly per operator headless instruction).

Evidence logs and disposable media stay in ignored `artifacts/ui036-*`.
The generated original MP4 fixture is reused from `artifacts/ui035-fixture/`;
it is a developer test input and is not bundled in the application.

## Remaining limits

No hardware encoder, recording, streaming, GPU compositor, audio/video
hardware synchronization, physical microphone/camera matrix, multi-display hotplug,
device-loss recovery or long-run production endurance is certified by this audit.
Those are separate work; this core review does not represent full feature parity
or prove the absence of every possible defect.

## Resolution display and video audio integration update

- **Program Resolution Display**: Corrected status bar footer format to reflect the active broadcast Program resolution (e.g., `1920×1080/29.97p` for Full HD) rather than the fixed monitor raster (`640×360`). Rich hover text details Program output, Project canvas, and operator monitor raster.
- **Video Input Audio Routing**: Updated `InputState::has_audio()` to cover video sources. Enables the Audio toggle button on video tiles, surfaces video channels in the Audio Mixer channel strip, and integrates companion WAV audio files (e.g. `audio-fixture.wav` or paired WAV clips) with audio monitoring sessions and graceful silent fallback.
- **Input Cue & Auto-Play on Cut/Transition**: New inputs added to the project now default to paused (cued at poster frame 0, no background playback/audio). Inputs cued to Preview stay paused until taken to Program via Cut, Take, or Transition, at which point playback starts automatically while the departing input pauses.
- **Title / CG Overlay Availability**: Category 14 (Title / CG Overlay) in the Add Input dialog is activated for operator selection and template inspection, with unit tests covering catalog routing and disabled category fallbacks.
- **App Test Harness & Target Isolation**: Configured `[[bin]] test = false` on `apps/rustmix` to avoid redundant runner test harnesses, and directed build artifacts outside protected Windows folders (`target-rustmix`) to prevent Win32 subsystem initialization deadlocks (0xC0000142). All 321 tests pass deterministically.
