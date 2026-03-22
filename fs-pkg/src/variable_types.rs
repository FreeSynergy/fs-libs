// variable_types.rs — Variable type system for FreeSynergy packages.
//
// Every configurable variable in a package manifest has a type. The type
// determines:
//   - How the value is validated
//   - Whether the value is encrypted at rest (age via fs-crypto)
//   - How the value is displayed in the installer UI
//
// Encrypted types (secret, password, api-key, certificate, private-key) are
// never stored as plain text. The type IS the security policy.
//
// Design:
//   VariableKind    — the type discriminant (enum)
//   VariableSpec    — a fully described variable (name + kind + metadata)
//   ValidationError — why a value failed validation
//
// Pattern: Enum with methods (each variant carries its own validation logic).

use serde::{Deserialize, Serialize};

// ── VariableKind ──────────────────────────────────────────────────────────────

/// The type of a package variable.
///
/// Encrypted variants are automatically handled by `fs-crypto`; callers
/// never see the plaintext after it is stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VariableKind {
    // ── Plaintext types ───────────────────────────────────────────────────
    /// Free-form text.
    String,
    /// Integer, optionally bounded by min/max.
    Integer,
    /// Boolean yes/no toggle.
    Boolean,
    /// HTTP or HTTPS URL.
    Url,
    /// E-mail address.
    Email,
    /// DNS hostname or fully-qualified domain name.
    Hostname,
    /// IPv4 or IPv6 address.
    Ip,
    /// TCP/UDP port number (1–65535).
    Port,
    /// Filesystem path.
    Path,
    /// Database connection string (e.g. `postgres://user:pass@host/db`).
    ConnectionString,
    /// Byte size with SI suffix (e.g. `"100MB"`, `"2GB"`).
    Bytes,
    /// Duration with unit (e.g. `"5m"`, `"1h30m"`).
    Duration,
    /// Cron expression (5 or 6 fields).
    Cron,
    /// Selection from a fixed list of options.
    Select,

    // ── Encrypted types (stored age-encrypted) ────────────────────────────
    /// Generic secret value — stored encrypted.
    Secret,
    /// Password — stored encrypted, strength-checked on input.
    Password,
    /// API key — stored encrypted.
    ApiKey,
    /// PEM certificate — stored encrypted.
    Certificate,
    /// PEM private key — stored encrypted.
    PrivateKey,
}

impl VariableKind {
    /// Returns `true` if values of this kind are stored age-encrypted.
    pub fn is_encrypted(&self) -> bool {
        matches!(
            self,
            Self::Secret | Self::Password | Self::ApiKey | Self::Certificate | Self::PrivateKey
        )
    }


    /// Validate a string value against this type.
    ///
    /// Returns `Ok(())` if the value is acceptable, otherwise a `ValidationError`.
    pub fn validate(&self, value: &str) -> Result<(), ValidationError> {
        match self {
            Self::String           => Ok(()),
            Self::Integer          => parse_integer(value),
            Self::Boolean          => parse_boolean(value),
            Self::Url              => validate_url(value),
            Self::Email            => validate_email(value),
            Self::Hostname         => validate_hostname(value),
            Self::Ip               => validate_ip(value),
            Self::Port             => validate_port(value),
            Self::Path             => Ok(()),  // existence check is caller's responsibility
            Self::ConnectionString => validate_connection_string(value),
            Self::Bytes            => validate_bytes(value),
            Self::Duration         => validate_duration(value),
            Self::Cron             => validate_cron(value),
            Self::Select           => Ok(()),  // options are checked by VariableSpec
            // Encrypted: any non-empty string is accepted
            Self::Secret | Self::Password | Self::ApiKey
            | Self::Certificate | Self::PrivateKey => {
                if value.is_empty() {
                    Err(ValidationError::empty())
                } else {
                    Ok(())
                }
            }
        }
    }
}

// ── ValidationError ───────────────────────────────────────────────────────────

/// Why a value failed type validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Human-readable description of the problem.
    pub message: String,
}

impl ValidationError {
    fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }

    fn empty() -> Self {
        Self::new("value must not be empty")
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Display for VariableKind {
    /// Renders the human-readable UI label (e.g. `"Text"`, `"URL"`, `"Secret 🔒"`).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::String           => "Text",
            Self::Integer          => "Integer",
            Self::Boolean          => "Yes/No",
            Self::Url              => "URL",
            Self::Email            => "E-mail",
            Self::Hostname         => "Hostname",
            Self::Ip               => "IP Address",
            Self::Port             => "Port",
            Self::Path             => "Path",
            Self::ConnectionString => "Connection String",
            Self::Bytes            => "Byte Size",
            Self::Duration         => "Duration",
            Self::Cron             => "Cron Expression",
            Self::Select           => "Selection",
            Self::Secret           => "Secret 🔒",
            Self::Password         => "Password 🔒",
            Self::ApiKey           => "API Key 🔒",
            Self::Certificate      => "Certificate 🔒",
            Self::PrivateKey       => "Private Key 🔒",
        };
        f.write_str(s)
    }
}

// ── VariableSpec ──────────────────────────────────────────────────────────────

/// A fully described package variable.
///
/// Serializes to/from the `[[variables]]` array in a package manifest.
///
/// ```toml
/// [[variables]]
/// name     = "OAUTH_ISSUER_URL"
/// label    = "OAuth Issuer URL"
/// kind     = "url"
/// role     = "iam.oidc-discovery-url"
/// required = true
/// priority = 1
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableSpec {
    /// Environment variable name, e.g. `"OAUTH_ISSUER_URL"`.
    pub name: String,

    /// Display label shown in the installer UI.
    #[serde(default)]
    pub label: String,

    /// The variable type.
    pub kind: VariableKind,

    /// Optional role for automatic value resolution via capability matching.
    ///
    /// E.g. `"iam.oidc-discovery-url"` means: find a running IAM service
    /// and fill this variable with its OIDC discovery URL.
    #[serde(default)]
    pub role: String,

    /// Whether the installer must have a value before proceeding.
    #[serde(default)]
    pub required: bool,

    /// Display priority (1 = most important, shown first).
    #[serde(default = "default_priority")]
    pub priority: u8,

    /// Default value (plaintext; ignored for encrypted kinds).
    #[serde(default)]
    pub default: Option<String>,

    /// Allowed options for `Select` kind.
    #[serde(default)]
    pub options: Vec<String>,

    /// Help text shown in the installer UI.
    #[serde(default)]
    pub help: String,
}

fn default_priority() -> u8 { 3 }

impl VariableSpec {
    /// Validate a value against this spec.
    ///
    /// Checks `required`, type validation, and option membership for `Select`.
    pub fn validate(&self, value: Option<&str>) -> Result<(), ValidationError> {
        match value {
            None | Some("") => {
                if self.required {
                    Err(ValidationError::new(format!(
                        "{} is required but has no value",
                        self.name
                    )))
                } else {
                    Ok(())
                }
            }
            Some(v) => {
                self.kind.validate(v)?;
                if self.kind == VariableKind::Select && !self.options.is_empty() {
                    if !self.options.iter().any(|o| o == v) {
                        return Err(ValidationError::new(format!(
                            "'{}' is not a valid option for {}; expected one of: {}",
                            v,
                            self.name,
                            self.options.join(", ")
                        )));
                    }
                }
                Ok(())
            }
        }
    }

    /// Returns `true` if this variable's value will be stored encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.kind.is_encrypted()
    }

    /// Returns `true` if this variable has a role for auto-resolution.
    pub fn has_role(&self) -> bool {
        !self.role.is_empty()
    }
}

// ── Private validators ────────────────────────────────────────────────────────

fn parse_integer(v: &str) -> Result<(), ValidationError> {
    v.parse::<i64>()
        .map(|_| ())
        .map_err(|_| ValidationError::new(format!("'{}' is not a valid integer", v)))
}

fn parse_boolean(v: &str) -> Result<(), ValidationError> {
    match v.to_lowercase().as_str() {
        "true" | "false" | "yes" | "no" | "1" | "0" => Ok(()),
        _ => Err(ValidationError::new(format!(
            "'{}' is not a valid boolean (expected true/false/yes/no/1/0)",
            v
        ))),
    }
}

fn validate_url(v: &str) -> Result<(), ValidationError> {
    if v.starts_with("http://") || v.starts_with("https://") {
        Ok(())
    } else {
        Err(ValidationError::new(format!(
            "'{}' must start with http:// or https://",
            v
        )))
    }
}

fn validate_email(v: &str) -> Result<(), ValidationError> {
    if v.contains('@') && v.contains('.') {
        Ok(())
    } else {
        Err(ValidationError::new(format!("'{}' is not a valid e-mail address", v)))
    }
}

fn validate_hostname(v: &str) -> Result<(), ValidationError> {
    // Must not be empty and must not contain spaces.
    if v.is_empty() || v.contains(' ') {
        return Err(ValidationError::new(format!("'{}' is not a valid hostname", v)));
    }
    Ok(())
}

fn validate_ip(v: &str) -> Result<(), ValidationError> {
    use std::net::IpAddr;
    v.parse::<IpAddr>()
        .map(|_| ())
        .map_err(|_| ValidationError::new(format!("'{}' is not a valid IP address", v)))
}

fn validate_port(v: &str) -> Result<(), ValidationError> {
    match v.parse::<u16>() {
        Ok(0) => Err(ValidationError::new("port 0 is not valid")),
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new(format!(
            "'{}' is not a valid port number (1–65535)",
            v
        ))),
    }
}

fn validate_connection_string(v: &str) -> Result<(), ValidationError> {
    let known_prefixes = ["postgres://", "postgresql://", "mysql://", "sqlite://", "redis://"];
    if known_prefixes.iter().any(|p| v.starts_with(p)) {
        Ok(())
    } else {
        Err(ValidationError::new(format!(
            "'{}' is not a recognised connection string (expected postgres://, mysql://, …)",
            v
        )))
    }
}

fn validate_bytes(v: &str) -> Result<(), ValidationError> {
    let suffixes = ["KiB", "MiB", "GiB", "TiB", "KB", "MB", "GB", "TB", "B"];
    let upper = v.to_uppercase();
    for sfx in &suffixes {
        if upper.ends_with(sfx) {
            let numeric = &v[..v.len() - sfx.len()];
            // No whitespace allowed between number and unit.
            if numeric.contains(' ') || numeric.contains('\t') {
                return Err(ValidationError::new(format!(
                    "'{}' must not have spaces between the number and unit",
                    v
                )));
            }
            return numeric.trim().parse::<f64>()
                .map(|_| ())
                .map_err(|_| ValidationError::new(format!("'{}' has an invalid numeric part", v)));
        }
    }
    // Also allow a bare integer (bytes)
    v.parse::<u64>()
        .map(|_| ())
        .map_err(|_| ValidationError::new(format!(
            "'{}' is not a valid byte size (e.g. 100MB, 2GB)",
            v
        )))
}

fn validate_duration(v: &str) -> Result<(), ValidationError> {
    // Simple check: must end with a valid time unit and have a numeric part.
    let units = ["ms", "s", "m", "h", "d"];
    for u in &units {
        if v.ends_with(u) {
            let numeric = &v[..v.len() - u.len()];
            return numeric.parse::<f64>()
                .map(|_| ())
                .map_err(|_| ValidationError::new(format!("'{}' has an invalid numeric part", v)));
        }
    }
    Err(ValidationError::new(format!(
        "'{}' is not a valid duration (e.g. 5m, 1h, 30s)",
        v
    )))
}

fn validate_cron(v: &str) -> Result<(), ValidationError> {
    let parts: Vec<&str> = v.split_whitespace().collect();
    if parts.len() == 5 || parts.len() == 6 {
        Ok(())
    } else {
        Err(ValidationError::new(format!(
            "'{}' is not a valid cron expression (expected 5 or 6 fields)",
            v
        )))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_kinds() {
        assert!(VariableKind::Secret.is_encrypted());
        assert!(VariableKind::Password.is_encrypted());
        assert!(VariableKind::ApiKey.is_encrypted());
        assert!(VariableKind::Certificate.is_encrypted());
        assert!(VariableKind::PrivateKey.is_encrypted());
        assert!(!VariableKind::String.is_encrypted());
        assert!(!VariableKind::Url.is_encrypted());
    }

    #[test]
    fn url_validation() {
        assert!(VariableKind::Url.validate("https://example.com").is_ok());
        assert!(VariableKind::Url.validate("http://localhost:8080").is_ok());
        assert!(VariableKind::Url.validate("ftp://nope").is_err());
        assert!(VariableKind::Url.validate("example.com").is_err());
    }

    #[test]
    fn port_validation() {
        assert!(VariableKind::Port.validate("80").is_ok());
        assert!(VariableKind::Port.validate("65535").is_ok());
        assert!(VariableKind::Port.validate("0").is_err());
        assert!(VariableKind::Port.validate("65536").is_err());
        assert!(VariableKind::Port.validate("abc").is_err());
    }

    #[test]
    fn ip_validation() {
        assert!(VariableKind::Ip.validate("192.168.1.1").is_ok());
        assert!(VariableKind::Ip.validate("::1").is_ok());
        assert!(VariableKind::Ip.validate("not-an-ip").is_err());
    }

    #[test]
    fn bytes_validation() {
        assert!(VariableKind::Bytes.validate("100MB").is_ok());
        assert!(VariableKind::Bytes.validate("2GB").is_ok());
        assert!(VariableKind::Bytes.validate("1024").is_ok());
        assert!(VariableKind::Bytes.validate("10 MB").is_err());
    }

    #[test]
    fn duration_validation() {
        assert!(VariableKind::Duration.validate("5m").is_ok());
        assert!(VariableKind::Duration.validate("1h").is_ok());
        assert!(VariableKind::Duration.validate("30s").is_ok());
        assert!(VariableKind::Duration.validate("500ms").is_ok());
        assert!(VariableKind::Duration.validate("abc").is_err());
    }

    #[test]
    fn cron_validation() {
        assert!(VariableKind::Cron.validate("0 0 * * *").is_ok());
        assert!(VariableKind::Cron.validate("*/5 * * * * *").is_ok());
        assert!(VariableKind::Cron.validate("* *").is_err());
    }

    #[test]
    fn connection_string_validation() {
        assert!(VariableKind::ConnectionString.validate("postgres://user:pass@host/db").is_ok());
        assert!(VariableKind::ConnectionString.validate("mysql://user@host/db").is_ok());
        assert!(VariableKind::ConnectionString.validate("http://not-a-db").is_err());
    }

    #[test]
    fn variable_spec_required() {
        let spec = VariableSpec {
            name:     "DB_URL".into(),
            label:    "Database URL".into(),
            kind:     VariableKind::ConnectionString,
            role:     "database.postgres.url".into(),
            required: true,
            priority: 1,
            default:  None,
            options:  vec![],
            help:     String::new(),
        };
        assert!(spec.validate(None).is_err());
        assert!(spec.validate(Some("")).is_err());
        assert!(spec.validate(Some("postgres://user@host/db")).is_ok());
    }

    #[test]
    fn variable_spec_select_validates_options() {
        let spec = VariableSpec {
            name:     "LOG_LEVEL".into(),
            label:    "Log Level".into(),
            kind:     VariableKind::Select,
            role:     String::new(),
            required: true,
            priority: 3,
            default:  Some("info".into()),
            options:  vec!["debug".into(), "info".into(), "warn".into(), "error".into()],
            help:     String::new(),
        };
        assert!(spec.validate(Some("info")).is_ok());
        assert!(spec.validate(Some("trace")).is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let spec = VariableSpec {
            name:     "SECRET_KEY".into(),
            label:    "Secret Key".into(),
            kind:     VariableKind::Secret,
            role:     String::new(),
            required: true,
            priority: 1,
            default:  None,
            options:  vec![],
            help:     "A random secret key.".into(),
        };
        let toml_str = toml::to_string(&spec).unwrap();
        let back: VariableSpec = toml::from_str(&toml_str).unwrap();
        assert_eq!(back.name, spec.name);
        assert!(back.kind.is_encrypted());
    }
}
