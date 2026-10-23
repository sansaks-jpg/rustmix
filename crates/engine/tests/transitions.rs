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

// ---- golden curve fixtures -------------------------------------------------

/// Deterministic 8x8 source/target pair for the golden frames.
fn golden_surfaces() -> (Vec<[u8; 4]>, Vec<[u8; 4]>) {
    let from: Vec<_> = (0..64)
        .map(|i| {
            [
                ((i * 3) % 251) as u8,
                ((i * 7 + 11) % 253) as u8,
                ((i * 13) % 249) as u8,
                255,
            ]
        })
        .collect();
    let to: Vec<_> = (0..64)
        .map(|i| {
            [
                ((i * 11 + 3) % 255) as u8,
                ((i * 5 + 90) % 255) as u8,
                ((i * 29 + 17) % 250) as u8,
                255,
            ]
        })
        .collect();
    (from, to)
}

fn golden_frame(kind: TransitionKind, progress: f32) -> Vec<[u8; 4]> {
    let p = ProjectState::demo();
    let (from, to) = golden_surfaces();
    let t = TransitionState {
        from: p.program,
        to: p.preview,
        kind,
        start_ms: 0,
        duration_ms: 1000,
        progress,
        manual: false,
    };
    render::program_from_sources(&p, Some(&t), 0.0, 8, 8, &from, Some(&to)).unwrap()
}

/// FNV-1a over the rendered RGBA bytes; the golden record of one frame.
fn frame_hash(pixels: &[[u8; 4]]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for pixel in pixels {
        for byte in pixel {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

/// Golden curve record per effect: FNV-1a frame hashes at progress
/// 25 %, 50 % and 75 % of the deterministic 8x8 fixture in
/// [`golden_surfaces`]. The endpoints are pinned structurally (they must hash
/// exactly like the untouched source and target surfaces), so the three
/// midpoints are where each effect's easing and direction live. Any change to
/// a curve changes its row here; re-pin only with an intentional, documented
/// curve change.
const GOLDEN_CURVES: [(TransitionKind, [u64; 3]); 20] = [
    (
        TransitionKind::Cut,
        [
            13667097510672658273,
            13667097510672658273,
            13667097510672658273,
        ],
    ),
    (
        TransitionKind::Fade,
        [
            10338775658510959270,
            5207640270878736897,
            3338820306610590117,
        ],
    ),
    (
        TransitionKind::Wipe,
        [
            1151008605619035206,
            13475517136387768395,
            3828939522765298217,
        ],
    ),
    (
        TransitionKind::WipeReverse,
        [6820244222279510678, 541757646102600628, 4724663602808667329],
    ),
    (
        TransitionKind::VerticalWipe,
        [
            11152145129395025459,
            16523125700777360992,
            15789033413341245460,
        ],
    ),
    (
        TransitionKind::VerticalWipeReverse,
        [
            16901832431339403839,
            16027251358329585023,
            15969859398576322888,
        ],
    ),
    (
        TransitionKind::Slide,
        [
            13465718467589726402,
            68237484358947475,
            11779607574331677877,
        ],
    ),
    (
        TransitionKind::SlideReverse,
        [
            9258693325419543470,
            4239310230275583764,
            5318729027358648049,
        ],
    ),
    (
        TransitionKind::VerticalSlide,
        [
            13994690458790821055,
            12008478003580121824,
            11543574791528461032,
        ],
    ),
    (
        TransitionKind::VerticalSlideReverse,
        [
            14417978403499170175,
            12118119042296312447,
            7369801683987524840,
        ],
    ),
    (
        TransitionKind::BarnDoor,
        [
            8711803273088719786,
            13891177940802273102,
            6446065364976466071,
        ],
    ),
    (
        TransitionKind::RollerDoor,
        [
            14928865546290563596,
            8385022384829748390,
            5450834561641683596,
        ],
    ),
    (
        TransitionKind::Zoom,
        [
            4028084434396013838,
            3348561793225773874,
            12137454528071854502,
        ],
    ),
    (
        TransitionKind::Fly,
        [936521757884843415, 2213306679856958954, 7829577180697147676],
    ),
    (
        TransitionKind::CrossZoom,
        [
            12484758494104113326,
            12049832513706136762,
            18156618608771316604,
        ],
    ),
    (
        TransitionKind::FlyRotate,
        [
            1295180033815881740,
            18378314617136000011,
            17586506036648526504,
        ],
    ),
    (
        TransitionKind::Cube,
        [
            17322689883575070358,
            15139411231231667493,
            15988213905078095625,
        ],
    ),
    (
        TransitionKind::CubeZoom,
        [
            10384671045942657286,
            15139411231231667493,
            14632856265920362400,
        ],
    ),
    (
        TransitionKind::Merge,
        [
            17826111920289211068,
            17911688280495688997,
            9075156003718932625,
        ],
    ),
    (
        TransitionKind::AlphaFade,
        [
            14079929587866576466,
            6425255731441348049,
            17718211278466042885,
        ],
    ),
];

/// Progress points the golden table pins, as fractions of the transition.
const GOLDEN_POINTS: [f32; 3] = [0.25, 0.5, 0.75];

#[test]
fn golden_frames_pin_every_effect_curve() {
    let (from, to) = golden_surfaces();
    let (source_hash, target_hash) = (frame_hash(&from), frame_hash(&to));
    // The golden table must cover the whole effect catalogue.
    for kind in TransitionKind::ALL {
        assert!(
            GOLDEN_CURVES.iter().any(|(covered, _)| *covered == kind),
            "{kind:?} has no golden curve"
        );
    }
    for (kind, golden) in GOLDEN_CURVES {
        // 0 % and 100 % are exact endpoints for every effect, so they are
        // pinned against the surfaces themselves rather than magic numbers.
        assert_eq!(
            frame_hash(&golden_frame(kind, 0.0)),
            source_hash,
            "{kind:?} start"
        );
        assert_eq!(
            frame_hash(&golden_frame(kind, 1.0)),
            target_hash,
            "{kind:?} end"
        );
        for (point, expected) in GOLDEN_POINTS.iter().zip(golden) {
            let actual = frame_hash(&golden_frame(kind, *point));
            assert_eq!(
                actual, expected,
                "{kind:?} curve moved at p={point}; re-pin the golden table only for an intentional, documented curve change"
            );
        }
    }
}

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
