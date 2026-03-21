// i18n.rs — I18nBundle and related types.
//
// A bundle contains locale metadata + the full ui.toml as a TOML value.
// Consumers call to_hashmap() for dot-notation t("section.key") lookups.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Text direction for a locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TextDirection {
    #[default]
    Ltr,
    Rtl,
}

/// The `[i18n]` section from a locale's `manifest.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nMeta {
    /// BCP-47 locale code, e.g. "de", "ar", "pt-br".
    pub locale_code: String,
    /// Display name in the locale's own script, e.g. "Deutsch".
    pub native_name: String,
    #[serde(default)]
    pub direction: TextDirection,
    #[serde(default)]
    pub completeness: u8,
    #[serde(default = "default_api_version")]
    pub api_version: u32,
}

fn default_api_version() -> u32 { 1 }

/// A fully loaded locale bundle for one namespace + locale code.
pub struct I18nBundle {
    pub meta: I18nMeta,
    /// Full `ui.toml` as a raw TOML value (hierarchy preserved).
    pub ui: toml::Value,
}

impl I18nBundle {
    /// Flattens the TOML hierarchy to dot-notation `HashMap`.
    ///
    /// `[welcome]\ntitle = "Hi"` → `"welcome.title" → "Hi"`.
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        flatten(&self.ui, String::new(), &mut map);
        map
    }

    /// Serializes ui to a JSON string (for REST API / Vue i18n).
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.ui)
    }
}

fn flatten(value: &toml::Value, prefix: String, out: &mut HashMap<String, String>) {
    match value {
        toml::Value::Table(t) => {
            for (k, v) in t {
                let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
                flatten(v, key, out);
            }
        }
        toml::Value::String(s) => { out.insert(prefix, s.clone()); }
        other => { out.insert(prefix, other.to_string()); }
    }
}
