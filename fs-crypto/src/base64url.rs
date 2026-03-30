#![deny(clippy::all, clippy::pedantic, warnings)]
//! Base64url (no padding, RFC 4648 §5) encode/decode helpers.
//!
//! Used by `tokens` and `keygen`. Kept in one place to avoid duplication.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode `input` as base64url without padding.
#[must_use]
pub(crate) fn encode(input: &[u8]) -> String {
    let mut output = String::with_capacity((input.len() * 4).div_ceil(3));
    for chunk in input.chunks(3) {
        let b0 = usize::from(chunk[0]);
        let b1 = if chunk.len() > 1 {
            usize::from(chunk[1])
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            usize::from(chunk[2])
        } else {
            0
        };

        output.push(ALPHABET[b0 >> 2] as char);
        output.push(ALPHABET[((b0 & 0x3) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            output.push(ALPHABET[((b1 & 0xf) << 2) | (b2 >> 6)] as char);
        }
        if chunk.len() > 2 {
            output.push(ALPHABET[b2 & 0x3f] as char);
        }
    }
    output
}

/// Decode a base64url string (no padding) into bytes.
///
/// # Errors
/// Returns `Err(())` if any character is not in the base64url alphabet or
/// if the input length is invalid (remainder of 1 after dividing by 4).
pub(crate) fn decode(input: &str) -> Result<Vec<u8>, ()> {
    // Lookup table: 255 = invalid character, otherwise value 0..63.
    const DECODE: [u8; 256] = {
        let mut table = [255u8; 256];
        let mut i = 0u8;
        loop {
            let ch = ALPHABET[i as usize];
            table[ch as usize] = i;
            i += 1;
            if i == 64 {
                break;
            }
        }
        table
    };

    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut i = 0;

    while i < bytes.len() {
        let remaining = bytes.len() - i;
        if remaining < 2 {
            return Err(());
        }

        let v0 = DECODE[bytes[i] as usize];
        let v1 = DECODE[bytes[i + 1] as usize];
        if v0 == 255 || v1 == 255 {
            return Err(());
        }
        output.push((v0 << 2) | (v1 >> 4));

        if remaining >= 3 {
            let v2 = DECODE[bytes[i + 2] as usize];
            if v2 == 255 {
                return Err(());
            }
            output.push(((v1 & 0xf) << 4) | (v2 >> 2));

            if remaining >= 4 {
                let v3 = DECODE[bytes[i + 3] as usize];
                if v3 == 255 {
                    return Err(());
                }
                output.push(((v2 & 0x3) << 6) | v3);
                i += 4;
            } else {
                i += 3;
            }
        } else {
            i += 2;
        }
    }

    Ok(output)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_known_vector() {
        // RFC 4648 §10: "Man" → "TWFu"
        assert_eq!(encode(b"Man"), "TWFu");
    }

    #[test]
    fn encode_empty() {
        assert_eq!(encode(b""), "");
    }

    #[test]
    fn roundtrip() {
        for data in [b"".as_slice(), b"a", b"ab", b"abc", b"hello world!"] {
            let encoded = encode(data);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, data);
        }
    }

    #[test]
    fn decode_invalid_char() {
        assert!(decode("!!!").is_err());
    }
}
