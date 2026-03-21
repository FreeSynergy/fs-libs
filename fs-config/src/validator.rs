use toml::Value;

use fs_error::ValidationIssue;

use crate::schema::ConfigSchema;

// ── SchemaValidator ───────────────────────────────────────────────────────────

/// Validates a parsed [`toml::Value`] against a [`ConfigSchema`].
///
/// Returns a list of [`ValidationIssue`]s — one per detected problem.
/// An empty list means the value fully conforms to the schema.
pub struct SchemaValidator;

impl SchemaValidator {
    /// Validate `value` against `schema` and return all found issues.
    pub fn validate(schema: &ConfigSchema, value: &Value) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for field in &schema.fields {
            match get_at_path(value, &field.path) {
                None if field.required => {
                    issues.push(ValidationIssue::error(
                        &field.path,
                        format!("required field `{}` is missing", field.path),
                    ));
                }
                None => { /* optional, absent — fine */ }
                Some(v) => {
                    if !field.kind.matches(v) {
                        issues.push(ValidationIssue::error(
                            &field.path,
                            format!(
                                "expected {} but got {}",
                                field.kind.name(),
                                toml_kind_name(v),
                            ),
                        ));
                    }
                }
            }
        }

        issues
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Navigate a dot-separated path into a TOML value.
///
/// Returns `None` if any segment along the path is missing or not a table.
fn get_at_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = root;
    for segment in path.split('.') {
        match current {
            Value::Table(map) => {
                current = map.get(segment)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// Human-readable TOML kind name for error messages.
fn toml_kind_name(value: &Value) -> &'static str {
    match value {
        Value::String(_)   => "string",
        Value::Integer(_)  => "integer",
        Value::Float(_)    => "float",
        Value::Boolean(_)  => "boolean",
        Value::Array(_)    => "array",
        Value::Table(_)    => "table",
        Value::Datetime(_) => "datetime",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ConfigSchema, FieldKind, FieldSchema};

    fn parse(s: &str) -> Value {
        s.parse().unwrap()
    }

    #[test]
    fn valid_config_no_issues() {
        let schema = ConfigSchema::new()
            .field(FieldSchema::required("name", FieldKind::String, ""))
            .field(FieldSchema::required("port", FieldKind::Integer, ""));

        let value = parse(r#"name = "my-project"
port = 8080"#);

        let issues = SchemaValidator::validate(&schema, &value);
        assert!(issues.is_empty());
    }

    #[test]
    fn missing_required_field() {
        let schema = ConfigSchema::new()
            .field(FieldSchema::required("name", FieldKind::String, ""));

        let value = parse("other = 1");
        let issues = SchemaValidator::validate(&schema, &value);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("missing"));
    }

    #[test]
    fn wrong_type_detected() {
        let schema = ConfigSchema::new()
            .field(FieldSchema::required("port", FieldKind::Integer, ""));

        let value = parse(r#"port = "not-a-number""#);
        let issues = SchemaValidator::validate(&schema, &value);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("integer"));
    }

    #[test]
    fn optional_missing_field_no_issue() {
        let schema = ConfigSchema::new()
            .field(FieldSchema::optional("debug", FieldKind::Boolean, ""));

        let value = parse("name = \"x\"");
        let issues = SchemaValidator::validate(&schema, &value);
        assert!(issues.is_empty());
    }

    #[test]
    fn nested_path_validated() {
        let schema = ConfigSchema::new()
            .field(FieldSchema::required("server.host", FieldKind::String, ""));

        let ok = parse("[server]\nhost = \"localhost\"");
        assert!(SchemaValidator::validate(&schema, &ok).is_empty());

        let missing = parse("[server]\nport = 80");
        assert!(!SchemaValidator::validate(&schema, &missing).is_empty());
    }

    #[test]
    fn multiple_issues_collected() {
        let schema = ConfigSchema::new()
            .field(FieldSchema::required("name", FieldKind::String, ""))
            .field(FieldSchema::required("port", FieldKind::Integer, ""))
            .field(FieldSchema::required("enabled", FieldKind::Boolean, ""));

        let value = parse("unrelated = 1");
        let issues = SchemaValidator::validate(&schema, &value);
        assert_eq!(issues.len(), 3);
    }
}
