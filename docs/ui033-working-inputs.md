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

## Verification ledger

- f178755 / run39598705556: formatting, inventory, locked workspace check and
  warnings-as-errors Clippy pass. 283 tests pass,2 fail (158UI pass,2 fail).
  Failures are old expectations: Photos was unavailable, decoder errors cleared
  all pixels. Release/native did not run. Assertions were updated to require the
  real Photos form with unvalidated OK disabled, and exact retained pixels plus
  explicit failure status. A red held-frame tile warning was added.
- a190375 / run39602452707: all285 tests pass, build/native pending.
- 8fa671b / run39602671522: stopped sources invalidate monitor pixels; superseded
  by20ea0b6 and cancellation requested. It is not accepted verification evidence.
- 20ea0b6 / run39603068866: source labels corrected, local-only photo descriptors
  strengthened, Browser restart reloads its persistent page; exact checks pending.

- a190375 native source stage3 failed: canonical Windows photo paths retained
  the verbatim-disk prefix and were rejected as non-local. No acceptance claimed;
  later native processes were not run. f6ecbdd adds disk-prefix normalization,
  rejects verbatim network/device identities, and adds a real mixed PNG/JPEG/BMP
  folder plus corrupt-file regression. No assertion or native gate was removed.
-20ea0b6 was superseded after this reproducible path failure; cancellation
  requested rather than spending another native run on the same known defect.
