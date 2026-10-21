//! Portable keyboard bindings; UI libraries and hardware key codes stay outside the engine.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShortcutAction {
    Cut,
    TransitionSlot(usize),
    FadeBlack,
    CancelTransition,
    Preview(InputId),
    /// Deliberately positional defaults, matching the visible input numbering.
    PreviewNumber(usize),
    ToggleOverlay(usize),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutBinding {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub action: ShortcutAction,
}
pub fn shortcut_keys() -> Vec<String> {
    ["Space", "Enter", "Escape"]
        .into_iter()
        .map(str::to_owned)
        .chain(('A'..='Z').map(|c| c.to_string()))
        .chain((0..=9).map(|n| n.to_string()))
        .chain((1..=12).map(|n| format!("F{n}")))
        .collect()
}
pub fn default_key_bindings() -> Vec<ShortcutBinding> {
    let mut bindings = vec![
        ("Space".to_owned(), ShortcutAction::Cut),
        ("Enter".to_owned(), ShortcutAction::TransitionSlot(0)),
        ("B".to_owned(), ShortcutAction::FadeBlack),
        ("Escape".to_owned(), ShortcutAction::CancelTransition),
    ];
    bindings.extend((1..=7).map(|n| (n.to_string(), ShortcutAction::PreviewNumber(n))));
    bindings.extend(
        (0..crate::overlays::STANDARD_CHANNELS)
            .map(|n| (format!("F{}", n + 1), ShortcutAction::ToggleOverlay(n))),
    );
    bindings
        .into_iter()
        .map(|(key, action)| ShortcutBinding {
            key,
            ctrl: false,
            shift: false,
            alt: false,
            action,
        })
        .collect()
}
impl ShortcutBinding {
    pub fn label(&self) -> String {
        format!(
            "{}{}{}{}",
            if self.ctrl { "Ctrl+" } else { "" },
            if self.alt { "Alt+" } else { "" },
            if self.shift { "Shift+" } else { "" },
            self.key
        )
    }
}
pub(crate) fn validate(project: &ProjectState) -> Result<()> {
    let bindings = &project.preferences.key_bindings;
    if bindings.len() > 64 {
        return Err(invalid("Maximum 64 keyboard bindings"));
    }
    let keys = shortcut_keys();
    let mut used = HashSet::new();
    for binding in bindings {
        if !keys.contains(&binding.key) {
            return Err(invalid("Unsupported shortcut key"));
        }
        if !used.insert((&binding.key, binding.ctrl, binding.shift, binding.alt)) {
            return Err(invalid("Two shortcuts cannot use the same key combination"));
        }
        if binding.ctrl && !binding.alt && matches!(binding.key.as_str(), "S" | "O" | "I") {
            return Err(invalid(
                "Ctrl+S, Ctrl+Shift+S, Ctrl+O and Ctrl+I are reserved",
            ));
        }
        match binding.action {
            ShortcutAction::TransitionSlot(slot) if slot >= 4 => {
                return Err(invalid("Shortcut transition slot is missing"));
            }
            ShortcutAction::Preview(id) if project.input(id).is_none() => {
                return Err(invalid("Shortcut input is missing"));
            }
            ShortcutAction::PreviewNumber(number) if !(1..=32).contains(&number) => {
                return Err(invalid("Shortcut input number must be 1–32"));
            }
            ShortcutAction::ToggleOverlay(slot) if slot >= crate::overlays::STANDARD_CHANNELS => {
                return Err(invalid("Shortcut overlay slot is missing"));
            }
            _ => {}
        }
    }
    Ok(())
}
impl Engine {
    pub fn shortcut_command(&self, action: &ShortcutAction) -> Result<Command> {
        Ok(match *action {
            ShortcutAction::Cut => Command::Cut,
            ShortcutAction::TransitionSlot(slot) if slot < 4 => Command::RunTransitionSlot(slot),
            ShortcutAction::FadeBlack => Command::FadeBlack,
            ShortcutAction::CancelTransition => Command::CancelTransition,
            ShortcutAction::ToggleOverlay(slot) => {
                let mut overlay = self
                    .project
                    .overlays
                    .standard(slot)
                    .cloned()
                    .ok_or_else(|| invalid("Shortcut overlay is missing"))?;
                if let Some(preview_input) = self.project.input(self.project.preview) {
                    let is_current = overlay.enabled && overlay.title == preview_input.name;
                    if is_current {
                        overlay.enabled = false;
                    } else {
                        overlay.enabled = true;
                        overlay.title = preview_input.name.clone();
                        overlay.subtitle = preview_input.source_label().into();
                    }
                } else {
                    overlay.enabled = !overlay.enabled;
                }
                Command::SetOverlay(slot, overlay)
            }
            ShortcutAction::Preview(id) if self.project.input(id).is_some() => {
                Command::SelectPreview(id)
            }
            ShortcutAction::PreviewNumber(number) => Command::SelectPreview(
                self.project
                    .inputs
                    .get(
                        number
                            .checked_sub(1)
                            .ok_or_else(|| invalid("Shortcut input is missing"))?,
                    )
                    .ok_or_else(|| invalid("Shortcut input is missing"))?
                    .id,
            ),
            _ => return Err(invalid("Shortcut target is missing")),
        })
    }
}
