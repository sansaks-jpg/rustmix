//! Opt-in release-mode measurement of the full-HD reference compositor.
//! Run with: cargo test -p rustmix-engine --release --test render_perf -- --ignored --nocapture
#![allow(clippy::unwrap_used)]
use rustmix_engine::{render, *};
use std::time::{Duration, Instant};

fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = ((sorted.len() - 1) as f64 * fraction).round() as usize;
    sorted.get(index).copied().unwrap_or(0.0)
}

fn measure(mut frame: impl FnMut(), iterations: usize) -> Vec<f64> {
    for _ in 0..5 {
        frame();
    }
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        frame();
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    samples.sort_by(f64::total_cmp);
    samples
}

fn summary(name: &str, samples: &[f64]) {
    println!(
        "{name}: p50={:.2}ms p95={:.2}ms p99={:.2}ms max={:.2}ms",
        percentile(samples, 0.50),
        percentile(samples, 0.95),
        percentile(samples, 0.99),
        samples.last().copied().unwrap_or(0.0)
    );
}

#[test]
#[ignore = "release-mode performance measurement, not a correctness gate"]
fn measure_full_hd_compositor_and_downscale() {
    let mut project = ProjectState::demo();
    project.preferences.width = 1920;
    project.preferences.height = 1080;
    project.preferences.rate = Rate::new(60, 1).unwrap();
    let from = vec![[20_u8, 40, 60, 255]; 1920 * 1080];
    let target = vec![[200_u8, 160, 120, 255]; 1920 * 1080];
    let transition = TransitionState {
        from: InputId(1),
        to: InputId(2),
        kind: TransitionKind::Fade,
        start_ms: 0,
        duration_ms: 1000,
        progress: 0.5,
        manual: false,
    };

    let cut = measure(
        || {
            let _ =
                render::program_from_sources(&project, None, 0.0, 1920, 1080, &from, None).unwrap();
        },
        30,
    );
    summary("cut copy", &cut);

    let fade = measure(
        || {
            let _ = render::program_from_sources(
                &project,
                Some(&transition),
                0.0,
                1920,
                1080,
                &from,
                Some(&target),
            )
            .unwrap();
        },
        30,
    );
    summary("fade", &fade);

    project.overlays.standard[0].enabled = true;
    let overlay = measure(
        || {
            let _ =
                render::program_from_sources(&project, None, 0.0, 1920, 1080, &from, None).unwrap();
        },
        30,
    );
    summary("overlay", &overlay);

    let downscale = measure(
        || {
            let _ = render::downscale(&from, 1920, 1080, 640, 360).unwrap();
        },
        30,
    );
    summary("downscale", &downscale);

    // Keep the process time bounded even on a slow debug runner.
    std::thread::sleep(Duration::from_millis(1));
}
