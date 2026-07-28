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

zenbench::main!(bench_swizzle);
