//! Opt-in hosted native acceptance for the separate Program viewport.
use super::*;

impl RustMixApp {
    pub(crate) fn smoke_program_output(
        &mut self,
        ctx: &egui::Context,
        s: &mut smoke::Smoke,
    ) -> Result<()> {
        if self.start.elapsed() > Duration::from_secs(240) {
            return Err(Error::Invalid(format!(
                "Program output smoke timed out at {}",
                s.stage
            )));
        }
        if s.wait < 5 || self.job.is_some() {
            return Ok(());
        }
        let reopen = std::env::args().any(|arg| arg == "--ui-output-smoke-reopen");
        if reopen {
            if self.program_output.is_open()
                || self.fullscreen
                || self.engine.project().program != InputId(3)
                || self.engine.project().preview != InputId(6)
                || !self
                    .engine
                    .project()
                    .overlays
                    .standard
                    .first()
                    .is_some_and(|o| o.enabled)
            {
                return Err(Error::Invalid(
                    "Program output project/reopen or safe closed startup mismatch".into(),
                ));
            }
            std::fs::write(
                "artifacts/program-output-reopen-ok.json",
                serde_json::to_vec_pretty(self.engine.project())?,
            )?;
            self.saved_revision = self.engine.project().revision;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return Ok(());
        }
        match s.stage {
            0 => {
                std::fs::create_dir_all("artifacts")?;
                self.execute(Command::SetInputsPaused(true));
                self.execute(Command::TakeInput(InputId(6)));
                self.native_click(s, "output.program.menu")?;
                s.stage = 1;
            }
            1 => {
                self.native_click(s, "output.program.open")?;
                s.stage = 2;
            }
            2 => {
                if !self.program_output.native_seen() {
                    return Ok(());
                }
                self.native_click(s, "output.program.menu")?;
                s.stage = 3;
            }
            3 => {
                self.native_click(s, "output.program.fullscreen")?;
                s.stage = 4;
            }
            4 => {
                if !self.program_output.native_fullscreen() {
                    return Ok(());
                }
                if self.fullscreen {
                    return Err(Error::Invalid(
                        "Program fullscreen changed the operator window".into(),
                    ));
                }
                self.native_click(s, "output.program.menu")?;
                s.stage = 5;
            }
            5 => {
                self.native_click(s, "output.program.fullscreen")?;
                s.stage = 6;
            }
            6 => {
                if self.program_output.native_fullscreen() {
                    return Ok(());
                }
                self.execute(Command::SelectPreview(InputId(3)));
                s.stage = 7;
            }
            7 => {
                if !self.output_smoke_frame_current()? {
                    return Ok(());
                }
                self.program_output.request_capture(ctx);
                s.stage = 8;
            }
            8 => {
                if !self.save_output_capture("program-output-window", false)? {
                    return Ok(());
                }
                self.execute(Command::FadeBlack);
                s.stage = 9;
            }
            9 => {
                if self.engine.black_level() < 1.0 || !self.output_smoke_frame_current()? {
                    return Ok(());
                }
                self.program_output.request_capture(ctx);
                s.stage = 10;
            }
            10 => {
                if !self.save_output_capture("program-output-black", true)? {
                    return Ok(());
                }
                self.execute(Command::FadeBlack);
                self.execute(Command::Transition(TransitionKind::Fade));
                s.stage = 11;
            }
            11 => {
                if self.engine.transition().is_some() || self.engine.black_level() > 0.0 {
                    return Ok(());
                }
                let mut overlay = self
                    .engine
                    .project()
                    .overlays
                    .standard
                    .first()
                    .cloned()
                    .ok_or_else(|| Error::Invalid("Program overlay bank missing".into()))?;
                overlay.enabled = true;
                self.execute(Command::SetOverlay(0, overlay));
                s.stage = 12;
            }
            12 => {
                if !self.output_smoke_frame_current()? {
                    return Ok(());
                }
                self.program_output.request_capture(ctx);
                s.stage = 13;
            }
            13 => {
                if !self.save_output_capture("program-output-live", false)? {
                    return Ok(());
                }
                // Native close request must tear down only the child window.
                ctx.send_viewport_cmd_to(
                    program_output::viewport_id(),
                    egui::ViewportCommand::Close,
                );
                s.stage = 14;
            }
            14 => {
                if self.program_output.is_open() {
                    return Ok(());
                }
                if self.fullscreen
                    || self.engine.project().program != InputId(3)
                    || self.engine.project().preview != InputId(6)
                    || self.allow_close
                {
                    return Err(Error::Invalid(
                        "Closing Program viewport disturbed the operator or buses".into(),
                    ));
                }
                self.native_click(s, "output.program.menu")?;
                s.stage = 15;
            }
            15 => {
                self.native_click(s, "output.program.open")?;
                s.stage = 16;
            }
            16 => {
                if !self.program_output.native_seen() {
                    return Ok(());
                }
                self.native_click(s, "output.program.menu")?;
                s.stage = 17;
            }
            17 => {
                self.native_click(s, "output.program.close")?;
                s.stage = 18;
            }
            18 => {
                if self.program_output.is_open() {
                    return Err(Error::Invalid(
                        "Program Close control did not close output".into(),
                    ));
                }
                if self.scripted_save("artifacts/program-output.rustmix.json", ctx)? {
                    s.stage = 19;
                }
            }
            19 => {
                if !PathBuf::from("artifacts/program-output.rustmix.json").exists() {
                    return Err(Error::Invalid("Program output smoke save missing".into()));
                }
                std::fs::write(
                    "artifacts/program-output-ok.json",
                    serde_json::to_vec_pretty(self.engine.project())?,
                )?;
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
        }
        Ok(())
    }

    fn output_smoke_frame_current(&mut self) -> Result<bool> {
        let expected = program::ProgramRenderer::default().render(&self.engine, &self.input_fx)?;
        Ok(self
            .program_pixels
            .as_ref()
            .is_some_and(|actual| actual.as_ref() == expected.as_ref()))
    }

    fn save_output_capture(&self, name: &str, black: bool) -> Result<bool> {
        let Some(capture) = self.program_output.take_capture() else {
            return Ok(false);
        };
        if capture.viewport != program_output::viewport_id() || !self.program_output.native_seen() {
            return Err(Error::Invalid(
                "Program screenshot came from the wrong viewport".into(),
            ));
        }
        validate_output_capture(&capture.image, &self.last_program_rgba, black)?;
        let bytes: Vec<u8> = capture
            .image
            .pixels
            .iter()
            .flat_map(|pixel| pixel.to_array())
            .collect();
        image::save_buffer(
            format!("artifacts/{name}.png"),
            &bytes,
            capture.image.width() as u32,
            capture.image.height() as u32,
            image::ColorType::Rgba8,
        )
        .map_err(|error| Error::Invalid(error.to_string()))?;
        std::fs::write(
            format!("artifacts/{name}.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "viewport": format!("{:?}", capture.viewport),
                "native": true, "program": self.engine.project().program.0,
                "preview": self.engine.project().preview.0, "width": capture.image.width(),
                "height": capture.image.height(), "program_pixel_samples_verified": true,
                "fullscreen_changes_main_window": self.fullscreen,
            }))?,
        )?;
        Ok(true)
    }
}

fn validate_output_capture(image: &egui::ColorImage, program: &[u8], black: bool) -> Result<()> {
    if image.width() < 320 || image.height() < 180 || program.len() != 640 * 360 * 4 {
        return Err(Error::Invalid(
            "Program capture dimensions/buffer are invalid".into(),
        ));
    }
    if black {
        if image
            .pixels
            .iter()
            .any(|pixel| pixel.r() > 2 || pixel.g() > 2 || pixel.b() > 2)
        {
            return Err(Error::Invalid(
                "FTB did not reach the separate Program framebuffer".into(),
            ));
        }
        return Ok(());
    }
    let bounds = Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(image.width() as f32, image.height() as f32),
    );
    let raster = program_output::fit_program(bounds);
    for y in [0.2_f32, 0.4, 0.6] {
        for x in [0.075_f32, 0.225, 0.425, 0.675, 0.875] {
            let col = (raster.left() + raster.width() * x) as usize;
            let row = (raster.top() + raster.height() * y) as usize;
            let actual = row
                .checked_mul(image.width())
                .and_then(|offset| offset.checked_add(col))
                .and_then(|index| image.pixels.get(index))
                .ok_or_else(|| Error::Invalid("Program screenshot sample outside buffer".into()))?;
            let offset = ((y * 360.0) as usize * 640 + (x * 640.0) as usize) * 4;
            let expected = program
                .get(offset..offset + 3)
                .ok_or_else(|| Error::Invalid("Program source sample outside buffer".into()))?;
            if actual
                .to_array()
                .iter()
                .take(3)
                .zip(expected)
                .any(|(a, b)| a.abs_diff(*b) > 8)
            {
                return Err(Error::Invalid(format!(
                    "Separate Program viewport pixel differs at {x}/{y}"
                )));
            }
        }
    }
    Ok(())
}
