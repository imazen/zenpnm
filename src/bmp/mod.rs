//! Basic BMP image format decoder and encoder (internal).
//!
//! Use top-level [`crate::decode_bmp`], [`crate::encode_bmp`], etc.

mod decode;
mod encode;

use crate::decode::DecodeOutput;
use crate::error::PnmError;
use crate::limits::Limits;
use crate::pixel::PixelLayout;
use alloc::vec::Vec;
use enough::Stop;

/// Decode BMP data.
pub(crate) fn decode<'a>(
    data: &'a [u8],
    limits: Option<&Limits>,
    stop: &dyn Stop,
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

    let pixels = decode::decode_bmp_pixels(data, width, height, layout, stop)?;
    Ok(DecodeOutput::owned(pixels, width, height, layout))
}

/// Encode to BMP.
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
