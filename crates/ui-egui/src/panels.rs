use super::*;
use eframe::egui::{self, Align2, FontId, Rect, Sense, Stroke, Vec2};

/// Vertical group label used between the mixer buses and input strips.
/// Original RustMix artwork; text is drawn by the pinned egui painter.
fn mixer_rail(ui: &mut egui::Ui, height: f32, label: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, height), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, MIXER_RAIL);
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
        let foreground = if program {
            Color32::WHITE
        } else {
            Color32::BLACK
        };
        egui::Frame::new()
            .fill(color)
            .inner_margin(4.0)
            .show(ui, |ui| {
                ui.set_min_width((ui.available_width() - 1.0).max(20.0));
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [(ui.available_width() - 150.0).max(40.0), 18.0],
                        egui::Label::new(
                            RichText::new(format!("{}  {}", id.0, input.name))
                                .strong()
                                .color(foreground),
                        )
                        .truncate(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let gear = ui
                            .add(
                                egui::Button::new(RichText::new("⚙").size(12.0))
                                    .min_size(egui::vec2(23.0, 17.0)),
                            )
                            .on_hover_text("Source properties for this monitor");
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
                        ui.label(
                            RichText::new(if input.online {
                                if program {
                                    "PROGRAM · DEMO"
                                } else {
                                    "PREVIEW"
                                }
                            } else {
                                "OFFLINE · BLACK"
                            })
                            .size(10.0)
                            .strong()
                            .color(foreground),
                        );
                    });
                });
            });
        let available = ui.available_size();
        let width = available
            .x
            .max(16.0)
            .min((available.y - 30.0).max(9.0) * 16.0 / 9.0);
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
        if visible > 0.01 && input.online {
            let tint = Color32::WHITE.gamma_multiply(visible);
            let title = match input.kind {
                InputKind::Grid => "CAMERA A / MOTION LAB",
                InputKind::Geometry => "CAMERA B / GEOMETRY",
                InputKind::Bars => "CAMERA C / COLOR REFERENCE",
                InputKind::Graphics => "RUSTMIX / GRAPHICS",
                InputKind::Studio => "NEWSROOM / STUDIO 01",
                InputKind::Color => "COLOR / GENERATOR",
                InputKind::Countdown => "COUNTDOWN / STANDBY",
            };
            // Source titles stay outside the transitioning image: identity is shown in the monitor header.
            if !program || self.engine.transition().is_none() {
                ui.painter().text(
                    rect.left_top() + egui::vec2(20.0, 18.0),
                    Align2::LEFT_TOP,
                    title,
                    FontId::proportional((rect.width() / 35.0).clamp(10.0, 18.0)),
                    tint,
                );
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
        }
        if program {
            for (i, o) in p.overlays.iter().enumerate() {
                if o.enabled {
                    let offset = i as f32 * 0.105;
                    let (left, top) = match o.position {
                        Position::LowerLeft => (0.05, 0.78 - offset),
                        Position::LowerRight => (0.38, 0.78 - offset),
                        Position::TopLeft => (0.05, 0.08 + offset),
                    };
                    let text_rect = Rect::from_min_size(
                        rect.lerp_inside(egui::vec2(left + 0.02, top)),
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
        ui.painter().rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, color.gamma_multiply(0.6)),
            egui::StrokeKind::Inside,
        );
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "DEMO {}  ·  canvas {}×{} / {}p target",
                    if program { "640×360" } else { "384×216" },
                    p.preferences.width,
                    p.preferences.height,
                    p.preferences.rate.label()
                ))
                .small()
                .color(DIM),
            );
        });
    }
    pub(crate) fn switcher(&mut self, ui: &mut egui::Ui) {
        if self.multiview {
            ui.horizontal(|ui| {
                ui.label(RichText::new("MULTIVIEW · click a tile to select Preview").color(TEAL));
                if ui.button("Return to switcher").clicked() {
                    self.actions.push(Action::Multiview);
                }
            });
            self.source_tiles(ui, true);
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
        let center = 132.0;
        let side = ((available.x - center - 110.0) / 2.0).max(200.0);
        ui.horizontal_top(|ui| {
            ui.add_space(15.0);
            ui.allocate_ui_with_layout(
                egui::vec2(side, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.monitor(ui, false),
            );
            ui.add_space(20.0);
            ui.allocate_ui_with_layout(
                egui::vec2(center, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(center);
                    for (id, label, command) in [
                        ("switcher.quickPlay", "Quick Play", Command::Cut),
                        ("switcher.cut", "Cut", Command::Cut),
                        (
                            "switcher.fade",
                            "Fade",
                            Command::Transition(TransitionKind::Fade),
                        ),
                    ] {
                        self.command_button(ui, id, label, command, None);
                    }
                    ui.add_enabled(
                        false,
                        egui::Button::new("Merge").min_size(egui::vec2(center, 28.0)),
                    )
                    .on_disabled_hover_text("Merge transition is not implemented");
                    self.command_button(
                        ui,
                        "switcher.wipe",
                        "Wipe",
                        Command::Transition(TransitionKind::Wipe),
                        None,
                    );
                    let mut duration = self.engine.project().duration_ms;
                    egui::ComboBox::from_id_salt("switcher.duration")
                        .width(center - 16.0)
                        .selected_text(RichText::new(format!("{duration} ms")).size(11.0))
                        .show_ui(ui, |ui| {
                            for value in [250, 500, 1000, 1500, 2000, 3000] {
                                if ui
                                    .selectable_value(&mut duration, value, format!("{value} ms"))
                                    .changed()
                                {
                                    self.actions
                                        .push(Action::Engine(Command::SetDuration(value)));
                                }
                            }
                        });
                    self.command_button(
                        ui,
                        "switcher.ftb",
                        if self.engine.black_level() > 0.5 {
                            "Restore"
                        } else {
                            "FTB"
                        },
                        Command::FadeBlack,
                        None,
                    );
                    ui.add_space(4.0);
                    for row in 0..2 {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 3.0;
                            for col in 0..4 {
                                let slot = row * 4 + col;
                                if let Some(o) = self.engine.project().overlays.get(slot).cloned() {
                                    let mut next = o.clone();
                                    let r = ui.add(
                                        egui::Button::new((slot + 1).to_string())
                                            .selected(o.enabled)
                                            .min_size(egui::vec2(24.0, 22.0)),
                                    );
                                    self.automation
                                        .insert(format!("overlay.{}.toggle", slot + 1), r.rect);
                                    if r.clicked() {
                                        next.enabled = !next.enabled;
                                        self.actions
                                            .push(Action::Engine(Command::SetOverlay(slot, next)));
                                    }
                                } else {
                                    ui.add_enabled(
                                        false,
                                        egui::Button::new((slot + 1).to_string())
                                            .min_size(egui::vec2(24.0, 22.0)),
                                    )
                                    .on_disabled_hover_text(
                                        "This milestone provides four overlay slots",
                                    );
                                }
                            }
                        });
                    }
                    ui.add_space(6.0);
                    let progress = self.engine.transition().map_or(0.0, |t| t.progress);
                    let auto = self.engine.transition().is_some_and(|t| !t.manual);
                    // vMix-style vertical T-Bar: the handle rests at the top and is
                    // pulled down to take Preview. The engine keeps progress 0..1.
                    let mut handle = 1.0 - progress;
                    let reserve = 56.0;
                    let tbar_length = (ui.available_height() - reserve).clamp(56.0, 520.0);
                    ui.vertical_centered(|ui| {
                        let response = ui
                            .add_enabled_ui(!auto, |ui| {
                                ui.spacing_mut().slider_width = tbar_length;
                                ui.spacing_mut().interact_size.y = 34.0;
                                ui.visuals_mut().widgets.inactive.bg_fill =
                                    Color32::from_gray(133);
                                ui.visuals_mut().widgets.hovered.bg_fill = Color32::from_gray(164);
                                ui.visuals_mut().widgets.active.bg_fill = Color32::from_gray(185);
                                ui.add(
                                    egui::Slider::new(&mut handle, 0.0..=1.0)
                                        .show_value(false)
                                        .vertical()
                                        .handle_shape(egui::style::HandleShape::Rect {
                                            aspect_ratio: 1.2,
                                        }),
                                )
                            })
                            .inner
                            .on_hover_text(
                                "Manual transition · drag down from the top to take Preview; return to the top to cancel",
                            );
                        self.automation.insert("switcher.tbar".into(), response.rect);
                        if response.changed() && !self.tbar_latched {
                            let value = 1.0 - handle;
                            self.actions
                                .push(Action::Engine(Command::SetManualProgress(value)));
                            if value >= 1.0 {
                                self.tbar_latched = true;
                            }
                        }
                        // egui reports the final slider value on the release frame too.
                        // Keep the completion latch through that value before rearming.
                        if !ui.input(|i| i.pointer.primary_down()) {
                            self.tbar_latched = false;
                        }
                        ui.label(
                            RichText::new(format!(
                                "{:03.0}% {}",
                                progress * 100.0,
                                if auto {
                                    "AUTO"
                                } else if progress > 0.0 {
                                    "T-BAR"
                                } else {
                                    "READY"
                                }
                            ))
                            .monospace()
                            .small(),
                        );
                    });
                    self.command_button(
                        ui,
                        "switcher.cancel",
                        "Cancel",
                        Command::CancelTransition,
                        None,
                    );
                },
            );
            ui.add_space(20.0);
            ui.allocate_ui_with_layout(
                egui::vec2(side, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.monitor(ui, true),
            );
        });
    }
    pub(crate) fn inputs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 3.0;
            let selected_all = self.category.is_none();
            let all = ui
                .add(
                    egui::Button::new(RichText::new("All").size(11.0))
                        .selected(selected_all)
                        .min_size(egui::vec2(34.0, 21.0)),
                )
                .on_hover_text("Show every input category");
            self.automation
                .insert("inputs.category.all".into(), all.rect);
            if all.clicked() {
                self.category = None;
            }
            for (kind, color) in InputKind::ALL.into_iter().zip([
                Color32::from_rgb(145, 0, 0),
                PROGRAM,
                PREVIEW,
                Color32::from_rgb(132, 29, 145),
                Color32::from_rgb(0, 144, 229),
                Color32::from_rgb(30, 30, 95),
                Color32::from_rgb(30, 131, 116),
            ]) {
                let selected = self.category == Some(kind);
                let response = ui
                    .add(
                        egui::Button::new(RichText::new(" ").size(14.0))
                            .fill(color)
                            .stroke(if selected {
                                Stroke::new(1.5, Color32::WHITE)
                            } else {
                                Stroke::new(0.5, Color32::from_gray(105))
                            })
                            .min_size(if selected {
                                egui::vec2(26.0, 25.0)
                            } else {
                                egui::vec2(24.0, 21.0)
                            }),
                    )
                    .on_hover_text(if selected {
                        format!("{} · click to clear the category filter", kind.label())
                    } else {
                        format!("Filter inputs by {}", kind.label())
                    });
                if selected {
                    let bar = egui::Rect::from_min_size(
                        response.rect.left_bottom() - egui::vec2(0.0, 3.0),
                        egui::vec2(response.rect.width(), 3.0),
                    );
                    ui.painter().rect_filled(bar, 0.0, Color32::WHITE);
                }
                self.automation
                    .insert(format!("inputs.category.{kind:?}"), response.rect);
                if response.clicked() {
                    self.category = if selected { None } else { Some(kind) };
                }
            }
            ui.add_space(4.0);
            let search = ui.add(
                egui::TextEdit::singleline(&mut self.input_filter)
                    .hint_text("Search inputs")
                    .desired_width(150.0)
                    .char_limit(64),
            );
            self.automation
                .insert("inputs.filter.search".into(), search.rect);
            if self.category.is_some() || !self.input_filter.is_empty() {
                let clear = ui.small_button("Clear filters");
                self.automation
                    .insert("inputs.filter.clear".into(), clear.rect);
                if clear.clicked() {
                    self.category = None;
                    self.input_filter.clear();
                }
            }
            let filter = self.input_filter.trim().to_lowercase();
            let inputs = &self.engine.project().inputs;
            let visible = inputs
                .iter()
                .filter(|input| {
                    input.name.to_lowercase().contains(&filter)
                        && self.category.is_none_or(|kind| kind == input.kind)
                })
                .count();
            ui.label(
                RichText::new(format!(
                    "{} · {visible}/{}",
                    self.category.map_or("All inputs", InputKind::label),
                    inputs.len()
                ))
                .small()
                .color(DIM),
            );
            let offline = inputs.iter().filter(|input| !input.online).count();
            if offline > 0 {
                ui.colored_label(RED, format!("{offline} offline"));
            }
        });
        self.source_tiles(ui, false);
    }
    fn source_tiles(&mut self, ui: &mut egui::Ui, large: bool) {
        let project = self.engine.project();
        let inputs = project.inputs.clone();
        let program = project.program;
        let preview = project.preview;
        let target = if large { 300.0 } else { 235.0 };
        let columns = ((ui.available_width() + 8.0) / (target + 8.0))
            .floor()
            .max(1.0) as usize;
        let tile_width = ((ui.available_width() - columns.saturating_sub(1) as f32 * 8.0)
            / columns as f32)
            .max(80.0);
        let filter = self.input_filter.trim().to_lowercase();
        let filtered: Vec<_> = inputs
            .into_iter()
            .filter(|input| {
                input.name.to_lowercase().contains(&filter)
                    && self.category.is_none_or(|kind| kind == input.kind)
            })
            .collect();
        egui::ScrollArea::vertical().id_salt(if large { "multiview.scroll" } else { "inputs.scroll" }).show(ui, |ui| {
            if filtered.is_empty() {
                ui.add_space(20.0);
                ui.strong("No matching inputs");
                ui.label(RichText::new("Change the search or clear the category filter above.").color(DIM));
            }
            for row in filtered.chunks(columns) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    for input in row {
                        ui.push_id(("input", input.id.0), |ui| {
                            ui.allocate_ui_with_layout(egui::vec2(tile_width, tile_width * 9.0 / 16.0 + 92.0),
                                egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                let color = if input.id == program { PROGRAM } else if input.id == preview { PREVIEW } else { IDLE_TILE };
                                egui::Frame::new().fill(Color32::from_rgb(20,25,29)).stroke(Stroke::new(1.0, color)).inner_margin(2.0).show(ui, |ui| {
                                    ui.set_width((tile_width - 6.0).max(60.0));
                                    let foreground = if input.id == preview { Color32::BLACK } else { Color32::WHITE };
                                    let state = if !input.online { "OFFLINE" } else if input.id == program { "PGM" } else if input.id == preview { "PVW" } else { "" };
                                    egui::Frame::new().fill(color).inner_margin(egui::Margin::symmetric(4, 2)).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 3.0;
                                            ui.add_sized([(ui.available_width() - 58.0).max(24.0), 16.0], egui::Label::new(RichText::new(format!("{}  {}", input.id.0, input.name)).strong().color(foreground)).truncate());
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                let remove = ui.add(egui::Button::new(RichText::new("×").size(13.0).color(foreground)).min_size(egui::vec2(17.0, 16.0)));
                                                self.automation.insert(format!("input.{}.remove", input.id.0), remove.rect);
                                                if remove.on_hover_text("Remove this input…").clicked() { self.actions.push(Action::RequestRemove(input.id)); }
                                                if !state.is_empty() {
                                                    ui.label(RichText::new(state).size(9.0).strong().color(foreground));
                                                }
                                            });
                                        });
                                    });
                                    let size = egui::vec2(ui.available_width(), ui.available_width() * 9.0 / 16.0);
                                    let response = if let Some(texture) = self.textures.get(&input.id) {
                                        ui.add(egui::Image::new(texture).fit_to_exact_size(size).sense(Sense::click()))
                                    } else { ui.allocate_response(size, Sense::click()) };
                                    self.automation.insert(format!("preview.input.{}", input.id.0), response.rect);
                                    if !input.online {
                                        ui.painter().rect_filled(response.rect, 0.0, Color32::BLACK);
                                        ui.painter().text(response.rect.center(), Align2::CENTER_CENTER, "OFFLINE / BLACK", FontId::proportional(12.0), RED);
                                        self.automation.insert(format!("input.{}.offline", input.id.0), response.rect);
                                    }
                                    if response.clicked() { self.actions.push(Action::Engine(Command::SelectPreview(input.id))); }
                                    response.on_hover_text(format!("{} · {}\nClick to select Preview. Program changes only with Cut / Fade.", input.name, input.kind.label())).context_menu(|ui| {
                                        self.command_button(ui, "preview.select", "Select Preview", Command::SelectPreview(input.id), None);
                                        if ui.button("Properties / rename…").clicked() { self.actions.push(Action::Properties(input.id)); ui.close(); }
                                        let check = self.engine.enabled(&Command::RemoveInput(input.id));
                                        if ui.add_enabled(check.is_ok(), egui::Button::new("Remove input…"))
                                            .on_disabled_hover_text(check.err().map(|error| error.to_string()).unwrap_or_default()).clicked() {
                                            self.actions.push(Action::RequestRemove(input.id)); ui.close();
                                        }
                                    });
                                    ui.add_space(2.0);
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 2.0;
                                        for slot in 0..4 {
                                            if let Some(mut overlay) = self.engine.project().overlays.get(slot).cloned() {
                                                let selected = overlay.enabled && overlay.title == input.name;
                                                let button = ui.add(egui::Button::new(RichText::new((slot + 1).to_string()).size(10.0)).selected(selected).min_size(egui::vec2(20.0, 19.0)))
                                                    .on_hover_text("Use this source name as an overlay title");
                                                self.automation.insert(format!("input.{}.overlay.{}", input.id.0, slot + 1), button.rect);
                                                if button.clicked() { overlay.enabled = !selected; overlay.title = input.name.clone(); overlay.subtitle = input.kind.label().into(); self.actions.push(Action::Engine(Command::SetOverlay(slot, overlay))); }
                                            }
                                        }
                                        let select = ui.add(egui::Button::new(RichText::new("PVW").size(10.0)).min_size(egui::vec2(32.0, 19.0))).on_hover_text("Select this source for Preview");
                                        self.automation.insert(format!("input.{}.preview", input.id.0), select.rect);
                                        if select.clicked() { self.actions.push(Action::Engine(Command::SelectPreview(input.id))); }
                                        let take = Command::TakeInput(input.id);
                                        let check = self.engine.enabled(&take);
                                        let cut = ui.add_enabled(check.is_ok(), egui::Button::new(RichText::new("Cut").size(10.0)).min_size(egui::vec2(30.0, 19.0)))
                                            .on_hover_text("Take this source directly to Program")
                                            .on_disabled_hover_text(check.err().map(|error| error.to_string()).unwrap_or_default());
                                        self.automation.insert(format!("input.{}.cut", input.id.0), cut.rect);
                                        if cut.clicked() { self.actions.push(Action::Engine(take)); }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 2.0;
                                        for slot in 5..=8 { ui.add_enabled(false, egui::Button::new(RichText::new(slot.to_string()).size(10.0)).min_size(egui::vec2(20.0, 18.0))).on_disabled_hover_text("Four overlay slots are implemented"); }
                                        let audio = ui.add(egui::Button::new(RichText::new("Audio").size(10.0)).selected(!input.audio.muted).min_size(egui::vec2(34.0, 18.0))).on_hover_text("Toggle simulated input mute");
                                        self.automation.insert(format!("input.{}.audio", input.id.0), audio.rect);
                                        if audio.clicked() { let mut channel = input.audio.clone(); channel.muted = !channel.muted; self.actions.push(Action::Engine(Command::SetInputAudio(input.id, channel))); }
                                        let edit = ui.add(egui::Button::new(RichText::new("⚙").size(11.0)).min_size(egui::vec2(20.0, 18.0))).on_hover_text("Input properties");
                                        self.automation.insert(format!("input.{}.properties", input.id.0), edit.rect);
                                        if edit.clicked() { self.actions.push(Action::Properties(input.id)); }
                                    });
                                    ui.add(egui::Label::new(RichText::new(if input.online { input.kind.label() } else { "Offline · black fallback" }).size(10.0).color(if input.online { DIM } else { RED })).truncate());
                                });
                            });
                        });
                    }
                });
                ui.add_space(3.0);
            }
        });
    }

    pub(crate) fn audio(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("SIMULATED METERS").color(TEAL).small());
            ui.label(
                RichText::new("48 kHz planned · no audio output")
                    .small()
                    .color(DIM),
            );
        });
        let p = self.engine.project().clone();
        let solo = p.inputs.iter().any(|i| i.audio.solo);
        let height = (ui.available_height() - 47.0).clamp(80.0, 225.0);
        egui::ScrollArea::horizontal()
            .id_salt("audio.strips")
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    mixer_rail(ui, height, "OUTPUTS");
                    for (bus, name) in ["MASTER", "AUX A", "AUX B"].into_iter().enumerate() {
                        if let Some(a) = p.buses.get(bus) {
                            let peaks = audio::bus_peaks(&p, bus, self.now);
                            let changed = self.strip(
                                ui,
                                (format!("bus.{bus}"), name),
                                a.clone(),
                                peaks,
                                height,
                                false,
                            );
                            if let Some(a) = changed {
                                self.actions.push(Action::Engine(Command::SetBus(bus, a)));
                            }
                        }
                    }
                    ui.add_space(5.0);
                    mixer_rail(ui, height, "INPUTS");
                    for input in &p.inputs {
                        let peaks = audio::input_peaks(input, solo, self.now);
                        let changed = self.strip(
                            ui,
                            (format!("input.{}", input.id.0), &input.name),
                            input.audio.clone(),
                            peaks,
                            height,
                            true,
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
        let detail = self.selected_audio.and_then(|id| p.input(id));
        ui.label(
            RichText::new(
                detail.map_or("Adjust a strip to inspect input routing".into(), |i| {
                    format!(
                        "{}  ·  {}  ·  M/A/B: {:?}",
                        i.name,
                        audio::gain_label(&i.audio),
                        i.audio.sends
                    )
                }),
            )
            .small()
            .color(DIM),
        );
    }
    fn strip(
        &mut self,
        ui: &mut egui::Ui,
        identity: (String, &str),
        mut a: AudioChannel,
        peaks: [f32; 2],
        height: f32,
        sends: bool,
    ) -> Option<AudioChannel> {
        let (id, name) = identity;
        let before = a.clone();
        ui.push_id(id, |ui| {
            ui.vertical(|ui| {
                ui.set_width(80.0);
                egui::Frame::new()
                    .fill(CHIP_BG)
                    .inner_margin(egui::Margin::symmetric(3, 1))
                    .show(ui, |ui| {
                        ui.set_width(70.0);
                        ui.add(egui::Label::new(RichText::new(name).small().strong()).truncate());
                    });
                ui.label(
                    RichText::new(audio::gain_label(&a))
                        .monospace()
                        .size(11.0)
                        .color(TEAL),
                );
                ui.horizontal(|ui| {
                    let fader = ui
                        .add_sized(
                            [27.0, (height - 82.0).max(42.0)],
                            egui::Slider::new(&mut a.gain_db, -60.0..=12.0)
                                .vertical()
                                .show_value(false),
                        )
                        .on_hover_text("Gain, -60 to +12 dB · simulated signal");
                    self.automation
                        .insert(format!("audio.{name}.gain"), fader.rect);
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(35.0, (height - 82.0).max(42.0)),
                        Sense::hover(),
                    );
                    ui.painter()
                        .rect_filled(rect, 1.0, Color32::from_rgb(9, 13, 17));
                    for (ch, peak) in peaks.into_iter().enumerate() {
                        let frac = ((audio::db(peak) + 60.0) / 60.0).clamp(0.0, 1.0);
                        let x = rect.left() + 3.0 + ch as f32 * 9.0;
                        let fill = Rect::from_min_max(
                            egui::pos2(x, rect.bottom() - rect.height() * frac),
                            egui::pos2(x + 5.0, rect.bottom()),
                        );
                        ui.painter().rect_filled(
                            fill,
                            0.0,
                            if peak >= 1.0 {
                                RED
                            } else if peak > 0.7 {
                                Color32::from_rgb(224, 181, 79)
                            } else {
                                TEAL
                            },
                        );
                    }
                    for db in [0, -12, -24, -48] {
                        let y = rect.bottom() - rect.height() * ((db + 60) as f32 / 60.0);
                        ui.painter().text(
                            egui::pos2(rect.right(), y),
                            Align2::RIGHT_CENTER,
                            db.to_string(),
                            FontId::monospace(7.0),
                            DIM,
                        );
                    }
                });
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    let mute = ui
                        .add(
                            egui::Button::new(RichText::new("M").size(11.0))
                                .selected(a.muted)
                                .min_size(egui::vec2(22.0, 19.0)),
                        )
                        .on_hover_text("Mute this channel across all buses");
                    if mute.clicked() {
                        a.muted = !a.muted;
                    }
                    let solo = ui
                        .add(
                            egui::Button::new(RichText::new("S").size(11.0))
                                .selected(a.solo)
                                .min_size(egui::vec2(22.0, 19.0)),
                        )
                        .on_hover_text("Solo this channel in the simulated mix");
                    if solo.clicked() {
                        a.solo = !a.solo;
                    }
                });
                if sends {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for (v, label) in a.sends.iter_mut().zip(["M", "A", "B"]) {
                            ui.toggle_value(v, RichText::new(label).size(10.0))
                                .on_hover_text("Enable send to this bus");
                        }
                    });
                }
            });
        });
        if a != before { Some(a) } else { None }
    }
    pub(crate) fn overlays(&mut self, ui: &mut egui::Ui) {
        let overlays = self.engine.project().overlays.clone();
        egui::ScrollArea::vertical()
            .id_salt("graphics.scroll")
            .show(ui, |ui| {
                ui.label(RichText::new("PROGRAM GRAPHICS").small().color(DIM));
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
                ui.label(
                    RichText::new("Four independent slots\nText is clipped to its safe area.")
                        .small()
                        .color(DIM),
                );
            });
    }
    pub(crate) fn dialogs(&mut self, ctx: &egui::Context) {
        self.add_input_dialog(ctx);
        self.input_properties_dialog(ctx);
        self.settings_dialog(ctx);
        self.remove_input_confirmation(ctx);
        if self.about {
            let mut open = true;
            egui::Window::new("About RustMix").open(&mut open).collapsible(false).show(ctx,|ui|{
            ui.heading(RichText::new("RustMix 0.1").color(TEAL));ui.label("Independent Rust native production switcher.\nExperimental synthetic UI milestone.\nNot affiliated with vMix or StudioCoast.");ui.separator();ui.label("Original procedural demo graphics.\neframe / egui: MIT OR Apache-2.0\negui_dock and rfd: MIT\nNo proprietary reference assets are included.");
            ui.collapsing("Session command log",|ui|{for line in &self.history{ui.label(line);}});
        });
            if !open {
                self.about = false;
            }
        }
        if self.confirm_new {
            egui::Window::new("New project?").collapsible(false).show(ctx,|ui|{ui.label("Replace the current workspace with the demo project?\nSave the current project first if you need to keep it.");ui.horizontal(|ui|{
            if ui.button("Save current").clicked(){self.actions.push(Action::Save(false));}
            if ui.add_enabled(self.job.is_none(),egui::Button::new("Create demo project")).clicked(){self.reset_editors();self.engine=Engine::default();self.dock=default_dock(false);self.path=None;self.saved_revision=0;self.textures.clear();self.last_demo_frame=None;self.confirm_new=false;}
            if ui.button("Cancel").clicked(){self.confirm_new=false;}
        });});
        }
        if self.confirm_close {
            egui::Window::new("Unsaved project")
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
            egui::Window::new("Open another project?").collapsible(false).show(ctx,|ui|{
                ui.label("The current project has unsaved changes. Save it before replacing the workspace.");
                ui.horizontal(|ui|{
                    if ui.button("Save current").clicked(){self.actions.push(Action::Save(false));}
                    if ui.add_enabled(self.job.is_none(),egui::Button::new("Open project")).clicked(){self.actions.push(Action::ConfirmOpen);}
                    if ui.button("Cancel").clicked(){self.confirm_open=false;self.pending_open=None;}
                });
            });
        }
    }
}
