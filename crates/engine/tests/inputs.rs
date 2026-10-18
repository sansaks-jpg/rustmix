#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use rustmix_engine::*;

#[test]
fn input_assets_effects_and_order_survive_atomic_save() {
    let mut e = Engine::default();
    let config = InputConfig {
        media_path: Some("C:/fixtures/Logo.png".into()),
        zoom: 1.4,
        mirror: true,
        pip_source: Some(InputId(3)),
        ..Default::default()
    };
    let result = e
        .execute(
            Command::AddInputAt {
                name: "Logo".into(),
                kind: InputKind::Color,
                config: config.clone(),
                index: 1,
            },
            0,
        )
        .unwrap();
    let id = result.affected[0];
    assert_eq!(e.project().inputs[1].id, id);
    assert_eq!(e.project().input_number(id), 2);
    assert_eq!(e.project().input_number(InputId(2)), 3);
    assert_eq!(e.project().input(id).unwrap().config, config);
    assert_eq!(e.project().program, InputId(1));
    assert_eq!(e.project().preview, InputId(2));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("inputs.json");
    persistence::save(&path, e.project()).unwrap();
    assert_eq!(&persistence::load(&path).unwrap(), e.project());
}

#[test]
fn invalid_input_edits_are_atomic_and_cycles_are_rejected() {
    let mut e = Engine::default();
    let mut first = e.project().inputs[0].clone();
    first.config.pip_source = Some(InputId(2));
    e.execute(Command::UpdateInput(first), 0).unwrap();
    let before = e.project().clone();
    for config in [
        InputConfig {
            zoom: f32::NAN,
            ..Default::default()
        },
        InputConfig {
            media_path: Some("movie.mp4".into()),
            ..Default::default()
        },
        InputConfig {
            pip_source: Some(InputId(99)),
            ..Default::default()
        },
        InputConfig {
            pip_source: Some(InputId(1)),
            ..Default::default()
        },
    ] {
        let mut input = before.inputs[1].clone();
        input.config = config;
        assert!(
            e.execute(Command::ApplyInput { input, audio: None }, 0)
                .is_err()
        );
        assert_eq!(e.project(), &before);
    }
    assert!(
        e.execute(
            Command::AddInputAt {
                name: "Invalid position".into(),
                kind: InputKind::Grid,
                config: InputConfig::default(),
                index: 999,
            },
            0
        )
        .is_err()
    );
    assert_eq!(e.project(), &before);
}

#[test]
fn duplication_copies_the_source_and_effects_with_safe_audio() {
    let mut e = Engine::default();
    let mut original = e.project().inputs[0].clone();
    original.name = "é".repeat(32);
    original.config.media_path = Some("C:/fixtures/photo.png".into());
    original.config.saturation = 0.2;
    e.execute(Command::UpdateInput(original.clone()), 0)
        .unwrap();
    let result = e
        .execute(Command::DuplicateInput(original.id), 250)
        .unwrap();
    let copy = e.project().input(result.affected[0]).unwrap();
    assert_eq!(copy.config, original.config);
    assert!(copy.name.len() <= 64 && copy.name.ends_with(" (Copy)"));
    assert!(copy.audio.muted && !copy.audio.solo);
    assert_eq!(e.project().inputs[1].id, copy.id);
    assert_eq!(e.playback(copy.id).unwrap().position_ms(), 0);
}

#[test]
fn removal_clears_persisted_layer_references() {
    let mut e = Engine::default();
    let mut input = e.project().inputs[0].clone();
    input.config.pip_source = Some(InputId(7));
    e.execute(Command::UpdateInput(input), 0).unwrap();
    let removed = e.execute(Command::RemoveInput(InputId(7)), 0).unwrap();
    assert!(removed.affected.contains(&InputId(1)) && removed.affected.contains(&InputId(7)));
    assert!(e.project().inputs[0].config.pip_source.is_none());
    e.project().validate().unwrap();
}

#[test]
fn source_apply_preserves_concurrent_audio_unless_audio_was_edited() {
    let mut e = Engine::default();
    let mut draft = e.project().inputs[0].clone();
    let mut audio = draft.audio.clone();
    audio.gain_db = -12.0;
    e.execute(Command::SetInputAudio(draft.id, audio.clone()), 0)
        .unwrap();
    draft.config.mirror = true;
    e.execute(
        Command::ApplyInput {
            input: draft.clone(),
            audio: None,
        },
        0,
    )
    .unwrap();
    assert_eq!(e.project().inputs[0].audio, audio);
    audio.gain_db = -6.0;
    e.execute(
        Command::ApplyInput {
            input: draft,
            audio: Some(audio.clone()),
        },
        0,
    )
    .unwrap();
    assert_eq!(e.project().inputs[0].audio, audio);
}

#[test]
fn schema_one_migrates_and_future_files_remain_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.json");
    let mut value = serde_json::to_value(ProjectState::demo()).unwrap();
    value["schema"] = 1.into();
    for input in value["inputs"].as_array_mut().unwrap() {
        input.as_object_mut().unwrap().remove("config");
    }
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let migrated = persistence::load(&path).unwrap();
    assert_eq!(migrated.schema, 5);
    assert!(
        migrated
            .inputs
            .iter()
            .all(|i| i.config == InputConfig::default())
    );
    value["schema"] = 999.into();
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(persistence::load(&path).is_err());
}

#[test]
fn migrating_save_keeps_the_original_schema_one_backup_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old.json");
    let mut legacy = serde_json::to_value(ProjectState::demo()).unwrap();
    legacy["schema"] = 1.into();
    for input in legacy["inputs"].as_array_mut().unwrap() {
        input.as_object_mut().unwrap().remove("config");
    }
    let original = serde_json::to_vec(&legacy).unwrap();
    std::fs::write(&path, &original).unwrap();
    let migrated = persistence::load(&path).unwrap();
    persistence::save(&path, &migrated).unwrap();
    assert_eq!(
        std::fs::read(persistence::backup_path(&path, 1)).unwrap(),
        original
    );
    assert_eq!(persistence::load(&path).unwrap().schema, 5);
    persistence::save(&path, &migrated).unwrap();
    assert_eq!(
        std::fs::read(persistence::backup_path(&path, 2)).unwrap(),
        original
    );
}

#[test]
fn loaded_project_origins_and_reusable_surfaces_do_not_accumulate_old_time_or_buffers() {
    let mut engine = Engine::from_project_at(ProjectState::demo(), 50_000).unwrap();
    engine.tick(50_000);
    assert_eq!(engine.playback(InputId(1)).unwrap().position_ms(), 0);
    engine.tick(50_100);
    assert_eq!(engine.playback(InputId(1)).unwrap().position_ms(), 100);
    let input = &engine.project().inputs[0];
    let mut pixels = Vec::new();
    render::source_into(input, 64, 36, 0, &mut pixels).unwrap();
    let pointer = pixels.as_ptr();
    for ms in 0..100 {
        render::source_into(input, 64, 36, ms, &mut pixels).unwrap();
        assert_eq!(pixels.as_ptr(), pointer);
    }
    let before = pixels.clone();
    assert!(render::source_into(input, 0, 36, 0, &mut pixels).is_err());
    assert_eq!(pixels, before);
}
