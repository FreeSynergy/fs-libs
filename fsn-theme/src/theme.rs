// fsn-theme — Theme: the complete theme definition loaded from `theme.toml`.

use serde::{Deserialize, Serialize};

use crate::palette::ColorPalette;
use crate::types::{Animation, Glass, Shadows, Spacing, Typography};

/// Complete theme definition loaded from `theme.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub name:       String,
    pub version:    String,
    pub colors:     ColorPalette,
    pub typography: Typography,
    pub spacing:    Spacing,
    /// Glassmorphism parameters. Defaults to `Glass::default()` when absent in TOML.
    #[serde(default)]
    pub glass:      Glass,
    /// Animation timing values. Defaults to `Animation::default()` when absent in TOML.
    #[serde(default)]
    pub animation:  Animation,
    /// Box-shadow definitions. Defaults to `Shadows::default()` when absent in TOML.
    #[serde(default)]
    pub shadows:    Shadows,
    /// Optional dark-mode color overrides emitted in `@media (prefers-color-scheme: dark)`.
    /// When `None`, a comment is emitted indicating the theme is already dark by default.
    #[serde(default)]
    pub dark_colors: Option<ColorPalette>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name:        "FreeSynergy Default".into(),
            version:     "1.0.0".into(),
            colors:      ColorPalette::default(),
            typography:  Typography::default(),
            spacing:     Spacing::default(),
            glass:       Glass::default(),
            animation:   Animation::default(),
            shadows:     Shadows::default(),
            dark_colors: None,
        }
    }
}
