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
    Modes(Vec<rustmix_engine::video_format::CameraFormat>),
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
    modes: Vec<rustmix_engine::video_format::CameraFormat>,
    mode: Option<rustmix_engine::video_format::CameraFormat>,
    modes_device: String,
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
        self.modes.clear();
        self.mode = None;
        self.modes_device.clear();
        self.width = 1280;
        self.height = 720;
        self.interval_ms = 5000;
        // Keep a pending receiver: repeated page changes cannot create unlimited probes.
    }
    fn key(&self, tab: usize) -> String {
        match tab {
            9 => format!("{}|{}", self.folder, self.interval_ms),
            17 => format!("{}|{}|{}", self.url, self.width, self.height),
            22 => format!("{}|{:?}", self.device, self.mode),
            _ => String::new(),
        }
    }
}

fn photo_descriptor_path(path: &Path) -> std::result::Result<String, String> {
    let text = path.to_str().ok_or("Photo path must be Unicode")?;
    if let Some(local) = text.strip_prefix(r"\\?\") {
        if local.as_bytes().get(1) != Some(&b':')
            || !local
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphabetic)
        {
            return Err(
                "Choose a local disk photo folder; network/device paths are unsupported".into(),
            );
        }
        Ok(local.to_owned())
    } else {
        Ok(text.to_owned())
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
        paths.push(photo_descriptor_path(&path)?);
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
            && !self.add_name.chars().any(char::is_control)
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
            Ok(Probe::Modes(modes)) if result.key == self.source_ui.key(self.add_tab) => {
                self.source_ui.modes_device = self.source_ui.device.clone();
                self.source_ui.modes = modes;
                self.source_ui.error = if self.source_ui.modes.is_empty() {
                    Some("No supported camera format found".into())
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
                format: self.source_ui.mode.clone(),
            }),
            _ => None,
        };
        let (sender, receiver) = mpsc::sync_channel(1);
        let load_modes = tab == 22 && !devices && self.source_ui.mode.is_none();
        let device = self.source_ui.device.clone();
        let repaint = ctx.clone();
        let spawn = std::thread::Builder::new()
            .name("rustmix-source-check".into())
            .spawn(move || {
                let result = if devices {
                    crate::video_player::discover_cameras().map(Probe::Devices)
                } else if load_modes {
                    crate::video_player::discover_camera_modes(&device).map(Probe::Modes)
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
        let title = match self.add_tab {
            9 => "Select a folder of photos for a slideshow.",
            17 => "Web Browser",
            _ => "Camera",
        };
        super::add_input::page_header(ui, title);
        let old_key = self.source_ui.key(self.add_tab);
        super::dialogs::light_form_frame(ui, 12, |ui| {
            ui.add_enabled_ui(!busy,|ui| {
                match self.add_tab {
                    9=>{
                        ui.horizontal(|ui| {
                            let width=(ui.available_width()-108.0).max(80.0);
                            let path=ui.add_sized([width,26.0],egui::TextEdit::singleline(&mut self.source_ui.folder).char_limit(4096).hint_text("Photo folder"));
                            self.automation.insert("dialog.add.photos.folder".into(),path.rect);
                            let browse=ui.add_sized([100.0,26.0],egui::Button::new("Browse…"));
                            self.automation.insert("dialog.add.photos.browse".into(),browse.rect);
                            if browse.clicked(){self.start_extra_probe(ctx,true,false);}
                        });
                        ui.horizontal(|ui| {
                            ui.label("Slide interval (seconds)");
                            let mut seconds=self.source_ui.interval_ms as f32/1000.0;
                            let interval=ui.add(egui::DragValue::new(&mut seconds).range(0.5..=3600.0).speed(0.5));
                            self.automation.insert("dialog.add.photos.interval".into(),interval.rect);
                            if interval.changed(){self.source_ui.interval_ms=(seconds*1000.0).round() as u32;}
                        });
                        ui.label("PNG / JPEG / BMP · alphabetical order · up to 256 photos");
                        if let Some(VideoSource::Photos{paths,..})=&self.source_ui.validated {
                            egui::ScrollArea::vertical().id_salt("photos.files").max_height(160.0).show(ui,|ui| {
                                for path in paths {ui.label(Path::new(path).file_name().map_or_else(||"Photo".into(),|p|p.to_string_lossy()));}
                            });
                        }
                    },
                    17=>{
                        ui.horizontal(|ui| {
                            ui.label("URL");
                            let url=ui.add_sized([ui.available_width(),26.0],egui::TextEdit::singleline(&mut self.source_ui.url).char_limit(4096).hint_text("https://example.com"));
                            self.automation.insert("dialog.add.browser.url".into(),url.rect);
                        });
                        let preset=egui::ComboBox::from_id_salt("add.browser.size").selected_text(format!("{}×{}",self.source_ui.width,self.source_ui.height)).show_ui(ui,|ui| {
                            for (w,h) in [(1280,720),(1920,1080),(1080,1920),(720,1280),(640,360)] {
                                let item=ui.selectable_label((w,h)==(self.source_ui.width,self.source_ui.height),format!("{w}×{h}{}",if h>w {" · Portrait"}else{""}));
                                self.automation.insert(format!("dialog.add.browser.size.{w}x{h}"),item.rect);
                                if item.clicked(){self.source_ui.width=w;self.source_ui.height=h;ui.close();}
                            }
                        });
                        self.automation.insert("dialog.add.browser.size".into(),preset.response.rect);
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Width");let width=ui.add(egui::DragValue::new(&mut self.source_ui.width).range(64..=1920));
                            self.automation.insert("dialog.add.browser.width".into(),width.rect);
                            ui.label("Height");let height=ui.add(egui::DragValue::new(&mut self.source_ui.height).range(64..=1920));
                            self.automation.insert("dialog.add.browser.height".into(),height.rect);
                        });
                        ui.label("Capture: up to 5 fps · video only · no page audio or interaction");
                        ui.label("Uses an installed Edge/Chrome with a separate temporary profile. Check opens the page temporarily; OK starts the live input.");
                    },
                    _=>{
                        let refresh=ui.button("Refresh cameras");
                        self.automation.insert("dialog.add.camera.refresh".into(),refresh.rect);
                        if refresh.clicked(){self.source_ui.devices.clear();self.source_ui.modes.clear();self.source_ui.mode=None;self.source_ui.validated=None;self.start_extra_probe(ctx,false,true);}
                        let previous=self.source_ui.device.clone();
                        let camera=egui::ComboBox::from_id_salt("add.camera.device").width(360.0).selected_text(self.source_ui.devices.iter().find(|d|d.0==self.source_ui.device).map_or("Choose a camera",|d|d.1.as_str())).show_ui(ui,|ui| {
                            for (index,(id,label)) in self.source_ui.devices.iter().enumerate(){let option=ui.selectable_value(&mut self.source_ui.device,id.clone(),label);self.automation.insert(format!("dialog.add.camera.device.{index}"),option.rect);}
                        });
                        self.automation.insert("dialog.add.camera.device".into(),camera.response.rect);
                        if previous!=self.source_ui.device {self.source_ui.mode=None;self.source_ui.modes.clear();self.source_ui.modes_device.clear();}
                        let formats=ui.add_enabled(!self.source_ui.device.is_empty(),egui::Button::new("Load formats"));
                        self.automation.insert("dialog.add.camera.formats".into(),formats.rect);
                        if formats.clicked(){self.source_ui.mode=None;self.source_ui.validated=None;self.start_extra_probe(ctx,false,false);}
                        let mode=egui::ComboBox::from_id_salt("add.camera.mode").width(430.0).selected_text(self.source_ui.mode.as_ref().map_or_else(||"Choose resolution / frame rate / scan".into(),|m|m.label())).show_ui(ui,|ui| {
                            for (index,mode) in self.source_ui.modes.iter().enumerate(){
                                let label=format!("{} · {}",mode.label(),mode.subtype.get(..8).unwrap_or("video"));
                                let option=ui.selectable_value(&mut self.source_ui.mode,Some(mode.clone()),label);
                                self.automation.insert(format!("dialog.add.camera.mode.{index}"),option.rect);
                            }
                        });
                        self.automation.insert("dialog.add.camera.mode".into(),mode.response.rect);
                        ui.label("Formats come from the selected camera: resolution, actual frames/s, p or i, and field order. Interlaced camera pictures are converted to progressive for monitoring.");
                        ui.label("Load formats opens the selected camera temporarily. Check verifies its first picture. OK starts capture. Add camera audio separately through Audio Input.");
                    }
                }
                ui.horizontal(|ui| {
                    ui.label("Name");let name=ui.add_sized([ui.available_width(),26.0],egui::TextEdit::singleline(&mut self.add_name).char_limit(64));
                    self.automation.insert("dialog.add.extra.name".into(),name.rect);
                });
            });
            if self.source_ui.key(self.add_tab) != old_key {
                self.source_ui.validated = None;
                self.source_ui.error = None;
            }
            ui.horizontal(|ui| {
                let ready_to_check = self.add_tab != 22
                    || (self.source_ui.mode.is_some()
                        && self.source_ui.modes_device == self.source_ui.device);
                let check =
                    ui.add_enabled(!busy && ready_to_check, egui::Button::new("Check source"));
                self.automation
                    .insert("dialog.add.extra.check".into(), check.rect);
                if check.clicked() {
                    self.start_extra_probe(ctx, false, false);
                }
                let clear = ui.add_enabled(!busy, egui::Button::new("Clear selection"));
                self.automation
                    .insert("dialog.add.extra.clear".into(), clear.rect);
                if clear.clicked() {
                    self.source_ui.retire();
                }
            });
            let status = if busy {
                "Checking source…".into()
            } else if let Some(error) = &self.source_ui.error {
                error.clone()
            } else if let Some(source) = &self.source_ui.validated {
                match source {
                    VideoSource::Photos { paths, .. } => format!("Ready · {} photos", paths.len()),
                    VideoSource::Camera {
                        format: Some(mode), ..
                    } => format!("Ready · {}", mode.label()),
                    _ => "Ready · source checked".into(),
                }
            } else {
                "Choose a source and select Check source.".into()
            };
            let status = ui.label(status);
            self.automation
                .insert("dialog.add.extra.status".into(), status.rect);
            if matches!(self.add_tab, 17 | 22) {
                ui.label(
                    "After reopening a project, use Start source in Input Settings to reconnect.",
                );
            }
        });
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::panic)]
    use super::*;
    #[test]
    fn canonical_windows_photo_paths_use_local_drive_descriptors() {
        assert_eq!(
            photo_descriptor_path(Path::new(r"\\?\C:\Photos\one.png")).unwrap(),
            r"C:\Photos\one.png"
        );
        assert!(photo_descriptor_path(Path::new(r"\\?\UNC\server\one.png")).is_err());
        assert!(photo_descriptor_path(Path::new(r"\\?\Volume{bad}\one.png")).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn real_photo_folder_checks_png_jpeg_bmp_and_rejects_corrupt_media() {
        let directory = tempfile::tempdir().unwrap();
        for name in ["c.bmp", "a.png", "b.jpg"] {
            image::RgbImage::from_pixel(16, 9, image::Rgb([25_u8, 165, 70]))
                .save(directory.path().join(name))
                .unwrap();
        }
        let VideoSource::Photos { paths, interval_ms } =
            photo_folder(directory.path().to_str().unwrap(), 500).unwrap()
        else {
            panic!("Photos descriptor required");
        };
        assert_eq!(paths.len(), 3);
        assert_eq!(interval_ms, 500);
        assert!(
            paths[0].ends_with("a.png")
                && paths[1].ends_with("b.jpg")
                && paths[2].ends_with("c.bmp")
        );
        assert!(paths.iter().all(|p| !p.starts_with(r"\\?\")));
        std::fs::write(directory.path().join("bad.png"), b"not a PNG").unwrap();
        assert!(photo_folder(directory.path().to_str().unwrap(), 500).is_err());
    }
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
                    format: None,
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
