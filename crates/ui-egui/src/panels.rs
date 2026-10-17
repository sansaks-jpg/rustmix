use super::*;
use eframe::egui::{self, Align2, FontId, Rect, Sense, Stroke, Vec2};

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
            .inner_margin(4.0)
            .show(ui, |ui| {
                ui.set_min_width((ui.available_width() - 1.0).max(20.0));
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("{}  {}", id.0, input.name))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .truncate(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
                            .strong()
                            .color(Color32::WHITE),
                        );
                    });
                });
            });
        let available = ui.available_size();
        let width = available
            .x
            .max(16.0)
            .min((available.y - 30.0).max(9.0) * 16.0 / 9.0);
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(width, width * 9.0 / 16.0), Sense::hover());
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
                    let remaining = 60 - (self.now / 1000) % 60;
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
                    "SYNTHETIC  ·  {}×{}  ·  {}p",
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
        let center = 110.0;
        let side = ((available.x - center - 100.0) / 2.0).max(200.0);
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
                        .selected_text(format!("{duration} ms"))
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
                                            .min_size(egui::vec2(23.0, 22.0)),
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
                                            .min_size(egui::vec2(23.0, 22.0)),
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
                    let mut manual = progress;
                    let r = ui.add_enabled(
                        !auto,
                        egui::Slider::new(&mut manual, 0.0..=1.0).show_value(false),
                    );
                    self.automation.insert("switcher.tbar".into(), r.rect);
                    if !ui.input(|i| i.pointer.primary_down()) {
                        self.tbar_latched = false;
                    }
                    if r.changed() && !self.tbar_latched {
                        self.actions
                            .push(Action::Engine(Command::SetManualProgress(manual)));
                        if manual >= 1.0 {
                            self.tbar_latched = true;
                        }
                    }
                    ui.label(
                        RichText::new(format!(
                            "{:03.0}% {}",
                            progress * 100.0,
                            if auto { "AUTO" } else { "T-BAR" }
                        ))
                        .monospace()
                        .small(),
                    );
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
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.category.is_none(), "All")
                .clicked()
            {
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
                if ui
                    .add(
                        egui::Button::new(" ")
                            .fill(color)
                            .stroke(Stroke::new(
                                if self.category == Some(kind) {
                                    2.0
                                } else {
                                    0.5
                                },
                                Color32::GRAY,
                            ))
                            .min_size(egui::vec2(20.0, 19.0)),
                    )
                    .on_hover_text(kind.label())
                    .clicked()
                {
                    self.category = Some(kind);
                }
            }
            ui.add(
                egui::TextEdit::singleline(&mut self.input_filter)
                    .hint_text("Search inputs")
                    .desired_width(130.0),
            );
        });
        self.source_tiles(ui, false);
    }
    fn source_tiles(&mut self, ui: &mut egui::Ui, large: bool) {
        let inputs = self.engine.project().inputs.clone();
        let program = self.engine.project().program;
        let preview = self.engine.project().preview;
        let target = if large { 300.0 } else { 205.0 };
        let columns = ((ui.available_width() + 8.0) / (target + 8.0))
            .floor()
            .max(1.0) as usize;
        let tile_width = ((ui.available_width() - (columns.saturating_sub(1) as f32 * 8.0))
            / columns as f32)
            .max(80.0);
        let filter = self.input_filter.to_lowercase();
        let filtered: Vec<_> = inputs
            .into_iter()
            .filter(|i| {
                i.name.to_lowercase().contains(&filter) && self.category.is_none_or(|k| k == i.kind)
            })
            .collect();
        egui::ScrollArea::vertical().id_salt("inputs.scroll").show(ui,|ui| {
            if filtered.is_empty(){ui.label("No inputs match the filter.");}
            for row in filtered.chunks(columns) {
                ui.horizontal_top(|ui| {
                    for input in row {
                        ui.push_id(("input",input.id.0),|ui| {
                            ui.allocate_ui_with_layout(egui::vec2(tile_width,tile_width*9.0/16.0+62.0),egui::Layout::top_down(egui::Align::Min),|ui| {
                                let color=if input.id==program{PROGRAM}else if input.id==preview{PREVIEW}else{Color32::from_rgb(53,64,77)};
                                egui::Frame::new().fill(Color32::from_rgb(20,25,29)).stroke(Stroke::new(1.0,color)).inner_margin(1.0).show(ui,|ui| {
                                    ui.set_width((tile_width-4.0).max(60.0));
                                    let state=if input.id==program{"PGM"}else if input.id==preview{"PVW"}else{"IN"};
                                    egui::Frame::new().fill(color).inner_margin(2.0).show(ui,|ui| {
                                        ui.horizontal(|ui| {
                                            ui.add_sized([ui.available_width()-35.0,16.0],egui::Label::new(RichText::new(format!("{}  {}",input.id.0,input.name)).strong().color(Color32::WHITE)).truncate());
                                            ui.label(RichText::new(state).small().color(Color32::WHITE));
                                        });
                                    });
                                    let size=egui::vec2(ui.available_width(),ui.available_width()*9.0/16.0);
                                    let response=if let Some(t)=self.textures.get(&input.id){ui.add(egui::Image::new(t).fit_to_exact_size(size).sense(Sense::click()))}else{ui.allocate_response(size,Sense::click())};
                                    self.automation.insert(format!("preview.input.{}",input.id.0),response.rect);
                                    if response.clicked(){self.actions.push(Action::Engine(Command::SelectPreview(input.id)));}
                                    response.on_hover_text("Select into Preview. Program changes only with CUT / FADE.").context_menu(|ui| {
                                        self.command_button(ui,"preview.select","Select Preview",Command::SelectPreview(input.id),None);
                                        if ui.button("Properties / rename…").clicked(){self.actions.push(Action::Properties(input.id));ui.close();}
                                        self.command_button(ui,"input.remove","Remove input",Command::RemoveInput(input.id),None);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x=2.0;
                                        for slot in 0..4 {
                                            if let Some(mut overlay)=self.engine.project().overlays.get(slot).cloned() {
                                                let selected=overlay.enabled&&overlay.title==input.name;
                                                if ui.add(egui::Button::new(RichText::new((slot+1).to_string()).size(10.0)).selected(selected).min_size(egui::vec2(20.0,18.0))).on_hover_text("Use this input name as the lower-third title in this overlay slot").clicked(){overlay.enabled=!selected;overlay.title=input.name.clone();overlay.subtitle=input.kind.label().into();self.actions.push(Action::Engine(Command::SetOverlay(slot,overlay)));}
                                            }
                                        }
                                        if ui.small_button("PVW").clicked(){self.actions.push(Action::Engine(Command::SelectPreview(input.id)));}
                                        let take=Command::TakeInput(input.id);
                                        if ui.add_enabled(self.engine.enabled(&take).is_ok(),egui::Button::new(RichText::new("Cut").size(10.0))).on_hover_text("Explicitly take this source directly to Program").clicked(){self.actions.push(Action::Engine(take));}
                                    });
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x=2.0;
                                        for slot in 5..=8 {ui.add_enabled(false,egui::Button::new(RichText::new(slot.to_string()).size(10.0)).min_size(egui::vec2(20.0,17.0))).on_disabled_hover_text("Four overlay slots are implemented");}
                                        if ui.add(egui::Button::new(RichText::new("Audio").size(10.0)).selected(!input.audio.muted)).on_hover_text("Toggle simulated input audio mute").clicked(){let mut a=input.audio.clone();a.muted=!a.muted;self.actions.push(Action::Engine(Command::SetInputAudio(input.id,a)));}
                                        if ui.small_button("Edit").clicked(){self.actions.push(Action::Properties(input.id));}
                                    });

                                });
                            });
                        });
                    }
                });
                ui.add_space(2.0);
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
                            ui.separator();
                        }
                    }
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
                ui.set_width(78.0);
                ui.add(egui::Label::new(RichText::new(name).small().strong()).truncate());
                ui.label(
                    RichText::new(audio::gain_label(&a))
                        .monospace()
                        .size(11.0)
                        .color(TEAL),
                );
                ui.horizontal(|ui| {
                    let fader = ui
                        .add_sized(
                            [27.0, (height - 74.0).max(42.0)],
                            egui::Slider::new(&mut a.gain_db, -60.0..=12.0)
                                .vertical()
                                .show_value(false),
                        )
                        .on_hover_text("Gain, -60 to +12 dB · simulated signal");
                    self.automation
                        .insert(format!("audio.{name}.gain"), fader.rect);
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(35.0, (height - 74.0).max(42.0)),
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
                    ui.toggle_value(&mut a.muted, RichText::new("M").color(RED))
                        .on_hover_text("Mute this channel across all buses");
                    ui.toggle_value(&mut a.solo, RichText::new("S").color(TEAL))
                        .on_hover_text("Solo within input strips / bus strips");
                });
                if sends {
                    ui.horizontal(|ui| {
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
        if self.add_open {
            let mut open = true;
            egui::Window::new("Add synthetic input")
                .id(egui::Id::new("input.add.dialog"))
                .open(&mut open)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Supported procedural sources only. Hardware capture is unavailable.");
                    ui.add(egui::TextEdit::singleline(&mut self.add_name).char_limit(64));
                    egui::ComboBox::from_id_salt("add.kind")
                        .selected_text(self.add_kind.label())
                        .show_ui(ui, |ui| {
                            for kind in InputKind::ALL {
                                ui.selectable_value(&mut self.add_kind, kind, kind.label());
                            }
                        });
                    if ui.button("Add input").clicked() {
                        self.actions.push(Action::Engine(Command::AddInput {
                            name: self.add_name.clone(),
                            kind: self.add_kind,
                        }));
                        self.add_open = false;
                    }
                });
            if !open {
                self.add_open = false;
            }
        }
        if let Some(mut input) = self.properties.clone() {
            let mut open = true;
            egui::Window::new(format!("Input properties · {:02}", input.id.0))
                .id(egui::Id::new("input.properties"))
                .open(&mut open)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Name");
                    ui.add(egui::TextEdit::singleline(&mut input.name).char_limit(64));
                    egui::ComboBox::from_id_salt("properties.kind")
                        .selected_text(input.kind.label())
                        .show_ui(ui, |ui| {
                            for kind in InputKind::ALL {
                                ui.selectable_value(&mut input.kind, kind, kind.label());
                            }
                        });
                    ui.color_edit_button_srgb(&mut input.color);
                    ui.checkbox(
                        &mut input.online,
                        "Source online (offline = black + warning)",
                    );
                    ui.label(
                        RichText::new("Synthetic source · camera/media backends unavailable")
                            .color(DIM),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Apply properties").clicked() {
                            self.actions
                                .push(Action::Engine(Command::UpdateInput(input.clone())));
                            self.properties = None;
                        }
                        self.command_button(
                            ui,
                            "input.remove.properties",
                            "Remove input",
                            Command::RemoveInput(input.id),
                            None,
                        );
                    });
                });
            if !open {
                self.properties = None;
            } else if self.properties.is_some() {
                self.properties = Some(input);
            }
        }
        if self.settings {
            let mut open = true;
            let mut p = self.engine.project().preferences.clone();
            let mut name = self.engine.project().name.clone();
            egui::Window::new("Settings / workspace").id(egui::Id::new("settings")).open(&mut open).resizable(true).default_width(440.0).show(ctx,|ui|{
                ui.heading("Project format");ui.add(egui::TextEdit::singleline(&mut name).char_limit(120));
                egui::ComboBox::from_id_salt("canvas").selected_text(format!("{} × {}",p.width,p.height)).show_ui(ui,|ui|{for (w,h) in [(1280,720),(1920,1080),(2560,1440),(3840,2160)]{if ui.selectable_label(p.width==w,format!("{w} × {h}")).clicked(){p.width=w;p.height=h;}}});
                egui::ComboBox::from_id_salt("rate").selected_text(p.rate.label()).show_ui(ui,|ui|{for (n,d) in [(24,1),(25,1),(30,1),(50,1),(60,1),(24000,1001),(30000,1001),(60000,1001)]{if let Ok(rate)=Rate::new(n,d){ui.selectable_value(&mut p.rate,rate,rate.label());}}});
                ui.label(RichText::new("Canvas metadata only. Monitor raster: 640×360; source previews: 384×216.").color(DIM).small());
                ui.separator();ui.heading("Appearance / controls");ui.checkbox(&mut p.high_contrast,"High contrast text");ui.add(egui::Slider::new(&mut p.ui_scale,0.8..=1.5).text("UI scale"));ui.checkbox(&mut p.shortcuts,"Enable switching shortcuts");
                ui.label("1–7: Preview   Space: CUT   Enter: FADE\nB: Fade black / restore   F1–F4: Overlays\nEsc: Cancel transition   Ctrl+I: Add input\nCtrl+S: Save   Ctrl+Shift+S: Save As   Ctrl+O: Open");
                ui.label(RichText::new("Shortcuts pause while editing text. Key remapping is not available yet.").small().color(DIM));
                ui.separator();ui.label("Drag panel tabs to dock / float. Drag split boundaries to resize.");
                ui.horizontal(|ui|{if ui.button("Production layout").clicked(){self.actions.push(Action::Workspace(false));}
if ui.button("Audio layout").clicked(){self.actions.push(Action::Workspace(true));}});
                if ui.button("Recover checkpoint…").clicked(){self.actions.push(Action::Recover);}
                ui.label(RichText::new("Recover loads the last autosave. Current project is retained if validation fails.").small().color(DIM));
                ui.separator();ui.label("Unavailable: capture devices, real audio, encoders, recording, streaming.");
            });
            if p != self.engine.project().preferences {
                self.actions
                    .push(Action::Engine(Command::SetPreferences(p)));
            }
            if name != self.engine.project().name {
                self.actions
                    .push(Action::Engine(Command::RenameProject(name)));
            }
            if !open {
                self.settings = false;
            }
        }
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
            if ui.add_enabled(self.job.is_none(),egui::Button::new("Create demo project")).clicked(){self.engine=Engine::default();self.dock=default_dock(false);self.path=None;self.saved_revision=0;self.textures.clear();self.last_demo_frame=None;self.confirm_new=false;}
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
