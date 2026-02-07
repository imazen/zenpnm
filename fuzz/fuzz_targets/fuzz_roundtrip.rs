#![no_main]
use libfuzzer_sys::fuzz_target;
use zenpnm::*;

fuzz_target!(|data: &[u8]| {
    // Try PNM decode, then roundtrip
    if let Ok(decoded) = decode(data, enough::Unstoppable) {
        let reencoded = match decoded.format {
            BitmapFormat::Ppm => encode_ppm(
                decoded.pixels(), decoded.width, decoded.height,
                decoded.layout, enough::Unstoppable,
            ),
            BitmapFormat::Pgm => encode_pgm(
                decoded.pixels(), decoded.width, decoded.height,
                decoded.layout, enough::Unstoppable,
            ),
            BitmapFormat::Pam => encode_pam(
                decoded.pixels(), decoded.width, decoded.height,
                decoded.layout, enough::Unstoppable,
            ),
            _ => return, // PFM has float precision concerns, skip
        };

        let Ok(reencoded) = reencoded else { return };
        let Ok(decoded2) = decode(&reencoded, enough::Unstoppable) else {
            panic!("re-encoded PNM data failed to decode");
        };

        assert_eq!(decoded.pixels(), decoded2.pixels(), "PNM roundtrip pixel mismatch");
        assert_eq!(decoded.width, decoded2.width);
        assert_eq!(decoded.height, decoded2.height);
    }

    // Try BMP decode, then roundtrip
    if let Ok(decoded) = bmp::decode_bmp(data, enough::Unstoppable) {
        let reencoded = if decoded.layout == PixelLayout::Rgba8 {
            bmp::encode_bmp_rgba(
                decoded.pixels(), decoded.width, decoded.height,
                decoded.layout, enough::Unstoppable,
            )
        } else {
            bmp::encode_bmp(
                decoded.pixels(), decoded.width, decoded.height,
                decoded.layout, enough::Unstoppable,
            )
        };

        let Ok(reencoded) = reencoded else { return };
        let Ok(decoded2) = bmp::decode_bmp(&reencoded, enough::Unstoppable) else {
            panic!("re-encoded BMP data failed to decode");
        };

        assert_eq!(decoded.pixels(), decoded2.pixels(), "BMP roundtrip pixel mismatch");
        assert_eq!(decoded.width, decoded2.width);
        assert_eq!(decoded.height, decoded2.height);
    }
});
