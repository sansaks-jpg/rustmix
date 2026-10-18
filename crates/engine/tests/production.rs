#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use rustmix_engine::*;

#[test]
fn named_playlists_are_bounded_and_active_lists_and_sources_are_protected() {
    let mut e = Engine::default();
    assert!(
        e.execute(Command::StartPlaylist { id: 1, selected: 0 }, 0)
            .is_err()
    );
    e.execute(Command::AddPlaylist("Show rundown".into()), 0)
        .unwrap();
    let before = e.project().clone();
    assert!(
        e.execute(Command::AddPlaylist("show RUNDOWN".into()), 0)
            .is_err()
    );
    assert_eq!(e.project(), &before);
    playlist(&mut e, true, false);
    let mut list = e.project().playlists[0].clone();
    list.begin_selected = true;
    e.execute(Command::UpdatePlaylist(list), 0).unwrap();
    e.execute(Command::StartPlaylist { id: 1, selected: 1 }, 0)
        .unwrap();
    assert_eq!(e.playlist_run().unwrap().index, 1);
    e.tick(500);
    let before = e.project().clone();
    assert!(
        e.execute(Command::UpdatePlaylist(before.playlists[0].clone()), 500)
            .is_err()
    );
    assert!(e.execute(Command::RemovePlaylist(1), 500).is_err());
    assert!(e.execute(Command::RemoveInput(InputId(2)), 500).is_err());
    assert_eq!(e.project(), &before);
    e.execute(Command::StopPlaylist, 500).unwrap();
    // An inactive list retains valid references when an unused source is removed.
    e.execute(Command::RemoveInput(InputId(2)), 500).unwrap();
    assert!(
        e.project().playlists[0]
            .items
            .iter()
            .all(|i| i.input != InputId(2))
    );
    e.execute(Command::RemovePlaylist(1), 500).unwrap();
    assert!(e.execute(Command::RemovePlaylist(2), 500).is_err());
}

fn playlist(engine: &mut Engine, manual: bool, looped: bool) {
    let mut list = engine.project().playlists[0].clone();
    list.manual = manual;
    list.looped = looped;
    list.clear_overlays = true;
    list.items = [InputId(2), InputId(3)]
        .into_iter()
        .map(|input| PlaylistItem {
            input,
            hold_ms: 1000,
            transition: TransitionSlot {
                kind: TransitionKind::Fade,
                duration_ms: 500,
            },
        })
        .collect();
    engine.execute(Command::UpdatePlaylist(list), 0).unwrap();
}

#[test]
fn slots_pin_effect_and_duration_until_the_single_commit() {
    let mut e = Engine::default();
    e.execute(
        Command::SetTransitionSlot(
            1,
            TransitionSlot {
                kind: TransitionKind::SlideReverse,
                duration_ms: 500,
            },
        ),
        0,
    )
    .unwrap();
    e.execute(Command::RunTransitionSlot(1), 10).unwrap();
    e.execute(
        Command::SetTransitionSlot(
            1,
            TransitionSlot {
                kind: TransitionKind::Wipe,
                duration_ms: 2000,
            },
        ),
        100,
    )
    .unwrap();
    assert_eq!(e.transition().unwrap().kind, TransitionKind::SlideReverse);
    e.tick(509);
    assert_eq!(e.project().program, InputId(1));
    e.tick(510);
    assert_eq!(
        (e.project().program, e.project().preview),
        (InputId(2), InputId(1))
    );
    let revision = e.project().revision;
    e.tick(5000);
    assert_eq!(e.project().revision, revision);
    let before = e.project().clone();
    assert!(e.execute(Command::RunTransitionSlot(99), 5000).is_err());
    assert!(
        e.execute(
            Command::SetTransitionSlot(
                0,
                TransitionSlot {
                    kind: TransitionKind::Cut,
                    duration_ms: 0,
                }
            ),
            5000
        )
        .is_err()
    );
    assert_eq!(e.project(), &before);
}

#[test]
fn playlist_holds_after_transition_then_stops_on_last_program() {
    let mut e = Engine::default();
    playlist(&mut e, false, false);
    let mut overlay = e.project().overlays[0].clone();
    overlay.enabled = true;
    e.execute(Command::SetOverlay(0, overlay), 0).unwrap();
    e.execute(Command::StartPlaylist { id: 1, selected: 1 }, 100)
        .unwrap();
    assert!(!e.project().overlays[0].enabled);
    assert_eq!(e.playlist_run().unwrap().index, 0); // begin-selected is off
    assert_eq!(e.project().program, InputId(1));
    e.tick(600);
    assert_eq!(e.project().program, InputId(2));
    e.tick(1599);
    assert!(e.transition().is_none());
    e.tick(1600);
    assert_eq!(e.playlist_run().unwrap().index, 1);
    assert_eq!(e.project().program, InputId(2));
    e.tick(2100);
    assert_eq!(e.project().program, InputId(3));
    e.tick(3100);
    assert!(e.playlist_run().is_none());
    assert_eq!(e.project().program, InputId(3));
}

#[test]
fn manual_loop_stop_and_operator_takeover_are_explicit() {
    let mut e = Engine::default();
    playlist(&mut e, true, true);
    e.execute(Command::StartPlaylist { id: 1, selected: 0 }, 0)
        .unwrap();
    e.tick(60_000);
    assert_eq!(e.project().program, InputId(2));
    assert_eq!(e.playlist_run().unwrap().index, 0);
    e.execute(Command::PlaylistPrevious, 60_000).unwrap();
    e.tick(60_500);
    assert_eq!(e.project().program, InputId(3));
    e.execute(Command::PlaylistNext, 60_500).unwrap();
    e.execute(Command::StopPlaylist, 60_600).unwrap();
    assert!(e.playlist_run().is_none());
    // Stopping automation lets an in-flight take finish rather than tearing Program.
    e.tick(61_000);
    assert_eq!(e.project().program, InputId(2));
    e.execute(Command::StartPlaylist { id: 1, selected: 0 }, 61_000)
        .unwrap();
    e.execute(Command::TakeInput(InputId(4)), 61_000).unwrap();
    assert!(e.playlist_run().is_none());
    e.tick(70_000);
    assert_eq!(e.project().program, InputId(4));
}

#[test]
fn invalid_or_offline_playlist_targets_never_change_preview_or_program() {
    let mut e = Engine::default();
    playlist(&mut e, false, false);
    let mut input = e.project().input(InputId(3)).unwrap().clone();
    input.online = false;
    e.execute(Command::UpdateInput(input), 0).unwrap();
    e.execute(Command::StartPlaylist { id: 1, selected: 0 }, 0)
        .unwrap();
    e.tick(500);
    let buses = (e.project().program, e.project().preview);
    e.tick(1500);
    assert_eq!((e.project().program, e.project().preview), buses);
    assert!(e.playlist_run().is_none());
    assert!(e.playlist_alarm().is_some());
    let before = e.project().clone();
    let mut list = before.playlists[0].clone();
    list.items[0].input = InputId(999);
    assert!(e.execute(Command::UpdatePlaylist(list), 1600).is_err());
    assert_eq!(e.project(), &before);
}

#[test]
fn production_configuration_roundtrips_but_automation_never_restarts_on_load() {
    let mut e = Engine::default();
    playlist(&mut e, false, true);
    e.execute(Command::StartPlaylist { id: 1, selected: 0 }, 0)
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("production.json");
    persistence::save(&path, e.project()).unwrap();
    let loaded = persistence::load(&path).unwrap();
    assert_eq!(&loaded, e.project());
    assert!(
        Engine::from_project(loaded)
            .unwrap()
            .playlist_run()
            .is_none()
    );
    let mut legacy = serde_json::to_value(ProjectState::demo()).unwrap();
    legacy["schema"] = 2.into();
    legacy.as_object_mut().unwrap().remove("playlists");
    legacy.as_object_mut().unwrap().remove("transition_slots");
    legacy["preferences"]
        .as_object_mut()
        .unwrap()
        .remove("key_bindings");
    std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
    let loaded = persistence::load(&path).unwrap();
    assert_eq!(loaded.schema, 3);
    assert!(loaded.playlists[0].items.is_empty());
    assert!(!loaded.preferences.key_bindings.is_empty());
}

#[test]
fn shortcut_conflicts_reserved_keys_and_missing_targets_are_rejected_atomically() {
    let mut e = Engine::default();
    let before = e.project().clone();
    let mut p = before.preferences.clone();
    p.key_bindings.push(p.key_bindings[0].clone());
    assert!(e.execute(Command::SetPreferences(p), 0).is_err());
    let mut p = before.preferences.clone();
    p.key_bindings.push(ShortcutBinding {
        key: "S".into(),
        ctrl: true,
        shift: true,
        alt: false,
        action: ShortcutAction::Cut,
    });
    assert!(e.execute(Command::SetPreferences(p), 0).is_err());
    assert_eq!(e.project(), &before);
    assert!(
        e.shortcut_command(&ShortcutAction::Preview(InputId(999)))
            .is_err()
    );
}
