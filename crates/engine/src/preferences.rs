//! Persisted original operator presentation and stable input category identities.
use crate::{ProjectState, Result, invalid};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiTheme {
    #[default]
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryDefinition {
    pub id: u8,
    pub name: String,
    pub color: [u8; 3],
}

pub fn default_categories() -> Vec<CategoryDefinition> {
    let standard = [
        ("Red", [174, 63, 66]),
        ("Green", [47, 140, 98]),
        ("Orange", [218, 132, 47]),
        ("Purple", [146, 91, 180]),
        ("Azure", [48, 145, 194]),
        ("Midnight Blue", [65, 84, 145]),
    ];
    (0_u8..22)
        .map(|id| {
            let (name, color) = standard.get(usize::from(id)).map_or_else(
                || (format!("Custom {}", id.saturating_sub(5)), [108, 120, 138]),
                |(name, color)| ((*name).into(), *color),
            );
            CategoryDefinition { id, name, color }
        })
        .collect()
}

pub fn default_preview_color() -> [u8; 3] {
    [234, 128, 24]
}
pub fn default_program_color() -> [u8; 3] {
    [22, 163, 74]
}

pub(crate) fn validate(project: &ProjectState) -> Result<()> {
    let categories = &project.preferences.categories;
    if categories.len() != 22 {
        return Err(invalid("Exactly 22 category definitions are required"));
    }
    let mut ids = HashSet::new();
    for category in categories {
        if category.id >= 22 || !ids.insert(category.id) {
            return Err(invalid("Category IDs must uniquely cover 0–21"));
        }
        if category.name.trim().is_empty()
            || category.name.chars().count() > 48
            || category.name.chars().any(char::is_control)
        {
            return Err(invalid("Category names need 1–48 visible characters"));
        }
    }
    if project
        .inputs
        .iter()
        .any(|input| input.category.is_some_and(|id| !ids.contains(&id)))
    {
        return Err(invalid("Input references an unknown category"));
    }
    Ok(())
}
