// fs-theme — ThemeProvider trait and its three built-in implementations.

use crate::theme::Theme;
use crate::types::{AnimationKind, ShadowLevel};

// ── ThemeProvider trait ───────────────────────────────────────────────────────

/// Trait for types that can provide resolved theme values for use in components.
///
/// Three built-in implementors:
/// - [`TomlTheme`]           — returns concrete values from a loaded `Theme`
/// - [`SystemTheme`]         — returns CSS `var()` references (for runtime-theming)
/// - [`HighContrastTheme`]   — accessibility variant (no blur, strong borders)
pub trait ThemeProvider {
    /// CSS value for the glassmorphism background, e.g. `"rgba(22,27,34,0.08)"`.
    fn glass(&self) -> String;
    /// CSS `box-shadow` value for the given elevation level.
    fn shadow(&self, level: ShadowLevel) -> String;
    /// CSS `animation-duration` value for the given timing category, e.g. `"250ms"`.
    fn animation(&self, kind: AnimationKind) -> String;
}

// ── TomlTheme ─────────────────────────────────────────────────────────────────

/// A [`ThemeProvider`] backed by a loaded [`Theme`] — returns concrete values.
pub struct TomlTheme(pub Theme);

impl ThemeProvider for TomlTheme {
    fn glass(&self) -> String {
        format!("rgba(22,27,34,{})", self.0.glass.bg_opacity)
    }

    fn shadow(&self, level: ShadowLevel) -> String {
        match level {
            ShadowLevel::Sm => self.0.shadows.sm.clone(),
            ShadowLevel::Md => self.0.shadows.md.clone(),
            ShadowLevel::Lg => self.0.shadows.lg.clone(),
            ShadowLevel::Xl => self.0.shadows.xl.clone(),
        }
    }

    fn animation(&self, kind: AnimationKind) -> String {
        match kind {
            AnimationKind::Fast => format!("{}ms", self.0.animation.fast),
            AnimationKind::Base => format!("{}ms", self.0.animation.base),
            AnimationKind::Slow => format!("{}ms", self.0.animation.slow),
        }
    }
}

// ── SystemTheme ───────────────────────────────────────────────────────────────

/// A [`ThemeProvider`] that returns CSS `var()` references for runtime theming.
pub struct SystemTheme;

impl ThemeProvider for SystemTheme {
    fn glass(&self) -> String {
        "rgba(22,27,34,var(--glass-bg-opacity,0.08))".into()
    }

    fn shadow(&self, level: ShadowLevel) -> String {
        format!("var({})", level.css_var())
    }

    fn animation(&self, kind: AnimationKind) -> String {
        format!("var({})", kind.css_var())
    }
}

// ── HighContrastTheme ─────────────────────────────────────────────────────────

/// A [`ThemeProvider`] for accessibility: no blur, strong borders, normal durations.
pub struct HighContrastTheme;

impl ThemeProvider for HighContrastTheme {
    fn glass(&self) -> String {
        "var(--bg-surface)".into()
    }

    fn shadow(&self, _level: ShadowLevel) -> String {
        "none".into()
    }

    fn animation(&self, kind: AnimationKind) -> String {
        // Keep normal durations; the caller should honour `prefers-reduced-motion` separately.
        format!("{}ms", kind.default_ms())
    }
}
