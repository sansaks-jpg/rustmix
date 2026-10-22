//! Experimental, synchronous Windows Media Foundation decoding of local MP4
//! files. Create, read, seek and drop a `VideoFile` on one dedicated worker.
//! `ReadSample`, opening and seeking can block that worker; this is not a
//! bounded-cancellation camera/network implementation. No audio is decoded.
//!
//! RGB32 processing/copying is deliberately CPU based and bounded. It is not a
//! zero-copy path or a realtime performance qualification. This module belongs
//! inside the platform crate's explicitly reviewed unsafe-code exception.

use std::{fmt, marker::PhantomData, os::windows::ffi::OsStrExt, path::Path, rc::Rc};
use windows::{
    Win32::{
        Media::MediaFoundation::*,
        Storage::FileSystem::GetDriveTypeW,
        System::{
            Com::{
                COINIT_MULTITHREADED, CoInitializeEx, CoTaskMemFree, CoUninitialize,
                StructuredStorage::PROPVARIANT,
            },
            WindowsProgramming::{DRIVE_CDROM, DRIVE_FIXED, DRIVE_RAMDISK, DRIVE_REMOVABLE},
        },
    },
    core::{GUID, Interface, PCWSTR},
};

pub const MAX_VIDEO_WIDTH: u32 = 4096;
pub const MAX_VIDEO_HEIGHT: u32 = 2160;
pub const MAX_VIDEO_BUFFER_BYTES: usize = 64 * 1024 * 1024;
const REFERENCE_WIDTH: u32 = 1920;
const REFERENCE_HEIGHT: u32 = 1080;
const FIRST_VIDEO: u32 = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
const ALL_STREAMS: u32 = MF_SOURCE_READER_ALL_STREAMS.0 as u32;

#[derive(Debug)]
pub enum VideoError {
    Invalid(&'static str),
    File(std::io::Error),
    UnsupportedCodec { operation: &'static str, code: i32 },
    Windows { operation: &'static str, code: i32 },
}

impl fmt::Display for VideoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => formatter.write_str(message),
            Self::File(error) => write!(formatter, "Local video file could not be opened: {error}"),
            Self::UnsupportedCodec { operation, code } => write!(
                formatter,
                "Windows cannot decode this container/codec as RGB32 ({operation}, HRESULT 0x{:08X})",
                *code as u32
            ),
            Self::Windows { operation, code } => write!(
                formatter,
                "Windows video operation failed ({operation}, HRESULT 0x{:08X})",
                *code as u32
            ),
        }
    }
}

impl std::error::Error for VideoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::File(error) => Some(error),
            _ => None,
        }
    }
}

type Result<T> = std::result::Result<T, VideoError>;

fn mf<T>(operation: &'static str, result: windows::core::Result<T>) -> Result<T> {
    result.map_err(|error| {
        let code = error.code();
        if matches!(
            code,
            MF_E_TOPO_CODEC_NOT_FOUND
                | MF_E_INVALIDMEDIATYPE
                | MF_E_INVALID_FILE_FORMAT
                | MF_E_UNSUPPORTED_BYTESTREAM_TYPE
        ) {
            VideoError::UnsupportedCodec {
                operation,
                code: code.0,
            }
        } else {
            VideoError::Windows {
                operation,
                code: code.0,
            }
        }
    })
}

/// Actual negotiated RGB32 output metadata, not an assumed project format.
/// Colour values are the numeric MF attributes, preserved when supplied by the
/// decoder. No HDR conversion/compliance claim is made by this CPU path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoFormat {
    pub width: u32,
    pub height: u32,
    pub stride: i32,
    pub frame_rate_numerator: u32,
    pub frame_rate_denominator: u32,
    pub pixel_aspect_numerator: u32,
    pub pixel_aspect_denominator: u32,
    pub video_primaries: Option<u32>,
    pub transfer_function: Option<u32>,
    pub nominal_range: Option<u32>,
    pub yuv_matrix: Option<u32>,
}

#[derive(Debug)]
pub struct DecodedVideoFrame {
    pub width: u32,
    pub height: u32,
    pub format: VideoFormat,
    /// Top-down RGBA; RGB32's unused alpha byte is replaced with opaque 255.
    pub pixels: Vec<[u8; 4]>,
    pub timestamp_100ns: i64,
    pub duration_100ns: Option<i64>,
    /// Monotonic within this reader, including across seeks.
    pub sequence: u64,
    pub discontinuity: bool,
    pub media_type_changed: bool,
}

#[derive(Debug)]
pub enum VideoRead {
    Frame(DecodedVideoFrame),
    StreamTick {
        timestamp_100ns: i64,
    },
    /// A status-only read is not EOS. Callers must avoid a busy retry loop.
    NoSample {
        timestamp_100ns: i64,
    },
    EndOfStream,
}

// Field order in `VideoFile` releases its reader before this runtime guard.
// Rc's phantom marker independently prevents moving COM initialization across
// threads, even if the generated interface's auto traits change in the future.
struct Runtime {
    mf_started: bool,
    _owner_thread: PhantomData<Rc<()>>,
}

impl Runtime {
    fn start() -> Result<Self> {
        // SAFETY: No reserved pointer is passed. The caller is the dedicated
        // owner worker; each successful COM initialization is paired with an
        // uninitialization on that same thread by this non-Send guard.
        mf("initialize COM MTA", unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()
        })?;
        let mut runtime = Self {
            mf_started: false,
            _owner_thread: PhantomData,
        };
        // SAFETY: This is the owner worker, not an MF callback/work-queue thread.
        // Version/flags are SDK constants; the guard pairs successful startup
        // with shutdown after all this reader's COM resources have been dropped.
        mf("start Media Foundation", unsafe {
            MFStartup(MF_VERSION, MFSTARTUP_FULL)
        })?;
        runtime.mf_started = true;
        Ok(runtime)
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if self.mf_started {
            // SAFETY: This non-Send guard remains on its initializing thread;
            // reader/sample interfaces are gone before this field is dropped.
            // Drop cannot report shutdown errors and never panics.
            let _ = unsafe { MFShutdown() };
        }
        // SAFETY: start() obtained a successful CoInitializeEx on this exact
        // thread, including S_FALSE; this is its single matching uninitialize.
        unsafe { CoUninitialize() };
    }
}

struct CaptureOwner(IMFMediaSource);
impl Drop for CaptureOwner {
    fn drop(&mut self) {
        // SAFETY: Owned media source remains on its COM owner thread, reader is
        // released first by field order; Shutdown releases driver capture resources.
        let _ = unsafe { self.0.Shutdown() };
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraDevice {
    pub id: String,
    pub name: String,
}

/// Device-reported native mode; plain data can cross to the UI control thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraMode {
    pub width: u32,
    pub height: u32,
    pub numerator: u32,
    pub denominator: u32,
    pub interlace: u32,
    pub subtype: String,
}

fn camera_mode(media_type: &IMFMediaType) -> Result<CameraMode> {
    let [width, height] = media_size(media_type)?;
    // SAFETY: The owned media type stays on this initialized COM worker. All
    // getters return scalar values, no application memory or interfaces escape.
    let (rate, interlace, subtype) = unsafe {
        (
            mf("camera frame rate", media_type.GetUINT64(&MF_MT_FRAME_RATE))?,
            mf(
                "camera scan mode",
                media_type.GetUINT32(&MF_MT_INTERLACE_MODE),
            )?,
            mf("camera pixel format", media_type.GetGUID(&MF_MT_SUBTYPE))?,
        )
    };
    let (numerator, denominator) = ((rate >> 32) as u32, rate as u32);
    if numerator == 0
        || denominator == 0
        || u64::from(numerator) > 120 * u64::from(denominator)
        || ![2, 3, 4].contains(&interlace)
    {
        return Err(VideoError::Invalid(
            "Camera mode must have a valid frame rate and progressive or interleaved scan",
        ));
    }
    let (mut gcd, mut remainder) = (numerator, denominator);
    while remainder != 0 {
        let next = gcd % remainder;
        gcd = remainder;
        remainder = next;
    }
    Ok(CameraMode {
        width,
        height,
        numerator: numerator / gcd,
        denominator: denominator / gcd,
        interlace,
        subtype: format!("{subtype:?}"),
    })
}

fn camera_native_types(reader: &IMFSourceReader) -> Result<Vec<(CameraMode, IMFMediaType)>> {
    let mut modes = Vec::new();
    for index in 0..256 {
        // SAFETY: The owned reader stays on its COM worker; bounded SDK type
        // indices return owned interfaces which are released on this thread.
        let media_type = match unsafe { reader.GetNativeMediaType(FIRST_VIDEO, index) } {
            Ok(value) => value,
            Err(error) if error.code() == MF_E_NO_MORE_TYPES => return Ok(modes),
            Err(error) => return mf("enumerate camera formats", Err(error)),
        };
        if let Ok(mode) = camera_mode(&media_type) {
            modes.push((mode, media_type));
        }
    }
    Err(VideoError::Invalid(
        "Camera advertises more than 256 formats",
    ))
}

fn camera_attributes() -> Result<IMFAttributes> {
    let mut attributes = None;
    // SAFETY: Valid exclusive out pointer, then owned attributes on this COM worker.
    unsafe {
        mf("camera attributes", MFCreateAttributes(&mut attributes, 2))?;
        let attributes = attributes.ok_or(VideoError::Invalid("Missing camera attributes"))?;
        mf(
            "camera source type",
            attributes.SetGUID(
                &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
            ),
        )?;
        Ok(attributes)
    }
}

fn camera_string(attributes: &IMFAttributes, key: &GUID) -> Result<String> {
    // SAFETY: Attributes/key are live on their COM thread. The 4097-code-unit
    // buffer is initialized and owned; GetString cannot write beyond its slice.
    unsafe {
        let length = mf("camera string length", attributes.GetStringLength(key))?;
        if length == 0 || length > 4096 {
            return Err(VideoError::Invalid("Camera identity exceeds its bound"));
        }
        let mut text = vec![0; length as usize + 1];
        mf(
            "read camera identity",
            attributes.GetString(key, &mut text, None),
        )?;
        let end = text.iter().position(|c| *c == 0).unwrap_or(text.len());
        String::from_utf16(
            text.get(..end)
                .ok_or(VideoError::Invalid("Camera identity length"))?,
        )
        .map_err(|_| VideoError::Invalid("Camera identity is not valid text"))
    }
}

/// Discovery only: no source is activated or captured by enumeration.
pub fn camera_devices() -> Result<Vec<CameraDevice>> {
    let _runtime = Runtime::start()?;
    let attributes = camera_attributes()?;
    let mut pointer = std::ptr::null_mut();
    let mut count = 0;
    // SAFETY: MF writes an owned CoTaskMem array with count initialized COM
    // references. Windows owns discovery, all interfaces stay on this thread.
    mf("enumerate cameras", unsafe {
        MFEnumDeviceSources(&attributes, &mut pointer, &mut count)
    })?;
    if pointer.is_null() {
        return Ok(Vec::new());
    }
    struct Activations {
        pointer: *mut Option<IMFActivate>,
        count: usize,
    }
    impl Drop for Activations {
        fn drop(&mut self) {
            // SAFETY: Array/count come directly from successful MF enumeration;
            // take releases each owned reference exactly once before freeing array.
            unsafe {
                for item in std::slice::from_raw_parts_mut(self.pointer, self.count) {
                    *item = None;
                }
                CoTaskMemFree(Some(self.pointer.cast()));
            }
        }
    }
    let array = Activations {
        pointer,
        count: count as usize,
    };
    if count > 128 {
        return Err(VideoError::Invalid(
            "More than 128 camera devices are unsupported",
        ));
    }
    let mut result = Vec::new();
    // SAFETY: Same live initialized MF array, bounded to 128. Borrowed only until
    // the RAII owner above drops it; no interface escapes this worker.
    for activation in unsafe { std::slice::from_raw_parts(array.pointer, array.count) }
        .iter()
        .flatten()
    {
        result.push(CameraDevice {
            id: camera_string(
                activation,
                &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
            )?,
            name: camera_string(activation, &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME)?,
        });
    }
    Ok(result)
}

pub struct VideoFile {
    reader: IMFSourceReader,
    _capture: Option<CaptureOwner>,
    _runtime: Runtime,
    format: VideoFormat,
    source_size: [u32; 2],
    duration_100ns: Option<u64>,
    sequence: u64,
    ended: bool,
    pending_discontinuity: bool,
    pending_type_change: bool,
}

impl VideoFile {
    /// Open a local `.mp4` file and request RGB32 downscaled to at most 1920x1080 (Full HD).
    /// A processor may reject resizing; then a bounded native-size RGB32
    /// fallback is attempted and `format()` exposes the actual returned raster.
    pub fn open(path: &Path) -> Result<Self> {
        let wide = local_mp4_path(path)?;
        let runtime = Runtime::start()?;
        let mut attributes = None;
        // SAFETY: The initialized owner thread provides an exclusively borrowed
        // valid out pointer; the returned COM reference is owned by Option and
        // is released before the runtime guard on all success/error paths.
        mf("create reader attributes", unsafe {
            MFCreateAttributes(&mut attributes, 2)
        })?;
        let attributes = attributes.ok_or(VideoError::Invalid(
            "Windows returned no video attribute store",
        ))?;
        // SAFETY: Owned attributes stay live throughout these calls. Values
        // enable the Windows video processor and force this prototype's CPU
        // decode path; no D3D device/surface pointers are supplied.
        unsafe {
            mf(
                "enable video processor",
                attributes.SetUINT32(&MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING, 1),
            )?;
            mf(
                "select CPU decode",
                attributes.SetUINT32(&MF_SOURCE_READER_DISABLE_DXVA, 1),
            )?;
        }
        // SAFETY: `wide` is a validated, NUL-terminated local filename and lives
        // through this synchronous call. Attributes remain live; the returned
        // interface is exclusively managed on the COM/MF owner thread.
        let reader = mf("open local MP4", unsafe {
            MFCreateSourceReaderFromURL(PCWSTR(wide.as_ptr()), &attributes)
        })?;
        Self::from_reader(runtime, None, reader, None)
    }

    fn from_reader(
        runtime: Runtime,
        capture: Option<CaptureOwner>,
        reader: IMFSourceReader,
        selected: Option<IMFMediaType>,
    ) -> Result<Self> {
        // SAFETY: Reader is a live owned interface on its initializing worker.
        // Stream selectors are SDK constants; only video is enabled (no audio
        // device, network URL or capture source is opened by this module).
        let native = unsafe {
            mf(
                "disable unused streams",
                reader.SetStreamSelection(ALL_STREAMS, false),
            )?;
            mf(
                "select video stream",
                reader.SetStreamSelection(FIRST_VIDEO, true),
            )?;
            match selected {
                Some(native) => native,
                None => mf(
                    "inspect native video type",
                    reader.GetNativeMediaType(FIRST_VIDEO, 0),
                )?,
            }
        };
        let source_size = media_size(&native)?;
        let [source_width, source_height] = source_size;
        let target = fit_reference(source_width, source_height);
        if configure_rgb32(&reader, Some(target)).is_err() {
            // The fallback still requires an actual RGB32 decoder/converter.
            // It does not substitute bars, silently claim resizing or infer
            // that a container extension proves codec support.
            configure_rgb32(&reader, None)?;
        }
        // SAFETY: Reader is live on the owning thread and returns an owned type.
        let current = mf("inspect decoded video type", unsafe {
            reader.GetCurrentMediaType(FIRST_VIDEO)
        })?;
        let format = decoded_format(&current)?;
        let duration_100ns = unsafe {
            reader
                .GetPresentationAttribute(0xFFFFFFFF, &MF_PD_DURATION)
                .ok()
                .map(|p| p.Anonymous.Anonymous.Anonymous.uhVal)
                .filter(|&d| d > 0)
        };
        Ok(Self {
            reader,
            _capture: capture,
            _runtime: runtime,
            format,
            source_size,
            duration_100ns,
            sequence: 0,
            ended: false,
            pending_discontinuity: true,
            pending_type_change: false,
        })
    }

    /// Explicit operator-selected camera. Never selects a different device on failure.
    pub fn open_camera(device_id: &str) -> Result<Self> {
        let (runtime, source, reader) = Self::camera_reader(device_id)?;
        Self::from_reader(runtime, Some(source), reader, None)
    }

    fn camera_reader(device_id: &str) -> Result<(Runtime, CaptureOwner, IMFSourceReader)> {
        if device_id.is_empty() || device_id.len() > 4096 || device_id.chars().any(char::is_control)
        {
            return Err(VideoError::Invalid("Invalid camera device ID"));
        }
        let runtime = Runtime::start()?;
        let attributes = camera_attributes()?;
        let wide: Vec<u16> = device_id.encode_utf16().chain(Some(0)).collect();
        // SAFETY: Attributes and NUL-terminated identity live through each call;
        // the selected source and reader are owned on this COM worker only.
        let (source, reader) = unsafe {
            mf(
                "select camera",
                attributes.SetString(
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                    PCWSTR(wide.as_ptr()),
                ),
            )?;
            let source = mf("open selected camera", MFCreateDeviceSource(&attributes))?;
            let mut processing = None;
            mf(
                "camera processor attributes",
                MFCreateAttributes(&mut processing, 2),
            )?;
            let processing =
                processing.ok_or(VideoError::Invalid("Missing camera processor attributes"))?;
            mf(
                "camera conversion",
                processing.SetUINT32(&MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING, 1),
            )?;
            mf(
                "camera low latency",
                processing.SetUINT32(&MF_LOW_LATENCY, 1),
            )?;
            let owner = CaptureOwner(source);
            let reader = mf(
                "open camera reader",
                MFCreateSourceReaderFromMediaSource(&owner.0, &processing),
            )?;
            (owner, reader)
        };
        Ok((runtime, source, reader))
    }

    /// Opens only the explicitly selected device; mode discovery reads no frames.
    pub fn camera_modes(device_id: &str) -> Result<Vec<CameraMode>> {
        let (_runtime, _source, reader) = Self::camera_reader(device_id)?;
        Ok(camera_native_types(&reader)?
            .into_iter()
            .map(|(mode, _)| mode)
            .collect())
    }

    pub fn open_camera_mode(device_id: &str, mode: Option<&CameraMode>) -> Result<Self> {
        let Some(mode) = mode else {
            return Self::open_camera(device_id);
        };
        let (runtime, source, reader) = Self::camera_reader(device_id)?;
        let (_, native) = camera_native_types(&reader)?
            .into_iter()
            .find(|(candidate, _)| candidate == mode)
            .ok_or(VideoError::Invalid(
                "Selected camera format is no longer available; refresh formats",
            ))?;
        // SAFETY: Owned native type and reader live on the same COM worker,
        // no reads are pending; MF retains its own reference to the exact type.
        unsafe {
            mf(
                "select native camera format",
                reader.SetCurrentMediaType(FIRST_VIDEO, None, &native),
            )?;
        }
        let mut camera = Self::from_reader(runtime, Some(source), reader, Some(native))?;
        let size = fit_reference(mode.width, mode.height);
        let rate = Some([mode.numerator, mode.denominator]);
        if configure_rgb32_rate(&camera.reader, Some(size), rate).is_err() {
            configure_rgb32_rate(&camera.reader, None, rate)?;
        }
        // SAFETY: The reader returns an owned current type on its COM worker.
        let current = unsafe {
            mf(
                "inspect selected camera output",
                camera.reader.GetCurrentMediaType(FIRST_VIDEO),
            )?
        };
        camera.format = decoded_format(&current)?;
        if u64::from(camera.format.frame_rate_numerator) * u64::from(mode.denominator)
            != u64::from(mode.numerator) * u64::from(camera.format.frame_rate_denominator)
        {
            return Err(VideoError::Invalid(
                "Camera processor changed the selected frame rate",
            ));
        }
        Ok(camera)
    }

    pub fn format(&self) -> VideoFormat {
        self.format
    }

    pub fn source_size(&self) -> [u32; 2] {
        self.source_size
    }

    /// Perform one synchronous reader operation; this can block its worker.
    pub fn read_frame(&mut self) -> Result<VideoRead> {
        if self.ended {
            return Ok(VideoRead::EndOfStream);
        }
        let mut flags = 0_u32;
        let mut timestamp = 0_i64;
        let mut sample = None;
        // SAFETY: Reader remains on its COM owner thread. Out pointers refer to
        // exclusive initialized stack slots that live through this synchronous
        // call. The returned sample is owned and released before the next call.
        mf("read video sample", unsafe {
            self.reader.ReadSample(
                FIRST_VIDEO,
                0,
                None,
                Some(&mut flags),
                Some(&mut timestamp),
                Some(&mut sample),
            )
        })?;
        if flags & MF_SOURCE_READERF_ERROR.0 as u32 != 0 {
            return Err(VideoError::Invalid(
                "Windows reported a video source failure",
            ));
        }
        if flags & MF_SOURCE_READERF_NATIVEMEDIATYPECHANGED.0 as u32 != 0 {
            // SAFETY: Reader stays on its owning COM thread and returns an
            // owned media type. Revalidate the encoded source raster as well as
            // the processed output, so an input change cannot bypass 4K limits.
            let native = mf("inspect changed native video type", unsafe {
                self.reader.GetNativeMediaType(FIRST_VIDEO, 0)
            })?;
            self.source_size = media_size(&native)?;
        }
        if flags
            & (MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED.0
                | MF_SOURCE_READERF_NATIVEMEDIATYPECHANGED.0) as u32
            != 0
        {
            // SAFETY: This owned reader is still live on its initializing
            // thread; the media type out value is returned as an owned COM ref.
            let current = mf("inspect changed video type", unsafe {
                self.reader.GetCurrentMediaType(FIRST_VIDEO)
            })?;
            self.format = decoded_format(&current)?;
            self.pending_type_change = true;
            self.pending_discontinuity = true;
        }
        self.ended = flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0;
        if flags & MF_SOURCE_READERF_STREAMTICK.0 as u32 != 0 {
            self.pending_discontinuity = true;
        }
        let Some(sample) = sample else {
            return Ok(if self.ended {
                VideoRead::EndOfStream
            } else if flags & MF_SOURCE_READERF_STREAMTICK.0 as u32 != 0 {
                VideoRead::StreamTick {
                    timestamp_100ns: timestamp,
                }
            } else {
                VideoRead::NoSample {
                    timestamp_100ns: timestamp,
                }
            });
        };
        // SAFETY: Sample is owned by this operation on the COM owner thread;
        // attribute getters return scalar values and retain no borrowed data.
        let duration = unsafe {
            match sample.GetSampleDuration() {
                Ok(value) if value > 0 => Some(value),
                Ok(_) => return Err(VideoError::Invalid("Invalid video sample duration")),
                Err(error) if error.code() == MF_E_NO_SAMPLE_DURATION => None,
                Err(error) => return mf("read video sample duration", Err(error)),
            }
        };
        // SAFETY: The sample's inherited attributes remain alive for the call;
        // absence is handled explicitly rather than hiding arbitrary failures.
        let sample_discontinuity = unsafe {
            match sample.GetUINT32(&MFSampleExtension_Discontinuity) {
                Ok(value) => value != 0,
                Err(error) if error.code() == MF_E_ATTRIBUTENOTFOUND => false,
                Err(error) => return mf("read sample discontinuity", Err(error)),
            }
        };
        let pixels = sample_pixels(&sample, self.format)?;
        let next_sequence = self
            .sequence
            .checked_add(1)
            .ok_or(VideoError::Invalid("Video frame sequence exhausted"))?;
        let frame = DecodedVideoFrame {
            width: self.format.width,
            height: self.format.height,
            format: self.format,
            pixels,
            timestamp_100ns: timestamp,
            duration_100ns: duration,
            sequence: self.sequence,
            discontinuity: sample_discontinuity || self.pending_discontinuity,
            media_type_changed: self.pending_type_change,
        };
        self.sequence = next_sequence;
        self.pending_discontinuity = false;
        self.pending_type_change = false;
        Ok(VideoRead::Frame(frame))
    }

    /// Flush and seek in MF's 100 ns clock domain. Keyframe seeking can return
    /// preroll frames before the requested time; callers select by actual PTS.
    pub fn seek_100ns(&mut self, position: i64) -> Result<()> {
        if position < 0 {
            return Err(VideoError::Invalid(
                "Video seek position must not be negative",
            ));
        }
        let value = PROPVARIANT::from(position);
        // SAFETY: Reader and PROPVARIANT are owned and stay on this COM thread.
        // GUID_NULL requests the documented presentation-time 100 ns format.
        // The call borrows `value` only until return; its scalar variant has no
        // caller-owned pointer. This synchronous flush/seek may block worker.
        unsafe {
            mf("flush video before seek", self.reader.Flush(FIRST_VIDEO))?;
            mf(
                "seek local video",
                self.reader.SetCurrentPosition(&GUID::zeroed(), &value),
            )?;
        }
        self.ended = false;
        self.pending_discontinuity = true;
        Ok(())
    }

    pub fn duration_100ns(&self) -> Option<u64> {
        self.duration_100ns
    }

    pub fn duration_ms(&self) -> Option<u64> {
        self.duration_100ns.map(|d| d / 10_000)
    }
}

fn local_mp4_path(path: &Path) -> Result<Vec<u16>> {
    let text = path
        .to_str()
        .ok_or(VideoError::Invalid("Video filename must be valid Unicode"))?;
    if text.len() > 4096
        || text.trim().is_empty()
        || text.chars().any(char::is_control)
        || text.contains("://")
        || text.starts_with("//")
        || text.starts_with(r"\\")
        || !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("mp4"))
    {
        return Err(VideoError::Invalid(
            "Select a local MP4 file (maximum 4096-byte path)",
        ));
    }
    let canonical = path.canonicalize().map_err(VideoError::File)?;
    let resolved = canonical.to_str().ok_or(VideoError::Invalid(
        "Resolved video filename must be valid Unicode",
    ))?;
    // canonicalize produces the extended Windows prefix for normal drive paths.
    // Strip it for the source resolver, but never turn a UNC target into a URL.
    let resolved = resolved.strip_prefix(r"\\?\").unwrap_or(resolved);
    if resolved.len() > 4096
        || resolved.chars().any(char::is_control)
        || !resolved
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic)
        || resolved.as_bytes().get(1) != Some(&b':')
        || resolved
            .char_indices()
            .any(|(index, ch)| ch == ':' && index != 1)
    {
        return Err(VideoError::Invalid(
            "Video must resolve to a local Windows drive file",
        ));
    }
    let drive = resolved
        .as_bytes()
        .first()
        .copied()
        .ok_or(VideoError::Invalid("Missing local video drive"))?;
    let root = [u16::from(drive), u16::from(b':'), u16::from(b'\\'), 0];
    // SAFETY: This fixed UTF-16 drive-root array is NUL terminated and remains
    // live throughout the synchronous kernel call. No COM context is required.
    // Only positively identified local storage is accepted, not mapped shares.
    let drive_type = unsafe { GetDriveTypeW(PCWSTR(root.as_ptr())) };
    if !matches!(
        drive_type,
        DRIVE_FIXED | DRIVE_REMOVABLE | DRIVE_CDROM | DRIVE_RAMDISK
    ) {
        return Err(VideoError::Invalid(
            "Video source must be on a local storage drive",
        ));
    }
    if !canonical.metadata().map_err(VideoError::File)?.is_file() {
        return Err(VideoError::Invalid(
            "Video source must be a regular local file",
        ));
    }
    let capacity = resolved
        .len()
        .checked_add(1)
        .ok_or(VideoError::Invalid("Video filename length overflow"))?;
    let mut wide = Vec::new();
    // Valid UTF-16 has no more code units than its UTF-8 byte count, leaving
    // one reserved element for the NUL terminator without a second allocation.
    wide.try_reserve_exact(capacity)
        .map_err(|_| VideoError::Invalid("Not enough memory for the video filename"))?;
    wide.extend(Path::new(resolved).as_os_str().encode_wide());
    wide.push(0);
    Ok(wide)
}

fn configure_rgb32(reader: &IMFSourceReader, size: Option<[u32; 2]>) -> Result<()> {
    configure_rgb32_rate(reader, size, None)
}
fn configure_rgb32_rate(
    reader: &IMFSourceReader,
    size: Option<[u32; 2]>,
    rate: Option<[u32; 2]>,
) -> Result<()> {
    // SAFETY: All COM values are owned on the initialized worker. Attribute
    // GUIDs and scalar values are SDK constants; SetCurrentMediaType borrows
    // this owned media type for the call and retains its own COM reference.
    unsafe {
        let media_type = mf("create decoded video type", MFCreateMediaType())?;
        mf(
            "set video major type",
            media_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video),
        )?;
        mf(
            "request RGB32",
            media_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32),
        )?;
        mf(
            "request progressive video",
            media_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32),
        )?;
        if let Some([width, height]) = size {
            mf(
                "request bounded decoded size",
                media_type.SetUINT64(
                    &MF_MT_FRAME_SIZE,
                    (u64::from(width) << 32) | u64::from(height),
                ),
            )?;
        }
        if let Some([numerator, denominator]) = rate {
            mf(
                "request selected camera rate",
                media_type.SetUINT64(
                    &MF_MT_FRAME_RATE,
                    (u64::from(numerator) << 32) | u64::from(denominator),
                ),
            )?;
        }
        mf(
            "configure RGB32 decoder",
            reader.SetCurrentMediaType(FIRST_VIDEO, None, &media_type),
        )
    }
}

fn media_size(media_type: &IMFMediaType) -> Result<[u32; 2]> {
    // SAFETY: Caller owns the live media type on the initialized worker;
    // GetUINT64 returns a scalar packed pair, retaining no application pointer.
    let packed = mf("read video dimensions", unsafe {
        media_type.GetUINT64(&MF_MT_FRAME_SIZE)
    })?;
    let width = (packed >> 32) as u32;
    let height = packed as u32;
    validate_dimensions(width, height)?;
    Ok([width, height])
}

fn validate_dimensions(width: u32, height: u32) -> Result<()> {
    if width == 0 || height == 0 || width > MAX_VIDEO_WIDTH || height > MAX_VIDEO_HEIGHT {
        return Err(VideoError::Invalid(
            "Video raster must be 1–4096 by 1–2160 pixels",
        ));
    }
    Ok(())
}

fn fit_reference(width: u32, height: u32) -> [u32; 2] {
    if width <= REFERENCE_WIDTH && height <= REFERENCE_HEIGHT {
        [width, height]
    } else if u64::from(width) * u64::from(REFERENCE_HEIGHT)
        >= u64::from(height) * u64::from(REFERENCE_WIDTH)
    {
        [
            REFERENCE_WIDTH,
            ((u64::from(height) * u64::from(REFERENCE_WIDTH)) / u64::from(width)) as u32,
        ]
        .map(|dimension| dimension.max(1))
    } else {
        [
            ((u64::from(width) * u64::from(REFERENCE_HEIGHT)) / u64::from(height)) as u32,
            REFERENCE_HEIGHT,
        ]
        .map(|dimension| dimension.max(1))
    }
}

fn optional_attribute(media_type: &IMFMediaType, key: &GUID) -> Result<Option<u32>> {
    // SAFETY: Owned media type and borrowed GUID outlive the synchronous scalar
    // getter; only the documented absent-attribute error becomes None.
    match unsafe { media_type.GetUINT32(key) } {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.code() == MF_E_ATTRIBUTENOTFOUND => Ok(None),
        Err(error) => mf("read video attribute", Err(error)),
    }
}

fn decoded_format(media_type: &IMFMediaType) -> Result<VideoFormat> {
    let [width, height] = media_size(media_type)?;
    // SAFETY: Type is an owned COM reference on its worker. These operations
    // read scalar metadata only; borrowed GUID addresses live through each call.
    let (major, subtype, packed_rate, aspect) = unsafe {
        (
            mf("read major type", media_type.GetGUID(&MF_MT_MAJOR_TYPE))?,
            mf("read pixel subtype", media_type.GetGUID(&MF_MT_SUBTYPE))?,
            mf("read video rate", media_type.GetUINT64(&MF_MT_FRAME_RATE))?,
            match media_type.GetUINT64(&MF_MT_PIXEL_ASPECT_RATIO) {
                Ok(value) => value,
                Err(error) if error.code() == MF_E_ATTRIBUTENOTFOUND => (1_u64 << 32) | 1,
                Err(error) => return mf("read pixel aspect ratio", Err(error)),
            },
        )
    };
    if major != MFMediaType_Video || subtype != MFVideoFormat_RGB32 {
        return Err(VideoError::Invalid("Decoder did not negotiate RGB32 video"));
    }
    let numerator = (packed_rate >> 32) as u32;
    let denominator = packed_rate as u32;
    if numerator == 0 || denominator == 0 || u64::from(numerator) > 120 * u64::from(denominator) {
        return Err(VideoError::Invalid(
            "Decoded frame rate must be greater than zero and at most 120 fps",
        ));
    }
    let aspect_numerator = (aspect >> 32) as u32;
    let aspect_denominator = aspect as u32;
    if aspect_numerator == 0 || aspect_denominator == 0 {
        return Err(VideoError::Invalid("Invalid decoded pixel aspect ratio"));
    }
    if optional_attribute(media_type, &MF_MT_INTERLACE_MODE)?
        != Some(MFVideoInterlace_Progressive.0 as u32)
    {
        return Err(VideoError::Invalid(
            "Decoder must provide progressive RGB32 frames",
        ));
    }
    let stride = match optional_attribute(media_type, &MF_MT_DEFAULT_STRIDE)? {
        // MF stores signed pitch's bit representation in this UINT32 attribute.
        Some(value) => i32::from_ne_bytes(value.to_ne_bytes()),
        None => {
            // SAFETY: Verified RGB32 subtype and bounded width are passed as
            // scalar SDK bitmap-format/width values; no pointers are involved.
            mf("derive RGB32 stride", unsafe {
                MFGetStrideForBitmapInfoHeader(MFVideoFormat_RGB32.data1, width)
            })?
        }
    };
    validate_stride(width, height, stride)?;
    Ok(VideoFormat {
        width,
        height,
        stride,
        frame_rate_numerator: numerator,
        frame_rate_denominator: denominator,
        pixel_aspect_numerator: aspect_numerator,
        pixel_aspect_denominator: aspect_denominator,
        video_primaries: optional_attribute(media_type, &MF_MT_VIDEO_PRIMARIES)?,
        transfer_function: optional_attribute(media_type, &MF_MT_TRANSFER_FUNCTION)?,
        nominal_range: optional_attribute(media_type, &MF_MT_VIDEO_NOMINAL_RANGE)?,
        yuv_matrix: optional_attribute(media_type, &MF_MT_YUV_MATRIX)?,
    })
}

fn validate_stride(width: u32, height: u32, stride: i32) -> Result<()> {
    validate_dimensions(width, height)?;
    let minimum = width
        .checked_mul(4)
        .ok_or(VideoError::Invalid("Video row size overflow"))?;
    let length = u64::from(stride.unsigned_abs())
        .checked_mul(u64::from(height))
        .ok_or(VideoError::Invalid("Video buffer size overflow"))?;
    if stride.unsigned_abs() < minimum || length > MAX_VIDEO_BUFFER_BYTES as u64 {
        return Err(VideoError::Invalid("Invalid or oversized RGB32 stride"));
    }
    Ok(())
}

/// Safe independent row conversion, including padding and bottom-up buffers.
/// `scanline_zero` is the byte offset of the top logical row in `bytes`.
/// Bounds are checked before allocation/access; RGB32 is B,G,R,unused.
pub fn copy_rgb32_rows(
    bytes: &[u8],
    width: u32,
    height: u32,
    stride: i32,
    scanline_zero: usize,
) -> Result<Vec<[u8; 4]>> {
    convert_rgb32_rows(
        bytes.len(),
        width,
        height,
        stride,
        scanline_zero,
        |offset| {
            let end = offset
                .checked_add(4)
                .ok_or(VideoError::Invalid("Video pixel offset overflow"))?;
            let pixel = bytes
                .get(offset..end)
                .ok_or(VideoError::Invalid("Truncated RGB32 pixel"))?;
            let [blue, green, red, _] = pixel else {
                return Err(VideoError::Invalid("Incomplete RGB32 pixel"));
            };
            Ok([*red, *green, *blue, 255])
        },
    )
}

fn convert_rgb32_rows(
    byte_length: usize,
    width: u32,
    height: u32,
    stride: i32,
    scanline_zero: usize,
    mut read_pixel: impl FnMut(usize) -> Result<[u8; 4]>,
) -> Result<Vec<[u8; 4]>> {
    validate_stride(width, height, stride)?;
    if byte_length > MAX_VIDEO_BUFFER_BYTES {
        return Err(VideoError::Invalid("Decoded buffer exceeds 64 MiB"));
    }
    let row_bytes = usize::try_from(width)
        .ok()
        .and_then(|value| value.checked_mul(4))
        .ok_or(VideoError::Invalid("Video row size overflow"))?;
    let row_offset = |row: u32| -> Result<usize> {
        let start = i128::try_from(scanline_zero)
            .ok()
            .and_then(|base| base.checked_add(i128::from(stride) * i128::from(row)))
            .and_then(|value| usize::try_from(value).ok())
            .ok_or(VideoError::Invalid(
                "RGB32 row offset is outside the buffer",
            ))?;
        let end = start
            .checked_add(row_bytes)
            .ok_or(VideoError::Invalid("Video row overflow"))?;
        if end > byte_length {
            return Err(VideoError::Invalid("Truncated RGB32 video buffer"));
        }
        Ok(start)
    };
    row_offset(0)?;
    row_offset(height - 1)?;
    let count = usize::try_from(width)
        .ok()
        .zip(usize::try_from(height).ok())
        .and_then(|(w, h)| w.checked_mul(h))
        .ok_or(VideoError::Invalid("Video pixel count overflow"))?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(count)
        .map_err(|_| VideoError::Invalid("Not enough memory for decoded video"))?;
    for row in 0..height {
        let start = row_offset(row)?;
        for column in 0..width {
            let offset = usize::try_from(column)
                .ok()
                .and_then(|column| column.checked_mul(4))
                .and_then(|column| start.checked_add(column))
                .ok_or(VideoError::Invalid("Video pixel offset overflow"))?;
            pixels.push(read_pixel(offset)?);
        }
    }
    Ok(pixels)
}

/// # Safety
/// `start` must identify one locked MF allocation of at least `byte_length`
/// accessible bytes. Each validated pixel's B/G/R channels must contain decoded
/// data. The caller must retain the lock, sample and buffer on their owning
/// thread until this synchronous conversion returns. Padding and X are unread.
unsafe fn copy_locked_rgb32_rows(
    start: *const u8,
    byte_length: usize,
    width: u32,
    height: u32,
    stride: i32,
    scanline_zero: usize,
) -> Result<Vec<[u8; 4]>> {
    convert_rgb32_rows(
        byte_length,
        width,
        height,
        stride,
        scanline_zero,
        |offset| {
            // SAFETY: The enclosing function's caller retains the matching MF lock
            // and owned buffer for this entire conversion. convert_rgb32_rows has
            // checked this pixel's four-byte span lies within that allocation.
            // Only the three decoded B/G/R bytes are read; padding and the undefined
            // RGB32 X byte are never read or exposed through a Rust byte slice.
            let pixel = unsafe {
                let pixel = start.add(offset);
                [pixel.add(2).read(), pixel.add(1).read(), pixel.read(), 255]
            };
            Ok(pixel)
        },
    )
}

struct Locked2D<'a>(&'a IMF2DBuffer2);
impl Drop for Locked2D<'_> {
    fn drop(&mut self) {
        // SAFETY: Guard exists only after a successful Lock2DSize; its borrowed
        // interface stays alive on this owner thread until the single unlock.
        let _ = unsafe { self.0.Unlock2D() };
    }
}

struct LockedBuffer<'a>(&'a IMFMediaBuffer);
impl Drop for LockedBuffer<'_> {
    fn drop(&mut self) {
        // SAFETY: Guard exists only after successful Lock and borrows the owned
        // live interface on this worker; no slice survives the matching unlock.
        let _ = unsafe { self.0.Unlock() };
    }
}

fn sample_pixels(sample: &IMFSample, format: VideoFormat) -> Result<Vec<[u8; 4]>> {
    // SAFETY: Sample is owned on this COM worker. These scalar getters do not
    // retain application pointers or copy/allocate pixel storage.
    let (count, length) = unsafe {
        (
            mf("inspect video buffer count", sample.GetBufferCount())?,
            mf("inspect video sample length", sample.GetTotalLength())?,
        )
    };
    if count == 0 || count > 16 || length == 0 || u64::from(length) > MAX_VIDEO_BUFFER_BYTES as u64
    {
        return Err(VideoError::Invalid(
            "Invalid or oversized decoded sample buffers",
        ));
    }
    // SAFETY: The owned sample remains live, buffer count is in 1..=16 and its
    // total data is bounded before any contiguous copy. Index 0 exists when
    // used. Returning an owned reference retains the buffer through conversion.
    let buffer = mf("get decoded video buffer", unsafe {
        if count == 1 {
            sample.GetBufferByIndex(0)
        } else {
            sample.ConvertToContiguousBuffer()
        }
    })?;
    if let Ok(buffer2d) = buffer.cast::<IMF2DBuffer2>() {
        let mut scanline = std::ptr::null_mut();
        let mut pitch = 0_i32;
        let mut start = std::ptr::null_mut();
        let mut length = 0_u32;
        // SAFETY: Exclusive stack out-slots live through Lock2DSize. The SDK
        // guarantees start/length identify readable locked buffer storage and
        // scanline/pitch describe its first logical row until Unlock2D.
        mf("lock decoded 2D buffer", unsafe {
            buffer2d.Lock2DSize(
                MF2DBuffer_LockFlags_Read,
                &mut scanline,
                &mut pitch,
                &mut start,
                &mut length,
            )
        })?;
        let _guard = Locked2D(&buffer2d);
        let length = usize::try_from(length)
            .map_err(|_| VideoError::Invalid("Video buffer length overflow"))?;
        if start.is_null() || scanline.is_null() || length == 0 || length > MAX_VIDEO_BUFFER_BYTES {
            return Err(VideoError::Invalid("Invalid decoded 2D buffer span"));
        }
        let offset = scanline
            .addr()
            .checked_sub(start.addr())
            .ok_or(VideoError::Invalid("Invalid decoded scanline address"))?;
        if offset >= length {
            return Err(VideoError::Invalid(
                "Decoded scanline is outside its buffer",
            ));
        }
        // SAFETY: Successful Lock2DSize promises this non-null allocation span
        // and decoded channel bytes remain accessible until the guard unlocks.
        // The guard and owned COM interfaces remain on this worker through the
        // call. Signed row bounds are checked; no padding/X bytes are read.
        unsafe {
            copy_locked_rgb32_rows(
                start.cast_const(),
                length,
                format.width,
                format.height,
                pitch,
                offset,
            )
        }
    } else {
        let mut start = std::ptr::null_mut();
        let mut maximum = 0_u32;
        let mut current = 0_u32;
        // SAFETY: Owned buffer is live; all exclusive out-slots live through the
        // call. A successful lock promises readable current bytes until Unlock.
        mf("lock decoded buffer", unsafe {
            buffer.Lock(&mut start, Some(&mut maximum), Some(&mut current))
        })?;
        let _guard = LockedBuffer(&buffer);
        let length = usize::try_from(current)
            .map_err(|_| VideoError::Invalid("Video buffer length overflow"))?;
        if start.is_null() || current > maximum || length == 0 || length > MAX_VIDEO_BUFFER_BYTES {
            return Err(VideoError::Invalid("Invalid decoded buffer span"));
        }
        let offset = if format.stride < 0 {
            usize::try_from(format.stride.unsigned_abs())
                .ok()
                .zip(usize::try_from(format.height - 1).ok())
                .and_then(|(pitch, rows)| pitch.checked_mul(rows))
                .ok_or(VideoError::Invalid("Video row origin overflow"))?
        } else {
            0
        };
        // SAFETY: Lock guarantees the decoded pixel data remains accessible
        // within current<=maximum allocation bytes until this guard unlocks.
        // The guard and owned buffer outlive conversion on this worker; only
        // bounds-checked B/G/R channels are read, never padding or unused alpha.
        unsafe {
            copy_locked_rgb32_rows(
                start.cast_const(),
                length,
                format.width,
                format.height,
                format.stride,
                offset,
            )
        }
    }
}
