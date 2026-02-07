#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try decoding as PNM — must never panic
    let _ = zenpnm::decode(data, enough::Unstoppable);

    // Try decoding as BMP explicitly — must never panic
    let _ = zenpnm::decode_bmp(data, enough::Unstoppable);
});
