# Workspace architecture

UI-037 conditionally renders live transport controls under Preview (PVW) and
Program (PGM) monitors strictly for sources with duration metadata (video files,
audio files, and timed procedural sources). Blank, black/color generators, color
bars, graphics, cameras, and live microphone capture hide the transport bar entirely.
The timeline bar supports interactive click-and-drag scrubbing with visual playhead
indicators, seek tooltips, and synchronized seeking across the engine clock, Media
Foundation video decoder, and audio clip cursors.

UI-036 keeps CPU composition and wgpu monitor display explicit, shares the
validated source renderer with the thumbnail cache, keys decoded payloads by
generation and immutable identity, and bounds worker-start/file-retry failures.
Core navigation excludes deferred capabilities; project schemas/dependencies
are unchanged. [ADR 0036](adr/0036-core-cpu-gpu-audit.md) records the decisions.


UI-035 keeps destination/format inspection and topic-specific settings in
`ui-egui::output_dialogs` / `settings_pages`. Engine mutations retain the existing
typed command boundary. Test-only software capture consumes actual egui meshes
and textures and exposes no production renderer or external API. See
[ADR 0035](adr/0035-reference-uiux-output-and-settings.md).

UI-034 adds the headless video-format/new-session constructor, schema-10 scan
and exact camera descriptors, UI-only New Preset drafts and bounded native mode
discovery. Program surfaces/monitor/snapshot orientation changes retain the same
pixel budget. Browser control reads length-delimited HTTP with two process-owner
permits. No dependency or layer change. See [ADR 0034](adr/0034-new-preset-and-native-input-formats.md).

UI-032 Add Input candidate: `ui-egui::add_input` owns source-page presentation,
explicit category/icon mapping, incompatible draft retirement and engine
precondition/readiness queries. PNG workers only publish a draft; only explicit
validated OK commits through AddInputAt. `add_smoke` is an opt-in hosted capture
workflow. No engine/platform/dependency/schema boundary changes. New hosted
acceptance is pending; see ADR0032 and [UI-032](ui032-add-input-changes.md).

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

2026-10-19 overlay caption continuation: `engine::overlay_text` owns the Program caption raster. It
holds an original 5x8 bitmap font (printable ASCII, no third-party font file or new dependency) and
the panel origin/size fractions shared with the compositor. `render::program_from_sources` applies
source processing, the transition, the overlay panel geometry, the enabled caption lines and then
fade-to-black, in that order, so every Program consumer sees the same picture; the operator monitor
no longer paints its own copy of the captions. Writes are bounds-checked, lines are capped at 96
characters and the text edge is 0.52w, so a malformed caption degrades instead of interrupting
output. Held under [ADR 0019](adr/0019-overlay-captions-in-program-surface.md); hosted acceptance is
pending for this commit. Remaining gaps: bitmap-only typography, ASCII-only glyphs, no animation or
data binding, no per-source overlay, and the synthetic `Graphics` input's decorative text still
being monitor-only.

2026-10-19 local video continuation: schema 8 persists the `VideoSource::File` descriptor.
`engine::video_source` owns a single-latest `VideoFeed` per source: one immutable Arc RGBA frame,
monotonic sequence, signed 100 ns source PTS, generation identity, health states and nonblocking
try-lock publication. `ui-egui::video_player` owns one bounded decoder thread per committed input
(global maximum eight live owner workers); the synchronous `!Send` Windows Media Foundation
`VideoFile` is created, read, seeked and dropped only on that thread, and frames are published at
exact PTS into the feed. The UI thread never decodes: `poll_video` snapshots feeds and copies the
newest `Arc` pixel buffer into the existing decoded-media (`custom_rgba`) cache, so the existing
effects pipeline and the fixed 640×360 CPU Program composite need no separate video path. Add Input
validates the chosen MP4 on a bounded probe worker and commits the descriptor only after the probe;
external-media inputs without a frame render black rather than a procedural fallback. Held under
[ADR 0018](adr/0018-operator-local-mp4-video-source.md) and accepted in hosted run 39273839106.

2026-10-19 audio continuation: schema 7 introduced typed audio-only WAV/microphone descriptors and
schema 8 carries them forward. Portable `engine::audio_engine` renders actual fixed 48 kHz stereo
blocks with Master/A/B and guest returns; `engine::wav` validates PCM16/float32 RIFF data.
`rustmix-platform::audio` encapsulates Windows WASAPI streams and bounded PCM rings behind an
explicitly started monitoring session. `ui-egui::program_output` presents the validated Program
texture in a separate native viewport with sequence-tagged screenshot acceptance. Descriptor and
Program-viewport workflows passed hosted run 39270445070; no audio device, physical display, A/V
synchronization or production output was qualified.

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
clock. Audio processing (48 kHz mixer, WAV/microphone descriptors and the WASAPI adapter) and local
video processing (Media Foundation decode into `VideoFeed`) now exist, but there is no A/V
synchronization, recording/encoder worker, hardware source worker or production output
qualification.
