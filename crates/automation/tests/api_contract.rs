#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
use rustmix_automation::{DispatchError, Telemetry, document, parse_query, server};
use rustmix_engine::{Command, Engine, InputId, ProjectState, TransitionKind};

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
fn engine_cut_matches_the_wire_cut_semantics() {
    let mut engine = Engine::from_project(ProjectState::demo()).unwrap();
    engine.execute(Command::Cut, 0).unwrap();
    assert_eq!(engine.project().program, InputId(2));
    assert_eq!(engine.project().preview, InputId(1));
}
