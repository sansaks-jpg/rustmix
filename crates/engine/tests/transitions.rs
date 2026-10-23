#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use rustmix_engine::*;

fn pixels(kind: TransitionKind, progress: f32) -> Vec<[u8; 4]> {
    let p = ProjectState::demo();
    let t = TransitionState {
        from: p.program,
        to: p.preview,
        kind,
        start_ms: 0,
        duration_ms: 1000,
        progress,
        manual: false,
    };
    let from: Vec<_> = (0..16).map(|n| [n, 0, 0, 255]).collect();
    let to: Vec<_> = (100..116).map(|n| [n, 0, 0, 255]).collect();
    render::program_from_sources(&p, Some(&t), 0.0, 4, 4, &from, Some(&to)).unwrap()
}

#[test]
fn every_supported_effect_preserves_exact_endpoints() {
    let from: Vec<_> = (0..16).map(|n| [n, 0, 0, 255]).collect();
    let to: Vec<_> = (100..116).map(|n| [n, 0, 0, 255]).collect();
    for kind in TransitionKind::ALL {
        assert_eq!(pixels(kind, 0.0), from, "{kind:?} start");
        assert_eq!(pixels(kind, 1.0), to, "{kind:?} end");
    }
}

#[test]
fn directional_geometry_matches_independent_pixel_vectors() {
    let row = |kind| {
        pixels(kind, 0.5)
            .into_iter()
            .take(4)
            .map(|p| p[0])
            .collect::<Vec<_>>()
    };
    assert_eq!(row(TransitionKind::Wipe), [100, 101, 2, 3]);
    assert_eq!(row(TransitionKind::WipeReverse), [0, 1, 102, 103]);
    assert_eq!(row(TransitionKind::Slide), [2, 3, 100, 101]);
    assert_eq!(row(TransitionKind::SlideReverse), [102, 103, 0, 1]);
    assert_eq!(row(TransitionKind::BarnDoor), [0, 101, 102, 3]);
    let column = |kind| {
        pixels(kind, 0.5)
            .iter()
            .step_by(4)
            .map(|p| p[0])
            .collect::<Vec<_>>()
    };
    assert_eq!(column(TransitionKind::VerticalWipe), [100, 104, 8, 12]);
    assert_eq!(
        column(TransitionKind::VerticalWipeReverse),
        [0, 4, 108, 112]
    );
    assert_eq!(column(TransitionKind::VerticalSlide), [8, 12, 100, 104]);
    assert_eq!(
        column(TransitionKind::VerticalSlideReverse),
        [108, 112, 0, 4]
    );
    assert_eq!(column(TransitionKind::RollerDoor), [0, 104, 108, 12]);
}

const SCALED_EFFECTS: [TransitionKind; 8] = [
    TransitionKind::Zoom,
    TransitionKind::Fly,
    TransitionKind::CrossZoom,
    TransitionKind::FlyRotate,
    TransitionKind::Cube,
    TransitionKind::CubeZoom,
    TransitionKind::Merge,
    TransitionKind::AlphaFade,
];

#[test]
fn scaled_effects_blend_between_endpoints_and_stay_distinct() {
    let from: Vec<_> = (0..16).map(|n| [n, 0, 0, 255]).collect();
    let to: Vec<_> = (100..116).map(|n| [n, 0, 0, 255]).collect();
    let fade_mid = pixels(TransitionKind::Fade, 0.5);
    for kind in SCALED_EFFECTS {
        let mid = pixels(kind, 0.5);
        assert_ne!(mid, from, "{kind:?} midpoint never left the source");
        assert_ne!(mid, to, "{kind:?} midpoint jumped to the target");
        assert_ne!(
            pixels(kind, 0.25),
            pixels(kind, 0.75),
            "{kind:?} shows no progression"
        );
        // AlphaFade on a fully opaque target is a plain fade by definition;
        // its distinct behaviour is covered by the alpha-coverage test below.
        if !matches!(
            kind,
            TransitionKind::Cube | TransitionKind::CubeZoom | TransitionKind::AlphaFade
        ) {
            assert_ne!(mid, fade_mid, "{kind:?} midpoint equals a plain fade");
        }
    }
}

#[test]
fn cube_family_passes_through_black_at_the_face_edge() {
    // A cube turn is edge-on at the midpoint: the honest frame is black.
    // Either side of it the face foreshortens horizontally, so a row is
    // transformed only from its own source row and never mixes rows, and the
    // visible face keeps shrinking towards the edge-on midpoint.
    let from: Vec<_> = (0..16).map(|n| [n, 0, 0, 255]).collect();
    let to: Vec<_> = (100..116).map(|n| [n, 0, 0, 255]).collect();
    for kind in [TransitionKind::Cube, TransitionKind::CubeZoom] {
        let mid = pixels(kind, 0.5);
        assert!(mid.iter().all(|p| p[..3] == [0, 0, 0]), "{kind:?} midpoint");
        let frame = |progress| {
            let p = ProjectState::demo();
            let t = TransitionState {
                from: p.program,
                to: p.preview,
                kind,
                start_ms: 0,
                duration_ms: 1000,
                progress,
                manual: false,
            };
            render::program_from_sources(&p, Some(&t), 0.0, 4, 4, &from, Some(&to)).unwrap()
        };
        for progress in [0.25, 0.4] {
            for (y, row) in frame(progress).chunks(4).enumerate() {
                let source_row: Vec<_> = from[y * 4..y * 4 + 4].iter().map(|c| c[0]).collect();
                for pixel in row {
                    assert!(
                        pixel[..3] == [0, 0, 0] || source_row.contains(&pixel[0]),
                        "{kind:?} p={progress} row {y} mixes rows"
                    );
                }
            }
        }
        assert!(
            frame(0.25).iter().filter(|p| p[..3] != [0, 0, 0]).count()
                > frame(0.4).iter().filter(|p| p[..3] != [0, 0, 0]).count(),
            "{kind:?} face does not narrow towards the edge"
        );
    }
}

#[test]
fn alpha_fade_follows_the_target_alpha_coverage() {
    let p = ProjectState::demo();
    let from: Vec<_> = (0..16).map(|n| [n, 0, 0, 255]).collect();
    let mut to: Vec<_> = (100..116).map(|n| [n, 0, 0, 255]).collect();
    for pixel in to.iter_mut().step_by(2) {
        pixel[3] = 0;
    }
    let render = |kind, progress| {
        let t = TransitionState {
            from: p.program,
            to: p.preview,
            kind,
            start_ms: 0,
            duration_ms: 1000,
            progress,
            manual: false,
        };
        render::program_from_sources(&p, Some(&t), 0.0, 4, 4, &from, Some(&to)).unwrap()
    };
    let frame = render(TransitionKind::AlphaFade, 0.75);
    for (index, pixel) in frame.iter().enumerate() {
        if index % 2 == 0 {
            assert_eq!(*pixel, from[index], "transparent target stays on source");
        }
    }
    // With full target alpha at the same progress the source is gone.
    let opaque: Vec<_> = to.iter().map(|c| [c[0], c[1], c[2], 255]).collect();
    let t = TransitionState {
        from: p.program,
        to: p.preview,
        kind: TransitionKind::AlphaFade,
        start_ms: 0,
        duration_ms: 1000,
        progress: 0.75,
        manual: false,
    };
    let frame =
        render::program_from_sources(&p, Some(&t), 0.0, 4, 4, &from, Some(&opaque)).unwrap();
    assert!(frame.iter().enumerate().all(|(i, pixel)| *pixel != from[i]));
}

#[test]
fn fresh_sessions_open_with_the_reference_transition_slots() {
    let slots = default_transition_slots();
    assert_eq!(
        slots[0],
        TransitionSlot {
            kind: TransitionKind::Fade,
            duration_ms: 500
        }
    );
    assert_eq!(
        slots[1],
        TransitionSlot {
            kind: TransitionKind::Merge,
            duration_ms: 1000
        }
    );
    assert_eq!(
        slots[2],
        TransitionSlot {
            kind: TransitionKind::Wipe,
            duration_ms: 1000
        }
    );
    assert_eq!(
        slots[3],
        TransitionSlot {
            kind: TransitionKind::Merge,
            duration_ms: 1000
        }
    );
}

#[test]
fn the_catalog_matches_the_reference_effect_set() {
    // The observed reference switcher offers exactly these twenty effects.
    assert_eq!(TransitionKind::ALL.len(), 20);
    for expected in [
        "Cut",
        "Fade",
        "Wipe",
        "Wipe reverse",
        "Vertical wipe",
        "Vertical wipe reverse",
        "Slide",
        "Slide reverse",
        "Vertical slide",
        "Vertical slide reverse",
        "Barn door",
        "Roller door",
        "Zoom",
        "Fly",
        "Cross zoom",
        "Fly rotate",
        "Cube",
        "Cube zoom",
        "Merge",
        "Alpha fade",
    ] {
        assert!(
            TransitionKind::ALL.iter().any(|k| k.label() == expected),
            "missing effect {expected}"
        );
    }
}
