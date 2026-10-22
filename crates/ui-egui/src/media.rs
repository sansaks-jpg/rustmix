//! One bounded PNG worker. Results are tagged so obsolete dialogs/projects
//! cannot attach a late image to a different source with a reused ID.
use super::*;
use std::{
    fs::File,
    io::{Cursor, Read},
    sync::Arc,
};

pub(crate) const MAX_PNG_BYTES: u64 = 32 * 1024 * 1024;

pub(crate) struct DecodedPng {
    pub path: String,
    pub original_size: [u32; 2],
    pub width: u32,
    pub height: u32,
    pub pixels: Arc<Vec<[u8; 4]>>,
}

pub(crate) fn decode_png(path: &Path) -> std::result::Result<DecodedPng, String> {
    if !path
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("png"))
    {
        return Err("Only PNG images are supported; video decoding is unavailable".into());
    }
    let path = path
        .canonicalize()
        .map_err(|e| format!("PNG unavailable: {e}"))?;
    let name = path
        .to_str()
        .ok_or("PNG path is not valid Unicode")?
        .to_owned();
    if name.len() > 4096 {
        return Err("PNG path exceeds 4096 bytes".into());
    }
    let file = File::open(&path).map_err(|e| format!("PNG unavailable: {e}"))?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_PNG_BYTES {
        return Err("PNG must be a regular file of at most 32 MiB".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_PNG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_PNG_BYTES {
        return Err("PNG exceeds 32 MiB".into());
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|e| format!("PNG could not be decoded (4096px / 64 MiB limits): {e}"))?;
    let original_size = [image.width(), image.height()];
    // Cache the Full HD raster (1920x1080), preserving crisp clarity for overlays and stills.
    let raster = image
        .resize(1920, 1080, image::imageops::FilterType::Triangle)
        .to_rgba8();
    let pixels = raster.pixels().map(|p| p.0).collect();
    Ok(DecodedPng {
        path: name,
        original_size,
        width: raster.width(),
        height: raster.height(),
        pixels: Arc::new(pixels),
    })
}

#[derive(Clone)]
pub(crate) enum MediaTarget {
    AddPreview { token: u64 },
    Properties { id: InputId, token: u64 },
    Restore { id: InputId, path: String },
}
pub(crate) struct PickerDone {
    pub target: MediaTarget,
    pub path: Option<PathBuf>,
}
struct Request {
    path: PathBuf,
    target: MediaTarget,
    generation: u64,
}
type DecodeResult = std::result::Result<DecodedPng, String>;
#[derive(Default)]
pub(crate) struct MediaLoader {
    generation: u64,
    pending: VecDeque<Request>,
    active: Option<(Request, Receiver<DecodeResult>)>,
}
impl MediaLoader {
    pub fn reset(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.pending.clear();
        // An in-flight decode finishes on its one worker; its epoch is discarded.
    }
    pub fn busy(&self) -> bool {
        self.active.is_some() || !self.pending.is_empty()
    }
    pub fn enqueue(&mut self, path: PathBuf, target: MediaTarget) -> bool {
        if self.pending.len() >= 32 {
            return false;
        }
        self.pending.push_back(Request {
            path,
            target,
            generation: self.generation,
        });
        true
    }
    pub fn poll(&mut self) -> Option<(MediaTarget, DecodeResult)> {
        if self.active.is_none()
            && let Some(request) = self.pending.pop_front()
        {
            let path = request.path.clone();
            let (tx, rx) = mpsc::sync_channel(1);
            let spawn = std::thread::Builder::new()
                .name("rustmix-png-decode".into())
                .spawn(move || {
                    let _ = tx.send(decode_png(&path));
                });
            if let Err(error) = spawn {
                return Some((
                    request.target,
                    Err(format!(
                        "PNG worker could not start ({:?}); current source retained",
                        error.kind()
                    )),
                ));
            }
            self.active = Some((request, rx));
        }
        let result = match self.active.as_ref()?.1.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err("PNG worker stopped; current source retained".into())
            }
        };
        let (request, _) = self.active.take()?;
        (request.generation == self.generation).then_some((request.target, result))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    #[test]
    fn png_decode_is_bounded_and_keeps_alpha_and_aspect() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("small.PNG");
        image::save_buffer_with_format(
            &path,
            &[255, 0, 0, 128],
            1,
            1,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .unwrap();
        let decoded = decode_png(&path).unwrap();
        assert_eq!(decoded.original_size, [1, 1]);
        assert_eq!(decoded.width, decoded.height);
        assert!(decoded.pixels.len() <= 1920 * 1080);
        assert!(decoded.pixels.iter().all(|p| *p == [255, 0, 0, 128]));
        let huge = dir.path().join("too-wide.png");
        image::save_buffer(&huge, &vec![0; 4097 * 4], 4097, 1, image::ColorType::Rgba8).unwrap();
        assert!(decode_png(&huge).is_err());
        std::fs::write(&path, b"invalid PNG").unwrap();
        assert!(decode_png(&path).is_err());
        let oversized = dir.path().join("oversized.png");
        File::create(&oversized)
            .unwrap()
            .set_len(MAX_PNG_BYTES + 1)
            .unwrap();
        assert!(decode_png(&oversized).is_err());
        assert!(decode_png(Path::new("absent.png")).is_err());
        assert!(decode_png(Path::new("movie.mp4")).is_err());
    }
    #[test]
    fn obsolete_worker_results_and_overflow_are_rejected() {
        let mut loader = MediaLoader::default();
        let target = MediaTarget::Restore {
            id: InputId(1),
            path: "missing.png".into(),
        };
        for _ in 0..32 {
            assert!(loader.enqueue("missing.png".into(), target.clone()));
        }
        assert!(!loader.enqueue("overflow.png".into(), target.clone()));
        let (tx, rx) = mpsc::sync_channel(1);
        loader.active = Some((
            Request {
                path: "old.png".into(),
                target,
                generation: loader.generation,
            },
            rx,
        ));
        loader.reset();
        tx.send(Err("obsolete error".into())).unwrap();
        assert!(loader.poll().is_none());
        assert!(!loader.busy());
    }
}
