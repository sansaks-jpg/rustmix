//! Opt-in hosted measurements, separate from deterministic correctness tests.
use super::*;
use std::hash::{Hash, Hasher};
const RATES: [(u32, u32); 3] = [(50, 1), (60000, 1001), (60, 1)];
#[derive(Default)]
pub(crate) struct Profile {
    phase: usize,
    configured: bool,
    warm: Option<Instant>,
    baseline: Option<(u64, program::VideoStats, u64)>,
    samples: Vec<serde_json::Value>,
    bench_done: bool,
}
impl RustMixApp {
    pub(crate) fn profile_step(&mut self, ctx: &egui::Context) {
        let Some(mut profile) = self.profile.take() else {
            return;
        };
        match self.run_profile(&mut profile, ctx) {
            Ok(false) => self.profile = Some(profile),
            result => {
                if let Err(error) = result {
                    let _ = std::fs::create_dir_all("artifacts");
                    let _ = std::fs::write("artifacts/profile-error.txt", error.to_string());
                }
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
    fn run_profile(&mut self, profile: &mut Profile, ctx: &egui::Context) -> Result<bool> {
        if self.start.elapsed() > Duration::from_secs(180) {
            return Err(Error::Invalid(
                "Program measurement exceeded 180 seconds".into(),
            ));
        }
        if !profile.bench_done {
            // Stop the app producer so the measurement is not competing with a
            // second full-HD renderer on the hosted runner.
            let worker = self.program_worker.take();
            drop(worker);
            let evidence = self.run_hd_bench();
            self.program_worker = Some(
                program::ProgramWorker::spawn(
                    program::Snapshot {
                        generation: self.program_generation,
                        engine: self.engine.clone(),
                        sources: self.input_fx.clone(),
                    },
                    self.start,
                    Some(ctx.clone()),
                )
                .map_err(Error::Invalid)?,
            );
            let evidence = evidence?;
            std::fs::create_dir_all("artifacts")?;
            std::fs::write(
                "artifacts/program-hd-bench.json",
                serde_json::to_vec_pretty(&evidence)?,
            )?;
            profile.bench_done = true;
            return Ok(false);
        }
        let Some((n, d)) = RATES.get(profile.phase).copied() else {
            return Ok(true);
        };
        let rate = Rate::new(n, d)?;
        if !profile.configured {
            let mut preferences = self.engine.project().preferences.clone();
            preferences.rate = rate;
            self.execute(Command::SetPreferences(preferences));
            profile.configured = true;
            return Ok(false);
        }
        if self.video_timing.is_none_or(|(_, actual)| actual != rate) || self.video_produced_ns == 0
        {
            return Ok(false);
        }
        let warm = *profile.warm.get_or_insert_with(Instant::now);
        if warm.elapsed() < Duration::from_secs(2) {
            return Ok(false);
        }
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        self.last_program_rgba.hash(&mut hash);
        let fingerprint = hash.finish();
        let (start_ns, baseline, first_pixels) = *profile.baseline.get_or_insert((
            self.video_produced_ns,
            self.video_stats,
            fingerprint,
        ));
        let elapsed_ns = self.video_produced_ns.saturating_sub(start_ns);
        if elapsed_ns < 8_000_000_000 {
            return Ok(false);
        }
        let frames = self.video_stats.rendered.saturating_sub(baseline.rendered);
        let skipped = self.video_stats.skipped.saturating_sub(baseline.skipped);
        let fps = frames as f64 * 1_000_000_000.0 / elapsed_ns as f64;
        let nominal = f64::from(n) / f64::from(d);
        if frames == 0 || first_pixels == fingerprint {
            return Err(Error::Invalid(
                "Program measurement produced no moving frames".into(),
            ));
        }
        profile.samples.push(serde_json::json!({
            "rate_numerator":n,"rate_denominator":d,"raster":self.program_full_size,
            "monitor_raster":self.engine.project().preferences.video_standard().monitor_size(),
            "seconds":elapsed_ns as f64/1e9,"frames_produced":frames,"frames_skipped":skipped,
            "actual_fps":fps,"last_render_ms":self.video_stats.render_ms,"ui_fps":self.fps,
            "cached_surface_bytes_upper_bound":self.video_stats.cached_bytes,
            "program_bytes":self.video_stats.program_bytes,
            "moving_pixels":true,"within_sample_target":fps >= nominal*0.995 && skipped as f64/(frames+skipped) as f64 <= 0.001,
        }));
        profile.phase += 1;
        profile.configured = false;
        profile.warm = None;
        profile.baseline = None;
        if profile.phase == RATES.len() {
            std::fs::create_dir_all("artifacts")?;
            std::fs::write(
                "artifacts/program-cadence.json",
                serde_json::to_vec_pretty(&serde_json::json!({
                    "scope":"CPU Program at the project full-HD raster with a 640x360 monitor preview; not encoder, hardware-output or physical-device qualification",
                    "samples":profile.samples,
                }))?,
            )?;
            return Ok(true);
        }
        Ok(false)
    }
    fn run_hd_bench(&mut self) -> Result<serde_json::Value> {
        let mut engine = Engine::default();
        let mut preferences = engine.project().preferences.clone();
        preferences.width = 1920;
        preferences.height = 1080;
        preferences.rate = Rate::new(60, 1)?;
        engine.execute(Command::SetPreferences(preferences), 0)?;
        engine.execute(Command::TakeInput(InputId(6)), 0)?;
        engine.execute(Command::SelectPreview(InputId(5)), 0)?;
        let config = InputConfig {
            video_source: Some(VideoSource::Browser {
                url: "http://127.0.0.1/".into(),
                width: 1920,
                height: 1080,
            }),
            ..Default::default()
        };
        for id in [InputId(5), InputId(6)] {
            let mut input = engine
                .project()
                .input(id)
                .cloned()
                .ok_or_else(|| Error::Invalid("Benchmark input is missing".into()))?;
            input.config = config.clone();
            engine.execute(Command::ApplyInput { input, audio: None }, 0)?;
        }
        let frames = [
            Arc::new(vec![[16_u8, 32, 48, 255]; 1920 * 1080]),
            Arc::new(vec![[208_u8, 160, 96, 255]; 1920 * 1080]),
        ];
        let mut sources = HashMap::new();
        for id in [InputId(5), InputId(6)] {
            sources.insert(
                id,
                InputFx {
                    config: config.clone(),
                    custom_rgba: Some((1920, 1080, Arc::clone(&frames[0]))),
                    image_generation: 1,
                    ..InputFx::default()
                },
            );
        }
        let mut renderer = program::ProgramRenderer::default();
        let cut = measure_scene(
            &engine,
            &mut renderer,
            &mut sources,
            &[InputId(6)],
            &frames,
            120,
        )?;
        engine.execute(
            Command::SetTransitionSlot(
                0,
                TransitionSlot {
                    kind: TransitionKind::Fade,
                    duration_ms: 1000,
                },
            ),
            0,
        )?;
        engine.execute(Command::SetManualProgress(0.5), 0)?;
        let fade = measure_scene(
            &engine,
            &mut renderer,
            &mut sources,
            &[InputId(6), InputId(5)],
            &frames,
            120,
        )?;
        engine.execute(Command::CancelTransition, 0)?;
        let mut overlay = engine
            .project()
            .overlays
            .standard
            .first()
            .cloned()
            .ok_or_else(|| Error::Invalid("Benchmark overlay is missing".into()))?;
        overlay.enabled = true;
        overlay.title = "FULL HD".into();
        overlay.subtitle = "60 FPS CHECK".into();
        engine.execute(Command::SetOverlay(0, overlay), 0)?;
        let overlay_samples = measure_scene(
            &engine,
            &mut renderer,
            &mut sources,
            &[InputId(6)],
            &frames,
            120,
        )?;
        let budget = 1000.0 / 60.0;
        Ok(serde_json::json!({
            "scope":"release native synthetic 1920x1080 frames through ProgramRenderer::render_full plus 640x360 monitor downscale; no decode, encoder or UI paint",
            "program_size":[1920,1080],"monitor_size":[640,360],"iterations":120,"budget_ms":budget,
            "cut_ms":summary_json(&cut),"fade_ms":summary_json(&fade),"overlay_ms":summary_json(&overlay_samples),
            "cut_within_60fps":percentile(&cut,0.95) <= budget,
            "fade_within_60fps":percentile(&fade,0.95) <= budget,
            "overlay_within_60fps":percentile(&overlay_samples,0.95) <= budget,
        }))
    }
}

fn percentile(samples: &[f64], fraction: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let index = ((samples.len() - 1) as f64 * fraction).round() as usize;
    samples.get(index).copied().unwrap_or(0.0)
}

fn summary_json(samples: &[f64]) -> serde_json::Value {
    serde_json::json!({
        "p50":percentile(samples,0.50),
        "p95":percentile(samples,0.95),
        "p99":percentile(samples,0.99),
        "max":samples.last().copied().unwrap_or(0.0),
    })
}

fn measure_scene(
    engine: &Engine,
    renderer: &mut program::ProgramRenderer,
    sources: &mut HashMap<InputId, InputFx>,
    inputs: &[InputId],
    frames: &[Arc<Vec<[u8; 4]>>],
    iterations: usize,
) -> Result<Vec<f64>> {
    let full_size = engine.project().preferences.video_standard().program_size();
    let monitor_size = engine.project().preferences.video_standard().monitor_size();
    let expected_full = (full_size[0] as usize)
        .checked_mul(full_size[1] as usize)
        .ok_or_else(|| Error::Invalid("Benchmark raster overflow".into()))?;
    let expected_monitor = (monitor_size[0] as usize)
        .checked_mul(monitor_size[1] as usize)
        .ok_or_else(|| Error::Invalid("Benchmark monitor overflow".into()))?;
    let mut render = |iteration: usize| -> Result<()> {
        for (offset, id) in inputs.iter().enumerate() {
            let source = sources
                .get_mut(id)
                .ok_or_else(|| Error::Invalid("Benchmark source is missing".into()))?;
            let frame = frames
                .get((iteration + offset) % frames.len())
                .ok_or_else(|| Error::Invalid("Benchmark frame is missing".into()))?;
            source.custom_rgba = Some((full_size[0], full_size[1], Arc::clone(frame)));
            source.image_generation = source.image_generation.wrapping_add(1);
        }
        let (full, monitor) = renderer.render_full(engine, sources)?;
        if full.len() != expected_full || monitor.len() != expected_monitor {
            return Err(Error::Invalid(
                "Benchmark render returned an unexpected raster size".into(),
            ));
        }
        Ok(())
    };
    for _ in 0..5 {
        render(0)?;
    }
    let mut samples = Vec::with_capacity(iterations);
    for iteration in 0..iterations {
        let start = Instant::now();
        render(iteration)?;
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    samples.sort_by(f64::total_cmp);
    Ok(samples)
}
