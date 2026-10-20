# Roadmap

- UI-021 candidate: implement explicit overlay draft decisions and truthful preview; refine Playlist list/transfer/footer geometry. Exact-revision hosted acceptance and screenshot comparison pending; GPU/cadence and remaining UI-020 gaps stay open. See ADR0021.

- UI-020 (2026-10-20): responsive operator proportions and dialog corrections are implemented
  with exact-code screenshot comparison. bec97a8 passed hosted check/Clippy/tests/build but failed
  native GPU shutdown drain. Next: qualify GPU reliability, preserve Overlay Settings drafts and
  truthful preview, then refine remaining Settings/Playlist/record/stream form differences. Full
  parity and production acceptance remain open. See docs/ui020-visual-review.md and ADR 0020.

- Task 1.3 continuation (2026-10-19): finish the existing four-slot external-output configuration
  slice, atomic source/config Apply, preserved drafts, schema-6 migration and validation tests.
  Source implementation only; new hosted build/test/native acceptance pending. No hardware or
  network output backend. Next broadcast blockers remain G01 Program delivery and G02 real audio.

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
- M1/UI-001: native interactive synthetic switcher, CPU monitor composition, dockable panels, simulated audio controls and overlay UI — experimental; custom GPU compositor remains open.
- UI-002: observed broadcast-style placement, native system typography, compact input controls and atomic direct takes — experimental prototype verified by hosted tests, native smoke and screenshot review; see docs/ui-progress.md.
- UI-003: sidebar-based source selector, input properties/audio/status, staged settings and denser monitor/input layout — experimentally verified by the hosted widget/native workflow; hardware qualification remains open.
- UI-004: bounded sidebar dialogs with fixed confirmation controls, independent generator samples, monitor properties access, readable input filters/offline state and unified removal confirmation — hosted compilation, widget geometry and native dialog captures passed; physical DPI/accessibility qualification remains open.
- Next: M1 reliability — independent timestamped Program frame delivery, deterministic CPU golden fixtures, GPU reference comparisons and malformed-input/fault coverage.
- M2: still images, local media and bounded source workers; add hardware capture only after device evidence.
- M3: real sample-based audio graph, routing, click-safe parameters and mix-minus.
- M4: recoverable recording and local test streaming after separate encoder/licensing review.
- M5+: complete automation schemas, hardware/accessibility/soak matrix, secondary outputs and packaging.

Later milestones in AGENTS.md are plans, not shipped capabilities.

- M2/UI-010: persisted still-image input lifecycle, staged effects, insertion order, duplication, relink and bounded PNG workers implemented. Hosted functional acceptance passed, including the current native PNG save/reopen workflow. File video/capture and hardware qualification remain open.

- UI-011: bounded independent RGBA Program producer, rational deadlines, cached surfaces, reduced preview updates and process-memory/cadence evidence — functional worker tests and hosted measurements completed; cadence targets failed. 1080p60/hardware broadcast qualification remains open.
