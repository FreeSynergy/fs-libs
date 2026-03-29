//! `BridgeResource` — maps a standardized role API to a concrete service API.

use super::meta::{ResourceMeta, Role};
use serde::{Deserialize, Serialize};

// ── HttpMethod ────────────────────────────────────────────────────────────────

/// HTTP method used for a bridge call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl HttpMethod {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
        }
    }
}

// ── FieldMapping ─────────────────────────────────────────────────────────────

/// A list of field name translations between the standard role API and the
/// concrete service API.
///
/// Each entry maps `standard_field → service_field` (for requests) or
/// `service_field → standard_field` (for responses).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Pairs of `(standard_field, service_field)`.
    pub fields: Vec<(String, String)>,
}

impl FieldMapping {
    /// Create an empty mapping (1:1 pass-through).
    #[must_use]
    pub fn identity() -> Self {
        Self::default()
    }

    /// Add a field translation.
    #[must_use]
    pub fn map(mut self, standard: impl Into<String>, service: impl Into<String>) -> Self {
        self.fields.push((standard.into(), service.into()));
        self
    }
}

// ── BridgeMethod ─────────────────────────────────────────────────────────────

/// One method of a bridge — maps a single standard role API call to the
/// concrete service endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMethod {
    /// Standard role API method name, e.g. `"user.create"`.
    pub standard_name: String,
    /// HTTP method used when calling the service.
    pub http_method: HttpMethod,
    /// Service API endpoint path, e.g. `"/v1/person"`.
    pub endpoint: String,
    /// How standard request fields map to service request fields.
    pub request_mapping: FieldMapping,
    /// How service response fields map to standard response fields.
    pub response_mapping: FieldMapping,
}

// ── BridgeResource ────────────────────────────────────────────────────────────

/// A bridge that maps a standardized role API to a concrete service API.
///
/// Example: `kanidm-iam-bridge` implements `role = "iam"` and maps every
/// standard IAM method (`user.create`, `group.list`, …) to Kanidm's REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResource {
    /// Shared metadata present on every resource.
    pub meta: ResourceMeta,
    /// The standardized role this bridge implements.
    pub target_role: Role,
    /// The concrete service this bridge talks to, e.g. `"kanidm"`.
    pub target_service: String,
    /// All method mappings provided by this bridge.
    pub methods: Vec<BridgeMethod>,
}
