use super::*;
use eframe::egui::{self, Align2, FontId, Rect, Sense, Stroke, Vec2};

/// Vertical blue group label (`OUTPUTS` / `INPUTS`) with modern rounded corners.
fn mixer_rail(ui: &mut egui::Ui, height: f32, label: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, height), Sense::hover());
    ui.painter().rect_filled(rect, 3.0, MIXER_RAIL);
    ui.painter().rect_stroke(
        rect,
        3.0,
        Stroke::new(1.0, Color32::from_rgb(38, 142, 230)),
        egui::StrokeKind::Inside,
    );
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_owned(), FontId::proportional(10.5), Color32::WHITE);
    ui.painter().add(
        egui::epaint::TextShape::new(rect.center(), galley, Color32::WHITE)
            .with_angle_and_anchor(-std::f32::consts::FRAC_PI_2, Align2::CENTER_CENTER),
    );
}

/// CPU reference effects applied before bus composition.
fn process_source_pixels(
    base: &[[u8; 4]],
    w: u32,
    h: u32,
    fx: &InputFx,
    pip_pixels: Option<&[[u8; 4]]>,
) -> Vec<[u8; 4]> {
    let resized;
    let src = if let Some((cw, ch, custom)) = &fx.custom_rgba
        && *cw > 0
        && *ch > 0
        && cw
            .checked_mul(*ch)
            .is_some_and(|size| custom.len() == size as usize)
    {
        if *cw == w && *ch == h {
            custom.as_slice()
        } else {
            resized = (0..h)
                .flat_map(|y| {
                    (0..w).map(move |x| {
                        let sx = u64::from(x) * u64::from(*cw) / u64::from(w.max(1));
                        let sy = u64::from(y) * u64::from(*ch) / u64::from(h.max(1));
                        custom
                            .get((sy * u64::from(*cw) + sx) as usize)
                            .copied()
                            .unwrap_or([0, 0, 0, 255])
                    })
                })
                .collect::<Vec<_>>();
            resized.as_slice()
        }
    } else {
        base
    };
    let identity = (fx.zoom - 1.0).abs() < 0.005
        && fx.pan_x.abs() < 0.005
        && fx.pan_y.abs() < 0.005
        && !fx.mirror
        && !fx.sharpen
        && fx.lift.abs() < 0.005
        && (fx.gain - 1.0).abs() < 0.005
        && (fx.saturation - 1.0).abs() < 0.005
        && !fx.chroma_key
        && pip_pixels.is_none();
    if identity {
        return src.to_vec();
    }
    let zoom = fx.zoom.clamp(0.25, 4.0);
    let mut out = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        let ny = ((y as f32 / h.max(1) as f32) - 0.5 - fx.pan_y * 0.45) / zoom + 0.5;
        for x in 0..w {
            let mut ux = (x as f32 / w.max(1) as f32) - 0.5 - fx.pan_x * 0.45;
            if fx.mirror {
                ux = -ux;
            }
            let nx = ux / zoom + 0.5;
            let mut px = if (0.0..1.0).contains(&nx) && (0.0..1.0).contains(&ny) {
                let sx = ((nx * w as f32) as u32).min(w.saturating_sub(1));
                let sy = ((ny * h as f32) as u32).min(h.saturating_sub(1));
                let mut sample = src
                    .get((sy * w + sx) as usize)
                    .copied()
                    .unwrap_or([0, 0, 0, 255]);
                if fx.sharpen {
                    let neighbours = [
                        (sx.saturating_sub(1), sy),
                        ((sx + 1).min(w.saturating_sub(1)), sy),
                        (sx, sy.saturating_sub(1)),
                        (sx, (sy + 1).min(h.saturating_sub(1))),
                    ];
                    for (channel, value) in sample.iter_mut().take(3).enumerate() {
                        let sum: u32 = neighbours
                            .iter()
                            .filter_map(|(x, y)| src.get((y * w + x) as usize))
                            .map(|p| u32::from(p.get(channel).copied().unwrap_or(0)))
                            .sum();
                        *value = (f32::from(*value) * 2.0 - sum as f32 / 4.0)
                            .clamp(0.0, 255.0)
                            .round() as u8;
                    }
                }
                sample
            } else {
                [10, 14, 18, 255]
            };

            // Colour Key (green/teal dominance key to dark slate backdrop)
            if fx.chroma_key {
                let [pr, pg, pb, _] = px;
                let r = pr as f32 / 255.0;
                let g = pg as f32 / 255.0;
                let b = pb as f32 / 255.0;
                if g > (r.max(b) + (0.45 - fx.chroma_tolerance.clamp(0.05, 0.42))) {
                    px = [18, 24, 34, 255];
                }
            }

            // Colour Adjust (Lift, Gain, Saturation)
            if fx.lift.abs() >= 0.005
                || (fx.gain - 1.0).abs() >= 0.005
                || (fx.saturation - 1.0).abs() >= 0.005
            {
                let [pr, pg, pb, pa] = px;
                let mut r = (pr as f32 / 255.0 + fx.lift) * fx.gain;
                let mut g = (pg as f32 / 255.0 + fx.lift) * fx.gain;
                let mut b = (pb as f32 / 255.0 + fx.lift) * fx.gain;
                let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                r = luma + (r - luma) * fx.saturation;
                g = luma + (g - luma) * fx.saturation;
                b = luma + (b - luma) * fx.saturation;
                px = [
                    (r.clamp(0.0, 1.0) * 255.0) as u8,
                    (g.clamp(0.0, 1.0) * 255.0) as u8,
                    (b.clamp(0.0, 1.0) * 255.0) as u8,
                    pa,
                ];
            }

            // Optional Picture-in-Picture layer in top-right quadrant
            if let Some(pip) = pip_pixels {
                let u = x as f32 / w.max(1) as f32;
                let v = y as f32 / h.max(1) as f32;
                if (0.62..=0.95).contains(&u) && (0.08..=0.41).contains(&v) {
                    let pu = (u - 0.62) / 0.33;
                    let pv = (v - 0.08) / 0.33;
                    if !(0.015..=0.985).contains(&pu) || !(0.025..=0.975).contains(&pv) {
                        px = [38, 175, 255, 255];
                    } else {
                        let px_x = ((pu * w as f32) as u32).min(w.saturating_sub(1));
                        let px_y = ((pv * h as f32) as u32).min(h.saturating_sub(1));
                        if let Some(p_col) = pip.get((px_y * w + px_x) as usize) {
                            px = *p_col;
                        }
                    }
                }
            }
            out.push(px);
        }
    }
    out
}

impl RustMixApp {
    pub(crate) fn refresh_textures(&mut self, ctx: &egui::Context) {
        // Avoid flooding the software WARP command queue with 8 texture uploads per frame
        // while static dialog screenshots are being read back in CI smoke mode.
        if self.smoke.as_ref().is_some_and(|s| s.is_dialog_stage())
            && !self.textures.is_empty()
            && self.program_texture.is_some()
        {
            return;
        }
        use std::hash::{Hash, Hasher};
        let mut stamp = std::collections::hash_map::DefaultHasher::new();
        let project = self.engine.project();
        project.revision.hash(&mut stamp);
        for input in &project.inputs {
            let ms = self
                .engine
                .playback(input.id)
                .map_or(0, |clock| clock.position_ms());
            project.preferences.rate.frame_at(ms).hash(&mut stamp);
        }
        let black = self.engine.black_level();
        let black_state = if black <= 0.0 {
            0_u8
        } else if black >= 1.0 {
            1
        } else {
            2
        };
        black_state.hash(&mut stamp);
        let automatic = self.engine.transition().is_some_and(|t| !t.manual) || black_state == 2;
        automatic.hash(&mut stamp);
        if automatic {
            project.preferences.rate.frame_at(self.now).hash(&mut stamp);
        }
        let frame = stamp.finish();
        if self.last_demo_frame == Some(frame) {
            return;
        }
        let start = Instant::now();
        let p = self.engine.project().clone();
        self.textures.retain(|id, _| p.input(*id).is_some());
        let mut raw_sources: HashMap<InputId, Vec<[u8; 4]>> = HashMap::new();
        for input in &p.inputs {
            let input_ms = self
                .engine
                .playback(input.id)
                .map_or(0, |clock| clock.position_ms());
            if let Ok(pixels) = render::source(input, 384, 216, input_ms) {
                raw_sources.insert(input.id, pixels);
            }
        }

        for input in &p.inputs {
            let fx = self.input_fx.get(&input.id).cloned().unwrap_or_default();
            if let Some(base) = raw_sources.get(&input.id) {
                let pip_buf = fx.pip_source.and_then(|pid| raw_sources.get(&pid));
                let processed = if input.online {
                    process_source_pixels(base, 384, 216, &fx, pip_buf.map(Vec::as_slice))
                } else {
                    base.clone()
                };
                let image = egui::ColorImage::from_rgba_unmultiplied(
                    [384, 216],
                    &processed.into_iter().flatten().collect::<Vec<_>>(),
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
        }
        let mut program_sources = HashMap::new();
        let from = self.engine.transition().map_or(p.program, |t| t.from);
        let to = self.engine.transition().map(|t| t.to);
        let mut ids = vec![from];
        if let Some(to) = to {
            ids.push(to);
        }
        // PIP dependencies use their own transport, too, and never recurse.
        for id in ids.clone() {
            if let Some(pip) = self.input_fx.get(&id).and_then(|fx| fx.pip_source) {
                ids.push(pip);
            }
        }
        for id in ids {
            if let Some(input) = p.input(id) {
                let ms = self
                    .engine
                    .playback(id)
                    .map_or(0, |clock| clock.position_ms());
                if let Ok(pixels) = render::source(input, 640, 360, ms) {
                    program_sources.insert(id, pixels);
                }
            }
        }
        let mut processed_sources = HashMap::new();
        for id in [Some(from), to].into_iter().flatten() {
            if let Some(base) = program_sources.get(&id) {
                let fx = self.input_fx.get(&id).cloned().unwrap_or_default();
                let pip = fx.pip_source.and_then(|id| program_sources.get(&id));
                let pixels = if p.input(id).is_some_and(|i| i.online) {
                    process_source_pixels(base, 640, 360, &fx, pip.map(Vec::as_slice))
                } else {
                    base.clone()
                };
                processed_sources.insert(id, pixels);
            }
        }
        let composed = processed_sources
            .get(&from)
            .ok_or_else(|| Error::Invalid("Program source missing".into()))
            .and_then(|from_pixels| {
                render::program_from_sources(
                    &p,
                    self.engine.transition(),
                    self.engine.black_level(),
                    640,
                    360,
                    from_pixels,
                    to.and_then(|id| processed_sources.get(&id))
                        .map(Vec::as_slice),
                )
            });
        match composed {
            Ok(pixels) => {
                let flat: Vec<u8> = pixels.into_iter().flatten().collect();
                self.last_program_rgba = flat.clone();
                let image = egui::ColorImage::from_rgba_unmultiplied([640, 360], &flat);
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
        let Some(input) = p.input(id).cloned() else {
            ui.colored_label(RED, "Source missing");
            return;
        };
        let color = if program { PROGRAM } else { PREVIEW };
        egui::Frame::new()
            .fill(color)
            .corner_radius(egui::CornerRadius {
                nw: 4,
                ne: 4,
                sw: 0,
                se: 0,
            })
            .inner_margin(egui::Margin::symmetric(8, 3))
            .show(ui, |ui| {
                ui.set_min_width((ui.available_width() - 1.0).max(20.0));
                ui.horizontal(|ui| {
                    let badge = if program { "PGM" } else { "PVW" };
                    ui.add_sized(
                        [(ui.available_width() - 96.0).max(40.0), 18.0],
                        egui::Label::new(
                            RichText::new(format!("{badge}  {} · {}", id.0, input.name))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .truncate(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        let gear = vector_icon_button(
                            ui,
                            VectorIcon::Gear,
                            egui::vec2(20.0, 18.0),
                            false,
                            Some(Color32::from_black_alpha(45)),
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
                            // Position / PIP box icon on Preview bar
                            let (pos_rect, pos_resp) =
                                ui.allocate_exact_size(egui::vec2(18.0, 16.0), Sense::click());
                            ui.painter()
                                .rect_filled(pos_rect, 2.0, Color32::from_black_alpha(45));
                            ui.painter().rect_stroke(
                                pos_rect.shrink(2.0),
                                1.0,
                                Stroke::new(1.2, Color32::WHITE),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().rect_stroke(
                                pos_rect.shrink(4.5),
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
                            // RGB color bars icon on Preview bar
                            let (rgb_rect, rgb_resp) =
                                ui.allocate_exact_size(egui::vec2(18.0, 16.0), Sense::click());
                            ui.painter().rect_filled(rgb_rect, 2.0, Color32::WHITE);
                            let third = (rgb_rect.width() - 2.0) / 3.0;
                            for (idx, col) in [
                                Color32::from_rgb(230, 45, 45),
                                Color32::from_rgb(40, 195, 65),
                                Color32::from_rgb(45, 110, 235),
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
        let transport_h = 28.0;
        let canvas_avail_h = (available.y - transport_h - 4.0).max(40.0);
        let width = available.x.max(16.0).min(canvas_avail_h * 16.0 / 9.0);
        let (letterbox, _) = ui.allocate_exact_size(
            Vec2::new(available.x.max(width), canvas_avail_h),
            Sense::hover(),
        );
        ui.painter()
            .rect_filled(letterbox, 0.0, Color32::from_rgb(8, 10, 14));
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
                let clock = self
                    .engine
                    .playback(id)
                    .map_or(0, |clock| clock.position_ms());
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

        // Modern live transport & timecode bar under Preview/Program monitor
        let transport = egui::Frame::new()
            .fill(Color32::from_rgb(22, 28, 36))
            .corner_radius(egui::CornerRadius {
                nw: 0,
                ne: 0,
                sw: 4,
                se: 4,
            })
            .inner_margin(egui::Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ui.set_min_width((available.x - 12.0).max(40.0));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let is_paused = self.engine.inputs_paused()
                        || self.engine.playback(id).is_some_and(|clock| clock.paused());
                    let play_btn = vector_icon_button(
                        ui,
                        if is_paused {
                            VectorIcon::Play
                        } else {
                            VectorIcon::Pause
                        },
                        egui::vec2(20.0, 18.0),
                        !is_paused,
                        None,
                    )
                    .on_hover_text("Play / Pause source playback");
                    self.automation.insert(
                        format!(
                            "monitor.{}.play",
                            if program { "program" } else { "preview" }
                        ),
                        play_btn.rect,
                    );
                    if play_btn.clicked() {
                        let paused = self.engine.playback(id).is_some_and(|clock| clock.paused());
                        self.actions
                            .push(Action::Engine(Command::SetInputPaused(id, !paused)));
                    }
                    let restart_btn = vector_icon_button(
                        ui,
                        VectorIcon::Restart,
                        egui::vec2(20.0, 18.0),
                        false,
                        None,
                    )
                    .on_hover_text("Restart playback from 00:00:00");
                    self.automation.insert(
                        format!(
                            "monitor.{}.restart",
                            if program { "program" } else { "preview" }
                        ),
                        restart_btn.rect,
                    );
                    if restart_btn.clicked() {
                        self.actions.push(Action::Engine(Command::RestartInput(id)));
                    }
                    let position = self
                        .engine
                        .playback(id)
                        .map_or(0, |clock| clock.position_ms());
                    let minutes = position / 60_000;
                    let sec = (position / 1000) % 60;
                    let frm = p
                        .preferences
                        .rate
                        .frame_at(position)
                        .saturating_sub(p.preferences.rate.frame_at(position / 1000 * 1000));
                    let prog_frac =
                        (position as f32 / playback::DEMO_DURATION_MS as f32).clamp(0.0, 1.0);
                    let bar_w = (ui.available_width() - 100.0).max(12.0);
                    let (bar_rect, _) =
                        ui.allocate_exact_size(egui::vec2(bar_w, 6.0), Sense::hover());
                    ui.painter()
                        .rect_filled(bar_rect, 3.0, Color32::from_rgb(12, 16, 22));
                    let fill_rect = Rect::from_min_size(
                        bar_rect.min,
                        egui::vec2(bar_rect.width() * prog_frac, bar_rect.height()),
                    );
                    ui.painter().rect_filled(fill_rect, 3.0, color);
                    self.automation.insert(
                        format!(
                            "monitor.{}.progress",
                            if program { "program" } else { "preview" }
                        ),
                        fill_rect,
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("00:{minutes:02}:{sec:02}:{frm:02}"))
                                .size(10.5)
                                .color(DIM),
                        );
                    });
                });
            });
        self.automation.insert(
            format!(
                "monitor.{}.transport",
                if program { "program" } else { "preview" }
            ),
            transport.response.rect,
        );
    }

    pub(crate) fn switcher(&mut self, ui: &mut egui::Ui) {
        // Allocate the bank once. Child minimum sizes must never feed back into
        // persisted PanelState and grow the bank on every repaint.
        let (bounds, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        self.automation.insert("monitor.bank.bounds".into(), bounds);
        let mut bounded = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("switcher.bounded")
                .max_rect(bounds)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        bounded.set_clip_rect(bounds.intersect(ui.clip_rect()));
        let ui = &mut bounded;
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
        let center = 110.0;
        let side = ((available.x - center - 64.0) / 2.0).max(200.0);
        ui.horizontal_top(|ui| {
            ui.add_space(10.0);
            ui.allocate_ui_with_layout(
                egui::vec2(side, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.monitor(ui, false),
            );
            ui.add_space(14.0);
            ui.allocate_ui_with_layout(
                egui::vec2(center, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(center);
                    ui.spacing_mut().item_spacing.y = 3.0;
                    self.command_button(ui, "switcher.quickPlay", "Quick Play", Command::Cut, None);
                    self.command_button(ui, "switcher.cut", "Cut", Command::Cut, None);

                    // 4 Configurable Transition Slots: Fade, Merge, Wipe, CubeZoom
                    for slot_idx in 0..4 {
                        let slot_name = self
                            .transition_slots
                            .get(slot_idx)
                            .copied()
                            .unwrap_or("Fade");
                        let kind = if matches!(slot_name, "Wipe" | "Slide" | "CubeZoom") {
                            TransitionKind::Wipe
                        } else {
                            TransitionKind::Fade
                        };
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            let enabled = self.engine.enabled(&Command::Transition(kind));
                            let implemented = matches!(slot_name, "Fade" | "Wipe");
                            let btn = ui
                                .add_enabled(
                                    enabled.is_ok() && implemented,
                                    egui::Button::new(RichText::new(slot_name).strong().size(11.5))
                                        .min_size(egui::vec2(center - 22.0, 25.0)),
                                )
                                .on_disabled_hover_text(
                                    "Choose Fade or Wipe; this transition has no implementation",
                                );
                            if slot_idx == 0 {
                                self.automation.insert("switcher.fade".into(), btn.rect);
                            } else if slot_idx == 2 {
                                self.automation.insert("switcher.wipe".into(), btn.rect);
                            }
                            if btn.clicked() {
                                self.actions.push(Action::Engine(Command::Transition(kind)));
                            }
                            caret_menu_button(ui, 25.0, |ui| {
                                for candidate in [
                                    "Fade",
                                    "Merge",
                                    "Wipe",
                                    "CubeZoom",
                                    "Zoom",
                                    "Slide",
                                    "Fly",
                                    "CrossZoom",
                                ] {
                                    if ui
                                        .add_enabled(
                                            matches!(candidate, "Fade" | "Wipe"),
                                            egui::Button::selectable(
                                                slot_name == candidate,
                                                candidate,
                                            ),
                                        )
                                        .clicked()
                                    {
                                        if let Some(slot_ref) =
                                            self.transition_slots.get_mut(slot_idx)
                                        {
                                            *slot_ref = candidate;
                                        }
                                        ui.close();
                                    }
                                }
                                ui.separator();
                                for ms in [250, 500, 1000, 1500, 2000] {
                                    if ui.button(format!("{slot_name} · {ms} ms")).clicked() {
                                        self.actions.push(Action::Engine(Command::SetDuration(ms)));
                                        ui.close();
                                    }
                                }
                            });
                        });
                    }

                    let black = self.engine.black_level() > 0.01;
                    self.command_button(
                        ui,
                        "switcher.ftb",
                        "FTB",
                        Command::FadeBlack,
                        black.then_some(RED),
                    );

                    // 4x2 Overlay number buttons (1..4, 5..8)
                    let cell_w = ((center - 6.0) / 4.0).floor();
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        ui.spacing_mut().button_padding = egui::vec2(2.0, 1.0);
                        for slot in 0..4 {
                            let active = self
                                .engine
                                .project()
                                .overlays
                                .get(slot)
                                .is_some_and(|o| o.enabled);
                            let btn = ui.add(
                                egui::Button::new(
                                    RichText::new(format!("{}", slot + 1)).size(10.5).strong(),
                                )
                                .selected(active)
                                .min_size(egui::vec2(cell_w, 20.0)),
                            );
                            if btn.clicked()
                                && let Some(mut o) =
                                    self.engine.project().overlays.get(slot).cloned()
                            {
                                o.enabled = !o.enabled;
                                self.actions
                                    .push(Action::Engine(Command::SetOverlay(slot, o)));
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        ui.spacing_mut().button_padding = egui::vec2(2.0, 1.0);
                        for slot in 5..=8 {
                            let _ = ui.add(
                                egui::Button::new(
                                    RichText::new(format!("{slot}")).size(10.5).color(DIM),
                                )
                                .min_size(egui::vec2(cell_w, 20.0)),
                            );
                        }
                    });

                    // Horizontal manual transition: left is Preview 0%, right 100%.
                    ui.add_space(2.0);
                    let mut progress = self
                        .engine
                        .transition()
                        .map_or(if self.tbar_latched { 1.0 } else { 0.0 }, |t| t.progress);
                    let can_tbar = self.engine.project().program != self.engine.project().preview
                        && self.engine.transition().is_none_or(|t| t.manual);
                    let (tbar_rect, tbar) = ui.allocate_exact_size(
                        egui::vec2(center, 36.0),
                        if can_tbar {
                            Sense::click_and_drag()
                        } else {
                            Sense::hover()
                        },
                    );
                    let cyl_rect = Rect::from_min_size(
                        egui::pos2(tbar_rect.left() + 6.0, tbar_rect.center().y - 4.0),
                        egui::vec2(tbar_rect.width() - 12.0, 8.0),
                    );
                    ui.painter()
                        .rect_filled(cyl_rect, 4.0, Color32::from_rgb(76, 86, 98));
                    ui.painter().rect_stroke(
                        cyl_rect,
                        4.0,
                        Stroke::new(1.0, Color32::from_rgb(38, 44, 52)),
                        egui::StrokeKind::Inside,
                    );
                    let handle_x =
                        egui::lerp(cyl_rect.left()..=cyl_rect.right(), progress.clamp(0.0, 1.0));
                    for step in 0..=4 {
                        let x = egui::lerp(cyl_rect.x_range(), step as f32 / 4.0);
                        ui.painter().vline(
                            x,
                            (cyl_rect.bottom() + 3.0)..=(cyl_rect.bottom() + 6.0),
                            Stroke::new(1.0, DIM),
                        );
                    }
                    let knob = Rect::from_center_size(
                        egui::pos2(handle_x, cyl_rect.center().y),
                        egui::vec2(10.0, 24.0),
                    );
                    ui.painter()
                        .rect_filled(knob, 3.0, Color32::from_rgb(20, 130, 240));
                    self.automation.insert("switcher.tbar".into(), tbar_rect);
                    let tbar =
                        tbar.on_hover_text("Manual T-Bar · drag left to right to blend PVW → PGM");
                    if can_tbar
                        && (tbar.is_pointer_button_down_on() || tbar.dragged() || tbar.clicked())
                    {
                        if let Some(pos) = tbar.interact_pointer_pos() {
                            progress = ((pos.x - cyl_rect.left()) / cyl_rect.width().max(1.0))
                                .clamp(0.0, 1.0);
                        }
                        if progress < 1.0 {
                            self.tbar_latched = false;
                        }
                        if !(progress >= 1.0 && self.tbar_latched) {
                            if progress >= 1.0 {
                                self.tbar_latched = true;
                            }
                            self.actions
                                .push(Action::Engine(Command::SetManualProgress(progress)));
                        }
                    } else if self.tbar_latched && !tbar.is_pointer_button_down_on() {
                        self.tbar_latched = false;
                    }
                },
            );
            ui.add_space(14.0);
            ui.allocate_ui_with_layout(
                egui::vec2(side, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.monitor(ui, true),
            );
        });
    }

    pub(crate) fn inputs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let all_selected = self.category.is_none();
            let all_btn = ui.add(
                egui::Button::new("")
                    .fill(if all_selected {
                        Color32::from_rgb(76, 88, 102)
                    } else {
                        Color32::from_rgb(42, 50, 60)
                    })
                    .stroke(Stroke::new(
                        1.0,
                        if all_selected {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(26, 32, 40)
                        },
                    ))
                    .min_size(egui::vec2(20.0, 20.0)),
            );
            self.automation
                .insert("inputs.category.all".into(), all_btn.rect);
            if all_btn.on_hover_text("Show all categories").clicked() {
                self.category = None;
            }
            for kind in InputKind::ALL {
                let selected = self.category == Some(kind);
                let fill = kind_color(kind);
                let chip = ui.add(
                    egui::Button::new("")
                        .fill(fill)
                        .stroke(Stroke::new(
                            if selected { 1.5 } else { 1.0 },
                            if selected {
                                Color32::WHITE
                            } else {
                                Color32::from_rgb(20, 24, 28)
                            },
                        ))
                        .min_size(egui::vec2(20.0, 20.0)),
                );
                self.automation
                    .insert(format!("inputs.category.{kind:?}"), chip.rect);
                if chip.on_hover_text(kind.label()).clicked() {
                    self.category = if selected { None } else { Some(kind) };
                }
            }
            let search_btn = vector_icon_button(
                ui,
                VectorIcon::Search,
                egui::vec2(22.0, 20.0),
                self.search_open,
                None,
            )
            .on_hover_text("Search inputs");
            if search_btn.clicked() {
                self.search_open = !self.search_open;
            }
            if self.search_open || !self.input_filter.is_empty() {
                let filter = ui.add_sized(
                    [150.0, 20.0],
                    egui::TextEdit::singleline(&mut self.input_filter)
                        .char_limit(48)
                        .hint_text("Search"),
                );
                self.automation.insert("inputs.filter".into(), filter.rect);
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
        });
        ui.add_space(2.0);
        self.source_tiles(ui, false);
    }

    pub(crate) fn source_tiles(&mut self, ui: &mut egui::Ui, multiview_grid: bool) {
        let p = self.engine.project().clone();
        let needle = self.input_filter.trim().to_lowercase();
        let filtered: Vec<_> = p
            .inputs
            .iter()
            .filter(|i| {
                self.category.is_none_or(|k| i.kind == k)
                    && (needle.is_empty()
                        || i.name.to_lowercase().contains(&needle)
                        || i.kind.label().to_lowercase().contains(&needle)
                        || i.id.0.to_string() == needle)
            })
            .cloned()
            .collect();
        egui::ScrollArea::vertical()
            .id_salt(if multiview_grid {
                "multiview.tiles"
            } else {
                "inputs.tiles"
            })
            .show(ui, |ui| {
                let width = ui.available_width().max(260.0);
                let columns = if multiview_grid {
                    ((width / 230.0).floor() as usize).clamp(2, 6)
                } else {
                    ((width / 225.0).floor() as usize).clamp(1, 6)
                };
                let tile_w =
                    ((width - (columns.saturating_sub(1) as f32) * 6.0) / columns as f32).max(175.0);
                let inner_w = (tile_w - 4.0).max(165.0);
                for row in filtered.chunks(columns) {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        for input in row {
                            let is_pgm = p.program == input.id;
                            let is_pvw = p.preview == input.id;
                            let border = if is_pgm {
                                PROGRAM
                            } else if is_pvw {
                                PREVIEW
                            } else {
                                Color32::from_rgb(56, 66, 78)
                            };
                            let header_fill = if is_pgm {
                                PROGRAM
                            } else if is_pvw {
                                PREVIEW
                            } else {
                                IDLE_TILE
                            };
                            ui.push_id(input.id.0, |ui| {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(tile_w, 10.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        egui::Frame::new()
                                            .fill(Color32::from_rgb(20, 25, 32))
                                            .corner_radius(4.0)
                                            .stroke(Stroke::new(1.5, border))
                                            .inner_margin(1.0)
                                            .show(ui, |ui| {
                                                ui.set_width(inner_w);
                                                ui.spacing_mut().item_spacing =
                                                    egui::vec2(0.0, 1.0);
                                                // Tile header bar
                                                egui::Frame::new()
                                                    .fill(header_fill)
                                                    .corner_radius(egui::CornerRadius {
                                                        nw: 3,
                                                        ne: 3,
                                                        sw: 0,
                                                        se: 0,
                                                    })
                                                    .inner_margin(egui::Margin::symmetric(6, 2))
                                                    .show(ui, |ui| {
                                                        ui.set_width((inner_w - 12.0).max(40.0));
                                                        ui.horizontal(|ui| {
                                                            ui.add_sized(
                                                                [
                                                                    (inner_w - 38.0).max(30.0),
                                                                    16.0,
                                                                ],
                                                                egui::Label::new(
                                                                    RichText::new(format!(
                                                                        "{}   {}",
                                                                        input.id.0, input.name
                                                                    ))
                                                                    .size(11.0)
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
                                                                    let remove_ok = self
                                                                        .engine
                                                                        .enabled(
                                                                            &Command::RemoveInput(
                                                                                input.id,
                                                                            ),
                                                                        )
                                                                        .is_ok();
                                                                    let remove = ui.add_enabled(
                                                                        remove_ok,
                                                                        egui::Button::new("")
                                                                            .fill(
                                                                                Color32::TRANSPARENT,
                                                                            )
                                                                            .stroke(Stroke::NONE)
                                                                            .min_size(egui::vec2(
                                                                                18.0, 16.0,
                                                                            )),
                                                                    );
                                                                    paint_vector_icon(
                                                                        ui.painter(),
                                                                        remove.rect,
                                                                        VectorIcon::Close,
                                                                        Color32::WHITE,
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
                                                // Modern 2-row control strip:
                                                // Row 1: [1][2][3][4][GO][Cut][Loop]
                                                // Row 2: [5][6][7][8][Audio][Display][Gear]
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
                                                                RichText::new("GO")
                                                                    .size(10.5)
                                                                    .strong(),
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
                                                            RichText::new("Cut")
                                                                .size(10.5)
                                                                .strong(),
                                                        )
                                                        .min_size(egui::vec2(cell_w, 20.0)),
                                                    );
                                                    self.automation.insert(
                                                        format!("input.{}.cut", input.id.0),
                                                        cut.rect,
                                                    );
                                                    if cut
                                                        .on_hover_text("Direct Cut to Program")
                                                        .clicked()
                                                    {
                                                        self.actions.push(Action::Engine(take));
                                                    }
                                                    let looped = self.engine.playback(input.id).is_none_or(|clock| clock.looping());
                                                    let loop_btn = vector_icon_button(
                                                        ui,
                                                        VectorIcon::Loop,
                                                        egui::vec2(rem_w, 20.0),
                                                        looped,
                                                        None,
                                                    )
                                                    .on_hover_text("Toggle Loop playback");
                                                    self.automation.insert(format!("input.{}.loop", input.id.0), loop_btn.rect);
                                                    if loop_btn.clicked() {
                                                        self.actions.push(Action::Engine(Command::SetInputLoop(input.id, !looped)));
                                                    }
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.spacing_mut().item_spacing.x = 0.0;
                                                    ui.spacing_mut().button_padding =
                                                        egui::vec2(2.0, 1.0);
                                                    for slot in 5..=8 {
                                                        let _ = ui.add(
                                                            egui::Button::new(
                                                                RichText::new(format!("{slot}"))
                                                                    .size(10.5)
                                                                    .color(DIM),
                                                            )
                                                            .min_size(egui::vec2(cell_w, 20.0)),
                                                        );
                                                    }
                                                    let audio_w = cell_w * 1.5;
                                                    let icon_w =
                                                        ((inner_w - cell_w * 4.0 - audio_w) * 0.5)
                                                            .floor()
                                                            .max(18.0);
                                                    let gear_w = (inner_w
                                                        - cell_w * 4.0
                                                        - audio_w
                                                        - icon_w)
                                                        .max(18.0);
                                                    let mut audio_btn = egui::Button::new(
                                                        RichText::new("Audio").size(10.5).color(
                                                            if !input.audio.muted {
                                                                Color32::WHITE
                                                            } else {
                                                                DIM
                                                            },
                                                        ),
                                                    )
                                                    .min_size(egui::vec2(audio_w, 20.0));
                                                    if !input.audio.muted {
                                                        audio_btn = audio_btn.fill(PROGRAM);
                                                    }
                                                    let audio_resp = ui
                                                        .add(audio_btn)
                                                        .on_hover_text("Toggle Input Audio");
                                                    self.automation.insert(
                                                        format!("input.{}.audio", input.id.0),
                                                        audio_resp.rect,
                                                    );
                                                    if audio_resp.clicked() {
                                                        let mut a = input.audio.clone();
                                                        a.muted = !a.muted;
                                                        self.actions.push(Action::Engine(
                                                            Command::SetInputAudio(input.id, a),
                                                        ));
                                                    }
                                                    let disp_resp = vector_icon_button(
                                                        ui,
                                                        VectorIcon::Display,
                                                        egui::vec2(icon_w, 20.0),
                                                        false,
                                                        None,
                                                    )
                                                    .on_hover_text("Cue on Preview monitor");
                                                    if disp_resp.clicked() {
                                                        self.actions.push(Action::Engine(
                                                            Command::SelectPreview(input.id),
                                                        ));
                                                    }
                                                    let edit = vector_icon_button(
                                                        ui,
                                                        VectorIcon::Gear,
                                                        egui::vec2(gear_w, 20.0),
                                                        false,
                                                        None,
                                                    )
                                                    .on_hover_text("Open Input Settings");
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
                    // Pin button stacked directly above the OUTPUTS rail
                    ui.allocate_ui_with_layout(
                        egui::vec2(20.0, height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            let _ = vector_icon_button(
                                ui,
                                VectorIcon::Pin,
                                egui::vec2(20.0, 20.0),
                                true,
                                None,
                            )
                            .on_hover_text("Pin Audio Mixer");
                            mixer_rail(ui, (height - 22.0).max(80.0), "OUTPUTS");
                        },
                    );
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
                .fill(Color32::from_rgb(20, 25, 32))
                .corner_radius(4.0)
                .stroke(Stroke::new(1.0, Color32::from_rgb(52, 62, 74)))
                .inner_margin(2.0)
                .show(ui, |ui| {
                    ui.set_width(76.0);
                    ui.set_min_height(height - 4.0);
                    egui::Frame::new()
                        .fill(header_fill)
                        .corner_radius(egui::CornerRadius {
                            nw: 3,
                            ne: 3,
                            sw: 0,
                            se: 0,
                        })
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
                        // Left button column inside strip: [Gear], [i / S], [Speaker]
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 4.0;
                            ui.add_space(8.0);
                            let gear = vector_icon_button(
                                ui,
                                VectorIcon::Gear,
                                egui::vec2(22.0, 22.0),
                                false,
                                None,
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
                            ui.add_space(12.0);
                            if is_master {
                                let info = ui
                                    .add(
                                        egui::Button::new(RichText::new("i").strong().size(11.0))
                                            .min_size(egui::vec2(22.0, 20.0)),
                                    )
                                    .on_hover_text("Master Audio Outputs");
                                if info.clicked() {
                                    self.actions
                                        .push(Action::SettingsAt(SettingsPage::AudioOutputs));
                                }
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
                            let spk = vector_icon_button(
                                ui,
                                VectorIcon::Speaker(!a.muted),
                                egui::vec2(22.0, 22.0),
                                !a.muted,
                                Some(if a.muted { CHIP_BG } else { PROGRAM }),
                            )
                            .on_hover_text("Mute / Unmute channel");
                            if spk.clicked() {
                                a.muted = !a.muted;
                            }
                        });
                        // Right column inside strip: vertical fader with scale ticks + stereo level meter
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
                                Stroke::new(1.0, Color32::from_gray(190)),
                            );
                            ui.painter().hline(
                                (fader.rect.right() - 4.0)..=(fader.rect.right() - 1.0),
                                y,
                                Stroke::new(1.0, Color32::from_gray(190)),
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
                            .rect_filled(meter_rect, 2.0, Color32::from_rgb(10, 14, 18));
                        for (ch, peak) in peaks.into_iter().enumerate() {
                            let frac = ((audio::db(peak) + 60.0) / 60.0).clamp(0.0, 1.0);
                            let x = meter_rect.left() + 1.5 + ch as f32 * 6.0;
                            let fill = Rect::from_min_max(
                                egui::pos2(x, meter_rect.bottom() - meter_rect.height() * frac),
                                egui::pos2(x + 4.5, meter_rect.bottom()),
                            );
                            ui.painter().rect_filled(
                                fill,
                                1.0,
                                if peak >= 1.0 {
                                    RED
                                } else {
                                    Color32::from_rgb(35, 195, 65)
                                },
                            );
                        }
                    });
                    // Bottom row inside strip: vector headphone + rotary dial on Master, or M/A/B bus buttons on Inputs
                    ui.horizontal(|ui| {
                        if is_master {
                            ui.add_space(8.0);
                            let (hp_rect, _) =
                                ui.allocate_exact_size(egui::vec2(18.0, 18.0), Sense::hover());
                            paint_vector_icon(
                                ui.painter(),
                                hp_rect,
                                VectorIcon::Headphones,
                                Color32::WHITE,
                            );
                            let (dial, dial_resp) = ui.allocate_exact_size(
                                egui::vec2(18.0, 18.0),
                                Sense::click_and_drag(),
                            );
                            if dial_resp.dragged() {
                                self.monitor_level = (self.monitor_level
                                    - dial_resp.drag_delta().y * 0.01)
                                    .clamp(0.0, 1.0);
                            }
                            ui.painter().circle_filled(
                                dial.center(),
                                8.0,
                                Color32::from_rgb(52, 62, 74),
                            );
                            ui.painter().circle_stroke(
                                dial.center(),
                                8.0,
                                Stroke::new(1.0, Color32::from_rgb(30, 36, 44)),
                            );
                            let ang = -std::f32::consts::FRAC_PI_4 * 3.0
                                + self.monitor_level * std::f32::consts::FRAC_PI_2 * 3.0;
                            ui.painter().line_segment(
                                [
                                    dial.center(),
                                    dial.center() + egui::vec2(ang.cos() * 6.0, ang.sin() * 6.0),
                                ],
                                Stroke::new(1.5, TEAL),
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
                                    let settings = vector_icon_button(
                                        ui,
                                        VectorIcon::Gear,
                                        egui::vec2(24.0, 20.0),
                                        false,
                                        None,
                                    )
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
                    ui.label("Independent Rust native offline switcher.\nCPU reference composition with wgpu display. Synthetic sources and PNG images; audio meters are simulated.");
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
                            self.input_fx.clear();
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

#[cfg(test)]
mod processing_tests {
    use super::*;
    #[test]
    fn custom_images_reach_program_size_and_sharpen_changes_edges() {
        let base = vec![[0, 0, 0, 255]; 16];
        let image = InputFx {
            custom_rgba: Some((2, 2, vec![[200, 40, 80, 255]; 4])),
            ..Default::default()
        };
        assert_eq!(
            process_source_pixels(&base, 4, 4, &image, None),
            vec![[200, 40, 80, 255]; 16]
        );
        let edge = vec![[40, 40, 40, 255], [100, 100, 100, 255], [40, 40, 40, 255]];
        let fx = InputFx {
            sharpen: true,
            ..Default::default()
        };
        assert_ne!(process_source_pixels(&edge, 3, 1, &fx, None), edge);
    }
}
