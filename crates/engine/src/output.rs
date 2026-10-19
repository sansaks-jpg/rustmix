//! Four external output slots: independent routing and configuration.
//!
//! Independent configuration for the four-output workflow observed in the
//! task 1.3 reference inventory: source, rate, raster, hardware connector,
//! audio assignment, alpha and delay. This module opens no device, encoder
//! or network session and exposes no live state. Device/connector names are
//! descriptors only until a backend advertises verified capabilities.
//! Fullscreen outputs and network transport flags are outside this model.

use crate::{InputId, InputState, Rate, Result, invalid};
use serde::{Deserialize, Serialize};

/// Number of general-purpose external output slots.
pub const EXTERNAL_OUTPUTS: usize = 4;

/// Maximum output raster width accepted by configuration validation.
pub const MAX_OUTPUT_WIDTH: u32 = 7680;
/// Maximum output raster height accepted by configuration validation.
pub const MAX_OUTPUT_HEIGHT: u32 = 4320;
/// Maximum audio delay in milliseconds.
pub const MAX_AUDIO_DELAY_MS: u32 = 5_000;
/// Maximum device-name length accepted by configuration validation.
const MAX_DEVICE_NAME: usize = 96;

/// What an external output renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OutputSource {
    /// The output is not assigned.
    #[default]
    None,
    /// The Program bus.
    Program,
    /// The Preview bus.
    Preview,
    /// A single input, referenced by stable identity.
    Input(InputId),
}

/// Persisted configuration of one external output slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalOutputConfig {
    /// Output frame rate as an exact rational (e.g. 30000/1001 for 29.97 fps).
    pub frame_rate: Rate,
    /// Output raster width in pixels.
    pub width: u32,
    /// Output raster height in pixels.
    pub height: u32,
    /// Hardware device name; empty selects the application renderer.
    pub device: String,
    /// Hardware connector number (1-based); `None` selects the device default.
    /// This is a descriptor, not a network port or verified device capability.
    pub port: Option<u16>,
    /// Audio rendering device name; empty selects the default.
    pub audio_device: String,
    /// Bus index into `ProjectState::buses`; `None` routes Master.
    pub audio_bus: Option<u8>,
    /// Alpha-channel publishing enabled.
    pub alpha_channel: bool,
    /// Audio delay in milliseconds.
    pub audio_delay_ms: u32,
}

impl Default for ExternalOutputConfig {
    fn default() -> Self {
        // Defaults mirror the default project canvas: 1080p30, default
        // devices, Master bus, default connector and no delay.
        Self {
            frame_rate: Rate {
                numerator: 30,
                denominator: 1,
            },
            width: 1920,
            height: 1080,
            device: String::new(),
            port: None,
            audio_device: String::new(),
            audio_bus: None,
            alpha_channel: false,
            audio_delay_ms: 0,
        }
    }
}

impl ExternalOutputConfig {
    /// Validate the configuration against the session's bus count.
    pub fn validate(&self, bus_count: usize) -> Result<()> {
        if Rate::new(self.frame_rate.numerator, self.frame_rate.denominator)? != self.frame_rate {
            return Err(invalid(
                "Output frame rate must be a reduced rate between 1 and 120 fps",
            ));
        }
        if self.width == 0 || self.height == 0 {
            return Err(invalid("Output size must be non-empty"));
        }
        if self.width > MAX_OUTPUT_WIDTH || self.height > MAX_OUTPUT_HEIGHT {
            return Err(invalid("Output size exceeds the 8K raster bound"));
        }
        if self.port == Some(0) {
            return Err(invalid("Output device connector must be 1 or greater"));
        }
        if self
            .audio_bus
            .is_some_and(|bus| usize::from(bus) >= bus_count)
        {
            return Err(invalid("Output audio bus is out of range"));
        }
        if self.audio_delay_ms > MAX_AUDIO_DELAY_MS {
            return Err(invalid("Output audio delay must be 0–5000 ms"));
        }
        valid_device(&self.device)?;
        valid_device(&self.audio_device)
    }
}

/// Device names may be empty (the default device) but must stay bounded text.
fn valid_device(s: &str) -> Result<()> {
    if s.len() > MAX_DEVICE_NAME || s.chars().any(char::is_control) {
        Err(invalid(
            "Device name is too long or contains control characters",
        ))
    } else {
        Ok(())
    }
}

/// One external output slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalOutput {
    /// 1-based slot number.
    pub number: u8,
    pub source: OutputSource,
    pub config: ExternalOutputConfig,
}

impl ExternalOutput {
    /// Validate a complete staged slot before publishing either of its fields.
    pub fn validate(&self, inputs: &[InputState], bus_count: usize) -> Result<()> {
        if ExternalOutputBank::index(self.number).is_none() {
            return Err(invalid("External output not found"));
        }
        if let OutputSource::Input(id) = self.source
            && !inputs.iter().any(|input| input.id == id)
        {
            return Err(invalid("External output references a missing input"));
        }
        self.config.validate(bus_count)
    }
}

/// The four external output slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalOutputBank {
    slots: [ExternalOutput; EXTERNAL_OUTPUTS],
}

impl Default for ExternalOutputBank {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|index| ExternalOutput {
                number: (index + 1) as u8,
                source: OutputSource::None,
                config: ExternalOutputConfig::default(),
            }),
        }
    }
}

impl ExternalOutputBank {
    /// Every slot in slot order.
    pub fn iter(&self) -> impl Iterator<Item = &ExternalOutput> {
        self.slots.iter()
    }

    /// Read a slot by its 1-based number.
    pub fn slot(&self, number: u8) -> Option<&ExternalOutput> {
        Self::index(number).and_then(|index| self.slots.get(index))
    }

    /// Replace source and configuration together after complete validation.
    pub fn apply(
        &mut self,
        output: ExternalOutput,
        inputs: &[InputState],
        bus_count: usize,
    ) -> Result<()> {
        output.validate(inputs, bus_count)?;
        let slot = Self::index(output.number)
            .and_then(|index| self.slots.get_mut(index))
            .ok_or_else(|| invalid("External output not found"))?;
        *slot = output;
        Ok(())
    }

    /// Assign a source to a slot by its 1-based number.
    ///
    /// Returns `false` — leaving the slot untouched — when the number is out
    /// of range.
    pub fn set_source(&mut self, number: u8, source: OutputSource) -> bool {
        match Self::index(number).and_then(|index| self.slots.get_mut(index)) {
            Some(slot) => {
                slot.source = source;
                true
            }
            None => false,
        }
    }

    /// Store a slot configuration by its 1-based number.
    ///
    /// Returns `false` — leaving the slot untouched — when the number is out
    /// of range or the configuration fails validation.
    pub fn set_config(
        &mut self,
        number: u8,
        config: ExternalOutputConfig,
        bus_count: usize,
    ) -> bool {
        if config.validate(bus_count).is_err() {
            return false;
        }
        match Self::index(number).and_then(|index| self.slots.get_mut(index)) {
            Some(slot) => {
                slot.config = config;
                true
            }
            None => false,
        }
    }

    /// True when any slot currently renders the given input.
    pub fn any_references(&self, input: InputId) -> bool {
        self.slots
            .iter()
            .any(|slot| slot.source == OutputSource::Input(input))
    }

    /// Drop references to an input that no longer exists.
    pub fn forget_input(&mut self, input: InputId) {
        for slot in &mut self.slots {
            if slot.source == OutputSource::Input(input) {
                slot.source = OutputSource::None;
            }
        }
    }

    /// Validate the complete bank against the session's inputs and bus count.
    pub fn validate(&self, inputs: &[InputState], bus_count: usize) -> Result<()> {
        for (index, slot) in self.slots.iter().enumerate() {
            if slot.number != (index + 1) as u8 {
                return Err(invalid("External output slots must stay numbered 1–4"));
            }
            slot.validate(inputs, bus_count)?;
        }
        Ok(())
    }

    fn index(number: u8) -> Option<usize> {
        (1..=EXTERNAL_OUTPUTS as u8)
            .contains(&number)
            .then(|| usize::from(number) - 1)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::ProjectState;

    fn valid_config() -> ExternalOutputConfig {
        ExternalOutputConfig {
            frame_rate: Rate {
                numerator: 60000,
                denominator: 1001,
            },
            width: 1280,
            height: 720,
            device: "test-device".into(),
            port: Some(3),
            audio_device: "line-out".into(),
            audio_bus: Some(2),
            alpha_channel: true,
            audio_delay_ms: 25,
        }
    }

    #[test]
    fn default_bank_has_four_idle_slots_with_project_defaults() {
        let bank = ExternalOutputBank::default();
        assert_eq!(bank.iter().count(), EXTERNAL_OUTPUTS);
        for (index, slot) in bank.iter().enumerate() {
            assert_eq!(slot.number, (index + 1) as u8);
            assert_eq!(slot.source, OutputSource::None);
            assert_eq!(slot.config, ExternalOutputConfig::default());
        }
        let slot = bank.slot(1).unwrap();
        assert_eq!(slot.config.width, 1920);
        assert_eq!(slot.config.height, 1080);
        assert_eq!(slot.config.frame_rate.label(), "30.00");
        assert_eq!(slot.config.audio_bus, None);
        assert_eq!(slot.config.audio_delay_ms, 0);
        assert!(!slot.config.alpha_channel);
        assert!(bank.slot(0).is_none());
        assert!(bank.slot(5).is_none());
    }

    #[test]
    fn routing_accepts_program_preview_and_inputs_by_identity() {
        let mut bank = ExternalOutputBank::default();
        assert!(bank.set_source(1, OutputSource::Program));
        assert!(bank.set_source(4, OutputSource::Preview));
        assert!(bank.set_source(3, OutputSource::Input(InputId(7))));
        assert_eq!(bank.slot(1).unwrap().source, OutputSource::Program);
        assert_eq!(bank.slot(4).unwrap().source, OutputSource::Preview);
        assert_eq!(
            bank.slot(3).unwrap().source,
            OutputSource::Input(InputId(7))
        );
        assert_eq!(bank.slot(2).unwrap().source, OutputSource::None);
        assert!(!bank.set_source(0, OutputSource::Program));
        assert!(!bank.set_source(5, OutputSource::Program));
    }

    #[test]
    fn configuration_validation_rejects_invalid_fields() {
        let mut bank = ExternalOutputBank::default();
        let config = valid_config();
        assert!(bank.set_config(3, config.clone(), 3));
        assert_eq!(bank.slot(3).unwrap().config, config);
        assert_eq!(
            bank.slot(1).unwrap().config,
            ExternalOutputConfig::default()
        );

        let bus_count = 3;
        let mut rejected = config.clone();
        rejected.frame_rate.denominator = 0;
        assert!(rejected.validate(bus_count).is_err());
        assert!(!bank.set_config(2, rejected.clone(), bus_count));
        assert_eq!(
            bank.slot(2).unwrap().config,
            ExternalOutputConfig::default()
        );

        rejected = config.clone();
        rejected.frame_rate = Rate {
            numerator: 50,
            denominator: 2,
        };
        assert!(
            rejected.validate(bus_count).is_err(),
            "unreduced rates are rejected"
        );

        rejected = config.clone();
        rejected.height = 0;
        assert!(rejected.validate(bus_count).is_err());

        rejected = config.clone();
        rejected.width = MAX_OUTPUT_WIDTH + 1;
        assert!(rejected.validate(bus_count).is_err());

        rejected = config.clone();
        rejected.audio_bus = Some(3);
        assert!(rejected.validate(bus_count).is_err());

        rejected = config.clone();
        rejected.audio_delay_ms = MAX_AUDIO_DELAY_MS + 1;
        assert!(rejected.validate(bus_count).is_err());

        rejected = config.clone();
        rejected.device = "bad\u{7}device".into();
        assert!(rejected.validate(bus_count).is_err());

        rejected = config;
        rejected.audio_device = "x".repeat(MAX_DEVICE_NAME + 1);
        assert!(rejected.validate(bus_count).is_err());
    }

    #[test]
    fn bank_validation_matches_sources_against_live_inputs() {
        let project = ProjectState::demo();
        let mut bank = ExternalOutputBank::default();
        bank.validate(&project.inputs, project.buses.len()).unwrap();
        assert!(bank.set_source(2, OutputSource::Input(InputId(3))));
        bank.validate(&project.inputs, project.buses.len()).unwrap();
        assert!(bank.set_source(2, OutputSource::Input(InputId(999))));
        assert!(bank.validate(&project.inputs, project.buses.len()).is_err());
    }

    #[test]
    fn inputs_that_leave_are_forgotten() {
        let mut bank = ExternalOutputBank::default();
        assert!(bank.set_source(1, OutputSource::Input(InputId(3))));
        assert!(bank.set_source(4, OutputSource::Input(InputId(5))));
        assert!(bank.any_references(InputId(3)));
        bank.forget_input(InputId(3));
        assert!(!bank.any_references(InputId(3)));
        assert!(bank.any_references(InputId(5)));
        assert_eq!(bank.slot(1).unwrap().source, OutputSource::None);
        assert_eq!(
            bank.slot(4).unwrap().source,
            OutputSource::Input(InputId(5))
        );
    }

    #[test]
    fn serde_round_trip_preserves_routing_and_configuration() {
        let mut bank = ExternalOutputBank::default();
        assert!(bank.set_config(2, valid_config(), 3));
        assert!(bank.set_source(2, OutputSource::Input(InputId(4))));
        let json = serde_json::to_string(&bank).unwrap();
        let back: ExternalOutputBank = serde_json::from_str(&json).unwrap();
        assert_eq!(back, bank);
    }

    #[test]
    fn tampered_slot_numbers_are_rejected_on_validate() {
        let json = r#"{"slots":[
            {"number":7,"source":"None","config":{"frame_rate":{"numerator":30,"denominator":1},
            "width":1920,"height":1080,"device":"","port":null,"audio_device":"",
            "audio_bus":null,"alpha_channel":false,"audio_delay_ms":0}},
            {"number":2,"source":"None","config":{"frame_rate":{"numerator":30,"denominator":1},
            "width":1920,"height":1080,"device":"","port":null,"audio_device":"",
            "audio_bus":null,"alpha_channel":false,"audio_delay_ms":0}},
            {"number":3,"source":"None","config":{"frame_rate":{"numerator":30,"denominator":1},
            "width":1920,"height":1080,"device":"","port":null,"audio_device":"",
            "audio_bus":null,"alpha_channel":false,"audio_delay_ms":0}},
            {"number":4,"source":"None","config":{"frame_rate":{"numerator":30,"denominator":1},
            "width":1920,"height":1080,"device":"","port":null,"audio_device":"",
            "audio_bus":null,"alpha_channel":false,"audio_delay_ms":0}}
        ]}"#;
        let bank: ExternalOutputBank = serde_json::from_str(json).unwrap();
        let project = ProjectState::demo();
        assert!(bank.validate(&project.inputs, project.buses.len()).is_err());
    }
}
