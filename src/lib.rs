//! # zenpnm
//!
//! PNM/PAM/PFM image format decoder and encoder, with optional basic BMP support.
//!
//! ## Zero-Copy Decoding
//!
//! For PNM files with maxval=255 (the common case), decoding returns a borrowed
//! slice into the input buffer — no allocation or copy needed. Formats that
//! require transformation (16-bit downscaling, PFM byte reordering) allocate.
//!
//! ## Supported Formats
//!
//! ### PNM family (`pnm` feature, default)
//! - **P5** (PGM binary) — grayscale, 8-bit and 16-bit
//! - **P6** (PPM binary) — RGB, 8-bit and 16-bit
//! - **P7** (PAM) — arbitrary channels (grayscale, RGB, RGBA), 8-bit and 16-bit
//! - **PFM** — floating-point grayscale and RGB (32-bit float per channel)
//!
//! ### Basic BMP (`basic-bmp` feature, opt-in)
//! - Uncompressed 24-bit (RGB) and 32-bit (RGBA) only
//! - **Not auto-detected** — use [`decode_bmp`] and [`encode_bmp`] explicitly
//! - No RLE, no indexed color, no advanced headers
//!
//! ## Usage
//!
//! ```no_run
//! use zenpnm::*;
//! use enough::Unstoppable;
//!
//! # #[cfg(feature = "pnm")]
//! # {
//! // Encode pixels to PPM
//! let pixels = vec![255u8, 0, 0, 0, 255, 0]; // 2 RGB pixels
//! let encoded = encode_ppm(&pixels, 2, 1, PixelLayout::Rgb8, Unstoppable)?;
//!
//! // Decode (auto-detects PNM format, zero-copy when possible)
//! let decoded = decode(&encoded, Unstoppable)?;
//! assert!(decoded.is_borrowed()); // zero-copy for PPM with maxval=255
//! assert_eq!(decoded.pixels(), &pixels[..]);
//! # }
//! # Ok::<(), zenpnm::PnmError>(())
//! ```
//!
//! ## Credits
//!
//! PNM implementation draws from [zune-ppm](https://github.com/etemesi254/zune-image)
//! by Caleb Etemesi (MIT/Apache-2.0/Zlib licensed).

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod decode;
mod error;
mod limits;
mod pixel;

#[cfg(feature = "pnm")]
mod pnm;

#[cfg(feature = "basic-bmp")]
mod bmp;

pub use decode::DecodeOutput;
pub use enough::{Stop, Unstoppable};
pub use error::PnmError;
pub use limits::Limits;
pub use pixel::PixelLayout;

// ── PNM decode (auto-detects P5/P6/P7/PFM from magic bytes) ─────────

/// Decode any PNM format (auto-detected from magic bytes).
///
/// Zero-copy when possible — the returned [`DecodeOutput`] borrows from `data`.
///
/// Does **not** auto-detect BMP. For BMP, use [`decode_bmp`] explicitly.
#[cfg(feature = "pnm")]
pub fn decode(data: &[u8], stop: impl Stop) -> Result<DecodeOutput<'_>, PnmError> {
    pnm::decode(data, None, &stop)
}

/// Decode any PNM format with resource limits.
#[cfg(feature = "pnm")]
pub fn decode_with_limits<'a>(
    data: &'a [u8],
    limits: &'a Limits,
    stop: impl Stop,
) -> Result<DecodeOutput<'a>, PnmError> {
    pnm::decode(data, Some(limits), &stop)
}

// ── PNM encode ───────────────────────────────────────────────────────

/// Encode pixels as PPM (P6, binary RGB).
#[cfg(feature = "pnm")]
pub fn encode_ppm(
    pixels: &[u8],
    width: u32,
    height: u32,
    layout: PixelLayout,
    stop: impl Stop,
) -> Result<alloc::vec::Vec<u8>, PnmError> {
    pnm::encode(pixels, width, height, layout, pnm::PnmFormat::Ppm, &stop)
}

/// Encode pixels as PGM (P5, binary grayscale).
#[cfg(feature = "pnm")]
pub fn encode_pgm(
    pixels: &[u8],
    width: u32,
    height: u32,
    layout: PixelLayout,
    stop: impl Stop,
) -> Result<alloc::vec::Vec<u8>, PnmError> {
    pnm::encode(pixels, width, height, layout, pnm::PnmFormat::Pgm, &stop)
}

/// Encode pixels as PAM (P7, arbitrary channels).
#[cfg(feature = "pnm")]
pub fn encode_pam(
    pixels: &[u8],
    width: u32,
    height: u32,
    layout: PixelLayout,
    stop: impl Stop,
) -> Result<alloc::vec::Vec<u8>, PnmError> {
    pnm::encode(pixels, width, height, layout, pnm::PnmFormat::Pam, &stop)
}

/// Encode pixels as PFM (floating-point).
#[cfg(feature = "pnm")]
pub fn encode_pfm(
    pixels: &[u8],
    width: u32,
    height: u32,
    layout: PixelLayout,
    stop: impl Stop,
) -> Result<alloc::vec::Vec<u8>, PnmError> {
    pnm::encode(pixels, width, height, layout, pnm::PnmFormat::Pfm, &stop)
}

// ── BMP (explicit only, not auto-detected) ───────────────────────────

/// Decode BMP data to pixels (explicit, not auto-detected).
///
/// BMP always allocates (BGR→RGB conversion + row flip).
#[cfg(feature = "basic-bmp")]
pub fn decode_bmp(data: &[u8], stop: impl Stop) -> Result<DecodeOutput<'_>, PnmError> {
    bmp::decode(data, None, &stop)
}

/// Decode BMP with resource limits.
#[cfg(feature = "basic-bmp")]
pub fn decode_bmp_with_limits<'a>(
    data: &'a [u8],
    limits: &'a Limits,
    stop: impl Stop,
) -> Result<DecodeOutput<'a>, PnmError> {
    bmp::decode(data, Some(limits), &stop)
}

/// Encode pixels as 24-bit BMP (RGB, no alpha).
#[cfg(feature = "basic-bmp")]
pub fn encode_bmp(
    pixels: &[u8],
    width: u32,
    height: u32,
    layout: PixelLayout,
    stop: impl Stop,
) -> Result<alloc::vec::Vec<u8>, PnmError> {
    bmp::encode(pixels, width, height, layout, false, &stop)
}

/// Encode pixels as 32-bit BMP (RGBA with alpha).
#[cfg(feature = "basic-bmp")]
pub fn encode_bmp_rgba(
    pixels: &[u8],
    width: u32,
    height: u32,
    layout: PixelLayout,
    stop: impl Stop,
) -> Result<alloc::vec::Vec<u8>, PnmError> {
    bmp::encode(pixels, width, height, layout, true, &stop)
}
