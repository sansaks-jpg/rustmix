//! Operator dialogs modeled after the reference workflows in `rustmix-reference`.
use super::add_input::{
    CATEGORY_ORDER, SourceIcon, draw_source_icon, generator_category, source_icon,
};
pub(super) use super::operator_window::operator_window;
use super::operator_window::{FooterRole, footer_size};
use super::*;
use egui::{Align, Color32, RichText, Sense, Stroke, Vec2};

const SIDEBAR: Color32 = Color32::from_rgb(14, 18, 24);
const HEADER_DARK: Color32 = Color32::from_rgb(20, 26, 34);
const FORM_BORDER: Color32 = Color32::from_rgb(190, 196, 203);

pub(super) fn form_visuals() -> egui::Visuals {
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

/// Applies the compact, flat form surface used inside operator dialogs (`Add Input`,
/// `Input Settings`, `Settings`, `Recording Setup`, `Streaming Settings`, `Overlay Settings`,
/// `PlayList`, `MultiCorder`).
pub(super) fn light_form_frame<R>(
    ui: &mut egui::Ui,
    margin: i8,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    // Frame paints through its parent painter after laying out its child.
    // Restore opacity before construction so both the background and controls
    // stay readable; this never changes the parent's disabled interaction state.
    let parent_opacity = ui.opacity();
    if !ui.is_enabled() {
        ui.set_opacity(1.0);
    }
    let response = egui::Frame::new()
        .fill(FORM_LIGHT)
        .corner_radius(1.0)
        .stroke(Stroke::new(1.0, FORM_BORDER))
        .inner_margin(margin)
        .show(ui, |ui| {
            *ui.visuals_mut() = form_visuals();
            // Unsupported forms stay non-interactive, but their entire light
            // background and labels must not fade into the dark workspace.
            if !ui.is_enabled() {
                ui.set_opacity(1.0);
            }
            add_contents(ui)
        });
    ui.set_opacity(parent_opacity);
    response
}

// Presentation-only recording navigation never changes engine/output configuration.
fn recording_rate_description(rate: Rate) -> String {
    format!(
        "{}p ({}/{})",
        rate.label(),
        rate.numerator,
        rate.denominator
    )
}

fn recording_form_row<R>(
    ui: &mut egui::Ui,
    label: &str,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.horizontal(|ui| {
        let label_width = (ui.available_width() * 0.21).clamp(86.0, 112.0);
        ui.allocate_ui_with_layout(
            Vec2::new(label_width, 22.0),
            egui::Layout::right_to_left(Align::Center),
            |ui| {
                ui.label(label);
            },
        );
        ui.add_space(8.0);
        contents(ui)
    })
}

fn sidebar_row(
    ui: &mut egui::Ui,
    active: bool,
    label: &str,
    row_height: f32,
    icon: Option<SourceIcon>,
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
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                ui.is_enabled(),
                active,
                label,
            )
        });
        if response.clicked() {
            response.request_focus();
        }
        if response.has_focus() {
            ui.painter().rect_stroke(
                rect.shrink(1.0),
                1.0,
                Stroke::new(1.0, Color32::WHITE),
                egui::StrokeKind::Inside,
            );
        }
        let text_x = if let Some(icon) = icon {
            let icon_rect = Rect::from_min_size(
                egui::pos2(rect.left() + 6.0, rect.center().y - 6.0),
                Vec2::splat(12.0),
            );
            draw_source_icon(ui.painter(), icon_rect, icon, Color32::WHITE);
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
                egui::FontId::proportional((row_height * 14.0 / 38.0).clamp(12.0, 14.0)),
                Color32::WHITE,
            );
        response.on_hover_text(label)
    })
    .inner
}

pub(super) const ADD_CATEGORIES: [(&str, InputKind, &str); 26] = [
    (
        "Generators",
        InputKind::Grid,
        "Choose an original procedural generator.",
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
        "Stream / SRT",
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
        "Title",
        InputKind::Graphics,
        "Procedural lower-third title and CG graphic overlay.",
    ),
    (
        "Countdown",
        InputKind::Countdown,
        "Original procedural countdown generator.",
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
    ("DVD", InputKind::Color, "DVD input is unavailable."),
    (
        "Camera",
        InputKind::Geometry,
        "Camera input is unavailable.",
    ),
    (
        "NDI / OMT / Desktop Capture",
        InputKind::Studio,
        "Network and desktop capture are unavailable.",
    ),
    (
        "Video Delay",
        InputKind::Color,
        "Video delay is unavailable.",
    ),
    (
        "Telestrator",
        InputKind::Graphics,
        "Telestrator is unavailable.",
    ),
];

/// Add-dialog category index for the local MP4 video-file source. Appended so
/// the observed reference categories keep their existing ordinals.
pub(crate) const VIDEO_TAB: usize = 20;
const VIDEO_TAB_LABEL: &str = "Video file";

pub(super) const GENERATOR_SAMPLES: [(&str, InputKind); 7] = [
    ("Motion grid", InputKind::Grid),
    ("Moving shapes", InputKind::Geometry),
    ("Reference bars", InputKind::Bars),
    ("Lower third graphic", InputKind::Graphics),
    ("Studio pattern", InputKind::Studio),
    ("Colour gradient", InputKind::Color),
    ("Countdown 60 seconds", InputKind::Countdown),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum OverlaySwitchDecision {
    Apply,
    Discard,
    Keep,
}

fn aligned_field_label(ui: &mut egui::Ui, label: &str, width: f32) -> Rect {
    // Grid measures occupied child bounds, not the requested child size.
    // Reserve the whole column before painting its right-aligned label.
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 26.0), Sense::hover());
    let mut label_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::right_to_left(Align::Center)),
    );
    label_ui.label(label);
    rect
}

/// A selected staged panel rendered by the Program compositor, with no live mutation.
fn overlay_draft_pixels(
    project: &ProjectState,
    slot: usize,
    draft: &OverlayState,
) -> Result<Vec<[u8; 4]>> {
    let mut preview = project.clone();
    let selected = preview
        .overlays
        .standard_mut(slot)
        .ok_or_else(|| Error::Invalid("Overlay channel is unavailable".into()))?;
    *selected = draft.clone();
    preview.overlays.validate(&preview.inputs)?;
    for (index, overlay) in preview.overlays.standard.iter_mut().enumerate() {
        if index != slot {
            overlay.enabled = false;
        }
    }
    let background = vec![[18, 23, 30, 255]; 640 * 360];
    render::program_from_sources(&preview, None, 0.0, 640, 360, &background, None)
}

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
        let navigation_width = if id == "dialog.settings" {
            (width * 0.218).max(140.0)
        } else if width >= 900.0 {
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
        if matches!(id, "dialog.add" | "dialog.properties" | "dialog.settings") {
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
            && fx.media_path.as_deref() == Some(self.add_png_path.as_str())
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
                    update_texture_image(texture, image);
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
        if generator_category(self.add_tab) || (self.add_tab == 1 && self.add_image.is_some()) {
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
            Vec2::new(
                (ctx.viewport_rect().width() * 0.707).max(780.0),
                height + 70.0,
            ),
        )
        .show(ctx, |ui| {
            self.dialog_columns(
                ui,
                "dialog.add",
                height,
                |app, ui| {
                    if app.add_loading {
                        ui.disable();
                    }
                    ui.spacing_mut().item_spacing.y = 1.0;
                    let categories = CATEGORY_ORDER.into_iter().filter_map(|idx| {
                        ADD_CATEGORIES
                            .get(idx)
                            .copied()
                            .map(|category| (idx, category))
                    });
                    let row_height =
                        (ctx.viewport_rect().width() * 38.0 / 1917.0).clamp(25.0, 38.0);
                    for (idx, (label, kind, _)) in categories {
                        let active = app.add_tab == idx;
                        let r = sidebar_row(ui, active, label, row_height, Some(source_icon(idx)));
                        if active && app.add_reveal {
                            if !ui.clip_rect().contains_rect(r.rect) {
                                r.scroll_to_me(None);
                            }
                            app.add_reveal = false;
                        }
                        app.automation
                            .insert(format!("dialog.add.category.{idx}"), r.rect);
                        if idx == VIDEO_TAB {
                            app.automation.insert("dialog.add.video".into(), r.rect);
                        } else if matches!(idx, 0 | 1 | 3 | 4 | 6 | 11 | 14 | 15) {
                            app.automation.insert(
                                if idx == 1 {
                                    "dialog.add.image".into()
                                } else {
                                    format!("dialog.add.kind.{kind:?}")
                                },
                                r.rect,
                            );
                        }
                        if matches!(idx, 12 | 13) {
                            app.automation.insert(
                                if idx == 12 {
                                    "dialog.add.audio.wav".into()
                                } else {
                                    "dialog.add.audio.microphone".into()
                                },
                                r.rect,
                            );
                        }
                        if r.clicked() {
                            app.select_add_category(idx);
                        }
                    }
                },
                |app, ui| {
                    if app.add_loading {
                        ui.disable();
                    }
                    app.add_source_page(ui, ctx);
                },
            );
            ui.separator();
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add(
                        egui::Button::new(RichText::new("Cancel").color(Color32::WHITE))
                            .fill(CHIP_BG)
                            .min_size(footer_size(ui, FooterRole::Cancel)),
                    );
                    self.automation
                        .insert("dialog.add.cancel".into(), cancel.rect);
                    if cancel.clicked() {
                        close_requested = true;
                    }
                    let disabled_reason = self.add_disabled_reason();
                    let can_add = disabled_reason.is_none();
                    let confirm = ui.add_enabled(
                        can_add,
                        egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong())
                            .fill(PROGRAM)
                            .min_size(footer_size(ui, FooterRole::Confirm)),
                    );
                    let confirm =
                        confirm.on_disabled_hover_text(disabled_reason.unwrap_or_default());
                    self.automation
                        .insert("dialog.add.confirm".into(), confirm.rect);
                    if confirm.clicked() && !self.add_loading {
                        close_requested = if matches!(self.add_tab, 9 | 17 | 22) {
                            self.confirm_extra_source(ctx)
                        } else if matches!(self.add_tab, 12 | 13) {
                            self.confirm_audio_input(ctx)
                        } else if self.add_tab == VIDEO_TAB {
                            self.confirm_video_input(ctx)
                        } else {
                            self.request_add_input()
                        };
                    }
                    ui.add_space(12.0);
                    ui.add_enabled_ui(!self.add_loading, |ui| {
                        let number = egui::ComboBox::from_id_salt("dialog.add.number")
                            .width(64.0)
                            .selected_text(self.add_number.to_string())
                            .show_ui(ui, |ui| {
                                for n in 1..=self.engine.project().inputs.len() + 1 {
                                    ui.selectable_value(&mut self.add_number, n, n.to_string());
                                }
                            });
                        self.automation
                            .insert("dialog.add.number".into(), number.response.rect);
                    });
                    ui.label(RichText::new("Number").size(12.0).color(Color32::WHITE));
                    if self.add_loading {
                        ui.label("Loading PNG…");
                    }
                });
            });
        });
        if close_requested || !open {
            self.add_open = false;
            self.add_loading = false;
            self.add_image = None;
            self.add_preview = None;
            self.add_png_path.clear();
            self.add_error = None;
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
                        let row_height = if ctx.viewport_rect().height() < 700.0 { 18.5 } else { (ctx.viewport_rect().width() * 38.0 / 1917.0).clamp(25.0,38.0) };
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
                                let (header,_) = ui.allocate_exact_size(Vec2::new(label_w,20.0),Sense::hover());
                                let mut header_ui = ui.new_child(egui::UiBuilder::new()
                                    .id_salt("properties.sourceCaption")
                                    .max_rect(header)
                                    .layout(egui::Layout::left_to_right(Align::Center)));
                                header_ui.set_clip_rect(header.intersect(ui.clip_rect()));
                                header_ui.add(egui::Label::new(RichText::new(&path_lbl).color(Color32::WHITE).size(12.0)).truncate()).on_hover_text(&path_lbl);
                                app.automation.insert("dialog.properties.sourceHeader".into(),header);
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
                                    let label_width = (left_col.available_width() * 0.23).clamp(76.0,114.0);
                                    let field_width = (left_col.available_width() - label_width - 8.0).max(80.0);
                                    egui::Grid::new("input_props_left")
                                        .num_columns(2)
                                        .spacing([8.0, 4.0])
                                        .show(left_col, |ui| {
                                            let label = aligned_field_label(ui,"Name",label_width);
                                            app.automation.insert("dialog.properties.name.label".into(),label);
                                            let name = ui.add_sized(
                                                [field_width, 22.0],
                                                egui::TextEdit::singleline(&mut draft.name)
                                                    .char_limit(64),
                                            );
                                            app.automation
                                                .insert("dialog.properties.name".into(), name.rect);
                                            ui.end_row();

                                            let label = aligned_field_label(ui,"Aspect Ratio",label_width);
                                            app.automation.insert("dialog.properties.aspect.label".into(),label);
                                            let aspect = ui.add_enabled(false, egui::Button::new("Widescreen")
                                                .min_size(Vec2::new(field_width, 22.0)))
                                                .on_disabled_hover_text("The current compositor uses a fixed 16:9 working raster. Use Image fit below for Contain, Cover or Stretch.");
                                            app.automation.insert("dialog.properties.aspect".into(),aspect.rect);
                                            ui.end_row();

                                            let label = aligned_field_label(ui,"Category",label_width);
                                            app.automation.insert("dialog.properties.category.label".into(),label);
                                            let selected_category = draft.category
                                                .and_then(|id| app.engine.project().preferences.categories.iter().find(|category| category.id == id))
                                                .map_or("None", |category| category.name.as_str());
                                            let category = egui::ComboBox::from_id_salt("prop.category")
                                                .width(field_width)
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

                                            aligned_field_label(ui,"Generator",label_width);
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
                                    // This bounded margin aligns the flags with the fields above.
                                    egui::Frame::new().inner_margin(egui::Margin {
                                        left:(label_width+8.0).round() as i8,
                                        ..Default::default()
                                    }).show(left_col, |ui| {
                                    ui.add_enabled(false, egui::Checkbox::new(&mut app.prop_deinterlace, "Deinterlace Blend"));
                                    if ui
                                        .checkbox(&mut app.prop_sharpen, "Sharpen")
                                        .changed()
                                    {
                                        app.sync_fx_from_prop(id);
                                    }
                                    let mirror = ui.checkbox(&mut app.prop_mirror, "Mirror");
                                    app.automation.insert("dialog.properties.mirror".into(),mirror.rect);
                                    if mirror.changed() {
                                        app.sync_fx_from_prop(id);
                                    }
                                    ui.add_enabled_ui(false, |ui| {
                                        ui.checkbox(&mut app.prop_flatten, "Flatten Layers");
                                        ui.checkbox(&mut app.prop_auto_mix, "Automatically mix audio");
                                        ui.checkbox(&mut app.prop_auto_play, "Automatically Play with Transition");
                                        ui.checkbox(&mut app.prop_auto_restart, "Automatically Restart with Transition");
                                        ui.checkbox(&mut app.prop_auto_pause, "Automatically Pause after Transition");
                                    }).response.on_hover_text("Automatic input triggers are unavailable");
                                    });
                                    left_col.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 8.0;
                                        aligned_field_label(ui,"Colour",label_width);
                                        let colour = ui.color_edit_button_srgb(&mut draft.color);
                                        app.automation.insert("dialog.properties.colour".into(), colour.rect);
                                    });
                                    if let Some(fx) = app.properties_fx.as_mut() {
                                        left_col.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 8.0;
                                            aligned_field_label(ui,"Image fit",label_width);
                                        let fit = egui::ComboBox::from_id_salt("prop.image_fit")
                                            .width(field_width)
                                            .selected_text(format!("{:?}", fx.image_fit))
                                            .show_ui(ui, |ui| {
                                                for fit in [ImageFit::Contain, ImageFit::Cover, ImageFit::Stretch] {
                                                    ui.selectable_value(&mut fx.config.image_fit, fit, format!("{fit:?}"));
                                                }
                                            });
                                            app.automation.insert("dialog.properties.imageFit".into(),fit.response.rect);
                                        });
                                    }
                                    let metadata_label_width = (right_col.available_width()*0.33).clamp(110.0,180.0);
                                    egui::Grid::new("input_props_right")
                                        .num_columns(2)
                                        .spacing([8.0, 4.0])
                                        .show(right_col, |ui| {
                                            for (key, label, value) in [
                                                ("mouse","Mouse Click Action","Preview (fixed)"),
                                                ("go","GO Action","Preview (fixed)"),
                                                ("sourceAspect","Source Aspect Ratio", "Widescreen"),
                                                ("previewResolution","Preview resolution", "384x216 (demo)"),
                                                ("programResolution","Program resolution", "640x360 (demo)"),
                                                ("deinterlacing","Deinterlacing", "None"),
                                            ] {
                                                let reserved = aligned_field_label(ui,label,metadata_label_width);
                                                app.automation.insert(format!("dialog.properties.metadata.{key}.label"),reserved);
                                                let text = ui.label(RichText::new(value).size(11.5).color(FORM_TEXT));
                                                app.automation.insert(format!("dialog.properties.metadata.{key}"),text.rect);
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
                                const ROUTE_LABELS: [&str; rustmix_engine::audio_engine::BUS_COUNT] =
                                    [
                                        "MASTER", "BUS A", "BUS B", "BUS C", "BUS D", "BUS E",
                                        "BUS F", "BUS G", "HEADPHONES",
                                    ];
                                for (route, label) in draft.audio.sends.iter_mut().zip(ROUTE_LABELS)
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
                            if let Some(source)=&draft.config.video_source
                                && matches!(source,VideoSource::Camera{..}|VideoSource::Browser{..}) {
                                ui.label(format!("Source type: {}",source.label()));
                                let armed=app.armed_sources.contains(&id);
                                if ui.button(if armed {"Stop source"} else {"Start source"}).clicked() {
                                    if armed {app.armed_sources.remove(&id);} else {app.armed_sources.insert(id);}
                                }
                                if let Some(error)=app.input_fx.get(&id).and_then(|fx|fx.image_error.as_ref()) { ui.colored_label(RED,error); }
                            }
                            if let Some(VideoSource::File { path }) = &draft.config.video_source {
                                ui.label("Source type: MP4 video, decoded on its own owner worker. Video audio is linked to the Audio Mixer.");
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
                        InputPage::Position | InputPage::ColourAdjust | InputPage::ColourCorrection | InputPage::Effects => {
                            ui.label(
                                RichText::new(match app.input_page {
                                    InputPage::Position => "Position",
                                    InputPage::Effects => "Effects",
                                    InputPage::ColourCorrection => "Colour Correction",
                                    _ => "Colour Adjust",
                                })
                                    .size(13.0)
                                    .strong()
                                    .color(FORM_TEXT),
                            );
                            let mut changed = false;
                            if app.input_page == InputPage::Position {
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
                                .add(egui::Slider::new(&mut app.prop_pan_y, -1.0..=1.0).text("Pan Y"))
                                .changed();
                            } else if app.input_page == InputPage::Effects {
                                changed |= ui.checkbox(&mut app.prop_mirror, "Mirror").changed();
                                changed |= ui.checkbox(&mut app.prop_sharpen, "Sharpen").changed();
                            } else {
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
                            }
                            let reset = ui.small_button("Reset this page");
                            app.automation.insert("dialog.properties.processing.reset".into(), reset.rect);
                            if reset.clicked() {
                                if app.input_page == InputPage::Position {
                                app.prop_zoom = 1.0;
                                app.prop_pan_x = 0.0;
                                app.prop_pan_y = 0.0;
                                } else if app.input_page == InputPage::Effects {
                                    app.prop_mirror = false;
                                    app.prop_sharpen = false;
                                } else {
                                app.prop_lift = 0.0;
                                app.prop_gain = 1.0;
                                app.prop_saturation = 1.0;
                                }
                                changed = true;
                            }
                            if changed {
                                app.sync_fx_from_prop(id);
                            }
                        }
                        InputPage::Triggers | InputPage::TallyLights | InputPage::Ptz | InputPage::Advanced => {
                            ui.label("Unavailable: this input has no backend for this page.");
                        }
                    });
                    ui.add_space(4.0);
                    // Bottom black preview section with live GPU texture and zoom slider.
                    let consumed = ui.cursor().top() - ui.min_rect().top();
                    let visible_preview_height = (height - consumed - 16.0).max(96.0);
                    egui::Frame::new()
                        .fill(Color32::from_rgb(8, 10, 14))
                        .corner_radius(4.0)
                        .inner_margin(6.0)
                        .show(ui, |ui| {
                            ui.set_min_width((ui.available_width() - 2.0).max(80.0));
                            ui.horizontal(|ui| {
                                let avail_w = (ui.available_width() - 42.0).max(160.0);
                                let preview_h = if ctx.viewport_rect().height() >= 850.0 {
                                    (height * 0.40).clamp(190.0, 320.0).min(visible_preview_height)
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
                        egui::Button::new(RichText::new("Cancel").color(Color32::WHITE))
                            .fill(CHIP_BG)
                            .min_size(footer_size(ui, FooterRole::Cancel)),
                    );
                    self.automation
                        .insert("dialog.properties.cancel".into(), cancel.rect);
                    if cancel.clicked() {
                        cancel_clicked = true;
                    }
                    let apply = ui.add_enabled(!self.properties_loading,
                        egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong())
                            .fill(PROGRAM)
                            .min_size(footer_size(ui, FooterRole::Confirm)),
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
            Vec2::new(ctx.viewport_rect().width() * 0.60, 550.0),
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
                        (SettingsPage::Outputs, "Outputs / NDI / OMT / SRT", "Outputs"),
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
                        (SettingsPage::Workspace, "Workspace", "Workspace"),
                        (SettingsPage::TallyLights, "Tally Lights", "TallyLights"),
                        (SettingsPage::Shortcuts, "Shortcuts", "Shortcuts"),
                        (SettingsPage::Activators, "Activators", "Activators"),
                        (SettingsPage::Scripting, "Scripting", "Scripting"),
                        (SettingsPage::Alerts, "Alerts", "Alerts"),
                        (SettingsPage::About, "About", "About"),
                    ].into_iter().filter(|(page, _, _)| core_scope::settings_page(*page)) {
                        let r = sidebar_row(ui, app.settings_page == page, label, 27.0, None);
                        app.automation
                            .insert(format!("dialog.settings.page.{key}"), r.rect);
                        if r.clicked() {
                            app.settings_page = page;
                        }
                    }
                    // Keep the navigation actions together at the bottom when
                    // all pages fit. Compact layouts can scroll this navigation
                    // independently of the permanently reachable OK/Cancel footer.
                    let remaining = height - 12.0 - ui.min_rect().height() - 31.0;
                    ui.add_space(remaining.max(4.0));
                    let footer_width = ui.available_width();
                    let footer = ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        ui.spacing_mut().button_padding = Vec2::new(2.0, 3.0);
                        let button_width = footer_width.max(30.0);
                        {
                            let action = ui.add(
                                egui::Button::new(RichText::new("Default").size(10.0))
                                    .fill(CHIP_BG)
                                    .min_size(Vec2::new(button_width, 27.0)),
                            );
                            app.automation.insert("dialog.settings.navigation.Default".into(), action.rect);
                            if action.clicked() {
                                reset_default = true;
                            }
                        }
                    });
                    app.automation.insert("dialog.settings.navigation.footer".into(), footer.response.rect);
                },
                |app, ui| {
                    let field_width = (ui.ctx().viewport_rect().width() * (300.0 / 1600.0))
                        .clamp(140.0, 360.0);
                    light_form_frame(ui, 12, |ui| {
                        ui.spacing_mut().combo_width = field_width;
                        match app.settings_page {
                        SettingsPage::Display => {
                            app.display_settings(ui, &mut draft, field_width);
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
                                RichText::new("Workspace Layout")
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
                        page => app.settings_detail_page(ui, page, &mut draft),
                        }
                    });
                },
            );
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Changes apply only with OK.");
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add(
                        egui::Button::new(RichText::new("Cancel").color(Color32::WHITE))
                            .fill(CHIP_BG)
                            .min_size(footer_size(ui, FooterRole::Cancel)),
                    );
                    self.automation
                        .insert("dialog.settings.cancel".into(), cancel.rect);
                    if cancel.clicked() {
                        cancel_clicked = true;
                    }
                    let apply = ui.add(
                        egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong())
                            .fill(CHIP_BG)
                            .min_size(footer_size(ui, FooterRole::Confirm)),
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

    fn overlay_preview_texture(
        &mut self,
        ctx: &egui::Context,
        slot: usize,
        draft: &OverlayState,
    ) -> Result<egui::TextureHandle> {
        if let Some((cached_slot, cached_draft, texture)) = &self.overlay_preview
            && *cached_slot == slot
            && cached_draft == draft
        {
            return Ok(texture.clone());
        }
        let pixels = overlay_draft_pixels(self.engine.project(), slot, draft)?;
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [640, 360],
            &pixels.iter().flatten().copied().collect::<Vec<_>>(),
        );
        let texture = if let Some((_, _, texture)) = &mut self.overlay_preview {
            update_texture_image(texture, image);
            texture.clone()
        } else {
            ctx.load_texture("overlay.draft", image, egui::TextureOptions::LINEAR)
        };
        self.overlay_preview = Some((slot, draft.clone(), texture.clone()));
        Ok(texture)
    }

    pub(crate) fn overlay_settings_dialog(&mut self, ctx: &egui::Context) {
        let Some((mut slot, mut draft)) = self.overlay_settings.clone() else {
            return;
        };
        let mut pending = self.overlay_switch_pending;
        let mut requested_slot = None;
        let mut switch_decision = None;
        let mut open = true;
        let mut apply_clicked = false;
        let mut cancel_clicked = false;
        let mut valid_draft = false;
        let height = (ctx.viewport_rect().height() * 0.68 - 90.0)
            .clamp(200.0, 640.0)
            .min(ctx.viewport_rect().height() - 180.0);
        operator_window(ctx, "Overlay Settings", "dialog.overlay_settings", &mut open,
            Vec2::new((ctx.viewport_rect().width() * 0.62).max(620.0), height + 60.0))
        .show(ctx, |ui| {
            let width = ui.available_width().min((ui.ctx().viewport_rect().width() - 48.0).max(250.0));
            let (body, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
            let mut body_ui = ui.new_child(egui::UiBuilder::new().id_salt("dialog.overlay.single_body")
                .max_rect(body).layout(egui::Layout::top_down(Align::Min)));
            body_ui.set_clip_rect(body.intersect(ui.clip_rect()));
            egui::ScrollArea::vertical().id_salt("dialog.overlay.scroll")
                .max_width(body.width()).max_height(height).auto_shrink([false, false])
                .show(&mut body_ui, |ui| {
                    light_form_frame(ui, 8, |ui| {
                        let label_width = (ui.available_width() * 0.16).clamp(96.0, 156.0);
                        let field_width = (ui.available_width() - label_width - 12.0).max(240.0);
                        egui::Grid::new("overlay.fields").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                            let label = aligned_field_label(ui, "Number", label_width);
                            self.automation.insert("dialog.overlay.label.number".into(), label);
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 3.0;
                                for i in 0..8 {
                                    let row = ui.add_sized([29.0, 26.0], egui::Button::new(format!("{}", i+1)).selected(slot == i));
                                    self.automation.insert(format!("dialog.overlay.slot.{}", i+1), row.rect);
                                    if row.clicked() { requested_slot = Some(i); }
                                }
                            });
                            ui.end_row();
                            let label = aligned_field_label(ui, "Title", label_width);
                            self.automation.insert("dialog.overlay.label.title".into(), label);
                            let title = ui.add_sized([field_width, 28.0], egui::TextEdit::singleline(&mut draft.title).char_limit(96));
                            self.automation.insert("dialog.overlay.title".into(), title.rect);
                            ui.end_row();
                            let label = aligned_field_label(ui, "Subtitle", label_width);
                            self.automation.insert("dialog.overlay.label.subtitle".into(), label);
                            let subtitle = ui.add_sized([field_width, 28.0], egui::TextEdit::singleline(&mut draft.subtitle).char_limit(160));
                            self.automation.insert("dialog.overlay.subtitle".into(), subtitle.rect);
                            ui.end_row();
                            let label = aligned_field_label(ui, "Position", label_width);
                            self.automation.insert("dialog.overlay.label.position".into(), label);
                            ui.horizontal(|ui| {
                                for (position, label) in [(Position::LowerLeft, "Lower left"), (Position::LowerRight, "Lower right"), (Position::TopLeft, "Top left")] {
                                    let response = ui.selectable_value(&mut draft.position, position, label);
                                    self.automation.insert(format!("dialog.overlay.position.{position:?}"), response.rect);
                                }
                            });
                            ui.end_row();
                            let label = aligned_field_label(ui, "Visibility", label_width);
                            self.automation.insert("dialog.overlay.label.visibility".into(), label);
                            let enabled = ui.checkbox(&mut draft.enabled, "Show on Program when applied");
                            self.automation.insert("dialog.overlay.enabled".into(), enabled.rect);
                            ui.end_row();
                        });
                    });
                    let preview = self.overlay_preview_texture(ctx, slot, &draft);
                    valid_draft = preview.is_ok();
                    if let Some(target) = pending {
                        ui.label(RichText::new(format!("Channel {} has unsaved changes. Switch to channel {}?", slot+1, target+1)).color(PREVIEW));
                        ui.horizontal_wrapped(|ui| {
                            let apply = ui.add_enabled(valid_draft, egui::Button::new("Apply and switch"));
                            self.automation.insert("dialog.overlay.switch.apply".into(), apply.rect);
                            if apply.clicked() { switch_decision = Some(OverlaySwitchDecision::Apply); }
                            let discard = ui.button("Discard and switch");
                            self.automation.insert("dialog.overlay.switch.discard".into(), discard.rect);
                            if discard.clicked() { switch_decision = Some(OverlaySwitchDecision::Discard); }
                            let keep = ui.button("Keep editing");
                            self.automation.insert("dialog.overlay.switch.keep".into(), keep.rect);
                            if keep.clicked() { switch_decision = Some(OverlaySwitchDecision::Keep); }
                        });
                    }
                    ui.add_space(8.0);
                    let remaining = (body.bottom() - ui.cursor().top() - 26.0).max(90.0);
                    let pane_width = (ui.available_width() - 12.0) * 0.5;
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        ui.allocate_ui_with_layout(Vec2::new(pane_width, remaining), egui::Layout::top_down(Align::Min), |ui| {
                            ui.set_width(pane_width);
                            ui.strong("Channel capabilities");
                            light_form_frame(ui, 6, |ui| {
                                // Six pixels of margin plus the one-pixel stroke on each side.
                                ui.set_min_width((pane_width - 14.0).max(0.0));
                                egui::Grid::new("overlay.capabilities").num_columns(2).spacing([8.0, 3.0]).show(ui, |ui| {
                                    for (id, label, value, reason) in [
                                        ("type", "Type", "Title panel only", "Only title-panel composition is implemented."),
                                        ("source", "Source overlay", "Unavailable", "Input-as-overlay composition is not implemented."),
                                        ("effect", "Effect / timing", "Unavailable", "Overlay transitions and duration controls are not implemented."),
                                        ("alpha", "Alpha", "Unavailable", "Independent channel alpha is not implemented."),
                                        ("transform", "Transform / border", "Unavailable", "Free transforms and borders are not implemented."),
                                    ] {
                                        ui.label(label);
                                        let response = ui.label(RichText::new(value).color(Color32::from_gray(90))).on_hover_text(reason);
                                        self.automation.insert(format!("dialog.overlay.capability.{id}"), response.rect);
                                        ui.end_row();
                                    }
                                });
                            });
                            ui.label(RichText::new("Isolated preview · ASCII captions, 96 chars/line.").small().color(DIM));
                        });
                        ui.allocate_ui_with_layout(Vec2::new(pane_width, remaining), egui::Layout::top_down(Align::Min), |ui| {
                            ui.set_width(pane_width);
                            ui.strong(if draft.enabled { "Preview · visible when applied" } else { "Preview · hidden" });
                            let picture_width = pane_width.min(remaining * 16.0 / 9.0);
                            let size = Vec2::new(picture_width, picture_width * 9.0 / 16.0);
                            match preview {
                                Ok(texture) => {
                                    let response = ui.add(egui::Image::new((texture.id(), size)));
                                    self.automation.insert("dialog.overlay.preview".into(), response.rect);
                                }
                                Err(error) => { ui.colored_label(RED, error.to_string()); }
                            }
                        });
                    });
                });
            self.automation.insert("dialog.overlay.body".into(), body);
            ui.separator();
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add_sized(footer_size(ui, FooterRole::Cancel), egui::Button::new(RichText::new("Cancel").color(Color32::WHITE)).fill(CHIP_BG));
                    self.automation.insert("dialog.overlay.cancel".into(), cancel.rect);
                    cancel_clicked = cancel.clicked();
                    let apply = ui.add_enabled(valid_draft && pending.is_none(), egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong()).fill(CHIP_BG).min_size(footer_size(ui, FooterRole::Confirm)));
                    self.automation.insert("dialog.overlay.apply".into(), apply.rect);
                    apply_clicked = apply.clicked();
                });
            });
        });
        if cancel_clicked || !open {
            self.overlay_settings = None;
            self.overlay_switch_pending = None;
            self.overlay_preview = None;
        } else if apply_clicked {
            self.actions
                .push(Action::Engine(Command::SetOverlay(slot, draft)));
            self.overlay_settings = None;
            self.overlay_switch_pending = None;
            self.overlay_preview = None;
        } else {
            if let Some(target) = requested_slot.filter(|target| *target != slot)
                && self.engine.project().overlays.standard(target).is_some()
            {
                if self.engine.project().overlays.standard(slot) == Some(&draft) {
                    if let Some(committed) =
                        self.engine.project().overlays.standard(target).cloned()
                    {
                        slot = target;
                        draft = committed;
                        pending = None;
                    }
                } else {
                    pending = Some(target);
                }
            }
            if let Some(decision) = switch_decision {
                if decision == OverlaySwitchDecision::Keep {
                    pending = None;
                } else if let Some(target) = pending
                    && let Some(committed) =
                        self.engine.project().overlays.standard(target).cloned()
                {
                    if decision == OverlaySwitchDecision::Apply {
                        self.actions
                            .push(Action::Engine(Command::SetOverlay(slot, draft.clone())));
                    }
                    slot = target;
                    draft = committed;
                    pending = None;
                }
            }
            self.overlay_switch_pending = pending;
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
                        egui::Button::new(RichText::new("Cancel").color(Color32::WHITE))
                            .fill(CHIP_BG)
                            .min_size(footer_size(ui, FooterRole::Cancel)),
                    );
                    self.automation
                        .insert("dialog.external.cancel".into(), cancel.rect);
                    if cancel.clicked() {
                        cancel_clicked = true;
                    }
                    let apply = ui.add(
                        egui::Button::new(RichText::new("Apply selected output").color(Color32::WHITE).strong())
                            .fill(CHIP_BG)
                            .min_size(footer_size(ui, FooterRole::Confirm)),
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
        let viewport = ctx.viewport_rect().size();
        let body_height = (viewport.y * 0.53 - 80.0).clamp(200.0, 460.0);
        let program_rate = self.engine.project().preferences.rate;
        operator_window(
            ctx,
            "Recording Setup",
            "dialog.recording_setup",
            &mut open,
            Vec2::new(
                (viewport.x * 0.405).clamp(560.0, 800.0),
                body_height + 125.0,
            ),
        )
        .show(ctx, |ui| {
            ui.label("No recording backend. Configuration is unavailable and is not saved.");
            ui.horizontal(|ui| {
                for slot in [1_u8, 2_u8] {
                    let response = ui.add_sized(
                        [42.0, 28.0],
                        egui::Button::new(slot.to_string()).selected(self.rec_slot == slot),
                    );
                    self.automation
                        .insert(format!("dialog.recording.slot.{slot}"), response.rect);
                    if response.clicked() {
                        self.rec_slot = slot;
                    }
                }
                ui.add_space(8.0);
                let output =
                    ui.add_enabled(false, egui::Button::new("Program output · unavailable"));
                self.automation
                    .insert("dialog.recording.output".into(), output.rect);
            });
            let width = ui.available_width();
            let (body, _) = ui.allocate_exact_size(Vec2::new(width, body_height), Sense::hover());
            self.automation.insert("dialog.recording.body".into(), body);
            let rail_width = (width * 0.26).clamp(130.0, 210.0);
            let rail = Rect::from_min_size(body.min, Vec2::new(rail_width, body.height()));
            let page = Rect::from_min_max(egui::pos2(rail.right() + 6.0, body.top()), body.max);
            self.automation.insert("dialog.recording.rail".into(), rail);
            self.automation.insert("dialog.recording.page".into(), page);
            let mut rail_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("recording.rail")
                    .max_rect(rail)
                    .layout(egui::Layout::top_down(Align::Min)),
            );
            rail_ui.set_clip_rect(rail.intersect(ui.clip_rect()));
            rail_ui.painter().rect_filled(rail, 0.0, SIDEBAR);
            for (index, label) in [
                (1, "AVI"),
                (4, "WMV"),
                (0, "MP4"),
                (2, "MKV"),
                (3, "MPEG-2"),
            ] {
                let response =
                    sidebar_row(&mut rail_ui, self.rec_format == index, label, 30.0, None);
                self.automation
                    .insert(format!("dialog.recording.format.{index}"), response.rect);
                if response.clicked() {
                    self.rec_format = index;
                }
            }
            let format = ["MP4", "AVI", "MKV", "MPEG-2", "WMV"]
                .get(self.rec_format)
                .copied()
                .unwrap_or("MKV");
            let mut page_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("recording.page")
                    .max_rect(page)
                    .layout(egui::Layout::top_down(Align::Min)),
            );
            page_ui.set_clip_rect(page.intersect(ui.clip_rect()));
            page_ui.painter().rect_filled(page, 0.0, FORM_LIGHT);
            *page_ui.visuals_mut() = form_visuals();
            egui::ScrollArea::vertical()
                .id_salt("recording.page.scroll")
                .max_height(page.height())
                .auto_shrink([false, false])
                .show(&mut page_ui, |ui| {
                    ui.add_enabled_ui(false, |ui| {
                        light_form_frame(ui, 8, |ui| {
                            ui.strong(format!("{format} encoder settings · unavailable"));
                            let filename = recording_form_row(ui, "Filename", |ui| {
                                ui.add_sized(
                                    [ui.available_width(), 22.0],
                                    egui::TextEdit::singleline(&mut self.rec_filename),
                                )
                            })
                            .inner;
                            self.automation
                                .insert("dialog.recording.filename".into(), filename.rect);
                            recording_form_row(ui, "", |ui| {
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    let browse =
                                        ui.add_sized([96.0, 26.0], egui::Button::new("Browse"));
                                    self.automation
                                        .insert("dialog.recording.browse".into(), browse.rect);
                                });
                            });
                            ui.add_space(6.0);
                            let video = ui.vertical(|ui| {
                                let raster = recording_form_row(ui, "Size", |ui| {
                                    ui.label("640×360 CPU reference")
                                })
                                .inner;
                                self.automation
                                    .insert("dialog.recording.programRaster".into(), raster.rect);
                                let rate = recording_form_row(ui, "Frame Rate", |ui| {
                                    ui.label(recording_rate_description(program_rate))
                                })
                                .inner;
                                self.automation
                                    .insert("dialog.recording.programRate".into(), rate.rect);
                                let codec = if self.rec_format == 0 {
                                    "H.264 profile · unavailable"
                                } else if self.rec_format == 2 {
                                    "Container codec · unavailable"
                                } else {
                                    "Codec · unavailable"
                                };
                                recording_form_row(ui, "Codec", |ui| ui.label(codec));
                                recording_form_row(ui, "Bit Rate", |ui| {
                                    ui.add_sized(
                                        [70.0, 22.0],
                                        egui::TextEdit::singleline(&mut self.rec_bitrate),
                                    );
                                    ui.label("Mbps");
                                });
                                if self.rec_format == 0 || self.rec_format == 2 {
                                    recording_form_row(ui, "Hardware", |ui| {
                                        ui.checkbox(
                                            &mut self.rec_hw_encoder,
                                            "Use hardware encoder",
                                        );
                                    });
                                }
                                if self.rec_format == 2 {
                                    recording_form_row(ui, "Recovery", |ui| {
                                        ui.checkbox(&mut self.rec_fault_tolerant, "Fault tolerant");
                                    });
                                }
                            });
                            self.automation
                                .insert("dialog.recording.video".into(), video.response.rect);
                            ui.add_space(10.0);
                            ui.separator();
                            let audio = ui.vertical(|ui| {
                                recording_form_row(ui, "Audio", |ui| {
                                    ui.checkbox(&mut self.rec_audio_master, "Master · unavailable");
                                });
                                recording_form_row(ui, "Audio Delay", |ui| {
                                    ui.add_sized(
                                        [70.0, 22.0],
                                        egui::TextEdit::singleline(&mut self.rec_audio_delay),
                                    );
                                    ui.label("ms");
                                });
                                recording_form_row(ui, "Audio Bit Rate", |ui| {
                                    ui.add_sized(
                                        [70.0, 22.0],
                                        egui::TextEdit::singleline(&mut self.rec_audio_kbps),
                                    );
                                    ui.label("Kbps");
                                });
                            });
                            self.automation
                                .insert("dialog.recording.audio".into(), audio.response.rect);
                            ui.add_space(8.0);
                            recording_form_row(ui, "New File Every", |ui| {
                                ui.label(format!("{} · unavailable", self.rec_split_mins))
                            });
                        });
                    });
                });
            ui.separator();
            if self.rec_active {
                ui.label(format!(
                    "Recording state: {} frames · {} bytes",
                    self.rec_frames, self.rec_bytes
                ));
            }
            ui.horizontal(|ui| {
                let start = ui
                    .add_enabled(
                        false,
                        egui::Button::new("Start Recording").min_size(Vec2::new(124.0, 29.0)),
                    )
                    .on_disabled_hover_text("No encoder/output backend; recording cannot start");
                self.automation
                    .insert("dialog.recording.start".into(), start.rect);
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add_sized(
                        footer_size(ui, FooterRole::Cancel),
                        egui::Button::new(RichText::new("Cancel").color(Color32::WHITE))
                            .fill(CHIP_BG),
                    );
                    self.automation
                        .insert("dialog.recording.cancel".into(), cancel.rect);
                    if cancel.clicked() {
                        close_clicked = true;
                    }
                    let ok = ui.add_sized(
                        footer_size(ui, FooterRole::Confirm),
                        egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong())
                            .fill(PROGRAM),
                    );
                    self.automation
                        .insert("dialog.recording.ok".into(), ok.rect);
                    if ok.clicked() {
                        self.notice =
                            "Recording backend unavailable; no settings were saved".into();
                        self.warning = false;
                        close_clicked = true;
                    }
                });
            });
        });
        if close_clicked || !open {
            self.recording_setup = false;
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
            Vec2::new((ctx.viewport_rect().width() * 0.616).max(620.0), 260.0),
        )
        .min_size(Vec2::new(560.0, 220.0))
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
                ("Pause inputs\n[Transport]".into(), Action::PauseDemo),
                ("Program monitor\n[Window]".into(), Action::ProgramOutputOpen),
            ];
            let columns = ((ui.available_width() + 6.0) / 104.0).floor().clamp(3.0, 9.0) as usize;
            let tile_width = (ui.available_width() - (columns.saturating_sub(1) as f32 * 6.0)) / columns as f32;
            egui::ScrollArea::vertical()
                .id_salt("shortcuts.bank.scroll")
                .max_height((ctx.viewport_rect().height() - 190.0).clamp(120.0, 250.0))
                .show(ui, |ui| {
            egui::Grid::new("shortcuts_button_grid")
                .num_columns(columns)
                .spacing([6.0, 6.0])
                .show(ui, |ui| {
                    for (idx, (label, action)) in tiles.into_iter().enumerate() {
                        let enabled = match &action {
                            Action::Unavailable(_) => false,
                            Action::Engine(command) => self.engine.enabled(command).is_ok(),
                            _ => true,
                        };
                        let button = ui.add_enabled_ui(enabled, |ui| {
                            ui.add_sized([tile_width, 74.0], egui::Button::new(
                                RichText::new(label).size(13.0).color(Color32::WHITE),
                            ).fill(CHIP_BG).wrap())
                        }).inner.on_disabled_hover_text("Unavailable: finish the active transition or choose an implemented action");
                        self.automation.insert(format!("dialog.shortcuts.action.{idx}"), button.rect);
                        if button.clicked()
                        {
                            self.actions.push(action);
                        }
                        if (idx + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                });
            });
            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                if ui
                    .add_sized([78.0, 29.0], egui::Button::new(RichText::new("Add").color(Color32::WHITE)).fill(CHIP_BG))
                    .clicked()
                {
                    self.shortcuts_open = false;
                    self.actions.push(Action::SettingsAt(SettingsPage::Shortcuts));
                }
                if ui
                    .add_sized([84.0, 29.0], egui::Button::new(RichText::new("Settings").color(Color32::WHITE)).fill(CHIP_BG))
                    .clicked()
                {
                    self.shortcuts_open = false;
                    self.actions.push(Action::SettingsAt(SettingsPage::Shortcuts));
                }
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let close = ui.add(
                        egui::Button::new(RichText::new("Close").color(Color32::WHITE))
                            .fill(CHIP_BG)
                            .min_size(footer_size(ui, FooterRole::Confirm)),
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

    fn click_at_size(app: &mut RustMixApp, ctx: &egui::Context, id: &str, size: Vec2) {
        for _ in 0..4 {
            frame_at_size(app, ctx, vec![], size);
        }
        let rect = *app.automation.get(id).expect("visible settings widget");
        assert!(ctx.viewport_rect().contains_rect(rect));
        for pressed in [true, false] {
            frame_at_size(
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
                size,
            );
        }
    }

    #[test]
    fn overlay_label_glyphs_end_at_the_reserved_columns_right_edge() {
        for width in [96.0, 144.0] {
            for label in ["Title", "Subtitle", "Visibility"] {
                let ctx = egui::Context::default();
                let mut reserved = Rect::NOTHING;
                let output = ctx.run_ui(egui::RawInput::default(), |ui| {
                    reserved = aligned_field_label(ui, label, width);
                });
                let text = output
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::epaint::Shape::Text(text) if text.galley.job.text == label => {
                            Some(text)
                        }
                        _ => None,
                    })
                    .expect("actual painted label text");
                let painted = text.visual_bounding_rect();
                assert!(
                    (painted.right() - reserved.right()).abs() <= 2.0,
                    "Label glyphs must end at the reserved column: {painted:?} / {reserved:?}"
                );
                assert!(reserved.expand(2.0).contains_rect(painted));
            }
        }
    }

    #[test]
    fn settings_proportions_navigation_actions_and_real_cancel_fit_production_sizes() {
        for size in [
            Vec2::new(1100.0, 700.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(1920.0, 1080.0),
        ] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            app.dispatch(Action::Settings, &ctx);
            for _ in 0..4 {
                frame_at_size(&mut app, &ctx, vec![], size);
            }
            let before = app.engine.project().clone();
            let body = *app.automation.get("dialog.settings.body").unwrap();
            let nav = *app.automation.get("dialog.settings.navigation").unwrap();
            let page = *app.automation.get("dialog.settings.page").unwrap();
            assert!((body.width() / size.x - 0.60).abs() < 0.025);
            assert!((nav.width() / body.width() - 0.218).abs() < 0.015);
            let display = *app.automation.get("dialog.settings.page.Display").unwrap();
            let next = *app.automation.get("dialog.settings.page.Options").unwrap();
            assert!((display.height() - 27.0).abs() <= 1.0);
            assert!((next.top() - display.top() - 28.0).abs() <= 1.0);
            for id in [
                "dialog.settings.navigation.footer",
                "dialog.settings.navigation.Default",
            ] {
                let rect = *app.automation.get(id).unwrap();
                assert!(
                    nav.contains_rect(rect),
                    "Navigation footer escaped at {size:?}: {id}"
                );
            }
            let expected = (size.x * (300.0 / 1600.0)).clamp(140.0, 360.0);
            for id in [
                "dialog.settings.frameRate",
                "dialog.settings.projectName",
                "dialog.settings.programWindow",
            ] {
                let rect = *app.automation.get(id).unwrap();
                assert!(
                    page.contains_rect(rect),
                    "Settings field escaped at {size:?}: {id}"
                );
                if id == "dialog.settings.frameRate" || id == "dialog.settings.projectName" {
                    assert!((rect.width() - expected).abs() < 12.0);
                }
            }
            click_at_size(&mut app, &ctx, "dialog.settings.page.Options", size);
            assert!(app.settings_page == SettingsPage::Options);
            click_at_size(&mut app, &ctx, "dialog.settings.page.Display", size);
            click_at_size(&mut app, &ctx, "dialog.settings.projectName", size);
            frame_at_size(
                &mut app,
                &ctx,
                vec![egui::Event::Text("UI022".into())],
                size,
            );
            assert!(app.settings_name.contains("UI022"));
            assert_eq!(*app.engine.project(), before);
            click_at_size(&mut app, &ctx, "dialog.settings.cancel", size);
            assert!(!app.settings && app.settings_draft.is_none());
            assert_eq!(*app.engine.project(), before);
        }
    }

    #[test]
    fn settings_compact_footer_stays_reachable_and_apply_uses_staged_values() {
        for size in [Vec2::new(734.0, 466.0), Vec2::new(1600.0, 900.0)] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            app.dispatch(Action::Settings, &ctx);
            for _ in 0..4 {
                frame_at_size(&mut app, &ctx, vec![], size);
            }
            let body = *app.automation.get("dialog.settings.body").unwrap();
            let apply = *app.automation.get("dialog.settings.apply").unwrap();
            let cancel = *app.automation.get("dialog.settings.cancel").unwrap();
            assert!(body.bottom() <= apply.top() && body.bottom() <= cancel.top());
            assert!(apply.right() <= cancel.left());
            assert!(ctx.viewport_rect().contains_rect(apply));
            assert!(ctx.viewport_rect().contains_rect(cancel));
            let program = app.engine.project().program;
            let preview = app.engine.project().preview;
            app.settings_name = "UI022 staged settings".into();
            click_at_size(&mut app, &ctx, "dialog.settings.apply", size);
            frame_at_size(&mut app, &ctx, vec![], size);
            assert_eq!(app.engine.project().name, "UI022 staged settings");
            assert_eq!(app.engine.project().program, program);
            assert_eq!(app.engine.project().preview, preview);
        }
    }

    #[test]
    fn dirty_overlay_channel_switch_requires_an_explicit_choice() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let before = app.engine.project().clone();
        app.dispatch(Action::OverlaySettings(0), &ctx);
        click(&mut app, &ctx, "dialog.overlay.position.TopLeft");
        click(&mut app, &ctx, "dialog.overlay.slot.2");
        assert_eq!(
            app.overlay_settings
                .as_ref()
                .map(|(slot, draft)| (*slot, draft.position)),
            Some((0, Position::TopLeft))
        );
        assert_eq!(app.overlay_switch_pending, Some(1));
        assert_eq!(*app.engine.project(), before);
        click(&mut app, &ctx, "dialog.overlay.switch.keep");
        assert_eq!(app.overlay_switch_pending, None);
        assert_eq!(
            app.overlay_settings
                .as_ref()
                .map(|(_, draft)| draft.position),
            Some(Position::TopLeft)
        );
        click(&mut app, &ctx, "dialog.overlay.slot.2");
        click(&mut app, &ctx, "dialog.overlay.switch.discard");
        assert_eq!(
            app.overlay_settings.as_ref().map(|(slot, _)| *slot),
            Some(1)
        );
        assert_eq!(*app.engine.project(), before);
        click(&mut app, &ctx, "dialog.overlay.position.LowerRight");
        click(&mut app, &ctx, "dialog.overlay.slot.3");
        click(&mut app, &ctx, "dialog.overlay.switch.apply");
        assert_eq!(
            app.overlay_settings.as_ref().map(|(slot, _)| *slot),
            Some(2)
        );
        assert_eq!(
            app.engine.project().overlays.standard(1).unwrap().position,
            Position::LowerRight
        );
        assert_eq!(
            app.engine.project().overlays.standard(0),
            before.overlays.standard(0)
        );
        assert_eq!(app.engine.project().program, before.program);
        assert_eq!(app.engine.project().preview, before.preview);
        let applied = app.engine.project().clone();
        click(&mut app, &ctx, "dialog.overlay.enabled");
        click(&mut app, &ctx, "dialog.overlay.slot.4");
        click(&mut app, &ctx, "dialog.overlay.cancel");
        assert_eq!(*app.engine.project(), applied);
        assert!(
            app.overlay_settings.is_none()
                && app.overlay_switch_pending.is_none()
                && app.overlay_preview.is_none()
        );
    }

    #[test]
    fn overlay_draft_preview_uses_both_caption_lines_and_hidden_state_without_live_mutation() {
        let app = RustMixApp::headless();
        let before = app.engine.project().clone();
        let mut draft = before.overlays.standard(0).unwrap().clone();
        draft.enabled = true;
        draft.title = "TITLE".into();
        draft.subtitle = "SUBTITLE".into();
        let both = overlay_draft_pixels(&before, 0, &draft).unwrap();
        let mut other_on_air = before.clone();
        let other = other_on_air.overlays.standard_mut(1).unwrap();
        other.enabled = true;
        other.title = "OTHER CHANNEL".into();
        assert_eq!(
            overlay_draft_pixels(&other_on_air, 0, &draft).unwrap(),
            both,
            "the selected draft preview must not include another on-air channel"
        );
        assert!(
            both.chunks_exact(640)
                .skip(282)
                .take(17)
                .flatten()
                .any(|pixel| *pixel == [255, 255, 255, 255]),
            "the title has visible raster glyphs"
        );
        assert!(
            both.chunks_exact(640)
                .skip(299)
                .take(12)
                .flatten()
                .any(|pixel| *pixel == [190, 190, 190, 255]),
            "the subtitle has visible raster glyphs"
        );
        draft.subtitle.clear();
        let title_only = overlay_draft_pixels(&before, 0, &draft).unwrap();
        assert_ne!(both, title_only);
        assert_eq!(
            both.chunks_exact(640)
                .skip(282)
                .take(17)
                .collect::<Vec<_>>(),
            title_only
                .chunks_exact(640)
                .skip(282)
                .take(17)
                .collect::<Vec<_>>()
        );
        draft.enabled = false;
        let hidden = overlay_draft_pixels(&before, 0, &draft).unwrap();
        assert!(hidden.iter().all(|pixel| *pixel == [18, 23, 30, 255]));
        assert_eq!(*app.engine.project(), before);
        draft.title = "TOO LONG".repeat(20);
        assert!(overlay_draft_pixels(&before, 0, &draft).is_err());
        assert!(overlay_draft_pixels(&before, 8, &draft).is_err());
    }

    #[test]
    fn overlay_editor_keeps_preview_fields_and_footer_bounded_on_operator_displays() {
        for size in [
            Vec2::new(1100.0, 700.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(1920.0, 1080.0),
        ] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            let before = app.engine.project().clone();
            app.dispatch(Action::OverlaySettings(0), &ctx);
            if let Some((_, draft)) = &mut app.overlay_settings {
                draft.enabled = true;
                draft.title = "A".repeat(96);
                draft.subtitle = "B".repeat(160);
            }
            for _ in 0..5 {
                frame_at_size(&mut app, &ctx, vec![], size);
            }
            let body = app.automation.get("dialog.overlay.body").unwrap();
            assert!(body.width() >= size.x * 0.60 && body.width() <= size.x * 0.64);
            let preview = app.automation.get("dialog.overlay.preview").unwrap();
            assert!(
                body.contains_rect(*preview),
                "bounded draft preview at {size:?}: {preview:?}, {body:?}"
            );
            assert!((preview.width() / preview.height() - 16.0 / 9.0).abs() < 0.01);
            for id in [
                "dialog.overlay.title",
                "dialog.overlay.subtitle",
                "dialog.overlay.enabled",
                "dialog.overlay.apply",
                "dialog.overlay.cancel",
            ] {
                let rect = app.automation.get(id).unwrap();
                assert!(
                    ctx.viewport_rect().contains_rect(*rect),
                    "visible {id} at {size:?}"
                );
            }
            let title = app.automation.get("dialog.overlay.title").unwrap();
            let subtitle = app.automation.get("dialog.overlay.subtitle").unwrap();
            assert!((title.left() - subtitle.left()).abs() < 0.5);
            assert!(title.width() >= body.width() * 0.70);
            for (label, field) in [
                ("number", "slot.1"),
                ("title", "title"),
                ("subtitle", "subtitle"),
                ("position", "position.LowerLeft"),
                ("visibility", "enabled"),
            ] {
                let label = app
                    .automation
                    .get(&format!("dialog.overlay.label.{label}"))
                    .unwrap();
                let field = app
                    .automation
                    .get(&format!("dialog.overlay.{field}"))
                    .unwrap();
                assert!(label.width() >= 96.0);
                assert!(
                    label.right() + 6.0 <= field.left(),
                    "label overlaps field at {size:?}: {label:?}, {field:?}"
                );
                assert!(body.contains_rect(*label));
            }
            assert_eq!(*app.engine.project(), before);
        }
    }

    #[test]
    fn invalid_overlay_draft_stays_open_and_cancel_keeps_live_state() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let before = app.engine.project().clone();
        app.dispatch(Action::OverlaySettings(0), &ctx);
        if let Some((_, draft)) = &mut app.overlay_settings {
            // The widget limit is in characters; the engine validates bytes.
            draft.title = "é".repeat(96);
        }
        click(&mut app, &ctx, "dialog.overlay.apply");
        assert!(app.overlay_settings.is_some());
        assert_eq!(*app.engine.project(), before);
        click(&mut app, &ctx, "dialog.overlay.cancel");
        assert!(app.overlay_settings.is_none());
        assert_eq!(*app.engine.project(), before);
    }

    #[test]
    fn overlay_preview_uploads_only_for_changed_drafts() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        let mut draft = app.engine.project().overlays.standard(0).unwrap().clone();
        let mut handle = None;
        let first = ctx.run_ui(egui::RawInput::default(), |_| {
            handle = Some(app.overlay_preview_texture(&ctx, 0, &draft).unwrap());
        });
        let id = handle.unwrap().id();
        assert!(
            first
                .textures_delta
                .set
                .iter()
                .any(|(key, delta)| *key == id && delta.pos.is_none())
        );
        let unchanged = ctx.run_ui(egui::RawInput::default(), |_| {
            assert_eq!(
                app.overlay_preview_texture(&ctx, 0, &draft).unwrap().id(),
                id
            );
        });
        assert!(
            unchanged
                .textures_delta
                .set
                .iter()
                .all(|(key, _)| *key != id)
        );
        draft.enabled = !draft.enabled;
        let changed = ctx.run_ui(egui::RawInput::default(), |_| {
            assert_eq!(
                app.overlay_preview_texture(&ctx, 0, &draft).unwrap().id(),
                id
            );
        });
        assert!(
            changed
                .textures_delta
                .set
                .iter()
                .any(|(key, delta)| *key == id && delta.pos == Some([0, 0]))
        );
    }

    #[test]
    fn unavailable_form_paints_an_opaque_background_without_enabling_controls() {
        let ctx = egui::Context::default();
        let mut form_rect = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.add_enabled_ui(false, |ui| {
                ui.set_opacity(0.35);
                let form = light_form_frame(ui, 6, |ui| {
                    assert!(!ui.is_enabled());
                    assert!(!ui.button("Unavailable action").enabled());
                });
                form_rect = Some(form.response.rect);
                assert!(
                    (ui.opacity() - 0.35).abs() < f32::EPSILON,
                    "restore the caller's opacity"
                );
                assert!(!ui.is_enabled());
            });
        });
        let form_rect = form_rect.unwrap();
        assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::Shape::Rect(rect) if rect.fill == FORM_LIGHT && rect.rect.contains_rect(form_rect)
        )), "the form background must not inherit the disabled parent's fade");
    }

    #[test]
    fn production_dialog_banks_and_table_fields_stay_bounded_across_viewports() {
        for size in [
            Vec2::new(734.0, 466.0),
            Vec2::new(1100.0, 700.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(1920.0, 1080.0),
        ] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            app.dispatch(Action::ShortcutsWindow, &ctx);
            for _ in 0..4 {
                frame_at_size(&mut app, &ctx, vec![], size);
            }
            let close = app.automation.get("dialog.shortcuts.close").unwrap();
            assert!(ctx.viewport_rect().contains_rect(*close));
            for index in 0..12 {
                let rect = app
                    .automation
                    .get(&format!("dialog.shortcuts.action.{index}"))
                    .unwrap();
                assert!(ctx.viewport_rect().contains_rect(*rect));
                assert!(rect.bottom() <= close.top());
            }
            if size.x >= 1600.0 {
                let first = app.automation.get("dialog.shortcuts.action.0").unwrap();
                let ninth = app.automation.get("dialog.shortcuts.action.8").unwrap();
                let tenth = app.automation.get("dialog.shortcuts.action.9").unwrap();
                assert!(
                    (first.top() - ninth.top()).abs() < 1.0,
                    "wide bank exposes nine actions per row"
                );
                assert!(tenth.top() > ninth.bottom());
            }
            app.shortcuts_open = false;
            app.dispatch(Action::MultiCorder, &ctx);
            for _ in 0..4 {
                frame_at_size(&mut app, &ctx, vec![], size);
            }
            let name = app.automation.get("dialog.multicorder.input.0").unwrap();
            let folder = app.automation.get("dialog.multicorder.folder.0").unwrap();
            let duration = app.automation.get("dialog.multicorder.duration.0").unwrap();
            assert!(name.right() + 8.0 <= folder.left());
            assert!(folder.right() + 8.0 <= duration.left());
            assert!(ctx.viewport_rect().contains_rect(*duration));
            assert_eq!(app.engine.project().program, InputId(1));
        }
    }

    #[test]
    fn playlist_lists_have_separate_bodies_and_a_full_size_exit() {
        for size in [Vec2::new(734.0, 466.0), Vec2::new(1600.0, 900.0)] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            app.playlist_open = true;
            for _ in 0..4 {
                frame_at_size(&mut app, &ctx, vec![], size);
            }
            let available = app
                .automation
                .get("dialog.playlist.available.body")
                .unwrap();
            let ordered = app.automation.get("dialog.playlist.items.body").unwrap();
            let close = app.automation.get("dialog.playlist.close").unwrap();
            assert!(available.right() <= ordered.left());
            assert!(available.height() >= 90.0 && ordered.height() >= 90.0);
            assert!(close.width() >= 92.0 && close.height() >= 29.0);
            assert!(ctx.viewport_rect().contains_rect(*close));
            assert_eq!(app.engine.project().program, InputId(1));
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
                    if prefix == "dialog.properties" {
                        let preview = *app.automation.get("dialog.properties.preview").unwrap();
                        assert!(
                            body.contains_rect(preview),
                            "Default source preview is clipped"
                        );
                    } else {
                        let number = *app.automation.get("dialog.add.number").unwrap();
                        let confirm = *app.automation.get("dialog.add.confirm").unwrap();
                        assert!(confirm.left() - number.right() < 24.0);
                    }
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
        click(&mut app, &ctx, "dialog.add.kind.Grid");
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

#[cfg(test)]
mod recording_layout_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn frame(
        app: &mut RustMixApp,
        ctx: &egui::Context,
        size: Vec2,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |_| app.recording_setup_dialog(ctx),
        )
    }

    fn click(app: &mut RustMixApp, ctx: &egui::Context, size: Vec2, id: &str) {
        for _ in 0..4 {
            let _ = frame(app, ctx, size, vec![]);
        }
        let rect = *app.automation.get(id).expect("recording widget bounds");
        assert!(ctx.viewport_rect().contains_rect(rect));
        for pressed in [true, false] {
            let _ = frame(
                app,
                ctx,
                size,
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

    fn text(output: &egui::FullOutput) -> String {
        output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn recording_navigation_changes_only_local_presentation_and_cannot_arm_output() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.recording_setup = true;
        let before = app.engine.project().clone();
        let filename = app.rec_filename.clone();
        let size = Vec2::new(1600.0, 900.0);
        for (index, label) in [
            (0, "MP4"),
            (1, "AVI"),
            (2, "MKV"),
            (3, "MPEG-2"),
            (4, "WMV"),
        ] {
            click(
                &mut app,
                &ctx,
                size,
                &format!("dialog.recording.format.{index}"),
            );
            assert_eq!(app.rec_format, index);
            let output = frame(&mut app, &ctx, size, vec![]);
            let painted = text(&output);
            assert!(painted.contains(&format!("{label} encoder settings · unavailable")));
            assert_eq!(painted.matches("encoder settings · unavailable").count(), 1);
        }
        click(&mut app, &ctx, size, "dialog.recording.slot.2");
        assert_eq!(app.rec_slot, 2);
        click(&mut app, &ctx, size, "dialog.recording.filename");
        let _ = frame(
            &mut app,
            &ctx,
            size,
            vec![egui::Event::Text("must not edit".into())],
        );
        click(&mut app, &ctx, size, "dialog.recording.browse");
        click(&mut app, &ctx, size, "dialog.recording.start");
        assert_eq!(app.rec_filename, filename);
        assert!(!app.rec_active);
        assert_eq!(app.rec_frames, 0);
        assert_eq!(app.rec_bytes, 0);
        assert_eq!(*app.engine.project(), before);
        assert!(app.actions.is_empty());
        click(&mut app, &ctx, size, "dialog.recording.ok");
        assert!(!app.recording_setup);
        assert_eq!(*app.engine.project(), before);
        assert!(app.actions.is_empty());
    }

    #[test]
    fn recording_truthful_rate_raster_and_unavailable_encoder_are_painted() {
        for (numerator, denominator) in [(30, 1), (30000, 1001), (50, 1)] {
            let mut app = RustMixApp::headless();
            let rate = Rate::new(numerator, denominator).unwrap();
            let mut preferences = app.engine.project().preferences.clone();
            preferences.rate = rate;
            app.engine
                .execute(Command::SetPreferences(preferences), 0)
                .unwrap();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            app.recording_setup = true;
            let size = Vec2::new(1600.0, 900.0);
            for _ in 0..4 {
                let _ = frame(&mut app, &ctx, size, vec![]);
            }
            let output = frame(&mut app, &ctx, size, vec![]);
            let painted = text(&output);
            assert!(painted.contains("640×360 CPU reference"));
            assert!(painted.contains(&format!("{}p ({numerator}/{denominator})", rate.label())));
            assert!(painted.contains("No recording backend"));
            assert!(!painted.contains("NTSC 29.97p"));
        }
    }

    #[test]
    fn recording_rail_page_and_footer_stay_contained_with_vertical_browse() {
        for size in [
            Vec2::new(734.0, 466.0),
            Vec2::new(1100.0, 700.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(1920.0, 1200.0),
        ] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            app.recording_setup = true;
            for _ in 0..4 {
                let _ = frame(&mut app, &ctx, size, vec![]);
            }
            let bounds = |id: &str| *app.automation.get(id).expect("recording geometry");
            let body = bounds("dialog.recording.body");
            let rail = bounds("dialog.recording.rail");
            let page = bounds("dialog.recording.page");
            assert!((rail.width() / body.width() - 0.26).abs() < 0.015);
            assert!(body.contains_rect(rail) && body.contains_rect(page));
            assert!(rail.right() + 5.0 <= page.left());
            let filename = bounds("dialog.recording.filename");
            let browse = bounds("dialog.recording.browse");
            assert!(page.contains_rect(filename));
            assert!(page.contains_rect(browse));
            assert!(filename.bottom() <= browse.top());
            let ok = bounds("dialog.recording.ok");
            let cancel = bounds("dialog.recording.cancel");
            let start = bounds("dialog.recording.start");
            for rect in [body, ok, cancel, start] {
                assert!(ctx.viewport_rect().contains_rect(rect));
            }
            assert!(body.bottom() <= ok.top() && body.bottom() <= start.top());
            assert!(start.right() <= ok.left() && ok.right() <= cancel.left());
            click(&mut app, &ctx, size, "dialog.recording.format.0");
            click(&mut app, &ctx, size, "dialog.recording.cancel");
            assert!(!app.recording_setup);
        }
    }
}
