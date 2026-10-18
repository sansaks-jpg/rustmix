# Workspace architecture

UI-009: source transport is owned by the headless engine; monitors consume those positions. The UI renders a bounded bank and processed source surfaces are composed before overlays/FTB. See [ADR 0006](adr/0006-stable-monitor-bank-and-source-transports.md) and the [engine audit](engine-audit.md) for the actual CPU/GPU/backend boundary.

See [UI architecture](ui-architecture.md) and [ADR 0001](adr/0001-native-demo-ui.md). Current package layers: engine L3, UI L4, app L5. Lower-layer concerns are modules inside the engine until needed by multiple packages. No empty planned crates exist.

All code forbids unsafe. Runtime core paths use fallible validation, checked IDs/revisions, bounded dimensions/files/inputs and explicit errors. Dependencies contain their own unsafe/FFI internally; this is not a claim that the entire dependency tree is unsafe-free.

This is an operator UI demo. There is no independent broadcast Program clock, recording worker, hardware source worker or actual audio/video synchronization.
