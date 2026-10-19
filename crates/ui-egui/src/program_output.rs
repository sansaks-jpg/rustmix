//! Program-only native monitor. This presents the last validated composite;
//! it does not create a second media scheduler, encoder or device backend.
use super::*;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
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
    width: AtomicU32,
    height: AtomicU32,
}

pub(crate) struct Capture {
    pub viewport: egui::ViewportId,
    pub image: Arc<egui::ColorImage>,
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
                width: AtomicU32::new(preferences.width),
                height: AtomicU32::new(preferences.height),
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
        self.signals.native_seen.store(false, Ordering::Release);
        self.signals
            .capture_requested
            .store(false, Ordering::Release);
        let _ = self.captures.try_recv();
        self.signals.open.store(true, Ordering::Release);
        ctx.request_repaint();
    }
    pub fn close(&self, ctx: &egui::Context) {
        self.signals.open.store(false, Ordering::Release);
        ctx.send_viewport_cmd_to(viewport_id(), egui::ViewportCommand::Close);
        ctx.request_repaint();
    }
    pub fn toggle_fullscreen(&self, ctx: &egui::Context) {
        if self.is_open() {
            let next = !self.signals.fullscreen.fetch_xor(true, Ordering::AcqRel);
            ctx.send_viewport_cmd_to(viewport_id(), egui::ViewportCommand::Fullscreen(next));
            ctx.request_repaint();
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
    pub fn request_capture(&self, ctx: &egui::Context) {
        self.signals
            .capture_requested
            .store(true, Ordering::Release);
        ctx.request_repaint_of(viewport_id());
    }
    pub fn take_capture(&self) -> Option<Capture> {
        self.captures.try_recv().ok()
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
        ctx.show_viewport_deferred(
            viewport_id(),
            egui::ViewportBuilder::default()
                .with_title("RustMix Program · F11 fullscreen · Escape closes output")
                .with_inner_size(size)
                .with_min_inner_size([320.0, 180.0])
                .with_fullscreen(self.is_fullscreen()),
            move |ui, class| {
                // Never turn an embedded fallback into an apparent second output.
                if class != egui::ViewportClass::Deferred {
                    signals.open.store(false, Ordering::Release);
                    ui.label("A separate native Program window is unavailable on this backend.");
                    return;
                }
                signals.native_seen.store(true, Ordering::Release);
                let ctx = ui.ctx().clone();
                let native_fullscreen =
                    ctx.input(|input| input.viewport().fullscreen == Some(true));
                signals
                    .native_fullscreen
                    .store(native_fullscreen, Ordering::Release);
                if ctx.input(|input| {
                    input.viewport().close_requested() || input.key_pressed(egui::Key::Escape)
                }) {
                    signals.open.store(false, Ordering::Release);
                    ctx.request_repaint_of(egui::ViewportId::ROOT);
                    return;
                }
                if ctx.input(|input| input.key_pressed(egui::Key::F11)) {
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
                        fit_program(bounds),
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
                for event in ctx.input(|input| input.events.clone()) {
                    if let egui::Event::Screenshot {
                        viewport_id: id,
                        image,
                        ..
                    } = event
                        && id == viewport_id()
                    {
                        let _ = captures.try_send(Capture {
                            viewport: id,
                            image,
                        });
                        ctx.request_repaint_of(egui::ViewportId::ROOT);
                    }
                }
                if signals.capture_requested.swap(false, Ordering::AcqRel) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                        egui::UserData::default(),
                    ));
                }
            },
        );
    }
}

pub(crate) fn fit_program(bounds: Rect) -> Rect {
    let size = bounds.size();
    let width = size.x.min(size.y * (16.0 / 9.0)).max(0.0);
    Rect::from_center_size(bounds.center(), Vec2::new(width, width * (9.0 / 16.0)))
}

impl RustMixApp {
    pub(crate) fn program_output_menu(&mut self, ui: &mut egui::Ui) {
        ui.label("Separate Program monitor");
        ui.label("Open windowed, drag to the target monitor, then Fullscreen / F11.");
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
            self.program_output.is_open(),
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
