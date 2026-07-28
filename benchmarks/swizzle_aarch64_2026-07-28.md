# R↔B swizzle: routing BMP through garb — 2026-07-28

Platform: Apple Silicon (aarch64, NEON), darwin 25.5.0
Bench: `benches/swizzle.rs` (zenbench, interleaved arms), `--features all`

## Context

QOI and TGA already routed their R↔B swaps through `garb::bytes::*_inplace`, but **BMP decode
still hand-rolled `for px in buf.chunks_exact_mut(N) { px.swap(0, 2) }`** over the whole
decoded buffer and over each row — so the most-used format in this crate was the one left
scalar. garb is the workspace owner for pixel-format swizzles (per the one-owner rule), and
zenbitmaps already carried it as an optional dep behind the `simd` feature, so this needed no
new dependency and no new public API.

## Measured

| case | scalar loop | garb SIMD | speedup |
|---|---|---|---|
| rgb24 640×480 | 180.7 µs | 19.9 µs | **9.08×** |
| rgba32 640×480 | 188.4 µs | 30.5 µs | **6.18×** |
| rgb24 1920×1080 | 1126.1 µs | 128.1 µs | **8.79×** |
| rgba32 1920×1080 | 1045.5 µs | 173.7 µs | **6.02×** |

The SIMD arm reaches 37–45 GB/s against the scalar loop's 4.8–7.4 GB/s. `swap(0, 2)` on a
3- or 4-byte chunk is a byte-granular permute that LLVM does not turn into a vector shuffle;
garb's kernels do it with the ISA's permute directly.

## Correctness

`garb::bytes::*_inplace` returns `Err(SizeError::NotPixelAligned)` and performs **no work**
when the buffer length is not a multiple of the pixel size, whereas `chunks_exact_mut`
swizzles the aligned prefix and ignores the remainder. The existing call sites used
`let _ = garb::bytes::…`, which therefore **silently skipped the swizzle and emitted wrong
colours** on any misaligned buffer.

The new `src/swizzle.rs` helpers fall back to the scalar loop when garb declines, making them
byte-identical to the original loops at every length, aligned or not, with or without the
`simd` feature. `swap_rb_matches_scalar_at_every_length` asserts exactly that over lengths
0..200 (covering every misalignment of both 3 and 4). The QOI and TGA sites were moved onto
the same helpers, which removes that hazard there too.

In practice the buffers at all these sites are `w·h·bpp` and so always aligned — this is a
latent-correctness fix, not an observed-bug fix.

## Note for packaging

`simd` is **not** in `default = []`, so this speedup only lands for builds that enable `simd`
(or `all`). Since `bmp`, `tga` and `qoi` are also non-default, any consumer of those formats
is already selecting features; still, moving `simd` into `default` would change the dependency
graph (pulls garb + archmage) for every consumer, so it is flagged here rather than changed
unilaterally.
