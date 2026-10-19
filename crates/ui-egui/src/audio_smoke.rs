//! Hosted native descriptor workflow. It intentionally never starts an audio
//! session or opens a microphone/playback device. This is not WASAPI acceptance.
use super::*;

const FIXTURE_PATH: &str = "artifacts/audio-fixture.wav";
const SOURCE_LABEL: &str = "Acceptance tone";
const FIXTURE_RATE: u32 = 44_100;
const FIXTURE_FRAMES: usize = 44_100;

impl RustMixApp {
    pub(crate) fn smoke_audio(&mut self, ctx: &egui::Context, s: &mut smoke::Smoke) -> Result<()> {
        if self.start.elapsed() > Duration::from_secs(360) {
            return Err(Error::Invalid(format!(
                "Native audio descriptor workflow timed out at stage {}",
                s.stage
            )));
        }
        if s.wait > 240 {
            return Err(Error::Invalid(format!(
                "Native audio workflow stage {} did not advance after 240 frames",
                s.stage
            )));
        }
        // An accidental Start is a failure, not an excuse to claim device proof.
        if self.audio_ui.is_active() || self.audio_ui.monitoring_active() {
            self.stop_audio_for_project_change();
            return Err(Error::Invalid(
                "Audio descriptor smoke unexpectedly opened an audio session".into(),
            ));
        }
        let reopen = std::env::args().any(|argument| argument == "--ui-audio-smoke-reopen");
        for event in ctx.input(|input| input.events.clone()) {
            if let egui::Event::Screenshot {
                viewport_id, image, ..
            } = event
                && viewport_id == egui::ViewportId::ROOT
            {
                let name = match s.stage {
                    8 => "audio-input",
                    12 => "audio-io",
                    15 => "audio-source",
                    _ => {
                        return Err(Error::Invalid(
                            "Unexpected screenshot stage in native audio workflow".into(),
                        ));
                    }
                };
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
                s.captured = true;
            }
        }
        if s.wait < 5 || self.job.is_some() {
            return Ok(());
        }
        if reopen {
            assert_audio_project(self.engine.project())?;
            if self.audio_ui.status_label() != "off" || self.audio_ui.open {
                return Err(Error::Invalid(
                    "Reopening an audio project automatically armed monitoring".into(),
                ));
            }
            if !audio_program_black(&self.last_program_rgba) {
                return Ok(());
            }
            std::fs::write(
                "artifacts/audio-workflow-reopen-ok.json",
                serde_json::to_vec_pretty(self.engine.project())?,
            )?;
            self.saved_revision = self.engine.project().revision;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return Ok(());
        }
        if s.stage <= 13
            && (self.engine.project().program != InputId(1)
                || self.engine.project().preview != InputId(2))
        {
            return Err(Error::Invalid(
                "Audio source configuration changed Program/Preview before explicit Take".into(),
            ));
        }
        match s.stage {
            0 => {
                std::fs::create_dir_all("artifacts")?;
                let bytes = audio_fixture()?;
                let audio = wav::WavAudio::decode(&bytes)?;
                if audio.sample_rate() != FIXTURE_RATE || audio.frames().len() != FIXTURE_FRAMES {
                    return Err(Error::Invalid(
                        "Generated WAV fixture has incorrect duration or rate".into(),
                    ));
                }
                std::fs::write(FIXTURE_PATH, bytes)?;
                self.basic_mode = false;
                self.execute(Command::SetInputsPaused(true));
                self.native_click(s, "production.add")?;
                s.stage = 1;
            }
            1 => {
                self.native_click(s, "dialog.add.audio.wav")?;
                s.stage = 2;
            }
            2 => {
                self.native_click(s, "dialog.add.audio.name")?;
                s.stage = 3;
            }
            3 => {
                s.events = vec![egui::Event::Text(SOURCE_LABEL.into())];
                s.stage = 4;
            }
            4 => {
                self.native_click(s, "dialog.add.audio.path")?;
                s.stage = 5;
            }
            5 => {
                s.events = vec![egui::Event::Text(FIXTURE_PATH.into())];
                s.stage = 6;
            }
            6 => {
                self.native_click(s, "dialog.add.audio.validate")?;
                s.stage = 7;
            }
            7 => {
                if self.audio_ui.wav_validation_pending() {
                    return Ok(());
                }
                if let Some(error) = self.audio_ui.wav_validation_error() {
                    return Err(Error::Invalid(format!(
                        "Native WAV validation failed: {error}"
                    )));
                }
                if self.audio_ui.validated_wav_metadata() != Some((FIXTURE_RATE, FIXTURE_FRAMES)) {
                    return Err(Error::Invalid(
                        "Native Add Input did not validate the actual one-second 44.1 kHz WAV"
                            .into(),
                    ));
                }
                self.validate_dialog_geometry(ctx, "dialog.add")?;
                write_audio_elements("audio-input", &self.automation)?;
                std::fs::write(
                    "artifacts/audio-validation.json",
                    serde_json::to_vec_pretty(&serde_json::json!({
                        "sample_rate": FIXTURE_RATE, "frames": FIXTURE_FRAMES, "duration_seconds": 1,
                        "format": "generated mono PCM16 RIFF/WAVE", "audio_monitoring_started": false,
                        "evidence_scope": "native input descriptor and metadata workflow; no device/callback acceptance",
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
                let added = self.engine.project().input(InputId(8));
                if self.add_open
                    || self.engine.project().inputs.len() != 8
                    || added.is_none_or(|input| {
                        input.name != SOURCE_LABEL
                            || input.config.audio_source
                                != Some(AudioSource::WavFile {
                                    path: FIXTURE_PATH.into(),
                                })
                            || input.config.media_path.is_some()
                            || input.kind != InputKind::Color
                    })
                {
                    return Err(Error::Invalid(format!(
                        "Native Audio Add failed: {}",
                        self.notice
                    )));
                }
                self.native_click(s, "toolbar.settings.menu")?;
                s.stage = 10;
            }
            10 => {
                self.native_click(s, "toolbar.audioIO")?;
                s.stage = 11;
            }
            11 => {
                if !self.audio_ui.open || self.audio_ui.status_label() != "off" {
                    return Err(Error::Invalid(
                        "Audio I/O did not open in the safe devices-closed state".into(),
                    ));
                }
                assert_audio_io_geometry(ctx.viewport_rect(), &self.automation)?;
                write_audio_elements("audio-io", &self.automation)?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 12;
            }
            12 if s.captured => {
                self.native_click(s, "dialog.audio_io.close")?;
                s.stage = 13;
            }
            13 => {
                if self.audio_ui.open {
                    return Err(Error::Invalid(
                        "Audio I/O Close left the dialog open".into(),
                    ));
                }
                // Selection is isolated; the following real Cut control is the
                // only action that publishes the audio-only input to Program.
                self.execute(Command::SelectPreview(InputId(8)));
                if self.engine.project().program != InputId(1) {
                    return Err(Error::Invalid(
                        "Selecting an audio Preview changed Program".into(),
                    ));
                }
                self.native_click(s, "switcher.cut")?;
                s.stage = 14;
            }
            14 => {
                assert_audio_project(self.engine.project())?;
                if !audio_program_black(&self.last_program_rgba) {
                    return Ok(());
                }
                if self.engine.black_level() != 0.0 {
                    return Err(Error::Invalid(
                        "Audio slate test was hidden by Fade to Black".into(),
                    ));
                }
                write_audio_elements("audio-source", &self.automation)?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 15;
            }
            15 if s.captured => {
                if self.scripted_save("artifacts/audio.rustmix.json", ctx)? {
                    s.stage = 16;
                }
            }
            16 => {
                assert_audio_project(self.engine.project())?;
                if self.saved_revision != self.engine.project().revision
                    || !Path::new("artifacts/audio.rustmix.json").exists()
                {
                    return Err(Error::Invalid(format!(
                        "Native audio project save did not acknowledge the current revision: {}",
                        self.notice
                    )));
                }
                std::fs::write(
                    "artifacts/audio-workflow-ok.json",
                    serde_json::to_vec_pretty(self.engine.project())?,
                )?;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
        }
        Ok(())
    }
}

fn assert_audio_project(project: &ProjectState) -> Result<()> {
    let input = project
        .input(InputId(8))
        .ok_or_else(|| Error::Invalid("Persisted audio input missing".into()))?;
    if project.schema != SCHEMA
        || project.inputs.len() != 8
        || project.program != InputId(8)
        || project.preview != InputId(1)
        || input.name != SOURCE_LABEL
        || input.kind != InputKind::Color
        || input.config.media_path.is_some()
        || input.config.audio_source
            != Some(AudioSource::WavFile {
                path: FIXTURE_PATH.into(),
            })
    {
        return Err(Error::Invalid(
            "Audio project descriptor, source type or Program/Preview did not persist".into(),
        ));
    }
    Ok(())
}

fn audio_program_black(pixels: &[u8]) -> bool {
    pixels.len() == 640 * 360 * 4
        && pixels.chunks_exact(4).all(|pixel| {
            pixel.first() == Some(&0)
                && pixel.get(1) == Some(&0)
                && pixel.get(2) == Some(&0)
                && pixel.get(3) == Some(&255)
        })
}

fn write_audio_elements(name: &str, elements: &HashMap<String, Rect>) -> Result<()> {
    let relevant = elements
        .iter()
        .filter(|(id, _)| id.starts_with("dialog.") || id.starts_with("monitor."))
        .map(|(id, rect)| (id, format!("{rect:?}")))
        .collect::<HashMap<_, _>>();
    std::fs::write(
        format!("artifacts/{name}-elements.json"),
        serde_json::to_vec_pretty(&relevant)?,
    )?;
    Ok(())
}

fn assert_audio_io_geometry(bounds: Rect, elements: &HashMap<String, Rect>) -> Result<()> {
    let body = elements
        .get("dialog.audio_io.body")
        .ok_or_else(|| Error::Invalid("Audio I/O body missing".into()))?;
    for id in [
        "dialog.audio_io.start",
        "dialog.audio_io.stop",
        "dialog.audio_io.close",
    ] {
        let footer = elements
            .get(id)
            .ok_or_else(|| Error::Invalid(format!("Audio I/O control missing: {id}")))?;
        if !footer.is_finite()
            || !bounds.contains_rect(*footer)
            || footer.height() < 27.0
            || body.bottom() > footer.top() + 1.0
        {
            return Err(Error::Invalid(format!("Audio I/O footer is clipped: {id}")));
        }
    }
    Ok(())
}

fn audio_fixture() -> Result<Vec<u8>> {
    let data_bytes = u32::try_from(FIXTURE_FRAMES)
        .ok()
        .and_then(|frames| frames.checked_mul(2))
        .ok_or_else(|| Error::Invalid("Audio fixture length overflow".into()))?;
    let riff_bytes = data_bytes
        .checked_add(36)
        .ok_or_else(|| Error::Invalid("Audio fixture RIFF overflow".into()))?;
    let capacity = usize::try_from(data_bytes)
        .ok()
        .and_then(|length| length.checked_add(44))
        .ok_or_else(|| Error::Invalid("Audio fixture allocation overflow".into()))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| Error::Invalid("Audio fixture allocation failed".into()))?;
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_bytes.to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&FIXTURE_RATE.to_le_bytes());
    bytes.extend_from_slice(
        &FIXTURE_RATE
            .checked_mul(2)
            .ok_or_else(|| Error::Invalid("Audio fixture byte rate overflow".into()))?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for frame in 0..FIXTURE_FRAMES {
        let phase = frame as f64 * 440.0 * std::f64::consts::TAU / f64::from(FIXTURE_RATE);
        let sample = (phase.sin() * 4000.0).round() as i16;
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn native_audio_fixture_contains_one_second_of_real_mono_pcm16() {
        let bytes = audio_fixture().unwrap();
        let audio = wav::WavAudio::decode(&bytes).unwrap();
        assert_eq!(audio.sample_rate(), 44_100);
        assert_eq!(audio.frames().len(), 44_100);
        assert!(
            audio
                .frames()
                .iter()
                .all(|frame| frame.first() == frame.get(1))
        );
        assert!(audio.frames().iter().flatten().any(|sample| *sample > 0.1));
        assert!(audio.frames().iter().flatten().any(|sample| *sample < -0.1));
        assert!(
            audio
                .frames()
                .iter()
                .flatten()
                .all(|sample| sample.is_finite())
        );
    }

    #[test]
    fn native_audio_reopen_expectation_round_trips_descriptor_and_black_slate() {
        let mut engine = Engine::default();
        engine
            .execute(
                Command::AddInputAt {
                    name: SOURCE_LABEL.into(),
                    kind: InputKind::Color,
                    config: InputConfig {
                        audio_source: Some(AudioSource::WavFile {
                            path: FIXTURE_PATH.into(),
                        }),
                        ..InputConfig::default()
                    },
                    index: 7,
                },
                0,
            )
            .unwrap();
        engine
            .execute(Command::SelectPreview(InputId(8)), 0)
            .unwrap();
        assert_eq!(engine.project().program, InputId(1));
        engine.execute(Command::Cut, 0).unwrap();
        assert_audio_project(engine.project()).unwrap();
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("audio.rustmix.json");
        persistence::save(&path, engine.project()).unwrap();
        let reopened = persistence::load(&path).unwrap();
        assert_audio_project(&reopened).unwrap();
        assert_eq!(reopened, *engine.project());
        let mut invalid = reopened.clone();
        invalid.program = InputId(1);
        assert!(assert_audio_project(&invalid).is_err());
        assert!(audio_program_black(&[0, 0, 0, 255].repeat(640 * 360)));
        assert!(!audio_program_black(&[30, 100, 20, 255].repeat(640 * 360)));
        assert!(!audio_program_black(&[0, 0, 0, 255]));
    }
}
