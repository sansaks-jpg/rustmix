//! Add Input presentation and draft isolation. Only the engine commits sources.
use super::dialogs::{ADD_CATEGORIES, GENERATOR_SAMPLES, VIDEO_TAB, light_form_frame};
use super::*;

// Preserve existing automation ordinals; presentation order is explicit.
pub(super) const CATEGORY_ORDER: [usize; 26] = [
    VIDEO_TAB, 0, 21, 2, 22, 23, 5, 7, 8, 24, 1, 9, 10, 11, 12, 13, 14, 16, 17, 18, 19, 25, 3, 4,
    6, 15,
];

pub(super) fn unavailable_reason(tab: usize) -> Option<&'static str> {
    match tab {
        2 => Some("File-list inputs are unavailable. Use individual video or audio files."),
        5 => Some("Network stream inputs are unavailable. No connection will be opened."),
        7 => Some("Instant Replay capture and storage are unavailable."),
        8 => Some("Image-sequence and stinger file inputs are unavailable."),
        9 => Some("Photo-folder slideshows are unavailable. Use a single PNG image."),
        10 => Some("PowerPoint presentation inputs are unavailable."),
        14 => Some(
            "Title template inputs are unavailable. The Generators library includes an original lower-third sample.",
        ),
        16 => Some("Virtual-set inputs are unavailable. Studio pattern is a procedural generator."),
        17 => Some("Web Browser inputs are unavailable. No page will be opened."),
        18 => Some("Video Call inputs are unavailable. No guest connection will be started."),
        19 => Some(
            "Zoom inputs are unavailable. No plugin is installed or downloaded by this dialog.",
        ),
        21 => Some("DVD playback inputs are unavailable."),
        22 => Some("Camera and capture-card inputs are unavailable. No camera will be opened."),
        23 => Some(
            "NDI, OMT and desktop-capture inputs are unavailable. No device or network discovery will be started.",
        ),
        24 => Some("Video Delay capture and buffering are unavailable."),
        25 => Some("Telestrator inputs are unavailable. No web controller will be enabled."),
        0 | 1 | 3 | 4 | 6 | 11 | 12 | 13 | 15 | VIDEO_TAB => None,
        _ => Some("This input category is unavailable."),
    }
}

pub(super) fn generator_category(tab: usize) -> bool {
    matches!(tab, 0 | 3 | 4 | 6 | 11 | 15)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SourceIcon {
    Film,
    Disc,
    List,
    Camera,
    Display,
    Network,
    Replay,
    Image,
    Colour,
    Speaker,
    Microphone,
    Text,
    Clock,
    Shapes,
    Studio,
    Browser,
    Pen,
}

pub(super) fn source_icon(tab: usize) -> SourceIcon {
    match tab {
        VIDEO_TAB => SourceIcon::Film,
        21 => SourceIcon::Disc,
        0 | 2 => SourceIcon::List,
        22 | 18 | 19 => SourceIcon::Camera,
        23 => SourceIcon::Display,
        5 => SourceIcon::Network,
        7 | 24 => SourceIcon::Replay,
        1 | 8 | 9 => SourceIcon::Image,
        11 | 6 => SourceIcon::Colour,
        12 => SourceIcon::Speaker,
        13 => SourceIcon::Microphone,
        10 | 14 => SourceIcon::Text,
        15 => SourceIcon::Clock,
        3 => SourceIcon::Shapes,
        4 | 16 => SourceIcon::Studio,
        17 => SourceIcon::Browser,
        _ => SourceIcon::Pen,
    }
}

/// Original vector marks, drawn from generic source conventions.
pub(super) fn draw_source_icon(p: &egui::Painter, rect: Rect, icon: SourceIcon, color: Color32) {
    let stroke = Stroke::new(1.2, color);
    let at = |x, y| Pos2::new(rect.left() + x, rect.top() + y);
    let line = |a, b| {
        p.line_segment([a, b], stroke);
    };
    let outline = |r| {
        p.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Inside);
    };
    match icon {
        SourceIcon::Film => {
            outline(rect);
            for y in [2.0, 6.0, 10.0] {
                line(at(1.0, y), at(3.0, y));
                line(at(9.0, y), at(11.0, y));
            }
            p.add(egui::Shape::convex_polygon(
                vec![at(5.0, 3.0), at(8.0, 6.0), at(5.0, 9.0)],
                color,
                Stroke::NONE,
            ));
        }
        SourceIcon::Disc => {
            p.circle_stroke(rect.center(), 5.0, stroke);
            p.circle_stroke(rect.center(), 1.4, stroke);
        }
        SourceIcon::List => {
            for y in [2.0, 6.0, 10.0] {
                p.circle_filled(at(1.0, y), 0.8, color);
                line(at(4.0, y), at(12.0, y));
            }
        }
        SourceIcon::Camera => {
            outline(Rect::from_min_max(at(0.0, 2.0), at(8.0, 10.0)));
            line(at(8.0, 5.0), at(12.0, 2.0));
            line(at(12.0, 2.0), at(12.0, 10.0));
            line(at(12.0, 10.0), at(8.0, 7.0));
        }
        SourceIcon::Display | SourceIcon::Browser => {
            outline(Rect::from_min_max(at(0.0, 0.0), at(12.0, 9.0)));
            if icon == SourceIcon::Browser {
                line(at(0.0, 3.0), at(12.0, 3.0));
            } else {
                line(at(6.0, 9.0), at(6.0, 12.0));
                line(at(3.0, 12.0), at(9.0, 12.0));
            }
        }
        SourceIcon::Network => {
            p.circle_filled(at(6.0, 2.0), 1.7, color);
            line(at(6.0, 4.0), at(6.0, 7.0));
            line(at(1.0, 7.0), at(11.0, 7.0));
            for x in [1.0, 6.0, 11.0] {
                line(at(x, 7.0), at(x, 10.0));
                p.circle_filled(at(x, 11.0), 1.0, color);
            }
        }
        SourceIcon::Replay => {
            p.circle_stroke(rect.center(), 4.5, stroke);
            line(at(6.0, 6.0), at(6.0, 3.0));
            line(at(6.0, 6.0), at(9.0, 7.0));
            line(at(1.0, 0.0), at(1.0, 4.0));
            line(at(1.0, 4.0), at(5.0, 4.0));
        }
        SourceIcon::Image => {
            outline(rect);
            p.circle_filled(at(8.5, 3.5), 1.2, color);
            line(at(1.0, 10.0), at(4.0, 5.0));
            line(at(4.0, 5.0), at(7.0, 9.0));
            line(at(7.0, 9.0), at(10.0, 6.0));
        }
        SourceIcon::Colour => {
            for (x, c) in [
                (0.0, Color32::from_rgb(234, 85, 80)),
                (4.0, Color32::from_rgb(80, 203, 130)),
                (8.0, Color32::from_rgb(85, 158, 230)),
            ] {
                p.rect_filled(
                    Rect::from_min_size(at(x, 1.0), Vec2::new(3.0, 10.0)),
                    0.0,
                    c,
                );
            }
        }
        SourceIcon::Speaker => {
            outline(Rect::from_min_max(at(0.0, 4.0), at(3.0, 8.0)));
            line(at(3.0, 4.0), at(7.0, 1.0));
            line(at(7.0, 1.0), at(7.0, 11.0));
            line(at(7.0, 11.0), at(3.0, 8.0));
            line(at(10.0, 3.0), at(12.0, 6.0));
            line(at(12.0, 6.0), at(10.0, 9.0));
        }
        SourceIcon::Microphone => {
            p.rect_stroke(
                Rect::from_min_max(at(4.0, 0.0), at(8.0, 7.0)),
                2.0,
                stroke,
                egui::StrokeKind::Inside,
            );
            line(at(2.0, 5.0), at(2.0, 8.0));
            line(at(2.0, 8.0), at(6.0, 10.0));
            line(at(6.0, 10.0), at(10.0, 8.0));
            line(at(10.0, 8.0), at(10.0, 5.0));
            line(at(6.0, 10.0), at(6.0, 12.0));
            line(at(3.0, 12.0), at(9.0, 12.0));
        }
        SourceIcon::Text => {
            line(at(1.0, 1.0), at(11.0, 1.0));
            line(at(6.0, 1.0), at(6.0, 11.0));
        }
        SourceIcon::Clock => {
            p.circle_stroke(rect.center(), 5.0, stroke);
            line(at(6.0, 6.0), at(6.0, 2.0));
            line(at(6.0, 6.0), at(9.0, 6.0));
        }
        SourceIcon::Shapes => {
            p.circle_stroke(at(4.0, 4.0), 3.0, stroke);
            outline(Rect::from_min_max(at(6.0, 6.0), at(12.0, 12.0)));
        }
        SourceIcon::Studio => {
            outline(rect);
            line(at(0.0, 9.0), at(12.0, 9.0));
            line(at(4.0, 0.0), at(4.0, 9.0));
            line(at(8.0, 0.0), at(8.0, 9.0));
        }
        SourceIcon::Pen => {
            line(at(1.0, 11.0), at(10.0, 2.0));
            line(at(3.0, 12.0), at(12.0, 3.0));
            line(at(10.0, 2.0), at(12.0, 3.0));
        }
    }
}

pub(super) fn page_header(ui: &mut egui::Ui, title: &str) {
    egui::Frame::new()
        .fill(Color32::from_rgb(10, 15, 20))
        .inner_margin(egui::Margin::symmetric(8, 8))
        .show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 2.0).max(80.0));
            ui.label(RichText::new(title).size(13.0).color(Color32::WHITE));
        });
}

impl RustMixApp {
    /// The sole category-selection path for sidebar and quick-menu actions.
    pub(crate) fn select_add_category(&mut self, tab: usize) {
        let Some((label, kind, _)) = ADD_CATEGORIES.get(tab).copied() else {
            return;
        };
        if tab == self.add_tab {
            return;
        }
        self.add_token = self.add_token.wrapping_add(1);
        self.add_loading = false;
        self.add_image = None;
        self.add_preview = None;
        self.add_png_path.clear();
        self.add_error = None;
        self.cancel_audio_add();
        self.retire_video_category();
        self.add_tab = tab;
        self.add_reveal = true;
        self.add_kind = kind;
        self.add_name = if tab == 0 {
            "New synthetic input".into()
        } else {
            label.into()
        };
    }

    fn add_placement_error(&self) -> Option<String> {
        let count = self.engine.project().inputs.len();
        if self.add_number == 0 || self.add_number > count.saturating_add(1) {
            return Some("Choose an available input number.".into());
        }
        let command = Command::AddInputAt {
            name: "New input".into(),
            kind: self.add_kind,
            config: InputConfig::default(),
            index: self.add_number.saturating_sub(1),
        };
        if let Err(error) = self.engine.enabled(&command) {
            return Some(error.to_string());
        }
        None
    }

    pub(crate) fn add_disabled_reason(&self) -> Option<String> {
        if let Some(reason) = unavailable_reason(self.add_tab) {
            return Some(reason.into());
        }
        if let Some(reason) = self.add_placement_error() {
            return Some(reason);
        }
        if self.add_loading {
            return Some("Wait for the image check to finish.".into());
        }
        if self.add_tab == VIDEO_TAB && !self.video_add_ready() {
            return Some("Choose a video file and wait for its check to finish.".into());
        }
        if matches!(self.add_tab, 12 | 13) && !self.audio_add_ready() {
            return Some(
                "Choose a supported audio source and wait for its check to finish.".into(),
            );
        }
        if self.add_tab == 1
            && !self.add_image.as_ref().is_some_and(|image| {
                image.media_path.as_deref() == Some(self.add_png_path.as_str())
                    && image.custom_rgba.is_some()
            })
        {
            return Some("Choose a PNG image and check it before adding.".into());
        }
        if generator_category(self.add_tab) {
            let name = self.add_name.trim();
            if name.is_empty() || name.len() > 64 || name.chars().any(char::is_control) {
                return Some("Enter a source name of up to 64 UTF-8 bytes.".into());
            }
        }
        None
    }

    pub(crate) fn check_add_png(&mut self) {
        if self.add_tab != 1 || self.add_loading {
            return;
        }
        self.add_image = None;
        self.add_preview = None;
        self.add_error = None;
        let path = PathBuf::from(&self.add_png_path);
        if !path
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(|x| x.eq_ignore_ascii_case("png"))
        {
            self.add_error = Some("Choose a PNG file. Use Video for MP4 files.".into());
            return;
        }
        self.add_loading = self.media_loader.enqueue(
            path,
            media::MediaTarget::AddPreview {
                token: self.add_token,
            },
        );
        if !self.add_loading {
            self.add_error = Some("Image check is busy. Try again shortly.".into());
        }
    }

    pub(crate) fn add_source_page(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Media pages own their validation status. Engine blockers remain inline.
        if unavailable_reason(self.add_tab).is_none() {
            let reason = self.add_error.clone().or_else(|| {
                if matches!(self.add_tab, 1 | 12 | 13 | VIDEO_TAB) {
                    self.add_placement_error()
                } else {
                    self.add_disabled_reason()
                }
            });
            if let Some(reason) = reason {
                let message = ui
                    .add(egui::Label::new(RichText::new(&reason).color(FORM_TEXT)).truncate())
                    .on_hover_text(&reason);
                self.automation
                    .insert("dialog.add.feedback".into(), message.rect);
            }
        }
        if let Some(reason) = unavailable_reason(self.add_tab) {
            let label = ADD_CATEGORIES.get(self.add_tab).map_or("Input", |x| x.0);
            page_header(ui, label);
            light_form_frame(ui, 12, |ui| {
                let state = ui.label(RichText::new("Unavailable").strong().color(FORM_TEXT));
                self.automation
                    .insert("dialog.add.unavailable".into(), state.rect);
                ui.label(RichText::new(reason).color(FORM_TEXT));
            });
        } else if self.add_tab == VIDEO_TAB {
            self.video_add_page(ui, ctx);
        } else if matches!(self.add_tab, 12 | 13) {
            self.audio_add_page(ui, ctx);
        } else if self.add_tab == 1 {
            self.png_add_page(ui, ctx);
        } else {
            self.generator_add_page(ui);
        }
    }

    fn png_add_page(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        page_header(ui, "Select a PNG still image.");
        egui::Frame::new().inner_margin(8).show(ui, |ui| {
            ui.horizontal(|ui| {
                let width = (ui.available_width() - 94.0).max(120.0);
                let path = ui
                    .add_enabled_ui(!self.add_loading, |ui| {
                        ui.add_sized(
                            [width, 26.0],
                            egui::TextEdit::singleline(&mut self.add_png_path)
                                .char_limit(4096)
                                .hint_text("PNG image file"),
                        )
                    })
                    .inner;
                self.automation
                    .insert("dialog.add.png.path".into(), path.rect);
                // Legacy inspection ID remains an alias for the actual PNG path.
                self.automation.insert("dialog.add.name".into(), path.rect);
                if path.changed() {
                    self.add_image = None;
                    self.add_preview = None;
                    self.add_error = None;
                }
                let browse = ui.add_enabled(
                    !self.add_loading,
                    egui::Button::new("Browse…").min_size(Vec2::new(90.0, 26.0)),
                );
                self.automation
                    .insert("dialog.add.png.browse".into(), browse.rect);
                if browse.clicked() && self.smoke.is_none() {
                    self.start_png_picker(media::MediaTarget::AddPreview {
                        token: self.add_token,
                    });
                }
                if path.lost_focus() && ctx.input(|x| x.key_pressed(egui::Key::Enter)) {
                    self.check_add_png();
                }
            });
            let size = Vec2::new(
                ui.available_width(),
                (ui.available_height() - 84.0).max(84.0),
            );
            let (preview, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter().rect_filled(preview, 0.0, Color32::WHITE);
            ui.painter().rect_stroke(
                preview,
                0.0,
                Stroke::new(1.0, Color32::from_gray(190)),
                egui::StrokeKind::Inside,
            );
            if let Some((_, texture)) = &self.add_preview {
                let size = Vec2::new(
                    (preview.width() - 16.0)
                        .min((preview.height() - 16.0) * 16.0 / 9.0)
                        .max(1.0),
                    0.0,
                );
                let image = Rect::from_center_size(
                    preview.center(),
                    Vec2::new(size.x, size.x * 9.0 / 16.0),
                );
                ui.painter().image(
                    texture.id(),
                    image,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                self.automation.insert("dialog.add.preview".into(), image);
            } else {
                ui.painter().text(
                    preview.center(),
                    egui::Align2::CENTER_CENTER,
                    "No image selected",
                    egui::FontId::proportional(13.0),
                    FORM_TEXT,
                );
            }
            self.automation
                .insert("dialog.add.png.preview".into(), preview);
            ui.horizontal(|ui| {
                let check = ui.add_enabled(
                    !self.add_loading && !self.add_png_path.trim().is_empty(),
                    egui::Button::new("Check image"),
                );
                self.automation
                    .insert("dialog.add.png.validate".into(), check.rect);
                if check.clicked() {
                    self.check_add_png();
                }
                let clear = ui.add_enabled(
                    !self.add_loading && !self.add_png_path.is_empty(),
                    egui::Button::new("Clear selection"),
                );
                self.automation
                    .insert("dialog.add.png.clear".into(), clear.rect);
                if clear.clicked() {
                    self.add_png_path.clear();
                    self.add_image = None;
                    self.add_preview = None;
                    self.add_error = None;
                }
            });
            let status = if self.add_loading {
                "Checking image…".into()
            } else if let Some(image) = &self.add_image {
                image
                    .image_size
                    .map_or("PNG ready".into(), |[w, h]| format!("{w}×{h} · PNG ready"))
            } else {
                "PNG only · maximum 32 MiB / 4096 pixels. Clear selection keeps the file.".into()
            };
            ui.add(egui::Label::new(RichText::new(&status).color(FORM_TEXT)).truncate())
                .on_hover_text(status);
        });
    }

    fn generator_add_page(&mut self, ui: &mut egui::Ui) {
        page_header(
            ui,
            if self.add_tab == 11 {
                "Colour generators"
            } else {
                "Procedural generator"
            },
        );
        light_form_frame(ui, 10, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name");
                let width = ui.available_width();
                let name = ui.add_sized(
                    [width, 26.0],
                    egui::TextEdit::singleline(&mut self.add_name).char_limit(64),
                );
                self.automation.insert("dialog.add.name".into(), name.rect);
                if name.changed() {
                    self.add_error = None;
                }
            });
            if self.add_tab == 11 {
                ui.label("Select an available generator:");
                for (key, label, kind) in [
                    ("gradient", "Colour gradient", InputKind::Color),
                    ("bars", "Reference bars", InputKind::Bars),
                ] {
                    let r = ui.radio_value(&mut self.add_kind, kind, label);
                    self.automation
                        .insert(format!("dialog.add.colour.{key}"), r.rect);
                    if r.changed() {
                        self.add_preview = None;
                    }
                }
                ui.label("Solid colour and transparent inputs are unavailable.");
            } else if self.add_tab == 0 {
                for (index, (name, kind)) in GENERATOR_SAMPLES.into_iter().enumerate() {
                    let r = ui.selectable_label(self.add_kind == kind, name);
                    self.automation
                        .insert(format!("dialog.add.generator.{index}"), r.rect);
                    if r.clicked() {
                        self.add_kind = kind;
                        self.add_name = name.into();
                        self.add_preview = None;
                    }
                }
            }
            if let Some((_, texture)) = &self.add_preview {
                let w = ui.available_width().clamp(100.0, 320.0);
                let r = ui.add(egui::Image::new((
                    texture.id(),
                    Vec2::new(w, w * 9.0 / 16.0),
                )));
                self.automation.insert("dialog.add.preview".into(), r.rect);
            }
            ui.label(
                RichText::new("Generated picture · no file or device is opened.").color(FORM_TEXT),
            );
        });
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn frame(app: &mut RustMixApp, ctx: &egui::Context, size: Vec2, events: Vec<egui::Event>) {
        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ui| app.frame_ui(ui),
        );
    }

    fn click(app: &mut RustMixApp, ctx: &egui::Context, id: &str) {
        let size = Vec2::new(1600.0, 900.0);
        for _ in 0..4 {
            frame(app, ctx, size, vec![]);
        }
        let point = app
            .automation
            .get(id)
            .expect("actual visible control")
            .center();
        assert!(
            ctx.viewport_rect().contains(point),
            "target outside viewport: {id}"
        );
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                size,
                vec![
                    egui::Event::PointerMoved(point),
                    egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }

    fn key(app: &mut RustMixApp, ctx: &egui::Context, key: egui::Key, modifiers: egui::Modifiers) {
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                Vec2::new(1600.0, 900.0),
                vec![egui::Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed,
                    repeat: false,
                    modifiers,
                }],
            );
        }
    }

    fn edit(app: &mut RustMixApp, ctx: &egui::Context, id: &str, text: &str) {
        click(app, ctx, id);
        key(
            app,
            ctx,
            egui::Key::A,
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..egui::Modifiers::NONE
            },
        );
        frame(
            app,
            ctx,
            Vec2::new(1600.0, 900.0),
            vec![egui::Event::Text(text.into())],
        );
    }

    fn fixture(path: &Path, color: [u8; 4]) {
        image::save_buffer(path, &color.repeat(16 * 9), 16, 9, image::ColorType::Rgba8).unwrap();
    }

    fn wait_image(app: &mut RustMixApp) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            app.poll_media();
            if !app.add_loading && !app.media_loader.busy() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "bounded PNG worker did not complete"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn validated_png_cannot_be_committed_as_colour_title_or_a_dotted_generator_name() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("original.png");
        fixture(&path, [170, 30, 50, 255]);
        let original_file = std::fs::read(&path).unwrap();
        for target in [0, 11, 14] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            let before = app.engine.project().clone();
            app.dispatch(Action::Add, &ctx);
            app.select_add_category(1);
            app.add_png_path = path.display().to_string();
            click(&mut app, &ctx, "dialog.add.png.validate");
            wait_image(&mut app);
            assert!(app.add_image.is_some());
            assert!(app.add_disabled_reason().is_none());
            click(&mut app, &ctx, &format!("dialog.add.category.{target}"));
            assert_eq!(app.add_tab, target);
            assert!(app.add_image.is_none() && app.add_png_path.is_empty());
            if target == 14 {
                click(&mut app, &ctx, "dialog.add.confirm");
                assert!(app.add_open);
                assert!(!app.request_add_input());
                assert_eq!(*app.engine.project(), before);
            } else {
                edit(&mut app, &ctx, "dialog.add.name", "Studio A.1");
                click(&mut app, &ctx, "dialog.add.confirm");
                let input = app.engine.project().inputs.last().unwrap();
                assert_eq!(input.name, "Studio A.1");
                assert_eq!(
                    input.kind,
                    if target == 0 {
                        InputKind::Grid
                    } else {
                        InputKind::Color
                    }
                );
                assert!(
                    input.config.media_path.is_none()
                        && input.config.video_source.is_none()
                        && input.config.audio_source.is_none()
                );
                assert_eq!(
                    (app.engine.project().program, app.engine.project().preview),
                    (before.program, before.preview)
                );
                assert!(!app.add_open);
            }
        }
        assert_eq!(std::fs::read(path).unwrap(), original_file);
    }

    #[test]
    fn editing_png_path_retires_its_pixels_until_the_new_file_is_checked() {
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first.png");
        let second = directory.path().join("second.png");
        fixture(&first, [180, 0, 0, 255]);
        fixture(&second, [0, 180, 0, 255]);
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.dispatch(Action::Add, &ctx);
        app.select_add_category(1);
        app.add_png_path = first.display().to_string();
        click(&mut app, &ctx, "dialog.add.png.validate");
        wait_image(&mut app);
        edit(
            &mut app,
            &ctx,
            "dialog.add.png.path",
            second.to_str().unwrap(),
        );
        assert!(app.add_image.is_none() && app.add_preview.is_none());
        let before = app.engine.project().clone();
        click(&mut app, &ctx, "dialog.add.confirm");
        assert_eq!(*app.engine.project(), before);
        click(&mut app, &ctx, "dialog.add.png.validate");
        wait_image(&mut app);
        click(&mut app, &ctx, "dialog.add.confirm");
        let input = app.engine.project().inputs.last().unwrap();
        assert_eq!(
            input.config.media_path.as_deref(),
            second.canonicalize().unwrap().to_str()
        );
        assert_eq!(
            app.input_fx
                .get(&input.id)
                .unwrap()
                .custom_rgba
                .as_ref()
                .unwrap()
                .2
                .first(),
            Some(&[0, 180, 0, 255])
        );
        assert_eq!(
            (app.engine.project().program, app.engine.project().preview),
            (before.program, before.preview)
        );
    }

    #[test]
    fn late_png_result_cannot_repopulate_a_different_category() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("late.png");
        fixture(&path, [10, 20, 30, 255]);
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.dispatch(Action::Add, &ctx);
        app.select_add_category(1);
        app.add_png_path = path.display().to_string();
        app.check_add_png();
        let before = app.engine.project().clone();
        app.select_add_category(11);
        wait_image(&mut app);
        assert_eq!(app.add_tab, 11);
        assert!(app.add_image.is_none() && app.add_png_path.is_empty());
        assert_eq!(*app.engine.project(), before);
    }

    #[test]
    fn unavailable_pages_are_inspectable_and_never_commit_dummy_sources() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let before = app.engine.project().clone();
        app.dispatch(Action::Add, &ctx);
        for tab in [2, 5, 7, 8, 9, 10, 14, 16, 17, 18, 19, 21, 22, 23, 24, 25] {
            app.select_add_category(tab);
            click(&mut app, &ctx, "dialog.add.confirm");
            assert!(app.automation.contains_key("dialog.add.unavailable"));
            assert!(app.add_disabled_reason().is_some());
            assert!(!app.request_add_input());
            assert_eq!(*app.engine.project(), before);
        }
        click(&mut app, &ctx, "dialog.add.cancel");
        assert!(!app.add_open);
    }

    #[test]
    fn fields_and_footer_fit_compact_and_large_forms_without_unrelated_generator_controls() {
        for size in [
            Vec2::new(734.0, 466.0),
            Vec2::new(1100.0, 700.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(1917.0, 1168.0),
        ] {
            for (tab, field) in [
                (1, "dialog.add.png.path"),
                (11, "dialog.add.name"),
                (14, "dialog.add.unavailable"),
                (0, "dialog.add.name"),
                (12, "dialog.add.audio.path"),
                (13, "dialog.add.audio.capture"),
            ] {
                let mut app = RustMixApp::headless();
                let ctx = egui::Context::default();
                app.theme(&ctx);
                app.dispatch(Action::Add, &ctx);
                app.select_add_category(tab);
                for _ in 0..5 {
                    frame(&mut app, &ctx, size, vec![]);
                }
                app.validate_dialog_geometry(&ctx, "dialog.add").unwrap();
                let page = *app.automation.get("dialog.add.page").unwrap();
                assert!(
                    page.expand(1.0)
                        .contains_rect(*app.automation.get(field).unwrap()),
                    "{tab} {size:?}"
                );
                if tab == 1 {
                    for key in ["path", "browse", "preview", "validate", "clear"] {
                        assert!(
                            page.expand(1.0).contains_rect(
                                *app.automation
                                    .get(&format!("dialog.add.png.{key}"))
                                    .unwrap()
                            ),
                            "{key} {size:?}"
                        );
                    }
                    let path = app.automation.get("dialog.add.png.path").unwrap();
                    let browse = app.automation.get("dialog.add.png.browse").unwrap();
                    assert!(
                        path.right() < browse.left()
                            && (path.center().y - browse.center().y).abs() < 1.0
                    );
                    assert!(!app.automation.contains_key("dialog.add.generator.0"));
                }
            }
        }
    }

    #[test]
    fn full_capacity_and_an_active_transition_disable_add_without_changing_program() {
        for full in [true, false] {
            let mut app = RustMixApp::headless();
            let ctx = egui::Context::default();
            app.theme(&ctx);
            if full {
                for n in 7..32 {
                    app.execute(Command::AddInput {
                        name: format!("Generator {n}"),
                        kind: InputKind::Grid,
                    });
                }
            } else {
                app.execute(Command::SetManualProgress(0.4));
            }
            app.dispatch(Action::Add, &ctx);
            app.select_add_category(0);
            let before = app.engine.project().clone();
            assert!(app.add_disabled_reason().is_some());
            click(&mut app, &ctx, "dialog.add.confirm");
            assert_eq!(*app.engine.project(), before);
            assert!(app.add_open);
            if !full {
                assert!(app.engine.transition().is_some());
            }
        }
    }

    #[test]
    fn sidebar_supports_keyboard_selection_and_source_icons_follow_semantics() {
        assert_eq!(source_icon(VIDEO_TAB), SourceIcon::Film);
        assert_eq!(source_icon(1), SourceIcon::Image);
        assert_eq!(source_icon(12), SourceIcon::Speaker);
        assert_eq!(source_icon(13), SourceIcon::Microphone);
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.dispatch(Action::Add, &ctx);
        let before = app.engine.project().clone();
        click(&mut app, &ctx, "dialog.add.category.1");
        key(&mut app, &ctx, egui::Key::Tab, egui::Modifiers::NONE);
        key(&mut app, &ctx, egui::Key::Enter, egui::Modifiers::NONE);
        assert_eq!(
            app.add_tab, 9,
            "Tab then Enter selects the Photos row following Image"
        );
        assert!(app.automation.contains_key("dialog.add.unavailable"));
        assert_eq!(*app.engine.project(), before);
    }
}
