// oci.rs — OCI image reference parsing.
//
// Follows the OCI Distribution Spec reference grammar:
//   reference  = name [ ":" tag ] [ "@" digest ]
//   name       = [registry "/"] repository
//   registry   = host [":" port]
//   tag        = /[a-zA-Z0-9_.-]{1,128}/
//   digest     = algorithm ":" encoded   (e.g. sha256:abc123…)
//
// Examples:
//   ubuntu:22.04
//   ghcr.io/freesynergy/zentinel:0.1.0
//   ghcr.io/freesynergy/zentinel:0.1.0@sha256:abc123

use std::fmt;

use fs_error::FsError;
use serde::{Deserialize, Serialize};

// ── OciRef ────────────────────────────────────────────────────────────────────

/// An OCI image reference with optional registry, tag, and digest.
///
/// Compliant with the OCI Distribution Spec reference grammar.
///
/// # Examples
///
/// ```rust
/// use fs_pkg::OciRef;
///
/// let r = OciRef::parse("ghcr.io/freesynergy/zentinel:0.1.0").unwrap();
/// assert_eq!(r.registry(), Some("ghcr.io"));
/// assert_eq!(r.repository(), "freesynergy/zentinel");
/// assert_eq!(r.tag(), Some("0.1.0"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OciRef {
    /// Optional registry host (e.g. `"ghcr.io"`, `"docker.io"`).
    /// Absent for Docker Hub short-form refs (`ubuntu:22.04`).
    registry:   Option<String>,

    /// Repository path (e.g. `"freesynergy/zentinel"` or `"library/ubuntu"`).
    repository: String,

    /// Optional tag (e.g. `"latest"`, `"0.1.0"`).
    tag:        Option<String>,

    /// Optional content-addressable digest (e.g. `"sha256:abc123…"`).
    digest:     Option<String>,
}

impl OciRef {
    /// Parse an OCI reference string.
    ///
    /// Returns `Err` if the string is empty or structurally invalid.
    pub fn parse(s: &str) -> Result<Self, FsError> {
        if s.is_empty() {
            return Err(FsError::parse("OCI reference must not be empty"));
        }

        // Split off digest (@sha256:…)
        let (without_digest, digest) = match s.split_once('@') {
            Some((before, after)) => {
                validate_digest(after)?;
                (before, Some(after.to_string()))
            }
            None => (s, None),
        };

        // Split off tag (:tag) — careful not to confuse with registry port
        let (without_tag, tag) = split_tag(without_digest)?;

        // Determine if there is a registry prefix
        let (registry, repository) = split_registry(without_tag);

        if repository.is_empty() {
            return Err(FsError::parse("OCI reference: repository must not be empty"));
        }

        Ok(Self { registry, repository: repository.to_string(), tag, digest })
    }

    /// The registry host, if present (e.g. `"ghcr.io"`).
    pub fn registry(&self) -> Option<&str> {
        self.registry.as_deref()
    }

    /// The repository path (e.g. `"freesynergy/zentinel"`).
    pub fn repository(&self) -> &str {
        &self.repository
    }

    /// The tag, if present (e.g. `"0.1.0"`).
    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    /// The digest, if present (e.g. `"sha256:abc123…"`).
    pub fn digest(&self) -> Option<&str> {
        self.digest.as_deref()
    }

    /// Returns `true` if this reference pins a specific content digest.
    pub fn is_pinned(&self) -> bool {
        self.digest.is_some()
    }

    /// The effective tag or `"latest"` if no tag was specified.
    pub fn tag_or_latest(&self) -> &str {
        self.tag.as_deref().unwrap_or("latest")
    }

    /// The full pull URL for use with registry clients.
    ///
    /// Format: `{registry}/{repository}:{tag}` (or `@{digest}` if pinned).
    pub fn pull_url(&self) -> String {
        let mut s = String::new();
        if let Some(reg) = &self.registry {
            s.push_str(reg);
            s.push('/');
        }
        s.push_str(&self.repository);
        if let Some(digest) = &self.digest {
            s.push('@');
            s.push_str(digest);
        } else {
            s.push(':');
            s.push_str(self.tag_or_latest());
        }
        s
    }
}

impl fmt::Display for OciRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.pull_url())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Split `"name:tag"` into `("name", Some("tag"))`.
///
/// Registry ports (`"ghcr.io:443/…"`) are NOT treated as tags — only the last
/// colon that appears after the final `/` is considered a tag separator.
fn split_tag(s: &str) -> Result<(&str, Option<String>), FsError> {
    // Find the last '/' — everything after it is the image name component
    let name_start = s.rfind('/').map(|i| i + 1).unwrap_or(0);
    let name_part = &s[name_start..];

    match name_part.find(':') {
        Some(colon) => {
            let tag = &name_part[colon + 1..];
            validate_tag(tag)?;
            let without_tag = &s[..name_start + colon];
            Ok((without_tag, Some(tag.to_string())))
        }
        None => Ok((s, None)),
    }
}

/// A registry is present when the first path component contains a `.` or `:`.
///
/// `ubuntu` → no registry
/// `ghcr.io/freesynergy/zentinel` → registry `ghcr.io`
/// `localhost:5000/myimage` → registry `localhost:5000`
fn split_registry(s: &str) -> (Option<String>, &str) {
    match s.split_once('/') {
        Some((first, rest)) if first.contains('.') || first.contains(':') => {
            (Some(first.to_string()), rest)
        }
        _ => (None, s),
    }
}

fn validate_tag(tag: &str) -> Result<(), FsError> {
    if tag.is_empty() || tag.len() > 128 {
        return Err(FsError::parse(format!(
            "OCI tag must be 1–128 chars, got {:?}",
            tag
        )));
    }
    if !tag.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '-') {
        return Err(FsError::parse(format!(
            "OCI tag contains invalid chars: {tag:?}"
        )));
    }
    Ok(())
}

fn validate_digest(digest: &str) -> Result<(), FsError> {
    if !digest.contains(':') {
        return Err(FsError::parse(format!(
            "OCI digest must be 'algorithm:encoded', got {digest:?}"
        )));
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_image_with_tag() {
        let r = OciRef::parse("ubuntu:22.04").unwrap();
        assert_eq!(r.registry(), None);
        assert_eq!(r.repository(), "ubuntu");
        assert_eq!(r.tag(), Some("22.04"));
        assert_eq!(r.digest(), None);
    }

    #[test]
    fn full_ref_with_registry_and_tag() {
        let r = OciRef::parse("ghcr.io/freesynergy/zentinel:0.1.0").unwrap();
        assert_eq!(r.registry(), Some("ghcr.io"));
        assert_eq!(r.repository(), "freesynergy/zentinel");
        assert_eq!(r.tag(), Some("0.1.0"));
    }

    #[test]
    fn ref_with_digest() {
        let r = OciRef::parse(
            "ghcr.io/freesynergy/zentinel:0.1.0@sha256:abcdef1234567890",
        )
        .unwrap();
        assert_eq!(r.tag(), Some("0.1.0"));
        assert_eq!(r.digest(), Some("sha256:abcdef1234567890"));
        assert!(r.is_pinned());
    }

    #[test]
    fn localhost_registry_with_port() {
        let r = OciRef::parse("localhost:5000/myimage:latest").unwrap();
        assert_eq!(r.registry(), Some("localhost:5000"));
        assert_eq!(r.repository(), "myimage");
        assert_eq!(r.tag(), Some("latest"));
    }

    #[test]
    fn image_without_tag_defaults_to_latest() {
        let r = OciRef::parse("ubuntu").unwrap();
        assert_eq!(r.tag(), None);
        assert_eq!(r.tag_or_latest(), "latest");
    }

    #[test]
    fn pull_url_with_registry() {
        let r = OciRef::parse("ghcr.io/freesynergy/zentinel:0.1.0").unwrap();
        assert_eq!(r.pull_url(), "ghcr.io/freesynergy/zentinel:0.1.0");
    }

    #[test]
    fn pull_url_pinned() {
        let r = OciRef::parse(
            "ghcr.io/freesynergy/zentinel@sha256:abc123",
        )
        .unwrap();
        assert_eq!(r.pull_url(), "ghcr.io/freesynergy/zentinel@sha256:abc123");
    }

    #[test]
    fn empty_string_is_error() {
        assert!(OciRef::parse("").is_err());
    }

    #[test]
    fn display_matches_pull_url() {
        let r = OciRef::parse("ghcr.io/freesynergy/zentinel:0.1.0").unwrap();
        assert_eq!(r.to_string(), r.pull_url());
    }

    #[test]
    fn serde_roundtrip() {
        let r = OciRef::parse("ghcr.io/freesynergy/zentinel:0.1.0").unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let back: OciRef = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}
