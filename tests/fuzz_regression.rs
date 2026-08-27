//! Replay seed inputs from `fuzz/regression/` through every fuzz target
//! entry point. Shared scaffolding lives in `zen-fuzz-regress`.

use std::path::Path;
use zenutils_fuzz::RegressionSuite;

/// Lower bound on the replayable seed corpus committed under `fuzz/regression/`.
///
/// `RegressionSuite` treats a missing or empty seed directory as a clean no-op,
/// so an emptied, renamed, or never-checked-out corpus would let this test pass
/// without replaying a single seed. Pinning the floor makes that a loud failure.
/// Raise this when seeds are added; only lower it when deleting seeds on purpose.
const MIN_SEEDS: usize = 5;

/// Count the files `RegressionSuite::run` will actually replay, using its own
/// filters: recurse into subdirectories, skip dotfiles, `*.md` and `*.txt`.
fn replayable_seeds(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut found = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            found += replayable_seeds(&path);
        } else if path.is_file() {
            let lower = name.to_ascii_lowercase();
            if !lower.ends_with(".md") && !lower.ends_with(".txt") {
                found += 1;
            }
        }
    }
    found
}

/// Fail loudly when the corpus this suite exists to replay is not there.
fn assert_corpus_present() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/regression");
    let found = replayable_seeds(&dir);
    assert!(
        found >= MIN_SEEDS,
        "{} holds {found} replayable seeds, expected at least {MIN_SEEDS} — \
         the committed regression corpus is missing or was renamed, which would \
         otherwise let this test pass without replaying anything",
        dir.display()
    );
}

#[test]
fn fuzz_regression() {
    assert_corpus_present();
    RegressionSuite::new("fuzz/regression")
        .target("decode", |input| {
            let _ = zenbitmaps::decode(input, enough::Unstoppable);
            #[cfg(feature = "bmp")]
            {
                let _ = zenbitmaps::decode_bmp(input, enough::Unstoppable);
            }
            let _ = zenbitmaps::decode_farbfeld(input, enough::Unstoppable);
        })
        .target("roundtrip", |input| {
            // Mirror fuzz/fuzz_targets/fuzz_roundtrip.rs exactly, INCLUDING its
            // pixel-equality asserts — a bare "does not panic" replay would not
            // catch a lossy roundtrip (e.g. zenbitmaps#10, where 16-bit ASCII
            // PPM decoded to a 6-byte "Rgb8" buffer that `encode_pam` truncated
            // back to 3 bytes). The asserts ARE the regression gate.
            use zenbitmaps::{decode, encode_pam};
            if let Ok(decoded) = decode(input, enough::Unstoppable)
                && let Ok(reencoded) = encode_pam(
                    decoded.pixels(),
                    decoded.width,
                    decoded.height,
                    decoded.layout,
                    enough::Unstoppable,
                )
            {
                let decoded2 =
                    decode(&reencoded, enough::Unstoppable).expect("re-encoded PAM must decode");
                assert_eq!(
                    decoded.pixels(),
                    decoded2.pixels(),
                    "PNM PAM roundtrip pixel mismatch"
                );
                assert_eq!(decoded.width, decoded2.width);
                assert_eq!(decoded.height, decoded2.height);
            }
            #[cfg(feature = "bmp")]
            {
                use zenbitmaps::{PixelLayout, decode_bmp, encode_bmp, encode_bmp_rgba};
                if let Ok(decoded) = decode_bmp(input, enough::Unstoppable) {
                    let reencoded = if decoded.layout == PixelLayout::Rgba8 {
                        encode_bmp_rgba(
                            decoded.pixels(),
                            decoded.width,
                            decoded.height,
                            decoded.layout,
                            enough::Unstoppable,
                        )
                    } else {
                        encode_bmp(
                            decoded.pixels(),
                            decoded.width,
                            decoded.height,
                            decoded.layout,
                            enough::Unstoppable,
                        )
                    };
                    if let Ok(reencoded) = reencoded {
                        let decoded2 = decode_bmp(&reencoded, enough::Unstoppable)
                            .expect("re-encoded BMP must decode");
                        assert_eq!(
                            decoded.pixels(),
                            decoded2.pixels(),
                            "BMP roundtrip pixel mismatch"
                        );
                        assert_eq!(decoded.width, decoded2.width);
                        assert_eq!(decoded.height, decoded2.height);
                    }
                }
            }
        })
        .run();
}
