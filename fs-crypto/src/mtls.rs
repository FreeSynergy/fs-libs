//! mTLS certificate generation via rcgen.

use fs_error::FsError;
use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair,
    KeyUsagePurpose, SanType,
};
use time::{Duration, OffsetDateTime};

// ── CertBundle ────────────────────────────────────────────────────────────────

/// A PEM-encoded certificate and private key bundle.
#[derive(Debug, Clone)]
pub struct CertBundle {
    /// PEM-encoded certificate.
    pub cert_pem: String,
    /// PEM-encoded private key.
    pub key_pem: String,
}

// ── CaBundle ──────────────────────────────────────────────────────────────────

/// A CA certificate and private key bundle.
#[derive(Debug, Clone)]
pub struct CaBundle {
    /// PEM-encoded CA certificate.
    pub cert_pem: String,
    /// PEM-encoded CA private key.
    pub key_pem: String,
}

impl CaBundle {
    /// Generate a self-signed CA certificate.
    ///
    /// - `common_name` — e.g. `"FreeSynergy Internal CA"`
    /// - `days_valid` — validity period in days
    pub fn generate(common_name: &str, days_valid: u32) -> Result<Self, FsError> {
        let key_pair = KeyPair::generate()
            .map_err(|e| FsError::internal(format!("CA key generation failed: {e}")))?;

        let mut params = CertificateParams::default();
        params
            .distinguished_name
            .push(DnType::CommonName, common_name);
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];

        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::days(days_valid as i64);

        let cert = params
            .self_signed(&key_pair)
            .map_err(|e| FsError::internal(format!("CA self-sign failed: {e}")))?;

        Ok(Self {
            cert_pem: cert.pem(),
            key_pem: key_pair.serialize_pem(),
        })
    }

    /// Issue a server certificate signed by this CA.
    ///
    /// - `common_name` — server hostname (e.g. `"zentinel.example.com"`)
    /// - `san` — Subject Alternative Names (DNS names)
    /// - `days_valid` — validity period in days
    pub fn issue_server_cert(
        &self,
        common_name: &str,
        san: &[&str],
        days_valid: u32,
    ) -> Result<CertBundle, FsError> {
        let ca_key_pair = KeyPair::from_pem(&self.key_pem)
            .map_err(|e| FsError::internal(format!("CA key parse failed: {e}")))?;
        let ca_cert = CertificateParams::from_ca_cert_pem(&self.cert_pem)
            .map_err(|e| FsError::internal(format!("CA cert parse failed: {e}")))?
            .self_signed(&ca_key_pair)
            .map_err(|e| FsError::internal(format!("CA cert rebuild failed: {e}")))?;

        let key_pair = KeyPair::generate()
            .map_err(|e| FsError::internal(format!("server key generation failed: {e}")))?;

        let mut params = CertificateParams::default();
        params
            .distinguished_name
            .push(DnType::CommonName, common_name);
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];

        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::days(days_valid as i64);

        for dns_name in san {
            params
                .subject_alt_names
                .push(SanType::DnsName((*dns_name).try_into().map_err(|e| {
                    FsError::internal(format!("invalid SAN DNS name '{dns_name}': {e}"))
                })?));
        }

        let cert = params
            .signed_by(&key_pair, &ca_cert, &ca_key_pair)
            .map_err(|e| FsError::internal(format!("server cert signing failed: {e}")))?;

        Ok(CertBundle {
            cert_pem: cert.pem(),
            key_pem: key_pair.serialize_pem(),
        })
    }

    /// Issue a client certificate signed by this CA.
    ///
    /// Used for mTLS client authentication.
    pub fn issue_client_cert(
        &self,
        common_name: &str,
        days_valid: u32,
    ) -> Result<CertBundle, FsError> {
        let ca_key_pair = KeyPair::from_pem(&self.key_pem)
            .map_err(|e| FsError::internal(format!("CA key parse failed: {e}")))?;
        let ca_cert = CertificateParams::from_ca_cert_pem(&self.cert_pem)
            .map_err(|e| FsError::internal(format!("CA cert parse failed: {e}")))?
            .self_signed(&ca_key_pair)
            .map_err(|e| FsError::internal(format!("CA cert rebuild failed: {e}")))?;

        let key_pair = KeyPair::generate()
            .map_err(|e| FsError::internal(format!("client key generation failed: {e}")))?;

        let mut params = CertificateParams::default();
        params
            .distinguished_name
            .push(DnType::CommonName, common_name);
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];

        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::days(days_valid as i64);

        let cert = params
            .signed_by(&key_pair, &ca_cert, &ca_key_pair)
            .map_err(|e| FsError::internal(format!("client cert signing failed: {e}")))?;

        Ok(CertBundle {
            cert_pem: cert.pem(),
            key_pem: key_pair.serialize_pem(),
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ca_pem_starts_with_header() {
        let ca = CaBundle::generate("Test CA", 365).unwrap();
        assert!(ca.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(ca.key_pem.contains("BEGIN"));
    }

    #[test]
    fn server_cert_is_issued() {
        let ca = CaBundle::generate("Test CA", 365).unwrap();
        let bundle = ca
            .issue_server_cert("test.example.com", &["test.example.com"], 90)
            .unwrap();
        assert!(bundle.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(bundle.key_pem.contains("BEGIN"));
    }

    #[test]
    fn client_cert_is_issued() {
        let ca = CaBundle::generate("Test CA", 365).unwrap();
        let bundle = ca.issue_client_cert("node-agent", 90).unwrap();
        assert!(bundle.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(bundle.key_pem.contains("BEGIN"));
    }

    #[test]
    fn ca_and_server_pems_differ() {
        let ca = CaBundle::generate("Test CA", 365).unwrap();
        let srv = ca
            .issue_server_cert("a.example.com", &["a.example.com"], 30)
            .unwrap();
        assert_ne!(ca.cert_pem, srv.cert_pem);
        assert_ne!(ca.key_pem, srv.key_pem);
    }

    #[test]
    fn two_cas_produce_different_keys() {
        let ca1 = CaBundle::generate("CA 1", 365).unwrap();
        let ca2 = CaBundle::generate("CA 2", 365).unwrap();
        assert_ne!(ca1.key_pem, ca2.key_pem);
    }
}
