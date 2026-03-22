// fs-types/src/label.rs — StrLabel trait + blanket Display.
//
// All FreeSynergy enums that have a human-readable label implement `StrLabel`.
// The blanket `impl Display` removes the boilerplate of repeating
//
//   impl std::fmt::Display for Foo {
//       fn fmt(&self, f: &mut ...) -> ... { f.write_str(self.label()) }
//   }
//
// across every enum. Add `impl StrLabel for T` and Display comes for free.

// ── StrLabel ──────────────────────────────────────────────────────────────────

/// Human-readable label for enum variants.
///
/// Implement this trait on any enum whose variants have a canonical display
/// string (e.g. `"Online"`, `"stable"`, `"fire-and-forget"`).
///
/// # Provides
///
/// - `Display` — via the blanket impl below; no manual `impl Display` needed.
///
/// # Example
///
/// ```rust
/// use fs_types::StrLabel;
///
/// enum Color { Red, Blue }
///
/// impl StrLabel for Color {
///     fn label(&self) -> &'static str {
///         match self { Self::Red => "Red", Self::Blue => "Blue" }
///     }
/// }
///
/// fs_types::impl_str_label_display!(Color);
///
/// assert_eq!(format!("{}", Color::Red), "Red");
/// ```
pub trait StrLabel {
    /// The canonical human-readable string for this variant.
    fn label(&self) -> &'static str;
}

// ── impl_str_label_display! ───────────────────────────────────────────────────

/// Generates `impl Display` for every listed type by delegating to [`StrLabel::label`].
///
/// Use this in the same crate that defines the type, after implementing `StrLabel`:
///
/// ```rust,ignore
/// use fs_types::{StrLabel, impl_str_label_display};
///
/// impl StrLabel for MyEnum {
///     fn label(&self) -> &'static str { match self { ... } }
/// }
///
/// impl_str_label_display!(MyEnum);
/// ```
#[macro_export]
macro_rules! impl_str_label_display {
    ($($T:ty),+ $(,)?) => {
        $(
            impl ::std::fmt::Display for $T {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    ::std::fmt::Formatter::write_str(f, $crate::StrLabel::label(self))
                }
            }
        )+
    };
}
