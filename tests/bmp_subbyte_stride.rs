//! Regression for sweep issue #20: 1/2/4-bpp uncompressed BMP rows were read
//! without the 4-byte row alignment the format mandates, scrambling every
//! scanline after the first for any width whose packed row size isn't a
//! multiple of 4. (The standard pal1/pal4 fixtures are 127 px wide — 16- and
//! 64-byte packed rows, accidentally aligned — so they never tripped it.)
#![cfg(feature = "bmp")]

fn le32(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}
fn le16(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}

/// Minimal bottom-up 1-bpp BMP: width 9 (packed row = 2 bytes, stride 4),
/// 3 rows: bottom all-white, middle all-black, top all-white.
fn build_1bpp_9x3() -> Vec<u8> {
    let mut b = Vec::new();
    // BITMAPFILEHEADER
    b.extend_from_slice(b"BM");
    b.extend_from_slice(&le32(62 + 12)); // file size
    b.extend_from_slice(&le32(0));
    b.extend_from_slice(&le32(62)); // pixel data offset
    // BITMAPINFOHEADER
    b.extend_from_slice(&le32(40));
    b.extend_from_slice(&le32(9)); // width
    b.extend_from_slice(&le32(3)); // height (bottom-up)
    b.extend_from_slice(&le16(1)); // planes
    b.extend_from_slice(&le16(1)); // bpp
    b.extend_from_slice(&le32(0)); // BI_RGB
    b.extend_from_slice(&le32(12)); // image size (3 * stride 4)
    b.extend_from_slice(&le32(2835));
    b.extend_from_slice(&le32(2835));
    b.extend_from_slice(&le32(2)); // colors used
    b.extend_from_slice(&le32(0));
    // Palette: 0 = black, 1 = white (BGRX)
    b.extend_from_slice(&[0, 0, 0, 0]);
    b.extend_from_slice(&[255, 255, 255, 0]);
    // Rows, bottom-up, each 2 data bytes + 2 padding bytes.
    // 9 pixels of value 1: 0b1111_1111, 0b1000_0000.
    b.extend_from_slice(&[0xFF, 0x80, 0xAA, 0xAA]); // bottom: white (pad = junk)
    b.extend_from_slice(&[0x00, 0x00, 0x55, 0x55]); // middle: black
    b.extend_from_slice(&[0xFF, 0x80, 0xCC, 0xCC]); // top: white
    b
}

#[test]
fn one_bpp_unaligned_row_stride_decodes_correctly() {
    let bmp = build_1bpp_9x3();
    let img = zenbitmaps::decode_bmp(&bmp, enough::Unstoppable).expect("decode");
    let px = img.pixels();
    let w = 9usize;
    let comps = px.len() / (w * 3);
    assert_eq!(comps * w * 3, px.len(), "unexpected layout: {}", px.len());
    // Output is top-down: row 0 white, row 1 black, row 2 white.
    for (row, expect) in [(0usize, 255u8), (1, 0), (2, 255)] {
        for x in 0..w {
            let i = (row * w + x) * comps;
            assert_eq!(
                px[i], expect,
                "row {row} x {x}: expected {expect} — misaligned sub-byte \
                 row stride scrambles scanlines (issue #20)"
            );
        }
    }
}
