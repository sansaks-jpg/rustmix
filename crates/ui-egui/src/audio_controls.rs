//! Operator-armed local audio. Discovery, file dialogs and WAV validation stay
//! off the UI thread; adding a descriptor never starts microphone capture.
use super::*;
use crate::audio_session::{AudioSession, SessionPhase, SessionStatus};
use rustmix_engine::wav::{MAX_WAV_BYTES, WavAudio};
use rustmix_platform::audio::{AudioDevice, Direction};
use std::io::Read;
use std::sync::mpsc::TryRecvError;

#[derive(Clone)]
struct WavInfo {
    path: String,
    sample_rate: u32,
    frames: usize,
}
struct Discovery {
    playback: std::result::Result<Vec<AudioDevice>, String>,
    capture: std::result::Result<Vec<AudioDevice>, String>,
}
struct WavResult {
    token: u64,
    result: std::result::Result<Option<WavInfo>, String>,
}

#[derive(Default)]
pub(crate) struct AudioUi {
    pub(crate) open: bool,
    session: Option<AudioSession>,
    status: Option<SessionStatus>,
    last_revision: Option<u64>,
    stopping: bool,
    output_device: Option<String>,
    playback_devices: Vec<AudioDevice>,
    capture_devices: Vec<AudioDevice>,
    discovery: Option<Receiver<Discovery>>,
    discovery_error: Option<String>,
    label: String,
    wav_path: String,
    microphone: Option<String>,
    wav_info: Option<WavInfo>,
    wav_job: Option<Receiver<WavResult>>,
    wav_job_token: u64,
    add_error: Option<String>,
}
impl AudioUi {
    pub(crate) fn master_peaks(&self) -> [f32; 2] {
        if self.monitoring_active() {
            self.status.as_ref().map_or([0.0; 2], |status| status.peak)
        } else {
            [0.0; 2]
        }
    }
    pub(crate) fn source_position(&self, id: InputId) -> Option<(u64, Option<u64>)> {
        if !self.monitoring_active() {
            return None;
        }
        let source = self
            .status
            .as_ref()?
            .sources
            .iter()
            .find(|source| source.id == id)?;
        let millis = |frames: u64| {
            frames.saturating_mul(1000) / u64::from(rustmix_engine::audio_engine::SAMPLE_RATE)
        };
        Some((
            millis(source.position_frames),
            (source.duration_frames > 0).then(|| millis(source.duration_frames)),
        ))
    }

    pub(crate) fn validated_wav_metadata(&self) -> Option<(u32, usize)> {
        self.wav_info
            .as_ref()
            .map(|info| (info.sample_rate, info.frames))
    }
    pub(crate) fn wav_validation_pending(&self) -> bool {
        self.wav_job.is_some()
    }
    pub(crate) fn wav_validation_error(&self) -> Option<&str> {
        self.add_error.as_deref()
    }
    pub(crate) fn is_active(&self) -> bool {
        self.session.is_some()
    }
    pub(crate) fn status_label(&self) -> &'static str {
        if self.stopping {
            return "stopping";
        }
        match self.status.as_ref().map(|status| status.phase) {
            Some(SessionPhase::Failed) => "failed",
            Some(SessionPhase::Stopped) => "off",
            Some(SessionPhase::Running) if self.is_active() => "running",
            Some(SessionPhase::Stopping) if self.is_active() => "stopping",
            _ if self.is_active() => "starting",
            _ => "off",
        }
    }
    pub(crate) fn monitoring_active(&self) -> bool {
        self.is_active()
            && !self.stopping
            && self
                .status
                .as_ref()
                .is_some_and(|status| status.phase == SessionPhase::Running)
    }
}

fn selected_device(devices: &[AudioDevice], selected: Option<&str>, default: &str) -> String {
    match selected {
        None => default.to_owned(),
        Some(id) => devices.iter().find(|device| device.id == id).map_or_else(
            || "Unavailable selection — refresh or select another device".into(),
            |device| device.label.clone(),
        ),
    }
}

fn validate_wav(path: PathBuf) -> std::result::Result<WavInfo, String> {
    let path_string = path
        .to_str()
        .ok_or_else(|| "WAV path must be valid text".to_owned())?
        .to_owned();
    AudioSource::WavFile {
        path: path_string.clone(),
    }
    .validate()
    .map_err(|error| error.to_string())?;
    let file = std::fs::File::open(path)
        .map_err(|error| format!("Could not open WAV ({:?})", error.kind()))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("Could not inspect WAV ({:?})", error.kind()))?;
    if !metadata.is_file() || metadata.len() > MAX_WAV_BYTES as u64 {
        return Err("Choose a regular WAV file of at most 64 MiB".into());
    }
    let mut bytes = Vec::new();
    let capacity = usize::try_from(metadata.len())
        .ok()
        .and_then(|length| length.checked_add(1))
        .ok_or_else(|| "WAV file length overflow".to_owned())?;
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| "WAV file allocation failed".to_owned())?;
    file.take(MAX_WAV_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read WAV ({:?})", error.kind()))?;
    let audio = WavAudio::decode(&bytes).map_err(|error| error.to_string())?;
    Ok(WavInfo {
        path: path_string,
        sample_rate: audio.sample_rate(),
        frames: audio.frames().len(),
    })
}

impl RustMixApp {
    pub(crate) fn poll_audio(&mut self, _ctx: &egui::Context) {
        if let Some(session) = &self.audio_ui.session {
            let revision = self.engine.project().revision;
            if self.audio_ui.last_revision != Some(revision) && !self.audio_ui.stopping {
                session.update(self.engine.clone());
                self.audio_ui.last_revision = Some(revision);
            }
            if let Some(status) = session.poll() {
                let finished = matches!(status.phase, SessionPhase::Stopped | SessionPhase::Failed);
                self.audio_ui.status = Some(status);
                if finished {
                    self.audio_ui.session = None;
                    self.audio_ui.stopping = false;
                    self.audio_ui.last_revision = None;
                }
            }
        }
        let discovery = self
            .audio_ui
            .discovery
            .as_ref()
            .and_then(|job| match job.try_recv() {
                Ok(result) => Some(Ok(result)),
                Err(TryRecvError::Disconnected) => {
                    Some(Err("Audio device discovery stopped unexpectedly".to_owned()))
                }
                Err(TryRecvError::Empty) => None,
            });
        if let Some(result) = discovery {
            self.audio_ui.discovery = None;
            match result {
                Ok(result) => {
                    let mut errors = Vec::new();
                    match result.playback {
                        Ok(devices) => self.audio_ui.playback_devices = devices,
                        Err(error) => {
                            self.audio_ui.playback_devices.clear();
                            errors.push(format!("Playback: {error}"));
                        }
                    }
                    match result.capture {
                        Ok(devices) => self.audio_ui.capture_devices = devices,
                        Err(error) => {
                            self.audio_ui.capture_devices.clear();
                            errors.push(format!("Capture: {error}"));
                        }
                    }
                    self.audio_ui.discovery_error = (!errors.is_empty()).then(|| errors.join("; "));
                }
                Err(error) => self.audio_ui.discovery_error = Some(error),
            }
        }
        let wav = self
            .audio_ui
            .wav_job
            .as_ref()
            .and_then(|job| match job.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Disconnected) => Some(WavResult {
                    token: self.audio_ui.wav_job_token,
                    result: Err("WAV worker stopped unexpectedly; choose the file again".into()),
                }),
                Err(TryRecvError::Empty) => None,
            });
        if let Some(result) = wav {
            self.audio_ui.wav_job = None;
            if self.add_open && self.add_tab == 12 && result.token == self.add_token {
                match result.result {
                    Ok(Some(info)) => {
                        self.audio_ui.wav_path = info.path.clone();
                        self.audio_ui.wav_info = Some(info);
                        self.audio_ui.add_error = None;
                    }
                    Ok(None) => {} // picker cancellation leaves the current draft intact
                    Err(error) => {
                        self.audio_ui.wav_info = None;
                        self.audio_ui.add_error = Some(error);
                    }
                }
            }
        }
    }

    fn refresh_audio_devices(&mut self, ctx: &egui::Context) {
        if self.audio_ui.discovery.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        let repaint = ctx.clone();
        let spawn = std::thread::Builder::new()
            .name("rustmix-audio-discovery".into())
            .spawn(move || {
                let result = Discovery {
                    playback: rustmix_platform::audio::devices(Direction::Playback)
                        .map_err(|error| error.to_string()),
                    capture: rustmix_platform::audio::devices(Direction::Capture)
                        .map_err(|error| error.to_string()),
                };
                let _ = sender.try_send(result);
                repaint.request_repaint();
            });
        if spawn.is_ok() {
            self.audio_ui.discovery = Some(receiver);
            self.audio_ui.discovery_error = None;
        } else {
            self.audio_ui.discovery_error = Some("Could not start audio device discovery".into());
        }
    }

    fn start_wav_job(&mut self, ctx: &egui::Context, picker: bool) {
        if self.audio_ui.wav_job.is_some() {
            return;
        }
        let path = self.audio_ui.wav_path.clone();
        let token = self.add_token;
        let (sender, receiver) = mpsc::sync_channel(1);
        let repaint = ctx.clone();
        let spawn = std::thread::Builder::new()
            .name("rustmix-wav-validation".into())
            .spawn(move || {
                let chosen = if picker {
                    rfd::FileDialog::new()
                        .add_filter("WAV audio", &["wav"])
                        .pick_file()
                } else {
                    Some(PathBuf::from(path))
                };
                let result = chosen.map(validate_wav).transpose();
                let _ = sender.try_send(WavResult { token, result });
                repaint.request_repaint();
            });
        if spawn.is_ok() {
            self.audio_ui.wav_job = Some(receiver);
            self.audio_ui.wav_job_token = token;
            self.audio_ui.add_error = None;
        } else {
            self.audio_ui.add_error = Some("Could not start WAV validation".into());
        }
    }

    pub(crate) fn cancel_audio_add(&mut self) {
        self.audio_ui.label.clear();
        self.audio_ui.wav_path.clear();
        self.audio_ui.microphone = None;
        self.audio_ui.wav_info = None;
        self.audio_ui.add_error = None;
    }

    pub(crate) fn stop_audio_for_project_change(&mut self) {
        if let Some(session) = &self.audio_ui.session {
            self.audio_ui.stopping = true;
            session.request_stop();
        } else {
            self.audio_ui.status = None;
            self.audio_ui.last_revision = None;
        }
        self.audio_ui.open = false;
        self.cancel_audio_add();
    }

    pub(crate) fn audio_add_ready(&self) -> bool {
        let label = self.audio_ui.label.trim();
        if label.len() > 64 || label.chars().any(char::is_control) {
            return false;
        }
        match self.add_tab {
            12 => {
                self.audio_ui.wav_job.is_none()
                    && self
                        .audio_ui
                        .wav_info
                        .as_ref()
                        .is_some_and(|info| info.path == self.audio_ui.wav_path)
            }
            13 => AudioSource::Microphone {
                device_id: self.audio_ui.microphone.clone(),
            }
            .validate()
            .is_ok(),
            _ => false,
        }
    }

    pub(crate) fn audio_add_page(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        super::add_input::page_header(
            ui,
            if self.add_tab == 12 {
                "Select a WAV audio file."
            } else {
                "Select a microphone."
            },
        );
        egui::Frame::new().inner_margin(8).show(ui, |ui| {
            if self.add_tab == 12 {
                ui.horizontal(|ui| {
                    let width = (ui.available_width()-94.0).max(120.0);
                    let path = ui.add_enabled_ui(self.audio_ui.wav_job.is_none(), |ui| ui.add_sized([width,26.0],
                        egui::TextEdit::singleline(&mut self.audio_ui.wav_path).char_limit(4096).hint_text("WAV audio file"))).inner;
                    self.automation.insert("dialog.add.audio.path".into(),path.rect);
                    if path.changed() { self.audio_ui.wav_info=None; self.audio_ui.add_error=None; }
                    let browse=ui.add_enabled(self.audio_ui.wav_job.is_none(),egui::Button::new("Browse…").min_size(Vec2::new(90.0,26.0)));
                    self.automation.insert("dialog.add.audio.browse".into(),browse.rect);
                    if browse.clicked() && self.smoke.is_none() { self.start_wav_job(ctx,true); }
                    if path.lost_focus() && ctx.input(|x|x.key_pressed(egui::Key::Enter)) && !self.audio_ui.wav_path.trim().is_empty() { self.start_wav_job(ctx,false); }
                });
                super::dialogs::light_form_frame(ui,10,|ui| {
                    ui.set_min_width((ui.available_width()-2.0).max(80.0));
                    ui.label("WAV audio · PCM16 or float32 · mono/stereo · up to 120 seconds.");
                    if self.audio_ui.wav_job.is_some() { ui.label("Checking audio file…"); }
                    else if let Some(info)=&self.audio_ui.wav_info {
                        ui.label(format!("{} Hz · {:.2} seconds · Ready",info.sample_rate,info.frames as f64 / f64::from(info.sample_rate)));
                    } else { ui.label("Choose a file with Browse, or enter a path and select Check file."); }
                    ui.label("MP3 and compressed WAV are unavailable.");
                });
                let validate=ui.add_enabled(self.audio_ui.wav_job.is_none() && !self.audio_ui.wav_path.trim().is_empty(),egui::Button::new("Check file"));
                self.automation.insert("dialog.add.audio.validate".into(),validate.rect);
                if validate.clicked() { self.start_wav_job(ctx,false); }
            } else {
            let label = selected_device(
                &self.audio_ui.capture_devices,
                self.audio_ui.microphone.as_deref(),
                "System default microphone",
            );
            let picker = egui::ComboBox::from_id_salt("dialog.add.audio.capture")
                .selected_text(label)
                .width(ui.available_width().min(300.0))
                .show_ui(ui, |ui| {
                    let default = ui.selectable_value(
                        &mut self.audio_ui.microphone,
                        None,
                        "System default microphone",
                    );
                    self.automation
                        .insert("dialog.add.audio.capture.default".into(), default.rect);
                    for (index, device) in self.audio_ui.capture_devices.iter().enumerate() {
                        let response = ui.selectable_value(
                            &mut self.audio_ui.microphone,
                            Some(device.id.clone()),
                            &device.label,
                        );
                        self.automation
                            .insert(format!("dialog.add.audio.capture.{index}"), response.rect);
                    }
                });
            self.automation
                .insert("dialog.add.audio.capture".into(), picker.response.rect);
            self.audio_refresh_button(ui, ctx, "dialog.add.audio.refresh");

                ui.label("Adding a microphone keeps capture off. Start monitoring explicitly in Audio I/O.");
                if let Some(error)=&self.audio_ui.discovery_error { ui.colored_label(RED,error); }
            }
            ui.horizontal(|ui| {
                ui.label("Name");
                let width=ui.available_width();
                let name=ui.add_sized([width,26.0],egui::TextEdit::singleline(&mut self.audio_ui.label).char_limit(64)
                    .hint_text(if self.add_tab==12 {"Audio file"} else {"Microphone"}));
                self.automation.insert("dialog.add.audio.name".into(),name.rect);
            });
            ui.label("Audio-only input · black video slate. Adding never starts monitoring.");
            if let Some(error)=&self.audio_ui.add_error {
                let label=ui.add(egui::Label::new(RichText::new(error).color(RED)).truncate()).on_hover_text(error);
                self.automation.insert("dialog.add.audio.error".into(),label.rect);
            }
        });
    }

    pub(crate) fn confirm_audio_input(&mut self, ctx: &egui::Context) -> bool {
        let source = if self.add_tab == 12 {
            let Some(info) = &self.audio_ui.wav_info else {
                self.audio_ui.add_error =
                    Some("Validate a supported WAV file before adding it".into());
                return false;
            };
            if info.path != self.audio_ui.wav_path || self.audio_ui.wav_job.is_some() {
                self.audio_ui.add_error =
                    Some("Wait for the current WAV file to finish validation".into());
                return false;
            }
            AudioSource::WavFile {
                path: info.path.clone(),
            }
        } else if self.add_tab == 13 {
            AudioSource::Microphone {
                device_id: self.audio_ui.microphone.clone(),
            }
        } else {
            return false;
        };
        if let Err(error) = source.validate() {
            self.audio_ui.add_error = Some(error.to_string());
            return false;
        }
        let default_label = if self.add_tab == 12 {
            "Audio file"
        } else {
            "Microphone"
        };
        let label = if self.audio_ui.label.trim().is_empty() {
            default_label
        } else {
            self.audio_ui.label.trim()
        }
        .to_owned();
        let config = InputConfig {
            audio_source: Some(source),
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
            self.audio_ui.add_error = Some(self.notice.clone());
            return false;
        }
        self.submit_program();
        // The coordinator has no microphone open until the explicit session Start.
        // Sending a descriptor update does not implicitly arm a newly added mic.
        if let Some(session) = &self.audio_ui.session {
            session.update(self.engine.clone());
        }
        self.cancel_audio_add();
        self.add_open = false;
        ctx.request_repaint();
        true
    }

    fn audio_refresh_button(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, id: &str) {
        let refresh = ui.add_enabled(
            self.audio_ui.discovery.is_none(),
            egui::Button::new("Refresh devices"),
        );
        self.automation.insert(id.into(), refresh.rect);
        if refresh.clicked() {
            self.refresh_audio_devices(ctx);
        }
        if self.audio_ui.discovery.is_some() {
            ui.label("Discovering devices…");
        }
        if let Some(error) = &self.audio_ui.discovery_error {
            ui.colored_label(RED, error);
        }
    }

    pub(crate) fn audio_io_dialog(&mut self, ctx: &egui::Context) {
        if !self.audio_ui.open {
            return;
        }
        let mut open = true;
        let mut start = false;
        let mut stop = false;
        let mut close = false;
        dialogs::operator_window(ctx, "Audio I/O · local monitoring", "dialog.audio_io", &mut open, Vec2::new(740.0, 560.0))
            .show(ctx, |ui| {
                let height = (ctx.viewport_rect().height() - 200.0).clamp(160.0, 400.0);
                let (body, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
                let mut body_ui = ui.new_child(egui::UiBuilder::new().max_rect(body).layout(egui::Layout::top_down(egui::Align::Min)));
                body_ui.set_clip_rect(body.intersect(ui.clip_rect()));
                egui::ScrollArea::vertical().max_height(height).show(&mut body_ui, |ui| {
                    ui.label("Real local Master audio · 48 kHz stereo. The main mixer controls gain/mute/sends and shows measured Master peaks. Per-input peak metering is unavailable.");
                    ui.colored_label(PREVIEW, "Start monitoring opens the selected speakers/headphones and all configured microphones. Use headphones to avoid feedback. Closing this dialog keeps monitoring running; Stop releases the devices.");
                    let active = self.audio_ui.is_active();
                    let picker = egui::ComboBox::from_id_salt("dialog.audio_io.output")
                        .width(360.0).selected_text(selected_device(&self.audio_ui.playback_devices, self.audio_ui.output_device.as_deref(), "System default playback device"))
                        .show_ui(ui, |ui| {
                            ui.add_enabled_ui(!active, |ui| {
                                let response = ui.selectable_value(&mut self.audio_ui.output_device, None, "System default playback device");
                                self.automation.insert("dialog.audio_io.output.default".into(), response.rect);
                                for (index, device) in self.audio_ui.playback_devices.iter().enumerate() {
                                    let response = ui.selectable_value(&mut self.audio_ui.output_device, Some(device.id.clone()), &device.label);
                                    self.automation.insert(format!("dialog.audio_io.output.{index}"), response.rect);
                                }
                            });
                        });
                    self.automation.insert("dialog.audio_io.output".into(), picker.response.rect);
                    if active { ui.label("Stop monitoring before changing playback devices."); }
                    self.audio_refresh_button(ui, ctx, "dialog.audio_io.refresh");
                    let phase = self.audio_ui.status_label();
                    let label = ui.label(format!("Monitoring state: {phase}"));
                    self.automation.insert("dialog.audio_io.state".into(), label.rect);
                    if let Some(status) = &self.audio_ui.status {
                        if let Some(error) = &status.error {
                            let label = ui.colored_label(RED, error);
                            self.automation.insert("dialog.audio_io.error".into(), label.rect);
                        }
                        ui.label(format!("Actual device callback frames: {} · mixed frames: {}", status.callback_frames, status.mixed_frames));
                        ui.label(format!("Playback underruns: {} · capture dropped/missing: {}/{} · queue drops: {}", status.playback_underruns, status.capture_dropped_frames, status.capture_missing_frames, status.backpressure_dropped_frames));
                        ui.label(format!("Rejected samples: {} · clipped samples: {}", status.rejected_samples, status.clipped_samples));
                        ui.label("Actual Master block peak / RMS (linear sample amplitude; not loudness or true peak)");
                        for ((peak, rms), channel) in status.peak.iter().zip(status.rms).zip(["L", "R"]) {
                            ui.label(format!("{channel}: peak {peak:.4} · RMS {rms:.4}"));
                        }
                        for source in &status.sources {
                            ui.label(format!("Input {} · ready {} · capturing {} · paused {} · ended {} · 48 kHz frame {}/{}", source.id.0, source.ready, source.capturing, source.paused, source.ended, source.position_frames, source.duration_frames));
                            if let Some(error) = &source.error { ui.colored_label(RED, error); }
                        }
                    }
                });
                self.automation.insert("dialog.audio_io.body".into(), body);
                ui.separator();
                ui.horizontal(|ui| {
                    let response = ui.add_enabled(self.audio_ui.session.is_none(), egui::Button::new("Start monitoring").min_size(Vec2::new(126.0, 29.0)));
                    self.automation.insert("dialog.audio_io.start".into(), response.rect);
                    start = response.clicked();
                    let response = ui.add_enabled(self.audio_ui.session.is_some() && !self.audio_ui.stopping, egui::Button::new("Stop").min_size(Vec2::new(92.0, 29.0)));
                    self.automation.insert("dialog.audio_io.stop".into(), response.rect);
                    stop = response.clicked();
                    let response = ui.add(egui::Button::new("Close").min_size(Vec2::new(92.0, 29.0)));
                    self.automation.insert("dialog.audio_io.close".into(), response.rect);
                    close = response.clicked();
                });
            });
        if start && self.audio_ui.session.is_none() {
            self.audio_ui.status = None;
            self.audio_ui.stopping = false;
            self.audio_ui.last_revision = Some(self.engine.project().revision);
            self.audio_ui.session = Some(AudioSession::spawn(
                self.engine.clone(),
                self.audio_ui.output_device.clone(),
                ctx.clone(),
            ));
        }
        if stop && let Some(session) = &self.audio_ui.session {
            self.audio_ui.stopping = true;
            session.request_stop();
        }
        self.audio_ui.open = open && !close;
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
                .expect("Visible audio operator control");
            if previous.is_some_and(|old| {
                old.center().distance(bounds.center()) < 0.5
                    && (old.size() - bounds.size()).length() < 0.5
            }) {
                stable = Some(bounds);
                break;
            }
            previous = Some(bounds);
        }
        let bounds = stable.expect("Audio control must settle within six frames");
        assert!(
            ctx.viewport_rect().contains_rect(bounds),
            "Audio control outside viewport: {id}"
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
    fn valid_wav() -> Vec<u8> {
        let samples = [1200_i16, -1200, 2400, -2400, 600, -600, 0, 0];
        let data_len = u32::try_from(samples.len() * 2).unwrap();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&48_000_u32.to_le_bytes());
        bytes.extend_from_slice(&192_000_u32.to_le_bytes());
        bytes.extend_from_slice(&4_u16.to_le_bytes());
        bytes.extend_from_slice(&16_u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        bytes
    }
    fn wait_for_wav(app: &mut RustMixApp, ctx: &egui::Context, size: [f32; 2]) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while app.audio_ui.wav_job.is_some() {
            assert!(Instant::now() < deadline, "WAV worker did not finish");
            frame(app, ctx, vec![], size);
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn microphone_widgets_cancel_and_confirm_keep_program_and_never_open_capture() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        let before = app.engine.project().clone();
        app.audio_ui.capture_devices = vec![AudioDevice {
            id: "capture:test-stable-id".into(),
            label: "Test microphone".into(),
            direction: Direction::Capture,
        }];
        app.dispatch(Action::Add, &ctx);
        click(&mut app, &ctx, "dialog.add.audio.microphone", size);
        click(&mut app, &ctx, "dialog.add.audio.capture", size);
        click(&mut app, &ctx, "dialog.add.audio.capture.0", size);
        click(&mut app, &ctx, "dialog.add.cancel", size);
        assert_eq!(*app.engine.project(), before);
        assert!(app.audio_ui.session.is_none());
        assert!(app.audio_ui.microphone.is_none());

        app.dispatch(Action::Add, &ctx);
        click(&mut app, &ctx, "dialog.add.audio.microphone", size);
        click(&mut app, &ctx, "dialog.add.audio.name", size);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Text("Guest microphone".into())],
            size,
        );
        click(&mut app, &ctx, "dialog.add.audio.capture", size);
        click(&mut app, &ctx, "dialog.add.audio.capture.0", size);
        click(&mut app, &ctx, "dialog.add.confirm", size);
        assert!(!app.add_open);
        let added = app.engine.project().inputs.last().unwrap();
        assert_eq!(added.name, "Guest microphone");
        assert_eq!(added.kind, InputKind::Color);
        assert_eq!(added.config.media_path, None);
        assert_eq!(
            added.config.audio_source,
            Some(AudioSource::Microphone {
                device_id: Some("capture:test-stable-id".into())
            })
        );
        assert_eq!(app.engine.project().revision, before.revision + 1);
        assert_eq!(app.engine.project().program, before.program);
        assert_eq!(app.engine.project().preview, before.preview);
        assert!(
            app.audio_ui.session.is_none(),
            "Adding a descriptor must not open a microphone"
        );
        assert!(
            app.audio_ui.discovery.is_none(),
            "Opening Add does not enumerate devices implicitly"
        );
    }

    #[test]
    fn wav_widgets_validate_off_thread_insert_at_selected_position_and_reject_corrupt_source() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = [1600.0, 900.0];
        let before = app.engine.project().clone();
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("procedural-tone.wav");
        std::fs::write(&path, valid_wav()).unwrap();
        app.dispatch(Action::Add, &ctx);
        app.add_number = 3;
        click(&mut app, &ctx, "dialog.add.audio.wav", size);
        click(&mut app, &ctx, "dialog.add.audio.path", size);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Text(path.to_str().unwrap().to_owned())],
            size,
        );
        click(&mut app, &ctx, "dialog.add.confirm", size);
        assert_eq!(*app.engine.project(), before);
        assert!(!app.audio_add_ready());
        click(&mut app, &ctx, "dialog.add.audio.validate", size);
        wait_for_wav(&mut app, &ctx, size);
        let info = app
            .audio_ui
            .wav_info
            .as_ref()
            .expect("WAV worker must validate supported PCM");
        assert_eq!(info.sample_rate, 48_000);
        assert_eq!(info.frames, 4);
        assert_eq!(
            *app.engine.project(),
            before,
            "Validation alone cannot insert or route a source"
        );
        click(&mut app, &ctx, "dialog.add.confirm", size);
        let project = app.engine.project();
        let added = &project.inputs[2];
        assert_eq!(added.id, InputId(8));
        assert_eq!(
            added.config.audio_source,
            Some(AudioSource::WavFile {
                path: path.to_str().unwrap().into()
            })
        );
        assert!(added.config.media_path.is_none());
        assert_eq!(project.inputs[3].id, before.inputs[2].id);
        assert_eq!(project.revision, before.revision + 1);
        assert_eq!(
            (project.program, project.preview),
            (before.program, before.preview)
        );
        assert!(app.audio_ui.session.is_none());

        let accepted = project.clone();
        let corrupt = folder.path().join("corrupt.wav");
        std::fs::write(&corrupt, b"not a WAV file").unwrap();
        app.dispatch(Action::Add, &ctx);
        click(&mut app, &ctx, "dialog.add.audio.wav", size);
        click(&mut app, &ctx, "dialog.add.audio.path", size);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Text(corrupt.to_str().unwrap().to_owned())],
            size,
        );
        click(&mut app, &ctx, "dialog.add.audio.validate", size);
        wait_for_wav(&mut app, &ctx, size);
        click(&mut app, &ctx, "dialog.add.confirm", size);
        assert!(app.audio_ui.add_error.is_some());
        assert!(app.audio_ui.wav_info.is_none());
        assert_eq!(*app.engine.project(), accepted);
        click(&mut app, &ctx, "dialog.add.cancel", size);
    }

    #[test]
    fn late_wav_result_after_cancel_cannot_restore_draft_or_add_source() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        let before = app.engine.project().clone();
        let old_token = app.add_token;
        let (sender, receiver) = mpsc::sync_channel(1);
        app.audio_ui.wav_job = Some(receiver);
        app.add_open = true;
        app.add_tab = 12;
        app.cancel_audio_add();
        app.add_open = false;
        app.add_token = app.add_token.wrapping_add(1);
        sender
            .send(WavResult {
                token: old_token,
                result: Ok(Some(WavInfo {
                    path: "stale.wav".into(),
                    sample_rate: 48_000,
                    frames: 100,
                })),
            })
            .unwrap();
        app.poll_audio(&ctx);
        assert!(app.audio_ui.wav_job.is_none() && app.audio_ui.wav_info.is_none());
        assert_eq!(*app.engine.project(), before);
        assert!(app.audio_ui.session.is_none());
    }

    #[test]
    fn audio_io_footer_and_error_stay_visible_without_devices_or_fabricated_session() {
        for size in [[734.0, 466.0], [1100.0, 700.0], [1600.0, 900.0]] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            let before = app.engine.project().clone();
            app.audio_ui.open = true;
            app.audio_ui.status = Some(SessionStatus {
                phase: SessionPhase::Failed,
                callback_frames: 1024,
                mixed_frames: 1024,
                playback_underruns: 2,
                capture_dropped_frames: 0,
                capture_missing_frames: 0,
                backpressure_dropped_frames: 0,
                rejected_samples: 0,
                clipped_samples: 0,
                peak: [0.25, 0.5],
                rms: [0.125, 0.25],
                sources: Vec::new(),
                error: Some("Selected playback device unavailable; refresh and retry".into()),
            });
            for _ in 0..4 {
                frame(&mut app, &ctx, vec![], size);
            }
            let body = app.automation["dialog.audio_io.body"];
            for id in [
                "dialog.audio_io.start",
                "dialog.audio_io.stop",
                "dialog.audio_io.close",
            ] {
                let footer = app.automation[id];
                assert!(ctx.viewport_rect().contains_rect(footer));
                assert!(body.bottom() <= footer.top());
                assert!(footer.height() >= 27.0);
            }
            assert!(app.automation.contains_key("dialog.audio_io.error"));
            click(&mut app, &ctx, "dialog.audio_io.stop", size);
            assert!(app.audio_ui.session.is_none());
            assert_eq!(app.audio_ui.status.as_ref().unwrap().callback_frames, 1024);
            click(&mut app, &ctx, "dialog.audio_io.close", size);
            assert!(!app.audio_ui.open);
            assert_eq!(*app.engine.project(), before);
            app.audio_ui.open = true;
            app.audio_ui.label = "Discard this draft".into();
            app.stop_audio_for_project_change();
            assert!(!app.audio_ui.open && app.audio_ui.label.is_empty());
            assert_eq!(*app.engine.project(), before);
        }
    }
}
