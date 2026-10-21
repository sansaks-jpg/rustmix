# UI-033 — Prioritized working inputs

2026-10-21. Source candidate; new hosted compilation/native acceptance pending.
Prior UI-032 accepted runtime source4850b0f/run39587449408 attempt3 remains the
baseline. This record does not promote source code to tested hardware support.

- Video: existing local MP4 Media Foundation decode, playback/pause/restart/loop,
  Preview/Program and saved project flow retained. MP4 audio is still absent.
- PNG: existing bounded alpha image import/check/preview/explicit OK retained.
- Photos: new alphabetical PNG/JPEG/BMP folder slideshow with explicit interval,
  check before OK, pause/restart/loop, independent feed and persistence.
- Browser: new real persistent isolated Edge/Chrome page, HTTP(S), configurable
  viewport, live captures capped5fps into Program, failure isolation and Start/Stop.
  Browser audio, interaction, local HTML and physical performance are not included.
- Camera: new Media Foundation discovery, stable selected identity, first-frame
  check and video source worker with operator-controlled Start/Stop. No automatic
  capture after reopening a project. Device/driver/permission errors stay visible.
  Generic hosted CI can only prove discovery/no-device behavior; physical capture
  remains experimental and requires hardware validation.
- Audio and Audio Input: existing WAV playback/mixer/48kHz route and WASAPI
  microphone descriptors retained. Audio I/O monitoring is explicitly started by
  the operator; adding an input never secretly starts capture. MP3, arbitrary
  sample rates and physical endpoint compatibility are not newly qualified.

Schema9 is required for new descriptors; older projects migrate, older binaries
reject new projects. No backend for DVD, NDI/SRT, PowerPoint, virtual sets, Zoom,
remote guest, replay, stingers or title templates is implied by this task.
See [ADR0033](adr/0033-real-prioritized-add-input-sources.md) for architecture,
licences, bounds, shutdown and dependency review.

New evidence required: all existing suites and16 native modes plus the mandatory
--ui-source-smoke, real PNG/JPEG/BMP changing frames and loopback HTML changing
Program pixels, source dialogs/screenshots/element metadata, camera enumeration,
project roundtrip and all GPU drain gates. Physical camera/audio hardware,
shutdown endurance and real-world browser latency remain outside hosted proof.
Local permitted checks: formatting, metadata/dependency inventory, script syntax
and diff inspection only. No local Rust compilation, tests, build or app launch.
