//! R↔B swizzle: garb's SIMD kernel vs the hand-rolled scalar loop.
//!
//! BMP decode swizzled the whole decoded buffer with
//! `for px in buf.chunks_exact_mut(N) { px.swap(0, 2) }` while QOI and TGA had
//! already been routed through garb — so the most-used format here was the one
//! left scalar. This measures what that routing is worth.
//!
//! Both arms are called directly (not through `crate::swizzle`, which resolves
//! at compile time via `cfg(feature = "simd")` and so could only measure one
//! side per binary).
//!
//! Run: `cargo bench --bench swizzle --features all`

use zenbench::prelude::*;

fn buf(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i * 37 % 251) as u8).collect()
}

/// The PNM encode arms as they were: per-pixel `Vec::push` into a growing
/// buffer. Kept so the comparison is against what actually shipped.
fn push_bgra_to_rgb(pixels: &[u8], n: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(n * 3);
    for i in 0..n {
        let off = i * 4;
        out.push(pixels[off + 2]);
        out.push(pixels[off + 1]);
        out.push(pixels[off]);
    }
    out
}

fn bench_swizzle(suite: &mut Suite) {
    // 1920x1080 is the realistic whole-buffer decode case; 640x480 shows a
    // smaller image where per-call overhead is a larger share.
    for &(label, px) in &[("640x480", 640usize * 480), ("1920x1080", 1920 * 1080)] {
        for &(bpp, name) in &[(3usize, "rgb24"), (4, "rgba32")] {
            let n = px * bpp;
            suite.compare(format!("swap_rb/{name}/{label}"), |g| {
                g.throughput(Throughput::Bytes(n as u64));
                g.bench("garb_simd", move |b| {
                    let mut v = buf(n);
                    b.iter(move || {
                        if bpp == 3 {
                            garb::bytes::rgb_to_bgr_inplace(&mut v).unwrap();
                        } else {
                            garb::bytes::rgba_to_bgra_inplace(&mut v).unwrap();
                        }
                    })
                });
                g.bench("scalar_loop", move |b| {
                    let mut v = buf(n);
                    b.iter(move || {
                        for px in v.chunks_exact_mut(bpp) {
                            px.swap(0, 2);
                        }
                    })
                });
            });
        }
    }
}

/// PNM/QOI encode paths: both arms allocate and return their output buffer.
fn bench_encode_paths(suite: &mut Suite) {
    for &(label, px) in &[("640x480", 640usize * 480), ("1920x1080", 1920 * 1080)] {
        let src = buf(px * 4);
        let src: &'static [u8] = Box::leak(src.into_boxed_slice());
        let expected = push_bgra_to_rgb(src, px);
        let mut actual = vec![0; px * 3];
        garb::bytes::bgra_to_rgb(src, &mut actual).unwrap();
        assert_eq!(expected, actual);
        suite.compare(format!("bgra_to_rgb/{label}"), |g| {
            g.throughput(Throughput::Bytes((px * 3) as u64));
            g.bench("push_was", move |b| {
                b.iter(move || push_bgra_to_rgb(src, px))
            });
            g.bench("garb_now", move |b| {
                b.iter(move || {
                    let mut out = vec![0u8; px * 3];
                    garb::bytes::bgra_to_rgb(src, &mut out).unwrap();
                    out
                })
            });
        });
    }
}

zenbench::main!(bench_swizzle, bench_encode_paths);
