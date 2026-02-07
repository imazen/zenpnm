/// Pixel memory layout.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelLayout {
    /// Single channel, 8-bit grayscale.
    Gray8,
    /// Single channel, 16-bit grayscale (native endian).
    Gray16,
    /// 3 channels, 8-bit RGB.
    Rgb8,
    /// 4 channels, 8-bit RGBA.
    Rgba8,
    /// 3 channels, 8-bit BGR.
    Bgr8,
    /// 4 channels, 8-bit BGRA.
    Bgra8,
    /// Single channel, 32-bit float grayscale.
    GrayF32,
    /// 3 channels, 32-bit float RGB.
    RgbF32,
}

impl PixelLayout {
    /// Bytes per pixel for this layout.
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            Self::Gray8 => 1,
            Self::Gray16 => 2,
            Self::Rgb8 | Self::Bgr8 => 3,
            Self::Rgba8 | Self::Bgra8 => 4,
            Self::GrayF32 => 4,
            Self::RgbF32 => 12,
        }
    }

    /// Number of channels.
    pub fn channels(&self) -> usize {
        match self {
            Self::Gray8 | Self::Gray16 | Self::GrayF32 => 1,
            Self::Rgb8 | Self::Bgr8 | Self::RgbF32 => 3,
            Self::Rgba8 | Self::Bgra8 => 4,
        }
    }
}
