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

fn escape_events(repeat: bool, modifiers: egui::Modifiers) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: Some(egui::Key::Escape),
            pressed,
            repeat: repeat && pressed,
            modifiers,
        })
        .collect()
}

#[test]
fn input_general_labels_fields_header_and_navigation_use_the_available_form_width() {
    for size in [
        Vec2::new(734.0, 466.0),
        Vec2::new(1100.0, 700.0),
        Vec2::new(1600.0, 900.0),
        Vec2::new(1917.0, 1168.0),
    ] {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.dispatch(Action::Properties(InputId(3)), &ctx);
        let before = app.engine.project().clone();
        for _ in 0..5 {
            let _ = paint(&mut app, &ctx, size);
        }
        let output = paint(&mut app, &ctx, size);
        app.validate_dialog_geometry(&ctx, "dialog.properties")
            .unwrap();
        let page = app.automation["dialog.properties.page"];
        for (label, key) in [
            ("Name", "name"),
            ("Aspect Ratio", "aspect"),
            ("Category", "category"),
        ] {
            let reserved = app.automation[&format!("dialog.properties.{key}.label")];
            let field = app.automation[&format!("dialog.properties.{key}")];
            let glyph = text_in(&output.shapes, label, reserved);
            assert!(
                (glyph.right() - reserved.right()).abs() <= 2.0,
                "{label}: {glyph:?} {reserved:?}"
            );
            assert!(field.left() - reserved.right() >= 6.0);
            assert!(field.left() - reserved.right() <= 10.0);
            assert!(page.expand(1.0).contains_rect(field));
            if size.x >= 1600.0 {
                assert!(
                    field.width() >= page.width() * 0.30,
                    "{key} should use its form column"
                );
            }
        }
        let header = app.automation["dialog.properties.sourceHeader"];
        let caption = text_in(&output.shapes, "Camera C · synthetic Color bars", header);
        assert!((caption.left() - header.left()).abs() <= 2.0);
        assert!(header.right() + 2.0 <= app.automation["dialog.properties.change"].left());
        let general = app.automation["dialog.properties.page.General"];
        let expected = if size.y < 700.0 {
            18.5
        } else {
            (size.x * 38.0 / 1917.0).clamp(25.0, 38.0)
        };
        assert!((general.height() - expected).abs() <= 1.0);
        assert_eq!(*app.engine.project(), before);
    }
}

#[test]
fn expanded_input_name_keeps_real_typing_staged_until_ok_and_escape_discards_it() {
    let mut app = RustMixApp::headless();
    let ctx = egui::Context::default();
    app.theme(&ctx);
    let before = app.engine.project().clone();
    for (name, commit) in [("Discarded source name", false), ("Renamed source", true)] {
        app.dispatch(Action::Properties(InputId(3)), &ctx);
        click(&mut app, &ctx, "dialog.properties.name");
        event_frame(
            &mut app,
            &ctx,
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: Some(egui::Key::A),
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..egui::Modifiers::NONE
                    },
                },
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: Some(egui::Key::A),
                    pressed: false,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..egui::Modifiers::NONE
                    },
                },
                egui::Event::Text(name.into()),
            ],
        );
        assert_eq!(app.properties.as_ref().unwrap().name, name);
        assert_eq!(*app.engine.project(), before);
        if commit {
            click(&mut app, &ctx, "dialog.properties.apply");
            assert_eq!(app.engine.project().input(InputId(3)).unwrap().name, name);
        } else {
            event_frame(&mut app, &ctx, escape_events(false, egui::Modifiers::NONE));
            assert_eq!(*app.engine.project(), before);
        }
        assert!(app.properties.is_none());
        assert_eq!(app.engine.project().program, before.program);
        assert_eq!(app.engine.project().preview, before.preview);
    }
}

#[test]
fn default_source_dialog_width_tracks_the_parent_and_preserves_a_real_operator_resize() {
    let mut app = RustMixApp::headless();
    let ctx = egui::Context::default();
    app.theme(&ctx);
    app.dispatch(Action::Add, &ctx);
    let before = app.engine.project().clone();
    for size in [
        Vec2::new(1600.0, 900.0),
        Vec2::new(1917.0, 1168.0),
        Vec2::new(1100.0, 700.0),
        Vec2::new(1917.0, 1168.0),
    ] {
        for _ in 0..5 {
            let _ = paint(&mut app, &ctx, size);
        }
        let window = *app
            .automation
            .get("dialog.add_input.chrome.window")
            .unwrap();
        let expected = (size.x * 0.707).max(780.0).min(size.x - 36.0);
        assert!(
            (window.width() - expected).abs() <= 4.0,
            "Retained default width must follow the parent: {size:?}, {window:?}, expected {expected}"
        );
        app.validate_dialog_geometry(&ctx, "dialog.add").unwrap();
        assert_eq!(*app.engine.project(), before);
    }
    // The starting dialog is below its right boundary, leaving room for a real
    // edge drag to distinguish a custom width from the responsive default.
    let size = Vec2::new(1917.0, 1168.0);
    let original = *app
        .automation
        .get("dialog.add_input.chrome.window")
        .unwrap();
    let start = egui::pos2(original.right() - 2.0, original.center().y);
    for (pos, pressed) in [
        (start, Some(true)),
        (start + Vec2::new(100.0, 0.0), None),
        (start + Vec2::new(100.0, 0.0), Some(false)),
    ] {
        let mut events = vec![egui::Event::PointerMoved(pos)];
        if let Some(pressed) = pressed {
            events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
        }
        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ui| app.frame_ui(ui),
        );
    }
    for _ in 0..5 {
        let _ = paint(&mut app, &ctx, size);
    }
    let custom = *app
        .automation
        .get("dialog.add_input.chrome.window")
        .unwrap();
    assert!(
        custom.width() > original.width() + 50.0,
        "The test must perform an actual operator resize"
    );
    for _ in 0..5 {
        let _ = paint(&mut app, &ctx, Vec2::new(2100.0, 1300.0));
    }
    let retained = app
        .automation
        .get("dialog.add_input.chrome.window")
        .unwrap();
    assert!(
        (retained.width() - custom.width()).abs() <= 2.0,
        "A custom operator width must remain intact"
    );
    assert_eq!(*app.engine.project(), before);
}

#[test]
fn escape_cancels_operator_drafts_without_committing_or_disturbing_a_manual_transition() {
    for action in [
        Action::Add,
        Action::Properties(InputId(2)),
        Action::Settings,
        Action::OverlaySettings(0),
        Action::ExternalSettings(1),
        Action::RecordingSetup,
        Action::StreamingSetup,
        Action::MultiCorder,
        Action::PlayList,
        Action::ShortcutsWindow,
    ] {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.execute(Command::SetManualProgress(0.4));
        let before = app.engine.project().clone();
        app.dispatch(action, &ctx);
        if let Some(draft) = &mut app.settings_draft {
            draft.ui_scale = 1.25;
        }
        if let Some(draft) = &mut app.properties {
            draft.name = "Unsaved input name".into();
        }
        if let Some((_, draft)) = &mut app.overlay_settings {
            draft.title = "Unsaved title".into();
        }
        for _ in 0..5 {
            event_frame(&mut app, &ctx, vec![]);
        }
        if app.add_open {
            click(&mut app, &ctx, "dialog.add.video.path");
            event_frame(
                &mut app,
                &ctx,
                vec![egui::Event::Text("C:/media/not-added.mp4".into())],
            );
            assert!(ctx.text_edit_focused());
        }
        event_frame(&mut app, &ctx, escape_events(false, egui::Modifiers::NONE));
        assert!(
            !app.add_open
                && app.properties.is_none()
                && !app.settings
                && app.overlay_settings.is_none()
                && app.external_settings.is_none()
                && !app.recording_setup
                && !app.streaming_setup
                && !app.multicorder_open
                && !app.playlist_open
                && !app.shortcuts_open,
            "Escape must use the dialog's existing cancellation path"
        );
        assert_eq!(*app.engine.project(), before);
        assert!(app.engine.transition().is_some_and(
            |transition| transition.manual && (transition.progress - 0.4).abs() < 0.01
        ));
    }
}

#[test]
fn escape_closes_an_open_combo_before_its_dialog_and_ignores_repeat_or_modified_keys() {
    let mut app = RustMixApp::headless();
    let ctx = egui::Context::default();
    app.theme(&ctx);
    app.dispatch(Action::Settings, &ctx);
    click(&mut app, &ctx, "dialog.settings.frameRate");
    let before = app.engine.project().clone();
    assert!(egui::Popup::is_any_open(&ctx));
    let mut first_press = escape_events(false, egui::Modifiers::NONE);
    first_press.retain(|event| matches!(event, egui::Event::Key { pressed: true, .. }));
    event_frame(&mut app, &ctx, first_press);
    assert!(app.settings, "The first Escape belongs to the open combo");
    assert!(!egui::Popup::is_any_open(&ctx));
    let mut held_repeat = escape_events(true, egui::Modifiers::NONE);
    held_repeat.retain(|event| matches!(event, egui::Event::Key { pressed: true, .. }));
    event_frame(&mut app, &ctx, held_repeat);
    assert!(
        app.settings,
        "Holding Escape must not discard the parent dialog"
    );
    let mut release = escape_events(false, egui::Modifiers::NONE);
    release.retain(|event| matches!(event, egui::Event::Key { pressed: false, .. }));
    event_frame(&mut app, &ctx, release);
    event_frame(&mut app, &ctx, escape_events(false, egui::Modifiers::CTRL));
    assert!(
        app.settings,
        "A modified Escape is not the dialog cancel command"
    );
    event_frame(&mut app, &ctx, escape_events(false, egui::Modifiers::NONE));
    assert!(!app.settings);
    assert_eq!(*app.engine.project(), before);
}

#[test]
fn escape_only_closes_the_topmost_operator_window() {
    let mut app = RustMixApp::headless();
    let ctx = egui::Context::default();
    app.theme(&ctx);
    app.dispatch(Action::Settings, &ctx);
    for _ in 0..4 {
        event_frame(&mut app, &ctx, vec![]);
    }
    app.dispatch(Action::OverlaySettings(0), &ctx);
    for _ in 0..5 {
        event_frame(&mut app, &ctx, vec![]);
    }
    let before = app.engine.project().clone();
    event_frame(&mut app, &ctx, escape_events(false, egui::Modifiers::NONE));
    assert!(app.settings);
    assert!(app.overlay_settings.is_none());
    assert_eq!(*app.engine.project(), before);
}

#[test]
fn quick_input_categories_open_their_real_pages_and_survive_cancel_reopen() {
    let mut app = RustMixApp::headless();
    let ctx = egui::Context::default();
    app.theme(&ctx);
    app.execute(Command::SetInputsPaused(true));
    let before = app.engine.project().clone();
    for (key, tab, field) in [
        ("video", dialogs::VIDEO_TAB, "dialog.add.video.path"),
        ("image", 1, "dialog.add.name"),
        ("audio", 12, "dialog.add.audio.path"),
        ("microphone", 13, "dialog.add.audio.capture"),
        ("generators", 0, "dialog.add.name"),
        ("shapes", 3, "dialog.add.name"),
        ("bars", 6, "dialog.add.name"),
        ("title", 14, "dialog.add.name"),
        ("studio", 4, "dialog.add.name"),
        ("colour", 11, "dialog.add.name"),
        ("timer", 15, "dialog.add.name"),
    ] {
        click(&mut app, &ctx, "production.add.menu");
        click(&mut app, &ctx, &format!("production.add.category.{key}"));
        for _ in 0..4 {
            event_frame(&mut app, &ctx, vec![]);
        }
        assert!(app.add_open);
        assert_eq!(app.add_tab, tab, "Quick category {key} opened another page");
        assert!(
            ctx.viewport_rect()
                .contains_rect(*app.automation.get(field).unwrap())
        );
        click(&mut app, &ctx, "dialog.add.cancel");
        assert!(!app.add_open);
        click(&mut app, &ctx, "production.add");
        for _ in 0..4 {
            event_frame(&mut app, &ctx, vec![]);
        }
        assert_eq!(app.add_tab, tab, "Add Input forgot category {key}");
        assert!(
            ctx.viewport_rect()
                .contains_rect(*app.automation.get(field).unwrap())
        );
        click(&mut app, &ctx, "dialog.add.cancel");
        assert_eq!(*app.engine.project(), before);
        assert!(!app.rec_active && !app.stream_active && !app.mc_running);
    }
}

#[test]
fn unsupported_output_controls_have_neutral_surfaces_and_do_not_pretend_to_save_or_start() {
    for (action, id, close) in [
        (
            Action::StreamingSetup,
            "dialog.streaming.start",
            "dialog.streaming.close",
        ),
        (
            Action::StreamingSetup,
            "dialog.streaming.save",
            "dialog.streaming.close",
        ),
        (
            Action::MultiCorder,
            "dialog.multicorder.toggle",
            "dialog.multicorder.close",
        ),
    ] {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.dispatch(action, &ctx);
        for _ in 0..4 {
            event_frame(&mut app, &ctx, vec![]);
        }
        let before = app.engine.project().clone();
        let output = paint(&mut app, &ctx, Vec2::new(1600.0, 900.0));
        let target = *app.automation.get(id).unwrap();
        let surfaces: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect)
                    if target.expand(1.0).contains_rect(rect.rect)
                        && rect.rect.area() >= target.area() * 0.7 =>
                {
                    Some(rect.fill)
                }
                _ => None,
            })
            .collect();
        assert!(
            !surfaces.is_empty(),
            "Actual disabled button surface must be painted: {id}"
        );
        for fill in surfaces {
            let [r, g, b, _] = fill.to_array();
            assert!(
                g <= r.max(b).saturating_add(8),
                "Unavailable control must not advertise an active green surface: {id}, {fill:?}"
            );
        }
        click(&mut app, &ctx, id);
        assert!(
            app.streaming_setup || app.multicorder_open,
            "Unavailable action must not imply saving or close the setup dialog: {id}"
        );
        assert!(!app.rec_active && !app.stream_active && !app.mc_running && app.job.is_none());
        assert_eq!(*app.engine.project(), before);
        click(&mut app, &ctx, close);
        assert!(!app.streaming_setup && !app.multicorder_open);
        assert_eq!(*app.engine.project(), before);
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
        assert!(
            (preview.center().y - program.center().y).abs() <= 1.0,
            "Colour controls must share a centre at {size:?}: {preview:?} / {program:?}"
        );
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
            caret = caret_menu_button(ui, 24.0, |_| {}).response.rect;
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
fn dark_filled_dialog_exits_keep_white_text_in_light_theme_and_cancel_safely() {
    for (action, id, label) in [
        (Action::Add, "dialog.add.cancel", "Cancel"),
        (
            Action::Properties(InputId(3)),
            "dialog.properties.cancel",
            "Cancel",
        ),
        (Action::Settings, "dialog.settings.cancel", "Cancel"),
        (
            Action::OverlaySettings(0),
            "dialog.overlay.cancel",
            "Cancel",
        ),
        (
            Action::ExternalSettings(1),
            "dialog.external.cancel",
            "Cancel",
        ),
        (Action::RecordingSetup, "dialog.recording.cancel", "Cancel"),
        (Action::StreamingSetup, "dialog.streaming.close", "Close"),
        (Action::MultiCorder, "dialog.multicorder.close", "Close"),
        (Action::ShortcutsWindow, "dialog.shortcuts.close", "Close"),
    ] {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.theme(&ctx);
        app.dispatch(Action::Settings, &ctx);
        app.settings_draft.as_mut().unwrap().ui_theme = UiTheme::Light;
        click(&mut app, &ctx, "dialog.settings.apply");
        let before = app.engine.project().clone();
        app.dispatch(action, &ctx);
        for _ in 0..4 {
            let _ = paint(&mut app, &ctx, Vec2::new(1600.0, 900.0));
        }
        let output = paint(&mut app, &ctx, Vec2::new(1600.0, 900.0));
        let area = *app.automation.get(id).unwrap();
        let text = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text)
                    if text.galley.job.text == label
                        && area.expand(2.0).contains_rect(text.visual_bounding_rect()) =>
                {
                    Some(text)
                }
                _ => None,
            })
            .expect("actual painted exit label");
        assert!(!text.galley.job.sections.is_empty());
        assert!(
            text.galley
                .job
                .sections
                .iter()
                .all(|section| section.format.color == Color32::WHITE),
            "Dark-filled {id} must retain a readable label in Light theme"
        );
        click(&mut app, &ctx, id);
        assert_eq!(
            *app.engine.project(),
            before,
            "Closing {id} must cancel without live mutations"
        );
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
