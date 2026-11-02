//! Bounded offline Program producer. It continues while the UI is painting or
//! a file picker is open. This is an RGBA reference consumer, not an encoder.
use super::*;
use rustmix_engine::video_clock::{FrameClock, FrameTiming};
use std::sync::{
    Mutex, TryLockError, Weak,
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
    // The guarded state is only an Option swap/take; payload drops occur after
    // unlocking, so a poisoned slot still contains a valid Option. Control/UI
    // threads may wait for this short swap; the Program thread uses try-locks.
    fn put(&self, value: T) {
        let old = self
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .replace(value);
        drop(old);
    }
    fn take(&self) -> Option<T> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).take()
    }
    fn try_take(&self) -> Option<T> {
        match self.0.try_lock() {
            Ok(mut slot) => slot.take(),
            Err(TryLockError::Poisoned(error)) => error.into_inner().take(),
            Err(TryLockError::WouldBlock) => None,
        }
    }
    fn try_put(&self, value: T) -> std::result::Result<(), T> {
        let mut slot = match self.0.try_lock() {
            Ok(slot) => slot,
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
            Err(TryLockError::WouldBlock) => return Err(value),
        };
        let old = slot.replace(value);
        drop(slot);
        drop(old);
        Ok(())
    }
    fn try_put_if_empty(&self, value: T) -> std::result::Result<(), T> {
        let mut slot = match self.0.try_lock() {
            Ok(slot) => slot,
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
            Err(TryLockError::WouldBlock) => return Err(value),
        };
        if slot.is_some() {
            return Err(value);
        }
        *slot = Some(value);
        Ok(())
    }
}

// Errors and frames use independent single-item slots. A later good frame must
// not overwrite an unread failure; neither path waits on the Program thread.
struct ProgramMailbox<T> {
    frames: Latest<T>,
    errors: Latest<String>,
}
impl<T> Default for ProgramMailbox<T> {
    fn default() -> Self {
        Self {
            frames: Latest::default(),
            errors: Latest::default(),
        }
    }
}
impl<T> ProgramMailbox<T> {
    fn try_frame(&self, frame: T) -> std::result::Result<(), T> {
        self.frames.try_put(frame)
    }
    fn try_error(&self, error: String) -> std::result::Result<(), String> {
        self.errors.try_put_if_empty(error)
    }
    fn take(&self) -> Option<std::result::Result<T, String>> {
        self.errors
            .take()
            .map(Err)
            .or_else(|| self.frames.take().map(Ok))
    }
}

// Fixed storage and an in-place sort keep telemetry bounded and heap-free.
// Percentiles describe the most recent successful CPU render calls, not
// end-to-end latency or frames presented by a device.
const RENDER_TIMING_SAMPLES: usize = 128;
struct RenderTimingWindow {
    samples: [u64; RENDER_TIMING_SAMPLES],
    len: usize,
    next: usize,
}
impl Default for RenderTimingWindow {
    fn default() -> Self {
        Self {
            samples: [0; RENDER_TIMING_SAMPLES],
            len: 0,
            next: 0,
        }
    }
}
impl RenderTimingWindow {
    fn record(&mut self, elapsed_ns: u64) {
        if let Some(sample) = self.samples.get_mut(self.next) {
            *sample = elapsed_ns;
        }
        self.next = (self.next + 1) % RENDER_TIMING_SAMPLES;
        self.len = self.len.saturating_add(1).min(RENDER_TIMING_SAMPLES);
    }
    fn percentiles(&self) -> Option<[u64; 3]> {
        if self.len == 0 {
            return None;
        }
        let mut sorted = self.samples;
        let values = sorted.get_mut(..self.len)?;
        values.sort_unstable();
        let last = self.len - 1;
        Some([50, 95, 99].map(|percent| {
            values
                .get((last * percent + 50) / 100)
                .copied()
                .unwrap_or(0)
        }))
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
    pub handoff_dropped: u64,
    pub fps: f64,
    pub render_ms: f64,
    pub render_p50_ms: f64,
    pub render_p95_ms: f64,
    pub render_p99_ms: f64,
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
    latest: Arc<ProgramMailbox<ProgramFrame>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl ProgramWorker {
    pub fn spawn(
        mut snapshot: Snapshot,
        origin: Instant,
        ctx: Option<egui::Context>,
    ) -> std::result::Result<Self, String> {
        let pending = Arc::new(Latest::default());
        let latest = Arc::new(ProgramMailbox::default());
        let stop = Arc::new(AtomicBool::new(false));
        let (states, output, shutdown) =
            (Arc::clone(&pending), Arc::clone(&latest), Arc::clone(&stop));
        let failure_output = Arc::clone(&latest);
        let failure_ctx = ctx.clone();
        let thread = std::thread::Builder::new().name("rustmix-program".into()).spawn(move || {
          let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut rate = snapshot.engine.project().preferences.rate;
            let mut renderer = ProgramRenderer::default();
            // Reserve the initial CPU surfaces before the first live deadline.
            // Cold allocation/downscale is startup work, not a running frame.
            snapshot.engine.tick(u64::try_from(origin.elapsed().as_millis()).unwrap_or(u64::MAX));
            if let Err(error) = renderer.render_full(&snapshot.engine, &snapshot.sources) {
                output.errors.put(format!("Program initialization failed: {error}"));
                if let Some(ctx) = &ctx { ctx.request_repaint(); }
                return;
            }
            let mut clock = FrameClock::at(
                rate,
                u64::try_from(origin.elapsed().as_nanos()).unwrap_or(u64::MAX),
            );
            let mut stats = VideoStats::default();
            let mut sample_start = Instant::now();
            let mut sample_frames = 0_u64;
            let mut render_timings = RenderTimingWindow::default();
            let mut previous_pixels: Option<Pixels> = None;
            let mut pending_error: Option<String> = None;
            let mut failure_reported = false;
            while !shutdown.load(Ordering::Acquire) {
                if let Some(error) = pending_error.take() {
                    match output.try_error(error) {
                        Ok(()) => {
                            if let Some(ctx) = &ctx { ctx.request_repaint(); }
                        }
                        Err(error) => pending_error = Some(error),
                    }
                }
                if let Some(next) = states.try_take() {
                    snapshot = next;
                }
                if rate != snapshot.engine.project().preferences.rate {
                    rate = snapshot.engine.project().preferences.rate;
                    clock = FrameClock::at(
                        rate,
                        u64::try_from(origin.elapsed().as_nanos()).unwrap_or(u64::MAX),
                    );
                    stats = VideoStats::default();
                    render_timings = RenderTimingWindow::default();
                    sample_frames = 0;
                    sample_start = Instant::now();
                }
                let now_ns = u64::try_from(origin.elapsed().as_nanos()).unwrap_or(u64::MAX);
                if let Some(timing) = clock.due(now_ns) {
                    snapshot.engine.tick(now_ns / 1_000_000);
                    let start = Instant::now();
                    let program_size = snapshot
                        .engine
                        .project()
                        .preferences
                        .video_standard()
                        .program_size();
                    match renderer.render_full(&snapshot.engine, &snapshot.sources) {
                        Ok((full_pixels, pixels)) => {
                            failure_reported = false;
                            stats.rendered = stats.rendered.saturating_add(1);
                            sample_frames = sample_frames.saturating_add(1);
                            stats.skipped = clock.skipped();
                            let elapsed = start.elapsed();
                            stats.render_ms = elapsed.as_secs_f64() * 1000.0;
                            render_timings.record(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX));
                            stats.cached_bytes = renderer.cached_bytes();
                            stats.program_bytes = full_pixels.len() * 4;
                            if sample_start.elapsed() >= Duration::from_secs(1) {
                                stats.fps =
                                    sample_frames as f64 / sample_start.elapsed().as_secs_f64();
                                if let Some([p50, p95, p99]) = render_timings.percentiles() {
                                    stats.render_p50_ms = p50 as f64 / 1_000_000.0;
                                    stats.render_p95_ms = p95 as f64 / 1_000_000.0;
                                    stats.render_p99_ms = p99 as f64 / 1_000_000.0;
                                }
                                sample_frames = 0;
                                sample_start = Instant::now();
                            }
                            let repaint = previous_pixels
                                .as_ref()
                                .is_none_or(|old| !Arc::ptr_eq(old, &pixels));
                            let next_pixels = Arc::clone(&pixels);
                            let frame = ProgramFrame {
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
                            };
                            if output.try_frame(frame).is_ok() {
                                previous_pixels = Some(next_pixels);
                                if repaint && let Some(ctx) = &ctx { ctx.request_repaint(); }
                            } else {
                                // Preserve the last published picture. The next
                                // successful frame reports this bounded drop.
                                stats.handoff_dropped = stats.handoff_dropped.saturating_add(1);
                            }
                        }
                        Err(error) => {
                            if !failure_reported && pending_error.is_none() {
                                pending_error = Some(error.to_string());
                            }
                            failure_reported = true;
                        }
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
          }));
          if result.is_err() {
              failure_output.errors.put("Program worker stopped after an internal failure; last valid frame retained".into());
              if let Some(ctx) = failure_ctx { ctx.request_repaint(); }
          }
        }).map_err(|error| format!("Program worker could not start ({:?}); last valid frame retained", error.kind()))?;
        Ok(Self {
            pending,
            latest,
            stop,
            thread: Some(thread),
        })
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

#[derive(Clone)]
struct ImageKey {
    size: [u32; 2],
    generation: u64,
    // A Weak keeps the Arc's allocation identity alive without retaining the
    // decoded pixels. A bare address can be recycled by a different decoder.
    pixels: Weak<Vec<[u8; 4]>>,
}
impl PartialEq for ImageKey {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
            && self.generation == other.generation
            && Weak::ptr_eq(&self.pixels, &other.pixels)
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
    image: Option<ImageKey>,
    pip_generation: u64,
}
struct Surface {
    key: SourceKey,
    pixels: Pixels,
    generation: u64,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct OverlaySourceKey {
    kind: InputKind,
    color: [u8; 3],
    countdown_duration_ms: u64,
    frame: u64,
}
#[derive(PartialEq)]
struct CompositeKey {
    size: [u32; 2],
    from: u64,
    to: u64,
    transition: Option<(TransitionKind, u32)>,
    black: u32,
    overlays: OverlayBank,
    /// Only visible procedural PiPs affect the composite. Offline and
    /// media-backed inputs currently render a constant black placeholder.
    pip_sources: [Option<OverlaySourceKey>; overlays::STANDARD_CHANNELS],
}
// A source has at most one PiP child; a transition walks at most two paths.
// Keep this traversal scratch on the stack instead of allocating on every frame.
const MAX_LAYER_DEPTH: usize = 32;
const MAX_ACTIVE_LAYERS: usize = 2 * MAX_LAYER_DEPTH;
struct InputList<const N: usize> {
    items: [InputId; N],
    len: usize,
}
impl<const N: usize> Default for InputList<N> {
    fn default() -> Self {
        Self {
            items: [InputId(0); N],
            len: 0,
        }
    }
}
impl<const N: usize> InputList<N> {
    fn contains(&self, id: &InputId) -> bool {
        self.items.iter().take(self.len).any(|item| item == id)
    }
    fn push(&mut self, id: InputId) -> Result<()> {
        let slot = self
            .items
            .get_mut(self.len)
            .ok_or_else(|| Error::Invalid("Invalid Program layer graph".into()))?;
        *slot = id;
        self.len += 1;
        Ok(())
    }
    fn pop(&mut self) {
        self.len = self.len.saturating_sub(1);
    }
}

#[derive(Default)]
pub(crate) struct ProgramRenderer {
    surfaces: HashMap<InputId, Surface>,
    generation: u64,
    composite: Option<(CompositeKey, Pixels)>,
    monitor_cache: Option<(Pixels, Pixels)>,
}
impl ProgramRenderer {
    pub(crate) fn source_pixels(
        &mut self,
        id: InputId,
        engine: &Engine,
        sources: &HashMap<InputId, InputFx>,
        size: [u32; 2],
    ) -> Result<Pixels> {
        self.surface(
            id,
            engine,
            sources,
            size,
            &mut InputList::default(),
            &mut InputList::default(),
        )
        .map(|(pixels, _)| pixels)
    }

    pub(crate) fn retain_inputs(&mut self, project: &ProjectState) {
        self.surfaces.retain(|id, _| project.input(*id).is_some());
    }

    fn surface(
        &mut self,
        id: InputId,
        engine: &Engine,
        sources: &HashMap<InputId, InputFx>,
        size: [u32; 2],
        visiting: &mut InputList<MAX_LAYER_DEPTH>,
        active: &mut InputList<MAX_ACTIVE_LAYERS>,
    ) -> Result<(Pixels, u64)> {
        if visiting.contains(&id) || visiting.len >= MAX_LAYER_DEPTH {
            return Err(Error::Invalid("Invalid Program layer graph".into()));
        }
        let input = engine
            .project()
            .input(id)
            .ok_or_else(|| Error::Invalid("Program source is missing".into()))?;
        active.push(id)?;
        visiting.push(id)?;
        let pip = input
            .config
            .pip_source
            .map(|id| self.surface(id, engine, sources, size, visiting, active))
            .transpose()?;
        visiting.pop();
        let ms = engine.playback(id).map_or(0, |p| p.position_ms());
        let frame = if !input.online
            || input.config.media_path.is_some()
            || input.config.video_source.is_some()
            || !input.is_procedural_playback()
        {
            0
        } else {
            ms
        };
        let image = sources.get(&id).and_then(|fx| {
            fx.custom_rgba
                .as_ref()
                .map(|(w, h, pixels)| ([*w, *h], fx.image_generation, pixels))
        });
        let pip_generation = pip.as_ref().map_or(0, |(_, generation)| *generation);
        if let Some(surface) = self.surfaces.get(&id) {
            let key = &surface.key;
            let image_matches = match (&key.image, image) {
                (None, None) => true,
                (Some(old), Some((size, generation, pixels))) => {
                    old.size == size
                        && old.generation == generation
                        && std::ptr::eq(old.pixels.as_ptr(), Arc::as_ptr(pixels))
                }
                _ => false,
            };
            if key.size == size
                && key.kind == input.kind
                && key.color == input.color
                && key.online == input.online
                && key.config == input.config
                && key.frame == frame
                && image_matches
                && key.pip_generation == pip_generation
            {
                return Ok((Arc::clone(&surface.pixels), surface.generation));
            }
        }
        // Clone owned configuration and create the weak identity only on a
        // cache miss; stable Program frames need neither allocation.
        let key = SourceKey {
            size,
            kind: input.kind,
            color: input.color,
            online: input.online,
            config: input.config.clone(),
            frame,
            image: image.map(|(size, generation, pixels)| ImageKey {
                size,
                generation,
                pixels: Arc::downgrade(pixels),
            }),
            pip_generation,
        };
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
        let mut fx = sources.get(&id).cloned().unwrap_or_default();
        fx.config = input.config.clone();
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
    /// Mirror the fields used by the engine's overlay source generator.
    /// This keeps paused edits visible without recompositing for a hidden
    /// channel or an advancing media clock whose current placeholder is black.
    fn pip_sources(
        project: &ProjectState,
        engine: &Engine,
    ) -> [Option<OverlaySourceKey>; overlays::STANDARD_CHANNELS] {
        std::array::from_fn(|slot| {
            if !project.overlays.standard(slot)?.enabled {
                return None;
            }
            let id = project.overlays.binding(slot)?;
            let input = project.input(id)?;
            if !input.online
                || input.config.media_path.is_some()
                || input.config.audio_source.is_some()
                || input.config.video_source.is_some()
            {
                return None;
            }
            Some(OverlaySourceKey {
                kind: input.kind,
                color: input.color,
                countdown_duration_ms: if input.kind == InputKind::Countdown {
                    input.config.countdown_duration_ms
                } else {
                    0
                },
                frame: if matches!(input.kind, InputKind::Color | InputKind::Graphics) {
                    0
                } else {
                    engine.playback(id).map_or(0, |p| p.position_ms())
                },
            })
        })
    }
    fn render_at(
        &mut self,
        engine: &Engine,
        sources: &HashMap<InputId, InputFx>,
        size: [u32; 2],
    ) -> Result<Pixels> {
        let project = engine.project();
        let transition = engine.transition();
        let mut active = InputList::default();
        let (from, from_generation) = self.surface(
            transition.map_or(project.program, |t| t.from),
            engine,
            sources,
            size,
            &mut InputList::default(),
            &mut active,
        )?;
        let target = transition
            .map(|t| {
                self.surface(
                    t.to,
                    engine,
                    sources,
                    size,
                    &mut InputList::default(),
                    &mut active,
                )
            })
            .transpose()?;
        self.surfaces.retain(|id, _| active.contains(id));
        if transition.is_none()
            && engine.black_level() == 0.0
            && !project.overlays.standard.iter().any(|o| o.enabled)
        {
            self.composite = None;
            return Ok(from);
        }
        let to_generation = target.as_ref().map_or(0, |(_, g)| *g);
        let transition_key = transition.map(|t| (t.kind, t.progress.to_bits()));
        let black = engine.black_level().to_bits();
        let pip_sources = Self::pip_sources(project, engine);
        if let Some((old, pixels)) = &self.composite
            && old.size == size
            && old.from == from_generation
            && old.to == to_generation
            && old.transition == transition_key
            && old.black == black
            && old.overlays == project.overlays
            && old.pip_sources == pip_sources
        {
            return Ok(Arc::clone(pixels));
        }
        let pixels = Arc::new(render::program_from_sources_clocked(
            project,
            transition,
            engine.black_level(),
            size[0],
            size[1],
            &from,
            target.as_ref().map(|(p, _)| p.as_slice()),
            // Each bound input's picture-in-picture is generated at its own
            // playback position — the same clock that generated its
            // full-frame surface — so a playing bound input animates on
            // Program while a paused one stays parked with its surface.
            &|id| engine.playback(id).map_or(0, |p| p.position_ms()),
        )?);
        // Reuse the owned overlay snapshot across moving frames. Cloning the
        // caption strings on every Program frame is unnecessary when only the
        // source pixels, transition or playback clocks have changed.
        let overlays = self
            .composite
            .take()
            .map(|(old, _)| old.overlays)
            .filter(|old| *old == project.overlays)
            .unwrap_or_else(|| project.overlays.clone());
        let key = CompositeKey {
            size,
            from: from_generation,
            to: to_generation,
            transition: transition_key,
            black,
            overlays,
            pip_sources,
        };
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
    fn latest_handoff_is_nonblocking_and_preserves_the_previous_value() {
        let slot = Latest::default();
        slot.put(7);
        let guard = slot.0.lock().unwrap();
        assert!(slot.try_take().is_none());
        assert_eq!(slot.try_put(9), Err(9));
        assert_eq!(*guard, Some(7));
        drop(guard);
        assert_eq!(slot.try_take(), Some(7));
        assert_eq!(slot.try_put(11), Ok(()));
        assert_eq!(slot.take(), Some(11));
    }

    #[test]
    #[allow(clippy::panic)]
    fn poisoned_slot_keeps_a_valid_value_and_remains_nonblocking() {
        let slot = Arc::new(Latest::default());
        slot.put(7);
        let other = Arc::clone(&slot);
        let thread = std::thread::spawn(move || {
            let _guard = other.0.lock().unwrap();
            panic!("inject slot poison");
        });
        assert!(thread.join().is_err());
        assert_eq!(slot.try_take(), Some(7));
        assert_eq!(slot.try_put(9), Ok(()));
        assert_eq!(slot.try_take(), Some(9));
    }

    #[test]
    fn mailbox_delivers_errors_before_the_latest_frame_without_overwriting_them() {
        let mailbox = ProgramMailbox::<u32>::default();
        mailbox.try_error("first".into()).unwrap();
        assert_eq!(mailbox.try_error("second".into()), Err("second".into()));
        mailbox.try_frame(1).unwrap();
        mailbox.try_frame(2).unwrap();
        assert_eq!(mailbox.take(), Some(Err("first".into())));
        mailbox.try_error("second".into()).unwrap();
        assert_eq!(mailbox.take(), Some(Err("second".into())));
        assert_eq!(mailbox.take(), Some(Ok(2)));
        assert_eq!(mailbox.take(), None);
    }

    #[test]
    fn producer_advances_controls_while_the_frame_slot_is_contended() {
        let mut engine = Engine::default();
        let mut preferences = engine.project().preferences.clone();
        preferences.width = 1280;
        preferences.height = 720;
        engine
            .execute(Command::SetPreferences(preferences), 0)
            .unwrap();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        let worker = ProgramWorker::spawn(
            Snapshot {
                generation: 7,
                engine,
                sources: HashMap::new(),
            },
            Instant::now(),
            None,
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let first = loop {
            if let Some(Ok(frame)) = worker.take() {
                break frame;
            }
            assert!(Instant::now() < deadline, "Program did not start");
            std::thread::sleep(Duration::from_millis(10));
        };

        let guard = worker.latest.frames.0.lock().unwrap();
        // Let a frame deadline pass while the consumer holds the slot.
        std::thread::sleep(Duration::from_millis(150));
        let mut engine = first.engine;
        engine.execute(Command::TakeInput(InputId(4)), 200).unwrap();
        worker.submit(Snapshot {
            generation: 8,
            engine,
            sources: HashMap::new(),
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        let consumed_while_busy = loop {
            if worker.pending.0.lock().unwrap().is_none() {
                break true;
            }
            if Instant::now() >= deadline {
                break false;
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        drop(guard);
        assert!(
            consumed_while_busy,
            "Program stalled behind a held frame slot"
        );

        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(Ok(frame)) = worker.take()
                && frame.generation == 8
            {
                assert_eq!(frame.engine.project().program, InputId(4));
                assert!(frame.stats.handoff_dropped > 0);
                break;
            }
            assert!(
                Instant::now() < deadline,
                "Program did not resume publication"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn render_timing_window_is_bounded_and_reports_recent_percentiles() {
        let mut window = RenderTimingWindow::default();
        assert_eq!(window.percentiles(), None);
        for ns in 1..=128 {
            window.record(ns);
        }
        assert_eq!(window.percentiles(), Some([65, 122, 127]));
        window.record(1000);
        assert_eq!(window.percentiles(), Some([66, 123, 128]));
    }

    #[test]
    fn independent_decoders_with_equal_descriptors_and_generations_keep_their_own_picture() {
        let mut engine = Engine::new_preset(VideoStandard::default(), 0).unwrap();
        let config = InputConfig {
            video_source: Some(VideoSource::File {
                path: "same.mp4".into(),
            }),
            ..Default::default()
        };
        let mut sources = HashMap::new();
        for (id, pixel) in [
            (InputId(1), [255, 0, 0, 255]),
            (InputId(2), [0, 0, 255, 255]),
        ] {
            let mut input = engine.project().input(id).cloned().unwrap();
            input.config = config.clone();
            engine.execute(Command::UpdateInput(input), 0).unwrap();
            sources.insert(
                id,
                InputFx {
                    config: config.clone(),
                    custom_rgba: Some((16, 9, Arc::new(vec![pixel; 16 * 9]))),
                    image_generation: 1,
                    ..Default::default()
                },
            );
        }
        let mut renderer = ProgramRenderer::default();
        let before = renderer.render(&engine, &sources).unwrap();
        assert_eq!(before.first(), Some(&[255, 0, 0, 255]));
        engine.execute(Command::Cut, 0).unwrap();
        let after = renderer.render(&engine, &sources).unwrap();
        assert_eq!(after.first(), Some(&[0, 0, 255, 255]));
        assert_eq!(
            before.first(),
            Some(&[255, 0, 0, 255]),
            "Committed pixels stay immutable"
        );
    }
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
        )
        .unwrap();
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
    fn a_bound_overlay_pip_advances_with_the_input_transport_on_program() {
        let mut engine = Engine::default();
        // Program is the static Colour input; standard channel 1 carries the
        // Grid input. TakeInput parked input 1 as Preview, so resume its
        // transport explicitly — the bound input animates independently of
        // Program.
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        engine
            .execute(Command::SetOverlayInput(0, Some(InputId(1))), 0)
            .unwrap();
        engine
            .execute(Command::SetInputPaused(InputId(1), false), 0)
            .unwrap();
        engine.tick(0);
        let mut renderer = ProgramRenderer::default();
        let sources = HashMap::new();
        let first = renderer.render(&engine, &sources).unwrap();
        // An unchanged transport clock reuses the cached composite.
        assert!(Arc::ptr_eq(
            &first,
            &renderer.render(&engine, &sources).unwrap()
        ));
        // The bound transport advances; the picture-in-picture must move with
        // it on Program instead of staying frozen at clock 0.
        engine.tick(5000);
        let moved = renderer.render(&engine, &sources).unwrap();
        assert!(!Arc::ptr_eq(&first, &moved));
        assert_ne!(first.as_slice(), moved.as_slice());
        // Pausing the bound transport parks the picture: further ticks keep
        // reusing the cached composite rather than animating the PiP.
        engine
            .execute(Command::SetInputPaused(InputId(1), true), 5000)
            .unwrap();
        engine.tick(9000);
        assert!(Arc::ptr_eq(
            &moved,
            &renderer.render(&engine, &sources).unwrap()
        ));
    }
    #[test]
    fn layer_traversal_storage_is_bounded_and_reusable() {
        let mut visiting = InputList::<32>::default();
        assert!(!visiting.contains(&InputId(0)));
        for id in 0..32 {
            visiting.push(InputId(id)).unwrap();
            assert!(visiting.contains(&InputId(id)));
        }
        assert!(visiting.push(InputId(32)).is_err());
        visiting.pop();
        assert!(!visiting.contains(&InputId(31)));
        visiting.push(InputId(32)).unwrap();
        assert!(visiting.contains(&InputId(32)));

        // A transition can walk two 32-deep paths, including shared inputs.
        let mut active = InputList::<64>::default();
        for id in 0..64 {
            active.push(InputId(id % 32)).unwrap();
        }
        assert!(active.push(InputId(99)).is_err());
        assert!(active.contains(&InputId(0)));
        assert!(active.contains(&InputId(31)));
    }

    #[test]
    fn deepest_layer_chain_survives_a_transition_and_releases_old_surfaces() {
        let mut project = ProjectState::demo();
        let template = project.input(InputId(6)).cloned().unwrap();
        project.inputs = (1..=32)
            .map(|id| {
                let mut input = template.clone();
                input.id = InputId(id);
                input.name = format!("Layer {id}");
                input.color = [id as u8, 0, 0];
                input.config.pip_source = (id < 32).then_some(InputId(id + 1));
                input
            })
            .collect();
        project.program = InputId(1);
        project.preview = InputId(2);
        let mut engine = Engine::from_project(project).unwrap();
        let sources = HashMap::new();
        let mut renderer = ProgramRenderer::default();
        renderer.render_at(&engine, &sources, [16, 9]).unwrap();
        assert_eq!(renderer.surfaces.len(), 32);

        engine
            .execute(Command::Transition(TransitionKind::Fade), 0)
            .unwrap();
        engine.tick(500);
        let during = renderer.render_at(&engine, &sources, [16, 9]).unwrap();
        let fresh = ProgramRenderer::default()
            .render_at(&engine, &sources, [16, 9])
            .unwrap();
        assert_eq!(during.as_slice(), fresh.as_slice());
        assert_eq!(engine.project().program, InputId(1));
        assert_eq!(renderer.surfaces.len(), 32);

        engine.tick(1000);
        renderer.render_at(&engine, &sources, [16, 9]).unwrap();
        assert_eq!(engine.project().program, InputId(2));
        assert_eq!(renderer.surfaces.len(), 31);
        assert!(!renderer.surfaces.contains_key(&InputId(1)));
    }

    #[test]
    fn changing_a_paused_overlay_input_invalidates_the_composite() {
        let mut engine = Engine::default();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        engine
            .execute(Command::SetOverlayInput(0, Some(InputId(1))), 0)
            .unwrap();
        let sources = HashMap::new();
        let mut renderer = ProgramRenderer::default();
        let original = renderer.render(&engine, &sources).unwrap();

        let mut input = engine.project().input(InputId(1)).cloned().unwrap();
        input.online = false;
        engine
            .execute(Command::UpdateInput(input.clone()), 0)
            .unwrap();
        let offline = renderer.render(&engine, &sources).unwrap();
        assert!(!Arc::ptr_eq(&original, &offline));
        assert_ne!(original.as_slice(), offline.as_slice());
        assert_eq!(engine.project().program, InputId(6));

        input.online = true;
        input.kind = InputKind::Color;
        input.color = [255, 0, 0];
        engine
            .execute(Command::UpdateInput(input.clone()), 0)
            .unwrap();
        let restored = renderer.render(&engine, &sources).unwrap();
        assert!(!Arc::ptr_eq(&offline, &restored));
        assert_ne!(offline.as_slice(), restored.as_slice());
        input.color = [0, 255, 0];
        engine.execute(Command::UpdateInput(input), 0).unwrap();
        let recolored = renderer.render(&engine, &sources).unwrap();
        assert!(!Arc::ptr_eq(&restored, &recolored));
        assert_ne!(restored.as_slice(), recolored.as_slice());
        assert_eq!(engine.project().program, InputId(6));
    }

    #[test]
    fn changing_a_paused_countdown_overlay_duration_updates_its_picture() {
        let mut engine = Engine::default();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        engine
            .execute(Command::SetOverlayInput(0, Some(InputId(7))), 0)
            .unwrap();
        engine
            .execute(Command::SeekInput(InputId(7), 20_000), 0)
            .unwrap();
        let mut renderer = ProgramRenderer::default();
        let before = renderer.render(&engine, &HashMap::new()).unwrap();
        let mut input = engine.project().input(InputId(7)).cloned().unwrap();
        input.config.countdown_duration_ms = 30_000;
        engine.execute(Command::UpdateInput(input), 0).unwrap();
        let after = renderer.render(&engine, &HashMap::new()).unwrap();
        assert!(!Arc::ptr_eq(&before, &after));
        assert_ne!(before.as_slice(), after.as_slice());
        assert_eq!(engine.project().program, InputId(6));
    }

    #[test]
    fn hidden_overlay_transport_does_not_invalidate_the_visible_composite() {
        let mut engine = Engine::default();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        let mut visible = engine.project().overlays.standard(0).cloned().unwrap();
        visible.enabled = true;
        engine.execute(Command::SetOverlay(0, visible), 0).unwrap();
        engine
            .execute(Command::SetOverlayInput(1, Some(InputId(1))), 0)
            .unwrap();
        let mut hidden = engine.project().overlays.standard(1).cloned().unwrap();
        hidden.enabled = false;
        engine
            .execute(Command::SetOverlay(1, hidden.clone()), 0)
            .unwrap();
        engine
            .execute(Command::SetInputPaused(InputId(1), false), 0)
            .unwrap();
        engine.tick(0);
        let sources = HashMap::new();
        let mut renderer = ProgramRenderer::default();
        let first = renderer.render(&engine, &sources).unwrap();
        engine.tick(5000);
        assert!(Arc::ptr_eq(
            &first,
            &renderer.render(&engine, &sources).unwrap()
        ));

        // Bringing the channel back on air must still expose its live clock.
        hidden.enabled = true;
        engine
            .execute(Command::SetOverlay(1, hidden), 5000)
            .unwrap();
        let visible = renderer.render(&engine, &sources).unwrap();
        assert_ne!(first.as_slice(), visible.as_slice());
        engine.tick(9000);
        assert_ne!(
            visible.as_slice(),
            renderer.render(&engine, &sources).unwrap().as_slice()
        );
        assert_eq!(engine.project().program, InputId(6));
    }

    /// Run with: cargo test --release -p rustmix-ui --lib measure_hidden_overlay_cache_hit -- --ignored --nocapture
    #[test]
    #[ignore = "release-mode cache measurement, not a correctness gate"]
    fn measure_hidden_overlay_cache_hit() {
        let mut engine = Engine::default();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        let mut visible = engine.project().overlays.standard(0).cloned().unwrap();
        visible.enabled = true;
        engine.execute(Command::SetOverlay(0, visible), 0).unwrap();
        engine
            .execute(Command::SetOverlayInput(1, Some(InputId(1))), 0)
            .unwrap();
        let mut hidden = engine.project().overlays.standard(1).cloned().unwrap();
        hidden.enabled = false;
        engine.execute(Command::SetOverlay(1, hidden), 0).unwrap();
        engine
            .execute(Command::SetInputPaused(InputId(1), false), 0)
            .unwrap();
        let sources = HashMap::new();
        let mut renderer = ProgramRenderer::default();
        renderer.render_full(&engine, &sources).unwrap();
        let mut samples = Vec::with_capacity(100);
        for frame in 1..=105 {
            engine.tick(frame * 33);
            let started = Instant::now();
            std::hint::black_box(renderer.render_full(&engine, &sources).unwrap());
            if frame > 5 {
                samples.push(started.elapsed().as_secs_f64() * 1000.0);
            }
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "hidden-overlay render_full: p50={:.6}ms p95={:.6}ms",
            samples.get(50).copied().unwrap(),
            samples.get(95).copied().unwrap()
        );
    }

    #[test]
    fn media_overlay_placeholder_does_not_recompose_for_clock_changes() {
        let mut engine = Engine::default();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        let mut input = engine.project().input(InputId(1)).cloned().unwrap();
        input.config.video_source = Some(VideoSource::File {
            path: "placeholder.mp4".into(),
        });
        engine.execute(Command::UpdateInput(input), 0).unwrap();
        engine
            .execute(Command::SetOverlayInput(0, Some(InputId(1))), 0)
            .unwrap();
        engine
            .execute(Command::SetInputPaused(InputId(1), false), 0)
            .unwrap();
        engine.tick(0);
        let mut renderer = ProgramRenderer::default();
        let first = renderer.render(&engine, &HashMap::new()).unwrap();
        engine.tick(5000);
        assert!(Arc::ptr_eq(
            &first,
            &renderer.render(&engine, &HashMap::new()).unwrap()
        ));
    }

    #[test]
    fn cached_decoded_identity_cannot_alias_a_freed_frame() {
        let mut engine = Engine::default();
        engine.execute(Command::TakeInput(InputId(6)), 0).unwrap();
        let mut input = engine.project().input(InputId(6)).cloned().unwrap();
        input.config.image_fit = ImageFit::Stretch;
        engine.execute(Command::UpdateInput(input), 0).unwrap();

        let frame = Arc::new(vec![[255, 0, 0, 255]; 4]);
        let mut sources = HashMap::new();
        sources.insert(
            InputId(6),
            InputFx {
                custom_rgba: Some((2, 2, Arc::clone(&frame))),
                image_generation: 1,
                ..InputFx::default()
            },
        );
        let mut renderer = ProgramRenderer::default();
        let first = renderer.render(&engine, &sources).unwrap();
        assert_eq!(first.first(), Some(&[255, 0, 0, 255]));
        // The cache retains the allocation's identity, not its pixel payload.
        // Otherwise the allocator can reuse an address for a different decoder
        // with the same generation and make the old processed image appear.
        assert!(Arc::weak_count(&frame) > 0);
        let released = Arc::downgrade(&frame);
        drop(frame);
        sources.clear();
        assert!(released.upgrade().is_none());
        sources.insert(
            InputId(6),
            InputFx {
                custom_rgba: Some((2, 2, Arc::new(vec![[0, 255, 0, 255]; 4]))),
                image_generation: 1,
                ..InputFx::default()
            },
        );
        let second = renderer.render(&engine, &sources).unwrap();
        assert_eq!(second.first(), Some(&[0, 255, 0, 255]));
        assert_eq!(first.first(), Some(&[255, 0, 0, 255]));
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
        )
        .unwrap();
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
