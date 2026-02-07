//! # zenpnm
//!
//! PNM/PAM/PFM and BMP image format decoder and encoder.
//!
//! ## Supported Formats
//!
//! ### PNM family (`pnm` feature)
//! - **P5** (PGM binary) — grayscale, 8-bit and 16-bit
//! - **P6** (PPM binary) — RGB, 8-bit and 16-bit
//! - **P7** (PAM) — arbitrary channels (grayscale, RGB, RGBA, etc.), 8-bit and 16-bit
//! - **PFM** — floating-point grayscale and RGB (32-bit float per channel)
//!
//! ### BMP (`bmp` feature)
//! - Decode and encode of uncompressed BMP (8-bit RGB/RGBA)
//!
//! ## Non-Goals
//!
//! - ASCII PNM formats (P1, P2, P3) — too slow for any real use
//! - Animated formats
//! - Color management (use zencodecs for that)
//!
//! ## Credits
//!
//! The PNM implementation draws heavily from [zune-ppm](https://github.com/etemesi254/zune-image)
//! by Caleb Etemesi (MIT/Apache-2.0/Zlib licensed). We credit that work and recommend it
//! if you need a PNM decoder integrated with the zune-image ecosystem.
//!
//! ## Usage
//!
//! ```no_run
//! use zenpnm::{PnmDecoder, PnmEncoder, PnmFormat, PixelLayout};
//!
//! // Decode
//! let data: &[u8] = &[]; // your PNM bytes
//! let decoded = PnmDecoder::new(data).decode()?;
//! println!("{}x{} {:?}", decoded.width, decoded.height, decoded.layout);
//!
//! // Encode to P6 (PPM)
//! let encoded = PnmEncoder::new(PnmFormat::Ppm)
//!     .encode(&decoded.pixels, decoded.width, decoded.height, decoded.layout)?;
//! # Ok::<(), zenpnm::PnmError>(())
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod error;
mod pixel;

#[cfg(feature = "pnm")]
mod pnm;

#[cfg(feature = "bmp")]
mod bmp;

// Re-exports
pub use error::PnmError;
pub use pixel::PixelLayout;

#[cfg(feature = "pnm")]
pub use pnm::{PnmDecoder, PnmEncoder, PnmFormat, PnmOutput};

#[cfg(feature = "bmp")]
pub use bmp::{BmpDecoder, BmpEncoder, BmpOutput};
