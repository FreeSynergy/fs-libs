// fs-theme — ColorPalette: all color fields + CSS/Tailwind emission.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Full color specification for a theme, in `#RRGGBB` hex format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorPalette {
    pub primary: String,
    pub primary_hover: String,
    pub primary_text: String,
    pub secondary: String,
    pub bg_base: String,
    pub bg_surface: String,
    pub bg_sidebar: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub text_muted: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub border_default: String,
    pub border_focus: String,
}

impl Default for ColorPalette {
    fn default() -> Self {
        // FreeSynergy house style: cyan primary on dark navy.
        Self {
            primary: "#00BCD4".into(),
            primary_hover: "#26C6DA".into(),
            primary_text: "#000000".into(),
            secondary: "#ffffff".into(),
            bg_base: "#0d1117".into(),
            bg_surface: "#161b22".into(),
            bg_sidebar: "#0f3460".into(),
            text_primary: "#e6edf3".into(),
            text_secondary: "#8b949e".into(),
            text_muted: "#484f58".into(),
            success: "#3fb950".into(),
            warning: "#d29922".into(),
            error: "#f85149".into(),
            border_default: "#30363d".into(),
            border_focus: "#00BCD4".into(),
        }
    }
}

// CSS var name → field setter function mapping.
// Using fn pointers avoids a secondary match block on field name strings.
type ColorSetter = fn(&mut ColorPalette, String);
const CSS_VAR_SETTERS: &[(&str, ColorSetter)] = &[
    ("--primary", |p, v| p.primary = v),
    ("--primary-hover", |p, v| p.primary_hover = v),
    ("--primary-text", |p, v| p.primary_text = v),
    ("--secondary", |p, v| p.secondary = v),
    ("--bg-base", |p, v| p.bg_base = v),
    ("--bg-surface", |p, v| p.bg_surface = v),
    ("--bg-sidebar", |p, v| p.bg_sidebar = v),
    ("--text-primary", |p, v| p.text_primary = v),
    ("--text-secondary", |p, v| p.text_secondary = v),
    ("--text-muted", |p, v| p.text_muted = v),
    ("--success", |p, v| p.success = v),
    ("--warning", |p, v| p.warning = v),
    ("--error", |p, v| p.error = v),
    ("--border-default", |p, v| p.border_default = v),
    ("--border-focus", |p, v| p.border_focus = v),
];

impl ColorPalette {
    /// Apply a map of CSS custom property declarations to this palette.
    pub(crate) fn apply_css_vars(&mut self, vars: &HashMap<&str, &str>) {
        for (css_var, setter) in CSS_VAR_SETTERS {
            if let Some(val) = vars.get(css_var) {
                setter(self, val.to_string());
            }
        }
    }

    /// Emit all color palette fields as CSS custom property declarations (no `:root` wrapper).
    pub(crate) fn to_css_vars(&self) -> String {
        format!(
            "  --primary:        {primary};\n\
             \x20 --primary-hover:  {primary_hover};\n\
             \x20 --primary-text:   {primary_text};\n\
             \x20 --secondary:      {secondary};\n\
             \x20 --bg-base:        {bg_base};\n\
             \x20 --bg-surface:     {bg_surface};\n\
             \x20 --bg-sidebar:     {bg_sidebar};\n\
             \x20 --text-primary:   {text_primary};\n\
             \x20 --text-secondary: {text_secondary};\n\
             \x20 --text-muted:     {text_muted};\n\
             \x20 --success:        {success};\n\
             \x20 --warning:        {warning};\n\
             \x20 --error:          {error};\n\
             \x20 --border-default: {border_default};\n\
             \x20 --border-focus:   {border_focus};",
            primary = self.primary,
            primary_hover = self.primary_hover,
            primary_text = self.primary_text,
            secondary = self.secondary,
            bg_base = self.bg_base,
            bg_surface = self.bg_surface,
            bg_sidebar = self.bg_sidebar,
            text_primary = self.text_primary,
            text_secondary = self.text_secondary,
            text_muted = self.text_muted,
            success = self.success,
            warning = self.warning,
            error = self.error,
            border_default = self.border_default,
            border_focus = self.border_focus,
        )
    }

    /// Emit the color palette as a JSON object for Tailwind's `theme.extend.colors`.
    pub(crate) fn to_tailwind_colors(&self) -> String {
        format!(
            "\"primary\":        \"{primary}\",\n\
             \x20   \"primary-hover\":  \"{primary_hover}\",\n\
             \x20   \"primary-text\":   \"{primary_text}\",\n\
             \x20   \"secondary\":      \"{secondary}\",\n\
             \x20   \"bg-base\":        \"{bg_base}\",\n\
             \x20   \"bg-surface\":     \"{bg_surface}\",\n\
             \x20   \"bg-sidebar\":     \"{bg_sidebar}\",\n\
             \x20   \"text-primary\":   \"{text_primary}\",\n\
             \x20   \"text-secondary\": \"{text_secondary}\",\n\
             \x20   \"text-muted\":     \"{text_muted}\",\n\
             \x20   \"success\":        \"{success}\",\n\
             \x20   \"warning\":        \"{warning}\",\n\
             \x20   \"error\":          \"{error}\",\n\
             \x20   \"border\":         \"{border_default}\",\n\
             \x20   \"border-focus\":   \"{border_focus}\"",
            primary = self.primary,
            primary_hover = self.primary_hover,
            primary_text = self.primary_text,
            secondary = self.secondary,
            bg_base = self.bg_base,
            bg_surface = self.bg_surface,
            bg_sidebar = self.bg_sidebar,
            text_primary = self.text_primary,
            text_secondary = self.text_secondary,
            text_muted = self.text_muted,
            success = self.success,
            warning = self.warning,
            error = self.error,
            border_default = self.border_default,
            border_focus = self.border_focus,
        )
    }
}
