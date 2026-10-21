//! Opt-in hosted native acceptance for the local MP4 video source. Requires the
//! original generated fixture (`RUSTMIX_TEST_VIDEO`) and a real Windows Media
//! Foundation decode; it never opens an audio device, camera or network.
use super::*;

const VIDEO_INPUT: InputId = InputId(8);
const PROGRAM_WIDTH: usize = 640;
const PROGRAM_HEIGHT: usize = 360;
const QUADRANT_TOLERANCE: u8 = 40;

impl RustMixApp {
    pub(crate) fn smoke_video(&mut self, ctx: &egui::Context, s: &mut smoke::Smoke) -> Result<()> {
        if self.start.elapsed() > Duration::from_secs(360) {
            return Err(Error::Invalid(format!(
                "Native video workflow timed out at stage {}",
                s.stage
            )));
        }
        if s.wait > 240 {
            return Err(Error::Invalid(format!(
                "Native video workflow stage {} did not advance after 240 frames",
                s.stage
            )));
        }
        let reopen = std::env::args().any(|arg| arg == "--ui-video-smoke-reopen");
        for event in ctx.input(|input| input.events.clone()) {
            // Only this window's own readback may advance a stage. A Program
            // output viewport or a delayed foreign capture must not turn a
            // passing workflow into an "unexpected screenshot" failure.
            if let egui::Event::Screenshot {
                viewport_id, image, ..
            } = event
                && viewport_id == egui::ViewportId::ROOT
            {
                let name = match s.stage {
                    8 => "video-source",
                    12 => "video-program",
                    16 => "video-resumed",
                    _ => {
                        return Err(Error::Invalid(format!(
                            "Unexpected screenshot stage in native video workflow: {}",
                            s.stage
                        )));
                    }
                };
                save_png(name, &image)?;
                s.captured = true;
            }
        }
        if s.wait < 5 || self.job.is_some() {
            return Ok(());
        }
        if reopen {
            assert_video_project(self.engine.project())?;
            if self.video_players.len() != 1 {
                return Err(Error::Invalid(format!(
                    "Reopened video project must start exactly one owner worker, found {}",
                    self.video_players.len()
                )));
            }
            if !self.video_frame_ready() || !program_quadrants(&self.last_program_rgba) {
                return Ok(());
            }
            std::fs::write(
                "artifacts/video-reopen-ok.json",
                serde_json::to_vec_pretty(self.engine.project())?,
            )?;
            self.saved_revision = self.engine.project().revision;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return Ok(());
        }
        match s.stage {
            0 => {
                std::fs::create_dir_all("artifacts")?;
                let fixture = video_fixture()?;
                self.add_number = 8;
                self.basic_mode = false;
                self.execute(Command::SetInputsPaused(false));
                self.execute(Command::SelectPreview(InputId(2)));
                self.native_click(s, "production.add.menu")?;
                s.scratch_text = Some(fixture);
                s.stage = 1;
            }
            1 => {
                self.native_click(s, "production.add.category.video")?;
                s.stage = 2;
            }
            2 => {
                self.native_click(s, "dialog.add.video.name")?;
                s.stage = 3;
            }
            3 => {
                s.events = vec![egui::Event::Text("Fixture video".into())];
                s.stage = 4;
            }
            4 => {
                self.native_click(s, "dialog.add.video.path")?;
                s.stage = 5;
            }
            5 => {
                let fixture = s
                    .scratch_text
                    .clone()
                    .ok_or_else(|| Error::Invalid("Video fixture path was not retained".into()))?;
                s.events = vec![egui::Event::Text(fixture)];
                s.stage = 6;
            }
            6 => {
                self.native_click(s, "dialog.add.video.validate")?;
                s.stage = 7;
            }
            7 => {
                if self.video_probe_pending() {
                    return Ok(());
                }
                if let Some(error) = self.video_probe_error() {
                    return Err(Error::Invalid(format!(
                        "Native MP4 validation failed: {error}"
                    )));
                }
                let format = self.video_probe_format().ok_or_else(|| {
                    Error::Invalid("Native MP4 probe produced no metadata".into())
                })?;
                // The platform acceptance checks the ratio, because Media
                // Foundation may report an unreduced 60/2 for the same 30 fps
                // stream; only the raster is exact.
                if format.0 != 160
                    || format.1 != 90
                    || format.3 == 0
                    || u64::from(format.2) != 30 * u64::from(format.3)
                {
                    return Err(Error::Invalid(format!(
                        "Native MP4 probe reported unexpected format {format:?}"
                    )));
                }
                self.validate_dialog_geometry(ctx, "dialog.add")?;
                std::fs::write(
                    "artifacts/video-validation.json",
                    serde_json::to_vec_pretty(&serde_json::json!({
                        "width": format.0, "height": format.1,
                        "frame_rate": format!("{}/{}", format.2, format.3),
                        "decoder": "Windows Media Foundation RGB32 (platform worker)",
                        "audio_decoded": false, "device_opened": false,
                        "evidence_scope": "native descriptor and decoder negotiation; playback pixels follow",
                    }))?,
                )?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 8;
            }
            8 if s.captured => {
                self.native_click(s, "dialog.add.confirm")?;
                s.stage = 9;
            }
            9 => {
                let Some(fixture) = s.scratch_text.clone() else {
                    return Err(Error::Invalid("Video fixture path was not retained".into()));
                };
                let input = self.engine.project().input(VIDEO_INPUT).cloned();
                if self.add_open
                    || self.engine.project().inputs.len() != 8
                    || input.as_ref().is_none_or(|input| {
                        input.name != "Fixture video"
                            || input.kind != InputKind::Color
                            || input.config.video_source.as_ref()
                                != Some(&VideoSource::File { path: fixture })
                            || input.config.media_path.is_some()
                            || input.config.audio_source.is_some()
                    })
                {
                    return Err(Error::Invalid(format!(
                        "Native video input add failed: {}",
                        self.notice
                    )));
                }
                self.execute(Command::SelectPreview(VIDEO_INPUT));
                self.native_click(s, "switcher.cut")?;
                s.stage = 10;
            }
            10 => {
                if self.engine.project().program != VIDEO_INPUT {
                    return Err(Error::Invalid(
                        "Taking the video Preview did not publish it to Program".into(),
                    ));
                }
                if !self.video_frame_ready() {
                    return Ok(());
                }
                s.stage = 11;
            }
            11 => {
                if !program_quadrants(&self.last_program_rgba) {
                    return Ok(());
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 12;
            }
            12 if s.captured => {
                self.execute(Command::SetInputPaused(VIDEO_INPUT, true));
                s.stage = 13;
            }
            13 => {
                // Allow the owner worker to observe the pause and settle on the
                // last accepted frame before the frozen-picture assertion.
                if s.wait < 20 {
                    return Ok(());
                }
                if !program_quadrants(&self.last_program_rgba) {
                    return Err(Error::Invalid(
                        "Paused video does not retain its last valid frame".into(),
                    ));
                }
                s.scratch_pixels = Some(self.last_program_rgba.clone());
                s.stage = 14;
            }
            14 => {
                if s.wait < 25 {
                    return Ok(());
                }
                let baseline = s
                    .scratch_pixels
                    .as_deref()
                    .ok_or_else(|| Error::Invalid("Paused frame baseline is missing".into()))?;
                if baseline != self.last_program_rgba.as_slice() {
                    return Err(Error::Invalid(
                        "Paused local video did not freeze its Program picture".into(),
                    ));
                }
                self.execute(Command::SetInputPaused(VIDEO_INPUT, false));
                s.stage = 15;
            }
            15 => {
                let baseline = s
                    .scratch_pixels
                    .as_deref()
                    .ok_or_else(|| Error::Invalid("Paused frame baseline is missing".into()))?;
                let changed = differing_pixels(baseline, &self.last_program_rgba);
                if changed == 0 {
                    return Ok(());
                }
                std::fs::write(
                    "artifacts/video-motion.json",
                    serde_json::to_vec_pretty(&serde_json::json!({
                        "paused_frame_frozen": true,
                        "resumed_frame_differs": true,
                        "changed_channels_after_resume": changed,
                        "transport": "engine SetInputPaused only; decoder loop is internal",
                    }))?,
                )?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 16;
            }
            16 if s.captured => {
                s.stage = 17;
            }
            17 => {
                if self.scripted_save("artifacts/video.rustmix.json", ctx)? {
                    s.stage = 18;
                }
            }
            18 => {
                if !Path::new("artifacts/video.rustmix.json").exists()
                    || self.saved_revision != self.engine.project().revision
                {
                    return Err(Error::Invalid(format!(
                        "Native video workflow save did not acknowledge the current revision: {}",
                        self.notice
                    )));
                }
                assert_video_project(self.engine.project())?;
                std::fs::write(
                    "artifacts/video-workflow-ok.json",
                    serde_json::to_vec_pretty(self.engine.project())?,
                )?;
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
        }
        Ok(())
    }

    fn video_frame_ready(&self) -> bool {
        self.input_fx
            .get(&VIDEO_INPUT)
            .is_some_and(|fx| fx.custom_rgba.is_some())
    }
}

fn video_fixture() -> Result<String> {
    let path = std::env::var_os("RUSTMIX_TEST_VIDEO").ok_or_else(|| {
        Error::Invalid(
            "Windows video acceptance requires scripts/ci-video-fixture.ps1 and RUSTMIX_TEST_VIDEO; a missing fixture is a failure".into(),
        )
    })?;
    let path = PathBuf::from(path);
    if !path.is_absolute() || !path.is_file() {
        return Err(Error::Invalid(
            "Generated MP4 fixture path must be absolute and refer to an existing file".into(),
        ));
    }
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::Invalid("Generated MP4 fixture path is not valid text".into()))
}

fn save_png(name: &str, image: &egui::ColorImage) -> Result<()> {
    let bytes: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|pixel| pixel.to_array())
        .collect();
    image::save_buffer(
        format!("artifacts/{name}.png"),
        &bytes,
        image.width() as u32,
        image.height() as u32,
        image::ColorType::Rgba8,
    )
    .map_err(|error| Error::Invalid(error.to_string()))?;
    std::fs::write(
        format!("artifacts/{name}.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "width": image.width(), "height": image.height(),
            "screenshot": name,
            "viewport": "operator-root",
            "note": "Operator-window capture; decoded-video pixel claims are in video-validation.json and video-motion.json",
        }))?,
    )?;
    Ok(())
}

fn sample(pixels: &[u8], x: usize, y: usize) -> Option<[u8; 3]> {
    let index = y.checked_mul(PROGRAM_WIDTH)?.checked_add(x)?;
    let pixel = pixels.chunks_exact(4).nth(index)?;
    Some([*pixel.first()?, *pixel.get(1)?, *pixel.get(2)?])
}

fn near(actual: [u8; 3], expected: [u8; 3]) -> bool {
    actual
        .iter()
        .zip(expected)
        .all(|(a, b)| a.abs_diff(b) <= QUADRANT_TOLERANCE)
}

/// The generated fixture has red/green/blue/white quadrants and a moving black
/// square. Matching all four quadrant centres rejects a frozen, inverted or
/// substituted picture while tolerating limited-range H.264 decode error.
pub(crate) fn program_quadrants(pixels: &[u8]) -> bool {
    if pixels.len() != PROGRAM_WIDTH * PROGRAM_HEIGHT * 4 {
        return false;
    }
    let samples = [
        (PROGRAM_WIDTH / 4, PROGRAM_HEIGHT / 4, [255, 0, 0]),
        (PROGRAM_WIDTH * 3 / 4, PROGRAM_HEIGHT / 4, [0, 255, 0]),
        (PROGRAM_WIDTH / 4, PROGRAM_HEIGHT * 3 / 4, [0, 0, 255]),
        (
            PROGRAM_WIDTH * 3 / 4,
            PROGRAM_HEIGHT * 3 / 4,
            [255, 255, 255],
        ),
    ];
    samples
        .into_iter()
        .all(|(x, y, expected)| sample(pixels, x, y).is_some_and(|actual| near(actual, expected)))
}

fn differing_pixels(left: &[u8], right: &[u8]) -> usize {
    left.iter().zip(right).filter(|(a, b)| a != b).count()
}

fn assert_video_project(project: &ProjectState) -> Result<()> {
    let input = project
        .input(VIDEO_INPUT)
        .ok_or_else(|| Error::Invalid("Persisted video input missing".into()))?;
    if project.schema != SCHEMA
        || project.inputs.len() != 8
        || project.program != VIDEO_INPUT
        || input.name != "Fixture video"
        || input.kind != InputKind::Color
        || input.config.media_path.is_some()
        || input.config.audio_source.is_some()
        || !matches!(input.config.video_source, Some(VideoSource::File { .. }))
    {
        return Err(Error::Invalid(
            "Video project descriptor, source type or Program routing did not persist".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]
    use super::*;

    fn frame_with(colours: [[u8; 3]; 4]) -> Vec<u8> {
        let mut pixels = vec![0_u8; PROGRAM_WIDTH * PROGRAM_HEIGHT * 4];
        for (y, row) in pixels.chunks_exact_mut(PROGRAM_WIDTH * 4).enumerate() {
            for (x, pixel) in row.chunks_exact_mut(4).enumerate() {
                let quadrant =
                    usize::from(x >= PROGRAM_WIDTH / 2) + usize::from(y >= PROGRAM_HEIGHT / 2) * 2;
                let colour = colours[quadrant];
                pixel[0] = colour[0];
                pixel[1] = colour[1];
                pixel[2] = colour[2];
                pixel[3] = 255;
            }
        }
        pixels
    }

    #[test]
    fn generated_quadrant_layout_is_required_for_video_picture_acceptance() {
        let good = frame_with([[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]]);
        assert!(program_quadrants(&good));
        assert!(!program_quadrants(&frame_with([
            [0, 0, 255],
            [255, 0, 0],
            [0, 255, 0],
            [255, 255, 255]
        ])));
        assert!(!program_quadrants(&frame_with([[0, 0, 0]; 4])));
        assert!(!program_quadrants(&good[..good.len() - 4]));
        assert!(!program_quadrants(&[]));
    }

    #[test]
    fn motion_evidence_counts_changed_channels_and_ignores_identical_frames() {
        let a = frame_with([[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]]);
        let b = frame_with([[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 240]]);
        assert_eq!(differing_pixels(&a, &a), 0);
        assert!(differing_pixels(&a, &b) > 0);
        assert_eq!(differing_pixels(&a, &[]), 0);
    }

    #[test]
    fn video_reopen_expectations_reject_audio_or_png_substitution() {
        let mut engine = Engine::default();
        engine
            .execute(
                Command::AddInputAt {
                    name: "Fixture video".into(),
                    kind: InputKind::Color,
                    config: InputConfig {
                        video_source: Some(VideoSource::File {
                            path: "fixture.mp4".into(),
                        }),
                        ..InputConfig::default()
                    },
                    index: 7,
                },
                0,
            )
            .unwrap();
        engine
            .execute(Command::SelectPreview(VIDEO_INPUT), 0)
            .unwrap();
        engine.execute(Command::Cut, 0).unwrap();
        assert_video_project(engine.project()).unwrap();
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("video.rustmix.json");
        persistence::save(&path, engine.project()).unwrap();
        let reopened = persistence::load(&path).unwrap();
        assert_video_project(&reopened).unwrap();
        assert_eq!(reopened, *engine.project());
        let mut invalid = reopened.clone();
        invalid.program = InputId(1);
        assert!(assert_video_project(&invalid).is_err());
        let mut substituted = reopened.clone();
        substituted.inputs[7].config.video_source = None;
        substituted.inputs[7].config.media_path = Some("still.png".into());
        assert!(assert_video_project(&substituted).is_err());
    }
}
