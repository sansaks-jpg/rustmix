#![forbid(unsafe_code)]
mod panels;
mod smoke;
mod workspace;

use eframe::egui::{self, Color32, RichText};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use rustmix_engine::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

pub(crate) const TEAL: Color32 = Color32::from_rgb(56, 190, 113);
pub(crate) const PREVIEW: Color32 = Color32::from_rgb(244, 143, 0);
pub(crate) const PROGRAM: Color32 = Color32::from_rgb(0, 116, 0);
pub(crate) const RED: Color32 = Color32::from_rgb(241, 100, 105);
pub(crate) const DIM: Color32 = Color32::from_rgb(141, 155, 171);
pub(crate) const BG: Color32 = Color32::from_rgb(29, 35, 39);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Panel {
    Inputs,
    Audio,
    Overlays,
}
impl Panel {
    fn title(self) -> &'static str {
        match self {
            Self::Inputs => "Inputs",
            Self::Audio => "Audio Mixer · simulated",
            Self::Overlays => "Overlay / Graphics",
        }
    }
}
fn default_dock(audio_focus: bool) -> DockState<Panel> {
    let mut dock = DockState::new(vec![Panel::Inputs, Panel::Overlays]);
    dock.main_surface_mut().split_right(
        NodeIndex::root(),
        if audio_focus { 0.59 } else { 0.80 },
        vec![Panel::Audio],
    );
    dock
}
pub(crate) enum Action {
    Engine(Command),
    New,
    Open,
    Save(bool),
    Add,
    Properties(InputId),
    Settings,
    About,
    Workspace(bool),
    Multiview,
    Fullscreen,
    Close,
    Recover,
    ConfirmOpen,
    CloseDiscard,
    FocusPanel(Panel),
    PauseDemo,
    Last,
}
enum JobDone {
    Saved(PathBuf, u64, bool),
    Loaded(Box<ProjectState>, PathBuf),
    Cancelled,
    Failed(String),
}
pub struct RustMixApp {
    pub(crate) engine: Engine,
    dock: DockState<Panel>,
    pub(crate) actions: Vec<Action>,
    pub(crate) start: Instant,
    pub(crate) now: u64,
    pub(crate) textures: HashMap<InputId, egui::TextureHandle>,
    pub(crate) program_texture: Option<egui::TextureHandle>,
    last_demo_frame: Option<u64>,
    pub(crate) notice: String,
    pub(crate) warning: bool,
    pub(crate) history: VecDeque<String>,
    path: Option<PathBuf>,
    saved_revision: u64,
    job: Option<Receiver<JobDone>>,
    last_autosave: Instant,
    pub(crate) add_open: bool,
    pub(crate) add_name: String,
    pub(crate) add_kind: InputKind,
    pub(crate) properties: Option<InputState>,
    pub(crate) settings: bool,
    pub(crate) about: bool,
    pub(crate) multiview: bool,
    pub(crate) fullscreen: bool,
    pub(crate) confirm_new: bool,
    pub(crate) confirm_close: bool,
    pub(crate) confirm_open: bool,
    pub(crate) allow_close: bool,
    pub(crate) pending_open: Option<PathBuf>,
    pub(crate) fps: f32,
    fps_start: Instant,
    fps_frames: u32,
    pub(crate) generation_ms: f32,
    pub(crate) selected_audio: Option<InputId>,
    pub(crate) input_filter: String,
    pub(crate) smoke: Option<smoke::Smoke>,
    pub(crate) automation: HashMap<String, egui::Rect>,
    pub(crate) demo_paused: bool,
    pub(crate) paused_at: u64,
    pub(crate) category: Option<InputKind>,
    pub(crate) tbar_latched: bool,
    monitor_view_height: Option<f32>,
}
impl RustMixApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::headless();
        if let Some(storage) = cc.storage
            && let Some(json) = storage.get_string("workspace.v2")
        {
            match workspace::decode(&json) {
                Ok(dock) => app.dock = dock,
                Err(_) => {
                    app.notice = "Saved workspace was invalid; using Production layout".into()
                }
            }
        }
        if let Some(path) = std::env::args()
            .skip(1)
            .find(|s| s.ends_with(".rustmix.json"))
        {
            app.start_load(Some(PathBuf::from(path)));
        }
        if std::env::args().any(|s| s == "--ui-smoke" || s == "--ui-smoke-reopen") {
            app.smoke = Some(smoke::Smoke::default());
        }
        app.theme(&cc.egui_ctx);
        #[cfg(target_os = "windows")]
        if let Some(windows) = std::env::var_os("WINDIR") {
            let font_path = PathBuf::from(windows).join("Fonts").join("segoeui.ttf");
            if std::fs::metadata(&font_path).is_ok_and(|m| m.len() < 5_000_000)
                && let Ok(bytes) = std::fs::read(&font_path)
            {
                let mut fonts = egui::FontDefinitions::default();
                fonts.font_data.insert(
                    "windows-ui".into(),
                    std::sync::Arc::new(egui::FontData::from_owned(bytes)),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "windows-ui".into());
                cc.egui_ctx.set_fonts(fonts);
            }
        }
        app
    }
    fn headless() -> Self {
        Self {
            engine: Engine::default(),
            dock: default_dock(false),
            actions: Vec::new(),
            start: Instant::now(),
            now: 0,
            textures: HashMap::new(),
            program_texture: None,
            last_demo_frame: None,
            notice: "Offline demo ready · no hardware or network outputs".into(),
            warning: false,
            history: VecDeque::new(),
            path: None,
            saved_revision: 0,
            job: None,
            last_autosave: Instant::now(),
            add_open: false,
            add_name: "New synthetic input".into(),
            add_kind: InputKind::Grid,
            properties: None,
            settings: false,
            about: false,
            multiview: false,
            fullscreen: false,
            confirm_new: false,
            confirm_close: false,
            confirm_open: false,
            allow_close: false,
            pending_open: None,
            fps: 0.0,
            fps_start: Instant::now(),
            fps_frames: 0,
            generation_ms: 0.0,
            selected_audio: Some(InputId(1)),
            input_filter: String::new(),
            smoke: None,
            automation: HashMap::new(),
            demo_paused: false,
            paused_at: 0,
            category: None,
            tbar_latched: false,
            monitor_view_height: None,
        }
    }
    fn theme(&self, ctx: &egui::Context) {
        let mut style = egui::Style {
            visuals: egui::Visuals::dark(),
            ..Default::default()
        };
        style.spacing.item_spacing = egui::vec2(5.0, 4.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        style.visuals.panel_fill = BG;
        style.visuals.window_fill = Color32::from_rgb(34, 41, 46);
        style.visuals.extreme_bg_color = Color32::from_rgb(12, 16, 21);
        style.visuals.faint_bg_color = Color32::from_rgb(34, 41, 46);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(63, 75, 85);
        style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(43, 52, 60);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(81, 96, 107);
        style.visuals.selection.bg_fill = Color32::from_rgb(27, 105, 111);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, TEAL);
        style.visuals.override_text_color =
            Some(if self.engine.project().preferences.high_contrast {
                Color32::WHITE
            } else {
                Color32::from_rgb(219, 227, 235)
            });
        for v in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
        ] {
            v.corner_radius = egui::CornerRadius::ZERO;
        }
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(12.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(12.0));
        ctx.set_theme(egui::ThemePreference::Dark);
        ctx.set_style_of(egui::Theme::Dark, style);
        ctx.set_zoom_factor(self.engine.project().preferences.ui_scale);
    }
    pub(crate) fn command_button(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        label: &str,
        c: Command,
        color: Option<Color32>,
    ) -> egui::Response {
        let enabled = self.engine.enabled(&c);
        let definition = c.definition();
        let mut button = egui::Button::new(RichText::new(label).strong());
        if let Some(color) = color {
            button = button.fill(color).stroke(egui::Stroke::NONE);
        }
        button = button.min_size(egui::vec2(ui.available_width().clamp(80.0, 160.0), 28.0));
        let response = ui
            .push_id(id, |ui| ui.add_enabled(enabled.is_ok(), button))
            .inner
            .on_hover_text(format!(
                "{}\n{} · {}\n{}",
                definition.label,
                definition.id,
                definition.shortcut,
                enabled.err().map(|e| e.to_string()).unwrap_or_default()
            ));
        self.automation.insert(id.into(), response.rect);
        if response.clicked() {
            self.actions.push(Action::Engine(c));
        }
        response
    }
    fn say(&mut self, message: String, error: bool) {
        self.notice = message.clone();
        self.warning = error;
        self.history.push_back(message);
        while self.history.len() > 32 {
            self.history.pop_front();
        }
    }
    fn execute(&mut self, c: Command) {
        let label = c.definition().label;
        match self.engine.execute(c, self.now) {
            Ok(_) => self.say(format!("{label} completed"), false),
            Err(e) => self.say(e.to_string(), true),
        }
        self.last_demo_frame = None;
    }
    fn recovery_path() -> PathBuf {
        let base = std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("XDG_DATA_HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        base.join("RustMix").join("recovery.rustmix.json")
    }
    fn capture_workspace(&mut self) {
        match serde_json::to_string(&self.dock) {
            Ok(json) => {
                let mut w = self.engine.project().workspace.clone();
                w.dock_json = json;
                if w != self.engine.project().workspace {
                    self.execute(Command::SetWorkspace(w));
                }
            }
            Err(e) => self.say(format!("Workspace could not be serialized: {e}"), true),
        }
    }
    fn start_save(&mut self, save_as: bool, auto: bool) {
        if self.job.is_some() {
            self.say("A file operation is already running".into(), true);
            return;
        }
        self.capture_workspace();
        let project = self.engine.project().clone();
        let revision = project.revision;
        let path = if auto {
            Some(Self::recovery_path())
        } else if save_as {
            None
        } else {
            self.path.clone()
        };
        let (tx, rx) = mpsc::sync_channel(1);
        self.job = Some(rx);
        std::thread::spawn(move || {
            let path = path.or_else(|| {
                rfd::FileDialog::new()
                    .add_filter("RustMix project", &["json"])
                    .set_file_name("production.rustmix.json")
                    .save_file()
            });
            let result = if let Some(path) = path {
                let setup = if auto {
                    path.parent().map(std::fs::create_dir_all).transpose()
                } else {
                    Ok(None)
                };
                match setup
                    .and_then(|_| persistence::save(&path, &project).map_err(std::io::Error::other))
                {
                    Ok(()) => JobDone::Saved(path, revision, auto),
                    Err(e) => JobDone::Failed(e.to_string()),
                }
            } else {
                JobDone::Cancelled
            };
            let _ = tx.send(result);
        });
    }
    fn start_load(&mut self, path: Option<PathBuf>) {
        if self.job.is_some() {
            return;
        }
        let (tx, rx) = mpsc::sync_channel(1);
        self.job = Some(rx);
        std::thread::spawn(move || {
            let path = path.or_else(|| {
                rfd::FileDialog::new()
                    .add_filter("RustMix project", &["json"])
                    .pick_file()
            });
            let result = if let Some(path) = path {
                match persistence::load(&path) {
                    Ok(p) => JobDone::Loaded(Box::new(p), path),
                    Err(e) => JobDone::Failed(e.to_string()),
                }
            } else {
                JobDone::Cancelled
            };
            let _ = tx.send(result);
        });
    }
    fn poll_job(&mut self, ctx: &egui::Context) {
        let result = self.job.as_ref().map(Receiver::try_recv);
        match result {
            Some(Ok(done)) => {
                self.job = None;
                match done {
                    JobDone::Saved(path, revision, auto) => {
                        if !auto {
                            self.path = Some(path);
                            self.saved_revision = revision;
                        }
                        self.say(
                            if auto {
                                "Recovery checkpoint saved".into()
                            } else {
                                "Project saved atomically · two backup generations retained".into()
                            },
                            false,
                        );
                    }
                    JobDone::Loaded(project, path) => match Engine::from_project(*project) {
                        Ok(engine) => {
                            let layout = engine.project().workspace.dock_json.clone();
                            self.engine = engine;
                            self.last_demo_frame = None;
                            self.theme(ctx);
                            self.path = Some(path);
                            self.saved_revision = self.engine.project().revision;
                            self.textures.clear();
                            self.properties = None;
                            if !layout.is_empty() {
                                match workspace::decode(&layout) {
                                    Ok(dock) => self.dock = dock,
                                    Err(_) => {
                                        self.dock = default_dock(false);
                                        self.say(
                                            "Project loaded; invalid layout reset".into(),
                                            true,
                                        );
                                        return;
                                    }
                                }
                            }
                            self.last_demo_frame = None;
                            self.say(
                                "Project reopened · transient transitions and FTB reset".into(),
                                false,
                            );
                        }
                        Err(e) => self.say(e.to_string(), true),
                    },
                    JobDone::Cancelled => self.say("File operation cancelled".into(), false),
                    JobDone::Failed(e) => self.say(format!("File operation failed: {e}"), true),
                }
            }
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                self.job = None;
                self.say(
                    "File worker ended unexpectedly; current project retained".into(),
                    true,
                );
            }
            _ => {}
        }
    }
    fn dispatch(&mut self, action: Action, ctx: &egui::Context) {
        match action {
            Action::Engine(c) => {
                let appearance = matches!(c, Command::SetPreferences(_));
                self.execute(c);
                if appearance {
                    self.theme(ctx);
                }
            }
            Action::New => {
                self.confirm_new = true;
            }
            Action::Open => {
                self.pending_open = None;
                if self.saved_revision != self.engine.project().revision {
                    self.confirm_open = true;
                } else {
                    self.start_load(None);
                }
            }
            Action::ConfirmOpen => {
                self.confirm_open = false;
                let path = self.pending_open.take();
                self.start_load(path);
            }
            Action::Save(as_new) => self.start_save(as_new, false),
            Action::Add => self.add_open = true,
            Action::Properties(id) => self.properties = self.engine.project().input(id).cloned(),
            Action::Settings => self.settings = true,
            Action::About => self.about = true,
            Action::Workspace(audio) => {
                self.dock = default_dock(audio);
                self.execute(Command::SetWorkspace(WorkspaceState {
                    name: if audio {
                        "Audio focus".into()
                    } else {
                        "Production".into()
                    },
                    dock_json: String::new(),
                }));
            }
            Action::Multiview => self.multiview = !self.multiview,
            Action::Fullscreen => {
                self.fullscreen = !self.fullscreen;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
            }
            Action::Close => self.confirm_close = true,
            Action::Recover => {
                self.pending_open = Some(Self::recovery_path());
                if self.saved_revision != self.engine.project().revision {
                    self.confirm_open = true;
                } else {
                    let path = self.pending_open.take();
                    self.start_load(path);
                }
            }
            Action::CloseDiscard => {
                self.allow_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Action::FocusPanel(panel) => {
                if let Some(path) = self.dock.find_tab(&panel) {
                    match self.dock.set_active_tab(path) {
                        Ok(()) => self.dock.set_focused_node_and_surface(path.node_path()),
                        Err(e) => self.say(format!("Cannot focus panel: {e:?}"), true),
                    }
                }
            }
            Action::PauseDemo => {
                self.demo_paused = !self.demo_paused;
                self.paused_at = self.now;
                self.last_demo_frame = None;
            }
            Action::Last => {
                if let Some(path) = self.path.clone() {
                    self.pending_open = Some(path);
                    self.confirm_open = true;
                }
            }
        }
    }
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("RustMix").strong())
                .on_hover_text(&self.engine.project().name)
                .clicked()
            {
                self.actions.push(Action::About);
            }
            for (label, action, hint) in [
                ("New", Action::New, "New demo project"),
                ("Open", Action::Open, "Open project · Ctrl+O"),
                ("Save", Action::Save(false), "Save · Ctrl+S"),
                ("Save As", Action::Save(true), "Save a copy · Ctrl+Shift+S"),
            ] {
                if ui
                    .add_enabled(
                        self.job.is_none(),
                        egui::Button::new(label).min_size(egui::vec2(66.0, 23.0)),
                    )
                    .on_hover_text(hint)
                    .clicked()
                {
                    self.actions.push(action);
                }
            }
            if ui
                .add_enabled(
                    self.path.is_some() && self.job.is_none(),
                    egui::Button::new("Last"),
                )
                .on_disabled_hover_text("Save or open a project first")
                .clicked()
            {
                self.actions.push(Action::Last);
            }
            ui.add_space((ui.max_rect().width() * 0.5 - ui.cursor().left() - 65.0).max(5.0));
            if ui.button("Fullscreen").clicked() {
                self.actions.push(Action::Fullscreen);
            }
            ui.menu_button("View", |ui| {
                for (label, action) in [
                    ("Multiview", Action::Multiview),
                    ("Production layout", Action::Workspace(false)),
                    ("Audio layout", Action::Workspace(true)),
                    ("Audio Mixer", Action::FocusPanel(Panel::Audio)),
                    ("Overlay / Graphics", Action::FocusPanel(Panel::Overlays)),
                ] {
                    if ui.button(label).clicked() {
                        self.actions.push(action);
                        ui.close();
                    }
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("?").clicked() {
                    self.actions.push(Action::About);
                }
                if ui.button("Settings").clicked() {
                    self.actions.push(Action::Settings);
                }
                if ui
                    .selectable_label(self.demo_paused, "Pause Inputs")
                    .on_hover_text(
                        "Freeze the procedural demo source clock. Transitions remain responsive.",
                    )
                    .clicked()
                {
                    self.actions.push(Action::PauseDemo);
                }
                ui.label(RichText::new("SYNTHETIC / OFFLINE").small().color(DIM));
            });
        });
    }
    fn production_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Add Input  +").clicked() {
                self.actions.push(Action::Add);
            }
            ui.separator();
            for (name, reason) in [
                ("Record", "No recording or recoverable encoder backend"),
                ("External", "No external video output backend"),
                ("Stream", "No encoder or streaming backend"),
                ("MultiCorder", "ISO recording is unavailable"),
                ("PlayList", "Media playlists are unavailable"),
            ] {
                ui.add_enabled(
                    false,
                    egui::Button::new(name).min_size(egui::vec2(86.0, 25.0)),
                )
                .on_disabled_hover_text(reason);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Overlay").clicked() {
                    self.actions.push(Action::FocusPanel(Panel::Overlays));
                }
                if ui.button("Audio Mixer").clicked() {
                    self.actions.push(Action::FocusPanel(Panel::Audio));
                }
                if ui.button("Multiview").clicked() {
                    self.actions.push(Action::Multiview);
                }
            });
        });
    }
    fn status(&mut self, ui: &mut egui::Ui) {
        let p = self.engine.project();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{}p{} target",
                    p.preferences.height,
                    p.preferences.rate.label()
                ))
                .strong()
                .color(TEAL),
            );
            ui.label(format!(
                "UI FPS: {:.1}   Demo CPU: {:.2} ms   Inputs: {}   Revision: {}",
                self.fps,
                self.generation_ms,
                p.inputs.len(),
                p.revision
            ))
            .on_hover_text(
                "Measured UI refresh and demo CPU generation; not broadcast or encoder performance",
            );
            ui.label(RichText::new("Record / Stream unavailable").color(DIM));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::Label::new(RichText::new(&self.notice).small().color(if self.warning {
                        RED
                    } else {
                        DIM
                    }))
                    .truncate(),
                );
            });
        });
    }
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.text_edit_focused()
            || self.properties.is_some()
            || self.settings
            || self.add_open
            || self.confirm_new
            || self.confirm_close
            || self.confirm_open
        {
            return;
        }
        if !self.engine.project().preferences.shortcuts {
            return;
        }
        ctx.input_mut(|i| {
            for (key, action) in [
                (egui::Key::S, Action::Save(false)),
                (egui::Key::O, Action::Open),
                (egui::Key::I, Action::Add),
            ] {
                if i.consume_key(egui::Modifiers::CTRL, key) {
                    self.actions.push(action);
                }
            }
            if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::S) {
                self.actions.push(Action::Save(true));
            }
            for (key, command) in [
                (egui::Key::Space, Command::Cut),
                (egui::Key::Enter, Command::Transition(TransitionKind::Fade)),
                (egui::Key::B, Command::FadeBlack),
                (egui::Key::Escape, Command::CancelTransition),
            ] {
                if i.consume_key(egui::Modifiers::NONE, key) {
                    self.actions.push(Action::Engine(command));
                }
            }
            for (n, key) in [
                egui::Key::Num1,
                egui::Key::Num2,
                egui::Key::Num3,
                egui::Key::Num4,
                egui::Key::Num5,
                egui::Key::Num6,
                egui::Key::Num7,
            ]
            .into_iter()
            .enumerate()
            {
                if i.consume_key(egui::Modifiers::NONE, key)
                    && let Some(input) = self.engine.project().inputs.get(n)
                {
                    self.actions
                        .push(Action::Engine(Command::SelectPreview(input.id)));
                }
            }
            for (n, key) in [egui::Key::F1, egui::Key::F2, egui::Key::F3, egui::Key::F4]
                .into_iter()
                .enumerate()
            {
                if i.consume_key(egui::Modifiers::NONE, key)
                    && let Some(o) = self.engine.project().overlays.get(n)
                {
                    let mut o = o.clone();
                    o.enabled = !o.enabled;
                    self.actions.push(Action::Engine(Command::SetOverlay(n, o)));
                }
            }
        });
    }
    fn frame_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.now = u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.engine.tick(self.now);
        self.poll_job(&ctx);
        self.shortcuts(&ctx);
        self.refresh_textures(&ctx);
        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.status(ui));
        egui::Panel::bottom("production.controls").show(ui, |ui| self.production_toolbar(ui));
        let viewport_height = ctx.viewport_rect().height();
        let monitor_height = (viewport_height * 0.53).max(390.0);
        // Preserve the operator's monitor/input proportion when resizing the window.
        let monitor_id = egui::Id::new("monitor.bank");
        if let Some(previous) = self.monitor_view_height
            && (previous - viewport_height).abs() > 1.0
            && previous > 0.0
            && let Some(mut state) = egui::containers::panel::PanelState::load(&ctx, monitor_id)
        {
            let height = (state.outer_rect.height() * viewport_height / previous)
                .clamp(390.0, (viewport_height - 200.0).max(390.0));
            state.outer_rect.max.y = state.outer_rect.min.y + height;
            ctx.data_mut(|d| d.insert_persisted(monitor_id, state));
        }
        self.monitor_view_height = Some(viewport_height);
        egui::Panel::top("monitor.bank")
            .resizable(true)
            .default_size(monitor_height)
            .min_size(390.0)
            .max_size((viewport_height - 200.0).max(390.0))
            .show(ui, |ui| {
                ui.take_available_space();
                self.switcher(ui);
            });
        let mut dock = std::mem::replace(&mut self.dock, DockState::new(vec![]));
        let style = egui_dock::Style::from_egui(ui.style());
        DockArea::new(&mut dock)
            .style(style)
            .show_close_buttons(false)
            .show_leaf_close_all_buttons(false)
            .show_leaf_collapse_buttons(false)
            .show_inside(ui, &mut Viewer { app: self });
        self.dock = dock;
        self.dialogs(&ctx);
        for action in std::mem::take(&mut self.actions) {
            self.dispatch(action, &ctx);
        }
        if self.job.is_none() && self.last_autosave.elapsed() > Duration::from_secs(60) {
            self.start_save(false, true);
            self.last_autosave = Instant::now();
        }
        self.fps_frames = self.fps_frames.saturating_add(1);
        if self.fps_start.elapsed() >= Duration::from_secs(1) {
            self.fps = self.fps_frames as f32 / self.fps_start.elapsed().as_secs_f32();
            self.fps_start = Instant::now();
            self.fps_frames = 0;
        }
        ctx.request_repaint_after(Duration::from_millis(33));
    }
}
struct Viewer<'a> {
    app: &'a mut RustMixApp,
}
impl TabViewer for Viewer<'_> {
    type Tab = Panel;
    fn id(&mut self, tab: &mut Panel) -> egui::Id {
        egui::Id::new(("workspace.panel", *tab))
    }
    fn on_tab_button(&mut self, tab: &mut Panel, response: &egui::Response) {
        self.app
            .automation
            .insert(format!("workspace.tab.{tab:?}"), response.rect);
    }
    fn title(&mut self, tab: &mut Panel) -> egui::WidgetText {
        tab.title().into()
    }
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Panel) {
        ui.push_id(*tab, |ui| match tab {
            Panel::Inputs => self.app.inputs(ui),
            Panel::Audio => self.app.audio(ui),
            Panel::Overlays => self.app.overlays(ui),
        });
    }
    fn is_closeable(&self, _: &Panel) -> bool {
        false
    }
    fn scroll_bars(&self, _: &Panel) -> [bool; 2] {
        [false, false]
    }
}
impl eframe::App for RustMixApp {
    fn raw_input_hook(&mut self, _: &egui::Context, raw: &mut egui::RawInput) {
        if let Some(smoke) = &mut self.smoke {
            raw.events.append(&mut smoke.events);
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        if ui.ctx().input(|i| i.viewport().close_requested())
            && self.saved_revision != self.engine.project().revision
            && !self.allow_close
        {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.actions.push(Action::Close);
        }
        self.frame_ui(ui);
        if self.smoke.is_some() {
            self.smoke_step(ui.ctx());
        }
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(json) = serde_json::to_string(&self.dock) {
            storage.set_string("workspace.v2", json);
        }
    }
}
