//! age X25519 encryption and decryption.

use std::io::{Read, Write};

use age::secrecy::ExposeSecret;
use fs_error::FsError;

// ── Shared helper ─────────────────────────────────────────────────────────────

/// Wrap an already-configured `age::Encryptor` with ASCII armor, write `plaintext`, and
/// finalize — returns the armored ciphertext bytes.
fn finish_armored_write(encryptor: age::Encryptor, plaintext: &[u8]) -> Result<Vec<u8>, FsError> {
    let mut output = Vec::new();
    let armored = age::armor::ArmoredWriter::wrap_output(
        &mut output,
        age::armor::Format::AsciiArmor,
    )
    .map_err(|e| FsError::internal(format!("age armored writer failed: {e}")))?;

    let mut writer = encryptor
        .wrap_output(armored)
        .map_err(|e| FsError::internal(format!("age wrap_output failed: {e}")))?;

    writer
        .write_all(plaintext)
        .map_err(|e| FsError::internal(format!("age write failed: {e}")))?;

    writer
        .finish()
        .and_then(|w| w.finish())
        .map_err(|e| FsError::internal(format!("age finish failed: {e}")))?;

    Ok(output)
}

// ── AgeEncryptor ──────────────────────────────────────────────────────────────

/// Encrypts data using the age encryption format.
///
/// Supports X25519 (public-key) encryption with age recipient keys.
pub struct AgeEncryptor {
    recipient: age::x25519::Recipient,
}

impl AgeEncryptor {
    /// Create from an age X25519 public key string (e.g. `"age1..."`).
    pub fn from_public_key(key: &str) -> Result<Self, FsError> {
        let recipient = key
            .parse::<age::x25519::Recipient>()
            .map_err(|e| FsError::config(format!("invalid age public key: {e}")))?;
        Ok(Self { recipient })
    }

    /// Encrypt plaintext. Returns armored ASCII output.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, FsError> {
        let recipient = self.recipient.clone();
        let encryptor =
            age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
                .map_err(|e| FsError::internal(format!("age encryptor init failed: {e}")))?;
        finish_armored_write(encryptor, plaintext)
    }
}

// ── AgeDecryptor ──────────────────────────────────────────────────────────────

/// Decrypts age-encrypted data using an identity (private key).
pub struct AgeDecryptor {
    identity: age::x25519::Identity,
}

impl AgeDecryptor {
    /// Create from an age X25519 private key string (e.g. `"AGE-SECRET-KEY-1..."`).
    pub fn from_private_key(key: &str) -> Result<Self, FsError> {
        let identity = key
            .parse::<age::x25519::Identity>()
            .map_err(|e| FsError::config(format!("invalid age private key: {e}")))?;
        Ok(Self { identity })
    }

    /// Decrypt armored age ciphertext.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, FsError> {
        let armored = age::armor::ArmoredReader::new(ciphertext);
        let decryptor = age::Decryptor::new(armored)
            .map_err(|e| FsError::internal(format!("age decryptor init failed: {e}")))?;

        let mut reader = decryptor
            .decrypt(std::iter::once(&self.identity as &dyn age::Identity))
            .map_err(|e| FsError::internal(format!("age decrypt failed: {e}")))?;

        let mut plaintext = Vec::new();
        reader
            .read_to_end(&mut plaintext)
            .map_err(|e| FsError::internal(format!("age read failed: {e}")))?;

        Ok(plaintext)
    }
}

// ── AgePassphraseEncryptor ────────────────────────────────────────────────────

/// Encrypts data using a passphrase (age scrypt recipient).
///
/// Used for `vault.toml` secrets protected by a user-supplied passphrase.
pub struct AgePassphraseEncryptor {
    passphrase: age::secrecy::SecretString,
}

impl AgePassphraseEncryptor {
    /// Create a new encryptor with the given passphrase.
    pub fn new(passphrase: impl Into<String>) -> Self {
        use age::secrecy::SecretString;
        Self {
            passphrase: SecretString::new(passphrase.into().into()),
        }
    }

    /// Encrypt `plaintext` with the configured passphrase. Returns armored ASCII output.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, FsError> {
        let encryptor = age::Encryptor::with_user_passphrase(self.passphrase.clone());
        finish_armored_write(encryptor, plaintext)
    }
}

// ── AgePassphraseDecryptor ────────────────────────────────────────────────────

/// Decrypts passphrase-encrypted age data.
pub struct AgePassphraseDecryptor {
    passphrase: age::secrecy::SecretString,
}

impl AgePassphraseDecryptor {
    /// Create a new decryptor with the given passphrase.
    pub fn new(passphrase: impl Into<String>) -> Self {
        use age::secrecy::SecretString;
        Self {
            passphrase: SecretString::new(passphrase.into().into()),
        }
    }

    /// Decrypt armored passphrase-encrypted age ciphertext.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, FsError> {
        let armored = age::armor::ArmoredReader::new(ciphertext);
        let decryptor = age::Decryptor::new(armored)
            .map_err(|e| FsError::internal(format!("age decryptor init failed: {e}")))?;

        let passphrase = self.passphrase.clone();
        let mut reader = decryptor
            .decrypt(std::iter::once(
                &age::scrypt::Identity::new(passphrase) as &dyn age::Identity,
            ))
            .map_err(|e| FsError::internal(format!("age passphrase decrypt failed: {e}")))?;

        let mut plaintext = Vec::new();
        reader
            .read_to_end(&mut plaintext)
            .map_err(|e| FsError::internal(format!("age read failed: {e}")))?;

        Ok(plaintext)
    }
}

// ── Keypair generation ────────────────────────────────────────────────────────

/// Generate a new age X25519 keypair.
///
/// Returns `(public_key_str, private_key_str)`.
pub fn generate_age_keypair() -> (String, String) {
    let identity = age::x25519::Identity::generate();
    let private_key = identity.to_string().expose_secret().to_owned();
    let public_key = identity.to_public().to_string();
    (public_key, private_key)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_roundtrip() {
        let (pub_key, priv_key) = generate_age_keypair();
        let enc = AgeEncryptor::from_public_key(&pub_key).unwrap();
        let dec = AgeDecryptor::from_private_key(&priv_key).unwrap();

        let plaintext = b"hello freeSynergy";
        let ciphertext = enc.encrypt(plaintext).unwrap();
        let recovered  = dec.decrypt(&ciphertext).unwrap();

        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn ciphertext_differs_from_plaintext() {
        let (pub_key, _) = generate_age_keypair();
        let enc = AgeEncryptor::from_public_key(&pub_key).unwrap();
        let ct = enc.encrypt(b"secret").unwrap();
        assert_ne!(&ct, b"secret");
    }

    #[test]
    fn different_plaintext_different_ciphertext() {
        let (pub_key, _) = generate_age_keypair();
        let enc = AgeEncryptor::from_public_key(&pub_key).unwrap();
        let ct1 = enc.encrypt(b"message 1").unwrap();
        let ct2 = enc.encrypt(b"message 2").unwrap();
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let (pub_key, _) = generate_age_keypair();
        let (_, wrong_priv) = generate_age_keypair();

        let enc = AgeEncryptor::from_public_key(&pub_key).unwrap();
        let ct = enc.encrypt(b"secret").unwrap();

        let dec = AgeDecryptor::from_private_key(&wrong_priv).unwrap();
        assert!(dec.decrypt(&ct).is_err(), "wrong key must fail to decrypt");
    }

    #[test]
    fn invalid_public_key_is_rejected() {
        assert!(AgeEncryptor::from_public_key("not-an-age-key").is_err());
    }

    #[test]
    fn invalid_private_key_is_rejected() {
        assert!(AgeDecryptor::from_private_key("not-an-age-key").is_err());
    }

    #[test]
    fn passphrase_roundtrip() {
        let enc = AgePassphraseEncryptor::new("correct-horse-battery-staple");
        let dec = AgePassphraseDecryptor::new("correct-horse-battery-staple");

        let plaintext = b"vault secret";
        let ct = enc.encrypt(plaintext).unwrap();
        let pt = dec.decrypt(&ct).unwrap();

        assert_eq!(pt, plaintext);
    }

    #[test]
    fn wrong_passphrase_fails_to_decrypt() {
        let enc = AgePassphraseEncryptor::new("correct-passphrase");
        let ct = enc.encrypt(b"secret").unwrap();

        let dec = AgePassphraseDecryptor::new("wrong-passphrase");
        assert!(dec.decrypt(&ct).is_err());
    }

    #[test]
    fn keypair_keys_start_with_age_prefix() {
        let (pub_key, priv_key) = generate_age_keypair();
        assert!(pub_key.starts_with("age1"), "public key should start with 'age1'");
        assert!(
            priv_key.starts_with("AGE-SECRET-KEY-"),
            "private key should start with 'AGE-SECRET-KEY-'"
        );
    }
}
