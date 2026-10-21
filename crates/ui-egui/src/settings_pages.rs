//! Page-specific settings summaries; no unrelated controls on unavailable pages.
use super::*;

fn section(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.label(RichText::new(title).strong().size(14.0));
    ui.add_space(6.0);
    ui.label(description);
    ui.add_space(10.0);
}

fn summary(ui: &mut egui::Ui, id: &str, rows: &[(&str, String)]) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([18.0, 9.0])
        .show(ui, |ui| {
            for (label, value) in rows {
                ui.label(*label);
                ui.add(egui::Label::new(value).wrap());
                ui.end_row();
            }
        });
}

impl RustMixApp {
    pub(super) fn settings_detail_page(
        &mut self,
        ui: &mut egui::Ui,
        page: SettingsPage,
        draft: &mut Preferences,
    ) {
        match page {
            SettingsPage::Options => {
                section(ui, "Options", "Operator controls for this production.");
                let shortcuts = ui.checkbox(
                    &mut draft.shortcuts,
                    "Enable single-key production shortcuts",
                );
                self.automation
                    .insert("dialog.settings.options.shortcuts".into(), shortcuts.rect);
                ui.label("Shortcuts are suspended while editing a dialog or text field.");
                ui.add_space(12.0);
                summary(
                    ui,
                    "settings.options.summary",
                    &[
                        (
                            "Project files",
                            "Versioned RustMix JSON · atomic save with two previous generations"
                                .into(),
                        ),
                        (
                            "Recovery",
                            "Checkpoint recovery is available from the project controls".into(),
                        ),
                    ],
                );
            }
            SettingsPage::Outputs => {
                section(
                    ui,
                    "Outputs / NDI / OMT / SRT",
                    "Stored output routes. Transport and encoder backends are unavailable.",
                );
                egui::Grid::new("settings.outputs.summary")
                    .num_columns(3)
                    .spacing([18.0, 8.0])
                    .show(ui, |ui| {
                        ui.strong("Output");
                        ui.strong("Source");
                        ui.strong("Transport");
                        ui.end_row();
                        for output in self.engine.project().outputs.iter() {
                            ui.label(format!("External {}", output.number));
                            ui.label(match output.source {
                                OutputSource::None => "None".into(),
                                OutputSource::Program => "Program".into(),
                                OutputSource::Preview => "Preview".into(),
                                OutputSource::Input(id) => self
                                    .engine
                                    .project()
                                    .input(id)
                                    .map(|i| i.name.clone())
                                    .unwrap_or_else(|| "Missing input".into()),
                            });
                            ui.label("Unavailable");
                            ui.end_row();
                        }
                    });
                ui.add_space(12.0);
                ui.label("Edit routing from External → gear. NDI, OMT and SRT do not transmit media in this build.");
            }
            SettingsPage::Decoders => {
                section(
                    ui,
                    "Decoders",
                    "Source handling available in the current application.",
                );
                summary(
                    ui,
                    "settings.decoders.summary",
                    &[
                        (
                            "Local video",
                            "Windows Media Foundation · experimental video-only playback".into(),
                        ),
                        (
                            "Images",
                            "PNG stills; photo lists also accept JPEG / BMP".into(),
                        ),
                        ("Local audio", "WAV → 48 kHz stereo local mixer".into()),
                        ("Hardware decode", "Not qualified".into()),
                        (
                            "Third-party codecs",
                            "No bundled FFmpeg / GStreamer runtime".into(),
                        ),
                    ],
                );
            }
            SettingsPage::Recording => {
                section(
                    ui,
                    "Recording",
                    "Program recording requires an encoder/output backend.",
                );
                summary(
                    ui,
                    "settings.recording.summary",
                    &[
                        ("State", "Unavailable · no recording has started".into()),
                        (
                            "Recording setup",
                            "Record → gear opens format and output navigation".into(),
                        ),
                        ("Configuration", "Read-only · not saved".into()),
                        (
                            "Recovery / disk space",
                            "Not implemented for recordings".into(),
                        ),
                    ],
                );
            }
            SettingsPage::Audio | SettingsPage::AudioOutputs => {
                section(
                    ui,
                    if page == SettingsPage::Audio {
                        "Audio"
                    } else {
                        "Audio Outputs"
                    },
                    "Local monitoring and the saved routing matrix are separate from recording and streaming.",
                );
                summary(
                    ui,
                    "settings.audio.summary",
                    &[
                        ("Mix format", "48 kHz · stereo".into()),
                        ("Monitoring", self.audio_ui.status_label().into()),
                        ("Buses", "Master / A / B".into()),
                        (
                            "Legacy mixer meters",
                            "Simulated; Audio I/O shows actual sample peak / RMS".into(),
                        ),
                        ("Recording / streaming", "Unavailable".into()),
                    ],
                );
                ui.add_space(12.0);
                ui.label("Audio I/O is available from the Audio Mixer controls. Opening it does not start monitoring; Start opens the selected devices.");
            }
            SettingsPage::TallyLights => {
                section(
                    ui,
                    "Tally Lights",
                    "Input cards and monitor headers show Program / Preview state.",
                );
                summary(
                    ui,
                    "settings.tally.summary",
                    &[
                        ("Program", "PGM label and configured Program colour".into()),
                        ("Preview", "PVW label and configured Preview colour".into()),
                        (
                            "External tally",
                            "Unavailable · no hardware or web tally service".into(),
                        ),
                    ],
                );
            }
            SettingsPage::Activators => {
                section(
                    ui,
                    "Activators",
                    "Controller feedback is unavailable in this build.",
                );
                self.unavailable_settings_table(
                    ui,
                    "activators",
                    &["Control", "Event", "Input", "State"],
                    "No controller feedback backend",
                );
            }
            SettingsPage::Scripting => {
                section(
                    ui,
                    "Scripting",
                    "Script execution is unavailable. Existing engine commands remain typed and validated.",
                );
                self.unavailable_settings_table(
                    ui,
                    "scripting",
                    &["Name", "State", "Trigger"],
                    "No script runtime",
                );
            }
            SettingsPage::Alerts => {
                section(
                    ui,
                    "Alerts",
                    "Current session warnings and command results.",
                );
                if self.history.is_empty() {
                    ui.label("No session messages.");
                }
                for line in self.history.iter().rev().take(32) {
                    ui.label(line);
                }
                ui.add_space(10.0);
                ui.label(
                    "Email, external notifications and configurable alarm rules are unavailable.",
                );
            }
            SettingsPage::About => {
                section(
                    ui,
                    "RustMix",
                    "Independent Rust live-production software · experimental.",
                );
                summary(
                    ui,
                    "settings.about.summary",
                    &[
                        ("Version", env!("CARGO_PKG_VERSION").into()),
                        ("Project schema", SCHEMA.to_string()),
                        ("Interface", "Native egui / wgpu".into()),
                        (
                            "Implementation",
                            "Original branding and controls · not affiliated with vMix".into(),
                        ),
                        (
                            "Qualification",
                            "See feature matrix for tested scope and hardware limitations".into(),
                        ),
                    ],
                );
            }
            // These pages retain their dedicated editors in dialogs.rs.
            SettingsPage::Display
            | SettingsPage::Appearance
            | SettingsPage::Categories
            | SettingsPage::Workspace
            | SettingsPage::Shortcuts
            | SettingsPage::ExternalOutput => {}
        }
    }

    fn unavailable_settings_table(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        columns: &[&str],
        reason: &str,
    ) {
        egui::Grid::new(("settings.empty", id))
            .num_columns(columns.len())
            .spacing([18.0, 8.0])
            .show(ui, |ui| {
                for column in columns {
                    ui.strong(*column);
                }
                ui.end_row();
            });
        ui.add_space(8.0);
        ui.label(reason);
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            for label in ["Add", "Edit", "Delete"] {
                let action = ui
                    .add_enabled(
                        false,
                        egui::Button::new(label).min_size(Vec2::new(68.0, 27.0)),
                    )
                    .on_disabled_hover_text(reason);
                self.automation
                    .insert(format!("dialog.settings.{id}.{label}"), action.rect);
            }
        });
    }
}
