// variable_roles.rs — Role type system for FreeSynergy package variables.
//
// A role links a variable to a capability that a running service provides.
// The format is `"service-type.capability-value"`, e.g.:
//   "iam.oidc-discovery-url"   → resolved from a running IAM service
//   "smtp.host"                → resolved from a running SMTP service
//   "database.postgres.url"    → resolved from a running PostgreSQL service
//
// Design:
//   VariableRole — parsed role with service_type + capability
//   KNOWN_ROLES  — catalogue of all well-known role strings
//   RoleRegistry — resolves a role string to its metadata
//
// Pattern: Value object (VariableRole), Registry (RoleRegistry).

use serde::{Deserialize, Serialize};

// ── RoleMeta ──────────────────────────────────────────────────────────────────

/// Metadata for a known variable role.
#[derive(Debug, Clone)]
pub struct RoleMeta {
    /// Full role string, e.g. `"iam.oidc-discovery-url"`.
    pub role: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// The service type that provides this role.
    pub service_type: &'static str,
    /// Example value for display in the installer UI.
    pub example: &'static str,
}

/// All well-known variable roles.
///
/// This list is not exhaustive — packages may declare custom roles.
/// The registry uses this as its built-in catalogue.
pub const KNOWN_ROLES: &[RoleMeta] = &[
    // ── IAM ───────────────────────────────────────────────────────────────
    RoleMeta {
        role: "iam.oidc-discovery-url",
        label: "IAM OIDC Discovery URL",
        service_type: "iam",
        example: "https://auth.example.com/.well-known/openid-configuration",
    },
    RoleMeta {
        role: "iam.client-id",
        label: "IAM Client ID",
        service_type: "iam",
        example: "my-app-client",
    },
    RoleMeta {
        role: "iam.client-secret",
        label: "IAM Client Secret",
        service_type: "iam",
        example: "(generated)",
    },
    RoleMeta {
        role: "iam.issuer-url",
        label: "IAM Issuer URL",
        service_type: "iam",
        example: "https://auth.example.com",
    },
    RoleMeta {
        role: "iam.admin-token",
        label: "IAM Admin Token",
        service_type: "iam",
        example: "(generated)",
    },
    // ── SMTP ──────────────────────────────────────────────────────────────
    RoleMeta {
        role: "smtp.host",
        label: "SMTP Hostname",
        service_type: "smtp",
        example: "mail.example.com",
    },
    RoleMeta {
        role: "smtp.port",
        label: "SMTP Port",
        service_type: "smtp",
        example: "587",
    },
    RoleMeta {
        role: "smtp.username",
        label: "SMTP Username",
        service_type: "smtp",
        example: "noreply@example.com",
    },
    RoleMeta {
        role: "smtp.password",
        label: "SMTP Password",
        service_type: "smtp",
        example: "(generated)",
    },
    RoleMeta {
        role: "smtp.from-address",
        label: "SMTP From Address",
        service_type: "smtp",
        example: "noreply@example.com",
    },
    // ── Database / PostgreSQL ──────────────────────────────────────────────
    RoleMeta {
        role: "database.postgres.url",
        label: "PostgreSQL Connection URL",
        service_type: "database",
        example: "postgres://user:pass@localhost/dbname",
    },
    RoleMeta {
        role: "database.postgres.host",
        label: "PostgreSQL Host",
        service_type: "database",
        example: "localhost",
    },
    RoleMeta {
        role: "database.postgres.port",
        label: "PostgreSQL Port",
        service_type: "database",
        example: "5432",
    },
    RoleMeta {
        role: "database.postgres.user",
        label: "PostgreSQL User",
        service_type: "database",
        example: "appuser",
    },
    RoleMeta {
        role: "database.postgres.password",
        label: "PostgreSQL Password",
        service_type: "database",
        example: "(generated)",
    },
    RoleMeta {
        role: "database.postgres.name",
        label: "PostgreSQL Database Name",
        service_type: "database",
        example: "myapp",
    },
    // ── Database / MySQL ───────────────────────────────────────────────────
    RoleMeta {
        role: "database.mysql.url",
        label: "MySQL Connection URL",
        service_type: "database",
        example: "mysql://user:pass@localhost/dbname",
    },
    // ── Cache / Redis ──────────────────────────────────────────────────────
    RoleMeta {
        role: "cache.redis.url",
        label: "Redis Connection URL",
        service_type: "cache",
        example: "redis://localhost:6379",
    },
    // ── Git ────────────────────────────────────────────────────────────────
    RoleMeta {
        role: "git.api-url",
        label: "Git API URL",
        service_type: "git",
        example: "https://git.example.com/api/v1",
    },
    RoleMeta {
        role: "git.base-url",
        label: "Git Base URL",
        service_type: "git",
        example: "https://git.example.com",
    },
    RoleMeta {
        role: "git.admin-token",
        label: "Git Admin Token",
        service_type: "git",
        example: "(generated)",
    },
    // ── Wiki ───────────────────────────────────────────────────────────────
    RoleMeta {
        role: "wiki.api-url",
        label: "Wiki API URL",
        service_type: "wiki",
        example: "https://wiki.example.com/api",
    },
    RoleMeta {
        role: "wiki.base-url",
        label: "Wiki Base URL",
        service_type: "wiki",
        example: "https://wiki.example.com",
    },
    // ── Map ────────────────────────────────────────────────────────────────
    RoleMeta {
        role: "map.api-url",
        label: "Map API URL",
        service_type: "map",
        example: "https://map.example.com/api",
    },
    // ── Chat ───────────────────────────────────────────────────────────────
    RoleMeta {
        role: "chat.homeserver-url",
        label: "Matrix Homeserver URL",
        service_type: "chat",
        example: "https://matrix.example.com",
    },
    RoleMeta {
        role: "chat.admin-token",
        label: "Matrix Admin Token",
        service_type: "chat",
        example: "(generated)",
    },
    // ── Monitoring ─────────────────────────────────────────────────────────
    RoleMeta {
        role: "monitoring.api-url",
        label: "Monitoring API URL",
        service_type: "monitoring",
        example: "https://metrics.example.com",
    },
];

// ── VariableRole ──────────────────────────────────────────────────────────────

/// A parsed variable role string.
///
/// Roles have the format `"service_type.capability"`, e.g.:
///   `"iam.oidc-discovery-url"` → service_type=`"iam"`, capability=`"oidc-discovery-url"`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariableRole {
    /// The service type prefix, e.g. `"iam"`, `"smtp"`, `"database"`.
    pub service_type: String,

    /// The capability suffix, e.g. `"oidc-discovery-url"`, `"host"`.
    pub capability: String,

    /// Full role string (reconstructed from service_type + capability).
    pub role: String,
}

impl VariableRole {
    /// Parse a role string.
    ///
    /// The role must contain at least one dot separating service type from capability.
    ///
    /// ```rust
    /// use fs_pkg::variable_roles::VariableRole;
    ///
    /// let role = VariableRole::parse("iam.oidc-discovery-url").unwrap();
    /// assert_eq!(role.service_type, "iam");
    /// assert_eq!(role.capability, "oidc-discovery-url");
    /// ```
    pub fn parse(role: &str) -> Option<Self> {
        let dot = role.find('.')?;
        let service_type = role[..dot].to_string();
        let capability = role[dot + 1..].to_string();
        if service_type.is_empty() || capability.is_empty() {
            return None;
        }
        Some(Self {
            service_type,
            capability,
            role: role.to_string(),
        })
    }

    /// Returns `true` if this is an encrypted role (password / secret / token).
    pub fn is_sensitive(&self) -> bool {
        let cap = self.capability.as_str();
        cap.contains("password")
            || cap.contains("secret")
            || cap.contains("token")
            || cap.contains("key")
    }
}

// ── RoleRegistry ──────────────────────────────────────────────────────────────

/// Provides metadata lookup for variable roles.
pub struct RoleRegistry;

impl RoleRegistry {
    /// Look up a role by its full string, e.g. `"iam.oidc-discovery-url"`.
    pub fn find(role: &str) -> Option<&'static RoleMeta> {
        KNOWN_ROLES.iter().find(|r| r.role == role)
    }

    /// All roles for a given service type, e.g. `"iam"`.
    pub fn roles_for_service(service_type: &str) -> Vec<&'static RoleMeta> {
        KNOWN_ROLES
            .iter()
            .filter(|r| r.service_type == service_type)
            .collect()
    }

    /// All known service types.
    pub fn service_types() -> Vec<&'static str> {
        let mut types: Vec<&str> = KNOWN_ROLES.iter().map(|r| r.service_type).collect();
        types.dedup();
        types
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_role() {
        let r = VariableRole::parse("iam.oidc-discovery-url").unwrap();
        assert_eq!(r.service_type, "iam");
        assert_eq!(r.capability, "oidc-discovery-url");
    }

    #[test]
    fn parse_nested_role() {
        let r = VariableRole::parse("database.postgres.url").unwrap();
        assert_eq!(r.service_type, "database");
        assert_eq!(r.capability, "postgres.url");
    }

    #[test]
    fn parse_invalid_returns_none() {
        assert!(VariableRole::parse("nodot").is_none());
        assert!(VariableRole::parse("").is_none());
        assert!(VariableRole::parse(".capability").is_none());
    }

    #[test]
    fn sensitive_roles() {
        assert!(VariableRole::parse("iam.client-secret")
            .unwrap()
            .is_sensitive());
        assert!(VariableRole::parse("smtp.password").unwrap().is_sensitive());
        assert!(VariableRole::parse("git.admin-token")
            .unwrap()
            .is_sensitive());
        assert!(!VariableRole::parse("smtp.host").unwrap().is_sensitive());
    }

    #[test]
    fn registry_find() {
        let m = RoleRegistry::find("iam.oidc-discovery-url").unwrap();
        assert_eq!(m.service_type, "iam");
        assert!(!m.example.is_empty());
    }

    #[test]
    fn registry_roles_for_service() {
        let smtp = RoleRegistry::roles_for_service("smtp");
        assert!(smtp.iter().any(|r| r.role == "smtp.host"));
        assert!(smtp.iter().any(|r| r.role == "smtp.port"));
        assert!(smtp.iter().any(|r| r.role == "smtp.password"));
    }

    #[test]
    fn registry_service_types_no_duplicates() {
        let types = RoleRegistry::service_types();
        let mut seen = std::collections::HashSet::new();
        for t in &types {
            assert!(seen.insert(t), "duplicate service type: {}", t);
        }
    }

    #[test]
    fn known_roles_coverage() {
        // Make sure all service types from the workpackage are present.
        let types = RoleRegistry::service_types();
        for expected in &["iam", "smtp", "database", "git", "wiki", "map", "chat"] {
            assert!(
                types.contains(expected),
                "missing service type: {}",
                expected
            );
        }
    }
}
