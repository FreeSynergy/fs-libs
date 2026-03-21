# fs-crypto

Cryptographic primitives for FreeSynergy: age file encryption, mTLS certificate
generation, and random key derivation. All functionality is feature-gated to
avoid pulling in unused dependencies.

## Features

| Feature  | Default | Description |
|---|---|---|
| `age`    | no | age X25519 + passphrase encryption/decryption |
| `mtls`   | no | mTLS CA + server/client cert generation (rcgen) |
| `keygen` | no | Random secrets, hex generation, PBKDF2 key derivation |

## Usage

### age encryption (`age` feature)

```rust
use fs_crypto::{generate_age_keypair, AgeEncryptor, AgeDecryptor};

let (public_key, private_key) = generate_age_keypair();

let enc = AgeEncryptor::from_public_key(&public_key)?;
let ciphertext = enc.encrypt(b"my secret data")?;

let dec = AgeDecryptor::from_private_key(&private_key)?;
let plaintext = dec.decrypt(&ciphertext)?;
```

### Passphrase encryption (`age` feature)

```rust
use fs_crypto::{AgePassphraseEncryptor, AgePassphraseDecryptor};

let enc = AgePassphraseEncryptor::new("my passphrase");
let ct  = enc.encrypt(b"vault secret")?;

let dec = AgePassphraseDecryptor::new("my passphrase");
let pt  = dec.decrypt(&ct)?;
```

### mTLS certificates (`mtls` feature)

```rust
use fs_crypto::CaBundle;

let ca = CaBundle::generate("FreeSynergy Internal CA", 3650)?;

let server = ca.issue_server_cert("zentinel.example.com", &["zentinel.example.com"], 365)?;
let client = ca.issue_client_cert("node-agent", 365)?;

// server.cert_pem / server.key_pem  → TLS server config
// client.cert_pem / client.key_pem  → mTLS client config
```

### Key generation (`keygen` feature)

```rust
use fs_crypto::{random_secret, random_hex, derive_key};

let secret = random_secret(32);       // 32-byte base64url secret
let token  = random_hex(16);          // 16-byte hex token
let key    = derive_key(b"password", b"salt", 100_000); // PBKDF2-HMAC-SHA256
```
