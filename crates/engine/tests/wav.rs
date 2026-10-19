#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use rustmix_engine::wav::*;

fn riff(format: u16, channels: u16, rate: u32, bits: u16, data: &[u8]) -> Vec<u8> {
    let align = channels * (bits / 8);
    let mut chunks = Vec::new();
    chunks.extend_from_slice(b"fmt ");
    chunks.extend_from_slice(&16_u32.to_le_bytes());
    chunks.extend_from_slice(&format.to_le_bytes());
    chunks.extend_from_slice(&channels.to_le_bytes());
    chunks.extend_from_slice(&rate.to_le_bytes());
    chunks.extend_from_slice(&(rate * u32::from(align)).to_le_bytes());
    chunks.extend_from_slice(&align.to_le_bytes());
    chunks.extend_from_slice(&bits.to_le_bytes());
    chunks.extend_from_slice(b"data");
    chunks.extend_from_slice(&(data.len() as u32).to_le_bytes());
    chunks.extend_from_slice(data);
    if !data.len().is_multiple_of(2) {
        chunks.push(0);
    }
    let mut bytes = b"RIFF".to_vec();
    bytes.extend_from_slice(&((chunks.len() + 4) as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend(chunks);
    bytes
}

#[test]
fn pcm16_mono_duplicates_channels_and_stereo_preserves_polarity() {
    let bytes = riff(1, 1, 48000, 16, &[0x00, 0x80, 0x00, 0x40, 0xff, 0x7f]);
    let wav = WavAudio::decode(&bytes).unwrap();
    assert_eq!(wav.sample_rate(), 48000);
    assert_eq!(wav.frames()[0], [-1.0, -1.0]);
    assert_eq!(wav.frames()[1], [0.5, 0.5]);
    assert_eq!(wav.frames()[2], [32767.0 / 32768.0; 2]);
    let bytes = riff(1, 2, 44100, 16, &[0x00, 0x40, 0x00, 0xc0]);
    let wav = WavAudio::decode(&bytes).unwrap();
    assert_eq!(wav.sample_rate(), 44100); // no silent rate conversion
    assert_eq!(wav.frames(), &[[0.5, -0.5]]);
}

#[test]
fn float32_decodes_actual_samples_and_rejects_nonfinite_values() {
    let data: Vec<u8> = [0.25_f32, -0.5, 1.25, -1.25]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    let wav = WavAudio::decode(&riff(3, 2, 48000, 32, &data)).unwrap();
    assert_eq!(wav.frames(), &[[0.25, -0.5], [1.25, -1.25]]);
    for sample in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(WavAudio::decode(&riff(3, 1, 48000, 32, &sample.to_le_bytes())).is_err());
    }
}

#[test]
fn odd_unknown_chunks_require_padding_and_do_not_change_pcm() {
    let mut bytes = riff(1, 1, 48000, 16, &0x4000_i16.to_le_bytes());
    let chunk = [b'J', b'U', b'N', b'K', 1, 0, 0, 0, 7, 0];
    bytes.splice(12..12, chunk);
    let size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&size.to_le_bytes());
    assert_eq!(WavAudio::decode(&bytes).unwrap().frames(), &[[0.5; 2]]);
    bytes.remove(21); // removes required unknown-chunk pad
    let size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&size.to_le_bytes());
    assert!(WavAudio::decode(&bytes).is_err());
}

#[test]
fn truncated_or_malformed_headers_and_unsupported_formats_are_rejected() {
    let valid = riff(1, 2, 48000, 16, &[0; 8]);
    for length in 0..valid.len() {
        assert!(WavAudio::decode(&valid[..length]).is_err());
        if length >= 8 {
            let mut truncated = valid[..length].to_vec();
            truncated[4..8].copy_from_slice(&((length - 8) as u32).to_le_bytes());
            assert!(WavAudio::decode(&truncated).is_err());
        }
    }
    for (format, channels, rate, bits) in [
        (6, 2, 48000, 16),
        (1, 3, 48000, 16),
        (1, 2, 0, 16),
        (1, 2, 48000, 24),
        (3, 2, 48000, 16),
    ] {
        assert!(WavAudio::decode(&riff(format, channels, rate, bits, &[0; 8])).is_err());
    }
    let mut wrong_align = valid.clone();
    wrong_align[32..34].copy_from_slice(&1_u16.to_le_bytes());
    assert!(WavAudio::decode(&wrong_align).is_err());
    let mut wrong_rate = valid.clone();
    wrong_rate[28..32].copy_from_slice(&1_u32.to_le_bytes());
    assert!(WavAudio::decode(&wrong_rate).is_err());
    assert!(WavAudio::decode(&riff(1, 2, 48000, 16, &[0; 3])).is_err());
    assert!(WavAudio::decode(&riff(1, 1, 48000, 16, &[])).is_err());
}

#[test]
fn duplicate_data_and_forged_chunk_lengths_fail_before_decoding() {
    let mut bytes = riff(1, 1, 48000, 16, &[0, 0]);
    bytes.extend_from_slice(b"data\x02\x00\x00\x00\x00\x00");
    let size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&size.to_le_bytes());
    assert!(WavAudio::decode(&bytes).is_err());
    let mut huge = riff(1, 1, 48000, 16, &[0, 0]);
    huge[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(WavAudio::decode(&huge).is_err());
    let mut trailing = riff(1, 1, 48000, 16, &[0, 0]);
    trailing.push(0);
    assert!(WavAudio::decode(&trailing).is_err());
}

#[test]
fn duration_limits_missing_format_and_duplicate_format_are_explicit_errors() {
    let long = riff(
        1,
        1,
        8000,
        16,
        &vec![0; (8000 * MAX_WAV_DURATION_SECONDS as usize + 1) * 2],
    );
    assert!(WavAudio::decode(&long).is_err());
    let mut duplicate = riff(1, 1, 48000, 16, &[0, 0]);
    let format_chunk = duplicate[12..36].to_vec();
    duplicate.extend_from_slice(&format_chunk);
    let size = (duplicate.len() - 8) as u32;
    duplicate[4..8].copy_from_slice(&size.to_le_bytes());
    assert!(WavAudio::decode(&duplicate).is_err());
    let mut missing = riff(1, 1, 48000, 16, &[0, 0]);
    missing.drain(12..36);
    let size = (missing.len() - 8) as u32;
    missing[4..8].copy_from_slice(&size.to_le_bytes());
    assert!(WavAudio::decode(&missing).is_err());
}

#[test]
fn zero_length_format_extension_and_data_before_format_are_supported() {
    let valid = riff(1, 1, 48000, 16, &0x4000_i16.to_le_bytes());
    let mut extension = valid.clone();
    extension.splice(36..36, [0, 0]);
    extension[16..20].copy_from_slice(&18_u32.to_le_bytes());
    let size = (extension.len() - 8) as u32;
    extension[4..8].copy_from_slice(&size.to_le_bytes());
    assert_eq!(WavAudio::decode(&extension).unwrap().frames(), &[[0.5; 2]]);
    extension[36] = 1;
    assert!(WavAudio::decode(&extension).is_err());
    let mut reordered = valid[..12].to_vec();
    reordered.extend_from_slice(&valid[36..]);
    reordered.extend_from_slice(&valid[12..36]);
    assert_eq!(WavAudio::decode(&reordered).unwrap().frames(), &[[0.5; 2]]);
}
