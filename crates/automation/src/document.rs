//! The state document.
//!
//! Reproduces the reference endpoint's element and attribute structure:
//! presence, names, spelling and value forms. Two spelling details are
//! load-bearing and deliberately not prettified:
//!
//! * booleans render as capitalised `True` / `False`;
//! * an empty overlay channel is a self-closing element.
use std::fmt::Write;

use rustmix_engine::{
    AudioSource, ExternalOutput, InputKind, InputState, OutputSource, ProjectState, VideoSource,
    audio_engine::BUS_COUNT,
};

/// Live readings that are telemetry rather than engine state. Defaults render
/// the honest document of an idle session: zero meters, no fade to black.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Telemetry {
    /// Master meter, left channel, 0.0..=1.0.
    pub master_f1: f32,
    /// Master meter, right channel, 0.0..=1.0.
    pub master_f2: f32,
    /// Fade-to-black currently engaged on the operator surface.
    pub fade_to_black: bool,
}

/// Render a boolean the way the state document spells it.
pub fn boolean(v: bool) -> &'static str {
    if v { "True" } else { "False" }
}

/// Map the master fader's dB gain onto the document's 0..=100 volume scale.
fn volume_percent(gain_db: f32) -> i32 {
    ((gain_db.clamp(-60.0, 0.0) + 60.0) * (100.0 / 60.0)).round() as i32
}

/// Escape text for XML content and attribute values.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Deterministic GUID-shaped key for one input. Stable within a session and
/// clearly synthetic — it encodes the stable input ID, never a random value.
fn input_key(id: rustmix_engine::InputId) -> String {
    format!("00000000-0000-0000-0000-{id:012x}", id = id.0)
}

/// rustmix's own source vocabulary for the `type` attribute. Generic media
/// words are used where the meaning matches the reference document exactly;
/// generator kinds use their PascalCase kind name.
fn wire_type(input: &InputState) -> &'static str {
    if let Some(source) = &input.config.audio_source {
        return match source {
            AudioSource::WavFile { .. } => "AudioFile",
            AudioSource::Microphone { .. } => "AudioInput",
        };
    }
    if let Some(source) = &input.config.video_source {
        return match source {
            VideoSource::File { .. } => "Video",
            VideoSource::Photos { .. } => "Photos",
            VideoSource::Browser { .. } => "Browser",
            VideoSource::Camera { .. } => "Capture",
        };
    }
    if input.config.media_path.is_some() {
        return "Image";
    }
    match input.kind {
        InputKind::Grid => "MotionGrid",
        InputKind::Geometry => "Geometry",
        InputKind::Bars => "Colour",
        InputKind::Graphics => "Graphics",
        InputKind::Studio => "VirtualSet",
        InputKind::Color => "Colour",
        InputKind::Countdown => "Countdown",
    }
}

fn output_source_wire(source: &OutputSource) -> String {
    match source {
        OutputSource::None => "Output".into(),
        OutputSource::Program => "Output".into(),
        OutputSource::Preview => "Preview".into(),
        OutputSource::Input(id) => format!("Input {}", id.0),
    }
}

fn external_output_line(output: &ExternalOutput) -> String {
    let mut line = format!(
        "<output type=\"output\" number=\"{}\" source=\"{}\"",
        output.number,
        escape(&output_source_wire(&output.source))
    );
    // The reference document carries ndi="True" only on outputs actually
    // publishing NDI; rustmix publishes none, so the attribute is omitted.
    line.push_str(" />");
    line
}

/// Serialise the complete state document.
pub fn render(project: &ProjectState, telemetry: &Telemetry) -> String {
    let mut x = String::with_capacity(2048);

    x.push_str("<vmix>");
    x.push_str("<version>rustmix 0.1.0</version>");
    x.push_str("<edition>Free</edition>");

    // ---- inputs -----------------------------------------------------------
    x.push_str("<inputs>");
    for (index, input) in project.inputs.iter().enumerate() {
        let number = index + 1;
        let _ = write!(
            x,
            "<input key=\"{}\" number=\"{}\" type=\"{}\" title=\"{}\" shortTitle=\"{}\" \
             state=\"Running\" position=\"0\" duration=\"0\" loop=\"False\">{}</input>",
            input_key(input.id),
            number,
            wire_type(input),
            escape(&input.name),
            escape(&input.name),
            escape(&input.name),
        );
    }
    x.push_str("</inputs>");

    // ---- outputs ----------------------------------------------------------
    x.push_str("<outputs>");
    let _ = write!(
        x,
        "<output type=\"fullscreen\" number=\"1\" source=\"{}\" />",
        output_source_wire(&OutputSource::Program)
    );
    let _ = write!(
        x,
        "<output type=\"fullscreen\" number=\"2\" source=\"{}\" />",
        output_source_wire(&OutputSource::Program)
    );
    for output in project.outputs.iter() {
        x.push_str(&external_output_line(output));
    }
    x.push_str("</outputs>");

    // ---- overlays ---------------------------------------------------------
    // All sixteen channels render empty: the engine's overlay bank is caption
    // panels without input bindings, so an occupied element would be false.
    x.push_str("<overlays>");
    for number in 1..=16 {
        let _ = write!(x, "<overlay number=\"{number}\" />");
    }
    x.push_str("</overlays>");

    // ---- routing ----------------------------------------------------------
    let _ = write!(
        x,
        "<preview>{}</preview>",
        project.input_number(project.preview)
    );
    let _ = write!(
        x,
        "<active>{}</active>",
        project.input_number(project.program)
    );
    let _ = write!(
        x,
        "<fadeToBlack>{}</fadeToBlack>",
        boolean(telemetry.fade_to_black)
    );

    // ---- transitions ------------------------------------------------------
    x.push_str("<transitions>");
    for (index, slot) in project.transition_slots.iter().enumerate() {
        let _ = write!(
            x,
            "<transition number=\"{}\" effect=\"{}\" duration=\"{}\" />",
            index + 1,
            wire_effect(slot.kind),
            slot.duration_ms
        );
    }
    x.push_str("</transitions>");

    // ---- transports -------------------------------------------------------
    // Recording, streaming and multicorder have no backend yet; their flags
    // are honest False values, never optimistic placeholders.
    x.push_str("<recording>False</recording>");
    let _ = write!(
        x,
        "<external>{}</external>",
        boolean(
            project
                .outputs
                .iter()
                .any(|o| o.source != OutputSource::None)
        )
    );
    x.push_str("<streaming>False</streaming>");
    x.push_str("<playList>False</playList>");
    x.push_str("<multiCorder>False</multiCorder>");
    x.push_str("<fullscreen>False</fullscreen>");

    // ---- audio ------------------------------------------------------------
    let master = project.buses.first().cloned().unwrap_or_default();
    let _ = write!(
        x,
        "<audio><master volume=\"{}\" muted=\"{}\" meterF1=\"{}\" meterF2=\"{}\" \
         headphonesVolume=\"100\" /></audio>",
        volume_percent(master.gain_db),
        boolean(master.muted),
        trim_level(telemetry.master_f1),
        trim_level(telemetry.master_f2),
    );

    // ---- dynamic ----------------------------------------------------------
    x.push_str("<dynamic>");
    for number in 1..=4 {
        let _ = write!(x, "<input{number}></input{number}>");
    }
    for number in 1..=4 {
        let _ = write!(x, "<value{number}></value{number}>");
    }
    x.push_str("</dynamic>");

    x.push_str("</vmix>");
    x
}

/// Render a 0.0..=1.0 meter reading the way the document spells a float.
fn trim_level(level: f32) -> String {
    if !level.is_finite() || level <= 0.0 {
        "0".into()
    } else {
        let rounded = (level * 10_000.0).round() / 10_000.0;
        format!("{rounded}")
    }
}

/// The effect spellings of the reference transition catalogue.
pub fn wire_effect(kind: rustmix_engine::TransitionKind) -> &'static str {
    use rustmix_engine::TransitionKind as K;
    match kind {
        K::Cut => "Cut",
        K::Fade => "Fade",
        K::Wipe => "Wipe",
        K::WipeReverse => "WipeReverse",
        K::VerticalWipe => "VerticalWipe",
        K::VerticalWipeReverse => "VerticalWipeReverse",
        K::Slide => "Slide",
        K::SlideReverse => "SlideReverse",
        K::VerticalSlide => "VerticalSlide",
        K::VerticalSlideReverse => "VerticalSlideReverse",
        K::BarnDoor => "BarnDoor",
        K::RollerDoor => "RollerDoor",
        K::Zoom => "Zoom",
        K::Fly => "Fly",
        K::CrossZoom => "CrossZoom",
        K::FlyRotate => "FlyRotate",
        K::Cube => "Cube",
        K::CubeZoom => "CubeZoom",
        K::Merge => "Merge",
        K::AlphaFade => "AlphaFade",
    }
}

/// Reverse mapping for `SetTransitionEffectN` requests.
pub fn effect_from_wire(name: &str) -> Option<rustmix_engine::TransitionKind> {
    use rustmix_engine::TransitionKind as K;
    let kind = match name.trim() {
        "Cut" => K::Cut,
        "Fade" => K::Fade,
        "Wipe" => K::Wipe,
        "WipeReverse" => K::WipeReverse,
        "VerticalWipe" => K::VerticalWipe,
        "VerticalWipeReverse" => K::VerticalWipeReverse,
        "Slide" => K::Slide,
        "SlideReverse" => K::SlideReverse,
        "VerticalSlide" => K::VerticalSlide,
        "VerticalSlideReverse" => K::VerticalSlideReverse,
        "BarnDoor" => K::BarnDoor,
        "RollerDoor" => K::RollerDoor,
        "Zoom" => K::Zoom,
        "Fly" => K::Fly,
        "CrossZoom" => K::CrossZoom,
        "FlyRotate" => K::FlyRotate,
        "Cube" => K::Cube,
        "CubeZoom" => K::CubeZoom,
        "Merge" => K::Merge,
        "AlphaFade" => K::AlphaFade,
        _ => return None,
    };
    Some(kind)
}

/// The bus count the document's mixer implies; kept alongside the render code
/// so a matrix change cannot silently desynchronise the control surface.
pub const DOCUMENT_BUS_COUNT: usize = BUS_COUNT;
