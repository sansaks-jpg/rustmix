//! Independently implemented category, search and operator appearance controls.
use super::*;

fn display_label(ui: &mut egui::Ui, label: &str, width: f32) -> Rect {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 26.0), Sense::hover());
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    child
        .add(egui::Label::new(label).truncate())
        .on_hover_text(label);
    rect
}

impl RustMixApp {
    pub(crate) fn preview_color(&self) -> Color32 {
        let [r, g, b] = self.engine.project().preferences.preview_color;
        Color32::from_rgb(r, g, b)
    }
    pub(crate) fn program_color(&self) -> Color32 {
        let [r, g, b] = self.engine.project().preferences.program_color;
        Color32::from_rgb(r, g, b)
    }
    pub(crate) fn toggle_audio_pane(&mut self) {
        if let Some(complete) = self.audio_restore.take() {
            self.dock = complete;
        } else if let Some(path) = self.dock.find_tab(&Panel::Audio) {
            let complete = self.dock.clone();
            if self.dock.remove_tab(path).is_some() {
                self.audio_restore = Some(complete);
            }
        }
    }
    pub(crate) fn category_filter(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 3.0;
            let all = ui.selectable_label(self.category.is_none(), "All");
            self.automation
                .insert("inputs.category.all".into(), all.rect);
            if all.clicked() {
                self.category = None;
            }
            let categories = self.engine.project().preferences.categories.clone();
            for category in categories.iter().filter(|c| c.id < 6) {
                let [r, g, b] = category.color;
                let response = ui
                    .add(
                        egui::Button::new("")
                            .fill(Color32::from_rgb(r, g, b))
                            .stroke(Stroke::new(
                                if self.category == Some(category.id) {
                                    2.0
                                } else {
                                    1.0
                                },
                                Color32::WHITE,
                            ))
                            .min_size(Vec2::new(20.0, 20.0)),
                    )
                    .on_hover_text(&category.name);
                self.automation
                    .insert(format!("inputs.category.{}", category.id), response.rect);
                if response.clicked() {
                    self.category = Some(category.id);
                }
            }
            let selected = self
                .category
                .and_then(|id| categories.iter().find(|c| c.id == id))
                .map(|c| c.name.clone());
            let menu = ui.menu_button(
                selected
                    .filter(|_| self.category.is_some_and(|id| id >= 6))
                    .unwrap_or_else(|| "Custom".into()),
                |ui| {
                    for category in categories.iter().filter(|c| c.id >= 6) {
                        let r =
                            ui.selectable_label(self.category == Some(category.id), &category.name);
                        self.automation
                            .insert(format!("inputs.category.{}", category.id), r.rect);
                        if r.clicked() {
                            self.category = Some(category.id);
                            ui.close();
                        }
                    }
                },
            );
            self.automation
                .insert("inputs.category.custom".into(), menu.response.rect);
            let search = vector_icon_button(
                ui,
                VectorIcon::Search,
                Vec2::new(22.0, 20.0),
                !self.input_filter.is_empty(),
                None,
            )
            .on_hover_text("Input Search");
            self.automation.insert("inputs.search".into(), search.rect);
            if search.clicked() {
                self.search_draft = self.input_filter.clone();
                self.search_open = true;
            }
            if self.category.is_some() || !self.input_filter.is_empty() {
                let clear = ui.small_button("Clear");
                self.automation
                    .insert("inputs.filter.clear".into(), clear.rect);
                if clear.clicked() {
                    self.category = None;
                    self.input_filter.clear();
                }
            }
            if !self.input_filter.is_empty() {
                ui.label(format!("Search: {}", self.input_filter));
            }
        });
    }
    pub(crate) fn search_dialog(&mut self, ctx: &egui::Context) {
        if !self.search_open {
            return;
        }
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        dialogs::operator_window(
            ctx,
            "Input Search",
            "dialog.search",
            &mut open,
            Vec2::new(560.0, 300.0),
        )
        .show(ctx, |ui| {
            ui.label(
                "Filter input titles, numbers or source types. Program and Preview stay selected.",
            );
            let entry = ui.add(
                egui::TextEdit::singleline(&mut self.search_draft)
                    .char_limit(48)
                    .hint_text("Title or keyword"),
            );
            self.automation
                .insert("dialog.search.text".into(), entry.rect);
            if !ctx.egui_wants_keyboard_input() {
                entry.request_focus();
            }
            ui.separator();
            ui.horizontal(|ui| {
                let ok = ui.button("OK");
                self.automation
                    .insert("dialog.search.apply".into(), ok.rect);
                apply = ok.clicked();
                let close = ui.button("Cancel");
                self.automation
                    .insert("dialog.search.cancel".into(), close.rect);
                cancel = close.clicked();
            });
        });
        if apply {
            self.input_filter = self.search_draft.trim().to_owned();
            self.search_open = false;
        } else if cancel || !open {
            self.search_open = false;
        }
    }
    pub(crate) fn appearance_controls(&mut self, ui: &mut egui::Ui, draft: &mut Preferences) {
        let theme = egui::ComboBox::from_id_salt("settings.theme")
            .selected_text(match draft.ui_theme {
                UiTheme::Dark => "RustMix Dark",
                UiTheme::Light => "RustMix Light",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut draft.ui_theme, UiTheme::Dark, "RustMix Dark");
                ui.selectable_value(&mut draft.ui_theme, UiTheme::Light, "RustMix Light");
            });
        self.automation
            .insert("dialog.settings.theme".into(), theme.response.rect);
    }

    pub(crate) fn display_settings(
        &mut self,
        ui: &mut egui::Ui,
        draft: &mut Preferences,
        field_width: f32,
    ) {
        let viewport = ui.ctx().viewport_rect().size();
        let label_width = (viewport.x * 172.0 / 1912.0).clamp(104.0, 172.0);
        let compact = viewport.y < 600.0;
        egui::Grid::new("settings.display.primary").num_columns(2)
            .spacing([12.0, 6.0]).show(ui, |ui| {
                let label = display_label(ui, "Theme:", label_width);
                self.automation.insert("dialog.settings.label.theme".into(), label);
                self.appearance_controls(ui, draft);
                ui.end_row();
                let label = display_label(ui, "Preview Colour:", label_width);
                self.automation.insert("dialog.settings.label.previewColor".into(), label);
                let swatch = (viewport.x * 30.0 / 1912.0).clamp(22.0, 30.0);
                ui.allocate_ui_with_layout(egui::vec2(field_width.max(208.0),swatch.max(26.0)),
                    egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let preview = ui.scope(|ui| {
                        ui.spacing_mut().interact_size = Vec2::splat(swatch);
                        ui.color_edit_button_srgb(&mut draft.preview_color)
                    }).inner;
                    self.automation.insert("dialog.settings.previewColor".into(), preview.rect);
                    ui.add_space((viewport.x * 76.0 / 1912.0 - 4.0).clamp(16.0, 72.0));
                    let label = display_label(ui, "Output Colour:", 92.0);
                    self.automation.insert("dialog.settings.label.programColor".into(), label);
                    ui.add_space(8.0);
                    let program = ui.scope(|ui| {
                        ui.spacing_mut().interact_size = Vec2::splat(swatch);
                        ui.color_edit_button_srgb(&mut draft.program_color)
                    }).inner;
                    self.automation.insert("dialog.settings.programColor".into(), program.rect);
                });
                ui.end_row();
                let label = display_label(ui, "Master Frame Rate:", label_width);
                self.automation.insert("dialog.settings.label.frameRate".into(), label);
                let rate = egui::ComboBox::from_id_salt("settings.rate").width(field_width)
                    .selected_text(draft.scan.rate_label(draft.rate)).show_ui(ui, |ui| {
                        for (rate,scan) in rustmix_engine::video_format::rate_options() {
                            let option = ui.selectable_label(draft.rate == rate && draft.scan == scan, scan.rate_label(rate));
                            if scan==ScanMode::Progressive { self.automation.insert(format!("dialog.settings.rate.{}", rate.label()), option.rect); }
                            self.automation.insert(format!("dialog.settings.rate.{}.{}", rate.label(),scan.label()), option.rect);
                            if option.clicked() { draft.rate=rate; draft.scan=scan; ui.close(); }
                        }
                    });
                self.automation.insert("dialog.settings.frameRate".into(), rate.response.rect);
                ui.end_row();
                let label = display_label(ui, "Output Size:", label_width);
                self.automation.insert("dialog.settings.label.outputSize".into(), label);
                let raster = egui::ComboBox::from_id_salt("settings.canvas").width(field_width)
                    .selected_text(draft.video_standard().canvas_label()).show_ui(ui, |ui| {
                        for (width, height) in rustmix_engine::video_format::CANVAS_SIZES {
                            let selected = draft.width == width && draft.height == height;
                            let option = ui.selectable_label(selected, VideoStandard { width, height, ..draft.video_standard() }.canvas_label());
                            self.automation.insert(format!("dialog.settings.canvas.{width}"), option.rect);
                            if option.clicked() { draft.width = width; draft.height = height; ui.close(); }
                        }
                    });
                self.automation.insert("dialog.settings.outputSize".into(), raster.response.rect);
                raster.response.on_hover_text("Project canvas size. Monitor processing uses 640×360 landscape or 360×640 portrait. Full-resolution and interlaced output is unavailable.");
                ui.end_row();
                if compact {
                    self.display_project_name(ui, label_width, field_width);
                }
            });
        if compact {
            ui.collapsing("More display settings", |ui| {
                self.display_details(ui, draft, label_width, field_width, false);
            });
        } else {
            self.display_details(ui, draft, label_width, field_width, true);
        }
    }

    fn display_project_name(&mut self, ui: &mut egui::Ui, label_width: f32, field_width: f32) {
        display_label(ui, "Production Name:", label_width);
        let name = ui.add_sized(
            [field_width, 22.0],
            egui::TextEdit::singleline(&mut self.settings_name).char_limit(64),
        );
        self.automation
            .insert("dialog.settings.projectName".into(), name.rect);
        ui.end_row();
    }

    fn display_details(
        &mut self,
        ui: &mut egui::Ui,
        draft: &Preferences,
        label_width: f32,
        field_width: f32,
        include_name: bool,
    ) {
        egui::Grid::new("settings.display.details").num_columns(2)
            .spacing([12.0, 6.0]).show(ui, |ui| {
                display_label(ui, "Output Aspect Ratio:", label_width);
                let aspect = ui.add_enabled_ui(false, |ui| {
                    egui::ComboBox::from_id_salt("settings.aspect").width(field_width)
                        .selected_text(if draft.height > draft.width { "Portrait (9:16)" } else { "Widescreen (16:9)" }).show_ui(ui, |_| {})
                }).inner;
                self.automation.insert("dialog.settings.aspectRatio".into(), aspect.response.rect);
                aspect.response.on_disabled_hover_text("Aspect ratio follows the selected canvas orientation.");
                ui.end_row();
                ui.allocate_space(egui::vec2(0.0,12.0));
                ui.allocate_space(egui::vec2(0.0,12.0));
                ui.end_row();
                display_label(ui, "Fullscreen:", label_width);
                ui.horizontal(|ui| {
                    let width = field_width.max(86.0);
                    let menu = ui.allocate_ui_with_layout(egui::vec2(width,32.0),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                            ui.menu_button("Fullscreen 1", |ui| {
                                self.extended_output_menu(ui);
                                ui.separator();
                                self.program_output_menu(ui);
                            })
                        }).inner;
                    self.automation.insert("dialog.settings.programWindow".into(), menu.response.rect);
                });
                ui.end_row();
                display_label(ui, "Display:", label_width);
                let selected = self.extended_output.selected_id().unwrap_or("No extended display selected");
                let display = ui.add_sized([field_width,22.0], egui::Label::new(selected).truncate());
                self.automation.insert("dialog.settings.display".into(), display.rect);
                display.on_hover_text("Configure the display through Fullscreen 1. No output starts until explicitly requested.");
                ui.end_row();
                display_label(ui, "Program surface:", label_width);
                let [program_w, program_h] = draft.video_standard().program_size();
                let [monitor_w, monitor_h] = draft.video_standard().monitor_size();
                let description = format!("{program_w}×{program_h} CPU · {monitor_w}×{monitor_h} monitor");
                ui.add_sized([field_width, 26.0], egui::Label::new(&description).truncate()).on_hover_text(description);
                ui.end_row();
                ui.allocate_space(egui::vec2(0.0,20.0));
                ui.allocate_space(egui::vec2(0.0,20.0));
                ui.end_row();
                display_label(ui, "Input Size:", label_width);
                let input_size = ui.add_enabled(false, egui::Button::new("Automatic · 16:9 thumbnails")
                    .min_size(egui::vec2(field_width,22.0)))
                    .on_disabled_hover_text("Source cards currently size automatically with the operator viewport.");
                self.automation.insert("dialog.settings.inputSize".into(), input_size.rect);
                ui.end_row();
                if include_name { self.display_project_name(ui, label_width, field_width); }
            });
    }
    pub(crate) fn category_settings(&mut self, ui: &mut egui::Ui, draft: &mut Preferences) {
        ui.label(
            "Six standard and sixteen custom categories. Assign inputs through their context menu.",
        );
        for category in &mut draft.categories {
            ui.push_id(category.id, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}", category.id + 1));
                    ui.add(
                        egui::TextEdit::singleline(&mut category.name)
                            .char_limit(48)
                            .desired_width(180.0),
                    );
                    ui.color_edit_button_srgb(&mut category.color);
                });
            });
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]
    use super::*;
    fn frame(app: &mut RustMixApp, ctx: &egui::Context, events: Vec<egui::Event>) {
        let raw = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
            events,
            ..Default::default()
        };
        let _ = ctx.run_ui(raw, |ui| app.frame_ui(ui));
    }
    fn click(app: &mut RustMixApp, ctx: &egui::Context, id: &str) {
        for _ in 0..6 {
            frame(app, ctx, vec![]);
        }
        let pos = app.automation.get(id).unwrap().center();
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }
    #[test]
    fn categories_filter_assignments_not_generator_kinds_and_preserve_program() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.execute(Command::SetInputCategory(InputId(1), Some(0)));
        app.execute(Command::SetInputCategory(InputId(2), Some(21)));
        let before = (app.engine.project().program, app.engine.project().preview);
        click(&mut app, &ctx, "inputs.category.0");
        frame(&mut app, &ctx, vec![]);
        assert!(app.automation.contains_key("input.1.preview"));
        assert!(!app.automation.contains_key("input.2.preview"));
        app.category = Some(21);
        frame(&mut app, &ctx, vec![]);
        assert!(!app.automation.contains_key("input.1.preview"));
        assert!(app.automation.contains_key("input.2.preview"));
        click(&mut app, &ctx, "inputs.filter.clear");
        frame(&mut app, &ctx, vec![]);
        assert!(app.automation.contains_key("input.1.preview"));
        assert_eq!(
            (app.engine.project().program, app.engine.project().preview),
            before
        );
    }
    #[test]
    fn search_is_a_staged_dialog_and_blocks_switching_until_closed() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        click(&mut app, &ctx, "inputs.search");
        assert!(app.search_open);
        app.search_draft = "Camera B".into();
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(app.engine.project().program, InputId(1));
        click(&mut app, &ctx, "dialog.search.cancel");
        assert!(app.input_filter.is_empty());
        click(&mut app, &ctx, "inputs.search");
        app.search_draft = "Camera B".into();
        click(&mut app, &ctx, "dialog.search.apply");
        frame(&mut app, &ctx, vec![]);
        assert!(!app.search_open);
        assert_eq!(app.input_filter, "Camera B");
        assert!(!app.automation.contains_key("input.1.preview"));
        assert!(app.automation.contains_key("input.2.preview"));
        assert_eq!(app.engine.project().program, InputId(1));
    }
    #[test]
    fn theme_and_categories_cancel_or_apply_as_one_validated_settings_change() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let original = app.engine.project().clone();
        app.dispatch(Action::Settings, &ctx);
        let draft = app.settings_draft.as_mut().unwrap();
        draft.preview_color = [80, 100, 200];
        draft.program_color = [100, 160, 50];
        draft.ui_theme = UiTheme::Light;
        draft.categories[21].name = "Interview".into();
        click(&mut app, &ctx, "dialog.settings.cancel");
        assert_eq!(*app.engine.project(), original);
        app.dispatch(Action::Settings, &ctx);
        let draft = app.settings_draft.as_mut().unwrap();
        draft.preview_color = [80, 100, 200];
        draft.program_color = [100, 160, 50];
        draft.ui_theme = UiTheme::Light;
        draft.categories[21].name = "Interview".into();
        click(&mut app, &ctx, "dialog.settings.apply");
        frame(&mut app, &ctx, vec![]);
        assert!(!app.settings);
        assert_eq!(app.engine.project().revision, original.revision + 1);
        assert_eq!(app.preview_color(), Color32::from_rgb(80, 100, 200));
        assert_eq!(app.program_color(), Color32::from_rgb(100, 160, 50));
        assert!(
            !ctx.style_of(egui::Theme::Light).visuals.dark_mode,
            "the applied light theme must carry light visuals"
        );
        assert_eq!(
            app.engine.project().preferences.categories[21].name,
            "Interview"
        );
        assert_eq!(app.engine.project().program, original.program);
    }
    #[test]
    fn hiding_audio_restores_original_complete_dock_and_never_changes_engine() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        // egui 0.35 serialises the not-laid-out rectangles of a fresh dock as
        // JSON null, which its own deserialiser rejects; a laid-out frame
        // replaces them with real geometry so the round trip is meaningful.
        frame(&mut app, &ctx, vec![]);
        let project = app.engine.project().clone();
        let layout = serde_json::to_string(&app.dock).unwrap();
        app.toggle_audio_pane();
        assert!(app.dock.find_tab(&Panel::Audio).is_none());
        assert!(
            workspace::decode(&serde_json::to_string(app.audio_restore.as_ref().unwrap()).unwrap())
                .is_ok()
        );
        app.toggle_audio_pane();
        assert_eq!(serde_json::to_string(&app.dock).unwrap(), layout);
        assert_eq!(*app.engine.project(), project);
    }
}
