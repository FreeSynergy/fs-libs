// fsn-theme — ThemeEngine: converts a Theme into CSS or Tailwind config output.

use std::collections::HashMap;
use std::path::Path;

use fsn_error::FsnError;

use crate::theme::Theme;

/// Converts a [`Theme`] into CSS or Tailwind config output for Dioxus rendering.
pub struct ThemeEngine {
    pub(crate) theme: Theme,
}

impl ThemeEngine {
    /// Build from a parsed [`Theme`].
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// FreeSynergy Default theme (cyan/white on dark navy).
    pub fn default() -> Self {
        Self::new(Theme::default())
    }

    /// Load from a `theme.toml` file on disk.
    pub fn from_toml(path: &Path) -> Result<Self, FsnError> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| FsnError::Config(format!("cannot read theme {}: {e}", path.display())))?;
        Self::from_toml_str(&text)
    }

    /// Parse a TOML string (e.g. downloaded from the store).
    pub fn from_toml_str(text: &str) -> Result<Self, FsnError> {
        let theme: Theme = toml::from_str(text)
            .map_err(|e| FsnError::Parse(format!("theme TOML: {e}")))?;
        Ok(Self::new(theme))
    }

    /// Parse a CSS block containing `--*` or `--fsn-*` custom properties.
    ///
    /// Starts from the FreeSynergy Default theme and overlays any recognized vars.
    /// Both `--primary` and `--fsn-primary` are accepted (`--fsn-*` takes priority).
    ///
    /// ```
    /// let css = ":root { --primary: #ff6600; --bg-base: #000000; }";
    /// let engine = fsn_theme::ThemeEngine::from_css(css, "Custom").unwrap();
    /// assert_eq!(engine.theme().colors.primary, "#ff6600");
    /// ```
    pub fn from_css(css: &str, name: &str) -> Result<Self, FsnError> {
        // Split by `;` so both multi-line and single-line CSS blocks are handled.
        let mut plain_vars: HashMap<String, String> = HashMap::new();
        let mut fsn_vars:   HashMap<String, String> = HashMap::new();

        for chunk in css.split(';') {
            let Some(dash_pos) = chunk.find("--") else { continue };
            let decl = &chunk[dash_pos..];
            let Some(colon) = decl.find(':') else { continue };
            let var   = decl[..colon].trim();
            let value = decl[colon + 1..].trim().to_string();
            if value.is_empty() || !var.starts_with("--") { continue; }

            if var.starts_with("--fsn-") {
                fsn_vars.insert(format!("--{}", &var["--fsn-".len()..]), value);
            } else {
                plain_vars.insert(var.to_string(), value);
            }
        }

        let mut theme = Theme::default();
        theme.name = name.to_string();

        let plain_refs: HashMap<&str, &str> =
            plain_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        theme.colors.apply_css_vars(&plain_refs);

        // --fsn-* overrides plain vars
        let fsn_refs: HashMap<&str, &str> =
            fsn_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        theme.colors.apply_css_vars(&fsn_refs);

        Ok(Self::new(theme))
    }

    /// Access the underlying theme definition.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Emit the theme as CSS custom properties.
    ///
    /// Output: `:root { --primary: #00bcd4; … }`
    /// Includes colors, typography, spacing, animation, shadow, and glass vars.
    pub fn to_css(&self) -> String {
        let c  = &self.theme.colors;
        let t  = &self.theme.typography;
        let s  = &self.theme.spacing;
        let a  = &self.theme.animation;
        let sh = &self.theme.shadows;
        let g  = &self.theme.glass;
        format!(
            ":root {{\n\
            {colors}\n\
            \x20 --font-family:    {font_family};\n\
            \x20 --font-size:      {font_size}px;\n\
            \x20 --line-height:    {line_height};\n\
            \x20 --spacing-xs:     {xs}px;\n\
            \x20 --spacing-sm:     {sm}px;\n\
            \x20 --spacing-md:     {md}px;\n\
            \x20 --spacing-lg:     {lg}px;\n\
            \x20 --spacing-xl:     {xl}px;\n\
            \x20 --transition-fast: {afast}ms;\n\
            \x20 --transition-base: {abase}ms;\n\
            \x20 --transition-slow: {aslow}ms;\n\
            \x20 --shadow-sm:      {shadow_sm};\n\
            \x20 --shadow-md:      {shadow_md};\n\
            \x20 --shadow-lg:      {shadow_lg};\n\
            \x20 --shadow-xl:      {shadow_xl};\n\
            \x20 --glass-bg-opacity:     {glass_bg_opacity};\n\
            \x20 --glass-blur:           {glass_blur}px;\n\
            \x20 --glass-border-opacity: {glass_border_opacity};\n\
            }}",
            colors         = c.to_css_vars(),
            font_family    = t.font_family,
            font_size      = t.font_size,
            line_height    = t.line_height,
            xs = s.xs, sm = s.sm, md = s.md, lg = s.lg, xl = s.xl,
            afast = a.fast, abase = a.base, aslow = a.slow,
            shadow_sm = sh.sm, shadow_md = sh.md, shadow_lg = sh.lg, shadow_xl = sh.xl,
            glass_bg_opacity     = g.bg_opacity,
            glass_blur           = g.blur,
            glass_border_opacity = g.border_opacity,
        )
    }

    /// Emit the full CSS including `:root`, `@media (prefers-color-scheme: dark)`,
    /// and `@media (prefers-contrast: more)` blocks.
    pub fn to_full_css(&self) -> String {
        let root = self.to_css();

        let dark_block = match &self.theme.dark_colors {
            Some(dark) => format!(
                "@media (prefers-color-scheme: dark) {{\n  :root {{\n{}\n  }}\n}}",
                dark.to_css_vars()
            ),
            None => "/* theme is dark by default */".to_string(),
        };

        let contrast_block = format!(
            "@media (prefers-contrast: more) {{\n\
             \x20 :root {{\n\
             \x20   --glass-bg-opacity:     1.0;\n\
             \x20   --glass-blur:           0px;\n\
             \x20   --glass-border-opacity: 1.0;\n\
             \x20 }}\n\
             }}"
        );

        format!("{root}\n\n{dark_block}\n\n{contrast_block}")
    }

    /// The embedded glassmorphism utility CSS (from `assets/glass.css`).
    ///
    /// Include this in your Dioxus `document::Style` or `<style>` element alongside
    /// the output of `to_css()` to get the `.glass`, `.glass-card`, `.glass-sidebar` classes.
    pub fn glass_css() -> &'static str {
        include_str!("../assets/glass.css")
    }

    /// The embedded animation utility CSS (from `assets/animations.css`).
    ///
    /// Include this in your Dioxus `document::Style` or `<style>` element to get
    /// `.fsd-fade-in-up`, `.fsd-slide-in-right`, and `.fsd-skeleton` classes.
    pub fn animations_css() -> &'static str {
        include_str!("../assets/animations.css")
    }

    /// Emit a Tailwind CSS `theme.extend` JSON block.
    ///
    /// Paste the output into your `tailwind.config.js`:
    /// ```js
    /// module.exports = { theme: { extend: /* paste here */ } }
    /// ```
    pub fn to_tailwind_config(&self) -> String {
        let c = &self.theme.colors;
        let t = &self.theme.typography;
        let s = &self.theme.spacing;

        // Build fontFamily as a JSON array, splitting on comma so each font
        // gets its own array entry. Quotes and whitespace are stripped per entry.
        let font_array: Vec<String> = t.font_family
            .split(',')
            .map(|f| {
                let clean = f.trim().trim_matches('"').trim();
                format!("\"{clean}\"")
            })
            .collect();
        let font_json = font_array.join(", ");

        format!(
            "{{\n\
            \x20 \"colors\": {{\n\
            \x20   {colors}\n\
            \x20 }},\n\
            \x20 \"fontFamily\": {{\n\
            \x20   \"base\": [{font_json}]\n\
            \x20 }},\n\
            \x20 \"fontSize\": {{\n\
            \x20   \"base\": \"{font_size}px\"\n\
            \x20 }},\n\
            \x20 \"spacing\": {{\n\
            \x20   \"xs\": \"{xs}px\",\n\
            \x20   \"sm\": \"{sm}px\",\n\
            \x20   \"md\": \"{md}px\",\n\
            \x20   \"lg\": \"{lg}px\",\n\
            \x20   \"xl\": \"{xl}px\"\n\
            \x20 }}\n\
            }}",
            colors    = c.to_tailwind_colors(),
            font_json = font_json,
            font_size = t.font_size,
            xs = s.xs, sm = s.sm, md = s.md, lg = s.lg, xl = s.xl,
        )
    }
}
