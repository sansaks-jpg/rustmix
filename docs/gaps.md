# Known gaps

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
| G02 | P0 for broadcast | Audio is partially implemented: a fixed 48 kHz stereo block mixer with Master/Aux A/Aux B and guest returns, strict WAV decode, typed WAV/microphone descriptors and a Windows WASAPI adapter with an explicitly started monitoring session passed the native descriptor workflow in hosted run 39270445070. No audio device or callback is hardware-qualified, no A/V synchronization or mix-minus hardware test exists, and the main mixer's meters remain simulations. Next: physical device callback/underrun soak, resampling quality review and A/V clock alignment. |
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
