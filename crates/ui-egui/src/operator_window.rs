//! Original operator caption around egui's bounded, resizable floating window.
use super::*;

const CHROME_KEY: &str = "operator.window.chrome.v1";

#[derive(Clone, Copy, Serialize, Deserialize)]
struct DialogWidth {
    viewport: f32,
    preferred: f32,
    rendered: f32,
}

impl DialogWidth {
    fn valid(self) -> bool {
        [self.viewport, self.preferred, self.rendered]
            .into_iter()
            .all(|value| value.is_finite() && (1.0..=65536.0).contains(&value))
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct WindowChrome {
    pub name: &'static str,
    pub window: Rect,
    pub header: Rect,
    pub title: Rect,
    pub close: Rect,
}

#[derive(Clone, Default)]
struct ChromeFrame {
    pass: u64,
    entries: [Option<WindowChrome>; 16],
}

pub(crate) fn current_chrome(ctx: &egui::Context) -> Vec<WindowChrome> {
    let pass = ctx.cumulative_pass_nr();
    ctx.data(|data| {
        data.get_temp::<ChromeFrame>(egui::Id::new(CHROME_KEY))
            .filter(|frame| frame.pass == pass)
            .map(|frame| frame.entries.into_iter().flatten().collect())
            .unwrap_or_default()
    })
}

fn store_chrome(ctx: &egui::Context, chrome: WindowChrome) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|data| {
        let frame = data.get_temp_mut_or_default::<ChromeFrame>(egui::Id::new(CHROME_KEY));
        if frame.pass != pass {
            frame.entries.fill(None);
            frame.pass = pass;
        }
        if let Some(entry) = frame.entries.iter_mut().find(|entry| {
            entry
                .as_ref()
                .is_none_or(|previous| previous.name == chrome.name)
        }) {
            *entry = Some(chrome);
        }
    });
}

#[derive(Clone, Copy)]
pub(crate) enum FooterRole {
    Confirm,
    Cancel,
}

pub(crate) fn footer_size(ui: &egui::Ui, role: FooterRole) -> Vec2 {
    let scale = (ui.ctx().viewport_rect().width() / 1912.0).clamp(0.6, 1.0);
    let width = match role {
        FooterRole::Confirm => (140.0 * scale).max(92.0),
        FooterRole::Cancel => (186.0 * scale).max(112.0),
    };
    egui::vec2(width, (42.0 * scale).max(29.0))
}

pub(super) struct OperatorWindow<'a> {
    title: egui::WidgetText,
    name: &'static str,
    open: &'a mut bool,
    size: Vec2,
    bounds: Rect,
    maximum: Vec2,
    minimum: Vec2,
    resizable: bool,
}

pub(super) fn operator_window<'a>(
    ctx: &egui::Context,
    title: impl Into<egui::WidgetText>,
    name: &'static str,
    open: &'a mut bool,
    preferred: Vec2,
) -> OperatorWindow<'a> {
    let bounds = ctx.viewport_rect().shrink(12.0);
    let maximum = (bounds.size() - egui::vec2(12.0, 28.0)).max(egui::vec2(280.0, 220.0));
    OperatorWindow {
        title: title.into(),
        name,
        open,
        size: preferred.min(maximum),
        bounds,
        maximum,
        minimum: egui::vec2(560.0, 300.0).min(maximum),
        resizable: true,
    }
}

impl OperatorWindow<'_> {
    pub(super) fn min_size(mut self, minimum: Vec2) -> Self {
        if minimum.is_finite() && minimum.x > 0.0 && minimum.y > 0.0 {
            self.minimum = minimum.min(self.maximum);
        }
        self
    }

    pub(super) fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub(super) fn show<R>(
        self,
        ctx: &egui::Context,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<egui::InnerResponse<Option<R>>> {
        let id = egui::Id::new(self.name);
        let drag_id = id.with("caption.drag");
        let width_id = id.with("operator.default.width.v1");
        let viewport_width = ctx.viewport_rect().width();
        let width_before = ctx
            .data_mut(|data| data.get_persisted::<DialogWidth>(width_id))
            .filter(|width| width.valid());
        let previous = ctx.memory(|memory| memory.area_rect(id));
        let dragging = ctx
            .read_response(drag_id)
            .is_some_and(|response| response.dragged());
        let delta = if dragging {
            ctx.input(|input| input.pointer.delta())
        } else {
            Vec2::ZERO
        };
        let adapt_default = !dragging
            && width_before.is_some_and(|width| {
                (width.viewport - viewport_width).abs() > 1.0
                    && (width.rendered - width.preferred).abs() <= 4.0
            });
        let mut position = previous
            .map(|rect| rect.min + delta)
            .filter(|position| position.is_finite())
            .unwrap_or(self.bounds.center() - self.size * 0.5);
        if adapt_default && let Some(width) = width_before {
            let scaled_x = position.x * viewport_width / width.viewport;
            if scaled_x.is_finite() {
                position.x = scaled_x;
            }
        }
        let frame = egui::Frame::window(&ctx.global_style()).inner_margin(0);
        let mut native_open = *self.open;
        let popup_was_open = egui::Popup::is_any_open(ctx);
        let mut close_clicked = false;
        let mut chrome = None;
        let window = egui::Window::new(self.title.clone())
            .id(id)
            .open(&mut native_open)
            .title_bar(false)
            .collapsible(false)
            .movable(false)
            .resizable(self.resizable)
            .frame(frame)
            .default_size(self.size)
            .current_pos(position)
            .min_size(self.minimum)
            .max_size(self.maximum)
            .constrain_to(self.bounds);
        // A retained default tracks the operator viewport. Limit width for
        // this paint only; subsequent paints keep normal manual resizing.
        let window = if adapt_default {
            window.min_width(self.size.x).max_width(self.size.x)
        } else {
            window
        };
        let output = window.show(ctx, |ui| {
            let viewport_width = ctx.viewport_rect().width();
            let height = (viewport_width * 32.0 / 1912.0).clamp(24.0, 32.0);
            let (header, _) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::hover());
            ui.painter().rect_filled(header, 0.0, FORM_LIGHT);
            let close = Rect::from_min_max(
                egui::pos2(header.right() - height * 1.45, header.top()),
                header.max,
            );
            let drag = Rect::from_min_max(header.min, egui::pos2(close.left(), header.bottom()));
            let response = ui.interact(drag, drag_id, Sense::drag());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Label,
                    ui.is_enabled(),
                    self.title.text(),
                )
            });
            let close_response = ui.interact(close, id.with("caption.close"), Sense::click());
            close_response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), "Close window")
            });
            let close_hovered = close_response.hovered();
            if close_hovered {
                ui.painter()
                    .rect_filled(close, 0.0, Color32::from_rgb(196, 43, 44));
            }
            paint_vector_icon(
                ui.painter(),
                close,
                VectorIcon::Close,
                if close_hovered {
                    Color32::WHITE
                } else {
                    FORM_TEXT
                },
            );
            close_clicked = close_response.on_hover_text("Close window (Esc)").clicked();
            let badge_size = (height - 10.0).clamp(14.0, 20.0);
            let badge = Rect::from_center_size(
                egui::pos2(header.left() + 6.0 + badge_size * 0.5, header.center().y),
                Vec2::splat(badge_size),
            );
            ui.painter()
                .rect_filled(badge, 1.0, Color32::from_rgb(24, 124, 83));
            ui.painter().text(
                badge.center(),
                egui::Align2::CENTER_CENTER,
                "R",
                egui::FontId::monospace((badge_size * 0.7).clamp(9.0, 14.0)),
                Color32::WHITE,
            );
            let title_rect = Rect::from_min_max(
                egui::pos2(badge.right() + 6.0, header.top()),
                egui::pos2(close.left() - 6.0, header.bottom()),
            );
            let mut title_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt(id.with("caption.title"))
                    .max_rect(title_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            title_ui.set_clip_rect(title_rect.intersect(ui.clip_rect()));
            let title = title_ui.add(
                egui::Label::new(
                    RichText::new(self.title.text())
                        .color(FORM_TEXT)
                        .size((viewport_width * 14.0 / 1912.0).clamp(12.0, 14.0)),
                )
                .truncate()
                .selectable(false),
            );
            chrome = Some(WindowChrome {
                name: self.name,
                window: Rect::NOTHING,
                header,
                title: title.rect,
                close,
            });
            egui::Frame::new()
                .inner_margin(6)
                .show(ui, add_contents)
                .inner
        });
        if native_open
            && !popup_was_open
            && !egui::Popup::is_any_open(ctx)
            && output
                .as_ref()
                .is_some_and(|window| ctx.top_layer_id() == Some(window.response.layer_id))
            && controls::consume_press(ctx, egui::Modifiers::NONE, egui::Key::Escape)
        {
            close_clicked = true;
        }
        *self.open = native_open && !close_clicked;
        if let (Some(output), Some(mut chrome)) = (&output, chrome) {
            chrome.window = output.response.rect;
            let width = DialogWidth {
                viewport: viewport_width,
                preferred: self.size.x,
                rendered: chrome.window.width(),
            };
            if width.valid() {
                ctx.data_mut(|data| data.insert_persisted(width_id, width));
            }
            store_chrome(ctx, chrome);
        }
        output
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    fn frame(
        ctx: &egui::Context,
        open: &mut bool,
        size: Vec2,
        events: Vec<egui::Event>,
    ) -> (Option<WindowChrome>, egui::FullOutput) {
        let mut chrome = None;
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |_| {
                operator_window(
                    ctx,
                    "Settings",
                    "test.operator",
                    open,
                    egui::vec2(560.0, 330.0),
                )
                .show(ctx, |ui| {
                    ui.label("Operator controls");
                });
                chrome = current_chrome(ctx).into_iter().next();
            },
        );
        (chrome, output)
    }

    fn button(pos: Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }

    #[test]
    fn caption_paints_small_left_title_and_real_close_at_reference_sizes() {
        for size in [
            egui::vec2(734.0, 466.0),
            egui::vec2(1600.0, 900.0),
            egui::vec2(1917.0, 1198.0),
        ] {
            let ctx = egui::Context::default();
            RustMixApp::headless().theme(&ctx);
            let mut open = true;
            for _ in 0..4 {
                frame(&ctx, &mut open, size, vec![]);
            }
            let (chrome, output) = frame(&ctx, &mut open, size, vec![]);
            let chrome = chrome.expect("visible window caption");
            let title = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.job.text == "Settings" => Some(text),
                    _ => None,
                })
                .expect("actual caption text");
            let ink = title.visual_bounding_rect();
            assert!(chrome.window.contains_rect(chrome.header));
            assert!(chrome.header.contains_rect(chrome.close));
            assert!((24.0..=32.1).contains(&chrome.header.height()));
            assert!((ink.left() - chrome.header.left() - 32.0).abs() <= 7.0);
            assert!(ink.right() < chrome.close.left());
            assert!(ink.height() <= 18.0);
            assert!((ink.center().y - chrome.header.center().y).abs() <= 3.0);
            let close = chrome.close.center();
            frame(
                &ctx,
                &mut open,
                size,
                vec![egui::Event::PointerMoved(close), button(close, true)],
            );
            frame(&ctx, &mut open, size, vec![button(close, false)]);
            assert!(!open, "caption close must really close at {size:?}");
        }
    }

    #[test]
    fn header_drag_moves_window_while_body_drag_leaves_it_stationary() {
        for size in [egui::vec2(734.0, 466.0), egui::vec2(1600.0, 900.0)] {
            let ctx = egui::Context::default();
            let mut open = true;
            for _ in 0..4 {
                frame(&ctx, &mut open, size, vec![]);
            }
            let before = frame(&ctx, &mut open, size, vec![]).0.unwrap();
            let start = before.header.center();
            let delta = egui::vec2(20.0, 12.0);
            frame(
                &ctx,
                &mut open,
                size,
                vec![egui::Event::PointerMoved(start), button(start, true)],
            );
            frame(
                &ctx,
                &mut open,
                size,
                vec![egui::Event::PointerMoved(start + delta)],
            );
            frame(&ctx, &mut open, size, vec![button(start + delta, false)]);
            let moved = frame(&ctx, &mut open, size, vec![]).0.unwrap();
            assert!(
                (moved.window.min - before.window.min - delta).length() <= 2.0,
                "header drag lost its movement: {before:?} -> {moved:?}"
            );
            assert!(Rect::from_min_size(Pos2::ZERO, size).contains_rect(moved.window));
            let body = egui::pos2(moved.header.center().x, moved.header.bottom() + 60.0);
            frame(
                &ctx,
                &mut open,
                size,
                vec![egui::Event::PointerMoved(body), button(body, true)],
            );
            frame(
                &ctx,
                &mut open,
                size,
                vec![egui::Event::PointerMoved(body + delta)],
            );
            frame(&ctx, &mut open, size, vec![button(body + delta, false)]);
            let after = frame(&ctx, &mut open, size, vec![]).0.unwrap();
            assert!(
                (after.window.min - moved.window.min).length() <= 1.0,
                "dragging a form body must not move the dialog"
            );
        }
    }
}
