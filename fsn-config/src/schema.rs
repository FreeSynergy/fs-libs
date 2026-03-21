/// The expected value kind for a config field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    /// UTF-8 string value.
    String,
    /// 64-bit integer.
    Integer,
    /// 64-bit float.
    Float,
    /// Boolean (`true` / `false`).
    Boolean,
    /// TOML array (`[...]`).
    Array,
    /// TOML inline table or section (`{...}` / `[section]`).
    Table,
}

impl FieldKind {
    /// Human-readable name for error messages.
    pub fn name(&self) -> &'static str {
        match self {
            FieldKind::String  => "string",
            FieldKind::Integer => "integer",
            FieldKind::Float   => "float",
            FieldKind::Boolean => "boolean",
            FieldKind::Array   => "array",
            FieldKind::Table   => "table",
        }
    }
}

// ── FieldSchema ───────────────────────────────────────────────────────────────

/// Description of a single field expected in a TOML config.
#[derive(Debug, Clone)]
pub struct FieldSchema {
    /// Dot-separated path to the field, e.g. `"project.name"` or `"host.port"`.
    pub path: String,
    /// Expected value kind.
    pub kind: FieldKind,
    /// Whether the field must be present.
    pub required: bool,
    /// Human-readable description shown in validation errors and help text.
    pub description: String,
    /// Default value to use during auto-repair (TOML-encoded string, e.g. `r#""default-name""#`).
    pub default_value: Option<String>,
}

impl FieldSchema {
    /// Create a required field schema.
    pub fn required(
        path: impl Into<String>,
        kind: FieldKind,
        description: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            kind,
            required: true,
            description: description.into(),
            default_value: None,
        }
    }

    /// Create an optional field schema.
    pub fn optional(
        path: impl Into<String>,
        kind: FieldKind,
        description: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            kind,
            required: false,
            description: description.into(),
            default_value: None,
        }
    }

    /// Set the default value (TOML-encoded string). Used by auto-repair.
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default_value = Some(default.into());
        self
    }
}

// ── ConfigSchema ──────────────────────────────────────────────────────────────

/// Declarative description of a TOML config file's expected structure.
///
/// Build a schema with [`ConfigSchema::new`] and chain [`field`](ConfigSchema::field) calls,
/// then pass it to [`SchemaValidator::validate`](crate::validator::SchemaValidator::validate).
///
/// # Example
///
/// ```rust
/// use fsn_config::schema::{ConfigSchema, FieldKind, FieldSchema};
///
/// let schema = ConfigSchema::new()
///     .field(FieldSchema::required("name", FieldKind::String, "Project name"))
///     .field(FieldSchema::optional("port", FieldKind::Integer, "HTTP port")
///         .with_default("8080"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ConfigSchema {
    /// All field descriptors in declaration order.
    pub fields: Vec<FieldSchema>,
}

impl ConfigSchema {
    /// Create an empty schema.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a field descriptor and return `self` (builder pattern).
    pub fn field(mut self, field: FieldSchema) -> Self {
        self.fields.push(field);
        self
    }

    /// Return the descriptor for `path`, or `None` if not declared.
    pub fn get(&self, path: &str) -> Option<&FieldSchema> {
        self.fields.iter().find(|f| f.path == path)
    }

    /// Return all required fields.
    pub fn required_fields(&self) -> impl Iterator<Item = &FieldSchema> {
        self.fields.iter().filter(|f| f.required)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_kind_names() {
        assert_eq!(FieldKind::String.name(), "string");
        assert_eq!(FieldKind::Integer.name(), "integer");
        assert_eq!(FieldKind::Boolean.name(), "boolean");
    }

    #[test]
    fn schema_builder() {
        let schema = ConfigSchema::new()
            .field(FieldSchema::required("name", FieldKind::String, "project name"))
            .field(
                FieldSchema::optional("port", FieldKind::Integer, "port number")
                    .with_default("8080"),
            );

        assert_eq!(schema.fields.len(), 2);
        assert!(schema.get("name").unwrap().required);
        assert!(!schema.get("port").unwrap().required);
        assert_eq!(schema.get("port").unwrap().default_value.as_deref(), Some("8080"));
        assert!(schema.get("missing").is_none());
    }

    #[test]
    fn required_fields_iterator() {
        let schema = ConfigSchema::new()
            .field(FieldSchema::required("a", FieldKind::String, ""))
            .field(FieldSchema::optional("b", FieldKind::Boolean, ""));

        let required: Vec<_> = schema.required_fields().collect();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0].path, "a");
    }
}
