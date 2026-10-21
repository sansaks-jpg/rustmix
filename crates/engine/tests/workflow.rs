#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
use rustmix_engine::*;

#[test]
fn complete_operator_workflow_survives_save_and_restart() {
    let mut e = Engine::default();
    e.execute(Command::SelectPreview(InputId(2)), 0).unwrap();
    assert_eq!(e.project().program, InputId(1));
    e.execute(Command::Cut, 0).unwrap();
    assert_eq!(e.project().program, InputId(2));
    assert_eq!(e.project().preview, InputId(1));
    e.execute(Command::SelectPreview(InputId(3)), 0).unwrap();
    e.execute(Command::Transition(TransitionKind::Fade), 0)
        .unwrap();
    e.tick(500);
    assert_eq!(e.project().program, InputId(2));
    assert_eq!(e.transition().unwrap().progress, 0.5);
    e.tick(1000);
    assert_eq!(e.project().program, InputId(3));
    let mut overlay = e.project().overlays.standard[0].clone();
    overlay.enabled = true;
    e.execute(Command::SetOverlay(0, overlay), 1000).unwrap();
    let mut audio = e.project().buses[0].clone();
    audio.gain_db = -9.0;
    e.execute(Command::SetBus(0, audio), 1000).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("show.rustmix.json");
    persistence::save(&path, e.project()).unwrap();
    let restored = persistence::load(&path).unwrap();
    assert_eq!(&restored, e.project());
    assert!(restored.overlays.standard[0].enabled);
    assert_eq!(restored.buses[0].gain_db, -9.0);
}

#[test]
fn invalid_commands_are_atomic_and_on_air_sources_are_protected() {
    let mut e = Engine::default();
    let before = e.project().clone();
    for c in [
        Command::SelectPreview(InputId(99)),
        Command::RemoveInput(InputId(1)),
        Command::SetManualProgress(f32::NAN),
        Command::SetDuration(0),
    ] {
        assert!(e.execute(c, 0).is_err());
        assert_eq!(&before, e.project());
    }
    let mut bad = before.clone();
    bad.inputs[0].audio.gain_db = f32::INFINITY;
    assert!(Engine::from_project(bad).is_err());
}

#[test]
fn manual_transition_is_reversible_and_commits_exactly_once() {
    let mut e = Engine::default();
    e.execute(Command::SetManualProgress(0.4), 0).unwrap();
    assert!(e.execute(Command::SelectPreview(InputId(3)), 0).is_err());
    assert!(e.execute(Command::Cut, 0).is_err());
    e.execute(Command::SetManualProgress(0.0), 1).unwrap();
    assert!(e.transition().is_none());
    assert_eq!(e.project().program, InputId(1));
    e.execute(Command::SetManualProgress(1.0), 2).unwrap();
    assert_eq!(e.project().program, InputId(2));
    assert!(e.transition().is_none());
    e.tick(5000);
    assert_eq!(e.project().program, InputId(2));
}

#[test]
fn fade_black_restore_and_offline_black_are_visible_in_pixels() {
    let mut e = Engine::default();
    let normal = render::program(e.project(), None, 0.0, 16, 9, 0).unwrap();
    e.execute(Command::FadeBlack, 0).unwrap();
    e.tick(1000);
    assert_eq!(e.black_level(), 1.0);
    let black = render::program(e.project(), None, e.black_level(), 16, 9, 1000).unwrap();
    assert!(black.iter().all(|p| *p == [0, 0, 0, 255]));
    assert_ne!(normal, black);
    e.execute(Command::FadeBlack, 1000).unwrap();
    e.tick(2000);
    assert_eq!(e.black_level(), 0.0);
    let mut input = e.project().inputs[0].clone();
    input.online = false;
    e.execute(Command::UpdateInput(input), 2000).unwrap();
    let offline = render::program(e.project(), None, 0.0, 16, 9, 0).unwrap();
    assert!(offline.iter().all(|p| *p == [0, 0, 0, 255]));
}

#[test]
fn corrupt_and_future_projects_do_not_replace_valid_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("show.json");
    let p = ProjectState::demo();
    persistence::save(&path, &p).unwrap();
    let mut bad = p.clone();
    bad.schema = 900;
    assert!(persistence::save(&path, &bad).is_err());
    assert_eq!(persistence::load(&path).unwrap(), p);
    persistence::save(&path, &p).unwrap();
    assert!(persistence::backup_path(&path, 1).exists());
    std::fs::write(&path, b"{broken").unwrap();
    assert!(persistence::load(&path).is_err());
    assert_eq!(
        persistence::load(&persistence::backup_path(&path, 1)).unwrap(),
        p
    );
}

#[test]
fn source_management_routing_and_validation() {
    let mut e = Engine::default();
    e.execute(
        Command::AddInput {
            name: "Extra".into(),
            kind: InputKind::Grid,
        },
        0,
    )
    .unwrap();
    let id = e.project().inputs.last().unwrap().id;
    let mut input = e.project().input(id).unwrap().clone();
    input.name = "Renamed".into();
    input.audio.muted = true;
    input.audio.solo = true;
    input.audio.sends = [false, true, false];
    e.execute(Command::UpdateInput(input.clone()), 0).unwrap();
    e.execute(Command::SetInputAudio(id, input.audio), 0)
        .unwrap();
    assert_eq!(e.project().input(id).unwrap().name, "Renamed");
    assert_eq!(audio::bus_peaks(e.project(), 0, 1000), [0.0, 0.0]);
    e.execute(Command::RemoveInput(id), 0).unwrap();
    assert!(e.project().input(id).is_none());
    assert!(Rate::new(0, 1).is_err());
    assert!(Rate::new(30, 0).is_err());
    assert_eq!(
        Rate::new(60000, 2002).unwrap(),
        Rate::new(30000, 1001).unwrap()
    );
    assert!(render::source(&e.project().inputs[0], 99999, 99999, 0).is_err());
}

#[test]
fn audio_controls_remain_independent_during_transition() {
    let mut e = Engine::default();
    e.execute(Command::Transition(TransitionKind::Wipe), 0)
        .unwrap();
    let mut audio = e.project().inputs[0].audio.clone();
    audio.gain_db = -18.0;
    audio.sends = [true, true, false];
    e.execute(Command::SetInputAudio(InputId(1), audio), 100)
        .unwrap();
    assert_eq!(e.project().inputs[0].audio.gain_db, -18.0);
    assert!(e.transition().is_some());
    e.tick(1000);
    assert_eq!(e.project().program, InputId(2));
}

#[test]
fn stale_source_properties_cannot_reset_audio_routes() {
    let mut e = Engine::default();
    let mut properties = e.project().inputs[0].clone();
    let audio = AudioChannel {
        gain_db: -21.0,
        muted: true,
        solo: false,
        sends: [false, true, true],
    };
    e.execute(Command::SetInputAudio(properties.id, audio.clone()), 0)
        .unwrap();
    properties.name = "Renamed from an older dialog".into();
    e.execute(Command::UpdateInput(properties), 1).unwrap();
    assert_eq!(e.project().inputs[0].audio, audio);
}

#[test]
fn direct_take_is_explicit_atomic_and_rejects_busy_or_missing_input() {
    let mut e = Engine::default();
    let before = e.project().clone();
    assert!(e.execute(Command::TakeInput(InputId(900)), 0).is_err());
    assert_eq!(&before, e.project());
    e.execute(Command::TakeInput(InputId(7)), 0).unwrap();
    assert_eq!(e.project().program, InputId(7));
    assert_eq!(e.project().preview, InputId(1));
    let cue = e.project().clone();
    assert!(e.execute(Command::TakeInput(InputId(7)), 0).is_err());
    assert_eq!(&cue, e.project());
    e.execute(Command::Transition(TransitionKind::Fade), 1)
        .unwrap();
    assert!(e.execute(Command::TakeInput(InputId(2)), 1).is_err());
    assert_eq!(e.project().program, InputId(7));
}

#[test]
fn source_transport_is_independent_and_does_not_pause_transitions() {
    let mut e = Engine::default();
    e.execute(Command::SetInputPaused(InputId(2), false), 0)
        .unwrap();
    e.tick(1250);
    e.execute(Command::SetInputPaused(InputId(1), true), 1250)
        .unwrap();
    e.tick(3000);
    e.execute(Command::SetInputPaused(InputId(2), true), 3000)
        .unwrap();
    e.tick(5000);
    assert_eq!(e.playback(InputId(1)).unwrap().position_ms(), 1250);
    assert_eq!(e.playback(InputId(2)).unwrap().position_ms(), 3000);
    e.execute(Command::SetInputPaused(InputId(1), false), 5000)
        .unwrap();
    e.tick(5200);
    assert_eq!(e.playback(InputId(1)).unwrap().position_ms(), 1450);
    e.execute(Command::SetInputsPaused(true), 5200).unwrap();
    e.execute(Command::Transition(TransitionKind::Fade), 5200)
        .unwrap();
    e.tick(8000);
    assert_eq!(e.project().program, InputId(2));
    assert_eq!(e.playback(InputId(1)).unwrap().position_ms(), 1450);
    e.execute(Command::SetInputsPaused(false), 8000).unwrap();
    e.execute(Command::SetInputPaused(InputId(1), false), 8000)
        .unwrap();
    e.execute(Command::SetInputPaused(InputId(2), true), 8000)
        .unwrap();
    e.tick(8100);
    assert_eq!(e.playback(InputId(1)).unwrap().position_ms(), 1550);
    assert_eq!(e.playback(InputId(2)).unwrap().position_ms(), 3000);
    e.execute(Command::RestartInput(InputId(2)), 8100).unwrap();
    e.tick(8200);
    assert_eq!(e.playback(InputId(2)).unwrap().position_ms(), 0);
    assert!(
        e.execute(Command::RestartInput(InputId(999)), 8200)
            .is_err()
    );
}

#[test]
fn loop_off_holds_last_frame_and_new_inputs_have_their_own_origin() {
    let mut e = Engine::default();
    e.execute(Command::SetInputPaused(InputId(2), false), 0)
        .unwrap();
    e.execute(Command::SetInputLoop(InputId(1), false), 0)
        .unwrap();
    e.tick(65_000);
    assert_eq!(e.playback(InputId(1)).unwrap().position_ms(), 60_000);
    assert_eq!(e.playback(InputId(2)).unwrap().position_ms(), 5000);
    e.execute(
        Command::AddInput {
            name: "Late source".into(),
            kind: InputKind::Grid,
        },
        65_000,
    )
    .unwrap();
    let id = e.project().inputs.last().unwrap().id;
    e.execute(Command::SetInputPaused(id, false), 65_000)
        .unwrap();
    e.tick(65_200);
    assert_eq!(e.playback(id).unwrap().position_ms(), 200);
    e.execute(Command::RemoveInput(id), 65_200).unwrap();
    assert!(e.playback(id).is_none());
}
