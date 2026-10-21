# ADR 0018: operator-addable local MP4 video source

2026-10-19 · hosted native acceptance passed in [run 39273839106](https://github.com/sansaks-jpg/rustmix/actions/runs/39273839106)
(commit `1df8c47`; all 16 hosted steps green). `--ui-video-smoke` /
`--ui-video-smoke-reopen` produced `video-validation.json` (160×90, 30/1, Windows Media Foundation
RGB32, no audio/device), 1600×900 `video-source.png` / `video-program.png` / `video-resumed.png`
with the decoded quadrant fixture, `video-motion.json` (`paused_frame_frozen: true`,
`resumed_frame_differs: true`, 4224 changed channels) and byte-identical `video-workflow-ok.json` /
`video-reopen-ok.json` persisting schema 8. Follow-up hardening passed hosted
[run 39279752517](https://github.com/sansaks-jpg/rustmix/actions/runs/39279752517) (commit `695d2ef`,
all steps green): relinking clears the previous file's decoded frame, Program surfaces are keyed by a
raster generation instead of an `Arc` address, a cancelled Add dialog retires its probe, operator
screenshots are tagged with their viewport and no longer claim decoded pixels, the probe accepts an
unreduced 30 fps ratio, and the save stage waits for the committed revision. The non-claims below
remain in force.

The owner's delivery plan requires real video inputs before recording or streaming can be honest.
The engine already had a typed `VideoSource::File` descriptor (schema 8) and a bounded
`VideoFeed`, and `rustmix-platform` already had an audited synchronous Media Foundation RGB32
decoder, but nothing connected an operator action, a decoder owner worker and the Program
compositor. A hand-edited descriptor could even present the input's procedural fallback picture,
which violates the never-show-the-wrong-camera rule.

Decision:

- `crates/ui-egui/src/video_player.rs` owns one decoder thread per committed video input. The
  synchronous, `!Send` Media Foundation reader lives and dies on that thread. Frames are paced by
  exact 100 ns presentation timestamps, paused without losing the anchor, looped through a real
  seek at end of stream, restarted only on an explicit restart generation, and published as
  immutable `VideoFrame`s into the engine's single-latest `VideoFeed`. A bounded global permit
  limits abandoned owner workers to eight; provenance/health/error text is sanitised and bounded.
- The UI keeps a `VideoSlot` per input (path, feed revision, last sequence, last control tuple).
  Slots are created when a committed descriptor is online, updated from the engine transport
  clock, retired when the descriptor is removed or relinked, and restarted only for an explicit
  restart. The UI thread never decodes, opens Media Foundation, or reads a sample; `poll_video`
  only snapshots feeds and copies the newest `Arc` pixel buffer into the existing `InputFx`
  decoded-media cache.
- The Add Input dialog gains an appended `Video file` category (index 20 so observed reference
  category ordinals do not move). Validation opens the file on a bounded probe worker, reports the
  actual decoded raster/frame rate, and the descriptor is committed only after that probe. MP4
  audio tracks are not decoded, no device/camera/network source is opened, and the page says so.
- Program composition needed no new path: the UI's existing decoded-media (`custom_rgba`) surface
  and effects pipeline already feeds the fixed 640×360 CPU Program composite. `render::pixel`
  now returns black for video-file (and all external-media) inputs in the headless reference
  generator, so a missing frame can never look like a procedural source. Offline inputs show black
  plus their warning; decoder failures show `VIDEO UNAVAILABLE` in the tile and in Input Settings →
  Source status.
- Native acceptance `--ui-video-smoke` adds the generated fixture through the real dialog, asserts
  the probed 160×90/30 fps format, takes it to Program, verifies the fixture's red/green/blue/white
  quadrant pixels, verifies that pausing freezes the Program picture and resuming changes it,
  captures GPU screenshots, saves the project, reopens it and re-verifies decode from the persisted
  descriptor. `scripts/ci-video-fixture.ps1` remains an external developer-only oracle; FFmpeg is
  not a runtime dependency.

Non-claims: this first slice is Windows Media Foundation MP4 video only. It does not decode MP4
audio, does not synchronize A/V, does not provide cameras, capture cards, network or browser
inputs, GPU decode, hardware acceleration, frame-accurate seek UI, a native relink acceptance stage
(relinking a committed descriptor now clears the previous file's decoded frame and is covered by a
UI unit test), or sustained/endurance measurement. The Program raster remains the experimental 640×360 CPU
composite; hosted acceptance does not qualify physical hardware, codec compatibility beyond the
generated fixture, or broadcast readiness. Codec/patent and distribution questions for shipped
Windows Media components still require release review.
