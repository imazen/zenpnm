//! R↔B channel swaps, routed through garb's SIMD kernels when available.
//!
//! Every call site previously hand-rolled `for px in buf.chunks_exact_mut(N)
//! { px.swap(0, 2) }`. BMP decode still did that on the whole decoded buffer
//! while QOI and TGA had been moved to garb, so the most-used format was the
//! one left scalar.
//!
//! # Why these wrap garb rather than calling it directly
//!
//! `garb::bytes::*_inplace` returns `Err(SizeError::NotPixelAligned)` and
//! performs NO work when the buffer length is not a multiple of the pixel
//! size. The hand-rolled `chunks_exact_mut` loop instead swizzles the aligned
//! prefix and ignores the remainder. So `let _ = garb::bytes::…` — the form
//! the existing QOI/TGA sites used — silently skips the swap on a misaligned
//! buffer and emits wrong colours, where the scalar loop would not.
//!
//! These helpers fall back to the scalar loop when garb declines, which makes
//! them byte-identical to the original loops for every input length, aligned
//! or not, with or without the `simd` feature.

/// Swap R↔B for 3-byte pixels (RGB↔BGR), in place.
#[inline]
pub(crate) fn swap_rb_3(buf: &mut [u8]) {
    #[cfg(feature = "simd")]
    {
        if garb::bytes::rgb_to_bgr_inplace(buf).is_ok() {
            return;
        }
    }
    for px in buf.chunks_exact_mut(3) {
        px.swap(0, 2);
    }
}

/// Swap R↔B for 4-byte pixels (RGBA↔BGRA), in place. Alpha is untouched.
#[inline]
pub(crate) fn swap_rb_4(buf: &mut [u8]) {
    #[cfg(feature = "simd")]
    {
        if garb::bytes::rgba_to_bgra_inplace(buf).is_ok() {
            return;
        }
    }
    for px in buf.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    /// The fallback must agree with the SIMD path at every length, including
    /// lengths that are NOT pixel-aligned — that misalignment is precisely
    /// where `let _ = garb::…` used to silently do nothing.
    #[test]
    fn swap_rb_matches_scalar_at_every_length() {
        for len in 0..200usize {
            let src: Vec<u8> = (0..len).map(|i| (i * 37 % 251) as u8).collect();

            let mut a = src.clone();
            swap_rb_3(&mut a);
            let mut b = src.clone();
            for px in b.chunks_exact_mut(3) {
                px.swap(0, 2);
            }
            assert_eq!(a, b, "swap_rb_3 diverged at len {len}");

            let mut a = src.clone();
            swap_rb_4(&mut a);
            let mut b = src.clone();
            for px in b.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
            assert_eq!(a, b, "swap_rb_4 diverged at len {len}");
        }
    }

    #[test]
    fn swap_rb_is_an_involution() {
        let src: Vec<u8> = (0..96u8).collect();
        let mut v = src.clone();
        swap_rb_3(&mut v);
        assert_ne!(v, src);
        swap_rb_3(&mut v);
        assert_eq!(v, src);

        let mut v = src.clone();
        swap_rb_4(&mut v);
        swap_rb_4(&mut v);
        assert_eq!(v, src);
    }
}
