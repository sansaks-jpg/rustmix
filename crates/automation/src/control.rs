//! Control dispatch: `GET /api/?Function=<name>&<param>=<value>` onto typed
//! engine commands.
//!
//! Wire contract, as observed on the analysed product:
//!
//! * an unrecognised function name is an **error**, not a silent no-op;
//! * a well-formed request whose `Input=` target does not resolve in this
//!   session is an accepted no-op (the reference endpoint answers HTTP 200
//!   and leaves state unchanged);
//! * a parameter that is present but malformed is reported.
//!
//! The implemented catalogue is the subset that maps onto real engine
//! behaviour today; transport families with no backend (recording, streaming,
//! multicorder) are declared unsupported with an operator-readable reason
//! instead of pretending to succeed. The overlay vocabulary is implemented
//! against the engine's input-backed standard channels: `OverlayInput1`–`8`
//! bind an input onto the channel, `OverlayInput<n>Off` clears it,
//! `OverlayInput<n>Out` is declared but not implemented (it never clears), and
//! `OverlayInput9`–`16` do not exist because channels 9–16 are `Stinger1`–`8`.
use std::collections::BTreeMap;

use rustmix_engine::{AudioChannel, Command, Engine, InputId, TransitionKind, TransitionSlot};

use crate::document::effect_from_wire;

/// A parsed control request.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Request {
    /// The `Function=` value.
    pub function: String,
    /// All remaining query parameters, in declaration order.
    pub params: BTreeMap<String, String>,
}

impl Request {
    /// Case-insensitive parameter lookup.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }
}

/// Why a control request could not be executed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DispatchError {
    #[error("missing required parameter: {0}")]
    MissingParameter(&'static str),
    #[error("parameter {name} is not valid: {value}")]
    BadParameter { name: &'static str, value: String },
    #[error("unknown function: {0}")]
    UnknownFunction(String),
    #[error("{function} is not implemented: {reason}")]
    Unsupported {
        function: String,
        reason: &'static str,
    },
    #[error("{0}")]
    Rejected(String),
}

/// Parse a query string such as `Function=CutDirect&Input=2`.
pub fn parse_query(query: &str) -> Request {
    let mut req = Request::default();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        let k = percent_decode(k);
        let v = percent_decode(v);
        if k.eq_ignore_ascii_case("Function") {
            req.function = v;
        } else {
            req.params.insert(k, v);
        }
    }
    req
}

/// Minimal percent-decoding, including `+` as a space.
pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(current) = bytes.get(i).copied() {
        match current {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = s.get(i + 1..i + 3).unwrap_or_default();
                match u8::from_str_radix(hex, 16) {
                    Ok(b) => {
                        out.push(b);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(current);
                        i += 1;
                    }
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn required<'a>(req: &'a Request, name: &'static str) -> Result<&'a str, DispatchError> {
    req.get(name).ok_or(DispatchError::MissingParameter(name))
}

/// Resolve an `Input=` parameter against the live session. An absent or
/// unresolvable input number is `Ok(None)` — the observed no-op contract.
fn input_id(engine: &Engine, req: &Request) -> Result<Option<InputId>, DispatchError> {
    let Some(raw) = req.get("Input") else {
        return Ok(None);
    };
    let number: usize = raw
        .trim()
        .parse()
        .map_err(|_| DispatchError::BadParameter {
            name: "Input",
            value: raw.trim().to_string(),
        })?;
    if number == 0 {
        return Err(DispatchError::BadParameter {
            name: "Input",
            value: raw.trim().to_string(),
        });
    }
    Ok(engine
        .project()
        .inputs
        .get(number.wrapping_sub(1))
        .map(|input| input.id))
}

fn u32_value(req: &Request) -> Result<u32, DispatchError> {
    let raw = required(req, "Value")?;
    raw.trim().parse().map_err(|_| DispatchError::BadParameter {
        name: "Value",
        value: raw.trim().to_string(),
    })
}

fn f32_value(req: &Request) -> Result<f32, DispatchError> {
    let raw = required(req, "Value")?;
    raw.trim()
        .parse::<f32>()
        .map_err(|_| DispatchError::BadParameter {
            name: "Value",
            value: raw.trim().to_string(),
        })
        .and_then(|v| {
            v.is_finite()
                .then_some(v)
                .ok_or(DispatchError::BadParameter {
                    name: "Value",
                    value: raw.trim().to_string(),
                })
        })
}

fn bool_value(req: &Request) -> Result<bool, DispatchError> {
    let raw = required(req, "Value")?;
    match raw.trim() {
        "true" | "True" | "1" => Ok(true),
        "false" | "False" | "0" => Ok(false),
        other => Err(DispatchError::BadParameter {
            name: "Value",
            value: other.to_string(),
        }),
    }
}

/// Map the wire's 0..=100 volume scale onto the engine's dB fader, clamped to
/// the engine's validated range so float rounding cannot push it outside.
fn volume_to_db(value: f32) -> f32 {
    ((value.clamp(0.0, 100.0) - 100.0) * 0.6).clamp(-60.0, 12.0)
}

fn unsupported(function: &str, reason: &'static str) -> DispatchError {
    DispatchError::Unsupported {
        function: function.to_string(),
        reason,
    }
}

/// Names accepted by [`dispatch`], for documentation and probing.
pub fn accepted_functions() -> Vec<&'static str> {
    vec![
        "PreviewInput",
        "PreviewInputNext",
        "PreviewInputPrevious",
        "Cut",
        "CutDirect",
        "Transition1",
        "Transition2",
        "Transition3",
        "Transition4",
        "FadeToBlack",
        "SetTransitionEffect1",
        "SetTransitionEffect2",
        "SetTransitionEffect3",
        "SetTransitionEffect4",
        "SetTransitionDuration1",
        "SetTransitionDuration2",
        "SetTransitionDuration3",
        "SetTransitionDuration4",
        "Play",
        "Pause",
        "PlayPause",
        "Restart",
        "SetPosition",
        "LoopOn",
        "LoopOff",
        "RemoveInput",
        "SetInputName",
        "SetCategory",
        "SetVolume",
        "SetMute",
        "SetSolo",
        "SetMasterVolume",
        "SetBusAVolume",
        "SetBusBVolume",
        "SetBusCVolume",
        "SetBusDVolume",
        "SetBusEVolume",
        "SetBusFVolume",
        "SetBusGVolume",
        "SetHeadphonesVolume",
        "OverlayInput1",
        "OverlayInput2",
        "OverlayInput3",
        "OverlayInput4",
        "OverlayInput5",
        "OverlayInput6",
        "OverlayInput7",
        "OverlayInput8",
        "OverlayInput1Off",
        "OverlayInput2Off",
        "OverlayInput3Off",
        "OverlayInput4Off",
        "OverlayInput5Off",
        "OverlayInput6Off",
        "OverlayInput7Off",
        "OverlayInput8Off",
        "Stinger1",
        "Stinger2",
        "Stinger3",
        "Stinger4",
        "Stinger5",
        "Stinger6",
        "Stinger7",
        "Stinger8",
        "StartPlayList",
        "StopPlayList",
    ]
}

/// A recognised `OverlayInput*` wire function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayFunction {
    /// `OverlayInput<n>`: put the request's `Input` on standard channel `n`.
    Bind(usize),
    /// `OverlayInput<n>Off`: clear standard channel `n`.
    Off(usize),
    /// `OverlayInput<n>Out`, `In`, `Zoom` and `Last`: declared by the
    /// reference but not implemented; none of them clears the channel, and
    /// the reason says which function does.
    Variant(&'static str),
    /// `OverlayInputAllOff`: declared; per-channel clearing is implemented.
    AllOff,
    /// `OverlayInput9`–`OverlayInput16`: those channels are Stingers 1–8, so
    /// no such overlay spelling exists in the reference vocabulary.
    StingerChannel,
}

/// Parse the `OverlayInput*` wire family.
///
/// The executable vocabulary is exactly `OverlayInput1`–`OverlayInput8` with
/// the `Off` suffix. The sibling spellings the reference declares (`Out`,
/// `In`, `Zoom`, `Last`, `AllOff`) parse to declared-not-implemented
/// rejections rather than unknown-function errors, and channels 9–16 belong
/// to the stingers. Anything else is not an overlay function and falls
/// through to the unknown-function arm.
fn overlay_function(name: &str) -> Option<OverlayFunction> {
    let suffix = name.strip_prefix("OverlayInput")?;
    if suffix == "AllOff" {
        return Some(OverlayFunction::AllOff);
    }
    let (digits, action) = if let Some(rest) = suffix.strip_suffix("Off") {
        (rest, Action::Off)
    } else if let Some(rest) = suffix.strip_suffix("Out") {
        (rest, Action::Out)
    } else if let Some(rest) = suffix.strip_suffix("In") {
        (rest, Action::In)
    } else if let Some(rest) = suffix.strip_suffix("Zoom") {
        (rest, Action::Zoom)
    } else if let Some(rest) = suffix.strip_suffix("Last") {
        (rest, Action::Last)
    } else {
        (suffix, Action::Plain)
    };
    let number: usize = digits.parse().ok()?;
    match number {
        1..=8 => Some(match action {
            Action::Off => OverlayFunction::Off(number - 1),
            Action::Out => OverlayFunction::Variant(
                "the animated overlay-out is not implemented; OverlayInput<n>Off clears the channel",
            ),
            Action::In => OverlayFunction::Variant(
                "re-showing an assigned overlay input is not implemented; OverlayInput<n>Off clears the channel",
            ),
            Action::Zoom => OverlayFunction::Variant(
                "the overlay zoom variant is not implemented; OverlayInput<n>Off clears the channel",
            ),
            Action::Last => OverlayFunction::Variant(
                "the overlay last variant is not implemented; OverlayInput<n>Off clears the channel",
            ),
            Action::Plain => OverlayFunction::Bind(number - 1),
        }),
        9..=16 => Some(OverlayFunction::StingerChannel),
        _ => None,
    }
}

enum Action {
    Plain,
    Off,
    Out,
    In,
    Zoom,
    Last,
}

/// `Stinger1`–`Stinger8` → the 0-based stinger slot to trigger. No other
/// `Stinger*` spelling exists on the wire.
fn stinger_function(name: &str) -> Option<usize> {
    let number: usize = name.strip_prefix("Stinger")?.parse().ok()?;
    (1..=8).contains(&number).then_some(number - 1)
}

/// The outcome of a dispatched request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// A command executed and returned a revision.
    Executed,
    /// A well-formed request whose input target does not exist in this
    /// session; the observed endpoint answers success and changes nothing.
    Noop,
}

/// Translate a request into a command and execute it against the engine.
pub fn dispatch(engine: &mut Engine, req: &Request, now: u64) -> Result<Outcome, DispatchError> {
    let name = req.function.trim().to_string();
    let input = input_id(engine, req)?;

    // Input-targeted families resolve first: an absent target is a no-op, so
    // they never fall through to the unknown-function arm.
    let input_command: Option<Command> = match name.as_str() {
        "PreviewInput" => Some(match input {
            Some(id) => Command::SelectPreview(id),
            None => return Ok(Outcome::Noop),
        }),
        "CutDirect" => Some(match input {
            Some(id) => Command::TakeInput(id),
            None => return Ok(Outcome::Noop),
        }),
        "Play" => Some(match input {
            Some(id) => Command::SetInputPaused(id, false),
            None => return Ok(Outcome::Noop),
        }),
        "Pause" => Some(match input {
            Some(id) => Command::SetInputPaused(id, true),
            None => return Ok(Outcome::Noop),
        }),
        "Restart" => Some(match input {
            Some(id) => Command::RestartInput(id),
            None => return Ok(Outcome::Noop),
        }),
        "LoopOn" | "LoopOff" => Some(match input {
            Some(id) => Command::SetInputLoop(id, name == "LoopOn"),
            None => return Ok(Outcome::Noop),
        }),
        "SetPosition" => {
            return match input {
                Some(id) => {
                    let position = u64::from(u32_value(req)?);
                    engine
                        .execute(Command::SeekInput(id, position), now)
                        .map(|_| Outcome::Executed)
                        .map_err(|e| DispatchError::Rejected(e.to_string()))
                }
                None => Ok(Outcome::Noop),
            };
        }
        "SetInputName" => {
            return match input {
                Some(id) => {
                    let title = required(req, "Value")?.to_string();
                    let mut state = engine
                        .project()
                        .input(id)
                        .cloned()
                        .ok_or_else(|| DispatchError::Rejected("input is missing".into()))?;
                    state.name = title;
                    engine
                        .execute(Command::UpdateInput(state), now)
                        .map(|_| Outcome::Executed)
                        .map_err(|e| DispatchError::Rejected(e.to_string()))
                }
                None => Ok(Outcome::Noop),
            };
        }
        "SetCategory" => {
            return match input {
                Some(id) => {
                    let ordinal = u32_value(req)?;
                    let category = if ordinal == 0 {
                        None
                    } else {
                        Some(
                            u8::try_from(ordinal).map_err(|_| DispatchError::BadParameter {
                                name: "Value",
                                value: ordinal.to_string(),
                            })?,
                        )
                    };
                    engine
                        .execute(Command::SetInputCategory(id, category), now)
                        .map(|_| Outcome::Executed)
                        .map_err(|e| DispatchError::Rejected(e.to_string()))
                }
                None => Ok(Outcome::Noop),
            };
        }
        "RemoveInput" => Some(match input {
            Some(id) => Command::RemoveInput(id),
            None => return Ok(Outcome::Noop),
        }),
        _ => None,
    };
    if let Some(command) = input_command {
        return engine
            .execute(command, now)
            .map(|_| Outcome::Executed)
            .map_err(|e| DispatchError::Rejected(e.to_string()));
    }

    // Audio families read the input's live channel state and write it back.
    if let Some(bus_index) = bus_volume_function(&name) {
        let value = volume_to_db(f32_value(req)?);
        let mut channel = engine
            .project()
            .buses
            .get(bus_index)
            .cloned()
            .unwrap_or_default();
        channel.gain_db = value;
        return engine
            .execute(Command::SetBus(bus_index, channel), now)
            .map(|_| Outcome::Executed)
            .map_err(|e| DispatchError::Rejected(e.to_string()));
    }
    match name.as_str() {
        "SetVolume" | "SetMute" | "SetSolo" => {
            let Some(id) = input else {
                return Ok(Outcome::Noop);
            };
            let mut channel: AudioChannel = engine
                .project()
                .input(id)
                .map(|i| i.audio.clone())
                .unwrap_or_default();
            match name.as_str() {
                "SetVolume" => channel.gain_db = volume_to_db(f32_value(req)?),
                "SetMute" => channel.muted = bool_value(req)?,
                _ => channel.solo = bool_value(req)?,
            }
            return engine
                .execute(Command::SetInputAudio(id, channel), now)
                .map(|_| Outcome::Executed)
                .map_err(|e| DispatchError::Rejected(e.to_string()));
        }
        _ => {}
    }

    // Overlay and stinger families. `OverlayInput<n>` puts the request's input
    // on the channel; an absent or unresolvable target is the usual no-op.
    if let Some(overlay) = overlay_function(&name) {
        return match overlay {
            OverlayFunction::Bind(slot) => match input {
                Some(id) => engine
                    .execute(Command::SetOverlayInput(slot, Some(id)), now)
                    .map(|_| Outcome::Executed)
                    .map_err(|e| DispatchError::Rejected(e.to_string())),
                None => Ok(Outcome::Noop),
            },
            OverlayFunction::Off(slot) => engine
                .execute(Command::SetOverlayInput(slot, None), now)
                .map(|_| Outcome::Executed)
                .map_err(|e| DispatchError::Rejected(e.to_string())),
            OverlayFunction::Variant(reason) => Err(unsupported(&name, reason)),
            OverlayFunction::AllOff => Err(unsupported(
                &name,
                "clearing all overlay channels at once is not implemented; clear each with OverlayInput<n>Off",
            )),
            OverlayFunction::StingerChannel => Err(unsupported(
                &name,
                "overlay channels 9-16 are Stingers 1-8; trigger them with Stinger1-Stinger8",
            )),
        };
    }
    if let Some(slot) = stinger_function(&name) {
        return engine
            .execute(Command::TriggerStinger(slot), now)
            .map(|_| Outcome::Executed)
            .map_err(|e| DispatchError::Rejected(e.to_string()));
    }

    // Session-level switcher commands.
    let command = match name.as_str() {
        "Cut" => Command::Cut,
        "Transition1" => Command::RunTransitionSlot(0),
        "Transition2" => Command::RunTransitionSlot(1),
        "Transition3" => Command::RunTransitionSlot(2),
        "Transition4" => Command::RunTransitionSlot(3),
        "FadeToBlack" => Command::FadeBlack,
        "StartPlayList" => Command::StartPlaylist { id: 1, selected: 0 },
        "StopPlayList" => Command::StopPlaylist,
        "PreviewInputNext" | "PreviewInputPrevious" => {
            let project = engine.project();
            let count = project.inputs.len();
            if count == 0 {
                return Ok(Outcome::Noop);
            }
            let current = project.input_number(project.preview);
            let next = if name == "PreviewInputNext" {
                current % count + 1
            } else if current <= 1 {
                count
            } else {
                current - 1
            };
            match project.inputs.get(next.saturating_sub(1)) {
                Some(target) => Command::SelectPreview(target.id),
                None => return Ok(Outcome::Noop),
            }
        }
        "PlayPause" => {
            // The engine exposes its per-input transport clock publicly, so the
            // toggle resolves honestly: a playing input pauses, a paused input
            // (including one parked at the end of a non-looping source) plays.
            return match input {
                Some(id) => {
                    let paused = engine.playback(id).is_some_and(|clock| clock.paused());
                    engine
                        .execute(Command::SetInputPaused(id, !paused), now)
                        .map(|_| Outcome::Executed)
                        .map_err(|e| DispatchError::Rejected(e.to_string()))
                }
                None => Ok(Outcome::Noop),
            };
        }
        slot_effect @ ("SetTransitionEffect1"
        | "SetTransitionEffect2"
        | "SetTransitionEffect3"
        | "SetTransitionEffect4") => {
            let index = slot_digit(slot_effect).saturating_sub(1);
            let raw = required(req, "Value")?;
            let kind: TransitionKind =
                effect_from_wire(raw).ok_or_else(|| DispatchError::BadParameter {
                    name: "Value",
                    value: raw.to_string(),
                })?;
            let mut slot: TransitionSlot = engine
                .project()
                .transition_slots
                .get(index)
                .cloned()
                .ok_or_else(|| DispatchError::Rejected("transition slot is missing".into()))?;
            slot.kind = kind;
            Command::SetTransitionSlot(index, slot)
        }
        slot_duration @ ("SetTransitionDuration1"
        | "SetTransitionDuration2"
        | "SetTransitionDuration3"
        | "SetTransitionDuration4") => {
            let index = slot_digit(slot_duration).saturating_sub(1);
            let duration = u32_value(req)?;
            let mut slot: TransitionSlot = engine
                .project()
                .transition_slots
                .get(index)
                .cloned()
                .ok_or_else(|| DispatchError::Rejected("transition slot is missing".into()))?;
            slot.duration_ms = duration;
            Command::SetTransitionSlot(index, slot)
        }

        // Transport families with no backend: declared, never faked.
        "StartRecording" | "StopRecording" | "ToggleRecording" => {
            return Err(unsupported(
                &name,
                "recording has no encoder backend yet; a start would not produce a file",
            ));
        }
        "StartStreaming" | "StopStreaming" | "StartStopStreaming" => {
            return Err(unsupported(
                &name,
                "streaming has no encoder/transport backend yet",
            ));
        }
        "StartMultiCorder" | "StopMultiCorder" => {
            return Err(unsupported(&name, "multicorder has no encoder backend yet"));
        }
        _ => {
            return Err(DispatchError::UnknownFunction(name));
        }
    };
    engine
        .execute(command, now)
        .map(|_| Outcome::Executed)
        .map_err(|e| DispatchError::Rejected(e.to_string()))
}

/// Volume family → bus index into the nine-bus matrix.
fn bus_volume_function(name: &str) -> Option<usize> {
    match name {
        "SetMasterVolume" => Some(0),
        "SetBusAVolume" => Some(1),
        "SetBusBVolume" => Some(2),
        "SetBusCVolume" => Some(3),
        "SetBusDVolume" => Some(4),
        "SetBusEVolume" => Some(5),
        "SetBusFVolume" => Some(6),
        "SetBusGVolume" => Some(7),
        "SetHeadphonesVolume" => Some(8),
        _ => None,
    }
}

fn slot_digit(name: &str) -> usize {
    name.as_bytes()
        .last()
        .and_then(|b| (b.is_ascii_digit()).then(|| usize::from(b - b'0')))
        .unwrap_or(1)
}
