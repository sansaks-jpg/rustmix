//! New-session format draft. Cancelling never mutates the active engine.
use super::*;
use rustmix_engine::video_format::{CANVAS_SIZES, rate_options};

#[derive(Default)]
pub(crate) struct NewPresetUi {
    pub(crate) format: VideoStandard,
    replace_unsaved: bool,
}

impl RustMixApp {
    /// Offline hosted acceptance using the same initialization as normal startup.
    pub(crate) fn smoke_new_preset(
        &mut self,
        ctx: &egui::Context,
        s: &mut smoke::Smoke,
    ) -> Result<()> {
        if self.start.elapsed() > Duration::from_secs(150) {
            return Err(Error::Invalid(format!(
                "New Preset workflow timed out at {}",
                s.stage
            )));
        }
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let name = match s.stage {
                    1 => "new-preset-startup",
                    8 => "new-preset-portrait",
                    10 => "new-preset-workspace",
                    _ => return Err(Error::Invalid("Unexpected preset screenshot".into())),
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
        if s.wait < 4 {
            return Ok(());
        }
        match s.stage {
            0 => {
                if !self.confirm_new || self.engine.project().inputs.len() != 2 {
                    return Err(Error::Invalid(
                        "Cold startup did not open a blank preset selector".into(),
                    ));
                }
                if !s.visual_settled(ctx) {
                    return Ok(());
                }
                std::fs::create_dir_all("artifacts")?;
                s.scratch_text = Some(serde_json::to_string(self.engine.project())?);
                self.capture_new_preset(ctx, "new-preset-startup")?;
                s.captured = false;
                s.stage = 1;
            }
            1 if s.captured => {
                self.native_click(s, "dialog.new.cancel")?;
                s.stage = 2;
            }
            2 => {
                if self.confirm_new
                    || s.scratch_text.as_deref()
                        != Some(serde_json::to_string(self.engine.project())?.as_str())
                {
                    return Err(Error::Invalid(
                        "Cancelling startup changed the preset".into(),
                    ));
                }
                self.native_click(s, "toolbar.new")?;
                s.stage = 3;
            }
            3 => {
                self.native_click(s, "dialog.new.size")?;
                s.stage = 4;
            }
            4 => {
                self.native_click(s, "dialog.new.size.1080x1920")?;
                s.stage = 5;
            }
            5 => {
                self.native_click(s, "dialog.new.rate")?;
                s.stage = 6;
            }
            6 => {
                self.native_click(s, "dialog.new.rate.60.00.p")?;
                s.stage = 7;
            }
            7 => {
                if !s.visual_settled(ctx) {
                    return Ok(());
                }
                if s.scratch_text.as_deref()
                    != Some(serde_json::to_string(self.engine.project())?.as_str())
                {
                    return Err(Error::Invalid(
                        "New Preset draft changed the live engine".into(),
                    ));
                }
                self.capture_new_preset(ctx, "new-preset-portrait")?;
                s.captured = false;
                s.stage = 8;
            }
            8 if s.captured => {
                self.native_click(s, "dialog.new.confirm")?;
                s.stage = 9;
            }
            9 => {
                if self.confirm_new {
                    return Err(Error::Invalid("New Preset did not close after OK".into()));
                }
                if self
                    .program_texture
                    .as_ref()
                    .is_none_or(|t| t.size() != [360, 640])
                    || self.program_presented_generation != Some(self.program_generation)
                {
                    return Ok(());
                }
                let project = self.engine.project();
                if (project.preferences.width, project.preferences.height) != (1080, 1920)
                    || project.preferences.rate != Rate::new(60, 1)?
                    || project.inputs.len() != 2
                    || !self.video_players.is_empty()
                    || self.audio_ui.is_active()
                {
                    return Err(Error::Invalid(
                        "New Preset format or isolation failed".into(),
                    ));
                }
                persistence::save(Path::new("artifacts/new-preset.rustmix.json"), project)?;
                if persistence::load(Path::new("artifacts/new-preset.rustmix.json"))? != *project {
                    return Err(Error::Invalid("Preset roundtrip changed format".into()));
                }
                std::fs::write(
                    "artifacts/new-preset-ok.json",
                    serde_json::to_vec_pretty(
                        &serde_json::json!({"startup_selector":true,"new_button":true,"cancel_unchanged":true,"width":1080,"height":1920,"rate":[60,1],"scan":"Progressive","monitor_size":[360,640],"frame_bytes":self.last_program_rgba.len(),"audio_started":false,"roundtrip":true}),
                    )?,
                )?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 10;
            }
            10 if s.captured => {
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
        }
        Ok(())
    }
    fn capture_new_preset(&self, ctx: &egui::Context, name: &str) -> Result<()> {
        std::fs::write(
            format!("artifacts/{name}-elements.json"),
            serde_json::to_vec_pretty(
                &serde_json::json!({"width":self.new_preset.format.width,"height":self.new_preset.format.height,"rate":self.new_preset.format.rate,"scan":self.new_preset.format.scan,"elements":self.automation.iter().filter(|(id,_)|id.starts_with("dialog.new")).map(|(id,r)|(id.clone(),format!("{r:?}"))).collect::<HashMap<_,_>>()}),
            )?,
        )?;
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        Ok(())
    }
    pub(crate) fn initialize_blank_startup(&mut self, ctx: &egui::Context) {
        match Engine::new_preset(VideoStandard::default(), self.now) {
            Ok(engine) => {
                self.engine = engine;
                self.input_fx.clear();
                self.sync_input_cache();
                self.theme(ctx);
                self.open_new_preset();
            }
            Err(error) => self.say(error.to_string(), true),
        }
    }
    pub(crate) fn open_new_preset(&mut self) {
        self.new_preset.format = self.engine.project().preferences.video_standard();
        self.new_preset.replace_unsaved = false;
        self.confirm_new = true;
    }
    pub(crate) fn accept_new_preset(&mut self, ctx: &egui::Context) -> bool {
        if self.job.is_some()
            || (self.saved_revision != self.engine.project().revision
                && !self.new_preset.replace_unsaved)
        {
            return false;
        }
        match Engine::new_preset(self.new_preset.format, self.now) {
            Ok(engine) => {
                self.engine = engine;
                self.reset_editors();
                self.media_loader.reset();
                self.input_fx.clear();
                self.sync_input_cache();
                self.program_pixels = None;
                self.program_texture = None;
                self.last_program_rgba.clear();
                self.last_program_full = Arc::new(Vec::new());
                self.program_full_size = [0, 0];
                self.program_renderer = program::ProgramRenderer::default();
                self.submit_program();
                self.theme(ctx);
                self.dock = default_dock(false);
                self.audio_restore = None;
                self.path = None;
                self.saved_revision = 0;
                self.textures.clear();
                self.thumbnail_pixels.clear();
                self.last_demo_frame = None;
                self.confirm_new = false;
                self.say("New preset created · add a source to begin".into(), false);
                true
            }
            Err(error) => {
                self.say(error.to_string(), true);
                false
            }
        }
    }
    pub(crate) fn new_preset_dialog(&mut self, ctx: &egui::Context) {
        if !self.confirm_new {
            return;
        }
        let mut open = true;
        let mut create = false;
        let mut cancel = false;
        dialogs::operator_window(ctx, "New Preset", "dialog.new", &mut open, Vec2::new(760.0, 254.0))
            .show(ctx, |ui| {
                egui::Frame::new().fill(Color32::from_rgb(10,15,20)).inner_margin(14).show(ui, |ui| {
                    ui.set_min_width((ui.available_width()-1.0).max(100.0));
                    ui.vertical_centered(|ui| { ui.strong("Create a new blank preset with the following video format"); });
                });
                ui.add_space(18.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    let width = (ui.available_width()*0.63).clamp(150.0,460.0);
                    let format = &mut self.new_preset.format;
                    let size = egui::ComboBox::from_id_salt("new.size").width(width)
                        .selected_text(format!("{}   ({}×{})", if format.height > format.width {"Portrait"} else {"Landscape"},format.width,format.height))
                        .show_ui(ui, |ui| {
                            for (w,h) in CANVAS_SIZES {
                                let option = ui.selectable_label((w,h)==(format.width,format.height),format!("{}   {w}×{h}",if h>w {"Portrait"} else {"Landscape"}));
                                self.automation.insert(format!("dialog.new.size.{w}x{h}"),option.rect);
                                if option.clicked() { format.width=w; format.height=h; ui.close(); }
                            }
                        });
                    self.automation.insert("dialog.new.size".into(),size.response.rect);
                    let rate = egui::ComboBox::from_id_salt("new.rate").width(210.0)
                        .selected_text(format.scan.rate_label(format.rate)).show_ui(ui, |ui| {
                            for (rate,scan) in rate_options() {
                                let option = ui.selectable_label(format.rate==rate && format.scan==scan,scan.rate_label(rate));
                                self.automation.insert(format!("dialog.new.rate.{}.{}",rate.label(),scan.label()),option.rect);
                                if option.clicked() { format.rate=rate; format.scan=scan; ui.close(); }
                            }
                        });
                    self.automation.insert("dialog.new.rate".into(),rate.response.rect);
                });
                ui.add_space(12.0);
                ui.label(RichText::new("Program monitoring uses a reduced progressive picture. Full-resolution / interlaced broadcast output is not available in this build.").small());
                let dirty = self.saved_revision != self.engine.project().revision;
                if dirty {
                    ui.horizontal_wrapped(|ui| {
                        let save=ui.add_enabled(self.job.is_none(),egui::Button::new("Save current"));
                        self.automation.insert("dialog.new.save".into(),save.rect);
                        if save.clicked(){ self.actions.push(Action::Save(false)); }
                        let acknowledge=ui.checkbox(&mut self.new_preset.replace_unsaved,"Replace the current unsaved preset");
                        self.automation.insert("dialog.new.replaceUnsaved".into(),acknowledge.rect);
                    });
                }
                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let response=ui.add_sized([150.0,38.0],egui::Button::new("Cancel"));
                    self.automation.insert("dialog.new.cancel".into(),response.rect);
                    cancel=response.clicked();
                    let response=ui.add_enabled(self.job.is_none() && (!dirty || self.new_preset.replace_unsaved),egui::Button::new("OK").min_size(Vec2::new(150.0,38.0)));
                    self.automation.insert("dialog.new.confirm".into(),response.rect);
                    create=response.clicked();
                });
            });
        if create {
            self.accept_new_preset(ctx);
        }
        if cancel || !open {
            self.confirm_new = false;
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]
    use super::*;
    #[test]
    fn cold_start_opens_the_format_dialog_without_opening_any_sources() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.initialize_blank_startup(&ctx);
        assert!(app.confirm_new);
        assert_eq!(app.engine.project().inputs.len(), 2);
        assert!(app.video_players.is_empty());
        assert!(!app.audio_ui.is_active());
        assert_eq!(app.new_preset.format, VideoStandard::default());
    }
    fn frame(app: &mut RustMixApp, ctx: &egui::Context, size: [f32; 2], events: Vec<egui::Event>) {
        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::from(size))),
                events,
                ..Default::default()
            },
            |ui| app.frame_ui(ui),
        );
    }
    fn click(app: &mut RustMixApp, ctx: &egui::Context, id: &str, size: [f32; 2]) {
        for _ in 0..4 {
            frame(app, ctx, size, vec![]);
        }
        let point = app.automation.get(id).unwrap().center();
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                size,
                vec![
                    egui::Event::PointerMoved(point),
                    egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }
    #[test]
    fn real_new_widgets_stage_formats_cancel_and_protect_unsaved_work() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1100.0, 700.0];
        let before = app.engine.project().clone();
        app.dispatch(Action::New, &ctx);
        click(&mut app, &ctx, "dialog.new.size", size);
        click(&mut app, &ctx, "dialog.new.size.1080x1920", size);
        click(&mut app, &ctx, "dialog.new.rate", size);
        click(&mut app, &ctx, "dialog.new.rate.60.00.p", size);
        assert_eq!(*app.engine.project(), before);
        click(&mut app, &ctx, "dialog.new.cancel", size);
        assert_eq!(*app.engine.project(), before);
        app.execute(Command::RenameProject("Unsaved production".into()));
        let dirty = app.engine.project().clone();
        app.dispatch(Action::New, &ctx);
        click(&mut app, &ctx, "dialog.new.confirm", size);
        assert!(app.confirm_new);
        assert_eq!(*app.engine.project(), dirty);
        click(&mut app, &ctx, "dialog.new.replaceUnsaved", size);
        app.new_preset.format.width = 1080;
        app.new_preset.format.height = 1920;
        app.new_preset.format.rate = Rate::new(25, 1).unwrap();
        click(&mut app, &ctx, "dialog.new.confirm", size);
        assert!(!app.confirm_new);
        assert_eq!(app.engine.project().inputs.len(), 2);
        assert_eq!(
            app.engine
                .project()
                .preferences
                .video_standard()
                .monitor_size(),
            [360, 640]
        );
        frame(&mut app, &ctx, size, vec![]);
        assert_eq!(app.program_texture.as_ref().unwrap().size(), [360, 640]);
    }
    #[test]
    fn full_hd_25_and_60_landscape_widgets_commit_the_expected_monitor_raster() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1100.0, 700.0];
        for (rate_id, expected) in [
            ("25.00.p", Rate::new(25, 1).unwrap()),
            ("60.00.p", Rate::new(60, 1).unwrap()),
        ] {
            app.dispatch(Action::New, &ctx);
            click(&mut app, &ctx, "dialog.new.size", size);
            click(&mut app, &ctx, "dialog.new.size.1920x1080", size);
            click(&mut app, &ctx, "dialog.new.rate", size);
            click(&mut app, &ctx, &format!("dialog.new.rate.{rate_id}"), size);
            click(&mut app, &ctx, "dialog.new.confirm", size);
            assert!(!app.confirm_new, "New did not close for {rate_id}");
            let preferences = &app.engine.project().preferences;
            assert_eq!((preferences.width, preferences.height), (1920, 1080));
            assert_eq!(preferences.rate, expected);
            assert_eq!(preferences.scan, ScanMode::Progressive);
            assert_eq!(preferences.video_standard().monitor_size(), [640, 360]);
            assert_eq!(app.engine.project().inputs.len(), 2);
            frame(&mut app, &ctx, size, vec![]);
            assert_eq!(
                app.program_texture.as_ref().map(egui::TextureHandle::size),
                Some([640, 360]),
                "monitor raster did not follow {rate_id}"
            );
            app.saved_revision = app.engine.project().revision;
        }
    }
}
