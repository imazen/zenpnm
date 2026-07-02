use alloc::string::String;
use enough::StopReason;
use whereat::At;

/// Result alias with `At<BitmapError>` for automatic file:line location tracking.
///
/// Every public decode/encode entry point returns this. The error carries a
/// captured call-site trace (file, line, and — with [`whereat::define_at_crate_info!`]
/// in scope — a GitHub source link), which is invaluable for diagnosing
/// malformed-input failures in server logs. Match on the underlying enum via
/// [`At::error`]:
///
/// ```
/// use zenbitmaps::{decode, BitmapError};
/// use enough::Unstoppable;
///
/// match decode(b"not an image", Unstoppable) {
///     Ok(_) => {}
///     Err(e) => {
///         // `e` is `At<BitmapError>`; the inner enum is `e.error()`.
///         assert!(matches!(e.error(), BitmapError::UnrecognizedFormat));
///     }
/// }
/// ```
pub type Result<T> = core::result::Result<T, At<BitmapError>>;

/// Which kind of "unsupported" a [`BitmapError::UnsupportedVariant`] describes.
///
/// Mirrors the origin-first split in zencodec's `UnsupportedImageKind` — when
/// the `zencodec` feature is enabled, [`category()`](zencodec::CategorizedError::category)
/// maps [`Type`](Self::Type) to `ImageError::Unsupported(UnsupportedImageKind::Type)`
/// and [`Feature`](Self::Feature) to `..::Feature` — but this enum has no
/// dependency on `zencodec` itself, so it stays available in the bare
/// (no-`zencodec`-feature) build too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnsupportedKind {
    /// The format, container, sub-format, or profile itself is not
    /// implemented at all — an unrecognized enum-level value (a BMP
    /// compression scheme, a TGA image type, a PNM sub-format like PBM, a
    /// whole format disabled at compile time) rather than a narrower
    /// configuration within an otherwise-handled variant.
    Type,
    /// The format (and the specific sub-variant/profile) IS handled, but a
    /// narrower configuration within it — a bit depth, pixel layout/channel
    /// combination, or header field value — is not implemented.
    Feature,
}

/// Errors from PNM/BMP decoding and encoding.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BitmapError {
    #[error("unrecognized format magic bytes")]
    UnrecognizedFormat,

    #[error("invalid header: {0}")]
    InvalidHeader(String),

    #[error("unsupported format variant: {1}")]
    UnsupportedVariant(UnsupportedKind, String),

    #[error("invalid pixel data: {0}")]
    InvalidData(String),

    #[error("dimensions too large: {width}x{height}")]
    DimensionsTooLarge { width: u32, height: u32 },

    /// A configured [`crate::Limits`] / `zencodec::ResourceLimits` cap
    /// (width, height, pixel count, memory, or input size) was exceeded.
    /// Distinct from [`OutOfMemory`](Self::OutOfMemory): this fires because
    /// the caller (or the default) *chose* a cap, not because allocation
    /// itself failed.
    #[error("limit exceeded: {0}")]
    LimitExceeded(String),

    /// Real allocation failure: a fallible `try_reserve` returned an error,
    /// or a size computation overflowed the platform's address space (so the
    /// buffer could never be allocated regardless of any configured cap).
    /// Distinct from [`LimitExceeded`](Self::LimitExceeded).
    #[error("out of memory: {0}")]
    OutOfMemory(String),

    /// The declared/expanding output size is wildly disproportionate to the
    /// available input bytes (an RLE/bitfield amplification bomb guard) — a
    /// security-relevant anti-DoS bound distinct from an absolute
    /// [`LimitExceeded`](Self::LimitExceeded) byte cap.
    #[error("decompression bomb detected: {0}")]
    DecompressionBomb(String),

    #[error("unexpected end of input")]
    UnexpectedEof,

    #[error("pixel layout mismatch: expected {expected:?}, got {actual:?}")]
    LayoutMismatch {
        expected: crate::PixelLayout,
        actual: crate::PixelLayout,
    },

    #[error("buffer too small: need {needed} bytes, got {actual}")]
    BufferTooSmall { needed: usize, actual: usize },

    #[error("operation cancelled")]
    Cancelled(StopReason),

    /// Unsupported codec operation.
    #[cfg(feature = "zencodec")]
    #[error(transparent)]
    UnsupportedOperation(#[from] zencodec::UnsupportedOperation),
}

impl From<StopReason> for BitmapError {
    fn from(r: StopReason) -> Self {
        BitmapError::Cancelled(r)
    }
}

// ============================================================================
// zencodec CategorizedError taxonomy (unpublished ErrorCategory reshape,
// zencodec PR #116 / commit 2427387f86c77fdf773ae2fa219926a49cd32d99). Maps
// every `BitmapError` variant to exactly one coarse `ErrorCategory` so
// consumers can route (HTTP status, retry policy, logging) without matching
// this enum directly.
// ============================================================================

/// Best-effort recovery of which [`zencodec::LimitKind`] a
/// [`BitmapError::LimitExceeded`] message refers to, from the message's fixed
/// prefix.
///
/// `BitmapError::LimitExceeded` carries only a `String` (kept as-is —
/// reshaping it to carry a structured kind directly would break the existing
/// public `BitmapError::LimitExceeded(String)` pattern-match shape, unlike
/// [`UnsupportedVariant`](BitmapError::UnsupportedVariant) which this same
/// migration *did* reshape on explicit instruction). This recovers the axis
/// from the small, closed set of message templates this crate itself
/// generates — [`crate::limits`], `bmp::decode::parse_bmp_header`'s
/// declared-pixels check, and each codec adapter's `max_input_bytes` check;
/// there is no other producer of this variant. The
/// `category_limit_exceeded_matches_every_real_site` test below locks every
/// real template against this function, so a future wording change fails a
/// test rather than silently miscategorizing.
#[cfg(feature = "zencodec")]
fn limit_kind_from_message(msg: &str) -> zencodec::LimitKind {
    use zencodec::LimitKind;
    if msg.starts_with("width ") {
        LimitKind::Width
    } else if msg.starts_with("height ") {
        LimitKind::Height
    } else if msg.starts_with("pixel count ") {
        LimitKind::Pixels
    } else if msg.starts_with("input size ") {
        LimitKind::InputSize
    } else {
        // "output size ... exceeds memory limit ..." (limits::check_output_size)
        // and any future template default to the byte/memory axis.
        LimitKind::Memory
    }
}

#[cfg(feature = "zencodec")]
impl zencodec::CategorizedError for BitmapError {
    fn codec_name(&self) -> Option<&'static str> {
        Some("zenbitmaps")
    }

    fn category(&self) -> zencodec::ErrorCategory {
        use zencodec::{
            ImageError, InvalidKind, RequestError, ResourceError, UnsupportedImageKind,
        };
        match self {
            // Corrupt/invalid bitstream content — the bytes themselves are wrong.
            BitmapError::InvalidHeader(_) | BitmapError::InvalidData(_) => {
                ImageError::Malformed.into()
            }

            // Doesn't match any format this crate recognizes at all.
            BitmapError::UnrecognizedFormat => {
                ImageError::Unsupported(UnsupportedImageKind::Type).into()
            }

            // A recognized format/sub-format whose specific profile/feature
            // isn't implemented — Type vs Feature per the tagged kind.
            BitmapError::UnsupportedVariant(UnsupportedKind::Type, _) => {
                ImageError::Unsupported(UnsupportedImageKind::Type).into()
            }
            BitmapError::UnsupportedVariant(UnsupportedKind::Feature, _) => {
                ImageError::Unsupported(UnsupportedImageKind::Feature).into()
            }

            // Truncated input.
            BitmapError::UnexpectedEof => ImageError::UnexpectedEof.into(),

            // A `checked_mul` overflow while computing row/output byte counts
            // from header-declared dimensions: the buffer can never be
            // allocated on this platform regardless of any configured cap —
            // structurally the same as a failed allocation (see
            // `zencodec::ResourceError::OutOfMemory`'s own doc: "a size
            // computation overflowed the platform's address space").
            BitmapError::DimensionsTooLarge { .. } => ResourceError::OutOfMemory.into(),
            // A fallible `try_reserve` genuinely failed (allocator exhaustion).
            BitmapError::OutOfMemory(_) => ResourceError::OutOfMemory.into(),

            // Declared/expanding output is disproportionate to the input size
            // (RLE/bitfield amplification bomb guard).
            BitmapError::DecompressionBomb(_) => {
                ResourceError::Limits(zencodec::LimitKind::DecompressionRatio).into()
            }

            // A configured `ResourceLimits` cap was exceeded.
            BitmapError::LimitExceeded(msg) => {
                ResourceError::Limits(limit_kind_from_message(msg)).into()
            }

            // Caller-supplied pixel buffer has invalid geometry (size/stride).
            BitmapError::LayoutMismatch { .. } | BitmapError::BufferTooSmall { .. } => {
                RequestError::Invalid(InvalidKind::Buffer).into()
            }

            BitmapError::Cancelled(reason) => zencodec::ErrorCategory::Lifecycle(*reason),

            #[cfg(feature = "zencodec")]
            BitmapError::UnsupportedOperation(op) => op.category(),
        }
    }
}

/// Bridge a bare [`BitmapError`] into the shared
/// [`CodecError`](zencodec::CodecError) envelope (Pattern B).
///
/// Mirrors zenpng/zenwebp/zengif/zenjxl/zenavif's identical bridge:
/// `.start_at()` begins the location trace; [`CodecError::of`](zencodec::CodecError::of)
/// then reads the [`category`](zencodec::CategorizedError::category) *and* the
/// [`codec_name`](zencodec::CategorizedError::codec_name) from the value,
/// keeping the trace on the outside. With this, `?`/`.into()` on a bare
/// `BitmapError` auto-wraps into the envelope the zencodec trait impls return.
///
/// Already-located `At<BitmapError>` values convert via
/// `.map_err(CodecError::of)` instead — the orphan rule forbids a
/// `From<At<BitmapError>>` impl here (`At` is not a fundamental type, so
/// `At<BitmapError>` is not a local type).
#[cfg(feature = "zencodec")]
impl From<BitmapError> for At<zencodec::CodecError> {
    #[track_caller]
    fn from(e: BitmapError) -> Self {
        use whereat::ErrorAtExt;
        zencodec::CodecError::of(e.start_at())
    }
}

#[cfg(all(test, feature = "zencodec"))]
mod tests {
    use super::*;
    use whereat::at;
    use zencodec::{
        CategorizedError, ErrorCategory as C, ImageError, InvalidKind, LimitKind as L,
        RequestError, ResourceError, UnsupportedImageKind as UIK,
    };

    #[test]
    fn codec_name_is_zenbitmaps() {
        assert_eq!(
            BitmapError::UnrecognizedFormat.codec_name(),
            Some("zenbitmaps")
        );
    }

    #[test]
    fn category_maps_each_variant() {
        assert_eq!(
            BitmapError::UnrecognizedFormat.category(),
            C::Image(ImageError::Unsupported(UIK::Type))
        );
        assert_eq!(
            BitmapError::InvalidHeader("x".into()).category(),
            C::Image(ImageError::Malformed)
        );
        assert_eq!(
            BitmapError::InvalidData("x".into()).category(),
            C::Image(ImageError::Malformed)
        );
        assert_eq!(
            BitmapError::UnsupportedVariant(UnsupportedKind::Type, "x".into()).category(),
            C::Image(ImageError::Unsupported(UIK::Type))
        );
        assert_eq!(
            BitmapError::UnsupportedVariant(UnsupportedKind::Feature, "x".into()).category(),
            C::Image(ImageError::Unsupported(UIK::Feature))
        );
        assert_eq!(
            BitmapError::UnexpectedEof.category(),
            C::Image(ImageError::UnexpectedEof)
        );
        assert_eq!(
            BitmapError::DimensionsTooLarge {
                width: 1,
                height: 1
            }
            .category(),
            C::Resource(ResourceError::OutOfMemory)
        );
        assert_eq!(
            BitmapError::OutOfMemory("x".into()).category(),
            C::Resource(ResourceError::OutOfMemory)
        );
        assert_eq!(
            BitmapError::DecompressionBomb("x".into()).category(),
            C::Resource(ResourceError::Limits(L::DecompressionRatio))
        );
        assert_eq!(
            BitmapError::LayoutMismatch {
                expected: crate::PixelLayout::Rgb8,
                actual: crate::PixelLayout::Gray8,
            }
            .category(),
            C::Request(RequestError::Invalid(InvalidKind::Buffer))
        );
        assert_eq!(
            BitmapError::BufferTooSmall {
                needed: 10,
                actual: 5
            }
            .category(),
            C::Request(RequestError::Invalid(InvalidKind::Buffer))
        );
        assert_eq!(
            BitmapError::Cancelled(enough::StopReason::Cancelled).category(),
            C::Lifecycle(enough::StopReason::Cancelled)
        );
        assert_eq!(
            BitmapError::Cancelled(enough::StopReason::TimedOut).category(),
            C::Lifecycle(enough::StopReason::TimedOut)
        );
        assert_eq!(
            BitmapError::from(zencodec::UnsupportedOperation::PixelFormat).category(),
            C::Request(RequestError::Unsupported(
                zencodec::UnsupportedOperation::PixelFormat
            ))
        );
    }

    // Every real `LimitExceeded` message template this crate constructs
    // (limits.rs, bmp/decode.rs, and the codec adapters' max_input_bytes
    // checks) must classify to the LimitKind that template actually means.
    // Locks `limit_kind_from_message` against wording drift.
    #[test]
    fn category_limit_exceeded_matches_every_real_site() {
        let cases: &[(&str, L)] = &[
            // limits::check_dimensions
            ("width 5000 exceeds limit 4096", L::Width),
            ("height 5000 exceeds limit 4096", L::Height),
            ("pixel count 1001000 exceeds limit 1000000", L::Pixels),
            // limits::check_output_size
            ("output size 100 bytes exceeds memory limit 50", L::Memory),
            // codec/*.rs adapters' max_input_bytes checks
            ("input size 100 exceeds limit 50", L::InputSize),
            // bmp/decode.rs declared-pixels-vs-max_pixels check
            ("pixel count 225000000 exceeds limit 120000000", L::Pixels),
        ];
        for (msg, want) in cases {
            assert_eq!(
                BitmapError::LimitExceeded((*msg).into()).category(),
                C::Resource(ResourceError::Limits(*want)),
                "message {msg:?} should classify as {want:?}"
            );
        }
    }

    #[test]
    fn error_with_whereat() {
        fn inner() -> Result<()> {
            Err(at!(BitmapError::InvalidData("test".into())))
        }
        fn outer() -> Result<()> {
            inner().map_err(|e| e.at())?;
            Ok(())
        }
        let err = outer().unwrap_err();
        assert!(err.frame_count() >= 1);
    }

    #[test]
    fn category_through_at() {
        let err: At<BitmapError> = at!(BitmapError::UnexpectedEof);
        assert_eq!(err.category(), C::Image(ImageError::UnexpectedEof));
        assert_eq!(err.codec_name(), Some("zenbitmaps"));
    }

    // The bridge `From<BitmapError> for At<CodecError>` preserves both the
    // category and the codec name through the envelope.
    #[test]
    fn bridge_into_codec_error_preserves_category_and_codec() {
        let e: At<zencodec::CodecError> = BitmapError::UnexpectedEof.into();
        assert_eq!(e.category(), C::Image(ImageError::UnexpectedEof));
        assert_eq!(e.error().codec(), Some("zenbitmaps"));
    }
}
