//! Opt-in hosted-runner harness. Real native frames, egui keyboard/pointer input,
//! state assertions, project reopen, and GPU readback screenshots; no external output.
use super::*;
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
#[derive(Default)]
pub(crate) struct Smoke {
    pub events: Vec<egui::Event>,
    pub(crate) stage: u32,
    frames: u32,
    last_stage: u32,
    pub(crate) wait: u32,
    pub(crate) captured: bool,
    bank_baseline: Option<Rect>,
    paused_program: Option<(u64, Vec<u8>, Rect)>,
    transport_target: Option<Rect>,
    basic_state: Option<(InputId, InputId, u64)>,
}
impl Smoke {
    pub(crate) fn is_dialog_stage(&self) -> bool {
        self.stage >= 20
    }
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
    pub(crate) fn smoke_step(&mut self, ctx: &egui::Context) {
        let Some(mut s) = self.smoke.take() else {
            return;
        };
        s.frames = s.frames.saturating_add(1);
        if s.stage != s.last_stage {
            s.wait = 0;
            s.last_stage = s.stage;
        }
        s.wait = s.wait.saturating_add(1);
        let result = self.smoke_run(ctx, &mut s);
        if let Err(e) = result {
            let _ = std::fs::create_dir_all("artifacts");
            let _ = std::fs::write("artifacts/smoke-error.txt", e.to_string());
            self.saved_revision = self.engine.project().revision;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.smoke = Some(s);
    }
    fn smoke_run(&mut self, ctx: &egui::Context, s: &mut Smoke) -> Result<()> {
        if std::env::args()
            .any(|arg| arg == "--ui-reference-smoke" || arg == "--ui-reference-smoke-reopen")
        {
            return self.smoke_reference(ctx, s);
        }
        if std::env::args().any(|s| s == "--ui-input-smoke" || s == "--ui-input-smoke-reopen") {
            return self.smoke_inputs(ctx, s);
        }
        let reopen = std::env::args().any(|s| s == "--ui-smoke-reopen");
        // Includes 60 idle + 35 paused native frames and all nine dialogs on
        // hosted software graphics. Keep the matching external limit at 270s.
        if self.start.elapsed() > Duration::from_secs(240) {
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
                std::fs::create_dir_all("artifacts")?;
                self.path = Some(PathBuf::from("artifacts/smoke.rustmix.json"));
                self.dispatch(Action::Save(false), ctx);
                s.stage = 11;
            }
            11 => {
                if self.job.is_some() {
                    return Ok(());
                }
                if !PathBuf::from("artifacts/smoke.rustmix.json").exists() {
                    return Err(Error::Invalid(
                        "UI save worker did not write the project".into(),
                    ));
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
            18 if s.captured => s.stage = 20,

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
                self.assert_smoke_state()?;
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            _ => {}
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
            if self.job.is_some() || self.media_loader.busy() {
                return Ok(());
            }
            self.assert_input_smoke()?;
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
                self.add_name = path.display().to_string();
                self.add_number = 2;
                s.stage = 1;
            }
            1 => {
                self.validate_dialog_geometry(ctx, "dialog.add")?;
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
                self.path = Some("artifacts/inputs.rustmix.json".into());
                self.start_save(false, false);
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 8;
            }
            8 => {
                if self.job.is_some() || !s.captured {
                    return Ok(());
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
            || !p.overlays.first().is_some_and(|o| o.enabled)
            || !p.buses.first().is_some_and(|b| b.gain_db < -1.0)
            || p.workspace.dock_json.is_empty()
        {
            return Err(Error::Invalid(
                "Smoke final state mismatch (PGM/PVW/overlay/master/workspace)".into(),
            ));
        }
        Ok(())
    }
    fn validate_dialog_geometry(&self, ctx: &egui::Context, prefix: &str) -> Result<()> {
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
        Ok(())
    }
    fn validate_monitor_geometry(&self, right: f32) -> Result<()> {
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
        if !self.basic_mode {
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
    use super::*;

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
        app.add_kind = InputKind::Bars;
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
        assert!(app.engine.project().overlays[0].enabled);
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
        app.add_name = "missing-video.mp4".into();
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![], size);
        }
        let count = app.engine.project().inputs.len();
        click(&mut app, &ctx, "dialog.add.confirm", size);
        assert_eq!(app.engine.project().inputs.len(), count);
        assert!(app.add_open && app.warning);
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
            app.engine.project().overlays[0].position,
            Position::LowerLeft
        );
        assert_eq!(app.engine.project().revision, before);

        // Switching the number reloads that slot's committed state.
        click(&mut app, &ctx, "dialog.overlay.slot.2", size);
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
            app.engine.project().overlays[0].position,
            Position::LowerLeft
        );

        // Apply commits only the selected slot.
        click(&mut app, &ctx, "overlay.2.settings", size);
        click(&mut app, &ctx, "dialog.overlay.enabled", size);
        click(&mut app, &ctx, "dialog.overlay.position.LowerRight", size);
        click(&mut app, &ctx, "dialog.overlay.apply", size);
        let project = app.engine.project();
        assert!(project.overlays[1].enabled);
        assert_eq!(project.overlays[1].position, Position::LowerRight);
        assert_eq!(project.overlays[0].position, Position::LowerLeft);
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
        app.add_name = path.display().to_string();
        app.add_number = 2;
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
        app.add_name = path.display().to_string();
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
        app.add_name = "some.png".into();
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
        let selected = app.add_name.clone();
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
        assert_eq!(app.add_name, selected);
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
        click(&mut app, &ctx, "dialog.add.kind.Geometry", [1600.0, 900.0]);
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
