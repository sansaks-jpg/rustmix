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
