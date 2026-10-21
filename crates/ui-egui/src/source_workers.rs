//! Real slideshow and isolated Chromium page frames. No UI-thread I/O.
use super::*;
use base64::Engine as _;
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};
use tungstenite::{Message, WebSocket, protocol::WebSocketConfig};

type Raster = (u32, u32, Arc<Vec<[u8; 4]>>);

const MESSAGE_LIMIT: usize = 12 * 1024 * 1024;

fn raster(bytes: Vec<u8>, format: image::ImageFormat) -> Result<Raster, String> {
    if bytes.len() > 32 * 1024 * 1024 {
        return Err("Image exceeds 32 MiB".into());
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    let mut reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), format);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|_| "Image cannot be decoded within raster limits")?
        .resize(640, 360, image::imageops::FilterType::Triangle)
        .to_rgba8();
    Ok((
        image.width(),
        image.height(),
        Arc::new(image.pixels().map(|p| p.0).collect()),
    ))
}

pub(super) fn photo(path: &Path) -> Result<Raster, String> {
    let format = image::ImageFormat::from_path(path).map_err(|_| "Unsupported photo extension")?;
    if !matches!(
        format,
        image::ImageFormat::Png | image::ImageFormat::Jpeg | image::ImageFormat::Bmp
    ) {
        return Err("Use PNG, JPEG or BMP photos".into());
    }
    let file = std::fs::File::open(path).map_err(|_| "Photo cannot be opened")?;
    if !file
        .metadata()
        .map_err(|_| "Photo metadata unavailable")?
        .is_file()
    {
        return Err("Choose a regular photo file".into());
    }
    let mut bytes = Vec::new();
    file.take(32 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Photo read failed")?;
    raster(bytes, format)
}

struct ProcessOwner {
    child: Child,
    // Temp profile is removed only after this owner's process tree is retired.
    _profile: tempfile::TempDir,
}
impl Drop for ProcessOwner {
    fn drop(&mut self) {
        if let Ok(mut killer) = Command::new("taskkill.exe")
            .args(["/PID", &self.child.id().to_string(), "/T", "/F"])
            .creation_flags(0x08000000)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            let until = Instant::now();
            while until.elapsed() < Duration::from_secs(2) {
                if killer.try_wait().ok().flatten().is_some() {
                    break;
                }
                thread::sleep(Duration::from_millis(20));
            }
            let _ = killer.kill();
            let _ = killer.wait();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
struct Browser {
    socket: WebSocket<TcpStream>,
    _process: ProcessOwner,
    command_id: u64,
}

pub(super) fn executable() -> Result<PathBuf, String> {
    for root in ["PROGRAMFILES(X86)", "PROGRAMFILES", "LOCALAPPDATA"] {
        if let Some(root) = std::env::var_os(root) {
            for suffix in [
                "Microsoft/Edge/Application/msedge.exe",
                "Google/Chrome/Application/chrome.exe",
            ] {
                let path = PathBuf::from(&root).join(suffix);
                if path.is_file() {
                    return Ok(path);
                }
            }
        }
    }
    Err("Install Microsoft Edge or Google Chrome to use Browser input".into())
}

fn tcp(port: u16) -> Result<TcpStream, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let socket = TcpStream::connect_timeout(&address, Duration::from_secs(2))
        .map_err(|_| "Browser control connection unavailable")?;
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|_| "Browser read timeout unavailable")?;
    socket
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|_| "Browser write timeout unavailable")?;
    Ok(socket)
}
impl Browser {
    fn open(url: &str, width: u32, height: u32, stop: &AtomicBool) -> Result<Self, String> {
        let profile = tempfile::tempdir().map_err(|_| "Browser temporary profile unavailable")?;
        let child = Command::new(executable()?)
            .args([
                "--headless",
                "--disable-gpu",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-background-networking",
                "--disable-sync",
                "--mute-audio",
                "--remote-debugging-address=127.0.0.1",
                "--remote-debugging-port=0",
                "--hide-scrollbars",
            ])
            .arg(format!("--user-data-dir={}", profile.path().display()))
            .arg(format!("--window-size={width},{height}"))
            .arg("about:blank")
            .creation_flags(0x08000000)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "Browser process could not start")?;
        let mut process = ProcessOwner {
            child,
            _profile: profile,
        };
        // Failed initialization also retires this exact process tree.
        let connected = (|| {
            let start = Instant::now();
            let port = loop {
                if stop.load(Ordering::Acquire) {
                    return Err("Browser start cancelled".to_owned());
                }
                if process
                    .child
                    .try_wait()
                    .map_err(|_| "Browser status unavailable")?
                    .is_some()
                {
                    return Err("Browser exited during startup".into());
                }
                if let Ok(text) =
                    std::fs::read_to_string(process._profile.path().join("DevToolsActivePort"))
                    && let Some(port) = text
                        .lines()
                        .next()
                        .and_then(|s| s.parse::<u16>().ok())
                        .filter(|p| *p > 0)
                {
                    break port;
                }
                if start.elapsed() > Duration::from_secs(15) {
                    return Err("Browser startup timed out".into());
                }
                thread::sleep(Duration::from_millis(50));
            };
            let mut discovery = tcp(port)?;
            discovery
                .write_all(
                    b"GET /json/list HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                )
                .map_err(|_| "Browser page discovery failed")?;
            let mut response = String::new();
            discovery
                .take(128 * 1024)
                .read_to_string(&mut response)
                .map_err(|_| "Browser page discovery timed out")?;
            let (_, body) = response
                .split_once("\r\n\r\n")
                .ok_or("Invalid browser discovery response")?;
            let pages: Vec<Value> =
                serde_json::from_str(body).map_err(|_| "Invalid browser page list")?;
            let ws = pages
                .iter()
                .find(|p| p.get("type").and_then(Value::as_str) == Some("page"))
                .and_then(|p| p.get("webSocketDebuggerUrl"))
                .and_then(Value::as_str)
                .ok_or("Browser page unavailable")?;
            if !ws.starts_with(&format!("ws://127.0.0.1:{port}/"))
                && !ws.starts_with(&format!("ws://localhost:{port}/"))
            {
                return Err("Unexpected browser control endpoint".into());
            }
            let config = WebSocketConfig::default()
                .max_message_size(Some(MESSAGE_LIMIT))
                .max_frame_size(Some(MESSAGE_LIMIT));
            let (socket, _) = tungstenite::client::client_with_config(ws, tcp(port)?, Some(config))
                .map_err(|_| "Browser websocket handshake failed")?;
            Ok(socket)
        })();
        let socket = connected?;
        let mut browser = Self {
            socket,
            _process: process,
            command_id: 0,
        };
        browser.call(
            "Emulation.setDeviceMetricsOverride",
            json!({"width":width,"height":height,"deviceScaleFactor":1,"mobile":false}),
        )?;
        let navigation = browser.call("Page.navigate", json!({"url":url}))?;
        if navigation.get("errorText").is_some() {
            return Err("Browser navigation failed".into());
        }
        Ok(browser)
    }
    fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.command_id = self
            .command_id
            .checked_add(1)
            .ok_or("Browser command counter exhausted")?;
        let id = self.command_id;
        self.socket
            .send(Message::Text(
                json!({"id":id,"method":method,"params":params})
                    .to_string()
                    .into(),
            ))
            .map_err(|_| "Browser control write failed")?;
        let start = Instant::now();
        for _ in 0..256 {
            if start.elapsed() > Duration::from_secs(5) {
                return Err("Browser command timed out".into());
            }
            let message = self
                .socket
                .read()
                .map_err(|_| "Browser control read failed")?;
            if let Message::Text(text) = message {
                let response: Value = serde_json::from_str(text.as_str())
                    .map_err(|_| "Invalid browser command response")?;
                if response.get("id").and_then(Value::as_u64) == Some(id) {
                    if response.get("error").is_some() {
                        return Err("Browser rejected the capture command".into());
                    }
                    return response
                        .get("result")
                        .cloned()
                        .ok_or_else(|| "Browser result is missing".into());
                }
            }
        }
        Err("Browser event limit exceeded".into())
    }
    fn capture(&mut self) -> Result<Raster, String> {
        let result = self.call(
            "Page.captureScreenshot",
            json!({"format":"png","captureBeyondViewport":false}),
        )?;
        let data = result
            .get("data")
            .and_then(Value::as_str)
            .ok_or("Browser screenshot missing")?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|_| "Browser screenshot invalid")?;
        raster(bytes, image::ImageFormat::Png)
    }
}

pub(super) fn run(
    shared: &Shared,
    input_id: InputId,
    generation: u64,
    source: &VideoSource,
) -> Result<(), String> {
    let mut browser = match source {
        VideoSource::Browser { url, width, height } => {
            Some(Browser::open(url, *width, *height, &shared.stop)?)
        }
        _ => None,
    };
    let mut sequence = 0_u64;
    let mut index = 0_usize;
    let mut last_restart = None;
    let mut elapsed = Duration::ZERO;
    let origin = Instant::now();
    let mut previous = origin;
    let mut last_picture = None;
    let mut last_capture = origin.checked_sub(Duration::from_secs(1)).unwrap_or(origin);
    while !shared.stop.load(Ordering::Acquire) {
        let control = *shared
            .control
            .lock()
            .map_err(|_| "Source control handoff failed")?;
        let now = Instant::now();
        let delta = now.duration_since(previous);
        previous = now;
        if !control.paused {
            elapsed = elapsed.saturating_add(delta);
        }
        if last_restart.is_some_and(|r| r != control.restart_generation) {
            elapsed = Duration::ZERO;
            last_picture = None;
            if let Some(browser) = &mut browser {
                browser.call("Page.reload", json!({"ignoreCache":true}))?;
            }
        }
        last_restart = Some(control.restart_generation);
        let picture = match source {
            VideoSource::Photos { paths, interval_ms } => {
                let step = elapsed.as_millis() / u128::from(*interval_ms);
                let desired = if control.looping {
                    (step % paths.len() as u128) as usize
                } else {
                    usize::try_from(step)
                        .unwrap_or(usize::MAX)
                        .min(paths.len().saturating_sub(1))
                };
                if last_picture.is_some() && desired == index {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                index = desired;
                photo(Path::new(
                    paths.get(index).ok_or("Photo index unavailable")?,
                ))?
            }
            VideoSource::Browser { .. } => {
                if last_picture.is_some()
                    && (control.paused || last_capture.elapsed() < Duration::from_millis(200))
                {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                last_capture = now;
                browser.as_mut().ok_or("Browser owner missing")?.capture()?
            }
            _ => return Err("Unsupported source worker".into()),
        };
        sequence = sequence
            .checked_add(1)
            .ok_or("Source frame counter exhausted")?;
        let timestamp = i64::try_from(origin.elapsed().as_nanos() / 100)
            .map_err(|_| "Source timestamp overflow")?;
        let metadata = VideoFrameMetadata {
            input_id,
            generation,
            sequence,
            width: picture.0,
            height: picture.1,
            timestamp_100ns: timestamp,
            duration_100ns: None,
            discontinuity: sequence == 1,
        };
        let frame = VideoFrame::new(metadata, picture.2).map_err(|e| e.to_string())?;
        for attempt in 0..8 {
            match shared.feed.publish(frame.clone()) {
                Ok(()) => break,
                Err(VideoFeedError::Busy) if attempt < 7 => thread::sleep(Duration::from_millis(2)),
                Err(e) => return Err(e.to_string()),
            }
        }
        last_picture = Some(sequence);
        shared.repaint.request_repaint();
    }
    Ok(())
}

pub(super) fn probe_browser(url: &str, width: u32, height: u32) -> Result<(), String> {
    let mut browser = Browser::open(url, width, height, &AtomicBool::new(false))?;
    browser.capture()?;
    Ok(())
}
