// store.rs — ManagerStore: minimal storage interface for FreeSynergy managers.
//
// Managers (Theme, Container, …) need to persist and read settings without
// being coupled to the concrete StoreClient (fs-store).  This trait is the
// narrow DI port they actually depend on.
//
// Two built-in implementations:
//   NoopStore  — in-memory no-op (tests, offline, default init)
//   Arc<T>     — blanket impl forwards to the inner store
//
// The real implementation (backed by fs-store or SQLite) lives in the binary
// crates and is injected at startup via Manager::new(store).

use std::sync::Arc;

use crate::error::ManagerError;

// ── ManagerStore ──────────────────────────────────────────────────────────────

/// Minimal persistent storage interface required by FreeSynergy managers.
///
/// Managers read and write typed settings through this trait.  Implementations
/// may back it with SQLite, a TOML file, the FreeSynergy Store, or a no-op
/// (for tests and offline usage).
///
/// # Example
///
/// ```rust,ignore
/// use fs_core::store::{ManagerStore, NoopStore};
/// use std::sync::Arc;
///
/// let store: Arc<dyn ManagerStore> = Arc::new(NoopStore);
/// let mgr = ThemeManager::new(Arc::clone(&store));
/// ```
pub trait ManagerStore: Send + Sync {
    /// Read a setting value by key (e.g. `"theme.active"`).
    ///
    /// Returns `None` when the key has not been set.
    fn read_setting(&self, key: &str) -> Option<String>;

    /// Persist a setting value.
    ///
    /// Returns `ManagerError::StoreError` when the write fails.
    fn write_setting(&self, key: &str, value: &str) -> Result<(), ManagerError>;
}

// Blanket impl: `Arc<T>` where `T: ManagerStore` forwards transparently.
impl<T: ManagerStore + ?Sized> ManagerStore for Arc<T> {
    fn read_setting(&self, key: &str) -> Option<String> {
        self.as_ref().read_setting(key)
    }

    fn write_setting(&self, key: &str, value: &str) -> Result<(), ManagerError> {
        self.as_ref().write_setting(key, value)
    }
}

// ── NoopStore ─────────────────────────────────────────────────────────────────

/// No-op store — always returns `None` for reads, silently ignores writes.
///
/// Use in tests, offline builds, or before a real store is wired up.
pub struct NoopStore;

impl ManagerStore for NoopStore {
    fn read_setting(&self, _key: &str) -> Option<String> {
        None
    }

    fn write_setting(&self, _key: &str, _value: &str) -> Result<(), ManagerError> {
        Ok(())
    }
}
