//! Output setup presentation. Navigation is available; absent encoders never start.
use super::*;
use dialogs::{light_form_frame, operator_window};
use operator_window::{FooterRole, footer_size};

const OUTPUT_UNAVAILABLE: &str =
    "No encoder/output backend. Configuration is read-only and is not saved.";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StreamPage {
    Main,
    Advanced,
    Status,
}

fn field_row<R>(ui: &mut egui::Ui, label: &str, contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.horizontal(|ui| {
        let width = (ui.available_width() * 0.20).clamp(104.0, 136.0);
        ui.allocate_ui_with_layout(
            Vec2::new(width, 24.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                ui.label(label);
            },
        );
        ui.add_space(4.0);
        contents(ui)
    })
    .inner
}

fn unavailable_button(ui: &mut egui::Ui, label: &str, size: Vec2) -> egui::Response {
    ui.add_enabled(false, egui::Button::new(label).fill(CHIP_BG).min_size(size))
        .on_disabled_hover_text(OUTPUT_UNAVAILABLE)
}

// Keep unavailable fields readable against the dark output setup surface.
// Opacity changes presentation only: the child remains disabled for all input.
fn read_only_field<R>(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.scope(|ui| {
        *ui.visuals_mut() = dialogs::form_visuals();
        ui.add_enabled_ui(false, |ui| {
            ui.set_opacity(1.0);
            content(ui)
        })
        .inner
    })
    .inner
}

impl RustMixApp {
    fn stream_main(&mut self, ui: &mut egui::Ui) {
        field_row(ui, "Destination", |ui| {
            let response = read_only_field(ui, |ui| {
                egui::ComboBox::from_id_salt("stream.destination")
                    .width(ui.available_width() - 8.0)
                    .selected_text(
                        [
                            "Custom RTMP Server",
                            "YouTube Live",
                            "Twitch",
                            "Facebook Live",
                            "SRT Caller",
                        ]
                        .get(self.stream_destination)
                        .copied()
                        .unwrap_or("Custom RTMP Server"),
                    )
                    .show_ui(ui, |_| {})
            })
            .response;
            self.automation
                .insert("dialog.streaming.destination".into(), response.rect);
        });
        for (label, key, secret) in [("URL", "url", false), ("Stream Name or Key", "key", true)] {
            let field = field_row(ui, label, |ui| {
                let text = if secret {
                    &mut self.stream_key
                } else {
                    &mut self.stream_url
                };
                read_only_field(ui, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(text)
                            .hint_text(if secret {
                                "No key configured"
                            } else {
                                "No destination configured"
                            })
                            .password(secret)
                            .desired_width(ui.available_width() - 8.0),
                    )
                })
            });
            self.automation
                .insert(format!("dialog.streaming.{key}"), field.rect);
        }
        ui.add_space(10.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let advanced = ui.add_sized([120.0, 28.0], egui::Button::new("Advanced"));
            self.automation
                .insert("dialog.streaming.advanced".into(), advanced.rect);
            if advanced.clicked() {
                self.stream_page = StreamPage::Advanced;
            }
        });
        ui.add_space(20.0);
        field_row(ui, "Quality", |ui| {
            let response = read_only_field(ui, |ui| {
                egui::ComboBox::from_id_salt("stream.quality")
                    .width(ui.available_width() - 8.0)
                    .selected_text(
                        [
                            "H264 720p / AAC",
                            "H264 1080p / AAC",
                            "H264 1080p high bitrate / AAC",
                        ]
                        .get(self.stream_quality)
                        .copied()
                        .unwrap_or("Not configured"),
                    )
                    .show_ui(ui, |_| {})
            })
            .response;
            self.automation
                .insert("dialog.streaming.quality".into(), response.rect);
        });
        if self.stream_channel != 0 {
            field_row(ui, "", |ui| {
                ui.add_enabled(
                    false,
                    egui::Checkbox::new(&mut self.stream_use_q1, "Use Stream 1 Quality"),
                );
            });
        }
        field_row(ui, "Encoder", |ui| {
            ui.label("Unavailable · no runtime configured");
        });
    }

    pub(crate) fn streaming_setup_dialog(&mut self, ctx: &egui::Context) {
        if !self.streaming_setup {
            return;
        }
        let mut open = true;
        let mut close_clicked = false;
        let body_height = (ctx.viewport_rect().height() - 225.0).clamp(180.0, 390.0);
        operator_window(ctx, "Streaming Settings", "dialog.streaming_setup", &mut open,
            Vec2::new((ctx.viewport_rect().width() * 0.44).clamp(650.0, 840.0), body_height + 105.0))
            .show(ctx, |ui| {
                ui.label(OUTPUT_UNAVAILABLE);
                let body = egui::ScrollArea::vertical().id_salt("stream.body")
                    .max_height(body_height).auto_shrink([false, false]).show(ui, |ui| {
                    field_row(ui, "Profile", |ui| {
                        let width = (ui.available_width() - 124.0).max(120.0);
                        read_only_field(ui, |ui| {
                            egui::ComboBox::from_id_salt("stream.profile").width(width)
                                .selected_text("Default").show_ui(ui, |_| {});
                        });
                        unavailable_button(ui, "Add", Vec2::new(50.0, 24.0));
                        unavailable_button(ui, "Delete", Vec2::new(58.0, 24.0));
                    });
                    field_row(ui, "Destination", |ui| {
                        for ch in 0..5 {
                            let tab = ui.add_sized([42.0, 28.0], egui::Button::new((ch + 1).to_string())
                                .selected(self.stream_channel == ch));
                            self.automation.insert(format!("dialog.streaming.slot.{}", ch + 1), tab.rect);
                            if tab.clicked() { self.stream_channel = ch; }
                        }
                    });
                    ui.add_space(8.0);
                    match self.stream_page {
                        StreamPage::Main => self.stream_main(ui),
                        StreamPage::Advanced => {
                            ui.label(RichText::new("Advanced · destination settings").strong());
                            light_form_frame(ui, 10, |ui| {
                                for (label, value) in [("Video encoder", "Unavailable"), ("Audio encoder", "Unavailable"),
                                    ("Audio bus", "Master (not connected)"), ("Reconnect", "Unavailable"),
                                    ("Hardware encoding", "Not qualified")] {
                                    field_row(ui, label, |ui| { ui.label(value); });
                                }
                            });
                            ui.label("Encoding, transport and retry options become available with a supported backend.");
                        }
                        StreamPage::Status => {
                            ui.label(RichText::new("Stream status").strong());
                            light_form_frame(ui, 10, |ui| {
                                egui::Grid::new("stream.status.table").num_columns(4).spacing([18.0, 8.0]).show(ui, |ui| {
                                    for heading in ["Destination", "State", "Bitrate", "Dropped frames"] { ui.strong(heading); }
                                    ui.end_row();
                                    for ch in 1..=5 {
                                        ui.label(ch.to_string()); ui.label("Unavailable"); ui.label("—"); ui.label("—"); ui.end_row();
                                    }
                                });
                            });
                            ui.label("No stream has started. Metrics are unavailable until an output worker reports them.");
                        }
                    }
                    if self.stream_page != StreamPage::Main {
                        let back = ui.button("Back to stream settings");
                        self.automation.insert("dialog.streaming.back".into(), back.rect);
                        if back.clicked() { self.stream_page = StreamPage::Main; }
                    }
                });
                self.automation.insert("dialog.streaming.body".into(), body.inner_rect);
                ui.separator();
                ui.horizontal(|ui| {
                    let start_all = unavailable_button(ui, "Start All", Vec2::new(84.0, 29.0));
                    self.automation.insert("dialog.streaming.startAll".into(), start_all.rect);
                    let start = unavailable_button(ui, &format!("Start {}", self.stream_channel + 1), Vec2::new(84.0, 29.0));
                    self.automation.insert("dialog.streaming.start".into(), start.rect);
                    let status = ui.add_sized([92.0, 29.0], egui::Button::new("View Status"));
                    self.automation.insert("dialog.streaming.status".into(), status.rect);
                    if status.clicked() { self.stream_page = StreamPage::Status; }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let close = ui.add_sized(footer_size(ui, FooterRole::Confirm), egui::Button::new(RichText::new("Close").color(Color32::WHITE)).fill(CHIP_BG));
                        self.automation.insert("dialog.streaming.close".into(), close.rect);
                        close_clicked = close.clicked();
                        let save = unavailable_button(ui, "Save and Close", Vec2::new(110.0, 29.0));
                        self.automation.insert("dialog.streaming.save".into(), save.rect);
                    });
                });
            });
        if !open || close_clicked {
            self.streaming_setup = false;
            self.stream_page = StreamPage::Main;
        }
    }

    pub(crate) fn multicorder_dialog(&mut self, ctx: &egui::Context) {
        if !self.multicorder_open {
            return;
        }
        let mut open = true;
        let mut close_clicked = false;
        let body_height = (ctx.viewport_rect().height() - 200.0).clamp(180.0, 490.0);
        operator_window(
            ctx,
            "MultiCorder",
            "dialog.multicorder",
            &mut open,
            Vec2::new(
                (ctx.viewport_rect().width() * 0.48).clamp(690.0, 920.0),
                body_height + 90.0,
            ),
        )
        .show(ctx, |ui| {
            ui.label(OUTPUT_UNAVAILABLE);
            let body = egui::ScrollArea::vertical()
                .id_salt("mc.body")
                .max_height(body_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (index, label) in ["AVI", "MKV", "MP4"].into_iter().enumerate() {
                            let tab = ui.add_sized(
                                [80.0, 28.0],
                                egui::Button::new(label).selected(self.mc_format == index),
                            );
                            self.automation
                                .insert(format!("dialog.multicorder.format.{index}"), tab.rect);
                            if tab.clicked() {
                                self.mc_format = index;
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let start = unavailable_button(ui, "Start", Vec2::new(110.0, 56.0));
                            self.automation
                                .insert("dialog.multicorder.toggle".into(), start.rect);
                        });
                    });
                    field_row(ui, "Video format", |ui| {
                        ui.label(format!(
                            "{} · encoder unavailable",
                            ["AVI", "MKV", "MP4"]
                                .get(self.mc_format)
                                .copied()
                                .unwrap_or("AVI")
                        ));
                    });
                    field_row(ui, "Audio", |ui| {
                        ui.label("Input · unavailable");
                    });
                    field_row(ui, "New file every", |ui| {
                        ui.label("Unavailable");
                    });
                    ui.add_enabled(
                        false,
                        egui::Checkbox::new(
                            &mut self.mc_ndi_mov,
                            "Record network inputs in original format",
                        ),
                    );
                    ui.add_space(6.0);
                    let table = light_form_frame(ui, 6, |ui| {
                        let content_width = (ui.available_width() - 48.0).max(280.0);
                        let widths =
                            [0.26, 0.34, 0.16, 0.11, 0.13].map(|share| content_width * share);
                        egui::ScrollArea::vertical()
                            .id_salt("mc.inputs.scroll")
                            .max_height((body_height - 220.0).clamp(90.0, 280.0))
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                egui::Grid::new("mc.inputs.table")
                                    .striped(true)
                                    .num_columns(5)
                                    .min_col_width(0.0)
                                    .spacing([12.0, 4.0])
                                    .show(ui, |ui| {
                                        for (label, width) in
                                            ["Input", "Folder", "Duration", "Frames", "Dropped"]
                                                .into_iter()
                                                .zip(widths)
                                        {
                                            ui.add_sized(
                                                [width, 20.0],
                                                egui::Label::new(RichText::new(label).strong())
                                                    .truncate(),
                                            );
                                        }
                                        ui.end_row();
                                        let outputs = (1..=4).map(|n| format!("Output {n}"));
                                        let inputs =
                                            self.engine.project().inputs.iter().map(|input| {
                                                format!(
                                                    "{}: {}",
                                                    self.engine.project().input_number(input.id),
                                                    input.name
                                                )
                                            });
                                        for (idx, label) in outputs.chain(inputs).enumerate() {
                                            let name = ui
                                                .allocate_ui_with_layout(
                                                    Vec2::new(widths[0], 22.0),
                                                    egui::Layout::left_to_right(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        let mut selected = self
                                                            .mc_checked
                                                            .get(idx)
                                                            .copied()
                                                            .unwrap_or(false);
                                                        ui.add_enabled(
                                                            false,
                                                            egui::Checkbox::new(&mut selected, ""),
                                                        );
                                                        ui.add_sized(
                                                            [ui.available_width(), 20.0],
                                                            egui::Label::new(&label).truncate(),
                                                        )
                                                        .on_hover_text(&label)
                                                    },
                                                )
                                                .inner;
                                            self.automation.insert(
                                                format!("dialog.multicorder.input.{idx}"),
                                                name.rect,
                                            );
                                            let folder = ui.add_sized(
                                                [widths[1], 22.0],
                                                egui::Label::new("Not configured").truncate(),
                                            );
                                            self.automation.insert(
                                                format!("dialog.multicorder.folder.{idx}"),
                                                folder.rect,
                                            );
                                            let duration = ui.add_sized(
                                                [widths[2], 22.0],
                                                egui::Label::new("—").truncate(),
                                            );
                                            self.automation.insert(
                                                format!("dialog.multicorder.duration.{idx}"),
                                                duration.rect,
                                            );
                                            ui.add_sized([widths[3], 22.0], egui::Label::new("—"));
                                            ui.add_sized([widths[4], 22.0], egui::Label::new("—"));
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
                    self.automation
                        .insert("dialog.multicorder.table".into(), table.response.rect);
                    ui.horizontal(|ui| {
                        ui.add_enabled(
                            false,
                            egui::Checkbox::new(
                                &mut self.mc_show_unsupported,
                                "Show unsupported inputs",
                            ),
                        );
                        let folder = unavailable_button(
                            ui,
                            "Change Recording Folder",
                            Vec2::new(180.0, 27.0),
                        );
                        self.automation
                            .insert("dialog.multicorder.changeFolder".into(), folder.rect);
                    });
                });
            self.automation
                .insert("dialog.multicorder.body".into(), body.inner_rect);
            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let close = ui.add_sized(
                    footer_size(ui, FooterRole::Confirm),
                    egui::Button::new(RichText::new("Close").color(Color32::WHITE)).fill(CHIP_BG),
                );
                self.automation
                    .insert("dialog.multicorder.close".into(), close.rect);
                close_clicked = close.clicked();
            });
        });
        if !open || close_clicked {
            self.multicorder_open = false;
        }
    }
}
