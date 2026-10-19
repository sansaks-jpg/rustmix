#![cfg(windows)]
#![allow(clippy::unwrap_used)]

use rustmix_platform::media_foundation::{VideoFile, copy_rgb32_rows};

#[test]
fn rgb32_conversion_preserves_row_order_channels_and_ignores_alpha_and_padding() {
    let top_down = [
        30, 20, 10, 0, 60, 50, 40, 17, 99, 99, 99, 99, 90, 80, 70, 1, 120, 110, 100, 255, 88, 88,
        88, 88,
    ];
    let expected = vec![
        [10, 20, 30, 255],
        [40, 50, 60, 255],
        [70, 80, 90, 255],
        [100, 110, 120, 255],
    ];
    assert_eq!(copy_rgb32_rows(&top_down, 2, 2, 12, 0).unwrap(), expected);

    let bottom_up = [
        90, 80, 70, 1, 120, 110, 100, 255, 88, 88, 88, 88, 30, 20, 10, 0, 60, 50, 40, 17, 99, 99,
        99, 99,
    ];
    assert_eq!(
        copy_rgb32_rows(&bottom_up, 2, 2, -12, 12).unwrap(),
        expected
    );
}

#[test]
fn rgb32_conversion_rejects_bad_dimensions_strides_offsets_and_truncated_storage() {
    let bytes = [0_u8; 32];
    for (width, height, stride, offset) in [
        (0, 1, 4, 0),
        (1, 0, 4, 0),
        (4097, 1, 16388, 0),
        (1, 2161, 4, 0),
        (2, 2, 0, 0),
        (2, 2, 7, 0),
        (2, 2, -7, 7),
        (1, 1, i32::MIN, 0),
        (1, 1, 4, usize::MAX),
        (2, 2, -8, 0),
        (2, 2, 8, 24),
    ] {
        assert!(copy_rgb32_rows(&bytes, width, height, stride, offset).is_err());
    }
    assert!(copy_rgb32_rows(&[], 1, 1, 4, 0).is_err());
    assert!(copy_rgb32_rows(&[0; 15], 2, 2, 8, 0).is_err());
}

#[test]
fn rgb32_conversion_honours_a_nonzero_scanline_origin() {
    let bytes = [99, 99, 99, 99, 3, 2, 1, 0, 6, 5, 4, 1];
    assert_eq!(
        copy_rgb32_rows(&bytes, 2, 1, 8, 4).unwrap(),
        vec![[1, 2, 3, 255], [4, 5, 6, 255]]
    );
}

#[test]
fn decoder_rejects_network_or_non_mp4_descriptors_before_opening_a_reader() {
    for path in [
        "https://example.invalid/video.mp4",
        "file:///C:/video.mp4",
        "//server/share/video.mp4",
        r"\\server\share\video.mp4",
        "clip.mkv",
        "clip.png",
        "clip\0.mp4",
    ] {
        assert!(VideoFile::open(std::path::Path::new(path)).is_err());
    }
}
