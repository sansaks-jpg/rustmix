//! Engine-backed production controls. Dialog selection is UI state; switching is not.
use super::*;

pub(crate) fn consume_press(
    ctx: &egui::Context,
    modifiers: egui::Modifiers,
    key: egui::Key,
) -> bool {
    let pressed = ctx.input(|input| {
        input.events.iter().any(|event| {
            matches!(event,
        egui::Event::Key { key: k, pressed: true, repeat: false, modifiers: m, .. }
            if *k == key && m.matches_exact(modifiers))
        })
    });
    pressed && ctx.input_mut(|input| input.consume_key(modifiers, key))
}

fn action_label(action: ShortcutAction, project: &ProjectState) -> String {
    match action {
        ShortcutAction::Cut => "Cut".into(),
        ShortcutAction::TransitionSlot(n) => format!("Transition slot {}", n + 1),
        ShortcutAction::FadeBlack => "Fade to black / restore".into(),
        ShortcutAction::CancelTransition => "Cancel transition".into(),
        ShortcutAction::ToggleOverlay(n) => format!("Toggle overlay {}", n + 1),
        ShortcutAction::PreviewNumber(n) => format!("Preview input number {n}"),
        ShortcutAction::Preview(id) => project.input(id).map_or_else(
            || "Missing input".into(),
            |input| format!("Preview: {}", input.name),
        ),
    }
}

impl RustMixApp {
    pub(crate) fn playlist_toggle_command(&self) -> Command {
        if self.engine.playlist_run().is_some() {
            Command::StopPlaylist
        } else {
            Command::StartPlaylist {
                id: self.playlist_selected_id,
                selected: self.playlist_selected_item,
            }
        }
    }

    pub(crate) fn dispatch_key_bindings(&mut self, ctx: &egui::Context) {
        if self.ui_locked {
            return;
        }
        let bindings = self.engine.project().preferences.key_bindings.clone();
        for binding in bindings {
            let Some(key) = egui::Key::from_name(&binding.key) else {
                continue;
            };
            let modifiers = egui::Modifiers {
                ctrl: binding.ctrl,
                shift: binding.shift,
                alt: binding.alt,
                ..Default::default()
            };
            // Exact modifiers and no repeats: holding a take key cannot switch twice.
            if consume_press(ctx, modifiers, key) {
                match self.engine.shortcut_command(&binding.action) {
                    Ok(command) => self.actions.push(Action::Engine(command)),
                    Err(error) => self.say(error.to_string(), true),
                }
            }
        }
    }

    pub(crate) fn shortcut_settings(&mut self, ui: &mut egui::Ui, draft: &mut Preferences) {
        ui.label(RichText::new("Keyboard shortcuts").strong());
        ui.label("Changes apply with OK. Ctrl+S / Ctrl+Shift+S / Ctrl+O / Ctrl+I are reserved.");
        ui.checkbox(&mut draft.shortcuts, "Enable production shortcuts");
        let keys = shortcuts::shortcut_keys();
        let mut actions = vec![
            ShortcutAction::Cut,
            ShortcutAction::FadeBlack,
            ShortcutAction::CancelTransition,
        ];
        actions.extend((0..4).map(ShortcutAction::TransitionSlot));
        actions.extend((0..4).map(ShortcutAction::ToggleOverlay));
        actions.extend((1..=7).map(ShortcutAction::PreviewNumber));
        actions.extend(
            self.engine
                .project()
                .inputs
                .iter()
                .map(|input| ShortcutAction::Preview(input.id)),
        );
        let mut remove = None;
        egui::Grid::new("settings.bindings")
            .striped(true)
            .show(ui, |ui| {
                for (n, binding) in draft.key_bindings.iter_mut().enumerate() {
                    egui::ComboBox::from_id_salt(("shortcut.key", n))
                        .width(60.0)
                        .selected_text(&binding.key)
                        .show_ui(ui, |ui| {
                            for key in &keys {
                                ui.selectable_value(&mut binding.key, key.clone(), key);
                            }
                        });
                    ui.checkbox(&mut binding.ctrl, "Ctrl");
                    ui.checkbox(&mut binding.shift, "Shift");
                    ui.checkbox(&mut binding.alt, "Alt");
                    egui::ComboBox::from_id_salt(("shortcut.action", n))
                        .width(200.0)
                        .selected_text(action_label(binding.action, self.engine.project()))
                        .show_ui(ui, |ui| {
                            for action in &actions {
                                ui.selectable_value(
                                    &mut binding.action,
                                    *action,
                                    action_label(*action, self.engine.project()),
                                );
                            }
                        });
                    let button = ui.button("Remove");
                    self.automation
                        .insert(format!("dialog.settings.binding.remove.{n}"), button.rect);
                    if button.clicked() {
                        remove = Some(n);
                    }
                    ui.end_row();
                }
            });
        if let Some(index) = remove {
            draft.key_bindings.remove(index);
        }
        ui.horizontal(|ui| {
            let add = ui.add_enabled(
                draft.key_bindings.len() < 64,
                egui::Button::new("Add shortcut"),
            );
            self.automation
                .insert("dialog.settings.binding.add".into(), add.rect);
            if add.clicked()
                && let Some(key) = keys.iter().find(|key| {
                    !draft
                        .key_bindings
                        .iter()
                        .any(|b| &b.key == *key && !b.ctrl && !b.shift && !b.alt)
                })
            {
                draft.key_bindings.push(ShortcutBinding {
                    key: key.clone(),
                    ctrl: false,
                    shift: false,
                    alt: false,
                    action: ShortcutAction::Cut,
                });
            }
            if ui.button("Restore default keys").clicked() {
                draft.key_bindings = default_key_bindings();
            }
        });
        let mut candidate = self.engine.project().clone();
        candidate.preferences = draft.clone();
        if let Err(error) = candidate.validate() {
            ui.colored_label(RED, error.to_string());
        }
    }

    pub(crate) fn playlist_dialog(&mut self, ctx: &egui::Context) {
        if !self.playlist_open {
            return;
        }
        let lists = self.engine.project().playlists.clone();
        let Some(mut draft) = lists
            .iter()
            .find(|p| p.id == self.playlist_selected_id)
            .or_else(|| lists.first())
            .cloned()
        else {
            return;
        };
        if self.playlist_selected_id != draft.id {
            self.playlist_selected_id = draft.id;
            self.playlist_name = draft.name.clone();
        }
        let original = draft.clone();
        let running = self
            .engine
            .playlist_run()
            .is_some_and(|run| run.id == draft.id);
        let mut open = true;
        let mut close = false;
        let mut delete = false;
        dialogs::operator_window(ctx, "Playlist", "dialog.playlist", &mut open, Vec2::new(820.0, 550.0))
            .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                let new = ui.add_enabled(lists.len() < 16, egui::Button::new("New playlist"));
                self.automation.insert("dialog.playlist.new".into(), new.rect);
                if new.clicked() {
                    let name = (1..=17).map(|n| format!("Playlist {n}"))
                        .find(|name| !lists.iter().any(|p| p.name.eq_ignore_ascii_case(name)));
                    if let Some(name) = name {
                        self.actions.push(Action::Engine(Command::AddPlaylist(name.clone())));
                        if let Some(id) = lists.iter().map(|p| p.id).max().and_then(|n| n.checked_add(1)) {
                            self.playlist_selected_id = id;
                            self.playlist_selected_item = 0;
                            self.playlist_name = name;
                        }
                    }
                }
                let remove = ui.add_enabled(!running && lists.len() > 1, egui::Button::new("Delete playlist"));
                self.automation.insert("dialog.playlist.delete".into(), remove.rect);
                if remove.clicked() { self.playlist_delete_confirm = true; }
                egui::ComboBox::from_id_salt("playlist.select").width(230.0).selected_text(&draft.name)
                    .show_ui(ui, |ui| {
                    for list in &lists {
                        if ui.selectable_label(list.id == draft.id, &list.name).clicked() {
                            self.playlist_selected_id = list.id;
                            self.playlist_name = list.name.clone();
                            self.playlist_selected_item = 0;
                            self.playlist_delete_confirm = false;
                        }
                    }
                });
            });
            if self.playlist_delete_confirm {
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("Remove playlist '{}' and its item list?", draft.name));
                    if ui.button("Keep playlist").clicked() { self.playlist_delete_confirm = false; }
                    if ui.button("Confirm delete").clicked() { delete = true; }
                });
            }
            if let Some(run) = self.engine.playlist_run() {
                ui.colored_label(TEAL, format!("Running playlist {} · item {} · {}", run.id, run.index + 1,
                    if run.hold_started_ms.is_some() { "holding" } else { "transitioning" }));
            }
            if let Some(error) = self.engine.playlist_alarm() { ui.colored_label(RED, error); }
            let height = (ctx.viewport_rect().height() - 240.0).clamp(95.0, 330.0);
            egui::ScrollArea::vertical().id_salt("playlist.body").max_height(height).show(ui, |ui| {
                ui.add_enabled_ui(!running && !self.playlist_delete_confirm, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.add(egui::TextEdit::singleline(&mut self.playlist_name).desired_width(230.0));
                        if ui.button("Rename").clicked() { draft.name = self.playlist_name.clone(); }
                    });
                    let inputs = self.engine.project().inputs.clone();
                    ui.columns(2, |columns| {
                        if let [left, right] = columns {
                            left.strong("Available inputs");
                            egui::ScrollArea::vertical().id_salt("playlist.available").max_height(140.0)
                                .min_scrolled_height(90.0).show(left, |ui| {
                                for (index, input) in inputs.iter().enumerate() {
                                    let row = ui.selectable_label(index == self.playlist_selected_avail,
                                        format!("{} · {}{}", index + 1, input.name, if input.online { "" } else { " [OFFLINE]" }));
                                    self.automation.insert(format!("dialog.playlist.available.{}", input.id.0), row.rect);
                                    if row.clicked() { self.playlist_selected_avail = index; }
                                }
                            });
                            let add = left.add_enabled(draft.items.len() < 128 && !inputs.is_empty(), egui::Button::new("Add →"));
                            self.automation.insert("dialog.playlist.add".into(), add.rect);
                            if add.clicked() && let Some(input) = inputs.get(self.playlist_selected_avail) {
                                draft.items.push(PlaylistItem { input: input.id, hold_ms: 3500,
                                    transition: draft_default_slot(self.engine.project()) });
                                self.playlist_selected_item = draft.items.len().saturating_sub(1);
                            }
                            right.strong("Playlist order");
                            egui::ScrollArea::vertical().id_salt("playlist.items").max_height(140.0)
                                .min_scrolled_height(90.0).show(right, |ui| {
                                for (index, item) in draft.items.iter().enumerate() {
                                    let name = inputs.iter().find(|p| p.id == item.input).map_or("Missing input", |p| p.name.as_str());
                                    let row = ui.selectable_label(index == self.playlist_selected_item,
                                        format!("{} · {} · {:.1}s", index+1, name, item.hold_ms as f64 / 1000.0));
                                    self.automation.insert(format!("dialog.playlist.item.{index}"), row.rect);
                                    if row.clicked() { self.playlist_selected_item = index; }
                                }
                            });
                            right.horizontal(|ui| {
                                let index = self.playlist_selected_item;
                                let valid = index < draft.items.len();
                                for (id, label, available) in [("remove", "Remove", valid),
                                    ("up", "Up", valid && index > 0), ("down", "Down", valid && index+1 < draft.items.len())] {
                                    let button = ui.add_enabled(available, egui::Button::new(label));
                                    self.automation.insert(format!("dialog.playlist.{id}"), button.rect);
                                    if button.clicked() {
                                        match id {
                                            "up" => { draft.items.swap(index, index-1); self.playlist_selected_item -= 1; }
                                            "down" => { draft.items.swap(index, index+1); self.playlist_selected_item += 1; }
                                            _ => { draft.items.remove(index); self.playlist_selected_item = index.min(draft.items.len().saturating_sub(1)); }
                                        }
                                    }
                                }
                            });
                        }
                    });
                    if let Some(item) = draft.items.get_mut(self.playlist_selected_item) {
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Hold (ms)");
                            ui.add(egui::DragValue::new(&mut item.hold_ms).range(100..=86_400_000).speed(100));
                            effect_picker(ui, "playlist.effect", &mut item.transition.kind);
                            ui.label("Transition (ms)");
                            ui.add(egui::DragValue::new(&mut item.transition.duration_ms).range(100..=10_000).speed(50));
                        });
                    }
                    ui.horizontal_wrapped(|ui| {
                        for (id, value, label) in [
                            ("loop", &mut draft.looped, "Loop"),
                            ("beginSelected", &mut draft.begin_selected, "Begin from selected item"),
                            ("clearOverlays", &mut draft.clear_overlays, "Clear overlays at start"),
                            ("manual", &mut draft.manual, "Manual mode"),
                        ] {
                            let response = ui.checkbox(value, label);
                            self.automation.insert(format!("dialog.playlist.{id}"), response.rect);
                        }
                    });
                });
                ui.label("Edits save with the project. Hold starts after each transition. Stop keeps the current take intact.");
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                let toggle = self.playlist_toggle_command();
                self.command_button(ui, "dialog.playlist.start", if self.engine.playlist_run().is_some() { "Stop" } else { "Start" }, toggle, Some(if running { RED } else { PROGRAM }));
                ui.add_enabled_ui(running, |ui| {
                    self.command_button(ui, "dialog.playlist.previous", "Previous", Command::PlaylistPrevious, None);
                    self.command_button(ui, "dialog.playlist.next", "Next", Command::PlaylistNext, None);
                });
                let button = ui.button("Close");
                self.automation.insert("dialog.playlist.close".into(), button.rect);
                if button.clicked() { close = true; }
            });
        });
        if delete {
            self.actions
                .push(Action::Engine(Command::RemovePlaylist(draft.id)));
            self.playlist_delete_confirm = false;
        } else if draft != original {
            // Commit edits before any Start/Next command queued in the same frame.
            self.actions
                .insert(0, Action::Engine(Command::UpdatePlaylist(draft)));
        }
        if close || !open {
            self.playlist_open = false;
        }
    }
}

fn draft_default_slot(project: &ProjectState) -> TransitionSlot {
    project
        .transition_slots
        .first()
        .copied()
        .unwrap_or(TransitionSlot {
            kind: TransitionKind::Fade,
            duration_ms: 1000,
        })
}
pub(crate) fn effect_picker(ui: &mut egui::Ui, id: &str, kind: &mut TransitionKind) {
    egui::ComboBox::from_id_salt(id)
        .width(135.0)
        .selected_text(kind.label())
        .show_ui(ui, |ui| {
            for candidate in TransitionKind::ALL {
                ui.selectable_value(kind, candidate, candidate.label());
            }
        });
}
