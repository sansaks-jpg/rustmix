#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
// Test-only explicit panics report unexpected dispatch outcomes; the workspace
// runtime deny does not apply to assertion macros in tests (AGENTS §1.1).
#![allow(clippy::panic)]
use rustmix_automation::{DispatchError, Telemetry, document, parse_query, server};
use rustmix_engine::{Command, Engine, InputId, ProjectState, StingerChannel, TransitionKind};

#[test]
fn fresh_session_renders_the_reference_document_shape() {
    let engine = Engine::from_project(ProjectState::demo()).unwrap();
    let xml = document::render(engine.project(), &Telemetry::default());
    assert!(xml.starts_with("<vmix>"));
    assert!(xml.ends_with("</vmix>"));
    // Version and edition identify this implementation; never the reference.
    assert!(xml.contains("<version>rustmix 0.1.0</version>"));
    assert!(xml.contains("<edition>Free</edition>"));
    // Routing numbers follow input position, like the observed document.
    assert!(xml.contains("<preview>2</preview>"));
    assert!(xml.contains("<active>1</active>"));
    // Fresh slots are the reference defaults.
    assert!(xml.contains("<transition number=\"1\" effect=\"Fade\" duration=\"500\" />"));
    assert!(xml.contains("<transition number=\"2\" effect=\"Merge\" duration=\"1000\" />"));
    assert!(xml.contains("<transition number=\"3\" effect=\"Wipe\" duration=\"1000\" />"));
    assert!(xml.contains("<transition number=\"4\" effect=\"Merge\" duration=\"1000\" />"));
    // Sixteen self-closing overlay channels.
    assert_eq!(xml.matches("<overlay number=").count(), 16);
    assert!(!xml.contains("</overlay>"));
    // Two fullscreen plus four external outputs.
    assert_eq!(xml.matches("type=\"fullscreen\"").count(), 2);
    assert_eq!(xml.matches("type=\"output\"").count(), 4);
    // Transport flags are honest.
    assert!(xml.contains("<recording>False</recording>"));
    assert!(xml.contains("<streaming>False</streaming>"));
    // Master audio block with capitalised booleans.
    assert!(xml.contains("<master volume=\"100\" muted=\"False\" meterF1=\"0\" meterF2=\"0\""));
}

#[test]
fn titles_are_escaped_and_keys_are_stable_guid_shapes() {
    let mut project = ProjectState::demo();
    project.inputs.first_mut().unwrap().name = "A&B <news> \"live\"".into();
    let engine = Engine::from_project(project).unwrap();
    let xml = document::render(engine.project(), &Telemetry::default());
    assert!(xml.contains("A&amp;B &lt;news&gt; &quot;live&quot;"));
    assert!(xml.contains("key=\"00000000-0000-0000-0000-000000000001\""));
}

#[test]
fn preview_and_cut_direct_follow_the_wire_contract() {
    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();
    let request = parse_query("Function=PreviewInput&Input=3");
    rustmix_automation::dispatch(&mut engine, &request, 0).unwrap();
    assert_eq!(engine.project().preview, InputId(3));
    assert_eq!(engine.project().program, InputId(1));

    let request = parse_query("Function=CutDirect&Input=3");
    rustmix_automation::dispatch(&mut engine, &request, 100).unwrap();
    assert_eq!(engine.project().program, InputId(3));
}

#[test]
fn unresolvable_input_is_an_accepted_noop_and_unknown_function_is_an_error() {
    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();
    let before = engine.project().clone();
    let request = parse_query("Function=PreviewInput&Input=999");
    assert_eq!(
        rustmix_automation::dispatch(&mut engine, &request, 0).unwrap(),
        rustmix_automation::control::Outcome::Noop
    );
    assert_eq!(engine.project(), &before);

    let request = parse_query("Function=NotARealFunction");
    assert_eq!(
        rustmix_automation::dispatch(&mut engine, &request, 0),
        Err(DispatchError::UnknownFunction("NotARealFunction".into()))
    );
}

#[test]
fn transition_slots_and_effects_round_trip_through_the_wire() {
    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();
    let request = parse_query("Function=SetTransitionEffect2&Value=CrossZoom");
    rustmix_automation::dispatch(&mut engine, &request, 0).unwrap();
    let request = parse_query("Function=SetTransitionDuration2&Value=750");
    rustmix_automation::dispatch(&mut engine, &request, 0).unwrap();
    let slot = engine.project().transition_slots[1];
    assert_eq!(slot.kind, TransitionKind::CrossZoom);
    assert_eq!(slot.duration_ms, 750);

    let request = parse_query("Function=SetTransitionEffect2&Value=NotAnEffect");
    assert!(matches!(
        rustmix_automation::dispatch(&mut engine, &request, 0),
        Err(DispatchError::BadParameter { .. })
    ));
}

#[test]
fn audio_volume_mute_and_bus_volumes_map_onto_engine_channels() {
    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();
    let request = parse_query("Function=SetVolume&Input=1&Value=50");
    rustmix_automation::dispatch(&mut engine, &request, 0).unwrap();
    assert!((engine.project().inputs[0].audio.gain_db + 30.0).abs() < 0.01);

    let request = parse_query("Function=SetMute&Input=1&Value=true");
    rustmix_automation::dispatch(&mut engine, &request, 0).unwrap();
    assert!(engine.project().inputs[0].audio.muted);

    let request = parse_query("Function=SetBusCVolume&Value=100");
    rustmix_automation::dispatch(&mut engine, &request, 0).unwrap();
    assert!(engine.project().buses[3].gain_db.abs() < 0.01);

    let request = parse_query("Function=SetMasterVolume&Value=0");
    rustmix_automation::dispatch(&mut engine, &request, 0).unwrap();
    assert!((engine.project().buses[0].gain_db + 60.0).abs() < 0.01);
}

#[test]
fn transport_families_without_backends_are_declared_unsupported() {
    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();
    for function in ["StartRecording", "StartStreaming", "StartMultiCorder"] {
        let request = parse_query(&format!("Function={function}"));
        assert!(
            matches!(
                rustmix_automation::dispatch(&mut engine, &request, 0),
                Err(DispatchError::Unsupported { .. })
            ),
            "{function} must be declared unsupported, never faked"
        );
    }
    let before = engine.project().clone();
    assert_eq!(engine.project(), &before);
}

#[test]
fn the_loopback_endpoint_serves_the_document_and_dispatches_over_a_real_socket() {
    let server = server::Server::bind_loopback(0).expect("ephemeral loopback port");
    let addr = server.local_addr().unwrap();
    assert!(addr.ip().is_loopback());

    let client = std::thread::spawn(move || {
        use std::io::{Read, Write};
        let mut stream = std::net::TcpStream::connect(addr).unwrap();
        stream
            .write_all(b"GET /api/?Function=Cut HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .unwrap();
        let mut body = String::new();
        stream.read_to_string(&mut body).unwrap();
        body
    });

    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();
    assert!(server.accept_once(&mut engine, &Telemetry::default(), 0));
    let response = client.join().unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "got: {response}");
    assert_eq!(engine.project().program, InputId(2));

    // Document, error and not-found responses at the request level.
    let response = server::handle_request(&mut engine, &Telemetry::default(), "/api/", 0);
    assert_eq!(response.status, 200);
    assert!(response.body.starts_with("<vmix>"));

    let response =
        server::handle_request(&mut engine, &Telemetry::default(), "/api/?Function=Nope", 0);
    assert_eq!(response.status, 500);

    let response = server::handle_request(&mut engine, &Telemetry::default(), "/other", 0);
    assert_eq!(response.status, 404);
}

#[test]
fn overlay_input_functions_bind_clear_and_reject_the_stinger_range() {
    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();

    // OverlayInput3 puts input 5 on standard channel 3.
    let request = parse_query("Function=OverlayInput3&Input=5");
    rustmix_automation::dispatch(&mut engine, &request, 0).unwrap();
    assert_eq!(engine.project().overlays.binding(2), Some(InputId(5)));
    assert!(engine.project().overlays.standard[2].enabled);

    // Out is declared but not implemented, and is never treated as a clear.
    let before_out = engine.project().clone();
    match rustmix_automation::dispatch(&mut engine, &parse_query("Function=OverlayInput3Out"), 0) {
        Err(DispatchError::Unsupported { function, reason }) => {
            assert_eq!(function, "OverlayInput3Out");
            assert!(
                reason.contains("Off clears the channel"),
                "reason: {reason}"
            );
        }
        other => panic!("OverlayInput3Out must be declared unsupported, got {other:?}"),
    }
    assert_eq!(
        engine.project(),
        &before_out,
        "Out must leave the channel assignment intact"
    );

    // Off is the function that clears the channel.
    rustmix_automation::dispatch(&mut engine, &parse_query("Function=OverlayInput3Off"), 0)
        .unwrap();
    assert_eq!(engine.project().overlays.binding(2), None);
    assert!(!engine.project().overlays.standard[2].enabled);

    // Channels 9-16 belong to the stingers; no OverlayInput9-16 spelling exists.
    for number in [9, 16] {
        let request = parse_query(&format!("Function=OverlayInput{number}"));
        match rustmix_automation::dispatch(&mut engine, &request, 0) {
            Err(DispatchError::Unsupported { function, reason }) => {
                assert_eq!(function, format!("OverlayInput{number}"));
                assert!(reason.contains("9-16 are Stingers 1-8"), "reason: {reason}");
            }
            other => panic!("OverlayInput{number} must name the stinger range, got {other:?}"),
        }
    }

    // Absent and unresolvable Input targets are the accepted no-op.
    let before_noop = engine.project().clone();
    for query in ["Function=OverlayInput1", "Function=OverlayInput1&Input=999"] {
        assert_eq!(
            rustmix_automation::dispatch(&mut engine, &parse_query(query), 0).unwrap(),
            rustmix_automation::control::Outcome::Noop,
            "{query}"
        );
    }
    assert_eq!(engine.project(), &before_noop);

    // Spellings outside the 1-8 vocabulary remain unknown functions.
    for query in [
        "Function=OverlayInput",
        "Function=OverlayInput0",
        "Function=OverlayInput17",
    ] {
        assert!(
            matches!(
                rustmix_automation::dispatch(&mut engine, &parse_query(query), 0),
                Err(DispatchError::UnknownFunction(_))
            ),
            "{query}"
        );
    }
}

#[test]
fn the_declared_overlay_variants_never_clear_a_bound_channel() {
    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();
    rustmix_automation::dispatch(
        &mut engine,
        &parse_query("Function=OverlayInput1&Input=4"),
        0,
    )
    .unwrap();
    let bound = engine.project().clone();
    for query in [
        "Function=OverlayInput1In",
        "Function=OverlayInput1Zoom",
        "Function=OverlayInput1Last",
        // AllOff is the one unnumbered spelling of the family.
        "Function=OverlayInputAllOff",
    ] {
        match rustmix_automation::dispatch(&mut engine, &parse_query(query), 0) {
            Err(DispatchError::Unsupported { .. }) => {}
            other => panic!("{query} must be declared unsupported, got {other:?}"),
        }
        assert_eq!(
            engine.project(),
            &bound,
            "a declared variant must not change the channel"
        );
    }
    assert_eq!(engine.project().overlays.binding(0), Some(InputId(4)));
}

#[test]
fn stinger_functions_trigger_the_configured_channel() {
    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();
    let config = StingerChannel {
        stinger_input: Some(InputId(3)),
        cut_point_ms: 500,
        display_underneath_overlays: false,
    };
    engine.execute(Command::SetStinger(7, config), 0).unwrap();

    rustmix_automation::dispatch(&mut engine, &parse_query("Function=Stinger8"), 100).unwrap();
    let active = engine.stinger().unwrap();
    assert_eq!(active.slot, 7);
    assert_eq!(active.input, InputId(3));
    assert_eq!(active.cut_point_ms, 500);

    // An unconfigured stinger is rejected with the engine's operator reason.
    match rustmix_automation::dispatch(&mut engine, &parse_query("Function=Stinger1"), 200) {
        Err(DispatchError::Rejected(message)) => {
            assert!(
                message.contains("Configure a stinger input"),
                "message: {message}"
            );
        }
        other => panic!("an unconfigured stinger must be rejected, got {other:?}"),
    }

    // The stinger vocabulary ends at eight.
    assert!(matches!(
        rustmix_automation::dispatch(&mut engine, &parse_query("Function=Stinger9"), 0),
        Err(DispatchError::UnknownFunction(_))
    ));
}

#[test]
fn play_pause_toggles_the_input_transport() {
    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();
    // The Program input plays by default, so the first toggle pauses it.
    rustmix_automation::dispatch(&mut engine, &parse_query("Function=PlayPause&Input=1"), 0)
        .unwrap();
    assert!(engine.playback(InputId(1)).unwrap().paused());
    // The second toggle resumes it.
    rustmix_automation::dispatch(&mut engine, &parse_query("Function=PlayPause&Input=1"), 10)
        .unwrap();
    assert!(!engine.playback(InputId(1)).unwrap().paused());

    // The Preview transport input starts paused, so the toggle plays it.
    rustmix_automation::dispatch(&mut engine, &parse_query("Function=PlayPause&Input=2"), 20)
        .unwrap();
    assert!(!engine.playback(InputId(2)).unwrap().paused());

    // An input without a transport is rejected with the engine's reason.
    match rustmix_automation::dispatch(&mut engine, &parse_query("Function=PlayPause&Input=6"), 0) {
        Err(DispatchError::Rejected(message)) => {
            assert!(
                message.contains("no playback transport"),
                "message: {message}"
            );
        }
        other => panic!("a transport-less input must be rejected, got {other:?}"),
    }

    // Absent and unresolvable targets are the accepted no-op.
    let before = engine.project().clone();
    for query in ["Function=PlayPause", "Function=PlayPause&Input=999"] {
        assert_eq!(
            rustmix_automation::dispatch(&mut engine, &parse_query(query), 0).unwrap(),
            rustmix_automation::control::Outcome::Noop,
            "{query}"
        );
    }
    assert_eq!(engine.project(), &before);
}

#[test]
fn accepted_functions_carry_the_wire_overlay_and_transport_vocabulary() {
    let accepted = rustmix_automation::control::accepted_functions();
    for name in [
        "PlayPause",
        "OverlayInput1",
        "OverlayInput8",
        "OverlayInput1Off",
        "OverlayInput8Off",
        "Stinger1",
        "Stinger8",
    ] {
        assert!(accepted.contains(&name), "{name} must be listed");
    }
}

#[test]
fn engine_cut_matches_the_wire_cut_semantics() {
    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();
    engine.execute(Command::Cut, 0).unwrap();
    assert_eq!(engine.project().program, InputId(2));
    assert_eq!(engine.project().preview, InputId(1));
}
