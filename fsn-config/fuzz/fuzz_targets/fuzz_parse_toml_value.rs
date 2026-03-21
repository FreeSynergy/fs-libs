//! Fuzz target: parse arbitrary bytes as a TOML value.
//!
//! Finds panics, OOM, or infinite loops in the toml crate's parser when
//! handling malformed or adversarial input.
//!
//! Run: cargo fuzz run fuzz_parse_toml_value
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Exercise the raw TOML parser with arbitrary input
        let _ = toml::from_str::<toml::Value>(s);
        // Also exercise fsn-config's parse_str helper (calls same path)
        let _ = fsn_config::parse_str::<toml::Value>(s);
    }
});
