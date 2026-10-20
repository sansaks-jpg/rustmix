//! Operator dialogs modeled after the reference workflows in `rustmix-reference`.
use super::*;
use egui::{Align, Color32, RichText, Sense, Stroke, Vec2};

const SIDEBAR: Color32 = Color32::from_rgb(14, 18, 24);
const HEADER_DARK: Color32 = Color32::from_rgb(20, 26, 34);
const FORM_BORDER: Color32 = Color32::from_rgb(190, 196, 203);

fn form_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    visuals.override_text_color = Some(FORM_TEXT);
    visuals.widgets.noninteractive.bg_fill = FORM_LIGHT;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, FORM_TEXT);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(252, 252, 253);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, FORM_TEXT);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, FORM_BORDER);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(229, 237, 245);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, FORM_TEXT);
    visuals.widgets.active.bg_fill = Color32::from_rgb(204, 224, 241);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, FORM_TEXT);
    visuals.selection.bg_fill = Color32::from_rgb(204, 224, 241);
    visuals.selection.stroke = Stroke::new(1.0, FORM_TEXT);
    visuals.extreme_bg_color = Color32::WHITE;
    visuals
}

// Keep compact/DPI-scaled workspaces on the existing footer-safe height policy.
// Large production workspaces give the form and source monitor room to grow.
fn source_dialog_body_height(viewport_height: f32) -> f32 {
    if viewport_height <= 700.0 {
        (viewport_height - 180.0).clamp(200.0, 520.0)
    } else {
        (viewport_height * 0.70 - 60.0)
            .max(520.0)
            .min(viewport_height - 180.0)
    }
}

pub(super) fn operator_window<'a>(
    ctx: &egui::Context,
    title: impl Into<egui::WidgetText>,
    id: &str,
    open: &'a mut bool,
    preferred: Vec2,
) -> egui::Window<'a> {
    let bounds = ctx.viewport_rect().shrink(12.0);
    let maximum = (bounds.size() - Vec2::new(12.0, 28.0)).max(Vec2::new(280.0, 220.0));
    let size = preferred.min(maximum);
    egui::Window::new(title)
        .id(egui::Id::new(id))
        .open(open)
        .collapsible(false)
        .resizable(true)
        .default_size(size)
        .default_pos(bounds.center() - size * 0.5)
        .min_size(Vec2::new(560.0, 300.0).min(maximum))
        .max_size(maximum)
        .constrain_to(bounds)
}

/// Applies the compact, flat form surface used inside operator dialogs (`Add Input`,
/// `Input Settings`, `Settings`, `Recording Setup`, `Streaming Settings`, `Overlay Settings`,
/// `PlayList`, `MultiCorder`).
fn light_form_frame<R>(
    ui: &mut egui::Ui,
    margin: i8,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Frame::new()
        .fill(FORM_LIGHT)
        .corner_radius(1.0)
        .stroke(Stroke::new(1.0, FORM_BORDER))
        .inner_margin(margin)
        .show(ui, |ui| {
            *ui.visuals_mut() = form_visuals();
            add_contents(ui)
        })
}

fn sidebar_row(
    ui: &mut egui::Ui,
    active: bool,
    label: &str,
    row_height: f32,
    icon_index: Option<usize>,
) -> egui::Response {
    ui.push_id(label, |ui| {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), row_height),
            egui::Sense::click(),
        );
        let bg = if active {
            PROGRAM
        } else if response.hovered() {
            Color32::from_rgb(38, 48, 60)
        } else {
            SIDEBAR
        };
        ui.painter().rect_filled(rect, 1.0, bg);
        let text_x = if let Some(idx) = icon_index {
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(rect.left() + 12.0, rect.center().y),
                Vec2::new(12.0, 12.0),
            );
            let stroke = Stroke::new(1.2, Color32::WHITE);
            match idx % 6 {
                0 => {
                    ui.painter()
                        .rect_stroke(icon_rect, 1.0, stroke, egui::StrokeKind::Inside);
                    ui.painter()
                        .hline(icon_rect.x_range(), icon_rect.top() + 3.0, stroke);
                }
                1 => {
                    ui.painter().circle_stroke(icon_rect.center(), 5.0, stroke);
                    ui.painter()
                        .circle_filled(icon_rect.center(), 1.5, Color32::WHITE);
                }
                2 => {
                    for y in [
                        icon_rect.top() + 2.0,
                        icon_rect.center().y,
                        icon_rect.bottom() - 2.0,
                    ] {
                        ui.painter().hline(icon_rect.x_range(), y, stroke);
                    }
                }
                3 => {
                    let body = egui::Rect::from_min_max(
                        icon_rect.min,
                        egui::pos2(icon_rect.right() - 3.0, icon_rect.bottom()),
                    );
                    ui.painter()
                        .rect_stroke(body, 1.0, stroke, egui::StrokeKind::Inside);
                    ui.painter().line_segment(
                        [
                            egui::pos2(body.right(), body.center().y),
                            egui::pos2(icon_rect.right(), icon_rect.top() + 2.0),
                        ],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(body.right(), body.center().y),
                            egui::pos2(icon_rect.right(), icon_rect.bottom() - 2.0),
                        ],
                        stroke,
                    );
                }
                4 => {
                    ui.painter()
                        .hline(icon_rect.x_range(), icon_rect.top() + 2.0, stroke);
                    ui.painter().vline(
                        icon_rect.center().x,
                        (icon_rect.top() + 2.0)..=icon_rect.bottom(),
                        stroke,
                    );
                }
                _ => {
                    for (bar_i, frac) in [0.45_f32, 0.9, 0.65].into_iter().enumerate() {
                        let x = icon_rect.left() + 1.0 + bar_i as f32 * 3.6;
                        let bar = egui::Rect::from_min_max(
                            egui::pos2(x, icon_rect.bottom() - icon_rect.height() * frac),
                            egui::pos2(x + 2.2, icon_rect.bottom()),
                        );
                        ui.painter().rect_filled(bar, 0.0, Color32::WHITE);
                    }
                }
            }
            rect.left() + 24.0
        } else {
            rect.left() + 8.0
        };
        ui.painter()
            .with_clip_rect(rect.shrink2(Vec2::new(2.0, 0.0)))
            .text(
                egui::pos2(text_x, rect.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );
        response.on_hover_text(label)
    })
    .inner
}

const ADD_CATEGORIES: [(&str, InputKind, &str); 21] = [
    (
        "Generators / PNG",
        InputKind::Grid,
        "Choose a procedural source or PNG image.",
    ),
    (
        "PNG Image",
        InputKind::Color,
        "Choose a PNG still image (up to 32 MiB / 4096px).",
    ),
    (
        "List",
        InputKind::Countdown,
        "Select the Video List or playlist items.",
    ),
    (
        "Moving shapes",
        InputKind::Geometry,
        "Procedural moving shapes; no camera device is opened.",
    ),
    (
        "Studio pattern",
        InputKind::Studio,
        "Procedural studio pattern; no network capture is opened.",
    ),
    (
        "SRT",
        InputKind::Grid,
        "Configure SRT Listener / Caller input stream.",
    ),
    (
        "Reference bars",
        InputKind::Bars,
        "Procedural reference bars.",
    ),
    (
        "Instant Replay",
        InputKind::Countdown,
        "Configure multi-camera Instant Replay.",
    ),
    (
        "Image Sequence / Stinger",
        InputKind::Graphics,
        "Select an Image Sequence or Stinger folder.",
    ),
    (
        "Photos",
        InputKind::Studio,
        "Select a Photo slideshow folder.",
    ),
    (
        "PowerPoint",
        InputKind::Graphics,
        "Select a presentation slide deck.",
    ),
    (
        "Colour",
        InputKind::Color,
        "Select a solid Colour or gradient bars generator.",
    ),
    (
        "Audio",
        InputKind::Bars,
        "Select and validate a local WAV audio file.",
    ),
    (
        "Audio Input",
        InputKind::Geometry,
        "Configure a microphone for explicitly started local monitoring.",
    ),
    (
        "Title / XAML",
        InputKind::Graphics,
        "Select a Title / Lower Third template.",
    ),
    (
        "GT",
        InputKind::Countdown,
        "Select a GT animated graphic or timer.",
    ),
    (
        "Virtual Set",
        InputKind::Studio,
        "Select a multi-layer Virtual Set studio.",
    ),
    (
        "Web Browser",
        InputKind::Grid,
        "Enter a Web Browser URL and resolution.",
    ),
    (
        "Video Call",
        InputKind::Geometry,
        "Connect a remote Video Call guest.",
    ),
    (
        "Zoom",
        InputKind::Studio,
        "Connect a Zoom meeting participant feed.",
    ),
    (
        VIDEO_TAB_LABEL,
        InputKind::Color,
        "Select a local MP4 video file (video only, Windows Media Foundation).",
    ),
];

/// Add-dialog category index for the local MP4 video-file source. Appended so
/// the observed reference categories keep their existing ordinals.
pub(crate) const VIDEO_TAB: usize = 20;
const VIDEO_TAB_LABEL: &str = "Video file";

const GENERATOR_SAMPLES: [(&str, InputKind); 7] = [
    ("Motion grid", InputKind::Grid),
    ("Moving shapes", InputKind::Geometry),
    ("Reference bars", InputKind::Bars),
    ("Lower third graphic", InputKind::Graphics),
    ("Studio pattern", InputKind::Studio),
    ("Colour gradient", InputKind::Color),
    ("Countdown 60 seconds", InputKind::Countdown),
];

impl RustMixApp {
    fn dialog_columns(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        height: f32,
        navigation: impl FnOnce(&mut Self, &mut egui::Ui),
        content: impl FnOnce(&mut Self, &mut egui::Ui),
    ) {
        let width = ui
            .available_width()
            .min((ui.ctx().viewport_rect().width() - 48.0).max(250.0));
        let (body, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        // A wider source library keeps long source names readable on production
        // workspaces; compact displays retain room for the independently scrolling form.
        let navigation_width = if width >= 900.0 {
            let ratio = if id == "dialog.add" { 0.184 } else { 0.142 };
            (width * ratio).max(164.0)
        } else {
            164.0
        };
        let navigation_rect =
            egui::Rect::from_min_size(body.min, Vec2::new(navigation_width, height));
        let page_rect = egui::Rect::from_min_max(
            egui::pos2(navigation_rect.right() + 6.0, body.top()),
            body.max,
        );
        let mut navigation_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((id, "navigation"))
                .max_rect(navigation_rect)
                .layout(egui::Layout::top_down(Align::Min)),
        );
        navigation_ui.set_clip_rect(navigation_rect.intersect(ui.clip_rect()));
        egui::Frame::new()
            .fill(SIDEBAR)
            .inner_margin(6.0)
            .show(&mut navigation_ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt((id, "navigation.scroll"))
                    .max_width(navigation_width - 12.0)
                    .max_height((height - 12.0).max(80.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| navigation(self, ui));
            });
        let mut page_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((id, "page"))
                .max_rect(page_rect)
                .layout(egui::Layout::top_down(Align::Min)),
        );
        page_ui.set_clip_rect(page_rect.intersect(ui.clip_rect()));
        if matches!(id, "dialog.add" | "dialog.properties") {
            page_ui.painter().rect_filled(page_rect, 0.0, FORM_LIGHT);
            *page_ui.visuals_mut() = form_visuals();
        }
        self.automation
            .insert(format!("{id}.navigation"), navigation_rect);
        self.automation.insert(format!("{id}.page"), page_rect);
        egui::ScrollArea::vertical()
            .id_salt((id, "page.scroll"))
            .max_width(page_rect.width())
            .max_height(height)
            .auto_shrink([false, false])
            .show(&mut page_ui, |ui| content(self, ui));
        self.automation.insert(format!("{id}.body"), body);
    }

    fn prepare_add_preview(&mut self, ctx: &egui::Context) {
        if let Some(fx) = &self.add_image
            && fx.media_path.as_deref() == Some(self.add_name.as_str())
        {
            if self.add_preview.is_none() {
                let pixels = panels::process_source_pixels(
                    &vec![[0, 0, 0, 255]; 320 * 180],
                    320,
                    180,
                    fx,
                    None,
                );
                let image = egui::ColorImage::from_rgba_unmultiplied(
                    [320, 180],
                    &pixels.into_iter().flatten().collect::<Vec<_>>(),
                );
                self.add_preview = Some((
                    InputKind::Color,
                    ctx.load_texture("dialog.add.png", image, egui::TextureOptions::LINEAR),
                ));
            }
            return;
        }
        if self
            .add_preview
            .as_ref()
            .is_some_and(|(kind, _)| *kind == self.add_kind)
        {
            return;
        }
        let input = InputState {
            id: InputId(0),
            name: "Generator sample".into(),
            kind: self.add_kind,
            color: [38, 156, 192],
            online: true,
            audio: AudioChannel {
                muted: true,
                ..Default::default()
            },
            config: InputConfig::default(),
            category: None,
        };
        match render::source(&input, 320, 180, 1500) {
            Ok(pixels) => {
                let bytes: Vec<u8> = pixels.into_iter().flatten().collect();
                let image = egui::ColorImage::from_rgba_unmultiplied([320, 180], &bytes);
                if let Some((cached_kind, texture)) = &mut self.add_preview {
                    *cached_kind = self.add_kind;
                    texture.set(image, egui::TextureOptions::LINEAR);
                } else {
                    self.add_preview = Some((
                        self.add_kind,
                        ctx.load_texture("dialog.add.preview", image, egui::TextureOptions::LINEAR),
                    ));
                }
            }
            Err(e) => {
                self.notice = e.to_string();
                self.warning = true;
            }
        }
    }

    pub(crate) fn add_input_dialog(&mut self, ctx: &egui::Context) {
        if !self.add_open {
            return;
        }
        if !matches!(self.add_tab, 12 | 13 | VIDEO_TAB) {
            self.prepare_add_preview(ctx);
        }
        let mut open = self.add_open;
        let mut close_requested = false;
        let height = source_dialog_body_height(ctx.viewport_rect().height());
        operator_window(
            ctx,
            "Input Select",
            "dialog.add_input",
            &mut open,
            Vec2::new((ctx.viewport_rect().width() * 0.707).max(780.0), height + 70.0),
        )
        .show(ctx, |ui| {
            self.dialog_columns(
                ui,
                "dialog.add",
                height,
                |app, ui| {
                    if app.add_loading { ui.disable(); }
                    ui.spacing_mut().item_spacing.y = 1.0;
                    // File video is a primary source choice. Keep stable category ordinals
                    // while placing it above the procedural library and future integrations.
                    let categories = ADD_CATEGORIES.into_iter().enumerate().skip(VIDEO_TAB)
                        .chain(ADD_CATEGORIES.into_iter().enumerate().take(VIDEO_TAB));
                    for (idx, (label, kind, _)) in categories {
                        let active = app.add_tab == idx && app.add_kind == kind;
                        let supported = matches!(idx, 0 | 1 | 3 | 4 | 6 | 11 | 12 | 13 | 14 | 15 | VIDEO_TAB);
                        let r = ui.add_enabled_ui(supported, |ui| sidebar_row(ui, active, label, 25.0, Some(idx))).inner
                            .on_disabled_hover_text("Unavailable: no media/device backend for this source type");
                        if idx == VIDEO_TAB {
                            app.automation.insert("dialog.add.video".into(), r.rect);
                        } else if matches!(idx, 0 | 1 | 3 | 4 | 6 | 11 | 14 | 15) {
                            app.automation.insert(if idx == 1 { "dialog.add.image".into() } else { format!("dialog.add.kind.{kind:?}") }, r.rect);
                        }
                        if matches!(idx, 12 | 13) {
                            app.automation.insert(if idx == 12 { "dialog.add.audio.wav".into() } else { "dialog.add.audio.microphone".into() }, r.rect);
                        }
                        if r.clicked() {
                            app.add_tab = idx;
                            app.add_kind = kind;
                        }
                    }
                },
                |app, ui| {
                    if app.add_loading { ui.disable(); }
                    if matches!(app.add_tab, 12 | 13) {
                        app.audio_add_page(ui, ctx);
                        return;
                    }
                    if app.add_tab == VIDEO_TAB {
                        app.video_add_page(ui, ctx);
                        return;
                    }
                    let (_, _, banner_text) = ADD_CATEGORIES.get(app.add_tab).copied().unwrap_or((
                        "Video",
                        InputKind::Grid,
                        "Select the Video file to open.",
                    ));
                    egui::Frame::new()
                        .fill(HEADER_DARK)
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            ui.set_min_width((ui.available_width() - 2.0).max(80.0));
                            ui.label(RichText::new(banner_text).color(Color32::WHITE).size(15.0));
                            ui.label(RichText::new("Synthetic sources / PNG / local video · capture and network backends unavailable").size(10.5).color(DIM));
                        });
                    light_form_frame(ui, 10, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [48.0, 22.0],
                                egui::Label::new(
                                    RichText::new("Source").color(FORM_TEXT).size(12.0),
                                ),
                            );
                            let text_w = (ui.available_width() - 94.0).max(120.0);
                            let name = ui.add_sized(
                                [text_w, 22.0],
                                egui::TextEdit::singleline(&mut app.add_name)
                                    .char_limit(4096)
                                    .hint_text("Generator name or PNG file path"),
                            );
                            app.automation.insert("dialog.add.name".into(), name.rect);
                            if name.changed() { app.add_image = None; app.add_preview = None; }
                            if ui
                                .add_sized([80.0, 22.0], egui::Button::new("Browse"))
                                .clicked() && app.smoke.is_none() && !app.add_loading
                            {
                                app.start_png_picker(media::MediaTarget::AddPreview { token: app.add_token });
                            }
                        });
                        ui.add_space(6.0);
                        ui.columns(2, |cols| {
                            if let [left_col, right_col, ..] = cols {
                                egui::Frame::new()
                                    .fill(Color32::WHITE)
                                    .corner_radius(4.0)
                                    .stroke(Stroke::new(1.0, FORM_BORDER))
                                    .inner_margin(6.0)
                                    .show(left_col, |ui| {
                                        ui.set_min_height(155.0);
                                        ui.label(
                                            RichText::new("Procedural generators")
                                                .size(11.0)
                                                .strong()
                                                .color(FORM_TEXT),
                                        );
                                        ui.separator();
                                        for (path, kind) in GENERATOR_SAMPLES {
                                            let selected =
                                                app.add_kind == kind && app.add_name == path;
                                            let r = ui.selectable_label(
                                                selected,
                                                RichText::new(path).size(11.0).color(FORM_TEXT),
                                            );
                                            if r.clicked() {
                                                app.add_kind = kind;
                                                app.add_name = path.into();
                                                app.add_tab = match kind { InputKind::Grid => 0, InputKind::Geometry => 3, InputKind::Studio => 4, InputKind::Bars => 6, InputKind::Color => 11, InputKind::Graphics => 14, InputKind::Countdown => 15 };
                                                app.add_image = None; app.add_preview = None;
                                            }
                                        }
                                    });
                                right_col.vertical(|ui| {
                                    egui::Frame::new()
                                        .fill(Color32::from_gray(14))
                                        .corner_radius(4.0)
                                        .stroke(Stroke::new(1.0, FORM_BORDER))
                                        .inner_margin(4.0)
                                        .show(ui, |ui| {
                                            if app.add_image.is_none() && (app.add_tab == 1 || Path::new(&app.add_name).extension().is_some()) {
                                                ui.allocate_space(egui::vec2(180.0, 95.0));
                                                ui.label(RichText::new("PNG is validated on import").color(Color32::WHITE));
                                            } else if let Some((_, texture)) = &app.add_preview {
                                                let w = ui.available_width().clamp(140.0, 260.0);
                                                let r = ui.add(
                                                    egui::Image::new((
                                                        texture.id(),
                                                        egui::vec2(w, w * 9.0 / 16.0),
                                                    ))
                                                    .corner_radius(3.0),
                                                );
                                                app.automation
                                                    .insert("dialog.add.preview".into(), r.rect);
                                            }
                                        });
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "{} · 320x180 preview",
                                            if app.add_tab == 1 { "PNG image".into() } else { app.add_kind.label().to_owned() }
                                        ))
                                        .size(11.0)
                                        .color(FORM_TEXT),
                                    );
                                });
                            }
                        });
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.add_enabled(false, egui::Checkbox::new(&mut app.add_alpha, "PNG alpha is composited over black"));
                            ui.add_space(8.0);
                            ui.add_enabled(false, egui::Checkbox::new(&mut app.add_interlaced, "Interlaced video"));
                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                if ui
                                    .add_sized([74.0, 22.0], egui::Button::new("Clear"))
                                    .clicked()
                                {
                                    app.add_name.clear();
                                    app.add_image = None;
                                    app.add_preview = None;
                                }
                            });
                        });
                    });
                },
            );
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Number").size(12.0).color(Color32::WHITE));
                ui.add_enabled_ui(!self.add_loading, |ui| {
                egui::ComboBox::from_id_salt("dialog.add.number")
                    .width(54.0)
                    .selected_text(self.add_number.to_string())
                    .show_ui(ui, |ui| {
                        for n in 1..=self.engine.project().inputs.len() + 1 {
                            ui.selectable_value(&mut self.add_number, n, n.to_string());
                        }
                    });
                });
                if !matches!(self.add_tab, 12 | 13 | VIDEO_TAB) && self.add_name.trim().is_empty() {
                    ui.colored_label(PREVIEW, "Enter a non-empty source label.");
                }
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add(
                        egui::Button::new("Cancel")
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.add.cancel".into(), cancel.rect);
                    if cancel.clicked() {
                        close_requested = true;
                    }
                    let confirm = ui.add_enabled(!self.add_loading && (matches!(self.add_tab, 12 | 13 | VIDEO_TAB) || !self.add_name.trim().is_empty()),
                        egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong())
                            .fill(PROGRAM)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.add.confirm".into(), confirm.rect);
                    if confirm.clicked() && !self.add_loading {
                        close_requested = if matches!(self.add_tab, 12 | 13) {
                            self.confirm_audio_input(ctx)
                        } else if self.add_tab == VIDEO_TAB {
                            self.confirm_video_input(ctx)
                        } else {
                            self.request_add_input()
                        };
                    }
                    if self.add_loading { ui.label("Loading PNG…"); }
                });
            });
        });
        if close_requested || !open {
            self.add_open = false;
            self.add_loading = false;
            self.add_token = self.add_token.wrapping_add(1);
            self.cancel_audio_add();
            self.cancel_video_add();
        }
    }

    pub(crate) fn input_properties_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut draft) = self.properties.clone() else {
            return;
        };
        let mut open = true;
        let mut apply_clicked = false;
        let mut cancel_clicked = false;
        let id = draft.id;
        let title_name = self
            .engine
            .project()
            .input(id)
            .map(|input| input.name.as_str())
            .unwrap_or(draft.name.as_str())
            .to_owned();
        let height = source_dialog_body_height(ctx.viewport_rect().height());
        operator_window(
            ctx,
            format!("Input: {} {title_name}", self.engine.project().input_number(id)),
            "dialog.input_properties",
            &mut open,
            Vec2::new((ctx.viewport_rect().width() * 0.731).max(790.0), height + 70.0),
        )
        .show(ctx, |ui| {
            self.dialog_columns(
                ui,
                "dialog.properties",
                height,
                |app, ui| {
                    if app.properties_loading { ui.disable(); }
                    ui.spacing_mut().item_spacing.y = 1.0;
                    for (page, label, key) in [
                        (InputPage::General, "General", "General"),
                        (InputPage::ColourAdjust, "Colour Adjust", "ColourAdjust"),
                        (InputPage::ColourKey, "Colour Key", "ColourKey"),
                        (
                            InputPage::ColourCorrection,
                            "Colour Correction",
                            "ColourCorrection",
                        ),
                        (InputPage::Effects, "Effects", "Effects"),
                        (InputPage::Position, "Position", "Position"),
                        (InputPage::Layers, "Layers", "Layers"),
                        (InputPage::Triggers, "Triggers", "Triggers"),
                        (InputPage::TallyLights, "Tally Lights", "TallyLights"),
                        (InputPage::Ptz, "PTZ", "Ptz"),
                        (InputPage::Advanced, "Advanced", "Advanced"),
                        (InputPage::Audio, "Audio / buses", "Audio"),
                        (InputPage::Status, "Source status", "Status"),
                    ]
                    .into_iter()
                    {
                        let supported = !matches!(page, InputPage::Triggers | InputPage::TallyLights | InputPage::Ptz | InputPage::Advanced);
                        let row_height = if ctx.viewport_rect().height() < 700.0 { 18.5 } else { 25.0 };
                        let r = ui.add_enabled_ui(supported, |ui| sidebar_row(ui, app.input_page == page, label, row_height, None)).inner
                            .on_disabled_hover_text("This source control is unavailable");
                        app.automation
                            .insert(format!("dialog.properties.page.{key}"), r.rect);
                        if r.clicked() {
                            app.input_page = page;
                        }
                    }
                    ui.add_space(4.0);
                    if ui
                        .add_sized(
                            [152.0, 22.0],
                            egui::Button::new(RichText::new("Copy From...").size(11.0))
                                .fill(CHIP_BG),
                        )
                        .clicked()
                    {
                        let pvw_id = app.engine.project().preview;
                        if let Some(src_fx) = app.input_fx.get(&pvw_id).cloned() {
                            app.properties_fx = Some(src_fx);
                            app.load_draft_controls();
                            app.last_demo_frame = None;
                        }
                    }
                },
                |app, ui| {
                    if app.properties_loading { ui.disable(); }
                    egui::Frame::new()
                        .fill(HEADER_DARK)
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.set_min_width((ui.available_width() - 2.0).max(80.0));
                            ui.horizontal(|ui| {
                                let path_lbl = app
                                    .properties_fx
                                    .as_ref()
                                    .and_then(|f| f.media_path.as_deref())
                                    .map(str::to_owned)
                                    .unwrap_or_else(|| format!("{} · synthetic {}", draft.name, draft.kind.label()));
                                let label_w = (ui.available_width() - 84.0).max(120.0);
                                ui.add_sized([label_w, 20.0], egui::Label::new(RichText::new(&path_lbl).color(Color32::WHITE).size(12.0)).truncate())
                                    .on_hover_text(&path_lbl);
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    let change = ui.add_enabled(!app.properties_loading, egui::Button::new(RichText::new("Change").color(Color32::WHITE)).fill(CHIP_BG));
                                    app.automation.insert("dialog.properties.change".into(), change.rect);
                                    if change.clicked() && app.smoke.is_none() {
                                        app.start_png_picker(media::MediaTarget::Properties { id, token: app.properties_token });
                                    }
                                });
                            });
                        });
                    if app.properties_loading { ui.colored_label(PREVIEW, "Loading PNG… current source stays active"); }
                    if let Some(fx) = &app.properties_fx {
                        if let Some(error) = &fx.image_error { ui.colored_label(RED, error); }
                        if let Some([w, h]) = fx.image_size {
                            if fx.video_source.is_some() {
                                ui.label(format!("Local MP4 video: {w} × {h} · owner decoder / CPU preview / Program"));
                            } else {
                                ui.label(format!("PNG source: {w} × {h} · CPU preview / Program"));
                            }
                        }
                    }
                    ui.add_space(4.0);
                    light_form_frame(ui, 8, |ui| match app.input_page {
                        InputPage::General => {
                            ui.columns(2, |cols| {
                                if let [left_col, right_col, ..] = cols {
                                    egui::Grid::new("input_props_left")
                                        .num_columns(2)
                                        .spacing([8.0, 4.0])
                                        .show(left_col, |ui| {
                                            ui.label(
                                                RichText::new("Name").size(11.5).color(FORM_TEXT),
                                            );
                                            let name = ui.add_sized(
                                                [175.0, 20.0],
                                                egui::TextEdit::singleline(&mut draft.name)
                                                    .char_limit(64),
                                            );
                                            app.automation
                                                .insert("dialog.properties.name".into(), name.rect);
                                            ui.end_row();

                                            ui.label(
                                                RichText::new("Aspect Ratio")
                                                    .size(11.5)
                                                    .color(FORM_TEXT),
                                            );
                                            ui.add_enabled(false, egui::Button::new("Widescreen")
                                                .min_size(Vec2::new(175.0, 20.0)))
                                                .on_disabled_hover_text("The current compositor uses a fixed 16:9 working raster. Use Image fit below for Contain, Cover or Stretch.");
                                            ui.end_row();

                                            ui.label(RichText::new("Category").size(11.5).color(FORM_TEXT));
                                            let selected_category = draft.category
                                                .and_then(|id| app.engine.project().preferences.categories.iter().find(|category| category.id == id))
                                                .map_or("None", |category| category.name.as_str());
                                            let category = egui::ComboBox::from_id_salt("prop.category")
                                                .width(175.0)
                                                .selected_text(selected_category)
                                                .show_ui(ui, |ui| {
                                                    let none = ui.selectable_value(&mut draft.category, None, "None");
                                                    app.automation.insert("dialog.properties.category.none".into(), none.rect);
                                                    for category in &app.engine.project().preferences.categories {
                                                        let response = ui.selectable_value(&mut draft.category, Some(category.id), &category.name);
                                                        app.automation.insert(format!("dialog.properties.category.{}", category.id), response.rect);
                                                    }
                                                });
                                            app.automation.insert("dialog.properties.category".into(), category.response.rect);
                                            ui.end_row();

                                            ui.label(
                                                RichText::new("Generator")
                                                    .size(11.5)
                                                    .color(FORM_TEXT),
                                            );
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 3.0;
                                                for kind in [
                                                    InputKind::Grid,
                                                    InputKind::Geometry,
                                                    InputKind::Bars,
                                                    InputKind::Graphics,
                                                    InputKind::Studio,
                                                    InputKind::Color,
                                                    InputKind::Countdown,
                                                ] {
                                                    let swatch_c = kind_color(kind);
                                                    let (rect, r) = ui.allocate_exact_size(
                                                        Vec2::new(22.0, 16.0),
                                                        Sense::click(),
                                                    );
                                                    ui.painter().rect_filled(rect, 2.0, swatch_c);
                                                    ui.painter().rect_stroke(
                                                        rect,
                                                        2.0,
                                                        Stroke::new(
                                                            if draft.kind == kind {
                                                                2.0
                                                            } else {
                                                                1.0
                                                            },
                                                            if draft.kind == kind {
                                                                Color32::WHITE
                                                            } else {
                                                                Color32::from_gray(90)
                                                            },
                                                        ),
                                                        egui::StrokeKind::Inside,
                                                    );
                                                    app.automation.insert(
                                                        format!("dialog.properties.kind.{kind:?}"),
                                                        rect,
                                                    );
                                                    if r.clicked() {
                                                        draft.kind = kind;
                                                        if let Some(fx) = app.properties_fx.as_mut() {
                                                            fx.media_path = None;
                                                            fx.custom_rgba = None;
                                                            fx.image_size = None;
                                                            fx.image_error = None;
                                                            fx.image_pending = false;
                                                        }
                                                        draft.color = [
                                                            swatch_c.r(),
                                                            swatch_c.g(),
                                                            swatch_c.b(),
                                                        ];
                                                    }
                                                }
                                            });
                                            ui.end_row();
                                        });
                                    left_col.add_space(2.0);
                                    left_col.spacing_mut().item_spacing.y = 1.5;
                                    left_col.add_enabled(false, egui::Checkbox::new(&mut app.prop_deinterlace, "Deinterlace Blend"));
                                    if left_col
                                        .checkbox(&mut app.prop_sharpen, "Sharpen")
                                        .changed()
                                    {
                                        app.sync_fx_from_prop(id);
                                    }
                                    if left_col.checkbox(&mut app.prop_mirror, "Mirror").changed() {
                                        app.sync_fx_from_prop(id);
                                    }
                                    left_col.add_enabled_ui(false, |ui| {
                                        ui.checkbox(&mut app.prop_flatten, "Flatten Layers");
                                        ui.checkbox(&mut app.prop_auto_mix, "Automatically mix audio");
                                        ui.checkbox(&mut app.prop_auto_play, "Automatically Play with Transition");
                                        ui.checkbox(&mut app.prop_auto_restart, "Automatically Restart with Transition");
                                        ui.checkbox(&mut app.prop_auto_pause, "Automatically Pause after Transition");
                                    }).response.on_hover_text("Automatic input triggers are unavailable");
                                    left_col.horizontal(|ui| {
                                        ui.label("Generator colour");
                                        let colour = ui.color_edit_button_srgb(&mut draft.color);
                                        app.automation.insert("dialog.properties.colour".into(), colour.rect);
                                    });
                                    if let Some(fx) = app.properties_fx.as_mut() {
                                        egui::ComboBox::from_id_salt("prop.image_fit")
                                            .selected_text(format!("Image fit: {:?}", fx.image_fit))
                                            .show_ui(left_col, |ui| {
                                                for fit in [ImageFit::Contain, ImageFit::Cover, ImageFit::Stretch] {
                                                    ui.selectable_value(&mut fx.config.image_fit, fit, format!("{fit:?}"));
                                                }
                                            });
                                    }
                                    egui::Grid::new("input_props_right")
                                        .num_columns(2)
                                        .spacing([8.0, 4.0])
                                        .show(right_col, |ui| {
                                            ui.label(
                                                RichText::new("Mouse Click Action")
                                                    .size(11.5)
                                                    .color(FORM_TEXT),
                                            );
                                            ui.label("Preview (fixed)");
                                            ui.end_row();

                                            ui.label(
                                                RichText::new("GO Action")
                                                    .size(11.5)
                                                    .color(FORM_TEXT),
                                            );
                                            ui.label("Preview (fixed)");
                                            ui.end_row();

                                            for (k, v) in [
                                                ("Source Aspect Ratio", "Widescreen"),
                                                ("Preview resolution", "384x216 (demo)"),
                                                ("Program resolution", "640x360 (demo)"),
                                                ("Deinterlacing", "None"),
                                            ] {
                                                ui.label(
                                                    RichText::new(k).size(11.5).color(FORM_TEXT),
                                                );
                                                ui.label(
                                                    RichText::new(v).size(11.5).color(FORM_TEXT),
                                                );
                                                ui.end_row();
                                            }
                                        });
                                    right_col.add_space(6.0);
                                    let audio_btn = right_col.add_sized(
                                        [240.0, 24.0],
                                        egui::Button::new("Audio Settings"),
                                    );
                                    app.automation.insert(
                                        "dialog.properties.audioSettings".into(),
                                        audio_btn.rect,
                                    );
                                    if audio_btn.clicked() {
                                        app.input_page = InputPage::Audio;
                                    }
                                    right_col.add_space(4.0);
                                    if right_col
                                        .add_enabled(
                                            app.engine.enabled(&Command::DuplicateInput(id)).is_ok(),
                                            egui::Button::new("Duplicate Input").min_size(Vec2::new(240.0, 24.0)),
                                        )
                                        .clicked() && app.apply_properties(draft.clone())
                                    {
                                        app.actions.push(Action::Engine(Command::DuplicateInput(id)));
                                        cancel_clicked = true;
                                    }
                                }
                            });
                        }
                        InputPage::ColourKey => {
                            ui.label(
                                RichText::new("CPU Chroma Key · experimental")
                                    .size(13.0)
                                    .strong()
                                    .color(FORM_TEXT),
                            );
                            let fx = app.properties_fx.get_or_insert_default();
                            if ui
                                .checkbox(
                                    &mut fx.chroma_key,
                                    "Enable Chroma Key (Key out green / high-G pixels in real time)",
                                )
                                .changed()
                            {
                                app.last_demo_frame = None;
                            }
                            if ui
                                .add(
                                    egui::Slider::new(&mut fx.chroma_tolerance, 0.05..=0.85)
                                        .text("Chroma Key Tolerance"),
                                )
                                .changed()
                            {
                                app.last_demo_frame = None;
                            }
                        }
                        InputPage::Layers => {
                            ui.label(
                                RichText::new("Multi-View / Picture-in-Picture Layers")
                                    .size(13.0)
                                    .strong()
                                    .color(FORM_TEXT),
                            );
                            let inputs_list: Vec<(InputId, String)> = app
                                .engine
                                .project()
                                .inputs
                                .iter()
                                .filter(|i| i.id != id)
                                .map(|i| (i.id, format!("{}: {}", app.engine.project().input_number(i.id), i.name)))
                                .collect();
                            let fx = app.properties_fx.get_or_insert_default();
                            let selected_lbl = fx
                                .pip_source
                                .and_then(|pid| {
                                    inputs_list
                                        .iter()
                                        .find(|(i, _)| *i == pid)
                                        .map(|(_, l)| l.as_str())
                                })
                                .unwrap_or("None");
                            ui.horizontal(|ui| {
                                ui.label("Layer 1 (PIP Overlay):");
                                egui::ComboBox::from_id_salt("prop.layer1.pip")
                                    .width(220.0)
                                    .selected_text(selected_lbl)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_value(&mut fx.pip_source, None, "None")
                                            .clicked()
                                        {
                                            app.last_demo_frame = None;
                                        }
                                        for (other_id, lbl) in &inputs_list {
                                            if ui
                                                .selectable_value(
                                                    &mut fx.pip_source,
                                                    Some(*other_id),
                                                    lbl,
                                                )
                                                .clicked()
                                            {
                                                app.last_demo_frame = None;
                                            }
                                        }
                                    });
                            });
                        }
                        InputPage::Audio => {
                            ui.label(
                                RichText::new("Audio Routing & Channel Strip")
                                    .size(13.0)
                                    .strong()
                                    .color(FORM_TEXT),
                            );
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut draft.audio.muted, "Mute channel");
                                ui.checkbox(&mut draft.audio.solo, "Solo channel");
                            });
                            ui.add(
                                egui::Slider::new(&mut draft.audio.gain_db, -60.0..=12.0)
                                    .text("Gain (dB)"),
                            );
                            ui.horizontal(|ui| {
                                for (route, label) in draft
                                    .audio
                                    .sends
                                    .iter_mut()
                                    .zip(["MASTER", "BUS A", "BUS B"])
                                {
                                    ui.checkbox(route, label);
                                }
                            });
                        }
                        InputPage::Status => {
                            ui.label(
                                RichText::new("Source Health & Tally")
                                    .size(13.0)
                                    .strong()
                                    .color(FORM_TEXT),
                            );
                            ui.checkbox(
                                &mut draft.online,
                                "Source online (off = black)",
                            );
                            if let Some(VideoSource::File { path }) = &draft.config.video_source {
                                ui.label("Source type: MP4 video, decoded on its own owner worker. MP4 audio is not decoded.");
                                ui.label(format!("Path: {path}"));
                                if let Some(fx) = app.input_fx.get(&id) {
                                    match (&fx.image_error, fx.custom_rgba.is_some()) {
                                        (Some(error), _) => {
                                            ui.colored_label(RED, format!("Decode: {error}"));
                                        }
                                        (None, true) => {
                                            ui.colored_label(
                                                TEAL,
                                                "Decode: running; the newest accepted frame is presented",
                                            );
                                        }
                                        (None, false) if fx.image_pending => {
                                            ui.label("Decode: opening the owner worker…");
                                        }
                                        (None, false) => {
                                            ui.label("Decode: waiting for the first frame");
                                        }
                                    }
                                }
                                ui.label("Relinking a different file requires removing and re-adding this input in this build.");
                            }
                            let p = app.engine.project();
                            let tally = if p.program == id {
                                "ON AIR — PROGRAM"
                            } else if p.preview == id {
                                "CUED — PREVIEW"
                            } else {
                                "STANDBY"
                            };
                            ui.label(format!("Current Tally State: {tally}"));
                        }
                        _ => {
                            ui.label(
                                RichText::new(format!("Input {:?} Controls", app.input_page))
                                    .size(13.0)
                                    .strong()
                                    .color(FORM_TEXT),
                            );
                            let mut changed = false;
                            changed |= ui
                                .add(egui::Slider::new(&mut app.prop_zoom, 0.25..=4.0).text("Zoom"))
                                .changed();
                            changed |= ui
                                .add(
                                    egui::Slider::new(&mut app.prop_pan_x, -1.0..=1.0)
                                        .text("Pan X"),
                                )
                                .changed();
                            changed |= ui
                                .add(
                                    egui::Slider::new(&mut app.prop_pan_y, -1.0..=1.0)
                                        .text("Pan Y"),
                                )
                                .changed();
                            changed |= ui
                                .add(
                                    egui::Slider::new(&mut app.prop_lift, -1.0..=1.0)
                                        .text("Lift / Y"),
                                )
                                .changed();
                            changed |= ui
                                .add(egui::Slider::new(&mut app.prop_gain, 0.0..=2.0).text("Gain"))
                                .changed();
                            changed |= ui
                                .add(
                                    egui::Slider::new(&mut app.prop_saturation, 0.0..=2.0)
                                        .text("Saturation"),
                                )
                                .changed();
                            if ui.small_button("Reset Transform & Color").clicked() {
                                app.prop_zoom = 1.0;
                                app.prop_pan_x = 0.0;
                                app.prop_pan_y = 0.0;
                                app.prop_lift = 0.0;
                                app.prop_gain = 1.0;
                                app.prop_saturation = 1.0;
                                changed = true;
                            }
                            if changed {
                                app.sync_fx_from_prop(id);
                            }
                        }
                    });
                    ui.add_space(4.0);
                    // Bottom black preview section with live GPU texture and zoom slider.
                    egui::Frame::new()
                        .fill(Color32::from_rgb(8, 10, 14))
                        .corner_radius(4.0)
                        .inner_margin(6.0)
                        .show(ui, |ui| {
                            ui.set_min_width((ui.available_width() - 2.0).max(80.0));
                            ui.horizontal(|ui| {
                                let avail_w = (ui.available_width() - 42.0).max(160.0);
                                let preview_h = if ctx.viewport_rect().height() >= 850.0 {
                                    (height * 0.40).clamp(190.0, 320.0)
                                } else { 132.0_f32 };
                                let preview_w = (preview_h * 16.0 / 9.0).min(avail_w);
                                let pad_left = ((avail_w - preview_w) * 0.5).max(0.0);
                                ui.add_space(pad_left);
                                if let Some(texture) = app.draft_texture(ctx, &draft) {
                                    let r = ui.add(egui::Image::new((
                                        texture,
                                        egui::vec2(preview_w, preview_h),
                                    )));
                                    app.automation
                                        .insert("dialog.properties.preview".into(), r.rect);
                                }
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    ui.vertical_centered(|ui| {
                                        if ui
                                            .add_sized(
                                                [20.0, 108.0],
                                                egui::Slider::new(&mut app.prop_zoom, 0.25..=2.0)
                                                    .vertical()
                                                    .show_value(false),
                                            )
                                            .changed()
                                        {
                                            app.sync_fx_from_prop(id);
                                        }
                                        ui.label(
                                            RichText::new(format!("{:.2}x", app.prop_zoom))
                                                .size(10.5)
                                                .color(Color32::WHITE),
                                        );
                                    });
                                });
                            });
                        });
                },
            );
            ui.separator();
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        self.engine.enabled(&Command::RemoveInput(id)).is_ok(),
                        egui::Button::new(RichText::new("Remove…").color(RED))
                            .min_size(Vec2::new(88.0, 29.0)),
                    )
                    .clicked()
                {
                    self.confirm_remove_input = Some(id);
                }
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add(
                        egui::Button::new("Cancel")
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.properties.cancel".into(), cancel.rect);
                    if cancel.clicked() {
                        cancel_clicked = true;
                    }
                    let apply = ui.add_enabled(!self.properties_loading,
                        egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong())
                            .fill(PROGRAM)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.properties.apply".into(), apply.rect);
                    if apply.clicked() {
                        apply_clicked = true;
                    }
                });
            });
        });
        if apply_clicked {
            self.apply_properties(draft);
        } else if cancel_clicked || !open {
            self.close_properties();
        } else {
            self.properties = Some(draft);
        }
    }

    pub(crate) fn settings_dialog(&mut self, ctx: &egui::Context) {
        if !self.settings {
            return;
        }
        if self.settings_draft.is_none() {
            let p = self.engine.project();
            self.settings_draft = Some(p.preferences.clone());
            self.settings_name = p.name.clone();
            self.settings_layout = None;
        }
        let Some(mut draft) = self.settings_draft.clone() else {
            return;
        };
        let mut open = self.settings;
        let mut apply_clicked = false;
        let mut cancel_clicked = false;
        let mut reset_default = false;
        let height = (ctx.viewport_rect().height() - 180.0).clamp(200.0, 520.0);
        operator_window(
            ctx,
            "Settings",
            "dialog.settings",
            &mut open,
            Vec2::new(760.0, 550.0),
        )
        .show(ctx, |ui| {
            self.dialog_columns(
                ui,
                "dialog.settings",
                height,
                |app, ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    for (page, label, key) in [
                        (SettingsPage::Display, "Display", "Display"),
                        (SettingsPage::Outputs, "Outputs / NDI / OMT", "Outputs"),
                        (SettingsPage::Options, "Options", "Options"),
                        (SettingsPage::Appearance, "Performance", "Appearance"),
                        (SettingsPage::Categories, "Input Categories", "Categories"),
                        (SettingsPage::Decoders, "Decoders", "Decoders"),
                        (SettingsPage::Recording, "Recording", "Recording"),
                        (
                            SettingsPage::ExternalOutput,
                            "External Output",
                            "ExternalOutput",
                        ),
                        (SettingsPage::Audio, "Audio", "Audio"),
                        (SettingsPage::AudioOutputs, "Audio Outputs", "AudioOutputs"),
                        (SettingsPage::Workspace, "Web Controller", "Workspace"),
                        (SettingsPage::TallyLights, "Tally Lights", "TallyLights"),
                        (SettingsPage::Shortcuts, "Shortcuts", "Shortcuts"),
                        (SettingsPage::Activators, "Activators", "Activators"),
                        (SettingsPage::Scripting, "Scripting", "Scripting"),
                        (SettingsPage::Alerts, "Alerts", "Alerts"),
                        (SettingsPage::About, "About", "About"),
                    ] {
                        let r = sidebar_row(ui, app.settings_page == page, label, 17.5, None);
                        app.automation
                            .insert(format!("dialog.settings.page.{key}"), r.rect);
                        if r.clicked() {
                            app.settings_page = page;
                        }
                    }
                    ui.add_space(4.0);
                    for action_lbl in ["Import", "Export", "Default"] {
                        if ui
                            .add_enabled(action_lbl == "Default", egui::Button::new(RichText::new(action_lbl).size(11.0)).fill(CHIP_BG).min_size(Vec2::new(152.0,21.0)))
                            .clicked()
                            && action_lbl == "Default"
                        {
                            reset_default = true;
                        }
                    }
                },
                |app, ui| {
                    light_form_frame(ui, 12, |ui| match app.settings_page {
                        SettingsPage::Display => {
                            egui::Grid::new("settings_display_grid")
                                .num_columns(2)
                                .spacing([18.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Theme").size(11.5).color(FORM_TEXT));
                                    app.appearance_controls(ui, &mut draft);

                                    ui.label(
                                        RichText::new("Master Frame Rate")
                                            .size(11.5)
                                            .color(FORM_TEXT),
                                    );
                                    egui::ComboBox::from_id_salt("settings.rate")
                                        .width(210.0)
                                        .selected_text(format!("{}p", draft.rate.label()))
                                        .show_ui(ui, |ui| {
                                            for (num, den, lbl) in [
                                                (24, 1, "24p"),
                                                (25, 1, "25p"),
                                                (30000, 1001, "NTSC 29.97p"),
                                                (30, 1, "NTSC 30p"),
                                                (50, 1, "PAL 50p"),
                                                (60000, 1001, "NTSC 59.94p"),
                                                (60, 1, "60p"),
                                            ] {
                                                let rate = Rate {
                                                    numerator: num,
                                                    denominator: den,
                                                };
                                                let r =
                                                    ui.selectable_value(&mut draft.rate, rate, lbl);
                                                app.automation.insert(
                                                    format!(
                                                        "dialog.settings.rate.{}",
                                                        rate.label()
                                                    ),
                                                    r.rect,
                                                );
                                            }
                                        });
                                    ui.end_row();

                                    ui.label("Actual Program raster"); ui.label("640×360 CPU reference"); ui.end_row();
                                    ui.label("Independent fullscreen output"); ui.label("Unavailable: no output router"); ui.end_row();
                                    ui.label(
                                        RichText::new("Production Name")
                                            .size(11.5)
                                            .color(FORM_TEXT),
                                    );
                                    let title = ui.add_sized(
                                        [210.0, 20.0],
                                        egui::TextEdit::singleline(&mut app.settings_name)
                                            .char_limit(64),
                                    );
                                    app.automation
                                        .insert("dialog.settings.projectName".into(), title.rect);
                                    ui.end_row();
                                });
                        }
                        SettingsPage::Appearance => {
                            ui.label(
                                RichText::new("Performance & Appearance")
                                    .size(13.0)
                                    .strong()
                                    .color(FORM_TEXT),
                            );
                            let scale = ui.add(
                                egui::Slider::new(&mut draft.ui_scale, 0.9..=1.35).text("UI scale"),
                            );
                            app.automation
                                .insert("dialog.settings.uiScale".into(), scale.rect);
                            let contrast = ui.checkbox(
                                &mut draft.high_contrast,
                                "High-contrast broadcast theme",
                            );
                            app.automation
                                .insert("dialog.settings.highContrast".into(), contrast.rect);
                            let shortcuts_chk = ui.checkbox(
                                &mut draft.shortcuts,
                                "Enable single-key production shortcuts",
                            );
                            app.automation
                                .insert("dialog.settings.shortcuts".into(), shortcuts_chk.rect);
                        }
                        SettingsPage::Workspace => {
                            ui.label(
                                RichText::new("Web Controller & Workspace Layout")
                                    .size(13.0)
                                    .strong()
                                    .color(FORM_TEXT),
                            );
                            let current_audio = app
                                .settings_layout
                                .unwrap_or(app.engine.project().workspace.name == "Audio focus");
                            if ui
                                .radio(!current_audio, "Production (inputs wide)")
                                .clicked()
                            {
                                app.settings_layout = Some(false);
                            }
                            if ui
                                .radio(current_audio, "Audio focus (wider mixer)")
                                .clicked()
                            {
                                app.settings_layout = Some(true);
                            }
                            ui.add_space(6.0);
                            ui.label("Web Controller unavailable: no HTTP listener is running.");
                        }
                        SettingsPage::Shortcuts => app.shortcut_settings(ui, &mut draft),
                        SettingsPage::Categories => app.category_settings(ui, &mut draft),
                        SettingsPage::ExternalOutput => {
                            ui.label(
                                "External outputs are routed and configured in the External Output \
                                 dialog (Outputs area → gear). These are the current stored values.",
                            );
                            ui.add_space(4.0);
                            for output in app.engine.project().outputs.iter() {
                                let source = match output.source {
                                    OutputSource::None => "None".to_string(),
                                    OutputSource::Program => "Program".to_string(),
                                    OutputSource::Preview => "Preview".to_string(),
                                    OutputSource::Input(id) => {
                                        app.engine.project().input(id).map_or_else(
                                            || "Missing input".to_string(),
                                            |i| i.name.clone(),
                                        )
                                    }
                                };
                                let bus = match output.config.audio_bus {
                                    None | Some(0) => "Master".to_string(),
                                    Some(1) => "A".to_string(),
                                    Some(2) => "B".to_string(),
                                    Some(other) => format!("Bus {other}"),
                                };
                                ui.label(format!(
                                    "External{} · {} · {}×{} · {} fps · bus {bus} · delay {} ms",
                                    if output.number == 1 {
                                        String::new()
                                    } else {
                                        format!(" {}", output.number)
                                    },
                                    source,
                                    output.config.width,
                                    output.config.height,
                                    output.config.frame_rate.label(),
                                    output.config.audio_delay_ms,
                                ));
                            }
                            ui.add_space(4.0);
                            ui.label(
                                "Device, NDI, OMT and SRT transport backends are absent; the \
                                 External button stays unavailable.",
                            );
                        }
                        _ => {
                            ui.label(
                                RichText::new(format!("{:?} Configuration", app.settings_page))
                                    .size(13.0)
                                    .strong()
                                    .color(FORM_TEXT),
                            );
                            ui.label("Unavailable in this build. This page does not configure a backend.");
                            ui.checkbox(
                                &mut draft.shortcuts,
                                "Enable single-key production shortcuts",
                            );
                        }
                    });
                },
            );
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Changes apply only with OK.");
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add(
                        egui::Button::new("Cancel")
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.settings.cancel".into(), cancel.rect);
                    if cancel.clicked() {
                        cancel_clicked = true;
                    }
                    let apply = ui.add(
                        egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong())
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.settings.apply".into(), apply.rect);
                    if apply.clicked() {
                        apply_clicked = true;
                    }
                });
            });
        });
        if reset_default {
            draft = ProjectState::demo().preferences;
        }
        if apply_clicked {
            let mut candidate = self.engine.project().clone();
            candidate.preferences = draft.clone();
            candidate.name = self.settings_name.trim().to_owned();
            if let Err(error) = candidate.validate() {
                self.say(error.to_string(), true);
                self.settings_draft = Some(draft);
                return;
            }
            if let Some(audio_focus) = self.settings_layout {
                self.actions.push(Action::Workspace(audio_focus));
            }
            self.actions.push(Action::Engine(Command::ApplySettings {
                name: self.settings_name.trim().to_owned(),
                preferences: draft,
            }));
            self.settings = false;
            self.settings_draft = None;
            self.settings_layout = None;
        } else if cancel_clicked || !open {
            self.settings = false;
            self.settings_draft = None;
            self.settings_layout = None;
        } else {
            self.settings_draft = Some(draft);
        }
    }

    pub(crate) fn overlay_settings_dialog(&mut self, ctx: &egui::Context) {
        let Some((mut slot, mut draft)) = self.overlay_settings.clone() else {
            return;
        };
        let mut open = true;
        let mut apply_clicked = false;
        let mut cancel_clicked = false;
        let height = (ctx.viewport_rect().height() - 180.0).clamp(200.0, 520.0);
        operator_window(
            ctx,
            "Overlay Settings",
            "dialog.overlay_settings",
            &mut open,
            Vec2::new(760.0, 560.0),
        )
        .show(ctx, |ui| {
            let width = ui
                .available_width()
                .min((ui.ctx().viewport_rect().width() - 48.0).max(250.0));
            let (body, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
            let mut body_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("dialog.overlay.single_body")
                    .max_rect(body)
                    .layout(egui::Layout::top_down(Align::Min)),
            );
            body_ui.set_clip_rect(body.intersect(ui.clip_rect()));
            egui::ScrollArea::vertical()
                .id_salt("dialog.overlay.scroll")
                .max_width(body.width())
                .max_height(height)
                .auto_shrink([false, false])
                .show(&mut body_ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Title panel");
                        for i in 0..8 {
                            let row=ui.selectable_label(slot==i,format!("{}",i+1));
                            self.automation.insert(format!("dialog.overlay.slot.{}",i+1),row.rect);
                            if row.clicked() && let Some(committed)=self.engine.project().overlays.standard(i).cloned(){slot=i;draft=committed;}
                        }
                    });
                    let enabled=ui.checkbox(&mut draft.enabled,"Show on Program");
                    self.automation.insert("dialog.overlay.enabled".into(),enabled.rect);
                    ui.horizontal(|ui|{
                        for position in [Position::LowerLeft,Position::LowerRight,Position::TopLeft]{
                            let r=ui.selectable_value(&mut draft.position,position,format!("{position:?}"));
                            self.automation.insert(format!("dialog.overlay.position.{position:?}"),r.rect);
                        }
                    });
                    ui.label("Title");let title=ui.add(egui::TextEdit::singleline(&mut draft.title).char_limit(96));
                    self.automation.insert("dialog.overlay.title".into(),title.rect);
                    ui.label("Subtitle");let subtitle=ui.add(egui::TextEdit::singleline(&mut draft.subtitle).char_limit(160));
                    self.automation.insert("dialog.overlay.subtitle".into(),subtitle.rect);
                    ui.label("Source overlays, channel effects/timing, alpha, free transforms and borders are unavailable. Only the title-panel fields above apply; stinger channels 9–16 are configured on the engine and do not composite a clip yet.");
                    let w=ui.available_width().min(420.0);
                    let (frame,_)=ui.allocate_exact_size(Vec2::new(w,w*9.0/16.0),Sense::hover());
                    ui.painter().rect_filled(frame,0.0,Color32::from_rgb(18,23,30));
                    let panel=overlay_panel_rect(frame,draft.position,slot);
                    ui.painter().rect_filled(panel,2.0,Color32::from_black_alpha(180));
                    ui.painter().text(panel.center(),egui::Align2::CENTER_CENTER,&draft.title,egui::FontId::proportional(11.0),Color32::WHITE);
                    self.automation.insert("dialog.overlay.preview".into(),frame);
                });
            self.automation.insert("dialog.overlay.body".into(), body);
            ui.separator();
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add(
                        egui::Button::new("Cancel")
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.overlay.cancel".into(), cancel.rect);
                    if cancel.clicked() {
                        cancel_clicked = true;
                    }
                    let apply = ui.add(
                        egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong())
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.overlay.apply".into(), apply.rect);
                    if apply.clicked() {
                        apply_clicked = true;
                    }
                });
            });
        });
        if apply_clicked {
            self.actions
                .push(Action::Engine(Command::SetOverlay(slot, draft)));
            self.overlay_settings = None;
        } else if cancel_clicked || !open {
            self.overlay_settings = None;
        } else {
            self.overlay_settings = Some((slot, draft));
        }
    }

    /// Bounded staged editor for the four external output slots (reference
    /// Task 1.3). Routing and the eight configuration fields are real engine
    /// state; no transport backend exists, so the dialog states that nothing is
    /// transmitted and no device is opened.
    pub(crate) fn external_settings_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut editor) = self.external_settings.clone() else {
            return;
        };
        let mut slot = editor.selected;
        let Some(mut draft) = editor.outputs.iter().find(|o| o.number == slot).cloned() else {
            return;
        };
        let mut open = true;
        let mut apply_clicked = false;
        let mut cancel_clicked = false;
        let project = self.engine.project().clone();
        let height = (ctx.viewport_rect().height() - 210.0).clamp(150.0, 400.0);
        operator_window(
            ctx,
            "External Output",
            "dialog.external_settings",
            &mut open,
            Vec2::new(720.0, 540.0),
        )
        .show(ctx, |ui| {
            let width = ui
                .available_width()
                .min((ui.ctx().viewport_rect().width() - 48.0).max(250.0));
            let (body, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
            let mut body_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("dialog.external.body")
                    .max_rect(body)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            body_ui.set_clip_rect(body.intersect(ui.clip_rect()));
            body_ui.painter().rect_filled(body, 4.0, FORM_LIGHT);
            *body_ui.visuals_mut() = form_visuals();
            egui::ScrollArea::vertical()
                .id_salt("dialog.external.scroll")
                .max_height(height)
                .show(&mut body_ui, |ui| {
            ui.label(
                RichText::new(
                    "Configuration only: no output backend, device or transmission is active.",
                )
                .size(11.5)
                .color(FORM_TEXT),
            );
            ui.label("Apply stores the selected output only. Other drafts remain staged; Cancel discards unapplied changes.");
            if let Some(error) = &editor.error {
                let error = ui.colored_label(RED, format!("Could not apply: {error}. Correct the source or configuration and try again."));
                self.automation.insert("dialog.external.error".into(), error.rect);
            } else if let Some(number) = editor.applied {
                ui.label(format!("Output {number} configuration applied; output remains unavailable."));
            }
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Output").size(11.5).color(FORM_TEXT));
                        for number in 1..=EXTERNAL_OUTPUTS as u8 {
                            let label = if number == 1 {
                                "External".to_string()
                            } else {
                                format!("External {number}")
                            };
                            let r = ui.selectable_label(slot == number, label);
                            self.automation
                                .insert(format!("dialog.external.slot.{number}"), r.rect);
                            if r.clicked() && number != slot {
                                if let Some(staged) = editor.outputs.iter_mut().find(|o| o.number == slot) {
                                    *staged = draft.clone();
                                }
                                slot = number;
                                if let Some(staged) = editor.outputs.iter().find(|o| o.number == number) {
                                    draft = staged.clone();
                                }
                                editor.error = None;
                                editor.applied = None;
                            }
                        }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Source").size(11.5).color(FORM_TEXT));
                        let mut source = draft.source;
                        let none = ui.selectable_value(&mut source, OutputSource::None, "None");
                        self.automation
                            .insert("dialog.external.source.none".into(), none.rect);
                        let program =
                            ui.selectable_value(&mut source, OutputSource::Program, "Program");
                        self.automation
                            .insert("dialog.external.source.program".into(), program.rect);
                        let preview =
                            ui.selectable_value(&mut source, OutputSource::Preview, "Preview");
                        self.automation
                            .insert("dialog.external.source.preview".into(), preview.rect);
                        let selected = match draft.source {
                            OutputSource::Input(id) => project
                                .input(id)
                                .map_or_else(|| "Missing input".to_string(), |i| i.name.clone()),
                            _ => "Input".to_string(),
                        };
                        let input_picker = egui::ComboBox::from_id_salt("dialog.external.input")
                            .width(150.0)
                            .selected_text(selected)
                            .show_ui(ui, |ui| {
                                for input in &project.inputs {
                                    let r = ui.selectable_value(
                                        &mut source,
                                        OutputSource::Input(input.id),
                                        &input.name,
                                    );
                                    self.automation
                                        .insert(format!("dialog.external.input.{}", input.id.0), r.rect);
                                }
                            });
                        self.automation.insert("dialog.external.input".into(), input_picker.response.rect);
                        draft.source = source;
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Frame rate").size(11.5).color(FORM_TEXT));
                        let rate_picker = egui::ComboBox::from_id_salt("dialog.external.rate")
                            .width(150.0)
                            .selected_text(format!("{} fps", draft.config.frame_rate.label()))
                            .show_ui(ui, |ui| {
                                for (num, den, lbl) in [
                                    (24, 1, "24p"),
                                    (25, 1, "25p"),
                                    (30000, 1001, "NTSC 29.97p"),
                                    (30, 1, "NTSC 30p"),
                                    (50, 1, "PAL 50p"),
                                    (60000, 1001, "NTSC 59.94p"),
                                    (60, 1, "60p"),
                                ] {
                                    let rate = Rate {
                                        numerator: num,
                                        denominator: den,
                                    };
                                    let r = ui.selectable_value(
                                        &mut draft.config.frame_rate,
                                        rate,
                                        lbl,
                                    );
                                    self.automation.insert(
                                        format!("dialog.external.rate.{}", rate.label()),
                                        r.rect,
                                    );
                                }
                            });
                        self.automation.insert("dialog.external.rate".into(), rate_picker.response.rect);
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Output size").size(11.5).color(FORM_TEXT));
                        let w = ui.add(
                            egui::DragValue::new(&mut draft.config.width)
                                .range(1..=MAX_OUTPUT_WIDTH)
                                .speed(8.0),
                        );
                        self.automation
                            .insert("dialog.external.width".into(), w.rect);
                        ui.label("×");
                        let h = ui.add(
                            egui::DragValue::new(&mut draft.config.height)
                                .range(1..=MAX_OUTPUT_HEIGHT)
                                .speed(8.0),
                        );
                        self.automation
                            .insert("dialog.external.height".into(), h.rect);
                        ui.label("px");
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Device").size(11.5).color(FORM_TEXT));
                        let d = ui.add(
                            egui::TextEdit::singleline(&mut draft.config.device)
                                .hint_text("Application renderer")
                                .char_limit(96)
                                .desired_width(240.0),
                        );
                        self.automation
                            .insert("dialog.external.device".into(), d.rect);
                    });
                    ui.horizontal(|ui| {
                        let mut has_port = draft.config.port.is_some();
                        let toggle = ui.checkbox(&mut has_port, "Device port");
                        self.automation
                            .insert("dialog.external.port_toggle".into(), toggle.rect);
                        if toggle.changed() {
                            draft.config.port = if has_port { Some(1) } else { None };
                        }
                        if let Some(port) = draft.config.port.as_mut() {
                            let p =
                                ui.add(egui::DragValue::new(port).range(1..=65_535).speed(1.0).prefix("Connector "));
                            self.automation
                                .insert("dialog.external.port".into(), p.rect);
                        } else {
                            ui.label("Default");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Audio device").size(11.5).color(FORM_TEXT));
                        let a = ui.add(
                            egui::TextEdit::singleline(&mut draft.config.audio_device)
                                .hint_text("Default")
                                .char_limit(96)
                                .desired_width(240.0),
                        );
                        self.automation
                            .insert("dialog.external.audio_device".into(), a.rect);
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Audio bus").size(11.5).color(FORM_TEXT));
                        let label = match draft.config.audio_bus {
                            None | Some(0) => "Master".to_string(),
                            Some(1) => "A".to_string(),
                            Some(2) => "B".to_string(),
                            Some(other) => format!("Bus {other}"),
                        };
                        let bus_picker = egui::ComboBox::from_id_salt("dialog.external.bus")
                            .width(120.0)
                            .selected_text(label)
                            .show_ui(ui, |ui| {
                                for (value, name) in
                                    [(None, "Master"), (Some(1_u8), "A"), (Some(2_u8), "B")]
                                {
                                    let r = ui.selectable_label(
                                        draft.config.audio_bus.unwrap_or(0) == value.unwrap_or(0),
                                        name,
                                    );
                                    self.automation
                                        .insert(format!("dialog.external.bus.{name}"), r.rect);
                                    if r.clicked() {
                                        draft.config.audio_bus = value;
                                    }
                                }
                            });
                        self.automation.insert("dialog.external.bus".into(), bus_picker.response.rect);
                    });
                    let alpha = ui.checkbox(&mut draft.config.alpha_channel, "Alpha channel requested (requires output backend)");
                    self.automation.insert("dialog.external.alpha".into(), alpha.rect);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Audio delay").size(11.5).color(FORM_TEXT));
                        let d = ui.add(
                            egui::DragValue::new(&mut draft.config.audio_delay_ms)
                                .range(0..=MAX_AUDIO_DELAY_MS)
                                .speed(5.0)
                                .suffix(" ms"),
                        );
                        self.automation
                            .insert("dialog.external.delay".into(), d.rect);
                    });
                });
            self.automation.insert("dialog.external.body".into(), body);
            ui.separator();
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add(
                        egui::Button::new("Cancel")
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.external.cancel".into(), cancel.rect);
                    if cancel.clicked() {
                        cancel_clicked = true;
                    }
                    let apply = ui.add(
                        egui::Button::new(RichText::new("Apply selected output").color(Color32::WHITE).strong())
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.external.apply".into(), apply.rect);
                    if apply.clicked() {
                        apply_clicked = true;
                    }
                });
            });
        });
        if cancel_clicked || !open {
            self.external_settings = None;
        } else {
            if let Some(staged) = editor.outputs.iter_mut().find(|o| o.number == slot) {
                *staged = draft.clone();
            }
            editor.selected = slot;
            self.external_settings = Some(editor);
            if apply_clicked {
                self.actions.push(Action::ApplyExternal(draft));
            }
        }
    }

    pub(crate) fn recording_setup_dialog(&mut self, ctx: &egui::Context) {
        if !self.recording_setup {
            return;
        }
        let mut open = self.recording_setup;
        let mut close_clicked = false;
        operator_window(
            ctx,
            "Recording Setup",
            "dialog.recording_setup",
            &mut open,
            Vec2::new(680.0, 500.0),
        )
        .show(ctx, |ui| {
            ui.label("Unavailable: no encoder/output backend. Settings below are read-only and are not saved.");
            egui::ScrollArea::vertical().max_height((ctx.viewport_rect().height()-300.0).clamp(180.0,450.0)).show(ui,|ui| {
            ui.add_enabled_ui(false,|ui| {
            ui.horizontal(|ui| {
                for slot in [1_u8, 2_u8] {
                    let active = self.rec_slot == slot;
                    if ui
                        .add_sized(
                            [34.0, 22.0],
                            egui::Button::new(
                                RichText::new(slot.to_string()).color(Color32::WHITE),
                            )
                            .fill(if active {
                                PROGRAM
                            } else {
                                CHIP_BG
                            }),
                        )
                        .clicked()
                    {
                        self.rec_slot = slot;
                    }
                }
            });
            light_form_frame(ui, 12, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Format").strong().color(FORM_TEXT));
                    egui::ComboBox::from_id_salt("rec.format")
                        .width(96.0)
                        .selected_text(
                            ["MP4", "AVI", "MKV", "MPEG-2", "vMix AVI"]
                                .get(self.rec_format)
                                .copied()
                                .unwrap_or("MP4"),
                        )
                        .show_ui(ui, |ui| {
                            for (idx, lbl) in ["MP4", "AVI", "MKV", "MPEG-2", "vMix AVI"]
                                .into_iter()
                                .enumerate()
                            {
                                ui.selectable_value(&mut self.rec_format, idx, lbl);
                            }
                        });
                    ui.label(RichText::new("Filename").strong().color(FORM_TEXT));
                    ui.add_sized(
                        [280.0, 22.0],
                        egui::TextEdit::singleline(&mut self.rec_filename),
                    );
                    let _ = ui.add_sized([74.0, 22.0], egui::Button::new("Browse"));
                });
                ui.add_space(6.0);
                ui.columns(2, |cols| {
                    if let [left_col, right_col, ..] = cols {
                        left_col.label(RichText::new("MP4").size(14.0).strong().color(FORM_TEXT));
                        egui::Grid::new("rec_mp4_grid")
                            .num_columns(2)
                            .spacing([12.0, 6.0])
                            .show(left_col, |ui| {
                                ui.label("Resolution");
                                ui.label("Same as Output");
                                ui.end_row();
                                ui.label("Frame Rate");
                                ui.label("NTSC 29.97p");
                                ui.end_row();
                                ui.label("Profile");
                                egui::ComboBox::from_id_salt("rec.profile")
                                    .width(130.0)
                                    .selected_text("H264Baseline")
                                    .show_ui(ui, |ui| {
                                        let _ = ui.selectable_label(true, "H264Baseline");
                                        let _ = ui.selectable_label(false, "H264High");
                                        let _ = ui.selectable_label(false, "HEVC");
                                    });
                                ui.end_row();
                                ui.label("Bit Rate (Mbps)");
                                ui.add_sized(
                                    [70.0, 20.0],
                                    egui::TextEdit::singleline(&mut self.rec_bitrate),
                                );
                                ui.end_row();
                            });
                        left_col.checkbox(&mut self.rec_hw_encoder, "Use Hardware Encoder");

                        right_col.label(
                            RichText::new("AVI / MKV")
                                .size(14.0)
                                .strong()
                                .color(FORM_TEXT),
                        );
                        egui::Grid::new("rec_avi_grid")
                            .num_columns(2)
                            .spacing([12.0, 6.0])
                            .show(right_col, |ui| {
                                ui.label("Codec");
                                egui::ComboBox::from_id_salt("rec.codec")
                                    .width(130.0)
                                    .selected_text("MPEG-2")
                                    .show_ui(ui, |ui| {
                                        let _ = ui.selectable_label(true, "MPEG-2");
                                        let _ = ui.selectable_label(false, "H264");
                                    });
                                ui.end_row();
                                ui.label("Quality");
                                egui::ComboBox::from_id_salt("rec.quality")
                                    .width(130.0)
                                    .selected_text("25Mbit")
                                    .show_ui(ui, |ui| {
                                        let _ = ui.selectable_label(true, "25Mbit");
                                        let _ = ui.selectable_label(false, "50Mbit");
                                    });
                                ui.end_row();
                            });
                        right_col.checkbox(
                            &mut self.rec_fault_tolerant,
                            "Use Fault Tolerant Recording Type (mkv)",
                        );
                    }
                });
                ui.separator();
                ui.label(RichText::new("Audio").size(13.0).strong().color(FORM_TEXT));
                ui.horizontal(|ui| {
                    ui.label("Audio");
                    egui::ComboBox::from_id_salt("rec.audio_bus")
                        .width(110.0)
                        .selected_text("Master")
                        .show_ui(ui, |ui| {
                            let _ = ui.selectable_label(true, "Master");
                            let _ = ui.selectable_label(false, "Bus A");
                            let _ = ui.selectable_label(false, "None");
                        });
                    ui.label("Audio Delay");
                    ui.add_sized(
                        [60.0, 20.0],
                        egui::TextEdit::singleline(&mut self.rec_audio_delay),
                    );
                    ui.label("Audio Bit Rate");
                    ui.add_sized(
                        [90.0, 20.0],
                        egui::TextEdit::singleline(&mut self.rec_audio_kbps),
                    );
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("New File Every");
                    egui::ComboBox::from_id_salt("rec.split")
                        .width(90.0)
                        .selected_text(&self.rec_split_mins)
                        .show_ui(ui, |ui| {
                            for opt in ["None", "5", "15", "30", "60"] {
                                ui.selectable_value(&mut self.rec_split_mins, opt.into(), opt);
                            }
                        });
                    ui.label("Minutes");
                    ui.add_space(16.0);
                    ui.checkbox(&mut self.rec_audio_master, "Record System Audio");
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                let rec_btn = ui.add_enabled(
                    false,
                    egui::Button::new(
                        RichText::new(if self.rec_active {
                            "Stop Recording"
                        } else {
                            "Start Recording"
                        })
                        .color(Color32::WHITE)
                        .strong(),
                    )
                    .fill(if self.rec_active { RED } else { PROGRAM })
                    .min_size(Vec2::new(124.0, 29.0)),
                );
                if rec_btn.clicked() {
                    self.actions.push(Action::ToggleRecordSim);
                }
                if self.rec_active {
                    ui.colored_label(
                        RED,
                        format!(
                            "REC ACTIVE · {} frames · {:.2} MB",
                            self.rec_frames,
                            self.rec_bytes as f64 / 1_048_576.0
                        ),
                    );
                }
            });
            });
            });
            // The footer stays interactive and outside the read-only,
            // scrollable form region (ADR 0003: dialogs stay operator closable).
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                let cancel = ui.add(
                    egui::Button::new("Cancel")
                        .fill(CHIP_BG)
                        .min_size(Vec2::new(92.0, 29.0)),
                );
                self.automation
                    .insert("dialog.recording.cancel".into(), cancel.rect);
                if cancel.clicked() {
                    close_clicked = true;
                }
                let ok = ui.add(
                    egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong())
                        .fill(PROGRAM)
                        .min_size(Vec2::new(92.0, 29.0)),
                );
                self.automation
                    .insert("dialog.recording.ok".into(), ok.rect);
                if ok.clicked() {
                    self.notice = "Recording backend unavailable; no settings were saved".into();
                    self.warning = false;
                    close_clicked = true;
                }
            });
        });
        if close_clicked || !open {
            self.recording_setup = false;
        }
    }

    pub(crate) fn streaming_setup_dialog(&mut self, ctx: &egui::Context) {
        if !self.streaming_setup {
            return;
        }
        let mut open = self.streaming_setup;
        let mut close_clicked = false;
        operator_window(
            ctx,
            "Streaming Settings",
            "dialog.streaming_setup",
            &mut open,
            Vec2::new(660.0, 490.0),
        )
        .show(ctx, |ui| {
            ui.label("Unavailable: no encoder/output backend. Settings below are read-only and are not saved.");
            egui::ScrollArea::vertical().max_height((ctx.viewport_rect().height()-300.0).clamp(180.0,450.0)).show(ui,|ui| {
            ui.add_enabled_ui(false,|ui| {
            ui.horizontal(|ui| {
                ui.label("Channel");
                for ch in 0..5 {
                    let active = self.stream_channel == ch;
                    if ui
                        .add_sized(
                            [30.0, 22.0],
                            egui::Button::new(
                                RichText::new((ch + 1).to_string()).color(Color32::WHITE),
                            )
                            .fill(if active { PROGRAM } else { CHIP_BG }),
                        )
                        .clicked()
                    {
                        self.stream_channel = ch;
                    }
                }
                ui.add_space(16.0);
                ui.label("Profile");
                egui::ComboBox::from_id_salt("stream.profile")
                    .width(150.0)
                    .selected_text("Default")
                    .show_ui(ui, |ui| {
                        let _ = ui.selectable_label(true, "Default");
                    });
                let _ = ui.add_sized([54.0, 22.0], egui::Button::new("Add").fill(CHIP_BG));
                let _ = ui.add_sized([58.0, 22.0], egui::Button::new("Delete").fill(CHIP_BG));
            });
            light_form_frame(ui, 12, |ui| {
                ui.label(RichText::new("Stream").size(13.0).strong().color(FORM_TEXT));
                egui::Grid::new("stream_setup_grid")
                    .num_columns(3)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Destination");
                        egui::ComboBox::from_id_salt("stream.dest")
                            .width(260.0)
                            .selected_text(
                                [
                                    "Custom RTMP Server",
                                    "YouTube Live",
                                    "Twitch",
                                    "Facebook Live",
                                    "SRT Caller",
                                ]
                                .get(self.stream_destination)
                                .copied()
                                .unwrap_or("Custom RTMP Server"),
                            )
                            .show_ui(ui, |ui| {
                                for (idx, lbl) in [
                                    "Custom RTMP Server",
                                    "YouTube Live",
                                    "Twitch",
                                    "Facebook Live",
                                    "SRT Caller",
                                ]
                                .into_iter()
                                .enumerate()
                                {
                                    ui.selectable_value(&mut self.stream_destination, idx, lbl);
                                }
                            });
                        ui.label("");
                        ui.end_row();

                        ui.label("URL");
                        ui.add_sized(
                            [310.0, 22.0],
                            egui::TextEdit::singleline(&mut self.stream_url),
                        );
                        ui.label("");
                        ui.end_row();

                        ui.label("Stream Key or Name");
                        ui.add_sized(
                            [310.0, 22.0],
                            egui::TextEdit::singleline(&mut self.stream_key).password(true),
                        );
                        let start = ui
                            .add_enabled(
                                false,
                                egui::Button::new(if self.stream_active {
                                    format!("Stop {}", self.stream_channel + 1)
                                } else {
                                    format!("Start {}", self.stream_channel + 1)
                                })
                                .fill(if self.stream_active { RED } else { PROGRAM })
                                .min_size(Vec2::new(78.0, 22.0)),
                            )
                            .on_disabled_hover_text("Unavailable: no streaming encoder or network backend");
                        self.automation.insert("dialog.streaming.start".into(), start.rect);
                        if start.clicked()
                        {
                            self.actions.push(Action::ToggleStreamSim);
                        }
                        ui.end_row();
                    });
                ui.checkbox(&mut self.stream_use_q1, "Use Stream 1 Quality");
                ui.separator();
                ui.label(
                    RichText::new("Quality")
                        .size(13.0)
                        .strong()
                        .color(FORM_TEXT),
                );
                egui::Grid::new("stream_quality_grid")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Quality");
                        egui::ComboBox::from_id_salt("stream.quality_preset")
                            .width(280.0)
                            .selected_text(
                                [
                                    "H264 720p 2.5mbps AAC 128kbps",
                                    "H264 1080p 4.5mbps AAC 192kbps",
                                    "H264 1080p 8mbps AAC 192kbps",
                                ]
                                .get(self.stream_quality)
                                .copied()
                                .unwrap_or("H264 720p 2.5mbps AAC 128kbps"),
                            )
                            .show_ui(ui, |ui| {
                                for (idx, lbl) in [
                                    "H264 720p 2.5mbps AAC 128kbps",
                                    "H264 1080p 4.5mbps AAC 192kbps",
                                    "H264 1080p 8mbps AAC 192kbps",
                                ]
                                .into_iter()
                                .enumerate()
                                {
                                    ui.selectable_value(&mut self.stream_quality, idx, lbl);
                                }
                            });
                        ui.end_row();

                        ui.label("Application");
                        egui::ComboBox::from_id_salt("stream.app")
                            .width(180.0)
                            .selected_text("FFMPEG")
                            .show_ui(ui, |ui| {
                                let _ = ui.selectable_label(true, "FFMPEG");
                                let _ = ui.selectable_label(false, "FMLE");
                            });
                        ui.end_row();
                    });
                ui.label(
                    RichText::new(
                        "Encoder: H264, Format: 1280x720, Bit Rate: 2500 kbps, Keyframe Frequency: 2 seconds\nAudio: Master, Rate: 128 kbps",
                    )
                    .size(11.0)
                    .color(FORM_TEXT),
                );
                ui.checkbox(&mut self.rec_hw_encoder, "Use Hardware Encoder");
            });
            ui.separator();
            ui.horizontal(|ui| {
                if self.stream_active {
                    ui.colored_label(
                        RED,
                        format!(
                            "LIVE · {} kbps · {} frames",
                            self.stream_kbps, self.stream_frames
                        ),
                    );
                } else {
                    ui.label(RichText::new("Unavailable: no encoder / network backend").color(DIM));
                }
            });
            });
            });
            // The footer stays interactive and outside the read-only,
            // scrollable form region (ADR 0003: dialogs stay operator closable).
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                let close = ui.add(
                    egui::Button::new("Close")
                        .fill(CHIP_BG)
                        .min_size(Vec2::new(92.0, 29.0)),
                );
                self.automation
                    .insert("dialog.streaming.close".into(), close.rect);
                if close.clicked() {
                    close_clicked = true;
                }
                let save = ui.add(
                    egui::Button::new(
                        RichText::new("Save and Close").color(Color32::WHITE).strong(),
                    )
                    .fill(PROGRAM)
                    .min_size(Vec2::new(112.0, 29.0)),
                );
                self.automation
                    .insert("dialog.streaming.save".into(), save.rect);
                if save.clicked() {
                    self.notice = "Streaming backend unavailable; no settings were saved".into();
                    self.warning = false;
                    close_clicked = true;
                }
            });
        });
        if close_clicked || !open {
            self.streaming_setup = false;
        }
    }

    pub(crate) fn multicorder_dialog(&mut self, ctx: &egui::Context) {
        if !self.multicorder_open {
            return;
        }
        let mut open = self.multicorder_open;
        let mut close_clicked = false;
        operator_window(
            ctx,
            "MultiCorder",
            "dialog.multicorder",
            &mut open,
            Vec2::new(690.0, 480.0),
        )
        .show(ctx, |ui| {
            ui.label("Unavailable: no encoder/output backend. Settings below are read-only and are not saved.");
            egui::ScrollArea::vertical().max_height((ctx.viewport_rect().height()-300.0).clamp(180.0,450.0)).show(ui,|ui| {
            ui.add_enabled_ui(false,|ui| {
            ui.columns(2, |cols| {
                if let [left_col, right_col, ..] = cols {
                    egui::Grid::new("mc_top_left")
                        .num_columns(4)
                        .spacing([8.0, 4.0])
                        .show(left_col, |ui| {
                            ui.label("Format");
                            egui::ComboBox::from_id_salt("mc.format")
                                .width(90.0)
                                .selected_text(
                                    ["AVI", "MKV", "MP4", "vMix AVI"]
                                        .get(self.mc_format)
                                        .copied()
                                        .unwrap_or("AVI"),
                                )
                                .show_ui(ui, |ui| {
                                    for (idx, lbl) in
                                        ["AVI", "MKV", "MP4", "vMix AVI"].into_iter().enumerate()
                                    {
                                        ui.selectable_value(&mut self.mc_format, idx, lbl);
                                    }
                                });
                            ui.label("Audio");
                            egui::ComboBox::from_id_salt("mc.audio")
                                .width(90.0)
                                .selected_text("Input")
                                .show_ui(ui, |ui| {
                                    let _ = ui.selectable_label(true, "Input");
                                    let _ = ui.selectable_label(false, "Master");
                                });
                            ui.end_row();

                            ui.label("Codec");
                            egui::ComboBox::from_id_salt("mc.codec")
                                .width(90.0)
                                .selected_text("MJPEG")
                                .show_ui(ui, |ui| {
                                    let _ = ui.selectable_label(true, "MJPEG");
                                    let _ = ui.selectable_label(false, "H264");
                                });
                            ui.label("Quality");
                            egui::ComboBox::from_id_salt("mc.quality")
                                .width(90.0)
                                .selected_text("50Mbit")
                                .show_ui(ui, |ui| {
                                    let _ = ui.selectable_label(true, "50Mbit");
                                    let _ = ui.selectable_label(false, "100Mbit");
                                });
                            ui.end_row();
                        });
                    left_col.checkbox(
                        &mut self.mc_ndi_mov,
                        "Record NDI QuickTime MOV where available",
                    );
                    left_col.checkbox(&mut self.mc_show_unsupported, "Show unsupported inputs");

                    right_col.horizontal(|ui| {
                        egui::Frame::new()
                            .fill(Color32::from_rgb(18, 23, 30))
                            .corner_radius(4.0)
                            .stroke(Stroke::new(1.0, FORM_BORDER))
                            .inner_margin(10.0)
                            .show(ui, |ui| {
                                ui.set_min_size(Vec2::new(135.0, 44.0));
                                ui.label(
                                    RichText::new(if self.mc_running {
                                        "Recording"
                                    } else {
                                        "Stopped"
                                    })
                                    .size(18.0)
                                    .color(if self.mc_running { RED } else { Color32::WHITE }),
                                );
                            });
                        let toggle = ui.add_enabled_ui(false, |ui| {
                            ui.add_sized(
                                [115.0, 56.0],
                                egui::Button::new(
                                    RichText::new(if self.mc_running { "Stop" } else { "Start" })
                                        .size(18.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(if self.mc_running {
                                    RED
                                } else {
                                    PROGRAM
                                }),
                            )
                        });
                        self.automation
                            .insert("dialog.multicorder.toggle".into(), toggle.inner.rect);
                        if toggle.inner.clicked() {
                            self.say(
                                "Unavailable: MultiCorder has no recording backend".into(),
                                true,
                            );
                        }
                    });
                }
            });
            ui.add_space(4.0);
            let mc_elapsed_s = if self.mc_running {
                self.now.saturating_sub(self.mc_started_ms) / 1000
            } else {
                0
            };
            let mc_frames = if self.mc_running {
                (self.now.saturating_sub(self.mc_started_ms) * 30 / 1000).max(1)
            } else {
                0
            };
            light_form_frame(ui, 6, |ui| {
                egui::Grid::new("mc_inputs_table")
                    .striped(true)
                    .num_columns(5)
                    .min_col_width(80.0)
                    .show(ui, |ui| {
                        for col in ["Input", "Folder", "Duration", "Frames", "Dropped"] {
                            ui.label(RichText::new(col).strong().color(FORM_TEXT));
                        }
                        ui.end_row();
                        let mut rows = vec![
                            "Output 1".to_owned(),
                            "Output 2".to_owned(),
                            "Output 3".to_owned(),
                            "Output 4".to_owned(),
                        ];
                        for input in &self.engine.project().inputs {
                            rows.push(format!(
                                "{}: {}",
                                self.engine.project().input_number(input.id),
                                input.name
                            ));
                        }
                        for (idx, label) in rows.into_iter().take(10).enumerate() {
                            let is_checked = self.mc_checked.get(idx).copied().unwrap_or(false);
                            if let Some(checked) = self.mc_checked.get_mut(idx) {
                                ui.checkbox(checked, label);
                            }
                            ui.label(&self.mc_folder);
                            ui.label(if self.mc_running && is_checked {
                                format!("00:{:02}:{:02}", mc_elapsed_s / 60, mc_elapsed_s % 60)
                            } else {
                                "00:00:00".into()
                            });
                            ui.label(if self.mc_running && is_checked {
                                mc_frames.to_string()
                            } else {
                                "0".into()
                            });
                            ui.label("0");
                            ui.end_row();
                        }
                    });
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Folder");
                ui.add_sized(
                    [260.0, 22.0],
                    egui::TextEdit::singleline(&mut self.mc_folder),
                );
                let _ = ui.add_sized([74.0, 22.0], egui::Button::new("Browse").fill(CHIP_BG));
            });
            });
            });
            // The footer stays interactive and outside the scrollable form
            // region (ADR 0003: dialogs stay operator closable).
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                let close = ui.add(
                    egui::Button::new("Close")
                        .fill(CHIP_BG)
                        .min_size(Vec2::new(92.0, 29.0)),
                );
                self.automation
                    .insert("dialog.multicorder.close".into(), close.rect);
                if close.clicked() {
                    close_clicked = true;
                }
            });
        });
        if close_clicked || !open {
            self.multicorder_open = false;
        }
    }

    pub(crate) fn shortcuts_dialog(&mut self, ctx: &egui::Context) {
        if !self.shortcuts_open {
            return;
        }
        let mut open = self.shortcuts_open;
        let mut close_clicked = false;
        operator_window(
            ctx,
            "Shortcuts",
            "dialog.shortcuts",
            &mut open,
            Vec2::new(620.0, 440.0),
        )
        .show(ctx, |ui| {
            let hint = |action| self.engine.project().preferences.key_bindings.iter()
                .find(|binding| binding.action == action).map_or_else(|| "Unbound".into(), ShortcutBinding::label);
            let transition = |index: usize| self.engine.project().transition_slots.get(index)
                .map_or_else(|| "Missing slot".into(), |slot| format!("{}\n[Slot {} · {} ms]", slot.kind.label(), index+1, slot.duration_ms));
            let preview = |number| self.engine.shortcut_command(&ShortcutAction::PreviewNumber(number))
                .map(Action::Engine).unwrap_or(Action::Unavailable("Input number is unavailable"));
            let tiles: [(String, Action); 12] = [
                (format!("Cut\n[{}]", hint(ShortcutAction::Cut)), Action::Engine(Command::Cut)),
                (transition(0), Action::Engine(Command::RunTransitionSlot(0))),
                (transition(1), Action::Engine(Command::RunTransitionSlot(1))),
                (transition(2), Action::Engine(Command::RunTransitionSlot(2))),
                (format!("FTB\n[{}]", hint(ShortcutAction::FadeBlack)), Action::Engine(Command::FadeBlack)),
                ("Overlay 1\n[Settings]".into(), Action::OverlaySettings(0)),
                ("Overlay 2\n[Settings]".into(), Action::OverlaySettings(1)),
                ("Preview 1\n[Input number]".into(), preview(1)),
                ("Preview 2\n[Input number]".into(), preview(2)),
                ("Preview 3\n[Input number]".into(), preview(3)),
                ("Record\n[Unavailable]".into(), Action::ToggleRecordSim),
                ("Stream\n[Unavailable]".into(), Action::ToggleStreamSim),
            ];
            egui::Grid::new("shortcuts_button_grid")
                .num_columns(3)
                .spacing([12.0, 12.0])
                .show(ui, |ui| {
                    for (idx, (label, action)) in tiles.into_iter().enumerate() {
                        let enabled = match &action {
                            Action::Unavailable(_) | Action::ToggleRecordSim | Action::ToggleStreamSim | Action::ToggleExternalSim => false,
                            Action::Engine(command) => self.engine.enabled(command).is_ok(),
                            _ => true,
                        };
                        let button = ui
                            .add_enabled(
                                enabled,
                                egui::Button::new(
                                    RichText::new(label).size(13.0).color(Color32::WHITE),
                                )
                                .fill(CHIP_BG).min_size(Vec2::new(182.0, 74.0)),
                            )
                            .on_disabled_hover_text("Unavailable: finish the active transition or choose an implemented action");
                        self.automation.insert(format!("dialog.shortcuts.action.{idx}"), button.rect);
                        if button.clicked()
                        {
                            self.actions.push(action);
                        }
                        if idx % 3 == 2 {
                            ui.end_row();
                        }
                    }
                });
            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                if ui
                    .add_sized([78.0, 29.0], egui::Button::new("Add").fill(CHIP_BG))
                    .clicked()
                {
                    self.shortcuts_open = false;
                    self.actions.push(Action::SettingsAt(SettingsPage::Shortcuts));
                }
                if ui
                    .add_sized([84.0, 29.0], egui::Button::new("Settings").fill(CHIP_BG))
                    .clicked()
                {
                    self.shortcuts_open = false;
                    self.actions.push(Action::SettingsAt(SettingsPage::Shortcuts));
                }
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let close = ui.add(
                        egui::Button::new("Close")
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.shortcuts.close".into(), close.rect);
                    if close.clicked() {
                        close_clicked = true;
                    }
                });
            });
        });
        if close_clicked || !open {
            self.shortcuts_open = false;
        }
    }

    pub(crate) fn remove_input_confirmation(&mut self, ctx: &egui::Context) {
        let Some(id) = self.confirm_remove_input else {
            return;
        };
        let label = self
            .engine
            .project()
            .input(id)
            .map(|input| {
                format!(
                    "input {} — {}",
                    self.engine.project().input_number(id),
                    input.name
                )
            })
            .unwrap_or_else(|| format!("input {}", id.0));
        let mut open = true;
        let mut cancel_clicked = false;
        operator_window(
            ctx,
            "Remove input?",
            "dialog.remove_input",
            &mut open,
            Vec2::new(500.0, 220.0),
        )
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading(RichText::new("Confirm input removal").color(PREVIEW));
            ui.label(format!(
                "Remove {label} from this preset? Inputs on Program or Preview are protected; select another input before removing them."
            ));
            ui.add_space(12.0);
            ui.separator();
            ui.horizontal(|ui| {
                let confirm = ui.add_enabled(
                    self.engine.enabled(&Command::RemoveInput(id)).is_ok(),
                    egui::Button::new(RichText::new("Remove input").color(RED).strong())
                        .min_size(Vec2::new(118.0, 29.0)),
                );
                self.automation
                    .insert("dialog.remove.confirm".into(), confirm.rect);
                if confirm.clicked() && self.try_execute(Command::RemoveInput(id)).is_some() {
                    self.submit_program();
                    self.confirm_remove_input = None;
                    if self.properties.as_ref().is_some_and(|input| input.id == id) {
                        self.close_properties();
                    }
                }
                let cancel =
                    ui.add(egui::Button::new("Keep input").min_size(Vec2::new(96.0, 29.0)));
                self.automation
                    .insert("dialog.remove.cancel".into(), cancel.rect);
                if cancel.clicked() {
                    cancel_clicked = true;
                }
            });
        });
        if cancel_clicked || !open {
            self.confirm_remove_input = None;
        }
    }
}

#[cfg(test)]
mod operator_dialog_tests {
    // Test fixtures intentionally fail fast; runtime paths remain fallible.
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn frame(app: &mut RustMixApp, ctx: &egui::Context, events: Vec<egui::Event>) {
        frame_at_size(app, ctx, events, Vec2::new(1600.0, 900.0));
    }

    fn frame_at_size(
        app: &mut RustMixApp,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
        size: Vec2,
    ) {
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            events,
            ..Default::default()
        };
        let _ = ctx.run_ui(raw, |ui| app.frame_ui(ui));
    }

    fn click(app: &mut RustMixApp, ctx: &egui::Context, id: &str) {
        for _ in 0..4 {
            frame(app, ctx, vec![]);
        }
        let rect = *app.automation.get(id).expect("visible dialog widget");
        assert!(
            ctx.viewport_rect().contains_rect(rect),
            "widget must be on screen: {id}"
        );
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(rect.center()),
                    egui::Event::PointerButton {
                        pos: rect.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }

    #[test]
    fn source_dialog_proportions_grow_while_compact_footers_stay_reachable() {
        for size in [
            Vec2::new(734.0, 466.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(1920.0, 1080.0),
        ] {
            for (action, prefix, width_ratio, navigation_ratio) in [
                (Action::Add, "dialog.add", 0.707, 0.184),
                (
                    Action::Properties(InputId(3)),
                    "dialog.properties",
                    0.731,
                    0.142,
                ),
            ] {
                let mut app = RustMixApp::headless();
                let ctx = egui::Context::default();
                app.theme(&ctx);
                app.dispatch(action, &ctx);
                for _ in 0..4 {
                    frame_at_size(&mut app, &ctx, vec![], size);
                }
                app.validate_dialog_geometry(&ctx, prefix).unwrap();
                let body = app.automation.get(&format!("{prefix}.body")).unwrap();
                let navigation = app.automation.get(&format!("{prefix}.navigation")).unwrap();
                if size.x >= 1600.0 {
                    assert!((body.width() / size.x - width_ratio).abs() < 0.04);
                    assert!((navigation.width() / body.width() - navigation_ratio).abs() < 0.02);
                    assert!(
                        body.height() > 520.0,
                        "large workspaces should expose more source content"
                    );
                }
                assert!(body.contains_rect(*navigation));
                assert_eq!(app.engine.project().program, InputId(1));
            }
        }
    }

    #[test]
    fn category_editor_stages_real_categories_without_changing_the_generator() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let original = app.engine.project().input(InputId(3)).unwrap().clone();
        app.dispatch(Action::Properties(original.id), &ctx);
        click(&mut app, &ctx, "dialog.properties.category");
        click(&mut app, &ctx, "dialog.properties.category.0");
        assert_eq!(app.properties.as_ref().unwrap().category, Some(0));
        assert_eq!(app.properties.as_ref().unwrap().kind, original.kind);
        assert_eq!(app.engine.project().input(original.id).unwrap(), &original);
        click(&mut app, &ctx, "dialog.properties.cancel");
        assert_eq!(app.engine.project().input(original.id).unwrap(), &original);

        app.dispatch(Action::Properties(original.id), &ctx);
        click(&mut app, &ctx, "dialog.properties.category");
        click(&mut app, &ctx, "dialog.properties.category.1");
        click(&mut app, &ctx, "dialog.properties.apply");
        let updated = app.engine.project().input(original.id).unwrap();
        assert_eq!(updated.category, Some(1));
        assert_eq!(updated.kind, original.kind);
        assert_eq!(app.engine.project().program, InputId(1));
        assert_eq!(app.engine.project().preview, InputId(2));
    }

    #[test]
    fn empty_generator_cannot_submit_from_the_add_dialog() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.dispatch(Action::Add, &ctx);
        app.add_name = "   ".into();
        let original_count = app.engine.project().inputs.len();
        click(&mut app, &ctx, "dialog.add.confirm");
        assert!(app.add_open);
        assert_eq!(app.engine.project().inputs.len(), original_count);
        app.add_name = "Operator generator".into();
        click(&mut app, &ctx, "dialog.add.confirm");
        assert!(!app.add_open);
        assert_eq!(app.engine.project().inputs.len(), original_count + 1);
    }
}
