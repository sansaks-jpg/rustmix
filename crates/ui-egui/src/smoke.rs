//! Opt-in hosted-runner harness. Real native frames, egui keyboard/pointer input,
//! state assertions, project reopen, and GPU readback screenshots; no external output.
use super::*;
#[derive(Default)]
pub(crate) struct Smoke {
    pub events: Vec<egui::Event>,
    stage: u32,
    frames: u32,
    last_stage: u32,
    wait: u32,
    captured: bool,
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
        let reopen = std::env::args().any(|s| s == "--ui-smoke-reopen");
        if self.start.elapsed() > Duration::from_secs(85) {
            return Err(Error::Invalid("Native UI smoke timed out".into()));
        }
        let events = ctx.input(|i| i.events.clone());
        for event in events {
            if let egui::Event::Screenshot { image, .. } = event {
                std::fs::create_dir_all("artifacts")?;
                let filename = if reopen {
                    "artifacts/reopened.png"
                } else if s.stage == 7 {
                    "artifacts/workspace-1600.png"
                } else {
                    "artifacts/workspace-1920.png"
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
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
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
    fn validate_monitor_geometry(&self, right: f32) -> Result<()> {
        let preview = self
            .automation
            .get("monitor.preview")
            .ok_or_else(|| Error::Invalid("Preview viewport was not rendered".into()))?;
        let program = self
            .automation
            .get("monitor.program")
            .ok_or_else(|| Error::Invalid("Program viewport was not rendered".into()))?;
        let cut = self
            .automation
            .get("switcher.cut")
            .ok_or_else(|| Error::Invalid("CUT control was not rendered".into()))?;
        let fade = self
            .automation
            .get("switcher.fade")
            .ok_or_else(|| Error::Invalid("FADE control was not rendered".into()))?;
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
        if preview.right() > cut.left()
            || cut.right() > program.left()
            || (cut.left() - fade.left()).abs() > 2.0
            || fade.top() < cut.bottom()
        {
            return Err(Error::Invalid(
                "Transition controls overlap the monitors or are not stacked".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
    use super::*;
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
        for size in [[1920.0, 1080.0], [1600.0, 900.0], [1100.0, 700.0]] {
            frame(&mut app, &ctx, vec![], size);
            assert!(app.automation.get("switcher.cut").unwrap().is_finite());
            app.validate_monitor_geometry(size[0]).unwrap();
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
}
