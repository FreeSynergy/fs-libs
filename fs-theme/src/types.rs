// fs-theme — Value objects: Typography, Spacing, Glass, Animation, Shadows,
//             and the enums ShadowLevel / AnimationKind.

use serde::{Deserialize, Serialize};

// ── Typography ─────────────────────────────────────────────────────────────────

/// Font and text settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Typography {
    pub font_family: String,
    pub font_size:   u8,
    pub line_height: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_family: "\"JetBrains Mono\", \"Fira Code\", monospace".into(),
            font_size:   14,
            line_height: 1.6,
        }
    }
}

// ── Spacing ────────────────────────────────────────────────────────────────────

/// Spacing scale (px values).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Spacing {
    pub xs: u8,
    pub sm: u8,
    pub md: u8,
    pub lg: u8,
    pub xl: u8,
}

impl Default for Spacing {
    fn default() -> Self {
        Self { xs: 4, sm: 8, md: 16, lg: 24, xl: 32 }
    }
}

// ── Glass ──────────────────────────────────────────────────────────────────────

/// Glassmorphism parameters for CSS generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Glass {
    /// Background opacity (0.0–1.0).
    pub bg_opacity:     f32,
    /// Blur radius in pixels.
    pub blur:           u8,
    /// Border opacity (0.0–1.0).
    pub border_opacity: f32,
}

impl Default for Glass {
    fn default() -> Self {
        Self { bg_opacity: 0.08, blur: 12, border_opacity: 0.15 }
    }
}

// ── Animation ─────────────────────────────────────────────────────────────────

/// Animation timing values in milliseconds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Animation {
    pub fast: u16,
    pub base: u16,
    pub slow: u16,
}

impl Default for Animation {
    fn default() -> Self {
        Self { fast: 150, base: 250, slow: 400 }
    }
}

// ── Shadows ────────────────────────────────────────────────────────────────────

/// Box-shadow definitions for the four standard elevation levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Shadows {
    pub sm: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
}

impl Default for Shadows {
    fn default() -> Self {
        Self {
            sm: "0 1px 3px rgba(0,0,0,0.4)".into(),
            md: "0 4px 12px rgba(0,0,0,0.5)".into(),
            lg: "0 8px 24px rgba(0,0,0,0.6)".into(),
            xl: "0 16px 48px rgba(0,0,0,0.7)".into(),
        }
    }
}

// ── ShadowLevel ───────────────────────────────────────────────────────────────

/// The four standard shadow elevation levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowLevel {
    Sm,
    Md,
    Lg,
    Xl,
}

impl ShadowLevel {
    /// CSS variable name for this shadow level (e.g. `"--shadow-sm"`).
    pub fn css_var(self) -> &'static str {
        match self {
            Self::Sm => "--shadow-sm",
            Self::Md => "--shadow-md",
            Self::Lg => "--shadow-lg",
            Self::Xl => "--shadow-xl",
        }
    }
}

// ── AnimationKind ─────────────────────────────────────────────────────────────

/// The three standard animation timing categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationKind {
    Fast,
    Base,
    Slow,
}

impl AnimationKind {
    /// CSS variable name for this animation kind (e.g. `"--transition-fast"`).
    pub fn css_var(self) -> &'static str {
        match self {
            Self::Fast => "--transition-fast",
            Self::Base => "--transition-base",
            Self::Slow => "--transition-slow",
        }
    }

    /// Default animation duration in milliseconds (used for high-contrast / fallback).
    pub fn default_ms(self) -> u16 {
        match self {
            Self::Fast => 150,
            Self::Base => 250,
            Self::Slow => 400,
        }
    }
}
