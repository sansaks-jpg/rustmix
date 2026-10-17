//! Opt-in hosted-runner harness. Real native frames, egui keyboard/pointer input,
//! state assertions, project reopen, and GPU readback screenshots; no external output.
use super::*;
// Only consumed by the opt-in hosted native harness, never by normal startup.
const DIALOG_SHOTS: [(&str, [f32; 2], u8); 10] = [
    ("add-input-1100", [1100.0, 700.0], 0),
    ("input-properties-1100", [1100.0, 700.0], 1),
    ("settings-1100", [1100.0, 700.0], 2),
    ("add-input-1600", [1600.0, 900.0], 0),
    ("input-properties-1600", [1600.0, 900.0], 1),
    ("settings-1600", [1600.0, 900.0], 2),
    ("add-input-1920", [1920.0, 1080.0], 0),
    ("input-properties-1920", [1920.0, 1080.0], 1),
    ("settings-1920", [1920.0, 1080.0], 2),
    ("overlay-settings-1600", [1600.0, 900.0], 3),
];
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
        if self.start.elapsed() > Duration::from_secs(180) {
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
                s.stage = 20;
            }

            20..=49 => {
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
                                _ => Action::OverlaySettings(0),
                            },
                            ctx,
                        );
                        let [width, height] = *size;
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            width, height,
                        )));
                        s.stage += 1;
                    }
                    1 => {
                        self.validate_dialog_geometry(
                            ctx,
                            match kind {
                                0 => "dialog.add",
                                1 => "dialog.properties",
                                2 => "dialog.settings",
                                _ => "dialog.overlay",
                            },
                        )?;
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
            50 => {
                self.assert_smoke_state()?;
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

        app.category = Some(InputKind::Bars);
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
        // The vMix-style T-Bar is vertical: the handle rests at the top and the
        // transition completes when it is pulled to the bottom.
        let pos = egui::pos2(rect.center().x, rect.bottom() - 1.0);
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
    }
}
