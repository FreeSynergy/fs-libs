// ManagerError — shared error type for all FreeSynergy managers.
//
// All managers (Theme, Language, Container, …) return this type so callers
// can match a single, well-known set of variants instead of per-manager enums.

/// Error type returned by all FreeSynergy managers.
#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    /// A resource (theme, language, app, …) was not found by the given ID.
    #[error("{0} not found")]
    NotFound(String),

    /// The resource is already installed / registered.
    #[error("{0} already exists")]
    AlreadyExists(String),

    /// The caller lacks permission to perform this operation.
    #[error("permission denied")]
    PermissionDenied,

    /// The persistent Store could not be read or written.
    #[error("store error: {0}")]
    StoreError(String),

    /// A lower-level runtime failure (process, I/O, …).
    #[error("runtime error: {0}")]
    Runtime(String),
}
