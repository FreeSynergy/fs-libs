use crate::severity::ErrorSeverity;

/// Trait for all `FreeSynergy` error types.
///
/// Every error domain (config, network, auth, …) should implement this trait
/// so that generic code can inspect errors uniformly — for logging, telemetry,
/// or user-facing i18n without pattern-matching on concrete enum variants.
///
/// # Implementing
///
/// ```rust
/// use fs_error::{FsErrorTrait, ErrorSeverity};
///
/// #[derive(Debug)]
/// struct MyError(String);
///
/// impl std::fmt::Display for MyError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "{}", self.0)
///     }
/// }
/// impl std::error::Error for MyError {}
///
/// impl FsErrorTrait for MyError {
///     fn code(&self) -> &'static str { "my_error" }
///     fn ftl_key(&self) -> &'static str { "errors.my_error" }
///     fn severity(&self) -> ErrorSeverity { ErrorSeverity::Error }
/// }
/// ```
pub trait FsErrorTrait: std::error::Error {
    /// Machine-readable error code — a short, stable `snake_case` identifier.
    ///
    /// Used for structured logging, metrics labels, and API error bodies.
    /// Example: `"config"`, `"not_found"`, `"auth"`.
    fn code(&self) -> &'static str;

    /// i18n snippet key used to look up the user-facing message via `fs_i18n`.
    ///
    /// The key follows the `"section.name"` convention, e.g. `"errors.config_error"`.
    /// Never display [`Display`](std::fmt::Display) output to end users — that is
    /// for developer logs only. Call `i18n.t(error.ftl_key())` instead.
    fn ftl_key(&self) -> &'static str;

    /// How serious this error is.
    fn severity(&self) -> ErrorSeverity;

    /// The underlying cause of this error, if any.
    ///
    /// Defaults to [`std::error::Error::source`] — implementors only need to
    /// override this if they store the cause differently.
    fn cause(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source()
    }
}
