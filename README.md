# RustMix

Independent Rust live-production switcher for Windows x64. Experimental core
build with original code, controls and branding; not affiliated with vMix.

## Current core

- Independent Preview and Program, Cut/Fade, T-bar, fade to black, existing CPU
  transitions, caption overlays and per-input processing.
- PNG, local MP4 video, colour/test generators, camera descriptors, local WAV and
  microphone inputs. Files/devices are validated; camera hardware is unqualified.
- Sample-based 48 kHz stereo Master/Aux A/Aux B routing, gain/mute/sends and
  explicitly started Windows local monitoring. Master peak levels are measured;
  per-input peak metering is unavailable. Startup audio is off.
- Input grid, multiview, native Program window, explicit extended-display routing,
  playlists, keyboard controls and editable layout.
- Versioned project save/load, validated atomic replacement, journal and rotating
  backups. Recovery writes only changed unsaved revisions, once per revision.

Normal startup opens New Preset with **Full HD (1920x1080), 29.97p**
(30000/1001), then two black safety inputs. The CPU Program surface and the
**640x360 GPU monitor picture** are distinct. Portrait presets preserve their
orientation. FPS/drop values are observations, not hardcoded specifications.

Recording, streaming, external video transports, multicorder, replay, guest/network
inputs, browser/photo entry points, title templates and future settings are deferred
from normal navigation. Existing saved-source readers and private inspection code
remain for compatibility; a menu or descriptor is not backend qualification.

## Operate

1. Accept the New Preset format, or open an existing `.rustmix.json` project.
2. Add and validate an input. Selecting its thumbnail changes Preview only.
3. Use Cut/Space, Fade/Enter or the T-bar to take Preview to Program. B toggles
   fade to black. Editing text/dialogs suppresses production shortcuts.
4. Timed generators/media have transport controls; blank/static/live inputs do
   not acquire a fabricated sixty-second progress bar. Audio strips appear only
   for sources that actually provide audio. MP4 currently provides video only.
5. Configure WAV/microphone sources and use Settings -> Audio I/O to explicitly
   start/stop local monitoring. Adding an audio descriptor never starts a device.
6. Use the Fullscreen arrow for the separate Program window/display selection.
   Program remains independent of Preview and the operator window.
7. Save/Save As commits project JSON; Open validates before replacing the current
   session. Missing assets retain their identity with black/visible warning.

## Windows build and verification

The current owner request authorizes local deterministic checks and manual laptop
acceptance before push. GitHub Actions compiles and packages the `.exe` **without
running automated tests**. Packaging requires a successful build and includes
`BUILD-COMMIT.txt`, `SHA256.txt` and `AUDIT.md` so the artifact can be traced to its
source. It is an unsigned portable executable, not an installer.

Download `rustmix-windows-x64` from the successful
[Windows workflow](https://github.com/sansaks-jpg/rustmix/actions/workflows/windows.yml).
For exact local results, failed intermediate attempts, manual checks and scope,
see the [core audit](docs/core-engine-audit.md).

Developer checks (Rust 1.94.1, compatible Windows native toolchain):

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build -p rustmix --locked
```

Platform decoder tests require the generated original MP4 fixture described in
[testing](docs/testing.md). No FFmpeg/GStreamer runtime or vendor artwork is
bundled. Repository/reference inventory checks do not establish feature parity.

## Architecture and limits

The headless engine owns validated state/commands, rational timing, CPU rendering
and audio sample mixing. The platform crate isolates Windows Media Foundation,
WASAPI and display integration. The native egui interface displays immutable
frames with wgpu, reuses cached surfaces/texture allocations and dispatches typed
commands. Workers and handoffs are bounded; media/device failures must remain
visible and independent of Program state.

GPU composition, hardware encoding, recording/streaming, MP4 audio, A/V
synchronization, physical camera/microphone/display matrices, device-loss recovery
and production endurance are not certified. This is not broadcast-ready or full
reference parity. See [gaps](docs/gaps.md), [feature matrix](docs/feature-matrix.md),
[architecture](docs/architecture.md), [ADR 0036](docs/adr/0036-core-cpu-gpu-audit.md),
[roadmap](ROADMAP.md) and [licensing](docs/licensing.md).
