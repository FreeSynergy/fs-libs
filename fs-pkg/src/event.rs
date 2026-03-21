// event.rs — Install/remove event bus.
//
// Observer pattern: components register hooks that are called when a package
// is installed or removed. Hooks receive an `InstallEvent` and can abort the
// operation by returning an `Err`.
//
// Design:
//   InstallEvent — the event payload (package id + kind + optional context)
//   InstallHook  — a boxed callback (Fn(&InstallEvent) -> Result<()>)
//   EventBus     — stores hooks and dispatches events

use fs_error::FsError;
use serde::{Deserialize, Serialize};

// ── InstallEvent ──────────────────────────────────────────────────────────────

/// The kind of install lifecycle event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallEventKind {
    /// A package install was started (before pre_install hook).
    InstallStarted,
    /// A package install completed successfully.
    InstallCompleted,
    /// A package install failed.
    InstallFailed,
    /// A package remove was started (before pre_remove hook).
    RemoveStarted,
    /// A package remove completed successfully.
    RemoveCompleted,
    /// A package upgrade was started.
    UpgradeStarted,
    /// A package upgrade completed successfully.
    UpgradeCompleted,
}

/// Payload for a package lifecycle event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallEvent {
    /// The package ID (e.g. `"proxy/zentinel"`).
    pub package_id: String,

    /// The package version being operated on.
    pub version: String,

    /// The event kind.
    pub kind: InstallEventKind,

    /// Optional human-readable message (e.g. error description on failure).
    #[serde(default)]
    pub message: Option<String>,
}

impl InstallEvent {
    /// Create a new event.
    pub fn new(
        package_id: impl Into<String>,
        version: impl Into<String>,
        kind: InstallEventKind,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            version: version.into(),
            kind,
            message: None,
        }
    }

    /// Attach a message to the event.
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
}

// ── InstallHook ───────────────────────────────────────────────────────────────

/// A named hook that runs in response to an [`InstallEvent`].
///
/// Return `Ok(())` to continue the operation; return `Err(…)` to abort.
pub type InstallHook = Box<dyn Fn(&InstallEvent) -> Result<(), FsError> + Send + Sync>;

// ── EventBus ─────────────────────────────────────────────────────────────────

/// In-process event bus for package install/remove events.
///
/// Register hooks with [`register`](Self::register); dispatch events with
/// [`emit`](Self::emit). Hooks are called in registration order.
///
/// # Example
///
/// ```rust
/// use fs_pkg::event::{EventBus, InstallEvent, InstallEventKind};
///
/// let mut bus = EventBus::new();
///
/// bus.register("logger", |event| {
///     println!("[pkg] {:?} — {}", event.kind, event.package_id);
///     Ok(())
/// });
///
/// let event = InstallEvent::new("proxy/zentinel", "0.1.0", InstallEventKind::InstallCompleted);
/// bus.emit(&event).unwrap();
/// ```
pub struct EventBus {
    hooks: Vec<(String, InstallHook)>,
}

impl EventBus {
    /// Create an empty event bus.
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Register a named hook.
    ///
    /// Hook names are used for logging and deduplication. Registering a hook
    /// with an existing name replaces the previous hook.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        hook: impl Fn(&InstallEvent) -> Result<(), FsError> + Send + Sync + 'static,
    ) {
        let name = name.into();
        // Replace if already registered
        if let Some(entry) = self.hooks.iter_mut().find(|(n, _)| n == &name) {
            entry.1 = Box::new(hook);
        } else {
            self.hooks.push((name, Box::new(hook)));
        }
    }

    /// Remove a registered hook by name.
    pub fn unregister(&mut self, name: &str) {
        self.hooks.retain(|(n, _)| n != name);
    }

    /// Dispatch an event to all registered hooks in order.
    ///
    /// If any hook returns `Err`, dispatch stops and the error is returned.
    pub fn emit(&self, event: &InstallEvent) -> Result<(), FsError> {
        for (name, hook) in &self.hooks {
            hook(event).map_err(|e| {
                FsError::internal(format!("install hook '{name}' failed: {e}"))
            })?;
        }
        Ok(())
    }

    /// Number of registered hooks.
    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    /// `true` if no hooks are registered.
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn completed_event() -> InstallEvent {
        InstallEvent::new("proxy/zentinel", "0.1.0", InstallEventKind::InstallCompleted)
    }

    #[test]
    fn hook_is_called_on_emit() {
        let called = Arc::new(Mutex::new(false));
        let called2 = called.clone();

        let mut bus = EventBus::new();
        bus.register("test", move |_| {
            *called2.lock().unwrap() = true;
            Ok(())
        });

        bus.emit(&completed_event()).unwrap();
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn multiple_hooks_all_called() {
        let count = Arc::new(Mutex::new(0u32));
        let c1 = count.clone();
        let c2 = count.clone();

        let mut bus = EventBus::new();
        bus.register("hook1", move |_| { *c1.lock().unwrap() += 1; Ok(()) });
        bus.register("hook2", move |_| { *c2.lock().unwrap() += 1; Ok(()) });

        bus.emit(&completed_event()).unwrap();
        assert_eq!(*count.lock().unwrap(), 2);
    }

    #[test]
    fn failing_hook_aborts_dispatch() {
        let second_called = Arc::new(Mutex::new(false));
        let sc = second_called.clone();

        let mut bus = EventBus::new();
        bus.register("fail", |_| Err(FsError::internal("boom")));
        bus.register("second", move |_| { *sc.lock().unwrap() = true; Ok(()) });

        assert!(bus.emit(&completed_event()).is_err());
        assert!(!*second_called.lock().unwrap(), "second hook must not run after failure");
    }

    #[test]
    fn unregister_removes_hook() {
        let called = Arc::new(Mutex::new(false));
        let c = called.clone();

        let mut bus = EventBus::new();
        bus.register("removable", move |_| { *c.lock().unwrap() = true; Ok(()) });
        bus.unregister("removable");

        bus.emit(&completed_event()).unwrap();
        assert!(!*called.lock().unwrap());
    }

    #[test]
    fn register_same_name_replaces_hook() {
        let value = Arc::new(Mutex::new(0u32));
        let v1 = value.clone();
        let v2 = value.clone();

        let mut bus = EventBus::new();
        bus.register("counter", move |_| { *v1.lock().unwrap() = 1; Ok(()) });
        bus.register("counter", move |_| { *v2.lock().unwrap() = 2; Ok(()) });

        assert_eq!(bus.len(), 1, "duplicate name should replace, not add");
        bus.emit(&completed_event()).unwrap();
        assert_eq!(*value.lock().unwrap(), 2);
    }

    #[test]
    fn event_carries_package_info() {
        let event = InstallEvent::new("iam/kanidm", "1.0.0", InstallEventKind::RemoveCompleted)
            .with_message("cleanup done");
        assert_eq!(event.package_id, "iam/kanidm");
        assert_eq!(event.version, "1.0.0");
        assert_eq!(event.kind, InstallEventKind::RemoveCompleted);
        assert_eq!(event.message.as_deref(), Some("cleanup done"));
    }
}
