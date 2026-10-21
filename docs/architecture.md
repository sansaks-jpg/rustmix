# Workspace architecture

UI-034 adds the headless video-format/new-session constructor, schema-10 scan
and exact camera descriptors, UI-only New Preset drafts and bounded native mode
discovery. Program surfaces/monitor/snapshot orientation changes retain the same
pixel budget. Browser control reads length-delimited HTTP with two process-owner
permits. No dependency or layer change. See [ADR 0034](adr/0034-new-preset-and-native-input-formats.md).

UI-028 through UI-031 (2026-10-21): Video-first Input Select with genuine bounded
recent files and validation; topmost Escape/popup/repeat safety; responsive
default widths preserving real resizing; Input Settings expanding fields,
aligned labels/flags/colour/fit/metadata and left source caption.
Video history is bounded application preference state,
selection reuses the existing probe and only OK dispatches AddInputAt. Escape
uses existing cancellation. A validated width cache preserves operator resizing.
General uses shared reserved labels and existing draft/typed actions. Opt-in
core, dialog and large-layout checks are independent mandatory native processes
loading the saved project; per-process state/readback/drain/deadline gates remain.
No backend/layer/dependency/project or dock-format change.
Source 56048cb/run 39556368477: locked check/Clippy, 274 tests (150 UI), release, all15 native processes and packaging pass.
See ADRs0028-0031 and UI-028/029/030/031 reviews for exact scope and every failed candidate.

2026-10-21 UI-024 through UI-027: caption/geometry/category presentation remains
inside ui-egui. The UI retains actual selected source categories instead of reconstructing
them from a shared render kind; existing validated source commits still dispatch typed
engine commands. Production default/reset dock proportions do not overwrite saved layouts.
Native reference capture restores its observed 1600x900 viewport before later processes
reuse the disposable window store. No layer/dependency/schema/backend change. See ADRs
0024-0027 and ui027-native-acceptance.md for exact verification and unresolved GPU scope.

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
