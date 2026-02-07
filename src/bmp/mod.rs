//! BMP image format decoder and encoder.
//!
//! Supports uncompressed BMP with 24-bit (RGB) and 32-bit (RGBA) pixel data.
//! RLE and indexed color are not yet supported.

mod decode;
mod encode;

pub use decode::BmpDecoder;
pub use encode::BmpEncoder;

use crate::PixelLayout;
use alloc::vec::Vec;

/// Decoded BMP output.
#[derive(Clone, Debug)]
pub struct BmpOutput {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub layout: PixelLayout,
}
