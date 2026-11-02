//! Drag-and-drop intake for media files. A dropped file runs the same bounded
//! validation as the Add Input dialog (video probe, WAV decode, image decode) on a
//! worker thread; only a validated descriptor is committed as an input. The
//! supported set matches the engine exactly — MP4/M4V/MOV/AVI/WMV video, WAV audio, PNG/JPEG/BMP images
//! — so a drop can never produce the rejected half-configured source the
//! extension match alone once implied. Failed validation leaves the project
//! untouched and reports the operator-readable reason.
use super::*;
use std::sync::mpsc::{Receiver, TryRecvError};

/// Bound on simultaneously queued drops; one worker validates at a time.
const MAX_PENDING_DROPS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DropMedia {
    Video,
    Audio,
    Image,
}

impl DropMedia {
    fn label(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Image => "image",
        }
    }
}

/// A fully validated dropped source, ready to commit at the next free input
/// position. Exactly one source descriptor per input, like the dialog.
pub(crate) enum DropValidated {
    Video { path: String },
    Audio { path: String },
    Image(media::DecodedImage),
}

struct DropRequest {
    path: PathBuf,
    media: DropMedia,
    generation: u64,
}

struct ActiveDrop {
    media: DropMedia,
    receiver: Receiver<std::result::Result<DropValidated, String>>,
    generation: u64,
}

#[derive(Default)]
pub(crate) struct DropIntake {
    generation: u64,
    pending: VecDeque<DropRequest>,
    active: Option<ActiveDrop>,
}

impl DropIntake {
    /// Retire queued drops when the project is replaced; a result from an
    /// in-flight worker is discarded through its stale generation.
    pub(crate) fn reset(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.pending.clear();
    }
    pub(crate) fn busy(&self) -> bool {
        self.active.is_some() || !self.pending.is_empty()
    }
    fn enqueue(&mut self, path: PathBuf, media: DropMedia) -> bool {
        if self.pending.len() >= MAX_PENDING_DROPS {
            return false;
        }
        self.pending.push_back(DropRequest {
            path,
            media,
            generation: self.generation,
        });
        true
    }
    /// Start the next queued worker when idle, then collect one finished
    /// result. A stale generation yields no result, like `MediaLoader::poll`.
    fn poll(
        &mut self,
        ctx: &egui::Context,
    ) -> Option<(DropMedia, std::result::Result<DropValidated, String>)> {
        if self.active.is_none()
            && let Some(request) = self.pending.pop_front()
            && request.generation == self.generation
        {
            let DropRequest {
                path,
                media,
                generation,
            } = request;
            let (sender, receiver) = mpsc::sync_channel(1);
            let repaint = ctx.clone();
            let spawn = std::thread::Builder::new()
                .name("rustmix-drop-intake".into())
                .spawn(move || {
                    let result = match media {
                        DropMedia::Video => video_controls::probe_video(path.clone())
                            .map(|probe| DropValidated::Video { path: probe.path }),
                        DropMedia::Audio => audio_controls::validate_wav(path.clone())
                            .map(|info| DropValidated::Audio { path: info.path }),
                        DropMedia::Image => media::decode_image(&path).map(DropValidated::Image),
                    };
                    let _ = sender.try_send(result);
                    repaint.request_repaint();
                });
            match spawn {
                Ok(_) => {
                    self.active = Some(ActiveDrop {
                        media,
                        receiver,
                        generation,
                    });
                }
                Err(error) => {
                    return Some((
                        media,
                        Err(format!("Drop worker could not start ({:?})", error.kind())),
                    ));
                }
            }
        }
        let result = match self.active.as_ref()?.receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err("Drop validation worker stopped unexpectedly".into())
            }
        };
        let active = self.active.take()?;
        (active.generation == self.generation).then_some((active.media, result))
    }
}

impl RustMixApp {
    /// Queue a dropped media file for validation. Nothing is committed here;
    /// `poll_drop_source` commits once the bounded worker reports success.
    pub(crate) fn queue_drop(&mut self, path: &Path, media: DropMedia) {
        let file = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file");
        let starts_now = !self.drop_intake.busy();
        if !self.drop_intake.enqueue(path.to_path_buf(), media) {
            self.say(
                "Drop check queue is full; drop the file again shortly".into(),
                true,
            );
            return;
        }
        let verb = if starts_now { "Checking" } else { "Queued" };
        self.say(
            format!("{verb} dropped {} file {file}…", media.label()),
            false,
        );
    }

    pub(crate) fn poll_drop_source(&mut self, ctx: &egui::Context) {
        let Some((media, result)) = self.drop_intake.poll(ctx) else {
            return;
        };
        match result {
            Ok(validated) => self.commit_validated_drop(validated, ctx),
            Err(error) => self.say(
                format!(
                    "Dropped {} was not added: {}",
                    media.label(),
                    video_controls::bounded_reason(&error)
                ),
                true,
            ),
        }
    }

    fn commit_validated_drop(&mut self, validated: DropValidated, ctx: &egui::Context) {
        // The position is recomputed at commit time: validation is async and
        // the project may have changed since the file was dropped.
        let index = {
            let project = self.engine.project();
            if project.is_blank_template() {
                0
            } else {
                project.inputs.len()
            }
        };
        let message = match validated {
            DropValidated::Video { path } => {
                self.video_ui.history.record(&path);
                let name = video_controls::file_stem_label(&path, "Video file");
                let config = InputConfig {
                    video_source: Some(VideoSource::File { path }),
                    ..InputConfig::default()
                };
                if self
                    .try_execute(Command::AddInputAt {
                        name,
                        kind: InputKind::Color,
                        config,
                        index,
                    })
                    .is_some()
                {
                    self.submit_program();
                    Some("Video input added from the dropped file")
                } else {
                    None
                }
            }
            DropValidated::Audio { path } => {
                let name = video_controls::file_stem_label(&path, "Audio file");
                let config = InputConfig {
                    audio_source: Some(AudioSource::WavFile { path }),
                    ..InputConfig::default()
                };
                if self
                    .try_execute(Command::AddInputAt {
                        name,
                        kind: InputKind::Color,
                        config,
                        index,
                    })
                    .is_some()
                {
                    self.submit_program();
                    Some("Audio input added from the dropped file")
                } else {
                    None
                }
            }
            DropValidated::Image(image) => {
                let mut fx = InputFx::default();
                fx.set_image(image);
                if self.commit_png_input(fx, index) {
                    Some("Image input added from the dropped file")
                } else {
                    None
                }
            }
        };
        if let Some(message) = message {
            self.say(message.into(), false);
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
    use super::*;

    fn wait_drop(app: &mut RustMixApp, ctx: &egui::Context) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while app.drop_intake.busy() {
            app.poll_drop_source(ctx);
            assert!(
                Instant::now() < deadline,
                "drop intake worker did not finish"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Canonical PCM16 stereo 48 kHz WAV, the engine's supported audio shape.
    fn wav_file(path: &Path) {
        let frames = 480_u32;
        let data_len = frames * 4;
        let mut bytes = Vec::with_capacity(44 + data_len as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&48_000_u32.to_le_bytes());
        bytes.extend_from_slice(&(48_000_u32 * 4).to_le_bytes());
        bytes.extend_from_slice(&4_u16.to_le_bytes());
        bytes.extend_from_slice(&16_u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        for frame in 0..frames {
            let sample = ((frame % 97) * 100) as i16;
            bytes.extend_from_slice(&sample.to_le_bytes());
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        std::fs::write(path, bytes).unwrap();
    }

    fn inject(
        app: &mut RustMixApp,
        media: DropMedia,
        result: std::result::Result<DropValidated, String>,
    ) {
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.send(result).unwrap();
        app.drop_intake.active = Some(ActiveDrop {
            media,
            receiver,
            generation: app.drop_intake.generation,
        });
    }

    #[test]
    fn dropped_wav_validates_then_commits_an_audio_input() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("tone.wav");
        wav_file(&path);
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.handle_dropped_file(&path);
        assert!(app.drop_intake.busy(), "WAV drop must queue validation");
        wait_drop(&mut app, &ctx);
        let input = app.engine.project().inputs.last().unwrap();
        assert_eq!(
            input.config.audio_source,
            Some(AudioSource::WavFile {
                path: path.to_str().unwrap().to_owned()
            })
        );
        assert!(input.config.video_source.is_none() && input.config.media_path.is_none());
        assert_eq!(input.kind, InputKind::Color);
        assert_eq!(input.name, "tone");
        assert!(!app.warning, "{}", app.notice);
    }

    #[test]
    fn dropped_png_validates_then_commits_an_image_input() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("still.png");
        image::save_buffer(&path, &[170, 30, 50, 255], 1, 1, image::ColorType::Rgba8).unwrap();
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.handle_dropped_file(&path);
        assert!(app.drop_intake.busy(), "PNG drop must queue validation");
        wait_drop(&mut app, &ctx);
        let input = app.engine.project().inputs.last().unwrap();
        assert_eq!(
            input.config.media_path.as_deref(),
            Some(path.canonicalize().unwrap().to_str().unwrap())
        );
        assert!(input.config.audio_source.is_none() && input.config.video_source.is_none());
        assert_eq!(input.kind, InputKind::Color);
        assert!(
            app.input_fx.get(&input.id).unwrap().custom_rgba.is_some(),
            "committed drop must display its pixels immediately"
        );
        assert!(!app.warning, "{}", app.notice);
    }

    #[test]
    fn dropped_jpeg_and_bmp_are_validated_before_commit() {
        let directory = tempfile::tempdir().unwrap();
        for (name, format) in [
            ("still.JPG", image::ImageFormat::Jpeg),
            ("still.bmp", image::ImageFormat::Bmp),
        ] {
            let path = directory.path().join(name);
            image::save_buffer_with_format(
                &path,
                &[170, 30, 50].repeat(16 * 9),
                16,
                9,
                image::ColorType::Rgb8,
                format,
            )
            .unwrap();
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            let before = (app.engine.project().program, app.engine.project().preview);
            app.handle_dropped_file(&path);
            assert!(app.drop_intake.busy());
            wait_drop(&mut app, &ctx);
            let input = app.engine.project().inputs.last().unwrap();
            assert_eq!(
                input.config.media_path.as_deref(),
                Some(path.canonicalize().unwrap().to_str().unwrap())
            );
            assert!(app.input_fx.get(&input.id).unwrap().custom_rgba.is_some());
            assert_eq!(
                (app.engine.project().program, app.engine.project().preview),
                before
            );
            assert!(!app.warning, "{}", app.notice);

            let invalid = directory.path().join(format!("invalid-{name}"));
            std::fs::write(&invalid, b"broken image").unwrap();
            let project = app.engine.project().clone();
            app.handle_dropped_file(&invalid);
            wait_drop(&mut app, &ctx);
            assert!(app.warning);
            assert_eq!(*app.engine.project(), project);
        }
    }

    #[cfg(windows)]
    #[test]
    fn dropped_mp4_probes_then_commits_a_video_only_input() {
        let path = match std::env::var_os("RUSTMIX_TEST_VIDEO") {
            Some(path) => PathBuf::from(path),
            None => {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../artifacts/video-fixture.mp4")
            }
        };
        assert!(
            path.is_file(),
            "Generated MP4 fixture is missing at {}; run scripts/ci-video-fixture.ps1 or set RUSTMIX_TEST_VIDEO",
            path.display()
        );
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.handle_dropped_file(&path);
        assert!(app.drop_intake.busy(), "MP4 drop must queue a probe");
        wait_drop(&mut app, &ctx);
        let input = app.engine.project().inputs.last().unwrap();
        let path_string = path.to_str().unwrap().to_owned();
        assert_eq!(
            input.config.video_source,
            Some(VideoSource::File {
                path: path_string.clone()
            })
        );
        assert!(input.config.audio_source.is_none() && input.config.media_path.is_none());
        assert_eq!(input.kind, InputKind::Color);
        assert!(
            app.video_ui.history.paths().contains(&path_string),
            "a dropped video joins the recent-files list like the dialog"
        );
        assert!(!app.warning, "{}", app.notice);
    }

    #[cfg(windows)]
    #[test]
    fn dropped_additional_video_containers_probe_before_commit() {
        let fixture = std::env::var_os("RUSTMIX_TEST_VIDEO")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../artifacts/video-fixture.mp4")
            });
        let directory = fixture.parent().unwrap();
        for name in [
            "video-fixture.mov",
            "video-fixture.m4v",
            "video-fixture.avi",
            "video-fixture.wmv",
        ] {
            let path = directory.join(name);
            assert!(path.is_file(), "Missing generated fixture {}", path.display());
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            let before = (app.engine.project().program, app.engine.project().preview);
            app.handle_dropped_file(&path);
            assert!(app.drop_intake.busy());
            wait_drop(&mut app, &ctx);
            assert!(!app.warning, "{name}: {}", app.notice);
            assert!(app.engine.project().inputs.iter().any(|input| {
                matches!(&input.config.video_source, Some(VideoSource::File { path: source }) if source == path.to_str().unwrap())
            }));
            assert_eq!((app.engine.project().program, app.engine.project().preview), before);
        }
    }

    #[test]
    fn dropped_media_with_unsupported_content_is_refused_not_half_added() {
        let directory = tempfile::tempdir().unwrap();
        let corrupt = directory.path().join("broken.mp4");
        std::fs::write(&corrupt, b"not a real mp4").unwrap();
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let before = app.engine.project().clone();
        app.handle_dropped_file(&corrupt);
        wait_drop(&mut app, &ctx);
        assert!(app.warning, "a failed probe must be visible");
        assert!(app.notice.contains("was not added"), "{}", app.notice);
        assert_eq!(*app.engine.project(), before);
    }

    #[test]
    fn unsupported_dropped_formats_are_refused_without_side_effects() {
        for (name, expected) in [
            ("clip.webm", "Use MP4, M4V, MOV, AVI or WMV"),
            ("clip.mkv", "Use MP4, M4V, MOV, AVI or WMV"),
            ("song.mp3", "Only WAV audio"),
            ("photo.gif", "Only PNG, JPEG and BMP"),
            ("sheet.tiff", "Only PNG, JPEG and BMP"),
            ("notes.txt", "Unsupported file format"),
            ("archive.tar", "Unsupported file format"),
            ("data.json", "Unsupported file format"),
        ] {
            let mut app = RustMixApp::headless();
            let before = app.engine.project().clone();
            app.handle_dropped_file(Path::new(name));
            assert!(app.warning, "{name}");
            assert!(app.notice.contains(expected), "{name}: {}", app.notice);
            assert!(!app.drop_intake.busy(), "{name}");
            assert_eq!(*app.engine.project(), before, "{name}");
        }
    }

    #[test]
    fn validated_drop_results_commit_exactly_one_source_each() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        inject(
            &mut app,
            DropMedia::Video,
            Ok(DropValidated::Video {
                path: r"C:\media\clip.mp4".into(),
            }),
        );
        app.poll_drop_source(&ctx);
        let video = app.engine.project().inputs.last().unwrap().clone();
        assert_eq!(
            video.config.video_source,
            Some(VideoSource::File {
                path: r"C:\media\clip.mp4".into()
            })
        );
        assert!(video.config.audio_source.is_none() && video.config.media_path.is_none());
        assert_eq!(video.name, "clip");
        assert!(
            app.video_ui
                .history
                .paths()
                .iter()
                .any(|p| p == r"C:\media\clip.mp4")
        );

        inject(
            &mut app,
            DropMedia::Audio,
            Ok(DropValidated::Audio {
                path: r"C:\media\tone.wav".into(),
            }),
        );
        app.poll_drop_source(&ctx);
        let audio = app.engine.project().inputs.last().unwrap();
        assert_eq!(
            audio.config.audio_source,
            Some(AudioSource::WavFile {
                path: r"C:\media\tone.wav".into()
            })
        );
        assert!(audio.config.video_source.is_none() && audio.config.media_path.is_none());
        assert_eq!(audio.name, "tone");

        inject(
            &mut app,
            DropMedia::Image,
            Ok(DropValidated::Image(media::DecodedImage {
                path: r"C:\media\still.png".into(),
                original_size: [2, 2],
                width: 2,
                height: 2,
                pixels: Arc::new(vec![[9, 8, 7, 255]; 4]),
            })),
        );
        app.poll_drop_source(&ctx);
        let image = app.engine.project().inputs.last().unwrap();
        assert_eq!(
            image.config.media_path.as_deref(),
            Some(r"C:\media\still.png")
        );
        assert!(image.config.video_source.is_none() && image.config.audio_source.is_none());
        assert!(app.input_fx.get(&image.id).unwrap().custom_rgba.is_some());
    }

    #[test]
    fn a_failed_drop_validation_reports_and_commits_nothing() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        let before = app.engine.project().clone();
        inject(
            &mut app,
            DropMedia::Video,
            Err("MP4 could not be opened".into()),
        );
        app.poll_drop_source(&ctx);
        assert!(app.warning);
        assert!(app.notice.contains("was not added"), "{}", app.notice);
        assert_eq!(*app.engine.project(), before);
    }

    #[test]
    fn a_replaced_project_discards_late_drop_results() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        let before = app.engine.project().clone();
        inject(
            &mut app,
            DropMedia::Video,
            Ok(DropValidated::Video {
                path: r"C:\media\clip.mp4".into(),
            }),
        );
        app.drop_intake.reset();
        app.poll_drop_source(&ctx);
        assert_eq!(*app.engine.project(), before);
        assert!(!app.drop_intake.busy());
    }

    #[test]
    fn the_drop_queue_is_bounded() {
        let mut app = RustMixApp::headless();
        for n in 0..8 {
            app.handle_dropped_file(Path::new(&format!("file{n}.png")));
        }
        assert_eq!(app.drop_intake.pending.len(), 8);
        app.handle_dropped_file(Path::new("file9.png"));
        assert_eq!(app.drop_intake.pending.len(), 8);
        assert!(app.warning, "the ninth simultaneous drop must be refused");
    }

    #[test]
    fn a_dropped_project_file_loads_instead_of_adding_an_input() {
        let mut app = RustMixApp::headless();
        let before = app.engine.project().clone();
        app.handle_dropped_file(Path::new("show.rustmix.json"));
        assert!(!app.drop_intake.busy());
        // The load worker runs off-thread; no synchronous project change.
        assert_eq!(*app.engine.project(), before);
        // Windows paths are case-insensitive; an uppercase suffix still loads.
        app.handle_dropped_file(Path::new("SHOW.RUSTMIX.JSON"));
        assert!(!app.drop_intake.busy());
        assert!(!app.warning, "{}", app.notice);
        assert_eq!(*app.engine.project(), before);
    }
}
