#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use rustmix_engine::*;

fn sample_config() -> ExternalOutputConfig {
    ExternalOutputConfig {
        frame_rate: Rate {
            numerator: 60000,
            denominator: 1001,
        },
        width: 1280,
        height: 720,
        device: "Test device".into(),
        port: Some(2),
        audio_device: "Studio out".into(),
        audio_bus: Some(2),
        alpha_channel: true,
        audio_delay_ms: 40,
    }
}

#[test]
fn external_output_routing_is_command_driven_and_atomic() {
    let mut engine = Engine::default();
    let revision = engine.project().revision;
    let command = Command::SetOutputSource(1, OutputSource::Preview);
    let definition = command.definition();
    assert_eq!(definition.id, "output.route.set");
    assert_eq!(definition.category, "Outputs");
    assert!(definition.idempotent && !definition.live);
    engine.execute(command, 0).unwrap();
    assert_eq!(engine.project().revision, revision + 1);
    assert_eq!(
        engine.project().outputs.slot(1).unwrap().source,
        OutputSource::Preview
    );
    assert_eq!(
        engine.project().outputs.slot(2).unwrap().source,
        OutputSource::None
    );

    // A missing source input is rejected without changing anything.
    let before = engine.project().clone();
    assert!(
        engine
            .execute(
                Command::SetOutputSource(2, OutputSource::Input(InputId(999))),
                0
            )
            .is_err()
    );
    assert_eq!(engine.project(), &before);

    // Out-of-range slots are rejected.
    assert!(
        engine
            .enabled(&Command::SetOutputSource(5, OutputSource::Program))
            .is_err()
    );
    assert!(
        engine
            .enabled(&Command::SetOutputSource(0, OutputSource::Program))
            .is_err()
    );

    // Input-sourced routing works and removing the input clears the route.
    engine
        .execute(
            Command::SetOutputSource(3, OutputSource::Input(InputId(4))),
            0,
        )
        .unwrap();
    assert!(engine.project().outputs.any_references(InputId(4)));
    engine.execute(Command::RemoveInput(InputId(4)), 0).unwrap();
    assert!(!engine.project().outputs.any_references(InputId(4)));
    assert_eq!(
        engine.project().outputs.slot(3).unwrap().source,
        OutputSource::None
    );
}

#[test]
fn external_output_configuration_validates_bounds_and_buses() {
    let mut engine = Engine::default();
    let config = sample_config();
    let command = Command::SetExternalOutputConfig(4, config.clone());
    let definition = command.definition();
    assert_eq!(definition.id, "output.external.configure");
    assert_eq!(definition.category, "Outputs");
    assert!(definition.idempotent && !definition.live);
    engine.execute(command, 0).unwrap();
    assert_eq!(engine.project().outputs.slot(4).unwrap().config, config);

    let before = engine.project().clone();
    let mut invalid = config.clone();
    invalid.frame_rate.denominator = 0;
    assert!(
        engine
            .enabled(&Command::SetExternalOutputConfig(1, invalid.clone()))
            .is_err()
    );
    invalid = config.clone();
    invalid.audio_bus = Some(3);
    assert!(
        engine
            .enabled(&Command::SetExternalOutputConfig(1, invalid))
            .is_err()
    );
    let mut too_big = config;
    too_big.height = 6_000;
    assert!(
        engine
            .execute(Command::SetExternalOutputConfig(1, too_big), 0)
            .is_err()
    );
    assert!(
        engine
            .enabled(&Command::SetExternalOutputConfig(
                9,
                ExternalOutputConfig::default()
            ))
            .is_err()
    );
    assert_eq!(engine.project(), &before);
    assert_eq!(
        engine.project().outputs.slot(1).unwrap().config,
        ExternalOutputConfig::default()
    );
}

#[test]
fn removing_an_input_clears_only_its_routes_and_keeps_every_configuration() {
    let mut engine = Engine::default();
    for (number, source) in [
        (1, OutputSource::Input(InputId(4))),
        (2, OutputSource::Program),
        (3, OutputSource::Input(InputId(4))),
        (4, OutputSource::Input(InputId(5))),
    ] {
        engine
            .execute(
                Command::SetExternalOutput(ExternalOutput {
                    number,
                    source,
                    config: sample_config(),
                }),
                0,
            )
            .unwrap();
    }
    let before = engine.project().clone();
    engine.execute(Command::RemoveInput(InputId(4)), 0).unwrap();
    for number in 1..=4 {
        let slot = engine.project().outputs.slot(number).unwrap();
        let previous = before.outputs.slot(number).unwrap();
        assert_eq!(slot.config, previous.config);
        assert_eq!(
            slot.source,
            if number == 1 || number == 3 {
                OutputSource::None
            } else {
                previous.source
            }
        );
    }
    assert_eq!(engine.project().program, before.program);
    assert_eq!(engine.project().preview, before.preview);
    assert_eq!(engine.project().buses, before.buses);
    engine.project().validate().unwrap();
}

#[test]
fn external_output_bank_round_trips_and_older_schemas_default_it() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("outputs.json");
    let mut engine = Engine::default();
    engine
        .execute(Command::SetOutputSource(2, OutputSource::Program), 0)
        .unwrap();
    engine
        .execute(Command::SetExternalOutputConfig(2, sample_config()), 0)
        .unwrap();
    persistence::save(&path, engine.project()).unwrap();
    assert_eq!(persistence::load(&path).unwrap(), *engine.project());

    // A schema-5 file has no `outputs` key and must load with the default bank.
    let mut legacy = serde_json::to_value(ProjectState::demo()).unwrap();
    legacy["schema"] = 5.into();
    legacy.as_object_mut().unwrap().remove("outputs");
    let legacy_path = directory.path().join("legacy-outputs.json");
    std::fs::write(&legacy_path, serde_json::to_vec(&legacy).unwrap()).unwrap();
    let migrated = persistence::load(&legacy_path).unwrap();
    assert_eq!(migrated.schema, SCHEMA);
    assert_eq!(migrated.outputs, ExternalOutputBank::default());
}

#[test]
fn applying_a_whole_output_is_one_atomic_revision_and_preserves_live_buses() {
    let mut engine = Engine::default();
    engine
        .execute(Command::Transition(TransitionKind::Fade), 50)
        .unwrap();
    let before = engine.project().clone();
    let transition = engine.transition().cloned();
    let output = ExternalOutput {
        number: 3,
        source: OutputSource::Input(InputId(7)),
        config: sample_config(),
    };
    let command = Command::SetExternalOutput(output.clone());
    let definition = command.definition();
    assert_eq!(definition.id, "output.external.apply");
    assert_eq!(definition.category, "Outputs");
    assert!(definition.idempotent && !definition.live);
    let result = engine.execute(command, 75).unwrap();
    assert_eq!(result.revision, before.revision + 1);
    assert_eq!(engine.project().outputs.slot(3), Some(&output));
    assert_eq!(engine.transition(), transition.as_ref());
    let mut expected = before;
    expected.revision = result.revision;
    expected.outputs = engine.project().outputs.clone();
    assert_eq!(engine.project(), &expected);
}

#[test]
fn invalid_combined_output_never_commits_a_partial_route_or_configuration() {
    let mut engine = Engine::default();
    let before = engine.project().clone();
    let valid = ExternalOutput {
        number: 2,
        source: OutputSource::Program,
        config: sample_config(),
    };
    for output in [
        ExternalOutput {
            number: 0,
            ..valid.clone()
        },
        ExternalOutput {
            number: 5,
            ..valid.clone()
        },
        ExternalOutput {
            source: OutputSource::Input(InputId(999)),
            ..valid.clone()
        },
        ExternalOutput {
            config: ExternalOutputConfig {
                port: Some(0),
                ..sample_config()
            },
            ..valid.clone()
        },
        ExternalOutput {
            config: ExternalOutputConfig {
                audio_bus: Some(3),
                ..sample_config()
            },
            ..valid.clone()
        },
        ExternalOutput {
            config: ExternalOutputConfig {
                width: 0,
                ..sample_config()
            },
            ..valid
        },
    ] {
        assert!(
            engine
                .enabled(&Command::SetExternalOutput(output.clone()))
                .is_err()
        );
        assert!(
            engine
                .execute(Command::SetExternalOutput(output), 0)
                .is_err()
        );
        assert_eq!(engine.project(), &before);
    }
}

#[test]
fn exhausted_revision_rejects_the_whole_output_edit() {
    let mut project = ProjectState::demo();
    project.revision = u64::MAX;
    let mut engine = Engine::from_project(project.clone()).unwrap();
    let output = ExternalOutput {
        number: 1,
        source: OutputSource::Preview,
        config: sample_config(),
    };
    assert!(
        engine
            .execute(Command::SetExternalOutput(output), 0)
            .is_err()
    );
    assert_eq!(engine.project(), &project);
}

#[test]
fn output_validation_covers_config_boundaries_without_cross_field_masking() {
    let mut engine = Engine::default();
    let before = engine.project().clone();
    for config in [
        ExternalOutputConfig {
            frame_rate: Rate {
                numerator: 0,
                denominator: 1,
            },
            ..sample_config()
        },
        ExternalOutputConfig {
            frame_rate: Rate {
                numerator: 121,
                denominator: 1,
            },
            ..sample_config()
        },
        ExternalOutputConfig {
            frame_rate: Rate {
                numerator: 60,
                denominator: 2,
            },
            ..sample_config()
        },
        ExternalOutputConfig {
            frame_rate: Rate {
                numerator: 30,
                denominator: 0,
            },
            ..sample_config()
        },
        ExternalOutputConfig {
            width: 7681,
            ..sample_config()
        },
        ExternalOutputConfig {
            height: 4321,
            ..sample_config()
        },
        ExternalOutputConfig {
            height: 0,
            ..sample_config()
        },
        ExternalOutputConfig {
            port: Some(0),
            ..sample_config()
        },
        ExternalOutputConfig {
            audio_bus: Some(3),
            ..sample_config()
        },
        ExternalOutputConfig {
            audio_delay_ms: 5001,
            ..sample_config()
        },
        ExternalOutputConfig {
            device: "x".repeat(97),
            ..sample_config()
        },
        ExternalOutputConfig {
            audio_device: "line\nfeed".into(),
            ..sample_config()
        },
    ] {
        assert!(
            engine
                .execute(Command::SetExternalOutputConfig(1, config), 0)
                .is_err()
        );
        assert_eq!(engine.project(), &before);
    }
    let largest = ExternalOutputConfig {
        frame_rate: Rate {
            numerator: 120,
            denominator: 1,
        },
        width: 7680,
        height: 4320,
        port: Some(u16::MAX),
        audio_delay_ms: 5000,
        ..sample_config()
    };
    engine
        .execute(Command::SetExternalOutputConfig(1, largest.clone()), 0)
        .unwrap();
    assert_eq!(engine.project().outputs.slot(1).unwrap().config, largest);
}

#[test]
fn every_pre_output_schema_migrates_without_changing_buses_or_rollback_bytes() {
    let directory = tempfile::tempdir().unwrap();
    for schema in 1..=5 {
        let project = ProjectState::demo();
        let mut legacy = serde_json::to_value(&project).unwrap();
        legacy["schema"] = schema.into();
        legacy.as_object_mut().unwrap().remove("outputs");
        if schema < 5 {
            legacy["overlays"] = serde_json::to_value(&project.overlays.standard[..4]).unwrap();
        }
        let original_bytes = serde_json::to_vec(&legacy).unwrap();
        let path = directory.path().join(format!("schema-{schema}.json"));
        std::fs::write(&path, &original_bytes).unwrap();
        let migrated = persistence::load(&path).unwrap();
        assert_eq!(migrated.schema, SCHEMA);
        assert_eq!(migrated.outputs, ExternalOutputBank::default());
        assert_eq!(migrated.program, project.program);
        assert_eq!(migrated.preview, project.preview);
        assert_eq!(migrated.inputs, project.inputs);
        assert_eq!(migrated.buses, project.buses);
        persistence::save(&path, &migrated).unwrap();
        assert_eq!(
            std::fs::read(persistence::backup_path(&path, 1)).unwrap(),
            original_bytes
        );
    }
}

#[test]
fn current_schema_missing_or_corrupt_output_state_is_rejected_without_overwrite() {
    let directory = tempfile::tempdir().unwrap();
    let project = ProjectState::demo();
    let base = serde_json::to_value(&project).unwrap();
    let mut missing = base.clone();
    missing.as_object_mut().unwrap().remove("outputs");
    let mut unknown_input = base.clone();
    unknown_input["outputs"]["slots"][0]["source"] = serde_json::json!({"Input": 999});
    let mut wrong_number = base.clone();
    wrong_number["outputs"]["slots"][1]["number"] = 1.into();
    let mut zero_port = base.clone();
    zero_port["outputs"]["slots"][0]["config"]["port"] = 0.into();
    let mut missing_config = base;
    missing_config["outputs"]["slots"][0]
        .as_object_mut()
        .unwrap()
        .remove("config");
    for (index, corrupt) in [
        missing,
        unknown_input,
        wrong_number,
        zero_port,
        missing_config,
    ]
    .iter()
    .enumerate()
    {
        let path = directory.path().join(format!("corrupt-{index}.json"));
        let bytes = serde_json::to_vec(corrupt).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(persistence::load(&path).is_err());
        assert!(persistence::save(&path, &project).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(!persistence::backup_path(&path, 0).exists());
    }
}
