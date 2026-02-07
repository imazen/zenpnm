//! BMP decoder: uncompressed 24-bit and 32-bit BMP.

use super::BmpOutput;
use crate::error::PnmError;
use crate::pixel::PixelLayout;
use alloc::vec::Vec;

/// BMP decoder. Supports uncompressed 24-bit (RGB) and 32-bit (RGBA) BMP.
pub struct BmpDecoder<'a> {
    data: &'a [u8],
}

impl<'a> BmpDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Probe: get dimensions without full decode.
    pub fn info(&self) -> Result<(u32, u32, PixelLayout), PnmError> {
        let header = self.parse_header()?;
        Ok((header.width, header.height, header.layout))
    }

    /// Decode to pixels (top-to-bottom, RGB8 or RGBA8).
    pub fn decode(self) -> Result<BmpOutput, PnmError> {
        let header = self.parse_header()?;

        if header.compression != 0 {
            return Err(PnmError::UnsupportedVariant(alloc::format!(
                "BMP compression type {} not supported",
                header.compression
            )));
        }

        let data_start = header.data_offset as usize;
        if data_start > self.data.len() {
            return Err(PnmError::UnexpectedEof);
        }

        let pixel_data = &self.data[data_start..];
        let w = header.width as usize;
        let h = header.height as usize;
        let bpp = header.bits_per_pixel as usize;

        match bpp {
            24 => self.decode_24bit(pixel_data, w, h, header.top_down),
            32 => self.decode_32bit(pixel_data, w, h, header.top_down),
            _ => Err(PnmError::UnsupportedVariant(alloc::format!(
                "BMP {bpp}-bit not supported (only 24/32)"
            ))),
        }
    }

    fn parse_header(&self) -> Result<BmpHeader, PnmError> {
        if self.data.len() < 54 {
            return Err(PnmError::UnexpectedEof);
        }
        if &self.data[0..2] != b"BM" {
            return Err(PnmError::UnrecognizedFormat);
        }

        let data_offset =
            u32::from_le_bytes([self.data[10], self.data[11], self.data[12], self.data[13]]);

        let width =
            i32::from_le_bytes([self.data[18], self.data[19], self.data[20], self.data[21]]);
        let height_raw =
            i32::from_le_bytes([self.data[22], self.data[23], self.data[24], self.data[25]]);
        let top_down = height_raw < 0;
        let height = height_raw.unsigned_abs();
        let width = width as u32;

        let bits_per_pixel = u16::from_le_bytes([self.data[28], self.data[29]]);
        let compression =
            u32::from_le_bytes([self.data[30], self.data[31], self.data[32], self.data[33]]);

        let layout = match bits_per_pixel {
            24 => PixelLayout::Rgb8,
            32 => PixelLayout::Rgba8,
            _ => PixelLayout::Rgb8, // will error later
        };

        Ok(BmpHeader {
            data_offset,
            width,
            height,
            bits_per_pixel,
            compression,
            layout,
            top_down,
        })
    }

    fn decode_24bit(
        &self,
        pixel_data: &[u8],
        w: usize,
        h: usize,
        top_down: bool,
    ) -> Result<BmpOutput, PnmError> {
        // BMP rows are padded to 4-byte boundaries
        let row_stride = (w * 3 + 3) & !3;
        let needed = row_stride * h;
        if pixel_data.len() < needed {
            return Err(PnmError::UnexpectedEof);
        }

        let mut out = Vec::with_capacity(w * h * 3);
        for row in 0..h {
            // BMP is bottom-up by default
            let src_row = if top_down { row } else { h - 1 - row };
            let row_start = src_row * row_stride;
            for col in 0..w {
                let off = row_start + col * 3;
                // BMP stores BGR
                out.push(pixel_data[off + 2]); // R
                out.push(pixel_data[off + 1]); // G
                out.push(pixel_data[off]); // B
            }
        }

        Ok(BmpOutput {
            pixels: out,
            width: w as u32,
            height: h as u32,
            layout: PixelLayout::Rgb8,
        })
    }

    fn decode_32bit(
        &self,
        pixel_data: &[u8],
        w: usize,
        h: usize,
        top_down: bool,
    ) -> Result<BmpOutput, PnmError> {
        let row_stride = w * 4; // 32-bit rows are always 4-byte aligned
        let needed = row_stride * h;
        if pixel_data.len() < needed {
            return Err(PnmError::UnexpectedEof);
        }

        let mut out = Vec::with_capacity(w * h * 4);
        for row in 0..h {
            let src_row = if top_down { row } else { h - 1 - row };
            let row_start = src_row * row_stride;
            for col in 0..w {
                let off = row_start + col * 4;
                // BMP stores BGRA
                out.push(pixel_data[off + 2]); // R
                out.push(pixel_data[off + 1]); // G
                out.push(pixel_data[off]); // B
                out.push(pixel_data[off + 3]); // A
            }
        }

        Ok(BmpOutput {
            pixels: out,
            width: w as u32,
            height: h as u32,
            layout: PixelLayout::Rgba8,
        })
    }
}

struct BmpHeader {
    data_offset: u32,
    width: u32,
    height: u32,
    bits_per_pixel: u16,
    compression: u32,
    layout: PixelLayout,
    top_down: bool,
}
