//! Fuzz target: parse arbitrary bytes as a generic JSON-Value via fs-config.
//!
//! Exercises the load → validate → repair pipeline with serde_json::Value
//! as a stand-in for any config type that implements Repairable.
//!
//! Run: cargo fuzz run fuzz_parse_str_generic
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Parse as serde_json::Value (accepts any valid TOML)
        let _ = fs_config::parse_str::<serde_json::Value>(s);
    }
});
