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
fn schema_roundtrip_and_all_legacy_migrations_preserve_backups() {
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
        assert_eq!(migrated.schema, SCHEMA);
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

#[test]
fn typed_category_filter_selects_the_assignment_and_pseudo_values() {
    let mut engine = Engine::default();
    engine
        .execute(Command::SetInputCategory(InputId(3), Some(0)), 0)
        .unwrap();
    engine
        .execute(Command::SetInputCategory(InputId(5), Some(21)), 0)
        .unwrap();

    let red = engine.project().visible_inputs(InputCategory::Red);
    assert_eq!(red.len(), 1);
    assert_eq!(red[0].id, InputId(3));

    let custom = engine.project().visible_inputs(InputCategory::custom(16));
    assert_eq!(custom.len(), 1);
    assert_eq!(custom[0].id, InputId(5));

    // The remaining inputs are uncategorised, so a concrete colour is empty.
    assert!(
        engine
            .project()
            .visible_inputs(InputCategory::Green)
            .is_empty()
    );

    let all = engine.project().visible_inputs(InputCategory::All);
    assert_eq!(all.len(), engine.project().inputs.len());
    let search = engine.project().visible_inputs(InputCategory::Search);
    assert_eq!(search.len(), engine.project().inputs.len());

    // The stored id of the newest assignment decodes back to the typed value.
    assert_eq!(
        InputCategory::from_stored(engine.project().input(InputId(5)).unwrap().category),
        Some(InputCategory::custom(16))
    );
}

#[test]
fn combined_settings_apply_validates_name_and_preferences_atomically() {
    let mut engine = Engine::default();
    let before = engine.project().clone();
    let mut changed = before.preferences.clone();
    changed.ui_theme = UiTheme::Light;
    assert!(
        engine
            .execute(
                Command::ApplySettings {
                    name: "\n".into(),
                    preferences: changed
                },
                0
            )
            .is_err()
    );
    assert_eq!(engine.project(), &before);
    let mut malformed = before.preferences.clone();
    malformed.categories[0].id = 21;
    assert!(
        engine
            .execute(
                Command::ApplySettings {
                    name: "Valid new name".into(),
                    preferences: malformed
                },
                0
            )
            .is_err()
    );
    assert_eq!(engine.project(), &before);
}

#[test]
fn combined_settings_apply_commits_once_and_preserves_active_program() {
    let mut engine = Engine::default();
    engine
        .execute(Command::Transition(TransitionKind::Wipe), 0)
        .unwrap();
    engine.tick(250);
    let before = engine.project().clone();
    let transition = engine.transition().unwrap().clone();
    let mut preferences = before.preferences.clone();
    preferences.ui_theme = UiTheme::Light;
    preferences.program_color = [4, 5, 6];
    let command = Command::ApplySettings {
        name: "Evening show".into(),
        preferences: preferences.clone(),
    };
    assert_eq!(command.definition().id, "project.settings.apply");
    assert!(command.definition().idempotent);
    assert!(!command.definition().live);
    engine.execute(command, 250).unwrap();
    assert_eq!(engine.project().revision, before.revision + 1);
    assert_eq!(engine.project().name, "Evening show");
    assert_eq!(engine.project().preferences, preferences);
    assert_eq!(engine.project().program, before.program);
    assert_eq!(engine.project().preview, before.preview);
    let active = engine.transition().unwrap();
    assert_eq!(active.kind, transition.kind);
    assert_eq!(active.from, transition.from);
    assert_eq!(active.to, transition.to);
    assert_eq!(active.start_ms, transition.start_ms);
    assert_eq!(active.progress, transition.progress);
}

#[test]
fn combined_settings_revision_overflow_has_no_partial_commit() {
    let mut project = ProjectState::demo();
    project.revision = u64::MAX;
    let mut engine = Engine::from_project(project).unwrap();
    let before = engine.project().clone();
    let mut preferences = before.preferences.clone();
    preferences.ui_theme = UiTheme::Light;
    assert!(
        engine
            .execute(
                Command::ApplySettings {
                    name: "Overflow attempt".into(),
                    preferences
                },
                0
            )
            .is_err()
    );
    assert_eq!(engine.project(), &before);
    assert!(engine.transition().is_none());
}
