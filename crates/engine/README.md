# rustmix-engine

Headless synthetic UI domain. `Engine::execute` validates typed commands; `tick` advances pinned transitions using supplied monotonic milliseconds. Project/model, procedural CPU rendering, deterministic simulated meters and bounded atomic project IO are modules with no UI dependencies.

Engine tests run on GitHub Actions. No real audio, media decoding, hardware, GPU types or output encoders are exposed.

`playback` owns independent monotonic clocks for each procedural source through `input.pause`, `input.restart`, `input.loop` and `inputs.pause`. Global source pause preserves local pause states and timed transitions continue. `render::program_from_sources` composites validated processed surfaces, then overlays and FTB. Source transports are not decoder support and are not persisted in project files.
