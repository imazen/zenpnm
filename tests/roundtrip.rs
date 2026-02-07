use zenpnm::*;

#[test]
fn ppm_roundtrip_rgb8() {
    let w = 4;
    let h = 3;
    let mut pixels = vec![0u8; w * h * 3];
    // Checkerboard pattern
    for y in 0..h {
        for x in 0..w {
            let off = (y * w + x) * 3;
            if (x + y) % 2 == 0 {
                pixels[off] = 255; // R
                pixels[off + 1] = 0;
                pixels[off + 2] = 128;
            } else {
                pixels[off] = 0;
                pixels[off + 1] = 200;
                pixels[off + 2] = 50;
            }
        }
    }

    let encoded = PnmEncoder::new(PnmFormat::Ppm)
        .encode(&pixels, w as u32, h as u32, PixelLayout::Rgb8)
        .unwrap();

    let decoded = PnmDecoder::new(&encoded).decode().unwrap();
    assert_eq!(decoded.width, w as u32);
    assert_eq!(decoded.height, h as u32);
    assert_eq!(decoded.layout, PixelLayout::Rgb8);
    assert_eq!(decoded.pixels, pixels);
}

#[test]
fn pam_roundtrip_rgba8() {
    let w = 2;
    let h = 2;
    let pixels = vec![
        255, 0, 0, 255, // red
        0, 255, 0, 128, // green semi-transparent
        0, 0, 255, 0, // blue transparent
        128, 128, 128, 255, // gray
    ];

    let encoded = PnmEncoder::new(PnmFormat::Pam)
        .encode(&pixels, w as u32, h as u32, PixelLayout::Rgba8)
        .unwrap();

    let decoded = PnmDecoder::new(&encoded).decode().unwrap();
    assert_eq!(decoded.width, w as u32);
    assert_eq!(decoded.height, h as u32);
    assert_eq!(decoded.layout, PixelLayout::Rgba8);
    assert_eq!(decoded.pixels, pixels);
}

#[test]
fn pgm_roundtrip_gray8() {
    let w = 3;
    let h = 2;
    let pixels = vec![0, 64, 128, 192, 255, 100];

    let encoded = PnmEncoder::new(PnmFormat::Pgm)
        .encode(&pixels, w as u32, h as u32, PixelLayout::Gray8)
        .unwrap();

    let decoded = PnmDecoder::new(&encoded).decode().unwrap();
    assert_eq!(decoded.width, w as u32);
    assert_eq!(decoded.height, h as u32);
    assert_eq!(decoded.layout, PixelLayout::Gray8);
    assert_eq!(decoded.pixels, pixels);
}

#[test]
fn bmp_roundtrip_rgb8() {
    let w = 3;
    let h = 2;
    let pixels = vec![
        255, 0, 0, 0, 255, 0, 0, 0, 255, // row 0: R G B
        128, 128, 128, 64, 64, 64, 0, 0, 0, // row 1: gray dark black
    ];

    let encoded = BmpEncoder::new()
        .encode(&pixels, w as u32, h as u32, PixelLayout::Rgb8)
        .unwrap();

    // Check BMP magic
    assert_eq!(&encoded[0..2], b"BM");

    let decoded = BmpDecoder::new(&encoded).decode().unwrap();
    assert_eq!(decoded.width, w as u32);
    assert_eq!(decoded.height, h as u32);
    assert_eq!(decoded.layout, PixelLayout::Rgb8);
    assert_eq!(decoded.pixels, pixels);
}

#[test]
fn bmp_roundtrip_rgba8() {
    let w = 2;
    let h = 2;
    let pixels = vec![
        255, 0, 0, 255, 0, 255, 0, 128, // row 0
        0, 0, 255, 64, 128, 128, 128, 255, // row 1
    ];

    let encoded = BmpEncoder::new()
        .with_alpha(true)
        .encode(&pixels, w as u32, h as u32, PixelLayout::Rgba8)
        .unwrap();

    let decoded = BmpDecoder::new(&encoded).decode().unwrap();
    assert_eq!(decoded.width, w as u32);
    assert_eq!(decoded.height, h as u32);
    assert_eq!(decoded.layout, PixelLayout::Rgba8);
    assert_eq!(decoded.pixels, pixels);
}
