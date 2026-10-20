# Workspace architecture

2026-10-20 UI-020: operator layout changes stay in ui-egui. Toolbar sizing and channel orientation
are presentation concerns; input/audio/category mutations continue through typed engine commands.
No crate, dependency, project-schema or platform-backend change is introduced. Hosted acceptance
of this source candidate remains pending. See ADR 0020.

2026-10-19 continuation: schema 7 adds typed audio-only WAV/microphone descriptors. Portable
`engine::audio_engine` renders actual fixed 48 kHz stereo blocks with Master/A/B and guest returns;
`engine::wav` validates PCM16/float32 RIFF data. `rustmix-platform` encapsulates Windows WASAPI
streams and bounded PCM rings. The new `ui-egui::program_output` presents the validated Program
texture in a separate native viewport. These candidates need their own hosted runtime acceptance;
none establishes a complete recording/streaming backend, AV synchronization or device qualification.

Task 1.3 continuation (2026-10-19): `engine::output` owns four bounded external-output
configuration records and stable input references. Source and configuration commit through one
validated engine command/revision. Schema 6 adds the bank; schemas 1–5 receive defaults through
migration, while current-schema files must contain the bank. UI drafts do not alter Program,
Preview or output liveness. There is no external frame sink/device/transport worker. Source
implementation and prepared regression coverage only; new hosted acceptance pending.

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
