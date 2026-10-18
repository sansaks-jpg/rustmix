#![forbid(unsafe_code)]
mod controls;
mod dialogs;
mod inputs;
mod media;
mod panels;
mod profile;
mod program;
mod reference_controls;
mod reference_smoke;
mod smoke;
mod snapshot;
mod workspace;

use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use rustmix_engine::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant},
};

pub(crate) const TEAL: Color32 = Color32::from_rgb(56, 196, 124);
pub(crate) const PREVIEW: Color32 = Color32::from_rgb(234, 128, 24);
pub(crate) const PROGRAM: Color32 = Color32::from_rgb(22, 163, 74);
pub(crate) const RED: Color32 = Color32::from_rgb(239, 68, 68);
pub(crate) const DIM: Color32 = Color32::from_rgb(156, 168, 182);
pub(crate) const BG: Color32 = Color32::from_rgb(17, 21, 28);
/// Idle source-tile header; Program and Preview override it with their own color.
pub(crate) const IDLE_TILE: Color32 = Color32::from_rgb(38, 46, 58);
/// Mixer side rails (`OUTPUTS` / `INPUTS`).
pub(crate) const MIXER_RAIL: Color32 = Color32::from_rgb(14, 116, 204);
/// Button and chip surface used across tiles, strips, and switcher stacks.
pub(crate) const CHIP_BG: Color32 = Color32::from_rgb(32, 39, 49);
/// Modern elevated card surface used inside operator dialogs.
pub(crate) const FORM_LIGHT: Color32 = Color32::from_rgb(24, 30, 39);
pub(crate) const FORM_TEXT: Color32 = Color32::from_rgb(232, 238, 245);

type CachedRgba = (u32, u32, Arc<Vec<[u8; 4]>>);

/// Decoded image cache plus the engine's committed processing configuration.
#[derive(Clone, Default)]
pub(crate) struct InputFx {
    pub config: InputConfig,
    pub custom_rgba: Option<CachedRgba>,
    pub image_size: Option<[u32; 2]>,
    pub image_error: Option<String>,
    pub image_pending: bool,
}
impl std::ops::Deref for InputFx {
    type Target = InputConfig;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}
impl std::ops::DerefMut for InputFx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}
impl InputFx {
    fn set_image(&mut self, image: media::DecodedPng) {
        self.media_path = Some(image.path);
        self.custom_rgba = Some((image.width, image.height, image.pixels));
        self.image_size = Some(image.original_size);
        self.image_error = None;
        self.image_pending = false;
    }
}

/// Clean procedural vector icons rendered via `egui::Painter` so no button ever depends on OS symbol/emoji fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VectorIcon {
    Gear,
    Speaker(bool),
    Headphones,
    Pin,
    Display,
    Search,
    Sliders,
    Grid,
    Camera,
    Lock(bool),
    Close,
    Play,
    Pause,
    Restart,
    Loop,
}

pub(crate) fn paint_vector_icon(
    painter: &egui::Painter,
    rect: Rect,
    icon: VectorIcon,
    color: Color32,
) {
    let c = rect.center();
    let s = Stroke::new(1.4, color);
    match icon {
        VectorIcon::Gear => {
            painter.circle_stroke(c, 3.6, Stroke::new(1.5, color));
            painter.circle_filled(c, 1.3, color);
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::FRAC_PI_4;
                let dir = egui::vec2(angle.cos(), angle.sin());
                painter.line_segment([c + dir * 3.6, c + dir * 5.6], Stroke::new(1.5, color));
            }
        }
        VectorIcon::Speaker(active) => {
            let pts = vec![
                c + egui::vec2(-5.0, -2.0),
                c + egui::vec2(-2.5, -2.0),
                c + egui::vec2(0.5, -4.8),
                c + egui::vec2(0.5, 4.8),
                c + egui::vec2(-2.5, 2.0),
                c + egui::vec2(-5.0, 2.0),
            ];
            painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
            if active {
                painter.line_segment(
                    [c + egui::vec2(2.4, -2.2), c + egui::vec2(3.4, 0.0)],
                    Stroke::new(1.3, color),
                );
                painter.line_segment(
                    [c + egui::vec2(3.4, 0.0), c + egui::vec2(2.4, 2.2)],
                    Stroke::new(1.3, color),
                );
                painter.line_segment(
                    [c + egui::vec2(4.2, -3.8), c + egui::vec2(5.4, 0.0)],
                    Stroke::new(1.3, color),
                );
                painter.line_segment(
                    [c + egui::vec2(5.4, 0.0), c + egui::vec2(4.2, 3.8)],
                    Stroke::new(1.3, color),
                );
            } else {
                painter.line_segment(
                    [c + egui::vec2(2.2, -2.5), c + egui::vec2(5.6, 2.5)],
                    Stroke::new(1.5, RED),
                );
                painter.line_segment(
                    [c + egui::vec2(5.6, -2.5), c + egui::vec2(2.2, 2.5)],
                    Stroke::new(1.5, RED),
                );
            }
        }
        VectorIcon::Headphones => {
            for idx in 0..6 {
                let a0 = std::f32::consts::PI + idx as f32 * std::f32::consts::PI / 6.0;
                let a1 = std::f32::consts::PI + (idx + 1) as f32 * std::f32::consts::PI / 6.0;
                painter.line_segment(
                    [
                        c + egui::vec2(a0.cos() * 4.8, a0.sin() * 4.5 + 0.5),
                        c + egui::vec2(a1.cos() * 4.8, a1.sin() * 4.5 + 0.5),
                    ],
                    Stroke::new(1.5, color),
                );
            }
            painter.rect_filled(
                Rect::from_center_size(c + egui::vec2(-4.4, 2.2), egui::vec2(2.4, 4.4)),
                1.0,
                color,
            );
            painter.rect_filled(
                Rect::from_center_size(c + egui::vec2(4.4, 2.2), egui::vec2(2.4, 4.4)),
                1.0,
                color,
            );
        }
        VectorIcon::Pin => {
            painter.circle_filled(c + egui::vec2(0.0, -2.2), 2.8, color);
            painter.line_segment([c + egui::vec2(0.0, 0.4), c + egui::vec2(0.0, 5.2)], s);
            painter.line_segment([c + egui::vec2(-3.2, 0.4), c + egui::vec2(3.2, 0.4)], s);
        }
        VectorIcon::Display => {
            let screen = Rect::from_center_size(c + egui::vec2(0.0, -1.0), egui::vec2(10.0, 6.8));
            painter.rect_stroke(screen, 1.0, s, egui::StrokeKind::Inside);
            painter.line_segment([c + egui::vec2(0.0, 2.4), c + egui::vec2(0.0, 4.6)], s);
            painter.line_segment([c + egui::vec2(-3.0, 4.6), c + egui::vec2(3.0, 4.6)], s);
        }
        VectorIcon::Search => {
            let lens = c + egui::vec2(-1.2, -1.2);
            painter.circle_stroke(lens, 3.6, s);
            painter.line_segment([lens + egui::vec2(2.5, 2.5), c + egui::vec2(4.6, 4.6)], s);
        }
        VectorIcon::Sliders => {
            for (dy, kx) in [(-3.6_f32, -1.8_f32), (0.0, 2.0), (3.6, -0.5)] {
                painter.line_segment([c + egui::vec2(-5.2, dy), c + egui::vec2(5.2, dy)], s);
                painter.circle_filled(c + egui::vec2(kx, dy), 1.8, color);
            }
        }
        VectorIcon::Grid => {
            for dx in [-2.6_f32, 2.6] {
                for dy in [-2.6_f32, 2.6] {
                    painter.rect_stroke(
                        Rect::from_center_size(c + egui::vec2(dx, dy), egui::vec2(4.0, 4.0)),
                        0.8,
                        s,
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }
        VectorIcon::Camera => {
            let body = Rect::from_center_size(c + egui::vec2(0.0, 0.8), egui::vec2(10.8, 7.2));
            painter.rect_stroke(body, 1.2, s, egui::StrokeKind::Inside);
            painter.line_segment([c + egui::vec2(-2.4, -2.8), c + egui::vec2(2.4, -2.8)], s);
            painter.circle_stroke(c + egui::vec2(0.0, 0.8), 2.2, s);
        }
        VectorIcon::Lock(locked) => {
            let body = Rect::from_center_size(c + egui::vec2(0.0, 1.8), egui::vec2(8.4, 6.0));
            painter.rect_filled(body, 1.2, color);
            let shackle_top = if locked { -4.4 } else { -5.4 };
            painter.line_segment(
                [
                    c + egui::vec2(-2.4, -1.2),
                    c + egui::vec2(-2.4, shackle_top),
                ],
                s,
            );
            painter.line_segment(
                [
                    c + egui::vec2(-2.4, shackle_top),
                    c + egui::vec2(2.4, shackle_top),
                ],
                s,
            );
            if locked {
                painter.line_segment(
                    [c + egui::vec2(2.4, shackle_top), c + egui::vec2(2.4, -1.2)],
                    s,
                );
            }
        }
        VectorIcon::Close => {
            painter.line_segment(
                [c + egui::vec2(-3.2, -3.2), c + egui::vec2(3.2, 3.2)],
                Stroke::new(1.6, color),
            );
            painter.line_segment(
                [c + egui::vec2(3.2, -3.2), c + egui::vec2(-3.2, 3.2)],
                Stroke::new(1.6, color),
            );
        }
        VectorIcon::Play => {
            let pts = vec![
                c + egui::vec2(-2.5, -3.8),
                c + egui::vec2(3.8, 0.0),
                c + egui::vec2(-2.5, 3.8),
            ];
            painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
        }
        VectorIcon::Pause => {
            painter.rect_filled(
                Rect::from_center_size(c + egui::vec2(-2.2, 0.0), egui::vec2(2.2, 7.2)),
                0.5,
                color,
            );
            painter.rect_filled(
                Rect::from_center_size(c + egui::vec2(2.2, 0.0), egui::vec2(2.2, 7.2)),
                0.5,
                color,
            );
        }
        VectorIcon::Restart => {
            painter.line_segment([c + egui::vec2(-3.2, -3.6), c + egui::vec2(-3.2, 3.6)], s);
            let pts = vec![
                c + egui::vec2(3.4, -3.6),
                c + egui::vec2(-1.8, 0.0),
                c + egui::vec2(3.4, 3.6),
            ];
            painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
        }
        VectorIcon::Loop => {
            let r = Rect::from_center_size(c, egui::vec2(8.8, 5.6));
            painter.rect_stroke(r, 2.0, s, egui::StrokeKind::Inside);
        }
    }
}

pub(crate) fn vector_icon_button(
    ui: &mut egui::Ui,
    icon: VectorIcon,
    size: Vec2,
    selected: bool,
    fill: Option<Color32>,
) -> egui::Response {
    let mut btn = egui::Button::new("").selected(selected).min_size(size);
    if let Some(f) = fill {
        btn = btn.fill(f);
    }
    let resp = ui.add(btn);
    paint_vector_icon(ui.painter(), resp.rect, icon, Color32::WHITE);
    resp
}

pub(crate) fn caret_menu_button<R>(
    ui: &mut egui::Ui,
    height: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<Option<R>> {
    let inner = ui.menu_button(" ", add_contents);
    let c = inner.response.rect.center();
    let pts = vec![
        Pos2::new(c.x - 3.6, c.y - 1.6),
        Pos2::new(c.x + 3.6, c.y - 1.6),
        Pos2::new(c.x, c.y + 2.4),
    ];
    let _ = height;
    ui.painter().add(egui::Shape::convex_polygon(
        pts,
        Color32::WHITE,
        Stroke::NONE,
    ));
    inner
}

/// Category palette shared by the input filter chips and dialog category selectors.
pub(crate) fn kind_color(kind: InputKind) -> Color32 {
    match kind {
        InputKind::Grid => Color32::from_rgb(185, 28, 28),
        InputKind::Geometry => PROGRAM,
        InputKind::Bars => PREVIEW,
        InputKind::Graphics => Color32::from_rgb(147, 51, 234),
        InputKind::Studio => Color32::from_rgb(14, 165, 233),
        InputKind::Color => Color32::from_rgb(59, 130, 246),
        InputKind::Countdown => Color32::from_rgb(20, 184, 166),
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
    Categories,
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
            Self::Audio => "Audio Mixer",
            Self::Overlays => "Overlay / Graphics",
        }
    }
}

fn default_dock(audio_focus: bool) -> DockState<Panel> {
    let mut dock = DockState::new(vec![Panel::Inputs, Panel::Overlays]);
    dock.main_surface_mut().split_right(
        NodeIndex::root(),
        if audio_focus { 0.59 } else { 0.76 },
        vec![Panel::Audio],
    );
    dock
}

pub(crate) enum Action {
    Unavailable(&'static str),
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
    Snapshot,
    About,
    Workspace(bool),
    BasicMode,
    Multiview,
    Fullscreen,
    Close,
    Recover,
    ConfirmOpen,
    CloseDiscard,
    ToggleAudioPane,
    PauseDemo,
    Last,
}

enum JobDone {
    Saved(PathBuf, u64, bool),
    SnapshotSaved(PathBuf),
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
    pub(crate) last_program_rgba: Vec<u8>,
    program_worker: Option<program::ProgramWorker>,
    program_generation: u64,
    program_renderer: program::ProgramRenderer,
    program_pixels: Option<Arc<Vec<[u8; 4]>>>,
    video_stats: program::VideoStats,
    video_timing: Option<(video_clock::FrameTiming, Rate)>,
    video_produced_ns: u64,
    profile: Option<profile::Profile>,
    pub(crate) input_fx: HashMap<InputId, InputFx>,
    media_loader: media::MediaLoader,
    image_picker: Option<Receiver<media::PickerDone>>,
    add_image: Option<InputFx>,
    properties_fx: Option<InputFx>,
    properties_audio_original: Option<AudioChannel>,
    properties_token: u64,
    properties_loading: bool,
    properties_texture: Option<egui::TextureHandle>,
    add_token: u64,
    add_loading: bool,
    pub(crate) monitor_level: f32,
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
    /// Staged Overlay Settings draft: selected slot plus its uncommitted copy.
    pub(crate) overlay_settings: Option<(usize, OverlayState)>,
    /// Reference dialogs and live production output engines.
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
    pub(crate) rec_started_ms: u64,
    pub(crate) rec_frames: u64,
    pub(crate) rec_bytes: u64,
    pub(crate) streaming_setup: bool,
    pub(crate) stream_channel: usize,
    pub(crate) stream_destination: usize,
    pub(crate) stream_url: String,
    pub(crate) stream_key: String,
    pub(crate) stream_use_q1: bool,
    pub(crate) stream_quality: usize,
    pub(crate) stream_active: bool,
    pub(crate) stream_started_ms: u64,
    pub(crate) stream_frames: u64,
    pub(crate) stream_kbps: u32,
    pub(crate) external_active: bool,
    pub(crate) multicorder_open: bool,
    pub(crate) mc_format: usize,
    pub(crate) mc_running: bool,
    pub(crate) mc_started_ms: u64,
    pub(crate) mc_ndi_mov: bool,
    pub(crate) mc_show_unsupported: bool,
    pub(crate) mc_folder: String,
    pub(crate) mc_checked: [bool; 12],
    pub(crate) playlist_open: bool,
    pub(crate) playlist_selected_id: u32,
    pub(crate) playlist_selected_avail: usize,
    pub(crate) playlist_selected_item: usize,
    pub(crate) playlist_name: String,
    pub(crate) playlist_delete_confirm: bool,
    pub(crate) shortcuts_open: bool,
    pub(crate) search_open: bool,
    pub(crate) search_draft: String,
    pub(crate) audio_restore: Option<DockState<Panel>>,
    pub(crate) ui_locked: bool,
    pub(crate) basic_mode: bool,
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
    pub(crate) category: Option<u8>,
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
        if let Some(storage) = cc.storage {
            app.basic_mode = storage.get_string("basic-mode.v1").as_deref() == Some("true");
        }
        if let Some(path) = std::env::args()
            .skip(1)
            .find(|s| s.ends_with(".rustmix.json"))
        {
            app.start_load(Some(PathBuf::from(path)));
        }
        if std::env::args().any(|s| {
            matches!(
                s.as_str(),
                "--ui-smoke"
                    | "--ui-smoke-reopen"
                    | "--ui-input-smoke"
                    | "--ui-input-smoke-reopen"
                    | "--ui-reference-smoke"
                    | "--ui-reference-smoke-reopen"
            )
        }) {
            app.smoke = Some(smoke::Smoke::default());
        }
        app.theme(&cc.egui_ctx);
        #[cfg(target_os = "windows")]
        if let Some(windows) = std::env::var_os("WINDIR") {
            let font_path = PathBuf::from(&windows).join("Fonts").join("segoeui.ttf");
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
        app.program_worker = Some(program::ProgramWorker::spawn(
            program::Snapshot {
                generation: app.program_generation,
                engine: app.engine.clone(),
                sources: app.input_fx.clone(),
            },
            app.start,
            Some(cc.egui_ctx.clone()),
        ));
        if std::env::args().any(|s| s == "--ui-perf") {
            app.profile = Some(profile::Profile::default());
        }
        app
    }

    fn headless() -> Self {
        let engine = Engine::default();
        let mut input_fx = HashMap::new();
        for inp in &engine.project().inputs {
            input_fx.insert(inp.id, InputFx::default());
        }
        Self {
            engine,
            dock: default_dock(false),
            actions: Vec::new(),
            start: Instant::now(),
            now: 0,
            textures: HashMap::new(),
            program_texture: None,
            last_program_rgba: Vec::new(),
            program_worker: None,
            program_generation: 0,
            program_renderer: program::ProgramRenderer::default(),
            program_pixels: None,
            video_stats: program::VideoStats::default(),
            video_timing: None,
            video_produced_ns: 0,
            profile: None,
            input_fx,
            media_loader: media::MediaLoader::default(),
            image_picker: None,
            add_image: None,
            properties_fx: None,
            properties_audio_original: None,
            properties_token: 0,
            properties_loading: false,
            properties_texture: None,
            add_token: 0,
            add_loading: false,
            monitor_level: 0.78,
            last_demo_frame: None,
            notice: "Ready · offline synthetic workspace".into(),
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
            overlay_settings: None,
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
            rec_started_ms: 0,
            rec_frames: 0,
            rec_bytes: 0,
            streaming_setup: false,
            stream_channel: 1,
            stream_destination: 0,
            stream_url: "rtmp://a.rtmp.youtube.com/live2".into(),
            stream_key: String::new(),
            stream_use_q1: true,
            stream_quality: 0,
            stream_active: false,
            stream_started_ms: 0,
            stream_frames: 0,
            stream_kbps: 4500,
            external_active: false,
            multicorder_open: false,
            mc_format: 0,
            mc_running: false,
            mc_started_ms: 0,
            mc_ndi_mov: false,
            mc_show_unsupported: false,
            mc_folder: r"C:\Broadcast\MultiCorder".into(),
            mc_checked: [
                true, true, true, false, false, false, false, false, true, false, false, false,
            ],
            playlist_open: false,
            playlist_selected_id: 1,
            playlist_selected_avail: 0,
            playlist_selected_item: 0,
            playlist_name: "Default".into(),
            playlist_delete_confirm: false,
            shortcuts_open: false,
            search_open: false,
            search_draft: String::new(),
            audio_restore: None,
            ui_locked: false,
            basic_mode: false,
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
            category: None,
            tbar_latched: false,
            monitor_view_height: None,
        }
    }

    pub(crate) fn sync_prop_from_fx(&mut self, id: InputId) {
        self.properties_token = self.properties_token.wrapping_add(1);
        self.properties_loading = false;
        self.properties_texture = None;
        self.properties_audio_original = self.engine.project().input(id).map(|i| i.audio.clone());
        self.properties_fx = Some(self.input_fx.get(&id).cloned().unwrap_or_else(|| {
            InputFx {
                config: self
                    .engine
                    .project()
                    .input(id)
                    .map(|i| i.config.clone())
                    .unwrap_or_default(),
                ..Default::default()
            }
        }));
        self.load_draft_controls();
    }

    fn load_draft_controls(&mut self) {
        let fx = self.properties_fx.clone().unwrap_or_default();
        self.prop_zoom = fx.zoom;
        self.prop_pan_x = fx.pan_x;
        self.prop_pan_y = fx.pan_y;
        self.prop_mirror = fx.mirror;
        self.prop_sharpen = fx.sharpen;
        self.prop_lift = fx.lift;
        self.prop_gain = fx.gain;
        self.prop_saturation = fx.saturation;
    }

    pub(crate) fn sync_fx_from_prop(&mut self, _id: InputId) {
        let Some(fx) = self.properties_fx.as_mut() else {
            return;
        };
        fx.zoom = self.prop_zoom;
        fx.pan_x = self.prop_pan_x;
        fx.pan_y = self.prop_pan_y;
        fx.mirror = self.prop_mirror;
        fx.sharpen = self.prop_sharpen;
        fx.lift = self.prop_lift;
        fx.gain = self.prop_gain;
        fx.saturation = self.prop_saturation;
        self.last_demo_frame = None;
    }

    fn reset_editors(&mut self) {
        self.search_open = false;
        self.category = None;
        self.input_filter.clear();
        self.add_open = false;
        self.add_image = None;
        self.add_loading = false;
        self.add_token = self.add_token.wrapping_add(1);
        self.close_properties();
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
        self.playlist_selected_id = self.engine.project().playlists.first().map_or(1, |p| p.id);
        self.playlist_name = self
            .engine
            .project()
            .playlists
            .first()
            .map_or_else(String::new, |p| p.name.clone());
        self.playlist_selected_item = 0;
        self.playlist_delete_confirm = false;
        self.shortcuts_open = false;
    }

    fn theme(&self, ctx: &egui::Context) {
        let light = self.engine.project().preferences.ui_theme == UiTheme::Light;
        let mut style = egui::Style {
            visuals: if light {
                egui::Visuals::light()
            } else {
                egui::Visuals::dark()
            },
            ..Default::default()
        };
        style.spacing.item_spacing = egui::vec2(4.0, 3.0);
        style.spacing.button_padding = egui::vec2(7.0, 3.0);
        if !light {
            style.visuals.panel_fill = BG;
            style.visuals.window_fill = Color32::from_rgb(20, 25, 33);
            style.visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(56, 68, 84));
            style.visuals.window_corner_radius = egui::CornerRadius::same(6);
            style.visuals.extreme_bg_color = Color32::from_rgb(12, 15, 20);
            style.visuals.faint_bg_color = Color32::from_rgb(26, 32, 42);
            style.visuals.widgets.inactive.bg_fill = CHIP_BG;
            style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(30, 37, 47);
            style.visuals.widgets.inactive.bg_stroke =
                Stroke::new(1.0, Color32::from_rgb(58, 70, 84));
            style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(52, 64, 78);
            style.visuals.widgets.hovered.bg_stroke =
                Stroke::new(1.0, Color32::from_rgb(96, 114, 134));
        }
        style.visuals.widgets.active.bg_fill = self.program_color();
        style.visuals.selection.bg_fill = self.program_color();
        style.visuals.selection.stroke = egui::Stroke::new(1.0, Color32::WHITE);
        style.visuals.override_text_color = if light {
            Some(Color32::from_rgb(25, 31, 40))
        } else {
            Some(if self.engine.project().preferences.high_contrast {
                Color32::WHITE
            } else {
                Color32::from_rgb(232, 238, 245)
            })
        };
        for v in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
        ] {
            v.corner_radius = egui::CornerRadius::same(4);
        }
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(12.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(12.0));
        ctx.set_theme(if light {
            egui::ThemePreference::Light
        } else {
            egui::ThemePreference::Dark
        });
        ctx.set_style_of(
            if light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            },
            style,
        );
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
        let _ = self.try_execute(c);
        self.submit_program();
    }

    fn submit_program(&mut self) {
        if self.program_worker.is_none() {
            return;
        }
        let Some(generation) = self.program_generation.checked_add(1) else {
            self.say(
                "Program update limit reached; last valid output retained".into(),
                true,
            );
            return;
        };
        self.program_generation = generation;
        if let Some(worker) = &self.program_worker {
            worker.submit(program::Snapshot {
                generation,
                engine: self.engine.clone(),
                sources: self.input_fx.clone(),
            });
        }
    }

    fn try_execute(&mut self, c: Command) -> Option<CommandResult> {
        let label = c.definition().label;
        if self.program_worker.is_some() {
            self.now = u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX);
        }
        match self.engine.execute(c, self.now) {
            Ok(result) => {
                self.say(format!("{label} completed"), false);
                self.sync_input_cache();
                self.last_demo_frame = None;
                Some(result)
            }
            Err(e) => {
                self.say(e.to_string(), true);
                None
            }
        }
    }

    fn recovery_path() -> PathBuf {
        let base = std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("XDG_DATA_HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        base.join("RustMix").join("recovery.rustmix.json")
    }

    fn capture_workspace(&mut self) {
        match serde_json::to_string(self.audio_restore.as_ref().unwrap_or(&self.dock)) {
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
                    JobDone::Loaded(project, path) => {
                        self.audio_restore = None;
                        match Engine::from_project_at(*project, self.now) {
                            Ok(engine) => {
                                let layout = engine.project().workspace.dock_json.clone();
                                self.engine = engine;
                                self.reset_editors();
                                self.media_loader.reset();
                                self.input_fx.clear();
                                self.sync_input_cache();
                                self.submit_program();
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
                        }
                    }
                    JobDone::SnapshotSaved(path) => self.say(
                        format!(
                            "Program snapshot saved: {} (temporary snapshots folder)",
                            path.file_name()
                                .map(|name| name.to_string_lossy().into_owned())
                                .unwrap_or_else(|| "snapshot.png".into())
                        ),
                        false,
                    ),
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
            Action::Unavailable(reason) => self.say(reason.into(), true),
            Action::Engine(c) => {
                let appearance = matches!(
                    c,
                    Command::SetPreferences(_) | Command::ApplySettings { .. }
                );
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
                self.add_token = self.add_token.wrapping_add(1);
                self.add_loading = false;
                self.add_number = self.engine.project().inputs.len() + 1;
                self.add_tab = match self.add_kind {
                    InputKind::Grid => 0,
                    InputKind::Geometry => 3,
                    InputKind::Studio => 4,
                    InputKind::Bars => 6,
                    InputKind::Color => 11,
                    InputKind::Graphics => 14,
                    InputKind::Countdown => 15,
                };
                self.add_open = true;
            }
            Action::Properties(id) => {
                self.sync_prop_from_fx(id);
                self.properties = self.engine.project().input(id).cloned();
                self.input_page = InputPage::General;
            }
            Action::PropertiesAt(id, page) => {
                self.sync_prop_from_fx(id);
                self.properties = self.engine.project().input(id).cloned();
                self.input_page = page;
            }
            Action::InputAudio(id) => {
                self.sync_prop_from_fx(id);
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
            Action::ToggleRecordSim | Action::ToggleStreamSim | Action::ToggleExternalSim => {
                self.say("Unavailable: this build has no encoder, network sender or external video backend".into(), true);
            }
            Action::Snapshot => self.start_snapshot(std::env::temp_dir().join("rustmix-snapshots")),
            Action::About => self.about = true,
            Action::Workspace(audio) => {
                self.audio_restore = None;
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
            Action::BasicMode => self.basic_mode = !self.basic_mode,
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
            Action::ToggleAudioPane => self.toggle_audio_pane(),
            Action::PauseDemo => {
                self.execute(Command::SetInputsPaused(!self.engine.inputs_paused()));
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
            ui.spacing_mut().item_spacing.x = 3.0;
            let brand = ui
                .add(
                    egui::Button::new(
                        RichText::new("Preset")
                            .strong()
                            .color(ui.visuals().text_color()),
                    )
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
            if !self.basic_mode
                && ui
                    .add_enabled(
                        self.job.is_none(),
                        egui::Button::new(RichText::new("New").strong())
                            .min_size(egui::vec2(56.0, 22.0)),
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
                        .min_size(egui::vec2(56.0, 22.0)),
                )
                .on_hover_text("Open preset · Ctrl+O")
                .clicked()
            {
                self.actions.push(Action::Open);
            }
            ui.add_enabled_ui(self.job.is_none(), |ui| {
                let open_menu = caret_menu_button(ui, 22.0, |ui| {
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
                        .min_size(egui::vec2(56.0, 22.0)),
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
                let save_menu = caret_menu_button(ui, 22.0, |ui| {
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
            if !self.basic_mode
                && ui
                    .add_enabled(
                        self.path.is_some() && self.job.is_none(),
                        egui::Button::new(RichText::new("Last").strong())
                            .min_size(egui::vec2(52.0, 22.0)),
                    )
                    .on_disabled_hover_text("Save or open a preset first")
                    .on_hover_text("Reload the most recently saved preset")
                    .clicked()
            {
                self.actions.push(Action::Last);
            }
            ui.add_space((ui.max_rect().width() * 0.48 - ui.cursor().left() - 48.0).max(4.0));
            if !self.basic_mode {
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
                    .on_hover_text(
                        "Expand the application window; independent Program output is unavailable",
                    );
                self.automation
                    .insert("toolbar.fullscreen".into(), fullscreen.rect);
                if fullscreen.clicked() {
                    self.actions.push(Action::Fullscreen);
                }
                caret_menu_button(ui, 22.0, |ui| {
                    ui.label("Independent fullscreen routing is unavailable.");
                });
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
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
                let settings_menu = caret_menu_button(ui, 22.0, |ui| {
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
                let mode = ui
                    .add(
                        egui::Button::new(
                            RichText::new(if self.basic_mode { "Advanced" } else { "Basic" })
                                .strong(),
                        )
                        .selected(self.basic_mode)
                        .min_size(egui::vec2(54.0, 22.0)),
                    )
                    .on_hover_text("Show Basic or Advanced operator controls");
                self.automation.insert("toolbar.basic".into(), mode.rect);
                if mode.clicked() {
                    self.actions.push(Action::BasicMode);
                }
                if !self.basic_mode
                    && ui
                        .add(
                            egui::Button::new(RichText::new("Pause Inputs").strong())
                                .selected(self.engine.inputs_paused())
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
            ui.spacing_mut().item_spacing.x = 3.0;
            let add = ui
                .add(
                    egui::Button::new(RichText::new("Add Input").strong())
                        .min_size(egui::vec2(72.0, 24.0)),
                )
                .on_hover_text("Open Input Select · Ctrl+I");
            self.automation.insert("production.add".into(), add.rect);
            if add.clicked() {
                self.actions.push(Action::Add);
            }
            if self.basic_mode {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.workspace_lock(ui);
                });
                return;
            }
            let caret = caret_menu_button(ui, 24.0, |ui| {
                for (idx, (label, kind)) in [
                    ("Generators / PNG", InputKind::Grid),
                    ("Moving shapes", InputKind::Geometry),
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
            let rec_gear =
                vector_icon_button(ui, VectorIcon::Gear, egui::vec2(26.0, 24.0), false, None)
                    .on_hover_text("Open Recording Setup");
            self.automation
                .insert("output.record.settings".into(), rec_gear.rect);
            if rec_gear.clicked() {
                self.actions.push(Action::RecordingSetup);
            }
            let rec_secs = if self.rec_active {
                self.now.saturating_sub(self.rec_started_ms) / 1000
            } else {
                0
            };
            let rec_lbl = if self.rec_active {
                format!("REC {:02}:{:02}", rec_secs / 60, rec_secs % 60)
            } else {
                "Record".into()
            };
            let mut rec_btn_w =
                egui::Button::new(RichText::new(rec_lbl).strong()).min_size(egui::vec2(68.0, 24.0));
            if self.rec_active {
                rec_btn_w = rec_btn_w.fill(Color32::from_rgb(200, 30, 30));
            }
            let rec_btn = ui
                .add_enabled(false, rec_btn_w)
                .on_hover_text("Unavailable: no recording encoder backend");
            self.automation.insert("output.record".into(), rec_btn.rect);
            if rec_btn.clicked() {
                self.actions.push(Action::ToggleRecordSim);
            }
            ui.add_space(10.0);

            // 2. External: [ ⚙ | External ]
            let ext_gear = ui.menu_button(" ", |ui| {
                ui.set_min_width(210.0);
                for item in [
                    "Output 2  (Program / Clean Feed)",
                    "Output 3  (Preview / Multiview)",
                    "Output 4  (Aux / Guest Return)",
                    "SRT / NDI Hardware Output",
                ] {
                    if ui.button(item).clicked() {
                        self.say(format!("External route selected: {item}"), false);
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Outputs / NDI / OMT / SRT Settings…").clicked() {
                    self.actions.push(Action::SettingsAt(SettingsPage::Outputs));
                    ui.close();
                }
                if ui.button("External Output Settings…").clicked() {
                    self.actions
                        .push(Action::SettingsAt(SettingsPage::ExternalOutput));
                    ui.close();
                }
            });
            paint_vector_icon(
                ui.painter(),
                ext_gear.response.rect,
                VectorIcon::Gear,
                Color32::WHITE,
            );
            self.automation
                .insert("output.external.settings".into(), ext_gear.response.rect);
            let mut ext_btn_w = egui::Button::new(
                RichText::new(if self.external_active {
                    "EXT ON"
                } else {
                    "External"
                })
                .strong(),
            )
            .min_size(egui::vec2(66.0, 24.0));
            if self.external_active {
                ext_btn_w = ext_btn_w.fill(Color32::from_rgb(200, 30, 30));
            }
            let ext_btn = ui
                .add_enabled(false, ext_btn_w)
                .on_hover_text("Unavailable: no external video backend");
            self.automation
                .insert("output.external".into(), ext_btn.rect);
            if ext_btn.clicked() {
                self.actions.push(Action::ToggleExternalSim);
            }
            ui.add_space(10.0);

            // 3. Stream: [ ⚙ | Stream | ▾ ]
            let stm_gear =
                vector_icon_button(ui, VectorIcon::Gear, egui::vec2(26.0, 24.0), false, None)
                    .on_hover_text("Open Streaming Settings");
            self.automation
                .insert("output.stream.settings".into(), stm_gear.rect);
            if stm_gear.clicked() {
                self.actions.push(Action::StreamingSetup);
            }
            let stm_secs = if self.stream_active {
                self.now.saturating_sub(self.stream_started_ms) / 1000
            } else {
                0
            };
            let stm_lbl = if self.stream_active {
                format!("LIVE {:02}:{:02}", stm_secs / 60, stm_secs % 60)
            } else {
                "Stream".into()
            };
            let mut stm_btn_w =
                egui::Button::new(RichText::new(stm_lbl).strong()).min_size(egui::vec2(64.0, 24.0));
            if self.stream_active {
                stm_btn_w = stm_btn_w.fill(Color32::from_rgb(200, 30, 30));
            }
            let stm_btn = ui
                .add_enabled(false, stm_btn_w)
                .on_hover_text("Unavailable: no streaming sender backend");
            self.automation.insert("output.stream".into(), stm_btn.rect);
            if stm_btn.clicked() {
                self.actions.push(Action::ToggleStreamSim);
            }
            caret_menu_button(ui, 24.0, |ui| {
                for ch in 1..=3 {
                    if ui.button(format!("Destination {ch} Settings…")).clicked() {
                        self.stream_channel = ch;
                        self.actions.push(Action::StreamingSetup);
                        ui.close();
                    }
                }
            });
            ui.add_space(10.0);

            // 4. MultiCorder: [ ⚙ | MultiCorder ]
            let mc_gear =
                vector_icon_button(ui, VectorIcon::Gear, egui::vec2(26.0, 24.0), false, None)
                    .on_hover_text("Open MultiCorder");
            self.automation
                .insert("output.multicorder.settings".into(), mc_gear.rect);
            if mc_gear.clicked() {
                self.actions.push(Action::MultiCorder);
            }
            let mut mc_btn_w = egui::Button::new(RichText::new("MultiCorder").strong())
                .min_size(egui::vec2(82.0, 24.0));
            if self.mc_running {
                mc_btn_w = mc_btn_w.fill(PROGRAM);
            }
            let mc_btn = ui.add(mc_btn_w).on_hover_text("Open MultiCorder");
            self.automation
                .insert("output.multicorder".into(), mc_btn.rect);
            if mc_btn.clicked() {
                self.actions.push(Action::MultiCorder);
            }
            ui.add_space(10.0);

            // 5. PlayList: [ ⚙ | PlayList ]
            let pl_gear =
                vector_icon_button(ui, VectorIcon::Gear, egui::vec2(26.0, 24.0), false, None)
                    .on_hover_text("Open PlayList");
            self.automation
                .insert("output.playlist.settings".into(), pl_gear.rect);
            if pl_gear.clicked() {
                self.actions.push(Action::PlayList);
            }
            let mut pl_btn_w = egui::Button::new(
                RichText::new(if self.engine.playlist_run().is_some() {
                    "PlayList ON"
                } else {
                    "PlayList"
                })
                .strong(),
            )
            .min_size(egui::vec2(68.0, 24.0));
            if self.engine.playlist_run().is_some() {
                pl_btn_w = pl_btn_w.fill(PROGRAM);
            }
            let playlist_command = self.playlist_toggle_command();
            let available = self.engine.enabled(&playlist_command);
            let pl_btn = ui
                .add_enabled(available.is_ok(), pl_btn_w)
                .on_hover_text("Start / stop playlist; use the gear to configure")
                .on_disabled_hover_text(available.err().map(|e| e.to_string()).unwrap_or_default());
            self.automation
                .insert("output.playlist".into(), pl_btn.rect);
            if pl_btn.clicked() {
                self.actions.push(Action::Engine(playlist_command));
            }

            // Right-side modern vector icon cluster
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                self.workspace_lock(ui);
                let snap =
                    vector_icon_button(ui, VectorIcon::Camera, egui::vec2(26.0, 24.0), false, None)
                        .on_hover_text("Save Program snapshot to PNG");
                if snap.clicked() {
                    self.actions.push(Action::Snapshot);
                }
                let mv = vector_icon_button(
                    ui,
                    VectorIcon::Grid,
                    egui::vec2(26.0, 24.0),
                    self.multiview,
                    None,
                )
                .on_hover_text("Toggle Multiview layout");
                self.automation.insert("output.multiview".into(), mv.rect);
                if mv.clicked() {
                    self.actions.push(Action::Multiview);
                }
                // Colored audio bar icon button
                let (audio_rect, audio_resp) =
                    ui.allocate_exact_size(egui::vec2(26.0, 24.0), Sense::click());
                ui.painter().rect_filled(audio_rect, 4.0, CHIP_BG);
                ui.painter().rect_stroke(
                    audio_rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgb(58, 70, 84)),
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
                    let bx = audio_rect.left() + 6.0 + idx as f32 * 5.0;
                    let bar = egui::Rect::from_min_max(
                        egui::pos2(bx, audio_rect.bottom() - 4.0 - 14.0 * h),
                        egui::pos2(bx + 3.5, audio_rect.bottom() - 4.0),
                    );
                    ui.painter().rect_filled(bar, 1.0, col);
                }
                self.automation
                    .insert("output.audioPane".into(), audio_resp.rect);
                if audio_resp
                    .on_hover_text("Show / hide Audio Mixer")
                    .clicked()
                {
                    self.actions.push(Action::ToggleAudioPane);
                }
                let sc_btn = vector_icon_button(
                    ui,
                    VectorIcon::Sliders,
                    egui::vec2(26.0, 24.0),
                    self.shortcuts_open,
                    None,
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

    fn workspace_lock(&mut self, ui: &mut egui::Ui) {
        let lock = vector_icon_button(
            ui,
            VectorIcon::Lock(self.ui_locked),
            egui::vec2(26.0, 24.0),
            self.ui_locked,
            None,
        )
        .on_hover_text(
            "Lock production shortcuts and tab dragging (pointer switching stays available)",
        );
        self.automation.insert("output.lock".into(), lock.rect);
        if lock.clicked() {
            self.ui_locked = !self.ui_locked;
        }
    }

    fn status(&mut self, ui: &mut egui::Ui) {
        let p = self.engine.project();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.label(
                RichText::new(format!("640×360/{}p", p.preferences.rate.label()))
                    .strong()
                    .color(TEAL),
            );
            ui.label(
                RichText::new("CPU mix · GPU display")
                    .strong()
                    .color(ui.visuals().text_color()),
            );
            if self.program_worker.is_some() {
                ui.label(
                    RichText::new(format!(
                        "PGM {:.1} fps · drop {}",
                        self.video_stats.fps, self.video_stats.skipped
                    ))
                    .color(DIM),
                );
            }
            ui.label(RichText::new("UI FPS:").strong().color(DIM));
            ui.label(RichText::new(format!("{:.0}", self.fps)).color(ui.visuals().text_color()));
            ui.label(RichText::new("CPU:").strong().color(DIM));
            ui.label(
                RichText::new(format!("{:.1} ms", self.generation_ms))
                    .color(ui.visuals().text_color()),
            );
            if self.rec_active {
                let s = self.now.saturating_sub(self.rec_started_ms) / 1000;
                ui.label(
                    RichText::new(format!(
                        "● REC {:02}:{:02} ({:.1} MB)",
                        s / 60,
                        s % 60,
                        self.rec_bytes as f64 / 1_048_576.0
                    ))
                    .strong()
                    .color(RED),
                );
            }
            if self.stream_active {
                let s = self.now.saturating_sub(self.stream_started_ms) / 1000;
                ui.label(
                    RichText::new(format!(
                        "● LIVE {:02}:{:02} ({} kbps)",
                        s / 60,
                        s % 60,
                        self.stream_kbps
                    ))
                    .strong()
                    .color(RED),
                );
            }
            if self.engine.playlist_run().is_some() {
                let manual = self
                    .engine
                    .playlist_run()
                    .and_then(|run| {
                        self.engine
                            .project()
                            .playlists
                            .iter()
                            .find(|p| p.id == run.id)
                    })
                    .is_some_and(|p| p.manual);
                ui.label(
                    RichText::new(if manual {
                        "● PLAYLIST MANUAL"
                    } else {
                        "● PLAYLIST AUTO"
                    })
                    .strong()
                    .color(TEAL),
                );
            }
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
            || self.search_open
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
        for (modifiers, key, action) in [
            (
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::S,
                Action::Save(true),
            ),
            (egui::Modifiers::CTRL, egui::Key::S, Action::Save(false)),
            (egui::Modifiers::CTRL, egui::Key::O, Action::Open),
            (egui::Modifiers::CTRL, egui::Key::I, Action::Add),
        ] {
            if controls::consume_press(ctx, modifiers, key) {
                self.actions.push(action);
            }
        }
        if !self.engine.project().preferences.shortcuts {
            return;
        }
        self.dispatch_key_bindings(ctx);
    }

    fn frame_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.automation.clear();
        self.now = u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let previous_alarm = self.engine.playlist_alarm().map(str::to_owned);
        self.poll_program(&ctx);
        self.engine.tick(self.now);
        if self.engine.playlist_alarm() != previous_alarm.as_deref()
            && let Some(message) = self.engine.playlist_alarm().map(str::to_owned)
        {
            self.say(message, true);
        }
        self.poll_job(&ctx);
        self.poll_media();
        self.shortcuts(&ctx);
        self.refresh_textures(&ctx);
        let dialog_busy = self.add_open
            || self.search_open
            || self.properties.is_some()
            || self.settings
            || self.search_open
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
        // Reserve an actual input/mixer working area after the toolbars, even
        // when restoring an oversized bank from an older version.
        let content_height = ui.available_height();
        let monitor_max = (content_height - 240.0).max(260.0);
        let monitor_min = 300.0_f32.min(monitor_max);
        let monitor_height = (content_height * 0.55).clamp(monitor_min, monitor_max);
        let monitor_id = egui::Id::new("monitor.bank");
        if let Some(previous) = self.monitor_view_height
            && (previous - viewport_height).abs() > 1.0
            && previous > 0.0
            && let Some(mut state) = egui::containers::panel::PanelState::load(&ctx, monitor_id)
        {
            let height = (state.outer_rect.height() * viewport_height / previous)
                .clamp(monitor_min, monitor_max);
            state.outer_rect.max.y = state.outer_rect.min.y + height;
            ctx.data_mut(|d| d.insert_persisted(monitor_id, state));
        }
        self.monitor_view_height = Some(viewport_height);
        egui::Panel::top("monitor.bank")
            .resizable(true)
            .default_size(monitor_height)
            .min_size(monitor_min)
            .max_size(monitor_max)
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
        style.tab_bar.hline_color = Color32::from_rgb(52, 64, 78);
        style.tab.active.bg_fill = PROGRAM;
        style.tab.active.text_color = Color32::WHITE;
        style.tab.active.outline_color = Color32::from_rgb(40, 195, 90);
        style.tab.focused.bg_fill = PROGRAM;
        style.tab.focused.text_color = Color32::WHITE;
        style.tab.focused.outline_color = Color32::from_rgb(40, 195, 90);
        style.tab.inactive.bg_fill = Color32::from_rgb(30, 37, 47);
        style.tab.inactive.text_color = DIM;
        style.tab.hovered.bg_fill = Color32::from_rgb(48, 58, 72);
        style.tab.hovered.text_color = Color32::WHITE;
        style.separator.color_idle = Color32::from_rgb(42, 50, 62);
        style.separator.color_hovered = TEAL;
        DockArea::new(&mut dock)
            .draggable_tabs(self.audio_restore.is_none() && !self.ui_locked)
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
        if self.smoke.is_some() {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_millis(33));
        }
        if self.profile.is_some() {
            self.profile_step(&ctx);
        }
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
        storage.set_string("basic-mode.v1", self.basic_mode.to_string());
        if let Ok(json) = serde_json::to_string(self.audio_restore.as_ref().unwrap_or(&self.dock)) {
            storage.set_string("workspace.v2", json);
        }
    }
}
