//! Hosted-only real source flow: procedural photos and a loopback HTML page.
use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::atomic::{AtomicBool, Ordering},
};

pub(crate) struct FixtureServer {
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
    pub(crate) url: String,
}
impl FixtureServer {
    fn start() -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        listener.set_nonblocking(true)?;
        let stop = Arc::new(AtomicBool::new(false));
        let done = Arc::clone(&stop);
        let worker=std::thread::Builder::new().name("rustmix-fixture-http".into()).spawn(move||{
            let start=Instant::now();
            while !done.load(Ordering::Acquire)&&start.elapsed()<Duration::from_secs(300) {
                match listener.accept(){
                    Ok((mut stream,_))=>{
                        let _=stream.set_read_timeout(Some(Duration::from_secs(1)));let _=stream.set_write_timeout(Some(Duration::from_secs(1)));
                        let mut request=[0;2048];let _=stream.read(&mut request);
                        let html="<!doctype html><style>html,body{margin:0;width:100%;height:100%;background:rgb(25,165,70)}</style><script>let n=0;setInterval(()=>{document.body.style.background=n++%2?'rgb(25,165,70)':'rgb(175,30,60)'},700)</script><body></body>";
                        let response=format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",html.len(),html);let _=stream.write_all(response.as_bytes());
                    },Err(e) if e.kind()==std::io::ErrorKind::WouldBlock=>std::thread::sleep(Duration::from_millis(10)),Err(_)=>break,
                }
            }
        })?;
        Ok(Self {
            stop,
            worker: Some(worker),
            url: format!("http://127.0.0.1:{port}/fixture"),
        })
    }
}
impl Drop for FixtureServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl RustMixApp {
    pub(crate) fn smoke_sources(
        &mut self,
        ctx: &egui::Context,
        s: &mut smoke::Smoke,
    ) -> Result<()> {
        if self.start.elapsed() > Duration::from_secs(300) {
            return Err(Error::Invalid(format!(
                "Real source workflow timed out at {}",
                s.stage
            )));
        }
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let name = match s.stage {
                    4 => "source-photos-form",
                    11 => "source-browser-form",
                    15 => "source-browser-program",
                    20 => "source-camera-discovery",
                    _ => return Err(Error::Invalid("Unexpected real-source capture".into())),
                };
                let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                image::save_buffer(
                    format!("artifacts/{name}.png"),
                    &bytes,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                )
                .map_err(|e| Error::Invalid(e.to_string()))?;
                s.captured = true;
            }
        }
        if s.wait < 4 {
            return Ok(());
        }
        match s.stage {
            0 => {
                std::fs::create_dir_all("artifacts/source-photos")?;
                for (file, color) in [
                    ("a.png", [25, 165, 70]),
                    ("b.jpg", [175, 30, 60]),
                    ("c.bmp", [30, 70, 175]),
                ] {
                    let image = image::RgbImage::from_pixel(64, 36, image::Rgb(color));
                    image
                        .save(format!("artifacts/source-photos/{file}"))
                        .map_err(|e| Error::Invalid(e.to_string()))?;
                }
                s.source_fixture = Some(FixtureServer::start()?);
                self.dispatch(Action::Add, ctx);
                self.select_add_category(9);
                s.stage = 1;
            }
            1 => {
                self.source_ui.folder = "artifacts/source-photos".into();
                self.source_ui.interval_ms = 500;
                self.add_name = "Fixture photos".into();
                s.stage = 2;
            }
            2 => {
                if !s.visual_settled(ctx) {
                    return Ok(());
                }
                self.native_click(s, "dialog.add.extra.check")?;
                s.stage = 3;
            }
            3 => {
                if self.extra_source_pending() {
                    return Ok(());
                }
                if !self.extra_source_ready() {
                    return Err(Error::Invalid(format!(
                        "Photo check failed: {}",
                        self.extra_source_error()
                    )));
                }
                self.validate_dialog_geometry(ctx, "dialog.add")?;
                self.source_capture(ctx, s, "source-photos-form")?;
                s.stage = 4;
            }
            4 if s.captured => {
                self.native_click(s, "dialog.add.confirm")?;
                s.stage = 5;
            }
            5 => {
                if self.add_open || self.engine.project().inputs.len() != 8 {
                    return Err(Error::Invalid(
                        "Photos did not commit exactly one source".into(),
                    ));
                }
                if self.engine.project().program != InputId(1)
                    || self.engine.project().preview != InputId(2)
                {
                    return Err(Error::Invalid(
                        "Photos changed Program/Preview without Take".into(),
                    ));
                }
                if let Some((_, _, pixels)) = self
                    .input_fx
                    .get(&InputId(8))
                    .and_then(|fx| fx.custom_rgba.as_ref())
                {
                    let bytes: Vec<u8> = pixels.iter().flat_map(|p| *p).collect();
                    if let Some(old) = &s.scratch_pixels {
                        if old != &bytes {
                            s.stage = 6;
                        }
                    } else {
                        s.scratch_pixels = Some(bytes);
                    }
                }
            }
            6 => {
                self.dispatch(Action::Add, ctx);
                self.select_add_category(17);
                s.stage = 7;
            }
            7 => {
                self.source_ui.url = s
                    .source_fixture
                    .as_ref()
                    .ok_or_else(|| Error::Invalid("Missing fixture server".into()))?
                    .url
                    .clone();
                self.add_name = "Fixture browser".into();
                self.source_ui.width = 1080;
                self.source_ui.height = 1920;
                s.stage = 8;
            }
            8 => {
                if !s.visual_settled(ctx) {
                    return Ok(());
                }
                self.native_click(s, "dialog.add.extra.check")?;
                s.stage = 9;
            }
            9 => {
                if self.extra_source_pending() {
                    return Ok(());
                }
                if !self.extra_source_ready() {
                    return Err(Error::Invalid(format!(
                        "Browser check failed: {}",
                        self.extra_source_error()
                    )));
                }
                s.stage = 10;
            }
            10 => {
                self.validate_dialog_geometry(ctx, "dialog.add")?;
                self.source_capture(ctx, s, "source-browser-form")?;
                s.stage = 11;
            }
            11 if s.captured => {
                self.native_click(s, "dialog.add.confirm")?;
                s.stage = 12;
            }
            12 => {
                if self.add_open || self.engine.project().inputs.len() != 9 {
                    return Err(Error::Invalid("Browser did not commit one source".into()));
                }
                if self.engine.project().program != InputId(1)
                    || self.engine.project().preview != InputId(2)
                {
                    return Err(Error::Invalid(
                        "Browser changed Program/Preview before Take".into(),
                    ));
                }
                if self
                    .input_fx
                    .get(&InputId(9))
                    .is_none_or(|fx| fx.custom_rgba.is_none())
                {
                    return Ok(());
                }
                self.execute(Command::SelectPreview(InputId(9)));
                self.native_click(s, "switcher.cut")?;
                s.stage = 13;
            }
            13 => {
                if self.engine.project().program != InputId(9) {
                    return Err(Error::Invalid("Browser Take failed".into()));
                }
                if !fixture_pixels(&self.last_program_rgba) {
                    return Ok(());
                }
                s.scratch_pixels = Some(self.last_program_rgba.clone());
                s.stage = 14;
            }
            14 => {
                if !fixture_pixels(&self.last_program_rgba)
                    || s.scratch_pixels.as_ref() == Some(&self.last_program_rgba)
                {
                    return Ok(());
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                s.captured = false;
                s.stage = 15;
            }
            15 if s.captured => {
                self.dispatch(Action::Add, ctx);
                self.select_add_category(22);
                self.start_extra_probe(ctx, false, true);
                s.stage = 16;
            }
            16 => {
                if self.extra_source_pending() {
                    return Ok(());
                }
                if self.extra_source_ready() {
                    return Err(Error::Invalid("Unselected camera enabled OK".into()));
                }
                s.stage = 19;
            }
            19 => {
                self.validate_dialog_geometry(ctx, "dialog.add")?;
                self.source_capture(ctx, s, "source-camera-discovery")?;
                s.stage = 20;
            }
            20 if s.captured => {
                self.native_click(s, "dialog.add.cancel")?;
                s.stage = 21;
            }
            21 => {
                if self.add_open
                    || self.audio_ui.is_active()
                    || self.rec_active
                    || self.stream_active
                {
                    return Err(Error::Invalid(
                        "Source workflow activated unrelated devices/output".into(),
                    ));
                }
                let directory = Path::new("artifacts/sources.rustmix.json");
                persistence::save(directory, self.engine.project())?;
                if persistence::load(directory)? != *self.engine.project() {
                    return Err(Error::Invalid("New sources did not roundtrip".into()));
                }
                let inputs = self
                    .engine
                    .project()
                    .inputs
                    .iter()
                    .filter(|i| i.config.video_source.is_some())
                    .cloned()
                    .collect::<Vec<_>>();
                for mut input in inputs {
                    input.online = false;
                    self.execute(Command::ApplyInput { input, audio: None });
                }
                self.stop_video_players();
                s.stage = 22;
            }
            22 => {
                s.source_fixture = None;
                std::fs::write(
                    "artifacts/sources-ok.json",
                    serde_json::to_vec_pretty(
                        &serde_json::json!({"photos_formats":["png","jpeg","bmp"],"photos_pixels_changed":true,"browser_program_pixels_changed":true,"camera_discovery_only":true,"camera_capture_hardware_tested":false,"recording_started":false,"streaming_started":false,"source_schema":SCHEMA}),
                    )?,
                )?;
                self.saved_revision = self.engine.project().revision;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
        }
        Ok(())
    }
    fn source_capture(
        &mut self,
        ctx: &egui::Context,
        s: &mut smoke::Smoke,
        name: &str,
    ) -> Result<()> {
        std::fs::write(
            format!("artifacts/{name}-elements.json"),
            serde_json::to_vec_pretty(
                &serde_json::json!({"tab":self.add_tab,"ready":self.extra_source_ready(),"camera_count":self.extra_camera_count(),"elements":self.automation.iter().filter(|(id,_)|id.starts_with("dialog.add")).map(|(id,r)|(id.clone(),format!("{r:?}"))).collect::<HashMap<_,_>>()}),
            )?,
        )?;
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        s.captured = false;
        Ok(())
    }
}
fn fixture_pixels(bytes: &[u8]) -> bool {
    bytes
        .get(0..3)
        .is_some_and(|p| p == [25, 165, 70] || p == [175, 30, 60])
}
