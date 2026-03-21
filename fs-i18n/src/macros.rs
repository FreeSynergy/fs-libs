//! Convenience macros for the i18n system.

/// Build a translation-arguments slice inline.
///
/// Returns `&[(&str, &str)]` suitable for passing to
/// [`I18n::t_with`](crate::I18n::t_with) or the free [`t_with`](crate::t_with)
/// function.
///
/// # Example
/// ```rust,ignore
/// use fs_i18n::{args, t_with};
///
/// let result = t_with("phrase-confirm-delete", args![("item", "module")]);
/// ```
#[macro_export]
macro_rules! args {
    ( $( ($k:expr, $v:expr) ),* $(,)? ) => {
        &[ $( ($k, $v) ),* ]
    };
}
