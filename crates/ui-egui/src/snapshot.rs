//! One bounded file job; PNG encoding and disk writes never run on the UI thread.
use super::*;
use std::io::Write;

fn write_snapshot(path: &std::path::Path, pixels: &[u8], size: [u32; 2]) -> Result<()> {
    if !matches!(size, [640, 360] | [360, 640]) || pixels.len() != 640 * 360 * 4 {
        return Err(Error::Invalid(
            "No complete Program frame is available".into(),
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| Error::Invalid("Snapshot folder is missing".into()))?;
    std::fs::create_dir_all(parent)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let result = (|| {
        let [width, height] = size;
        image::write_buffer_with_format(
            &mut file,
            pixels,
            width,
            height,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .map_err(|error| Error::Invalid(format!("Snapshot encoding failed: {error}")))?;
        file.flush()?;
        file.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        drop(file);
        let _ = std::fs::remove_file(path);
    }
    result
}
impl RustMixApp {
    pub(crate) fn start_snapshot(&mut self, folder: PathBuf) {
        if self.job.is_some() {
            self.say("A file operation is already running".into(), true);
            return;
        }
        if self.last_program_rgba.len() != 640 * 360 * 4 {
            self.say(
                "Snapshot unavailable: no complete Program frame".into(),
                true,
            );
            return;
        }
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let path = folder.join(format!("program-{}-{nonce}.png", std::process::id()));
        let pixels = self.last_program_rgba.clone();
        let size = self.program_texture.as_ref().map_or([640, 360], |t| {
            let [width, height] = t.size();
            [width as u32, height as u32]
        });
        let (tx, rx) = mpsc::sync_channel(1);
        self.job = Some(rx);
        std::thread::spawn(move || {
            let done = match write_snapshot(&path, &pixels, size) {
                Ok(()) => JobDone::SnapshotSaved(path),
                Err(error) => JobDone::Failed(error.to_string()),
            };
            let _ = tx.send(done);
        });
        self.say("Saving Program snapshot…".into(), false);
    }
}
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]
    use super::*;
    #[test]
    fn no_frame_is_an_error_and_existing_snapshots_are_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = RustMixApp::headless();
        app.start_snapshot(dir.path().into());
        assert!(app.warning);
        assert!(app.job.is_none());
        let path = dir.path().join("existing.png");
        std::fs::write(&path, b"keep").unwrap();
        assert!(write_snapshot(&path, &vec![10; 640 * 360 * 4], [640, 360]).is_err());
        assert_eq!(std::fs::read(path).unwrap(), b"keep");
    }
    #[test]
    fn snapshot_completion_is_acknowledged_only_after_real_worker_output() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = RustMixApp::headless();
        let ctx = egui::Context::default();
        app.last_program_rgba = [10, 20, 30, 255].repeat(640 * 360);
        let before = app.engine.project().clone();
        app.start_snapshot(dir.path().into());
        assert!(app.job.is_some());
        let deadline = Instant::now() + Duration::from_secs(5);
        while app.job.is_some() {
            app.poll_job(&ctx);
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(!app.warning, "{}", app.notice);
        assert!(app.notice.starts_with("Program snapshot saved"));
        let entry = std::fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        let pixels = image::open(entry.path()).unwrap().into_rgba8();
        assert_eq!(pixels.dimensions(), (640, 360));
        assert_eq!(pixels.get_pixel(0, 0).0, [10, 20, 30, 255]);
        assert_eq!(*app.engine.project(), before);
    }
}
