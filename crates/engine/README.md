# rustmix-engine

UI-034 candidate: `video_format` validates landscape/portrait project standards,
rational frame rates, scan/field order and exact Camera modes. `Engine::new_preset`
constructs two muted black safety inputs without I/O. Schema 10 migrates older
scan/camera defaults. Prepared tests cover atomic invalid input, persistence and
legacy migration; hosted execution pending. Interlaced output is not implemented.

2026-10-19 overlay caption candidate: `overlay_text` rasterizes enabled overlay titles and
subtitles into the Program surface. It carries an original 5x8 bitmap font for printable ASCII (no
third-party font file and no new dependency), exposes the panel fractions shared with the
compositor, and is applied by `render::program_from_sources` after the panel geometry and before
fade-to-black. Writes are bounds-checked, lines are capped at 96 characters, the text edge is 0.52w
and unknown characters advance blank, so a malformed caption cannot interrupt Program output. Engine
unit tests cover glyph coverage, ink containment for all three panel positions, subtitle ink,
caption ink being covered by fade-to-black and disabled overlays leaving source pixels untouched.
Hosted unit/native acceptance is pending for this commit. Non-claims: bitmap-only typography, ASCII
glyphs only, no font choice/outline/shadow/animation/ticker/data binding, no per-source overlay, and
only the experimental 640×360 CPU Program raster is covered.

2026-10-19 audio/video candidate accepted on hosted Windows: schema 8 keeps typed local-media
descriptors. `audio_engine` is a bounded, sample-based fixed 48 kHz stereo mixer (256-frame blocks,
Master/Aux A/Aux B, guest returns, click-safe gain ramps, peak/RMS/clipping/underrun counters) with
no device ownership; `wav` strictly decodes bounded PCM16/float32 RIFF/WAVE data and retains its
source rate so playback must resample explicitly. `input::AudioSource` persists WAV/microphone
descriptors (schema 7, carried into schema 8). `video_source` defines the `VideoFeed` contract: one
replaceable immutable Arc RGBA frame, monotonic sequence, 100 ns PTS/duration, generation identity,
health states and nonblocking try-locked snapshots/publication. `input::VideoSource::File` is
descriptor data only. The headless reference generator returns black for every external-media input,
so a missing decoded frame can never masquerade as a procedural source. The engine opens no audio
device, decodes no video and does not synchronize audio with video; hosted acceptance covers the
native audio descriptor workflow (run 39270445070) and local MP4 decode through UI/platform workers
(run 39273839106).

Task 1.3 source candidate: schema 6 adds four independent external-output configuration
slots (`outputs`). `output.route.set` and `output.external.configure` update individual
fields; `output.external.apply` validates and publishes a complete slot atomically in
one revision. Settings preserve Program/Preview, transitions and existing audio routing.
The hardware connector `port` is a 1-based descriptor (`None` = device default), not a
network port; device and connector support is unverified until a backend enumerates it.
Schemas 1-5 migrate with an unassigned default bank while preserving rollback bytes;
schema 6 requires the bank and rejects corrupt output state. Tests cover these paths,
invalid configurations, source removal and revision exhaustion but have not been run
for this patch. Local formatting/static checks only; hosted Windows execution remains
required. No hardware output, encoder, streaming, NDI/OMT, live state or separate
fullscreen output is implemented by this configuration model.

Task 1.1 port: schema 5 replaces the fixed four-overlay array with the sixteen-channel
production bank - eight title-panel channels and eight stinger channels driven by
`overlay.stinger.set` / `overlay.stinger.trigger` / `overlay.stinger.stop`, with a cut-point
Program/Preview exchange. Schemas 1-4 migrate on load and keep every authored panel and rollback
byte. Stinger clip compositing is not implemented (no media renderer); the model, commands and
exchange are real. Hosted native acceptance passed in run 39129117131. See ADR 0012.

Task 1.2 port: `categories` provides the typed `InputCategory` contract - `All`, the six preset
colours, `Custom(1..=16)` with clamping and the filter-only `Search` pseudo value - mapped onto
the persisted 22 definition ids without a schema change, plus `ProjectState::visible_inputs` for
the category filter the UI grid now uses. See ADR 0013.

UI-013 source candidate: schema 4 categories/theme, atomic Settings, slot-defined T-Bar,
real Input Search, temporary audio-pane visibility and worker snapshots. Unsupported scaffolds
are explicitly unavailable. Hosted compile/tests/native captures passed in run 39129117131; see ADR 0011.

UI-012: `production` provides persisted transition slots and named playlists over stable input
IDs, with explicit item hold/effect/duration, manual/loop/selected start and bounded progression.
Operator switching stops automation; stopping automation preserves an active take. Configuration
changes validate atomically and offline progression reports an alarm. `shortcuts` validates
portable key bindings and resolves them to typed commands. Schema 3 migrates 1/2 with idle runtime
state. Twelve original CPU effects have independent pixel-vector tests. All 36 engine tests passed
in hosted Windows run 39111788801 (`a01f498`); no full reference/hardware qualification is claimed.

Headless synthetic UI domain. `Engine::execute` validates typed commands; `tick` advances pinned transitions using supplied monotonic milliseconds. Project/model, procedural CPU rendering, deterministic simulated meters and bounded atomic project IO are modules with no UI dependencies.

Engine tests run on GitHub Actions. The engine exposes no hardware, GPU types or output encoders; it does not open audio/video devices or files. Local MP4 decoding belongs to the platform/UI workers, and real audio rendering is driven by the UI session.

`playback` owns independent monotonic clocks for each procedural source through `input.pause`, `input.restart`, `input.loop` and `inputs.pause`. Global source pause preserves local pause states and timed transitions continue. `render::program_from_sources` composites validated processed surfaces, then overlays and FTB. Source transports are not decoder support and are not persisted in project files.

`input::InputConfig` persists PNG descriptors and validated processing parameters since schema 2. Schema 1 migration supplies identity settings; older schemas load through the current migration chain. `AddInputAt`, `DuplicateInput` and `ApplyInput` are headless atomic commands; layer graphs are acyclic and removal clears dependencies. The engine does not open image paths; a media-free renderer shows black for external-image descriptors until a consumer supplies decoded surfaces.

`overlays` owns the sixteen-channel bank (`standard[8]` title panels plus `stingers[8]` with input, cut point and display-underneath configuration) and `ActiveStinger`. A trigger captures Program/Preview and exchanges them exactly once at the cut point; `tick` performs the exchange before transition completion, and a zero cut point exchanges at the trigger itself. Unconfigured stingers refuse with a disabled reason and failed commands leave the project untouched.

`video_clock` computes rational nanosecond frame deadlines from indices and records skipped deadlines without a catch-up queue. `source_into` reuses caller-owned capacity. `from_project_at` starts restored source clocks at the current session origin. These APIs do not establish a hardware output/audio master clock.
