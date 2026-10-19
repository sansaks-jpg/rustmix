#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use rustmix_engine::{input::AudioSource, *};

fn wav(path: &str) -> AudioSource {
    AudioSource::WavFile { path: path.into() }
}

#[test]
fn audio_descriptors_validate_local_paths_and_stable_endpoint_ids_without_io() {
    for source in [
        wav("C:/fixtures/voice.WAV"),
        wav("relative/voice.wav"),
        wav(r"D:\recordings\voice.wav"),
        wav("/fixtures/voice.wav"),
        wav(&format!("{}.wav", "x".repeat(4092))),
        AudioSource::Microphone { device_id: None },
        AudioSource::Microphone {
            device_id: Some("{0.0.1.00000000}.{stable-endpoint-id}".into()),
        },
        AudioSource::Microphone {
            device_id: Some("x".repeat(4096)),
        },
    ] {
        source.validate().unwrap();
    }
    assert_eq!(wav("voice.wav").label(), "WAV file");
    assert_eq!(
        AudioSource::Microphone { device_id: None }.label(),
        "Microphone"
    );
    for source in invalid_descriptors() {
        assert!(source.validate().is_err(), "Accepted {source:?}");
    }
}

fn invalid_descriptors() -> Vec<AudioSource> {
    vec![
        wav(""),
        wav("  "),
        wav("voice.mp3"),
        wav("voice.wav\0"),
        wav("voice\n.wav"),
        wav("https://example.invalid/voice.wav"),
        wav("file:///C:/voice.wav"),
        wav("//server/share/voice.wav"),
        wav(r"\\server\share\voice.wav"),
        wav("voice:stream.wav"),
        wav("C:/voice.wav:stream.wav"),
        wav(&format!("{}.wav", "x".repeat(4093))),
        AudioSource::Microphone {
            device_id: Some(String::new()),
        },
        AudioSource::Microphone {
            device_id: Some(" \t".into()),
        },
        AudioSource::Microphone {
            device_id: Some("endpoint\0id".into()),
        },
        AudioSource::Microphone {
            device_id: Some("endpoint\nid".into()),
        },
        AudioSource::Microphone {
            device_id: Some("x".repeat(4097)),
        },
        AudioSource::Microphone {
            device_id: Some("é".repeat(2049)),
        },
    ]
}

#[test]
fn malformed_audio_add_and_apply_leave_the_complete_project_unchanged() {
    let mut engine = Engine::default();
    let before = engine.project().clone();
    let mut invalid_configs: Vec<InputConfig> = invalid_descriptors()
        .into_iter()
        .map(|source| InputConfig {
            audio_source: Some(source),
            ..InputConfig::default()
        })
        .collect();
    // Both valid descriptors still cannot identify a single source together.
    invalid_configs.push(InputConfig {
        media_path: Some("C:/fixtures/logo.png".into()),
        audio_source: Some(wav("C:/fixtures/voice.wav")),
        ..InputConfig::default()
    });
    invalid_configs.push(InputConfig {
        media_path: Some("C:/fixtures/logo.png".into()),
        audio_source: Some(AudioSource::Microphone { device_id: None }),
        ..InputConfig::default()
    });
    for config in invalid_configs {
        assert!(
            engine
                .execute(
                    Command::AddInputAt {
                        name: "Invalid audio".into(),
                        kind: InputKind::Color,
                        config: config.clone(),
                        index: 2,
                    },
                    0,
                )
                .is_err()
        );
        assert_eq!(engine.project(), &before);
        let mut input = before.inputs[2].clone();
        input.name = "This rename must roll back".into();
        input.config = config;
        assert!(
            engine
                .execute(Command::ApplyInput { input, audio: None }, 0)
                .is_err()
        );
        assert_eq!(engine.project(), &before);
    }
}

#[test]
fn wav_and_microphone_identities_round_trip_without_opening_any_source() {
    let directory = tempfile::tempdir().unwrap();
    let missing_wav = directory.path().join("not-created.wav");
    let mut engine = Engine::default();
    let before = engine.project().clone();
    for (offset, source) in [
        AudioSource::WavFile {
            path: missing_wav.to_str().unwrap().into(),
        },
        AudioSource::Microphone { device_id: None },
        AudioSource::Microphone {
            device_id: Some("stable-endpoint-id".into()),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let config = InputConfig {
            audio_source: Some(source),
            ..InputConfig::default()
        };
        let result = engine
            .execute(
                Command::AddInputAt {
                    name: format!("Audio {offset}"),
                    kind: InputKind::Color,
                    config: config.clone(),
                    index: 2 + offset,
                },
                0,
            )
            .unwrap();
        let added = engine.project().input(result.affected[0]).unwrap();
        assert_eq!(added.config, config);
        assert!(added.config.media_path.is_none());
    }
    assert!(
        !missing_wav.exists(),
        "The engine must not create media files"
    );
    assert_eq!(engine.project().program, before.program);
    assert_eq!(engine.project().preview, before.preview);
    assert_eq!(engine.project().buses, before.buses);
    let path = directory.path().join("audio-project.json");
    persistence::save(&path, engine.project()).unwrap();
    let loaded = persistence::load(&path).unwrap();
    assert_eq!(loaded.schema, SCHEMA);
    assert_eq!(&loaded, engine.project());
}

#[test]
fn pre_audio_input_configurations_default_to_no_audio_source() {
    let config = InputConfig {
        media_path: Some("C:/fixtures/logo.png".into()),
        mirror: true,
        ..InputConfig::default()
    };
    let mut legacy_config = serde_json::to_value(&config).unwrap();
    legacy_config
        .as_object_mut()
        .unwrap()
        .remove("audio_source");
    let decoded: InputConfig = serde_json::from_value(legacy_config).unwrap();
    assert_eq!(decoded, config);
    assert!(decoded.audio_source.is_none());

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("old-input-configs.json");
    let project = ProjectState::demo();
    let mut legacy_project = serde_json::to_value(&project).unwrap();
    for input in legacy_project["inputs"].as_array_mut().unwrap() {
        input["config"]
            .as_object_mut()
            .unwrap()
            .remove("audio_source");
    }
    std::fs::write(&path, serde_json::to_vec(&legacy_project).unwrap()).unwrap();
    let loaded = persistence::load(&path).unwrap();
    assert_eq!(loaded.schema, SCHEMA);
    assert_eq!(loaded, project);
    assert!(
        loaded
            .inputs
            .iter()
            .all(|input| input.config.audio_source.is_none())
    );
}
