#![deny(clippy::all, clippy::pedantic, warnings)]

use crate::validation::ValidationIssue;

// ── RepairAction ──────────────────────────────────────────────────────────────

/// A concrete, recorded repair operation applied to a single field.
///
/// `RepairAction` describes *what was changed* during auto-repair so the caller
/// can display a human-readable changelog or write it to an audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairAction {
    /// A field was missing or invalid and was set to its default value.
    SetDefault {
        /// Dot-separated field path, e.g. `"project.name"`.
        field: String,
        /// The default value that was applied (as a human-readable string).
        value: String,
    },
    /// A field with a disallowed or unknown value was removed.
    Remove {
        /// Dot-separated field path that was removed.
        field: String,
    },
    /// A deprecated field was migrated to a new name.
    Rename {
        /// Old field path.
        from: String,
        /// New field path.
        to: String,
    },
    /// Leading/trailing whitespace was stripped from a string field.
    Trim {
        /// Dot-separated field path that was trimmed.
        field: String,
    },
    /// A required field was absent and was inserted with a placeholder value.
    Insert {
        /// Dot-separated field path that was inserted.
        field: String,
        /// The placeholder value (as a human-readable string).
        value: String,
    },
}

impl std::fmt::Display for RepairAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepairAction::SetDefault { field, value } => {
                write!(f, "set `{field}` to default `{value}`")
            }
            RepairAction::Remove { field } => write!(f, "removed unknown field `{field}`"),
            RepairAction::Rename { from, to } => write!(f, "renamed `{from}` → `{to}`"),
            RepairAction::Trim { field } => write!(f, "trimmed whitespace in `{field}`"),
            RepairAction::Insert { field, value } => write!(f, "inserted `{field}` = `{value}`"),
        }
    }
}

// ── RepairOption ──────────────────────────────────────────────────────────────

/// One possible fix that can be presented to the user when auto-repair is not
/// possible and a human decision is required.
#[derive(Debug, Clone)]
pub struct RepairOption {
    /// Short label shown in a selection list, e.g. `"Use default value"`.
    pub label: String,
    /// Description of what this option does.
    pub description: String,
}

impl RepairOption {
    /// Construct a repair option.
    pub fn new(label: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: description.into(),
        }
    }
}

// ── RepairOutcome ─────────────────────────────────────────────────────────────

/// What happened when [`Repairable::repair`] was called.
#[derive(Debug)]
pub enum RepairOutcome {
    /// The config was fixed automatically.
    /// Each entry describes one change that was made.
    AutoRepaired(Vec<RepairAction>),
    /// The repair needs a user decision — present these options.
    NeedsUserDecision(Vec<RepairOption>),
    /// Cannot be repaired automatically or by the user — must be recreated.
    Unrecoverable(String),
    /// No issues found — nothing to repair.
    AlreadyValid,
}

impl RepairOutcome {
    /// `true` when the config can be used as-is after this repair attempt.
    #[must_use]
    pub fn is_usable(&self) -> bool {
        matches!(
            self,
            RepairOutcome::AutoRepaired(_) | RepairOutcome::AlreadyValid
        )
    }

    /// Returns the applied repair actions for `AutoRepaired`, otherwise `&[]`.
    ///
    /// Each action implements [`Display`](std::fmt::Display) for human-readable output.
    #[must_use]
    pub fn actions(&self) -> &[RepairAction] {
        match self {
            RepairOutcome::AutoRepaired(actions) => actions,
            _ => &[],
        }
    }
}

// ── Repairable trait ──────────────────────────────────────────────────────────

/// A configuration type that can validate and self-repair its own fields.
///
/// Implement this on structs loaded from TOML so that `fs-config::ConfigLoader`
/// can call `validate()` after deserialization and `repair()` when issues are found.
pub trait Repairable {
    /// Check the config for problems. Returns all issues found (may be empty).
    fn validate(&self) -> Vec<ValidationIssue>;

    /// Attempt to fix the issues returned by `validate()`.
    ///
    /// Mutates `self` in place for auto-fixable issues.
    /// Returns a [`RepairOutcome`] describing what happened.
    fn repair(&mut self) -> RepairOutcome;

    /// Convenience: `true` when `validate()` returns no Error-level issues.
    fn is_valid(&self) -> bool {
        use crate::validation::IssueSeverity;
        self.validate()
            .iter()
            .all(|i| i.severity < IssueSeverity::Error)
    }

    /// Convenience: collect only Error-level issues.
    fn errors(&self) -> Vec<ValidationIssue> {
        use crate::validation::IssueSeverity;
        self.validate()
            .into_iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::ValidationIssue;

    #[test]
    fn repair_action_display() {
        let a = RepairAction::SetDefault {
            field: "x".into(),
            value: "42".into(),
        };
        assert!(a.to_string().contains('x'));
        assert!(a.to_string().contains("42"));

        let b = RepairAction::Rename {
            from: "old".into(),
            to: "new".into(),
        };
        assert!(b.to_string().contains("old"));
        assert!(b.to_string().contains("new"));
    }

    #[test]
    fn repair_outcome_is_usable() {
        assert!(RepairOutcome::AutoRepaired(vec![]).is_usable());
        assert!(RepairOutcome::AlreadyValid.is_usable());
        assert!(!RepairOutcome::NeedsUserDecision(vec![]).is_usable());
        assert!(!RepairOutcome::Unrecoverable("oops".into()).is_usable());
    }

    #[test]
    fn actions_from_auto_repaired() {
        let actions = vec![
            RepairAction::Trim {
                field: "name".into(),
            },
            RepairAction::Remove {
                field: "junk".into(),
            },
        ];
        let outcome = RepairOutcome::AutoRepaired(actions);
        assert_eq!(outcome.actions().len(), 2);
    }

    #[test]
    fn actions_empty_for_other_variants() {
        assert!(RepairOutcome::AlreadyValid.actions().is_empty());
        assert!(RepairOutcome::Unrecoverable("x".into())
            .actions()
            .is_empty());
    }

    struct TrivialConfig {
        name: String,
    }

    impl Repairable for TrivialConfig {
        fn validate(&self) -> Vec<ValidationIssue> {
            if self.name.is_empty() {
                vec![ValidationIssue::error("name", "must not be empty")]
            } else {
                vec![]
            }
        }

        fn repair(&mut self) -> RepairOutcome {
            if self.name.is_empty() {
                self.name = "default".into();
                RepairOutcome::AutoRepaired(vec![RepairAction::SetDefault {
                    field: "name".into(),
                    value: "default".into(),
                }])
            } else {
                RepairOutcome::AlreadyValid
            }
        }
    }

    #[test]
    fn repairable_is_valid() {
        let good = TrivialConfig { name: "ok".into() };
        assert!(good.is_valid());

        let bad = TrivialConfig {
            name: String::new(),
        };
        assert!(!bad.is_valid());
        assert_eq!(bad.errors().len(), 1);
    }

    #[test]
    fn repairable_repair_sets_default() {
        let mut cfg = TrivialConfig {
            name: String::new(),
        };
        let outcome = cfg.repair();
        assert_eq!(cfg.name, "default");
        assert!(outcome.is_usable());
        let summary = outcome.actions();
        assert!(summary[0].to_string().contains("default"));
    }
}
