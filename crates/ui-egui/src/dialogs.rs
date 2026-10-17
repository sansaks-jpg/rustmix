//! Operator dialogs modeled after the reference workflows in `rustmix-reference`.
use super::*;
use egui::{Align, Color32, RichText, Sense, Stroke, Vec2};

const SIDEBAR: Color32 = Color32::from_rgb(16, 20, 23);
const HEADER_DARK: Color32 = Color32::from_rgb(24, 29, 34);
const FORM_BORDER: Color32 = Color32::from_rgb(184, 190, 198);

fn operator_window<'a>(
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

/// Applies the light form surface used inside reference dialogs (`Add Input`, `Input Settings`,
/// `Settings`, `Recording Setup`, `Streaming Settings`, `Overlay Settings`, `PlayList`, `MultiCorder`).
fn light_form_frame<R>(
    ui: &mut egui::Ui,
    margin: i8,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Frame::new()
        .fill(FORM_LIGHT)
        .stroke(Stroke::new(1.0, FORM_BORDER))
        .inner_margin(margin)
        .show(ui, |ui| {
            let mut visuals = egui::Visuals::light();
            visuals.override_text_color = Some(FORM_TEXT);
            visuals.widgets.noninteractive.bg_fill = Color32::WHITE;
            visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, FORM_TEXT);
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(232, 235, 239);
            visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, FORM_TEXT);
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(172, 178, 186));
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(216, 226, 238);
            visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, FORM_TEXT);
            visuals.widgets.active.bg_fill = Color32::from_rgb(198, 214, 232);
            visuals.widgets.active.fg_stroke = Stroke::new(1.0, FORM_TEXT);
            visuals.extreme_bg_color = Color32::WHITE;
            ui.ctx().style_mut_of(egui::Theme::Light, |style| {
                style.visuals = visuals.clone();
            });
            *ui.visuals_mut() = visuals;
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
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(152.0, row_height), egui::Sense::click());
        let bg = if active {
            PROGRAM
        } else if response.hovered() {
            Color32::from_rgb(48, 58, 68)
        } else {
            SIDEBAR
        };
        ui.painter().rect_filled(rect, 0.0, bg);
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
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        response
    })
    .inner
}

fn position_label(position: Position) -> &'static str {
    match position {
        Position::LowerLeft => "Lower left",
        Position::LowerRight => "Lower right",
        Position::TopLeft => "Top left",
    }
}

const ADD_CATEGORIES: [(&str, InputKind, &str); 20] = [
    ("Video", InputKind::Grid, "Select the Video file to open."),
    (
        "DVD",
        InputKind::Bars,
        "Select the DVD source or test pattern.",
    ),
    (
        "List",
        InputKind::Countdown,
        "Select the Video List or playlist items.",
    ),
    (
        "Camera",
        InputKind::Geometry,
        "Select the Camera device and capture format.",
    ),
    (
        "NDI / Desktop Capture",
        InputKind::Studio,
        "Select an NDI source or local Desktop Capture.",
    ),
    (
        "SRT",
        InputKind::Grid,
        "Configure SRT Listener / Caller input stream.",
    ),
    (
        "Stream / OMT",
        InputKind::Bars,
        "Select network Stream / RTSP / RTMP source.",
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
        "Select an Audio file (.wav / .mp3) to open.",
    ),
    (
        "Audio Input",
        InputKind::Geometry,
        "Select a Microphone or ASIO/WASAPI Audio Input.",
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
];

const SAMPLE_VIDEO_FILES: [(&str, InputKind); 7] = [
    (
        "C:\\Broadcast\\Media\\Opening_Bumper_1080p.mp4",
        InputKind::Grid,
    ),
    (
        "C:\\Broadcast\\Media\\Camera_B_ISO_Feed.mp4",
        InputKind::Geometry,
    ),
    (
        "C:\\Broadcast\\Media\\SMPTE_Reference_Bars.mp4",
        InputKind::Bars,
    ),
    (
        "C:\\Broadcast\\Media\\LowerThird_Headline_Overlay.mp4",
        InputKind::Graphics,
    ),
    (
        "C:\\Broadcast\\Media\\Newsroom_VirtualSet_Wide.mp4",
        InputKind::Studio,
    ),
    (
        "C:\\Broadcast\\Media\\Solid_Slate_Backplate.mp4",
        InputKind::Color,
    ),
    (
        "C:\\Broadcast\\Media\\Countdown_60s_Clock.mp4",
        InputKind::Countdown,
    ),
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
        let navigation_rect = egui::Rect::from_min_size(body.min, Vec2::new(164.0, height));
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
                    .max_width(152.0)
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
        egui::ScrollArea::vertical()
            .id_salt((id, "page.scroll"))
            .max_width(page_rect.width())
            .max_height(height)
            .auto_shrink([false, false])
            .show(&mut page_ui, |ui| content(self, ui));
        self.automation.insert(format!("{id}.body"), body);
    }

    fn prepare_add_preview(&mut self, ctx: &egui::Context) {
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
        self.prepare_add_preview(ctx);
        let mut open = self.add_open;
        let mut close_requested = false;
        let height = (ctx.viewport_rect().height() - 180.0).clamp(200.0, 520.0);
        operator_window(
            ctx,
            "Input Select",
            "dialog.add_input",
            &mut open,
            Vec2::new(780.0, 560.0),
        )
        .show(ctx, |ui| {
            self.dialog_columns(
                ui,
                "dialog.add",
                height,
                |app, ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    for (idx, (label, kind, _)) in ADD_CATEGORIES.into_iter().enumerate() {
                        let active = app.add_tab == idx && app.add_kind == kind;
                        let r = sidebar_row(ui, active, label, 21.0, Some(idx));
                        if matches!(idx, 0 | 3 | 4 | 6 | 11 | 14 | 15) {
                            app.automation
                                .insert(format!("dialog.add.kind.{kind:?}"), r.rect);
                        }
                        if r.clicked() {
                            app.add_tab = idx;
                            app.add_kind = kind;
                        }
                    }
                },
                |app, ui| {
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
                        });
                    light_form_frame(ui, 10, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [48.0, 22.0],
                                egui::Label::new(
                                    RichText::new("Video").color(FORM_TEXT).size(12.0),
                                ),
                            );
                            let text_w = (ui.available_width() - 94.0).max(120.0);
                            let name = ui.add_sized(
                                [text_w, 22.0],
                                egui::TextEdit::singleline(&mut app.add_name)
                                    .char_limit(64)
                                    .hint_text("Select or enter source name / file path"),
                            );
                            app.automation.insert("dialog.add.name".into(), name.rect);
                            if ui
                                .add_sized([80.0, 22.0], egui::Button::new("Browse"))
                                .clicked()
                            {
                                let sample = SAMPLE_VIDEO_FILES
                                    .iter()
                                    .find(|(_, k)| *k == app.add_kind)
                                    .copied()
                                    .unwrap_or((
                                        "C:\\Broadcast\\Media\\Opening_Bumper_1080p.mp4",
                                        InputKind::Grid,
                                    ));
                                app.add_name = sample.0.into();
                            }
                        });
                        ui.add_space(6.0);
                        ui.columns(2, |cols| {
                            if let [left_col, right_col, ..] = cols {
                                egui::Frame::new()
                                    .fill(Color32::WHITE)
                                    .stroke(Stroke::new(1.0, FORM_BORDER))
                                    .inner_margin(6.0)
                                    .show(left_col, |ui| {
                                        ui.set_min_height(155.0);
                                        ui.label(
                                            RichText::new("Recent Media & Procedural Generators")
                                                .size(11.0)
                                                .strong()
                                                .color(FORM_TEXT),
                                        );
                                        ui.separator();
                                        for (path, kind) in SAMPLE_VIDEO_FILES {
                                            let selected =
                                                app.add_kind == kind && app.add_name == path;
                                            let r = ui.selectable_label(
                                                selected,
                                                RichText::new(path).size(11.0).color(FORM_TEXT),
                                            );
                                            if r.clicked() {
                                                app.add_kind = kind;
                                                app.add_name = path.into();
                                            }
                                        }
                                    });
                                right_col.vertical(|ui| {
                                    egui::Frame::new()
                                        .fill(Color32::from_gray(18))
                                        .stroke(Stroke::new(1.0, FORM_BORDER))
                                        .inner_margin(4.0)
                                        .show(ui, |ui| {
                                            if let Some((_, texture)) = &app.add_preview {
                                                let w = ui.available_width().clamp(140.0, 260.0);
                                                let r = ui.add(
                                                    egui::Image::new((
                                                        texture.id(),
                                                        egui::vec2(w, w * 9.0 / 16.0),
                                                    ))
                                                    .corner_radius(2.0),
                                                );
                                                app.automation
                                                    .insert("dialog.add.preview".into(), r.rect);
                                            }
                                        });
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "Generator: {:?} (1920x1080 29.97p)",
                                            app.add_kind
                                        ))
                                        .size(11.0)
                                        .color(FORM_TEXT),
                                    );
                                });
                            }
                        });
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.checkbox(
                                &mut app.add_alpha,
                                RichText::new("Video contains Alpha Channel")
                                    .size(11.5)
                                    .color(FORM_TEXT),
                            );
                            ui.add_space(12.0);
                            ui.checkbox(
                                &mut app.add_interlaced,
                                RichText::new("Video is Interlaced")
                                    .size(11.5)
                                    .color(FORM_TEXT),
                            );
                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                if ui
                                    .add_sized([74.0, 22.0], egui::Button::new("Clear"))
                                    .clicked()
                                {
                                    app.add_name.clear();
                                }
                            });
                        });
                    });
                },
            );
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Number").size(12.0).color(Color32::WHITE));
                egui::ComboBox::from_id_salt("dialog.add.number")
                    .width(54.0)
                    .selected_text(self.add_number.to_string())
                    .show_ui(ui, |ui| {
                        for n in 1..=16 {
                            ui.selectable_value(&mut self.add_number, n, n.to_string());
                        }
                    });
                if self.add_name.trim().is_empty() {
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
                    let confirm = ui.add(
                        egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong())
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.add.confirm".into(), confirm.rect);
                    if confirm.clicked() {
                        if self.add_name.trim().is_empty() {
                            self.notice = "Enter a source name before adding an input".into();
                            self.warning = true;
                        } else {
                            self.actions.push(Action::Engine(Command::AddInput {
                                name: self.add_name.clone(),
                                kind: self.add_kind,
                            }));
                            close_requested = true;
                        }
                    }
                });
            });
        });
        if close_requested || !open {
            self.add_open = false;
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
        let height = (ctx.viewport_rect().height() - 180.0).clamp(200.0, 540.0);
        operator_window(
            ctx,
            format!("Input: {} {title_name}", id.0),
            "dialog.input_properties",
            &mut open,
            Vec2::new(790.0, 580.0),
        )
        .show(ctx, |ui| {
            self.dialog_columns(
                ui,
                "dialog.properties",
                height,
                |app, ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    for (idx, (page, label, key)) in [
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
                    .enumerate()
                    {
                        let r = sidebar_row(ui, app.input_page == page, label, 18.5, Some(idx));
                        app.automation
                            .insert(format!("dialog.properties.page.{key}"), r.rect);
                        if r.clicked() {
                            app.input_page = page;
                        }
                    }
                    ui.add_space(4.0);
                    let _ = ui.add_sized(
                        [152.0, 22.0],
                        egui::Button::new(RichText::new("Copy From...").size(11.0)).fill(CHIP_BG),
                    );
                },
                |app, ui| {
                    egui::Frame::new()
                        .fill(HEADER_DARK)
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.set_min_width((ui.available_width() - 2.0).max(80.0));
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("{}.mp4", draft.name))
                                        .color(Color32::WHITE)
                                        .size(12.0),
                                );
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    let _ = ui.add_sized(
                                        [68.0, 20.0],
                                        egui::Button::new("Change").fill(CHIP_BG),
                                    );
                                });
                            });
                        });
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
                                            egui::ComboBox::from_id_salt("prop.aspect")
                                                .width(175.0)
                                                .selected_text("Widescreen")
                                                .show_ui(ui, |ui| {
                                                    let _ = ui.selectable_label(true, "Widescreen");
                                                    let _ = ui.selectable_label(false, "Normal");
                                                    let _ =
                                                        ui.selectable_label(false, "Anamorphic");
                                                });
                                            ui.end_row();

                                            ui.label(
                                                RichText::new("Category")
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
                                                    ui.painter().rect_filled(rect, 1.0, swatch_c);
                                                    ui.painter().rect_stroke(
                                                        rect,
                                                        1.0,
                                                        Stroke::new(
                                                            if draft.kind == kind {
                                                                2.0
                                                            } else {
                                                                1.0
                                                            },
                                                            if draft.kind == kind {
                                                                Color32::BLACK
                                                            } else {
                                                                Color32::from_gray(110)
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
                                    left_col
                                        .checkbox(&mut app.prop_deinterlace, "Deinterlace Blend");
                                    left_col.checkbox(&mut app.prop_sharpen, "Sharpen");
                                    left_col.checkbox(&mut app.prop_mirror, "Mirror");
                                    left_col.checkbox(&mut app.prop_flatten, "Flatten Layers");
                                    left_col.checkbox(
                                        &mut app.prop_auto_mix,
                                        "Automatically mix audio",
                                    );
                                    left_col.checkbox(
                                        &mut app.prop_auto_play,
                                        "Automatically Play with Transition",
                                    );
                                    left_col.checkbox(
                                        &mut app.prop_auto_restart,
                                        "Automatically Restart with Transition",
                                    );
                                    left_col.checkbox(
                                        &mut app.prop_auto_pause,
                                        "Automatically Pause after Transition",
                                    );

                                    egui::Grid::new("input_props_right")
                                        .num_columns(2)
                                        .spacing([8.0, 4.0])
                                        .show(right_col, |ui| {
                                            ui.label(
                                                RichText::new("Mouse Click Action")
                                                    .size(11.5)
                                                    .color(FORM_TEXT),
                                            );
                                            egui::ComboBox::from_id_salt("prop.mouse_action")
                                                .width(135.0)
                                                .selected_text("Preview")
                                                .show_ui(ui, |ui| {
                                                    let _ = ui.selectable_label(true, "Preview");
                                                    let _ =
                                                        ui.selectable_label(false, "Quick Play");
                                                    let _ = ui.selectable_label(false, "Cut");
                                                });
                                            ui.end_row();

                                            ui.label(
                                                RichText::new("GO Action")
                                                    .size(11.5)
                                                    .color(FORM_TEXT),
                                            );
                                            egui::ComboBox::from_id_salt("prop.go_action")
                                                .width(135.0)
                                                .selected_text("Quick Play")
                                                .show_ui(ui, |ui| {
                                                    let _ = ui.selectable_label(true, "Quick Play");
                                                    let _ = ui.selectable_label(false, "Fade");
                                                    let _ = ui.selectable_label(false, "Merge");
                                                });
                                            ui.end_row();

                                            for (k, v) in [
                                                ("Source Aspect Ratio", "Widescreen"),
                                                ("Resolution", "1920x1080"),
                                                ("Frame Rate", "29.97p"),
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
                                    let _ = right_col.add_sized(
                                        [240.0, 24.0],
                                        egui::Button::new("Create Virtual Input"),
                                    );
                                }
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
                                "Source online (uncheck to simulate fallback slate)",
                            );
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
                            ui.add(egui::Slider::new(&mut app.prop_zoom, 0.25..=4.0).text("Zoom"));
                            ui.add(
                                egui::Slider::new(&mut app.prop_pan_x, -1.0..=1.0).text("Pan X"),
                            );
                            ui.add(
                                egui::Slider::new(&mut app.prop_pan_y, -1.0..=1.0).text("Pan Y"),
                            );
                            ui.add(
                                egui::Slider::new(&mut app.prop_lift, -1.0..=1.0).text("Lift / Y"),
                            );
                            ui.add(egui::Slider::new(&mut app.prop_gain, 0.0..=2.0).text("Gain"));
                            ui.add(
                                egui::Slider::new(&mut app.prop_saturation, 0.0..=2.0)
                                    .text("Saturation"),
                            );
                        }
                    });
                    // Bottom black preview section matching `Input Settings.png`.
                    egui::Frame::new()
                        .fill(Color32::BLACK)
                        .inner_margin(6.0)
                        .show(ui, |ui| {
                            ui.set_min_width((ui.available_width() - 2.0).max(80.0));
                            ui.horizontal(|ui| {
                                let avail_w = (ui.available_width() - 42.0).max(160.0);
                                let preview_h = 132.0_f32;
                                let preview_w = (preview_h * 16.0 / 9.0).min(avail_w);
                                let pad_left = ((avail_w - preview_w) * 0.5).max(0.0);
                                ui.add_space(pad_left);
                                if let Some(texture) = app.textures.get(&id) {
                                    let r = ui.add(egui::Image::new((
                                        texture.id(),
                                        egui::vec2(preview_w, preview_h),
                                    )));
                                    app.automation
                                        .insert("dialog.properties.preview".into(), r.rect);
                                }
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.add_sized(
                                            [20.0, 108.0],
                                            egui::Slider::new(&mut app.prop_zoom, 0.25..=2.0)
                                                .vertical()
                                                .show_value(false),
                                        );
                                        ui.label(
                                            RichText::new(format!("{:.0}", app.prop_zoom))
                                                .size(11.0)
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
                        self.engine.project().inputs.len() > 2,
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
                    let apply = ui.add(
                        egui::Button::new(RichText::new("OK").color(Color32::WHITE).strong())
                            .fill(CHIP_BG)
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
            self.actions
                .push(Action::Engine(Command::UpdateInput(draft)));
            self.properties = None;
        } else if cancel_clicked || !open {
            self.properties = None;
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
                            .add_sized(
                                [152.0, 21.0],
                                egui::Button::new(RichText::new(action_lbl).size(11.0))
                                    .fill(CHIP_BG),
                            )
                            .clicked()
                            && action_lbl == "Default"
                        {
                            draft = ProjectState::demo().preferences;
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
                                    egui::ComboBox::from_id_salt("settings.theme")
                                        .width(210.0)
                                        .selected_text("Charcoal8")
                                        .show_ui(ui, |ui| {
                                            let _ = ui.selectable_label(true, "Charcoal8");
                                            let _ = ui.selectable_label(false, "Midnight");
                                        });
                                    ui.end_row();

                                    ui.label(
                                        RichText::new("Preview Colour").size(11.5).color(FORM_TEXT),
                                    );
                                    let (pvw_rect, _) = ui.allocate_exact_size(
                                        Vec2::new(210.0, 18.0),
                                        Sense::hover(),
                                    );
                                    ui.painter().rect_filled(pvw_rect, 1.0, PREVIEW);
                                    ui.end_row();

                                    ui.label(
                                        RichText::new("Output Colour").size(11.5).color(FORM_TEXT),
                                    );
                                    let (pgm_rect, _) = ui.allocate_exact_size(
                                        Vec2::new(210.0, 18.0),
                                        Sense::hover(),
                                    );
                                    ui.painter().rect_filled(pgm_rect, 1.0, PROGRAM);
                                    ui.end_row();

                                    ui.label(
                                        RichText::new("Master Frame Rate")
                                            .size(11.5)
                                            .color(FORM_TEXT),
                                    );
                                    egui::ComboBox::from_id_salt("settings.rate")
                                        .width(210.0)
                                        .selected_text(format!("NTSC {}p", draft.rate.label()))
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

                                    ui.label(
                                        RichText::new("Output Size").size(11.5).color(FORM_TEXT),
                                    );
                                    ui.vertical(|ui| {
                                        egui::ComboBox::from_id_salt("settings.output_size")
                                            .width(210.0)
                                            .selected_text("1920x1080")
                                            .show_ui(ui, |ui| {
                                                let _ = ui.selectable_label(true, "1920x1080");
                                                let _ = ui.selectable_label(false, "1280x720");
                                                let _ = ui.selectable_label(false, "3840x2160");
                                            });
                                        let (bar, _) = ui.allocate_exact_size(
                                            Vec2::new(210.0, 4.0),
                                            Sense::hover(),
                                        );
                                        ui.painter().rect_filled(
                                            bar,
                                            0.0,
                                            Color32::from_rgb(0, 68, 204),
                                        );
                                    });
                                    ui.end_row();

                                    ui.label(
                                        RichText::new("Output Aspect Ratio")
                                            .size(11.5)
                                            .color(FORM_TEXT),
                                    );
                                    egui::ComboBox::from_id_salt("settings.aspect")
                                        .width(210.0)
                                        .selected_text("Widescreen")
                                        .show_ui(ui, |ui| {
                                            let _ = ui.selectable_label(true, "Widescreen");
                                            let _ = ui.selectable_label(false, "Normal");
                                        });
                                    ui.end_row();

                                    ui.label("");
                                    ui.horizontal(|ui| {
                                        for (slot, lbl) in
                                            [(1_u8, "Fullscreen 1"), (2_u8, "Fullscreen 2")]
                                        {
                                            let active = app.settings_fullscreen_slot == slot;
                                            if ui
                                                .add_sized(
                                                    [101.0, 22.0],
                                                    egui::Button::new(
                                                        RichText::new(lbl)
                                                            .size(11.0)
                                                            .color(Color32::WHITE),
                                                    )
                                                    .fill(if active { PROGRAM } else { CHIP_BG }),
                                                )
                                                .clicked()
                                            {
                                                app.settings_fullscreen_slot = slot;
                                            }
                                        }
                                    });
                                    ui.end_row();

                                    ui.label(RichText::new("Display").size(11.5).color(FORM_TEXT));
                                    egui::ComboBox::from_id_salt("settings.display_idx")
                                        .width(210.0)
                                        .selected_text("1")
                                        .show_ui(ui, |ui| {
                                            let _ = ui.selectable_label(true, "1");
                                            let _ = ui.selectable_label(false, "2");
                                        });
                                    ui.end_row();

                                    ui.label(RichText::new("Position").size(11.5).color(FORM_TEXT));
                                    ui.horizontal(|ui| {
                                        for val in ["0", "0", "0", "0"] {
                                            let mut s = val.to_owned();
                                            ui.add_sized(
                                                [46.0, 20.0],
                                                egui::TextEdit::singleline(&mut s),
                                            );
                                        }
                                    });
                                    ui.end_row();

                                    ui.label("");
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut app.settings_hide_cursor, "Hide Cursor");
                                        ui.checkbox(&mut app.settings_on_top, "On Top");
                                        ui.checkbox(&mut app.settings_minimise, "Minimise");
                                    });
                                    ui.end_row();

                                    ui.label(
                                        RichText::new("Input Size").size(11.5).color(FORM_TEXT),
                                    );
                                    egui::ComboBox::from_id_salt("settings.input_size")
                                        .width(210.0)
                                        .selected_text("1280x720 (720p)")
                                        .show_ui(ui, |ui| {
                                            let _ = ui.selectable_label(true, "1280x720 (720p)");
                                            let _ = ui.selectable_label(false, "1920x1080 (1080p)");
                                        });
                                    ui.end_row();

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
                            ui.label("Web Controller Address: http://127.0.0.1:8088");
                        }
                        SettingsPage::Shortcuts => {
                            ui.label(
                                RichText::new("Keyboard & Surface Shortcuts")
                                    .size(13.0)
                                    .strong()
                                    .color(FORM_TEXT),
                            );
                            egui::Grid::new("settings.shortcuts.grid")
                                .striped(true)
                                .spacing([24.0, 6.0])
                                .show(ui, |ui| {
                                    for (k, v) in [
                                        ("1 .. 9", "Select source 1..9 on Preview"),
                                        ("Space", "CUT Preview to Program"),
                                        ("Enter", "FADE Preview to Program"),
                                        ("F1 .. F4", "Toggle overlay 1..4 on Program"),
                                        ("Ctrl+S / Ctrl+O", "Save / Open preset"),
                                    ] {
                                        ui.strong(k);
                                        ui.label(v);
                                        ui.end_row();
                                    }
                                });
                        }
                        _ => {
                            ui.label(
                                RichText::new(format!("{:?} Configuration", app.settings_page))
                                    .size(13.0)
                                    .strong()
                                    .color(FORM_TEXT),
                            );
                            ui.label("Configured for 1080p29,97 low-latency production routing.");
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
                ui.checkbox(
                    &mut self.settings_advanced,
                    RichText::new("Show Advanced Settings")
                        .size(11.5)
                        .color(Color32::WHITE),
                );
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
        if apply_clicked {
            if !self.settings_name.trim().is_empty()
                && self.settings_name.trim() != self.engine.project().name
            {
                self.actions.push(Action::Engine(Command::RenameProject(
                    self.settings_name.clone(),
                )));
            }
            if let Some(audio_focus) = self.settings_layout {
                self.actions.push(Action::Workspace(audio_focus));
            }
            self.actions
                .push(Action::Engine(Command::SetPreferences(draft)));
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
                    egui::Grid::new("overlay_top_form")
                        .num_columns(4)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Number");
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_salt("overlay.number.combo")
                                    .width(64.0)
                                    .selected_text(format!("{}", slot + 1))
                                    .show_ui(ui, |ui| {
                                        for i in 0..4 {
                                            if ui
                                                .selectable_label(slot == i, format!("{}", i + 1))
                                                .clicked()
                                                && let Some(committed) =
                                                    self.engine.project().overlays.get(i).cloned()
                                            {
                                                slot = i;
                                                draft = committed;
                                            }
                                        }
                                    });
                                for i in 0..4 {
                                    let r = ui.add_sized(
                                        [24.0, 20.0],
                                        egui::Button::new(format!("{}", i + 1))
                                            .fill(if slot == i { PROGRAM } else { CHIP_BG }),
                                    );
                                    self.automation
                                        .insert(format!("dialog.overlay.slot.{}", i + 1), r.rect);
                                    if r.clicked()
                                        && let Some(committed) =
                                            self.engine.project().overlays.get(i).cloned()
                                    {
                                        slot = i;
                                        draft = committed;
                                    }
                                }
                            });
                            let enabled = ui.checkbox(&mut draft.enabled, "Show on Program");
                            self.automation
                                .insert("dialog.overlay.enabled".into(), enabled.rect);
                            ui.horizontal(|ui| {
                                for position in
                                    [Position::LowerLeft, Position::LowerRight, Position::TopLeft]
                                {
                                    let r = ui.selectable_value(
                                        &mut draft.position,
                                        position,
                                        position_label(position),
                                    );
                                    self.automation.insert(
                                        format!("dialog.overlay.position.{position:?}"),
                                        r.rect,
                                    );
                                }
                            });
                            ui.end_row();

                            ui.label("Type");
                            let type_resp = egui::ComboBox::from_id_salt("overlay.type.combo")
                                .width(175.0)
                                .selected_text(
                                    ["Picture In Picture", "Fullscreen", "Lower Third Title"]
                                        .get(self.overlay_type_idx)
                                        .copied()
                                        .unwrap_or("Picture In Picture"),
                                )
                                .show_ui(ui, |ui| {
                                    for (idx, lbl) in
                                        ["Picture In Picture", "Fullscreen", "Lower Third Title"]
                                            .into_iter()
                                            .enumerate()
                                    {
                                        ui.selectable_value(&mut self.overlay_type_idx, idx, lbl);
                                    }
                                });
                            self.automation
                                .insert("dialog.overlay.type".into(), type_resp.response.rect);
                            ui.label("Title / Sub");
                            ui.horizontal(|ui| {
                                let t = ui.add_sized(
                                    [105.0, 20.0],
                                    egui::TextEdit::singleline(&mut draft.title).char_limit(96),
                                );
                                self.automation
                                    .insert("dialog.overlay.title".into(), t.rect);
                                let s = ui.add_sized(
                                    [105.0, 20.0],
                                    egui::TextEdit::singleline(&mut draft.subtitle).char_limit(160),
                                );
                                self.automation
                                    .insert("dialog.overlay.subtitle".into(), s.rect);
                            });
                            ui.end_row();

                            ui.label("Effect");
                            egui::ComboBox::from_id_salt("overlay.effect.combo")
                                .width(175.0)
                                .selected_text(
                                    ["Fade", "Zoom", "Slide", "Fly"]
                                        .get(self.overlay_effect_idx)
                                        .copied()
                                        .unwrap_or("Fade"),
                                )
                                .show_ui(ui, |ui| {
                                    for (idx, lbl) in
                                        ["Fade", "Zoom", "Slide", "Fly"].into_iter().enumerate()
                                    {
                                        ui.selectable_value(&mut self.overlay_effect_idx, idx, lbl);
                                    }
                                });
                            ui.label("Effect Duration");
                            ui.add(
                                egui::DragValue::new(&mut self.overlay_effect_ms)
                                    .range(0..=5000)
                                    .suffix(" ms"),
                            );
                            ui.end_row();

                            ui.label("Duration");
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut self.overlay_duration_val)
                                        .range(0..=60000),
                                );
                                ui.radio_value(
                                    &mut self.overlay_duration_frames,
                                    false,
                                    "Milliseconds",
                                );
                                ui.radio_value(&mut self.overlay_duration_frames, true, "Frames");
                            });
                            ui.label("Stinger Input");
                            egui::ComboBox::from_id_salt("overlay.stinger.combo")
                                .width(140.0)
                                .selected_text("None")
                                .show_ui(ui, |ui| {
                                    let _ = ui.selectable_label(true, "None");
                                    let _ = ui.selectable_label(false, "Input 1");
                                });
                            ui.end_row();

                            ui.label("Alpha");
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Slider::new(&mut self.overlay_alpha, 0..=255)
                                        .show_value(false),
                                );
                                ui.label(format!("{}", self.overlay_alpha));
                            });
                            ui.label("Stinger Cut");
                            ui.add(
                                egui::DragValue::new(&mut self.overlay_stinger_cut).range(0..=5000),
                            );
                            ui.end_row();
                        });
                    ui.add_space(6.0);
                    ui.columns(2, |cols| {
                        if let [left_col, right_col, ..] = cols {
                            light_form_frame(left_col, 8, |ui| {
                                ui.label(
                                    RichText::new("Picture In Picture Location")
                                        .size(12.0)
                                        .strong()
                                        .color(FORM_TEXT),
                                );
                                egui::Grid::new("overlay_pip_sliders")
                                    .num_columns(3)
                                    .spacing([6.0, 4.0])
                                    .show(ui, |ui| {
                                        for (lbl, val, min_v, max_v, def_v) in [
                                            ("Zoom", &mut self.overlay_zoom, 0.1_f32, 2.0, 0.4375),
                                            ("Zoom X", &mut self.overlay_zoom_x, 0.1, 2.0, 0.4375),
                                            ("Zoom Y", &mut self.overlay_zoom_y, 0.1, 2.0, 0.4375),
                                            ("Pan X", &mut self.overlay_pan_x, -1.0, 1.0, 0.463),
                                            ("Pan Y", &mut self.overlay_pan_y, -1.0, 1.0, -0.464),
                                            (
                                                "Rotate Z",
                                                &mut self.overlay_rotate_z,
                                                -3.14,
                                                3.14,
                                                0.0,
                                            ),
                                        ] {
                                            ui.label(
                                                RichText::new(lbl).size(11.0).color(FORM_TEXT),
                                            );
                                            ui.add(
                                                egui::Slider::new(val, min_v..=max_v)
                                                    .fixed_decimals(3),
                                            );
                                            if ui.small_button("Reset").clicked() {
                                                *val = def_v;
                                            }
                                            ui.end_row();
                                        }
                                    });
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Border")
                                            .size(11.5)
                                            .strong()
                                            .color(FORM_TEXT),
                                    );
                                    ui.label("Thickness");
                                    ui.add(
                                        egui::DragValue::new(&mut self.overlay_border_thickness)
                                            .range(0..=20),
                                    );
                                    ui.label("Radius");
                                    ui.add(
                                        egui::DragValue::new(&mut self.overlay_border_radius)
                                            .range(0..=40),
                                    );
                                });
                            });
                            right_col.vertical_centered(|ui| {
                                let w = ui.available_width().clamp(200.0, 340.0);
                                let h = w * 9.0 / 16.0;
                                let (frame, _) = ui
                                    .allocate_exact_size(Vec2::new(w, h), Sense::click_and_drag());
                                ui.painter()
                                    .rect_filled(frame, 0.0, Color32::from_rgb(33, 39, 45));
                                ui.painter().rect_stroke(
                                    frame,
                                    0.0,
                                    Stroke::new(1.0, Color32::WHITE),
                                    egui::StrokeKind::Inside,
                                );
                                let pip_w = frame.width() * self.overlay_zoom.clamp(0.15, 0.9);
                                let pip_h = frame.height() * self.overlay_zoom.clamp(0.15, 0.9);
                                let cx =
                                    frame.center().x + self.overlay_pan_x * frame.width() * 0.25;
                                let cy =
                                    frame.center().y - self.overlay_pan_y * frame.height() * 0.25;
                                let pip = egui::Rect::from_center_size(
                                    egui::pos2(cx, cy),
                                    Vec2::new(pip_w, pip_h),
                                )
                                .intersect(frame.shrink(3.0));
                                ui.painter()
                                    .rect_filled(pip, 0.0, Color32::from_rgb(122, 0, 0));
                                ui.painter().rect_stroke(
                                    pip,
                                    0.0,
                                    Stroke::new(1.5, Color32::from_rgb(0, 120, 215)),
                                    egui::StrokeKind::Inside,
                                );
                                let panel = overlay_panel_rect(frame, draft.position, slot);
                                ui.painter().rect_filled(
                                    panel,
                                    2.0,
                                    Color32::from_black_alpha(180),
                                );
                                ui.painter().text(
                                    egui::pos2(panel.left() + 6.0, panel.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    format!("O{} {}", slot + 1, draft.title),
                                    egui::FontId::proportional(10.5),
                                    Color32::WHITE,
                                );
                                self.automation
                                    .insert("dialog.overlay.preview".into(), frame);
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("Click and drag inside box to move.")
                                        .size(11.0)
                                        .color(Color32::WHITE),
                                );
                            });
                        }
                    });
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
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.recording.ok".into(), ok.rect);
                    if ok.clicked() {
                        self.notice = format!("Recording Setup saved ({})", self.rec_filename);
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
                        if ui
                            .add_sized(
                                [78.0, 22.0],
                                egui::Button::new(if self.stream_active {
                                    format!("Stop {}", self.stream_channel + 1)
                                } else {
                                    format!("Start {}", self.stream_channel + 1)
                                }),
                            )
                            .clicked()
                        {
                            self.stream_active = !self.stream_active;
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
                        .fill(CHIP_BG)
                        .min_size(Vec2::new(112.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.streaming.save".into(), save.rect);
                    if save.clicked() {
                        self.notice = "Streaming Settings saved".into();
                        self.warning = false;
                        close_clicked = true;
                    }
                });
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
                            .fill(Color32::WHITE)
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
                                    .color(if self.mc_running { RED } else { FORM_TEXT }),
                                );
                            });
                        let toggle = ui.add_sized(
                            [115.0, 56.0],
                            egui::Button::new(
                                RichText::new(if self.mc_running { "Stop" } else { "Start" })
                                    .size(18.0)
                                    .color(Color32::WHITE),
                            )
                            .fill(if self.mc_running {
                                RED
                            } else {
                                CHIP_BG
                            }),
                        );
                        self.automation
                            .insert("dialog.multicorder.toggle".into(), toggle.rect);
                        if toggle.clicked() {
                            self.mc_running = !self.mc_running;
                        }
                    });
                }
            });
            ui.add_space(4.0);
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
                            rows.push(format!("{}: {}", input.id.0, input.name));
                        }
                        for (idx, label) in rows.into_iter().take(10).enumerate() {
                            if let Some(checked) = self.mc_checked.get_mut(idx) {
                                ui.checkbox(checked, label);
                            }
                            ui.label(&self.mc_folder);
                            ui.label(if self.mc_running {
                                "00:01:12"
                            } else {
                                "00:00:00"
                            });
                            ui.label(if self.mc_running { "2160" } else { "0" });
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
        });
        if close_clicked || !open {
            self.multicorder_open = false;
        }
    }

    pub(crate) fn playlist_dialog(&mut self, ctx: &egui::Context) {
        if !self.playlist_open {
            return;
        }
        let mut open = self.playlist_open;
        let mut close_clicked = false;
        operator_window(
            ctx,
            "PlayList",
            "dialog.playlist",
            &mut open,
            Vec2::new(720.0, 460.0),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [105.0, 24.0],
                        egui::Button::new("New PlayList").fill(CHIP_BG),
                    )
                    .clicked()
                {
                    self.playlist_items.clear();
                }
                if ui
                    .add_sized(
                        [110.0, 24.0],
                        egui::Button::new("Delete PlayList").fill(CHIP_BG),
                    )
                    .clicked()
                {
                    self.playlist_items.clear();
                }
                egui::ComboBox::from_id_salt("playlist.preset_select")
                    .width(260.0)
                    .selected_text("Default PlayList")
                    .show_ui(ui, |ui| {
                        let _ = ui.selectable_label(true, "Default PlayList");
                    });
            });
            ui.add_space(6.0);
            let inputs: Vec<(InputId, String)> = self
                .engine
                .project()
                .inputs
                .iter()
                .map(|i| (i.id, format!("{}   {}", i.id.0, i.name)))
                .collect();
            ui.columns(3, |cols| {
                if let [left_col, mid_col, right_col, ..] = cols {
                    light_form_frame(left_col, 6, |ui| {
                        ui.set_min_height(210.0);
                        ui.label(RichText::new("Available Inputs").strong().color(FORM_TEXT));
                        ui.separator();
                        for (idx, (_, label)) in inputs.iter().enumerate() {
                            if ui
                                .selectable_label(self.playlist_selected_avail == idx, label)
                                .clicked()
                            {
                                self.playlist_selected_avail = idx;
                            }
                        }
                    });
                    mid_col.vertical_centered(|ui| {
                        ui.add_space(64.0);
                        if ui
                            .add_sized([48.0, 28.0], egui::Button::new(">").fill(CHIP_BG))
                            .clicked()
                            && let Some((id, _)) = inputs.get(self.playlist_selected_avail)
                        {
                            self.playlist_items.push(*id);
                        }
                        ui.add_space(8.0);
                        if ui
                            .add_sized([48.0, 28.0], egui::Button::new("<").fill(CHIP_BG))
                            .clicked()
                            && self.playlist_selected_item < self.playlist_items.len()
                        {
                            self.playlist_items.remove(self.playlist_selected_item);
                        }
                    });
                    light_form_frame(right_col, 6, |ui| {
                        ui.set_min_height(210.0);
                        ui.label(RichText::new("PlayList Items").strong().color(FORM_TEXT));
                        ui.separator();
                        for (idx, item_id) in self.playlist_items.clone().into_iter().enumerate() {
                            let label = inputs
                                .iter()
                                .find(|(id, _)| *id == item_id)
                                .map(|(_, l)| l.as_str())
                                .unwrap_or("Input");
                            if ui
                                .selectable_label(self.playlist_selected_item == idx, label)
                                .clicked()
                            {
                                self.playlist_selected_item = idx;
                            }
                        }
                    });
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.playlist_loop, "Loop");
                        ui.checkbox(
                            &mut self.playlist_begin_selected,
                            "Begin From Selected Item",
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.playlist_clear_overlays, "Clear Overlays");
                        ui.checkbox(&mut self.playlist_manual, "Manual");
                    });
                });
                ui.add_space(16.0);
                let start = ui.add_sized(
                    [92.0, 44.0],
                    egui::Button::new(
                        RichText::new(if self.playlist_running {
                            "Stop"
                        } else {
                            "Start"
                        })
                        .size(16.0)
                        .color(Color32::WHITE),
                    )
                    .fill(if self.playlist_running {
                        PROGRAM
                    } else {
                        CHIP_BG
                    }),
                );
                self.automation
                    .insert("dialog.playlist.start".into(), start.rect);
                if start.clicked() {
                    self.playlist_running = !self.playlist_running;
                }
                let _ = ui.add_sized([42.0, 26.0], egui::Button::new("|<<").fill(CHIP_BG));
                let _ = ui.add_sized([42.0, 26.0], egui::Button::new(">>|").fill(CHIP_BG));
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let close = ui.add(
                        egui::Button::new("Close")
                            .fill(CHIP_BG)
                            .min_size(Vec2::new(92.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.playlist.close".into(), close.rect);
                    if close.clicked() {
                        close_clicked = true;
                    }
                });
            });
        });
        if close_clicked || !open {
            self.playlist_open = false;
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
            let tiles: [(&str, Action); 12] = [
                ("Cut\n[Space]", Action::Engine(Command::Cut)),
                (
                    "Fade\n[Enter]",
                    Action::Engine(Command::Transition(TransitionKind::Fade)),
                ),
                (
                    "Merge\n[300 ms]",
                    Action::Engine(Command::Transition(TransitionKind::Fade)),
                ),
                (
                    "Wipe\n[300 ms]",
                    Action::Engine(Command::Transition(TransitionKind::Wipe)),
                ),
                ("FTB\n[Fade to Black]", Action::Engine(Command::FadeBlack)),
                ("Overlay 1\n[F1]", Action::OverlaySettings(0)),
                ("Overlay 2\n[F2]", Action::OverlaySettings(1)),
                (
                    "Preview 1\n[Key 1]",
                    Action::Engine(Command::SelectPreview(InputId(1))),
                ),
                (
                    "Preview 2\n[Key 2]",
                    Action::Engine(Command::SelectPreview(InputId(2))),
                ),
                (
                    "Preview 3\n[Key 3]",
                    Action::Engine(Command::SelectPreview(InputId(3))),
                ),
                ("Record\n[Toggle]", Action::ToggleRecordSim),
                ("Stream\n[Toggle]", Action::ToggleStreamSim),
            ];
            egui::Grid::new("shortcuts_button_grid")
                .num_columns(3)
                .spacing([12.0, 12.0])
                .show(ui, |ui| {
                    for (idx, (label, action)) in tiles.into_iter().enumerate() {
                        if ui
                            .add_sized(
                                [182.0, 74.0],
                                egui::Button::new(
                                    RichText::new(label).size(13.0).color(Color32::WHITE),
                                )
                                .fill(CHIP_BG),
                            )
                            .clicked()
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
                    self.actions
                        .push(Action::SettingsAt(SettingsPage::Shortcuts));
                }
                if ui
                    .add_sized([84.0, 29.0], egui::Button::new("Settings").fill(CHIP_BG))
                    .clicked()
                {
                    self.actions
                        .push(Action::SettingsAt(SettingsPage::Shortcuts));
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
            .map(|input| format!("input {} — {}", id.0, input.name))
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
                "Remove {label} from this preset? If it is selected on Program or Preview, the engine will reassign that bus only when you confirm."
            ));
            ui.add_space(12.0);
            ui.separator();
            ui.horizontal(|ui| {
                let confirm = ui.add(
                    egui::Button::new(RichText::new("Remove input").color(RED).strong())
                        .min_size(Vec2::new(118.0, 29.0)),
                );
                self.automation
                    .insert("dialog.remove.confirm".into(), confirm.rect);
                if confirm.clicked() {
                    self.actions.push(Action::Engine(Command::RemoveInput(id)));
                    self.confirm_remove_input = None;
                    if self.properties.as_ref().is_some_and(|input| input.id == id) {
                        self.properties = None;
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
