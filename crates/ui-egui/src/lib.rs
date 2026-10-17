#![forbid(unsafe_code)]
mod dialogs;
mod panels;
mod smoke;
mod workspace;

use eframe::egui::{self, Color32, RichText, Stroke};
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
pub(crate) const PREVIEW: Color32 = Color32::from_rgb(240, 128, 0);
pub(crate) const PROGRAM: Color32 = Color32::from_rgb(14, 122, 13);
pub(crate) const RED: Color32 = Color32::from_rgb(241, 100, 105);
pub(crate) const DIM: Color32 = Color32::from_rgb(148, 160, 172);
pub(crate) const BG: Color32 = Color32::from_rgb(30, 35, 40);
/// Idle source-tile header; Program and Preview override it with their own color.
pub(crate) const IDLE_TILE: Color32 = Color32::from_rgb(51, 60, 69);
/// Mixer side rails (`OUTPUTS` / `INPUTS`).
pub(crate) const MIXER_RAIL: Color32 = Color32::from_rgb(0, 102, 180);
/// Button and chip surface used across tiles, strips, and switcher stacks.
pub(crate) const CHIP_BG: Color32 = Color32::from_rgb(45, 53, 60);
/// Light form surface used inside reference dialogs (`Add Input`, `Input Settings`, `Settings`, `Recording Setup`, `Overlay Settings`).
pub(crate) const FORM_LIGHT: Color32 = Color32::from_rgb(244, 245, 247);
pub(crate) const FORM_TEXT: Color32 = Color32::from_rgb(22, 26, 30);

/// Category palette shared by the input filter chips and dialog category selectors.
pub(crate) fn kind_color(kind: InputKind) -> Color32 {
    match kind {
        InputKind::Grid => Color32::from_rgb(145, 0, 0),
        InputKind::Geometry => PROGRAM,
        InputKind::Bars => PREVIEW,
        InputKind::Graphics => Color32::from_rgb(132, 29, 145),
        InputKind::Studio => Color32::from_rgb(0, 144, 229),
        InputKind::Color => Color32::from_rgb(30, 30, 95),
        InputKind::Countdown => Color32::from_rgb(30, 131, 116),
    }
}

/// Frame-normalized title-overlay panel shared by the Program monitor painters and
/// the Overlay Settings preview.
pub(crate) fn overlay_panel_rect(frame: egui::Rect, position: Position, slot: usize) -> egui::Rect {
    let offset = slot as f32 * 0.105;
    let (left, top) = match position {
        Position::LowerLeft => (0.05, 0.78 - offset),
        Position::LowerRight => (0.38, 0.78 - offset),
        Position::TopLeft => (0.05, 0.08 + offset),
    };
    egui::Rect::from_min_size(
        frame.lerp_inside(egui::vec2(left, top)),
        egui::vec2(frame.width() * 0.56, frame.height() * 0.095),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Panel {
    Inputs,
    Audio,
    Overlays,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputPage {
    General,
    ColourAdjust,
    ColourKey,
    ColourCorrection,
    Effects,
    Position,
    Layers,
    Triggers,
    TallyLights,
    Ptz,
    Advanced,
    Audio,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsPage {
    Display,
    Outputs,
    Options,
    Appearance,
    Decoders,
    Recording,
    ExternalOutput,
    Audio,
    AudioOutputs,
    Workspace,
    TallyLights,
    Shortcuts,
    Activators,
    Scripting,
    Alerts,
    About,
}

impl Panel {
    fn title(self) -> &'static str {
        match self {
            Self::Inputs => "Inputs",
            Self::Audio => "🔊 Audio Mixer",
            Self::Overlays => "Overlay / Graphics",
        }
    }
}

fn default_dock(audio_focus: bool) -> DockState<Panel> {
    let mut dock = DockState::new(vec![Panel::Inputs, Panel::Overlays]);
    dock.main_surface_mut().split_right(
        NodeIndex::root(),
        if audio_focus { 0.59 } else { 0.82 },
        vec![Panel::Audio],
    );
    dock
}

pub(crate) enum Action {
    Engine(Command),
    New,
    Open,
    Save(bool),
    CheckpointNow,
    Add,
    Properties(InputId),
    PropertiesAt(InputId, InputPage),
    InputAudio(InputId),
    RequestRemove(InputId),
    Settings,
    SettingsAt(SettingsPage),
    OverlaySettings(usize),
    RecordingSetup,
    StreamingSetup,
    MultiCorder,
    PlayList,
    ShortcutsWindow,
    ToggleRecordSim,
    ToggleStreamSim,
    ToggleExternalSim,
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
    pub(crate) add_tab: usize,
    pub(crate) add_name: String,
    pub(crate) add_kind: InputKind,
    pub(crate) add_alpha: bool,
    pub(crate) add_interlaced: bool,
    pub(crate) add_number: usize,
    pub(crate) add_preview: Option<(InputKind, egui::TextureHandle)>,
    pub(crate) properties: Option<InputState>,
    pub(crate) input_page: InputPage,
    pub(crate) prop_deinterlace: bool,
    pub(crate) prop_sharpen: bool,
    pub(crate) prop_mirror: bool,
    pub(crate) prop_flatten: bool,
    pub(crate) prop_auto_mix: bool,
    pub(crate) prop_auto_play: bool,
    pub(crate) prop_auto_restart: bool,
    pub(crate) prop_auto_pause: bool,
    pub(crate) prop_zoom: f32,
    pub(crate) prop_pan_x: f32,
    pub(crate) prop_pan_y: f32,
    pub(crate) prop_lift: f32,
    pub(crate) prop_gain: f32,
    pub(crate) prop_saturation: f32,
    pub(crate) confirm_remove_input: Option<InputId>,
    pub(crate) settings: bool,
    pub(crate) settings_page: SettingsPage,
    pub(crate) settings_draft: Option<Preferences>,
    pub(crate) settings_name: String,
    pub(crate) settings_layout: Option<bool>,
    pub(crate) settings_advanced: bool,
    pub(crate) settings_fullscreen_slot: u8,
    pub(crate) settings_hide_cursor: bool,
    pub(crate) settings_on_top: bool,
    pub(crate) settings_minimise: bool,
    /// Staged Overlay Settings draft: selected slot plus its uncommitted copy.
    pub(crate) overlay_settings: Option<(usize, OverlayState)>,
    pub(crate) overlay_type_idx: usize,
    pub(crate) overlay_effect_idx: usize,
    pub(crate) overlay_effect_ms: u32,
    pub(crate) overlay_duration_val: u32,
    pub(crate) overlay_duration_frames: bool,
    pub(crate) overlay_stinger_cut: u32,
    pub(crate) overlay_alpha: u8,
    pub(crate) overlay_zoom: f32,
    pub(crate) overlay_zoom_x: f32,
    pub(crate) overlay_zoom_y: f32,
    pub(crate) overlay_pan_x: f32,
    pub(crate) overlay_pan_y: f32,
    pub(crate) overlay_rotate_z: f32,
    pub(crate) overlay_border_thickness: u32,
    pub(crate) overlay_border_radius: u32,
    /// Reference dialogs from `C:\Users\WORKPLUS\Documents\WEB\rustmix-reference`.
    pub(crate) recording_setup: bool,
    pub(crate) rec_slot: u8,
    pub(crate) rec_format: usize,
    pub(crate) rec_filename: String,
    pub(crate) rec_bitrate: String,
    pub(crate) rec_hw_encoder: bool,
    pub(crate) rec_fault_tolerant: bool,
    pub(crate) rec_audio_master: bool,
    pub(crate) rec_audio_delay: String,
    pub(crate) rec_audio_kbps: String,
    pub(crate) rec_split_mins: String,
    pub(crate) rec_active: bool,
    pub(crate) streaming_setup: bool,
    pub(crate) stream_channel: usize,
    pub(crate) stream_destination: usize,
    pub(crate) stream_url: String,
    pub(crate) stream_key: String,
    pub(crate) stream_use_q1: bool,
    pub(crate) stream_quality: usize,
    pub(crate) stream_active: bool,
    pub(crate) external_active: bool,
    pub(crate) multicorder_open: bool,
    pub(crate) mc_format: usize,
    pub(crate) mc_running: bool,
    pub(crate) mc_ndi_mov: bool,
    pub(crate) mc_show_unsupported: bool,
    pub(crate) mc_folder: String,
    pub(crate) mc_checked: [bool; 12],
    pub(crate) playlist_open: bool,
    pub(crate) playlist_items: Vec<InputId>,
    pub(crate) playlist_selected_avail: usize,
    pub(crate) playlist_selected_item: usize,
    pub(crate) playlist_loop: bool,
    pub(crate) playlist_begin_selected: bool,
    pub(crate) playlist_clear_overlays: bool,
    pub(crate) playlist_manual: bool,
    pub(crate) playlist_running: bool,
    pub(crate) shortcuts_open: bool,
    pub(crate) search_open: bool,
    pub(crate) ui_locked: bool,
    pub(crate) about: bool,
    pub(crate) multiview: bool,
    pub(crate) multiview_split: bool,
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
            notice: "Ready · 1080p29,97 production workspace".into(),
            warning: false,
            history: VecDeque::new(),
            path: None,
            saved_revision: 0,
            job: None,
            last_autosave: Instant::now(),
            add_open: false,
            add_tab: 0,
            add_name: "New synthetic input".into(),
            add_kind: InputKind::Grid,
            add_alpha: false,
            add_interlaced: false,
            add_number: 1,
            add_preview: None,
            properties: None,
            input_page: InputPage::General,
            prop_deinterlace: false,
            prop_sharpen: false,
            prop_mirror: false,
            prop_flatten: false,
            prop_auto_mix: true,
            prop_auto_play: true,
            prop_auto_restart: false,
            prop_auto_pause: true,
            prop_zoom: 1.0,
            prop_pan_x: 0.0,
            prop_pan_y: 0.0,
            prop_lift: 0.0,
            prop_gain: 1.0,
            prop_saturation: 1.0,
            confirm_remove_input: None,
            settings: false,
            settings_page: SettingsPage::Display,
            settings_draft: None,
            settings_name: String::new(),
            settings_layout: None,
            settings_advanced: false,
            settings_fullscreen_slot: 0,
            settings_hide_cursor: false,
            settings_on_top: false,
            settings_minimise: false,
            overlay_settings: None,
            overlay_type_idx: 0,
            overlay_effect_idx: 0,
            overlay_effect_ms: 500,
            overlay_duration_val: 0,
            overlay_duration_frames: false,
            overlay_stinger_cut: 0,
            overlay_alpha: 255,
            overlay_zoom: 1.0,
            overlay_zoom_x: 1.0,
            overlay_zoom_y: 1.0,
            overlay_pan_x: 0.0,
            overlay_pan_y: 0.0,
            overlay_rotate_z: 0.0,
            overlay_border_thickness: 0,
            overlay_border_radius: 0,
            recording_setup: false,
            rec_slot: 1,
            rec_format: 2,
            rec_filename: r"C:\Users\WORKPLUS\Documents\kopi\RB Rehearsel.mp4".into(),
            rec_bitrate: "12".into(),
            rec_hw_encoder: true,
            rec_fault_tolerant: false,
            rec_audio_master: true,
            rec_audio_delay: "0".into(),
            rec_audio_kbps: "192".into(),
            rec_split_mins: "None".into(),
            rec_active: false,
            streaming_setup: false,
            stream_channel: 5,
            stream_destination: 0,
            stream_url: "rtmp://a.rtmp.youtube.com/live2".into(),
            stream_key: "••••-••••-••••-••••".into(),
            stream_use_q1: true,
            stream_quality: 0,
            stream_active: false,
            external_active: false,
            multicorder_open: false,
            mc_format: 3,
            mc_running: false,
            mc_ndi_mov: true,
            mc_show_unsupported: false,
            mc_folder: r"C:\Users\WORKPLUS\Documents\vMixStorage".into(),
            mc_checked: [
                true, true, false, false, true, true, false, false, false, false, false, false,
            ],
            playlist_open: false,
            playlist_items: vec![InputId(1), InputId(2), InputId(3)],
            playlist_selected_avail: 0,
            playlist_selected_item: 0,
            playlist_loop: true,
            playlist_begin_selected: false,
            playlist_clear_overlays: false,
            playlist_manual: false,
            playlist_running: false,
            shortcuts_open: false,
            search_open: true,
            ui_locked: false,
            about: false,
            multiview: false,
            multiview_split: true,
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

    fn reset_editors(&mut self) {
        self.add_open = false;
        self.properties = None;
        self.input_page = InputPage::General;
        self.confirm_remove_input = None;
        self.settings = false;
        self.settings_draft = None;
        self.settings_name.clear();
        self.settings_layout = None;
        self.overlay_settings = None;
        self.recording_setup = false;
        self.streaming_setup = false;
        self.multicorder_open = false;
        self.playlist_open = false;
        self.shortcuts_open = false;
    }

    fn theme(&self, ctx: &egui::Context) {
        let mut style = egui::Style {
            visuals: egui::Visuals::dark(),
            ..Default::default()
        };
        style.spacing.item_spacing = egui::vec2(4.0, 3.0);
        style.spacing.button_padding = egui::vec2(7.0, 3.0);
        style.visuals.panel_fill = BG;
        style.visuals.window_fill = Color32::from_rgb(27, 32, 38);
        style.visuals.extreme_bg_color = Color32::from_rgb(12, 15, 18);
        style.visuals.faint_bg_color = Color32::from_rgb(34, 41, 46);
        style.visuals.widgets.inactive.bg_fill = CHIP_BG;
        style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(38, 45, 52);
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(72, 82, 92));
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(65, 76, 88);
        style.visuals.widgets.hovered.bg_stroke =
            Stroke::new(1.0, Color32::from_rgb(105, 118, 132));
        style.visuals.widgets.active.bg_fill = PROGRAM;
        style.visuals.selection.bg_fill = PROGRAM;
        style.visuals.selection.stroke = egui::Stroke::new(1.0, Color32::WHITE);
        style.visuals.override_text_color =
            Some(if self.engine.project().preferences.high_contrast {
                Color32::WHITE
            } else {
                Color32::from_rgb(230, 235, 240)
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
        let mut button = egui::Button::new(RichText::new(label).strong().size(11.5));
        if let Some(color) = color {
            button = button.fill(color).stroke(egui::Stroke::NONE);
        }
        button = button.min_size(egui::vec2(ui.available_width().clamp(72.0, 140.0), 26.0));
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
                            self.reset_editors();
                            self.engine = engine;
                            self.last_demo_frame = None;
                            self.theme(ctx);
                            self.path = Some(path);
                            self.saved_revision = self.engine.project().revision;
                            self.textures.clear();
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
            Action::CheckpointNow => self.start_save(false, true),
            Action::Add => {
                self.add_number = self.engine.project().inputs.len() + 1;
                self.add_open = true;
            }
            Action::Properties(id) => {
                self.properties = self.engine.project().input(id).cloned();
                self.input_page = InputPage::General;
            }
            Action::PropertiesAt(id, page) => {
                self.properties = self.engine.project().input(id).cloned();
                self.input_page = page;
            }
            Action::InputAudio(id) => {
                self.properties = self.engine.project().input(id).cloned();
                self.input_page = InputPage::Audio;
            }
            Action::RequestRemove(id) => match self.engine.enabled(&Command::RemoveInput(id)) {
                Ok(()) => self.confirm_remove_input = Some(id),
                Err(error) => self.say(error.to_string(), true),
            },
            Action::Settings => {
                self.settings = true;
                self.settings_page = SettingsPage::Display;
                self.settings_draft = Some(self.engine.project().preferences.clone());
                self.settings_name = self.engine.project().name.clone();
                self.settings_layout = None;
            }
            Action::SettingsAt(page) => {
                self.settings = true;
                self.settings_page = page;
                self.settings_draft = Some(self.engine.project().preferences.clone());
                self.settings_name = self.engine.project().name.clone();
                self.settings_layout = None;
            }
            Action::OverlaySettings(slot) => {
                if let Some(overlay) = self.engine.project().overlays.get(slot).cloned() {
                    self.overlay_settings = Some((slot, overlay));
                }
            }
            Action::RecordingSetup => self.recording_setup = true,
            Action::StreamingSetup => self.streaming_setup = true,
            Action::MultiCorder => self.multicorder_open = true,
            Action::PlayList => self.playlist_open = true,
            Action::ShortcutsWindow => self.shortcuts_open = true,
            Action::ToggleRecordSim => {
                self.rec_active = !self.rec_active;
                self.say(
                    if self.rec_active {
                        format!("Recording UI simulation active · {}", self.rec_filename)
                    } else {
                        "Recording UI simulation stopped".into()
                    },
                    false,
                );
            }
            Action::ToggleStreamSim => {
                self.stream_active = !self.stream_active;
                self.say(
                    if self.stream_active {
                        format!(
                            "Streaming UI simulation active · Profile Default (Channel {})",
                            self.stream_channel
                        )
                    } else {
                        "Streaming UI simulation stopped".into()
                    },
                    false,
                );
            }
            Action::ToggleExternalSim => {
                self.external_active = !self.external_active;
                self.say(
                    if self.external_active {
                        "External Output UI simulation active".into()
                    } else {
                        "External Output UI simulation stopped".into()
                    },
                    false,
                );
            }
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
            ui.spacing_mut().item_spacing.x = 2.0;
            let brand = ui
                .add(
                    egui::Button::new(RichText::new("Preset").strong().color(Color32::WHITE))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE)
                        .min_size(egui::vec2(52.0, 22.0)),
                )
                .on_hover_text(format!(
                    "{}\nClick for About RustMix",
                    self.engine.project().name
                ));
            self.automation.insert("toolbar.brand".into(), brand.rect);
            if brand.clicked() {
                self.actions.push(Action::About);
            }
            ui.add_space(4.0);
            if ui
                .add_enabled(
                    self.job.is_none(),
                    egui::Button::new(RichText::new("New").strong())
                        .min_size(egui::vec2(58.0, 22.0)),
                )
                .on_hover_text("New preset / project")
                .clicked()
            {
                self.actions.push(Action::New);
            }
            if ui
                .add_enabled(
                    self.job.is_none(),
                    egui::Button::new(RichText::new("Open").strong())
                        .min_size(egui::vec2(58.0, 22.0)),
                )
                .on_hover_text("Open preset · Ctrl+O")
                .clicked()
            {
                self.actions.push(Action::Open);
            }
            ui.add_enabled_ui(self.job.is_none(), |ui| {
                let open_menu = ui.menu_button(RichText::new("▾").size(10.0), |ui| {
                    if ui.button("Open project… · Ctrl+O").clicked() {
                        self.actions.push(Action::Open);
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            self.path.is_some(),
                            egui::Button::new("Reload last saved project"),
                        )
                        .clicked()
                    {
                        self.actions.push(Action::Last);
                        ui.close();
                    }
                    if ui.button("Recover last checkpoint…").clicked() {
                        self.actions.push(Action::Recover);
                        ui.close();
                    }
                });
                open_menu
                    .response
                    .on_hover_text("Open, reload or recover a preset file");
            });
            if ui
                .add_enabled(
                    self.job.is_none(),
                    egui::Button::new(RichText::new("Save").strong())
                        .min_size(egui::vec2(58.0, 22.0)),
                )
                .on_hover_text("Save · Ctrl+S")
                .clicked()
            {
                self.actions.push(Action::Save(false));
            }
            if ui
                .add_enabled(
                    self.job.is_none(),
                    egui::Button::new(RichText::new("Save As").strong())
                        .min_size(egui::vec2(62.0, 22.0)),
                )
                .on_hover_text("Save a copy · Ctrl+Shift+S")
                .clicked()
            {
                self.actions.push(Action::Save(true));
            }
            ui.add_enabled_ui(self.job.is_none(), |ui| {
                let save_menu = ui.menu_button(RichText::new("▾").size(10.0), |ui| {
                    if ui.button("Save project · Ctrl+S").clicked() {
                        self.actions.push(Action::Save(false));
                        ui.close();
                    }
                    if ui.button("Save project as… · Ctrl+Shift+S").clicked() {
                        self.actions.push(Action::Save(true));
                        ui.close();
                    }
                    if ui.button("Write recovery checkpoint now").clicked() {
                        self.actions.push(Action::CheckpointNow);
                        ui.close();
                    }
                });
                save_menu
                    .response
                    .on_hover_text("Save preset or write an immediate recovery checkpoint");
            });
            if ui
                .add_enabled(
                    self.path.is_some() && self.job.is_none(),
                    egui::Button::new(RichText::new("Last").strong())
                        .min_size(egui::vec2(54.0, 22.0)),
                )
                .on_disabled_hover_text("Save or open a preset first")
                .on_hover_text("Reload the most recently saved preset")
                .clicked()
            {
                self.actions.push(Action::Last);
            }
            ui.add_space((ui.max_rect().width() * 0.48 - ui.cursor().left() - 48.0).max(4.0));
            let fullscreen = ui
                .add(
                    egui::Button::new(
                        RichText::new(if self.fullscreen {
                            "Exit Fullscreen"
                        } else {
                            "Fullscreen"
                        })
                        .strong(),
                    )
                    .min_size(egui::vec2(78.0, 22.0)),
                )
                .on_hover_text("Toggle Fullscreen output");
            self.automation
                .insert("toolbar.fullscreen".into(), fullscreen.rect);
            if fullscreen.clicked() {
                self.actions.push(Action::Fullscreen);
            }
            ui.menu_button(RichText::new("▾").size(10.0), |ui| {
                if ui.button("Fullscreen 1 → Program").clicked() {
                    self.settings_fullscreen_slot = 0;
                    ui.close();
                }
                if ui.button("Fullscreen 2 → Multiview").clicked() {
                    self.settings_fullscreen_slot = 1;
                    ui.close();
                }
                ui.separator();
                if ui.button("Display Settings…").clicked() {
                    self.actions.push(Action::SettingsAt(SettingsPage::Display));
                    ui.close();
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                if ui
                    .add(
                        egui::Button::new(RichText::new("?").strong())
                            .min_size(egui::vec2(22.0, 22.0)),
                    )
                    .on_hover_text("About RustMix")
                    .clicked()
                {
                    self.actions.push(Action::About);
                }
                let settings_menu = ui.menu_button(RichText::new("▾").size(10.0), |ui| {
                    for (label, page) in [
                        ("Display…", SettingsPage::Display),
                        ("Outputs / NDI / OMT / SRT…", SettingsPage::Outputs),
                        ("Options…", SettingsPage::Options),
                        ("Performance / Appearance…", SettingsPage::Appearance),
                        ("Web Controller / Workspace…", SettingsPage::Workspace),
                        ("Shortcuts…", SettingsPage::Shortcuts),
                    ] {
                        if ui.button(label).clicked() {
                            self.actions.push(Action::SettingsAt(page));
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button("Overlay Settings…").clicked() {
                        self.actions.push(Action::OverlaySettings(0));
                        ui.close();
                    }
                });
                settings_menu
                    .response
                    .on_hover_text("Open a specific Settings category");
                let settings = ui
                    .add(
                        egui::Button::new(RichText::new("Settings").strong())
                            .min_size(egui::vec2(64.0, 22.0)),
                    )
                    .on_hover_text("Open Settings");
                self.automation
                    .insert("toolbar.settings".into(), settings.rect);
                if settings.clicked() {
                    self.actions.push(Action::Settings);
                }
                if ui
                    .add(
                        egui::Button::new(RichText::new("Basic").strong())
                            .selected(self.multiview)
                            .min_size(egui::vec2(54.0, 22.0)),
                    )
                    .on_hover_text("Toggle Basic / Multiview mode")
                    .clicked()
                {
                    self.actions.push(Action::Multiview);
                }
                if ui
                    .add(
                        egui::Button::new(RichText::new("Pause Inputs").strong())
                            .selected(self.demo_paused)
                            .min_size(egui::vec2(84.0, 22.0)),
                    )
                    .on_hover_text("Freeze input motion while keeping switching active")
                    .clicked()
                {
                    self.actions.push(Action::PauseDemo);
                }
            });
        });
    }

    fn production_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let add = ui
                .add(
                    egui::Button::new(RichText::new("Add Input").strong())
                        .min_size(egui::vec2(68.0, 24.0)),
                )
                .on_hover_text("Open Input Select · Ctrl+I");
            self.automation.insert("production.add".into(), add.rect);
            if add.clicked() {
                self.actions.push(Action::Add);
            }
            let caret = ui.menu_button(RichText::new("▾").size(10.0), |ui| {
                for (idx, (label, kind)) in [
                    ("Video", InputKind::Grid),
                    ("Camera", InputKind::Geometry),
                    ("Colour / Bars", InputKind::Bars),
                    ("Title / XAML", InputKind::Graphics),
                    ("Virtual Set", InputKind::Studio),
                    ("Solid Colour", InputKind::Color),
                    ("Timer / Clock", InputKind::Countdown),
                ]
                .into_iter()
                .enumerate()
                {
                    if ui.button(label).clicked() {
                        self.add_tab = idx;
                        self.add_kind = kind;
                        self.add_name = label.into();
                        self.actions.push(Action::Add);
                        ui.close();
                    }
                }
            });
            caret
                .response
                .on_hover_text("Quick input category selector");
            ui.add_space(10.0);

            // 1. Record: [ ⚙ | Record ]
            let rec_gear = ui
                .add(
                    egui::Button::new(RichText::new("⚙").size(13.0))
                        .min_size(egui::vec2(26.0, 24.0)),
                )
                .on_hover_text("Open Recording Setup");
            self.automation
                .insert("output.record.settings".into(), rec_gear.rect);
            if rec_gear.clicked() {
                self.actions.push(Action::RecordingSetup);
            }
            let mut rec_btn_w = egui::Button::new(RichText::new("Record").strong())
                .min_size(egui::vec2(66.0, 24.0));
            if self.rec_active {
                rec_btn_w = rec_btn_w.fill(Color32::from_rgb(180, 24, 24));
            }
            let rec_btn = ui.add(rec_btn_w).on_hover_text("Toggle Recording state");
            self.automation.insert("output.record".into(), rec_btn.rect);
            if rec_btn.clicked() {
                self.actions.push(Action::ToggleRecordSim);
            }
            ui.add_space(12.0);

            // 2. External: [ ⚙ | External ] with white popup menu matching External.png
            let ext_gear = ui.menu_button(RichText::new("⚙").size(13.0), |ui| {
                ui.visuals_mut().panel_fill = Color32::WHITE;
                ui.visuals_mut().window_fill = Color32::WHITE;
                ui.visuals_mut().override_text_color = Some(Color32::BLACK);
                ui.set_min_width(210.0);
                for item in [
                    "Output 2                                  ▸",
                    "Output 3                                  ▸",
                    "Output 4                                  ▸",
                    "SRT",
                ] {
                    if ui.button(item).clicked() {
                        self.say(format!("External route selected: {}", item.trim()), false);
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Outputs / NDI / OMT / SRT Settings").clicked() {
                    self.actions.push(Action::SettingsAt(SettingsPage::Outputs));
                    ui.close();
                }
                if ui.button("External Output Settings").clicked() {
                    self.actions
                        .push(Action::SettingsAt(SettingsPage::ExternalOutput));
                    ui.close();
                }
            });
            self.automation
                .insert("output.external.settings".into(), ext_gear.response.rect);
            let mut ext_btn_w = egui::Button::new(RichText::new("External").strong())
                .min_size(egui::vec2(66.0, 24.0));
            if self.external_active {
                ext_btn_w = ext_btn_w.fill(Color32::from_rgb(180, 24, 24));
            }
            let ext_btn = ui.add(ext_btn_w).on_hover_text("Toggle External Output");
            self.automation
                .insert("output.external".into(), ext_btn.rect);
            if ext_btn.clicked() {
                self.actions.push(Action::ToggleExternalSim);
            }
            ui.add_space(12.0);

            // 3. Stream: [ ⚙ | Stream | ▾ ]
            let stm_gear = ui
                .add(
                    egui::Button::new(RichText::new("⚙").size(13.0))
                        .min_size(egui::vec2(26.0, 24.0)),
                )
                .on_hover_text("Open Streaming Settings");
            self.automation
                .insert("output.stream.settings".into(), stm_gear.rect);
            if stm_gear.clicked() {
                self.actions.push(Action::StreamingSetup);
            }
            let mut stm_btn_w = egui::Button::new(RichText::new("Stream").strong())
                .min_size(egui::vec2(58.0, 24.0));
            if self.stream_active {
                stm_btn_w = stm_btn_w.fill(Color32::from_rgb(180, 24, 24));
            }
            let stm_btn = ui.add(stm_btn_w).on_hover_text("Toggle Streaming state");
            self.automation.insert("output.stream".into(), stm_btn.rect);
            if stm_btn.clicked() {
                self.actions.push(Action::ToggleStreamSim);
            }
            ui.menu_button(RichText::new("▾").size(10.0), |ui| {
                for ch in 1..=5 {
                    if ui.button(format!("Stream {ch}")).clicked() {
                        self.stream_channel = ch;
                        self.actions.push(Action::StreamingSetup);
                        ui.close();
                    }
                }
            });
            ui.add_space(12.0);

            // 4. MultiCorder: [ ⚙ | MultiCorder ]
            let mc_gear = ui
                .add(
                    egui::Button::new(RichText::new("⚙").size(13.0))
                        .min_size(egui::vec2(26.0, 24.0)),
                )
                .on_hover_text("Open MultiCorder");
            self.automation
                .insert("output.multicorder.settings".into(), mc_gear.rect);
            if mc_gear.clicked() {
                self.actions.push(Action::MultiCorder);
            }
            let mut mc_btn_w = egui::Button::new(RichText::new("MultiCorder").strong())
                .min_size(egui::vec2(78.0, 24.0));
            if self.mc_running {
                mc_btn_w = mc_btn_w.fill(PROGRAM);
            }
            let mc_btn = ui.add(mc_btn_w).on_hover_text("Open MultiCorder");
            self.automation
                .insert("output.multicorder".into(), mc_btn.rect);
            if mc_btn.clicked() {
                self.actions.push(Action::MultiCorder);
            }
            ui.add_space(12.0);

            // 5. PlayList: [ ⚙ | PlayList ]
            let pl_gear = ui
                .add(
                    egui::Button::new(RichText::new("⚙").size(13.0))
                        .min_size(egui::vec2(26.0, 24.0)),
                )
                .on_hover_text("Open PlayList");
            self.automation
                .insert("output.playlist.settings".into(), pl_gear.rect);
            if pl_gear.clicked() {
                self.actions.push(Action::PlayList);
            }
            let mut pl_btn_w = egui::Button::new(RichText::new("PlayList").strong())
                .min_size(egui::vec2(66.0, 24.0));
            if self.playlist_running {
                pl_btn_w = pl_btn_w.fill(PROGRAM);
            }
            let pl_btn = ui.add(pl_btn_w).on_hover_text("Open PlayList");
            self.automation
                .insert("output.playlist".into(), pl_btn.rect);
            if pl_btn.clicked() {
                self.actions.push(Action::PlayList);
            }

            // Right-side icon cluster matching Interface.png: [Overlay] [☰] [📊] [⊞] [📷] [🔒]
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let lock = ui
                    .add(
                        egui::Button::new(RichText::new("🔒").size(12.0))
                            .selected(self.ui_locked)
                            .min_size(egui::vec2(24.0, 24.0)),
                    )
                    .on_hover_text("Lock workspace controls");
                if lock.clicked() {
                    self.ui_locked = !self.ui_locked;
                }
                let snap = ui
                    .add(
                        egui::Button::new(RichText::new("📷").size(12.0))
                            .min_size(egui::vec2(24.0, 24.0)),
                    )
                    .on_hover_text("Save Program snapshot");
                if snap.clicked() {
                    self.say("Program snapshot captured".into(), false);
                }
                let mv = ui
                    .add(
                        egui::Button::new(RichText::new("⊞").size(13.0))
                            .selected(self.multiview)
                            .min_size(egui::vec2(24.0, 24.0)),
                    )
                    .on_hover_text("Toggle Multiview layout");
                self.automation.insert("output.multiview".into(), mv.rect);
                if mv.clicked() {
                    self.actions.push(Action::Multiview);
                }
                // Colored audio bar icon button
                let (audio_rect, audio_resp) =
                    ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::click());
                ui.painter().rect_filled(audio_rect, 0.0, CHIP_BG);
                ui.painter().rect_stroke(
                    audio_rect,
                    0.0,
                    Stroke::new(1.0, Color32::from_rgb(72, 82, 92)),
                    egui::StrokeKind::Inside,
                );
                for (idx, (h, col)) in [
                    (0.55_f32, Color32::from_rgb(35, 195, 65)),
                    (0.85, Color32::from_rgb(235, 195, 35)),
                    (0.70, Color32::from_rgb(230, 50, 50)),
                ]
                .into_iter()
                .enumerate()
                {
                    let bx = audio_rect.left() + 5.0 + idx as f32 * 5.0;
                    let bar = egui::Rect::from_min_max(
                        egui::pos2(bx, audio_rect.bottom() - 4.0 - 15.0 * h),
                        egui::pos2(bx + 3.5, audio_rect.bottom() - 4.0),
                    );
                    ui.painter().rect_filled(bar, 0.0, col);
                }
                if audio_resp.on_hover_text("Focus Audio Mixer").clicked() {
                    self.actions.push(Action::FocusPanel(Panel::Audio));
                }
                let sc_btn = ui
                    .add(
                        egui::Button::new(RichText::new("☰").size(13.0))
                            .selected(self.shortcuts_open)
                            .min_size(egui::vec2(24.0, 24.0)),
                    )
                    .on_hover_text("Open Shortcuts window");
                self.automation
                    .insert("output.shortcuts".into(), sc_btn.rect);
                if sc_btn.clicked() {
                    self.actions.push(Action::ShortcutsWindow);
                }
                let overlay_btn = ui
                    .add(
                        egui::Button::new(RichText::new("Overlay").strong())
                            .min_size(egui::vec2(64.0, 24.0)),
                    )
                    .on_hover_text("Open Overlay Settings");
                self.automation
                    .insert("output.overlay".into(), overlay_btn.rect);
                self.automation
                    .insert("output.overlay.settings".into(), overlay_btn.rect);
                if overlay_btn.clicked() {
                    self.actions.push(Action::OverlaySettings(0));
                }
            });
        });
    }

    fn status(&mut self, ui: &mut egui::Ui) {
        let p = self.engine.project();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.label(
                RichText::new(format!("{}p29,97", p.preferences.height))
                    .strong()
                    .color(Color32::from_rgb(40, 210, 75)),
            );
            ui.label(
                RichText::new("Update Available")
                    .underline()
                    .color(Color32::from_rgb(40, 210, 75)),
            );
            ui.label(RichText::new("EX").color(Color32::WHITE));
            ui.label(RichText::new("FPS:").strong().color(Color32::WHITE));
            ui.label(RichText::new(format!("{:.0}", self.fps.max(30.0))).color(Color32::WHITE));
            ui.label(RichText::new("Render Time:").strong().color(Color32::WHITE));
            ui.label(
                RichText::new(format!("{:.0} ms", self.generation_ms.max(1.0)))
                    .color(Color32::WHITE),
            );
            ui.label(RichText::new("GPU Mem:").strong().color(Color32::WHITE));
            ui.label(RichText::new("2 %").color(Color32::WHITE));
            ui.label(RichText::new("CPU vMix:").strong().color(Color32::WHITE));
            ui.label(RichText::new("1 %").color(Color32::WHITE));
            ui.label(RichText::new("Total:").strong().color(Color32::WHITE));
            ui.label(RichText::new("11 %").color(Color32::WHITE));
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
            || self.overlay_settings.is_some()
            || self.recording_setup
            || self.streaming_setup
            || self.multicorder_open
            || self.playlist_open
            || self.shortcuts_open
            || self.confirm_remove_input.is_some()
            || self.add_open
            || self.about
            || self.confirm_new
            || self.confirm_close
            || self.confirm_open
        {
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
        });
        if !self.engine.project().preferences.shortcuts {
            return;
        }
        ctx.input_mut(|i| {
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
        self.automation.clear();
        self.now = u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.engine.tick(self.now);
        self.poll_job(&ctx);
        self.shortcuts(&ctx);
        self.refresh_textures(&ctx);
        let dialog_busy = self.add_open
            || self.properties.is_some()
            || self.settings
            || self.overlay_settings.is_some()
            || self.recording_setup
            || self.streaming_setup
            || self.multicorder_open
            || self.playlist_open
            || self.shortcuts_open
            || self.confirm_remove_input.is_some()
            || self.about
            || self.confirm_new
            || self.confirm_close
            || self.confirm_open;
        egui::Panel::top("toolbar").show(ui, |ui| {
            if dialog_busy {
                ui.disable();
            }
            self.toolbar(ui);
        });
        egui::Panel::bottom("status").show(ui, |ui| self.status(ui));
        egui::Panel::bottom("production.controls").show(ui, |ui| {
            if dialog_busy {
                ui.disable();
            }
            self.production_toolbar(ui);
        });
        let viewport_height = ctx.viewport_rect().height();
        let monitor_height = (viewport_height * 0.51).max(380.0);
        let monitor_id = egui::Id::new("monitor.bank");
        if let Some(previous) = self.monitor_view_height
            && (previous - viewport_height).abs() > 1.0
            && previous > 0.0
            && let Some(mut state) = egui::containers::panel::PanelState::load(&ctx, monitor_id)
        {
            let height = (state.outer_rect.height() * viewport_height / previous)
                .clamp(380.0, (viewport_height - 200.0).max(380.0));
            state.outer_rect.max.y = state.outer_rect.min.y + height;
            ctx.data_mut(|d| d.insert_persisted(monitor_id, state));
        }
        self.monitor_view_height = Some(viewport_height);
        egui::Panel::top("monitor.bank")
            .resizable(true)
            .default_size(monitor_height)
            .min_size(380.0)
            .max_size((viewport_height - 200.0).max(380.0))
            .show(ui, |ui| {
                if dialog_busy {
                    ui.disable();
                }
                self.switcher(ui);
            });
        if dialog_busy {
            ui.disable();
        }
        let mut dock = std::mem::replace(&mut self.dock, DockState::new(vec![]));
        let mut style = egui_dock::Style::from_egui(ui.style());
        style.tab_bar.bg_fill = BG;
        style.tab_bar.hline_color = Color32::from_rgb(62, 70, 78);
        style.tab.active.bg_fill = PROGRAM;
        style.tab.active.text_color = Color32::WHITE;
        style.tab.active.outline_color = Color32::from_rgb(40, 180, 60);
        style.tab.focused.bg_fill = PROGRAM;
        style.tab.focused.text_color = Color32::WHITE;
        style.tab.focused.outline_color = Color32::from_rgb(40, 180, 60);
        style.tab.inactive.bg_fill = Color32::from_rgb(38, 45, 52);
        style.tab.inactive.text_color = DIM;
        style.tab.hovered.bg_fill = Color32::from_rgb(58, 68, 78);
        style.tab.hovered.text_color = Color32::WHITE;
        style.separator.color_idle = Color32::from_rgb(48, 56, 64);
        style.separator.color_hovered = TEAL;
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
