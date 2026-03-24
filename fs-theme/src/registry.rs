// fs-theme — ThemeRegistry: manage and switch between multiple themes.

use std::collections::HashMap;

use fs_error::FsError;

use crate::engine::ThemeEngine;
use crate::theme::Theme;

/// Registry of named themes with an active selection.
///
/// # Example
/// ```
/// use fs_theme::{ThemeRegistry, Theme};
///
/// let mut reg = ThemeRegistry::default();
/// reg.register(Theme { name: "Dark Amber".into(), ..Theme::default() });
/// reg.set_active("Dark Amber").unwrap();
/// assert_eq!(reg.active().name, "Dark Amber");
/// ```
pub struct ThemeRegistry {
    themes: HashMap<String, Theme>,
    active: String,
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        let default = Theme::default();
        let name = default.name.clone();
        let mut themes = HashMap::new();
        themes.insert(name.clone(), default);
        Self {
            themes,
            active: name,
        }
    }
}

impl ThemeRegistry {
    /// Register (add or replace) a theme.
    pub fn register(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }

    /// Load a theme from a TOML string and register it (for store integration).
    pub fn register_toml_str(&mut self, toml_str: &str) -> Result<&Theme, FsError> {
        let engine = ThemeEngine::from_toml_str(toml_str)?;
        let name = engine.theme().name.clone();
        self.themes.insert(name.clone(), engine.theme);
        Ok(self.themes.get(&name).unwrap())
    }

    /// Parse a CSS block, register the resulting theme, and return a reference.
    pub fn register_css(&mut self, css: &str, name: &str) -> Result<&Theme, FsError> {
        let engine = ThemeEngine::from_css(css, name)?;
        self.themes.insert(name.to_string(), engine.theme);
        Ok(self.themes.get(name).unwrap())
    }

    /// Set the active theme by name. Returns error if not registered.
    pub fn set_active(&mut self, name: &str) -> Result<(), FsError> {
        if self.themes.contains_key(name) {
            self.active = name.to_string();
            Ok(())
        } else {
            Err(FsError::Config(format!("theme '{name}' not registered")))
        }
    }

    /// The currently active [`Theme`].
    pub fn active(&self) -> &Theme {
        self.themes
            .get(&self.active)
            .expect("active theme always present")
    }

    /// A [`ThemeEngine`] for the active theme (for CSS/Tailwind generation).
    pub fn active_engine(&self) -> ThemeEngine {
        ThemeEngine::new(self.active().clone())
    }

    /// All registered theme names, sorted alphabetically.
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.themes.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    /// Remove a theme by name. Fails if it is currently active.
    pub fn remove(&mut self, name: &str) -> Result<(), FsError> {
        if self.active == name {
            return Err(FsError::Config(format!(
                "cannot remove active theme '{name}'"
            )));
        }
        self.themes.remove(name);
        Ok(())
    }
}
