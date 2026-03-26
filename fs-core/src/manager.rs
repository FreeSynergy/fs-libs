// FsManager / SelectableManager — common interfaces for all FreeSynergy managers.
//
// Every manager (Theme, Language, Container, Icons, Cursor, …) implements
// `FsManager` so callers can handle them uniformly in health dashboards,
// the Node API, and the Desktop.
//
// Managers that expose a user-selectable list (Theme, Language, Cursor) also
// implement `SelectableManager<Item>`, which gives the Desktop a single generic
// interface for building picker UIs without duplicating panel logic.

/// Common interface for all FreeSynergy managers.
///
/// Provides identity and health information without coupling callers to
/// any concrete manager type.
pub trait FsManager {
    /// Stable identifier used in configs and APIs (e.g. `"theme"`, `"icons"`).
    fn id(&self) -> &str;

    /// Human-readable name for display (e.g. `"Theme Manager"`).
    fn name(&self) -> &str;

    /// Returns `true` if the manager is fully operational.
    ///
    /// Default returns `true` — override for managers that perform
    /// real health checks (e.g. checking a running process or reachable DB).
    fn is_healthy(&self) -> bool {
        true
    }
}

// ── SelectableManager ─────────────────────────────────────────────────────────

/// Managers that expose a selectable list of items (active + available + set_active).
///
/// Implemented by `ThemeManager`, `LanguageManager`, and `CursorManager`.
/// The Desktop uses this trait to build the generic `PickerPanel` component
/// without duplicating panel logic per manager type.
///
/// # Example
///
/// ```rust,ignore
/// let mgr = ThemeManager::new();
/// let active: Theme      = mgr.active();
/// let all:    Vec<Theme> = mgr.available();
/// mgr.set_active("fs-dark")?;
/// ```
pub trait SelectableManager {
    /// The item type this manager selects over (e.g. `Theme`, `Language`).
    type Item;

    /// The error type returned by [`set_active`](Self::set_active).
    type Error: std::error::Error;

    /// The currently active item.
    fn active(&self) -> Self::Item;

    /// All available items in display order.
    fn available(&self) -> Vec<Self::Item>;

    /// Persist `id` as the new active item.
    fn set_active(&self, id: &str) -> Result<(), Self::Error>;
}
