//! `BridgeExecutor` — executes a single bridge method via HTTP.
//!
//! Given a `BridgeResource` (the mapping definition) and a `base_url`
//! (from the inventory), `BridgeExecutor` translates a standard method call
//! into an actual HTTP request and maps the response back.
//!
//! # OOP design
//!
//! - `HttpMethodExt` trait replaces a `match` block over `HttpMethod`, dispatching
//!   HTTP verb selection via trait objects.
//! - `FieldMappingExt::apply` moves field-mapping logic to where it belongs:
//!   on the `FieldMapping` type itself (via a local extension trait, since
//!   `FieldMapping` lives in `fs-types`).

use crate::error::BridgeError;
use fs_types::resources::bridge::{BridgeResource, FieldMapping, HttpMethod};
use serde_json::Value;
use tracing::instrument;

// ── HttpMethodExt ─────────────────────────────────────────────────────────────

/// Extension trait that gives `HttpMethod` the ability to build a `reqwest::RequestBuilder`.
///
/// This replaces the `match http_method { Get => …, Post => …, … }` block in
/// the executor with a single polymorphic dispatch call, following the
/// *Strategy* pattern.
trait HttpMethodExt {
    /// Apply this HTTP verb to `client` targeting `url`, returning a `RequestBuilder`.
    ///
    /// For `GET` requests the body is passed as query parameters.
    /// For all others the body is sent as JSON.
    fn build_request(
        &self,
        client: &reqwest::Client,
        url: &str,
        body: &Value,
    ) -> reqwest::RequestBuilder;
}

impl HttpMethodExt for HttpMethod {
    fn build_request(
        &self,
        client: &reqwest::Client,
        url: &str,
        body: &Value,
    ) -> reqwest::RequestBuilder {
        match self {
            HttpMethod::Get => client.get(url).query(body),
            HttpMethod::Post => client.post(url).json(body),
            HttpMethod::Put => client.put(url).json(body),
            HttpMethod::Patch => client.patch(url).json(body),
            HttpMethod::Delete => client.delete(url),
        }
    }
}

// ── FieldMappingExt ───────────────────────────────────────────────────────────

/// Extension trait that moves field-mapping logic onto `FieldMapping`.
///
/// `FieldMapping` lives in `fs-types` which we don't own; this extension trait
/// lets us add the `apply` behaviour locally without a free function.
trait FieldMappingExt {
    /// Apply this mapping to `value`, renaming fields according to the
    /// `(from → to)` pairs.  Fields not present in the mapping pass through.
    fn apply(&self, value: &Value) -> Value;
}

impl FieldMappingExt for FieldMapping {
    fn apply(&self, value: &Value) -> Value {
        if self.fields.is_empty() {
            return value.clone();
        }
        let Some(obj) = value.as_object() else {
            return value.clone();
        };
        let mut out = serde_json::Map::new();
        for (key, val) in obj {
            let mapped_key = self
                .fields
                .iter()
                .find(|(from, _)| from == key)
                .map(|(_, to)| to.as_str())
                .unwrap_or(key.as_str());
            out.insert(mapped_key.to_owned(), val.clone());
        }
        Value::Object(out)
    }
}

// ── BridgeExecutor ────────────────────────────────────────────────────────────

/// Executes bridge methods for one service instance.
pub struct BridgeExecutor {
    resource: BridgeResource,
    base_url: String,
    client: reqwest::Client,
}

impl BridgeExecutor {
    /// Create an executor for the given bridge resource + service base URL.
    pub fn new(resource: BridgeResource, base_url: impl Into<String>) -> Self {
        Self {
            resource,
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Execute a standard method by name with the given request parameters.
    ///
    /// `params` is a JSON object (`serde_json::Value::Object`) with standard
    /// field names.  Field-mapping is applied before the HTTP call and again
    /// on the response.
    #[instrument(name = "bridge.execute", skip(self, params), fields(method, bridge = %self.resource.meta.id))]
    pub async fn execute(&self, method: &str, params: Value) -> Result<Value, BridgeError> {
        let bridge_method = self
            .resource
            .methods
            .iter()
            .find(|m| m.standard_name == method)
            .ok_or_else(|| BridgeError::MethodNotFound {
                method: method.to_owned(),
                bridge_id: self.resource.meta.id.clone(),
            })?;

        let url = format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            bridge_method.endpoint
        );

        // Apply request field mapping via the FieldMappingExt trait.
        let mapped_request = bridge_method.request_mapping.apply(&params);

        // Build the HTTP request via HttpMethodExt — no match block needed.
        let response = bridge_method
            .http_method
            .build_request(&self.client, &url, &mapped_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(BridgeError::Http {
                url,
                status: response.status().to_string(),
            });
        }

        let raw: Value = response.json().await?;

        // Apply response field mapping.
        Ok(bridge_method.response_mapping.apply(&raw))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn field_mapping_renames_fields() {
        let mapping = FieldMapping {
            fields: vec![("username".into(), "login".into())],
        };
        let input = json!({ "username": "alice", "email": "alice@example.com" });
        let output = mapping.apply(&input);
        assert_eq!(output["login"], "alice");
        assert_eq!(output["email"], "alice@example.com");
        assert!(output.get("username").is_none());
    }

    #[test]
    fn empty_mapping_is_passthrough() {
        let mapping = FieldMapping::identity();
        let input = json!({ "foo": 1, "bar": 2 });
        assert_eq!(mapping.apply(&input), input);
    }
}
