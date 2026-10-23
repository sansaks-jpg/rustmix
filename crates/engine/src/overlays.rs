//! Sixteen production overlay channels: eight standard title-panel channels
//! (1–8) and eight stinger channels (9–16).
//!
//! The channel shape follows the reference task 1.1 findings: standard
//! channels `1..=8` and stinger channels `9..=16`, where each stinger carries
//! its source input, a cut point and a display-underneath flag. RustMix renders
//! standard channels as its existing title panels, and a standard channel may
//! additionally bind an input ([`OverlayBank::inputs`]): the bound input's
//! frame is composited as a picture-in-picture at the channel's panel position,
//! underneath the caption text. A triggered stinger
//! performs its Program/Preview exchange exactly once at the cut point on the
//! engine clock. Stinger *clip compositing* has no media renderer yet: the
//! model, commands and bus exchange are real, the animated clip layer is
//! explicitly not implemented and documented as such.

use crate::{InputId, InputState, OverlayState, Position, Result, invalid, valid_text};
use serde::{Deserialize, Serialize};

/// Standard title-panel channels (1..=8).
pub const STANDARD_CHANNELS: usize = 8;
/// Stinger channels (9..=16).
pub const STINGER_CHANNELS: usize = 8;
/// Total production overlay channels.
pub const TOTAL_CHANNELS: usize = STANDARD_CHANNELS + STINGER_CHANNELS;
/// Upper bound for a stinger cut point (one minute).
pub const MAX_CUT_POINT_MS: u32 = 60_000;

/// Which mechanism a 1-based channel number implements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayChannelKind {
    /// A standard title-panel channel (1..=8).
    Standard(usize),
    /// A stinger channel (9..=16).
    Stinger(usize),
}

impl OverlayChannelKind {
    /// Classify a 1-based channel number.
    pub fn from_channel(number: usize) -> Option<Self> {
        match number {
            1..=STANDARD_CHANNELS => Some(Self::Standard(number)),
            _ if (STANDARD_CHANNELS + 1..=TOTAL_CHANNELS).contains(&number) => {
                Some(Self::Stinger(number - STANDARD_CHANNELS))
            }
            _ => None,
        }
    }

    /// The 1-based channel number this kind addresses.
    pub fn channel_number(self) -> usize {
        match self {
            Self::Standard(n) => n,
            Self::Stinger(n) => STANDARD_CHANNELS + n,
        }
    }
}

/// Stinger-specific configuration carried by channels 9..=16.
///
/// The configured input names the source the stinger would play its clip
/// from; triggering requires it to be set. The cut point is the moment within
/// the (not yet rendered) clip at which the buses exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StingerChannel {
    /// The stable identity of the input this stinger plays from.
    pub stinger_input: Option<InputId>,
    /// Exchange moment in milliseconds from the trigger.
    pub cut_point_ms: u32,
    /// Composite the stinger beneath the other overlay channels (stored; the
    /// clip layer is not rendered yet).
    pub display_underneath_overlays: bool,
}

/// Idle input bindings for the standard channels.
fn default_standard_bindings() -> [Option<InputId>; STANDARD_CHANNELS] {
    std::array::from_fn(|_| None)
}

/// The sixteen production overlay channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlayBank {
    /// Title-panel channels 1–8.
    pub standard: [OverlayState; STANDARD_CHANNELS],
    /// Optional input binding per standard channel. A bound input composites
    /// its frame as a picture-in-picture at the channel's panel position,
    /// underneath the caption text, whenever the channel is enabled. The
    /// field is additive with a serde default, so every older project file
    /// loads with idle (unbound) channels.
    #[serde(default = "default_standard_bindings")]
    pub inputs: [Option<InputId>; STANDARD_CHANNELS],
    /// Stinger channels 9–16.
    pub stingers: [StingerChannel; STINGER_CHANNELS],
}

impl OverlayBank {
    /// Fresh-session defaults: idle title panels with the demo caption and
    /// unconfigured stingers.
    pub fn demo() -> Self {
        Self {
            standard: std::array::from_fn(|i| OverlayState {
                enabled: false,
                title: if i == 0 {
                    "RUSTMIX / STUDIO".into()
                } else {
                    format!("Overlay {}", i + 1)
                },
                subtitle: "Independent live production • Synthetic demo".into(),
                position: Position::LowerLeft,
            }),
            inputs: default_standard_bindings(),
            stingers: std::array::from_fn(|_| StingerChannel::default()),
        }
    }

    /// The standard channel at a 0-based slot (0..=7).
    pub fn standard(&self, slot: usize) -> Option<&OverlayState> {
        self.standard.get(slot)
    }

    /// Mutable access to the standard channel at a 0-based slot.
    pub fn standard_mut(&mut self, slot: usize) -> Option<&mut OverlayState> {
        self.standard.get_mut(slot)
    }

    /// The input bound to the standard channel at a 0-based slot.
    pub fn binding(&self, slot: usize) -> Option<InputId> {
        self.inputs.get(slot).copied().flatten()
    }

    /// Mutable access to the standard channel's input binding.
    pub fn binding_mut(&mut self, slot: usize) -> Option<&mut Option<InputId>> {
        self.inputs.get_mut(slot)
    }

    /// The stinger channel at a 0-based slot (0..=7).
    pub fn stinger(&self, slot: usize) -> Option<&StingerChannel> {
        self.stingers.get(slot)
    }

    /// Mutable access to the stinger channel at a 0-based slot.
    pub fn stinger_mut(&mut self, slot: usize) -> Option<&mut StingerChannel> {
        self.stingers.get_mut(slot)
    }

    /// The kind of the 0-based channel index (0..=15).
    pub fn kind(index: usize) -> Option<OverlayChannelKind> {
        index
            .checked_add(1)
            .and_then(OverlayChannelKind::from_channel)
    }

    /// Validate every channel against the project's inputs.
    pub fn validate(&self, inputs: &[InputState]) -> Result<()> {
        for overlay in &self.standard {
            valid_text(&overlay.title, 96)?;
            if overlay.subtitle.len() > 160 || overlay.subtitle.chars().any(char::is_control) {
                return Err(invalid(
                    "Subtitle is too long or contains control characters",
                ));
            }
        }
        for binding in &self.inputs {
            if let Some(id) = binding
                && !inputs.iter().any(|input| input.id == *id)
            {
                return Err(invalid("Overlay input is missing"));
            }
        }
        for stinger in &self.stingers {
            if stinger.cut_point_ms > MAX_CUT_POINT_MS {
                return Err(invalid("Stinger cut point must be 0–60000 ms"));
            }
            if let Some(id) = stinger.stinger_input
                && !inputs.iter().any(|input| input.id == id)
            {
                return Err(invalid("Stinger input is missing"));
            }
        }
        Ok(())
    }
}

/// A stinger currently in flight. The engine keeps it until an explicit stop,
/// because when the played clip ends is not observable without a media layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveStinger {
    /// 0-based stinger slot (0..8).
    pub slot: usize,
    /// The configured stinger source captured at the trigger.
    pub input: InputId,
    /// Program input captured at the trigger.
    pub from: InputId,
    /// Preview input captured at the trigger; it becomes Program at the cut
    /// point.
    pub to: InputId,
    /// Exchange moment in milliseconds from the trigger.
    pub cut_point_ms: u32,
    /// Monotonic trigger time.
    pub start_ms: u64,
    /// True once the exchange has happened (at most once per trigger).
    pub exchanged: bool,
}

impl ActiveStinger {
    /// Advance to `now`; returns true exactly when this call performs the
    /// cut-point exchange. A zero cut point exchanges immediately.
    pub fn advance(&mut self, now: u64) -> bool {
        if !self.exchanged && now.saturating_sub(self.start_ms) >= u64::from(self.cut_point_ms) {
            self.exchanged = true;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::ProjectState;

    #[test]
    fn channel_kinds_cover_one_to_sixteen() {
        assert_eq!(
            OverlayChannelKind::from_channel(1),
            Some(OverlayChannelKind::Standard(1))
        );
        assert_eq!(
            OverlayChannelKind::from_channel(8),
            Some(OverlayChannelKind::Standard(8))
        );
        assert_eq!(
            OverlayChannelKind::from_channel(9),
            Some(OverlayChannelKind::Stinger(1))
        );
        assert_eq!(
            OverlayChannelKind::from_channel(16),
            Some(OverlayChannelKind::Stinger(8))
        );
        assert_eq!(OverlayChannelKind::from_channel(0), None);
        assert_eq!(OverlayChannelKind::from_channel(17), None);
        for number in 1..=TOTAL_CHANNELS {
            let kind = OverlayChannelKind::from_channel(number).unwrap();
            assert_eq!(kind.channel_number(), number);
            assert_eq!(OverlayBank::kind(number - 1), Some(kind));
        }
    }

    #[test]
    fn demo_bank_has_titled_panels_and_idle_stingers() {
        let bank = OverlayBank::demo();
        assert_eq!(bank.standard.len(), 8);
        assert_eq!(bank.stingers.len(), 8);
        for (i, overlay) in bank.standard.iter().enumerate() {
            assert!(!overlay.enabled);
            assert!(overlay.title.len() >= 4, "channel {} needs a title", i + 1);
        }
        assert!(bank.inputs.iter().all(|binding| binding.is_none()));
        for stinger in &bank.stingers {
            assert_eq!(stinger.stinger_input, None);
            assert_eq!(stinger.cut_point_ms, 0);
            assert!(!stinger.display_underneath_overlays);
        }
    }

    #[test]
    fn bindings_roundtrip_through_the_bank_accessors() {
        let mut bank = OverlayBank::demo();
        assert_eq!(bank.binding(0), None);
        *bank.binding_mut(2).expect("slot 2 exists") = Some(InputId(4));
        assert_eq!(bank.binding(2), Some(InputId(4)));
        assert_eq!(
            bank.binding(8),
            None,
            "only the eight standard channels bind"
        );
        assert!(bank.binding_mut(8).is_none());
    }

    #[test]
    fn validation_rejects_bindings_to_missing_inputs() {
        let inputs = ProjectState::demo().inputs;
        let mut bank = OverlayBank::demo();
        assert!(bank.validate(&inputs).is_ok());
        *bank.binding_mut(0).expect("slot 0 exists") = Some(InputId(9_999));
        assert!(bank.validate(&inputs).is_err());
        *bank.binding_mut(0).expect("slot 0 exists") = Some(inputs[1].id);
        assert!(bank.validate(&inputs).is_ok());
    }

    #[test]
    fn deserialising_an_old_bank_keeps_bindings_idle() {
        // A bank persisted before input bindings existed must load unchanged,
        // with every standard channel unbound.
        let mut old = serde_json::to_value(OverlayBank::demo()).expect("demo bank serialises");
        let removed = old
            .as_object_mut()
            .expect("a bank serialises to an object")
            .remove("inputs");
        assert!(removed.is_some(), "the old shape has no bindings field");
        let bank: OverlayBank = serde_json::from_value(old).expect("old bank shape loads");
        assert_eq!(bank.standard[0].title, "RUSTMIX / STUDIO");
        assert!(bank.inputs.iter().all(|binding| binding.is_none()));
    }

    #[test]
    fn validation_rejects_bad_cut_points_and_missing_inputs() {
        let inputs = ProjectState::demo().inputs;
        let mut bank = OverlayBank::demo();
        assert!(bank.validate(&inputs).is_ok());
        bank.stingers[3].cut_point_ms = MAX_CUT_POINT_MS + 1;
        assert!(bank.validate(&inputs).is_err());
        bank.stingers[3].cut_point_ms = 0;
        bank.stingers[3].stinger_input = Some(InputId(9_999));
        assert!(bank.validate(&inputs).is_err());
        bank.stingers[3].stinger_input = Some(inputs[1].id);
        assert!(bank.validate(&inputs).is_ok());
        bank.standard[7].title = String::new();
        assert!(bank.validate(&inputs).is_err());
    }

    #[test]
    fn stinger_advance_waits_for_its_cut_point_and_exchanges_once() {
        let mut stinger = ActiveStinger {
            slot: 1,
            input: InputId(3),
            from: InputId(1),
            to: InputId(2),
            cut_point_ms: 750,
            start_ms: 1000,
            exchanged: false,
        };
        assert!(!stinger.advance(1000), "trigger frame is still short");
        assert!(!stinger.advance(1749), "349 ms is still short of 750");
        assert!(stinger.advance(1750), "exchange exactly at the cut point");
        assert!(!stinger.advance(10_000), "never a second exchange");
    }

    #[test]
    fn zero_cut_point_exchanges_immediately() {
        let mut stinger = ActiveStinger {
            slot: 0,
            input: InputId(3),
            from: InputId(1),
            to: InputId(2),
            cut_point_ms: 0,
            start_ms: 5000,
            exchanged: false,
        };
        assert!(stinger.advance(5000));
        assert!(stinger.exchanged);
    }
}
