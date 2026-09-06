//! Paired native/scalar runtime tiers for all six bitmap codec families.
//! Timings include allocation; setup and exact encoded/pixel checks are untimed.
use enough::Unstoppable;
use zenbench::prelude::*;
use zenbitmaps::{DecodeOutput, PixelLayout, Result};

#[cfg(target_arch = "aarch64")]
type TierToken = archmage::NeonToken;
#[cfg(target_arch = "x86_64")]
type TierToken = archmage::X64V3Token;
fn set_simd(enabled: bool) {
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    TierToken::dangerously_disable_token_process_wide(!enabled)
        .expect("runtime tier must be toggleable");
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    panic!("native tier benchmark requires ARM or x86_64: {enabled}");
}
fn encode(kind: usize, pixels: &[u8], side: u32, layout: PixelLayout) -> Vec<u8> {
    match kind {
        0 => zenbitmaps::encode_ppm(pixels, side, side, layout, Unstoppable),
        1 => zenbitmaps::encode_farbfeld(pixels, side, side, layout, Unstoppable),
        2 => zenbitmaps::encode_bmp(pixels, side, side, layout, Unstoppable),
        3 => zenbitmaps::encode_qoi(pixels, side, side, layout, Unstoppable),
        4 => zenbitmaps::encode_tga(pixels, side, side, layout, Unstoppable),
        5 => zenbitmaps::encode_hdr(pixels, side, side, layout, Unstoppable),
        _ => unreachable!(),
    }
    .unwrap()
}
fn decode(kind: usize, data: &[u8]) -> Result<DecodeOutput<'_>> {
    match kind {
        0 => zenbitmaps::decode(data, Unstoppable),
        1 => zenbitmaps::decode_farbfeld(data, Unstoppable),
        2 => zenbitmaps::decode_bmp(data, Unstoppable),
        3 => zenbitmaps::decode_qoi(data, Unstoppable),
        4 => zenbitmaps::decode_tga(data, Unstoppable),
        5 => zenbitmaps::decode_hdr(data, Unstoppable),
        _ => unreachable!(),
    }
}
zenbench::main!(|suite| {
    const { assert!(cfg!(feature = "simd"), "enable the simd feature") };
    for side in [64u32, 256, 1024, 4096] {
        let count = side as usize * side as usize;
        let rgb: &'static [u8] = Box::leak(
            (0..count)
                .flat_map(|i| {
                    [
                        (i ^ (i / side as usize)) as u8,
                        (i * 3) as u8,
                        (i * 7) as u8,
                    ]
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
        let rgba16: &'static [u8] = Box::leak(
            (0..count)
                .flat_map(|i| {
                    [i as u16, (i * 3) as u16, (i * 7) as u16, u16::MAX]
                        .map(u16::to_ne_bytes)
                        .into_iter()
                        .flatten()
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
        let rgbf: &'static [u8] = Box::leak(
            (0..count)
                .flat_map(|i| {
                    let v = (i % 997) as f32 / 997.0;
                    [v, v * 0.5, v * 0.25]
                        .map(f32::to_ne_bytes)
                        .into_iter()
                        .flatten()
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
        for (kind, name) in ["ppm", "farbfeld", "bmp", "qoi", "tga", "hdr"]
            .into_iter()
            .enumerate()
        {
            let (pixels, layout) = match kind {
                1 => (rgba16, PixelLayout::Rgba16),
                5 => (rgbf, PixelLayout::RgbF32),
                _ => (rgb, PixelLayout::Rgb8),
            };
            set_simd(false);
            let scalar = encode(kind, pixels, side, layout);
            set_simd(true);
            let native = encode(kind, pixels, side, layout);
            assert_eq!(scalar, native, "{name}/{side} encoded bytes");
            let encoded: &'static [u8] = Box::leak(native.into_boxed_slice());
            set_simd(false);
            let scalar = decode(kind, encoded).unwrap();
            set_simd(true);
            let native = decode(kind, encoded).unwrap();
            assert_eq!(
                (scalar.width, scalar.height, scalar.layout),
                (native.width, native.height, native.layout)
            );
            assert_eq!(native.width, side);
            assert_eq!(native.height, side);
            assert_eq!(scalar.pixels(), native.pixels(), "{name}/{side} pixels");
            for decoding in [true, false] {
                suite.compare(
                    format!(
                        "{name}/{}/{side}",
                        if decoding { "decode" } else { "encode" }
                    ),
                    move |g| {
                        g.throughput(Throughput::Elements(count as u64));
                        for (arm, enabled) in [("native", true), ("scalar", false)] {
                            g.bench(arm, move |b| {
                                b.with_input(move || set_simd(enabled)).run(move |_| {
                                    if decoding {
                                        black_box(decode(kind, encoded).unwrap());
                                    } else {
                                        black_box(encode(kind, pixels, side, layout));
                                    }
                                })
                            });
                        }
                    },
                );
            }
        }
    }
    set_simd(true);
});
