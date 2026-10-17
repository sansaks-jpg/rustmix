use super::*;
use eframe::egui::{self, Align2, FontId, Rect, Sense, Stroke, Vec2};

/// Vertical blue group label (`OUTPUTS` / `INPUTS`) matching `Interface.png`.
fn mixer_rail(ui: &mut egui::Ui, height: f32, label: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, height), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, MIXER_RAIL);
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, Color32::from_rgb(10, 65, 120)),
        egui::StrokeKind::Inside,
    );
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_owned(), FontId::proportional(11.0), Color32::WHITE);
    ui.painter().add(
        egui::epaint::TextShape::new(rect.center(), galley, Color32::WHITE)
            .with_angle_and_anchor(-std::f32::consts::FRAC_PI_2, Align2::CENTER_CENTER),
    );
}

impl RustMixApp {
    pub(crate) fn refresh_textures(&mut self, ctx: &egui::Context) {
        let frame = self.engine.project().preferences.rate.frame_at(self.now);
        if self.last_demo_frame == Some(frame) {
            return;
        }
        let start = Instant::now();
        let p = self.engine.project();
        self.textures.retain(|id, _| p.input(*id).is_some());
        for input in &p.inputs {
            match render::source(
                input,
                384,
                216,
                if self.demo_paused {
                    self.paused_at
                } else {
                    self.now
                },
            ) {
                Ok(pixels) => {
                    let image = egui::ColorImage::from_rgba_unmultiplied(
                        [384, 216],
                        &pixels.into_iter().flatten().collect::<Vec<_>>(),
                    );
                    if let Some(texture) = self.textures.get_mut(&input.id) {
                        texture.set(image, egui::TextureOptions::LINEAR);
                    } else {
                        self.textures.insert(
                            input.id,
                            ctx.load_texture(
                                format!("source.{}", input.id.0),
                                image,
                                egui::TextureOptions::LINEAR,
                            ),
                        );
                    }
                }
                Err(e) => {
                    self.notice = e.to_string();
                    self.warning = true;
                }
            }
        }
        match render::program(
            p,
            self.engine.transition(),
            self.engine.black_level(),
            640,
            360,
            if self.demo_paused {
                self.paused_at
            } else {
                self.now
            },
        ) {
            Ok(pixels) => {
                let image = egui::ColorImage::from_rgba_unmultiplied(
                    [640, 360],
                    &pixels.into_iter().flatten().collect::<Vec<_>>(),
                );
                if let Some(texture) = &mut self.program_texture {
                    texture.set(image, egui::TextureOptions::LINEAR);
                } else {
                    self.program_texture = Some(ctx.load_texture(
                        "program.composite",
                        image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }
            Err(e) => {
                self.notice = format!("Program retained last valid frame: {e}");
                self.warning = true;
            }
        }
        self.last_demo_frame = Some(frame);
        self.generation_ms = start.elapsed().as_secs_f32() * 1000.0;
    }

    fn monitor(&mut self, ui: &mut egui::Ui, program: bool) {
        let p = self.engine.project();
        let id = if program { p.program } else { p.preview };
        let Some(input) = p.input(id) else {
            ui.colored_label(RED, "Source missing");
            return;
        };
        let color = if program { PROGRAM } else { PREVIEW };
        egui::Frame::new()
            .fill(color)
            .inner_margin(egui::Margin::symmetric(6, 3))
            .show(ui, |ui| {
                ui.set_min_width((ui.available_width() - 1.0).max(20.0));
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [(ui.available_width() - 90.0).max(40.0), 18.0],
                        egui::Label::new(
                            RichText::new(format!("{}   {}", id.0, input.name))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .truncate(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        let gear = ui
                            .add(
                                egui::Button::new(
                                    RichText::new("⚙").size(13.0).color(Color32::WHITE),
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::NONE)
                                .min_size(egui::vec2(20.0, 18.0)),
                            )
                            .on_hover_text("Input Settings");
                        self.automation.insert(
                            format!(
                                "monitor.{}.properties",
                                if program { "program" } else { "preview" }
                            ),
                            gear.rect,
                        );
                        if gear.clicked() {
                            self.actions.push(Action::Properties(id));
                        }
                        if !program {
                            // Position / PIP box icon on Preview bar matching Interface.png
                            let (pos_rect, pos_resp) =
                                ui.allocate_exact_size(egui::vec2(16.0, 14.0), Sense::click());
                            ui.painter().rect_stroke(
                                pos_rect,
                                1.0,
                                Stroke::new(1.2, Color32::WHITE),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().rect_stroke(
                                pos_rect.shrink(3.0),
                                0.0,
                                Stroke::new(1.0, Color32::WHITE),
                                egui::StrokeKind::Inside,
                            );
                            self.automation
                                .insert("monitor.preview.position".into(), pos_rect);
                            if pos_resp.on_hover_text("Position / Zoom settings").clicked() {
                                self.actions
                                    .push(Action::PropertiesAt(id, InputPage::Position));
                            }
                            // RGB color bars icon on Preview bar matching Interface.png
                            let (rgb_rect, rgb_resp) =
                                ui.allocate_exact_size(egui::vec2(16.0, 14.0), Sense::click());
                            ui.painter().rect_filled(rgb_rect, 1.0, Color32::WHITE);
                            let third = (rgb_rect.width() - 2.0) / 3.0;
                            for (idx, col) in [
                                Color32::from_rgb(220, 35, 35),
                                Color32::from_rgb(35, 190, 55),
                                Color32::from_rgb(35, 95, 225),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                let stripe = Rect::from_min_size(
                                    egui::pos2(
                                        rgb_rect.left() + 1.0 + idx as f32 * third,
                                        rgb_rect.top() + 1.0,
                                    ),
                                    egui::vec2(third, rgb_rect.height() - 2.0),
                                );
                                ui.painter().rect_filled(stripe, 0.0, col);
                            }
                            self.automation
                                .insert("monitor.preview.color".into(), rgb_rect);
                            if rgb_resp.on_hover_text("Colour Adjust settings").clicked() {
                                self.actions
                                    .push(Action::PropertiesAt(id, InputPage::ColourAdjust));
                            }
                        }
                    });
                });
            });
        let available = ui.available_size();
        let width = available
            .x
            .max(16.0)
            .min((available.y - 6.0).max(9.0) * 16.0 / 9.0);
        let (letterbox, _) = ui.allocate_exact_size(
            Vec2::new(available.x.max(width), width * 9.0 / 16.0),
            Sense::hover(),
        );
        ui.painter().rect_filled(letterbox, 0.0, Color32::BLACK);
        let rect = Rect::from_center_size(letterbox.center(), Vec2::new(width, width * 9.0 / 16.0));
        self.automation.insert(
            format!("monitor.{}", if program { "program" } else { "preview" }),
            rect,
        );
        ui.painter().rect_filled(rect, 0.0, Color32::BLACK);
        let texture = if program {
            self.program_texture.as_ref()
        } else {
            self.textures.get(&id)
        };
        if let Some(texture) = texture {
            ui.painter().image(
                texture.id(),
                rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        let visible = if program {
            1.0 - self.engine.black_level()
        } else {
            1.0
        };
        if visible > 0.01 && input.online && (!program || self.engine.transition().is_none()) {
            let tint = Color32::WHITE.gamma_multiply(visible);
            if input.kind == InputKind::Countdown {
                let clock = if self.demo_paused {
                    self.paused_at
                } else {
                    self.now
                };
                let remaining = 60 - (clock / 1000) % 60;
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    format!("00:{remaining:02}"),
                    FontId::monospace(rect.height() * 0.22),
                    tint,
                );
            }
            if input.kind == InputKind::Graphics {
                ui.painter().text(
                    rect.lerp_inside(egui::vec2(0.12, 0.7)),
                    Align2::LEFT_CENTER,
                    "YOUR STORY. LIVE.",
                    FontId::proportional(rect.height() * 0.07),
                    tint,
                );
            }
        }
        if program {
            for (i, o) in p.overlays.iter().enumerate() {
                if o.enabled {
                    let panel = overlay_panel_rect(rect, o.position, i);
                    let text_rect = Rect::from_min_size(
                        panel.left_top() + egui::vec2(rect.width() * 0.02, 0.0),
                        egui::vec2(rect.width() * 0.52, rect.height() * 0.095),
                    );
                    let painter = ui.painter().with_clip_rect(text_rect);
                    painter.text(
                        text_rect.left_top() + egui::vec2(0.0, rect.height() * 0.006),
                        Align2::LEFT_TOP,
                        &o.title,
                        FontId::proportional((rect.height() * 0.038).clamp(8.0, 15.0)),
                        Color32::WHITE.gamma_multiply(visible),
                    );
                    painter.text(
                        text_rect.left_top() + egui::vec2(0.0, rect.height() * 0.052),
                        Align2::LEFT_TOP,
                        &o.subtitle,
                        FontId::proportional((rect.height() * 0.026).clamp(6.0, 11.0)),
                        Color32::from_gray(190).gamma_multiply(visible),
                    );
                }
            }
        }
    }

    pub(crate) fn switcher(&mut self, ui: &mut egui::Ui) {
        ui.set_min_height(ui.available_height());
        if self.multiview {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("MULTIVIEW · click a tile to select Preview")
                        .strong()
                        .color(TEAL),
                );
                ui.separator();
                let split = ui.selectable_label(self.multiview_split, "Split: Monitors + Grid");
                self.automation
                    .insert("multiview.preset.split".into(), split.rect);
                if split.clicked() {
                    self.multiview_split = true;
                }
                let grid = ui.selectable_label(!self.multiview_split, "Full Source Grid");
                self.automation
                    .insert("multiview.preset.grid".into(), grid.rect);
                if grid.clicked() {
                    self.multiview_split = false;
                }
                let back = ui.button("Return to switcher");
                self.automation.insert("multiview.return".into(), back.rect);
                if back.clicked() {
                    self.actions.push(Action::Multiview);
                }
            });
            if self.multiview_split
                && ui.available_height() >= 300.0
                && ui.available_width() >= 620.0
            {
                let top_h = (ui.available_height() * 0.44).clamp(145.0, 215.0);
                let side_w = ((ui.available_width() - 24.0) * 0.5).max(180.0);
                ui.horizontal_top(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(side_w, top_h),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| self.monitor(ui, false),
                    );
                    ui.add_space(12.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(side_w, top_h),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| self.monitor(ui, true),
                    );
                });
                ui.separator();
                self.source_tiles(ui, false);
            } else {
                self.source_tiles(ui, true);
            }
            return;
        }
        let width = ui.available_width();
        if width < 650.0 {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.set_min_width(620.0);
                self.switcher_content(ui);
            });
        } else {
            self.switcher_content(ui);
        }
    }

    fn switcher_content(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        ui.set_min_height(available.y);
        let center = 108.0;
        let side = ((available.x - center - 72.0) / 2.0).max(200.0);
        ui.horizontal_top(|ui| {
            ui.add_space(12.0);
            ui.allocate_ui_with_layout(
                egui::vec2(side, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.monitor(ui, false),
            );
            ui.add_space(16.0);
            ui.allocate_ui_with_layout(
                egui::vec2(center, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(center);
                    ui.spacing_mut().item_spacing.y = 3.0;
                    self.command_button(ui, "switcher.quickPlay", "Quick Play", Command::Cut, None);
                    self.command_button(ui, "switcher.cut", "Cut", Command::Cut, None);

                    // Fade + caret
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;
                        let enabled = self
                            .engine
                            .enabled(&Command::Transition(TransitionKind::Fade));
                        let btn = ui.add_enabled(
                            enabled.is_ok(),
                            egui::Button::new(RichText::new("Fade").strong().size(11.5))
                                .min_size(egui::vec2(center - 19.0, 26.0)),
                        );
                        self.automation.insert("switcher.fade".into(), btn.rect);
                        if btn.clicked() {
                            self.actions
                                .push(Action::Engine(Command::Transition(TransitionKind::Fade)));
                        }
                        ui.menu_button(RichText::new("▾").size(9.0), |ui| {
                            for ms in [250, 500, 1000, 1500, 2000] {
                                if ui.button(format!("Fade · {ms} ms")).clicked() {
                                    self.actions.push(Action::Engine(Command::SetDuration(ms)));
                                    ui.close();
                                }
                            }
                        });
                    });

                    // Merge + caret
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;
                        let btn = ui.add(
                            egui::Button::new(RichText::new("Merge").strong().size(11.5))
                                .min_size(egui::vec2(center - 19.0, 26.0)),
                        );
                        if btn.clicked() {
                            self.actions
                                .push(Action::Engine(Command::Transition(TransitionKind::Fade)));
                        }
                        ui.menu_button(RichText::new("▾").size(9.0), |ui| {
                            for kind in [
                                "Fade",
                                "Zoom",
                                "Wipe",
                                "Slide",
                                "Fly",
                                "CrossZoom",
                                "Merge",
                                "CubeZoom",
                            ] {
                                if ui.button(kind).clicked() {
                                    ui.close();
                                }
                            }
                        });
                    });

                    // Wipe + caret
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;
                        let enabled = self
                            .engine
                            .enabled(&Command::Transition(TransitionKind::Wipe));
                        let btn = ui.add_enabled(
                            enabled.is_ok(),
                            egui::Button::new(RichText::new("Wipe").strong().size(11.5))
                                .min_size(egui::vec2(center - 19.0, 26.0)),
                        );
                        self.automation.insert("switcher.wipe".into(), btn.rect);
                        if btn.clicked() {
                            self.actions
                                .push(Action::Engine(Command::Transition(TransitionKind::Wipe)));
                        }
                        ui.menu_button(RichText::new("▾").size(9.0), |ui| {
                            for ms in [250, 500, 1000, 1500, 2000] {
                                if ui.button(format!("Wipe · {ms} ms")).clicked() {
                                    self.actions.push(Action::Engine(Command::SetDuration(ms)));
                                    ui.close();
                                }
                            }
                        });
                    });

                    // Second Merge + caret (matching Interface.png slot 4)
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;
                        let btn = ui.add(
                            egui::Button::new(RichText::new("Merge").strong().size(11.5))
                                .min_size(egui::vec2(center - 19.0, 26.0)),
                        );
                        if btn.clicked() {
                            self.actions
                                .push(Action::Engine(Command::Transition(TransitionKind::Wipe)));
                        }
                        ui.menu_button(RichText::new("▾").size(9.0), |ui| {
                            for ms in [250, 500, 1000, 1500, 2000, 3000] {
                                if ui.button(format!("{ms} ms")).clicked() {
                                    self.actions.push(Action::Engine(Command::SetDuration(ms)));
                                    ui.close();
                                }
                            }
                        });
                    });

                    // FTB
                    self.command_button(
                        ui,
                        "switcher.ftb",
                        "FTB",
                        Command::FadeBlack,
                        if self.engine.black_level() > 0.5 {
                            Some(Color32::from_rgb(175, 28, 28))
                        } else {
                            None
                        },
                    );

                    ui.add_space(3.0);
                    // 1..4 and 5..8 overlay buttons matching Interface.png
                    for row in 0..2 {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            for col in 0..4 {
                                let slot = row * 4 + col;
                                if let Some(o) = self.engine.project().overlays.get(slot).cloned() {
                                    let mut next = o.clone();
                                    let r = ui.add(
                                        egui::Button::new(
                                            RichText::new((slot + 1).to_string())
                                                .strong()
                                                .size(11.0),
                                        )
                                        .selected(o.enabled)
                                        .min_size(egui::vec2(25.0, 20.0)),
                                    );
                                    self.automation
                                        .insert(format!("overlay.{}.toggle", slot + 1), r.rect);
                                    if r.clicked() {
                                        next.enabled = !next.enabled;
                                        self.actions
                                            .push(Action::Engine(Command::SetOverlay(slot, next)));
                                    }
                                } else {
                                    let r = ui.add(
                                        egui::Button::new(
                                            RichText::new((slot + 1).to_string())
                                                .strong()
                                                .size(11.0),
                                        )
                                        .min_size(egui::vec2(25.0, 20.0)),
                                    );
                                    if r.clicked() {
                                        self.actions.push(Action::OverlaySettings(0));
                                    }
                                }
                            }
                        });
                    }

                    ui.add_space(4.0);
                    // Cylindrical vertical T-Bar with blue indicator & horizontal bar matching Interface.png
                    let progress = self.engine.transition().map_or(0.0, |t| t.progress);
                    let auto = self.engine.transition().is_some_and(|t| !t.manual);
                    let mut handle = 1.0 - progress;
                    let tbar_length = (ui.available_height() - 8.0).clamp(52.0, 120.0);
                    ui.horizontal(|ui| {
                        let response = ui
                            .add_enabled_ui(!auto, |ui| {
                                ui.spacing_mut().slider_width = tbar_length;
                                ui.spacing_mut().interact_size.y = 22.0;
                                ui.add(
                                    egui::Slider::new(&mut handle, 0.0..=1.0)
                                        .show_value(false)
                                        .vertical(),
                                )
                            })
                            .inner
                            .on_hover_text(
                                "Manual T-Bar · drag down to transition Preview to Program",
                            );
                        // Overpaint the cylindrical gray barrel + blue center line + horizontal arm matching Interface.png
                        let barrel = Rect::from_center_size(
                            response.rect.center(),
                            egui::vec2(20.0, response.rect.height()),
                        );
                        ui.painter()
                            .rect_filled(barrel, 4.0, Color32::from_rgb(108, 112, 116));
                        ui.painter().rect_stroke(
                            barrel,
                            4.0,
                            Stroke::new(1.0, Color32::from_rgb(40, 44, 48)),
                            egui::StrokeKind::Inside,
                        );
                        let indicator_y = egui::lerp(
                            (barrel.top() + 6.0)..=(barrel.bottom() - 6.0),
                            1.0 - handle,
                        );
                        let blue_band = Rect::from_center_size(
                            egui::pos2(barrel.center().x, indicator_y),
                            egui::vec2(22.0, 7.0),
                        );
                        ui.painter()
                            .rect_filled(blue_band, 1.0, Color32::from_rgb(0, 128, 255));
                        let arm = Rect::from_min_size(
                            egui::pos2(barrel.right(), indicator_y - 1.5),
                            egui::vec2(72.0, 3.0),
                        );
                        ui.painter().rect_filled(arm, 0.0, Color32::BLACK);
                        ui.painter().rect_stroke(
                            arm,
                            0.0,
                            Stroke::new(1.0, Color32::from_gray(210)),
                            egui::StrokeKind::Inside,
                        );
                        self.automation
                            .insert("switcher.tbar".into(), response.rect);
                        self.automation.insert("switcher.cancel".into(), arm);
                        if response.changed() && !self.tbar_latched {
                            let value = 1.0 - handle;
                            self.actions
                                .push(Action::Engine(Command::SetManualProgress(value)));
                            if value >= 1.0 {
                                self.tbar_latched = true;
                            }
                        }
                        if !ui.input(|i| i.pointer.primary_down()) {
                            self.tbar_latched = false;
                        }
                    });
                },
            );
            ui.add_space(16.0);
            ui.allocate_ui_with_layout(
                egui::vec2(side, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.monitor(ui, true),
            );
        });
    }

    pub(crate) fn inputs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            let selected_all = self.category.is_none();
            let all = ui
                .add(
                    egui::Button::new(" ")
                        .fill(if selected_all {
                            Color32::from_rgb(76, 86, 96)
                        } else {
                            Color32::from_rgb(42, 48, 54)
                        })
                        .stroke(Stroke::new(1.0, Color32::from_rgb(90, 100, 110)))
                        .min_size(egui::vec2(22.0, 18.0)),
                )
                .on_hover_text("All inputs");
            self.automation
                .insert("inputs.category.all".into(), all.rect);
            if all.clicked() {
                self.category = None;
            }
            for (kind, color) in InputKind::ALL.into_iter().map(|k| (k, kind_color(k))) {
                let selected = self.category == Some(kind);
                let response = ui
                    .add(
                        egui::Button::new(" ")
                            .fill(color)
                            .stroke(if selected {
                                Stroke::new(1.5, Color32::WHITE)
                            } else {
                                Stroke::new(1.0, Color32::from_rgb(20, 24, 28))
                            })
                            .min_size(egui::vec2(22.0, 18.0)),
                    )
                    .on_hover_text(kind.label());
                self.automation
                    .insert(format!("inputs.category.{kind:?}"), response.rect);
                if response.clicked() {
                    self.category = if selected { None } else { Some(kind) };
                }
            }
            let mag = ui
                .add(
                    egui::Button::new(RichText::new("🔍").size(11.0))
                        .min_size(egui::vec2(22.0, 18.0)),
                )
                .on_hover_text("Search inputs");
            if mag.clicked() {
                self.search_open = !self.search_open;
            }
            if self.search_open || !self.input_filter.is_empty() {
                ui.add_space(4.0);
                let search = ui.add(
                    egui::TextEdit::singleline(&mut self.input_filter)
                        .hint_text("Search")
                        .desired_width(120.0)
                        .char_limit(64),
                );
                self.automation
                    .insert("inputs.filter.search".into(), search.rect);
            }
            if self.category.is_some() || !self.input_filter.is_empty() {
                let clear = ui.small_button("×");
                self.automation
                    .insert("inputs.filter.clear".into(), clear.rect);
                if clear.clicked() {
                    self.category = None;
                    self.input_filter.clear();
                }
            }
        });
        self.source_tiles(ui, false);
    }

    fn source_tiles(&mut self, ui: &mut egui::Ui, large: bool) {
        let project = self.engine.project();
        let inputs = project.inputs.clone();
        let program = project.program;
        let preview = project.preview;
        let target = if large { 290.0 } else { 222.0 };
        let columns = ((ui.available_width() + 6.0) / (target + 6.0))
            .floor()
            .max(1.0) as usize;
        let tile_width = ((ui.available_width() - columns.saturating_sub(1) as f32 * 6.0)
            / columns as f32)
            .max(185.0);
        let filter = self.input_filter.trim().to_lowercase();
        let filtered: Vec<_> = inputs
            .into_iter()
            .filter(|input| {
                input.name.to_lowercase().contains(&filter)
                    && self.category.is_none_or(|kind| kind == input.kind)
            })
            .collect();
        egui::ScrollArea::vertical()
            .id_salt(if large {
                "multiview.scroll"
            } else {
                "inputs.scroll"
            })
            .show(ui, |ui| {
                if filtered.is_empty() {
                    ui.add_space(16.0);
                    ui.strong("No matching inputs");
                }
                for row in filtered.chunks(columns) {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        for input in row {
                            ui.push_id(("input", input.id.0), |ui| {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(tile_width, tile_width * 9.0 / 16.0 + 70.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.y = 0.0;
                                        let header_color = if input.id == program {
                                            PROGRAM
                                        } else if input.id == preview {
                                            PREVIEW
                                        } else {
                                            IDLE_TILE
                                        };
                                        egui::Frame::new()
                                            .fill(Color32::from_rgb(18, 22, 26))
                                            .stroke(Stroke::new(1.0, Color32::from_rgb(65, 74, 82)))
                                            .inner_margin(1.0)
                                            .show(ui, |ui| {
                                                let inner_w = (tile_width - 4.0).max(160.0);
                                                ui.set_width(inner_w);
                                                // Tile header bar matching Interface.png: "1   Blank" + "X"
                                                egui::Frame::new()
                                                    .fill(header_color)
                                                    .inner_margin(egui::Margin::symmetric(5, 2))
                                                    .show(ui, |ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.add_sized(
                                                                [(inner_w - 34.0).max(24.0), 16.0],
                                                                egui::Label::new(
                                                                    RichText::new(format!(
                                                                        "{}    {}",
                                                                        input.id.0, input.name
                                                                    ))
                                                                    .strong()
                                                                    .color(Color32::WHITE),
                                                                )
                                                                .truncate(),
                                                            );
                                                            ui.with_layout(
                                                                egui::Layout::right_to_left(
                                                                    egui::Align::Center,
                                                                ),
                                                                |ui| {
                                                                    let remove = ui.add(
                                                                        egui::Button::new(
                                                                            RichText::new("X")
                                                                                .strong()
                                                                                .size(11.0)
                                                                                .color(
                                                                                    Color32::WHITE,
                                                                                ),
                                                                        )
                                                                        .fill(Color32::TRANSPARENT)
                                                                        .stroke(Stroke::NONE)
                                                                        .min_size(egui::vec2(
                                                                            18.0, 16.0,
                                                                        )),
                                                                    );
                                                                    self.automation.insert(
                                                                        format!(
                                                                            "input.{}.remove",
                                                                            input.id.0
                                                                        ),
                                                                        remove.rect,
                                                                    );
                                                                    if remove
                                                                        .on_hover_text(
                                                                            "Close input",
                                                                        )
                                                                        .clicked()
                                                                    {
                                                                        self.actions.push(
                                                                            Action::RequestRemove(
                                                                                input.id,
                                                                            ),
                                                                        );
                                                                    }
                                                                },
                                                            );
                                                        });
                                                    });
                                                // 16:9 thumbnail
                                                let size =
                                                    egui::vec2(inner_w, inner_w * 9.0 / 16.0);
                                                let response = if let Some(texture) =
                                                    self.textures.get(&input.id)
                                                {
                                                    ui.add(
                                                        egui::Image::new(texture)
                                                            .fit_to_exact_size(size)
                                                            .sense(Sense::click()),
                                                    )
                                                } else {
                                                    ui.allocate_response(size, Sense::click())
                                                };
                                                self.automation.insert(
                                                    format!("preview.input.{}", input.id.0),
                                                    response.rect,
                                                );
                                                if !input.online {
                                                    ui.painter().rect_filled(
                                                        response.rect,
                                                        0.0,
                                                        Color32::BLACK,
                                                    );
                                                    ui.painter().text(
                                                        response.rect.center(),
                                                        Align2::CENTER_CENTER,
                                                        "OFFLINE",
                                                        FontId::proportional(12.0),
                                                        RED,
                                                    );
                                                    self.automation.insert(
                                                        format!("input.{}.offline", input.id.0),
                                                        response.rect,
                                                    );
                                                }
                                                if response.clicked() {
                                                    self.actions.push(Action::Engine(
                                                        Command::SelectPreview(input.id),
                                                    ));
                                                }
                                                response.context_menu(|ui| {
                                                    self.command_button(
                                                        ui,
                                                        "preview.select",
                                                        "Select Preview",
                                                        Command::SelectPreview(input.id),
                                                        None,
                                                    );
                                                    if ui.button("Input Settings…").clicked() {
                                                        self.actions
                                                            .push(Action::Properties(input.id));
                                                        ui.close();
                                                    }
                                                    if ui.button("Audio Settings…").clicked() {
                                                        self.actions
                                                            .push(Action::InputAudio(input.id));
                                                        ui.close();
                                                    }
                                                    if ui.button("Close Input…").clicked() {
                                                        self.actions
                                                            .push(Action::RequestRemove(input.id));
                                                        ui.close();
                                                    }
                                                });
                                                // Compact 2-row control strip matching Interface.png:
                                                // Row 1: [1][2][3][4][GO][Cut][ ]
                                                // Row 2: [5][6][7][8][Audio][🖥][⚙]
                                                let cell_w = (inner_w / 7.0).floor().max(20.0);
                                                let rem_w = (inner_w - cell_w * 6.0).max(cell_w);
                                                ui.horizontal(|ui| {
                                                    ui.spacing_mut().item_spacing.x = 0.0;
                                                    ui.spacing_mut().button_padding =
                                                        egui::vec2(2.0, 1.0);
                                                    for slot in 0..4 {
                                                        if let Some(mut overlay) = self
                                                            .engine
                                                            .project()
                                                            .overlays
                                                            .get(slot)
                                                            .cloned()
                                                        {
                                                            let selected = overlay.enabled
                                                                && overlay.title == input.name;
                                                            let button = ui.add(
                                                                egui::Button::new(
                                                                    RichText::new(
                                                                        (slot + 1).to_string(),
                                                                    )
                                                                    .size(10.5),
                                                                )
                                                                .selected(selected)
                                                                .min_size(egui::vec2(cell_w, 20.0)),
                                                            );
                                                            self.automation.insert(
                                                                format!(
                                                                    "input.{}.overlay.{}",
                                                                    input.id.0,
                                                                    slot + 1
                                                                ),
                                                                button.rect,
                                                            );
                                                            if button.clicked() {
                                                                overlay.enabled = !selected;
                                                                overlay.title = input.name.clone();
                                                                overlay.subtitle =
                                                                    input.kind.label().into();
                                                                self.actions.push(Action::Engine(
                                                                    Command::SetOverlay(
                                                                        slot, overlay,
                                                                    ),
                                                                ));
                                                            }
                                                        }
                                                    }
                                                    let go = ui
                                                        .add(
                                                            egui::Button::new(
                                                                RichText::new("GO").size(10.5),
                                                            )
                                                            .min_size(egui::vec2(cell_w, 20.0)),
                                                        )
                                                        .on_hover_text(
                                                            "Select Preview / QuickPlay",
                                                        );
                                                    self.automation.insert(
                                                        format!("input.{}.preview", input.id.0),
                                                        go.rect,
                                                    );
                                                    if go.clicked() {
                                                        self.actions.push(Action::Engine(
                                                            Command::SelectPreview(input.id),
                                                        ));
                                                    }
                                                    let take = Command::TakeInput(input.id);
                                                    let check = self.engine.enabled(&take);
                                                    let cut = ui.add_enabled(
                                                        check.is_ok(),
                                                        egui::Button::new(
                                                            RichText::new("Cut").size(10.5),
                                                        )
                                                        .min_size(egui::vec2(cell_w, 20.0)),
                                                    );
                                                    self.automation.insert(
                                                        format!("input.{}.cut", input.id.0),
                                                        cut.rect,
                                                    );
                                                    if cut.clicked() {
                                                        self.actions.push(Action::Engine(take));
                                                    }
                                                    ui.add(
                                                        egui::Button::new(
                                                            RichText::new(" ").size(10.5),
                                                        )
                                                        .min_size(egui::vec2(rem_w, 20.0)),
                                                    );
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.spacing_mut().item_spacing.x = 0.0;
                                                    ui.spacing_mut().button_padding =
                                                        egui::vec2(2.0, 1.0);
                                                    for slot in 5..=8 {
                                                        ui.add(
                                                            egui::Button::new(
                                                                RichText::new(slot.to_string())
                                                                    .size(10.5),
                                                            )
                                                            .min_size(egui::vec2(cell_w, 20.0)),
                                                        );
                                                    }
                                                    let muted = input.audio.muted;
                                                    let audio_btn = egui::Button::new(
                                                        RichText::new("Audio").size(10.5).color(
                                                            if muted {
                                                                DIM
                                                            } else {
                                                                Color32::WHITE
                                                            },
                                                        ),
                                                    )
                                                    .fill(if muted { CHIP_BG } else { PROGRAM })
                                                    .min_size(egui::vec2(cell_w * 2.0, 20.0));
                                                    let audio = ui.add(audio_btn);
                                                    self.automation.insert(
                                                        format!("input.{}.audio", input.id.0),
                                                        audio.rect,
                                                    );
                                                    if audio.clicked() {
                                                        let mut channel = input.audio.clone();
                                                        channel.muted = !channel.muted;
                                                        self.actions.push(Action::Engine(
                                                            Command::SetInputAudio(
                                                                input.id, channel,
                                                            ),
                                                        ));
                                                    }
                                                    let mv_w = (rem_w * 0.5).max(18.0);
                                                    let gear_w = (rem_w - mv_w).max(18.0);
                                                    let mv = ui.add(
                                                        egui::Button::new(
                                                            RichText::new("🖥").size(10.5),
                                                        )
                                                        .min_size(egui::vec2(mv_w, 20.0)),
                                                    );
                                                    if mv.clicked() {
                                                        self.actions.push(Action::PropertiesAt(
                                                            input.id,
                                                            InputPage::Layers,
                                                        ));
                                                    }
                                                    let edit = ui.add(
                                                        egui::Button::new(
                                                            RichText::new("⚙").size(11.0),
                                                        )
                                                        .min_size(egui::vec2(gear_w, 20.0)),
                                                    );
                                                    self.automation.insert(
                                                        format!("input.{}.properties", input.id.0),
                                                        edit.rect,
                                                    );
                                                    if edit.clicked() {
                                                        self.actions
                                                            .push(Action::Properties(input.id));
                                                    }
                                                });
                                            });
                                    },
                                );
                            });
                        }
                    });
                    ui.add_space(4.0);
                }
            });
    }

    pub(crate) fn audio(&mut self, ui: &mut egui::Ui) {
        let p = self.engine.project().clone();
        let solo = p.inputs.iter().any(|i| i.audio.solo);
        let height = (ui.available_height() - 12.0).clamp(150.0, 240.0);
        egui::ScrollArea::horizontal()
            .id_salt("audio.strips")
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    // Pin icon at top-left of Audio Mixer matching Interface.png
                    ui.vertical(|ui| {
                        let _ = ui.add(
                            egui::Button::new(RichText::new("📌").size(11.0))
                                .min_size(egui::vec2(20.0, 20.0)),
                        );
                    });
                    mixer_rail(ui, height, "OUTPUTS");
                    if let Some(a) = p.buses.first() {
                        let peaks = audio::bus_peaks(&p, 0, self.now);
                        let changed = self.strip(
                            ui,
                            ("bus.0".into(), "Master"),
                            a.clone(),
                            peaks,
                            height,
                            None,
                        );
                        if let Some(a) = changed {
                            self.actions.push(Action::Engine(Command::SetBus(0, a)));
                        }
                    }
                    mixer_rail(ui, height, "INPUTS");
                    for input in &p.inputs {
                        let peaks = audio::input_peaks(input, solo, self.now);
                        let changed = self.strip(
                            ui,
                            (format!("input.{}", input.id.0), &input.name),
                            input.audio.clone(),
                            peaks,
                            height,
                            Some(input.id),
                        );
                        if let Some(a) = changed {
                            let mut i = input.clone();
                            i.audio = a;
                            self.selected_audio = Some(i.id);
                            self.actions
                                .push(Action::Engine(Command::SetInputAudio(i.id, i.audio)));
                        }
                    }
                });
            });
    }

    fn strip(
        &mut self,
        ui: &mut egui::Ui,
        identity: (String, &str),
        mut a: AudioChannel,
        peaks: [f32; 2],
        height: f32,
        input_id: Option<InputId>,
    ) -> Option<AudioChannel> {
        let (id, name) = identity;
        let before = a.clone();
        let is_master = input_id.is_none();
        let header_fill = if !a.muted { PROGRAM } else { CHIP_BG };
        ui.push_id(id, |ui| {
            egui::Frame::new()
                .fill(Color32::from_rgb(18, 22, 26))
                .stroke(Stroke::new(1.0, Color32::from_rgb(52, 60, 68)))
                .inner_margin(2.0)
                .show(ui, |ui| {
                    ui.set_width(76.0);
                    ui.set_min_height(height - 4.0);
                    // Green header bar ("Master" or input name) matching Interface.png
                    egui::Frame::new()
                        .fill(header_fill)
                        .inner_margin(egui::Margin::symmetric(4, 2))
                        .show(ui, |ui| {
                            ui.set_width(68.0);
                            ui.vertical_centered(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(name).small().strong().color(Color32::WHITE),
                                    )
                                    .truncate(),
                                );
                            });
                        });
                    ui.add_space(4.0);
                    let fader_h = (height - 58.0).max(68.0);
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        // Left button column inside strip matching Interface.png: [⚙], [i / S], [🔊]
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 4.0;
                            ui.add_space(10.0);
                            let gear = ui
                                .add(
                                    egui::Button::new(RichText::new("⚙").size(12.0))
                                        .min_size(egui::vec2(22.0, 22.0)),
                                )
                                .on_hover_text("Audio Settings");
                            if let Some(source_id) = input_id {
                                self.automation.insert(
                                    format!("audio.input.{}.settings", source_id.0),
                                    gear.rect,
                                );
                                if gear.clicked() {
                                    self.selected_audio = Some(source_id);
                                    self.actions.push(Action::InputAudio(source_id));
                                }
                            } else if gear.clicked() {
                                self.actions.push(Action::SettingsAt(SettingsPage::Audio));
                            }
                            ui.add_space(14.0);
                            if is_master {
                                let _ = ui.add(
                                    egui::Button::new(RichText::new("i").strong().size(11.0))
                                        .min_size(egui::vec2(22.0, 20.0)),
                                );
                            } else {
                                let solo = ui.add(
                                    egui::Button::new(RichText::new("S").strong().size(11.0))
                                        .selected(a.solo)
                                        .min_size(egui::vec2(22.0, 20.0)),
                                );
                                if solo.clicked() {
                                    a.solo = !a.solo;
                                }
                            }
                            let spk = ui.add(
                                egui::Button::new(RichText::new("🔊").size(11.0))
                                    .fill(if a.muted { CHIP_BG } else { PROGRAM })
                                    .min_size(egui::vec2(22.0, 22.0)),
                            );
                            if spk.clicked() {
                                a.muted = !a.muted;
                            }
                        });
                        // Right column inside strip: vertical fader with white scale ticks + stereo level meter
                        let fader = ui
                            .add_sized(
                                [24.0, fader_h],
                                egui::Slider::new(&mut a.gain_db, -60.0..=12.0)
                                    .vertical()
                                    .show_value(false),
                            )
                            .on_hover_text(audio::gain_label(&a));
                        for step in 0..=10 {
                            let y = egui::lerp(
                                (fader.rect.top() + 6.0)..=(fader.rect.bottom() - 6.0),
                                step as f32 / 10.0,
                            );
                            ui.painter().hline(
                                (fader.rect.left() + 1.0)..=(fader.rect.left() + 4.0),
                                y,
                                Stroke::new(1.0, Color32::from_gray(200)),
                            );
                            ui.painter().hline(
                                (fader.rect.right() - 4.0)..=(fader.rect.right() - 1.0),
                                y,
                                Stroke::new(1.0, Color32::from_gray(200)),
                            );
                        }
                        if is_master {
                            self.automation
                                .insert("audio.MASTER.gain".into(), fader.rect);
                        }
                        self.automation
                            .insert(format!("audio.{name}.gain"), fader.rect);
                        let (meter_rect, _) =
                            ui.allocate_exact_size(egui::vec2(14.0, fader_h), Sense::hover());
                        ui.painter()
                            .rect_filled(meter_rect, 0.0, Color32::from_rgb(10, 14, 18));
                        for (ch, peak) in peaks.into_iter().enumerate() {
                            let frac = ((audio::db(peak) + 60.0) / 60.0).clamp(0.0, 1.0);
                            let x = meter_rect.left() + 1.5 + ch as f32 * 6.0;
                            let fill = Rect::from_min_max(
                                egui::pos2(x, meter_rect.bottom() - meter_rect.height() * frac),
                                egui::pos2(x + 4.5, meter_rect.bottom()),
                            );
                            ui.painter().rect_filled(
                                fill,
                                0.0,
                                if peak >= 1.0 {
                                    RED
                                } else {
                                    Color32::from_rgb(35, 195, 65)
                                },
                            );
                        }
                    });
                    // Bottom row inside strip: headphone + rotary dial on Master, or M/A/B bus buttons on Inputs
                    ui.horizontal(|ui| {
                        if is_master {
                            ui.add_space(10.0);
                            ui.label(RichText::new("🎧").size(12.0));
                            let (dial, _) =
                                ui.allocate_exact_size(egui::vec2(18.0, 18.0), Sense::hover());
                            ui.painter().circle_filled(
                                dial.center(),
                                8.0,
                                Color32::from_rgb(56, 62, 68),
                            );
                            ui.painter().circle_stroke(
                                dial.center(),
                                8.0,
                                Stroke::new(1.0, Color32::from_rgb(30, 34, 38)),
                            );
                            ui.painter().line_segment(
                                [dial.center(), dial.center() + egui::vec2(-4.5, -3.5)],
                                Stroke::new(1.5, Color32::WHITE),
                            );
                        } else {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            ui.spacing_mut().button_padding = egui::vec2(3.0, 1.0);
                            for (v, label) in a.sends.iter_mut().zip(["M", "A", "B"]) {
                                ui.toggle_value(v, RichText::new(label).size(10.0));
                            }
                        }
                    });
                });
        });
        if a != before { Some(a) } else { None }
    }

    pub(crate) fn overlays(&mut self, ui: &mut egui::Ui) {
        let overlays = self.engine.project().overlays.clone();
        egui::ScrollArea::vertical()
            .id_salt("graphics.scroll")
            .show(ui, |ui| {
                ui.label(RichText::new("OVERLAY CHANNELS (1–4)").small().color(DIM));
                for (i, mut overlay) in overlays.into_iter().enumerate() {
                    ui.push_id(i, |ui| {
                        let before = overlay.clone();
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{:02}", i + 1)).color(TEAL).strong());
                            let response = ui.toggle_value(
                                &mut overlay.enabled,
                                if before.enabled { "ON PROGRAM" } else { "OFF" },
                            );
                            self.automation
                                .insert(format!("overlay.{}.toggle", i + 1), response.rect);
                            ui.label(RichText::new(format!("F{}", i + 1)).small().color(DIM));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let settings = ui
                                        .button(RichText::new("⚙").size(12.0))
                                        .on_hover_text("Overlay Settings for this channel");
                                    self.automation.insert(
                                        format!("overlay.{}.settings", i + 1),
                                        settings.rect,
                                    );
                                    if settings.clicked() {
                                        self.actions.push(Action::OverlaySettings(i));
                                    }
                                },
                            );
                        });
                        egui::CollapsingHeader::new("Title / position")
                            .default_open(i == 0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut overlay.title)
                                        .char_limit(96)
                                        .hint_text("Title"),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut overlay.subtitle)
                                        .char_limit(160)
                                        .hint_text("Subtitle"),
                                );
                                egui::ComboBox::from_id_salt("position")
                                    .selected_text(format!("{:?}", overlay.position))
                                    .show_ui(ui, |ui| {
                                        for (position, label) in [
                                            (Position::LowerLeft, "Lower left"),
                                            (Position::LowerRight, "Lower right"),
                                            (Position::TopLeft, "Top left"),
                                        ] {
                                            ui.selectable_value(
                                                &mut overlay.position,
                                                position,
                                                label,
                                            );
                                        }
                                    });
                            });
                        if overlay != before {
                            self.actions
                                .push(Action::Engine(Command::SetOverlay(i, overlay)));
                        }
                        ui.separator();
                    });
                }
            });
    }

    pub(crate) fn dialogs(&mut self, ctx: &egui::Context) {
        self.add_input_dialog(ctx);
        self.input_properties_dialog(ctx);
        self.settings_dialog(ctx);
        self.overlay_settings_dialog(ctx);
        self.recording_setup_dialog(ctx);
        self.streaming_setup_dialog(ctx);
        self.multicorder_dialog(ctx);
        self.playlist_dialog(ctx);
        self.shortcuts_dialog(ctx);
        self.remove_input_confirmation(ctx);
        if self.about {
            let mut open = true;
            egui::Window::new("About RustMix")
                .open(&mut open)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.heading(RichText::new("RustMix 0.1").color(TEAL));
                    ui.label("Independent Rust native production switcher.\nOriginal procedural graphics and clean-room UI implementation.");
                    ui.separator();
                    ui.collapsing("Session command log", |ui| {
                        for line in &self.history {
                            ui.label(line);
                        }
                    });
                });
            if !open {
                self.about = false;
            }
        }
        if self.confirm_new {
            egui::Window::new("New preset?")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Replace the current workspace with the default preset?");
                    ui.horizontal(|ui| {
                        if ui.button("Save current").clicked() {
                            self.actions.push(Action::Save(false));
                        }
                        if ui
                            .add_enabled(self.job.is_none(), egui::Button::new("Create new preset"))
                            .clicked()
                        {
                            self.reset_editors();
                            self.engine = Engine::default();
                            self.dock = default_dock(false);
                            self.path = None;
                            self.saved_revision = 0;
                            self.textures.clear();
                            self.last_demo_frame = None;
                            self.confirm_new = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_new = false;
                        }
                    });
                });
        }
        if self.confirm_close {
            egui::Window::new("Unsaved preset")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Save before closing, or explicitly close without saving.");
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.actions.push(Action::Save(false));
                        }
                        if ui.button("Close without saving").clicked() {
                            self.actions.push(Action::CloseDiscard);
                        }
                        if ui.button("Keep working").clicked() {
                            self.confirm_close = false;
                        }
                    });
                });
        }
        if self.confirm_open {
            egui::Window::new("Open another preset?")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("The current preset has unsaved changes. Save it before replacing the workspace.");
                    ui.horizontal(|ui| {
                        if ui.button("Save current").clicked() {
                            self.actions.push(Action::Save(false));
                        }
                        if ui
                            .add_enabled(self.job.is_none(), egui::Button::new("Open project"))
                            .clicked()
                        {
                            self.actions.push(Action::ConfirmOpen);
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_open = false;
                            self.pending_open = None;
                        }
                    });
                });
        }
    }
}
