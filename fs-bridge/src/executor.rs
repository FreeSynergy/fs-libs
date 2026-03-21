//! `BridgeExecutor` — executes a single bridge method via HTTP.
//!
//! Given a `BridgeResource` (the mapping definition) and a `base_url`
//! (from the inventory), `BridgeExecutor` translates a standard method call
//! into an actual HTTP request and maps the response back.

use crate::error::BridgeError;
use fs_types::resources::bridge::{BridgeResource, FieldMapping, HttpMethod};
use serde_json::Value;
use tracing::instrument;

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
    pub async fn execute(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Value, BridgeError> {
        let bridge_method = self
            .resource
            .methods
            .iter()
            .find(|m| m.standard_name == method)
            .ok_or_else(|| BridgeError::MethodNotFound {
                method: method.to_owned(),
                bridge_id: self.resource.meta.id.clone(),
            })?;

        let url = format!("{}{}", self.base_url.trim_end_matches('/'), bridge_method.endpoint);
        let mapped_request = apply_mapping(&params, &bridge_method.request_mapping);

        let response = match bridge_method.http_method {
            HttpMethod::Get    => self.client.get(&url).query(&mapped_request).send().await?,
            HttpMethod::Post   => self.client.post(&url).json(&mapped_request).send().await?,
            HttpMethod::Put    => self.client.put(&url).json(&mapped_request).send().await?,
            HttpMethod::Patch  => self.client.patch(&url).json(&mapped_request).send().await?,
            HttpMethod::Delete => self.client.delete(&url).send().await?,
        };

        if !response.status().is_success() {
            return Err(BridgeError::Http {
                url,
                status: response.status().to_string(),
            });
        }

        let raw: Value = response.json().await?;
        Ok(apply_mapping(&raw, &bridge_method.response_mapping))
    }
}

// ── Field mapping ─────────────────────────────────────────────────────────────

/// Apply a `FieldMapping` to a JSON object.
///
/// For each `(from, to)` pair in the mapping, renames `from` → `to` in the
/// input object.  Fields not in the mapping are passed through unchanged.
fn apply_mapping(value: &Value, mapping: &FieldMapping) -> Value {
    if mapping.fields.is_empty() {
        return value.clone();
    }
    let Some(obj) = value.as_object() else {
        return value.clone();
    };
    let mut out = serde_json::Map::new();
    for (key, val) in obj {
        let mapped_key = mapping
            .fields
            .iter()
            .find(|(from, _)| from == key)
            .map(|(_, to)| to.as_str())
            .unwrap_or(key.as_str());
        out.insert(mapped_key.to_owned(), val.clone());
    }
    Value::Object(out)
}

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
        let output = apply_mapping(&input, &mapping);
        assert_eq!(output["login"], "alice");
        assert_eq!(output["email"], "alice@example.com");
        assert!(output.get("username").is_none());
    }

    #[test]
    fn empty_mapping_is_passthrough() {
        let mapping = FieldMapping::identity();
        let input = json!({ "foo": 1, "bar": 2 });
        assert_eq!(apply_mapping(&input, &mapping), input);
    }
}
