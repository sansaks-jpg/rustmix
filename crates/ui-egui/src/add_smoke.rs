//! Opt-in hosted screenshots and real sidebar/cancel clicks for Add Input only.
use super::*;

const SHOTS: [(&str, usize, &[&str]); 6] = [
    (
        "add-image",
        1,
        &[
            "png.path",
            "png.browse",
            "png.preview",
            "png.validate",
            "png.clear",
        ],
    ),
    (
        "add-colour",
        11,
        &["name", "colour.gradient", "colour.bars"],
    ),
    ("add-title-unavailable", 14, &["unavailable"]),
    (
        "add-audio",
        12,
        &["audio.path", "audio.browse", "audio.name", "audio.validate"],
    ),
    (
        "add-microphone",
        13,
        &["audio.capture", "audio.name", "audio.refresh"],
    ),
    ("add-generators", 0, &["name", "generator.0", "preview"]),
];

impl RustMixApp {
    pub(crate) fn smoke_add_input(
        &mut self,
        ctx: &egui::Context,
        s: &mut smoke::Smoke,
    ) -> Result<()> {
        if self.start.elapsed() > Duration::from_secs(180) {
            return Err(Error::Invalid(format!(
                "Add Input workflow timed out at stage {}",
                s.stage
            )));
        }
        for event in ctx.input(|input| input.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let Some((name, _, _)) = s
                    .stage
                    .checked_sub(10)
                    .and_then(|n| SHOTS.get((n / 3) as usize))
                else {
                    return Err(Error::Invalid("Unexpected Add Input screenshot".into()));
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
                std::fs::create_dir_all("artifacts")?;
                image::save_buffer(
                    "artifacts/add-input-fixture.png",
                    &[30, 140, 75, 255].repeat(16 * 9),
                    16,
                    9,
                    image::ColorType::Rgba8,
                )
                .map_err(|e| Error::Invalid(e.to_string()))?;
                s.scratch_text = Some(serde_json::to_string(self.engine.project())?);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(1600.0, 900.0)));
                self.dispatch(Action::Add, ctx);
                s.stage = 1;
            }
            1 => {
                self.native_click(s, "dialog.add.category.1")?;
                s.stage = 2;
            }
            2 => {
                self.add_png_path = PathBuf::from("artifacts/add-input-fixture.png")
                    .display()
                    .to_string();
                self.native_click(s, "dialog.add.png.validate")?;
                s.stage = 3;
            }
            3 => {
                if self.add_loading || self.media_loader.busy() {
                    return Ok(());
                }
                if let Some(reason) = self.add_disabled_reason() {
                    return Err(Error::Invalid(reason));
                }
                s.stage = 10;
            }
            10..=27 => {
                let offset = s.stage - 10;
                let Some((name, tab, fields)) = SHOTS.get((offset / 3) as usize) else {
                    return Err(Error::Invalid("Unknown Add Input capture".into()));
                };
                match offset % 3 {
                    0 => {
                        if !self.add_open || self.add_tab != *tab {
                            return Err(Error::Invalid(format!("Sidebar did not open {name}")));
                        }
                        if *tab == 11 && (self.add_image.is_some() || !self.add_png_path.is_empty())
                        {
                            return Err(Error::Invalid(
                                "PNG draft survived category change".into(),
                            ));
                        }
                        if matches!(*tab, 12 | 14) && self.add_disabled_reason().is_none() {
                            return Err(Error::Invalid(
                                "Unvalidated/unavailable input enabled OK".into(),
                            ));
                        }
                        s.stage += 1;
                    }
                    1 => {
                        if !s.visual_settled(ctx) {
                            return Ok(());
                        }
                        self.validate_dialog_geometry(ctx, "dialog.add")?;
                        let page = self
                            .automation
                            .get("dialog.add.page")
                            .ok_or_else(|| Error::Invalid("Missing Add page".into()))?;
                        for key in *fields {
                            let field = self
                                .automation
                                .get(&format!("dialog.add.{key}"))
                                .ok_or_else(|| {
                                    Error::Invalid(format!("Missing Add field {key}"))
                                })?;
                            if !page.expand(1.0).contains_rect(*field) {
                                return Err(Error::Invalid(format!("Clipped Add field {key}")));
                            }
                        }
                        std::fs::write(
                            format!("artifacts/{name}-elements.json"),
                            serde_json::to_vec_pretty(&serde_json::json!({
                                "category":tab,"ok_enabled":self.add_disabled_reason().is_none(),
                                "viewport":[ctx.viewport_rect().width(),ctx.viewport_rect().height()],
                                "pixels_per_point":ctx.pixels_per_point(),
                                "elements":self.automation.iter().filter(|(id,_)|id.starts_with("dialog.add")).map(|(id,r)|(id.clone(),format!("{r:?}"))).collect::<HashMap<_,_>>()
                            }))?,
                        )?;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::default(),
                        ));
                        s.captured = false;
                        s.stage += 1;
                    }
                    _ if s.captured => {
                        if let Some((_, next, _)) = SHOTS.get((offset / 3) as usize + 1) {
                            self.native_click(s, &format!("dialog.add.category.{next}"))?;
                        } else {
                            self.native_click(s, "dialog.add.cancel")?;
                        }
                        s.stage += 1;
                    }
                    _ => {}
                }
            }
            28 => {
                if self.add_open
                    || self.rec_active
                    || self.stream_active
                    || self.mc_running
                    || self.audio_ui.is_active()
                    || s.scratch_text.as_deref()
                        != Some(serde_json::to_string(self.engine.project())?.as_str())
                {
                    return Err(Error::Invalid(
                        "Add Input inspection changed project/output state".into(),
                    ));
                }
                std::fs::write(
                    "artifacts/add-input-workflow-ok.json",
                    serde_json::to_vec_pretty(&serde_json::json!({
                        "inspected_pages":SHOTS.len(),"png_draft_cleared_on_category_change":true,
                        "project_unchanged":true,"audio_monitoring_started":false,
                        "recording_started":false,"streaming_started":false
                    }))?,
                )?;
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => return Err(Error::Invalid("Unknown Add Input workflow stage".into())),
        }
        Ok(())
    }
}
