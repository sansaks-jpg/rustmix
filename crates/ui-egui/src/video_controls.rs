//! Local MP4 video sources. Descriptor probing runs on a bounded worker; one
//! owner thread per input decodes and paces frames into an engine `VideoFeed`.
//! The UI thread never opens Media Foundation, decodes a sample or blocks on a
//! decoder read. Audio tracks, cameras and network inputs are not provided.
use super::*;
use crate::dialogs::VIDEO_TAB;
use crate::video_player::VideoPlayer;
use rustmix_engine::video_source::{VideoFeedError, VideoSourceHealth};
use std::sync::mpsc::TryRecvError;

/// Bounded local-file size for the descriptor; the decoder itself streams.
const MAX_MP4_BYTES: u64 = 4 * 1024 * 1024 * 1024;

#[derive(Clone)]
struct VideoProbe {
    path: String,
    width: u32,
    height: u32,
    frame_rate_numerator: u32,
    frame_rate_denominator: u32,
}
struct ProbeResult {
    token: u64,
    result: std::result::Result<Option<VideoProbe>, String>,
}

/// Add-dialog draft for a local video source. Nothing here starts a decoder.
#[derive(Default)]
pub(crate) struct VideoUi {
    label: String,
    path: String,
    probe: Option<VideoProbe>,
    job: Option<Receiver<ProbeResult>>,
    job_token: u64,
    add_error: Option<String>,
}

/// One live decoder owner for an input. Replacing the descriptor or an explicit
/// restart constructs a new slot so an obsolete reader cannot publish frames.
pub(crate) struct VideoSlot {
    player: VideoPlayer,
    path: String,
    sequence: Option<u64>,
    revision: u64,
    control: Option<(bool, bool, u64)>,
    restart: u64,
}

fn bounded_reason(reason: &str) -> String {
    let mut text = String::with_capacity(160);
    for character in reason.chars().filter(|character| !character.is_control()) {
        if text.len().saturating_add(character.len_utf8()) > 160 {
            break;
        }
        text.push(character);
    }
    if text.trim().is_empty() {
        "Video source unavailable".into()
    } else {
        text
    }
}

fn health_message(health: &VideoSourceHealth) -> Option<String> {
    match health {
        VideoSourceHealth::Degraded { reason }
        | VideoSourceHealth::Offline { reason }
        | VideoSourceHealth::Failed { reason } => Some(reason.clone()),
        _ => None,
    }
}

fn present_video_error(input_fx: &mut HashMap<InputId, InputFx>, id: InputId, error: &str) {
    let fx = input_fx.entry(id).or_default();
    fx.custom_rgba = None;
    fx.image_pending = false;
    fx.image_error = Some(bounded_reason(error));
}

#[cfg(windows)]
fn probe_video(path: PathBuf) -> std::result::Result<VideoProbe, String> {
    // Only reached from the bounded probe worker. `VideoFile` starts COM and
    // Media Foundation on its own thread and releases them when it is dropped.
    let path_string = path
        .to_str()
        .ok_or_else(|| "MP4 path must be valid text".to_owned())?
        .to_owned();
    VideoSource::File {
        path: path_string.clone(),
    }
    .validate()
    .map_err(|error| error.to_string())?;
    let metadata = std::fs::metadata(&path)
        .map_err(|error| format!("Could not inspect MP4 ({:?})", error.kind()))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_MP4_BYTES {
        return Err("Choose a regular local MP4 file".into());
    }
    let file = rustmix_platform::media_foundation::VideoFile::open(&path)
        .map_err(|error| error.to_string())?;
    let format = file.format();
    Ok(VideoProbe {
        path: path_string,
        width: format.width,
        height: format.height,
        frame_rate_numerator: format.frame_rate_numerator,
        frame_rate_denominator: format.frame_rate_denominator,
    })
}

#[cfg(not(windows))]
fn probe_video(_path: PathBuf) -> std::result::Result<VideoProbe, String> {
    // A descriptor must never imply that a decode backend exists here.
    Err("Local MP4 decoding requires Windows Media Foundation".into())
}

impl RustMixApp {
    pub(crate) fn poll_video(&mut self, ctx: &egui::Context) {
        self.poll_video_probe();
        let mut desired: Vec<(InputId, String, bool)> = Vec::new();
        {
            let project = self.engine.project();
            for input in &project.inputs {
                if let Some(VideoSource::File { path }) = &input.config.video_source {
                    desired.push((input.id, path.clone(), input.online));
                }
            }
        }
        // Removing or relinking an input must retire its owner worker before a
        // different file can publish through a reused input identity.
        self.video_players.retain(|id, slot| {
            desired
                .iter()
                .any(|(input, path, _)| input == id && path == &slot.path)
        });
        let paused_all = self.engine.inputs_paused();
        let mut changed = false;
        for (id, path, online) in &desired {
            let clock = self.engine.playback(*id);
            let control = clock.map_or((true, false, 0), |clock| {
                (
                    clock.manually_paused() || paused_all,
                    clock.looping(),
                    clock.restart_generation(),
                )
            });
            // A terminated reader is restarted only by an explicit restart.
            let terminated_restart = self
                .video_players
                .get(id)
                .is_some_and(|slot| slot.player.is_finished() && control.2 != slot.restart);
            if terminated_restart {
                self.video_players.remove(id);
            }
            if !self.video_players.contains_key(id) {
                if !online {
                    continue;
                }
                self.video_generation = self.video_generation.wrapping_add(1);
                let generation = self.video_generation;
                match VideoPlayer::spawn(*id, generation, path.clone(), ctx.clone()) {
                    Ok(player) => {
                        self.video_players.insert(
                            *id,
                            VideoSlot {
                                player,
                                path: path.clone(),
                                sequence: None,
                                revision: 0,
                                control: None,
                                restart: control.2,
                            },
                        );
                    }
                    Err(error) => {
                        present_video_error(&mut self.input_fx, *id, &error);
                        continue;
                    }
                }
            }
            let Some(slot) = self.video_players.get_mut(id) else {
                continue;
            };
            if slot.control != Some(control) {
                slot.player.update(control.0, control.1, control.2);
                slot.control = Some(control);
            }
            slot.restart = control.2;
            // An offline input stays in the project with its last valid frame
            // retained by the feed, but it is not presented as live picture.
            if !online {
                let fx = self.input_fx.entry(*id).or_default();
                fx.custom_rgba = None;
                fx.image_pending = false;
                fx.image_error = None;
                continue;
            }
            if let Some(error) = slot.player.error() {
                present_video_error(&mut self.input_fx, *id, &error);
                continue;
            }
            match slot.player.feed().snapshot() {
                Ok(snapshot) => {
                    if snapshot.revision == slot.revision {
                        continue;
                    }
                    slot.revision = snapshot.revision;
                    let health = health_message(&snapshot.health);
                    if let Some(frame) = snapshot.frame.as_ref() {
                        let metadata = frame.metadata();
                        if slot.sequence == Some(metadata.sequence) {
                            let fx = self.input_fx.entry(*id).or_default();
                            fx.image_error = health;
                            continue;
                        }
                        slot.sequence = Some(metadata.sequence);
                        let fx = self.input_fx.entry(*id).or_default();
                        fx.custom_rgba =
                            Some((metadata.width, metadata.height, frame.shared_pixels()));
                        fx.image_size = Some([metadata.width, metadata.height]);
                        fx.image_error = health;
                        fx.image_pending = false;
                        changed = true;
                    } else {
                        let starting = !slot.player.is_finished();
                        let fx = self.input_fx.entry(*id).or_default();
                        fx.image_error = health
                            .or_else(|| Some("Waiting for the first decoded video frame".into()));
                        fx.image_pending = starting;
                    }
                }
                // A contended snapshot keeps the previously presented frame.
                Err(VideoFeedError::Busy) => {}
                Err(error) => {
                    present_video_error(&mut self.input_fx, *id, &error.to_string());
                }
            }
        }
        if changed {
            // A new decoded raster must repaint tiles and submit a Program
            // snapshot; the fixed 640x360 composite is still the live output.
            self.last_demo_frame = None;
            self.submit_program();
        }
    }

    fn poll_video_probe(&mut self) {
        let result = self
            .video_ui
            .job
            .as_ref()
            .and_then(|job| match job.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Disconnected) => Some(ProbeResult {
                    token: self.video_ui.job_token,
                    result: Err(
                        "Video probe worker stopped unexpectedly; choose the file again".into(),
                    ),
                }),
                Err(TryRecvError::Empty) => None,
            });
        let Some(result) = result else {
            return;
        };
        self.video_ui.job = None;
        if !self.add_open || self.add_tab != VIDEO_TAB || result.token != self.add_token {
            return;
        }
        match result.result {
            Ok(Some(probe)) => {
                self.video_ui.path = probe.path.clone();
                self.video_ui.probe = Some(probe);
                self.video_ui.add_error = None;
            }
            // Picker cancellation keeps the current draft intact.
            Ok(None) => {}
            Err(error) => {
                self.video_ui.probe = None;
                self.video_ui.add_error = Some(bounded_reason(&error));
            }
        }
    }

    fn start_video_probe(&mut self, ctx: &egui::Context, picker: bool) {
        if self.video_ui.job.is_some() {
            return;
        }
        let path = self.video_ui.path.clone();
        let token = self.add_token;
        let (sender, receiver) = mpsc::sync_channel(1);
        let repaint = ctx.clone();
        let spawn = std::thread::Builder::new()
            .name("rustmix-video-probe".into())
            .spawn(move || {
                let chosen = if picker {
                    rfd::FileDialog::new()
                        .add_filter("MP4 video", &["mp4"])
                        .pick_file()
                } else {
                    Some(PathBuf::from(path))
                };
                let result = match chosen {
                    None => Ok(None),
                    Some(path) => probe_video(path).map(Some),
                };
                let _ = sender.try_send(ProbeResult { token, result });
                repaint.request_repaint();
            });
        if spawn.is_ok() {
            self.video_ui.job = Some(receiver);
            self.video_ui.job_token = token;
            self.video_ui.add_error = None;
        } else {
            self.video_ui.add_error = Some("Could not start MP4 validation".into());
        }
    }

    pub(crate) fn cancel_video_add(&mut self) {
        self.video_ui.label.clear();
        self.video_ui.path.clear();
        self.video_ui.probe = None;
        self.video_ui.add_error = None;
    }

    /// Bounded probe accessors used by the opt-in native acceptance harness.
    pub(crate) fn video_probe_pending(&self) -> bool {
        self.video_ui.job.is_some()
    }
    pub(crate) fn video_probe_error(&self) -> Option<String> {
        self.video_ui.add_error.clone()
    }
    pub(crate) fn video_probe_format(&self) -> Option<(u32, u32, u32, u32)> {
        self.video_ui.probe.as_ref().map(|probe| {
            (
                probe.width,
                probe.height,
                probe.frame_rate_numerator,
                probe.frame_rate_denominator,
            )
        })
    }

    pub(crate) fn stop_video_players(&mut self) {
        self.video_players.clear();
        self.cancel_video_add();
    }

    pub(crate) fn video_add_page(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Local MP4 video");
        ui.label("Video-only source decoded by Windows Media Foundation. Audio tracks, cameras and network inputs are not provided.");
        ui.label("Source label");
        let name = ui.add(
            egui::TextEdit::singleline(&mut self.video_ui.label)
                .char_limit(64)
                .hint_text("Video file"),
        );
        self.automation
            .insert("dialog.add.video.name".into(), name.rect);
        ui.label("Local MP4 path (validated before adding; frames decode on an owner worker)");
        let path = ui.add_enabled(
            self.video_ui.job.is_none(),
            egui::TextEdit::singleline(&mut self.video_ui.path).char_limit(4096),
        );
        self.automation
            .insert("dialog.add.video.path".into(), path.rect);
        if path.changed() {
            self.video_ui.probe = None;
            self.video_ui.add_error = None;
        }
        ui.horizontal(|ui| {
            let browse = ui.add_enabled(
                self.video_ui.job.is_none(),
                egui::Button::new("Browse MP4…"),
            );
            self.automation
                .insert("dialog.add.video.browse".into(), browse.rect);
            if browse.clicked() && self.smoke.is_none() {
                self.start_video_probe(ctx, true);
            }
            let validate = ui.add_enabled(
                self.video_ui.job.is_none() && !self.video_ui.path.trim().is_empty(),
                egui::Button::new("Validate video"),
            );
            self.automation
                .insert("dialog.add.video.validate".into(), validate.rect);
            if validate.clicked() {
                self.start_video_probe(ctx, false);
            }
        });
        if self.video_ui.job.is_some() {
            ui.label("Probing the MP4 container and its decoder…");
        }
        if let Some(probe) = &self.video_ui.probe {
            let rate = if probe.frame_rate_denominator > 0 {
                f64::from(probe.frame_rate_numerator) / f64::from(probe.frame_rate_denominator)
            } else {
                0.0
            };
            ui.label(format!(
                "Validated: {}×{} · {rate:.3} fps · video only",
                probe.width, probe.height
            ));
            ui.label("The owner worker decodes independently; Preview and Program use the newest accepted frame.");
        }
        if let Some(error) = &self.video_ui.add_error {
            let label = ui.colored_label(RED, error);
            self.automation
                .insert("dialog.add.video.error".into(), label.rect);
        }
        ui.label("Video descriptor only: no MP4 audio, GPU decode, camera, capture card or network stream is opened.");
    }

    pub(crate) fn confirm_video_input(&mut self, ctx: &egui::Context) -> bool {
        let Some(probe) = &self.video_ui.probe else {
            self.video_ui.add_error =
                Some("Validate a supported local MP4 before adding it".into());
            return false;
        };
        if probe.path != self.video_ui.path || self.video_ui.job.is_some() {
            self.video_ui.add_error =
                Some("Wait for the current video file to finish probing".into());
            return false;
        }
        let source = VideoSource::File {
            path: probe.path.clone(),
        };
        if let Err(error) = source.validate() {
            self.video_ui.add_error = Some(error.to_string());
            return false;
        }
        let label = if self.video_ui.label.trim().is_empty() {
            "Video file"
        } else {
            self.video_ui.label.trim()
        }
        .to_owned();
        let config = InputConfig {
            video_source: Some(source),
            ..InputConfig::default()
        };
        if self
            .try_execute(Command::AddInputAt {
                name: label,
                kind: InputKind::Color,
                config,
                index: self.add_number.saturating_sub(1),
            })
            .is_none()
        {
            self.video_ui.add_error = Some(self.notice.clone());
            return false;
        }
        self.submit_program();
        self.cancel_video_add();
        self.add_open = false;
        ctx.request_repaint();
        true
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
    use super::*;

    fn frame(app: &mut RustMixApp, ctx: &egui::Context, events: Vec<egui::Event>, size: [f32; 2]) {
        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(size[0], size[1]))),
                events,
                ..Default::default()
            },
            |ui| app.frame_ui(ui),
        );
    }
    fn click(app: &mut RustMixApp, ctx: &egui::Context, id: &str, size: [f32; 2]) {
        let mut previous: Option<Rect> = None;
        let mut stable = None;
        for _ in 0..6 {
            frame(app, ctx, vec![], size);
            let bounds = *app
                .automation
                .get(id)
                .expect("Visible video operator control");
            if previous.is_some_and(|old| {
                old.center().distance(bounds.center()) < 0.5
                    && (old.size() - bounds.size()).length() < 0.5
            }) {
                stable = Some(bounds);
                break;
            }
            previous = Some(bounds);
        }
        let bounds = stable.expect("Video control must settle within six frames");
        assert!(
            ctx.viewport_rect().contains_rect(bounds),
            "Video control outside viewport: {id}"
        );
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(bounds.center()),
                    egui::Event::PointerButton {
                        pos: bounds.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                size,
            );
        }
    }
    fn probe(path: &str) -> VideoProbe {
        VideoProbe {
            path: path.into(),
            width: 160,
            height: 90,
            frame_rate_numerator: 30,
            frame_rate_denominator: 1,
        }
    }
    fn wait_probe(app: &mut RustMixApp, ctx: &egui::Context, size: [f32; 2]) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while app.video_ui.job.is_some() {
            assert!(
                Instant::now() < deadline,
                "Video probe worker did not finish"
            );
            frame(app, ctx, vec![], size);
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn confirm_requires_a_validated_probe_and_never_mutates_the_project_early() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let before = app.engine.project().clone();
        // A path alone is not validation: the descriptor must be committed only
        // after the probe worker reports the actual decoded media type.
        app.video_ui.path = r"C:\media\clip.mp4".into();
        assert!(!app.confirm_video_input(&ctx));
        assert!(app.video_ui.add_error.is_some());
        assert_eq!(*app.engine.project(), before);

        app.video_ui.probe = Some(probe(r"C:\media\clip.mp4"));
        app.video_ui.path = r"C:\media\other.mp4".into();
        assert!(!app.confirm_video_input(&ctx));
        assert_eq!(*app.engine.project(), before);
    }

    #[test]
    fn validated_descriptor_commits_video_input_and_cancel_discards_late_drafts() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let before = app.engine.project().clone();
        app.video_ui.probe = Some(probe(r"C:\media\clip.mp4"));
        app.video_ui.path = r"C:\media\clip.mp4".into();
        app.video_ui.label = "Clip one".into();
        assert!(app.confirm_video_input(&ctx));
        let project = app.engine.project();
        assert_eq!(project.inputs.len(), before.inputs.len() + 1);
        let added = project
            .inputs
            .iter()
            .find(|input| input.name == "Clip one")
            .expect("video input inserted");
        assert_eq!(added.name, "Clip one");
        assert_eq!(added.kind, InputKind::Color);
        assert_eq!(
            added.config.video_source,
            Some(VideoSource::File {
                path: r"C:\media\clip.mp4".into()
            })
        );
        assert!(added.config.media_path.is_none() && added.config.audio_source.is_none());
        assert_eq!(
            (project.program, project.preview),
            (before.program, before.preview)
        );
        assert!(!app.add_open);
        assert!(app.video_ui.probe.is_none() && app.video_ui.path.is_empty());

        // A closed dialog cannot commit a stale draft or a late probe result.
        let accepted = app.engine.project().clone();
        let token = app.add_token;
        let (sender, receiver) = mpsc::sync_channel(1);
        app.video_ui.job = Some(receiver);
        app.video_ui.job_token = token;
        app.add_token = app.add_token.wrapping_add(1);
        sender
            .send(ProbeResult {
                token,
                result: Ok(Some(probe(r"C:\media\late.mp4"))),
            })
            .unwrap();
        app.poll_video_probe();
        assert!(app.video_ui.job.is_none() && app.video_ui.probe.is_none());
        assert_eq!(*app.engine.project(), accepted);
    }

    #[test]
    fn probe_worker_rejects_non_mp4_off_thread_and_reports_disconnect() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let before = app.engine.project().clone();
        let size = [1600.0, 900.0];
        app.add_open = true;
        app.add_tab = VIDEO_TAB;
        app.video_ui.path = r"C:\media\clip.avi".into();
        app.start_video_probe(&ctx, false);
        wait_probe(&mut app, &ctx, size);
        assert!(app.video_ui.probe.is_none());
        let error = app.video_ui.add_error.as_deref().unwrap_or_default();
        assert!(
            error.to_ascii_lowercase().contains("mp4"),
            "unexpected probe error: {error}"
        );
        assert_eq!(*app.engine.project(), before);

        let (sender, receiver) = mpsc::sync_channel::<ProbeResult>(1);
        drop(sender);
        app.video_ui.job = Some(receiver);
        app.poll_video_probe();
        assert!(app.video_ui.job.is_none());
        assert!(app.video_ui.add_error.is_some());
        assert_eq!(*app.engine.project(), before);
    }

    #[test]
    fn video_health_helpers_bound_reasons_and_classify_only_warning_states() {
        let long = format!("\n{}", "🎬".repeat(200));
        let bounded = bounded_reason(&long);
        assert!(bounded.len() <= 160 && !bounded.chars().any(char::is_control));
        assert!(!bounded_reason("\r\n").is_empty());
        assert!(health_message(&VideoSourceHealth::Running).is_none());
        assert!(health_message(&VideoSourceHealth::Stopped).is_none());
        for health in [
            VideoSourceHealth::Degraded {
                reason: "decoding late".into(),
            },
            VideoSourceHealth::Offline {
                reason: "input offline".into(),
            },
            VideoSourceHealth::Failed {
                reason: "decode failed".into(),
            },
        ] {
            assert!(health_message(&health).is_some());
        }
    }

    #[test]
    fn presenting_a_decoder_error_clears_the_frame_and_is_visible_to_the_tile() {
        let mut input_fx = HashMap::new();
        input_fx.insert(
            InputId(8),
            InputFx {
                custom_rgba: Some((2, 2, Arc::new(vec![[10, 20, 30, 255]; 4]))),
                image_pending: true,
                ..InputFx::default()
            },
        );
        present_video_error(&mut input_fx, InputId(8), "decoder lost");
        let fx = input_fx.get(&InputId(8)).expect("cache entry retained");
        assert!(fx.custom_rgba.is_none() && !fx.image_pending);
        assert_eq!(fx.image_error.as_deref(), Some("decoder lost"));
    }

    #[test]
    fn video_add_page_is_reachable_and_cancel_discards_the_draft() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        let before = app.engine.project().clone();
        app.dispatch(Action::Add, &ctx);
        click(&mut app, &ctx, "dialog.add.video", size);
        assert_eq!(app.add_tab, VIDEO_TAB);
        frame(&mut app, &ctx, vec![], size);
        for id in [
            "dialog.add.video.name",
            "dialog.add.video.path",
            "dialog.add.video.validate",
            "dialog.add.confirm",
            "dialog.add.cancel",
        ] {
            assert!(app.automation.contains_key(id), "missing control {id}");
        }
        click(&mut app, &ctx, "dialog.add.video.path", size);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Text(r"C:\media\clip.avi".into())],
            size,
        );
        click(&mut app, &ctx, "dialog.add.video.validate", size);
        wait_probe(&mut app, &ctx, size);
        assert!(app.video_ui.add_error.is_some());
        click(&mut app, &ctx, "dialog.add.cancel", size);
        assert!(!app.add_open);
        assert!(app.video_ui.path.is_empty() && app.video_ui.probe.is_none());
        assert_eq!(*app.engine.project(), before);
    }
}
