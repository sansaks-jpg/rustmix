//! One bounded still-image worker. Results are tagged so obsolete dialogs/projects
//! cannot attach a late image to a different source with a reused ID.
use super::*;
use std::{
    fs::File,
    io::{Cursor, Read},
    sync::Arc,
};

pub(crate) const MAX_IMAGE_BYTES: u64 = 32 * 1024 * 1024;

pub(crate) struct DecodedImage {
    pub path: String,
    pub original_size: [u32; 2],
    pub width: u32,
    pub height: u32,
    pub pixels: Arc<Vec<[u8; 4]>>,
}

pub(crate) fn image_format(path: &Path) -> Option<image::ImageFormat> {
    let extension = path.extension()?.to_str()?;
    if extension.eq_ignore_ascii_case("png") {
        Some(image::ImageFormat::Png)
    } else if extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg") {
        Some(image::ImageFormat::Jpeg)
    } else if extension.eq_ignore_ascii_case("bmp") {
        Some(image::ImageFormat::Bmp)
    } else {
        None
    }
}

pub(crate) fn decode_image(path: &Path) -> std::result::Result<DecodedImage, String> {
    let format = image_format(path).ok_or("Choose a PNG, JPEG or BMP image")?;
    let path = path
        .canonicalize()
        .map_err(|e| format!("Image unavailable: {e}"))?;
    let name = path
        .to_str()
        .ok_or("Image path is not valid Unicode")?
        .to_owned();
    if name.len() > 4096 {
        return Err("Image path exceeds 4096 bytes".into());
    }
    let file = File::open(&path).map_err(|e| format!("Image unavailable: {e}"))?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_IMAGE_BYTES {
        return Err("Image must be a regular file of at most 32 MiB".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_IMAGE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_IMAGE_BYTES {
        return Err("Image exceeds 32 MiB".into());
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|e| format!("Image could not be decoded (4096px / 64 MiB limits): {e}"))?;
    let original_size = [image.width(), image.height()];
    // Cache the Full HD raster (1920x1080), preserving crisp clarity for overlays and stills.
    let raster = image
        .resize(1920, 1080, image::imageops::FilterType::Triangle)
        .to_rgba8();
    let pixels = raster.pixels().map(|p| p.0).collect();
    Ok(DecodedImage {
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
type DecodeResult = std::result::Result<DecodedImage, String>;
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
                .name("rustmix-image-decode".into())
                .spawn(move || {
                    let _ = tx.send(decode_image(&path));
                });
            if let Err(error) = spawn {
                return Some((
                    request.target,
                    Err(format!(
                        "Image worker could not start ({:?}); current source retained",
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
                Err("Image worker stopped; current source retained".into())
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
        let decoded = decode_image(&path).unwrap();
        assert_eq!(decoded.original_size, [1, 1]);
        assert_eq!(decoded.width, decoded.height);
        assert!(decoded.pixels.len() <= 1920 * 1080);
        assert!(decoded.pixels.iter().all(|p| *p == [255, 0, 0, 128]));
        let huge = dir.path().join("too-wide.png");
        image::save_buffer(&huge, &vec![0; 4097 * 4], 4097, 1, image::ColorType::Rgba8).unwrap();
        assert!(decode_image(&huge).is_err());
        std::fs::write(&path, b"invalid PNG").unwrap();
        assert!(decode_image(&path).is_err());
        let oversized = dir.path().join("oversized.png");
        File::create(&oversized)
            .unwrap()
            .set_len(MAX_IMAGE_BYTES + 1)
            .unwrap();
        assert!(decode_image(&oversized).is_err());
        assert!(decode_image(Path::new("absent.png")).is_err());
        assert!(decode_image(Path::new("movie.mp4")).is_err());
    }
    #[test]
    fn jpeg_and_bmp_decode_and_reject_mismatched_or_corrupt_files() {
        let dir = tempfile::tempdir().unwrap();
        for (name, format) in [
            ("still.JPG", image::ImageFormat::Jpeg),
            ("still.jpeg", image::ImageFormat::Jpeg),
            ("still.BMP", image::ImageFormat::Bmp),
        ] {
            let path = dir.path().join(name);
            image::save_buffer_with_format(
                &path,
                &[180, 30, 50].repeat(16 * 9),
                16,
                9,
                image::ColorType::Rgb8,
                format,
            )
            .unwrap();
            let decoded = decode_image(&path).unwrap();
            assert_eq!(decoded.original_size, [16, 9]);
            assert!(decoded.width <= 1920 && decoded.height <= 1080);
            assert!(
                decoded
                    .pixels
                    .iter()
                    .all(|pixel| pixel.last() == Some(&255))
            );
            std::fs::write(&path, b"invalid image").unwrap();
            assert!(decode_image(&path).is_err());
        }
        let wrong = dir.path().join("wrong.jpg");
        image::save_buffer_with_format(
            &wrong,
            &[255, 0, 0, 255],
            1,
            1,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .unwrap();
        assert!(decode_image(&wrong).is_err());
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
