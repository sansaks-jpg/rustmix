# Workspace architecture

UI-012: `engine::production` owns transition slots, named playlists and bounded automatic input
sequencing; `engine::shortcuts` owns portable binding validation and typed action resolution.
The UI stages keyboard preferences and dispatches playlist commands. Project schema 5 stores
configuration only (the sixteen-channel overlay/stinger bank since the Task 1.1 port). The existing Program producer acknowledges its engine state with a control
generation; the UI reads that state before commands and does not resubmit snapshots every paint.
Old-generation frames cannot undo newer controls. The Basic/Advanced flag is UI presentation
stored separately from project/engine state and multiview selection; it does not publish a new
Program snapshot. See ADRs 0009/0010 and the accepted functional run in docs/ui-progress.md.
Performance and full reference acceptance remain open.

UI-009: source transport is owned by the headless engine; monitors consume those positions. The UI renders a bounded bank and processed source surfaces are composed before overlays/FTB. See [ADR 0006](adr/0006-stable-monitor-bank-and-source-transports.md) and the [engine audit](engine-audit.md) for the actual CPU/GPU/backend boundary.

See [UI architecture](ui-architecture.md) and [ADR 0001](adr/0001-native-demo-ui.md). Current package layers: engine L3, UI L4, app L5. Lower-layer concerns are modules inside the engine until needed by multiple packages. No empty planned crates exist.

All code forbids unsafe. Runtime core paths use fallible validation, checked IDs/revisions, bounded dimensions/files/inputs and explicit errors. Dependencies contain their own unsafe/FFI internally; this is not a claim that the entire dependency tree is unsafe-free.

This is an operator prototype with a bounded RGBA Program producer, not a qualified broadcast
clock. There is no recording worker, hardware source worker or actual audio/video synchronization.
