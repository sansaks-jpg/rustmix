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
