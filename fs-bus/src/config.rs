// fs-bus/src/config.rs — TOML-based routing rule configuration.

use serde::{Deserialize, Serialize};

use crate::error::BusError;
use crate::message::{DeliveryType, StorageType};
use crate::topic::topic_matches;

// ── RoutingRule ───────────────────────────────────────────────────────────────

/// A single routing rule: maps (topic_pattern, optional source_role) → delivery + storage policy.
///
/// Rules are evaluated in descending `priority` order; the first match wins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Unique name for this rule (used in logs and diagnostics).
    pub name: String,
    /// Glob-style topic pattern (same syntax as [`topic_matches`](crate::topic::topic_matches)).
    pub topic_pattern: String,
    /// Optional source role filter. If `None`, the rule matches any source.
    pub source_role: Option<String>,
    /// Delivery policy applied when this rule matches.
    #[serde(default)]
    pub delivery: DeliveryType,
    /// Storage policy applied when this rule matches.
    #[serde(default)]
    pub storage: StorageType,
    /// Higher priority = evaluated first. Default: `0`.
    #[serde(default)]
    pub priority: i32,
    /// Inactive rules are skipped during evaluation.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl RoutingRule {
    /// Returns `true` if this rule matches `topic` from `source_role`.
    pub fn matches(&self, topic: &str, source_role: Option<&str>) -> bool {
        if !self.enabled {
            return false;
        }
        if !topic_matches(&self.topic_pattern, topic) {
            return false;
        }
        if let Some(required) = &self.source_role {
            if source_role != Some(required.as_str()) {
                return false;
            }
        }
        true
    }
}

// ── RoutingConfig ─────────────────────────────────────────────────────────────

/// Bus routing configuration: a sorted list of [`RoutingRule`]s.
///
/// Load from a TOML file at startup. The bus evaluates rules in descending
/// priority order and stops at the first match.
///
/// # TOML example
///
/// ```toml
/// [[rules]]
/// name         = "auth-guaranteed"
/// topic_pattern = "auth.*"
/// source_role  = "iam"
/// delivery     = "guaranteed"
/// storage      = "until-ack"
/// priority     = 10
///
/// [[rules]]
/// name         = "catch-all"
/// topic_pattern = "#"
/// delivery     = "fire-and-forget"
/// storage      = "no-store"
/// priority     = 0
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoutingConfig {
    /// Routing rules sorted by priority (descending) after loading.
    #[serde(default)]
    pub rules: Vec<RoutingRule>,
}

impl RoutingConfig {
    /// Parse from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self, BusError> {
        let mut cfg: Self = toml::from_str(content)
            .map_err(|e| BusError::internal(format!("routing config parse error: {e}")))?;
        cfg.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(cfg)
    }

    /// Load from a file path.
    pub fn load(path: &str) -> Result<Self, BusError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| BusError::internal(format!("cannot read {path}: {e}")))?;
        Self::from_toml(&content)
    }

    /// Find the first matching rule for `topic` emitted by `source_role`.
    pub fn match_rule(&self, topic: &str, source_role: Option<&str>) -> Option<&RoutingRule> {
        self.rules.iter().find(|r| r.matches(topic, source_role))
    }

    /// Returns the delivery type for `topic`. Falls back to fire-and-forget.
    pub fn delivery_for(&self, topic: &str, source_role: Option<&str>) -> DeliveryType {
        self.match_rule(topic, source_role)
            .map(|r| r.delivery.clone())
            .unwrap_or_default()
    }

    /// Returns the storage type for `topic`. Falls back to no-store.
    pub fn storage_for(&self, topic: &str, source_role: Option<&str>) -> StorageType {
        self.match_rule(topic, source_role)
            .map(|r| r.storage.clone())
            .unwrap_or_default()
    }

    /// Total number of rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Returns `true` if no rules are configured.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TOML: &str = r##"
[[rules]]
name          = "auth-guaranteed"
topic_pattern = "auth.*"
source_role   = "iam"
delivery      = "guaranteed"
storage       = "until-ack"
priority      = 10

[[rules]]
name          = "catch-all"
topic_pattern = "#"
delivery      = "fire-and-forget"
storage       = "no-store"
priority      = 0
"##;

    #[test]
    fn parse_toml() {
        let cfg = RoutingConfig::from_toml(SAMPLE_TOML).unwrap();
        assert_eq!(cfg.len(), 2);
        // Higher priority first
        assert_eq!(cfg.rules[0].name, "auth-guaranteed");
    }

    #[test]
    fn match_auth_rule() {
        let cfg = RoutingConfig::from_toml(SAMPLE_TOML).unwrap();
        let rule = cfg.match_rule("auth.login", Some("iam")).unwrap();
        assert_eq!(rule.name, "auth-guaranteed");
        assert_eq!(rule.delivery, DeliveryType::Guaranteed);
    }

    #[test]
    fn fallback_to_catch_all() {
        let cfg = RoutingConfig::from_toml(SAMPLE_TOML).unwrap();
        let rule = cfg.match_rule("deploy.started", None).unwrap();
        assert_eq!(rule.name, "catch-all");
    }

    #[test]
    fn delivery_for_shorthand() {
        let cfg = RoutingConfig::from_toml(SAMPLE_TOML).unwrap();
        assert_eq!(
            cfg.delivery_for("auth.login", Some("iam")),
            DeliveryType::Guaranteed
        );
        assert_eq!(
            cfg.delivery_for("anything", None),
            DeliveryType::FireAndForget
        );
    }

    #[test]
    fn empty_config_fallback() {
        let cfg = RoutingConfig::default();
        assert_eq!(
            cfg.delivery_for("any.topic", None),
            DeliveryType::FireAndForget
        );
        assert_eq!(cfg.storage_for("any.topic", None), StorageType::NoStore);
    }
}
