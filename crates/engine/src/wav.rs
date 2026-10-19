//! Strict, bounded RIFF/WAVE PCM16 and IEEE-float32 ingest, outside audio render.
//!
//! Mono duplicates into stereo without changing gain. The original rate is
//! retained: playback must explicitly resample before the 48 kHz mixer. No
//! compressed, extensible, RF64, RIFX, multichannel or implicit format fallback.
use crate::{
    Result,
    audio_engine::{MAX_SAMPLE_MAGNITUDE, StereoFrame},
    invalid,
};

pub const MAX_WAV_BYTES: usize = 64 * 1024 * 1024;
/// Bound decoded storage to approximately 44 MiB, independently of encoded size.
pub const MAX_WAV_FRAMES: usize = 5_760_000;
pub const MAX_WAV_DURATION_SECONDS: u32 = 120;
const MAX_CHUNKS: usize = 4096;

#[derive(Debug, Clone)]
pub struct WavAudio {
    sample_rate: u32,
    frames: Vec<StereoFrame>,
}
impl WavAudio {
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub fn frames(&self) -> &[StereoFrame] {
        &self.frames
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_WAV_BYTES || bytes.len() < 12 {
            return Err(invalid(
                "WAV must contain a bounded RIFF header (maximum 64 MiB)",
            ));
        }
        if bytes.get(..4) != Some(b"RIFF".as_slice())
            || bytes.get(8..12) != Some(b"WAVE".as_slice())
        {
            return Err(invalid("Only little-endian RIFF/WAVE is supported"));
        }
        let riff_end = usize::try_from(read_u32(bytes, 4)?)
            .ok()
            .and_then(|size| size.checked_add(8))
            .ok_or_else(|| invalid("WAV RIFF length overflow"))?;
        if riff_end != bytes.len() {
            return Err(invalid("WAV RIFF length does not match file length"));
        }
        let mut cursor = 12_usize;
        let mut format = None;
        let mut data = None;
        let mut chunks = 0_usize;
        while cursor < riff_end {
            chunks = chunks
                .checked_add(1)
                .ok_or_else(|| invalid("WAV chunk count overflow"))?;
            if chunks > MAX_CHUNKS {
                return Err(invalid("WAV contains too many chunks"));
            }
            let payload_start = cursor
                .checked_add(8)
                .ok_or_else(|| invalid("WAV chunk offset overflow"))?;
            let header = bytes
                .get(cursor..payload_start)
                .ok_or_else(|| invalid("Truncated WAV chunk header"))?;
            let size = usize::try_from(read_u32(header, 4)?)
                .map_err(|_| invalid("WAV chunk size is not representable"))?;
            let payload_end = payload_start
                .checked_add(size)
                .ok_or_else(|| invalid("WAV chunk length overflow"))?;
            let payload = bytes
                .get(payload_start..payload_end)
                .ok_or_else(|| invalid("Truncated WAV chunk payload"))?;
            match header.get(..4) {
                Some(b"fmt ") => {
                    if format.is_some() {
                        return Err(invalid("Duplicate WAV format chunk"));
                    }
                    format = Some(Format::parse(payload)?);
                }
                Some(b"data") => {
                    if data.is_some() {
                        return Err(invalid("Duplicate WAV data chunk"));
                    }
                    data = Some(payload);
                }
                _ => {} // Unknown chunks are length-checked and skipped, including pad.
            }
            cursor = payload_end
                .checked_add(size % 2)
                .ok_or_else(|| invalid("WAV chunk padding overflow"))?;
            if cursor > riff_end {
                return Err(invalid("Truncated WAV chunk padding"));
            }
        }
        let format = format.ok_or_else(|| invalid("WAV format chunk is missing"))?;
        let data = data.ok_or_else(|| invalid("WAV data chunk is missing"))?;
        let alignment = usize::from(format.block_align);
        if data.is_empty() || data.len() % alignment != 0 {
            return Err(invalid(
                "WAV data must contain complete nonempty audio frames",
            ));
        }
        let frame_count = data.len() / alignment;
        let duration_bound = usize::try_from(format.rate)
            .ok()
            .and_then(|rate| rate.checked_mul(MAX_WAV_DURATION_SECONDS as usize))
            .ok_or_else(|| invalid("WAV duration bound overflow"))?;
        if frame_count > MAX_WAV_FRAMES || frame_count > duration_bound {
            return Err(invalid(
                "WAV exceeds decoded-frame or 120-second duration limit",
            ));
        }
        let mut frames = Vec::new();
        frames
            .try_reserve_exact(frame_count)
            .map_err(|_| invalid("WAV decoded-frame allocation failed"))?;
        for frame in data.chunks_exact(alignment) {
            let left = format.sample(frame, 0)?;
            let right = if format.channels == 1 {
                left
            } else {
                format.sample(frame, usize::from(format.bits / 8))?
            };
            frames.push([left, right]);
        }
        Ok(Self {
            sample_rate: format.rate,
            frames,
        })
    }
}

#[derive(Clone, Copy)]
struct Format {
    encoding: u16,
    channels: u16,
    rate: u32,
    block_align: u16,
    bits: u16,
}
impl Format {
    fn parse(bytes: &[u8]) -> Result<Self> {
        if !matches!(bytes.len(), 16 | 18) {
            return Err(invalid("WAV format must be PCM/float with no extension"));
        }
        if bytes.len() == 18 && read_u16(bytes, 16)? != 0 {
            return Err(invalid("WAV format extensions are unsupported"));
        }
        let encoding = read_u16(bytes, 0)?;
        let channels = read_u16(bytes, 2)?;
        let rate = read_u32(bytes, 4)?;
        let byte_rate = read_u32(bytes, 8)?;
        let block_align = read_u16(bytes, 12)?;
        let bits = read_u16(bytes, 14)?;
        if !matches!((encoding, bits), (1, 16) | (3, 32))
            || !matches!(channels, 1 | 2)
            || !(8000..=192000).contains(&rate)
        {
            return Err(invalid(
                "WAV requires PCM16/float32 mono/stereo at 8000-192000 Hz",
            ));
        }
        let expected_align = channels
            .checked_mul(bits / 8)
            .ok_or_else(|| invalid("WAV block alignment overflow"))?;
        let expected_rate = rate
            .checked_mul(u32::from(expected_align))
            .ok_or_else(|| invalid("WAV byte rate overflow"))?;
        if block_align != expected_align || byte_rate != expected_rate {
            return Err(invalid("WAV block alignment or byte rate is inconsistent"));
        }
        Ok(Self {
            encoding,
            channels,
            rate,
            block_align,
            bits,
        })
    }
    fn sample(self, bytes: &[u8], offset: usize) -> Result<f32> {
        let sample = match self.encoding {
            1 => f32::from(i16::from_le_bytes(read_array::<2>(bytes, offset)?)) / 32768.0,
            3 => f32::from_le_bytes(read_array::<4>(bytes, offset)?),
            _ => return Err(invalid("Unsupported WAV sample encoding")),
        };
        if !sample.is_finite() || sample.abs() > MAX_SAMPLE_MAGNITUDE {
            return Err(invalid("WAV sample must be finite and within +/-16"));
        }
        Ok(sample)
    }
}
fn read_array<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N]> {
    let end = offset
        .checked_add(N)
        .ok_or_else(|| invalid("WAV scalar offset overflow"))?;
    bytes
        .get(offset..end)
        .and_then(|value| value.try_into().ok())
        .ok_or_else(|| invalid("Truncated WAV scalar"))
}
fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(read_array(bytes, offset)?))
}
fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(read_array(bytes, offset)?))
}
