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

pub(crate) const TEAL: Color32 = Color32::from_rgb(59, 202, 190);
pub(crate) const RED: Color32 = Color32::from_rgb(241, 100, 105);
pub(crate) const DIM: Color32 = Color32::from_rgb(141, 155, 171);
pub(crate) const BG: Color32 = Color32::from_rgb(17, 21, 27);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Panel {
    Switcher,
    Inputs,
    Audio,
    Overlays,
}
impl Panel {
    fn title(self) -> &'static str {
        match self {
            Self::Switcher => "MONITORS / SWITCHER",
            Self::Inputs => "INPUTS",
            Self::Audio => "AUDIO MIXER · SIMULATED",
            Self::Overlays => "OVERLAYS / GRAPHICS",
        }
    }
}
fn default_dock(audio_focus: bool) -> DockState<Panel> {
    let mut dock = DockState::new(vec![Panel::Switcher]);
    let [_, bottom] =
        dock.main_surface_mut()
            .split_below(NodeIndex::root(), 0.60, vec![Panel::Inputs]);
    let [_, right] = dock.main_surface_mut().split_right(
        bottom,
        if audio_focus { 0.35 } else { 0.49 },
        vec![Panel::Audio],
    );
    dock.main_surface_mut()
        .split_right(right, 0.67, vec![Panel::Overlays]);
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
}
impl RustMixApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::headless();
        if let Some(storage) = cc.storage
            && let Some(json) = storage.get_string("workspace.v1")
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
        }
    }
    fn theme(&self, ctx: &egui::Context) {
        let mut style = egui::Style {
            visuals: egui::Visuals::dark(),
            ..Default::default()
        };
        style.spacing.item_spacing = egui::vec2(7.0, 6.0);
        style.spacing.button_padding = egui::vec2(9.0, 5.0);
        style.visuals.panel_fill = BG;
        style.visuals.window_fill = Color32::from_rgb(26, 32, 41);
        style.visuals.extreme_bg_color = Color32::from_rgb(12, 16, 21);
        style.visuals.faint_bg_color = Color32::from_rgb(26, 32, 41);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(37, 45, 57);
        style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(29, 36, 46);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(53, 69, 83);
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
            v.corner_radius = egui::CornerRadius::same(3);
        }
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
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
        }
    }
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("RUST").size(21.0).strong());ui.label(RichText::new("MIX").size(21.0).color(TEAL).strong());
            ui.separator();
            for (label,action,hint) in [("New",Action::New,"New demo project"),("Open",Action::Open,"Open a RustMix project · Ctrl+O"),("Save",Action::Save(false),"Save project · Ctrl+S"),("Save As",Action::Save(true),"Save a copy · Ctrl+Shift+S")] {
                if ui.add_enabled(self.job.is_none(),egui::Button::new(label)).on_hover_text(hint).clicked() {self.actions.push(action);}
            }
            ui.separator();
            if ui.button(RichText::new("+ Add Input").color(TEAL)).on_hover_text("Add a supported synthetic source · Ctrl+I").clicked(){self.actions.push(Action::Add);}
            egui::ComboBox::from_id_salt("workspace.selector").selected_text(&self.engine.project().workspace.name).width(112.0).show_ui(ui,|ui| {
                if ui.selectable_label(false,"Production").clicked(){self.actions.push(Action::Workspace(false));}
                if ui.selectable_label(false,"Audio focus").clicked(){self.actions.push(Action::Workspace(true));}
            });
            for (label,action) in [("Multiview",Action::Multiview),("Fullscreen",Action::Fullscreen),("Settings",Action::Settings),("About",Action::About)] {if ui.button(label).clicked(){self.actions.push(action);}}
            ui.add_enabled(false,egui::Button::new("REC")).on_disabled_hover_text("Recorder unavailable: encoder and recoverable recording backend are not implemented");
            ui.add_enabled(false,egui::Button::new("STREAM")).on_disabled_hover_text("Streaming unavailable: no encoder or network output backend");
        });
        ui.horizontal(|ui| {
            let dirty = self.saved_revision != self.engine.project().revision;
            ui.label(
                RichText::new(format!(
                    "{}{}",
                    self.engine.project().name,
                    if dirty { "  • unsaved" } else { "" }
                ))
                .color(DIM),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("SYNTHETIC WORKSPACE  /  OFFLINE")
                        .color(TEAL)
                        .small(),
                );
            });
        });
    }
    fn status(&mut self, ui: &mut egui::Ui) {
        let p = self.engine.project();
        ui.horizontal_wrapped(|ui|{
            ui.label(RichText::new(format!("{}×{}  /  {}p target",p.preferences.width,p.preferences.height,p.preferences.rate.label())).color(DIM));ui.separator();
            ui.label(format!("UI {:.1} fps · demo CPU {:.2} ms",self.fps,self.generation_ms)).on_hover_text("Measured UI callbacks / second and CPU demo generation time, not output encoder FPS or GPU time");ui.separator();
            ui.label(format!("{} inputs · rev {}",p.inputs.len(),p.revision));
            ui.label(RichText::new("REC unavailable · STREAM unavailable").color(DIM));
        });
        ui.label(RichText::new(&self.notice).color(if self.warning { RED } else { DIM }));
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
        let mut dock = std::mem::replace(&mut self.dock, DockState::new(vec![]));
        let style = egui_dock::Style::from_egui(ui.style());
        DockArea::new(&mut dock)
            .style(style)
            .show_close_buttons(false)
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
            Panel::Switcher => self.app.switcher(ui),
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
            storage.set_string("workspace.v1", json);
        }
    }
}
