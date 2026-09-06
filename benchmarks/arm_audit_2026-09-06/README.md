# Native ARM codec audit, 2026-09-06

Apple M4 Pro, Rust 1.98 / LLVM 22. Runtime dispatch, no target-cpu=native.
Heavy work serialized under nice -n19 with four build/Rayon/OMP workers.
Production baseline `0b19c3e2`, benchmark snapshot `e1b2df7a`.

The six-family benchmark covers PPM (PNM representative), farbfeld, BMP,
QOI, TGA and Radiance HDR at 64², 256², 1024² and 4096². It compares native
and scalar token states outside timed bodies, checking exact encoded bytes
and decoded pixels first. Input is a deterministic pattern, with RGB8 for
PPM/BMP/QOI/TGA, RGBA16 for farbfeld and RGBf32 for HDR. This is bounded
coverage of these modes, not a content corpus or all supported variants.
HDR is inherently quantized; equality here means equality across tiers.
PPM decode can borrow input; timing does not imply every decoder allocates.

`garb-swizzle.asm` is extracted with `otool -tvV` from
`target/release/deps/codecs-48ea24f1417bef0d`. RGB swaps contain `tbl.16b`
and `st3.16b`; RGBA swaps contain `tbl.16b`. The wrappers already route
BMP/TGA decoding through these kernels.

The older swizzle comparison reused the garb output buffer while allocating
in the push arm. Both BGRA-to-RGB arms now allocate and return their outputs,
with an untimed exact-byte check. Its older ratios must not be interpreted
as isolated SIMD gains.

All 48 baseline comparisons completed with exact tier checks. Full output
is retained without omission in [baseline-small.log](baseline-small.log) and
[baseline-large.log](baseline-large.log). Command: `cargo bench --all-features
--bench codecs`, with the resource settings above. The first candidate production
change replaces RGB8 BMP encoding's per-pixel layout match and push loop
with existing row swizzle calls, retaining bottom-up output, row padding
and cancellation cadence. All-feature tests, the scalar-only BMP regression and strict all-target clippy pass.
The full test run retains eight existing ignored corpus tests and one ignored
doctest; this change does not claim that corpus coverage. Native CI now also
runs all codec features and SIMD.

## RGB8 BMP encoding

| Size | Baseline native | Row-path native | Row-path scalar |
|---|---:|---:|---:|
| 64² | 24.6 us | 1.4 us | 1.4 us |
| 256² | 221.5 us | 14.4 us | 14.3 us |
| 1024² | 4.5 ms | 126.3 us | 126.3 us |
| 4096² | 40.3 ms | 3.6 ms | 3.6 ms |

Baseline and changed binaries were measured separately; these are observed
means, not paired before/after intervals. Every post-change native/scalar
interval crosses zero. The improvement comes from moving layout handling
and output growth out of the per-pixel loop; it benefits both runtime tiers.
The new regression checks exact original BMP header/pixel/padding bytes for
widths 1..65 and heights 1, 3 and 17. No public API or other layout path changed.
[Full post-change log](bmp-after.log).
