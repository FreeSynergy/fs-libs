//! Theme resource types — individually loadable style components.
//!
//! Every theme component has a **strictly standardized schema**.  A validator
//! checks that all mandatory fields are present.  Missing fields → `ValidationStatus::Broken`.

use super::meta::ResourceMeta;
use serde::{Deserialize, Serialize};

// ── ColorScheme ───────────────────────────────────────────────────────────────

/// All mandatory CSS colour tokens for a FreeSynergy colour scheme.
///
/// Every field corresponds to a CSS custom property prefixed with the
/// colour scheme id at load time (e.g. `--midnight-blue-bg-base`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorTokens {
    // Backgrounds
    pub bg_base: String,
    pub bg_surface: String,
    pub bg_elevated: String,
    pub bg_card: String,
    pub bg_input: String,
    // Text
    pub text_primary: String,
    pub text_secondary: String,
    pub text_muted: String,
    // Accent colours
    pub primary: String,
    pub primary_hover: String,
    pub primary_text: String,
    pub accent: String,
    // Semantic
    pub success: String,
    pub warning: String,
    pub error: String,
    // Borders
    pub border: String,
    pub border_focus: String,
}

impl ColorTokens {
    /// Validate that no token value is empty.
    pub fn is_complete(&self) -> bool {
        ![
            &self.bg_base, &self.bg_surface, &self.bg_elevated, &self.bg_card,
            &self.bg_input, &self.text_primary, &self.text_secondary, &self.text_muted,
            &self.primary, &self.primary_hover, &self.primary_text, &self.accent,
            &self.success, &self.warning, &self.error, &self.border, &self.border_focus,
        ]
        .iter()
        .any(|v| v.trim().is_empty())
    }
}

/// A colour palette resource — the most fundamental theme component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub meta: ResourceMeta,
    pub colors: ColorTokens,
}

impl ColorScheme {
    /// Validate tokens and update `meta.status`.
    pub fn validate(&mut self) {
        self.meta.validate();
        if !self.colors.is_complete() {
            self.meta.status = super::meta::ValidationStatus::Broken;
        }
    }
}

// ── StyleResource (G3) ────────────────────────────────────────────────────────

/// All mandatory style tokens for a FreeSynergy style resource.
///
/// Every field has a strict CSS value format.  Missing or empty → `Broken`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleTokens {
    // Radius
    pub radius_sm: String,
    pub radius: String,
    pub radius_lg: String,
    // Spacing
    pub spacing_xs: String,
    pub spacing_sm: String,
    pub spacing_md: String,
    pub spacing_lg: String,
    pub spacing_xl: String,
    // Shadows
    pub shadow_sm: String,
    pub shadow: String,
    pub shadow_lg: String,
    pub shadow_glow: String,
    // Borders
    pub border_width: String,
    // Scrollbar
    pub scrollbar_width: String,
    // Sidebar
    pub sidebar_width_collapsed: String,
    pub sidebar_width_expanded: String,
    // Transitions
    pub transition_fast: String,
    pub transition: String,
    pub transition_slow: String,
}

impl StyleTokens {
    /// Validate that no token value is empty.
    pub fn is_complete(&self) -> bool {
        ![
            &self.radius_sm, &self.radius, &self.radius_lg,
            &self.spacing_xs, &self.spacing_sm, &self.spacing_md,
            &self.spacing_lg, &self.spacing_xl,
            &self.shadow_sm, &self.shadow, &self.shadow_lg, &self.shadow_glow,
            &self.border_width, &self.scrollbar_width,
            &self.sidebar_width_collapsed, &self.sidebar_width_expanded,
            &self.transition_fast, &self.transition, &self.transition_slow,
        ]
        .iter()
        .any(|v| v.trim().is_empty())
    }

    /// Return the default style tokens (matching the FreeSynergy base theme).
    pub fn default_tokens() -> Self {
        Self {
            radius_sm:               "4px".into(),
            radius:                  "8px".into(),
            radius_lg:               "16px".into(),
            spacing_xs:              "4px".into(),
            spacing_sm:              "8px".into(),
            spacing_md:              "16px".into(),
            spacing_lg:              "24px".into(),
            spacing_xl:              "32px".into(),
            shadow_sm:               "0 1px 2px rgba(0,0,0,0.1)".into(),
            shadow:                  "0 2px 4px rgba(0,0,0,0.1)".into(),
            shadow_lg:               "0 4px 12px rgba(0,0,0,0.15)".into(),
            shadow_glow:             "0 0 20px rgba(77,139,245,0.15)".into(),
            border_width:            "1px".into(),
            scrollbar_width:         "6px".into(),
            sidebar_width_collapsed: "48px".into(),
            sidebar_width_expanded:  "220px".into(),
            transition_fast:         "150ms ease".into(),
            transition:              "200ms ease".into(),
            transition_slow:         "300ms ease".into(),
        }
    }
}

/// Spacing, radius, shadows — the standardized style resource.
///
/// All fields in `StyleTokens` are mandatory.  A missing field causes
/// `ValidationStatus::Broken`.  This is enforced by `validate()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleResource {
    pub meta: ResourceMeta,
    pub style: StyleTokens,
}

impl StyleResource {
    /// Validate tokens and update `meta.status`.
    pub fn validate(&mut self) {
        self.meta.validate();
        if !self.style.is_complete() {
            self.meta.status = super::meta::ValidationStatus::Broken;
        }
    }
}

// ── FontSet ───────────────────────────────────────────────────────────────────

/// A font family declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFace {
    /// CSS font-family name, e.g. `"Inter"`.
    pub family: String,
    /// Path to the font file within the package.
    pub file: String,
    /// CSS font-weight value, e.g. `"400"` or `"bold"`.
    pub weight: String,
    /// CSS font-style, e.g. `"normal"` or `"italic"`.
    pub style: String,
}

/// A collection of font face declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSet {
    pub meta: ResourceMeta,
    /// Primary UI font faces.
    pub ui_fonts: Vec<FontFace>,
    /// Monospace font faces (code, terminal).
    pub mono_fonts: Vec<FontFace>,
}

// ── CursorSet ─────────────────────────────────────────────────────────────────

/// A mouse cursor file mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorFile {
    /// CSS cursor name, e.g. `"default"`, `"pointer"`, `"text"`.
    pub cursor_name: String,
    /// Path to the `.cur` or `.svg` file within the package.
    pub file: String,
}

/// A collection of custom mouse cursor files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorSet {
    pub meta: ResourceMeta,
    pub cursors: Vec<CursorFile>,
}

// ── IconSet ───────────────────────────────────────────────────────────────────

/// A named SVG icon entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconEntry {
    /// Semantic name used in `<FsnIcon name="…" />`, e.g. `"home"`.
    pub name: String,
    /// Path to the SVG file within the package.
    pub file: String,
}

/// A collection of SVG icons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconSet {
    pub meta: ResourceMeta,
    pub icons: Vec<IconEntry>,
}

// ── ButtonStyle ───────────────────────────────────────────────────────────────

/// CSS token overrides for button appearance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonStyleTokens {
    pub border_radius: String,
    pub padding_x: String,
    pub padding_y: String,
    pub font_weight: String,
    pub border_width: String,
    pub hover_transform: String,
}

impl ButtonStyleTokens {
    pub fn is_complete(&self) -> bool {
        ![
            &self.border_radius, &self.padding_x, &self.padding_y,
            &self.font_weight, &self.border_width, &self.hover_transform,
        ]
        .iter()
        .any(|v| v.trim().is_empty())
    }
}

/// Button appearance resource — shape, padding, hover effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonStyle {
    pub meta: ResourceMeta,
    pub tokens: ButtonStyleTokens,
}

impl ButtonStyle {
    pub fn validate(&mut self) {
        self.meta.validate();
        if !self.tokens.is_complete() {
            self.meta.status = super::meta::ValidationStatus::Broken;
        }
    }
}

// ── WindowChrome ──────────────────────────────────────────────────────────────

/// CSS token overrides for window title bars and resize handles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowChromeTokens {
    pub titlebar_height: String,
    pub titlebar_bg: String,
    pub titlebar_text: String,
    pub resize_handle_size: String,
    pub button_close_color: String,
    pub button_minimize_color: String,
}

impl WindowChromeTokens {
    pub fn is_complete(&self) -> bool {
        ![
            &self.titlebar_height, &self.titlebar_bg, &self.titlebar_text,
            &self.resize_handle_size, &self.button_close_color, &self.button_minimize_color,
        ]
        .iter()
        .any(|v| v.trim().is_empty())
    }
}

/// Window title-bar and chrome resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowChrome {
    pub meta: ResourceMeta,
    pub tokens: WindowChromeTokens,
}

impl WindowChrome {
    pub fn validate(&mut self) {
        self.meta.validate();
        if !self.tokens.is_complete() {
            self.meta.status = super::meta::ValidationStatus::Broken;
        }
    }
}

// ── AnimationSet ─────────────────────────────────────────────────────────────

/// CSS transition and keyframe animation overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationTokens {
    pub transition_fast: String,
    pub transition: String,
    pub transition_slow: String,
    /// Additional raw CSS `@keyframes` blocks.
    pub keyframes: String,
}

impl AnimationTokens {
    pub fn is_complete(&self) -> bool {
        ![&self.transition_fast, &self.transition, &self.transition_slow]
            .iter()
            .any(|v| v.trim().is_empty())
    }
}

/// CSS animation and transition resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSet {
    pub meta: ResourceMeta,
    pub tokens: AnimationTokens,
}

impl AnimationSet {
    pub fn validate(&mut self) {
        self.meta.validate();
        if !self.tokens.is_complete() {
            self.meta.status = super::meta::ValidationStatus::Broken;
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::meta::{Dependency, ResourceType, ValidationStatus};
    use std::path::PathBuf;

    fn base_meta(rt: ResourceType) -> ResourceMeta {
        ResourceMeta {
            id: "test".into(),
            name: "Test".into(),
            description: "A sufficiently long description.".into(),
            version: "1.0.0".into(),
            author: "FreeSynergy".into(),
            license: "MIT".into(),
            icon: PathBuf::from("icon.svg"),
            tags: vec!["tag".into()],
            resource_type: rt,
            dependencies: Vec::<Dependency>::new(),
            signature: None,
            status: ValidationStatus::Incomplete,
        }
    }

    #[test]
    fn style_tokens_default_is_complete() {
        assert!(StyleTokens::default_tokens().is_complete());
    }

    #[test]
    fn style_resource_validates_ok_with_complete_tokens() {
        let mut r = StyleResource {
            meta: base_meta(ResourceType::Style),
            style: StyleTokens::default_tokens(),
        };
        r.validate();
        assert_eq!(r.meta.status, ValidationStatus::Ok);
    }

    #[test]
    fn style_resource_validates_broken_when_token_empty() {
        let mut tokens = StyleTokens::default_tokens();
        tokens.radius = String::new();
        let mut r = StyleResource {
            meta: base_meta(ResourceType::Style),
            style: tokens,
        };
        r.validate();
        assert_eq!(r.meta.status, ValidationStatus::Broken);
    }

    #[test]
    fn color_tokens_is_complete_with_all_values() {
        let tokens = ColorTokens {
            bg_base: "#000".into(), bg_surface: "#000".into(),
            bg_elevated: "#000".into(), bg_card: "#000".into(), bg_input: "#000".into(),
            text_primary: "#fff".into(), text_secondary: "#ccc".into(), text_muted: "#aaa".into(),
            primary: "#44f".into(), primary_hover: "#66f".into(), primary_text: "#fff".into(),
            accent: "#f90".into(), success: "#0f0".into(), warning: "#ff0".into(),
            error: "#f00".into(), border: "#333".into(), border_focus: "#44f".into(),
        };
        assert!(tokens.is_complete());
    }
}
