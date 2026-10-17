#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("RustMix · Offline Production Studio")
            .with_inner_size([1600.0, 900.0])
            .with_clamp_size_to_monitor_size(false)
            .with_icon(eframe::egui::IconData::default())
            .with_min_inner_size([1100.0, 700.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "RustMix",
        options,
        Box::new(|cc| Ok(Box::new(rustmix_ui::RustMixApp::new(cc)))),
    )
}
