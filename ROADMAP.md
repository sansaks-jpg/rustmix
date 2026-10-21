# Roadmap

UI-036: core-only operator navigation and an audit of all current CPU/GPU,
media, audio, project and worker boundaries. New backends remain deferred.
Current validation and remaining limits: [core audit](docs/core-engine-audit.md).


UI-035: existing-panel workflow alignment against `vmix-photos`, reviewed through
actual egui paint without building the application executable. See
[UI-035](docs/ui035-reference-uiux.md). Native/DPI/GPU and unavailable backend
qualification remain separate follow-up gates.

UI-034: implement Add Input refinements and the requested startup/New format
selector. Source complete; next is authorized exact-revision hosted checks,
mandatory preset/source/native captures and inspection. Physical camera/audio,
full-resolution/interlaced output and resource/cadence qualification remain open.
See [UI-034](docs/ui034-add-input-and-new-preset.md).

UI-032 Add Input fixes are implemented as an unverified source candidate:
draft isolation, dedicated forms, truthful categories/icons and OK readiness.
Next: exact-revision hosted compile/tests/build, six required native captures
and screenshot review. Missing media/device backends and full form equivalence
remain outside this candidate. See [UI-032](docs/ui032-add-input-changes.md).

2026-10-21 Add Input audit: [focused findings and acceptance gates](docs/add-input-uiux-audit.md).
Next within this scope: prevent a validated PNG draft from being committed under
another source category (ADD-UX-01), then separate Image/Colour/Title forms and
correct readiness/catalog/icon semantics. Audit only; no fixes implemented or
new native acceptance claimed.

UI-028 through UI-031 (2026-10-21): Video-first Input Select with genuine bounded
recent files and validation; topmost Escape/popup/repeat safety; responsive
default widths preserving real resizing; Input Settings expanding fields,
aligned labels/flags/colour/fit/metadata and left source caption.
Source 56048cb/run 39556368477: locked check/Clippy, 274 tests (150 UI), release, all15 native processes and packaging pass.
Next: source-icon semantics and remaining property/preview
proportions/navigation. Current changes are experimental selected workflows.
Full UI/workflow correspondence, original source-icon semantics, remaining
source/property/preview dimensions, GPU reliability, cadence and physical/
DPI/endurance qualification remain open. Light theme is deprioritized.
See ADRs0028-0031 and UI-028/029/030/031 reviews for exact scope and every failed candidate.

- UI-024 through UI-027: selected caption/Display, monitor/transport, Production
  proportions and source-navigation corrections verified on ba3b209/run 39540613350.
  All 261 tests (137 UI), build and 13 native workflows pass; exact images inspected.
  Next UI work: source-page structure, typography/spacing and dialog navigation.
  Light theme is deprioritized. Keep GPU reliability, cadence and hardware gaps
  separate from visual acceptance. See docs/ui027-native-acceptance.md.

- UI-023: flat operator treatment, left monitor titles, centered/bounded effect controls,
  selected-format Recording page and readable form gutter accepted for source 126291d;
  run 39452377970 passes 251 tests, build and 13 native processes with exact PNG/JSON review.
  Next: Light-theme icon contrast, remaining dialog title/footer/typography dimensions
  and the reproducible hosted slowdown. Full equivalence/backends/readiness stay open.
  See docs/ui023-visual-review.md.

- UI-022: selected Settings/Playlist/Overlay dimension/alignment corrections visually accepted on7aeac9c; run39434584384 fully green including121 UI tests and13 native processes. Next: review isolated UI023 Recording Setup candidate; resolve remaining header/typography/backend gaps, intermittent GPU failure and cadence. Full parity and production qualification stay open. See ui022-visual-review.md and ui023-recording-candidate.md.

- UI-021: overlay draft safety/preview and Playlist workflow corrections visually verified on be79b2e. Hosted check/lint/tests/build pass; main native reaches finalstage51 then GPU drain/Queue::drop failure. Next: resolve measured GPU reliability and refine Settings proportions; full parity/cadence/hardware qualification stay open. See ADR0021 and ui021-visual-review.md.

- UI-020 (2026-10-20): responsive operator proportions and dialog corrections are implemented
  with exact-code screenshot comparison. bec97a8 passed hosted check/Clippy/tests/build but failed
  native GPU shutdown drain. Next: qualify GPU reliability, preserve Overlay Settings drafts and
  truthful preview, then refine remaining Settings/Playlist/record/stream form differences. Full
  parity and production acceptance remain open. See docs/ui020-visual-review.md and ADR 0020.

- M5 progress, not complete (2026-10-19): Program overlay captions are rasterized into the Program
  surface by the engine compositor (original 5x8 bitmap font for printable ASCII, no new dependency),
  after the panel geometry and before fade-to-black, and the duplicated monitor-only caption painter
  was removed so the monitor, Program window, fullscreen output and captures show the same words.
  `--ui-output-smoke`'s live stage now verifies caption ink in the captured native Program viewport
  and records the counts. Hosted acceptance is pending for this commit. Remaining open: bitmap-only
  typography (no font choice, outline, shadow, animation, ticker or data binding), ASCII-only glyphs,
  no per-source overlay, stinger clip rendering (G20), and the synthetic `Graphics` input's
  decorative text still being monitor-only. See ADR 0019.

- M2 progress, not complete (2026-10-19): local MP4 video source implemented and hosted-accepted —
  schema-8 `VideoSource::File` descriptors, Windows Media Foundation RGB32 decode on one bounded
  owner thread per input, exact-PTS pacing, engine `VideoFeed`, off-thread Add Input probe
  validation, black fallback for external media, and native `--ui-video-smoke` /
  `--ui-video-smoke-reopen`. Hosted Windows
  [run 39273839106](https://github.com/sansaks-jpg/rustmix/actions/runs/39273839106) (commit
  `1df8c47`) passed every step, including probe format 160×90 at 30/1, quadrant Program pixels,
  pause-freeze/resume-motion evidence and byte-identical save/reopen. Camera/device capture,
  network ingest, MP4 audio, GPU/hardware decode and broad codec compatibility remain open;
  recording/streaming (M4) and a custom GPU compositor (M1) are still unimplemented.
  See ADR 0018.

- M3 progress, not complete (2026-10-19): real audio path implemented and hosted-accepted for its
  descriptor/metadata workflow — fixed 48 kHz stereo block mixer with Master/Aux A/Aux B and guest
  returns, click-safe gain ramps, strict PCM16/float32 WAV decode, typed WAV/microphone descriptors,
  the Windows WASAPI adapter through CPAL and an explicitly started monitoring session. Hosted
  Windows [run 39270445070](https://github.com/sansaks-jpg/rustmix/actions/runs/39270445070)
  (commit `3ed91a6`) passed every step; native acceptance never opened an audio device. Physical
  device callback/hardware qualification, mix-minus hardware tests, resampling/quality review and
  A/V synchronization remain open. See ADR 0015.

- Task 1.3 continuation (2026-10-19): finish the existing four-slot external-output configuration
  slice, atomic source/config Apply, preserved drafts, schema-6 migration and validation tests.
  Source implementation only; new hosted build/test/native acceptance pending. No hardware or
  network output backend. Next broadcast blockers remain Program clock/scheduler delivery (G01) and
  the remaining audio device/A-V-sync work (G02).

- Task 1.1 port (reference → rustmix): sixteen-channel overlay bank — eight title-panel channels
  and eight stinger channels (configured input, cut point, display-underneath) — with schema 5 and
  1–4 migration, `overlay.stinger.*` commands and the cut-point Program/Preview exchange. Hosted
  acceptance passed in run 39129117131 (commit `f597c22`): full native smoke, nine dialog captures,
  three byte-identical save/reopen pairs, empty native stderr; hosted cadence targets still fail.
  Stinger clip compositing absent. See ADR 0012 and docs/reference-audit.md.

- Task 1.2 port (reference → rustmix): typed `InputCategory` contract — `All`, six preset colours,
  `Custom(1..=16)` with clamping and the filter-only `Search` pseudo value — plus the
  `ProjectState::visible_inputs` engine query; the tile grid filters through the engine and the
  existing schema-5 ids, command, chips, context menu and Settings editor stay unchanged. Local
  headless verification passed (engine 61 tests, UI 47, clippy/fmt/guards clean); hosted acceptance
  for this commit is pending. No control endpoint, so no wire ordinals. See ADR 0013 and
  docs/reference-audit.md.

- UI-012: reference-driven production controls — persisted independent transition slots and
  twelve CPU effects, named engine playlists with per-item transitions/holds and offline handling,
  editable keyboard bindings, schema 3 migration, acknowledged producer state and independent
  Basic/Advanced presentation. Hosted functional acceptance passed in run 39111788801 (`a01f498`):
  75 tests, five native processes, reviewed captures and save/reopen. Cadence targets failed.
  Full audit: docs/reference-audit.md; complete reference acceptance remains false.
  Media/audio/output/hardware milestones remain open.

- UI-009: stable bounded monitor bank, horizontal T-Bar, independent headless source transport, consistent processed Program surfaces and truthful output/processing status — tested on hosted Windows: 32 tests plus native screenshot/save/reopen acceptance in [run 39089212707](https://github.com/sansaks-jpg/rustmix/actions/runs/39089212707). Complete vMix feature parity and production engine qualification remain separate uncompleted milestones.

- M0/UI-001: three-package workspace, headless commands, rational rates, synthetic media, project IO and hosted Windows CI — implemented, verification tracked in docs/ui-progress.md.
- M1/UI-001: native interactive synthetic switcher, CPU monitor composition, dockable panels, simulated audio controls and overlay UI — experimental; custom GPU compositor remains open. (Later increments add a real audio path and local MP4 video; see the M2/M3 progress entries above.)
- UI-002: observed broadcast-style placement, native system typography, compact input controls and atomic direct takes — experimental prototype verified by hosted tests, native smoke and screenshot review; see docs/ui-progress.md.
- UI-003: sidebar-based source selector, input properties/audio/status, staged settings and denser monitor/input layout — experimentally verified by the hosted widget/native workflow; hardware qualification remains open.
- UI-004: bounded sidebar dialogs with fixed confirmation controls, independent generator samples, monitor properties access, readable input filters/offline state and unified removal confirmation — hosted compilation, widget geometry and native dialog captures passed; physical DPI/accessibility qualification remains open.
- Next: M1 reliability — independent timestamped Program frame delivery, deterministic CPU golden fixtures, GPU reference comparisons and malformed-input/fault coverage.
- M2: still images, local media and bounded source workers; add hardware capture only after device evidence.
- M3: real sample-based audio graph, routing, click-safe parameters and mix-minus.
- M4: recoverable recording and local test streaming after separate encoder/licensing review.
- M5+: complete automation schemas, hardware/accessibility/soak matrix, secondary outputs and packaging.

Later milestones in AGENTS.md are plans, not shipped capabilities.

- M2/UI-010: persisted still-image input lifecycle, staged effects, insertion order, duplication, relink and bounded PNG workers implemented. Hosted functional acceptance passed, including the current native PNG save/reopen workflow. Local MP4 file video is implemented and hosted-accepted (see M2 progress above); camera/capture-device hardware and physical qualification remain open.

- UI-011: bounded independent RGBA Program producer, rational deadlines, cached surfaces, reduced preview updates and process-memory/cadence evidence — functional worker tests and hosted measurements completed; cadence targets failed. 1080p60/hardware broadcast qualification remains open.
