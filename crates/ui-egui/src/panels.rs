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
pub(crate) fn colour_image(size: [usize; 2], pixels: &[[u8; 4]]) -> egui::ColorImage {
    egui::ColorImage::new(
        size,
        pixels
            .iter()
            .map(|[r, g, b, a]| Color32::from_rgba_unmultiplied(*r, *g, *b, *a))
            .collect(),
    )
}

pub(crate) fn identity_processing(fx: &InputConfig, has_pip: bool) -> bool {
    (fx.zoom - 1.0).abs() < 0.005
        && fx.pan_x.abs() < 0.005
        && fx.pan_y.abs() < 0.005
        && !fx.mirror
        && !fx.sharpen
        && fx.lift.abs() < 0.005
        && (fx.gain - 1.0).abs() < 0.005
        && (fx.saturation - 1.0).abs() < 0.005
        && !fx.chroma_key
        && !has_pip
}

pub(crate) fn process_source_pixels(
    base: &[[u8; 4]],
    w: u32,
    h: u32,
    fx: &InputFx,
    pip_pixels: Option<&[[u8; 4]]>,
) -> Vec<[u8; 4]> {
    if fx.media_path.is_some() && fx.custom_rgba.is_none() {
        return vec![[0, 0, 0, 255]; (w * h) as usize];
    }
    let resized;
    let src = if let Some((cw, ch, custom)) = &fx.custom_rgba
        && *cw > 0
        && *ch > 0
        && cw
            .checked_mul(*ch)
            .is_some_and(|size| custom.len() == size as usize)
    {
        let scale = match fx.image_fit {
            ImageFit::Contain => (w as f32 / *cw as f32).min(h as f32 / *ch as f32),
            ImageFit::Cover => (w as f32 / *cw as f32).max(h as f32 / *ch as f32),
            ImageFit::Stretch => 1.0,
        };
        resized = (0..h)
            .flat_map(|y| {
                (0..w).map(move |x| {
                    let (sx, sy) = if fx.image_fit == ImageFit::Stretch {
                        (
                            (x as f32 + 0.5) * *cw as f32 / w.max(1) as f32,
                            (y as f32 + 0.5) * *ch as f32 / h.max(1) as f32,
                        )
                    } else {
                        (
                            ((x as f32 + 0.5) - w as f32 * 0.5) / scale + *cw as f32 * 0.5,
                            ((y as f32 + 0.5) - h as f32 * 0.5) / scale + *ch as f32 * 0.5,
                        )
                    };
                    if sx < 0.0 || sy < 0.0 || sx >= *cw as f32 || sy >= *ch as f32 {
                        return [0, 0, 0, 255];
                    }
                    let pixel = custom
                        .get((sy as u32 * *cw + sx as u32) as usize)
                        .copied()
                        .unwrap_or([0, 0, 0, 255]);
                    let alpha = u32::from(pixel.get(3).copied().unwrap_or(255));
                    let mut opaque = [0, 0, 0, 255];
                    for (out, value) in opaque.iter_mut().zip(pixel).take(3) {
                        *out = ((u32::from(value) * alpha + 127) / 255) as u8;
                    }
                    opaque
                })
            })
            .collect::<Vec<_>>();
        resized.as_slice()
    } else {
        base
    };
    let identity = identity_processing(fx, pip_pixels.is_some());
    if identity {
        return src.to_vec();
    }
    let zoom = fx.zoom.clamp(0.25, 4.0);
    let mut out = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        let ny = (((y as f32 + 0.5) / h.max(1) as f32) - 0.5 - fx.pan_y * 0.45) / zoom + 0.5;
        for x in 0..w {
            let mut ux = ((x as f32 + 0.5) / w.max(1) as f32) - 0.5 - fx.pan_x * 0.45;
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

pub(crate) fn processed_sources(
    project: &ProjectState,
    raw: &HashMap<InputId, Vec<[u8; 4]>>,
    fx: &HashMap<InputId, InputFx>,
    w: u32,
    h: u32,
) -> HashMap<InputId, Vec<[u8; 4]>> {
    fn visit(
        id: InputId,
        project: &ProjectState,
        raw: &HashMap<InputId, Vec<[u8; 4]>>,
        fx: &HashMap<InputId, InputFx>,
        size: [u32; 2],
        out: &mut HashMap<InputId, Vec<[u8; 4]>>,
        visiting: &mut Vec<InputId>,
    ) {
        if out.contains_key(&id) || visiting.contains(&id) || visiting.len() >= 32 {
            return;
        }
        let (Some(input), Some(base)) = (project.input(id), raw.get(&id)) else {
            return;
        };
        visiting.push(id);
        let effect = fx.get(&id).cloned().unwrap_or_else(|| InputFx {
            config: input.config.clone(),
            ..Default::default()
        });
        if let Some(pip) = effect.pip_source {
            visit(pip, project, raw, fx, size, out, visiting);
        }
        let [w, h] = size;
        let pixels = if input.online {
            process_source_pixels(
                base,
                w,
                h,
                &effect,
                effect
                    .pip_source
                    .and_then(|pip| out.get(&pip))
                    .map(Vec::as_slice),
            )
        } else {
            base.clone()
        };
        out.insert(id, pixels);
        let _ = visiting.pop();
    }
    let mut out = HashMap::new();
    for id in raw.keys() {
        visit(*id, project, raw, fx, [w, h], &mut out, &mut Vec::new());
    }
    out
}

impl RustMixApp {
    pub(crate) fn poll_program(&mut self, ctx: &egui::Context) {
        if let Some(worker) = &self.program_worker
            && let Some(frame) = worker.take()
        {
            match frame {
                Ok(frame) if frame.generation == self.program_generation => {
                    // The producer owns automatic progression between explicit
                    // control updates. Never send a stale UI copy back each paint.
                    self.engine = frame.engine;
                    self.video_stats = frame.stats;
                    self.video_timing = Some((frame.timing, frame.rate));
                    self.video_produced_ns = frame.produced_ns;
                    self.generation_ms = frame.stats.render_ms as f32;
                    self.upload_program(ctx, frame.pixels);
                }
                Ok(_) => {} // A frame from before the latest command cannot undo it.
                Err(error) => {
                    self.notice = format!("Program retained last valid frame: {error}");
                    self.warning = true;
                }
            }
        }
    }

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
            let frame = if self.program_worker.is_some() {
                ms.saturating_mul(15) / 1000
            } else {
                project.preferences.rate.frame_at(ms)
            };
            frame.hash(&mut stamp);
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
        if automatic && self.program_worker.is_none() {
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

        let processed_preview = processed_sources(&p, &raw_sources, &self.input_fx, 384, 216);
        for input in &p.inputs {
            if let Some(processed) = processed_preview.get(&input.id) {
                let image = colour_image([384, 216], processed);
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
        if self.program_worker.is_none() {
            match self.program_renderer.render(&self.engine, &self.input_fx) {
                Ok(pixels) => self.upload_program(ctx, pixels),
                Err(error) => {
                    self.notice = format!("Program retained last valid frame: {error}");
                    self.warning = true;
                }
            }
            self.generation_ms = start.elapsed().as_secs_f32() * 1000.0;
        }
        self.last_demo_frame = Some(frame);
    }

    fn upload_program(&mut self, ctx: &egui::Context, pixels: Arc<Vec<[u8; 4]>>) {
        if self
            .program_pixels
            .as_ref()
            .is_some_and(|old| Arc::ptr_eq(old, &pixels))
        {
            return;
        }
        self.last_program_rgba.clear();
        self.last_program_rgba
            .extend(pixels.iter().flatten().copied());
        let image = colour_image([640, 360], &pixels);
        if let Some(texture) = &mut self.program_texture {
            texture.set(image, egui::TextureOptions::LINEAR);
        } else {
            self.program_texture =
                Some(ctx.load_texture("program.composite", image, egui::TextureOptions::LINEAR));
        }
        self.program_pixels = Some(pixels);
        if self.program_output.is_open() {
            ctx.request_repaint_of(program_output::viewport_id());
        }
    }

    fn monitor(&mut self, ui: &mut egui::Ui, program: bool) {
        let p = self.engine.project();
        let id = if program { p.program } else { p.preview };
        let Some(input) = p.input(id).cloned() else {
            ui.colored_label(RED, "Source missing");
            return;
        };
        let color = if program {
            self.program_color()
        } else {
            self.preview_color()
        };
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
                            RichText::new(format!(
                                "{badge}  {} · {}",
                                p.input_number(id),
                                input.name
                            ))
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
            for (i, o) in p.overlays.standard.iter().enumerate() {
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
                    let play_btn = ui
                        .push_id(("source.play", program, id.0), |ui| {
                            vector_icon_button(
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
                        })
                        .inner
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
                    let restart_btn = ui
                        .push_id(("source.restart", program, id.0), |ui| {
                            vector_icon_button(
                                ui,
                                VectorIcon::Restart,
                                egui::vec2(20.0, 18.0),
                                false,
                                None,
                            )
                        })
                        .inner
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
        if self.multiview && !self.basic_mode {
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
                    if self.basic_mode {
                        return;
                    }
                    self.command_button(ui, "switcher.cut", "Cut", Command::Cut, None);

                    for (index, mut slot) in self
                        .engine
                        .project()
                        .transition_slots
                        .into_iter()
                        .enumerate()
                    {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            let command = Command::RunTransitionSlot(index);
                            let enabled = self.engine.enabled(&command);
                            let button = ui
                                .add_enabled(
                                    enabled.is_ok(),
                                    egui::Button::new(
                                        RichText::new(slot.kind.label()).strong().size(10.5),
                                    )
                                    .min_size(egui::vec2(center - 22.0, 25.0)),
                                )
                                .on_hover_text(format!(
                                    "Slot {} · {} ms",
                                    index + 1,
                                    slot.duration_ms
                                ))
                                .on_disabled_hover_text(
                                    enabled.err().map(|e| e.to_string()).unwrap_or_default(),
                                );
                            self.automation
                                .insert(format!("switcher.slot.{}", index + 1), button.rect);
                            if index == 0 {
                                self.automation.insert("switcher.fade".into(), button.rect);
                            }
                            if index == 2 {
                                self.automation.insert("switcher.wipe".into(), button.rect);
                            }
                            if button.clicked() {
                                self.actions.push(Action::Engine(command));
                            }
                            let original = slot;
                            let menu = caret_menu_button(ui, 25.0, |ui| {
                                for candidate in TransitionKind::ALL {
                                    let choice = ui.selectable_label(
                                        slot.kind == candidate,
                                        candidate.label(),
                                    );
                                    self.automation.insert(
                                        format!("switcher.slot.{}.effect.{candidate:?}", index + 1),
                                        choice.rect,
                                    );
                                    if choice.clicked() {
                                        slot.kind = candidate;
                                        ui.close();
                                    }
                                }
                                ui.separator();
                                ui.label("Duration (ms)");
                                ui.add(
                                    egui::DragValue::new(&mut slot.duration_ms)
                                        .range(100..=10_000)
                                        .speed(50),
                                );
                            });
                            self.automation.insert(
                                format!("switcher.slot.{}.settings", index + 1),
                                menu.response.rect,
                            );
                            if slot != original {
                                self.actions
                                    .push(Action::Engine(Command::SetTransitionSlot(index, slot)));
                            }
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

                    // 4x2 Overlay number buttons (1..4, 5..8); all eight are live.
                    let cell_w = ((center - 6.0) / 4.0).floor();
                    for row in 0..2 {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            ui.spacing_mut().button_padding = egui::vec2(2.0, 1.0);
                            for slot in (row * 4)..(row * 4 + 4) {
                                let active = self
                                    .engine
                                    .project()
                                    .overlays
                                    .standard(slot)
                                    .is_some_and(|o| o.enabled);
                                let btn = ui.add(
                                    egui::Button::new(
                                        RichText::new(format!("{}", slot + 1)).size(10.5).strong(),
                                    )
                                    .selected(active)
                                    .min_size(egui::vec2(cell_w, 20.0)),
                                );
                                self.automation
                                    .insert(format!("switcher.overlay.{}", slot + 1), btn.rect);
                                if btn.clicked()
                                    && let Some(mut o) =
                                        self.engine.project().overlays.standard(slot).cloned()
                                {
                                    o.enabled = !o.enabled;
                                    self.actions
                                        .push(Action::Engine(Command::SetOverlay(slot, o)));
                                }
                            }
                        });
                    }

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
        self.category_filter(ui);
        ui.add_space(2.0);
        self.source_tiles(ui, false);
    }

    pub(crate) fn source_tiles(&mut self, ui: &mut egui::Ui, multiview_grid: bool) {
        let p = self.engine.project().clone();
        // The category predicate lives in the engine (Task 1.2): `All` shows
        // everything, `Search` is filter-only, concrete values match exactly.
        let category = InputCategory::from_stored(self.category).unwrap_or(InputCategory::All);
        let visible: Vec<InputId> = self
            .engine
            .project()
            .visible_inputs(category)
            .into_iter()
            .map(|input| input.id)
            .collect();
        let needle = self.input_filter.trim().to_lowercase();
        let filtered: Vec<_> = p
            .inputs
            .iter()
            .filter(|i| {
                visible.contains(&i.id)
                    && (needle.is_empty()
                        || i.name.to_lowercase().contains(&needle)
                        || i.kind.label().to_lowercase().contains(&needle)
                        || self.engine.project().input_number(i.id).to_string() == needle)
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
                                self.program_color()
                            } else if is_pvw {
                                self.preview_color()
                            } else {
                                Color32::from_rgb(56, 66, 78)
                            };
                            let header_fill = if is_pgm {
                                self.program_color()
                            } else if is_pvw {
                                self.preview_color()
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
                                                                        self.engine.project().input_number(input.id), input.name
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
                                                if let Some(category)=input.category.and_then(|id|p.preferences.categories.iter().find(|c|c.id==id)) {
                                                    let [r,g,b]=category.color;
                                                    ui.colored_label(Color32::from_rgb(r,g,b),&category.name);
                                                }
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
                                                if let Some(fx) = self.input_fx.get(&input.id)
                                                    && fx.media_path.is_some() && fx.custom_rgba.is_none() {
                                                    ui.painter().text(response.rect.center(), egui::Align2::CENTER_CENTER,
                                                        if fx.image_error.is_some() { "PNG UNAVAILABLE" } else { "LOADING PNG…" },
                                                        egui::FontId::proportional(11.0), self.preview_color());
                                                    self.automation.insert(format!("input.{}.media-status", input.id.0), response.rect);
                                                }
                                                if response.clicked() {
                                                    self.actions.push(Action::Engine(
                                                        Command::SelectPreview(input.id),
                                                    ));
                                                }
                                                response.context_menu(|ui| {
                                                    ui.menu_button("Category", |ui| {
                                                        let clear=ui.button("Uncategorized");
                                                        if clear.clicked(){ self.actions.push(Action::Engine(Command::SetInputCategory(input.id,None))); ui.close(); }
                                                        for category in &p.preferences.categories {
                                                            let row=ui.selectable_label(input.category==Some(category.id),&category.name);
                                                            self.automation.insert(format!("input.{}.category.{}",input.id.0,category.id),row.rect);
                                                            if row.clicked(){ self.actions.push(Action::Engine(Command::SetInputCategory(input.id,Some(category.id)))); ui.close(); }
                                                        }
                                                    });
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
                                                    self.command_button(ui, &format!("input.{}.duplicate", input.id.0), "Duplicate Input", Command::DuplicateInput(input.id), None);
                                                    if ui.button("Close Input…").clicked() {
                                                        self.actions
                                                            .push(Action::RequestRemove(input.id));
                                                        ui.close();
                                                    }
                                                });
                                                // Modern 2-row control strip:
                                                // Row 1: [1][2][3][4][GO][Cut][Loop]
                                                // Row 2: [5][6][7][8][Audio][Display][Gear]
                                                let columns = if self.basic_mode { 6.0 } else { 7.0 };
                                                let cell_w = (inner_w / columns).floor().max(20.0);
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
                                                            .standard(slot)
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
                                                    if self.basic_mode {
                                                        return;
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
                                                if self.basic_mode {
                                                    return;
                                                }
                                                ui.horizontal(|ui| {
                                                    ui.spacing_mut().item_spacing.x = 0.0;
                                                    ui.spacing_mut().button_padding =
                                                        egui::vec2(2.0, 1.0);
                                                    for slot in 4..8 {
                                                        if let Some(mut overlay) = self
                                                            .engine
                                                            .project()
                                                            .overlays
                                                            .standard(slot)
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
                                                        audio_btn = audio_btn.fill(self.program_color());
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
                            .on_hover_text("Docked mixer; pinning is unavailable");
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
                            let dial_resp = dial_resp.on_hover_text(
                                "Visual demonstration only; headphone audio output is unavailable",
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
        let overlays = self.engine.project().overlays.standard.clone();
        egui::ScrollArea::vertical()
            .id_salt("graphics.scroll")
            .show(ui, |ui| {
                ui.label(RichText::new("OVERLAY CHANNELS (1–8)").small().color(DIM));
                ui.label(
                    RichText::new(
                        "Stinger channels 9–16 are configured on the engine; clip compositing is not implemented yet.",
                    )
                    .small()
                    .color(DIM),
                );
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
        self.search_dialog(ctx);
        self.add_input_dialog(ctx);
        self.input_properties_dialog(ctx);
        self.settings_dialog(ctx);
        self.overlay_settings_dialog(ctx);
        self.external_settings_dialog(ctx);
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
                        let create = ui.add_enabled(
                            self.job.is_none(),
                            egui::Button::new("Create new preset"),
                        );
                        self.automation
                            .insert("dialog.new.confirm".into(), create.rect);
                        if create.clicked() {
                            match Engine::from_project_at(ProjectState::demo(), self.now) {
                                Ok(engine) => {
                                    self.engine = engine;
                                    self.reset_editors();
                                    self.media_loader.reset();
                                    self.input_fx.clear();
                                    self.sync_input_cache();
                                    self.submit_program();
                                    self.theme(ctx);
                                    self.dock = default_dock(false);
                                    self.audio_restore = None;
                                    self.path = None;
                                    self.saved_revision = 0;
                                    self.textures.clear();
                                    self.last_demo_frame = None;
                                    self.confirm_new = false;
                                }
                                Err(error) => self.say(error.to_string(), true),
                            }
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
    #![allow(clippy::unwrap_used)]
    use super::*;
    #[test]
    fn custom_images_reach_program_size_and_sharpen_changes_edges() {
        let base = vec![[0, 0, 0, 255]; 16];
        let image = InputFx {
            custom_rgba: Some((2, 2, vec![[200, 40, 80, 255]; 4].into())),
            ..Default::default()
        };
        assert_eq!(
            process_source_pixels(&base, 4, 4, &image, None),
            vec![[200, 40, 80, 255]; 16]
        );
        let edge = vec![[40, 40, 40, 255], [100, 100, 100, 255], [40, 40, 40, 255]];
        let fx = InputFx {
            config: InputConfig {
                sharpen: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_ne!(process_source_pixels(&edge, 3, 1, &fx, None), edge);
    }
    #[test]
    fn mirror_preserves_every_edge_pixel() {
        let base = vec![
            [10, 20, 30, 255],
            [40, 50, 60, 255],
            [70, 80, 90, 255],
            [110, 120, 130, 255],
            [140, 150, 160, 255],
            [170, 180, 190, 255],
        ];
        let fx = InputFx {
            config: InputConfig {
                mirror: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            process_source_pixels(&base, 3, 2, &fx, None),
            vec![
                [70, 80, 90, 255],
                [40, 50, 60, 255],
                [10, 20, 30, 255],
                [170, 180, 190, 255],
                [140, 150, 160, 255],
                [110, 120, 130, 255]
            ]
        );
    }
    #[test]
    fn image_fit_preserves_aspect_and_composites_alpha_over_black() {
        let mut fx = InputFx {
            custom_rgba: Some((2, 1, vec![[200, 0, 0, 128], [0, 100, 0, 255]].into())),
            ..Default::default()
        };
        let base = vec![[0, 0, 0, 255]; 16];
        let contain = process_source_pixels(&base, 4, 4, &fx, None);
        assert_eq!(contain.first(), Some(&[0, 0, 0, 255]));
        assert_eq!(contain.get(4), Some(&[100, 0, 0, 255]));
        fx.image_fit = ImageFit::Stretch;
        let stretch = process_source_pixels(&base, 4, 4, &fx, None);
        assert_eq!(stretch.first(), Some(&[100, 0, 0, 255]));
        fx.image_fit = ImageFit::Cover;
        assert_ne!(process_source_pixels(&base, 4, 4, &fx, None), contain);
    }
    #[test]
    fn pip_uses_the_imported_and_processed_dependency() {
        let mut project = ProjectState::demo();
        if let Some(input) = project.inputs.first_mut() {
            input.config.pip_source = Some(InputId(2));
        }
        if let Some(input) = project.inputs.get_mut(1) {
            input.config.media_path = Some("fixture.png".into());
            input.config.gain = 0.5;
        }
        project.validate().unwrap();
        let mut fx = HashMap::new();
        for input in &project.inputs {
            fx.insert(
                input.id,
                InputFx {
                    config: input.config.clone(),
                    ..Default::default()
                },
            );
        }
        if let Some(effect) = fx.get_mut(&InputId(2)) {
            effect.custom_rgba = Some((16, 9, vec![[100, 40, 20, 255]; 16 * 9].into()));
        }
        let raw = HashMap::from([
            (InputId(1), vec![[0, 0, 0, 255]; 160 * 90]),
            (InputId(2), vec![[0, 0, 0, 255]; 160 * 90]),
        ]);
        let output = processed_sources(&project, &raw, &fx, 160, 90);
        assert_eq!(
            output.get(&InputId(1)).and_then(|p| p.get(22 * 160 + 125)),
            Some(&[50, 20, 10, 255])
        );
    }
}
