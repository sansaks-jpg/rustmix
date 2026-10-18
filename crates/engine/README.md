# rustmix-engine

Task 1.1 port: schema 5 replaces the fixed four-overlay array with the sixteen-channel
production bank — eight title-panel channels and eight stinger channels driven by
`overlay.stinger.set` / `overlay.stinger.trigger` / `overlay.stinger.stop`, with a cut-point
Program/Preview exchange. Schemas 1–4 migrate on load and keep every authored panel and rollback
byte. Stinger clip compositing is not implemented (no media renderer); the model, commands and
exchange are real. See ADR 0012.

UI-013 source candidate: schema 4 categories/theme, atomic Settings, slot-defined T-Bar,
real Input Search, temporary audio-pane visibility and worker snapshots. Unsupported scaffolds
are explicitly unavailable. Hosted compile/tests/native captures are pending; see ADR 0011.

UI-012: `production` provides persisted transition slots and named playlists over stable input
IDs, with explicit item hold/effect/duration, manual/loop/selected start and bounded progression.
Operator switching stops automation; stopping automation preserves an active take. Configuration
changes validate atomically and offline progression reports an alarm. `shortcuts` validates
portable key bindings and resolves them to typed commands. Schema 3 migrates 1/2 with idle runtime
state. Twelve original CPU effects have independent pixel-vector tests. All 36 engine tests passed
in hosted Windows run 39111788801 (`a01f498`); no full reference/hardware qualification is claimed.

Headless synthetic UI domain. `Engine::execute` validates typed commands; `tick` advances pinned transitions using supplied monotonic milliseconds. Project/model, procedural CPU rendering, deterministic simulated meters and bounded atomic project IO are modules with no UI dependencies.

Engine tests run on GitHub Actions. No real audio, media decoding, hardware, GPU types or output encoders are exposed.

`playback` owns independent monotonic clocks for each procedural source through `input.pause`, `input.restart`, `input.loop` and `inputs.pause`. Global source pause preserves local pause states and timed transitions continue. `render::program_from_sources` composites validated processed surfaces, then overlays and FTB. Source transports are not decoder support and are not persisted in project files.

`input::InputConfig` persists PNG descriptors and validated processing parameters since schema 2. Schema 1 migration supplies identity settings; older schemas load through the current migration chain. `AddInputAt`, `DuplicateInput` and `ApplyInput` are headless atomic commands; layer graphs are acyclic and removal clears dependencies. The engine does not open image paths; a media-free renderer shows black for external-image descriptors until a consumer supplies decoded surfaces.

`overlays` owns the sixteen-channel bank (`standard[8]` title panels plus `stingers[8]` with input, cut point and display-underneath configuration) and `ActiveStinger`. A trigger captures Program/Preview and exchanges them exactly once at the cut point; `tick` performs the exchange before transition completion, and a zero cut point exchanges at the trigger itself. Unconfigured stingers refuse with a disabled reason and failed commands leave the project untouched.

`video_clock` computes rational nanosecond frame deadlines from indices and records skipped deadlines without a catch-up queue. `source_into` reuses caller-owned capacity. `from_project_at` starts restored source clocks at the current session origin. These APIs do not establish a hardware output/audio master clock.
