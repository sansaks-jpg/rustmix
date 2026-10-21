//! Add Input for real photo folders, Chromium pages and selected Windows cameras.
use super::*;
use std::sync::mpsc::TryRecvError;

struct Checked {
    tab: usize,
    token: u64,
    key: String,
    result: std::result::Result<Probe, String>,
}
enum Probe {
    Devices(Vec<(String, String)>),
    Source(VideoSource),
}
#[derive(Default)]
pub(crate) struct SourceUi {
    pub(crate) folder: String,
    pub(crate) url: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) interval_ms: u32,
    device: String,
    devices: Vec<(String, String)>,
    validated: Option<VideoSource>,
    checked_key: String,
    job: Option<Receiver<Checked>>,
    job_token: u64,
    error: Option<String>,
}
impl SourceUi {
    pub(crate) fn retire(&mut self) {
        self.validated = None;
        self.checked_key.clear();
        self.error = None;
        self.folder.clear();
        self.url.clear();
        self.device.clear();
        self.width = 1280;
        self.height = 720;
        self.interval_ms = 5000;
        // Keep a pending receiver: repeated page changes cannot create unlimited probes.
    }
    fn key(&self, tab: usize) -> String {
        match tab {
            9 => format!("{}|{}", self.folder, self.interval_ms),
            17 => format!("{}|{}|{}", self.url, self.width, self.height),
            22 => self.device.clone(),
            _ => String::new(),
        }
    }
}

fn photo_folder(folder: &str, interval_ms: u32) -> std::result::Result<VideoSource, String> {
    let directory = Path::new(folder)
        .canonicalize()
        .map_err(|_| "Photo folder cannot be opened")?;
    if !directory.is_dir() {
        return Err("Choose a photo folder".into());
    }
    let mut paths = Vec::new();
    let entries = std::fs::read_dir(directory).map_err(|_| "Photo folder cannot be listed")?;
    for (index, entry) in entries.enumerate() {
        if index >= 1024 {
            return Err(
                "Photo folder contains more than 1024 entries; use a smaller folder".into(),
            );
        }
        let path = entry.map_err(|_| "Photo folder entry unavailable")?.path();
        if !path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
            ["png", "jpg", "jpeg", "bmp"]
                .iter()
                .any(|x| e.eq_ignore_ascii_case(x))
        }) {
            continue;
        }
        if !path.is_file() {
            continue;
        }
        if paths.len() >= 256 {
            return Err("Use at most 256 photos per input".into());
        }
        paths.push(
            path.to_str()
                .ok_or("Photo path must be Unicode")?
                .to_owned(),
        );
    }
    paths.sort();
    let source = VideoSource::Photos { paths, interval_ms };
    source.validate().map_err(|e| e.to_string())?;
    crate::video_player::probe_source(&source)?;
    Ok(source)
}

impl RustMixApp {
    pub(crate) fn extra_source_pending(&self) -> bool {
        self.source_ui.job.is_some()
    }
    pub(crate) fn extra_source_error(&self) -> String {
        self.source_ui.error.clone().unwrap_or_default()
    }
    pub(crate) fn extra_camera_count(&self) -> usize {
        self.source_ui.devices.len()
    }
    pub(crate) fn extra_source_ready(&self) -> bool {
        self.source_ui.job.is_none()
            && self.source_ui.validated.is_some()
            && self.source_ui.checked_key == self.source_ui.key(self.add_tab)
            && !self.add_name.trim().is_empty()
            && self.add_name.trim().len() <= 64
    }
    pub(crate) fn poll_extra_source(&mut self) {
        let result = self
            .source_ui
            .job
            .as_ref()
            .and_then(|job| match job.try_recv() {
                Ok(value) => Some(value),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => Some(Checked {
                    tab: self.add_tab,
                    token: self.source_ui.job_token,
                    key: String::new(),
                    result: Err("Source check worker stopped".into()),
                }),
            });
        let Some(result) = result else {
            return;
        };
        self.source_ui.job = None;
        if !self.add_open || result.tab != self.add_tab || result.token != self.add_token {
            return;
        }
        match result.result {
            Ok(Probe::Devices(devices)) => {
                self.source_ui.devices = devices;
                self.source_ui.error = if self.source_ui.devices.is_empty() {
                    Some("No camera found. Connect a camera, enable Windows camera permission, then refresh.".into())
                } else {
                    None
                };
            }
            Ok(Probe::Source(source)) if result.key == self.source_ui.key(self.add_tab) => {
                if let VideoSource::Photos { paths, .. } = &source
                    && let Some(folder) = paths
                        .first()
                        .and_then(|p| Path::new(p).parent())
                        .and_then(|p| p.to_str())
                {
                    self.source_ui.folder = folder.to_owned();
                }
                self.source_ui.checked_key = self.source_ui.key(self.add_tab);
                self.source_ui.validated = Some(source);

                self.source_ui.error = None;
            }
            Ok(_) => {}
            Err(error) => {
                self.source_ui.validated = None;
                self.source_ui.error = Some(error);
            }
        }
    }
    pub(crate) fn start_extra_probe(&mut self, ctx: &egui::Context, picker: bool, devices: bool) {
        if self.source_ui.job.is_some() {
            return;
        }
        let tab = self.add_tab;
        let token = self.add_token;
        let key = self.source_ui.key(tab);
        let folder = self.source_ui.folder.clone();
        let interval = self.source_ui.interval_ms;
        let source = match tab {
            17 => Some(VideoSource::Browser {
                url: self.source_ui.url.clone(),
                width: self.source_ui.width,
                height: self.source_ui.height,
            }),
            22 => Some(VideoSource::Camera {
                device_id: self.source_ui.device.clone(),
            }),
            _ => None,
        };
        let (sender, receiver) = mpsc::sync_channel(1);
        let repaint = ctx.clone();
        let spawn = std::thread::Builder::new()
            .name("rustmix-source-check".into())
            .spawn(move || {
                let result = if devices {
                    crate::video_player::discover_cameras().map(Probe::Devices)
                } else if tab == 9 {
                    let chosen = if picker {
                        rfd::FileDialog::new()
                            .pick_folder()
                            .and_then(|p| p.to_str().map(str::to_owned))
                    } else {
                        Some(folder)
                    };
                    match chosen {
                        Some(folder) => photo_folder(&folder, interval).map(Probe::Source),
                        None => Err("Folder selection cancelled".into()),
                    }
                } else {
                    source
                        .ok_or_else(|| "Unknown source page".to_owned())
                        .and_then(|source| {
                            crate::video_player::probe_source(&source)?;
                            Ok(Probe::Source(source))
                        })
                };
                let _ = sender.try_send(Checked {
                    tab,
                    token,
                    key,
                    result,
                });
                repaint.request_repaint();
            });
        if spawn.is_ok() {
            self.source_ui.job = Some(receiver);
            self.source_ui.job_token = token;
            self.source_ui.validated = None;
            self.source_ui.error = None;
        } else {
            self.source_ui.error = Some("Cannot start source check worker".into());
        }
    }
    pub(crate) fn confirm_extra_source(&mut self, ctx: &egui::Context) -> bool {
        if self.add_disabled_reason().is_some() || !self.extra_source_ready() {
            return false;
        }
        let Some(source) = self.source_ui.validated.clone() else {
            return false;
        };
        let config = InputConfig {
            video_source: Some(source.clone()),
            ..Default::default()
        };
        if self
            .try_execute(Command::AddInputAt {
                name: self.add_name.trim().to_owned(),
                kind: InputKind::Color,
                config,
                index: self.add_number.saturating_sub(1),
            })
            .is_none()
        {
            return false;
        }
        let index = self.add_number.saturating_sub(1);
        if let Some(input) = self.engine.project().inputs.get(index) {
            self.armed_sources.insert(input.id);
        }
        self.sync_input_cache();
        self.poll_video(ctx);
        self.submit_program();
        self.source_ui.retire();
        true
    }
    pub(crate) fn extra_source_page(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let busy = self.source_ui.job.is_some();
        let header = ui.label(
            egui::RichText::new(match self.add_tab {
                9 => "Photos — folder slideshow",
                17 => "Web Browser",
                _ => "Camera",
            })
            .strong()
            .color(FORM_TEXT),
        );
        self.automation
            .insert("dialog.add.extra.header".into(), header.rect);
        ui.separator();
        let old_key = self.source_ui.key(self.add_tab);
        ui.add_enabled_ui(!busy,|ui|{
            if self.add_tab==9 {
                ui.label("PNG / JPEG / BMP · alphabetical order · maximum 256 photos");
                let path=ui.add(egui::TextEdit::singleline(&mut self.source_ui.folder).char_limit(4096).hint_text("Photo folder"));
                self.automation.insert("dialog.add.photos.folder".into(),path.rect);
                if ui.button("Browse folder…").clicked(){self.start_extra_probe(ctx,true,false);}
                ui.horizontal(|ui|{ui.label("Interval (seconds)");let mut seconds=self.source_ui.interval_ms as f32/1000.0; if ui.add(egui::DragValue::new(&mut seconds).range(0.5..=3600.0)).changed(){self.source_ui.interval_ms=(seconds*1000.0).round() as u32;}});
            }else if self.add_tab==17 {
                let url=ui.add(egui::TextEdit::singleline(&mut self.source_ui.url).char_limit(4096).hint_text("https://example.com"));
                self.automation.insert("dialog.add.browser.url".into(),url.rect);
                ui.horizontal(|ui|{ui.label("Viewport");ui.add(egui::DragValue::new(&mut self.source_ui.width).range(64..=1920));ui.label("×");ui.add(egui::DragValue::new(&mut self.source_ui.height).range(64..=1080));});
                ui.label("Uses an installed Edge/Chrome with a separate disposable profile. Video only; no page audio or operator interaction. Capture is capped at 5 fps.");
                ui.label("Check opens the page temporarily. OK opens the live page input; no signed-in browser profile is used.");
            }else {
                if ui.button("Refresh cameras").clicked(){self.start_extra_probe(ctx,false,true);}
                egui::ComboBox::from_id_salt("add.camera.device").selected_text(self.source_ui.devices.iter().find(|d|d.0==self.source_ui.device).map_or("Choose a camera",|d|d.1.as_str())).show_ui(ui,|ui|{
                    for (id,label) in &self.source_ui.devices{ui.selectable_value(&mut self.source_ui.device,id.clone(),label);}
                });
                ui.label("Check opens the selected camera temporarily. OK starts video capture. Camera audio is added separately through Audio Input.");
            }
            ui.horizontal(|ui|{ui.label("Name");ui.add(egui::TextEdit::singleline(&mut self.add_name).char_limit(64));});
        });
        if self.source_ui.key(self.add_tab) != old_key {
            self.source_ui.validated = None;
            self.source_ui.error = None;
        }
        let check = ui.add_enabled(!busy, egui::Button::new("Check source"));
        self.automation
            .insert("dialog.add.extra.check".into(), check.rect);
        if check.clicked() {
            self.start_extra_probe(ctx, false, false);
        }
        let text = if busy {
            "Checking source…".into()
        } else if let Some(error) = &self.source_ui.error {
            error.clone()
        } else if let Some(source) = &self.source_ui.validated {
            match source {
                VideoSource::Photos { paths, .. } => format!("Ready · {} photos", paths.len()),
                _ => "Ready · source checked".into(),
            }
        } else {
            "Choose a source and select Check source.".into()
        };
        let status = ui.label(text);
        self.automation
            .insert("dialog.add.extra.status".into(), status.rect);
        if matches!(self.add_tab, 17 | 22) {
            ui.label("After reopening a project, use Start source in Input Settings to reconnect.");
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]
    use super::*;
    #[test]
    fn source_draft_is_checked_against_current_fields_and_retires_pending_results() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.dispatch(Action::Add, &ctx);
        app.select_add_category(17);
        app.source_ui.url = "https://example.com".into();
        app.source_ui.validated = Some(VideoSource::Browser {
            url: app.source_ui.url.clone(),
            width: 1280,
            height: 720,
        });
        app.source_ui.checked_key = app.source_ui.key(17);
        assert!(app.extra_source_ready());
        app.source_ui.url = "https://example.org".into();
        assert!(!app.extra_source_ready());
        let token = app.add_token;
        let key = app.source_ui.key(17);
        let (tx, rx) = mpsc::sync_channel(1);
        app.source_ui.job = Some(rx);
        app.select_add_category(9);
        assert!(app.source_ui.job.is_some());
        tx.send(Checked {
            tab: 17,
            token,
            key,
            result: Ok(Probe::Source(VideoSource::Browser {
                url: "https://example.org".into(),
                width: 1280,
                height: 720,
            })),
        })
        .unwrap();
        app.poll_extra_source();
        assert!(app.source_ui.validated.is_none());
        assert!(app.source_ui.job.is_none());
    }
    #[test]
    fn live_camera_and_browser_do_not_reconnect_after_project_reset() {
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.execute(Command::AddInputAt {
            name: "Selected camera".into(),
            kind: InputKind::Color,
            config: InputConfig {
                video_source: Some(VideoSource::Camera {
                    device_id: "test-not-opened".into(),
                }),
                ..Default::default()
            },
            index: 0,
        });
        app.sync_input_cache();
        let id = app.engine.project().inputs[0].id;
        app.input_fx.entry(id).or_default().custom_rgba =
            Some((1, 1, Arc::new(vec![[25, 165, 70, 255]])));
        app.poll_video(&ctx);
        assert!(
            app.input_fx[&id].custom_rgba.is_none(),
            "Stopped live sources must not retain an apparently live picture"
        );
        assert!(app.input_fx[&id].image_generation > 0);
        assert!(app.video_players.is_empty());
        let id = app.engine.project().inputs[0].id;
        assert!(
            app.input_fx[&id]
                .image_error
                .as_ref()
                .unwrap()
                .contains("Stopped")
        );
        app.armed_sources.insert(id);
        app.stop_video_players();
        assert!(app.armed_sources.is_empty());
    }
}
