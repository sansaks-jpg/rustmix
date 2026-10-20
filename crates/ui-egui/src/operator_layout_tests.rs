//! Check painted text rather than assuming that reserved widget bounds imply alignment.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

fn paint(app: &mut RustMixApp, ctx: &egui::Context, size: Vec2) -> egui::FullOutput {
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            ..Default::default()
        },
        |ui| app.frame_ui(ui),
    )
}

fn text_in(shapes: &[egui::epaint::ClippedShape], label: &str, area: Rect) -> Rect {
    shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == label => {
                Some(text.visual_bounding_rect())
            }
            _ => None,
        })
        .find(|rect| area.expand(2.0).contains_rect(*rect))
        .expect("Painted label must be contained in the expected control")
}

fn event_frame(app: &mut RustMixApp, ctx: &egui::Context, events: Vec<egui::Event>) {
    let _ = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
            events,
            ..Default::default()
        },
        |ui| app.frame_ui(ui),
    );
}

fn click(app: &mut RustMixApp, ctx: &egui::Context, id: &str) {
    for _ in 0..4 {
        event_frame(app, ctx, vec![]);
    }
    let pos = app.automation.get(id).expect("visible control").center();
    for pressed in [true, false] {
        event_frame(
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
fn display_labels_are_right_aligned_and_colours_share_one_row() {
    for size in [
        Vec2::new(1100.0, 700.0),
        Vec2::new(1600.0, 900.0),
        Vec2::new(1917.0, 1198.0),
    ] {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.dispatch(Action::Settings, &ctx);
        for _ in 0..4 {
            let _ = paint(&mut app, &ctx, size);
        }
        let output = paint(&mut app, &ctx, size);
        let page = *app.automation.get("dialog.settings.page").unwrap();
        let mut fields = vec![];
        for (name, label, field) in [
            ("theme", "Theme:", "theme"),
            ("previewColor", "Preview Colour:", "previewColor"),
            ("frameRate", "Master Frame Rate:", "frameRate"),
            ("outputSize", "Output Size:", "outputSize"),
        ] {
            let column = *app
                .automation
                .get(&format!("dialog.settings.label.{name}"))
                .unwrap();
            let field = *app
                .automation
                .get(&format!("dialog.settings.{field}"))
                .unwrap();
            let ink = text_in(&output.shapes, label, column);
            assert!(
                (ink.right() - column.right()).abs() <= 2.0,
                "actual label ink must align right: {label} at {size:?}"
            );
            assert!(column.right() + 8.0 <= field.left());
            assert!(page.contains_rect(field));
            fields.push(field);
        }
        assert!(fields.windows(2).all(|pair| {
            pair.first()
                .zip(pair.get(1))
                .is_some_and(|(left, right)| (left.left() - right.left()).abs() <= 1.0)
        }));
        let preview = *app.automation.get("dialog.settings.previewColor").unwrap();
        let program = *app.automation.get("dialog.settings.programColor").unwrap();
        assert!((preview.center().y - program.center().y).abs() <= 1.0);
        assert!(preview.right() < program.left() && page.contains_rect(program));
        let confirm = *app.automation.get("dialog.settings.apply").unwrap();
        let cancel = *app.automation.get("dialog.settings.cancel").unwrap();
        let scale = (size.x / 1912.0).clamp(0.6, 1.0);
        assert!(confirm.width() >= (140.0 * scale).max(92.0) - 1.0);
        assert!(cancel.width() >= (186.0 * scale).max(112.0) - 1.0);
        assert!(confirm.height() >= (42.0 * scale).max(29.0) - 1.0);
        assert!(confirm.right() <= cancel.left());
    }
}

#[test]
fn canvas_selection_is_staged_header_close_cancels_and_ok_commits_once() {
    let mut app = RustMixApp::headless();
    let ctx = egui::Context::default();
    app.theme(&ctx);
    let original = app.engine.project().clone();
    app.dispatch(Action::Settings, &ctx);
    click(&mut app, &ctx, "dialog.settings.outputSize");
    click(&mut app, &ctx, "dialog.settings.canvas.1280");
    assert_eq!(app.settings_draft.as_ref().unwrap().width, 1280);
    assert_eq!(*app.engine.project(), original);
    click(&mut app, &ctx, "dialog.settings.chrome.close");
    assert!(!app.settings && app.settings_draft.is_none());
    assert_eq!(*app.engine.project(), original);
    app.dispatch(Action::Settings, &ctx);
    click(&mut app, &ctx, "dialog.settings.outputSize");
    click(&mut app, &ctx, "dialog.settings.canvas.1280");
    click(&mut app, &ctx, "dialog.settings.apply");
    assert!(!app.settings && app.settings_draft.is_none());
    let committed = app.engine.project();
    assert_eq!(
        (committed.preferences.width, committed.preferences.height),
        (1280, 720)
    );
    assert_eq!(committed.revision, original.revision + 1);
    assert_eq!(
        (committed.program, committed.preview),
        (original.program, original.preview)
    );
}

#[test]
fn unfilled_vector_controls_paint_readable_glyphs_in_both_themes() {
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        let ctx = egui::Context::default();
        ctx.set_theme(theme);
        let mut gear = Rect::NOTHING;
        let mut caret = Rect::NOTHING;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            gear = vector_icon_button(ui, VectorIcon::Gear, Vec2::splat(24.0), false, None).rect;
            caret = caret_menu_button(ui, Vec2::new(22.0, 24.0), |_| {})
                .response
                .rect;
        });
        let expected = ctx.style_of(theme).visuals.text_color();
        for area in [gear, caret] {
            assert!(
                output.shapes.iter().any(|shape| match &shape.shape {
                    egui::Shape::Path(path) =>
                        area.contains_rect(path.visual_bounding_rect())
                            && (path.fill == expected
                                || matches!(&path.stroke.color, egui::epaint::ColorMode::Solid(color) if *color == expected)),
                    egui::Shape::LineSegment { points, stroke } =>
                        points.iter().all(|p| area.contains(*p)) && stroke.color == expected,
                    egui::Shape::Circle(circle) =>
                        area.contains_rect(circle.visual_bounding_rect())
                            && circle.stroke.color == expected,
                    _ => false,
                }),
                "{theme:?} vector glyph must use a readable foreground in {area:?}"
            );
        }
    }
}

#[test]
fn painted_monitor_titles_start_beside_left_inset_and_take_labels_are_centered() {
    for size in [
        Vec2::new(1100.0, 700.0),
        Vec2::new(1600.0, 900.0),
        Vec2::new(1920.0, 1200.0),
    ] {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        for _ in 0..4 {
            let _ = paint(&mut app, &ctx, size);
        }
        let output = paint(&mut app, &ctx, size);
        for (bus, input, badge) in [
            ("preview", app.engine.project().preview, "PVW"),
            ("program", app.engine.project().program, "PGM"),
        ] {
            let header = *app
                .automation
                .get(&format!("monitor.{bus}.header"))
                .unwrap();
            let title = *app.automation.get(&format!("monitor.{bus}.title")).unwrap();
            let source = app.engine.project().input(input).unwrap();
            let label = format!(
                "{}  {} · {badge}",
                app.engine.project().input_number(input),
                source.name
            );
            let ink = text_in(&output.shapes, &label, header);
            assert!(header.contains_rect(title));
            assert!(
                (ink.left() - header.left() - 8.0).abs() <= 2.0,
                "{bus} title must start beside the inset: ink={ink:?}, header={header:?}"
            );
            let gear = *app
                .automation
                .get(&format!("monitor.{bus}.properties"))
                .unwrap();
            assert!(ink.right() < gear.left());
        }
        for (id, label) in [
            ("switcher.quickPlay", "Quick Play"),
            ("switcher.cut", "Cut"),
            ("switcher.ftb", "FTB"),
        ] {
            let button = *app.automation.get(id).unwrap();
            let ink = text_in(&output.shapes, label, button);
            assert!(
                (ink.center().x - button.center().x).abs() <= 2.0,
                "{id} must center its painted label: ink={ink:?}, button={button:?}"
            );
            assert!((ink.center().y - button.center().y).abs() <= 2.0);
        }
        for (index, slot) in app.engine.project().transition_slots.iter().enumerate() {
            let button = *app
                .automation
                .get(&format!("switcher.slot.{}", index + 1))
                .unwrap();
            let ink = text_in(&output.shapes, slot.kind.label(), button);
            assert!((ink.center().x - button.center().x).abs() <= 2.0);
            let take = *app.automation.get("switcher.cut").unwrap();
            let menu = *app
                .automation
                .get(&format!("switcher.slot.{}.settings", index + 1))
                .unwrap();
            assert!((button.width() - (take.width() - 22.0)).abs() <= 1.0);
            assert!(menu.left() >= button.right());
            assert!(menu.right() <= take.right() + 1.0);
        }
    }
}

#[test]
fn recording_labels_have_a_readable_gutter_and_filename_uses_the_page_width() {
    for size in [
        Vec2::new(734.0, 466.0),
        Vec2::new(1100.0, 700.0),
        Vec2::new(1600.0, 900.0),
        Vec2::new(1920.0, 1200.0),
    ] {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.recording_setup = true;
        for _ in 0..4 {
            let _ = paint(&mut app, &ctx, size);
        }
        let output = paint(&mut app, &ctx, size);
        let page = *app.automation.get("dialog.recording.page").unwrap();
        let label = text_in(&output.shapes, "Size", page);
        let value = text_in(&output.shapes, "640×360 CPU reference", page);
        let gutter = value.left() - label.right();
        assert!(
            (11.0..=15.0).contains(&gutter),
            "Recording label/value ink needs a 12-point gutter, got {gutter}"
        );
        if size.x == 1600.0 {
            let filename = *app.automation.get("dialog.recording.filename").unwrap();
            assert!((335.0..=350.0).contains(&filename.width()));
        }
    }
}

#[test]
fn recording_native_guard_rejects_overlapping_footer_and_side_by_side_browse() {
    let mut app = RustMixApp::headless();
    let ctx = egui::Context::default();
    app.theme(&ctx);
    app.recording_setup = true;
    let size = Vec2::new(1600.0, 900.0);
    for _ in 0..4 {
        let _ = paint(&mut app, &ctx, size);
    }
    app.validate_recording_geometry(&ctx).unwrap();
    let body = *app.automation.get("dialog.recording.body").unwrap();
    let ok = *app.automation.get("dialog.recording.ok").unwrap();
    app.automation.insert("dialog.recording.ok".into(), body);
    assert!(app.validate_recording_geometry(&ctx).is_err());
    app.automation.insert("dialog.recording.ok".into(), ok);
    let filename = *app.automation.get("dialog.recording.filename").unwrap();
    let browse = *app.automation.get("dialog.recording.browse").unwrap();
    app.automation.insert(
        "dialog.recording.browse".into(),
        browse.translate(egui::vec2(0.0, filename.top() - browse.top())),
    );
    assert!(app.validate_recording_geometry(&ctx).is_err());
}
