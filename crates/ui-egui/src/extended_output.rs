//! Direct, fail-closed Program placement on a discovered extended display.
//! Inventory and HWND queries run on one bounded worker at a time. Monitor
//! indices are never persisted; the stable Windows display identity is checked
//! again before every start and periodically while output is active.
use super::*;
use rustmix_platform::displays::{self, DisplayInfo};
use std::sync::mpsc::TryRecvError;

const WATCH_INTERVAL: Duration = Duration::from_secs(2);
const PLACEMENT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
struct Inventory {
    displays: Vec<DisplayInfo>,
    operator: usize,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Purpose {
    Refresh,
    Start,
    Watch,
}

pub(crate) struct ExtendedOutput {
    selected_id: Option<String>,
    invalid_selection: bool,
    inventory: Option<Inventory>,
    query: Option<Receiver<std::result::Result<Inventory, String>>>,
    query_purpose: Purpose,
    query_started: Option<Instant>,
    query_timed_out: bool,
    pending_start: bool,
    target: Option<DisplayInfo>,
    active: bool,
    revealed: bool,
    placement_started: Option<Instant>,
    placement_wait_frames: u8,
    last_check: Instant,
    error: Option<String>,
    #[cfg(test)]
    injected: Option<std::result::Result<Inventory, String>>,
}
impl Default for ExtendedOutput {
    fn default() -> Self {
        Self {
            selected_id: None,
            invalid_selection: false,
            inventory: None,
            query: None,
            query_purpose: Purpose::Refresh,
            query_started: None,
            query_timed_out: false,
            pending_start: false,
            target: None,
            active: false,
            revealed: false,
            placement_started: None,
            placement_wait_frames: 0,
            last_check: Instant::now(),
            error: None,
            #[cfg(test)]
            injected: None,
        }
    }
}
impl ExtendedOutput {
    pub(crate) fn is_active(&self) -> bool {
        self.active
    }
    pub(crate) fn is_pending(&self) -> bool {
        self.pending_start
            || (self.query.is_some() && self.query_purpose == Purpose::Start)
            || (self.target.is_some() && !self.active)
    }
    pub(crate) fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }
    pub(crate) fn restore_selected_id(&mut self, id: Option<&str>) {
        self.invalid_selection = id.is_some_and(|id| !valid_id(id));
        self.selected_id = id.filter(|id| valid_id(id)).map(str::to_owned);
        // A preference is not a routing authorization: every Start resolves it
        // against fresh native inventory and rejects an absent/unsafe identity.
    }
    fn clear_route(&mut self) {
        self.target = None;
        self.active = false;
        self.revealed = false;
        self.pending_start = false;
        self.placement_started = None;
        self.placement_wait_frames = 0;
        if self.query_purpose == Purpose::Start {
            self.query_purpose = Purpose::Refresh;
        }
    }
}

fn valid_id(id: &str) -> bool {
    !id.trim().is_empty() && id.len() <= 128 && !id.chars().any(char::is_control)
}
fn display_rect(display: &DisplayInfo) -> Rect {
    let [x, y] = display.position;
    let [width, height] = display.size;
    Rect::from_min_size(
        egui::pos2(x as f32, y as f32),
        Vec2::new(width as f32, height as f32),
    )
}
fn overlaps(left: Rect, right: Rect) -> bool {
    left.left() < right.right()
        && right.left() < left.right()
        && left.top() < right.bottom()
        && right.top() < left.bottom()
}
fn choose_target(
    inventory: &Inventory,
    selected: Option<&str>,
) -> std::result::Result<DisplayInfo, String> {
    if inventory.displays.len() > displays::MAX_DISPLAYS || inventory.displays.is_empty() {
        return Err("Display inventory is empty or exceeds the supported limit".into());
    }
    for (index, display) in inventory.displays.iter().enumerate() {
        if display.index != index
            || !valid_id(&display.id)
            || display.size.contains(&0)
            || inventory
                .displays
                .iter()
                .take(index)
                .any(|old| old.id == display.id)
        {
            return Err("Display inventory is inconsistent; refresh displays".into());
        }
    }
    let operator = inventory
        .displays
        .iter()
        .find(|display| display.index == inventory.operator)
        .ok_or_else(|| {
            "The operator display could not be identified; Program output stayed closed".to_owned()
        })?;
    let eligible = |display: &&DisplayInfo| {
        display.index != operator.index && !overlaps(display_rect(display), display_rect(operator))
    };
    let target = match selected {
        Some(id) => inventory.displays.iter().find(|display| display.id == id)
            .ok_or_else(|| "The selected extended display is disconnected; select a connected display".to_owned())?,
        None => inventory.displays.iter().find(eligible)
            .ok_or_else(|| "No extended display is available. Program output stayed closed; the operator display is protected".to_owned())?,
    };
    if target.index == operator.index || overlaps(display_rect(target), display_rect(operator)) {
        return Err(
            "Program output cannot use the operator display or a mirrored/overlapping display"
                .into(),
        );
    }
    Ok(target.clone())
}
fn query_inventory(hwnd: usize) -> std::result::Result<Inventory, String> {
    let before = displays::enumerate().map_err(|error| error.to_string())?;
    let operator = displays::operator_display_index(hwnd).map_err(|error| error.to_string())?;
    let after = displays::enumerate().map_err(|error| error.to_string())?;
    let operator_after =
        displays::operator_display_index(hwnd).map_err(|error| error.to_string())?;
    if before != after || operator != operator_after {
        return Err(
            "Displays or the operator window changed during discovery; refresh and try again"
                .into(),
        );
    }
    Ok(Inventory {
        displays: after,
        operator,
    })
}
fn physical_rect(info: &egui::ViewportInfo, zoom: f32) -> Option<Rect> {
    let scale = info.native_pixels_per_point? * zoom;
    let rect = info.inner_rect?;
    if !scale.is_finite() || scale <= 0.0 || !rect.is_finite() || !rect.is_positive() {
        return None;
    }
    Some(Rect::from_min_max(
        (rect.min.to_vec2() * scale).to_pos2().round(),
        (rect.max.to_vec2() * scale).to_pos2().round(),
    ))
}
fn placement_matches(
    info: &egui::ViewportInfo,
    target: &DisplayInfo,
    operator: &DisplayInfo,
    zoom: f32,
) -> bool {
    let Some(actual) = physical_rect(info, zoom) else {
        return false;
    };
    let expected = display_rect(target);
    info.fullscreen == Some(true)
        && !overlaps(actual, display_rect(operator))
        && (actual.min - expected.min).abs().max_elem() <= 4.0
        && (actual.max - expected.max).abs().max_elem() <= 4.0
}

impl RustMixApp {
    fn extended_failure(&mut self, ctx: &egui::Context, error: String) {
        if self.extended_output.target.is_some() {
            self.program_output.close(ctx);
        }
        self.extended_output.clear_route();
        self.extended_output.error = Some(error.clone());
        self.say(error, true);
    }
    fn request_display_inventory(&mut self, ctx: &egui::Context, purpose: Purpose) {
        if self.extended_output.query.is_some() {
            return;
        }
        let Some(hwnd) = self.operator_hwnd.filter(|hwnd| *hwnd != 0) else {
            self.extended_failure(
                ctx,
                "Cannot identify the native operator window; Program output stayed closed".into(),
            );
            return;
        };
        let (sender, receiver) = mpsc::sync_channel(1);
        #[cfg(test)]
        if let Some(injected) = self.extended_output.injected.take() {
            let _ = sender.try_send(injected);
            self.extended_output.query = Some(receiver);
            self.extended_output.query_purpose = purpose;
            self.extended_output.query_started = Some(Instant::now());
            self.extended_output.query_timed_out = false;
            return;
        }
        let repaint = ctx.clone();
        let spawn = std::thread::Builder::new()
            .name("rustmix-display-discovery".into())
            .spawn(move || {
                let _ = sender.try_send(query_inventory(hwnd));
                repaint.request_repaint();
            });
        if spawn.is_err() {
            self.extended_failure(
                ctx,
                "Could not start display discovery; Program output stayed closed".into(),
            );
            return;
        }
        self.extended_output.query = Some(receiver);
        self.extended_output.query_purpose = purpose;
        self.extended_output.query_started = Some(Instant::now());
        self.extended_output.query_timed_out = false;
    }

    pub(crate) fn toggle_extended_output(&mut self, ctx: &egui::Context) {
        if self.extended_output.is_active() || self.extended_output.is_pending() {
            self.program_output.close(ctx);
            self.extended_output.clear_route();
            self.say(
                "Program extended-display output closed; operator window unchanged".into(),
                false,
            );
            return;
        }
        if ctx.embed_viewports() {
            self.extended_failure(
                ctx,
                "This backend cannot open a separate native Program display".into(),
            );
            return;
        }
        if self.extended_output.invalid_selection {
            self.extended_failure(ctx, "Stored display selection is invalid; explicitly select an extended display before starting".into());
            return;
        }
        if self.program_output.is_open() {
            self.program_output.close(ctx);
        }
        self.extended_output.error = None;
        if self.extended_output.query.is_some() {
            self.extended_output.pending_start = true;
        } else {
            self.request_display_inventory(ctx, Purpose::Start);
        }
        ctx.request_repaint();
    }

    pub(crate) fn poll_extended_output(&mut self, ctx: &egui::Context) {
        if self
            .extended_output
            .query_started
            .is_some_and(|started| started.elapsed() > PLACEMENT_TIMEOUT)
            && !self.extended_output.query_timed_out
        {
            self.extended_output.query_timed_out = true;
            self.extended_failure(ctx, "Display discovery timed out; Program output closed. Wait for discovery before retrying".into());
        }
        let result = self
            .extended_output
            .query
            .as_ref()
            .and_then(|query| match query.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Disconnected) => {
                    Some(Err("Display discovery stopped unexpectedly".into()))
                }
                Err(TryRecvError::Empty) => None,
            });
        if let Some(result) = result {
            let purpose = self.extended_output.query_purpose;
            self.extended_output.query = None;
            self.extended_output.query_started = None;
            self.extended_output.last_check = Instant::now();
            match result {
                Err(error) => self.extended_failure(ctx, error),
                Ok(inventory) => {
                    self.extended_output.inventory = Some(inventory.clone());
                    if purpose == Purpose::Start {
                        self.extended_output.pending_start = false;
                        match choose_target(&inventory, self.extended_output.selected_id.as_deref())
                        {
                            Err(error) => self.extended_failure(ctx, error),
                            Ok(target) => {
                                self.program_output.open_on_monitor(ctx, target.index);
                                self.extended_output.target = Some(target);
                                self.extended_output.active = false;
                                self.extended_output.revealed = false;
                                self.extended_output.placement_started = Some(Instant::now());
                                self.extended_output.placement_wait_frames = 1;
                            }
                        }
                    } else if let Some(current) = self.extended_output.target.clone() {
                        match choose_target(&inventory, Some(&current.id)) {
                            Err(error) => self.extended_failure(ctx, error),
                            Ok(target) => {
                                if target.index != current.index
                                    || target.position != current.position
                                    || target.size != current.size
                                {
                                    self.program_output.retarget_monitor(ctx, target.index);
                                    self.extended_output.active = false;
                                    self.extended_output.revealed = false;
                                    self.extended_output.placement_started = Some(Instant::now());
                                    self.extended_output.placement_wait_frames = 1;
                                }
                                self.extended_output.target = Some(target);
                            }
                        }
                    }
                }
            }
            if self.extended_output.pending_start && self.extended_output.target.is_none() {
                self.extended_output.pending_start = false;
                self.request_display_inventory(ctx, Purpose::Start);
            }
        }
        let Some(target) = self.extended_output.target.clone() else {
            return;
        };
        if !self.program_output.is_open() || self.program_output.target_monitor().is_none() {
            self.extended_output.clear_route();
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(33));
        if self.extended_output.placement_wait_frames > 0 {
            self.extended_output.placement_wait_frames -= 1;
            return;
        }
        let operator = self
            .extended_output
            .inventory
            .as_ref()
            .and_then(|inventory| {
                inventory
                    .displays
                    .iter()
                    .find(|display| display.index == inventory.operator)
            })
            .cloned();
        let Some(operator) = operator else {
            self.extended_failure(
                ctx,
                "Operator display became unknown; Program output closed".into(),
            );
            return;
        };
        let operator_window =
            ctx.input(|input| input.raw.viewports.get(&egui::ViewportId::ROOT).cloned());
        if operator_window
            .as_ref()
            .and_then(|info| physical_rect(info, ctx.zoom_factor()))
            .is_some_and(|rect| display_rect(&target).contains(rect.center()))
        {
            self.extended_failure(
                ctx,
                "The operator window moved onto the Program display; Program output closed".into(),
            );
            return;
        }
        let child = ctx.input(|input| {
            input
                .raw
                .viewports
                .get(&program_output::viewport_id())
                .cloned()
        });
        let matches = child
            .as_ref()
            .is_some_and(|info| placement_matches(info, &target, &operator, ctx.zoom_factor()));
        if matches {
            self.program_output.reveal_monitor(ctx);
            self.extended_output.revealed = true;
            self.extended_output.active =
                self.program_output.native_seen() && self.program_output.native_fullscreen();
        } else if self.extended_output.revealed
            || self
                .extended_output
                .placement_started
                .is_some_and(|started| started.elapsed() > PLACEMENT_TIMEOUT)
        {
            self.extended_failure(ctx, "The Program window could not remain on the selected extended display; output closed to protect the operator".into());
            return;
        }
        if self.extended_output.last_check.elapsed() >= WATCH_INTERVAL
            && self.extended_output.query.is_none()
        {
            self.request_display_inventory(ctx, Purpose::Watch);
        }
    }

    pub(crate) fn extended_output_menu(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.label("Fullscreen sends Program directly to an extended display.");
        let refresh = ui.add_enabled(
            self.extended_output.query.is_none(),
            egui::Button::new("Refresh displays"),
        );
        self.automation
            .insert("output.display.refresh".into(), refresh.rect);
        if refresh.clicked() {
            self.request_display_inventory(&ctx, Purpose::Refresh);
        }
        if self.extended_output.query.is_some() {
            ui.label("Discovering native displays…");
        }
        let locked = self.extended_output.is_active() || self.extended_output.is_pending();
        let automatic = ui.add_enabled(
            !locked,
            egui::Button::new("Automatic extended display")
                .selected(self.extended_output.selected_id.is_none()),
        );
        self.automation
            .insert("output.display.auto".into(), automatic.rect);
        if automatic.clicked() {
            self.extended_output.selected_id = None;
            self.extended_output.invalid_selection = false;
            self.extended_output.error = None;
        }
        if let Some(inventory) = self.extended_output.inventory.clone() {
            egui::ScrollArea::vertical()
                .id_salt("output.display.list")
                .max_height(180.0)
                .show(ui, |ui| {
                    for display in &inventory.displays {
                        let safe = choose_target(&inventory, Some(&display.id)).is_ok();
                        let label = format!(
                            "Display {} · {} · {}×{}{}{}",
                            display.index + 1,
                            display.id,
                            display.size.first().copied().unwrap_or(0),
                            display.size.get(1).copied().unwrap_or(0),
                            if display.primary { " · primary" } else { "" },
                            if display.index == inventory.operator {
                                " · operator (protected)"
                            } else {
                                ""
                            }
                        );
                        let choice = ui.add_enabled(
                            !locked && safe,
                            egui::Button::new(label).selected(
                                self.extended_output.selected_id.as_deref()
                                    == Some(display.id.as_str()),
                            ),
                        );
                        self.automation
                            .insert(format!("output.display.{}", display.index), choice.rect);
                        if choice.clicked() {
                            self.extended_output.selected_id = Some(display.id.clone());
                            self.extended_output.invalid_selection = false;
                            self.extended_output.error = None;
                        }
                    }
                });
        } else {
            ui.label("Refresh to inspect connected displays. No monitor is assumed.");
        }
        if locked {
            ui.label("Close Program output before changing the selected display.");
        }
        if let Some(error) = &self.extended_output.error {
            let label = ui.colored_label(RED, error);
            self.automation
                .insert("output.display.error".into(), label.rect);
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
    use super::*;

    fn display(index: usize, id: &str, x: i32) -> DisplayInfo {
        DisplayInfo {
            index,
            id: id.into(),
            primary: index == 0,
            position: [x, 0],
            size: [1920, 1080],
        }
    }
    fn inventory() -> Inventory {
        Inventory {
            operator: 0,
            displays: vec![display(0, "DISPLAY1", 0), display(1, "DISPLAY2", 1920)],
        }
    }
    fn info(display: &DisplayInfo) -> egui::ViewportInfo {
        egui::ViewportInfo {
            parent: Some(egui::ViewportId::ROOT),
            native_pixels_per_point: Some(1.0),
            inner_rect: Some(display_rect(display)),
            outer_rect: Some(display_rect(display)),
            fullscreen: Some(true),
            ..Default::default()
        }
    }
    fn parent(
        app: &mut RustMixApp,
        ctx: &egui::Context,
        child: Option<egui::ViewportInfo>,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let mut raw = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
            events,
            ..Default::default()
        };
        if let Some(child) = child {
            raw.viewports.insert(program_output::viewport_id(), child);
        }
        ctx.run_ui(raw, |ui| app.frame_ui(ui))
    }
    fn click_top(app: &mut RustMixApp, ctx: &egui::Context) {
        let mut last: Option<Rect> = None;
        let mut settled = None;
        for _ in 0..6 {
            parent(app, ctx, None, vec![]);
            let bounds = app.automation["toolbar.fullscreen"];
            if last.is_some_and(|old| {
                old.center().distance(bounds.center()) < 0.5
                    && (old.size() - bounds.size()).length() < 0.5
            }) {
                settled = Some(bounds);
                break;
            }
            last = Some(bounds);
        }
        let bounds = settled.expect("Top Fullscreen control must settle");
        for pressed in [true, false] {
            parent(
                app,
                ctx,
                None,
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
    fn mark_native(ctx: &egui::Context, output: &egui::FullOutput, target: &DisplayInfo) {
        let callback = output.viewport_output[&program_output::viewport_id()]
            .viewport_ui_cb
            .as_ref()
            .unwrap()
            .clone();
        let mut raw = egui::RawInput {
            viewport_id: program_output::viewport_id(),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0))),
            ..Default::default()
        };
        raw.viewports
            .insert(program_output::viewport_id(), info(target));
        let _ = ctx.run_ui(raw, |ui| callback(ui));
    }
    fn prepare() -> (RustMixApp, egui::Context) {
        let mut app = RustMixApp::headless();
        app.operator_hwnd = Some(1); // never used by the injected worker result
        let ctx = egui::Context::default();
        ctx.set_embed_viewports(false);
        app.theme(&ctx);
        (app, ctx)
    }

    #[test]
    fn top_fullscreen_button_routes_program_directly_to_verified_extended_monitor() {
        let (mut app, ctx) = prepare();
        let baseline = app.engine.project().clone();
        let inventory = inventory();
        app.extended_output.injected = Some(Ok(inventory.clone()));
        click_top(&mut app, &ctx);
        let opening = parent(&mut app, &ctx, None, vec![]);
        let child = &opening.viewport_output[&program_output::viewport_id()];
        assert_eq!(child.builder.monitor, Some(1));
        assert_eq!(child.builder.fullscreen, Some(true));
        assert_eq!(
            child.builder.visible,
            Some(false),
            "Never reveal an unverified fallback viewport"
        );
        assert!(app.extended_output.is_pending());
        assert!(!app.extended_output.is_active());
        assert!(!app.fullscreen, "The operator UI must remain windowed");
        assert_eq!(*app.engine.project(), baseline);
        assert!(
            !opening.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&egui::ViewportCommand::Fullscreen(true))
        );

        mark_native(&ctx, &opening, &inventory.displays[1]);
        let routed = parent(&mut app, &ctx, Some(info(&inventory.displays[1])), vec![]);
        assert_eq!(
            routed.viewport_output[&program_output::viewport_id()]
                .builder
                .visible,
            Some(true)
        );
        assert!(app.extended_output.is_active());
        assert_eq!(*app.engine.project(), baseline);
        app.dispatch(Action::ProgramOutputExtend, &ctx);
        assert!(!app.program_output.is_open());
        assert!(!app.extended_output.is_active());
        assert!(!app.fullscreen && !app.allow_close && !app.confirm_close);
        assert_eq!(*app.engine.project(), baseline);
    }

    #[test]
    fn single_display_missing_selection_operator_selection_and_clones_are_fail_closed() {
        for selection in [
            None,
            Some("DISPLAY1"),
            Some("not-connected"),
            Some("bad\nidentity"),
        ] {
            let (mut app, ctx) = prepare();
            let baseline = app.engine.project().clone();
            app.extended_output.restore_selected_id(selection);
            let mut inventory = inventory();
            if selection.is_none() {
                inventory.displays.truncate(1);
            }
            app.extended_output.injected = Some(Ok(inventory));
            app.dispatch(Action::ProgramOutputExtend, &ctx);
            parent(&mut app, &ctx, None, vec![]);
            assert!(!app.program_output.is_open());
            assert!(!app.extended_output.is_active() && !app.extended_output.is_pending());
            assert!(app.extended_output.error.is_some() && app.warning);
            assert!(!app.fullscreen);
            assert_eq!(*app.engine.project(), baseline);
        }
        let mut clones = inventory();
        clones.displays[1].position = [0, 0];
        assert!(choose_target(&clones, None).is_err());
        assert!(choose_target(&clones, Some("DISPLAY2")).is_err());
        let (mut app, ctx) = prepare();
        app.operator_hwnd = None;
        app.dispatch(Action::ProgramOutputExtend, &ctx);
        assert!(!app.program_output.is_open());
        assert!(app.warning);
    }

    #[test]
    fn target_loss_closes_program_and_reordered_indices_follow_stable_identity() {
        let (mut app, ctx) = prepare();
        let baseline = app.engine.project().clone();
        app.extended_output.injected = Some(Ok(inventory()));
        app.dispatch(Action::ProgramOutputExtend, &ctx);
        parent(&mut app, &ctx, None, vec![]);
        let reordered = Inventory {
            operator: 1,
            displays: vec![display(0, "DISPLAY2", 1920), display(1, "DISPLAY1", 0)],
        };
        app.extended_output.injected = Some(Ok(reordered.clone()));
        app.request_display_inventory(&ctx, Purpose::Watch);
        let output = parent(&mut app, &ctx, None, vec![]);
        assert_eq!(app.program_output.target_monitor(), Some(0));
        assert_eq!(
            output.viewport_output[&program_output::viewport_id()]
                .builder
                .monitor,
            Some(0)
        );
        assert_eq!(
            output.viewport_output[&program_output::viewport_id()]
                .builder
                .visible,
            Some(false)
        );
        assert!(
            output.viewport_output[&program_output::viewport_id()]
                .commands
                .contains(&egui::ViewportCommand::SetMonitor(0))
        );
        mark_native(&ctx, &output, &reordered.displays[0]);
        parent(&mut app, &ctx, Some(info(&reordered.displays[0])), vec![]);
        assert!(app.extended_output.is_active());
        let lost = Inventory {
            operator: 0,
            displays: vec![display(0, "DISPLAY1", 0)],
        };
        app.extended_output.injected = Some(Ok(lost));
        app.request_display_inventory(&ctx, Purpose::Watch);
        parent(&mut app, &ctx, None, vec![]);
        assert!(!app.program_output.is_open());
        assert!(app.extended_output.error.is_some() && app.warning);
        assert_eq!(*app.engine.project(), baseline);
        assert!(!app.fullscreen);
    }

    #[test]
    fn unverified_fallback_is_hidden_and_times_out_and_dpi_mapping_is_checked() {
        let (mut app, ctx) = prepare();
        let inventory = inventory();
        app.extended_output.injected = Some(Ok(inventory.clone()));
        app.dispatch(Action::ProgramOutputExtend, &ctx);
        parent(&mut app, &ctx, None, vec![]);
        let wrong = info(&inventory.displays[0]);
        let hidden = parent(&mut app, &ctx, Some(wrong.clone()), vec![]);
        assert_eq!(
            hidden.viewport_output[&program_output::viewport_id()]
                .builder
                .visible,
            Some(false)
        );
        assert!(!app.extended_output.is_active());
        app.extended_output.placement_started = Instant::now().checked_sub(Duration::from_secs(6));
        parent(&mut app, &ctx, Some(wrong), vec![]);
        assert!(!app.program_output.is_open());
        assert!(app.warning);

        let mut scaled = info(&inventory.displays[1]);
        scaled.native_pixels_per_point = Some(1.5);
        let rect = display_rect(&inventory.displays[1]);
        scaled.inner_rect = Some(Rect::from_min_max(
            (rect.min.to_vec2() / 1.875).to_pos2(),
            (rect.max.to_vec2() / 1.875).to_pos2(),
        ));
        assert!(placement_matches(
            &scaled,
            &inventory.displays[1],
            &inventory.displays[0],
            1.25
        ));
        scaled.native_pixels_per_point = Some(f32::NAN);
        assert!(!placement_matches(
            &scaled,
            &inventory.displays[1],
            &inventory.displays[0],
            1.25
        ));
    }
}
