//! Opt-in hosted-runner harness. Real native frames, egui keyboard/pointer input,
//! state assertions, project reopen, and GPU readback screenshots; no external output.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Observation of an opt-in callback for work submitted before the current paint.
/// Bounds describe callback delivery through poll/submit, not GPU execution time.
#[derive(Default, Serialize)]
struct QueueCompletionSample {
    requested_token: u64,
    observed_token: u64,
    request_elapsed_ms: Option<u64>,
    pending_age_lower_bound_ms: Option<u64>,
    callback_delivery_age_bounds_ms: Option<[u64; 2]>,
}

#[derive(Default)]
struct QueueCompletionProbe {
    requested_token: u64,
    observed_token: u64,
    requested_at_ms: Option<u64>,
    last_pending_observation_ms: Option<u64>,
    callback_delivery_age_bounds_ms: Option<[u64; 2]>,
    completed: Arc<AtomicU64>,
}

impl QueueCompletionProbe {
    /// Never register another callback while one is pending or before its
    /// completion has been recorded. A stalled queue retains at most one.
    fn begin_if_idle(&mut self, elapsed_ms: u64) -> Option<(u64, Arc<AtomicU64>)> {
        if self.completed.load(Ordering::Acquire) < self.requested_token
            || self.observed_token < self.requested_token
        {
            return None;
        }
        let token = self.requested_token.checked_add(1)?;
        self.requested_token = token;
        self.requested_at_ms = Some(elapsed_ms);
        self.last_pending_observation_ms = Some(elapsed_ms);
        self.callback_delivery_age_bounds_ms = None;
        Some((token, Arc::clone(&self.completed)))
    }

    fn observe(
        &mut self,
        before_observation_ms: u64,
        after_observation_ms: impl FnOnce() -> u64,
    ) -> QueueCompletionSample {
        let completed = self.completed.load(Ordering::Acquire);
        // Bracket the atomic observation. Using a timestamp only before the
        // load could understate the upper bound when delivery races this sample.
        let after_observation_ms = after_observation_ms();
        let pending = completed < self.requested_token;
        if pending {
            self.last_pending_observation_ms = Some(before_observation_ms);
        } else if self.observed_token < self.requested_token {
            self.callback_delivery_age_bounds_ms = self.requested_at_ms.map(|requested| {
                [
                    self.last_pending_observation_ms
                        .unwrap_or(requested)
                        .saturating_sub(requested),
                    after_observation_ms.saturating_sub(requested),
                ]
            });
        }
        self.observed_token = completed;
        QueueCompletionSample {
            requested_token: self.requested_token,
            observed_token: completed,
            request_elapsed_ms: self.requested_at_ms,
            pending_age_lower_bound_ms: self
                .requested_at_ms
                .filter(|_| pending)
                .map(|requested| before_observation_ms.saturating_sub(requested)),
            callback_delivery_age_bounds_ms: self.callback_delivery_age_bounds_ms,
        }
    }
}
// Only consumed by the opt-in hosted native harness, never by normal startup.
const DIALOG_SHOTS: [(&str, [f32; 2], u8); 9] = [
    ("add-input-1600", [1600.0, 900.0], 0),
    ("input-properties-1600", [1600.0, 900.0], 1),
    ("settings-1600", [1600.0, 900.0], 2),
    ("overlay-settings-1600", [1600.0, 900.0], 3),
    ("recording-setup-1600", [1600.0, 900.0], 4),
    ("streaming-setup-1600", [1600.0, 900.0], 5),
    ("multicorder-1600", [1600.0, 900.0], 6),
    ("playlist-1600", [1600.0, 900.0], 7),
    ("shortcuts-1600", [1600.0, 900.0], 8),
];
/// Bounded diagnostics for opt-in CI runs, with no media paths or project text.
#[derive(Default, Serialize)]
struct SmokeStageSample {
    stage: u32,
    frames: u32,
    elapsed_ms: u64,
    captured: bool,
    failed: bool,
    program_generation: u64,
    presented_generation: Option<u64>,
    gpu_poll_status: &'static str,
    queue_completion: Option<QueueCompletionSample>,
    viewport: [f32; 2],
    pixels_per_point: f32,
    previous_ui_cpu_seconds: Option<f32>,
    ui_fps: f32,
    program_rendered: u64,
    program_skipped: u64,
    program_render_ms: f64,
}

#[derive(Default)]
pub(crate) struct Smoke {
    pub events: Vec<egui::Event>,
    pub(crate) stage: u32,
    frames: u32,
    last_stage: u32,
    visual_started_at: Option<f64>,
    pub(crate) wait: u32,
    pub(crate) captured: bool,
    /// Scratch text/pixel evidence for opt-in smoke modes with staged values.
    pub(crate) scratch_text: Option<String>,
    pub(crate) source_fixture: Option<crate::source_smoke::FixtureServer>,
    pub(crate) scratch_pixels: Option<Vec<u8>>,
    bank_baseline: Option<Rect>,
    paused_program: Option<(u64, Vec<u8>, Rect)>,
    transport_target: Option<Rect>,
    basic_state: Option<(InputId, InputId, u64)>,
    stage_trace: VecDeque<SmokeStageSample>,
    queue_completion_probe: QueueCompletionProbe,
}
impl Smoke {
    fn visual_settled_at(&self, now: f64, animation_time: f32) -> bool {
        let animation_time = f64::from(animation_time);
        self.visual_started_at.is_some_and(|start| {
            start.is_finite()
                && now.is_finite()
                && animation_time.is_finite()
                && (0.0..=5.0).contains(&animation_time)
                && now - start >= animation_time + 0.10
        })
    }

    pub(crate) fn visual_settled(&self, ctx: &egui::Context) -> bool {
        self.visual_settled_at(
            ctx.input(|input| input.time),
            ctx.global_style().animation_time,
        )
    }

    fn push_stage_sample(&mut self, sample: SmokeStageSample) {
        if self.stage_trace.len() >= 128 {
            self.stage_trace.pop_front();
        }
        self.stage_trace.push_back(sample);
    }

    pub(crate) fn is_dialog_stage(&self) -> bool {
        self.stage >= 20
    }
}
fn native_mode_name() -> String {
    const MODES: [&str; 17] = [
        "--ui-smoke",
        "--ui-smoke-reopen",
        "--ui-layout-smoke",
        "--ui-dialog-smoke",
        "--ui-add-smoke",
        "--ui-source-smoke",
        "--ui-input-smoke",
        "--ui-input-smoke-reopen",
        "--ui-reference-smoke",
        "--ui-reference-smoke-reopen",
        "--ui-output-smoke",
        "--ui-output-smoke-reopen",
        "--ui-audio-smoke",
        "--ui-audio-smoke-reopen",
        "--ui-video-smoke",
        "--ui-video-smoke-reopen",
        "--ui-perf",
    ];
    std::env::args()
        .find(|arg| MODES.contains(&arg.as_str()))
        .map_or_else(
            || "ui-smoke".into(),
            |mode| mode.trim_start_matches("--").into(),
        )
}

/// Test-only hardware provenance; never records media paths or project contents.
pub(crate) fn record_native_environment(cc: &eframe::CreationContext<'_>) -> Result<()> {
    let adapter = cc
        .wgpu_render_state
        .as_ref()
        .map(|state| state.adapter.get_info());
    let value = serde_json::json!({
        "adapter": adapter.map(|info| serde_json::json!({
            "name": info.name,
            "backend": format!("{:?}", info.backend),
            "device_type": format!("{:?}", info.device_type),
            "vendor": info.vendor,
            "device": info.device,
            "driver": info.driver,
            "driver_info": info.driver_info,
        })),
        "logical_processors": std::thread::available_parallelism().ok().map(|count| count.get()),
        "pixels_per_point": cc.egui_ctx.pixels_per_point(),
    });
    std::fs::create_dir_all("artifacts")?;
    std::fs::write(
        format!("artifacts/{}-environment.json", native_mode_name()),
        serde_json::to_vec_pretty(&value)?,
    )?;
    Ok(())
}

fn key(key: egui::Key) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        })
        .collect()
}
impl RustMixApp {
    pub(crate) fn smoke_step(&mut self, ctx: &egui::Context, previous_ui_cpu_seconds: Option<f32>) {
        let Some(mut s) = self.smoke.take() else {
            return;
        };
        s.frames = s.frames.saturating_add(1);
        if s.stage != s.last_stage || s.visual_started_at.is_none() {
            s.wait = 0;
            s.last_stage = s.stage;
            s.visual_started_at = Some(ctx.input(|input| input.time));
        }
        s.wait = s.wait.saturating_add(1);
        let result = self.smoke_run(ctx, &mut s);
        let trace_result =
            self.record_smoke_stage(ctx, &mut s, result.is_err(), previous_ui_cpu_seconds);
        if let Err(e) = result.and(trace_result) {
            let _ = std::fs::create_dir_all("artifacts");
            let _ = std::fs::write("artifacts/smoke-error.txt", e.to_string());
            self.saved_revision = self.engine.project().revision;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.smoke = Some(s);
    }
    fn record_smoke_stage(
        &self,
        ctx: &egui::Context,
        smoke: &mut Smoke,
        failed: bool,
        previous_ui_cpu_seconds: Option<f32>,
    ) -> Result<()> {
        let elapsed_ms = u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX);
        if !failed
            && smoke.stage_trace.back().is_some_and(|sample| {
                sample.stage == smoke.stage && elapsed_ms.saturating_sub(sample.elapsed_ms) < 10_000
            })
        {
            return Ok(());
        }
        if let Some(queue) = &self.smoke_gpu_queue
            && let Some((token, completed)) = smoke.queue_completion_probe.begin_if_idle(elapsed_ms)
        {
            // Snapshot only work already submitted; the current frame is painted
            // after this UI callback. Delivery needs poll/submit and may be later
            // than physical GPU completion. The callback itself only stores a token.
            queue.on_submitted_work_done(move || completed.store(token, Ordering::Release));
        }
        // Poll is explicitly nonblocking; no Wait, timeout relaxation or
        // acceptance bypass. Samples distinguish a pending readback from a
        // stage that stopped receiving UI frames on a hosted software adapter.
        let gpu_poll_status = match self
            .gpu_device
            .as_ref()
            .map(|device| device.poll(eframe::wgpu::PollType::Poll))
        {
            Some(Ok(eframe::wgpu::PollStatus::QueueEmpty)) => "QueueEmpty",
            Some(Ok(eframe::wgpu::PollStatus::WaitSucceeded)) => "WaitSucceeded",
            Some(Ok(eframe::wgpu::PollStatus::Poll)) => "Poll",
            Some(Err(_)) => "poll-error",
            None => "unavailable",
        };
        let before_observation_ms =
            u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let queue_completion = self.smoke_gpu_queue.as_ref().map(|_| {
            smoke
                .queue_completion_probe
                .observe(before_observation_ms, || {
                    u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX)
                })
        });
        let elapsed_ms = u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX);
        smoke.push_stage_sample(SmokeStageSample {
            stage: smoke.stage,
            frames: smoke.frames,
            elapsed_ms,
            captured: smoke.captured,
            failed,
            program_generation: self.program_generation,
            presented_generation: self.program_presented_generation,
            gpu_poll_status,
            queue_completion,
            viewport: [ctx.viewport_rect().width(), ctx.viewport_rect().height()],
            pixels_per_point: ctx.pixels_per_point(),
            previous_ui_cpu_seconds: previous_ui_cpu_seconds.filter(|value| value.is_finite()),
            ui_fps: self.fps,
            program_rendered: self.video_stats.rendered,
            program_skipped: self.video_stats.skipped,
            program_render_ms: self.video_stats.render_ms,
        });
        std::fs::create_dir_all("artifacts")?;
        std::fs::write(
            format!("artifacts/{}-stages.json", native_mode_name()),
            serde_json::to_vec_pretty(&smoke.stage_trace)?,
        )?;
        Ok(())
    }

    fn smoke_run(&mut self, ctx: &egui::Context, s: &mut Smoke) -> Result<()> {
        if std::env::args().any(|arg| arg == "--ui-source-smoke") {
            return self.smoke_sources(ctx, s);
        }
        if std::env::args().any(|arg| arg == "--ui-add-smoke") {
            return self.smoke_add_input(ctx, s);
        }
        if std::env::args().any(|arg| arg == "--ui-video-smoke" || arg == "--ui-video-smoke-reopen")
        {
            return self.smoke_video(ctx, s);
        }
        if std::env::args().any(|arg| arg == "--ui-audio-smoke" || arg == "--ui-audio-smoke-reopen")
        {
            return self.smoke_audio(ctx, s);
        }
        if std::env::args()
            .any(|arg| arg == "--ui-output-smoke" || arg == "--ui-output-smoke-reopen")
        {
            return self.smoke_program_output(ctx, s);
        }
        if std::env::args()
            .any(|arg| arg == "--ui-reference-smoke" || arg == "--ui-reference-smoke-reopen")
        {
            return self.smoke_reference(ctx, s);
        }
        if std::env::args().any(|s| s == "--ui-input-smoke" || s == "--ui-input-smoke-reopen") {
            return self.smoke_inputs(ctx, s);
        }
        if std::env::args().any(|arg| arg == "--ui-layout-smoke") {
            return self.smoke_reference_layout(ctx, s);
        }
        let reopen = std::env::args().any(|s| s == "--ui-smoke-reopen");
        let dialogs_only = std::env::args().any(|arg| arg == "--ui-dialog-smoke");
        // Includes 60 idle + 35 paused native frames and 27 dialog stages on
        // hosted software graphics. Measured 2026-10-18: 329 frames in 240s
        // (~0.73 s/frame), so the full ~450-frame run needs a larger budget.
        // Keep the matching external limit at 600s.
        if self.start.elapsed() > Duration::from_secs(540) {
            return Err(Error::Invalid(format!(
                "Native UI smoke timed out at stage {} after {} frames",
                s.stage, s.frames
            )));
        }
        let events = ctx.input(|i| i.events.clone());
        for event in events {
            if let egui::Event::Screenshot { image, .. } = event {
                std::fs::create_dir_all("artifacts")?;
                let filename = if reopen {
                    "artifacts/reopened.png".into()
                } else if s.stage == 15 {
                    "artifacts/paused-program.png".into()
                } else if s.stage == 18 {
                    "artifacts/workspace-1100.png".into()
                } else if s.stage == 50 {
                    "artifacts/overlay-unsaved-1600.png".into()
                } else if let Some((name, _, _)) = s
                    .stage
                    .checked_sub(20)
                    .and_then(|stage| DIALOG_SHOTS.get((stage / 3) as usize))
                {
                    format!("artifacts/{name}.png")
                } else if s.stage == 7 {
                    "artifacts/workspace-1600.png".into()
                } else {
                    "artifacts/workspace-1920.png".into()
                };
                let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                image::save_buffer(
                    filename,
                    &bytes,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                )
                .map_err(|e| Error::Invalid(e.to_string()))?;
                s.captured = true;
            }
        }
        if s.wait < 4 {
            return Ok(());
        }
        self.validate_monitor_geometry(ctx.viewport_rect().right())?;
        if dialogs_only && s.stage == 0 {
            if self.job.is_some()
                || self.program_presented_generation != Some(self.program_generation)
            {
                return Ok(());
            }
            self.assert_smoke_state()?;
            s.stage = 20;
            return Ok(());
        }
        if reopen {
            if self.job.is_some() {
                return Ok(());
            }
            self.assert_smoke_state()?;
            if s.stage == 0 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.stage = 1;
            } else if s.captured {
                std::fs::write(
                    "artifacts/reopen-ok.json",
                    serde_json::to_vec_pretty(self.engine.project())?,
                )?;
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            return Ok(());
        }
        match s.stage {
            0 => {
                let bank = self
                    .automation
                    .get("monitor.bank.bounds")
                    .copied()
                    .ok_or_else(|| Error::Invalid("Monitor bank has no bounds".into()))?;
                if let Some(baseline) = s.bank_baseline {
                    if (bank.bottom() - baseline.bottom()).abs() > 1.0 {
                        return Err(Error::Invalid(
                            "Native monitor bank grew without operator input".into(),
                        ));
                    }
                } else {
                    s.bank_baseline = Some(bank);
                }
                if s.wait < 60 {
                    return Ok(());
                }
                std::fs::write(
                    "artifacts/layout-stability.json",
                    serde_json::to_vec_pretty(
                        &serde_json::json!({"frames":s.wait,"bounds":format!("{bank:?}"),"stable":true}),
                    )?,
                )?;
                s.events = key(egui::Key::Num2);
                s.stage = 1;
            }
            1 => {
                if self.engine.project().preview != InputId(2)
                    || self.engine.project().program != InputId(1)
                {
                    return Err(Error::Invalid(
                        "Preview selection changed Program or did not select input 2".into(),
                    ));
                }
                s.events = key(egui::Key::Space);
                s.stage = 2;
            }
            2 => {
                if self.engine.project().program != InputId(2) {
                    return Err(Error::Invalid("Native CUT shortcut did not switch".into()));
                }
                s.events = key(egui::Key::Num3);
                s.stage = 3;
            }
            3 => {
                if self.engine.project().preview != InputId(3) {
                    return Err(Error::Invalid("Native Preview 3 selection failed".into()));
                }
                s.events = key(egui::Key::Enter);
                s.stage = 4;
            }
            4 => {
                if self.engine.transition().is_some() {
                    return Ok(());
                }
                if self.engine.project().program != InputId(3) {
                    return Err(Error::Invalid("Native FADE did not complete".into()));
                }
                s.events = key(egui::Key::F1);
                s.stage = 5;
            }
            5 => {
                let rect = self
                    .automation
                    .get("audio.MASTER.gain")
                    .ok_or_else(|| Error::Invalid("Master fader has no UI bounds".into()))?;
                let pos = egui::pos2(rect.center().x, rect.center().y + rect.height() * 0.12);
                s.events = vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ];
                s.stage = 6;
            }
            6 => {
                let rect = self
                    .automation
                    .get("audio.MASTER.gain")
                    .ok_or_else(|| Error::Invalid("Master fader disappeared".into()))?;
                s.events = vec![
                    egui::Event::PointerButton {
                        pos: rect.center(),
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                    egui::Event::PointerGone,
                ];
                s.stage = 10;
            }
            10 => {
                if !self.scripted_save("artifacts/smoke.rustmix.json", ctx)? {
                    return Ok(());
                }
                s.stage = 11;
            }
            11 => {
                if self.job.is_some() {
                    return Ok(());
                }
                if !PathBuf::from("artifacts/smoke.rustmix.json").exists() {
                    return Err(Error::Invalid(format!(
                        "UI save worker did not write the project (status: {})",
                        self.notice
                    )));
                }
                self.assert_smoke_state()?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 7;
            }
            7 if s.captured => {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1920.0, 1080.0)));
                s.stage = 8;
            }
            8 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 9;
            }
            9 if s.captured => {
                std::fs::write(
                    "artifacts/smoke-ok.json",
                    serde_json::to_vec_pretty(self.engine.project())?,
                )?;
                std::fs::write(
                    "artifacts/ui-elements.json",
                    serde_json::to_vec_pretty(
                        &self
                            .automation
                            .iter()
                            .map(|(id, r)| (id, format!("{r:?}")))
                            .collect::<HashMap<_, _>>(),
                    )?,
                )?;
                s.stage = 12;
            }

            12 => {
                let current = *self
                    .automation
                    .get("monitor.program.play")
                    .ok_or_else(|| Error::Invalid("Program play control missing".into()))?;
                if s.transport_target.is_none_or(|previous| {
                    (previous.min - current.min).length() > 0.5
                        || (previous.max - current.max).length() > 0.5
                }) {
                    s.transport_target = Some(current);
                    return Ok(());
                }
                let pos = s.transport_target.unwrap_or(current).center();
                std::fs::write(
                    format!("artifacts/pause-click-{}.json", s.stage),
                    serde_json::to_vec_pretty(
                        &serde_json::json!({"stage":s.stage,"position":format!("{pos:?}"),"current_bounds":format!("{current:?}"),"source":self.engine.project().program.0,"at_ms":self.now}),
                    )?,
                )?;
                s.events = vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                ];
                // Deliver one click, not a multi-second hold on a slow WARP
                // runner. egui rejects holds past max_click_duration (0.8s).
                s.stage = 14;
            }
            14 => {
                let clock = self
                    .engine
                    .playback(self.engine.project().program)
                    .ok_or_else(|| Error::Invalid("Program transport missing".into()))?;
                if !clock.paused() {
                    std::fs::write(
                        "artifacts/pause-input-failure.txt",
                        format!(
                            "pointer={:?}, target={:?}, source={}, notice={}",
                            ctx.input(|i| i.pointer.clone()),
                            s.transport_target,
                            self.engine.project().program.0,
                            self.notice
                        ),
                    )?;
                    return Err(Error::Invalid(
                        "Native pause control did not pause Program".into(),
                    ));
                }
                let progress = *self
                    .automation
                    .get("monitor.program.progress")
                    .ok_or_else(|| Error::Invalid("Program progress bar missing".into()))?;
                if let Some((position, pixels, previous_progress)) = &s.paused_program {
                    if *position != clock.position_ms()
                        || *pixels != self.last_program_rgba
                        || *previous_progress != progress
                    {
                        return Err(Error::Invalid(
                            "Paused Program picture or progress advanced".into(),
                        ));
                    }
                } else {
                    s.paused_program = Some((
                        clock.position_ms(),
                        self.last_program_rgba.clone(),
                        progress,
                    ));
                }
                if s.wait < 35 {
                    return Ok(());
                }
                std::fs::write(
                    "artifacts/pause-transport.json",
                    serde_json::to_vec_pretty(
                        &serde_json::json!({"held_frames":s.wait,"source":self.engine.project().program.0,"position_ms":clock.position_ms(),"picture_frozen":true,"progress_frozen":true}),
                    )?,
                )?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 15;
            }
            15 if s.captured => {
                self.execute(Command::RestartInput(self.engine.project().program));
                s.stage = 16;
            }
            16 => {
                let id = self.engine.project().program;
                if self
                    .engine
                    .playback(id)
                    .is_none_or(|clock| clock.position_ms() != 0 || !clock.paused())
                {
                    return Err(Error::Invalid(
                        "Paused restart did not return Program to zero".into(),
                    ));
                }
                self.execute(Command::SetInputPaused(id, false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1100.0, 700.0)));
                s.stage = 17;
            }
            17 => {
                let bank = self
                    .automation
                    .get("monitor.bank.bounds")
                    .ok_or_else(|| Error::Invalid("Small monitor bank missing".into()))?;
                if ctx.viewport_rect().bottom() - bank.bottom() < 230.0 {
                    return Err(Error::Invalid("Small native workspace hides inputs".into()));
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 18;
            }
            18 if s.captured => {
                self.assert_smoke_state()?;
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            20..=46 => {
                let offset = s.stage - 20;
                let Some((name, size, kind)) = DIALOG_SHOTS.get((offset / 3) as usize) else {
                    return Err(Error::Invalid("Unknown dialog screenshot step".into()));
                };
                match offset % 3 {
                    0 => {
                        self.reset_editors();
                        self.dispatch(
                            match kind {
                                0 => Action::Add,
                                1 => Action::Properties(InputId(3)),
                                2 => Action::Settings,
                                3 => Action::OverlaySettings(0),
                                4 => Action::RecordingSetup,
                                5 => Action::StreamingSetup,
                                6 => Action::MultiCorder,
                                7 => Action::PlayList,
                                _ => Action::ShortcutsWindow,
                            },
                            ctx,
                        );
                        let [width, height] = *size;
                        let target = egui::vec2(width, height);
                        if (ctx.viewport_rect().size() - target).length() > 2.0 {
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(target));
                        }
                        s.stage += 1;
                    }
                    1 => {
                        if !s.visual_settled(ctx) {
                            return Ok(());
                        }
                        self.validate_operator_chrome(
                            ctx,
                            match kind {
                                0 => "dialog.add_input",
                                1 => "dialog.input_properties",
                                2 => "dialog.settings",
                                3 => "dialog.overlay_settings",
                                4 => "dialog.recording_setup",
                                5 => "dialog.streaming_setup",
                                6 => "dialog.multicorder",
                                7 => "dialog.playlist",
                                _ => "dialog.shortcuts",
                            },
                        )?;
                        if *kind <= 3 {
                            self.validate_dialog_geometry(
                                ctx,
                                match kind {
                                    0 => "dialog.add",
                                    1 => "dialog.properties",
                                    2 => "dialog.settings",
                                    _ => "dialog.overlay",
                                },
                            )?;
                        } else if *kind == 4 {
                            self.validate_recording_geometry(ctx)?;
                        } else if *kind == 7 {
                            self.validate_playlist_geometry(ctx)?;
                        }
                        std::fs::write(
                            format!("artifacts/{name}-elements.json"),
                            serde_json::to_vec_pretty(
                                &self
                                    .automation
                                    .iter()
                                    .filter(|(id, _)| id.starts_with("dialog."))
                                    .map(|(id, rect)| (id, format!("{rect:?}")))
                                    .collect::<HashMap<_, _>>(),
                            )?,
                        )?;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::default(),
                        ));
                        s.captured = false;
                        s.stage += 1;
                    }
                    _ if s.captured => s.stage += 1,
                    _ => {}
                }
            }
            47 => {
                self.reset_editors();
                let project = self.engine.project();
                s.basic_state = Some((project.program, project.preview, project.revision));
                self.dispatch(Action::OverlaySettings(0), ctx);
                if let Some((_, draft)) = &mut self.overlay_settings {
                    draft.title = "Unsaved channel title".into();
                }
                s.stage = 48;
            }
            48 => {
                self.native_click(s, "dialog.overlay.slot.2")?;
                s.stage = 49;
            }
            49 => {
                if !s.visual_settled(ctx) {
                    return Ok(());
                }
                self.validate_basic_state(s)?;
                if self.overlay_switch_pending != Some(1) {
                    return Err(Error::Invalid(
                        "Dirty channel switch did not preserve its draft".into(),
                    ));
                }
                self.validate_dialog_geometry(ctx, "dialog.overlay")?;
                std::fs::write(
                    "artifacts/overlay-unsaved-1600-elements.json",
                    serde_json::to_vec_pretty(
                        &self
                            .automation
                            .iter()
                            .filter(|(id, _)| id.starts_with("dialog.overlay."))
                            .map(|(id, rect)| (id, format!("{rect:?}")))
                            .collect::<HashMap<_, _>>(),
                    )?,
                )?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 50;
            }
            50 if s.captured => {
                self.native_click(s, "dialog.overlay.cancel")?;
                s.stage = 51;
            }
            51 => {
                if self.overlay_settings.is_some() {
                    return Err(Error::Invalid("Cancel left the overlay editor open".into()));
                }
                self.validate_basic_state(s)?;
                self.assert_smoke_state()?;
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            _ => {}
        }
        Ok(())
    }

    /// Large reference readbacks have their own process and trace. The main
    /// switcher workflow keeps every original state/picture check and deadline.
    fn smoke_reference_layout(&mut self, ctx: &egui::Context, s: &mut Smoke) -> Result<()> {
        if self.start.elapsed() > Duration::from_secs(540) {
            return Err(Error::Invalid(format!(
                "Native reference layout timed out at stage {} after {} frames",
                s.stage, s.frames
            )));
        }
        for event in ctx.input(|input| input.events.clone()) {
            if let egui::Event::Screenshot {
                viewport_id, image, ..
            } = event
                && viewport_id == egui::ViewportId::ROOT
            {
                let (stem, requested) = match s.stage {
                    54 => ("workspace-reference", [1917, 1198]),
                    57 => ("workspace-reference-client", [1917, 1168]),
                    60 => ("add-input-reference", [1917, 1168]),
                    _ => {
                        return Err(Error::Invalid(
                            "Unexpected reference layout screenshot".into(),
                        ));
                    }
                };
                let expected = ctx.viewport_rect().size() * ctx.pixels_per_point();
                if image.width() != expected.x.round() as usize
                    || image.height() != expected.y.round() as usize
                {
                    return Err(Error::Invalid(
                        "Reference workspace screenshot has the wrong raster".into(),
                    ));
                }
                std::fs::write(
                    format!("artifacts/{stem}-raster.json"),
                    serde_json::to_vec_pretty(&serde_json::json!({
                        "requested_points": requested,
                        "actual_points": [ctx.viewport_rect().width(),ctx.viewport_rect().height()],
                        "actual_pixels": [image.width(),image.height()],
                        "pixels_per_point": ctx.pixels_per_point(),
                    }))?,
                )?;
                let [width, height] = requested;
                let bytes: Vec<u8> = image
                    .pixels
                    .iter()
                    .flat_map(|pixel| pixel.to_array())
                    .collect();
                image::save_buffer(
                    format!("artifacts/{stem}-{width}x{height}.png"),
                    &bytes,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                )
                .map_err(|error| Error::Invalid(error.to_string()))?;
                s.captured = true;
            }
        }
        if s.wait < 4 || self.job.is_some() {
            return Ok(());
        }
        match s.stage {
            0 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1600.0, 900.0)));
                s.stage = 19;
            }
            19 => {
                if (ctx.viewport_rect().size() - egui::vec2(1600.0, 900.0)).length() > 1.0 {
                    return Err(Error::Invalid(
                        "Reference layout did not start at 1600x900".into(),
                    ));
                }
                if self.program_presented_generation != Some(self.program_generation) {
                    return Ok(());
                }
                self.assert_smoke_state()?;
                let project = self.engine.project();
                s.basic_state = Some((project.program, project.preview, project.revision));
                self.dispatch(Action::Add, ctx);
                s.stage = 20;
            }
            20 => {
                self.validate_dialog_geometry(ctx, "dialog.add")?;
                self.native_click(s, "dialog.add.cancel")?;
                s.stage = 21;
            }
            21 => {
                if self.add_open {
                    return Err(Error::Invalid(
                        "Initial reference Input Select Cancel did not close".into(),
                    ));
                }
                self.validate_basic_state(s)?;
                s.stage = 52;
            }
            52 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1917.0, 1198.0)));
                s.stage = 53;
            }
            53 => {
                if (ctx.viewport_rect().size() - egui::vec2(1917.0, 1198.0)).length() > 1.0 {
                    return Err(Error::Invalid(
                        "Native workspace did not reach the reference viewport".into(),
                    ));
                }
                self.validate_monitor_geometry(ctx.viewport_rect().width())?;
                self.validate_toolbar_geometry(ctx.viewport_rect().width())?;
                std::fs::write(
                    "artifacts/workspace-reference-elements.json",
                    serde_json::to_vec_pretty(
                        &self
                            .automation
                            .iter()
                            .map(|(id, rect)| (id, format!("{rect:?}")))
                            .collect::<HashMap<_, _>>(),
                    )?,
                )?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 54;
            }
            54 if s.captured => {
                s.stage = 55;
            }
            55 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1917.0, 1168.0)));
                s.stage = 56;
            }
            56 => {
                if (ctx.viewport_rect().size() - egui::vec2(1917.0, 1168.0)).length() > 1.0 {
                    return Err(Error::Invalid(
                        "Native workspace did not reach the reference client viewport".into(),
                    ));
                }
                self.validate_monitor_geometry(ctx.viewport_rect().width())?;
                std::fs::write(
                    "artifacts/workspace-reference-client-elements.json",
                    serde_json::to_vec_pretty(
                        &self
                            .automation
                            .iter()
                            .map(|(id, rect)| (id, format!("{rect:?}")))
                            .collect::<HashMap<_, _>>(),
                    )?,
                )?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 57;
            }
            57 if s.captured => {
                self.dispatch(Action::Add, ctx);
                s.stage = 59;
            }
            59 => {
                if !s.visual_settled(ctx) {
                    return Ok(());
                }
                if (ctx.viewport_rect().size() - egui::vec2(1917.0, 1168.0)).length() > 1.0
                    || self.add_tab != dialogs::VIDEO_TAB
                {
                    return Err(Error::Invalid(
                        "Reference Input Select has the wrong viewport or source page".into(),
                    ));
                }
                self.validate_dialog_geometry(ctx, "dialog.add")?;
                let body = self.native_geometry_rect(ctx, "dialog.add.body")?;
                if !(0.66..=0.73).contains(&(body.width() / ctx.viewport_rect().width())) {
                    return Err(Error::Invalid(
                        "Default Input Select width did not follow the enlarged viewport".into(),
                    ));
                }
                std::fs::write(
                    "artifacts/add-input-reference-elements.json",
                    serde_json::to_vec_pretty(
                        &self
                            .automation
                            .iter()
                            .filter(|(id, _)| id.starts_with("dialog.add"))
                            .map(|(id, rect)| (id, format!("{rect:?}")))
                            .collect::<HashMap<_, _>>(),
                    )?,
                )?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 60;
            }
            60 if s.captured => {
                self.native_click(s, "dialog.add.cancel")?;
                s.stage = 61;
            }
            61 => {
                if self.add_open {
                    return Err(Error::Invalid(
                        "Reference Input Select Cancel did not close".into(),
                    ));
                }
                self.assert_smoke_state()?;
                // Native processes share the disposable eframe window store.
                // Do not leak this additional large reference capture into the
                // existing source/audio/video workflows' 1600x900 workload.
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1600.0, 900.0)));
                s.stage = 58;
            }
            58 => {
                if (ctx.viewport_rect().size() - egui::vec2(1600.0, 900.0)).length() > 1.0 {
                    return Err(Error::Invalid(
                        "Reference capture did not restore the native workflow viewport".into(),
                    ));
                }
                self.validate_monitor_geometry(ctx.viewport_rect().width())?;
                std::fs::write(
                    "artifacts/workspace-reference-restore.json",
                    serde_json::to_vec_pretty(&serde_json::json!({
                        "requested_points": [1600,900],
                        "actual_points": [ctx.viewport_rect().width(),ctx.viewport_rect().height()],
                        "pixels_per_point": ctx.pixels_per_point(),
                        "reference_client_capture_completed": s.captured,
                    }))?,
                )?;
                self.assert_smoke_state()?;
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            54 | 57 | 60 => {} // A readback must arrive before advancing its stage.
            _ => {
                return Err(Error::Invalid(
                    "Unknown native reference layout stage".into(),
                ));
            }
        }
        if s.stage >= 52 {
            self.validate_basic_state(s)?;
        }
        Ok(())
    }

    pub(crate) fn native_click(&self, s: &mut Smoke, id: &str) -> Result<()> {
        let pos = self
            .automation
            .get(id)
            .ok_or_else(|| Error::Invalid(format!("Missing native control {id}")))?
            .center();
        s.events = vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ];
        Ok(())
    }
    fn assert_input_smoke(&self) -> Result<()> {
        let p = self.engine.project();
        let source = p
            .input(InputId(8))
            .ok_or_else(|| Error::Invalid("Imported PNG is missing".into()))?;
        let copy = p
            .input(InputId(9))
            .ok_or_else(|| Error::Invalid("Duplicated PNG is missing".into()))?;
        if p.program != InputId(9)
            || p.inputs.get(1).map(|i| i.id) != Some(InputId(8))
            || p.inputs.get(2).map(|i| i.id) != Some(InputId(9))
            || source.config != copy.config
            || !source.config.mirror
            || source.config.gain != 0.5
            || source.config.media_path.is_none()
            || !copy.audio.muted
            || self.engine.playlist_run().is_some()
            || p.playlists
                .first()
                .is_none_or(|list| list.items.len() != 2 || !list.manual)
            || p.transition_slots.first().is_none_or(|slot| {
                slot.kind != TransitionKind::SlideReverse || slot.duration_ms != 250
            })
            || !p.preferences.key_bindings.iter().any(|binding| {
                binding.key == "X" && binding.action == ShortcutAction::Preview(InputId(8))
            })
            || self.last_program_rgba.is_empty()
            || !self
                .last_program_rgba
                .chunks_exact(4)
                .all(|p| p == [50, 20, 10, 255])
        {
            return Err(Error::Invalid(
                "PNG import/order/effects/duplicate/pixel acceptance failed".into(),
            ));
        }
        Ok(())
    }
    fn input_reopen_presented(&self) -> Result<bool> {
        if self.job.is_some()
            || self.media_loader.busy()
            || self.program_presented_generation != Some(self.program_generation)
        {
            return Ok(false);
        }
        self.assert_input_smoke()?;
        Ok(true)
    }
    fn smoke_inputs(&mut self, ctx: &egui::Context, s: &mut Smoke) -> Result<()> {
        // Includes the playlist, keyboard editor and Basic-mode capture stages.
        if self.start.elapsed() > Duration::from_secs(150) {
            return Err(Error::Invalid(format!(
                "Native input workflow timed out at stage {}",
                s.stage
            )));
        }
        let reopen = std::env::args().any(|s| s == "--ui-input-smoke-reopen");
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let name = if reopen {
                    "input-reopened"
                } else if s.stage == 10 {
                    "input-properties"
                } else if s.stage == 20 {
                    "input-playlist"
                } else if s.stage == 24 {
                    "input-keyboard"
                } else if s.stage == 27 {
                    "input-basic"
                } else {
                    "input-workflow"
                };
                let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                image::save_buffer(
                    format!("artifacts/{name}.png"),
                    &bytes,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                )
                .map_err(|e| Error::Invalid(e.to_string()))?;
                s.captured = true;
            }
        }
        if s.wait < 5 {
            return Ok(());
        }
        self.validate_monitor_geometry(ctx.viewport_rect().right())?;
        if reopen {
            // Media completion submits a new producer generation after this
            // frame's poll_program. Never judge the previous (possibly black)
            // raster merely because the decode queue just became idle.
            if !self.input_reopen_presented()? {
                return Ok(());
            }
            if s.stage == 0 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 1;
            } else if s.captured {
                std::fs::write(
                    "artifacts/input-reopen-ok.json",
                    serde_json::to_vec_pretty(self.engine.project())?,
                )?;
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            return Ok(());
        }
        match s.stage {
            0 => {
                std::fs::create_dir_all("artifacts")?;
                let path = PathBuf::from("artifacts/input-fixture.png");
                image::save_buffer(
                    &path,
                    &[100, 40, 20, 255].repeat(16 * 9),
                    16,
                    9,
                    image::ColorType::Rgba8,
                )
                .map_err(|e| Error::Invalid(e.to_string()))?;
                self.dispatch(Action::Add, ctx);
                self.add_number = 2;
                s.stage = 1;
            }
            1 => {
                self.native_click(s, "dialog.add.image")?;
                s.stage = 29;
            }
            29 => {
                self.validate_dialog_geometry(ctx, "dialog.add")?;
                self.add_png_path = PathBuf::from("artifacts/input-fixture.png")
                    .display()
                    .to_string();
                self.check_add_png();
                s.stage = 30;
            }
            30 => {
                if self.add_loading || self.media_loader.busy() {
                    return Ok(());
                }
                if let Some(reason) = self.add_disabled_reason() {
                    return Err(Error::Invalid(reason));
                }
                self.native_click(s, "dialog.add.confirm")?;
                s.stage = 2;
            }
            2 => {
                if self.add_loading || self.media_loader.busy() {
                    return Ok(());
                }
                if self.add_open
                    || self.engine.project().inputs.get(1).map(|i| i.id) != Some(InputId(8))
                    || self.engine.project().program != InputId(1)
                    || self.engine.project().preview != InputId(2)
                {
                    return Err(Error::Invalid(format!(
                        "Native PNG Add Input failed: {}",
                        self.notice
                    )));
                }
                self.dispatch(Action::Properties(InputId(8)), ctx);
                s.stage = 3;
            }
            3 => {
                self.prop_mirror = true;
                self.prop_gain = 0.5;
                self.sync_fx_from_prop(InputId(8));
                self.native_click(s, "dialog.properties.cancel")?;
                s.stage = 4;
            }
            4 => {
                let input = self
                    .engine
                    .project()
                    .input(InputId(8))
                    .ok_or_else(|| Error::Invalid("PNG missing after Cancel".into()))?;
                if self.properties.is_some() || input.config.mirror || input.config.gain != 1.0 {
                    return Err(Error::Invalid(
                        "Native Properties Cancel changed committed effects".into(),
                    ));
                }
                self.dispatch(Action::Properties(InputId(8)), ctx);
                s.stage = 5;
            }
            5 => {
                self.prop_mirror = true;
                self.prop_gain = 0.5;
                self.sync_fx_from_prop(InputId(8));
                self.native_click(s, "dialog.properties.apply")?;
                s.stage = 6;
            }
            6 => {
                if self.properties.is_some() {
                    return Err(Error::Invalid(
                        "Native Apply did not close Properties".into(),
                    ));
                }
                self.execute(Command::DuplicateInput(InputId(8)));
                self.execute(Command::TakeInput(InputId(9)));
                s.stage = 11;
            }
            7 => {
                self.assert_input_smoke()?;
                if !self.scripted_save("artifacts/inputs.rustmix.json", ctx)? {
                    return Ok(());
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 8;
            }
            8 => {
                if self.job.is_some() || !s.captured {
                    return Ok(());
                }
                if !PathBuf::from("artifacts/inputs.rustmix.json").exists() {
                    return Err(Error::Invalid(format!(
                        "Input smoke save did not write the project (status: {})",
                        self.notice
                    )));
                }
                self.assert_input_smoke()?;
                std::fs::write(
                    "artifacts/input-smoke-ok.json",
                    serde_json::to_vec_pretty(self.engine.project())?,
                )?;
                self.dispatch(Action::Properties(InputId(9)), ctx);
                s.stage = 9;
            }
            9 => {
                self.validate_dialog_geometry(ctx, "dialog.properties")?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 10;
            }
            10 if s.captured => {
                self.close_properties();
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            11 => {
                self.execute(Command::SetTransitionSlot(
                    0,
                    TransitionSlot {
                        kind: TransitionKind::SlideReverse,
                        duration_ms: 250,
                    },
                ));
                let mut preferences = self.engine.project().preferences.clone();
                preferences.key_bindings.push(ShortcutBinding {
                    key: "X".into(),
                    ctrl: false,
                    shift: false,
                    alt: false,
                    action: ShortcutAction::Preview(InputId(8)),
                });
                self.execute(Command::SetPreferences(preferences));
                self.dispatch(Action::PlayList, ctx);
                s.stage = 12;
            }
            12 => {
                self.native_click(s, "dialog.playlist.available.8")?;
                s.stage = 13;
            }
            13 => {
                self.native_click(s, "dialog.playlist.add")?;
                s.stage = 14;
            }
            14 => {
                self.native_click(s, "dialog.playlist.available.9")?;
                s.stage = 15;
            }
            15 => {
                self.native_click(s, "dialog.playlist.add")?;
                s.stage = 16;
            }
            16 => {
                self.native_click(s, "dialog.playlist.manual")?;
                s.stage = 17;
            }
            17 => {
                self.native_click(s, "dialog.playlist.start")?;
                s.stage = 18;
            }
            18 => {
                if self.engine.playlist_run().is_none() {
                    return Err(Error::Invalid("Native playlist did not start".into()));
                }
                self.native_click(s, "dialog.playlist.start")?;
                s.stage = 19;
            }
            19 => {
                if self.engine.playlist_run().is_some() {
                    return Err(Error::Invalid("Native playlist did not stop".into()));
                }
                if self.engine.transition().is_some() {
                    return Ok(());
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 20;
            }
            20 if s.captured => {
                self.native_click(s, "dialog.playlist.close")?;
                s.stage = 21;
            }
            21 => {
                self.execute(Command::TakeInput(InputId(9)));
                // Taking 9 already puts 8 on Preview. Force a distinct initial
                // selection so the X assertion proves actual shortcut dispatch.
                self.execute(Command::SelectPreview(InputId(1)));
                s.events = key(egui::Key::X);
                s.stage = 22;
            }
            22 => {
                if self.engine.project().preview != InputId(8) {
                    return Err(Error::Invalid(
                        "Native remapped shortcut did not select its input".into(),
                    ));
                }
                self.dispatch(Action::SettingsAt(SettingsPage::Shortcuts), ctx);
                s.stage = 23;
            }
            23 => {
                self.validate_dialog_geometry(ctx, "dialog.settings")?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 24;
            }
            24 if s.captured => {
                self.native_click(s, "dialog.settings.cancel")?;
                s.stage = 25;
            }
            25 => {
                if self.settings {
                    return Err(Error::Invalid(
                        "Native keyboard settings did not close".into(),
                    ));
                }
                let project = self.engine.project();
                s.basic_state = Some((project.program, project.preview, project.revision));
                self.native_click(s, "toolbar.basic")?;
                s.stage = 26;
            }
            26 => {
                self.validate_basic_controls()?;
                self.validate_basic_state(s)?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 27;
            }
            27 if s.captured => {
                self.native_click(s, "toolbar.basic")?;
                s.stage = 28;
            }
            28 => {
                self.validate_basic_state(s)?;
                if self.basic_mode || !self.automation.contains_key("switcher.cut") {
                    return Err(Error::Invalid("Advanced controls were not restored".into()));
                }
                s.stage = 7;
            }
            _ => {}
        }
        Ok(())
    }
    fn validate_basic_state(&self, s: &Smoke) -> Result<()> {
        let project = self.engine.project();
        if s.basic_state != Some((project.program, project.preview, project.revision)) {
            return Err(Error::Invalid(
                "Changing operator mode modified the live project".into(),
            ));
        }
        Ok(())
    }
    fn validate_basic_controls(&self) -> Result<()> {
        if !self.basic_mode
            || [
                "switcher.cut",
                "switcher.fade",
                "output.record",
                "output.playlist",
                "toolbar.fullscreen",
                "input.1.loop",
                "input.1.audio",
            ]
            .iter()
            .any(|id| self.automation.contains_key(*id))
            || [
                "switcher.quickPlay",
                "production.add",
                "output.lock",
                "toolbar.basic",
                "input.1.cut",
                "input.1.preview",
            ]
            .iter()
            .any(|id| !self.automation.contains_key(*id))
        {
            return Err(Error::Invalid(
                "Basic operator controls do not match their scope".into(),
            ));
        }
        Ok(())
    }
    fn assert_smoke_state(&self) -> Result<()> {
        let p = self.engine.project();
        if p.program != InputId(3)
            || p.preview != InputId(2)
            || !p.overlays.standard.first().is_some_and(|o| o.enabled)
            || !p.buses.first().is_some_and(|b| b.gain_db < -1.0)
            || p.workspace.dock_json.is_empty()
        {
            return Err(Error::Invalid(
                "Smoke final state mismatch (PGM/PVW/overlay/master/workspace)".into(),
            ));
        }
        Ok(())
    }
    pub(crate) fn validate_dialog_geometry(&self, ctx: &egui::Context, prefix: &str) -> Result<()> {
        let window = match prefix {
            "dialog.add" => "dialog.add_input",
            "dialog.properties" => "dialog.input_properties",
            "dialog.overlay" => "dialog.overlay_settings",
            "dialog.external" => "dialog.external_settings",
            _ => prefix,
        };
        self.validate_operator_chrome(ctx, window)?;
        let bounds = ctx.viewport_rect();
        let body = self
            .automation
            .get(&format!("{prefix}.body"))
            .ok_or_else(|| Error::Invalid(format!("Missing {prefix} body")))?;
        let footer = if prefix == "dialog.add" {
            "confirm"
        } else {
            "apply"
        };
        for control in [footer, "cancel"] {
            let rect = self
                .automation
                .get(&format!("{prefix}.{control}"))
                .ok_or_else(|| Error::Invalid(format!("Missing {prefix} {control}")))?;
            if !rect.is_finite()
                || !bounds.contains_rect(*rect)
                || rect.height() < 27.0
                || rect.width() < 85.0
                || body.bottom() > rect.top() + 1.0
            {
                return Err(Error::Invalid(format!(
                    "Dialog body or footer is clipped: {prefix} {control} {rect:?}"
                )));
            }
        }
        if prefix == "dialog.add" && self.add_tab == dialogs::VIDEO_TAB {
            let page = self.native_geometry_rect(ctx, "dialog.add.page")?;
            for id in ["path", "browse", "recent", "name", "validate", "clear"] {
                let control = self.native_geometry_rect(ctx, &format!("dialog.add.video.{id}"))?;
                if !page.expand(1.0).contains_rect(control) {
                    return Err(Error::Invalid(format!(
                        "Video selection field escapes its page: {id}"
                    )));
                }
            }
            let path = self.native_geometry_rect(ctx, "dialog.add.video.path")?;
            let browse = self.native_geometry_rect(ctx, "dialog.add.video.browse")?;
            let list = self.native_geometry_rect(ctx, "dialog.add.video.recent")?;
            if (path.center().y - browse.center().y).abs() > 1.0
                || path.right() >= browse.left()
                || path.width() < page.width() * 0.65
                || list.width() < page.width() * 0.94
                || list.height() < page.height() * 0.45
            {
                return Err(Error::Invalid(
                    "Video file row/list proportions differ from the operator layout".into(),
                ));
            }
        } else if prefix == "dialog.properties" && self.input_page == InputPage::General {
            let page = self.native_geometry_rect(ctx, "dialog.properties.page")?;
            for key in ["name", "aspect", "category"] {
                let label =
                    self.native_geometry_rect(ctx, &format!("dialog.properties.{key}.label"))?;
                let field = self.native_geometry_rect(ctx, &format!("dialog.properties.{key}"))?;
                let gutter = field.left() - label.right();
                if !page.expand(1.0).contains_rect(field)
                    || !(6.0..=10.0).contains(&gutter)
                    || (bounds.width() >= 1600.0 && field.width() < page.width() * 0.30)
                {
                    return Err(Error::Invalid(format!(
                        "Input General field is narrow, clipped or overlaps its label: {key}"
                    )));
                }
            }
            let header = self.native_geometry_rect(ctx, "dialog.properties.sourceHeader")?;
            let change = self.native_geometry_rect(ctx, "dialog.properties.change")?;
            if header.right() + 2.0 > change.left() {
                return Err(Error::Invalid("Source caption overlaps Change".into()));
            }
        } else if prefix == "dialog.overlay" {
            for (label, field) in [
                ("number", "slot.1"),
                ("title", "title"),
                ("subtitle", "subtitle"),
                ("position", "position.LowerLeft"),
                ("visibility", "enabled"),
            ] {
                let label =
                    self.native_geometry_rect(ctx, &format!("dialog.overlay.label.{label}"))?;
                let field = self.native_geometry_rect(ctx, &format!("dialog.overlay.{field}"))?;
                if label.right() + 6.0 > field.left() || !body.contains_rect(label) {
                    return Err(Error::Invalid(
                        "Overlay label overlaps or escapes its field column".into(),
                    ));
                }
            }
        } else if prefix == "dialog.settings" {
            let nav = self.native_geometry_rect(ctx, "dialog.settings.navigation")?;
            let page = self.native_geometry_rect(ctx, "dialog.settings.page")?;
            if self.settings_page == SettingsPage::Display {
                for id in ["dialog.settings.frameRate", "dialog.settings.projectName"] {
                    if !page.contains_rect(self.native_geometry_rect(ctx, id)?) {
                        return Err(Error::Invalid(format!(
                            "Settings field escapes its page: {id}"
                        )));
                    }
                }
            }
            if ctx.viewport_rect().width() >= 1100.0
                && !(0.20..=0.24).contains(&(nav.width() / body.width()))
            {
                return Err(Error::Invalid(
                    "Settings navigation proportion differs from its reference target".into(),
                ));
            }
        }
        Ok(())
    }

    fn validate_operator_chrome(&self, ctx: &egui::Context, name: &str) -> Result<()> {
        let get = |suffix| self.native_geometry_rect(ctx, &format!("{name}.chrome.{suffix}"));
        let window = get("window")?;
        let header = get("header")?;
        let title = get("title")?;
        let close = get("close")?;
        if !window.contains_rect(header)
            || !header.contains_rect(title)
            || !header.contains_rect(close)
            || title.right() >= close.left()
            || !(24.0..=34.0).contains(&(title.left() - header.left()))
            || !(24.0..=32.1).contains(&header.height())
            || close.width() < 34.0
            || (close.right() - header.right()).abs() > 1.0
        {
            return Err(Error::Invalid(format!(
                "Operator caption is clipped or misaligned: {name}"
            )));
        }
        Ok(())
    }
    fn native_geometry_rect(&self, ctx: &egui::Context, id: &str) -> Result<Rect> {
        self.automation
            .get(id)
            .copied()
            .filter(|rect| {
                rect.is_finite() && rect.is_positive() && ctx.viewport_rect().contains_rect(*rect)
            })
            .ok_or_else(|| Error::Invalid(format!("Missing or clipped native control: {id}")))
    }

    fn validate_playlist_geometry(&self, ctx: &egui::Context) -> Result<()> {
        let available = self.native_geometry_rect(ctx, "dialog.playlist.available.body")?;
        let ordered = self.native_geometry_rect(ctx, "dialog.playlist.items.body")?;
        let add = self.native_geometry_rect(ctx, "dialog.playlist.add")?;
        let remove = self.native_geometry_rect(ctx, "dialog.playlist.remove")?;
        let up = self.native_geometry_rect(ctx, "dialog.playlist.up")?;
        let down = self.native_geometry_rect(ctx, "dialog.playlist.down")?;
        let flags = self.native_geometry_rect(ctx, "dialog.playlist.loop")?;
        let start = self.native_geometry_rect(ctx, "dialog.playlist.start")?;
        let previous = self.native_geometry_rect(ctx, "dialog.playlist.previous")?;
        let next = self.native_geometry_rect(ctx, "dialog.playlist.next")?;
        let close = self.native_geometry_rect(ctx, "dialog.playlist.close")?;
        let row = self.native_geometry_rect(ctx, "dialog.playlist.available.1")?;
        let transfer_in_gutter = [add, remove]
            .iter()
            .all(|button| available.right() <= button.left() && button.right() <= ordered.left());
        let footer = [start, previous, next, close];
        if !transfer_in_gutter
            || (available.top() - ordered.top()).abs() > 1.0
            || (ordered.width() / available.width() - 1.294).abs() > 0.05
            || ordered.right() > up.left()
            || (add.top() - up.top()).abs() > 1.0
            || (remove.top() - down.top()).abs() > 1.0
            || flags.top() + 0.5 < ordered.bottom() + 8.0
            || footer.iter().any(|button| {
                button.top() < ordered.bottom().max(available.bottom()) || button.height() < 27.0
            })
            || footer.windows(2).any(|pair| {
                pair.first()
                    .zip(pair.get(1))
                    .is_some_and(|(left, right)| left.right() > right.left())
            })
            || close.right() < up.right() - 1.0
            || !available.contains_rect(row)
            || row.width() < available.width() - 28.0
        {
            return Err(Error::Invalid(
                "Native Playlist panes, hit areas or controls differ from reference geometry"
                    .into(),
            ));
        }
        Ok(())
    }
    pub(crate) fn validate_recording_geometry(&self, ctx: &egui::Context) -> Result<()> {
        let read = |name: &str| {
            self.automation
                .get(&format!("dialog.recording.{name}"))
                .copied()
                .filter(|rect| rect.is_finite())
                .ok_or_else(|| Error::Invalid(format!("Missing recording geometry: {name}")))
        };
        let body = read("body")?;
        let rail = read("rail")?;
        let page = read("page")?;
        let filename = read("filename")?;
        let browse = read("browse")?;
        let start = read("start")?;
        let ok = read("ok")?;
        let cancel = read("cancel")?;
        if !body.contains_rect(rail)
            || !body.contains_rect(page)
            || (rail.width() / body.width().max(1.0) - 0.26).abs() > 0.015
            || rail.right() + 5.0 > page.left()
            || !page.contains_rect(filename)
            || !page.contains_rect(browse)
            || filename.bottom() > browse.top()
            || body.bottom() > start.top()
            || body.bottom() > ok.top()
            || body.bottom() > cancel.top()
            || start.right() > ok.left()
            || ok.right() > cancel.left()
            || [body, start, ok, cancel]
                .iter()
                .any(|rect| !ctx.viewport_rect().contains_rect(*rect))
        {
            return Err(Error::Invalid(
                "Recording page, navigation or footer is clipped or overlaps".into(),
            ));
        }
        Ok(())
    }
    fn validate_toolbar_geometry(&self, right: f32) -> Result<()> {
        let mut top = vec![
            "toolbar.brand",
            "toolbar.open",
            "toolbar.open.menu",
            "toolbar.save",
            "toolbar.saveAs",
            "toolbar.save.menu",
            "toolbar.settings",
            "toolbar.settings.menu",
            "toolbar.basic",
        ];
        if !self.basic_mode {
            top.extend([
                "toolbar.new",
                "toolbar.last",
                "toolbar.fullscreen",
                "output.program.menu",
                "toolbar.pause",
            ]);
        }
        let production = if self.basic_mode {
            vec!["production.add", "output.lock"]
        } else {
            vec![
                "production.add",
                "output.record.settings",
                "output.record",
                "output.external.settings",
                "output.external",
                "output.stream.settings",
                "output.stream",
                "output.multicorder.settings",
                "output.multicorder",
                "output.playlist.settings",
                "output.playlist",
                "output.overlay",
                "output.shortcuts",
                "output.audioPane",
                "output.multiview",
                "output.lock",
            ]
        };
        for row in [&top[..], &production[..]] {
            for (index, id) in row.iter().enumerate() {
                let rect = self
                    .automation
                    .get(*id)
                    .ok_or_else(|| Error::Invalid(format!("Missing toolbar control: {id}")))?;
                if !rect.is_finite()
                    || rect.left() < 0.0
                    || rect.right() > right
                    || rect.height() < 21.0
                {
                    return Err(Error::Invalid(format!(
                        "Toolbar target is clipped: {id} {rect:?}"
                    )));
                }
                for other in row.iter().skip(index + 1) {
                    if self
                        .automation
                        .get(*other)
                        .is_some_and(|next| rect.intersects(*next))
                    {
                        return Err(Error::Invalid(format!(
                            "Toolbar controls overlap: {id} / {other}"
                        )));
                    }
                }
            }
        }
        Ok(())
    }
    fn validate_monitor_geometry(&self, right: f32) -> Result<()> {
        self.validate_toolbar_geometry(right)?;
        let preview = self
            .automation
            .get("monitor.preview")
            .ok_or_else(|| Error::Invalid("Preview viewport was not rendered".into()))?;
        let program = self
            .automation
            .get("monitor.program")
            .ok_or_else(|| Error::Invalid("Program viewport was not rendered".into()))?;
        let take_id = if self.basic_mode {
            "switcher.quickPlay"
        } else {
            "switcher.cut"
        };
        let cut = self
            .automation
            .get(take_id)
            .ok_or_else(|| Error::Invalid(format!("{take_id} control was not rendered")))?;
        for rect in [preview, program] {
            if !rect.is_finite()
                || rect.width() < 300.0
                || rect.height() < 168.0
                || (rect.width() / rect.height() - 16.0 / 9.0).abs() > 0.01
                || rect.right() > right + 2.0
            {
                return Err(Error::Invalid(format!(
                    "Monitor viewport is clipped or collapsed: {rect:?}"
                )));
            }
        }
        if preview.right() > cut.left() || cut.right() > program.left() {
            return Err(Error::Invalid(
                "Transition controls overlap the monitors".into(),
            ));
        }
        for (name, picture) in [("preview", preview), ("program", program)] {
            let header = self
                .automation
                .get(&format!("monitor.{name}.header"))
                .ok_or_else(|| Error::Invalid(format!("Missing {name} header")))?;
            let transport = self
                .automation
                .get(&format!("monitor.{name}.transport"))
                .ok_or_else(|| Error::Invalid(format!("Missing {name} transport")))?;
            if (picture.top() - header.bottom()).abs() > 4.0
                || (transport.top() - picture.bottom()).abs() > 4.0
            {
                return Err(Error::Invalid(format!(
                    "{name} picture/transport floats inside the monitor bank: {header:?}, {picture:?}, {transport:?}"
                )));
            }
        }
        if !self.basic_mode {
            for index in 1..=4 {
                let button = self
                    .automation
                    .get(&format!("switcher.slot.{index}"))
                    .ok_or_else(|| Error::Invalid("Transition slot was not rendered".into()))?;
                let menu = self
                    .automation
                    .get(&format!("switcher.slot.{index}.settings"))
                    .ok_or_else(|| {
                        Error::Invalid("Transition slot menu was not rendered".into())
                    })?;
                if (button.left() - cut.left()).abs() > 1.0
                    || menu.left() < button.right()
                    || menu.right() > cut.right() + 1.0
                {
                    return Err(Error::Invalid(format!(
                        "Transition slot {index} exceeds the take column"
                    )));
                }
            }
            let fade = self
                .automation
                .get("switcher.fade")
                .ok_or_else(|| Error::Invalid("FADE control was not rendered".into()))?;
            if (cut.left() - fade.left()).abs() > 2.0 || fade.top() < cut.bottom() {
                return Err(Error::Invalid("Transition controls are not stacked".into()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
    #[test]
    fn native_visual_capture_requires_elapsed_animation_time_not_only_pass_count() {
        let mut state = super::Smoke {
            visual_started_at: Some(10.0),
            wait: 4000,
            ..Default::default()
        };
        assert!(!state.visual_settled_at(10.0, 0.2));
        assert!(!state.visual_settled_at(10.05, 0.2));
        assert!(!state.visual_settled_at(10.2, 0.2));
        assert!(state.visual_settled_at(10.5, 0.2));
        assert!(!state.visual_settled_at(9.0, 0.2));
        assert!(!state.visual_settled_at(f64::NAN, 0.2));
        assert!(!state.visual_settled_at(11.0, f32::NAN));
        state.visual_started_at = None;
        assert!(!state.visual_settled_at(11.0, 0.2));
    }

    #[test]
    fn queue_completion_probe_keeps_only_one_pending_registration_and_bounds_delivery() {
        let mut probe = super::QueueCompletionProbe::default();
        let (token, completed) = probe.begin_if_idle(10).unwrap();
        assert_eq!(token, 1);
        // Simulate a callback that never arrives. Heartbeats retain one atomic
        // registration; they must not enqueue a callback for every new sample.
        for elapsed_ms in 11..=1000 {
            assert!(probe.begin_if_idle(elapsed_ms).is_none());
            let sample = probe.observe(elapsed_ms, || elapsed_ms + 1);
            assert_eq!(sample.requested_token, 1);
            assert_eq!(sample.observed_token, 0);
            assert_eq!(sample.pending_age_lower_bound_ms, Some(elapsed_ms - 10));
            assert!(sample.callback_delivery_age_bounds_ms.is_none());
            assert_eq!(std::sync::Arc::strong_count(&completed), 2);
        }
        // The production callback consists solely of this Release store. This
        // unit test simulates delivery; actual GPU callback delivery needs CI.
        completed.store(token, super::Ordering::Release);
        assert!(
            probe.begin_if_idle(1001).is_none(),
            "record completion before superseding its token"
        );
        let delivered = probe.observe(1001, || 1002);
        assert_eq!(delivered.requested_token, delivered.observed_token);
        assert_eq!(delivered.request_elapsed_ms, Some(10));
        assert_eq!(delivered.pending_age_lower_bound_ms, None);
        assert_eq!(delivered.callback_delivery_age_bounds_ms, Some([990, 992]));
        let still_delivered = probe.observe(2000, || 2001);
        assert_eq!(
            still_delivered.callback_delivery_age_bounds_ms,
            Some([990, 992])
        );
        let (next, _) = probe.begin_if_idle(2002).unwrap();
        assert_eq!(next, 2);
        let pending = probe.observe(2003, || 2004);
        assert_eq!(pending.requested_token, 2);
        assert_eq!(pending.observed_token, 1);
        assert_eq!(pending.pending_age_lower_bound_ms, Some(1));
        assert_eq!(pending.callback_delivery_age_bounds_ms, None);
    }

    #[test]
    fn queue_completion_probe_handles_immediate_delivery_and_token_exhaustion() {
        let mut probe = super::QueueCompletionProbe::default();
        let (token, completed) = probe.begin_if_idle(20).unwrap();
        completed.store(token, super::Ordering::Release);
        // A callback delivered during a five-ms poll must not be labelled [0,0].
        let sample = probe.observe(25, || 25);
        assert_eq!(sample.callback_delivery_age_bounds_ms, Some([0, 5]));
        assert_eq!(sample.pending_age_lower_bound_ms, None);
        let (next, completed) = probe.begin_if_idle(26).unwrap();
        completed.store(next, super::Ordering::Release);
        let immediate = probe.observe(26, || 26);
        assert_eq!(immediate.callback_delivery_age_bounds_ms, Some([0, 0]));

        let mut racing = super::QueueCompletionProbe::default();
        let (token, completed) = racing.begin_if_idle(100).unwrap();
        let pending = racing.observe(105, || {
            completed.store(token, super::Ordering::Release);
            110
        });
        assert_eq!(pending.observed_token, 0);
        assert_eq!(pending.pending_age_lower_bound_ms, Some(5));
        let delivered = racing.observe(111, || 112);
        assert_eq!(delivered.callback_delivery_age_bounds_ms, Some([5, 12]));

        let mut exhausted = super::QueueCompletionProbe {
            requested_token: u64::MAX,
            observed_token: u64::MAX,
            completed: std::sync::Arc::new(super::AtomicU64::new(u64::MAX)),
            ..Default::default()
        };
        assert!(exhausted.begin_if_idle(30).is_none());
        assert_eq!(exhausted.requested_token, u64::MAX);
    }

    #[test]
    fn native_stage_diagnostics_bound_history_and_keep_the_latest_failure() {
        let mut smoke = super::Smoke::default();
        for stage in 0..200 {
            smoke.push_stage_sample(super::SmokeStageSample {
                stage,
                frames: stage,
                elapsed_ms: u64::from(stage) * 1000,
                captured: false,
                failed: stage == 199,
                program_generation: 0,
                presented_generation: None,
                gpu_poll_status: "Poll",
                ..Default::default()
            });
        }
        assert_eq!(smoke.stage_trace.len(), 128);
        assert_eq!(smoke.stage_trace.front().unwrap().stage, 72);
        let latest = smoke.stage_trace.back().unwrap();
        assert_eq!(latest.stage, 199);
        assert!(latest.failed);
    }

    use super::*;

    #[test]
    fn reference_layout_waits_for_each_gpu_readback_without_advancing_or_accepting_unknown_stages()
    {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        let before = app.engine.project().clone();
        let mut state = Smoke {
            wait: 5,
            basic_state: Some((before.program, before.preview, before.revision)),
            ..Default::default()
        };
        for stage in [54, 57, 60] {
            state.stage = stage;
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                app.smoke_reference_layout(ui.ctx(), &mut state).unwrap();
            });
            assert_eq!(
                state.stage, stage,
                "A missing readback cannot satisfy capture"
            );
            assert!(!state.captured);
            assert_eq!(*app.engine.project(), before);
        }
        state.stage = 999;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            assert!(app.smoke_reference_layout(ui.ctx(), &mut state).is_err());
        });
    }

    #[test]
    fn png_reopen_waits_for_the_new_generation_then_enforces_state_acceptance() {
        let mut app = RustMixApp::headless();
        // Reproduce the frame after decode completion submitted generation 2,
        // while only the old black frame has been presented.
        app.program_generation = 2;
        app.program_presented_generation = Some(1);
        app.last_program_rgba = vec![0; 640 * 360 * 4];
        assert!(!app.media_loader.busy());
        assert!(!app.input_reopen_presented().unwrap());
        app.program_presented_generation = None;
        assert!(!app.input_reopen_presented().unwrap());
        // The acknowledgment gate must not bypass the actual acceptance: a
        // current frame with missing imported sources remains a failure.
        app.program_presented_generation = Some(2);
        assert!(app.input_reopen_presented().is_err());
    }

    #[test]
    fn scripted_save_waits_for_the_file_job_slot_and_writes_the_project() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("scripted.rustmix.json");
        let path = path.to_str().unwrap().to_owned();
        let (_sender, receiver) = std::sync::mpsc::sync_channel(1);
        app.job = Some(receiver);
        assert!(!app.scripted_save(&path, &ctx).unwrap());
        assert!(app.job.is_some(), "busy slot must be left untouched");
        app.job = None;
        assert!(app.scripted_save(&path, &ctx).unwrap());
        let receiver = app.job.take().expect("scripted save started no job");
        let outcome = match receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("scripted save worker did not answer")
        {
            JobDone::Saved(..) => Ok(()),
            JobDone::Failed(error) => Err(error),
            _ => Err("unexpected scripted save result".to_owned()),
        };
        outcome.expect("scripted save did not write the project");
        assert!(Path::new(&path).exists());
    }

    #[test]
    fn input_lifecycle_and_new_preset_reach_the_running_program_producer() {
        fn acknowledged(app: &RustMixApp, check: impl Fn(&Engine) -> bool) {
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                if let Some(Ok(frame)) = app.program_worker.as_ref().unwrap().take()
                    && frame.generation == app.program_generation
                    && check(&frame.engine)
                {
                    return;
                }
                assert!(
                    Instant::now() < deadline,
                    "Worker did not acknowledge the input/project change"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        app.program_worker = Some(program::ProgramWorker::spawn(
            program::Snapshot {
                generation: 0,
                engine: app.engine.clone(),
                sources: app.input_fx.clone(),
            },
            app.start,
            None,
        ));
        app.dispatch(Action::Add, &ctx);
        app.select_add_category(0);
        app.add_name = "New stage".into();
        app.add_kind = InputKind::Color;
        assert!(app.request_add_input());
        app.add_open = false;
        acknowledged(&app, |engine| engine.project().input(InputId(8)).is_some());
        app.confirm_remove_input = Some(InputId(8));
        click(&mut app, &ctx, "dialog.remove.confirm", size);
        acknowledged(&app, |engine| engine.project().input(InputId(8)).is_none());
        app.execute(Command::RenameProject("Previous show".into()));
        app.execute(Command::AddPlaylist("Cues".into()));
        app.playlist_selected_id = 2;
        app.playlist_name = "Cues".into();
        app.dispatch(Action::New, &ctx);
        click(&mut app, &ctx, "dialog.new.confirm", size);
        acknowledged(&app, |engine| {
            engine.project().name == ProjectState::demo().name
                && engine.project().playlists.len() == 1
                && engine.project().program == InputId(1)
        });
        assert_eq!(app.playlist_selected_id, 1);
        assert_eq!(app.playlist_name, "Default");
    }

    #[test]
    fn playlist_widgets_commit_order_and_options_and_keep_program_until_start() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        app.dispatch(Action::PlayList, &ctx);
        click(&mut app, &ctx, "dialog.playlist.add", size);
        click(&mut app, &ctx, "dialog.playlist.available.3", size);
        click(&mut app, &ctx, "dialog.playlist.add", size);
        click(&mut app, &ctx, "dialog.playlist.up", size);
        click(&mut app, &ctx, "dialog.playlist.manual", size);
        assert_eq!(
            app.engine.project().playlists[0]
                .items
                .iter()
                .map(|i| i.input)
                .collect::<Vec<_>>(),
            [InputId(3), InputId(1)]
        );
        assert!(app.engine.project().playlists[0].manual);
        assert_eq!(app.engine.project().program, InputId(1));
        click(&mut app, &ctx, "dialog.playlist.start", size);
        assert_eq!(app.engine.playlist_run().unwrap().index, 0);
        assert_eq!(app.engine.transition().unwrap().to, InputId(3));
        click(&mut app, &ctx, "dialog.playlist.start", size);
        assert!(app.engine.playlist_run().is_none());
    }

    #[test]
    fn playlist_footer_stays_visible_at_small_and_large_viewports() {
        for size in [[734.0, 466.0], [1100.0, 700.0], [1600.0, 900.0]] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            app.dispatch(Action::PlayList, &ctx);
            for _ in 0..5 {
                frame(&mut app, &ctx, vec![], size);
            }
            for id in ["dialog.playlist.start", "dialog.playlist.close"] {
                assert!(
                    ctx.viewport_rect().contains_rect(app.automation[id]),
                    "{id}: {size:?}"
                );
            }
            let revision = app.engine.project().revision;
            click(&mut app, &ctx, "dialog.playlist.start", size);
            assert_eq!(app.engine.project().revision, revision);
            assert!(app.engine.playlist_run().is_none());
        }
    }

    #[test]
    fn remapped_shortcuts_respect_modifiers_repeats_and_dialog_focus() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        let mut preferences = app.engine.project().preferences.clone();
        preferences.key_bindings = vec![ShortcutBinding {
            key: "X".into(),
            ctrl: false,
            shift: true,
            alt: false,
            action: ShortcutAction::Cut,
        }];
        app.execute(Command::SetPreferences(preferences));
        frame(&mut app, &ctx, key(egui::Key::Space), size);
        frame(&mut app, &ctx, key(egui::Key::X), size);
        assert_eq!(app.engine.project().program, InputId(1));
        let events = |repeat| {
            vec![
                egui::Event::Key {
                    key: egui::Key::X,
                    physical_key: None,
                    pressed: true,
                    repeat,
                    modifiers: egui::Modifiers::SHIFT,
                },
                egui::Event::Key {
                    key: egui::Key::X,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: egui::Modifiers::SHIFT,
                },
            ]
        };
        // egui determines repeat from its held-key set, ignoring the raw repeat flag.
        // Keep the key held across frames before delivering its release.
        let held = vec![events(false).remove(0)];
        frame(&mut app, &ctx, held.clone(), size);
        assert_eq!(app.engine.project().program, InputId(2));
        frame(&mut app, &ctx, held, size);
        assert_eq!(app.engine.project().program, InputId(2));
        let released = vec![events(false).remove(1)];
        frame(&mut app, &ctx, released, size);
        assert_eq!(app.engine.project().program, InputId(2));
        app.dispatch(Action::PlayList, &ctx);
        frame(&mut app, &ctx, events(false), size);
        assert_eq!(app.engine.project().program, InputId(2));
    }

    #[test]
    fn shortcut_settings_cancel_discards_and_invalid_apply_stays_open() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        let before = app.engine.project().preferences.clone();
        app.dispatch(Action::SettingsAt(SettingsPage::Shortcuts), &ctx);
        app.settings_draft.as_mut().unwrap().key_bindings.clear();
        click(&mut app, &ctx, "dialog.settings.cancel", size);
        assert_eq!(app.engine.project().preferences, before);
        app.dispatch(Action::SettingsAt(SettingsPage::Shortcuts), &ctx);
        let draft = app.settings_draft.as_mut().unwrap();
        draft.key_bindings.push(draft.key_bindings[0].clone());
        click(&mut app, &ctx, "dialog.settings.apply", size);
        assert!(app.settings);
        assert_eq!(app.engine.project().preferences, before);
        app.settings_draft.as_mut().unwrap().key_bindings = vec![ShortcutBinding {
            key: "X".into(),
            ctrl: false,
            shift: false,
            alt: false,
            action: ShortcutAction::Cut,
        }];
        click(&mut app, &ctx, "dialog.settings.apply", size);
        assert!(!app.settings);
        frame(&mut app, &ctx, key(egui::Key::X), size);
        assert_eq!(app.engine.project().program, InputId(2));
    }
    fn frame(app: &mut RustMixApp, ctx: &egui::Context, events: Vec<egui::Event>, size: [f32; 2]) {
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(size[0], size[1]),
            )),
            events,
            ..Default::default()
        };
        let _ = ctx.run_ui(raw, |ui| app.frame_ui(ui));
    }
    #[test]
    fn native_reference_guards_reject_short_playlist_rows_and_overlapping_overlay_labels() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        app.dispatch(Action::PlayList, &ctx);
        for _ in 0..4 {
            frame(&mut app, &ctx, vec![], size);
        }
        app.validate_playlist_geometry(&ctx).unwrap();
        let row = app
            .automation
            .get_mut("dialog.playlist.available.1")
            .unwrap();
        row.max.x = row.min.x + 40.0;
        assert!(app.validate_playlist_geometry(&ctx).is_err());
        app.reset_editors();
        app.dispatch(Action::OverlaySettings(0), &ctx);
        for _ in 0..4 {
            frame(&mut app, &ctx, vec![], size);
        }
        app.validate_dialog_geometry(&ctx, "dialog.overlay")
            .unwrap();
        let field = app.automation["dialog.overlay.title"];
        app.automation
            .insert("dialog.overlay.label.title".into(), field);
        assert!(
            app.validate_dialog_geometry(&ctx, "dialog.overlay")
                .is_err()
        );
    }
    #[test]
    fn operator_toolbars_resize_without_clipping_or_switching_buses() {
        for size in [[1100.0, 700.0], [1600.0, 900.0], [1920.0, 1200.0]] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            for _ in 0..4 {
                frame(&mut app, &ctx, vec![], size);
            }
            app.validate_toolbar_geometry(size[0]).unwrap();
            let height = app.automation["toolbar.settings"].height();
            if size[0] >= 1600.0 {
                assert!(height >= 27.0, "Production display needs larger targets");
                assert!(app.automation["production.add"].height() >= 29.0);
                assert!(app.automation["switcher.cut"].height() >= 32.0);
                assert!(app.automation["switcher.fade"].height() >= 32.0);
            }
            let bank = app.automation["monitor.bank.bounds"];
            let source_name = app.automation["input.1.name"];
            let source_first_control = app.automation["input.1.overlay.1"];
            assert!(
                source_name.left() - source_first_control.left() <= 12.0,
                "Source names must start beside the left tile inset"
            );
            let first_overlay = app.automation["switcher.overlay.1"];
            let second_overlay = app.automation["switcher.overlay.5"];
            assert!(second_overlay.top() >= first_overlay.bottom());
            assert!(
                second_overlay.top() - first_overlay.top() <= 33.0,
                "Overlay rows must not inherit the tall transition-row minimum"
            );
            for id in [
                "switcher.cut",
                "switcher.fade",
                "switcher.ftb",
                "switcher.tbar",
            ] {
                assert!(
                    bank.contains_rect(app.automation[id]),
                    "Target clipped by bank: {id}"
                );
            }
            let program = app.engine.project().program;
            let preview = app.engine.project().preview;
            click(&mut app, &ctx, "toolbar.pause", size);
            assert!(app.engine.inputs_paused());
            assert_eq!(app.engine.project().program, program);
            assert_eq!(app.engine.project().preview, preview);
            click(&mut app, &ctx, "toolbar.pause", size);
            assert!(!app.engine.inputs_paused());
            click(&mut app, &ctx, "toolbar.basic", size);
            frame(&mut app, &ctx, vec![], size);
            app.validate_toolbar_geometry(size[0]).unwrap();
        }
    }
    fn click(app: &mut RustMixApp, ctx: &egui::Context, id: &str, size: [f32; 2]) {
        // A newly shown Window has an initial sizing pass. Use fresh stable
        // bounds before delivering one click, rather than targeting that pass.
        let mut previous: Option<egui::Rect> = None;
        let mut stable = None;
        for _ in 0..6 {
            frame(app, ctx, vec![], size);
            let bounds = *app.automation.get(id).expect("Visible operator control");
            if previous.is_some_and(|old| {
                old.center().distance(bounds.center()) < 0.5
                    && (old.size() - bounds.size()).length() < 0.5
            }) {
                stable = Some(bounds);
                break;
            }
            previous = Some(bounds);
        }
        let bounds = stable.expect("Operator control must settle within six frames");
        assert!(
            ctx.viewport_rect().contains_rect(bounds),
            "Click target is outside viewport: {id} {bounds:?}"
        );
        let pos = bounds.center();
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                size,
            );
        }
    }

    #[test]
    fn basic_mode_simplifies_controls_without_switching_program_or_multiview() {
        for size in [[1100.0, 700.0], [1600.0, 900.0], [1920.0, 1080.0]] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            let before = app.engine.project().clone();
            click(&mut app, &ctx, "toolbar.basic", size);
            frame(&mut app, &ctx, vec![], size);
            app.validate_basic_controls().unwrap();
            app.validate_monitor_geometry(size[0]).unwrap();
            assert_eq!(*app.engine.project(), before);
            assert!(!app.multiview);

            click(&mut app, &ctx, "switcher.quickPlay", size);
            assert_eq!(app.engine.project().program, InputId(2));
            assert_eq!(app.engine.project().preview, InputId(1));
            let taken = app.engine.project().clone();
            click(&mut app, &ctx, "toolbar.basic", size);
            frame(&mut app, &ctx, vec![], size);
            assert!(!app.basic_mode && !app.multiview);
            for id in [
                "switcher.cut",
                "switcher.fade",
                "output.record",
                "output.playlist",
                "input.1.loop",
                "input.1.audio",
            ] {
                assert!(
                    app.automation.contains_key(id),
                    "Advanced control was not restored: {id}"
                );
            }
            assert_eq!(*app.engine.project(), taken);

            // Entering Basic from a full multiview still shows both switcher
            // monitors; returning to Advanced restores the chosen multiview.
            app.multiview = true;
            app.multiview_split = false;
            click(&mut app, &ctx, "toolbar.basic", size);
            frame(&mut app, &ctx, vec![], size);
            app.validate_basic_controls().unwrap();
            app.validate_monitor_geometry(size[0]).unwrap();
            click(&mut app, &ctx, "toolbar.basic", size);
            frame(&mut app, &ctx, vec![], size);
            assert!(app.multiview && !app.multiview_split && !app.basic_mode);
            assert!(app.automation.contains_key("multiview.preset.grid"));
            assert_eq!(*app.engine.project(), taken);
        }
    }

    #[test]
    fn dialog_footers_remain_visible_on_small_and_large_workspaces() {
        // 734×466 also represents a 1100×700 viewport at 150% UI scale.
        for size in [
            [734.0, 466.0],
            [1100.0, 700.0],
            [1600.0, 900.0],
            [1920.0, 1080.0],
        ] {
            for (action, prefix, footer) in [
                (Action::Add, "dialog.add", "dialog.add.confirm"),
                (
                    Action::Properties(InputId(3)),
                    "dialog.properties",
                    "dialog.properties.apply",
                ),
                (Action::Settings, "dialog.settings", "dialog.settings.apply"),
                (
                    Action::OverlaySettings(0),
                    "dialog.overlay",
                    "dialog.overlay.apply",
                ),
                (
                    Action::ExternalSettings(1),
                    "dialog.external",
                    "dialog.external.apply",
                ),
            ] {
                let mut app = RustMixApp::headless();
                let ctx = egui::Context::default();
                app.theme(&ctx);
                app.dispatch(action, &ctx);
                for _ in 0..4 {
                    frame(&mut app, &ctx, vec![], size);
                }
                let bounds = ctx.viewport_rect();
                let button = app.automation.get(footer).unwrap();
                let body = app.automation.get(&format!("{prefix}.body")).unwrap();
                assert!(
                    bounds.contains_rect(*button),
                    "Footer outside {size:?}: {button:?}"
                );
                assert!(body.bottom() <= button.top(), "Body overlaps fixed footer");
                assert!(button.width() >= 85.0 && button.height() >= 27.0);
                app.validate_dialog_geometry(&ctx, prefix).unwrap();
                let pages: &[&str] = match prefix {
                    "dialog.properties" => &["Audio", "Status", "General"],
                    "dialog.settings" => &["Appearance", "Workspace", "Shortcuts", "Display"],
                    _ => &[],
                };
                for page in pages {
                    click(&mut app, &ctx, &format!("{prefix}.page.{page}"), size);
                    frame(&mut app, &ctx, vec![], size);
                    app.validate_dialog_geometry(&ctx, prefix).unwrap();
                }
                assert_eq!(app.engine.project().program, InputId(1));
            }
        }
    }

    #[test]
    fn monitor_edit_filter_reset_and_dialog_cancel_preserve_program() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![], size);
        }
        click(&mut app, &ctx, "monitor.preview.properties", size);
        assert_eq!(app.properties.as_ref().unwrap().id, InputId(2));
        app.properties.as_mut().unwrap().name = "Unsaved draft".into();
        frame(&mut app, &ctx, vec![], size);
        click(&mut app, &ctx, "dialog.properties.cancel", size);
        assert!(app.properties.is_none());
        assert_eq!(
            app.engine.project().input(InputId(2)).unwrap().name,
            "Camera B"
        );

        app.category = Some(0);
        app.input_filter = "no matching source".into();
        frame(&mut app, &ctx, vec![], size);
        click(&mut app, &ctx, "inputs.filter.clear", size);
        assert!(app.category.is_none() && app.input_filter.is_empty());

        let mut offline = app.engine.project().input(InputId(3)).unwrap().clone();
        offline.online = false;
        app.execute(Command::UpdateInput(offline));
        frame(&mut app, &ctx, vec![], size);
        assert!(app.automation.contains_key("input.3.offline"));

        app.dispatch(Action::RequestRemove(InputId(3)), &ctx);
        frame(&mut app, &ctx, key(egui::Key::Space), size);
        click(&mut app, &ctx, "dialog.remove.cancel", size);
        assert!(app.engine.project().input(InputId(3)).is_some());
        assert_eq!(app.engine.project().program, InputId(1));
        assert_eq!(app.engine.project().preview, InputId(2));
    }

    #[test]
    fn generator_sample_survives_removal_and_settings_cancel_discards_draft() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1100.0, 700.0];
        app.execute(Command::RemoveInput(InputId(3)));
        app.dispatch(Action::Add, &ctx);
        click(&mut app, &ctx, "dialog.add.kind.Grid", size);
        click(&mut app, &ctx, "dialog.add.generator.2", size);
        for _ in 0..4 {
            frame(&mut app, &ctx, vec![], size);
        }
        assert!(
            !app.engine
                .project()
                .inputs
                .iter()
                .any(|input| input.kind == InputKind::Bars)
        );
        assert!(
            app.add_preview
                .as_ref()
                .is_some_and(|(kind, _)| *kind == InputKind::Bars)
        );
        assert!(app.automation.contains_key("dialog.add.preview"));
        click(&mut app, &ctx, "dialog.add.cancel", size);

        app.dispatch(Action::Settings, &ctx);
        app.settings_name = "Discard this project name".into();
        app.settings_draft.as_mut().unwrap().high_contrast = true;
        app.settings_layout = Some(true);
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![], size);
        }
        click(&mut app, &ctx, "dialog.settings.cancel", size);
        assert!(!app.settings);
        assert!(!app.engine.project().preferences.high_contrast);
        assert_eq!(app.engine.project().workspace.name, "Production");
        assert_eq!(app.engine.project().name, "Studio / Demo production");
        assert_eq!(app.engine.project().program, InputId(1));
    }

    #[test]
    fn actual_widgets_dispatch_commands_and_survive_resize() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![], [1600.0, 900.0]);
        }
        frame(&mut app, &ctx, key(egui::Key::Num3), [1600.0, 900.0]);
        assert_eq!(app.engine.project().preview, InputId(3));
        assert_eq!(app.engine.project().program, InputId(1));
        let rect = app.automation.get("switcher.cut").unwrap();
        let pos = rect.center();
        frame(
            &mut app,
            &ctx,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            [1600.0, 900.0],
        );
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
            [1600.0, 900.0],
        );
        assert_eq!(app.engine.project().program, InputId(3));
        frame(&mut app, &ctx, key(egui::Key::F1), [1600.0, 900.0]);
        assert!(app.engine.project().overlays.standard[0].enabled);
        frame(&mut app, &ctx, key(egui::Key::F5), [1600.0, 900.0]);
        assert!(app.engine.project().overlays.standard[4].enabled);
        assert!(!app.engine.project().overlays.standard[3].enabled);
        // The source-tile strip row 2 (channels 5–8) is live, not a stub.
        click(&mut app, &ctx, "input.3.overlay.5", [1600.0, 900.0]);
        let tile_overlay = app.engine.project().overlays.standard[4].clone();
        assert!(tile_overlay.enabled);
        assert_eq!(tile_overlay.title, "Camera C");
        let original_monitor_height = app.automation.get("monitor.preview").unwrap().height();
        for size in [[1920.0, 1080.0], [1600.0, 900.0], [1100.0, 700.0]] {
            frame(&mut app, &ctx, vec![], size);
            assert!(app.automation.get("switcher.cut").unwrap().is_finite());
            app.validate_monitor_geometry(size[0]).unwrap();
            if size[0] == 1920.0 {
                assert!(
                    app.automation.get("monitor.preview").unwrap().height()
                        > original_monitor_height * 1.12,
                    "Monitor bank must grow with the enlarged viewport"
                );
            }
        }
        let layout = serde_json::to_string(&app.dock).unwrap();
        let restored: DockState<Panel> = serde_json::from_str(&layout).unwrap();
        assert_eq!(restored.iter_all_tabs().count(), 3);
        assert!(workspace::decode(&layout).is_ok());
        let mut hostile: serde_json::Value = serde_json::from_str(&layout).unwrap();
        hostile["focused_surface"] = serde_json::json!(999);
        assert!(workspace::decode(&hostile.to_string()).is_err());
        let mut hostile: serde_json::Value = serde_json::from_str(&layout).unwrap();
        hostile["surfaces"][0]["Main"]["nodes"][0] = serde_json::json!("Empty");
        assert!(workspace::decode(&hostile.to_string()).is_err());
    }

    #[test]
    fn source_dialog_and_settings_apply_dispatch_real_commands() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![], size);
        }
        // Confirmation overlays must block switching shortcuts behind the dialog.
        app.confirm_remove_input = Some(InputId(3));
        frame(&mut app, &ctx, key(egui::Key::Space), size);
        assert_eq!(app.engine.project().program, InputId(1));
        app.confirm_remove_input = None;
        app.dispatch(Action::Add, &ctx);
        click(&mut app, &ctx, "dialog.add.kind.Grid", size);
        app.add_name = " ".into();
        frame(&mut app, &ctx, vec![], size);
        let add = app.automation.get("dialog.add.confirm").unwrap().center();
        frame(
            &mut app,
            &ctx,
            vec![
                egui::Event::PointerMoved(add),
                egui::Event::PointerButton {
                    pos: add,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            size,
        );
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos: add,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
            size,
        );
        assert_eq!(app.engine.project().inputs.len(), 7);
        app.add_name = "Test grid".into();
        app.add_kind = InputKind::Grid;
        frame(&mut app, &ctx, vec![], size);
        click(&mut app, &ctx, "dialog.add.confirm", size);
        assert_eq!(app.engine.project().inputs.len(), 8);
        assert!(
            app.engine
                .project()
                .inputs
                .iter()
                .any(|i| i.name == "Test grid")
        );
        assert_eq!(app.engine.project().program, InputId(1));

        app.dispatch(Action::Settings, &ctx);
        app.settings_draft.as_mut().unwrap().ui_scale = 1.2;
        app.settings_layout = Some(true);
        frame(&mut app, &ctx, vec![], size);
        assert_eq!(app.engine.project().preferences.ui_scale, 1.0);
        assert_eq!(app.engine.project().workspace.name, "Production");
        click(&mut app, &ctx, "dialog.settings.apply", size);
        assert_eq!(app.engine.project().preferences.ui_scale, 1.2);
        assert_eq!(app.engine.project().workspace.name, "Audio focus");
        assert_eq!(app.engine.project().program, InputId(1));
    }

    #[test]
    fn held_tbar_endpoint_commits_only_once() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![], [1600.0, 900.0]);
        }
        let rect = app.automation.get("switcher.tbar").unwrap();
        assert!(
            rect.width() > rect.height() * 2.0,
            "T-Bar must be horizontal"
        );
        let pos = egui::pos2(rect.right() - 1.0, rect.center().y);
        frame(
            &mut app,
            &ctx,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            [1600.0, 900.0],
        );
        assert_eq!(app.engine.project().program, InputId(2));
        for _ in 0..4 {
            frame(
                &mut app,
                &ctx,
                vec![egui::Event::PointerMoved(pos)],
                [1600.0, 900.0],
            );
        }
        assert_eq!(app.engine.project().program, InputId(2));
        assert_eq!(app.engine.project().preview, InputId(1));
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
            [1600.0, 900.0],
        );
        assert!(app.engine.transition().is_none());
        assert_eq!(app.engine.project().program, InputId(2));
        assert_eq!(app.engine.project().preview, InputId(1));
    }

    #[test]
    fn monitor_bank_does_not_creep_and_leaves_inputs_visible() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.execute(Command::SetInputsPaused(true));
        for size in [
            [1600.0, 900.0],
            [1920.0, 1080.0],
            [1100.0, 700.0],
            [1600.0, 900.0],
        ] {
            for _ in 0..4 {
                frame(&mut app, &ctx, vec![], size);
            }
            let original = *app.automation.get("monitor.bank.bounds").unwrap();
            for _ in 0..90 {
                frame(&mut app, &ctx, vec![], size);
            }
            let settled = *app.automation.get("monitor.bank.bounds").unwrap();
            assert!(
                (settled.bottom() - original.bottom()).abs() < 1.0,
                "Monitor bank grew without user input: {original:?} -> {settled:?}"
            );
            assert!(
                size[1] - settled.bottom() >= 230.0,
                "Input bank was pushed offscreen: {settled:?}"
            );
            for name in ["preview", "program"] {
                let monitor = app.automation.get(&format!("monitor.{name}")).unwrap();
                let transport = app
                    .automation
                    .get(&format!("monitor.{name}.transport"))
                    .unwrap();
                assert!(monitor.bottom() <= transport.top());
                assert!(settled.contains_rect(*transport));
            }
        }
    }

    #[test]
    fn monitor_transport_controls_pause_restart_and_share_source_state() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![], size);
        }
        let id = app.engine.project().preview;
        click(&mut app, &ctx, "monitor.preview.play", size);
        assert!(app.engine.playback(id).unwrap().paused());
        let paused = app.engine.playback(id).unwrap().position_ms();
        app.engine.tick(app.now + 5000);
        assert_eq!(app.engine.playback(id).unwrap().position_ms(), paused);
        click(&mut app, &ctx, "monitor.preview.restart", size);
        assert_eq!(app.engine.playback(id).unwrap().position_ms(), 0);
        assert!(app.engine.playback(id).unwrap().paused());
        click(&mut app, &ctx, "monitor.preview.play", size);
        assert!(!app.engine.playback(id).unwrap().paused());
    }

    #[test]
    fn paused_program_pixels_hold_and_unsupported_outputs_cannot_go_live() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.now = 1250;
        app.engine.tick(app.now);
        let id = app.engine.project().program;
        app.execute(Command::SetInputPaused(id, true));
        app.refresh_textures(&ctx);
        let frozen = app.last_program_rgba.clone();
        app.now = 5000;
        app.engine.tick(app.now);
        app.refresh_textures(&ctx);
        assert_eq!(app.last_program_rgba, frozen);
        for action in [
            Action::ToggleRecordSim,
            Action::ToggleStreamSim,
            Action::ToggleExternalSim,
        ] {
            app.dispatch(action, &ctx);
            assert!(app.warning);
            assert!(!app.rec_active && !app.stream_active && !app.external_active);
        }
    }

    #[test]
    fn global_pause_reuses_surfaces_and_removed_ids_cannot_reuse_old_images() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.now = 1250;
        app.execute(Command::SetInputsPaused(true));
        app.refresh_textures(&ctx);
        let stamp = app.last_demo_frame;
        app.now = 5000;
        app.engine.tick(app.now);
        app.refresh_textures(&ctx);
        assert_eq!(stamp, app.last_demo_frame);
        let id = app.engine.project().inputs.last().unwrap().id;
        app.input_fx.entry(id).or_default().custom_rgba =
            Some((1, 1, vec![[255, 0, 0, 255]].into()));
        app.input_fx.entry(InputId(1)).or_default().pip_source = Some(id);
        app.execute(Command::RemoveInput(id));
        assert!(!app.input_fx.contains_key(&id));
        assert!(app.input_fx.get(&InputId(1)).unwrap().pip_source.is_none());
    }

    #[test]
    fn unsupported_media_does_not_create_input_or_replace_image_state() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let id = InputId(1);
        let original = vec![[120, 20, 10, 255]];
        app.input_fx.entry(id).or_default().custom_rgba = Some((1, 1, original.clone().into()));
        app.dispatch(Action::Properties(id), &ctx);
        app.request_property_png(id, "missing-video.mp4".into());
        finish_media(&mut app);
        app.close_properties();
        assert_eq!(
            app.input_fx
                .get(&id)
                .unwrap()
                .custom_rgba
                .as_ref()
                .unwrap()
                .2
                .as_ref(),
            &original
        );
        let size = [1600.0, 900.0];
        app.dispatch(Action::Add, &ctx);
        click(&mut app, &ctx, "dialog.add.image", size);
        app.add_png_path = "missing-video.mp4".into();
        app.check_add_png();
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![], size);
        }
        let count = app.engine.project().inputs.len();
        click(&mut app, &ctx, "dialog.add.confirm", size);
        assert_eq!(app.engine.project().inputs.len(), count);
        assert!(app.add_open && app.add_error.is_some());
    }

    #[test]
    fn png_pixels_reach_program_and_offline_sources_stay_black() {
        let folder = tempfile::tempdir().unwrap();
        let file = folder.path().join("source.png");
        image::save_buffer(&file, &[120, 20, 10, 255], 1, 1, image::ColorType::Rgba8).unwrap();
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        let id = app.engine.project().program;
        app.dispatch(Action::Properties(id), &ctx);
        app.request_property_png(id, file);
        finish_media(&mut app);
        app.properties_fx.as_mut().unwrap().image_fit = ImageFit::Stretch;
        assert!(app.apply_properties(app.properties.clone().unwrap()));
        app.refresh_textures(&ctx);
        assert!(
            app.last_program_rgba
                .chunks_exact(4)
                .all(|pixel| pixel == [120, 20, 10, 255])
        );
        let mut input = app.engine.project().input(id).unwrap().clone();
        input.online = false;
        app.execute(Command::UpdateInput(input));
        app.refresh_textures(&ctx);
        assert!(
            app.last_program_rgba
                .chunks_exact(4)
                .all(|pixel| pixel == [0, 0, 0, 255])
        );
    }

    #[test]
    fn unavailable_shortcut_and_stream_controls_cannot_dispatch_fake_features() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        app.dispatch(Action::ShortcutsWindow, &ctx);
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![], size);
        }
        let revision = app.engine.project().revision;
        for index in [10, 11] {
            click(
                &mut app,
                &ctx,
                &format!("dialog.shortcuts.action.{index}"),
                size,
            );
        }
        assert!(app.engine.transition().is_none());
        assert!(!app.rec_active && !app.stream_active);
        assert_eq!(app.engine.project().revision, revision);
        click(&mut app, &ctx, "dialog.shortcuts.close", size);
        app.dispatch(Action::StreamingSetup, &ctx);
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![], size);
        }
        click(&mut app, &ctx, "dialog.streaming.start", size);
        assert!(!app.stream_active);
        assert_eq!(app.engine.project().revision, revision);
    }

    #[test]
    fn dragging_a_panel_tab_changes_docking_and_roundtrips() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        for _ in 0..4 {
            frame(&mut app, &ctx, vec![], [1600.0, 900.0]);
        }
        let from = app.automation.get("workspace.tab.Audio").unwrap().center();
        let destination = app.dock.find_tab(&Panel::Inputs).unwrap().node_path();
        let to = app.dock.leaf(destination).unwrap().viewport.center();
        frame(
            &mut app,
            &ctx,
            vec![
                egui::Event::PointerMoved(from),
                egui::Event::PointerButton {
                    pos: from,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            [1600.0, 900.0],
        );
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerMoved(from + egui::vec2(15.0, 0.0))],
            [1600.0, 900.0],
        );
        for _ in 0..25 {
            frame(
                &mut app,
                &ctx,
                vec![egui::Event::PointerMoved(to)],
                [1600.0, 900.0],
            );
        }
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos: to,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
            [1600.0, 900.0],
        );
        frame(&mut app, &ctx, vec![], [1600.0, 900.0]);
        let input_path = app.dock.find_tab(&Panel::Inputs).unwrap();
        let audio_path = app.dock.find_tab(&Panel::Audio).unwrap();
        assert_eq!(
            input_path.node_path(),
            audio_path.node_path(),
            "Drag must dock Audio alongside Inputs"
        );
        let encoded = serde_json::to_string(&app.dock).unwrap();
        assert_eq!(
            workspace::decode(&encoded).unwrap().iter_all_tabs().count(),
            3
        );
    }

    #[test]
    fn external_output_widgets_preserve_slot_drafts_and_commit_atomically() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        app.execute(Command::SetInputsPaused(true));
        frame(&mut app, &ctx, vec![], size);
        let before = app.engine.project().clone();
        let pixels = app.last_program_rgba.clone();

        // The production gear is a real entry point, not a status-only menu.
        click(&mut app, &ctx, "output.external.settings", size);
        assert_eq!(app.external_settings.as_ref().unwrap().selected, 1);
        click(&mut app, &ctx, "dialog.external.source.program", size);
        click(&mut app, &ctx, "dialog.external.rate", size);
        click(&mut app, &ctx, "dialog.external.rate.59.94", size);
        click(&mut app, &ctx, "dialog.external.device", size);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Text("Test device".into())],
            size,
        );
        click(&mut app, &ctx, "dialog.external.port_toggle", size);
        click(&mut app, &ctx, "dialog.external.bus", size);
        click(&mut app, &ctx, "dialog.external.bus.B", size);
        click(&mut app, &ctx, "dialog.external.alpha", size);
        let first = app.external_settings.as_ref().unwrap().outputs[0].clone();
        assert_eq!(first.source, OutputSource::Program);
        assert_eq!(first.config.frame_rate, Rate::new(60000, 1001).unwrap());
        assert_eq!(first.config.device, "Test device");
        assert_eq!(
            first.config.port,
            Some(1),
            "Hardware connector defaults to 1"
        );
        assert_eq!(first.config.audio_bus, Some(2));
        assert!(first.config.alpha_channel);
        assert_eq!(*app.engine.project(), before);

        click(&mut app, &ctx, "dialog.external.slot.2", size);
        click(&mut app, &ctx, "dialog.external.source.preview", size);
        click(&mut app, &ctx, "dialog.external.slot.1", size);
        assert_eq!(app.external_settings.as_ref().unwrap().outputs[0], first);
        click(&mut app, &ctx, "dialog.external.apply", size);
        assert!(
            app.external_settings.is_some(),
            "Apply preserves other slot drafts"
        );
        assert_eq!(app.engine.project().revision, before.revision + 1);
        assert_eq!(app.engine.project().outputs.slot(1), Some(&first));
        assert_eq!(app.engine.project().outputs.slot(2), before.outputs.slot(2));
        click(&mut app, &ctx, "dialog.external.slot.2", size);
        assert_eq!(
            app.external_settings.as_ref().unwrap().outputs[1].source,
            OutputSource::Preview
        );
        click(&mut app, &ctx, "dialog.external.apply", size);
        assert_eq!(app.engine.project().revision, before.revision + 2);
        assert_eq!(
            app.engine.project().outputs.slot(2).unwrap().source,
            OutputSource::Preview
        );

        // Cancel discards subsequent edits without undoing applied slots.
        let applied = app.engine.project().clone();
        click(&mut app, &ctx, "dialog.external.source.none", size);
        click(&mut app, &ctx, "dialog.external.cancel", size);
        assert!(app.external_settings.is_none());
        assert_eq!(*app.engine.project(), applied);
        click(&mut app, &ctx, "output.external", size);
        assert_eq!(*app.engine.project(), applied);
        assert!(!app.external_active && !app.rec_active && !app.stream_active);
        assert_eq!(app.engine.project().program, before.program);
        assert_eq!(app.engine.project().preview, before.preview);
        assert_eq!(app.last_program_rgba, pixels);
    }

    #[test]
    fn external_output_cancel_and_invalid_or_disappeared_source_leave_state_untouched() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        let initial = app.engine.project().clone();
        click(&mut app, &ctx, "output.external.open", size);
        click(&mut app, &ctx, "dialog.external.source.program", size);
        click(&mut app, &ctx, "dialog.external.cancel", size);
        assert_eq!(*app.engine.project(), initial);

        click(&mut app, &ctx, "output.external.open", size);
        click(&mut app, &ctx, "dialog.external.source.program", size);
        // Malformed configuration must reject the complete source/config update.
        app.external_settings.as_mut().unwrap().outputs[0]
            .config
            .device = "bad\u{7}device".into();
        click(&mut app, &ctx, "dialog.external.apply", size);
        frame(&mut app, &ctx, vec![], size);
        assert_eq!(*app.engine.project(), initial);
        assert!(app.external_settings.as_ref().unwrap().error.is_some());
        assert!(app.automation.contains_key("dialog.external.error"));
        app.external_settings.as_mut().unwrap().outputs[0]
            .config
            .device
            .clear();

        click(&mut app, &ctx, "dialog.external.input", size);
        click(&mut app, &ctx, "dialog.external.input.7", size);
        app.execute(Command::RemoveInput(InputId(7)));
        let removed = app.engine.project().clone();
        click(&mut app, &ctx, "dialog.external.apply", size);
        assert_eq!(*app.engine.project(), removed);
        assert!(app.external_settings.as_ref().unwrap().error.is_some());
        assert_eq!(
            app.external_settings.as_ref().unwrap().outputs[0].source,
            OutputSource::Input(InputId(7))
        );
        // A valid replacement stays actionable in the same open editor.
        click(&mut app, &ctx, "dialog.external.source.preview", size);
        click(&mut app, &ctx, "dialog.external.apply", size);
        assert!(app.external_settings.as_ref().unwrap().error.is_none());
        assert_eq!(
            app.engine.project().outputs.slot(1).unwrap().source,
            OutputSource::Preview
        );
        assert_eq!(app.engine.project().revision, removed.revision + 1);
        assert_eq!(app.engine.project().program, initial.program);
        assert_eq!(app.engine.project().preview, initial.preview);
    }

    #[test]
    fn overlay_settings_stage_slot_edits_and_apply_only_on_apply() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![], size);
        }
        click(&mut app, &ctx, "workspace.tab.Overlays", size);
        let before = app.engine.project().revision;

        click(&mut app, &ctx, "overlay.1.settings", size);
        let (slot, draft) = app.overlay_settings.clone().expect("Overlay 1 draft");
        assert_eq!(slot, 0);
        assert!(!draft.enabled);
        assert_eq!(draft.position, Position::LowerLeft);

        // Position edits stay in the draft until Apply is selected.
        click(&mut app, &ctx, "dialog.overlay.position.TopLeft", size);
        assert_eq!(
            app.overlay_settings
                .as_ref()
                .map(|(_, draft)| draft.position),
            Some(Position::TopLeft)
        );
        assert_eq!(
            app.engine.project().overlays.standard[0].position,
            Position::LowerLeft
        );
        assert_eq!(app.engine.project().revision, before);

        // A dirty draft requires explicit discard before switching channels.
        click(&mut app, &ctx, "dialog.overlay.slot.2", size);
        assert_eq!(
            app.overlay_settings.as_ref().map(|(slot, _)| *slot),
            Some(0)
        );
        assert_eq!(app.engine.project().revision, before);
        click(&mut app, &ctx, "dialog.overlay.switch.discard", size);
        assert_eq!(
            app.overlay_settings.as_ref().map(|(slot, _)| *slot),
            Some(1)
        );
        assert_eq!(
            app.overlay_settings
                .as_ref()
                .map(|(_, draft)| draft.position),
            Some(Position::LowerLeft)
        );

        // Cancel discards every staged edit.
        click(&mut app, &ctx, "dialog.overlay.cancel", size);
        assert!(app.overlay_settings.is_none());
        assert_eq!(app.engine.project().revision, before);
        assert_eq!(
            app.engine.project().overlays.standard[0].position,
            Position::LowerLeft
        );

        // Apply commits only the selected slot.
        click(&mut app, &ctx, "overlay.2.settings", size);
        click(&mut app, &ctx, "dialog.overlay.enabled", size);
        click(&mut app, &ctx, "dialog.overlay.position.LowerRight", size);
        click(&mut app, &ctx, "dialog.overlay.apply", size);
        let project = app.engine.project();
        assert!(project.overlays.standard[1].enabled);
        assert_eq!(project.overlays.standard[1].position, Position::LowerRight);
        assert_eq!(project.overlays.standard[0].position, Position::LowerLeft);
        assert!(project.revision > before);
        assert!(app.overlay_settings.is_none());

        // Bottom production-bar gear opens Overlay Settings directly, and Audio strip gear opens Input Audio routing.
        click(&mut app, &ctx, "output.overlay.settings", size);
        assert_eq!(app.overlay_settings.as_ref().map(|(s, _)| *s), Some(0));
        click(&mut app, &ctx, "dialog.overlay.cancel", size);

        click(&mut app, &ctx, "audio.input.1.settings", size);
        assert_eq!(app.properties.as_ref().map(|i| i.id), Some(InputId(1)));
        assert_eq!(app.input_page, InputPage::Audio);
        click(&mut app, &ctx, "dialog.properties.page.General", size);
        assert_eq!(app.input_page, InputPage::General);
        click(&mut app, &ctx, "dialog.properties.audioSettings", size);
        assert_eq!(app.input_page, InputPage::Audio);
        click(&mut app, &ctx, "dialog.properties.cancel", size);

        // Multiview layout presets switch between Split (Monitors + Grid) and Full Source Grid.
        click(&mut app, &ctx, "output.multiview", size);
        assert!(app.multiview && app.multiview_split);
        click(&mut app, &ctx, "multiview.preset.grid", size);
        assert!(!app.multiview_split);
        click(&mut app, &ctx, "multiview.return", size);
        assert!(!app.multiview);

        // Reference production-bar dialogs (Recording Setup, Streaming Settings, MultiCorder, PlayList, Shortcuts).
        click(&mut app, &ctx, "output.record.settings", size);
        assert!(app.recording_setup);
        click(&mut app, &ctx, "dialog.recording.ok", size);
        assert!(!app.recording_setup);

        click(&mut app, &ctx, "output.stream.settings", size);
        assert!(app.streaming_setup);
        click(&mut app, &ctx, "dialog.streaming.close", size);
        assert!(!app.streaming_setup);

        click(&mut app, &ctx, "output.multicorder.settings", size);
        assert!(app.multicorder_open);
        click(&mut app, &ctx, "dialog.multicorder.close", size);
        assert!(!app.multicorder_open);

        click(&mut app, &ctx, "output.playlist.settings", size);
        assert!(app.playlist_open);
        click(&mut app, &ctx, "dialog.playlist.close", size);
        assert!(!app.playlist_open);

        click(&mut app, &ctx, "output.shortcuts", size);
        assert!(app.shortcuts_open);
        click(&mut app, &ctx, "dialog.shortcuts.close", size);
        assert!(!app.shortcuts_open);
    }
    fn finish_media(app: &mut RustMixApp) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            app.poll_media();
            if !app.media_loader.busy() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "bounded PNG worker did not finish"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    fn fixture(path: &Path, color: [u8; 4]) {
        image::save_buffer(path, &color.repeat(16 * 9), 16, 9, image::ColorType::Rgba8).unwrap();
    }
    fn attach_png(app: &mut RustMixApp, ctx: &egui::Context, id: InputId, path: PathBuf) {
        app.dispatch(Action::Properties(id), ctx);
        app.request_property_png(id, path);
        finish_media(app);
        assert!(app.apply_properties(app.properties.clone().unwrap()));
    }

    #[test]
    fn properties_cancel_discards_png_and_effects_without_touching_program() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("red.png");
        fixture(&path, [120, 20, 10, 255]);
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let id = app.engine.project().program;
        app.execute(Command::SetInputsPaused(true));
        app.refresh_textures(&ctx);
        let before = app.engine.project().clone();
        let pixels = app.last_program_rgba.clone();
        app.dispatch(Action::Properties(id), &ctx);
        app.request_property_png(id, path);
        finish_media(&mut app);
        app.prop_mirror = true;
        app.prop_gain = 0.5;
        app.sync_fx_from_prop(id);
        app.refresh_textures(&ctx);
        assert_eq!(app.last_program_rgba, pixels);
        click(&mut app, &ctx, "dialog.properties.cancel", [1600.0, 900.0]);
        assert_eq!(app.engine.project(), &before);
        assert!(app.properties.is_none());
        assert!(app.input_fx.get(&id).unwrap().custom_rgba.is_none());
    }

    #[test]
    fn png_add_order_duplicate_and_reopen_preserve_rendered_pixels() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("logo.png");
        fixture(&path, [100, 40, 20, 255]);
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.dispatch(Action::Add, &ctx);
        click(&mut app, &ctx, "dialog.add.image", [1600.0, 900.0]);
        app.add_png_path = path.display().to_string();
        app.add_number = 2;
        app.check_add_png();
        finish_media(&mut app);
        click(&mut app, &ctx, "dialog.add.confirm", [1600.0, 900.0]);
        finish_media(&mut app);
        let id = app.engine.project().inputs[1].id;
        assert_eq!(id, InputId(8));
        assert_eq!(app.engine.project().input_number(id), 2);
        frame(&mut app, &ctx, key(egui::Key::Num2), [1600.0, 900.0]);
        assert_eq!(app.engine.project().preview, id);
        app.execute(Command::SelectPreview(InputId(2)));
        assert_eq!(app.engine.project().program, InputId(1));
        assert_eq!(app.engine.project().preview, InputId(2));
        app.dispatch(Action::Properties(id), &ctx);
        app.prop_gain = 0.5;
        app.prop_mirror = true;
        click(&mut app, &ctx, "dialog.properties.apply", [1600.0, 900.0]);
        app.execute(Command::DuplicateInput(id));
        let copy = app.engine.project().inputs[2].id;
        assert_eq!(app.engine.project().input(id).unwrap().config.gain, 0.5);
        assert!(app.engine.project().input(id).unwrap().config.mirror);
        assert_eq!(
            app.engine.project().input(id).unwrap().config,
            app.engine.project().input(copy).unwrap().config
        );
        app.execute(Command::TakeInput(copy));
        app.refresh_textures(&ctx);
        assert!(
            app.last_program_rgba
                .chunks_exact(4)
                .all(|p| p == [50, 20, 10, 255])
        );
        let pixels = app.last_program_rgba.clone();
        let project_path = dir.path().join("show.rustmix.json");
        persistence::save(&project_path, app.engine.project()).unwrap();
        let restored = persistence::load(&project_path).unwrap();
        let mut reopened = RustMixApp::headless();
        reopened.engine = Engine::from_project(restored).unwrap();
        reopened.input_fx.clear();
        finish_media(&mut reopened);
        reopened.refresh_textures(&ctx);
        assert_eq!(reopened.last_program_rgba, pixels);
    }

    #[test]
    fn missing_png_is_black_and_can_be_relinked_without_changing_input_identity() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.png");
        let replacement = dir.path().join("replacement.png");
        fixture(&replacement, [20, 100, 50, 255]);
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        let id = app.engine.project().program;
        let mut input = app.engine.project().input(id).unwrap().clone();
        input.config.media_path = Some(path.display().to_string());
        app.execute(Command::UpdateInput(input));
        finish_media(&mut app);
        app.refresh_textures(&ctx);
        assert!(app.input_fx.get(&id).unwrap().image_error.is_some());
        assert!(
            app.last_program_rgba
                .chunks_exact(4)
                .all(|p| p == [0, 0, 0, 255])
        );
        attach_png(&mut app, &ctx, id, replacement);
        app.refresh_textures(&ctx);
        assert_eq!(app.engine.project().program, id);
        assert!(app.input_fx.get(&id).unwrap().image_error.is_none());
        assert!(
            app.last_program_rgba
                .chunks_exact(4)
                .all(|p| p == [20, 100, 50, 255])
        );
    }

    #[test]
    fn invalid_apply_retains_the_editor_and_current_picture() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ready.png");
        fixture(&path, [150, 30, 10, 255]);
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        let id = app.engine.project().program;
        app.execute(Command::SetInputsPaused(true));
        app.refresh_textures(&ctx);
        let pixels = app.last_program_rgba.clone();
        let before = app.engine.project().clone();
        app.dispatch(Action::Properties(id), &ctx);
        app.request_property_png(id, path);
        finish_media(&mut app);
        let mut draft = app.properties.clone().unwrap();
        draft.name.clear();
        assert!(!app.apply_properties(draft));
        assert!(app.properties.is_some() && app.warning);
        assert_eq!(app.engine.project(), &before);
        app.refresh_textures(&ctx);
        assert_eq!(app.last_program_rgba, pixels);
    }

    #[test]
    fn closed_dialog_and_new_project_ignore_late_png_results() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("late.png");
        fixture(&path, [150, 30, 10, 255]);
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.dispatch(Action::Properties(InputId(1)), &ctx);
        app.request_property_png(InputId(1), path.clone());
        app.close_properties();
        app.dispatch(Action::Properties(InputId(2)), &ctx);
        finish_media(&mut app);
        assert!(app.properties_fx.as_ref().unwrap().media_path.is_none());
        assert!(
            app.engine
                .project()
                .inputs
                .iter()
                .all(|i| i.config.media_path.is_none())
        );
        app.close_properties();
        app.dispatch(Action::Add, &ctx);
        app.select_add_category(1);
        app.add_png_path = path.display().to_string();
        app.check_add_png();
        assert!(!app.request_add_input());
        app.reset_editors();
        app.media_loader.reset();
        finish_media(&mut app);
        assert_eq!(app.engine.project().inputs.len(), 7);
    }

    #[test]
    fn png_capacity_failure_does_not_create_cache_or_source() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        for n in 7..32 {
            app.execute(Command::AddInput {
                name: format!("Generator {n}"),
                kind: InputKind::Grid,
            });
        }
        app.dispatch(Action::Add, &ctx);
        app.select_add_category(1);
        app.add_png_path = "some.png".into();
        let before = app.engine.project().clone();
        assert!(!app.request_add_input());
        assert_eq!(app.engine.project(), &before);
        assert!(!app.media_loader.busy());
        assert_eq!(app.input_fx.len(), 32);
    }
    #[test]
    fn png_browse_preview_and_cancel_preserve_selection_without_blocking_the_ui() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preview.png");
        fixture(&path, [50, 100, 150, 255]);
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.dispatch(Action::Add, &ctx);
        app.select_add_category(1);
        let original = app.add_name.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        app.image_picker = Some(rx);
        app.add_loading = true;
        tx.send(media::PickerDone {
            target: media::MediaTarget::AddPreview {
                token: app.add_token,
            },
            path: None,
        })
        .unwrap();
        finish_media(&mut app);
        assert_eq!(app.add_name, original);
        assert!(!app.add_loading);
        let (tx, rx) = mpsc::sync_channel(1);
        app.image_picker = Some(rx);
        app.add_loading = true;
        tx.send(media::PickerDone {
            target: media::MediaTarget::AddPreview {
                token: app.add_token,
            },
            path: Some(path),
        })
        .unwrap();
        finish_media(&mut app);
        assert!(app.add_image.is_some());
        assert_eq!(app.engine.project().inputs.len(), 7);
        frame(&mut app, &ctx, vec![], [1600.0, 900.0]);
        assert!(app.automation.contains_key("dialog.add.preview"));
        let selected = app.add_png_path.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        app.image_picker = Some(rx);
        app.add_loading = true;
        tx.send(media::PickerDone {
            target: media::MediaTarget::AddPreview {
                token: app.add_token,
            },
            path: None,
        })
        .unwrap();
        finish_media(&mut app);
        assert_eq!(app.add_png_path, selected);
        assert!(app.add_image.is_some());
        click(&mut app, &ctx, "dialog.add.confirm", [1600.0, 900.0]);
        assert_eq!(app.engine.project().inputs.len(), 8);
        assert!(!app.add_open);
        assert!(app.input_fx.get(&InputId(8)).unwrap().custom_rgba.is_some());
    }
    #[test]
    fn busy_source_editor_blocks_selection_changes_and_cancel_discards_picker_results() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.dispatch(Action::Add, &ctx);
        let kind = app.add_kind;
        let (tx, rx) = mpsc::sync_channel(1);
        app.image_picker = Some(rx);
        app.add_loading = true;
        let token = app.add_token;
        click(&mut app, &ctx, "dialog.add.kind.Grid", [1600.0, 900.0]);
        assert_eq!(app.add_kind, kind);
        click(&mut app, &ctx, "dialog.add.cancel", [1600.0, 900.0]);
        tx.send(media::PickerDone {
            target: media::MediaTarget::AddPreview { token },
            path: Some("obsolete.png".into()),
        })
        .unwrap();
        finish_media(&mut app);
        assert!(!app.add_open && !app.add_loading);
        assert_eq!(app.engine.project().inputs.len(), 7);
        let id = InputId(1);
        app.dispatch(Action::Properties(id), &ctx);
        let (tx, rx) = mpsc::sync_channel(1);
        app.image_picker = Some(rx);
        app.properties_loading = true;
        let token = app.properties_token;
        click(
            &mut app,
            &ctx,
            "dialog.properties.kind.Bars",
            [1600.0, 900.0],
        );
        assert_eq!(app.properties.as_ref().unwrap().kind, InputKind::Grid);
        click(&mut app, &ctx, "dialog.properties.cancel", [1600.0, 900.0]);
        tx.send(media::PickerDone {
            target: media::MediaTarget::Properties { id, token },
            path: Some("obsolete.png".into()),
        })
        .unwrap();
        finish_media(&mut app);
        assert!(app.properties.is_none());
        assert!(
            app.engine
                .project()
                .input(id)
                .unwrap()
                .config
                .media_path
                .is_none()
        );
    }

    #[test]
    fn removal_confirmation_rechecks_live_protection_and_closes_editor_after_success() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let id = InputId(3);
        app.dispatch(Action::Properties(id), &ctx);
        app.dispatch(Action::RequestRemove(id), &ctx);
        app.execute(Command::TakeInput(id));
        click(&mut app, &ctx, "dialog.remove.confirm", [1600.0, 900.0]);
        assert!(app.engine.project().input(id).is_some());
        assert_eq!(app.confirm_remove_input, Some(id));
        assert!(app.properties.is_some());
        app.execute(Command::TakeInput(InputId(1)));
        app.execute(Command::SelectPreview(InputId(2)));
        click(&mut app, &ctx, "dialog.remove.confirm", [1600.0, 900.0]);
        assert!(app.engine.project().input(id).is_none());
        assert!(app.confirm_remove_input.is_none());
        assert!(app.properties.is_none() && app.properties_fx.is_none());
    }
}
