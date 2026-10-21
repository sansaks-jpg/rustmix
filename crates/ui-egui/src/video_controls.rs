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
#[cfg_attr(not(windows), allow(dead_code))]
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
    pub(crate) history: crate::video_history::VideoHistory,
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

fn default_video_label(path: &str) -> String {
    let mut label = String::new();
    for character in Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Video file")
        .chars()
        .filter(|character| !character.is_control())
    {
        if label.len().saturating_add(character.len_utf8()) > 64 {
            break;
        }
        label.push(character);
    }
    if label.trim().is_empty() {
        "Video file".into()
    } else {
        label
    }
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
                        fx.image_generation = fx.image_generation.wrapping_add(1);
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
                self.video_ui.history.record(&probe.path);
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
        // Dropping the receiver retires a late probe result; the token is
        // advanced as well so a reopened dialog never accepts old metadata.
        self.video_ui.job = None;
        self.video_ui.job_token = self.video_ui.job_token.wrapping_add(1);
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

    pub(crate) fn video_add_ready(&self) -> bool {
        self.video_ui.job.is_none()
            && self
                .video_ui
                .probe
                .as_ref()
                .is_some_and(|probe| probe.path == self.video_ui.path)
    }

    pub(crate) fn stop_video_players(&mut self) {
        self.video_players.clear();
        self.cancel_video_add();
    }

    pub(crate) fn video_add_page(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let page_width = ui.available_width();
        let header = egui::Frame::new()
            .fill(Color32::from_rgb(10, 15, 20))
            .inner_margin(egui::Margin::symmetric(8, 8))
            .show(ui, |ui| {
                ui.set_min_width((page_width - 16.0).max(80.0));
                ui.label(
                    RichText::new("Select a video file.")
                        .size(13.0)
                        .color(Color32::WHITE),
                );
            });
        self.automation
            .insert("dialog.add.video.header".into(), header.response.rect);
        egui::Frame::new().inner_margin(8).show(ui, |ui| {
            ui.horizontal(|ui| {
                let path_width = (ui.available_width() - 94.0).max(120.0);
                let path = ui
                    .add_enabled_ui(self.video_ui.job.is_none(), |ui| {
                        ui.add_sized(
                            [path_width, 26.0],
                            egui::TextEdit::singleline(&mut self.video_ui.path)
                                .char_limit(4096)
                                .hint_text("MP4 video file"),
                        )
                    })
                    .inner;
                self.automation
                    .insert("dialog.add.video.path".into(), path.rect);
                if path.changed() {
                    self.video_ui.probe = None;
                    self.video_ui.add_error = None;
                }
                let browse = ui.add_enabled(
                    self.video_ui.job.is_none(),
                    egui::Button::new("Browse…").min_size(Vec2::new(90.0, 26.0)),
                );
                self.automation
                    .insert("dialog.add.video.browse".into(), browse.rect);
                if browse.clicked() && self.smoke.is_none() {
                    self.start_video_probe(ctx, true);
                }
                if path.lost_focus()
                    && ctx.input(|input| input.key_pressed(egui::Key::Enter))
                    && self.video_ui.job.is_none()
                    && !self.video_ui.path.trim().is_empty()
                {
                    self.start_video_probe(ctx, false);
                }
            });
            let list_height = (ui.available_height() - 72.0).max(84.0);
            let mut selected_path = None;
            let recent = egui::Frame::new()
                .fill(Color32::WHITE)
                .corner_radius(0)
                .stroke(Stroke::new(1.0, Color32::from_rgb(190, 196, 203)))
                .inner_margin(2)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("video.recent.files")
                        .max_height(list_height)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if self.video_ui.history.paths().is_empty() {
                                ui.label(
                                    RichText::new("Choose Browse or enter an MP4 path.")
                                        .color(FORM_TEXT),
                                );
                            }
                            for (index, path) in self.video_ui.history.paths().iter().enumerate() {
                                let enabled = self.video_ui.job.is_none();
                                let selected = self.video_ui.path == *path;
                                let row = ui
                                    .push_id(("video.recent.file", path), |ui| {
                                        let (rect, response) = ui.allocate_exact_size(
                                            Vec2::new(ui.available_width(), 22.0),
                                            if enabled {
                                                Sense::click()
                                            } else {
                                                Sense::hover()
                                            },
                                        );
                                        if selected || response.hovered() {
                                            let fill = if selected {
                                                ui.visuals().selection.bg_fill
                                            } else {
                                                Color32::from_rgb(235, 240, 245)
                                            };
                                            ui.painter().rect_filled(rect, 0.0, fill);
                                        }
                                        let mut label_ui = ui.new_child(
                                            egui::UiBuilder::new()
                                                .id_salt("filename")
                                                .max_rect(rect.shrink2(Vec2::new(4.0, 2.0)))
                                                .layout(egui::Layout::left_to_right(
                                                    egui::Align::Center,
                                                )),
                                        );
                                        label_ui.add(
                                            egui::Label::new(RichText::new(path).color(FORM_TEXT))
                                                .truncate(),
                                        );
                                        response.widget_info(|| {
                                            egui::WidgetInfo::labeled(
                                                egui::WidgetType::SelectableLabel,
                                                enabled,
                                                path,
                                            )
                                        });
                                        response.on_hover_text(path)
                                    })
                                    .inner;
                                self.automation
                                    .insert(format!("dialog.add.video.recent.{index}"), row.rect);
                                if enabled && row.clicked() {
                                    selected_path = Some(path.clone());
                                }
                            }
                        });
                });
            self.automation
                .insert("dialog.add.video.recent".into(), recent.response.rect);
            if let Some(path) = selected_path {
                self.video_ui.path = path;
                self.video_ui.probe = None;
                self.video_ui.add_error = None;
                self.start_video_probe(ctx, false);
            }
            ui.horizontal(|ui| {
                ui.add_sized([40.0, 26.0], egui::Label::new("Name"));
                let name_width = (ui.available_width() - 178.0).max(80.0);
                let hint = default_video_label(&self.video_ui.path);
                let name = ui.add_sized(
                    [name_width, 26.0],
                    egui::TextEdit::singleline(&mut self.video_ui.label)
                        .char_limit(64)
                        .hint_text(hint),
                );
                self.automation
                    .insert("dialog.add.video.name".into(), name.rect);
                let check = ui.add_enabled(
                    self.video_ui.job.is_none() && !self.video_ui.path.trim().is_empty(),
                    egui::Button::new("Check file").min_size(Vec2::new(90.0, 26.0)),
                );
                self.automation
                    .insert("dialog.add.video.validate".into(), check.rect);
                if check.clicked() {
                    self.start_video_probe(ctx, false);
                }
                let clear = ui
                    .add_enabled(
                        self.video_ui.job.is_none() && !self.video_ui.history.paths().is_empty(),
                        egui::Button::new("Clear").min_size(Vec2::new(76.0, 26.0)),
                    )
                    .on_hover_text("Clear recent files. Files and existing inputs stay in place.");
                self.automation
                    .insert("dialog.add.video.clear".into(), clear.rect);
                if clear.clicked() {
                    self.video_ui.history.clear();
                }
            });
            let (status, color) = if let Some(error) = &self.video_ui.add_error {
                (error.clone(), RED)
            } else if self.video_ui.job.is_some() {
                ("Checking video file…".into(), FORM_TEXT)
            } else if let Some(probe) = &self.video_ui.probe {
                let rate = if probe.frame_rate_denominator > 0 {
                    f64::from(probe.frame_rate_numerator) / f64::from(probe.frame_rate_denominator)
                } else {
                    0.0
                };
                (
                    format!(
                        "{}×{} · {rate:.3} fps · Video only",
                        probe.width, probe.height
                    ),
                    FORM_TEXT,
                )
            } else {
                (
                    "MP4 video only. Add audio as a separate input.".into(),
                    FORM_TEXT,
                )
            };
            let status = ui
                .allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 22.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.add(egui::Label::new(RichText::new(&status).color(color)).truncate())
                            .on_hover_text(&status)
                    },
                )
                .inner;
            self.automation
                .insert("dialog.add.video.status".into(), status.rect);
            if self.video_ui.add_error.is_some() {
                self.automation
                    .insert("dialog.add.video.error".into(), status.rect);
            }
        });
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
            default_video_label(&probe.path)
        } else {
            self.video_ui.label.trim().to_owned()
        };
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
    fn unnamed_video_uses_a_bounded_file_name_without_switching_preview_or_program() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        let path = format!("C:/media/{}.mp4", "é".repeat(64));
        app.video_ui.path = path.clone();
        app.video_ui.probe = Some(probe(&path));
        let before = app.engine.project().clone();
        assert!(app.video_add_ready());
        assert!(app.confirm_video_input(&ctx));
        let added = app.engine.project().inputs.iter().find(|input|
            matches!(&input.config.video_source,Some(VideoSource::File { path: selected }) if selected == &path)
        ).unwrap();
        assert_eq!(added.name, "é".repeat(32));
        assert_eq!(app.engine.project().program, before.program);
        assert_eq!(app.engine.project().preview, before.preview);
        assert_eq!(default_video_label(""), "Video file");
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
    fn relinking_a_video_descriptor_clears_the_previous_decoded_frame() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.engine
            .execute(
                Command::AddInputAt {
                    name: "Clip".into(),
                    kind: InputKind::Color,
                    config: InputConfig {
                        video_source: Some(VideoSource::File {
                            path: r"C:\media\old.mp4".into(),
                        }),
                        ..InputConfig::default()
                    },
                    index: 7,
                },
                0,
            )
            .unwrap();
        app.sync_input_cache();
        let id = InputId(8);
        let fx = app.input_fx.get_mut(&id).expect("video cache entry");
        fx.custom_rgba = Some((2, 2, Arc::new(vec![[255, 0, 0, 255]; 4])));
        fx.image_size = Some([2, 2]);
        let mut relinked = app.engine.project().input(id).cloned().unwrap();
        relinked.config.video_source = Some(VideoSource::File {
            path: r"C:\media\new.mp4".into(),
        });
        app.engine
            .execute(
                Command::ApplyInput {
                    input: relinked,
                    audio: None,
                },
                0,
            )
            .unwrap();
        app.sync_input_cache();
        let fx = app.input_fx.get(&id).unwrap();
        assert!(
            fx.custom_rgba.is_none() && fx.image_size.is_none(),
            "a relinked file must never show the previous file's last frame"
        );
    }

    #[test]
    fn cancel_retires_a_pending_probe_so_a_late_result_cannot_reappear() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let (sender, receiver) = mpsc::sync_channel::<ProbeResult>(1);
        app.video_ui.job = Some(receiver);
        app.video_ui.job_token = 9;
        app.video_ui.path = r"C:\media\clip.mp4".into();
        app.cancel_video_add();
        assert!(app.video_ui.job.is_none() && app.video_ui.job_token != 9);
        // The worker's bounded send now fails and cannot resurrect the draft.
        sender
            .try_send(ProbeResult {
                token: 9,
                result: Ok(Some(probe(r"C:\media\late.mp4"))),
            })
            .unwrap_err();
        app.poll_video_probe();
        assert!(app.video_ui.path.is_empty() && app.video_ui.probe.is_none());
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

    #[test]
    fn default_video_form_fills_its_page_and_keeps_real_controls_inside_compact_and_large_dialogs()
    {
        for size in [
            [734.0, 466.0],
            [1100.0, 700.0],
            [1600.0, 900.0],
            [1917.0, 1168.0],
        ] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            let before = app.engine.project().clone();
            app.dispatch(Action::Add, &ctx);
            assert_eq!(app.add_tab, VIDEO_TAB);
            for _ in 0..5 {
                frame(&mut app, &ctx, vec![], size);
            }
            let page = *app.automation.get("dialog.add.page").unwrap();
            let path = *app.automation.get("dialog.add.video.path").unwrap();
            let browse = *app.automation.get("dialog.add.video.browse").unwrap();
            let list = *app.automation.get("dialog.add.video.recent").unwrap();
            let video_category = *app.automation.get("dialog.add.video").unwrap();
            assert!(
                (video_category.height() - (size[0] * 38.0 / 1917.0).clamp(25.0, 38.0)).abs()
                    <= 1.0
            );
            assert!((path.center().y - browse.center().y).abs() <= 1.0);
            assert!(path.right() < browse.left());
            assert!(path.width() >= page.width() * 0.65);
            assert!(list.width() >= page.width() * 0.94);
            assert!(list.height() >= page.height() * 0.45);
            for id in [
                "dialog.add.video.path",
                "dialog.add.video.browse",
                "dialog.add.video.recent",
                "dialog.add.video.name",
                "dialog.add.video.validate",
                "dialog.add.video.clear",
            ] {
                assert!(
                    page.expand(1.0)
                        .contains_rect(*app.automation.get(id).unwrap()),
                    "Video form clipped {id} at {size:?}"
                );
            }
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(size[0], size[1]))),
                    ..Default::default()
                },
                |ui| app.frame_ui(ui),
            );
            for shape in output.shapes {
                if let egui::Shape::Text(text) = shape.shape {
                    let label = text.galley.job.text.to_lowercase();
                    for jargon in [
                        "owner worker",
                        "descriptor",
                        "media foundation",
                        "gpu decode",
                    ] {
                        assert!(
                            !label.contains(jargon),
                            "Implementation details are not operator guidance: {label}"
                        );
                    }
                }
            }
            click(&mut app, &ctx, "dialog.add.confirm", size);
            assert!(app.add_open);
            click(&mut app, &ctx, "dialog.add.cancel", size);
            assert!(!app.add_open);
            assert_eq!(*app.engine.project(), before);
        }
    }

    #[test]
    fn recent_file_selection_and_clear_never_add_or_delete_an_input_or_file() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("old clip.mp4");
        std::fs::write(&path, b"invalid MP4 fixture").unwrap();
        let path = path.to_str().unwrap().to_owned();
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.video_ui.history.record(&path);
        app.dispatch(Action::Add, &ctx);
        let before = app.engine.project().clone();
        let size = [1600.0, 900.0];
        click(&mut app, &ctx, "dialog.add.video.recent.0", size);
        wait_probe(&mut app, &ctx, size);
        assert_eq!(app.video_ui.path, path);
        assert!(app.video_ui.probe.is_none() && app.video_ui.add_error.is_some());
        assert_eq!(*app.engine.project(), before);
        click(&mut app, &ctx, "dialog.add.video.clear", size);
        assert!(app.video_ui.history.paths().is_empty());
        assert!(Path::new(&path).is_file());
        assert_eq!(*app.engine.project(), before);
        click(&mut app, &ctx, "dialog.add.cancel", size);
        assert!(app.video_ui.path.is_empty());
        assert_eq!(*app.engine.project(), before);
    }
}
