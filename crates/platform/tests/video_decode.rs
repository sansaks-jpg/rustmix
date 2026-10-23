#![cfg(windows)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use rustmix_platform::media_foundation::{DecodedVideoFrame, VideoFile, VideoFormat, VideoRead};
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    // CI runs scripts/ci-video-fixture.ps1 before the workspace tests and exports
    // RUSTMIX_TEST_VIDEO (docs/testing.md). Local `cargo test` runs without that
    // export, so fall back to the script's own default output location. Either
    // way a missing fixture is a hard failure, never a skipped test.
    let path = match std::env::var_os("RUSTMIX_TEST_VIDEO") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../artifacts/video-fixture.mp4"),
    };
    assert!(
        path.is_absolute(),
        "Video test fixture path must be absolute"
    );
    assert!(
        path.is_file(),
        "Generated MP4 fixture is missing at {}; run scripts/ci-video-fixture.ps1 or set RUSTMIX_TEST_VIDEO",
        path.display()
    );
    path
}

fn assert_fixture_format(format: VideoFormat) {
    assert_eq!([format.width, format.height], [160, 90]);
    assert_eq!(
        u64::from(format.frame_rate_numerator),
        30 * u64::from(format.frame_rate_denominator)
    );
    assert_ne!(format.frame_rate_denominator, 0);
    assert!(format.stride.unsigned_abs() >= 160 * 4);
    assert!(format.stride.unsigned_abs() <= 64 * 1024 * 1024 / 90);
    assert_eq!(
        format.pixel_aspect_numerator,
        format.pixel_aspect_denominator
    );
    assert_ne!(format.pixel_aspect_denominator, 0);
}

fn assert_rgb(frame: &DecodedVideoFrame, x: usize, y: usize, expected: [u8; 3]) {
    let offset = y * usize::try_from(frame.width).unwrap() + x;
    let actual = frame.pixels[offset];
    for (channel, expected) in actual.iter().take(3).zip(expected) {
        assert!(
            channel.abs_diff(expected) <= 32,
            "Unexpected decoded RGB at ({x}, {y}): {actual:?}, expected {expected} within 32"
        );
    }
    assert_eq!(actual[3], 255);
}

#[test]
fn generated_mp4_decodes_actual_colours_motion_pts_eof_and_seek() {
    let mut video =
        VideoFile::open(&fixture_path()).expect("Decode the generated H.264 MP4 fixture");
    assert_eq!(video.source_size(), [160, 90]);
    assert_fixture_format(video.format());
    let mut first: Option<DecodedVideoFrame> = None;
    let mut last: Option<DecodedVideoFrame> = None;
    let mut previous_pts = None;
    let mut previous_sequence = None;
    let mut frames = 0_usize;
    let mut reached_eof = false;
    for _ in 0..120 {
        match video.read_frame().expect("Read a decoded fixture frame") {
            VideoRead::Frame(frame) => {
                assert_fixture_format(frame.format);
                assert_eq!([frame.width, frame.height], [160, 90]);
                assert_eq!(frame.pixels.len(), 160 * 90);
                assert!(frame.pixels.iter().all(|pixel| pixel[3] == 255));
                if let Some(previous) = previous_pts {
                    assert!(
                        frame.timestamp_100ns > previous,
                        "Fixture PTS must progress"
                    );
                }
                if let Some(previous) = previous_sequence {
                    assert_eq!(frame.sequence, previous + 1);
                }
                if let Some(duration) = frame.duration_100ns {
                    assert!(duration > 0 && duration < 10_000_000);
                }
                previous_pts = Some(frame.timestamp_100ns);
                previous_sequence = Some(frame.sequence);
                frames += 1;
                if first.is_none() {
                    assert!(
                        frame.discontinuity,
                        "Initial source frame starts a new clock mapping"
                    );
                    assert!((0..=333_334).contains(&frame.timestamp_100ns));
                    first = Some(frame);
                } else {
                    last = Some(frame);
                }
            }
            VideoRead::StreamTick { .. } | VideoRead::NoSample { .. } => {}
            VideoRead::EndOfStream => {
                reached_eof = true;
                break;
            }
        }
    }
    assert!(
        reached_eof,
        "Fixture must reach EOF within 120 bounded reads"
    );
    assert_eq!(frames, 60, "Every generated frame must decode exactly once");
    assert!(matches!(
        video.read_frame().unwrap(),
        VideoRead::EndOfStream
    ));
    let first = first.expect("The fixture must yield its first frame");
    let last = last.expect("The fixture must yield later frames");
    assert!((19_000_000..21_000_000).contains(&last.timestamp_100ns));
    for frame in [&first, &last] {
        assert_rgb(frame, 40, 20, [255, 0, 0]);
        assert_rgb(frame, 120, 20, [0, 255, 0]);
        assert_rgb(frame, 40, 65, [0, 0, 255]);
        assert_rgb(frame, 120, 60, [255, 255, 255]);
    }
    // The original generated square travels right; these independent points
    // reject a frozen image, wrong source, inverted rows or swapped channels.
    assert_rgb(&first, 89, 78, [0, 0, 0]);
    assert_rgb(&first, 148, 78, [255, 255, 255]);
    assert_rgb(&last, 89, 78, [255, 255, 255]);
    assert_rgb(&last, 148, 78, [0, 0, 0]);

    assert!(video.seek_100ns(-1).is_err());
    video
        .seek_100ns(10_000_000)
        .expect("Seek after reaching EOF");
    let mut seek_frame = None;
    for _ in 0..30 {
        match video.read_frame().expect("Read after seek") {
            VideoRead::Frame(frame) => {
                seek_frame = Some(frame);
                break;
            }
            VideoRead::StreamTick { .. } | VideoRead::NoSample { .. } => {}
            VideoRead::EndOfStream => panic!("Seeking into the fixture must leave EOF state"),
        }
    }
    let seek_frame = seek_frame.expect("Seek must produce a frame within 30 reads");
    assert!(
        seek_frame.discontinuity,
        "Seek must reset timestamp mapping explicitly"
    );
    assert_eq!(seek_frame.sequence, last.sequence + 1);
    assert!((0..=10_333_334).contains(&seek_frame.timestamp_100ns));
    assert_fixture_format(seek_frame.format);
    assert_rgb(&seek_frame, 40, 20, [255, 0, 0]);
}

#[test]
fn corrupt_regular_mp4_is_reported_as_an_error_and_a_valid_source_still_opens() {
    let directory = tempfile::tempdir().unwrap();
    let corrupt = directory.path().join("corrupt.mp4");
    std::fs::write(
        &corrupt,
        b"RustMix original intentionally malformed MP4 test data",
    )
    .unwrap();
    assert!(corrupt.is_file());
    let result = VideoFile::open(&corrupt);
    assert!(
        result.is_err(),
        "Corrupt regular files must never become synthetic video"
    );
    let mut valid =
        VideoFile::open(&fixture_path()).expect("A rejected file must not poison MF/COM state");
    let mut decoded = false;
    for _ in 0..30 {
        match valid
            .read_frame()
            .expect("Decode after a rejected corrupt file")
        {
            VideoRead::Frame(frame) => {
                assert_fixture_format(frame.format);
                assert_rgb(&frame, 40, 20, [255, 0, 0]);
                decoded = true;
                break;
            }
            VideoRead::StreamTick { .. } | VideoRead::NoSample { .. } => {}
            VideoRead::EndOfStream => break,
        }
    }
    assert!(
        decoded,
        "Valid decoding must work after the malformed input fails"
    );
}
