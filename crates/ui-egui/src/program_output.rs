//! Program-only native monitor. This presents the last validated composite;
//! it does not create a second media scheduler, encoder or device backend.
use super::*;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    mpsc::{SyncSender, sync_channel},
};

pub(crate) fn viewport_id() -> egui::ViewportId {
    egui::ViewportId::from_hash_of("rustmix.program.output")
}

#[derive(Serialize, Deserialize)]
struct Preferences {
    width: u32,
    height: u32,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            width: 960,
            height: 540,
        }
    }
}
impl Preferences {
    fn valid(&self) -> bool {
        (320..=3840).contains(&self.width) && (180..=2160).contains(&self.height)
    }
}

struct Signals {
    open: AtomicBool,
    fullscreen: AtomicBool,
    native_seen: AtomicBool,
    native_fullscreen: AtomicBool,
    capture_requested: AtomicBool,
    /// Monotonic acceptance id carried through `ViewportCommand::Screenshot` so
    /// a stale readback can never satisfy a later request.
    capture_sequence: AtomicU64,
    width: AtomicU32,
    height: AtomicU32,
    target_monitor: AtomicUsize,
    visible: AtomicBool,
}

pub(crate) struct Capture {
    pub viewport: egui::ViewportId,
    pub sequence: u64,
    pub image: Arc<egui::ColorImage>,
}

/// Moves matching screenshot events into the bounded mailbox. A capacity-one
/// channel with `try_send` keeps a stalled acceptance consumer from blocking
/// either native window.
fn harvest_screenshots(
    events: &[egui::Event],
    sender: &SyncSender<Capture>,
    expected_sequence: u64,
) {
    for event in events {
        if let egui::Event::Screenshot {
            viewport_id: id,
            user_data,
            image,
        } = event
            && *id == viewport_id()
        {
            let sequence = user_data
                .data
                .as_ref()
                .and_then(|data| data.downcast_ref::<u64>())
                .copied();
            if sequence != Some(expected_sequence) {
                continue;
            }
            let _ = sender.try_send(Capture {
                viewport: *id,
                sequence: expected_sequence,
                image: Arc::clone(image),
            });
        }
    }
}

pub(crate) struct ProgramOutput {
    signals: Arc<Signals>,
    captures: Receiver<Capture>,
    capture_sender: SyncSender<Capture>,
}
impl Default for ProgramOutput {
    fn default() -> Self {
        let (capture_sender, captures) = sync_channel(1);
        let preferences = Preferences::default();
        Self {
            signals: Arc::new(Signals {
                open: AtomicBool::new(false),
                fullscreen: AtomicBool::new(false),
                native_seen: AtomicBool::new(false),
                native_fullscreen: AtomicBool::new(false),
                capture_requested: AtomicBool::new(false),
                capture_sequence: AtomicU64::new(0),
                width: AtomicU32::new(preferences.width),
                height: AtomicU32::new(preferences.height),
                target_monitor: AtomicUsize::new(usize::MAX),
                visible: AtomicBool::new(true),
            }),
            captures,
            capture_sender,
        }
    }
}
impl ProgramOutput {
    pub fn is_open(&self) -> bool {
        self.signals.open.load(Ordering::Acquire)
    }
    pub fn is_fullscreen(&self) -> bool {
        self.signals.fullscreen.load(Ordering::Acquire)
    }
    pub fn native_seen(&self) -> bool {
        self.signals.native_seen.load(Ordering::Acquire)
    }
    pub fn native_fullscreen(&self) -> bool {
        self.signals.native_fullscreen.load(Ordering::Acquire)
    }
    pub fn open(&self, ctx: &egui::Context) {
        // A deliberate fresh start is always windowed, so an unplugged monitor
        // or a previous fullscreen session cannot hide a new output window.
        self.signals.fullscreen.store(false, Ordering::Release);
        self.signals
            .target_monitor
            .store(usize::MAX, Ordering::Release);
        self.signals.visible.store(true, Ordering::Release);
        self.signals.native_seen.store(false, Ordering::Release);
        self.signals
            .capture_requested
            .store(false, Ordering::Release);
        while self.captures.try_recv().is_ok() {}
        self.signals.open.store(true, Ordering::Release);
        ctx.request_repaint();
    }
    pub fn close(&self, ctx: &egui::Context) {
        self.signals.open.store(false, Ordering::Release);
        self.signals
            .target_monitor
            .store(usize::MAX, Ordering::Release);
        ctx.send_viewport_cmd_to(viewport_id(), egui::ViewportCommand::Close);
        ctx.request_repaint();
    }
    pub fn toggle_fullscreen(&self, ctx: &egui::Context) {
        if self.is_open() && self.target_monitor().is_none() {
            let next = !self.signals.fullscreen.fetch_xor(true, Ordering::AcqRel);
            ctx.send_viewport_cmd_to(viewport_id(), egui::ViewportCommand::Fullscreen(next));
            ctx.request_repaint();
        }
    }
    pub fn target_monitor(&self) -> Option<usize> {
        let index = self.signals.target_monitor.load(Ordering::Acquire);
        (index != usize::MAX).then_some(index)
    }
    pub fn open_on_monitor(&self, ctx: &egui::Context, index: usize) {
        self.open(ctx);
        self.signals.target_monitor.store(index, Ordering::Release);
        self.signals.fullscreen.store(true, Ordering::Release);
        // A monitor index may become invalid after discovery. Verify the real
        // native rectangle before revealing, so winit cannot expose a fallback
        // window on the operator display during initial creation.
        self.signals.visible.store(false, Ordering::Release);
        // Reapply even if this viewport ID previously used the same monitor
        // index: an optional window may have been moved since that builder.
        ctx.send_viewport_cmd_to(viewport_id(), egui::ViewportCommand::Visible(false));
        ctx.send_viewport_cmd_to(viewport_id(), egui::ViewportCommand::SetMonitor(index));
    }
    pub fn retarget_monitor(&self, ctx: &egui::Context, index: usize) {
        self.signals.visible.store(false, Ordering::Release);
        self.signals.target_monitor.store(index, Ordering::Release);
        ctx.send_viewport_cmd_to(viewport_id(), egui::ViewportCommand::Visible(false));
        ctx.send_viewport_cmd_to(viewport_id(), egui::ViewportCommand::SetMonitor(index));
        ctx.request_repaint_of(viewport_id());
    }
    pub fn reveal_monitor(&self, ctx: &egui::Context) {
        if self.is_open()
            && self.target_monitor().is_some()
            && !self.signals.visible.swap(true, Ordering::AcqRel)
        {
            ctx.send_viewport_cmd_to(viewport_id(), egui::ViewportCommand::Visible(true));
            ctx.request_repaint_of(viewport_id());
        }
    }
    pub fn restore_preferences(&self, json: Option<&str>) {
        if let Some(preferences) =
            json.and_then(|text| serde_json::from_str::<Preferences>(text).ok())
            && preferences.valid()
        {
            self.signals
                .width
                .store(preferences.width, Ordering::Release);
            self.signals
                .height
                .store(preferences.height, Ordering::Release);
        }
    }
    pub fn preferences_json(&self) -> String {
        let preferences = Preferences {
            width: self.signals.width.load(Ordering::Acquire),
            height: self.signals.height.load(Ordering::Acquire),
        };
        serde_json::to_string(&preferences).unwrap_or_default()
    }
    /// Requests one accepted Program-viewport readback and returns its
    /// monotonic acceptance id. Any capture left by an earlier request is
    /// discarded, so a retry can never be mistaken for the picture a later
    /// acceptance stage is waiting for.
    pub fn request_capture(&self, ctx: &egui::Context) -> u64 {
        while self.captures.try_recv().is_ok() {}
        let sequence = self
            .signals
            .capture_sequence
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1);
        self.signals
            .capture_requested
            .store(true, Ordering::Release);
        ctx.request_repaint_of(viewport_id());
        sequence
    }
    pub fn take_capture(&self) -> Option<Capture> {
        let expected = self.signals.capture_sequence.load(Ordering::Acquire);
        match self.captures.try_recv() {
            Ok(capture) if capture.sequence == expected => Some(capture),
            _ => None,
        }
    }
    /// Forwards a deferred Program-viewport screenshot. `eframe` injects
    /// pending screenshots into whichever native window processes its next
    /// frame first, and the operator window repaints continuously during
    /// scripted acceptance, so a Program capture can arrive in the operator
    /// frame's input instead of the child callback's. Both frames call this.
    pub fn harvest_capture_events(&self, events: &[egui::Event]) {
        let expected = self.signals.capture_sequence.load(Ordering::Acquire);
        harvest_screenshots(events, &self.capture_sender, expected);
    }
    pub fn show(&self, ctx: &egui::Context, texture: Option<egui::TextureHandle>) {
        if !self.is_open() {
            return;
        }
        let signals = Arc::clone(&self.signals);
        let captures = self.capture_sender.clone();
        let size = Vec2::new(
            signals.width.load(Ordering::Acquire) as f32,
            signals.height.load(Ordering::Acquire) as f32,
        );
        let mut builder = egui::ViewportBuilder::default()
            .with_title("RustMix Program · F11 fullscreen · Escape closes output")
            .with_inner_size(size)
            .with_min_inner_size([320.0, 180.0])
            .with_fullscreen(self.is_fullscreen())
            .with_visible(self.signals.visible.load(Ordering::Acquire));
        if let Some(index) = self.target_monitor() {
            builder = builder
                .with_monitor(index)
                .with_active(false)
                .with_title("RustMix Program · Extended display · Escape closes output");
        }
        ctx.show_viewport_deferred(viewport_id(), builder, move |ui, class| {
            // Never turn an embedded fallback into an apparent second output.
            if class != egui::ViewportClass::Deferred {
                signals.open.store(false, Ordering::Release);
                signals.target_monitor.store(usize::MAX, Ordering::Release);
                ui.label("A separate native Program window is unavailable on this backend.");
                return;
            }
            signals.native_seen.store(true, Ordering::Release);
            let ctx = ui.ctx().clone();
            let native_fullscreen = ctx.input(|input| input.viewport().fullscreen == Some(true));
            signals
                .native_fullscreen
                .store(native_fullscreen, Ordering::Release);
            if ctx.input(|input| {
                input.viewport().close_requested() || input.key_pressed(egui::Key::Escape)
            }) {
                signals.open.store(false, Ordering::Release);
                signals.target_monitor.store(usize::MAX, Ordering::Release);
                ctx.request_repaint_of(egui::ViewportId::ROOT);
                return;
            }
            if signals.target_monitor.load(Ordering::Acquire) == usize::MAX
                && ctx.input(|input| input.key_pressed(egui::Key::F11))
            {
                let next = !signals.fullscreen.fetch_xor(true, Ordering::AcqRel);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(next));
                ctx.request_repaint_of(egui::ViewportId::ROOT);
            }
            if !native_fullscreen && !signals.fullscreen.load(Ordering::Acquire) {
                let size = ctx.viewport_rect().size();
                if size.is_finite() && size.x >= 320.0 && size.y >= 180.0 {
                    signals.width.store(
                        size.x.round().clamp(320.0, 3840.0) as u32,
                        Ordering::Release,
                    );
                    signals.height.store(
                        size.y.round().clamp(180.0, 2160.0) as u32,
                        Ordering::Release,
                    );
                }
            }
            let bounds = ctx.viewport_rect();
            ui.set_clip_rect(bounds);
            ui.painter().rect_filled(bounds, 0.0, Color32::BLACK);
            if let Some(texture) = &texture {
                ui.painter().image(
                    texture.id(),
                    fit_program_aspect(bounds, texture.aspect_ratio()),
                    Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                ui.painter().text(
                    bounds.center(),
                    egui::Align2::CENTER_CENTER,
                    "Waiting for a validated Program frame",
                    egui::FontId::proportional(16.0),
                    Color32::WHITE,
                );
            }
            // Screenshot readback is opt-in hosted acceptance only. The
            // capacity-one mailbox cannot stall either viewport.
            let expected = signals.capture_sequence.load(Ordering::Acquire);
            let events = ctx.input(|input| input.events.clone());
            harvest_screenshots(&events, &captures, expected);
            if signals.capture_requested.swap(false, Ordering::AcqRel) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                    expected,
                )));
            }
        });
    }
}

pub(crate) fn fit_program(bounds: Rect) -> Rect {
    fit_program_aspect(bounds, 16.0 / 9.0)
}
fn fit_program_aspect(bounds: Rect, aspect: f32) -> Rect {
    let size = bounds.size();
    let width = size.x.min(size.y * aspect).max(0.0);
    Rect::from_center_size(bounds.center(), Vec2::new(width, width / aspect))
}

impl RustMixApp {
    pub(crate) fn program_output_menu(&mut self, ui: &mut egui::Ui) {
        ui.label("Optional windowed Program monitor");
        ui.label("The top Fullscreen button sends Program directly to an extended display.");
        let open = ui.add_enabled(
            !self.program_output.is_open(),
            egui::Button::new("Open Program window"),
        );
        self.automation
            .insert("output.program.open".into(), open.rect);
        if open.clicked() {
            self.actions.push(Action::ProgramOutputOpen);
            ui.close();
        }
        let fullscreen = ui.add_enabled(
            self.program_output.is_open() && self.program_output.target_monitor().is_none(),
            egui::Button::new(if self.program_output.is_fullscreen() {
                "Windowed Program output"
            } else {
                "Fullscreen Program output"
            }),
        );
        self.automation
            .insert("output.program.fullscreen".into(), fullscreen.rect);
        if fullscreen.clicked() {
            self.actions.push(Action::ProgramOutputFullscreen);
            ui.close();
        }
        let close = ui.add_enabled(
            self.program_output.is_open(),
            egui::Button::new("Close Program window"),
        );
        self.automation
            .insert("output.program.close".into(), close.rect);
        if close.clicked() {
            self.actions.push(Action::ProgramOutputClose);
            ui.close();
        }
        ui.label("Program only · Esc closes output · no audio or device output");
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
    use super::*;

    fn parent(
        app: &mut RustMixApp,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
                events,
                ..Default::default()
            },
            |ui| app.frame_ui(ui),
        )
    }
    fn child(
        ctx: &egui::Context,
        callback: &Arc<egui::DeferredViewportUiCallback>,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let mut raw = egui::RawInput {
            viewport_id: viewport_id(),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(960.0, 540.0))),
            events,
            ..Default::default()
        };
        raw.viewports.insert(
            viewport_id(),
            egui::ViewportInfo {
                parent: Some(egui::ViewportId::ROOT),
                fullscreen: Some(false),
                ..Default::default()
            },
        );
        ctx.run_ui(raw, |ui| callback(ui))
    }
    fn key(key: egui::Key) -> Vec<egui::Event> {
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            })
            .collect()
    }
    fn click(app: &mut RustMixApp, ctx: &egui::Context, id: &str) {
        let mut previous: Option<Rect> = None;
        let mut stable = None;
        for _ in 0..6 {
            parent(app, ctx, vec![]);
            let bounds = *app
                .automation
                .get(id)
                .expect("Operator control must be visible");
            if previous.is_some_and(|old| {
                old.center().distance(bounds.center()) < 0.5
                    && (old.size() - bounds.size()).length() < 0.5
            }) {
                stable = Some(bounds);
                break;
            }
            previous = Some(bounds);
        }
        let bounds = stable.expect("Operator control should settle");
        assert!(ctx.viewport_rect().contains_rect(bounds));
        for pressed in [true, false] {
            parent(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(bounds.center()),
                    egui::Event::PointerButton {
                        pos: bounds.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }
    fn assert_program_mesh(
        output: &egui::FullOutput,
        program: egui::TextureId,
        preview: egui::TextureId,
    ) {
        let textures: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|clipped| {
                if let egui::epaint::Shape::Mesh(mesh) = &clipped.shape {
                    Some(mesh.texture_id)
                } else {
                    None
                }
            })
            .collect();
        assert!(
            textures.contains(&program),
            "Program texture must be painted into child"
        );
        assert!(
            !textures.contains(&preview),
            "Preview cannot enter child output"
        );
    }

    #[test]
    fn parent_frame_forwards_only_tagged_deferred_program_screenshots_through_bounded_mailbox() {
        let output = ProgramOutput::default();
        let ctx = egui::Context::default();
        let image = Arc::new(egui::ColorImage::filled(
            [4, 3],
            egui::Color32::from_rgb(10, 20, 30),
        ));
        let screenshot = |viewport_id, sequence: u64| egui::Event::Screenshot {
            viewport_id,
            user_data: egui::UserData::new(sequence),
            image: Arc::clone(&image),
        };
        let first = output.request_capture(&ctx);
        // Operator-window screenshots are never Program output frames.
        output.harvest_capture_events(&[screenshot(egui::ViewportId::ROOT, first)]);
        assert!(output.take_capture().is_none());
        // An untagged or obsolete readback cannot satisfy the current request.
        output.harvest_capture_events(&[egui::Event::Screenshot {
            viewport_id: viewport_id(),
            user_data: egui::UserData::default(),
            image: Arc::clone(&image),
        }]);
        assert!(output.take_capture().is_none());
        output.harvest_capture_events(&[screenshot(viewport_id(), first.wrapping_sub(1))]);
        assert!(output.take_capture().is_none());
        output.harvest_capture_events(&[screenshot(viewport_id(), first)]);
        let capture = output
            .take_capture()
            .expect("deferred Program screenshot must be forwarded");
        assert_eq!(capture.viewport, viewport_id());
        assert_eq!(capture.sequence, first);
        assert_eq!(capture.image.size, [4, 3]);
        assert_eq!(capture.image.pixels.first(), image.pixels.first());
        // A retry discards stale readbacks and accepts only its own sequence;
        // the capacity-one mailbox keeps a stalled consumer from blocking a
        // native window and drops duplicates instead of queueing them.
        let second = output.request_capture(&ctx);
        assert_ne!(second, first);
        output.harvest_capture_events(&[screenshot(viewport_id(), first)]);
        assert!(
            output.take_capture().is_none(),
            "a stale capture must never satisfy a newer request"
        );
        output.harvest_capture_events(&[
            screenshot(viewport_id(), second),
            screenshot(viewport_id(), second),
        ]);
        assert!(output.take_capture().is_some());
        assert!(output.take_capture().is_none());
    }

    #[test]
    fn native_program_menu_opens_fullscreens_and_closes_without_changing_project_or_main_window() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        ctx.set_embed_viewports(false);
        app.theme(&ctx);
        let baseline = app.engine.project().clone();
        click(&mut app, &ctx, "output.program.menu");
        click(&mut app, &ctx, "output.program.open");
        assert!(app.program_output.is_open());
        let output = parent(&mut app, &ctx, vec![]);
        let viewport = output.viewport_output.get(&viewport_id()).unwrap();
        assert!(viewport.class == egui::ViewportClass::Deferred);
        assert_eq!(viewport.parent, egui::ViewportId::ROOT);
        assert_eq!(viewport.builder.fullscreen, Some(false));
        assert_eq!(*app.engine.project(), baseline);
        click(&mut app, &ctx, "output.program.menu");
        click(&mut app, &ctx, "output.program.fullscreen");
        assert!(app.program_output.is_fullscreen());
        assert!(!app.fullscreen);
        assert_eq!(*app.engine.project(), baseline);
        click(&mut app, &ctx, "output.program.menu");
        click(&mut app, &ctx, "output.program.close");
        assert!(!app.program_output.is_open());
        assert!(!app.allow_close && !app.confirm_close && !app.external_active);
        assert_eq!(*app.engine.project(), baseline);
        assert!(
            !parent(&mut app, &ctx, vec![])
                .viewport_output
                .contains_key(&viewport_id())
        );
    }

    #[test]
    fn child_always_paints_validated_program_including_transition_overlay_black_and_escape_is_local()
     {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        ctx.set_embed_viewports(false);
        app.execute(Command::SetInputsPaused(true));
        app.execute(Command::TakeInput(InputId(6)));
        app.dispatch(Action::ProgramOutputOpen, &ctx);
        let mut parent_output = parent(&mut app, &ctx, vec![]);
        let baseline = app.last_program_rgba.clone();
        let program = app.program_texture.as_ref().unwrap().id();
        let preview = app
            .textures
            .get(&app.engine.project().preview)
            .unwrap()
            .id();
        let callback = parent_output
            .viewport_output
            .get(&viewport_id())
            .unwrap()
            .viewport_ui_cb
            .as_ref()
            .unwrap()
            .clone();
        assert_program_mesh(&child(&ctx, &callback, vec![]), program, preview);
        assert!(app.program_output.native_seen());

        app.execute(Command::SelectPreview(InputId(3)));
        parent_output = parent(&mut app, &ctx, vec![]);
        assert_eq!(
            app.last_program_rgba, baseline,
            "Preview selection cannot replace Program pixels"
        );
        app.execute(Command::SetManualProgress(0.5));
        parent(&mut app, &ctx, vec![]);
        assert_ne!(
            app.last_program_rgba, baseline,
            "Transition is in the shared Program composite"
        );
        app.execute(Command::SetManualProgress(1.0));
        let mut overlay = app.engine.project().overlays.standard[0].clone();
        overlay.enabled = true;
        app.execute(Command::SetOverlay(0, overlay));
        parent(&mut app, &ctx, vec![]);
        let composite = app.last_program_rgba.clone();
        app.engine.execute(Command::FadeBlack, 0).unwrap();
        app.engine.tick(1000);
        parent(&mut app, &ctx, vec![]);
        assert_ne!(app.last_program_rgba, composite);
        assert!(
            app.last_program_rgba
                .chunks_exact(4)
                .all(|pixel| pixel[..3] == [0, 0, 0])
        );
        let callback = parent_output
            .viewport_output
            .get(&viewport_id())
            .unwrap()
            .viewport_ui_cb
            .as_ref()
            .unwrap()
            .clone();
        assert_program_mesh(
            &child(&ctx, &callback, key(egui::Key::F11)),
            program,
            preview,
        );
        assert!(app.program_output.is_fullscreen());
        assert!(!app.fullscreen);
        let state = app.engine.project().clone();
        let closed = child(&ctx, &callback, key(egui::Key::Escape));
        assert!(!app.program_output.is_open());
        assert!(!app.allow_close && !app.confirm_close && !app.fullscreen);
        assert_eq!(*app.engine.project(), state);
        assert!(
            !closed
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .is_some_and(|output| output.commands.contains(&egui::ViewportCommand::Close))
        );
    }

    #[test]
    fn unsupported_embedded_backend_does_not_claim_separate_output_and_preferences_never_autostart()
    {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.dispatch(Action::ProgramOutputOpen, &ctx);
        assert!(!app.program_output.is_open());
        assert!(app.warning);
        let output = ProgramOutput::default();
        output.restore_preferences(Some(r#"{"width":1280,"height":720}"#));
        assert_eq!(output.preferences_json(), r#"{"width":1280,"height":720}"#);
        for invalid in [
            r#"{"width":0,"height":720}"#,
            r#"{"width":4294967295,"height":720}"#,
            "broken",
        ] {
            output.restore_preferences(Some(invalid));
            assert_eq!(output.preferences_json(), r#"{"width":1280,"height":720}"#);
        }
        assert!(!output.is_open() && !output.is_fullscreen());
        let bounds = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
        let fitted = fit_program(bounds);
        assert!(bounds.contains_rect(fitted));
        assert!((fitted.width() / fitted.height() - 16.0 / 9.0).abs() < 0.001);
        assert_eq!(fitted.center(), bounds.center());
    }
}
