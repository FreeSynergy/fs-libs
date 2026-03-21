// fs-config — TOML config loader / saver with validation, auto-repair, and schema support.
//
// Modules:
//   loader     — ConfigLoader, FeatureFlags, standalone load/save helpers
//   schema     — ConfigSchema, FieldSchema, FieldKind
//   validator  — SchemaValidator (validates toml::Value against a ConfigSchema)
//   repair     — TomlRepair (applies RepairActions to toml::Value, suggests repairs)

pub mod loader;
pub mod repair;
pub mod schema;
pub mod validator;

// ── Flat re-exports ───────────────────────────────────────────────────────────

pub use loader::{load_toml, parse_str, save_toml, ConfigLoader, FeatureFlags};
pub use repair::TomlRepair;
pub use schema::{ConfigSchema, FieldKind, FieldSchema};
pub use validator::SchemaValidator;
