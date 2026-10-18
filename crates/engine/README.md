# rustmix-engine

UI-012: `production` provides persisted transition slots and named playlists over stable input
IDs, with explicit item hold/effect/duration, manual/loop/selected start and bounded progression.
Operator switching stops automation; stopping automation preserves an active take. Configuration
changes validate atomically and offline progression reports an alarm. `shortcuts` validates
portable key bindings and resolves them to typed commands. Schema 3 migrates 1/2 with idle runtime
state. Twelve original CPU effects have pixel-vector tests prepared; hosted execution is pending.

Headless synthetic UI domain. `Engine::execute` validates typed commands; `tick` advances pinned transitions using supplied monotonic milliseconds. Project/model, procedural CPU rendering, deterministic simulated meters and bounded atomic project IO are modules with no UI dependencies.

Engine tests run on GitHub Actions. No real audio, media decoding, hardware, GPU types or output encoders are exposed.

`playback` owns independent monotonic clocks for each procedural source through `input.pause`, `input.restart`, `input.loop` and `inputs.pause`. Global source pause preserves local pause states and timed transitions continue. `render::program_from_sources` composites validated processed surfaces, then overlays and FTB. Source transports are not decoder support and are not persisted in project files.

`input::InputConfig` persists PNG descriptors and validated processing parameters in schema 2. Schema 1 loads migrate to identity settings. `AddInputAt`, `DuplicateInput` and `ApplyInput` are headless atomic commands; layer graphs are acyclic and removal clears dependencies. The engine does not open image paths; a media-free renderer shows black for external-image descriptors until a consumer supplies decoded surfaces.

`video_clock` computes rational nanosecond frame deadlines from indices and records skipped deadlines without a catch-up queue. `source_into` reuses caller-owned capacity. `from_project_at` starts restored source clocks at the current session origin. These APIs do not establish a hardware output/audio master clock.
