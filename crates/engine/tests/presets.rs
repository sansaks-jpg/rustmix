#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use rustmix_engine::{video_format::*, *};

#[test]
fn blank_presets_apply_every_canvas_fractional_and_scan_format_without_demo_media() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("preset.json");
    for (width, height) in CANVAS_SIZES {
        for (rate, scan) in rate_options() {
            let format = VideoStandard {
                width,
                height,
                rate,
                scan,
            };
            let expected_monitor = if height > width {
                [360, 640]
            } else {
                [640, 360]
            };
            assert_eq!(format.monitor_size(), expected_monitor);
            let engine = Engine::new_preset(format, 1234).unwrap();
            let project = engine.project();
            assert_eq!(
                project.preferences.video_standard(),
                format,
                "{width}x{height} {rate:?} {scan:?}"
            );
            assert_eq!(project.inputs.len(), 2);
            assert!(
                project
                    .inputs
                    .iter()
                    .all(|i| i.kind == InputKind::Color && i.color == [0; 3])
            );
            assert_ne!(project.program, project.preview);
            assert!(engine.transition().is_none());
            assert!(project.overlays.standard.iter().all(|o| !o.enabled));
            persistence::save(&path, project).unwrap();
            assert_eq!(persistence::load(&path).unwrap(), *project);
        }
    }
}

#[test]
fn full_hd_presets_accept_25_and_60_progressive_frames_per_second() {
    for (numerator, denominator) in [(25, 1), (60, 1)] {
        let format = VideoStandard {
            width: 1920,
            height: 1080,
            rate: Rate::new(numerator, denominator).unwrap(),
            scan: ScanMode::Progressive,
        };
        let engine = Engine::new_preset(format, 0).unwrap();
        assert_eq!(
            engine.project().preferences.video_standard(),
            format,
            "{numerator}/{denominator}"
        );
        assert_eq!(format.monitor_size(), [640, 360]);
    }
}

#[test]
fn invalid_new_format_is_rejected_and_legacy_scan_defaults_to_progressive() {
    let mut format = VideoStandard::default();
    for size in [(0, 0), (1920, u32::MAX), (123, 456)] {
        format.width = size.0;
        format.height = size.1;
        assert!(Engine::new_preset(format, 0).is_err());
    }
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("legacy.json");
    let mut json = serde_json::to_value(ProjectState::demo()).unwrap();
    json["schema"] = 9.into();
    json["preferences"].as_object_mut().unwrap().remove("scan");
    std::fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
    assert_eq!(
        persistence::load(&path).unwrap().preferences.scan,
        ScanMode::Progressive
    );
}

#[test]
fn native_camera_format_and_portrait_browser_roundtrip_and_reject_bad_modes() {
    let mode = CameraFormat {
        width: 1920,
        height: 1080,
        rate: Rate::new(25, 1).unwrap(),
        scan: ScanMode::InterlacedUpperFirst,
        subtype: "3231564E-0000-0010-8000-00AA00389B71".into(),
    };
    assert!(mode.label().contains("50.00i"));
    let mut engine = Engine::default();
    let before = engine.project().clone();
    let mut invalid = mode.clone();
    invalid.subtype = "unknown".into();
    assert!(
        engine
            .execute(
                Command::AddInputAt {
                    name: "Camera".into(),
                    kind: InputKind::Color,
                    config: InputConfig {
                        video_source: Some(VideoSource::Camera {
                            device_id: "stable-id".into(),
                            format: Some(invalid)
                        }),
                        ..Default::default()
                    },
                    index: 0
                },
                0
            )
            .is_err()
    );
    assert_eq!(*engine.project(), before);
    for source in [
        VideoSource::Camera {
            device_id: "stable-id".into(),
            format: Some(mode),
        },
        VideoSource::Browser {
            url: "http://127.0.0.1/".into(),
            width: 1080,
            height: 1920,
        },
    ] {
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
    assert!(
        VideoSource::Browser {
            url: "http://127.0.0.1/".into(),
            width: 1920,
            height: 1920
        }
        .validate()
        .is_err()
    );
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("capture.json");
    persistence::save(&path, engine.project()).unwrap();
    assert_eq!(persistence::load(&path).unwrap(), *engine.project());
    assert_eq!(engine.project().program, before.program);
    assert_eq!(engine.project().preview, before.preview);
}
