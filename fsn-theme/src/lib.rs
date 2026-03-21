// fsn-theme — Theme system for FreeSynergy.
//
// A `Theme` is loaded from a `theme.toml` file (or built in-memory).
// `ThemeEngine` converts it into CSS or Tailwind config for web/Dioxus rendering.
//
// Feature summary:
//   ThemeEngine::default()              — FreeSynergy Default (cyan/white on dark)
//   ThemeEngine::from_toml(path)        — Load from file
//   ThemeEngine::from_toml_str(text)    — Load from downloaded TOML string (store)
//   ThemeEngine::from_css(css, name)    — Parse CSS custom properties → Theme
//   engine.to_css()                     — Emit :root { --primary: … } block
//   engine.to_full_css()                — Emit :root + @media dark + @media contrast blocks
//   ThemeEngine::glass_css()            — Static glassmorphism utility CSS
//   ThemeEngine::animations_css()       — Static animation utility CSS
//   engine.to_tailwind_config()         — Emit Tailwind-compatible JSON extend block
//   ThemeRegistry                       — Manage + switch multiple themes

mod engine;
mod palette;
mod provider;
mod registry;
mod store;
mod theme;
mod types;

pub use engine::ThemeEngine;
pub use palette::ColorPalette;
pub use provider::{HighContrastTheme, SystemTheme, ThemeProvider, TomlTheme};
pub use registry::ThemeRegistry;
pub use store::{prefix_theme_css, validate_theme_vars, REQUIRED_VARS};
pub use theme::Theme;
pub use types::{Animation, AnimationKind, Glass, Shadows, ShadowLevel, Spacing, Typography};

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_has_correct_name() {
        let engine = ThemeEngine::default();
        assert_eq!(engine.theme().name, "FreeSynergy Default");
    }

    #[test]
    fn css_output_contains_custom_properties() {
        let css = ThemeEngine::default().to_css();
        assert!(css.contains("--primary:"));
        assert!(css.contains("--bg-base:"));
        assert!(css.contains("#00BCD4"), "primary cyan missing: {css}");
    }

    #[test]
    fn css_output_contains_typography_and_spacing() {
        let css = ThemeEngine::default().to_css();
        assert!(css.contains("--font-family:"));
        assert!(css.contains("--spacing-md:"));
    }

    #[test]
    fn css_output_no_duplicate_bg_surface() {
        let css = ThemeEngine::default().to_css();
        let count = css.matches("--bg-surface:").count();
        assert_eq!(count, 1, "--bg-surface must appear exactly once, found {count}");
    }

    #[test]
    fn css_output_contains_animation_vars() {
        let css = ThemeEngine::default().to_css();
        assert!(css.contains("--transition-fast:"));
        assert!(css.contains("--transition-base:"));
        assert!(css.contains("--transition-slow:"));
    }

    #[test]
    fn css_output_contains_shadow_vars() {
        let css = ThemeEngine::default().to_css();
        assert!(css.contains("--shadow-sm:"));
        assert!(css.contains("--shadow-md:"));
        assert!(css.contains("--shadow-lg:"));
        assert!(css.contains("--shadow-xl:"));
    }

    #[test]
    fn css_output_contains_glass_vars() {
        let css = ThemeEngine::default().to_css();
        assert!(css.contains("--glass-bg-opacity:"));
        assert!(css.contains("--glass-blur:"));
        assert!(css.contains("--glass-border-opacity:"));
    }

    #[test]
    fn default_palette_primary_is_cyan() {
        assert_eq!(ColorPalette::default().primary, "#00BCD4");
    }

    #[test]
    fn from_css_parses_custom_properties() {
        let css = ":root { --primary: #ff6600; --bg-base: #111111; --error: #cc0000; }";
        let engine = ThemeEngine::from_css(css, "Custom").unwrap();
        assert_eq!(engine.theme().colors.primary, "#ff6600");
        assert_eq!(engine.theme().colors.bg_base, "#111111");
        assert_eq!(engine.theme().colors.error, "#cc0000");
        // untouched values stay as default
        assert_eq!(engine.theme().colors.success, ColorPalette::default().success);
    }

    #[test]
    fn from_css_accepts_fsn_prefix_with_priority() {
        let css = ":root { --fsn-primary: #aabbcc; --primary: #ff0000; }";
        let engine = ThemeEngine::from_css(css, "FsnPrefixed").unwrap();
        // --fsn-* takes priority over plain --*
        assert_eq!(engine.theme().colors.primary, "#aabbcc");
    }

    #[test]
    fn to_tailwind_config_is_valid_json() {
        let json_str = ThemeEngine::default().to_tailwind_config();
        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .expect("to_tailwind_config must produce valid JSON");
        assert_eq!(parsed["colors"]["primary"], "#00BCD4");
        assert!(parsed["fontFamily"]["base"].is_array());
        assert!(parsed["spacing"]["md"].is_string());
    }

    #[test]
    fn from_css_roundtrip() {
        let original = ThemeEngine::default().to_css();
        let rt = ThemeEngine::from_css(&original, "Roundtrip").unwrap();
        assert_eq!(rt.theme().colors.primary, "#00BCD4");
        assert_eq!(rt.theme().colors.bg_base, ColorPalette::default().bg_base);
    }

    #[test]
    fn registry_default_has_freesynergy_theme() {
        let reg = ThemeRegistry::default();
        assert_eq!(reg.active().name, "FreeSynergy Default");
        assert!(reg.names().contains(&"FreeSynergy Default"));
    }

    #[test]
    fn registry_register_and_switch() {
        let mut reg = ThemeRegistry::default();
        reg.register(Theme { name: "Dark Amber".into(), ..Theme::default() });
        assert!(reg.names().contains(&"Dark Amber"));
        reg.set_active("Dark Amber").unwrap();
        assert_eq!(reg.active().name, "Dark Amber");
    }

    #[test]
    fn registry_cannot_remove_active() {
        let mut reg = ThemeRegistry::default();
        assert!(reg.remove("FreeSynergy Default").is_err());
    }

    #[test]
    fn registry_set_active_unknown_fails() {
        let mut reg = ThemeRegistry::default();
        assert!(reg.set_active("NonExistent").is_err());
    }

    #[test]
    fn registry_register_from_toml_str() {
        let toml = r##"
name    = "Test Theme"
version = "1.0.0"
[colors]
primary        = "#123456"
primary_hover  = "#234567"
primary_text   = "#000000"
secondary      = "#ffffff"
bg_base        = "#0a0a0a"
bg_surface     = "#1a1a1a"
bg_sidebar     = "#2a2a2a"
text_primary   = "#eeeeee"
text_secondary = "#aaaaaa"
text_muted     = "#555555"
success        = "#00ff00"
warning        = "#ffaa00"
error          = "#ff0000"
border_default = "#333333"
border_focus   = "#123456"
[typography]
font_family = "monospace"
font_size   = 14
line_height = 1.5
[spacing]
xs = 4
sm = 8
md = 16
lg = 24
xl = 32
"##;
        let mut reg = ThemeRegistry::default();
        let theme = reg.register_toml_str(toml).unwrap();
        assert_eq!(theme.name, "Test Theme");
        assert_eq!(theme.colors.primary, "#123456");
    }

    #[test]
    fn to_full_css_contains_root_and_contrast_blocks() {
        let full = ThemeEngine::default().to_full_css();
        assert!(full.contains(":root {"));
        assert!(full.contains("/* theme is dark by default */"));
        assert!(full.contains("@media (prefers-contrast: more)"));
        assert!(full.contains("--glass-bg-opacity:     1.0;"));
    }

    #[test]
    fn to_full_css_emits_dark_media_when_dark_colors_set() {
        let mut theme = Theme::default();
        theme.dark_colors = Some(ColorPalette {
            primary: "#ffffff".into(),
            ..ColorPalette::default()
        });
        let full = ThemeEngine::new(theme).to_full_css();
        assert!(full.contains("@media (prefers-color-scheme: dark)"));
        assert!(full.contains("#ffffff"));
    }

    #[test]
    fn glass_css_is_non_empty() {
        let css = ThemeEngine::glass_css();
        assert!(!css.is_empty());
        assert!(css.contains(".glass"));
    }

    #[test]
    fn animations_css_is_non_empty() {
        let css = ThemeEngine::animations_css();
        assert!(!css.is_empty());
        assert!(css.contains("fadeInUp"));
    }

    #[test]
    fn toml_theme_provider_returns_concrete_values() {
        let provider = TomlTheme(Theme::default());
        let glass = provider.glass();
        assert!(glass.starts_with("rgba("));
        let shadow = provider.shadow(ShadowLevel::Md);
        assert!(shadow.contains("rgba("));
        let anim = provider.animation(AnimationKind::Base);
        assert_eq!(anim, "250ms");
    }

    #[test]
    fn system_theme_provider_returns_var_references() {
        let provider = SystemTheme;
        assert_eq!(provider.shadow(ShadowLevel::Sm), "var(--shadow-sm)");
        assert_eq!(provider.animation(AnimationKind::Fast), "var(--transition-fast)");
    }

    #[test]
    fn high_contrast_theme_provider_returns_no_blur() {
        let provider = HighContrastTheme;
        assert_eq!(provider.glass(), "var(--bg-surface)");
        assert_eq!(provider.shadow(ShadowLevel::Xl), "none");
    }

    #[test]
    fn default_glass_values() {
        let g = Glass::default();
        assert_eq!(g.bg_opacity, 0.08);
        assert_eq!(g.blur, 12);
        assert_eq!(g.border_opacity, 0.15);
    }

    #[test]
    fn default_animation_values() {
        let a = Animation::default();
        assert_eq!(a.fast, 150);
        assert_eq!(a.base, 250);
        assert_eq!(a.slow, 400);
    }

    #[test]
    fn default_shadows_values() {
        let s = Shadows::default();
        assert!(s.sm.contains("rgba(0,0,0,0.4)"));
        assert!(s.xl.contains("rgba(0,0,0,0.7)"));
    }

    #[test]
    fn prefix_injection() {
        let css = ":root { --bg-base: #000; --text-primary: #fff; }";
        let out = prefix_theme_css(css, "fsn");
        assert!(out.contains("--fsn-bg-base: #000"), "got: {out}");
        assert!(out.contains("--fsn-text-primary: #fff"), "got: {out}");
    }

    #[test]
    fn no_double_prefix() {
        let css = ":root { --fsn-bg-base: #000; }";
        let out = prefix_theme_css(css, "fsn");
        assert!(!out.contains("--fsn-fsn-"), "got: {out}");
        assert!(out.contains("--fsn-bg-base: #000"), "got: {out}");
    }

    #[test]
    fn validate_missing_vars() {
        let css = "--bg-base: #0c1222; --text-primary: #e8edf5;";
        let missing = validate_theme_vars(css);
        assert!(missing.contains(&"bg-surface"), "expected bg-surface missing");
        assert!(!missing.contains(&"bg-base"), "bg-base should be present");
    }
}
