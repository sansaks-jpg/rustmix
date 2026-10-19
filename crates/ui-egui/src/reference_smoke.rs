//! Opt-in hosted native acceptance for the corrected reference workflows.
use super::*;
impl RustMixApp {
    pub(crate) fn smoke_reference(
        &mut self,
        ctx: &egui::Context,
        s: &mut smoke::Smoke,
    ) -> Result<()> {
        if self.start.elapsed() > Duration::from_secs(180) {
            return Err(Error::Invalid(format!(
                "Reference smoke timeout at {}",
                s.stage
            )));
        }
        let reopen = std::env::args().any(|arg| arg == "--ui-reference-smoke-reopen");
        for event in ctx.input(|input| input.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let name = if reopen {
                    "reference-reopened"
                } else {
                    match s.stage {
                        3 => "reference-categories",
                        6 => "reference-search",
                        10 => "reference-audio-hidden",
                        20 => "reference-output-settings",
                        _ => "reference-light",
                    }
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
        if s.wait < 5 || self.job.is_some() {
            return Ok(());
        }
        if self.engine.project().program != InputId(1)
            || self.engine.project().preview != InputId(2)
        {
            return Err(Error::Invalid(
                "Reference configuration changed Program/Preview".into(),
            ));
        }
        if reopen {
            let p = self.engine.project();
            Self::assert_reference_reopen(p)?;
            if s.stage == 0 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 1;
            } else if s.captured {
                std::fs::write(
                    "artifacts/reference-reopen-ok.json",
                    serde_json::to_vec_pretty(p)?,
                )?;
                self.saved_revision = p.revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            return Ok(());
        }
        match s.stage {
            0 => {
                std::fs::create_dir_all("artifacts")?;
                let mut preferences = self.engine.project().preferences.clone();
                preferences.preview_color = [80, 100, 200];
                preferences.program_color = [100, 160, 50];
                if let Some(c) = preferences.categories.iter_mut().find(|c| c.id == 21) {
                    c.name = "Interview".into();
                }
                self.dispatch(
                    Action::Engine(Command::ApplySettings {
                        name: self.engine.project().name.clone(),
                        preferences,
                    }),
                    ctx,
                );
                self.execute(Command::SetInputCategory(InputId(1), Some(0)));
                s.stage = 1;
            }
            1 => {
                self.native_click(s, "inputs.category.0")?;
                s.stage = 2;
            }
            2 => {
                if !self.automation.contains_key("input.1.preview")
                    || self.automation.contains_key("input.2.preview")
                {
                    return Err(Error::Invalid(
                        "Category filter does not use assignment".into(),
                    ));
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 3;
            }
            3 if s.captured => {
                self.native_click(s, "inputs.search")?;
                s.stage = 4;
            }
            4 => {
                if !self.search_open {
                    return Err(Error::Invalid("Input Search did not open".into()));
                }
                self.search_draft = "Camera B".into();
                s.stage = 5;
            }
            5 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 6;
            }
            6 if s.captured => {
                self.native_click(s, "dialog.search.apply")?;
                s.stage = 7;
            }
            7 => {
                if self.search_open || self.automation.contains_key("input.1.preview") {
                    return Err(Error::Invalid(
                        "Combined category/search filter failed".into(),
                    ));
                }
                self.native_click(s, "inputs.filter.clear")?;
                s.stage = 8;
            }
            8 => {
                self.native_click(s, "output.audioPane")?;
                s.stage = 9;
            }
            9 => {
                if self.audio_restore.is_none() || self.dock.find_tab(&Panel::Audio).is_some() {
                    return Err(Error::Invalid("Audio pane did not hide".into()));
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 10;
            }
            10 if s.captured => {
                self.native_click(s, "output.audioPane")?;
                s.stage = 11;
            }
            11 => {
                if self.dock.find_tab(&Panel::Audio).is_none() {
                    return Err(Error::Invalid("Audio pane did not restore".into()));
                }
                let mut preferences = self.engine.project().preferences.clone();
                preferences.ui_theme = UiTheme::Light;
                self.dispatch(
                    Action::Engine(Command::ApplySettings {
                        name: self.engine.project().name.clone(),
                        preferences,
                    }),
                    ctx,
                );
                s.stage = 12;
            }
            12 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 13;
            }
            13 if s.captured => {
                self.native_click(s, "output.external.open")?;
                s.stage = 14;
            }
            14 => {
                self.validate_dialog_geometry(ctx, "dialog.external")?;
                self.native_click(s, "dialog.external.source.program")?;
                s.stage = 15;
            }
            15 => {
                self.native_click(s, "dialog.external.rate")?;
                s.stage = 16;
            }
            16 => {
                self.native_click(s, "dialog.external.rate.59.94")?;
                s.stage = 17;
            }
            17 => {
                self.native_click(s, "dialog.external.port_toggle")?;
                s.stage = 18;
            }
            18 => {
                self.native_click(s, "dialog.external.apply")?;
                s.stage = 19;
            }
            19 => {
                Self::assert_reference_outputs(self.engine.project())?;
                if self.external_settings.is_none() || self.external_active {
                    return Err(Error::Invalid(
                        "Output Apply lost the editor or started an unavailable backend".into(),
                    ));
                }
                self.validate_dialog_geometry(ctx, "dialog.external")?;
                std::fs::write(
                    "artifacts/reference-output-elements.json",
                    serde_json::to_vec_pretty(
                        &self
                            .automation
                            .iter()
                            .filter(|(id, _)| id.starts_with("dialog.external"))
                            .map(|(id, rect)| (id, format!("{rect:?}")))
                            .collect::<HashMap<_, _>>(),
                    )?,
                )?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 20;
            }
            20 if s.captured => {
                self.native_click(s, "dialog.external.cancel")?;
                s.stage = 21;
            }
            21 => {
                if self.external_settings.is_some() {
                    return Err(Error::Invalid(
                        "External Output Cancel did not close".into(),
                    ));
                }
                if self.scripted_save("artifacts/reference.rustmix.json", ctx)? {
                    s.stage = 22;
                }
            }
            22 => {
                if self.job.is_some() {
                    return Ok(());
                }
                if !PathBuf::from("artifacts/reference.rustmix.json").exists() {
                    return Err(Error::Invalid(format!(
                        "Reference smoke save did not write the project (status: {})",
                        self.notice
                    )));
                }
                std::fs::write(
                    "artifacts/reference-smoke-ok.json",
                    serde_json::to_vec_pretty(self.engine.project())?,
                )?;
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
        }
        Ok(())
    }

    /// Hosted reopen expectation for the reference workflow. It is shared with
    /// the widget tests so a schema bump cannot silently break the native
    /// reopen check again.
    pub(crate) fn assert_reference_reopen(state: &ProjectState) -> Result<()> {
        Self::assert_reference_outputs(state)?;
        if state.schema != SCHEMA
            || state
                .input(InputId(1))
                .is_none_or(|i| i.category != Some(0))
            || state.preferences.ui_theme != UiTheme::Light
            || state.preferences.preview_color != [80, 100, 200]
            || state
                .preferences
                .categories
                .iter()
                .find(|c| c.id == 21)
                .is_none_or(|c| c.name != "Interview")
        {
            return Err(Error::Invalid(
                "Reference category/theme configuration did not reopen".into(),
            ));
        }
        Ok(())
    }

    fn assert_reference_outputs(state: &ProjectState) -> Result<()> {
        let expected_rate = Rate::new(60000, 1001)?;
        if state.outputs.slot(1).is_none_or(|output| {
            output.source != OutputSource::Program
                || output.config.frame_rate != expected_rate
                || output.config.port != Some(1)
                || output.config.width != 1920
                || output.config.height != 1080
        }) || state
            .outputs
            .iter()
            .skip(1)
            .any(|output| output.source != OutputSource::None)
        {
            return Err(Error::Invalid(
                "External output configuration did not apply/reopen independently".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn reference_reopen_expectation_survives_the_current_schema_round_trip() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        let mut preferences = app.engine.project().preferences.clone();
        preferences.ui_theme = UiTheme::Light;
        preferences.preview_color = [80, 100, 200];
        preferences.program_color = [100, 160, 50];
        if let Some(c) = preferences.categories.iter_mut().find(|c| c.id == 21) {
            c.name = "Interview".into();
        }
        app.dispatch(
            Action::Engine(Command::ApplySettings {
                name: app.engine.project().name.clone(),
                preferences,
            }),
            &ctx,
        );
        app.execute(Command::SetInputCategory(InputId(1), Some(0)));
        let mut output = app.engine.project().outputs.slot(1).unwrap().clone();
        output.source = OutputSource::Program;
        output.config.frame_rate = Rate::new(60000, 1001).unwrap();
        output.config.port = Some(1);
        app.execute(Command::SetExternalOutput(output));
        assert!(RustMixApp::assert_reference_reopen(app.engine.project()).is_ok());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reference.rustmix.json");
        persistence::save(&path, app.engine.project()).unwrap();
        let reopened = persistence::load(&path).unwrap();
        assert!(RustMixApp::assert_reference_reopen(&reopened).is_ok());
    }
}
