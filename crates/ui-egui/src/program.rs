//! Bounded offline Program producer. It continues while the UI is painting or
//! a file picker is open. This is an RGBA reference consumer, not an encoder.
use super::*;
use rustmix_engine::video_clock::{FrameClock, FrameTiming};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

type Pixels = Arc<Vec<[u8; 4]>>;

struct Latest<T>(Mutex<Option<T>>);
impl<T> Default for Latest<T> {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}
impl<T> Latest<T> {
    fn put(&self, value: T) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = Some(value);
    }
    fn take(&self) -> Option<T> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).take()
    }
}
pub(crate) struct Snapshot {
    pub generation: u64,
    pub engine: Engine,
    pub sources: HashMap<InputId, InputFx>,
}
#[derive(Clone, Copy, Default)]
pub(crate) struct VideoStats {
    pub rendered: u64,
    pub skipped: u64,
    pub fps: f64,
    pub render_ms: f64,
    pub cached_bytes: usize,
    pub program_bytes: usize,
}
pub(crate) struct ProgramFrame {
    pub generation: u64,
    pub engine: Engine,
    pub timing: FrameTiming,
    pub produced_ns: u64,
    pub rate: Rate,
    /// Operator-monitor raster uploaded to egui.
    pub pixels: Pixels,
    /// Full-resolution Program surface retained for the video backend/outputs.
    pub full_pixels: Pixels,
    pub program_size: [u32; 2],
    pub stats: VideoStats,
}
pub(crate) struct ProgramWorker {
    pending: Arc<Latest<Snapshot>>,
    latest: Arc<Latest<std::result::Result<ProgramFrame, String>>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl ProgramWorker {
    pub fn spawn(mut snapshot: Snapshot, origin: Instant, ctx: Option<egui::Context>) -> Self {
        let pending = Arc::new(Latest::default());
        let latest = Arc::new(Latest::default());
        let stop = Arc::new(AtomicBool::new(false));
        let (states, output, shutdown) =
            (Arc::clone(&pending), Arc::clone(&latest), Arc::clone(&stop));
        let thread = std::thread::spawn(move || {
            let mut rate = snapshot.engine.project().preferences.rate;
            let mut clock = FrameClock::at(
                rate,
                u64::try_from(origin.elapsed().as_nanos()).unwrap_or(u64::MAX),
            );
            let mut renderer = ProgramRenderer::default();
            let mut stats = VideoStats {
                fps: rate.numerator as f64 / rate.denominator.max(1) as f64,
                ..VideoStats::default()
            };
            let mut sample_start = Instant::now();
            let mut sample_frames = 0_u64;
            let mut previous_pixels: Option<Pixels> = None;
            while !shutdown.load(Ordering::Acquire) {
                if let Some(next) = states.take() {
                    snapshot = next;
                }
                if rate != snapshot.engine.project().preferences.rate {
                    rate = snapshot.engine.project().preferences.rate;
                    clock = FrameClock::at(
                        rate,
                        u64::try_from(origin.elapsed().as_nanos()).unwrap_or(u64::MAX),
                    );
                    stats = VideoStats {
                        fps: rate.numerator as f64 / rate.denominator.max(1) as f64,
                        ..VideoStats::default()
                    };
                    sample_frames = 0;
                    sample_start = Instant::now();
                }
                let now_ns = u64::try_from(origin.elapsed().as_nanos()).unwrap_or(u64::MAX);
                if let Some(timing) = clock.due(now_ns) {
                    snapshot.engine.tick(now_ns / 1_000_000);
                    let start = Instant::now();
                    let mut repaint = true;
                    let program_size = snapshot
                        .engine
                        .project()
                        .preferences
                        .video_standard()
                        .program_size();
                    match renderer.render_full(&snapshot.engine, &snapshot.sources) {
                        Ok((full_pixels, pixels)) => {
                            stats.rendered = stats.rendered.saturating_add(1);
                            sample_frames = sample_frames.saturating_add(1);
                            stats.skipped = clock.skipped();
                            stats.render_ms = start.elapsed().as_secs_f64() * 1000.0;
                            stats.cached_bytes = renderer.cached_bytes();
                            stats.program_bytes = full_pixels.len() * 4;
                            if sample_start.elapsed() >= Duration::from_secs(1) {
                                stats.fps =
                                    sample_frames as f64 / sample_start.elapsed().as_secs_f64();
                                sample_frames = 0;
                                sample_start = Instant::now();
                            }
                            repaint = previous_pixels
                                .as_ref()
                                .is_none_or(|old| !Arc::ptr_eq(old, &pixels));
                            previous_pixels = Some(Arc::clone(&pixels));
                            output.put(Ok(ProgramFrame {
                                generation: snapshot.generation,
                                engine: snapshot.engine.clone(),
                                timing,
                                produced_ns: u64::try_from(origin.elapsed().as_nanos())
                                    .unwrap_or(u64::MAX),
                                rate,
                                pixels,
                                full_pixels,
                                program_size,
                                stats,
                            }));
                        }
                        Err(error) => output.put(Err(error.to_string())),
                    }
                    if repaint && let Some(ctx) = &ctx {
                        ctx.request_repaint();
                    }
                }
                let now_ns = u64::try_from(origin.elapsed().as_nanos()).unwrap_or(u64::MAX);
                // Rust 1.94's Windows sleep uses a high-resolution waitable
                // timer. park_timeout can round to the coarse Windows tick.
                // No busy spin; bound shutdown/state-update latency to 10 ms.
                std::thread::sleep(Duration::from_nanos(
                    clock.wait_ns(now_ns).clamp(100_000, 10_000_000),
                ));
            }
        });
        Self {
            pending,
            latest,
            stop,
            thread: Some(thread),
        }
    }
    pub fn submit(&self, snapshot: Snapshot) {
        self.pending.put(snapshot);
    }
    pub fn take(&self) -> Option<std::result::Result<ProgramFrame, String>> {
        self.latest.take()
    }
}
impl Drop for ProgramWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[derive(Clone, PartialEq)]
struct SourceKey {
    size: [u32; 2],
    kind: InputKind,
    color: [u8; 3],
    online: bool,
    config: InputConfig,
    frame: u64,
    image: Option<(u32, u32, u64)>,
    pip_generation: u64,
}
struct Surface {
    key: SourceKey,
    pixels: Pixels,
    generation: u64,
}
#[derive(PartialEq)]
struct CompositeKey {
    size: [u32; 2],
    from: u64,
    to: u64,
    transition: Option<(TransitionKind, u32)>,
    black: u32,
    overlays: OverlayBank,
}
#[derive(Default)]
pub(crate) struct ProgramRenderer {
    surfaces: HashMap<InputId, Surface>,
    generation: u64,
    composite: Option<(CompositeKey, Pixels)>,
    monitor_cache: Option<(Pixels, Pixels)>,
}
impl ProgramRenderer {
    fn surface(
        &mut self,
        id: InputId,
        engine: &Engine,
        sources: &HashMap<InputId, InputFx>,
        size: [u32; 2],
        visiting: &mut Vec<InputId>,
        active: &mut Vec<InputId>,
    ) -> Result<(Pixels, u64)> {
        if visiting.contains(&id) || visiting.len() >= 32 {
            return Err(Error::Invalid("Invalid Program layer graph".into()));
        }
        let input = engine
            .project()
            .input(id)
            .ok_or_else(|| Error::Invalid("Program source is missing".into()))?;
        active.push(id);
        visiting.push(id);
        let pip = input
            .config
            .pip_source
            .map(|id| self.surface(id, engine, sources, size, visiting, active))
            .transpose()?;
        let _ = visiting.pop();
        let ms = engine.playback(id).map_or(0, |p| p.position_ms());
        let mut fx = sources.get(&id).cloned().unwrap_or_default();
        fx.config = input.config.clone();
        let key = SourceKey {
            size,
            kind: input.kind,
            color: input.color,
            online: input.online,
            config: input.config.clone(),
            frame: if !input.online
                || input.config.media_path.is_some()
                || input.config.video_source.is_some()
                || matches!(input.kind, InputKind::Color | InputKind::Graphics)
            {
                0
            } else {
                ms
            },
            image: fx
                .custom_rgba
                .as_ref()
                .map(|(w, h, _)| (*w, *h, fx.image_generation)),
            pip_generation: pip.as_ref().map_or(0, |(_, generation)| *generation),
        };
        if let Some(surface) = self.surfaces.get(&id)
            && surface.key == key
        {
            return Ok((Arc::clone(&surface.pixels), surface.generation));
        }
        // Duplicate still-image inputs can share the processed surface as well.
        if let Some(surface) = self.surfaces.values().find(|s| s.key == key) {
            let (pixels, generation) = (Arc::clone(&surface.pixels), surface.generation);
            self.surfaces.insert(
                id,
                Surface {
                    key,
                    pixels: Arc::clone(&pixels),
                    generation,
                },
            );
            return Ok((pixels, generation));
        }
        let [w, h] = size;
        let custom = fx.custom_rgba.clone();
        let identity = panels::identity_processing(&fx, pip.is_some());
        // A decoded frame already at Program resolution needs no copy or scale.
        if input.online
            && fx.video_source.is_some()
            && custom
                .as_ref()
                .is_some_and(|(cw, ch, _)| *cw == w && *ch == h)
            && identity
        {
            let pixels = custom
                .as_ref()
                .map(|(_, _, pixels)| Arc::clone(pixels))
                .ok_or_else(|| Error::Invalid("Video frame cache is missing".into()))?;
            self.generation = self.generation.wrapping_add(1);
            self.surfaces.insert(
                id,
                Surface {
                    key,
                    pixels: Arc::clone(&pixels),
                    generation: self.generation,
                },
            );
            return Ok((pixels, self.generation));
        }
        let old = self.surfaces.remove(&id);
        let mut raw = old
            .and_then(|s| Arc::try_unwrap(s.pixels).ok())
            .unwrap_or_default();
        if !input.online || custom.is_none() {
            render::source_into(input, w, h, ms, &mut raw)?;
        } else {
            // Media pixels replace the procedural base entirely.
            raw.clear();
        }
        let pixels = if input.online && (!identity || custom.is_some()) {
            panels::process_source_pixels(&raw, w, h, &fx, pip.as_ref().map(|(p, _)| p.as_slice()))
        } else {
            raw
        };
        self.generation = self.generation.wrapping_add(1);
        let pixels = Arc::new(pixels);
        self.surfaces.insert(
            id,
            Surface {
                key,
                pixels: Arc::clone(&pixels),
                generation: self.generation,
            },
        );
        Ok((pixels, self.generation))
    }
    /// Operator-monitor Program surface, used by the headless UI fallback.
    pub fn render(
        &mut self,
        engine: &Engine,
        sources: &HashMap<InputId, InputFx>,
    ) -> Result<Pixels> {
        let size = engine.project().preferences.video_standard().monitor_size();
        self.render_at(engine, sources, size)
    }
    /// Full-resolution Program plus its bounded operator-monitor preview.
    pub fn render_full(
        &mut self,
        engine: &Engine,
        sources: &HashMap<InputId, InputFx>,
    ) -> Result<(Pixels, Pixels)> {
        let format = engine.project().preferences.video_standard();
        let full_size = format.program_size();
        let monitor_size = format.monitor_size();
        let full = self.render_at(engine, sources, full_size)?;
        if full_size == monitor_size {
            return Ok((Arc::clone(&full), full));
        }
        if let Some((cached_full, cached_monitor)) = &self.monitor_cache
            && Arc::ptr_eq(cached_full, &full)
        {
            return Ok((full, Arc::clone(cached_monitor)));
        }
        let monitor = Arc::new(render::downscale(
            &full,
            full_size[0],
            full_size[1],
            monitor_size[0],
            monitor_size[1],
        )?);
        self.monitor_cache = Some((Arc::clone(&full), Arc::clone(&monitor)));
        Ok((full, monitor))
    }
    fn render_at(
        &mut self,
        engine: &Engine,
        sources: &HashMap<InputId, InputFx>,
        size: [u32; 2],
    ) -> Result<Pixels> {
        let project = engine.project();
        let transition = engine.transition();
        let mut active = Vec::new();
        let (from, from_generation) = self.surface(
            transition.map_or(project.program, |t| t.from),
            engine,
            sources,
            size,
            &mut Vec::new(),
            &mut active,
        )?;
        let target = transition
            .map(|t| self.surface(t.to, engine, sources, size, &mut Vec::new(), &mut active))
            .transpose()?;
        self.surfaces.retain(|id, _| active.contains(id));
        if transition.is_none()
            && engine.black_level() == 0.0
            && !project.overlays.standard.iter().any(|o| o.enabled)
        {
            self.composite = None;
            return Ok(from);
        }
        let key = CompositeKey {
            size,
            from: from_generation,
            to: target.as_ref().map_or(0, |(_, g)| *g),
            transition: transition.map(|t| (t.kind, t.progress.to_bits())),
            black: engine.black_level().to_bits(),
            overlays: project.overlays.clone(),
        };
        if let Some((old, pixels)) = &self.composite
            && *old == key
        {
            return Ok(Arc::clone(pixels));
        }
        let pixels = Arc::new(render::program_from_sources(
            project,
            transition,
            engine.black_level(),
            size[0],
            size[1],
            &from,
            target.as_ref().map(|(p, _)| p.as_slice()),
        )?);
        self.composite = Some((key, Arc::clone(&pixels)));
        Ok(pixels)
    }
    pub fn cached_bytes(&self) -> usize {
        self.surfaces
            .values()
            .map(|s| s.pixels.len() * 4)
            .sum::<usize>()
            + self.composite.as_ref().map_or(0, |(_, p)| p.len() * 4)
            + self.monitor_cache.as_ref().map_or(0, |(_, p)| p.len() * 4)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    #[test]
    fn playlist_advances_without_ui_and_reports_the_control_generation() {
        let mut engine = Engine::default();
        let mut preferences = engine.project().preferences.clone();
        preferences.width = 1280;
        preferences.height = 720;
        engine
            .execute(Command::SetPreferences(preferences), 0)
            .unwrap();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        let mut list = engine.project().playlists.first().cloned().unwrap();
        list.items = [InputId(6), InputId(4)]
            .into_iter()
            .map(|input| PlaylistItem {
                input,
                hold_ms: 100,
                transition: TransitionSlot {
                    kind: TransitionKind::Cut,
                    duration_ms: 100,
                },
            })
            .collect();
        engine.execute(Command::UpdatePlaylist(list), 0).unwrap();
        engine
            .execute(Command::StartPlaylist { id: 1, selected: 0 }, 0)
            .unwrap();
        let worker = ProgramWorker::spawn(
            Snapshot {
                generation: 7,
                engine,
                sources: HashMap::new(),
            },
            Instant::now(),
            None,
        );
        std::thread::sleep(Duration::from_millis(350));
        let deadline = Instant::now() + Duration::from_secs(2);
        let final_frame = loop {
            if let Some(Ok(frame)) = worker.take()
                && frame.engine.project().program == InputId(4)
                && frame.engine.playlist_run().is_none()
            {
                break Some(frame);
            }
            if Instant::now() >= deadline {
                break None;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        .unwrap();
        assert_eq!(final_frame.generation, 7);
        let mut engine = final_frame.engine;
        engine
            .execute(Command::TakeInput(InputId(6)), 3000)
            .unwrap();
        worker.submit(Snapshot {
            generation: 8,
            engine,
            sources: HashMap::new(),
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(Ok(frame)) = worker.take()
                && frame.generation == 8
            {
                assert_eq!(frame.engine.project().program, InputId(6));
                break;
            }
            assert!(
                Instant::now() < deadline,
                "Producer did not acknowledge the new control state"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    #[test]
    fn latest_slot_replaces_frames_and_static_program_reuses_pixels() {
        let slot = Latest::default();
        for value in 0..10_000 {
            slot.put(value);
        }
        assert_eq!(slot.take(), Some(9999));
        assert!(slot.take().is_none());
        let mut engine = Engine::default();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        let mut renderer = ProgramRenderer::default();
        let sources = HashMap::new();
        let first = renderer.render(&engine, &sources).unwrap();
        for ms in 0..100 {
            engine.tick(ms);
            assert!(Arc::ptr_eq(
                &first,
                &renderer.render(&engine, &sources).unwrap()
            ));
        }
        assert_eq!(renderer.surfaces.len(), 1);
        assert_eq!(renderer.cached_bytes(), 640 * 360 * 4);
    }
    #[test]
    fn a_new_decoded_raster_generation_invalidates_the_cached_surface() {
        let mut engine = Engine::default();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        let mut input = engine.project().input(InputId(6)).cloned().unwrap();
        input.config.image_fit = ImageFit::Stretch;
        engine
            .execute(Command::ApplyInput { input, audio: None }, 0)
            .unwrap();
        let mut renderer = ProgramRenderer::default();
        let mut sources = HashMap::new();
        let mut fx = InputFx {
            custom_rgba: Some((2, 2, Arc::new(vec![[255, 0, 0, 255]; 4]))),
            image_generation: 1,
            ..InputFx::default()
        };
        sources.insert(InputId(6), fx.clone());
        let first = renderer.render(&engine, &sources).unwrap();
        assert_eq!(first.first(), Some(&[255, 0, 0, 255]));
        // Dynamic video reuses the cache only while the raster generation is
        // unchanged, never because an Arc address happens to be reused.
        fx.custom_rgba = Some((2, 2, Arc::new(vec![[0, 255, 0, 255]; 4])));
        fx.image_generation = 2;
        sources.insert(InputId(6), fx);
        let second = renderer.render(&engine, &sources).unwrap();
        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(second.first(), Some(&[0, 255, 0, 255]));
    }
    #[test]
    fn producer_continues_without_a_ui_consumer_and_shuts_down() {
        let mut engine = Engine::default();
        let mut p = engine.project().preferences.clone();
        p.width = 1280;
        p.height = 720;
        p.rate = Rate::new(60, 1).unwrap();
        engine.execute(Command::SetPreferences(p), 0).unwrap();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        let worker = ProgramWorker::spawn(
            Snapshot {
                generation: 0,
                engine,
                sources: HashMap::new(),
            },
            Instant::now(),
            None,
        );
        std::thread::sleep(Duration::from_millis(500));
        let frame = worker.take().unwrap().unwrap();
        assert!(frame.stats.rendered >= 2);
        assert!(frame.timing.index >= 1);
        assert_eq!(frame.rate, Rate::new(60, 1).unwrap());
        assert_eq!(frame.pixels.len(), 640 * 360);
        assert_eq!(frame.full_pixels.len(), 1280 * 720);
        assert_eq!(frame.program_size, [1280, 720]);
        assert!(frame.stats.program_bytes >= 1280 * 720 * 4);
        drop(worker);
    }
    fn video_source_engine(
        project_width: u32,
        project_height: u32,
        source_width: u32,
        source_height: u32,
    ) -> (Engine, InputConfig, Arc<Vec<[u8; 4]>>) {
        let mut engine = Engine::default();
        let mut preferences = engine.project().preferences.clone();
        preferences.width = project_width;
        preferences.height = project_height;
        preferences.rate = Rate::new(60, 1).unwrap();
        engine
            .execute(Command::SetPreferences(preferences), 0)
            .unwrap();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        let config = InputConfig {
            video_source: Some(VideoSource::Browser {
                url: "http://127.0.0.1/".into(),
                width: source_width,
                height: source_height,
            }),
            ..Default::default()
        };
        let mut input = engine.project().input(InputId(6)).cloned().unwrap();
        input.config = config.clone();
        engine
            .execute(Command::ApplyInput { input, audio: None }, 0)
            .unwrap();
        (
            engine,
            config,
            Arc::new(vec![
                [12, 34, 56, 255];
                (source_width * source_height) as usize
            ]),
        )
    }
    #[test]
    fn native_1080p_video_frame_is_reused_and_downscaled_for_the_monitor() {
        let (engine, config, frame) = video_source_engine(1920, 1080, 1920, 1080);
        let mut renderer = ProgramRenderer::default();
        let mut sources = HashMap::new();
        sources.insert(
            InputId(6),
            InputFx {
                config,
                custom_rgba: Some((1920, 1080, Arc::clone(&frame))),
                image_generation: 1,
                ..InputFx::default()
            },
        );
        let (full, monitor) = renderer.render_full(&engine, &sources).unwrap();
        assert_eq!(full.len(), 1920 * 1080);
        assert_eq!(monitor.len(), 640 * 360);
        assert!(
            Arc::ptr_eq(&full, &frame),
            "a native 1080p frame must not be copied or rescaled"
        );
        assert_eq!(monitor.first(), Some(&[12, 34, 56, 255]));
    }
    #[test]
    fn decoded_720p_frame_scales_into_a_full_hd_program() {
        let (engine, config, frame) = video_source_engine(1920, 1080, 1280, 720);
        let mut renderer = ProgramRenderer::default();
        let mut sources = HashMap::new();
        sources.insert(
            InputId(6),
            InputFx {
                config,
                custom_rgba: Some((1280, 720, Arc::clone(&frame))),
                image_generation: 1,
                ..InputFx::default()
            },
        );
        let (full, monitor) = renderer.render_full(&engine, &sources).unwrap();
        assert_eq!(full.len(), 1920 * 1080);
        assert_eq!(full.first(), Some(&[12, 34, 56, 255]));
        assert_eq!(monitor.first(), Some(&[12, 34, 56, 255]));
    }
    #[test]
    fn portrait_full_hd_program_keeps_its_orientation_in_the_monitor() {
        let (engine, config, frame) = video_source_engine(1080, 1920, 1080, 1920);
        let mut renderer = ProgramRenderer::default();
        let mut sources = HashMap::new();
        sources.insert(
            InputId(6),
            InputFx {
                config,
                custom_rgba: Some((1080, 1920, Arc::clone(&frame))),
                image_generation: 1,
                ..InputFx::default()
            },
        );
        let (full, monitor) = renderer.render_full(&engine, &sources).unwrap();
        assert_eq!(full.len(), 1080 * 1920);
        assert_eq!(monitor.len(), 360 * 640);
        assert!(Arc::ptr_eq(&full, &frame));
        assert_eq!(monitor.first(), Some(&[12, 34, 56, 255]));
    }
}
