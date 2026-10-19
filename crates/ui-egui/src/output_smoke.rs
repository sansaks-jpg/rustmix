//! Opt-in hosted native acceptance for the separate Program viewport.
use super::*;

impl RustMixApp {
    pub(crate) fn smoke_program_output(
        &mut self,
        ctx: &egui::Context,
        s: &mut smoke::Smoke,
    ) -> Result<()> {
        if self.start.elapsed() > Duration::from_secs(360) {
            return Err(Error::Invalid(format!(
                "Program output smoke timed out at {}",
                s.stage
            )));
        }
        // A stage that never advances is a hang, not a slow capture. Bound it
        // separately so the failure names the stuck stage and stays far below
        // the external process limit.
        if s.wait > 240 {
            return Err(Error::Invalid(format!(
                "Program output smoke stage {} did not advance after 240 frames",
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
                // The Program menu is only available on the advanced surface;
                // a previous process may have persisted Basic mode.
                self.basic_mode = false;
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
                if !self.wait_output_capture(ctx, "program-output-window", false, s)? {
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
                if !self.wait_output_capture(ctx, "program-output-black", true, s)? {
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
                // Deterministic caption text so the native capture can prove the
                // rasterized glyphs reached the separate Program framebuffer.
                overlay.title = "PROGRAM LIVE".into();
                overlay.subtitle = "CAPTION CHECK".into();
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
                if !self.wait_output_capture(ctx, "program-output-live", false, s)? {
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

    /// A screenshot request or readback can be lost while the child window
    /// transitions between fullscreen states, so re-request on a bounded frame
    /// cadence. Consume first: a capture harvested this frame must not be
    /// discarded by the retry, and `take_capture` only accepts the current
    /// request sequence.
    fn wait_output_capture(
        &self,
        ctx: &egui::Context,
        name: &str,
        black: bool,
        s: &smoke::Smoke,
    ) -> Result<bool> {
        if self.save_output_capture(name, black)? {
            return Ok(true);
        }
        if s.wait > 0 && s.wait.is_multiple_of(20) {
            self.program_output.request_capture(ctx);
        }
        Ok(false)
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
        // The live capture is the one stage with an enabled overlay caption, so
        // it is also the native proof that rasterized caption ink reaches the
        // separate Program framebuffer rather than only the operator monitor.
        let caption = if name == "program-output-live" {
            let ink = validate_caption_ink(
                &capture.image,
                &self.last_program_rgba,
                &self.engine.project().overlays.standard,
            )?;
            serde_json::json!({
                "caption_title_ink_pixels": ink.title_ink,
                "caption_title_bright_pixels": ink.title_bright,
                "caption_subtitle_ink_pixels": ink.subtitle_ink,
                "caption_subtitle_bright_pixels": ink.subtitle_bright,
            })
        } else {
            serde_json::json!({})
        };
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
        let mut evidence = serde_json::json!({
            "viewport": format!("{:?}", capture.viewport),
            "native": true, "program": self.engine.project().program.0,
            "preview": self.engine.project().preview.0, "width": capture.image.width(),
            "height": capture.image.height(), "program_pixel_samples_verified": true,
            "fullscreen_changes_main_window": self.fullscreen,
        });
        if let Some(object) = evidence.as_object_mut()
            && let Some(extra) = caption.as_object()
        {
            for (key, value) in extra {
                object.insert(key.clone(), value.clone());
            }
        }
        std::fs::write(
            format!("artifacts/{name}.json"),
            serde_json::to_vec_pretty(&evidence)?,
        )?;
        Ok(true)
    }
}

/// Caption ink counts measured in a Program viewport capture.
struct CaptionInk {
    title_ink: usize,
    title_bright: usize,
    subtitle_ink: usize,
    subtitle_bright: usize,
}

/// Proves the rasterized overlay captions reached the separate Program
/// framebuffer: ink pixels of the Program raster inside the enabled panel must
/// reappear bright in the capture. The panel region is darkened by the overlay
/// fill, so a bright pixel there can only come from caption ink.
fn validate_caption_ink(
    image: &egui::ColorImage,
    program: &[u8],
    overlays: &[OverlayState],
) -> Result<CaptionInk> {
    let mut ink = CaptionInk {
        title_ink: 0,
        title_bright: 0,
        subtitle_ink: 0,
        subtitle_bright: 0,
    };
    let Some((slot, overlay)) = overlays.iter().enumerate().find(|(_, o)| o.enabled) else {
        return Ok(ink);
    };
    let (w, h) = (640i64, 360i64);
    let frame = Rect::from_min_size(Pos2::ZERO, Vec2::new(w as f32, h as f32));
    let panel = overlay_panel_rect(frame, overlay.position, slot);
    let x0 = panel.left().floor().max(0.0) as i64;
    let y0 = panel.top().floor().max(0.0) as i64;
    let x1 = panel.right().ceil().min(w as f32) as i64;
    let y1 = panel.bottom().ceil().min(h as f32) as i64;
    if x1 <= x0 || y1 <= y0 {
        return Err(Error::Invalid(
            "Enabled overlay panel is outside the Program raster".into(),
        ));
    }
    let bounds = Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(image.width() as f32, image.height() as f32),
    );
    let raster = program_output::fit_program(bounds);
    for py in y0..y1 {
        for px in x0..x1 {
            let offset = ((py as usize * w as usize) + px as usize) * 4;
            let Some(source) = program.get(offset..offset + 4) else {
                continue;
            };
            let Ok(source) = <[u8; 4]>::try_from(source) else {
                continue;
            };
            let title = source == [255, 255, 255, 255];
            let subtitle = source == [190, 190, 190, 255];
            if !title && !subtitle {
                continue;
            }
            let ix = (raster.left() + raster.width() * (px as f32 + 0.5) / w as f32).round() as i64;
            let iy = (raster.top() + raster.height() * (py as f32 + 0.5) / h as f32).round() as i64;
            if ix < 0 || iy < 0 || ix >= image.width() as i64 || iy >= image.height() as i64 {
                continue;
            }
            let index = iy as usize * image.width() + ix as usize;
            let Some(pixel) = image.pixels.get(index) else {
                continue;
            };
            let luma = (u32::from(pixel.r()) * 299
                + u32::from(pixel.g()) * 587
                + u32::from(pixel.b()) * 114)
                / 1000;
            if title {
                ink.title_ink += 1;
                if luma >= 180 {
                    ink.title_bright += 1;
                }
            } else {
                ink.subtitle_ink += 1;
                if luma >= 120 {
                    ink.subtitle_bright += 1;
                }
            }
        }
    }
    if ink.title_ink < 20 || ink.title_bright * 100 < ink.title_ink * 60 {
        return Err(Error::Invalid(format!(
            "Overlay title ink did not reach the Program framebuffer: {}/{} bright",
            ink.title_bright, ink.title_ink
        )));
    }
    if ink.subtitle_ink < 8 || ink.subtitle_bright * 100 < ink.subtitle_ink * 60 {
        return Err(Error::Invalid(format!(
            "Overlay subtitle ink did not reach the Program framebuffer: {}/{} bright",
            ink.subtitle_bright, ink.subtitle_ink
        )));
    }
    Ok(ink)
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
