// capability_match.rs — Capability provider/requirer matching.
//
// Each installed service advertises a set of capabilities (what it provides)
// and a set of requirements (what it needs). This module resolves which running
// service can fulfil a given requirement and, for each variable with a role,
// where that value should come from.
//
// Design:
//   ServiceCapabilities — capabilities + requirements declared by one service
//   CapabilityRegistry  — holds all running services' capabilities
//   CapabilityMatch     — one match: requirer variable → provider service + value
//   CapabilityMatcher   — resolves requirements against the registry
//
// Pattern: Registry + Strategy (CapabilityRegistry acts as the registry;
//          CapabilityMatcher is the strategy that picks providers).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::manifest::PackageId;

// ── ServiceCapabilities ───────────────────────────────────────────────────────

/// Capabilities and requirements declared by one installed service.
///
/// Serializes to/from TOML for the `[package.capabilities]` block.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceCapabilities {
    /// Typed unique service identifier (e.g. `"iam/kanidm"`).
    pub service_id: PackageId,

    /// Human-readable name for display.
    pub service_name: String,

    /// Capabilities this service provides, keyed by capability name.
    /// Value is `true` if supported, `false` if explicitly not supported.
    ///
    /// Example: `{ "oidc-provider": true, "scim-server": true, "saml": false }`
    #[serde(default)]
    pub capabilities: HashMap<String, bool>,

    /// Variable roles this service can fulfil, keyed by role string.
    /// Value is the resolved value for that role.
    ///
    /// Example: `{ "iam.oidc-discovery-url": "https://auth.example.com/.well-known/openid-configuration" }`
    #[serde(default)]
    pub provides_roles: HashMap<String, String>,
}

impl ServiceCapabilities {
    /// Returns `true` if this service provides the given capability.
    pub fn has_capability(&self, name: &str) -> bool {
        self.capabilities.get(name).copied().unwrap_or(false)
    }

    /// Returns `true` if this service can fulfil the given variable role.
    pub fn fulfils_role(&self, role: &str) -> bool {
        self.provides_roles.contains_key(role)
    }

    /// Returns the value this service provides for the given role, if any.
    pub fn role_value(&self, role: &str) -> Option<&str> {
        self.provides_roles.get(role).map(String::as_str)
    }

    /// List of capabilities this service explicitly does NOT provide.
    pub fn missing_capabilities(&self) -> Vec<&str> {
        self.capabilities
            .iter()
            .filter(|(_, &v)| !v)
            .map(|(k, _)| k.as_str())
            .collect()
    }
}

// ── CapabilityRegistry ────────────────────────────────────────────────────────

/// Registry of all installed/running services and their capabilities.
///
/// The registry is populated at startup from the local FSN state store.
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    services: Vec<ServiceCapabilities>,
}

impl CapabilityRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a service.
    pub fn register(&mut self, svc: ServiceCapabilities) {
        self.services.retain(|s| s.service_id != svc.service_id);
        self.services.push(svc);
    }

    /// Remove a service by ID.
    pub fn unregister(&mut self, service_id: &str) {
        self.services
            .retain(|s| s.service_id.as_str() != service_id);
    }

    /// All services that provide the given capability.
    pub fn providers_of(&self, capability: &str) -> Vec<&ServiceCapabilities> {
        self.services
            .iter()
            .filter(|s| s.has_capability(capability))
            .collect()
    }

    /// All services that can fulfil the given variable role.
    pub fn providers_for_role(&self, role: &str) -> Vec<&ServiceCapabilities> {
        self.services
            .iter()
            .filter(|s| s.fulfils_role(role))
            .collect()
    }

    /// Iterate over all registered services.
    pub fn all(&self) -> &[ServiceCapabilities] {
        &self.services
    }

    /// Find a service by ID.
    pub fn find(&self, service_id: &str) -> Option<&ServiceCapabilities> {
        self.services
            .iter()
            .find(|s| s.service_id.as_str() == service_id)
    }
}

// ── CapabilityMatch ───────────────────────────────────────────────────────────

/// One resolved match: a requirer variable fulfilled by a provider service.
#[derive(Debug, Clone)]
pub struct CapabilityMatch {
    /// The variable name that needs a value (e.g. `"OAUTH_ISSUER_URL"`).
    pub variable_name: String,

    /// The role this variable plays (e.g. `"iam.oidc-discovery-url"`).
    pub role: String,

    /// Typed identifier of the service that provides the value.
    pub provider_id: PackageId,

    /// Human-readable provider name.
    pub provider_name: String,

    /// The resolved value.
    pub value: String,

    /// `true` if there are multiple providers and the user must choose.
    pub requires_user_choice: bool,

    /// All candidate provider IDs (populated when `requires_user_choice` is true).
    pub candidates: Vec<PackageId>,
}

// ── CapabilityMatcher ─────────────────────────────────────────────────────────

/// Resolves variable roles against the capability registry.
///
/// For each variable that has a role, the matcher finds which running service
/// can provide the value. If multiple providers exist, the match is flagged for
/// user selection.
///
/// # Example
///
/// ```rust
/// use fs_pkg::capability_match::{CapabilityMatcher, CapabilityRegistry, ServiceCapabilities};
///
/// let mut registry = CapabilityRegistry::new();
/// let mut caps = ServiceCapabilities::default();
/// caps.service_id   = "iam/kanidm".into();
/// caps.service_name = "Kanidm".into();
/// caps.provides_roles.insert(
///     "iam.oidc-discovery-url".into(),
///     "https://auth.example.com/.well-known/openid-configuration".into(),
/// );
/// registry.register(caps);
///
/// let matcher = CapabilityMatcher::new(&registry);
/// let result = matcher.resolve_role("OAUTH_ISSUER_URL", "iam.oidc-discovery-url");
/// assert!(result.is_some());
/// ```
pub struct CapabilityMatcher<'a> {
    registry: &'a CapabilityRegistry,
}

impl<'a> CapabilityMatcher<'a> {
    /// Create a new matcher over `registry`.
    pub fn new(registry: &'a CapabilityRegistry) -> Self {
        Self { registry }
    }

    /// Resolve a single variable role to a `CapabilityMatch`.
    ///
    /// Returns `None` if no provider is registered for that role.
    pub fn resolve_role(&self, variable_name: &str, role: &str) -> Option<CapabilityMatch> {
        if role.is_empty() {
            return None;
        }
        let providers = self.registry.providers_for_role(role);
        if providers.is_empty() {
            return None;
        }

        let primary = &providers[0];
        let candidates: Vec<PackageId> = providers.iter().map(|p| p.service_id.clone()).collect();

        Some(CapabilityMatch {
            variable_name: variable_name.to_string(),
            role: role.to_string(),
            provider_id: primary.service_id.clone(),
            provider_name: primary.service_name.clone(),
            value: primary.role_value(role).unwrap_or("").to_string(),
            requires_user_choice: providers.len() > 1,
            candidates,
        })
    }

    /// Resolve multiple (variable_name, role) pairs at once.
    ///
    /// Only returns matches where a provider was found.
    pub fn resolve_all(&self, variables: &[(&str, &str)]) -> Vec<CapabilityMatch> {
        variables
            .iter()
            .filter_map(|(name, role)| self.resolve_role(name, role))
            .collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn kanidm() -> ServiceCapabilities {
        ServiceCapabilities {
            service_id: "iam/kanidm".into(),
            service_name: "Kanidm".into(),
            capabilities: [
                ("oidc-provider".into(), true),
                ("scim-server".into(), true),
                ("saml".into(), false),
            ]
            .into(),
            provides_roles: [
                (
                    "iam.oidc-discovery-url".into(),
                    "https://auth.example.com/.well-known/openid-configuration".into(),
                ),
                ("iam.client-id".into(), "kanidm-client".into()),
            ]
            .into(),
        }
    }

    fn keycloak() -> ServiceCapabilities {
        ServiceCapabilities {
            service_id: "iam/keycloak".into(),
            service_name: "KeyCloak".into(),
            capabilities: [("oidc-provider".into(), true), ("saml".into(), true)].into(),
            provides_roles: [(
                "iam.oidc-discovery-url".into(),
                "https://auth2.example.com/realms/master/.well-known/openid-configuration".into(),
            )]
            .into(),
        }
    }

    #[test]
    fn has_capability() {
        let k = kanidm();
        assert!(k.has_capability("oidc-provider"));
        assert!(k.has_capability("scim-server"));
        assert!(!k.has_capability("saml"));
        assert!(!k.has_capability("radius"));
    }

    #[test]
    fn missing_capabilities() {
        let k = kanidm();
        let missing = k.missing_capabilities();
        assert!(missing.contains(&"saml"));
        assert!(!missing.contains(&"oidc-provider"));
    }

    #[test]
    fn registry_single_provider() {
        let mut reg = CapabilityRegistry::new();
        reg.register(kanidm());

        let matcher = CapabilityMatcher::new(&reg);
        let m = matcher
            .resolve_role("OAUTH_ISSUER_URL", "iam.oidc-discovery-url")
            .unwrap();
        assert_eq!(m.provider_id.as_str(), "iam/kanidm");
        assert!(!m.requires_user_choice);
        assert!(m.value.contains("well-known"));
    }

    #[test]
    fn registry_multiple_providers_require_choice() {
        let mut reg = CapabilityRegistry::new();
        reg.register(kanidm());
        reg.register(keycloak());

        let matcher = CapabilityMatcher::new(&reg);
        let m = matcher
            .resolve_role("OAUTH_ISSUER_URL", "iam.oidc-discovery-url")
            .unwrap();
        assert!(m.requires_user_choice);
        assert_eq!(m.candidates.len(), 2);
    }

    #[test]
    fn resolve_role_missing_returns_none() {
        let reg = CapabilityRegistry::new();
        let matcher = CapabilityMatcher::new(&reg);
        assert!(matcher.resolve_role("SMTP_HOST", "smtp.host").is_none());
    }

    #[test]
    fn resolve_all() {
        let mut reg = CapabilityRegistry::new();
        reg.register(kanidm());

        let matcher = CapabilityMatcher::new(&reg);
        let results = matcher.resolve_all(&[
            ("OAUTH_ISSUER_URL", "iam.oidc-discovery-url"),
            ("CLIENT_ID", "iam.client-id"),
            ("SMTP_HOST", "smtp.host"), // no provider → omitted
        ]);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn registry_unregister() {
        let mut reg = CapabilityRegistry::new();
        reg.register(kanidm());
        reg.unregister("iam/kanidm");
        assert!(reg.find("iam/kanidm").is_none());
    }
}
