//! PNM decoder: P5, P6, P7, PFM (binary formats only).
//!
//! Credits: Draws from zune-ppm by Caleb Etemesi (MIT/Apache-2.0/Zlib).

use super::{PnmFormat, PnmOutput};
use crate::error::PnmError;
use crate::pixel::PixelLayout;
use alloc::string::String;
use alloc::vec::Vec;

/// PNM decoder. Supports P5 (PGM), P6 (PPM), P7 (PAM), and PFM.
pub struct PnmDecoder<'a> {
    data: &'a [u8],
}

/// Parsed PNM header info.
struct PnmHeader {
    format: PnmFormat,
    width: u32,
    height: u32,
    maxval: u32, // 0 for PFM (uses scale factor instead)
    depth: u32,  // channels (from PAM DEPTH or inferred)
    layout: PixelLayout,
    pfm_scale: f32,     // PFM scale factor (sign indicates endianness)
    data_offset: usize, // byte offset where pixel data starts
}

impl<'a> PnmDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Probe: get dimensions and format without decoding pixels.
    pub fn info(&self) -> Result<(u32, u32, PnmFormat, PixelLayout), PnmError> {
        let header = Self::parse_header(self.data)?;
        Ok((header.width, header.height, header.format, header.layout))
    }

    /// Decode to pixels.
    pub fn decode(self) -> Result<PnmOutput, PnmError> {
        let header = Self::parse_header(self.data)?;
        let pixel_data = self
            .data
            .get(header.data_offset..)
            .ok_or(PnmError::UnexpectedEof)?;

        let pixels = match header.format {
            PnmFormat::Pfm => Self::decode_pfm(pixel_data, &header)?,
            _ => Self::decode_integer(pixel_data, &header)?,
        };

        Ok(PnmOutput {
            pixels,
            width: header.width,
            height: header.height,
            layout: header.layout,
            format: header.format,
        })
    }

    fn parse_header(data: &[u8]) -> Result<PnmHeader, PnmError> {
        if data.len() < 3 {
            return Err(PnmError::UnexpectedEof);
        }

        match &data[..2] {
            b"P5" => Self::parse_p5_p6_header(data, PnmFormat::Pgm),
            b"P6" => Self::parse_p5_p6_header(data, PnmFormat::Ppm),
            b"P7" => Self::parse_p7_header(data),
            b"Pf" | b"PF" => Self::parse_pfm_header(data),
            _ => Err(PnmError::UnrecognizedFormat),
        }
    }

    /// Parse P5/P6 header: magic whitespace width whitespace height whitespace maxval whitespace
    fn parse_p5_p6_header(data: &[u8], format: PnmFormat) -> Result<PnmHeader, PnmError> {
        let mut pos = 2; // skip magic

        pos = skip_whitespace_and_comments(data, pos)?;
        let (width, new_pos) = parse_u32(data, pos)?;
        pos = skip_whitespace_and_comments(data, new_pos)?;
        let (height, new_pos) = parse_u32(data, pos)?;
        pos = skip_whitespace_and_comments(data, new_pos)?;
        let (maxval, new_pos) = parse_u32(data, pos)?;

        if maxval == 0 || maxval > 65535 {
            return Err(PnmError::InvalidHeader(alloc::format!(
                "maxval must be 1-65535, got {maxval}"
            )));
        }

        // Exactly one whitespace byte after maxval
        if new_pos >= data.len() {
            return Err(PnmError::UnexpectedEof);
        }
        let data_offset = new_pos + 1;

        let (depth, layout) = match format {
            PnmFormat::Pgm => {
                if maxval <= 255 {
                    (1, PixelLayout::Gray8)
                } else {
                    (1, PixelLayout::Gray16)
                }
            }
            PnmFormat::Ppm => (3, PixelLayout::Rgb8), // 16-bit downscaled to 8-bit on decode
            _ => unreachable!(),
        };

        Ok(PnmHeader {
            format,
            width,
            height,
            maxval,
            depth,
            layout,
            pfm_scale: 0.0,
            data_offset,
        })
    }

    /// Parse P7 (PAM) header with key-value pairs.
    fn parse_p7_header(data: &[u8]) -> Result<PnmHeader, PnmError> {
        let mut pos = 2; // skip "P7"
        pos = skip_whitespace_and_comments(data, pos)?;

        let mut width: Option<u32> = None;
        let mut height: Option<u32> = None;
        let mut depth: Option<u32> = None;
        let mut maxval: Option<u32> = None;
        let mut tupltype: Option<String> = None;

        loop {
            let line_end = data[pos..]
                .iter()
                .position(|&b| b == b'\n')
                .map(|i| pos + i)
                .unwrap_or(data.len());
            let line = core::str::from_utf8(&data[pos..line_end])
                .map_err(|_| PnmError::InvalidHeader("non-UTF8 in PAM header".into()))?
                .trim();

            if line == "ENDHDR" {
                pos = line_end + 1;
                break;
            }

            if let Some(rest) = line.strip_prefix("WIDTH ") {
                width = Some(
                    rest.trim()
                        .parse()
                        .map_err(|_| PnmError::InvalidHeader("bad WIDTH".into()))?,
                );
            } else if let Some(rest) = line.strip_prefix("HEIGHT ") {
                height = Some(
                    rest.trim()
                        .parse()
                        .map_err(|_| PnmError::InvalidHeader("bad HEIGHT".into()))?,
                );
            } else if let Some(rest) = line.strip_prefix("DEPTH ") {
                depth = Some(
                    rest.trim()
                        .parse()
                        .map_err(|_| PnmError::InvalidHeader("bad DEPTH".into()))?,
                );
            } else if let Some(rest) = line.strip_prefix("MAXVAL ") {
                maxval = Some(
                    rest.trim()
                        .parse()
                        .map_err(|_| PnmError::InvalidHeader("bad MAXVAL".into()))?,
                );
            } else if let Some(rest) = line.strip_prefix("TUPLTYPE ") {
                tupltype = Some(rest.trim().into());
            } else if line.starts_with('#') {
                // comment, skip
            }

            pos = if line_end < data.len() {
                line_end + 1
            } else {
                data.len()
            };
            if pos >= data.len() {
                return Err(PnmError::InvalidHeader("no ENDHDR found".into()));
            }
        }

        let width = width.ok_or_else(|| PnmError::InvalidHeader("missing WIDTH".into()))?;
        let height = height.ok_or_else(|| PnmError::InvalidHeader("missing HEIGHT".into()))?;
        let depth = depth.ok_or_else(|| PnmError::InvalidHeader("missing DEPTH".into()))?;
        let maxval = maxval.ok_or_else(|| PnmError::InvalidHeader("missing MAXVAL".into()))?;

        let layout = match (depth, maxval > 255) {
            (1, false) => PixelLayout::Gray8,
            (1, true) => PixelLayout::Gray16,
            (3, false) => PixelLayout::Rgb8,
            (3, true) => PixelLayout::Rgb8, // downscale
            (4, false) => PixelLayout::Rgba8,
            (4, true) => PixelLayout::Rgba8, // downscale
            _ => {
                return Err(PnmError::UnsupportedVariant(alloc::format!(
                    "PAM DEPTH={depth} not supported"
                )));
            }
        };

        let _ = tupltype; // validated by depth

        Ok(PnmHeader {
            format: PnmFormat::Pam,
            width,
            height,
            maxval,
            depth,
            layout,
            pfm_scale: 0.0,
            data_offset: pos,
        })
    }

    /// Parse PFM header: magic\n width height\n scale\n
    fn parse_pfm_header(data: &[u8]) -> Result<PnmHeader, PnmError> {
        let is_color = data[1] == b'F';
        let mut pos = 2;

        pos = skip_whitespace_and_comments(data, pos)?;
        let (width, new_pos) = parse_u32(data, pos)?;
        pos = skip_whitespace_and_comments(data, new_pos)?;
        let (height, new_pos) = parse_u32(data, pos)?;
        pos = skip_whitespace_and_comments(data, new_pos)?;

        // Parse scale factor (float, sign indicates endianness)
        let line_end = data[pos..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|i| pos + i)
            .unwrap_or(data.len());
        let scale_str = core::str::from_utf8(&data[pos..line_end])
            .map_err(|_| PnmError::InvalidHeader("non-UTF8 scale".into()))?
            .trim();
        let scale: f32 = scale_str
            .parse()
            .map_err(|_| PnmError::InvalidHeader(alloc::format!("bad scale: {scale_str}")))?;

        let data_offset = line_end + 1;

        let (depth, layout) = if is_color {
            (3, PixelLayout::RgbF32)
        } else {
            (1, PixelLayout::GrayF32)
        };

        Ok(PnmHeader {
            format: PnmFormat::Pfm,
            width,
            height,
            maxval: 0,
            depth,
            layout,
            pfm_scale: scale,
            data_offset,
        })
    }

    /// Decode integer (8-bit or 16-bit) pixel data for P5/P6/P7.
    fn decode_integer(pixel_data: &[u8], header: &PnmHeader) -> Result<Vec<u8>, PnmError> {
        let w = header.width as usize;
        let h = header.height as usize;
        let depth = header.depth as usize;
        let is_16bit = header.maxval > 255;
        let src_bytes_per_sample = if is_16bit { 2 } else { 1 };
        let expected_src = w
            .checked_mul(h)
            .and_then(|wh| wh.checked_mul(depth))
            .and_then(|whd| whd.checked_mul(src_bytes_per_sample))
            .ok_or(PnmError::DimensionsTooLarge {
                width: header.width,
                height: header.height,
            })?;

        if pixel_data.len() < expected_src {
            return Err(PnmError::UnexpectedEof);
        }

        if !is_16bit && header.maxval == 255 {
            // Direct copy — most common case
            Ok(pixel_data[..expected_src].to_vec())
        } else if !is_16bit {
            // Scale from maxval to 255
            let scale = 255.0 / header.maxval as f32;
            let mut out = Vec::with_capacity(expected_src);
            for &b in &pixel_data[..expected_src] {
                out.push((b as f32 * scale + 0.5) as u8);
            }
            Ok(out)
        } else {
            // 16-bit: for Gray16 keep as-is, for RGB downscale to 8-bit
            match header.layout {
                PixelLayout::Gray16 => Ok(pixel_data[..expected_src].to_vec()),
                _ => {
                    // Downscale 16-bit to 8-bit
                    let num_samples = w * h * depth;
                    let scale = 255.0 / header.maxval as f32;
                    let mut out = Vec::with_capacity(num_samples);
                    for i in 0..num_samples {
                        let hi = pixel_data[i * 2] as u16;
                        let lo = pixel_data[i * 2 + 1] as u16;
                        let val = (hi << 8) | lo; // big-endian
                        out.push((val as f32 * scale + 0.5) as u8);
                    }
                    Ok(out)
                }
            }
        }
    }

    /// Decode PFM float pixel data.
    fn decode_pfm(pixel_data: &[u8], header: &PnmHeader) -> Result<Vec<u8>, PnmError> {
        let w = header.width as usize;
        let h = header.height as usize;
        let depth = header.depth as usize;
        let num_floats = w * h * depth;
        let expected_bytes = num_floats * 4;

        if pixel_data.len() < expected_bytes {
            return Err(PnmError::UnexpectedEof);
        }

        let is_little_endian = header.pfm_scale < 0.0;
        let scale = header.pfm_scale.abs();

        // Output as f32 bytes (native endian)
        let mut out = Vec::with_capacity(expected_bytes);

        // PFM stores rows bottom-to-top
        let row_bytes = w * depth * 4;
        for row in (0..h).rev() {
            let row_start = row * row_bytes;
            for i in 0..(w * depth) {
                let offset = row_start + i * 4;
                let raw = if is_little_endian {
                    f32::from_le_bytes([
                        pixel_data[offset],
                        pixel_data[offset + 1],
                        pixel_data[offset + 2],
                        pixel_data[offset + 3],
                    ])
                } else {
                    f32::from_be_bytes([
                        pixel_data[offset],
                        pixel_data[offset + 1],
                        pixel_data[offset + 2],
                        pixel_data[offset + 3],
                    ])
                };
                let val = raw * scale;
                out.extend_from_slice(&val.to_ne_bytes());
            }
        }

        Ok(out)
    }
}

/// Skip whitespace bytes and # comments. Returns position of next non-whitespace.
fn skip_whitespace_and_comments(data: &[u8], mut pos: usize) -> Result<usize, PnmError> {
    loop {
        if pos >= data.len() {
            return Err(PnmError::UnexpectedEof);
        }
        match data[pos] {
            b' ' | b'\t' | b'\n' | b'\r' => pos += 1,
            b'#' => {
                // Skip to end of line
                while pos < data.len() && data[pos] != b'\n' {
                    pos += 1;
                }
                if pos < data.len() {
                    pos += 1; // skip the \n
                }
            }
            _ => return Ok(pos),
        }
    }
}

/// Parse a u32 from ASCII digits starting at pos. Returns (value, pos_after_digits).
fn parse_u32(data: &[u8], pos: usize) -> Result<(u32, usize), PnmError> {
    let mut end = pos;
    while end < data.len() && data[end].is_ascii_digit() {
        end += 1;
    }
    if end == pos {
        return Err(PnmError::InvalidHeader("expected number".into()));
    }
    let s = core::str::from_utf8(&data[pos..end])
        .map_err(|_| PnmError::InvalidHeader("non-UTF8 number".into()))?;
    let val: u32 = s
        .parse()
        .map_err(|_| PnmError::InvalidHeader(alloc::format!("number too large: {s}")))?;
    Ok((val, end))
}
