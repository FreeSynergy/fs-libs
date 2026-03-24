use toml::Value;

use fs_error::{RepairAction, ValidationIssue};

use crate::schema::ConfigSchema;

// ── TomlRepair ────────────────────────────────────────────────────────────────

/// Applies [`RepairAction`]s to a mutable [`toml::Value`] and suggests repairs
/// based on schema violations.
pub struct TomlRepair;

impl TomlRepair {
    /// Suggest repair actions for `issues` using default values from `schema`.
    ///
    /// Only produces [`RepairAction::SetDefault`] / [`RepairAction::Insert`] for
    /// fields that have a `default_value` in the schema.
    pub fn suggest(schema: &ConfigSchema, issues: &[ValidationIssue]) -> Vec<RepairAction> {
        let mut actions = Vec::new();

        for issue in issues {
            if let Some(field) = schema.get(&issue.field) {
                if let Some(default) = &field.default_value {
                    actions.push(RepairAction::SetDefault {
                        field: field.path.clone(),
                        value: default.clone(),
                    });
                }
            }
        }

        actions
    }

    /// Apply a list of [`RepairAction`]s to a mutable TOML value in place.
    ///
    /// Returns a list of human-readable descriptions for each successfully
    /// applied action.  Actions that cannot be applied (e.g. type mismatch) are
    /// silently skipped.
    pub fn apply(root: &mut Value, actions: &[RepairAction]) -> Vec<String> {
        let mut applied = Vec::new();

        for action in actions {
            match action {
                RepairAction::SetDefault { field, value }
                | RepairAction::Insert { field, value } => {
                    if let Some(parsed) = parse_toml_value(value) {
                        if set_at_path(root, field, parsed) {
                            applied.push(action.to_string());
                        }
                    }
                }
                RepairAction::Remove { field } => {
                    if remove_at_path(root, field) {
                        applied.push(action.to_string());
                    }
                }
                RepairAction::Rename { from, to } => {
                    if rename_path(root, from, to) {
                        applied.push(action.to_string());
                    }
                }
                RepairAction::Trim { field } => {
                    if trim_string_at_path(root, field) {
                        applied.push(action.to_string());
                    }
                }
            }
        }

        applied
    }
}

// ── Value parsing ─────────────────────────────────────────────────────────────

/// Parse a TOML-encoded value string (e.g. `r#""hello""#` or `"42"`) into a
/// [`toml::Value`].  The input is the *right-hand side* of a TOML assignment.
fn parse_toml_value(encoded: &str) -> Option<Value> {
    // Wrap as a key=value document so the TOML parser can handle it.
    let doc = format!("v = {encoded}");
    toml::from_str::<toml::Table>(&doc)
        .ok()
        .and_then(|mut t| t.remove("v"))
}

// ── Path helpers ──────────────────────────────────────────────────────────────

/// Navigate to a parent table and set `value` at the final path segment.
/// Creates intermediate tables as needed. Returns `true` on success.
fn set_at_path(root: &mut Value, path: &str, value: Value) -> bool {
    let segments: Vec<&str> = path.split('.').collect();
    let (parents, leaf) = segments.split_at(segments.len() - 1);

    let mut current = root;
    for seg in parents {
        if let Value::Table(map) = current {
            current = map
                .entry(*seg)
                .or_insert_with(|| Value::Table(toml::map::Map::new()));
        } else {
            return false;
        }
    }

    if let Value::Table(map) = current {
        map.insert(leaf[0].to_string(), value);
        true
    } else {
        false
    }
}

/// Remove the value at `path`. Returns `true` if something was removed.
fn remove_at_path(root: &mut Value, path: &str) -> bool {
    let segments: Vec<&str> = path.split('.').collect();
    let (parents, leaf) = segments.split_at(segments.len() - 1);

    let mut current = root;
    for seg in parents {
        match current {
            Value::Table(map) => {
                if let Some(next) = map.get_mut(*seg) {
                    current = next;
                } else {
                    return false;
                }
            }
            _ => return false,
        }
    }

    if let Value::Table(map) = current {
        map.remove(leaf[0]).is_some()
    } else {
        false
    }
}

/// Read the value at `from`, write it to `to`, and remove `from`.
/// Returns `true` if both operations succeeded.
fn rename_path(root: &mut Value, from: &str, to: &str) -> bool {
    // Clone the value at `from` first.
    let val = match get_at_path_cloned(root, from) {
        Some(v) => v,
        None => return false,
    };
    let inserted = set_at_path(root, to, val);
    if inserted {
        remove_at_path(root, from);
    }
    inserted
}

/// Trim whitespace from the string value at `path`. Returns `true` if trimmed.
fn trim_string_at_path(root: &mut Value, path: &str) -> bool {
    let segments: Vec<&str> = path.split('.').collect();
    let (parents, leaf) = segments.split_at(segments.len() - 1);

    let mut current = root;
    for seg in parents {
        match current {
            Value::Table(map) => {
                if let Some(next) = map.get_mut(*seg) {
                    current = next;
                } else {
                    return false;
                }
            }
            _ => return false,
        }
    }

    if let Value::Table(map) = current {
        if let Some(Value::String(s)) = map.get_mut(leaf[0]) {
            let trimmed = s.trim().to_string();
            if trimmed != *s {
                *s = trimmed;
                return true;
            }
        }
    }
    false
}

/// Clone the value at a dot-separated path, or `None` if not found.
fn get_at_path_cloned(root: &Value, path: &str) -> Option<Value> {
    let mut current = root;
    for seg in path.split('.') {
        match current {
            Value::Table(map) => current = map.get(seg)?,
            _ => return None,
        }
    }
    Some(current.clone())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ConfigSchema, FieldKind, FieldSchema};
    use crate::validator::SchemaValidator;

    fn parse(s: &str) -> Value {
        toml::from_str(s).unwrap()
    }

    #[test]
    fn suggest_set_default_for_missing_field() {
        let schema = ConfigSchema::new().field(
            FieldSchema::required("name", FieldKind::String, "").with_default(r#""unnamed""#),
        );

        let value = parse("other = 1");
        let issues = SchemaValidator::validate(&schema, &value);
        let actions = TomlRepair::suggest(&schema, &issues);

        assert_eq!(actions.len(), 1);
        match &actions[0] {
            RepairAction::SetDefault { field, value } => {
                assert_eq!(field, "name");
                assert_eq!(value, r#""unnamed""#);
            }
            _ => panic!("expected SetDefault"),
        }
    }

    #[test]
    fn suggest_no_action_when_no_default() {
        let schema =
            ConfigSchema::new().field(FieldSchema::required("name", FieldKind::String, ""));

        let value = parse("other = 1");
        let issues = SchemaValidator::validate(&schema, &value);
        let actions = TomlRepair::suggest(&schema, &issues);
        assert!(actions.is_empty());
    }

    #[test]
    fn apply_set_default_inserts_value() {
        let mut value = parse("other = 1");
        let actions = vec![RepairAction::SetDefault {
            field: "name".into(),
            value: r#""hello""#.into(),
        }];
        let applied = TomlRepair::apply(&mut value, &actions);
        assert_eq!(applied.len(), 1);
        assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("hello"));
    }

    #[test]
    fn apply_remove_deletes_field() {
        let mut value = parse("junk = true\nkeep = 1");
        let actions = vec![RepairAction::Remove {
            field: "junk".into(),
        }];
        TomlRepair::apply(&mut value, &actions);
        assert!(value.get("junk").is_none());
        assert!(value.get("keep").is_some());
    }

    #[test]
    fn apply_rename_moves_field() {
        let mut value = parse("old_name = \"test\"");
        let actions = vec![RepairAction::Rename {
            from: "old_name".into(),
            to: "name".into(),
        }];
        TomlRepair::apply(&mut value, &actions);
        assert!(value.get("old_name").is_none());
        assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("test"));
    }

    #[test]
    fn apply_trim_strips_whitespace() {
        let mut value = parse(r#"name = "  hello  ""#);
        let actions = vec![RepairAction::Trim {
            field: "name".into(),
        }];
        TomlRepair::apply(&mut value, &actions);
        assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("hello"));
    }

    #[test]
    fn apply_set_default_nested_path() {
        let mut value = parse("[server]\nport = 80");
        let actions = vec![RepairAction::SetDefault {
            field: "server.host".into(),
            value: r#""localhost""#.into(),
        }];
        TomlRepair::apply(&mut value, &actions);
        let host = value
            .get("server")
            .and_then(|s| s.get("host"))
            .and_then(|h| h.as_str());
        assert_eq!(host, Some("localhost"));
    }

    #[test]
    fn full_repair_flow_broken_toml() {
        // Simulate: required field 'name' is missing, schema has a default.
        let schema = ConfigSchema::new().field(
            FieldSchema::required("name", FieldKind::String, "project name")
                .with_default(r#""default-project""#),
        );

        let mut value = parse("version = 1");

        // 1. Validate
        let issues = SchemaValidator::validate(&schema, &value);
        assert_eq!(issues.len(), 1);

        // 2. Suggest repairs
        let actions = TomlRepair::suggest(&schema, &issues);
        assert_eq!(actions.len(), 1);

        // 3. Apply
        let applied = TomlRepair::apply(&mut value, &actions);
        assert_eq!(applied.len(), 1);

        // 4. Re-validate — should now pass
        let issues_after = SchemaValidator::validate(&schema, &value);
        assert!(
            issues_after.is_empty(),
            "no issues should remain after repair"
        );
        assert_eq!(
            value.get("name").and_then(|v| v.as_str()),
            Some("default-project"),
        );
    }
}
