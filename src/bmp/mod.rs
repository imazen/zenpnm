//! Basic BMP image format decoder and encoder.
//!
//! Supports uncompressed BMP with 24-bit (RGB) and 32-bit (RGBA) pixel data.
//! RLE, indexed color, and advanced header versions are not supported.
//!
//! **This module is not auto-detected.** Use [`decode_bmp`] or [`encode_bmp`]
//! explicitly. The generic [`crate::decode`] function does not handle BMP.

mod decode;
mod encode;

pub use decode::BmpDecoder;
pub use encode::BmpEncoder;

use crate::decode::DecodeOutput;
use crate::error::PnmError;
use crate::info::{BitmapFormat, ImageInfo};
use crate::limits::Limits;
use crate::pixel::PixelLayout;
use enough::Stop;

/// Probe BMP header for dimensions and layout without decoding pixels.
pub fn probe(data: &[u8]) -> Result<ImageInfo, PnmError> {
    let (width, height, layout) = decode::parse_bmp_header(data)?;
    Ok(ImageInfo {
        width,
        height,
        format: BitmapFormat::Bmp,
        native_layout: layout,
    })
}

/// Decode BMP data to pixels.
///
/// BMP always allocates (BGR→RGB conversion + row flip required).
pub fn decode_bmp<'a>(data: &'a [u8], stop: impl Stop) -> Result<DecodeOutput<'a>, PnmError> {
    decode_bmp_with_limits(data, None, stop)
}

/// Decode BMP data with resource limits.
pub fn decode_bmp_with_limits<'a>(
    data: &'a [u8],
    limits: Option<&Limits>,
    stop: impl Stop,
) -> Result<DecodeOutput<'a>, PnmError> {
    let (width, height, layout) = decode::parse_bmp_header(data)?;

    if let Some(limits) = limits {
        limits.check(width, height)?;
    }

    stop.check()?;

    let out_bytes = width as usize * height as usize * layout.bytes_per_pixel();
    if let Some(limits) = limits {
        limits.check_memory(out_bytes)?;
    }

    let pixels = decode::decode_bmp_pixels(data, width, height, layout, &stop)?;
    Ok(DecodeOutput::owned(
        pixels,
        width,
        height,
        layout,
        BitmapFormat::Bmp,
    ))
}

/// Encode pixels as 24-bit BMP (RGB, no alpha).
pub fn encode_bmp(
    pixels: &[u8],
    width: u32,
    height: u32,
    layout: PixelLayout,
    stop: impl Stop,
) -> Result<Vec<u8>, PnmError> {
    encode::encode_bmp(pixels, width, height, layout, false, &stop)
}

/// Encode pixels as 32-bit BMP (RGBA with alpha).
pub fn encode_bmp_rgba(
    pixels: &[u8],
    width: u32,
    height: u32,
    layout: PixelLayout,
    stop: impl Stop,
) -> Result<Vec<u8>, PnmError> {
    encode::encode_bmp(pixels, width, height, layout, true, &stop)
}

// Keep internal aliases for EncodeRequest
pub(crate) fn encode(
    pixels: &[u8],
    width: u32,
    height: u32,
    layout: PixelLayout,
    alpha: bool,
    stop: &dyn Stop,
) -> Result<Vec<u8>, PnmError> {
    encode::encode_bmp(pixels, width, height, layout, alpha, stop)
}
