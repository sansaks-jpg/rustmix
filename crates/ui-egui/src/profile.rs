//! Opt-in hosted measurements, separate from deterministic correctness tests.
use super::*;
use std::hash::{Hash, Hasher};
const RATES: [(u32, u32); 3] = [(50, 1), (60000, 1001), (60, 1)];
#[derive(Default)]
pub(crate) struct Profile {
    phase: usize,
    configured: bool,
    warm: Option<Instant>,
    baseline: Option<(u64, program::VideoStats, u64)>,
    samples: Vec<serde_json::Value>,
}
impl RustMixApp {
    pub(crate) fn profile_step(&mut self, ctx: &egui::Context) {
        let Some(mut profile) = self.profile.take() else {
            return;
        };
        match self.run_profile(&mut profile) {
            Ok(false) => self.profile = Some(profile),
            result => {
                if let Err(error) = result {
                    let _ = std::fs::create_dir_all("artifacts");
                    let _ = std::fs::write("artifacts/profile-error.txt", error.to_string());
                }
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
    fn run_profile(&mut self, profile: &mut Profile) -> Result<bool> {
        if self.start.elapsed() > Duration::from_secs(120) {
            return Err(Error::Invalid(
                "Program measurement exceeded 120 seconds".into(),
            ));
        }
        let Some((n, d)) = RATES.get(profile.phase).copied() else {
            return Ok(true);
        };
        let rate = Rate::new(n, d)?;
        if !profile.configured {
            let mut preferences = self.engine.project().preferences.clone();
            preferences.rate = rate;
            self.execute(Command::SetPreferences(preferences));
            profile.configured = true;
            return Ok(false);
        }
        if self.video_timing.is_none_or(|(_, actual)| actual != rate) || self.video_produced_ns == 0
        {
            return Ok(false);
        }
        let warm = *profile.warm.get_or_insert_with(Instant::now);
        if warm.elapsed() < Duration::from_secs(2) {
            return Ok(false);
        }
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        self.last_program_rgba.hash(&mut hash);
        let fingerprint = hash.finish();
        let (start_ns, baseline, first_pixels) = *profile.baseline.get_or_insert((
            self.video_produced_ns,
            self.video_stats,
            fingerprint,
        ));
        let elapsed_ns = self.video_produced_ns.saturating_sub(start_ns);
        if elapsed_ns < 8_000_000_000 {
            return Ok(false);
        }
        let frames = self.video_stats.rendered.saturating_sub(baseline.rendered);
        let skipped = self.video_stats.skipped.saturating_sub(baseline.skipped);
        let fps = frames as f64 * 1_000_000_000.0 / elapsed_ns as f64;
        let nominal = f64::from(n) / f64::from(d);
        if frames == 0 || first_pixels == fingerprint {
            return Err(Error::Invalid(
                "Program measurement produced no moving frames".into(),
            ));
        }
        profile.samples.push(serde_json::json!({
            "rate_numerator":n,"rate_denominator":d,"raster":[640,360],
            "seconds":elapsed_ns as f64/1e9,"frames_produced":frames,"frames_skipped":skipped,
            "actual_fps":fps,"last_render_ms":self.video_stats.render_ms,"ui_fps":self.fps,
            "cached_surface_bytes_upper_bound":self.video_stats.cached_bytes,
            "moving_pixels":true,"within_sample_target":fps >= nominal*0.995 && skipped as f64/(frames+skipped) as f64 <= 0.001,
        }));
        profile.phase += 1;
        profile.configured = false;
        profile.warm = None;
        profile.baseline = None;
        if profile.phase == RATES.len() {
            std::fs::create_dir_all("artifacts")?;
            std::fs::write(
                "artifacts/program-cadence.json",
                serde_json::to_vec_pretty(&serde_json::json!({
                    "scope":"CPU procedural Program, 640x360, seven inputs, bounded latest frame; not full-HD/hardware/output qualification",
                    "samples":profile.samples,
                }))?,
            )?;
            return Ok(true);
        }
        Ok(false)
    }
}
