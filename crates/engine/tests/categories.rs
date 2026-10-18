#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use rustmix_engine::*;

#[test]
fn category_and_theme_edits_preserve_live_sources_and_duplicate_identity() {
    let mut engine = Engine::default();
    let program = engine.project().program;
    let preview = engine.project().preview;
    engine
        .execute(Command::SetInputCategory(InputId(3), Some(21)), 0)
        .unwrap();
    let mut preferences = engine.project().preferences.clone();
    preferences.categories[21].name = "Remote reporters".into();
    preferences.ui_theme = UiTheme::Light;
    preferences.preview_color = [12, 34, 56];
    preferences.program_color = [65, 43, 21];
    engine
        .execute(Command::SetPreferences(preferences.clone()), 0)
        .unwrap();
    engine
        .execute(Command::DuplicateInput(InputId(3)), 0)
        .unwrap();
    assert_eq!(engine.project().inputs[3].category, Some(21));
    assert_eq!(engine.project().program, program);
    assert_eq!(engine.project().preview, preview);
    assert_eq!(engine.project().preferences, preferences);
    assert!(engine.transition().is_none());
    engine
        .execute(Command::SetInputCategory(InputId(3), None), 0)
        .unwrap();
    assert_eq!(engine.project().input(InputId(3)).unwrap().category, None);
    let definition = Command::SetInputCategory(InputId(3), None).definition();
    assert_eq!(definition.id, "input.category.set");
    assert!(definition.idempotent);
    assert!(!definition.live);
}

#[test]
fn malformed_categories_are_rejected_atomically() {
    let mut engine = Engine::default();
    let before = engine.project().clone();
    for command in [
        Command::SetInputCategory(InputId(3), Some(22)),
        Command::SetInputCategory(InputId(999), Some(0)),
    ] {
        assert!(engine.execute(command, 0).is_err());
        assert_eq!(engine.project(), &before);
    }
    for malformed in 0..6 {
        let mut preferences = before.preferences.clone();
        match malformed {
            0 => {
                preferences.categories.pop();
            }
            1 => preferences.categories[21].id = 0,
            2 => preferences.categories[21].id = 22,
            3 => preferences.categories[21].name = " ".into(),
            4 => preferences.categories[21].name = "x".repeat(49),
            _ => preferences.categories[21].name = "bad\nlabel".into(),
        }
        assert!(
            engine
                .execute(Command::SetPreferences(preferences), 0)
                .is_err()
        );
        assert_eq!(engine.project(), &before);
    }
    // A character bound permits non-ASCII labels without counting UTF-8 bytes.
    let mut preferences = before.preferences.clone();
    preferences.categories[21].name = "界".repeat(48);
    engine
        .execute(Command::SetPreferences(preferences), 0)
        .unwrap();
}

#[test]
fn schema_four_roundtrip_and_all_legacy_migrations_preserve_backups() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("categories.json");
    let mut engine = Engine::default();
    engine
        .execute(Command::SetInputCategory(InputId(3), Some(7)), 0)
        .unwrap();
    persistence::save(&path, engine.project()).unwrap();
    assert_eq!(persistence::load(&path).unwrap(), *engine.project());
    for version in 1..=3 {
        let path = directory.path().join(format!("legacy-{version}.json"));
        let mut value = serde_json::to_value(ProjectState::demo()).unwrap();
        value["schema"] = version.into();
        value["duration_ms"] = 650.into();
        let preferences = value["preferences"].as_object_mut().unwrap();
        for key in ["categories", "ui_theme", "preview_color", "program_color"] {
            preferences.remove(key);
        }
        for input in value["inputs"].as_array_mut().unwrap() {
            input.as_object_mut().unwrap().remove("category");
        }
        let original = serde_json::to_vec(&value).unwrap();
        std::fs::write(&path, &original).unwrap();
        let migrated = persistence::load(&path).unwrap();
        assert_eq!(migrated.schema, 4);
        assert_eq!(migrated.preferences.categories, default_categories());
        assert_eq!(migrated.preferences.ui_theme, UiTheme::Dark);
        assert_eq!(migrated.preferences.preview_color, [234, 128, 24]);
        assert_eq!(migrated.preferences.program_color, [22, 163, 74]);
        assert!(migrated.inputs.iter().all(|input| input.category.is_none()));
        assert_eq!(
            migrated.transition_slots[0].duration_ms,
            if version < 3 { 650 } else { 1000 }
        );
        persistence::save(&path, &migrated).unwrap();
        assert_eq!(
            std::fs::read(persistence::backup_path(&path, 1)).unwrap(),
            original
        );
        assert!(
            Engine::from_project(migrated)
                .unwrap()
                .playlist_run()
                .is_none()
        );
    }
}
#[test]
fn presentation_changes_leave_an_active_transition_and_automation_running() {
    let mut engine = Engine::default();
    engine
        .execute(Command::Transition(TransitionKind::Fade), 0)
        .unwrap();
    engine.tick(200);
    let transition = engine.transition().unwrap().clone();
    let program = engine.project().program;
    let preview = engine.project().preview;
    engine
        .execute(Command::SetInputCategory(InputId(1), Some(0)), 200)
        .unwrap();
    let mut preferences = engine.project().preferences.clone();
    preferences.ui_theme = UiTheme::Light;
    engine
        .execute(Command::SetPreferences(preferences), 200)
        .unwrap();
    assert_eq!(engine.project().program, program);
    assert_eq!(engine.project().preview, preview);
    let current = engine.transition().unwrap();
    assert_eq!(current.from, transition.from);
    assert_eq!(current.to, transition.to);
    assert_eq!(current.progress, transition.progress);
    assert_eq!(current.start_ms, transition.start_ms);
    assert!(
        engine
            .enabled(&Command::SetInputCategory(InputId(999), None))
            .is_err()
    );
    assert!(
        engine
            .enabled(&Command::SetInputCategory(InputId(1), Some(22)))
            .is_err()
    );
}
