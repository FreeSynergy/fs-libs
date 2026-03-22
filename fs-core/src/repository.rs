// Repository trait and RepositoryManager — shared abstraction for managing
// configurable source repositories across all FreeSynergy programs.
//
// Programs that have user-configurable sources (Store, Icon Manager, Bundle
// Manager) all use the same pattern: a list of repositories with
// builtin-protection rules. Rather than duplicating the logic, each program
// provides its own concrete type that implements `Repository` and uses
// `RepositoryManager<R>` for all operations.

// ── Repository trait ──────────────────────────────────────────────────────────

/// A source repository that can be enabled, disabled, or — if not builtin —
/// removed entirely.
///
/// Implement this trait on your program-specific repository type and hand it
/// to [`RepositoryManager`].
pub trait Repository {
    /// Stable, unique identifier for this repository.
    fn id(&self) -> &str;

    /// Builtin repositories ship with FreeSynergy and cannot be removed;
    /// only disabled.
    fn builtin(&self) -> bool;

    /// Whether this repository is currently active.
    fn enabled(&self) -> bool;

    /// Activate or deactivate this repository.
    fn set_enabled(&mut self, enabled: bool);
}

// ── RepositoryError ───────────────────────────────────────────────────────────

/// Errors returned by [`RepositoryManager`] operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepositoryError {
    /// No repository with the given ID exists.
    #[error("Repository not found: {0}")]
    NotFound(String),
    /// The repository exists but is marked builtin and cannot be removed.
    #[error("Cannot remove builtin repository: {0}")]
    CannotRemoveBuiltin(String),
}

// ── RepositoryManager ─────────────────────────────────────────────────────────

/// Manages a list of repositories with builtin-protection rules.
///
/// Instantiate with your program-specific repository type `R`:
///
/// ```rust,ignore
/// let mgr: RepositoryManager<IconRepository> =
///     RepositoryManager::new(vec![/* … */]);
/// ```
///
/// ## Rules enforced for every program
///
/// - Any repository can be enabled or disabled at any time.
/// - Repositories where [`Repository::builtin`] returns `true` cannot be
///   removed — only disabled. [`remove`](Self::remove) returns
///   [`RepositoryError::CannotRemoveBuiltin`] in that case.
/// - Non-builtin repositories can be freely added and removed.
pub struct RepositoryManager<R: Repository> {
    repositories: Vec<R>,
}

impl<R: Repository> RepositoryManager<R> {
    /// Creates a new manager with the given initial list of repositories.
    pub fn new(repositories: Vec<R>) -> Self {
        Self { repositories }
    }

    /// Returns all repositories (enabled and disabled).
    pub fn list(&self) -> &[R] {
        &self.repositories
    }

    /// Returns an iterator over currently enabled repositories only.
    pub fn enabled(&self) -> impl Iterator<Item = &R> {
        self.repositories.iter().filter(|r| r.enabled())
    }

    /// Looks up a repository by ID.
    pub fn get(&self, id: &str) -> Option<&R> {
        self.repositories.iter().find(|r| r.id() == id)
    }

    /// Appends a new repository.
    pub fn add(&mut self, repo: R) {
        self.repositories.push(repo);
    }

    /// Removes a repository by ID.
    ///
    /// Returns [`RepositoryError::CannotRemoveBuiltin`] if the repository is
    /// builtin; returns [`RepositoryError::NotFound`] if no such ID exists.
    pub fn remove(&mut self, id: &str) -> Result<(), RepositoryError> {
        let pos = self
            .repositories
            .iter()
            .position(|r| r.id() == id)
            .ok_or_else(|| RepositoryError::NotFound(id.into()))?;

        if self.repositories[pos].builtin() {
            return Err(RepositoryError::CannotRemoveBuiltin(id.into()));
        }

        self.repositories.remove(pos);
        Ok(())
    }

    /// Enables or disables a repository by ID.
    ///
    /// Returns [`RepositoryError::NotFound`] if no repository with that ID exists.
    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<(), RepositoryError> {
        let repo = self
            .repositories
            .iter_mut()
            .find(|r| r.id() == id)
            .ok_or_else(|| RepositoryError::NotFound(id.into()))?;
        repo.set_enabled(enabled);
        Ok(())
    }
}
