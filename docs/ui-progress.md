# UI milestone progress

2026-10-17 · Tasks UI-001 / UI-002 · Experimental synthetic desktop application.

## Delivered scope

The repository began with AGENTS.md only. The implementation now contains a three-package Rust workspace: a headless engine, an egui UI library and a native eframe/wgpu application. The public repository is [sansaks-jpg/rustmix](https://github.com/sansaks-jpg/rustmix). No executable was compiled on the owner's laptop; all compilation, Rust tests, native launch and packaging run on GitHub-hosted Windows.

Implemented typed Preview/Program selection, atomic CUT/direct input take, timed fade/wipe, reversible manual transitions, FTB/restore, seven procedural sources, source properties/lifecycle controls, simulated audio controls, four text overlays, project preferences, docking, resizing, workspace persistence and validated atomic project IO with recovery copies. Unsupported outputs are disabled with reasons. See the [feature matrix](feature-matrix.md) and [known gaps](gaps.md) for scope boundaries.

UI-002 follows the owner's locally observed vMix main-window arrangement: orange Preview, green Program, central transition stack, lower source grid, right-side mixer and bottom output controls. Code, media, branding and drawings are original. No proprietary screenshot or resource is committed or shipped. No pixel identity or feature parity claim is made.

## Verification history

An initial hosted run [39022239822](https://github.com/sansaks-jpg/rustmix/actions/runs/39022239822) passed its then-current checks and native smoke. Actual screenshot inspection exposed collapsed monitors and clipped widgets that state-only tests had missed. That visual result was rejected. Explicit vertical child layouts and monitor geometry regressions were added before the observed-layout refit.

Development runs caught egui API incompatibilities, warnings treated as errors and incorrect docking-test gestures. These failures were corrected rather than hidden. The drag test now initializes the library's drag payload with an intermediate pointer movement and targets the central docking overlay, retaining its assertion that Audio and Inputs share the resulting leaf.

Code revision `df2dd41` in [run 39024705813](https://github.com/sansaks-jpg/rustmix/actions/runs/39024705813) passed formatting, the dependency/asset guard, locked workspace check, warnings-as-errors clippy, all 13 then-current tests, release build, native smoke and packaging. Native final/reopened state was Program 3, Preview 2, Overlay 1 enabled and Master -24 dB; native stderr was empty. The actual 1600×900 and 1920×1080 screenshots were inspected. They exposed a monitor bank that retained too little height when enlarging the viewport; a larger T-Bar handle was also needed. These observations prompted another refinement and regression checks.

The latest source preserves the monitor/input proportion on resize, reserves the full panel extent, enlarges the gray T-Bar handle and freezes the countdown label with Pause Inputs. The new actual-widget test found a real double-take bug on pointer release: egui emits the endpoint value on the release frame, so clearing the completion latch before consuming it caused a second source exchange. The latch now remains active through that frame.

## Accepted prototype verification

[Run 39025762188](https://github.com/sansaks-jpg/rustmix/actions/runs/39025762188), code revision `db4b7c03486ba26bee8638634aca9312e1ba50e9`, completed successfully on 2026-10-17. Platform: hosted Windows Server build 20348, MSVC x64, Rust 1.94.1; reported graphics device Microsoft Hyper-V Video, driver 10.0.20348.1. This is not physical GPU/capture/audio hardware qualification.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `scripts/check-repository.ps1` | Passed dependency layering / asset inventory |
| `cargo check --workspace --locked` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed; warnings treated as errors |
| `cargo test --workspace --all-targets --locked` | 14 passed, 0 failed, 0 ignored: 9 integration, 2 renderer/meter, 3 widget |
| `cargo build --release --workspace --locked` | Passed on GitHub only |
| `scripts/ci-native-smoke.ps1` | Passed native launch, actual egui key/pointer events, GPU readback, save, process exit and reopen |
| Packaging / dependency licence manifest and notices | Passed; portable unsigned executable artifact generated |

The final smoke and reopened project JSON are byte-for-byte equal: Program 3, Preview 2, Overlay 1 enabled, Master -24 dB, revision 9 and saved three-tab dock topology. Native stderr is empty. Widget regressions pass for docking, T-Bar endpoint hold/release, hostile layout validation and noncollapsed 16:9 monitors at 1100×700, 1600×900 and 1920×1080. The monitor-growth regression also passes.

Actual RustMix screenshots `workspace-1600.png` (1600×900), `workspace-1920.png` (1920×1080) and `reopened.png` (1920×1080) were downloaded from this run and visually inspected. Monitors, transition controls, active input indicators, lower-third text and the mixer are visible. The enlarged layout retains monitor proportions. Lower input rows and additional mixer strips deliberately scroll when they exceed their panel. Dialog-by-dialog visual QA, physical pointer/keyboard delivery and a full accessibility audit remain open.

Download the [Windows executable artifact](https://github.com/sansaks-jpg/rustmix/actions/runs/39025762188/artifacts/12098313180) or [native evidence](https://github.com/sansaks-jpg/rustmix/actions/runs/39025762188/artifacts/12098273409). Only screenshot/JSON evidence was downloaded to the owner's laptop. No local `target` directory or project executable was generated. No cua-driver command was executed. Documentation-only commits after the code revision do not rebuild the unchanged executable.

## Limits and next milestone

This remains a UI prototype: CPU procedural media presented through egui's wgpu backend, deterministic simulated meters, and no real capture, sample-based audio, independent output clock, recording or streaming. Canvas preferences do not prove corresponding processing resolution. No physical workstation, hardware-device, accessibility, long-duration or GPU-loss qualification has been completed.

Next: establish timestamped Program frame delivery independent of UI redraw, CPU golden fixtures and GPU comparison tests; then bounded still-image ingest and a real synthetic sample-based audio routing graph.
