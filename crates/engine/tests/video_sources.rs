#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use rustmix_engine::{
    input::{AudioSource, VideoSource},
    video_source::{VideoFeed, VideoFeedError, VideoFrame, VideoFrameMetadata, VideoSourceHealth},
    *,
};
use std::sync::Arc;

fn metadata(sequence: u64, timestamp_100ns: i64) -> VideoFrameMetadata {
    VideoFrameMetadata {
        input_id: InputId(3),
        generation: 7,
        sequence,
        width: 2,
        height: 2,
        timestamp_100ns,
        duration_100ns: Some(333_333),
        discontinuity: false,
    }
}

fn frame(sequence: u64, timestamp_100ns: i64, red: u8) -> VideoFrame {
    VideoFrame::new(
        metadata(sequence, timestamp_100ns),
        Arc::new(vec![[red, 0, 0, 255]; 4]),
    )
    .unwrap()
}

#[test]
fn live_frame_contract_rejects_malformed_rasters_identity_and_time_without_allocating_them() {
    let pixels = Arc::new(vec![[0, 0, 0, 255]; 4]);
    let valid = metadata(0, 0);
    for invalid in [
        VideoFrameMetadata {
            input_id: InputId(0),
            ..valid
        },
        VideoFrameMetadata { width: 0, ..valid },
        VideoFrameMetadata { height: 0, ..valid },
        VideoFrameMetadata {
            width: 4097,
            ..valid
        },
        VideoFrameMetadata {
            height: 2161,
            ..valid
        },
        VideoFrameMetadata {
            width: u32::MAX,
            height: u32::MAX,
            ..valid
        },
        VideoFrameMetadata {
            duration_100ns: Some(0),
            ..valid
        },
        VideoFrameMetadata {
            duration_100ns: Some(-1),
            ..valid
        },
        VideoFrameMetadata {
            timestamp_100ns: i64::MAX,
            duration_100ns: Some(1),
            ..valid
        },
    ] {
        assert!(VideoFrame::new(invalid, Arc::clone(&pixels)).is_err());
    }
    assert!(VideoFrame::new(valid, Arc::new(vec![[0; 4]; 3])).is_err());
    assert!(VideoFrame::new(valid, Arc::new(vec![[0; 4]; 5])).is_err());
    assert!(VideoFeed::new(InputId(0), 7).is_err());
    // Signed decoder preroll remains source metadata, not a cast to huge u64.
    let preroll = VideoFrameMetadata {
        timestamp_100ns: -1_000_000,
        discontinuity: true,
        ..valid
    };
    let preroll = VideoFrame::new(preroll, pixels).unwrap();
    assert_eq!(preroll.metadata().timestamp_100ns, -1_000_000);
}

#[test]
fn latest_publication_is_bounded_and_independent_consumers_never_steal_frames() {
    let feed = VideoFeed::new(InputId(3), 7).unwrap();
    let empty = feed.snapshot().unwrap();
    assert!(empty.frame.is_none());
    assert_eq!(empty.health, VideoSourceHealth::Created);
    assert!(feed.set_health(VideoSourceHealth::Running).is_err());
    feed.publish(frame(0, 0, 10)).unwrap();
    let program = feed.snapshot().unwrap().frame.unwrap();
    let preview = feed.snapshot().unwrap().frame.unwrap();
    assert!(Arc::ptr_eq(&program, &preview));
    assert!(Arc::ptr_eq(
        &program.shared_pixels(),
        &preview.shared_pixels()
    ));
    // Publishing while consumers retain an old frame neither blocks nor changes
    // their pixels; only the feed's latest frame pointer is replaced.
    for sequence in 1..100 {
        feed.publish(frame(
            sequence,
            i64::try_from(sequence).unwrap() * 333_333,
            20,
        ))
        .unwrap();
    }
    let latest = feed.snapshot().unwrap();
    assert_eq!(latest.health, VideoSourceHealth::Running);
    assert_eq!(latest.revision, 100);
    let latest = latest.frame.unwrap();
    assert_eq!(latest.metadata().sequence, 99);
    assert!(!Arc::ptr_eq(&program, &latest));
    assert_eq!(program.metadata().sequence, 0);
    assert_eq!(program.pixels(), &[[10, 0, 0, 255]; 4]);
    assert_eq!(latest.pixels(), &[[20, 0, 0, 255]; 4]);
    assert_eq!(
        feed.snapshot().unwrap().frame.unwrap().metadata().sequence,
        99
    );
}

#[test]
fn obsolete_identity_generation_sequence_and_unmarked_clock_regression_preserve_state() {
    let feed = VideoFeed::new(InputId(3), 7).unwrap();
    feed.publish(frame(4, 1000, 10)).unwrap();
    let before = feed.snapshot().unwrap();
    for meta in [
        VideoFrameMetadata {
            input_id: InputId(4),
            ..metadata(5, 2000)
        },
        VideoFrameMetadata {
            generation: 6,
            ..metadata(5, 2000)
        },
        VideoFrameMetadata {
            generation: 8,
            ..metadata(5, 2000)
        },
        metadata(4, 2000),
        metadata(3, 2000),
        metadata(5, 999),
    ] {
        let rejected = VideoFrame::new(meta, Arc::new(vec![[20, 0, 0, 255]; 4])).unwrap();
        assert!(feed.publish(rejected).is_err());
        let unchanged = feed.snapshot().unwrap();
        assert_eq!(unchanged.revision, before.revision);
        assert_eq!(unchanged.health, before.health);
        assert!(Arc::ptr_eq(
            unchanged.frame.as_ref().unwrap(),
            before.frame.as_ref().unwrap()
        ));
    }
    let seek = VideoFrameMetadata {
        discontinuity: true,
        ..metadata(5, 0)
    };
    feed.publish(VideoFrame::new(seek, Arc::new(vec![[30, 0, 0, 255]; 4])).unwrap())
        .unwrap();
    assert_eq!(feed.snapshot().unwrap().frame.unwrap().metadata(), seek);
    // A newly created input generation cannot accept an old worker's result.
    let replacement = VideoFeed::new(InputId(3), 8).unwrap();
    assert!(matches!(
        replacement.publish(frame(6, 3000, 40)),
        Err(VideoFeedError::WrongGeneration { .. })
    ));
    assert!(replacement.snapshot().unwrap().frame.is_none());
}

#[test]
fn source_failures_are_visible_without_erasing_the_last_valid_frame() {
    let feed = VideoFeed::new(InputId(3), 7).unwrap();
    feed.publish(frame(0, 0, 10)).unwrap();
    let frame = feed.snapshot().unwrap().frame.unwrap();
    for health in [
        VideoSourceHealth::Degraded {
            reason: "Decoder is late".into(),
        },
        VideoSourceHealth::Offline {
            reason: "Source was removed".into(),
        },
        VideoSourceHealth::Failed {
            reason: "Unsupported codec".into(),
        },
        VideoSourceHealth::Stopped,
    ] {
        feed.set_health(health.clone()).unwrap();
        let state = feed.snapshot().unwrap();
        assert_eq!(state.health, health);
        assert!(Arc::ptr_eq(state.frame.as_ref().unwrap(), &frame));
    }
    let revision = feed.snapshot().unwrap().revision;
    for reason in [
        String::new(),
        "  ".into(),
        "bad\nstatus".into(),
        "x".repeat(257),
    ] {
        assert!(
            feed.set_health(VideoSourceHealth::Failed { reason })
                .is_err()
        );
        assert_eq!(feed.snapshot().unwrap().revision, revision);
    }
}

#[test]
fn video_descriptors_validate_without_io_and_invalid_add_or_apply_is_atomic() {
    for path in [
        "C:/fixtures/video.MP4",
        "relative/video.mp4",
        r"D:\media\video.mp4",
    ] {
        VideoSource::File { path: path.into() }.validate().unwrap();
    }
    let longest = VideoSource::File {
        path: format!("{}.mp4", "x".repeat(4092)),
    };
    longest.validate().unwrap();
    assert_eq!(longest.label(), "Video file");
    let mut engine = Engine::default();
    let before = engine.project().clone();
    let mut configurations: Vec<InputConfig> = [
        String::new(),
        "  ".into(),
        "video.wav".into(),
        "video\0.mp4".into(),
        "https://example.invalid/video.mp4".into(),
        "file:///C:/video.mp4".into(),
        "//server/share/video.mp4".into(),
        r"\\server\share\video.mp4".into(),
        "video:stream.mp4".into(),
        "C:/video.mp4:stream.mp4".into(),
        format!("{}.mp4", "x".repeat(4093)),
    ]
    .into_iter()
    .map(|path| InputConfig {
        video_source: Some(VideoSource::File { path }),
        ..InputConfig::default()
    })
    .collect();
    let video = Some(VideoSource::File {
        path: "C:/fixtures/video.mp4".into(),
    });
    configurations.extend([
        InputConfig {
            video_source: video.clone(),
            media_path: Some("logo.png".into()),
            ..InputConfig::default()
        },
        InputConfig {
            video_source: video.clone(),
            audio_source: Some(AudioSource::WavFile {
                path: "voice.wav".into(),
            }),
            ..InputConfig::default()
        },
        InputConfig {
            video_source: video,
            audio_source: Some(AudioSource::Microphone { device_id: None }),
            media_path: Some("logo.png".into()),
            ..InputConfig::default()
        },
    ]);
    for config in configurations {
        assert!(
            engine
                .execute(
                    Command::AddInputAt {
                        name: "Rejected video".into(),
                        kind: InputKind::Color,
                        config: config.clone(),
                        index: 2,
                    },
                    0
                )
                .is_err()
        );
        assert_eq!(engine.project(), &before);
        let mut input = before.inputs[2].clone();
        input.config = config;
        input.name = "Rejected rename".into();
        assert!(
            engine
                .execute(Command::ApplyInput { input, audio: None }, 0)
                .is_err()
        );
        assert_eq!(engine.project(), &before);
    }
}

#[test]
fn video_configuration_round_trips_and_older_inputs_default_to_no_video() {
    let mut engine = Engine::default();
    let before = engine.project().clone();
    let config = InputConfig {
        video_source: Some(VideoSource::File {
            path: "C:/fixtures/not-opened.mp4".into(),
        }),
        ..InputConfig::default()
    };
    let added = engine
        .execute(
            Command::AddInputAt {
                name: "File video".into(),
                kind: InputKind::Color,
                config: config.clone(),
                index: 2,
            },
            0,
        )
        .unwrap();
    assert_eq!(
        engine.project().input(added.affected[0]).unwrap().config,
        config
    );
    assert_eq!(engine.project().program, before.program);
    assert_eq!(engine.project().preview, before.preview);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("video.json");
    persistence::save(&path, engine.project()).unwrap();
    assert_eq!(persistence::load(&path).unwrap(), *engine.project());

    let mut legacy = serde_json::to_value(&before).unwrap();
    legacy["schema"] = SCHEMA.saturating_sub(1).into();
    for input in legacy["inputs"].as_array_mut().unwrap() {
        input["config"]
            .as_object_mut()
            .unwrap()
            .remove("video_source");
    }
    let old_path = directory.path().join("pre-video.json");
    let original_bytes = serde_json::to_vec(&legacy).unwrap();
    std::fs::write(&old_path, &original_bytes).unwrap();
    let migrated = persistence::load(&old_path).unwrap();
    assert_eq!(migrated.schema, SCHEMA);
    assert_eq!(migrated, before);
    assert!(
        migrated
            .inputs
            .iter()
            .all(|input| input.config.video_source.is_none())
    );
    persistence::save(&old_path, &migrated).unwrap();
    assert_eq!(
        std::fs::read(persistence::backup_path(&old_path, 1)).unwrap(),
        original_bytes
    );
}

#[test]
fn new_source_descriptors_validate_and_roundtrip_without_changing_program() {
    let sources = [
        VideoSource::Photos {
            paths: vec!["C:/photos/a.png".into(), "C:/photos/b.jpg".into()],
            interval_ms: 500,
        },
        VideoSource::Browser {
            url: "http://127.0.0.1:8912/graphics".into(),
            width: 1280,
            height: 720,
        },
        VideoSource::Camera {
            device_id: "test-camera-stable-id".into(),
            format: None,
        },
    ];
    let mut engine = Engine::default();
    let program = engine.project().program;
    let preview = engine.project().preview;
    for source in sources {
        source.validate().unwrap();
        engine
            .execute(
                Command::AddInputAt {
                    name: source.label().into(),
                    kind: InputKind::Color,
                    config: InputConfig {
                        video_source: Some(source),
                        ..Default::default()
                    },
                    index: 0,
                },
                0,
            )
            .unwrap();
    }
    for input in engine.project().inputs.iter().take(3) {
        assert_eq!(
            input.source_label(),
            input.config.video_source.as_ref().unwrap().label()
        );
    }
    assert_eq!(engine.project().program, program);
    assert_eq!(engine.project().preview, preview);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sources.json");
    persistence::save(&path, engine.project()).unwrap();
    assert_eq!(persistence::load(&path).unwrap(), *engine.project());
    for source in [
        VideoSource::Photos {
            paths: vec![],
            interval_ms: 1000,
        },
        VideoSource::Photos {
            paths: vec!["a.png".into(); 257],
            interval_ms: 1000,
        },
        VideoSource::Photos {
            paths: vec!["a.png".into()],
            interval_ms: 0,
        },
        VideoSource::Browser {
            url: "http://user:password@example.com".into(),
            width: 640,
            height: 360,
        },
        VideoSource::Browser {
            url: "file:///private.txt".into(),
            width: 640,
            height: 360,
        },
        VideoSource::Browser {
            url: "https://example.com".into(),
            width: u32::MAX,
            height: 360,
        },
        VideoSource::Photos {
            paths: vec!["https://example.com/image.png".into()],
            interval_ms: 1000,
        },
        VideoSource::Photos {
            paths: vec!["C:/photo.png:alternate.png".into()],
            interval_ms: 1000,
        },
        VideoSource::Camera {
            device_id: "".into(),
            format: None,
        },
    ] {
        assert!(source.validate().is_err());
    }
}
