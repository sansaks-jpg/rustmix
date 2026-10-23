//! Typed input-category contract ported from the reference Task 1.2.
//!
//! The reference model addresses categories as a typed value: `All` is the
//! default "uncategorised" encoding, six preset colours follow the analysed
//! catalogue, sixteen user categories clamp into `1..=16`, and `Search` is a
//! UI-only pseudo filter that shows everything. RustMix keeps its own
//! persisted representation — per-input `Option<u8>` definition ids over the
//! 22 editable definitions (six standard plus sixteen custom) — so this
//! module maps the typed contract onto that stored space without changing the
//! schema. The typed values address definition *slots*: operators may rename
//! or recolour any definition, and the preset labels below are the default
//! catalogue names, not a fixed vocabulary.
//!
//! Two reference details are deliberately not reproduced: the reference can
//! store its `Search` pseudo value on an input, and its HTTP layer decodes an
//! inferred, unprobed wire ordinal (0 = `All`, 1..=16 = custom). RustMix has
//! no control endpoint yet, so `Search` stays filter-only and no wire
//! encoding is exposed.

/// The six preset colours in catalogue order (reference `PRESET_CATEGORIES`).
///
/// The last two labels are the verified reference catalogue names `Aqua` and
/// `Blue` (confirmed against the reference binary strings and metadata);
/// neither `Azure` nor `MidnightBlue` appears there. Persisted definition ids
/// and rustmix's own colour values are unchanged, so existing projects load.
pub const PRESET_CATEGORIES: [InputCategory; 6] = [
    InputCategory::Red,
    InputCategory::Green,
    InputCategory::Orange,
    InputCategory::Purple,
    InputCategory::Aqua,
    InputCategory::Blue,
];

/// Persisted definition id of the first custom category.
const CUSTOM_ID_FIRST: u8 = PRESET_CATEGORIES.len() as u8;

/// Typed category value for assignment and filtering (reference Task 1.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum InputCategory {
    /// The default "uncategorised" value.
    #[default]
    All,
    Red,
    Green,
    Orange,
    Purple,
    Aqua,
    Blue,
    /// User category `1..=16`; use [`InputCategory::custom`] to clamp.
    Custom(u8),
    /// UI-only pseudo filter; the text matcher belongs to the caller.
    Search,
}

impl InputCategory {
    /// Number of user-definable categories (reference `CUSTOM_COUNT`).
    pub const CUSTOM_COUNT: u8 = 16;

    /// Clamp a custom index into `1..=16` (reference `InputCategory::custom`).
    ///
    /// A directly constructed out-of-range `Custom` value behaves like its
    /// clamped counterpart wherever it is used.
    pub fn custom(index: u8) -> Self {
        Self::Custom(index.clamp(1, Self::CUSTOM_COUNT))
    }

    /// The persisted definition id this value assigns; `None` is the `All`
    /// pseudo-category. `Search` is filter-only and never assigns an id.
    pub fn stored_id(self) -> Option<u8> {
        match self {
            Self::All | Self::Search => None,
            Self::Red => Some(0),
            Self::Green => Some(1),
            Self::Orange => Some(2),
            Self::Purple => Some(3),
            Self::Aqua => Some(4),
            Self::Blue => Some(5),
            Self::Custom(index) => Some(CUSTOM_ID_FIRST + index.clamp(1, Self::CUSTOM_COUNT) - 1),
        }
    }

    /// Decode a persisted assignment. `None` means `All`; ids outside the
    /// 22 editable definitions are rejected so a caller keeps its previous
    /// value instead of inventing a category.
    pub fn from_stored(id: Option<u8>) -> Option<Self> {
        match id {
            None => Some(Self::All),
            Some(0) => Some(Self::Red),
            Some(1) => Some(Self::Green),
            Some(2) => Some(Self::Orange),
            Some(3) => Some(Self::Purple),
            Some(4) => Some(Self::Aqua),
            Some(5) => Some(Self::Blue),
            Some(id @ CUSTOM_ID_FIRST..=LAST_CUSTOM_ID) => {
                Some(Self::Custom(id - CUSTOM_ID_FIRST + 1))
            }
            _ => None,
        }
    }

    /// Visibility semantics of the reference `State::visible_inputs`: `All`
    /// and `Search` show every input, every other value matches its own
    /// assignment exactly.
    pub fn matches(self, stored: Option<u8>) -> bool {
        match self {
            Self::All | Self::Search => true,
            value => value.stored_id() == stored,
        }
    }
}

/// Persisted id of the last custom category (`CUSTOM_ID_FIRST + 16 - 1`).
const LAST_CUSTOM_ID: u8 = CUSTOM_ID_FIRST + InputCategory::CUSTOM_COUNT - 1;

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn custom_indices_clamp_into_the_sixteen_definition_slots() {
        assert_eq!(InputCategory::custom(0), InputCategory::Custom(1));
        assert_eq!(InputCategory::custom(99), InputCategory::Custom(16));
        assert_eq!(InputCategory::custom(7), InputCategory::Custom(7));
        assert_eq!(InputCategory::Custom(0).stored_id(), Some(6));
        assert_eq!(InputCategory::Custom(99).stored_id(), Some(21));
    }

    #[test]
    fn stored_ids_roundtrip_with_the_persisted_definition_list() {
        for (index, preset) in PRESET_CATEGORIES.iter().enumerate() {
            assert_eq!(preset.stored_id(), Some(index as u8));
            assert_eq!(InputCategory::from_stored(Some(index as u8)), Some(*preset));
        }
        for index in 1..=InputCategory::CUSTOM_COUNT {
            let value = InputCategory::custom(index);
            assert_eq!(value.stored_id(), Some(5 + index));
            assert_eq!(InputCategory::from_stored(value.stored_id()), Some(value));
        }
        assert_eq!(InputCategory::from_stored(None), Some(InputCategory::All));
        assert_eq!(InputCategory::from_stored(Some(22)), None);
        assert_eq!(InputCategory::from_stored(Some(u8::MAX)), None);
        assert_eq!(InputCategory::Search.stored_id(), None);
    }

    #[test]
    fn filter_semantics_match_the_reference_visible_inputs() {
        assert!(InputCategory::All.matches(None));
        assert!(InputCategory::All.matches(Some(0)));
        assert!(InputCategory::All.matches(Some(21)));
        assert!(InputCategory::Search.matches(None));
        assert!(InputCategory::Search.matches(Some(7)));
        assert!(InputCategory::Green.matches(Some(1)));
        assert!(!InputCategory::Green.matches(Some(0)));
        assert!(!InputCategory::Green.matches(None));
        assert!(InputCategory::custom(1).matches(Some(6)));
        assert!(!InputCategory::custom(1).matches(Some(7)));
        assert!(!InputCategory::custom(16).matches(Some(20)));
    }

    #[test]
    fn default_definition_names_agree_with_the_typed_preset_order() {
        let definitions = crate::default_categories();
        assert_eq!(definitions.len(), usize::from(LAST_CUSTOM_ID) + 1);
        for (index, preset) in PRESET_CATEGORIES.iter().enumerate() {
            assert_eq!(definitions[index].id, preset.stored_id().unwrap());
        }
        let names: Vec<_> = definitions.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names[..6],
            ["Red", "Green", "Orange", "Purple", "Aqua", "Blue"]
        );
    }
}
