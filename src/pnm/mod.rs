//! PNM family: P5 (PGM), P6 (PPM), P7 (PAM), PFM.
//!
//! Credits: Implementation draws from [zune-ppm](https://github.com/etemesi254/zune-image)
//! by Caleb Etemesi (MIT/Apache-2.0/Zlib licensed).

mod decode;
mod encode;

pub use decode::PnmDecoder;
pub use encode::PnmEncoder;

use crate::PixelLayout;
use alloc::vec::Vec;

/// Which PNM sub-format to use.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PnmFormat {
    /// P5 — binary grayscale (PGM).
    Pgm,
    /// P6 — binary RGB (PPM).
    Ppm,
    /// P7 — PAM (arbitrary channels, with TUPLTYPE header).
    Pam,
    /// PFM — floating-point (grayscale or RGB, 32-bit float).
    Pfm,
}

/// Decoded PNM output.
#[derive(Clone, Debug)]
pub struct PnmOutput {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub layout: PixelLayout,
    pub format: PnmFormat,
}
