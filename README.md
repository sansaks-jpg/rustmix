# RustMix

UI-035 (2026-10-21): reference-based review of existing operator panels,
inspectable output navigation, page-specific Settings and isolated input draft
resets. Local Rust test runners are authorized for this task; no application
executable is built. See [scope and verification](docs/ui035-reference-uiux.md).

UI-034 source candidate (2026-10-21): New Preset on normal startup/New, staged
landscape/portrait/rational-rate/scan settings, two black safety inputs, native
Camera format selection, portrait Browser viewport and a bounded HTTP discovery
fix. **Hosted compilation and runtime acceptance are pending.** Monitoring stays
640x360 / 360x640 progressive; full-HD/interlaced broadcast output is unavailable.
See [scope and verification](docs/ui034-add-input-and-new-preset.md).

UI-032 Add Input source candidate (2026-10-21): isolated PNG/generator drafts,
dedicated Image/Colour/audio forms, inspectable unavailable categories, semantic
original icons and validation/engine-aware OK gating. Eight regressions and six
mandatory native page captures are prepared; new hosted compile/test/build/native
acceptance is **pending**. No new backend or full reference-equivalence claim.
See [change/verification record](docs/ui032-add-input-changes.md).

UI-028 through UI-031 (2026-10-21): Video-first Input Select with genuine bounded
recent files and validation; topmost Escape/popup/repeat safety; responsive
default widths preserving real resizing; Input Settings expanding fields,
aligned labels/flags/colour/fit/metadata and left source caption.
Source 56048cb/run 39556368477: locked check/Clippy, 274 tests (150 UI), release, all15 native processes and packaging pass.
Exact native PNG/JSON reviewed.
Full UI/workflow correspondence, original source-icon semantics, remaining
source/property/preview dimensions, GPU reliability, cadence and physical/
DPI/endurance qualification remain open. Light theme is deprioritized.
See ADRs0028-0031 and UI-028/029/030/031 reviews for exact scope and every failed candidate.

UI-024 through UI-027 (2026-10-21): monitor pictures/transport now align at the
top, Production gives more room to inputs, Add Input retains its selected category,
and the quick menu opens actual Video/PNG/WAV/Microphone pages. Unsupported output
actions are visibly unavailable. Source ba3b209/run 39540613350 passes all 261 tests
(137 UI), check/Clippy/build and 13 native workflows; exact native images reviewed.
Light theme is deprioritized. Full UI correspondence, intermittent hosted GPU
reliability, high-rate cadence and hardware/endurance qualification remain open.
See docs/ui027-native-acceptance.md for evidence and the retained failures.

UI-023 (2026-10-20): flat operator controls, left monitor titles, centered/bounded
transition controls and one selected Recording Setup page are implemented and visually
checked on exact source 126291d. [Windows run 39452377970](https://github.com/sansaks-jpg/rustmix/actions/runs/39452377970)
passes 251 tests, build and all 13 native processes. Recording remains unavailable;
full visual equivalence, Light-theme contrast and hosted responsiveness remain open.
See [comparison and evidence](docs/ui023-visual-review.md).

UI-022 (2026-10-20): Settings/Playlist reference dimensions and right-aligned Overlay labels are implemented and verified against exact-source screenshots. 7aeac9c/run39434584384 passed hosted checks,121 UI tests plus all suites, build and all13 native processes. Cadence, intermittent GPU reliability and full UI/feature equivalence remain open. See [visual review](docs/ui022-visual-review.md).

UI-021 (2026-10-20): overlay draft decisions/engine preview and Playlist layout/full-width selection implemented and checked against exact-source screenshots. be79b2e passed hosted check, Clippy, all tests including115 UI tests and build, but main native failed GPU shutdown. Full reference equivalence and runtime acceptance remain open. See [visual review](docs/ui021-visual-review.md).

UI-020 (2026-10-20): operator proportions, mixer strips and bounded dialogs were refined and
visually compared against the supplied references. Code bec97a8 passed hosted check, Clippy,
106 UI tests plus all other suites, and release build. Native screenshots were reviewed, but
GPU shutdown drain failed; complete native workflows and full reference equivalence remain open.
See [visual review](docs/ui020-visual-review.md) and [ADR 0020](docs/adr/0020-operator-layout-refinement.md).

Local MP4 video source accepted on hosted Windows (2026-10-19): an operator can add a local video
file through Add Input, see decoded Windows Media Foundation RGB32 frames in the source tile,
Preview and the 640×360 CPU Program composite, pause/resume them, and save/reopen the schema-8
descriptor. [Run 39273839106](https://github.com/sansaks-jpg/rustmix/actions/runs/39273839106)
(commit `1df8c47`) passed every step and uploaded the probe evidence, quadrant-pixel Program
capture, pause/resume motion evidence and byte-identical save/reopen files. This is MP4 video-only
and remains experimental: no MP4 audio track, camera, capture card, network input, GPU/hardware
decode, A/V synchronization or broad codec compatibility is implemented or qualified. See
[ADR 0018](docs/adr/0018-operator-local-mp4-video-source.md).

Fullscreen correction (2026-10-19): the top button now sends PGM directly to an
extended display and leaves the operator window in place. No F11 or dragging is
required. Select a display with its adjacent arrow. Missing/unsafe displays produce
an operator message. Hosted acceptance for the Program viewport capture path passed in [run 39270445070](https://github.com/sansaks-jpg/rustmix/actions/runs/39270445070);
direct extended-display routing remains an implemented candidate and physical two-monitor acceptance
is pending; see [ADR 0017](docs/adr/0017-direct-extended-program-output.md).

2026-10-19 full-production continuation: the independent Program-window/fullscreen controls, a real
48 kHz sample mixer, bounded WAV decoder, typed WAV/microphone descriptors and the Windows WASAPI
adapter passed hosted [run 39270445070](https://github.com/sansaks-jpg/rustmix/actions/runs/39270445070)
(all steps green). Native evidence covers the Program viewport with deferred screenshot delivery and
the audio descriptor/metadata workflow; it did not open an audio device, and no physical extended
display was qualified. New source candidates do not inherit previous test acceptance. See
[complete delivery plan](docs/final-delivery-plan.md), [ADR 0015](docs/adr/0015-real-audio-and-independent-program-window.md)
and [current progress](docs/ui-progress.md). Complete vMix-class feature readiness remains open.

Task 1.3 continuation (2026-10-19): four external-output configuration slots are implemented
in source, with validated independent routing, a staged editor and schema-6 persistence. These
are configuration records, not device outputs. External Start remains unavailable. Compilation,
regression execution and native screenshot/save-reopen acceptance for this change are **pending**;
previous hosted runs below do not verify it. See [ADR 0014](docs/adr/0014-external-output-configuration.md).

Independent Rust native live-production switcher. **Experimental synthetic desktop UI** with an experimental real audio path and experimental local MP4 video decode, not a broadcast-ready product. No affiliation with vMix. No capture devices, encoders, recording or streaming backends are implemented.

The desktop UI uses eframe/egui and wgpu. The headless engine owns typed commands, Program/Preview, transition timing, input properties, audio sample mixing, video-feed metadata and project validation. All demo images are generated by original Rust code; test media is generated by project code or scripts.

UI-012 adds engine-owned production controls based on the supplied reference findings: four
persisted transition slots with independent durations, twelve actual CPU effects, named input
playlists with per-item timing and manual/loop controls, and editable keyboard mappings under
Settings → Shortcuts. The playlist gear configures the sequence; the adjacent button starts/stops
it. Program state is acknowledged by the bounded producer instead of being overwritten on every
UI repaint. Basic now has its own operator presentation instead of toggling Multiview. These
changes are **experimental, functionally tested** in [Windows run 39111788801](https://github.com/sansaks-jpg/rustmix/actions/runs/39111788801):
75 tests, five native processes, save/reopen and required screenshots passed. See the
[feature comparison](docs/reference-feature-comparison.md) and [ADR 0009](docs/adr/0009-engine-owned-production-controls.md).

The reference Task 1.1 port adds the schema-5 sixteen-channel overlay bank: eight title-panel
channels and eight stinger channels with `overlay.stinger.*` commands and an exactly-once
cut-point Program/Preview exchange. Stinger clips do not composite yet (no media renderer), and
schemas 1–4 migrate on load while older binaries cannot open schema 5. This port is
**experimental, functionally tested** in [Windows run 39129117131](https://github.com/sansaks-jpg/rustmix/actions/runs/39129117131):
all workspace tests, the full native smoke (nine dialog captures, three byte-identical save/reopen
pairs, empty native stderr) and packaging passed; hosted cadence targets still fail. See
[ADR 0012](docs/adr/0012-production-overlay-bank-and-stingers.md) and the
[complete audit](docs/reference-audit.md).

The reference Task 1.2 port adds the typed `InputCategory` contract (`All`, six preset colours,
`Custom(1..=16)` clamping, filter-only `Search`) and the `ProjectState::visible_inputs` engine
query; the tile grid filters through the engine while chips, context menu, Settings editing and the
text search are unchanged. Local headless verification passed (engine 61 tests, UI 47, clippy/fmt
and both guards clean); hosted acceptance for this commit is pending. See
[ADR 0013](docs/adr/0013-typed-input-category-contract.md).

**Full reference parity is not achieved.** The [complete reference audit](docs/reference-audit.md)
covers all 30 observed areas and their input/effect/overlay/audio inventories. Run
`./scripts/check-reference-coverage.ps1 -RequireFullMatch` for the strict gate: it currently rejects
full acceptance because major backends, protocol compatibility and cadence/hardware qualification
remain missing. Ordinary inventory/functional test success is not full feature acceptance.

UI-009 repairs the growing monitor bank, changes the T-Bar to horizontal, and gives each synthetic input an engine-owned pause/restart/loop clock shared by its picture, progress bar and timecode. Program now processes each input before composition. Status distinguishes CPU composition from GPU display and unsupported outputs cannot display a false live state. All 32 tests and hosted native acceptance passed in [run 39089212707](https://github.com/sansaks-jpg/rustmix/actions/runs/39089212707); see [ADR 0006](docs/adr/0006-stable-monitor-bank-and-source-transports.md).

## Download / build

Windows x64 builds and tests run **on GitHub Actions**, not on the owner's laptop. Open the latest successful [Windows native UI run](https://github.com/sansaks-jpg/rustmix/actions/workflows/windows.yml) and download `rustmix-windows-x64`. Extract and run `rustmix.exe`. This is an unsigned portable prototype, not an installer.

`rustmix-ui-evidence` contains native rendered screenshots and smoke-state JSON when the graphics environment supports launch. A build is not accepted as a successful UI milestone until that smoke step passes.

Latest fully green hosted acceptance (2026-10-19): [run 39273839106](https://github.com/sansaks-jpg/rustmix/actions/runs/39273839106), runtime revision `1df8c47`. Every step passed, including the new native local-MP4 video acceptance and the reopened video project. **Cadence targets still fail** on hosted Hyper-V: 29.66 / 34.23 / 40.40 fps for 50 / 59.94 / 60 targets at 640×360 (248 / 285 / 334 frames produced, 170 / 214 / 162 skipped; `within_sample_target` false in all three). Physical GPU/devices, audio-device callbacks, AV synchronization, endurance and broadcast readiness remain open. See [verification details](docs/ui-progress.md) and the [complete reference audit](docs/reference-audit.md).

Earlier verified prototype (UI-012): [75 passing tests and native Windows acceptance](https://github.com/sansaks-jpg/rustmix/actions/runs/39111788801), runtime revision `a01f498`. Download [rustmix-windows-x64](https://github.com/sansaks-jpg/rustmix/actions/runs/39111788801/artifacts/12130624284) or [native evidence](https://github.com/sansaks-jpg/rustmix/actions/runs/39111788801/artifacts/12130639014). Basic/Advanced, playlist, key settings and reopened captures were reviewed. That run's **cadence targets failed** on hosted Hyper-V: 28.57 / 38.75 / 34.46 fps for 50 / 59.94 / 60 targets at 640×360.

Developer commands (for CI / an authorized build machine with Rust 1.94.1 and Windows MSVC tools):

```text
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --release --workspace --locked
```

## Operate the demo

Seven synthetic inputs are available on launch. Select an input thumbnail to change Preview. Space cuts, Enter fades, and the T-Bar scrubs a manual crossfade. The source exchange commits at completion; cancel returns to the original Program. B fades to black / restores. F1–F4 toggle Program overlays. Text editing suppresses switching shortcuts.

Add Input supports seven procedural types, PNG images, local WAV audio and an experimental local MP4 video file. Unsupported device/network categories remain disabled. A selected video file is validated by an off-thread probe before its descriptor commits, and decoded frames feed its tile, Preview and Program; an external-media input without a decoded frame shows black instead of a procedural substitute. Generator samples are named as generators, and unsupported files cannot silently become synthetic inputs. Per-tile Cut explicitly takes that source to Program. Per-tile 1–4 buttons use its name as a lower-third title, rather than overlaying the full video input. Quick Play cuts the already animating synthetic Preview; Pause Inputs freezes procedural video while transitions remain responsive. Input Edit opens rename, color, type and online/offline properties. Removing Program/Preview or editing sources during a transition is rejected explicitly. Offline inputs render black and stay in the project with a warning. No camera, capture-card, network or browser input is implemented.

The source selector now groups seven implemented procedural generators alongside explicitly disabled media/device categories. Input Properties separates General, staged audio routing, and Source status; source removal requires confirmation. Settings groups display, appearance, workspace and shortcuts with a draft Apply/Cancel flow for project preferences and layout presets. Checkpoint recovery runs when requested; mixer-strip changes apply immediately; Input Settings applies its audio draft with OK. UI-009 acceptance now covers hosted native screenshots and tests for this workflow. Physical device/DPI qualification remains open.

Audio strips control gain, mute, solo and Master/Aux A/Aux B assignments. An explicitly started
local monitoring session mixes validated WAV clips and configured microphones at 48 kHz through the
Windows WASAPI adapter and reports actual Master block peak/RMS; the main mixer's animated legacy
meters remain simulations. Hosted acceptance covers the native descriptor workflow only: no audio
device or callback has been qualified on hardware, and there is no mix-minus hardware qualification,
compressed audio or A/V synchronization.

The main layout follows common vMix-style placement: orange Preview, green Program, centered transition stack, inputs below, audio on the right and production controls along the bottom. All code/media/branding remain independent. Drag the lower panel tabs to rearrange, dock or float them inside the main window. Drag dividers and the lower edge of the monitor bank to resize. View restores Production / Audio focus presets. Multiview changes the main monitor area into source tiles. Fullscreen routes Program to a selected extended display or a separate Program window; it is not an SDI, NDI, virtual-camera or encoder output, and physical two-display acceptance is pending.

Save / Save As writes versioned `.rustmix.json` project JSON with the panel layout. Open restores it. Files are validated and replaced atomically on the same filesystem; `.bak1` / `.bak2` retain previous versions and `.bak0` is a recovery journal. Existing corrupt projects are not overwritten. File dialogs and disk operations run on a worker so the demo remains responsive. Automatic recovery checkpoints are written every 60 seconds to the user's application data RustMix folder; use Settings → Recover checkpoint. Project files are not silently restored at launch. Window geometry and panel placement are persisted by eframe.

Canvas resolution is a project preference; the UI demo raster is capped at 640×360 and source previews at 384×216. CPU demo generation time and UI callback FPS are measured; they are not broadcast output, encoder, GPU or end-to-end latency metrics.

UI-004 through UI-006 refine the broadcast operator workflow across all four bounded dialogs (`Add Input`, `Input Properties`, `Settings`, and `Overlay Settings`): left-aligned sidebar navigation with procedural vector icons, fixed Apply/Cancel footers, staged per-slot `Overlay Settings` with a normalized 16:9 placement preview, visual shortcut keycap cards, direct per-input audio routing shortcuts (`⚙` on mixer strips and `"Audio Settings…"` on Input Properties), toolbar dropdown carets (`Open ▼`, `Save As ▼`, `Settings ▼`), graduated horizontal T-Bar scale marks, broadcast-dark dock tabs, and `Multiview` Split/Grid layout presets.

## Documentation

- [UI references](docs/ui-reference.md), [UI architecture](docs/ui-architecture.md), [progress and evidence](docs/ui-progress.md)
- [Feature matrix](docs/feature-matrix.md), [gaps](docs/gaps.md), [testing](docs/testing.md), [roadmap](ROADMAP.md)
- [ADR 0001](docs/adr/0001-native-demo-ui.md), [ADR 0004](docs/adr/0004-main-window-control-fidelity.md), [ADR 0005](docs/adr/0005-overlay-and-reference-dialogs.md), [licensing](docs/licensing.md)

No production readiness, hardware support, GPU compositor parity, A/V synchronization, long-run timing or live output continuity is claimed.

UI-010 adds persisted PNG references and processing settings, ordered insertion, source duplication, transactional Input Settings and bounded asynchronous PNG loading. These descriptors were introduced in schema 2; schemas 1/2 migrate through the current chain to schema 8, which older builds cannot open. Images remain external references: keep the original files available. Missing images retain their input and show black with a relink warning. This input workflow passed hosted acceptance and is included in the current schema-8 test runs; see docs/ui-progress.md.
