// disk_cache.rs — Persistent on-disk cache for store responses.
//
// Used as fallback when HTTP fetches fail (offline mode).
// Stores raw TOML text under ~/.cache/fsn/store/{hash}.toml.
//
// Design:
//   - Cache key = hex SHA-256 of the URL/path
//   - Cache files are plain text (TOML/raw)
//   - No TTL on disk — disk cache is only used when HTTP fails

use std::path::PathBuf;

use tracing::{debug, warn};

// ── DiskCache ─────────────────────────────────────────────────────────────────

/// Persistent on-disk cache for store content.
///
/// Used as offline fallback: when an HTTP fetch fails, the last successful
/// response is returned from disk instead of propagating the error.
pub struct DiskCache {
    root: PathBuf,
}

impl DiskCache {
    /// Create a disk cache rooted at `dir`.
    ///
    /// The directory is created on first use if it does not exist.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Default cache location: `~/.cache/fsn/store/`.
    pub fn default_location() -> Self {
        let root = dirs_or_home().join("fsn").join("store");
        Self::new(root)
    }

    /// Return the cached content for `key`, or `None` if not cached.
    pub fn get(&self, key: &str) -> Option<String> {
        let path = self.path_for(key);
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                debug!(key, path = %path.display(), "disk cache hit");
                Some(content)
            }
            Err(_) => None,
        }
    }

    /// Persist `content` under `key`.
    ///
    /// Errors are logged but not propagated — caching is best-effort.
    pub fn insert(&self, key: &str, content: &str) {
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                warn!(path = %parent.display(), "disk cache: failed to create dir: {e}");
                return;
            }
        }
        if let Err(e) = std::fs::write(&path, content) {
            warn!(path = %path.display(), "disk cache: failed to write: {e}");
        } else {
            debug!(key, path = %path.display(), "disk cache: stored");
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn path_for(&self, key: &str) -> PathBuf {
        // Use a simple hash to avoid filesystem-unsafe characters in the key
        let hash = simple_hash(key);
        self.root.join(format!("{hash}.toml"))
    }
}

/// Minimal non-crypto hash — sufficient for cache key discrimination.
fn simple_hash(s: &str) -> String {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// Return `~/.cache` or fallback to `/tmp`.
fn dirs_or_home() -> PathBuf {
    // XDG_CACHE_HOME → ~/.cache → /tmp
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".cache");
    }
    PathBuf::from("/tmp")
}
