# Roadmap

- UI-012: reference-driven production controls — persisted independent transition slots and
  twelve CPU effects, named engine playlists with per-item transitions/holds and offline handling,
  editable keyboard bindings, schema 3 migration and acknowledged producer state. Source
  implemented; hosted compile/lint/unit/widget/native/reopen/performance review pending.
  Full capability map: docs/reference-feature-comparison.md. Media/audio/output milestones remain open.

- UI-009: stable bounded monitor bank, horizontal T-Bar, independent headless source transport, consistent processed Program surfaces and truthful output/processing status — tested on hosted Windows: 32 tests plus native screenshot/save/reopen acceptance in [run 39089212707](https://github.com/sansaks-jpg/rustmix/actions/runs/39089212707). Complete vMix feature parity and production engine qualification remain separate uncompleted milestones.

- M0/UI-001: three-package workspace, headless commands, rational rates, synthetic media, project IO and hosted Windows CI — implemented, verification tracked in docs/ui-progress.md.
- M1/UI-001: native interactive synthetic switcher, CPU monitor composition, dockable panels, simulated audio controls and overlay UI — experimental; custom GPU compositor remains open.
- UI-002: observed broadcast-style placement, native system typography, compact input controls and atomic direct takes — experimental prototype verified by hosted tests, native smoke and screenshot review; see docs/ui-progress.md.
- UI-003: sidebar-based source selector, input properties/audio/status, staged settings and denser monitor/input layout — implemented in source; CI build, native-dialog screenshot and smoke confirmation pending.
- UI-004: bounded sidebar dialogs with fixed confirmation controls, independent generator samples, monitor properties access, readable input filters/offline state and unified removal confirmation — source implemented; hosted compilation, widget tests and nine native dialog screenshots pending.
- Next: M1 reliability — independent timestamped Program frame delivery, deterministic CPU golden fixtures, GPU reference comparisons and malformed-input/fault coverage.
- M2: still images, local media and bounded source workers; add hardware capture only after device evidence.
- M3: real sample-based audio graph, routing, click-safe parameters and mix-minus.
- M4: recoverable recording and local test streaming after separate encoder/licensing review.
- M5+: complete automation schemas, hardware/accessibility/soak matrix, secondary outputs and packaging.

Later milestones in AGENTS.md are plans, not shipped capabilities.

- M2/UI-010: persisted still-image input lifecycle, staged effects, insertion order, duplication, relink and bounded PNG workers implemented. Hosted acceptance pending. File video/capture and hardware qualification remain open.

- UI-011: bounded independent RGBA Program producer, rational deadlines, cached surfaces, reduced preview updates and process-memory/cadence evidence — implemented, hosted verification pending; 1080p60/hardware broadcast qualification remains open.
