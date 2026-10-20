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
        actions.extend((0..8).map(ShortcutAction::ToggleOverlay));
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
        dialogs::operator_window(
            ctx,
            "Playlist",
            "dialog.playlist",
            &mut open,
            Vec2::new(
                (ctx.viewport_rect().width() * 0.60).max(640.0),
                (ctx.viewport_rect().height() * 0.55).clamp(360.0, 680.0),
            ),
        )
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                let new = ui
                    .add_enabled_ui(lists.len() < 16, |ui| {
                        ui.add_sized([110.0, 29.0], egui::Button::new("New playlist"))
                    })
                    .inner;
                self.automation
                    .insert("dialog.playlist.new".into(), new.rect);
                if new.clicked() {
                    let name = (1..=17).map(|n| format!("Playlist {n}")).find(|name| {
                        !lists
                            .iter()
                            .any(|p| p.name.trim().eq_ignore_ascii_case(name))
                    });
                    if let Some(name) = name {
                        self.actions
                            .push(Action::Engine(Command::AddPlaylist(name.clone())));
                        if let Some(id) = lists
                            .iter()
                            .map(|p| p.id)
                            .max()
                            .and_then(|n| n.checked_add(1))
                        {
                            self.playlist_selected_id = id;
                            self.playlist_selected_item = 0;
                            self.playlist_name = name;
                        }
                    }
                }
                let remove = ui
                    .add_enabled_ui(!running && lists.len() > 1, |ui| {
                        ui.add_sized([118.0, 29.0], egui::Button::new("Delete playlist"))
                    })
                    .inner;
                self.automation
                    .insert("dialog.playlist.delete".into(), remove.rect);
                if remove.clicked() {
                    self.playlist_delete_confirm = true;
                }
                egui::ComboBox::from_id_salt("playlist.select")
                    .width(230.0)
                    .selected_text(&draft.name)
                    .show_ui(ui, |ui| {
                        for list in &lists {
                            if ui
                                .selectable_label(list.id == draft.id, &list.name)
                                .clicked()
                            {
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
                    ui.label(format!(
                        "Remove playlist '{}' and its item list?",
                        draft.name
                    ));
                    if ui.button("Keep playlist").clicked() {
                        self.playlist_delete_confirm = false;
                    }
                    if ui.button("Confirm delete").clicked() {
                        delete = true;
                    }
                });
            }
            if let Some(run) = self.engine.playlist_run() {
                ui.colored_label(
                    TEAL,
                    format!(
                        "Running playlist {} · item {} · {}",
                        run.id,
                        run.index + 1,
                        if run.hold_started_ms.is_some() {
                            "holding"
                        } else {
                            "transitioning"
                        }
                    ),
                );
            }
            if let Some(error) = self.engine.playlist_alarm() {
                ui.colored_label(RED, error);
            }
            let height = (ctx.viewport_rect().height() - 190.0).clamp(180.0, 620.0);
            let pane_height = (ctx.viewport_rect().height() * 0.33).clamp(110.0, 400.0);
            egui::ScrollArea::vertical()
                .id_salt("playlist.body")
                .max_height(height)
                .show(ui, |ui| {
                    ui.add_enabled_ui(!running && !self.playlist_delete_confirm, |ui| {
                        let inputs = self.engine.project().inputs.clone();
                        let transfer_width = (ui.available_width() * 0.08).clamp(52.0, 80.0);
                        let reorder_width = 32.0;
                        let gap = 6.0;
                        let panes_width =
                            (ui.available_width() - transfer_width - reorder_width - gap * 3.0)
                                .max(300.0);
                        let available_width = panes_width * 0.436;
                        let ordered_width = panes_width - available_width;
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = gap;
                            ui.allocate_ui_with_layout(
                                Vec2::new(available_width, pane_height + 24.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(available_width);
                                    ui.strong("Available inputs");
                                    let available = dialogs::light_form_frame(ui, 4, |ui| {
                                        egui::ScrollArea::vertical()
                                            .id_salt("playlist.available")
                                            .max_height(pane_height)
                                            .min_scrolled_height(pane_height)
                                            .auto_shrink([false, false])
                                            .show(ui, |ui| {
                                                ui.with_layout(
                                                    egui::Layout::top_down_justified(
                                                        egui::Align::Min,
                                                    ),
                                                    |ui| {
                                                        for (index, input) in
                                                            inputs.iter().enumerate()
                                                        {
                                                            let row = ui.selectable_label(
                                                                index
                                                                    == self.playlist_selected_avail,
                                                                format!(
                                                                    "{} · {}{}",
                                                                    index + 1,
                                                                    input.name,
                                                                    if input.online {
                                                                        ""
                                                                    } else {
                                                                        " [OFFLINE]"
                                                                    }
                                                                ),
                                                            );
                                                            self.automation.insert(
                                                                format!(
                                                                    "dialog.playlist.available.{}",
                                                                    input.id.0
                                                                ),
                                                                row.rect,
                                                            );
                                                            if row.clicked() {
                                                                self.playlist_selected_avail =
                                                                    index;
                                                            }
                                                        }
                                                    },
                                                );
                                            });
                                    });
                                    self.automation.insert(
                                        "dialog.playlist.available.body".into(),
                                        available.response.rect,
                                    );
                                },
                            );
                            ui.allocate_ui_with_layout(
                                Vec2::new(transfer_width, pane_height + 24.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(transfer_width);
                                    ui.add_space(20.0 + (pane_height - 66.0).max(0.0) * 0.5);
                                    let add = ui
                                        .add_enabled_ui(
                                            draft.items.len() < 128 && !inputs.is_empty(),
                                            |ui| {
                                                ui.add_sized(
                                                    [transfer_width, 30.0],
                                                    egui::Button::new("→"),
                                                )
                                            },
                                        )
                                        .inner
                                        .on_hover_text("Add selected input to playlist");
                                    self.automation
                                        .insert("dialog.playlist.add".into(), add.rect);
                                    if add.clicked()
                                        && let Some(input) =
                                            inputs.get(self.playlist_selected_avail)
                                    {
                                        draft.items.push(PlaylistItem {
                                            input: input.id,
                                            hold_ms: 3500,
                                            transition: draft_default_slot(self.engine.project()),
                                        });
                                        self.playlist_selected_item =
                                            draft.items.len().saturating_sub(1);
                                    }
                                    let valid = self.playlist_selected_item < draft.items.len();
                                    let remove = ui
                                        .add_enabled_ui(valid, |ui| {
                                            ui.add_sized(
                                                [transfer_width, 30.0],
                                                egui::Button::new("←"),
                                            )
                                        })
                                        .inner
                                        .on_hover_text("Remove selected playlist item");
                                    self.automation
                                        .insert("dialog.playlist.remove".into(), remove.rect);
                                    if remove.clicked() {
                                        draft.items.remove(self.playlist_selected_item);
                                        self.playlist_selected_item = self
                                            .playlist_selected_item
                                            .min(draft.items.len().saturating_sub(1));
                                    }
                                },
                            );
                            ui.allocate_ui_with_layout(
                                Vec2::new(ordered_width, pane_height + 24.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(ordered_width);
                                    ui.strong("Playlist order");
                                    let ordered = dialogs::light_form_frame(ui, 4, |ui| {
                                        egui::ScrollArea::vertical()
                                            .id_salt("playlist.items")
                                            .max_height(pane_height)
                                            .min_scrolled_height(pane_height)
                                            .auto_shrink([false, false])
                                            .show(ui, |ui| {
                                                ui.with_layout(
                                                    egui::Layout::top_down_justified(
                                                        egui::Align::Min,
                                                    ),
                                                    |ui| {
                                                        for (index, item) in
                                                            draft.items.iter().enumerate()
                                                        {
                                                            let name = inputs
                                                                .iter()
                                                                .find(|p| p.id == item.input)
                                                                .map_or("Missing input", |p| {
                                                                    p.name.as_str()
                                                                });
                                                            let row = ui.selectable_label(
                                                                index
                                                                    == self.playlist_selected_item,
                                                                format!(
                                                                    "{} · {} · {:.1}s",
                                                                    index + 1,
                                                                    name,
                                                                    item.hold_ms as f64 / 1000.0
                                                                ),
                                                            );
                                                            self.automation.insert(
                                                                format!(
                                                                    "dialog.playlist.item.{index}"
                                                                ),
                                                                row.rect,
                                                            );
                                                            if row.clicked() {
                                                                self.playlist_selected_item = index;
                                                            }
                                                        }
                                                    },
                                                );
                                            });
                                    });
                                    self.automation.insert(
                                        "dialog.playlist.items.body".into(),
                                        ordered.response.rect,
                                    );
                                    ui.horizontal_wrapped(|ui| {
                                        for (id, value, label) in [
                                            ("loop", &mut draft.looped, "Loop"),
                                            (
                                                "beginSelected",
                                                &mut draft.begin_selected,
                                                "Begin from selected item",
                                            ),
                                            (
                                                "clearOverlays",
                                                &mut draft.clear_overlays,
                                                "Clear overlays at start",
                                            ),
                                            ("manual", &mut draft.manual, "Manual mode"),
                                        ] {
                                            let response = ui.checkbox(value, label);
                                            self.automation.insert(
                                                format!("dialog.playlist.{id}"),
                                                response.rect,
                                            );
                                        }
                                    });
                                },
                            );
                            ui.allocate_ui_with_layout(
                                Vec2::new(reorder_width, pane_height + 24.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(reorder_width);
                                    ui.add_space(20.0 + (pane_height - 66.0).max(0.0) * 0.5);
                                    let index = self.playlist_selected_item;
                                    let valid = index < draft.items.len();
                                    for (id, label, available) in [
                                        ("up", "▲", valid && index > 0),
                                        ("down", "▼", valid && index + 1 < draft.items.len()),
                                    ] {
                                        let button = ui
                                            .add_enabled_ui(available, |ui| {
                                                ui.add_sized(
                                                    [reorder_width, 30.0],
                                                    egui::Button::new(label),
                                                )
                                            })
                                            .inner;
                                        self.automation
                                            .insert(format!("dialog.playlist.{id}"), button.rect);
                                        if button.clicked() {
                                            if id == "up" {
                                                draft.items.swap(index, index - 1);
                                                self.playlist_selected_item -= 1;
                                            } else {
                                                draft.items.swap(index, index + 1);
                                                self.playlist_selected_item += 1;
                                            }
                                        }
                                    }
                                },
                            );
                        });
                        let details = egui::CollapsingHeader::new("Playlist name and item timing")
                            .id_salt("playlist.details")
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label("Name");
                                    let name = ui.add(
                                        egui::TextEdit::singleline(&mut self.playlist_name)
                                            .desired_width(230.0),
                                    );
                                    self.automation
                                        .insert("dialog.playlist.name".into(), name.rect);
                                    let rename = ui.button("Rename");
                                    self.automation
                                        .insert("dialog.playlist.rename".into(), rename.rect);
                                    if rename.clicked() {
                                        draft.name = self.playlist_name.clone();
                                    }
                                });
                                if let Some(item) = draft.items.get_mut(self.playlist_selected_item)
                                {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label("Hold (ms)");
                                        ui.add(
                                            egui::DragValue::new(&mut item.hold_ms)
                                                .range(100..=86_400_000)
                                                .speed(100),
                                        );
                                        effect_picker(
                                            ui,
                                            "playlist.effect",
                                            &mut item.transition.kind,
                                        );
                                        ui.label("Transition (ms)");
                                        ui.add(
                                            egui::DragValue::new(&mut item.transition.duration_ms)
                                                .range(100..=10_000)
                                                .speed(50),
                                        );
                                    });
                                }
                            });
                        self.automation.insert(
                            "dialog.playlist.details".into(),
                            details.header_response.rect,
                        );
                    });
                });
            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let button = ui.add_sized([92.0, 29.0], egui::Button::new("Close"));
                self.automation
                    .insert("dialog.playlist.close".into(), button.rect);
                if button.clicked() {
                    close = true;
                }
                ui.add_enabled_ui(running, |ui| {
                    self.command_button(
                        ui,
                        "dialog.playlist.next",
                        "Next",
                        Command::PlaylistNext,
                        None,
                    );
                    self.command_button(
                        ui,
                        "dialog.playlist.previous",
                        "Previous",
                        Command::PlaylistPrevious,
                        None,
                    );
                });
                let toggle = self.playlist_toggle_command();
                self.command_button(
                    ui,
                    "dialog.playlist.start",
                    if self.engine.playlist_run().is_some() {
                        "Stop"
                    } else {
                        "Start"
                    },
                    toggle,
                    Some(if running { RED } else { PROGRAM }),
                );
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

#[cfg(test)]
mod playlist_layout_tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn frame(app: &mut RustMixApp, ctx: &egui::Context, size: Vec2, events: Vec<egui::Event>) {
        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |_| {
                app.automation.clear();
                app.playlist_dialog(ctx);
                for action in std::mem::take(&mut app.actions) {
                    app.dispatch(action, ctx);
                }
            },
        );
    }

    fn click(app: &mut RustMixApp, ctx: &egui::Context, id: &str) {
        let size = Vec2::new(1600.0, 900.0);
        for _ in 0..12 {
            frame(app, ctx, size, vec![]);
        }
        let rect = *app.automation.get(id).unwrap();
        assert!(ctx.viewport_rect().contains_rect(rect), "{id}");
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                size,
                vec![
                    egui::Event::PointerMoved(rect.center()),
                    egui::Event::PointerButton {
                        pos: rect.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }

    fn items(app: &RustMixApp) -> Vec<InputId> {
        app.engine
            .project()
            .playlists
            .first()
            .unwrap()
            .items
            .iter()
            .map(|item| item.input)
            .collect()
    }

    #[test]
    fn playlist_panes_transfer_reorder_and_footer_fit_across_viewports() {
        for size in [
            Vec2::new(734.0, 466.0),
            Vec2::new(1100.0, 700.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(1920.0, 1080.0),
        ] {
            let mut app = RustMixApp::headless();
            app.playlist_open = true;
            let program = app.engine.project().program;
            let preview = app.engine.project().preview;
            let ctx = egui::Context::default();
            app.theme(&ctx);
            for _ in 0..4 {
                frame(&mut app, &ctx, size, vec![]);
            }
            let bounds = |id: &str| *app.automation.get(id).unwrap();
            let available = bounds("dialog.playlist.available.body");
            let ordered = bounds("dialog.playlist.items.body");
            let close = bounds("dialog.playlist.close");
            let up = bounds("dialog.playlist.up");
            let pane_height = (size.y * 0.33).clamp(110.0, 400.0);
            for pane in [available, ordered] {
                assert!(
                    ctx.viewport_rect().contains_rect(pane),
                    "pane at {size:?}: {pane:?}"
                );
                assert!(pane.height() >= pane_height);
                assert!(pane.bottom() < close.top());
            }
            assert!((available.top() - ordered.top()).abs() <= 1.0);
            assert!(ordered.width() > available.width());
            for id in ["dialog.playlist.add", "dialog.playlist.remove"] {
                let transfer = bounds(id);
                assert!(available.right() <= transfer.left());
                assert!(transfer.right() <= ordered.left());
                assert!(transfer.height() >= 29.0);
                assert!(ctx.viewport_rect().contains_rect(transfer));
            }
            assert!(ordered.right() <= up.left());
            assert!(close.right() >= up.right() - 1.0);
            assert!(close.width() >= 92.0 && close.height() >= 29.0);
            for id in [
                "dialog.playlist.start",
                "dialog.playlist.previous",
                "dialog.playlist.next",
                "dialog.playlist.close",
            ] {
                assert!(
                    ctx.viewport_rect().contains_rect(bounds(id)),
                    "footer {id} at {size:?}"
                );
            }
            assert_eq!(app.engine.project().program, program);
            assert_eq!(app.engine.project().preview, preview);
        }
    }

    #[test]
    fn playlist_rows_fill_pane_width_keep_text_left_and_accept_far_edge_clicks() {
        let mut app = RustMixApp::headless();
        app.playlist_open = true;
        let mut playlist = app.engine.project().playlists.first().unwrap().clone();
        for input in [InputId(2), InputId(3)] {
            playlist.items.push(PlaylistItem {
                input,
                hold_ms: 3500,
                transition: draft_default_slot(app.engine.project()),
            });
        }
        app.engine
            .execute(Command::UpdatePlaylist(playlist), 0)
            .unwrap();
        let program = app.engine.project().program;
        let preview = app.engine.project().preview;
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let size = Vec2::new(1600.0, 900.0);
        for _ in 0..4 {
            frame(&mut app, &ctx, size, vec![]);
        }
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                ..Default::default()
            },
            |_| {
                app.automation.clear();
                app.playlist_dialog(&ctx);
            },
        );
        for (pane_id, row_ids, caption) in [
            (
                "dialog.playlist.available.body",
                ["dialog.playlist.available.1", "dialog.playlist.available.2"],
                "1 · Camera A",
            ),
            (
                "dialog.playlist.items.body",
                ["dialog.playlist.item.0", "dialog.playlist.item.1"],
                "1 · Camera B · 3.5s",
            ),
        ] {
            let pane = *app.automation.get(pane_id).unwrap();
            for id in row_ids {
                let row = *app.automation.get(id).unwrap();
                assert!(
                    row.width() >= pane.width() - 26.0,
                    "full pane hit area: {id}"
                );
                assert!(row.width() <= pane.width() - 8.0, "retain pane inset: {id}");
                assert!((row.left() - pane.left() - 5.0).abs() <= 1.0);
                assert!(pane.contains_rect(row));
            }
            let row = *app.automation.get(row_ids[0]).unwrap();
            let text = output
                .shapes
                .iter()
                .find_map(|shape| {
                    if let egui::Shape::Text(text) = &shape.shape
                        && text.galley.job.text == caption
                    {
                        Some(text.visual_bounding_rect())
                    } else {
                        None
                    }
                })
                .unwrap();
            assert!(
                text.left() >= row.left() && text.left() <= row.left() + 12.0,
                "caption starts at the left inset: {caption}"
            );
            assert!(
                text.right() < row.center().x,
                "short captions must not be centered"
            );
        }
        for id in ["dialog.playlist.available.2", "dialog.playlist.item.1"] {
            let row = *app.automation.get(id).unwrap();
            let pos = egui::pos2(row.right() - 2.0, row.center().y);
            for pressed in [true, false] {
                frame(
                    &mut app,
                    &ctx,
                    size,
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                );
            }
        }
        assert_eq!(app.playlist_selected_avail, 1);
        assert_eq!(app.playlist_selected_item, 1);
        assert_eq!(app.engine.project().program, program);
        assert_eq!(app.engine.project().preview, preview);
    }

    #[test]
    fn playlist_transfer_and_reorder_widgets_commit_without_taking_program() {
        let mut app = RustMixApp::headless();
        app.playlist_open = true;
        let ctx = egui::Context::default();
        app.theme(&ctx);
        let program = app.engine.project().program;
        let preview = app.engine.project().preview;
        assert!(items(&app).is_empty());
        click(&mut app, &ctx, "dialog.playlist.available.2");
        click(&mut app, &ctx, "dialog.playlist.add");
        click(&mut app, &ctx, "dialog.playlist.available.3");
        click(&mut app, &ctx, "dialog.playlist.add");
        assert_eq!(items(&app), vec![InputId(2), InputId(3)]);
        click(&mut app, &ctx, "dialog.playlist.up");
        assert_eq!(items(&app), vec![InputId(3), InputId(2)]);
        click(&mut app, &ctx, "dialog.playlist.down");
        assert_eq!(items(&app), vec![InputId(2), InputId(3)]);
        click(&mut app, &ctx, "dialog.playlist.remove");
        assert_eq!(items(&app), vec![InputId(2)]);
        click(&mut app, &ctx, "dialog.playlist.remove");
        assert!(items(&app).is_empty());
        for id in [
            "dialog.playlist.remove",
            "dialog.playlist.up",
            "dialog.playlist.down",
        ] {
            click(&mut app, &ctx, id);
        }
        assert!(items(&app).is_empty());
        click(&mut app, &ctx, "dialog.playlist.details");
        app.playlist_name = "Operator rundown".into();
        click(&mut app, &ctx, "dialog.playlist.rename");
        assert_eq!(
            app.engine.project().playlists.first().unwrap().name,
            "Operator rundown"
        );
        assert_eq!(app.engine.project().program, program);
        assert_eq!(app.engine.project().preview, preview);
    }
}
