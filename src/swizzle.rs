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

/// Copy 3-byte pixels swapping R↔B (BGR→RGB or RGB→BGR).
#[inline]
pub(crate) fn bgr_to_rgb_into(src: &[u8], dst: &mut [u8]) {
    #[cfg(feature = "simd")]
    {
        if garb::bytes::rgb_to_bgr(src, dst).is_ok() {
            return;
        }
    }
    for (px, o) in src.chunks_exact(3).zip(dst.chunks_exact_mut(3)) {
        o[0] = px[2];
        o[1] = px[1];
        o[2] = px[0];
    }
}

/// RGBA (4 bpp) → RGB (3 bpp), dropping alpha.
#[inline]
pub(crate) fn rgba_to_rgb_into(src: &[u8], dst: &mut [u8]) {
    #[cfg(feature = "simd")]
    {
        if garb::bytes::rgba_to_rgb(src, dst).is_ok() {
            return;
        }
    }
    for (px, o) in src.chunks_exact(4).zip(dst.chunks_exact_mut(3)) {
        o[0] = px[0];
        o[1] = px[1];
        o[2] = px[2];
    }
}

/// BGRA (4 bpp) → RGB (3 bpp), swapping R↔B and dropping alpha.
#[inline]
pub(crate) fn bgra_to_rgb_into(src: &[u8], dst: &mut [u8]) {
    #[cfg(feature = "simd")]
    {
        if garb::bytes::bgra_to_rgb(src, dst).is_ok() {
            return;
        }
    }
    for (px, o) in src.chunks_exact(4).zip(dst.chunks_exact_mut(3)) {
        o[0] = px[2];
        o[1] = px[1];
        o[2] = px[0];
    }
}

/// Gray (1 bpp) → RGB (3 bpp), replicating luma.
///
/// No garb path: `gray_to_rgb` lives in garb's `experimental_api` module, and
/// a replicate is a shape LLVM widens on its own anyway.
#[inline]
pub(crate) fn gray_to_rgb_into(src: &[u8], dst: &mut [u8]) {
    for (g, o) in src.iter().zip(dst.chunks_exact_mut(3)) {
        o[0] = *g;
        o[1] = *g;
        o[2] = *g;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
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

    /// Every copy-converter must agree with its scalar reference at every
    /// length, including lengths that are not pixel-aligned — the garb path
    /// declines those and must fall through rather than silently do nothing.
    #[test]
    fn copy_converters_match_scalar_at_every_length() {
        for px in 0..60usize {
            let src3: Vec<u8> = (0..px * 3).map(|i| (i * 37 % 251) as u8).collect();
            let src4: Vec<u8> = (0..px * 4).map(|i| (i * 41 % 251) as u8).collect();
            let src1: Vec<u8> = (0..px).map(|i| (i * 53 % 251) as u8).collect();

            let mut a = vec![0u8; px * 3];
            bgr_to_rgb_into(&src3, &mut a);
            let mut b = vec![0u8; px * 3];
            for (p, o) in src3.chunks_exact(3).zip(b.chunks_exact_mut(3)) {
                o[0] = p[2];
                o[1] = p[1];
                o[2] = p[0];
            }
            assert_eq!(a, b, "bgr_to_rgb_into diverged at {px} px");

            let mut a = vec![0u8; px * 3];
            rgba_to_rgb_into(&src4, &mut a);
            let mut b = vec![0u8; px * 3];
            for (p, o) in src4.chunks_exact(4).zip(b.chunks_exact_mut(3)) {
                o[..3].copy_from_slice(&p[..3]);
            }
            assert_eq!(a, b, "rgba_to_rgb_into diverged at {px} px");

            let mut a = vec![0u8; px * 3];
            bgra_to_rgb_into(&src4, &mut a);
            let mut b = vec![0u8; px * 3];
            for (p, o) in src4.chunks_exact(4).zip(b.chunks_exact_mut(3)) {
                o[0] = p[2];
                o[1] = p[1];
                o[2] = p[0];
            }
            assert_eq!(a, b, "bgra_to_rgb_into diverged at {px} px");

            let mut a = vec![0u8; px * 3];
            gray_to_rgb_into(&src1, &mut a);
            let mut b = vec![0u8; px * 3];
            for (g, o) in src1.iter().zip(b.chunks_exact_mut(3)) {
                o[0] = *g;
                o[1] = *g;
                o[2] = *g;
            }
            assert_eq!(a, b, "gray_to_rgb_into diverged at {px} px");
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
