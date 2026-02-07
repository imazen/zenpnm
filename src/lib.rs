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
//! - **Not auto-detected** — use [`bmp::decode_bmp`] and [`bmp::encode_bmp`] explicitly
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
//! # use zenpnm::pnm::PnmFormat;
//! // Encode pixels to PPM
//! let pixels = vec![255u8, 0, 0, 0, 255, 0]; // 2 RGB pixels
//! let encoded = encode_ppm(&pixels, 2, 1, PixelLayout::Rgb8, Unstoppable)?;
//!
//! // Decode (auto-detects PNM format, zero-copy when possible)
//! let decoded = decode(&encoded, Unstoppable)?;
//! assert!(decoded.is_borrowed()); // zero-copy for PPM with maxval=255
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

mod error;
mod info;
mod limits;
mod pixel;

#[cfg(feature = "pnm")]
pub mod pnm;

#[cfg(feature = "basic-bmp")]
pub mod bmp;

mod decode;
mod encode;

// Re-exports
pub use decode::{DecodeOutput, DecodeRequest};
pub use encode::EncodeRequest;
pub use enough::{Stop, Unstoppable};
pub use error::PnmError;
pub use info::{BitmapFormat, ImageInfo};
pub use limits::Limits;
pub use pixel::PixelLayout;

// ── Flat one-shot functions (PNM only) ───────────────────────────────

/// Decode any supported PNM format (auto-detected from magic bytes).
///
/// Zero-copy when possible — the returned `DecodeOutput` borrows from `data`.
///
/// This does **not** auto-detect BMP. For BMP, use [`bmp::decode_bmp`] explicitly.
pub fn decode(data: &[u8], stop: impl Stop) -> Result<DecodeOutput<'_>, PnmError> {
    DecodeRequest::new(data).decode(stop)
}

/// Decode PNM with resource limits.
pub fn decode_with_limits<'a>(
    data: &'a [u8],
    limits: &'a Limits,
    stop: impl Stop,
) -> Result<DecodeOutput<'a>, PnmError> {
    DecodeRequest::new(data).with_limits(limits).decode(stop)
}

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
