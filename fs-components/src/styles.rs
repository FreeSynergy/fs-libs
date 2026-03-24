// fs-components/styles.rs — Shared styling traits for UI variants.
//
// `VariantStyle` is the common interface for all visual-state enums
// (ButtonVariant, BadgeVariant, …). Implementing it lets generic helpers
// and future tooling handle all variants uniformly without knowing the
// concrete type.

// ── VariantStyle ──────────────────────────────────────────────────────────────

/// Shared interface for visual-state variant enums.
///
/// Implement this trait on any enum whose variants map to distinct inline CSS
/// styles (e.g. `ButtonVariant`, `BadgeVariant`).
///
/// # Example
///
/// ```rust,ignore
/// use fs_components::styles::VariantStyle;
///
/// impl VariantStyle for MyVariant {
///     fn css(&self) -> &'static str {
///         match self {
///             Self::Primary => "background: cyan; color: black;",
///             Self::Danger  => "background: red;  color: white;",
///         }
///     }
/// }
/// ```
pub trait VariantStyle {
    /// Inline CSS string for this variant (background, color, border, …).
    fn css(&self) -> &'static str;

    /// CSS class suffix used for `class="fs-<component>--<suffix>"`.
    ///
    /// Returns an empty string by default; override when the component uses
    /// BEM-style class names for theming or test selection.
    fn class_suffix(&self) -> &'static str {
        ""
    }
}
