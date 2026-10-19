#![deny(unsafe_code)]
//! OS device integration behind bounded safe APIs.
pub mod audio;
/// Audited Windows monitor inventory, with explicit Unsupported elsewhere.
#[cfg_attr(windows, allow(unsafe_code))]
pub mod displays;
/// Audited COM and media-buffer boundary. All other platform code remains safe.
#[cfg(windows)]
#[allow(unsafe_code)]
pub mod media_foundation;
