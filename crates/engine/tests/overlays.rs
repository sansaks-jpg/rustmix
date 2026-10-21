#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
use rustmix_engine::*;

fn stinger(input: InputId, cut_point_ms: u32) -> StingerChannel {
    StingerChannel {
        stinger_input: Some(input),
        cut_point_ms,
        display_underneath_overlays: true,
    }
}

#[test]
fn stinger_exchange_waits_for_its_cut_point_and_happens_once() {
    let mut e = Engine::default();
    e.execute(Command::SetStinger(0, stinger(InputId(2), 750)), 0)
        .unwrap();
    assert_eq!(e.project().overlays.stingers[0].cut_point_ms, 750);
    e.execute(Command::TriggerStinger(0), 1_000).unwrap();
    // The cut point has not been reached, so the buses are still as they were.
    assert_eq!(e.project().program, InputId(1));
    assert_eq!(e.project().preview, InputId(2));
    let active = e.stinger().unwrap();
    assert_eq!(active.slot, 0);
    assert_eq!(active.input, InputId(2));
    assert!(!active.exchanged);
    e.tick(1_749);
    assert_eq!(e.project().program, InputId(1));
    e.tick(1_750);
    assert_eq!(e.project().program, InputId(2));
    assert_eq!(e.project().preview, InputId(1));
    assert!(e.stinger().unwrap().exchanged);
    // Later ticks never exchange a second time, but the stinger stays
    // trackable until the operator explicitly stops it.
    let revision = e.project().revision;
    e.tick(9_000);
    assert_eq!(e.project().program, InputId(2));
    assert_eq!(e.project().revision, revision);
    assert!(e.stinger().is_some());
    e.execute(Command::StopStinger, 9_000).unwrap();
    assert!(e.stinger().is_none());
}

#[test]
fn zero_cut_point_exchanges_immediately_at_the_trigger() {
    let mut e = Engine::default();
    e.execute(Command::SetStinger(3, stinger(InputId(3), 0)), 0)
        .unwrap();
    e.execute(Command::TriggerStinger(3), 100).unwrap();
    assert_eq!(e.project().program, InputId(2));
    assert_eq!(e.project().preview, InputId(1));
    let active = e.stinger().unwrap();
    assert_eq!(active.slot, 3);
    assert!(active.exchanged);
}

#[test]
fn retrigger_replaces_the_in_flight_stinger() {
    let mut e = Engine::default();
    e.execute(Command::SetStinger(0, stinger(InputId(2), 1_000)), 0)
        .unwrap();
    e.execute(Command::SetStinger(2, stinger(InputId(3), 1_000)), 0)
        .unwrap();
    e.execute(Command::TriggerStinger(0), 100).unwrap();
    e.execute(Command::TriggerStinger(2), 200).unwrap();
    let active = e.stinger().unwrap();
    assert_eq!(active.slot, 2);
    assert_eq!(active.input, InputId(3));
    assert_eq!(active.from, InputId(1));
    assert_eq!(active.to, InputId(2));
    assert_eq!(active.start_ms, 200);
    e.tick(1_200);
    assert_eq!(e.project().program, InputId(2));
    assert_eq!(e.project().preview, InputId(1));
    assert_eq!(e.stinger().unwrap().slot, 2);
}

#[test]
fn overlay_and_stinger_channels_are_validated_atomically() {
    let mut e = Engine::default();
    let mut panel = e.project().overlays.standard[7].clone();
    panel.enabled = true;
    e.execute(Command::SetOverlay(7, panel.clone()), 0).unwrap();
    assert!(e.project().overlays.standard[7].enabled);
    let accepted = e.project().clone();
    for rejected in [
        Command::SetOverlay(8, panel),
        Command::SetStinger(8, StingerChannel::default()),
        Command::SetStinger(
            0,
            StingerChannel {
                stinger_input: Some(InputId(9_999)),
                ..Default::default()
            },
        ),
        Command::SetStinger(
            0,
            StingerChannel {
                stinger_input: Some(InputId(2)),
                cut_point_ms: 60_001,
                ..Default::default()
            },
        ),
        Command::TriggerStinger(0),
        Command::TriggerStinger(8),
    ] {
        assert!(e.execute(rejected, 0).is_err());
    }
    let reason = e
        .enabled(&Command::TriggerStinger(0))
        .expect_err("an unconfigured stinger must be disabled");
    assert!(reason.to_string().contains("Configure a stinger input"));
    assert_eq!(e.project(), &accepted);
    // A configured stinger on a valid channel is accepted and triggerable.
    e.execute(Command::SetStinger(0, stinger(InputId(2), 500)), 0)
        .unwrap();
    assert!(e.execute(Command::TriggerStinger(0), 0).is_ok());
}

#[test]
fn only_standard_channels_paint_and_stinger_configuration_does_not() {
    let mut e = Engine::default();
    let from = vec![[10, 20, 30, 255]; 200 * 100];
    let paint = |e: &Engine| {
        render::program_from_sources(e.project(), None, 0.0, 200, 100, &from, None).unwrap()
    };
    let baseline = paint(&e);
    assert_eq!(baseline, from);
    e.execute(Command::SetStinger(0, stinger(InputId(2), 400)), 0)
        .unwrap();
    assert_eq!(paint(&e), baseline, "configuring a stinger paints nothing");
    let mut panel = e.project().overlays.standard[4].clone();
    panel.enabled = true;
    e.execute(Command::SetOverlay(4, panel), 0).unwrap();
    let painted = paint(&e);
    assert_ne!(painted, baseline, "channel 5 paints through the bank");
    assert_eq!(painted.len(), baseline.len());
}

#[test]
fn f_keys_cover_all_eight_standard_channels() {
    let mut e = Engine::default();
    let bindings = default_key_bindings();
    for slot in 0..8 {
        let key = format!("F{}", slot + 1);
        let action = bindings
            .iter()
            .find(|b| b.key == key)
            .expect("a default F-key binding")
            .action;
        assert_eq!(action, ShortcutAction::ToggleOverlay(slot));
        let command = e.shortcut_command(&action).unwrap();
        e.execute(command, 0).unwrap();
        assert!(
            e.project().overlays.standard[slot].enabled,
            "channel {} toggles from {key}",
            slot + 1
        );
    }
    assert!(
        e.shortcut_command(&ShortcutAction::ToggleOverlay(8))
            .is_err()
    );
    assert!(bindings.iter().all(|b| !matches!(
        b.action,
        ShortcutAction::ToggleOverlay(slot) if slot >= 8
    )));
}

#[test]
fn toggle_overlay_targets_and_toggles_preview_input() {
    let mut e = Engine::default();
    assert_eq!(e.project().preview, InputId(2));
    assert!(!e.project().overlays.standard[0].enabled);

    let cmd = e.shortcut_command(&ShortcutAction::ToggleOverlay(0)).unwrap();
    e.execute(cmd, 0).unwrap();
    let o = &e.project().overlays.standard[0];
    assert!(o.enabled);
    assert_eq!(o.title, "Camera B");

    let cmd = e.shortcut_command(&ShortcutAction::ToggleOverlay(0)).unwrap();
    e.execute(cmd, 0).unwrap();
    assert!(!e.project().overlays.standard[0].enabled);

    e.execute(Command::SelectPreview(InputId(3)), 0).unwrap();
    let cmd = e.shortcut_command(&ShortcutAction::ToggleOverlay(0)).unwrap();
    e.execute(cmd, 0).unwrap();
    let o = &e.project().overlays.standard[0];
    assert!(o.enabled);
    assert_eq!(o.title, "Camera C");
}

#[test]
fn schema_four_migrates_title_panels_into_the_channel_bank() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy-overlays.json");
    let mut value = serde_json::to_value(ProjectState::demo()).unwrap();
    value["schema"] = 4.into();
    value["overlays"] = serde_json::json!([
        {"enabled": true, "title": "Legacy One", "subtitle": "Kept intact", "position": "LowerRight"},
        {"enabled": false, "title": "Legacy Two", "subtitle": "Also kept", "position": "TopLeft"},
        {"enabled": true, "title": "Legacy Three", "subtitle": "Kept as well", "position": "LowerLeft"},
        {"enabled": false, "title": "Legacy Four", "subtitle": "Last one", "position": "LowerLeft"}
    ]);
    let original = serde_json::to_vec(&value).unwrap();
    std::fs::write(&path, &original).unwrap();
    let migrated = persistence::load(&path).unwrap();
    assert_eq!(migrated.schema, SCHEMA);
    assert_eq!(migrated.overlays.standard[0].title, "Legacy One");
    assert!(migrated.overlays.standard[0].enabled);
    assert_eq!(migrated.overlays.standard[0].position, Position::LowerRight);
    assert_eq!(migrated.overlays.standard[1].title, "Legacy Two");
    assert_eq!(migrated.overlays.standard[2].title, "Legacy Three");
    assert_eq!(migrated.overlays.standard[3].subtitle, "Last one");
    // Channels 5–8 keep their fresh defaults and every stinger stays idle.
    assert_eq!(migrated.overlays.standard[4].title, "Overlay 5");
    assert!(!migrated.overlays.standard[4].enabled);
    assert!(
        migrated
            .overlays
            .stingers
            .iter()
            .all(|s| s.stinger_input.is_none() && s.cut_point_ms == 0)
    );
    // Saving migrates the live file and keeps the original rollback bytes.
    persistence::save(&path, &migrated).unwrap();
    assert_eq!(
        std::fs::read(persistence::backup_path(&path, 1)).unwrap(),
        original
    );
    assert_eq!(persistence::load(&path).unwrap(), migrated);
}

#[test]
fn removing_an_input_clears_stinger_references_and_blocks_in_flight_stingers() {
    let mut e = Engine::default();
    e.execute(Command::SetStinger(1, stinger(InputId(7), 500)), 0)
        .unwrap();
    e.execute(Command::TriggerStinger(1), 0).unwrap();
    assert!(e.execute(Command::RemoveInput(InputId(7)), 100).is_err());
    e.execute(Command::StopStinger, 100).unwrap();
    e.execute(Command::RemoveInput(InputId(7)), 100).unwrap();
    assert_eq!(e.project().overlays.stingers[1].stinger_input, None);
    e.project().validate().unwrap();
}
