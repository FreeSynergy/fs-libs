use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use fs_error::{FsError, RepairOutcome, Repairable};

// ── ConfigLoader ──────────────────────────────────────────────────────────────

/// TOML config loader and saver with validation, auto-repair, and backup support.
///
/// All relative paths passed to [`load`](ConfigLoader::load) / [`save`](ConfigLoader::save)
/// are resolved against the `base_dir` supplied at construction time.
pub struct ConfigLoader {
    base_dir: PathBuf,
}

impl ConfigLoader {
    /// Create a loader rooted at `base_dir`.
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Resolve a path: if relative, join with `base_dir`; if absolute, use as-is.
    fn resolve(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_dir.join(path)
        }
    }

    /// Load and deserialize a TOML file, then validate and attempt auto-repair.
    ///
    /// Returns `(value, Some(outcome))` when a repair was attempted,
    /// or `(value, None)` when the config was already valid.
    pub fn load<T>(&self, path: &Path) -> Result<(T, Option<RepairOutcome>), FsError>
    where
        T: DeserializeOwned + Repairable,
    {
        let full = self.resolve(path);
        let text = std::fs::read_to_string(&full)
            .map_err(|e| FsError::Config(format!("cannot read {}: {}", full.display(), e)))?;

        let mut value: T = toml::from_str(&text)
            .map_err(|e| FsError::Parse(format!("{}: {}", full.display(), e)))?;

        let issues = value.validate();
        if issues.is_empty() {
            return Ok((value, None));
        }

        let outcome = value.repair();
        Ok((value, Some(outcome)))
    }

    /// Serialize `value` to TOML and write it to `path`, creating a `.bak` backup first.
    pub fn save<T: Serialize>(&self, path: &Path, value: &T) -> Result<(), FsError> {
        let full = self.resolve(path);

        if full.exists() {
            Self::backup(&full)?;
        }

        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(FsError::Io)?;
        }

        let text = toml::to_string_pretty(value)
            .map_err(|e| FsError::config(format!("TOML serialization failed: {e}")))?;

        // Write to a temp file first, then rename (atomic on most OS).
        let tmp = full.with_extension("toml.tmp");
        std::fs::write(&tmp, &text).map_err(FsError::Io)?;
        std::fs::rename(&tmp, &full).map_err(FsError::Io)?;

        Ok(())
    }

    /// Read raw TOML text from a file without deserializing or validating.
    pub fn read_raw(&self, path: &Path) -> Result<String, FsError> {
        let full = self.resolve(path);
        std::fs::read_to_string(&full).map_err(FsError::Io)
    }

    /// Write raw text to a file (no validation, with backup).
    pub fn write_raw(&self, path: &Path, content: &str) -> Result<(), FsError> {
        let full = self.resolve(path);
        if full.exists() {
            Self::backup(&full)?;
        }
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(FsError::Io)?;
        }
        std::fs::write(&full, content).map_err(FsError::Io)
    }

    /// Copy `path` → `path.bak` before overwriting.
    fn backup(path: &Path) -> Result<(), FsError> {
        let bak = path.with_extension("toml.bak");
        std::fs::copy(path, &bak).map_err(FsError::Io)?;
        Ok(())
    }
}

// ── FeatureFlags ──────────────────────────────────────────────────────────────

/// Runtime-readable feature flags loaded from a JSON file.
///
/// Flags are simple `"key": true/false` entries. Unknown keys default to `false`.
///
/// # Example JSON
/// ```json
/// {
///   "experimental_ui": true,
///   "federation": false
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureFlags {
    #[serde(flatten)]
    pub flags: HashMap<String, bool>,
}

impl FeatureFlags {
    /// Load flags from a JSON file. Returns an empty set if the file does not exist.
    pub fn load_json(path: &Path) -> Result<Self, FsError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)
            .map_err(|e| FsError::Config(format!("cannot read flags {}: {e}", path.display())))?;
        serde_json::from_str(&text).map_err(|e| FsError::Parse(format!("feature flags JSON: {e}")))
    }

    /// Save flags to a JSON file.
    pub fn save_json(&self, path: &Path) -> Result<(), FsError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(FsError::Io)?;
        }
        let text = serde_json::to_string_pretty(self)
            .map_err(|e| FsError::config(format!("flags JSON serialization: {e}")))?;
        std::fs::write(path, text).map_err(FsError::Io)
    }

    /// Returns `true` if the flag is explicitly set to `true`.
    pub fn is_enabled(&self, key: &str) -> bool {
        self.flags.get(key).copied().unwrap_or(false)
    }

    /// Enable a flag.
    pub fn enable(&mut self, key: impl Into<String>) {
        self.flags.insert(key.into(), true);
    }

    /// Disable a flag.
    pub fn disable(&mut self, key: impl Into<String>) {
        self.flags.insert(key.into(), false);
    }
}

// ── Standalone helpers ────────────────────────────────────────────────────────

/// Parse a TOML string directly without any file I/O.
///
/// Useful for fuzz tests and unit tests where the content is already in memory.
pub fn parse_str<T: DeserializeOwned>(content: &str) -> Result<T, FsError> {
    toml::from_str(content).map_err(|e| FsError::Parse(format!("TOML parse error: {e}")))
}

/// Load a TOML file directly without a [`ConfigLoader`] instance.
///
/// Useful for one-off loads where no base directory context is needed.
pub fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T, FsError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| FsError::Config(format!("cannot read {}: {}", path.display(), e)))?;
    toml::from_str(&text).map_err(|e| FsError::Parse(format!("{}: {}", path.display(), e)))
}

/// Serialize and write a TOML file directly without a [`ConfigLoader`] instance.
pub fn save_toml<T: Serialize>(path: &Path, value: &T) -> Result<(), FsError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(FsError::Io)?;
    }
    let text = toml::to_string_pretty(value)
        .map_err(|e| FsError::config(format!("TOML serialization failed: {e}")))?;
    std::fs::write(path, text).map_err(FsError::Io)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use fs_error::{RepairAction, RepairOutcome, ValidationIssue};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestConfig {
        name: String,
    }

    impl Repairable for TestConfig {
        fn validate(&self) -> Vec<ValidationIssue> {
            if self.name.is_empty() {
                vec![ValidationIssue::error("name", "must not be empty")]
            } else {
                vec![]
            }
        }

        fn repair(&mut self) -> RepairOutcome {
            if self.name.is_empty() {
                self.name = "default".to_string();
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
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("fs-config-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.toml");

        let loader = ConfigLoader::new(dir.clone());
        let cfg = TestConfig {
            name: "hello".to_string(),
        };
        loader.save(&path, &cfg).unwrap();

        let (loaded, outcome) = loader.load::<TestConfig>(&path).unwrap();
        assert_eq!(loaded.name, "hello");
        assert!(outcome.is_none());
    }

    #[test]
    fn relative_path_resolved_from_base() {
        let dir = std::env::temp_dir().join("fs-config-test-rel");
        std::fs::create_dir_all(&dir).unwrap();
        let loader = ConfigLoader::new(dir.clone());
        loader
            .save(
                Path::new("rel.toml"),
                &TestConfig {
                    name: "relative".to_string(),
                },
            )
            .unwrap();
        assert!(dir.join("rel.toml").exists());
    }

    #[test]
    fn absolute_path_not_prefixed_with_base() {
        let dir = std::env::temp_dir().join("fs-config-test-abs");
        let other = std::env::temp_dir().join("fs-config-test-abs-other");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let abs_path = other.join("abs.toml");

        let loader = ConfigLoader::new(dir.clone());
        loader
            .save(
                &abs_path,
                &TestConfig {
                    name: "absolute".to_string(),
                },
            )
            .unwrap();

        assert!(abs_path.exists());
        assert!(!dir.join("abs.toml").exists());
    }

    #[test]
    fn backup_created_on_overwrite() {
        let dir = std::env::temp_dir().join("fs-config-test-bak");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("backup.toml");
        let loader = ConfigLoader::new(dir.clone());

        loader
            .save(
                &path,
                &TestConfig {
                    name: "first".to_string(),
                },
            )
            .unwrap();
        loader
            .save(
                &path,
                &TestConfig {
                    name: "second".to_string(),
                },
            )
            .unwrap();

        assert!(
            path.with_extension("toml.bak").exists(),
            ".bak should be created on overwrite"
        );
    }

    #[test]
    fn auto_repair_triggered_when_invalid() {
        let dir = std::env::temp_dir().join("fs-config-test-repair");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("repair.toml");
        std::fs::write(&path, r#"name = """#).unwrap();

        let loader = ConfigLoader::new(dir.clone());
        let (loaded, outcome) = loader.load::<TestConfig>(&path).unwrap();
        assert_eq!(
            loaded.name, "default",
            "repair should set name to 'default'"
        );
        assert!(outcome.is_some(), "repair outcome should be returned");
    }

    #[test]
    fn load_returns_error_for_missing_file() {
        let loader = ConfigLoader::new(std::env::temp_dir());
        let result = loader.load::<TestConfig>(Path::new("does-not-exist-fsn.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn standalone_save_and_load_toml() {
        let path = std::env::temp_dir().join("fs-config-standalone.toml");
        save_toml(
            &path,
            &TestConfig {
                name: "standalone".to_string(),
            },
        )
        .unwrap();
        let loaded: TestConfig = load_toml(&path).unwrap();
        assert_eq!(loaded.name, "standalone");
    }

    #[test]
    fn read_raw_returns_file_contents() {
        let dir = std::env::temp_dir().join("fs-config-test-raw");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("raw.toml"), r#"name = "rawval""#).unwrap();

        let loader = ConfigLoader::new(dir.clone());
        let raw = loader.read_raw(Path::new("raw.toml")).unwrap();
        assert!(raw.contains("rawval"));
    }
}
