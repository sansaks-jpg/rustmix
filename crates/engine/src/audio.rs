//! Deterministic meter simulation only. No audio device or samples are produced.
use crate::{AudioChannel, InputState, ProjectState};
pub fn input_peaks(input: &InputState, solo_active: bool, ms: u64) -> [f32; 2] {
    if !input.online || input.audio.muted || (solo_active && !input.audio.solo) {
        return [0.0; 2];
    }
    let phase = (ms % 60000) as f32 / 1000.0 + input.id.0 as f32 * 1.7;
    let gain = 10.0_f32.powf(input.audio.gain_db / 20.0);
    [
        (phase * 2.1).sin().abs() * 0.35 * gain,
        (phase * 1.7 + 0.3).sin().abs() * 0.35 * gain,
    ]
}
pub fn bus_peaks(p: &ProjectState, bus: usize, ms: u64) -> [f32; 2] {
    let Some(a) = p.buses.get(bus) else {
        return [0.0; 2];
    };
    if a.muted || (p.buses.iter().any(|b| b.solo) && !a.solo) {
        return [0.0; 2];
    }
    let solo = p.inputs.iter().any(|i| i.audio.solo);
    let mut result = [0.0; 2];
    for input in &p.inputs {
        if input.audio.sends.get(bus).copied().unwrap_or(false) {
            let peaks = input_peaks(input, solo, ms);
            for (out, v) in result.iter_mut().zip(peaks) {
                *out += v;
            }
        }
    }
    for out in &mut result {
        *out = (*out * 10.0_f32.powf(a.gain_db / 20.0)).min(1.5);
    }
    result
}
pub fn db(peak: f32) -> f32 {
    if peak <= 0.001 {
        -60.0
    } else {
        20.0 * peak.log10()
    }
}
pub fn gain_label(a: &AudioChannel) -> String {
    format!("{:+.1} dB", a.gain_db)
}
