//! BMP encoder: uncompressed 24-bit and 32-bit BMP.

use crate::error::PnmError;
use crate::pixel::PixelLayout;
use alloc::vec::Vec;

/// BMP encoder. Produces uncompressed BMP files.
pub struct BmpEncoder {
    /// If true, include alpha channel (32-bit BGRA). Otherwise 24-bit BGR.
    include_alpha: bool,
}

impl BmpEncoder {
    /// Create encoder that produces 24-bit RGB BMP.
    pub fn new() -> Self {
        Self {
            include_alpha: false,
        }
    }

    /// Include alpha channel (32-bit BMP).
    pub fn with_alpha(self, alpha: bool) -> Self {
        Self {
            include_alpha: alpha,
        }
    }

    /// Encode pixels to BMP bytes.
    pub fn encode(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
        layout: PixelLayout,
    ) -> Result<Vec<u8>, PnmError> {
        let w = width as usize;
        let h = height as usize;
        let expected = w * h * layout.bytes_per_pixel();
        if pixels.len() < expected {
            return Err(PnmError::BufferTooSmall {
                needed: expected,
                actual: pixels.len(),
            });
        }

        if self.include_alpha {
            self.encode_32bit(pixels, width, height, w, h, layout)
        } else {
            self.encode_24bit(pixels, width, height, w, h, layout)
        }
    }

    fn encode_24bit(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
        w: usize,
        h: usize,
        layout: PixelLayout,
    ) -> Result<Vec<u8>, PnmError> {
        let row_stride = (w * 3 + 3) & !3;
        let pixel_data_size = row_stride * h;
        let file_size = 54 + pixel_data_size;

        let mut out = Vec::with_capacity(file_size);

        // File header (14 bytes)
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&(file_size as u32).to_le_bytes());
        out.extend_from_slice(&[0u8; 4]); // reserved
        out.extend_from_slice(&54u32.to_le_bytes()); // data offset

        // DIB header (BITMAPINFOHEADER, 40 bytes)
        out.extend_from_slice(&40u32.to_le_bytes()); // header size
        out.extend_from_slice(&(width as i32).to_le_bytes());
        out.extend_from_slice(&(height as i32).to_le_bytes()); // positive = bottom-up
        out.extend_from_slice(&1u16.to_le_bytes()); // planes
        out.extend_from_slice(&24u16.to_le_bytes()); // bits per pixel
        out.extend_from_slice(&0u32.to_le_bytes()); // compression (none)
        out.extend_from_slice(&(pixel_data_size as u32).to_le_bytes());
        out.extend_from_slice(&2835u32.to_le_bytes()); // h resolution (72 DPI)
        out.extend_from_slice(&2835u32.to_le_bytes()); // v resolution
        out.extend_from_slice(&0u32.to_le_bytes()); // colors used
        out.extend_from_slice(&0u32.to_le_bytes()); // important colors

        // Pixel data (bottom-up, BGR)
        let pad_bytes = row_stride - w * 3;
        for row in (0..h).rev() {
            for col in 0..w {
                let (r, g, b) = self.get_rgb(pixels, row * w + col, layout)?;
                out.push(b);
                out.push(g);
                out.push(r);
            }
            out.extend(core::iter::repeat_n(0u8, pad_bytes));
        }

        Ok(out)
    }

    fn encode_32bit(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
        w: usize,
        h: usize,
        layout: PixelLayout,
    ) -> Result<Vec<u8>, PnmError> {
        let row_stride = w * 4;
        let pixel_data_size = row_stride * h;
        let file_size = 54 + pixel_data_size;

        let mut out = Vec::with_capacity(file_size);

        // File header
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&(file_size as u32).to_le_bytes());
        out.extend_from_slice(&[0u8; 4]);
        out.extend_from_slice(&54u32.to_le_bytes());

        // DIB header
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&(width as i32).to_le_bytes());
        out.extend_from_slice(&(height as i32).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(pixel_data_size as u32).to_le_bytes());
        out.extend_from_slice(&2835u32.to_le_bytes());
        out.extend_from_slice(&2835u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());

        // Pixel data (bottom-up, BGRA)
        for row in (0..h).rev() {
            for col in 0..w {
                let (r, g, b, a) = self.get_rgba(pixels, row * w + col, layout)?;
                out.push(b);
                out.push(g);
                out.push(r);
                out.push(a);
            }
        }

        Ok(out)
    }

    fn get_rgb(
        &self,
        pixels: &[u8],
        idx: usize,
        layout: PixelLayout,
    ) -> Result<(u8, u8, u8), PnmError> {
        Ok(match layout {
            PixelLayout::Rgb8 => {
                let off = idx * 3;
                (pixels[off], pixels[off + 1], pixels[off + 2])
            }
            PixelLayout::Bgr8 => {
                let off = idx * 3;
                (pixels[off + 2], pixels[off + 1], pixels[off])
            }
            PixelLayout::Rgba8 => {
                let off = idx * 4;
                (pixels[off], pixels[off + 1], pixels[off + 2])
            }
            PixelLayout::Bgra8 => {
                let off = idx * 4;
                (pixels[off + 2], pixels[off + 1], pixels[off])
            }
            PixelLayout::Gray8 => {
                let g = pixels[idx];
                (g, g, g)
            }
            _ => {
                return Err(PnmError::UnsupportedVariant(alloc::format!(
                    "cannot get RGB from {:?}",
                    layout
                )));
            }
        })
    }

    fn get_rgba(
        &self,
        pixels: &[u8],
        idx: usize,
        layout: PixelLayout,
    ) -> Result<(u8, u8, u8, u8), PnmError> {
        Ok(match layout {
            PixelLayout::Rgba8 => {
                let off = idx * 4;
                (
                    pixels[off],
                    pixels[off + 1],
                    pixels[off + 2],
                    pixels[off + 3],
                )
            }
            PixelLayout::Bgra8 => {
                let off = idx * 4;
                (
                    pixels[off + 2],
                    pixels[off + 1],
                    pixels[off],
                    pixels[off + 3],
                )
            }
            PixelLayout::Rgb8 => {
                let off = idx * 3;
                (pixels[off], pixels[off + 1], pixels[off + 2], 255)
            }
            PixelLayout::Bgr8 => {
                let off = idx * 3;
                (pixels[off + 2], pixels[off + 1], pixels[off], 255)
            }
            PixelLayout::Gray8 => {
                let g = pixels[idx];
                (g, g, g, 255)
            }
            _ => {
                return Err(PnmError::UnsupportedVariant(alloc::format!(
                    "cannot get RGBA from {:?}",
                    layout
                )));
            }
        })
    }
}

impl Default for BmpEncoder {
    fn default() -> Self {
        Self::new()
    }
}
