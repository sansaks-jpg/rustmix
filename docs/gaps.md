# Known gaps

UI-040 (2026-10-24) completes the bounded follow-up to the UI-039 reference
alignment. Eight plan items landed, eight families were reviewed and deferred,
and the stale documentation rows were reconciled:

- **Standard overlay channels 1–8 bind inputs (PiP):** a serde-default
  `OverlayBank::inputs` array, the validated `overlay.input.set` command and a
  CPU render path compositing the bound input under the existing caption text.
  Old project files load unchanged; input removal clears bindings. Evidence:
  engine overlays suite 14 passed (5 new integration + 3 new unit tests).
- **Wire-verified overlay/transport functions on the HTTP control API:**
  `OverlayInput1–8` (+`Off`) map onto the binding command, `Stinger1–8` onto the
  existing stinger trigger, `OverlayInput9–16` are rejected with the
  "channels 9-16 are Stingers 1-8" reason, and `PlayPause` now works by reading
  the engine's public per-input clock (`Engine::playback` /
  `PlaybackClock::paused`) and dispatching the existing pause/play commands.
  The pre-UI-040 catch-all that rejected the whole `OverlayInput*`/`Stinger*`
  family is gone. Evidence: automation api_contract 14 passed (9→14).
- **Per-input and per-bus meters:** the engine computes per-source post-gain
  pre-clamp `SourceMeter` taps (`AudioMetrics.sources`) and the audio session
  publishes them; the mixer strips render measured input and bus meters with
  the `audio.input.<n>.meter` / `audio.bus.<n>.meter` automation IDs, and the
  Overlay panel gained per-channel input assignment controls
  (`overlay.channel.<n>.input`) with a readable disabled reason. Meters show
  honest zeros while monitoring is off — levels are never faked. Evidence:
  audio_samples 9 passed (incl. the per-source tap impulse test), UI suite 188
  passed, hosted `--ui-audio-smoke` exit 0 with its new stage-14 meter
  assertions and artifacts/overlay-panel.png inspected for the new row.
- **Reference-corrected details:** preset category labels renamed to the
  verified catalogue names Aqua/Blue (ids and colour values unchanged, so
  existing projects load), and golden-frame tests now pin every one of the 20
  transition effects' motion curves at 25/50/75% (transitions suite 8 passed
  incl. `golden_frames_pin_every_effect_curve`).
- **Deferred (owner none, deliberate):** network sources (NDI/SRT/RTSP/WebRTC),
  capture-card backends, recording/streaming/replay backends, browser inputs
  and a web controller, VST3 hosting, PTZ/data sources/virtual sets/GT title
  runtime, the TCP text protocol, and colour correction/LUT. Reasons: licence
  review required (AGENTS §2.2), no clean-room behaviour spec, no headless test
  target, or no backend without which a start/stop control would fake live
  output. See the deferred table in [feature-matrix](feature-matrix.md).
- **Verification:** the workspace gate passed on 2026-10-24 — `cargo fmt --all
  -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --all-targets --locked` (360 passed, 0 failed, 1
  opt-in ignored: engine 147, platform 11, automation 14, UI 188) and
  `cargo build -p rustmix --locked`. The UI-039 note about seven failing UI
  tests from an in-flight dock edit is resolved: the dock now includes the
  Overlays panel again and all 188 UI tests pass.
- **No ADR was needed:** no crate boundary, dependency edge, safety strategy or
  persistence schema changed. The overlay binding is a serde-default field on
  the existing bank (no schema bump), the meters extend the existing
  post-gain pre-clamp measure-and-clip contract, the control-function mapping
  encodes re-verified wire semantics as data, and PlayPause reuses existing
  commands. The one naming choice (bank-level parallel array instead of a field
  on `OverlayState`) is a reversible implementation detail forced by ui-egui
  struct literals, not an architecture decision (AGENTS §12.2).
- **Reference re-verification recorded:** the re-verification pass confirmed
  14 claims (grouped into seven recorded corrections: OverlayInput1–8 only,
  Off clears while Out does not, SetLayer without Alpha/Rotation, no
  `<fullscreen>` output child element, Aqua/Blue catalogue names, five
  transport setup|arm pairs, and the further document-internal contradictions)
  in the inventory: [reference-audit.md](reference-audit.md) and the
  `reverification` object in [reference-audit.json](reference-audit.json).

UI-039 (2026-10-24) aligns the fundamental switcher feature set with the verified
reference study (installed vMix 29.0.0.48, SHA-256 `05a67e53c184d8547d2668cac3814dce34cd40e4cb0fa3e6df28acdd5a57fad5`;
existing mapping in `rustmix-reference/vmix` re-verified against the binary — identity,
777 control functions, 20 transition effects, 40 input types, 16 overlay channels,
4-slot defaults and the 9-bus audio matrix all confirmed):
- Transition catalog completed from 12 to the reference 20 effects, adding Zoom, Fly,
  CrossZoom, FlyRotate, Cube, CubeZoom, Merge and AlphaFade as shared per-pixel CPU
  renderers (exact endpoints by construction; cube family passes through an honest
  edge-on black midpoint). Fresh sessions now open with the reference default slots:
  Fade 500 ms, Merge 1000, Wipe 1000, Merge 1000.
- Audio routing widened from 3 to the reference 9 buses (Master, A–G, Headphones):
  per-input nine-way sends, mix-minus guest returns on any bus, external outputs
  routable to any bus, project schema 11 migration pads schema ≤10 files with idle
  defaults while preserving every authored route.
- Mixer shows Master always; other bus strips surface only when routed to an external
  output or touched (reference behavior). Beyond Master, bus strips hold honest zero
  meters — per-bus metering is not implemented. Full HD performance of the eight new
  per-pixel effects is not yet benchmarked (the opt-in release benchmark still covers
  the original effect set).
- Verification: 128 engine tests, 11 platform tests (with generated MP4 fixture) and
  177 of 184 UI tests pass; strict Clippy and fmt clean; `rustmix.exe` debug build OK.
  The 7 failing UI tests are a **pre-existing in-flight breakage, not from this
  change**: an uncommitted edit starts the dock as `DockState::new(vec![Panel::Inputs])`
  without `Panel::Overlays`. Proven by positive control — applying only that one line
  at HEAD `abc2473` (where all 184 UI tests pass) reproduces the same failures. The
  line and the failing tests are left for the owner's in-flight work.

UI-038 (2026-10-22) completes workspace regression audit, engine zero-copy optimization, and Full HD 60 FPS performance verification:
- 321 workspace tests pass (126 engine, 11 platform, 184 UI), zero failures, zero warnings on strict Clippy.
- Full HD 1080p 60 FPS CPU composition benchmarks verified: all render operations (Cut 1.66 ms, Overlay 3.11 ms, Downscale 4.57 ms, Wipes/Slides 2.15–2.40 ms, Fade 6.85 ms, Procedural 0.18–7.30 ms) complete within <50% of the 16.67 ms frame budget.
- Clean release executable built (`rustmix.exe` 36.2 MB) without test harness overhead.
- Native Full HD Program output window (`program_full_texture`, 1920×1080) for sharp second-display delivery.
- Sharp high-resolution Preview monitor texture decoupled from 384×216 card thumbnails.
- Elevated Media Foundation decoding and PNG loaders to Full HD (1920×1080), activating engine zero-copy optimization for 1080p video feeds.
- See [core engine audit](core-engine-audit.md).

UI-037 aligns PVW and PGM monitor transport controls with professional switcher (vMix)
behavior: playback transport (play/pause, restart, progress/seek bar, timecode) is
hidden for blank, black, color generators, color bars, graphics, live cameras, and
microphones. Video/audio sources with duration metadata display transport controls
and support interactive click-and-drag timeline scrubbing.

UI-036 narrows normal navigation to core workflows. Deferred source/output/settings
entry points are hidden; their old menu presence is not feature readiness. CPU
composition/GPU display and actual monitoring are separate from hardware encode,
record/stream, A/V sync and endurance. See [current audit](core-engine-audit.md).


UI-035: existing-panel corrections are reviewed with headless egui captures and
regression tests. Native Windows, GPU/DPI, encoders, devices and full reference
equivalence remain separate gates. Reference-only companion tools are absent;
unsupported configuration stays explicitly unavailable. See
[scope and verification ledger](ui035-reference-uiux.md).

UI-034 (2026-10-21): startup/New format selector, portrait monitoring, persisted
scan and native Camera format selection are unverified source candidates.
Browser keepalive discovery fix awaits hosted reproduction. Prior f6ecbdd run
39603431352 failed Browser discovery and GPU teardown. No 60fps performance,
full-HD/interlaced output, physical camera/audio or GPU recovery claim is made.
See [current implementation/verification](ui034-add-input-and-new-preset.md).

UI-032 source candidate (2026-10-21) addresses selected Add Input audit findings
with isolated PNG drafts, separate forms, inspectable unavailable pages, explicit
icons and OK readiness. **New runtime/visual acceptance is pending**; no finding
is promoted to tested based on the old green run. Solid/transparent sources,
template loaders/galleries, recent Image/WAV, device/network backends and physical
DPI/accessibility qualification remain open. See [UI-032](ui032-add-input-changes.md).

Add Input audit (2026-10-21): **whole-dialog reference correspondence is not
accepted**. [Focused audit](add-input-uiux-audit.md) reviews all 33 supplied
Add Input images, exact matching runtime source and hosted evidence. Six open
findings cover PNG draft carryover into other categories, generic Image forms,
Colour/Title semantics, missing/unreachable category pages, inconsistent OK
readiness and ordinal-based icons. This is an audit, with no application fix or
new runtime test. Existing selected Video/Cancel/descriptor acceptance remains
bounded to its documented scope.

UI-028 through UI-031 (2026-10-21): Video-first Input Select with genuine bounded
recent files and validation; topmost Escape/popup/repeat safety; responsive
default widths preserving real resizing; Input Settings expanding fields,
aligned labels/flags/colour/fit/metadata and left source caption.
Source 56048cb/run 39556368477: locked check/Clippy, 274 tests (150 UI), release, all15 native processes and packaging pass.
Exact selected native images/geometry reviewed.
Earlier e3b12a9 video drain/panic,98bf2eb main deadline and fa5e4ba
deadline/drain/panic remain recorded. Later green runs prove no causal GPU fix.
Missing capture/network/encoder capabilities remain unqualified.
Full UI/workflow correspondence, original source-icon semantics, remaining
source/property/preview dimensions, GPU reliability, cadence and physical/
DPI/endurance qualification remain open. Light theme is deprioritized.
See ADRs0028-0031 and UI-028/029/030/031 reviews for exact scope and every failed candidate.

UI-024 through UI-027 selected corrections verified on ba3b209/run 39540613350:
261 tests (137 UI), check/Clippy/build and all 13 native workflows pass; exact native
workspace/dialog/Video images and viewport restoration reviewed. Full UI correspondence
remains open: source-page structure/initial Generators choice, remaining typography,
spacing and dialog navigation. Light theme is deprioritized by user instruction.
acb00d7/e44050b input-stage24 readback/drain failures remain in the ledger; later green
runs establish no causal GPU-driver fix. Short 50/59.94/60 fps samples still miss targets
at 640x360. Missing encoder/capture/network backends, physical devices, DPI and endurance
remain unqualified. See ui024/ui025/ui026 reviews and ui027-native-acceptance.md.

UI-023 selected corrections accepted for 126291d/run 39452377970:251 tests including 127 UI,
check/Clippy/build and 13 native processes pass; exact workspace/Recording PNG/JSON reviewed.
Monitor/take alignment, compact effect containment, single-format recording navigation,
12 px form gutter and 339.6 px filename field are verified. Recording configuration remains
unsaved/read-only and Start unavailable. Full visual equivalence remains open: Light
vector/caret contrast, dialog title bars/footer sizes/typography and matching-viewport
DPI qualification. Hosted native slowdown reproduces, cadence misses targets, and earlier
intermittent GPU failure still has no demonstrated causal fix. See ui023-visual-review.md.

UI-022 outcome:7aeac9c/run39434584384 passes check/Clippy/all tests (121 UI), build and complete native workflows. Exact Settings/Playlist/Overlay captures accept selected dimensions/alignment, not full equivalence. Earlier GPU drain failure remains unexplained;50/59.94/60cadence targets still fail. Header/footer placement/typography and advanced backends remain open. UI023 Recording source is isolated and untested. See ADR0022/ui022-visual-review.md.

UI-021 outcome: be79b2e passes hosted check/Clippy/all tests (115 UI) and build, with corrected Overlay labels/contained preview/full-width Playlist rows verified in exact screenshots. Main native GPU shutdown fails (Wait timeout, Queue::drop panic, exit-1073741819); later workflows do not run. Full parity, Settings proportions, Playlist dimensions, advanced overlays and cadence remain open. See ADR0021 and ui021-visual-review.md.

2026-10-20 UI-020: code bec97a8 passed hosted check/Clippy/all tests (106 UI) and build, with
all main native workspace/dialog captures inspected against supplied references. Main native
shutdown failed its GPU drain proof; later workflow stages did not run. Same-size texture reuse
is not a proven GPU hang fix. Full UI equivalence remains open: Overlay Settings draft/preview,
Settings proportions, Playlist controls, record/stream layout and disabled action affordance.
See ADR 0020 and ui020-visual-review.md for exact evidence and remaining measurements.

2026-10-19 · Program overlay captions (experimental; hosted unit/native acceptance pending for the
current commit): enabled overlay titles and subtitles are now rasterized into the Program surface by
the engine compositor, after the panel geometry and before fade-to-black, so the separate Program
window, fullscreen output, GPU captures and future recorders show the same words as the operator
monitor; the duplicated monitor-only `egui::Painter` captions were removed. Glyphs come from an
original 5x8 bitmap font in `crates/engine/src/overlay_text.rs` (no third-party font, no new
dependency or lockfile change), with 96-character lines, a 0.52w text edge, bounds-checked writes
and blank advances for unknown characters. Remaining gaps: bitmap-only typography (no font choice,
bold/italic, outline, shadow, background box, alpha ramp or animation), printable ASCII only, no
ticker/data binding, no per-source overlay, stinger channels still composite no clip (G20), and the
synthetic `Graphics` input's decorative text is still painted only by the operator monitor. Captions
are composited into the experimental 640×360 CPU Program raster only; no GPU, hardware, encoder or
physical-output qualification is implied. See
[ADR 0019](adr/0019-overlay-captions-in-program-surface.md).

2026-10-19 · local MP4 video source (experimental, hosted-accepted slice): the schema-8
`VideoSource::File` descriptor, one bounded Media Foundation decoder owner thread per input,
exact-PTS pacing, engine `VideoFeed`, off-thread Add Input probe and black external-media fallback
are implemented. Hosted Windows
[run 39273839106](https://github.com/sansaks-jpg/rustmix/actions/runs/39273839106) passed every
step: probe 160×90 at 30/1, quadrant pixels in the 1600×900 Program capture, pause freeze and
resume motion (4224 changed colour channels), GPU captures of source/Program/resumed views and
byte-identical schema-8 save/reopen. Follow-up hardening (relink clearing, generation-keyed Program
surfaces, probe-cancel retirement, honest operator-screenshot evidence, unreduced 30 fps ratio and
root-viewport filtering) then passed hosted
[run 39279752517](https://github.com/sansaks-jpg/rustmix/actions/runs/39279752517) (commit `695d2ef`,
all steps green). Non-claims: MP4 video-only (the audio track is not decoded),
no A/V synchronization, no cameras/capture cards/network/browser inputs, no GPU or hardware decode,
no frame-accurate seek UI, and no native acceptance stage for relinking (a committed relink now
clears the previous file's last decoded frame and is covered by a UI unit test). Only the generated 160×90 H.264
baseline fixture is qualified; the Program raster remains the experimental 640×360 CPU composite
and physical hardware is unqualified. See [ADR 0018](adr/0018-operator-local-mp4-video-source.md).

2026-10-19 · audio path (experimental, hosted-accepted descriptor workflow): the fixed 48 kHz
stereo block mixer, strict PCM16/float32 WAV decoder, typed WAV/microphone descriptors and Windows
WASAPI adapter exist, with an explicitly started local monitoring session. Hosted Windows
[run 39270445070](https://github.com/sansaks-jpg/rustmix/actions/runs/39270445070) passed every
step: a generated one-second 44.1 kHz mono PCM16 WAV was validated through the real Add Input
dialog, and `audio-validation.json` records `audio_monitoring_started: false`. The Program viewport
and its deferred screenshot delivery were validated in the same run. Non-claims: no audio
device/callback or physical hardware acceptance, no A/V synchronization, no compressed audio, no
mix-minus hardware qualification, and the main mixer's animated meters remain simulations while
the Audio I/O dialog reports actual Master block peak/RMS. See
[ADR 0015](adr/0015-real-audio-and-independent-program-window.md).

Task 1.3 continuation (2026-10-19): four external-output configuration slots and staged editing
are implemented in source, with schema 6 and atomic source/config changes. New regression tests
are prepared, not executed in this session. G05 remains open: configuration is not an SDI,
virtual-camera or network output backend, and External Start stays unavailable. Device port is a
hardware connector placeholder (Default or a numbered connector), not a network port. No new
hardware/cadence qualification is claimed. See [ADR 0014](adr/0014-external-output-configuration.md).

The [complete reference audit](reference-audit.md) enumerates all observed areas, input/effect/
overlay/audio contracts, UI workflows and public function names. Full-match acceptance is false:
major media/audio/output/API backends are absent, and hosted cadence/hardware qualification
does not pass. The Basic-to-Multiview alias is corrected and functionally verified in hosted run 39111788801; complete interface parity is still partial.

UI-012 (2026-10-18), experimental functional acceptance: the fixed-key limitation in G07 is
addressed by persisted remapping, and UI-owned playlist/slot state moves into the headless engine.
Four independent transition slots, twelve CPU effects, named playlists, worker acknowledgment and
Basic/Advanced passed hosted run 39111788801: 75 tests, native capture and save/reopen.
G01 still needs production-clock/output qualification; hosted 50/59.94/60 cadence targets failed.
G02–G05, hardware/API/accessibility/endurance and full parity remain open.
See [reference comparison](reference-feature-comparison.md) for capabilities not implemented here.

Task 1.1 port (2026-10-18): the schema-5 sixteen-channel overlay/stinger bank,
`overlay.stinger.*` commands, exactly-once cut-point Program/Preview exchange and the channels
1-8 UI are implemented and covered by engine/UI tests; hosted native acceptance passed in run
39129117131. Stinger clip compositing, `display_underneath_overlays` rasterisation and
per-channel effects/timing remain open (G20).

Task 1.2 port (2026-10-18): the typed `InputCategory` contract (`All`, six preset colours,
`Custom(1..=16)` clamping, filter-only `Search`) and the `ProjectState::visible_inputs` engine
query are implemented over the existing schema-5 definition ids, with the tile grid filtering
through the engine; local headless verification passed and hosted acceptance for this commit is
pending. No control endpoint exists, so no wire ordinals and no vendor category identity are
claimed (see the HTTP/API row in the reference audit); the text matcher stays in the UI (ADR 0013).

UI-009 audit (2026-10-18): the UI-008 GPU/output labels were not evidence of media backends. Composition/effects are CPU reference code, wgpu presents UI textures, recording/stream/external start paths were counter simulations, and real video decode/audio/hardware output remain absent. UI-009 prevents false live output state and repairs existing transport/composition/layout paths. It does not close G01–G05 or establish production readiness.

2026-10-17 · UI-001. No feature is production-ready.

| ID | Priority | Gap / next action |
|---|---|---|
| G01 | P0 for broadcast | Program is a native UI demo, without a dedicated real-time output scheduler. Introduce immutable frame metadata and an independent Program consumer before live claims. |
| G02 | P0 for broadcast | Audio is partially implemented: a real 48 kHz sample mixer with the nine-bus schema-11 matrix (Master, A–G, Headphones), per-input nine-way sends, mix-minus guest returns, strict WAV decode, typed WAV/microphone descriptors and a Windows WASAPI adapter with an explicitly started monitoring session. Per-source post-gain pre-clamp meter taps feed measured input/bus meters in the mixer strips (UI-040); the mixer no longer shows only simulated levels while monitoring runs. No audio device or callback is hardware-qualified, no A/V synchronization or mix-minus hardware test exists, no loudness/true-peak metering, and the headphone bus has no dedicated device output. Next: physical device callback/underrun soak, resampling quality review and A/V clock alignment. |
| G03 | P1 | No custom GPU compositor or CPU/GPU golden parity. Current CPU demo uploads to egui's wgpu surface. |
| G04 | P1 | UI-010 bounded PNG ingest/persistence/relink/source edits passed hosted acceptance, including run39111788801. Local MP4 video decode now exists and is experimentally accepted in hosted run 39273839106 (generated 160×90 H.264 fixture only; video-only, no MP4 audio). Camera/device capture and network ingest remain absent. |
| G05 | P1 | No encoders, recoverable recording or network streams. Controls remain disabled. |
| G06 | P1 | Crash recovery is manual checkpoint/backup loading; abrupt power-loss durability and filesystem fault injection need dedicated tests. |
| G07 | P2 | Validated custom key/modifier remapping is implemented and verified in UI-012. Full per-widget automation metadata, command JSON schemas and accessibility review remain incomplete. |
| G08 | P2 | Minimum 1100×700; dense panels use scrolling. Broader high-DPI/multi-monitor/physical-device QA remains needed. |
| G09 | P2 | FTB/active transitions reset at load. Demo countdown loops every minute; no editable production countdown. |
| G10 | P1 | Unsigned portable prototype only. No installer, SBOM standard, signing, upgrade/rollback or release audit. |
| G11 | Closed for UI-005; P2 residual | Bounded dialog layout, navigation, footer visibility, monitor header controls, input/tile states and the UI-005 main-window fidelity passed hosted compilation/widget execution and native screenshot review at 1100×700, 1600×900 and 1920×1080 (hosted Windows run 39046367904, commit `a19fc03`, all steps green). Logical 734×466 widget coverage is prepared; physical 150% DPI and physical-device screenshot review remain unverified. |
| G12 | Closed in UI-006 | Bounded `Overlay Settings` dialog (`dialog.overlay.*`) implemented for slots 1–4 with staged per-slot edits, live 16:9 normalized preview (`overlay_panel_rect`), and access from the Overlays panel, bottom `Overlay ⚙` button (`output.overlay.settings`), `☰` menu, and `Settings ▼` caret. Full-frame PIP/chroma overlay types and per-slot transition durations remain future engine work. |
| G13 | Closed in UI-006 | Per-input audio routing (`InputPage::Audio`) is now directly reachable from Audio Mixer input strip `⚙` buttons (`audio.input.<id>.settings`), `"Audio Settings…"` on `Input Properties → General` (`dialog.properties.audioSettings`), and source tile context menus. |
| G14 | Closed in UI-006 | Dialog sidebar buttons (`sidebar_button`) now render left-aligned labels with 16×16 colored badges and original procedural `egui::Painter` vector glyphs (`Grid`, `Geometry`, `Colour`, `Title`/`Overlay`, `Workspace`, etc.), eliminating centered-text overlap. |
| G15 | Closed in UI-006 | `egui_dock::Style` now uses broadcast-dark tab bar surfaces, `TEAL` active/focused outlines, and high-contrast active tab typography without altering tab geometry. |
| G16 | Closed in UI-006 | `Multiview` now offers two bounded layout presets (`Split: Monitors + Grid` and `Full Source Grid`) via `multiview.preset.split` and `multiview.preset.grid`. |
| G17 | Closed in UI-006 | All eleven private reference screenshots (`Interface.png`, `Add Input.png`, `Input Settings.png`, `Settings.png`, `Overlay Settings.png`, `External.png`, `Multicoder.png`, `Playlist.png`, `Recording Setup.png`, `Shortcut.png`, `Streaming Setup.png`) were reviewed read-only and documented in [ui-reference.md](ui-reference.md). Unimplemented output/encoder dialogs remain disabled by design. |
| G18 | Closed in UI-006 | Historical UI-006 label adjustment. Current tile control uses Audio with selected color for unmuted simulation; this is not real device audio or live-output status. |
| G19 | Closed in UI-006 | Top toolbar `Open ▼`, `Save As ▼`, and `Settings ▼` carets now expose real actions (open, reload last saved, recover checkpoint, write immediate recovery checkpoint, and jump directly to individual Settings pages or Overlay Settings). |
| G20 | P1 | Task 1.1 stinger channels (9-16) are configuration and Program/Preview bus exchange only: no stinger clip raster, no `display_underneath_overlays` compositing, no per-channel effects/alpha/timing, and no dedicated stinger UI (configured through `overlay.stinger.*` commands). Clip rendering depends on the media layer (G04). |

Do not claim 1080p/4K processing based on the project-format selector; actual demo rasters are capped and only displayed in the native UI.

UI-009 acceptance: [run 39089212707](https://github.com/sansaks-jpg/rustmix/actions/runs/39089212707), code `3a36cf2`, passed all 32 tests, native layout/pause/switch/save/reopen and review of 14 actual PNG captures. The reported idle-layout drift, vertical T-Bar and advancing paused-source progress are closed for the tested prototype. Real GPU media composition, decoding/capture/audio/output backends and hardware/endurance readiness remain open as detailed in the engine audit.
