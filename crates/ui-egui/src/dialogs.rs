//! Operator dialogs. Only expose implemented synthetic-source and project controls.
use super::*;
use egui::{Align, Color32, RichText, Stroke, Vec2};

const SIDEBAR: Color32 = Color32::from_rgb(19, 26, 31);
const SURFACE: Color32 = Color32::from_rgb(40, 49, 56);

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

fn sidebar_button(ui: &mut egui::Ui, active: bool, label: &str) -> egui::Response {
    ui.push_id(label, |ui| {
        ui.add_sized(
            [152.0, 29.0],
            egui::Button::new(RichText::new(label).strong().color(Color32::WHITE))
                .fill(if active { PROGRAM } else { SIDEBAR })
                .stroke(Stroke::NONE),
        )
    })
    .inner
}

fn section_heading(ui: &mut egui::Ui, title: &str, detail: &str) {
    ui.label(RichText::new(title).size(18.0).strong());
    ui.label(RichText::new(detail).color(DIM).size(11.0));
    ui.add_space(8.0);
    ui.separator();
}

fn kind_description(kind: InputKind) -> &'static str {
    match kind {
        InputKind::Grid => "Animated alignment grid and moving reference shapes.",
        InputKind::Geometry => "Animated graphic shapes for testing source switching.",
        InputKind::Bars => "Procedural color bars and grayscale reference.",
        InputKind::Graphics => "Generated broadcast graphics and title-safe reference.",
        InputKind::Studio => "Procedurally drawn newsroom-style background.",
        InputKind::Color => "Static color generator for a full-frame fill.",
        InputKind::Countdown => "Animated one-minute countdown reference.",
    }
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
        // Child layouts must not feed their content width back into the Window.
        // Explicit rectangles also keep the footer fixed when a long page scrolls.
        let width = ui
            .available_width()
            .min((ui.ctx().viewport_rect().width() - 48.0).max(250.0));
        let (body, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());
        let navigation_rect = egui::Rect::from_min_size(body.min, Vec2::new(168.0, height));
        let page_rect = egui::Rect::from_min_max(
            egui::pos2(navigation_rect.right() + 12.0, body.top()),
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
            .inner_margin(8.0)
            .show(&mut navigation_ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt((id, "navigation.scroll"))
                    .max_width(152.0)
                    .max_height((height - 16.0).max(80.0))
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
                self.add_preview = Some((
                    self.add_kind,
                    ctx.load_texture("dialog.source.sample", image, egui::TextureOptions::LINEAR),
                ));
            }
            Err(error) => self.say(format!("Generator sample unavailable: {error}"), true),
        }
    }

    pub(crate) fn add_input_dialog(&mut self, ctx: &egui::Context) {
        if !self.add_open {
            return;
        }
        self.prepare_add_preview(ctx);
        let mut open = true;
        let mut created = false;
        operator_window(
            ctx,
            "Add Input  /  source library",
            "input.add.dialog",
            &mut open,
            Vec2::new(820.0, 540.0),
        )
        .show(ctx, |ui| {
            ui.label(
                RichText::new("SOURCE SELECTOR   /   LOCAL SYNTHETIC GENERATORS")
                    .small()
                    .strong()
                    .color(TEAL),
            );
            ui.separator();
            let body_height = (ui.available_height() - 54.0).max(130.0);
            self.dialog_columns(
                ui,
                "dialog.add",
                body_height,
                |app, ui| {
                    ui.label(RichText::new("AVAILABLE").small().strong().color(DIM));
                    for kind in InputKind::ALL {
                        let button = sidebar_button(ui, app.add_kind == kind, kind.label());
                        app.automation
                            .insert(format!("dialog.add.kind.{kind:?}"), button.rect);
                        if button.clicked() {
                            app.add_kind = kind;
                        }
                    }
                    ui.add_space(12.0);
                    ui.label(RichText::new("UNAVAILABLE").small().strong().color(DIM));
                    for label in [
                        "Video / image",
                        "Camera / screen",
                        "Audio / device",
                        "Network / browser",
                    ] {
                        ui.add_enabled(
                            false,
                            egui::Button::new(label).min_size(Vec2::new(152.0, 27.0)),
                        )
                        .on_disabled_hover_text("No media ingestion or device backend exists yet");
                    }
                },
                |app, ui| {
                    section_heading(ui, app.add_kind.label(), kind_description(app.add_kind));
                    egui::Frame::new()
                        .fill(Color32::from_rgb(10, 14, 18))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            if let Some((_, texture)) = app
                                .add_preview
                                .as_ref()
                                .filter(|(kind, _)| *kind == app.add_kind)
                            {
                                let preview =
                                    ui.add(egui::Image::new(texture).fit_to_exact_size(Vec2::new(
                                        ui.available_width().min(320.0),
                                        ui.available_width().min(320.0) * 9.0 / 16.0,
                                    )));
                                app.automation
                                    .insert("dialog.add.preview".into(), preview.rect);
                            } else {
                                let (rect, _) = ui.allocate_exact_size(
                                    Vec2::new(
                                        ui.available_width().min(320.0),
                                        ui.available_width().min(320.0) * 9.0 / 16.0,
                                    ),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 0.0, SURFACE);
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    app.add_kind.label(),
                                    egui::FontId::proportional(20.0),
                                    Color32::WHITE,
                                );
                            }
                        });
                    ui.label(
                        RichText::new("Generated sample · independent of project inputs")
                            .small()
                            .color(DIM),
                    );
                    ui.add_space(8.0);
                    ui.label(RichText::new("Input name").strong());
                    let response = ui.add_sized(
                        [ui.available_width().min(420.0), 26.0],
                        egui::TextEdit::singleline(&mut app.add_name)
                            .char_limit(64)
                            .hint_text("Enter a descriptive source name"),
                    );
                    app.automation
                        .insert("dialog.add.name".into(), response.rect);
                    if app.add_name.trim().is_empty() || app.add_name.len() > 64 {
                        ui.colored_label(RED, "Enter a name (1–64 bytes)");
                    } else if let Err(error) = app.engine.enabled(&Command::AddInput {
                        name: app.add_name.clone(),
                        kind: app.add_kind,
                    }) {
                        ui.colored_label(RED, error.to_string());
                    }
                    ui.label(
                        RichText::new("New sources start with simulated audio muted.")
                            .small()
                            .color(DIM),
                    );
                },
            );

            ui.separator();
            let count = self.engine.project().inputs.len();
            let command = Command::AddInput {
                name: self.add_name.clone(),
                kind: self.add_kind,
            };
            let valid_name = !self.add_name.trim().is_empty() && self.add_name.len() <= 64;
            let enabled = count < 32 && valid_name && self.engine.enabled(&command).is_ok();
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Inputs: {count} / 32")).color(DIM));
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let cancel = ui.add_sized([88.0, 29.0], egui::Button::new("Cancel"));
                    self.automation
                        .insert("dialog.add.cancel".into(), cancel.rect);
                    if cancel.clicked() {
                        created = true;
                    }
                    let response = ui.add_enabled(
                        enabled,
                        egui::Button::new("Add source")
                            .fill(PROGRAM)
                            .min_size(Vec2::new(110.0, 29.0)),
                    );
                    self.automation
                        .insert("dialog.add.confirm".into(), response.rect);
                    if response.clicked() {
                        self.actions.push(Action::Engine(command));
                        created = true;
                    }
                });
            });
        });
        if !open || created {
            self.add_open = false;
        }
    }

    pub(crate) fn input_properties_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut input) = self.properties.clone() else {
            return;
        };
        let mut open = true;
        let mut close_after = false;
        operator_window(ctx, format!("Input {:02}  /  properties", input.id.0), "input.properties", &mut open, Vec2::new(820.0, 540.0))
            .show(ctx, |ui| {
                let body_height = (ui.available_height() - 54.0).max(130.0);
                self.dialog_columns(ui, "dialog.properties", body_height, |app, ui| {
                        for (page, label) in [
                            (InputPage::General, "General"),
                            (InputPage::Audio, "Audio / buses"),
                            (InputPage::Status, "Source status"),
                        ] {
                            let button = sidebar_button(ui, app.input_page == page, label);
                            app.automation.insert(format!("dialog.properties.page.{page:?}"), button.rect);
                            if button.clicked() {
                                app.input_page = page;
                            }
                        }
                        ui.add_space(10.0);
                        ui.label(RichText::new("SYNTHETIC INPUT").small().color(DIM));
                    }, |app, ui| {

                        match app.input_page {
                            InputPage::General => {
                                section_heading(ui, "General", "Identity, source generator and health state");
                                ui.label("Name");
                                let name = ui.add_sized(
                                    [ui.available_width().min(390.0), 25.0],
                                    egui::TextEdit::singleline(&mut input.name).char_limit(64),
                                );
                                app.automation.insert("dialog.properties.name".into(), name.rect);
                                if input.name.trim().is_empty() || input.name.len() > 64 {
                                    ui.colored_label(RED, "Enter an input name (1–64 bytes)");
                                }
                                ui.add_space(8.0);
                                ui.label("Generator");
                                egui::ComboBox::from_id_salt("properties.kind")
                                    .width(250.0)
                                    .selected_text(input.kind.label())
                                    .show_ui(ui, |ui| {
                                        for kind in InputKind::ALL {
                                            ui.selectable_value(&mut input.kind, kind, kind.label());
                                        }
                                    });
                                ui.label(RichText::new(kind_description(input.kind)).small().color(DIM));
                                ui.add_space(9.0);
                                ui.horizontal(|ui| {
                                    ui.label("Generator tint");
                                    ui.color_edit_button_srgb(&mut input.color);
                                });
                                ui.add_space(6.0);
                                let on_program = app.engine.project().program == input.id;
                                ui.add_enabled(
                                    !on_program || !input.online,
                                    egui::Checkbox::new(&mut input.online, "Source online"),
                                )
                                .on_disabled_hover_text(
                                    "Switch away from this active Program source before taking it offline",
                                );
                                if on_program && input.online {
                                    ui.label(
                                        RichText::new("This source is on Program. Switch away before taking it offline.")
                                            .small().color(PREVIEW),
                                    );
                                }
                                ui.label(
                                    RichText::new("Offline sources remain listed and render black with a warning.")
                                        .small()
                                        .color(DIM),
                                );
                                ui.add_space(10.0);
                                ui.label(
                                    RichText::new(
                                        "Source preview · current committed demo frame (384×216)",
                                    )
                                    .small()
                                    .color(DIM),
                                );
                                let id = input.id;
                                let width = ui.available_width().min(320.0);
                                if let Some(texture) = app.textures.get(&id) {
                                    let preview = ui.add(
                                        egui::Image::new(texture)
                                            .fit_to_exact_size(Vec2::new(width, width * 9.0 / 16.0))
                                            .sense(egui::Sense::hover()),
                                    );
                                    app.automation
                                        .insert("dialog.properties.preview".into(), preview.rect);
                                } else {
                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::new(width, width * 9.0 / 16.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0, Color32::BLACK);
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "No frame yet",
                                        egui::FontId::proportional(13.0),
                                        DIM,
                                    );
                                }
                            }
                            InputPage::Audio => {
                                section_heading(
                                    ui,
                                    "Audio routing · simulated",
                                    "Changes apply immediately. Cancel does not undo Audio adjustments.",
                                );
                                if let Some(source) = app.engine.project().input(input.id) {
                                    let mut a = source.audio.clone();
                                    ui.label("Channel gain");
                                    ui.add(
                                        egui::Slider::new(&mut a.gain_db, -60.0..=12.0)
                                            .suffix(" dB"),
                                    );
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut a.muted, "Mute");
                                        ui.checkbox(&mut a.solo, "Solo");
                                    });
                                    ui.add_space(10.0);
                                    ui.label("Bus sends");
                                    ui.horizontal(|ui| {
                                        for (enabled, label) in a.sends.iter_mut().zip([
                                            "Master", "Aux A", "Aux B",
                                        ]) {
                                            ui.checkbox(enabled, label);
                                        }
                                    });
                                    ui.label(
                                        RichText::new("Signal meters are simulated. No audio device or mix-minus DSP is connected.")
                                            .small().color(DIM),
                                    );
                                    if a != source.audio {
                                        app.actions.push(Action::Engine(Command::SetInputAudio(input.id, a)));
                                    }
                                }
                            }
                            InputPage::Status => {
                                section_heading(
                                    ui,
                                    "Source status",
                                    "Read-only state of the current synthetic input",
                                );
                                ui.label(format!("Input ID: {}", input.id.0));
                                let current = app.engine.project().input(input.id);
                                ui.label(format!(
                                    "Kind: {}",
                                    current.map_or("Missing", |source| source.kind.label())
                                ));
                                ui.label(format!(
                                    "State: {}",
                                    if current.is_some_and(|s| s.online) {
                                        "Running · procedural"
                                    } else {
                                        "Offline · black fallback"
                                    }
                                ));
                                ui.label(format!(
                                    "On Program: {}",
                                    app.engine.project().program == input.id
                                ));
                                ui.label(format!(
                                    "Selected for Preview: {}",
                                    app.engine.project().preview == input.id
                                ));
                                ui.add_space(12.0);
                                ui.label(
                                    RichText::new("No camera, file decoder, external media clock or hardware health data is available.")
                                        .color(DIM),
                                );
                            }
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        let cancel = ui.add_sized([88.0, 29.0], egui::Button::new("Cancel / close"));
                        self.automation.insert("dialog.properties.cancel".into(), cancel.rect);
                        if cancel.clicked() { close_after = true; }
                        let valid = !input.name.trim().is_empty() && input.name.len() <= 64
                            && self.engine.enabled(&Command::UpdateInput(input.clone())).is_ok();
                        let response = ui.add_enabled(
                            valid,
                            egui::Button::new("Apply properties")
                                .fill(PROGRAM)
                                .min_size(Vec2::new(130.0, 29.0)),
                        );
                        self.automation.insert("dialog.properties.apply".into(), response.rect);
                        if response.clicked() {
                            if let Some(old) = self.engine.project().input(input.id)
                                && (old.name != input.name || old.kind != input.kind
                                    || old.color != input.color || old.online != input.online)
                            {
                                self.actions.push(Action::Engine(Command::UpdateInput(input.clone())));
                            }
                            close_after = true;
                        }
                        let command = Command::RemoveInput(input.id);
                        let check = self.engine.enabled(&command);
                        let remove = ui.add_enabled(check.is_ok(), egui::Button::new("Remove…").min_size(Vec2::new(85.0, 29.0)))
                            .on_disabled_hover_text(check.err().map(|error| error.to_string()).unwrap_or_default());
                        self.automation.insert("dialog.properties.remove".into(), remove.rect);
                        if remove.clicked() { self.actions.push(Action::RequestRemove(input.id)); }
                    });
                });
            });
        if !open || close_after {
            self.properties = None;
        } else {
            self.properties = Some(input);
        }
    }

    pub(crate) fn settings_dialog(&mut self, ctx: &egui::Context) {
        if !self.settings {
            return;
        }
        let Some(mut p) = self.settings_draft.clone() else {
            self.settings = false;
            return;
        };
        let mut name = self.settings_name.clone();
        let mut open = true;
        let mut close_after = false;
        operator_window(ctx, "Settings  /  production workspace", "settings", &mut open, Vec2::new(820.0, 540.0))
            .show(ctx, |ui| {
                let body_height = (ui.available_height() - 54.0).max(130.0);
                self.dialog_columns(ui, "dialog.settings", body_height, |app, ui| {
                        for (page, label) in [
                            (SettingsPage::Display, "Display / format"),
                            (SettingsPage::Appearance, "Appearance"),
                            (SettingsPage::Workspace, "Workspace"),
                            (SettingsPage::Shortcuts, "Shortcuts"),
                        ] {
                            let button = sidebar_button(ui, app.settings_page == page, label);
                            app.automation.insert(format!("dialog.settings.page.{page:?}"), button.rect);
                            if button.clicked() {
                                app.settings_page = page;
                            }
                        }
                        ui.add_space(12.0);
                        ui.label(RichText::new("UNAVAILABLE").small().color(DIM));
                        for label in ["Outputs / stream", "Recording", "Hardware / audio"] {
                            ui.add_enabled(
                                false,
                                egui::Button::new(label).min_size(Vec2::new(152.0, 27.0)),
                            )
                            .on_disabled_hover_text("This backend is not implemented");
                        }
                    }, |app, ui| {

                        match app.settings_page {
                            SettingsPage::Display => {
                                section_heading(
                                    ui,
                                    "Project / display",
                                    "Source project metadata for the experimental switcher",
                                );
                                ui.label("Project name");
                                ui.add_sized(
                                    [ui.available_width().min(425.0), 25.0],
                                    egui::TextEdit::singleline(&mut name).char_limit(120),
                                );
                                if name.trim().is_empty() || name.len() > 120 {
                                    ui.colored_label(RED, "Enter a project name (1–120 bytes)");
                                }
                                ui.add_space(8.0);
                                ui.label("Target canvas resolution");
                                egui::ComboBox::from_id_salt("settings.canvas")
                                    .width(230.0)
                                    .selected_text(format!("{} × {}", p.width, p.height))
                                    .show_ui(ui, |ui| {
                                        for (w, h) in [
                                            (1280, 720), (1920, 1080),
                                            (2560, 1440), (3840, 2160),
                                        ] {
                                            if ui.selectable_label(
                                                p.width == w && p.height == h,
                                                format!("{w} × {h}"),
                                            ).clicked() {
                                                p.width = w;
                                                p.height = h;
                                            }
                                        }
                                    });
                                ui.add_space(8.0);
                                ui.label("Target frame rate");
                                egui::ComboBox::from_id_salt("settings.rate")
                                    .width(230.0)
                                    .selected_text(format!("{} fps", p.rate.label()))
                                    .show_ui(ui, |ui| {
                                        for (n, d) in [
                                            (24, 1), (25, 1), (30, 1), (50, 1),
                                            (60, 1), (24000, 1001), (30000, 1001),
                                            (60000, 1001),
                                        ] {
                                            if let Ok(rate) = Rate::new(n, d) {
                                                ui.selectable_value(
                                                    &mut p.rate,
                                                    rate,
                                                    format!("{} fps", rate.label()),
                                                );
                                            }
                                        }
                                    });
                                ui.add_space(12.0);
                                ui.label(
                                    RichText::new("Target format is project metadata. Actual demo Program raster is 640×360; thumbnails 384×216.")
                                        .small().color(DIM),
                                );
                            }
                            SettingsPage::Appearance => {
                                section_heading(
                                    ui,
                                    "Appearance",
                                    "Readable controls for the native operator workspace",
                                );
                                ui.checkbox(&mut p.high_contrast, "High contrast text");
                                ui.add_space(8.0);
                                ui.label("Interface scale");
                                ui.add(egui::Slider::new(&mut p.ui_scale, 0.8..=1.5).text("UI scale"));
                                ui.label(
                                    RichText::new("Changes apply after you select Apply. Windows system UI font is used when present.")
                                        .color(DIM).small(),
                                );
                                ui.add_space(15.0);
                                ui.horizontal(|ui| {
                                    ui.colored_label(PREVIEW, "PREVIEW");
                                    ui.label("Select source before taking it");
                                });
                                ui.horizontal(|ui| {
                                    ui.colored_label(PROGRAM, "PROGRAM");
                                    ui.label("Committed output in the offline demo");
                                });
                            }
                            SettingsPage::Workspace => {
                                section_heading(
                                    ui,
                                    "Workspace",
                                    "Dock, resize and restore panels for a production task",
                                );
                                ui.label("Current layout");
                                ui.strong(app.engine.project().workspace.name.as_str());
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    if ui
                                        .selectable_label(
                                            app.settings_layout == Some(false),
                                            "Production layout",
                                        )
                                        .clicked()
                                    {
                                        app.settings_layout = Some(false);
                                    }
                                    if ui
                                        .selectable_label(
                                            app.settings_layout == Some(true),
                                            "Audio focus layout",
                                        )
                                        .clicked()
                                    {
                                        app.settings_layout = Some(true);
                                    }
                                });
                                ui.label(
                                    RichText::new("Select a layout and click Apply. Drag tabs to dock or float; drag the monitor edge to resize.")
                                        .small().color(DIM),
                                );
                                ui.add_space(15.0);
                                if ui.button("Recover last checkpoint…").clicked() {
                                    app.actions.push(Action::Recover);
                                    close_after = true;
                                }
                                ui.label(
                                    RichText::new("Recovery validates the saved project before replacing the current session.")
                                        .small().color(DIM),
                                );
                            }
                            SettingsPage::Shortcuts => {
                                section_heading(
                                    ui,
                                    "Operator shortcuts",
                                    "Switching is suppressed while editing text or dialogs",
                                );
                                ui.checkbox(&mut p.shortcuts, "Enable switching shortcuts");
                                ui.add_space(8.0);
                                for (keys, action) in [
                                    ("1–7", "Select Preview source"),
                                    ("Space / Enter", "CUT / FADE"),
                                    ("B / Esc", "Fade to black / cancel transition"),
                                    ("F1–F4", "Toggle graphics overlays"),
                                    ("Ctrl+I / Ctrl+S", "Add source / save project"),
                                    ("Ctrl+O / Ctrl+Shift+S", "Open / Save As"),
                                ] {
                                    ui.horizontal(|ui| {
                                        ui.monospace(format!("{keys:22}"));
                                        ui.label(action);
                                    });
                                }
                                ui.add_space(8.0);
                                ui.label(RichText::new("Custom key remapping is not yet implemented.").small().color(DIM));
                            }
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    let valid_name = !name.trim().is_empty() && name.len() <= 120;
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        let cancel = ui.add_sized([88.0, 29.0], egui::Button::new("Cancel"));
                        self.automation.insert("dialog.settings.cancel".into(), cancel.rect);
                        if cancel.clicked() { close_after = true; }
                        let response = ui.add_enabled(
                            valid_name && self.engine.enabled(&Command::SetPreferences(p.clone())).is_ok(),
                            egui::Button::new("Apply")
                                .fill(PROGRAM)
                                .min_size(Vec2::new(110.0, 29.0)),
                        );
                        self.automation.insert("dialog.settings.apply".into(), response.rect);
                        if response.clicked() {
                            if let Some(audio) = self.settings_layout {
                                self.actions.push(Action::Workspace(audio));
                            }
                            if p != self.engine.project().preferences {
                                self.actions.push(Action::Engine(Command::SetPreferences(p.clone())));
                            }
                            if name != self.engine.project().name {
                                self.actions.push(Action::Engine(Command::RenameProject(name.clone())));
                            }
                            close_after = true;
                        }
                        ui.add(egui::Label::new(RichText::new("OFFLINE DEMO · outputs disabled").small().color(DIM)).truncate());
                    });
                });
            });
        if !open || close_after {
            self.settings = false;
            self.settings_draft = None;
            self.settings_layout = None;
        } else {
            self.settings_draft = Some(p);
            self.settings_name = name;
        }
    }

    pub(crate) fn remove_input_confirmation(&mut self, ctx: &egui::Context) {
        let Some(id) = self.confirm_remove_input else {
            return;
        };
        egui::Modal::new(egui::Id::new("input.remove.confirm")).show(ctx, |ui| {
            ui.label(format!(
                "Remove input {id:?} from this project? This action can be saved."
            ));
            ui.label(
                RichText::new("Preview and Program sources must be switched away first.")
                    .small()
                    .color(DIM),
            );
            ui.horizontal(|ui| {
                let command = Command::RemoveInput(id);
                let remove = ui.add_enabled(
                    self.engine.enabled(&command).is_ok(),
                    egui::Button::new("Remove").fill(Color32::from_rgb(125, 42, 49)),
                );
                self.automation
                    .insert("dialog.remove.confirm".into(), remove.rect);
                if remove.clicked() {
                    self.actions.push(Action::Engine(command));
                    self.confirm_remove_input = None;
                    self.properties = None;
                }
                let cancel = ui.button("Cancel");
                self.automation
                    .insert("dialog.remove.cancel".into(), cancel.rect);
                if cancel.clicked() {
                    self.confirm_remove_input = None;
                }
            });
        });
    }
}
