use alloc::borrow::Cow;
use alloc::vec::Vec;

use crate::pixel::PixelLayout;

/// Decoded image output. Pixels may be borrowed (zero-copy) or owned.
#[derive(Clone, Debug)]
pub struct DecodeOutput<'a> {
    pixels: Cow<'a, [u8]>,
    pub width: u32,
    pub height: u32,
    pub layout: PixelLayout,
}

impl<'a> DecodeOutput<'a> {
    /// Access the pixel data.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Take ownership of the pixel data (copies if borrowed).
    pub fn into_owned(self) -> DecodeOutput<'static> {
        DecodeOutput {
            pixels: Cow::Owned(self.pixels.into_owned()),
            width: self.width,
            height: self.height,
            layout: self.layout,
        }
    }

    /// Whether the pixel data is borrowed (zero-copy from input).
    pub fn is_borrowed(&self) -> bool {
        matches!(self.pixels, Cow::Borrowed(_))
    }

    pub(crate) fn borrowed(data: &'a [u8], width: u32, height: u32, layout: PixelLayout) -> Self {
        Self {
            pixels: Cow::Borrowed(data),
            width,
            height,
            layout,
        }
    }

    pub(crate) fn owned(data: Vec<u8>, width: u32, height: u32, layout: PixelLayout) -> Self {
        Self {
            pixels: Cow::Owned(data),
            width,
            height,
            layout,
        }
    }
}
