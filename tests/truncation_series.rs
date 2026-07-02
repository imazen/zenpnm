//! EOF/truncation conformance: cutting a known-good encoded file short must
//! categorize as *incomplete client input* — never panic, OOM, or surface as
//! an internal (5xx) error for what is a 4xx-class truncated request.
//!
//! Delegates to the zencodec-testkit [`check_decode_truncation_series`] check,
//! which builds a deterministic prefix series (header sizes + fractions) and
//! runs each through the dyn-erased full decode path, verifying the erased
//! `ErrorCategory` lands in the incomplete-input set (`ErrorCategory::Image`).
//!
//! Covers all six zenbitmaps formats: PNM and farbfeld are always compiled in
//! when the `zencodec` feature is on; BMP/QOI/HDR/TGA are additionally gated
//! on their own feature.

#![cfg(feature = "zencodec")]

use zencodec::encode::{EncodeJob, Encoder, EncoderConfig};
use zenpixels::{PixelDescriptor, PixelSlice};

/// Encode a tiny, known-good RGB8 image through the zencodec trait encode
/// path. RGB8_SRGB is natively supported by every zenbitmaps encoder
/// (PNM/BMP/farbfeld/QOI/HDR/TGA), so one helper covers all six formats.
fn valid_bytes<E>(cfg: E) -> Vec<u8>
where
    E: EncoderConfig,
    <E::Job as EncodeJob>::Enc: Encoder<Error = E::Error>,
{
    let (w, h) = (8u32, 8u32);
    let bytes = vec![0x77u8; (w * h * 3) as usize];
    let slice = PixelSlice::new(&bytes, w, h, (w * 3) as usize, PixelDescriptor::RGB8_SRGB)
        .expect("rgb8 slice");
    cfg.job()
        .encoder()
        .expect("encoder")
        .encode(slice)
        .expect("encode")
        .into_vec()
}

#[test]
fn pnm_truncation_series_categorizes_as_incomplete_input() {
    let valid = valid_bytes(zenbitmaps::PnmEncoderConfig::new());
    zencodec_testkit::check_decode_truncation_series(zenbitmaps::PnmDecoderConfig::new(), &valid)
        .expect(
            "truncated PNM must categorize as incomplete input (Image/UnexpectedEof or \
             Image/Malformed), never panic, OOM, or Internal",
        );
}

#[test]
fn farbfeld_truncation_series_categorizes_as_incomplete_input() {
    let valid = valid_bytes(zenbitmaps::FarbfeldEncoderConfig::new());
    zencodec_testkit::check_decode_truncation_series(
        zenbitmaps::FarbfeldDecoderConfig::new(),
        &valid,
    )
    .expect(
        "truncated farbfeld must categorize as incomplete input, never panic, OOM, or Internal",
    );
}

#[cfg(feature = "bmp")]
#[test]
fn bmp_truncation_series_categorizes_as_incomplete_input() {
    let valid = valid_bytes(zenbitmaps::BmpEncoderConfig::new());
    zencodec_testkit::check_decode_truncation_series(zenbitmaps::BmpDecoderConfig::new(), &valid)
        .expect("truncated BMP must categorize as incomplete input, never panic, OOM, or Internal");
}

#[cfg(feature = "qoi")]
#[test]
fn qoi_truncation_series_categorizes_as_incomplete_input() {
    let valid = valid_bytes(zenbitmaps::QoiEncoderConfig::new());
    zencodec_testkit::check_decode_truncation_series(zenbitmaps::QoiDecoderConfig::new(), &valid)
        .expect("truncated QOI must categorize as incomplete input, never panic, OOM, or Internal");
}

#[cfg(feature = "hdr")]
#[test]
fn hdr_truncation_series_categorizes_as_incomplete_input() {
    let valid = valid_bytes(zenbitmaps::HdrEncoderConfig::new());
    zencodec_testkit::check_decode_truncation_series(zenbitmaps::HdrDecoderConfig::new(), &valid)
        .expect("truncated HDR must categorize as incomplete input, never panic, OOM, or Internal");
}

#[cfg(feature = "tga")]
#[test]
fn tga_truncation_series_categorizes_as_incomplete_input() {
    let valid = valid_bytes(zenbitmaps::TgaEncoderConfig::new());
    zencodec_testkit::check_decode_truncation_series(zenbitmaps::TgaDecoderConfig::new(), &valid)
        .expect("truncated TGA must categorize as incomplete input, never panic, OOM, or Internal");
}
