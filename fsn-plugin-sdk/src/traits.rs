// traits.rs — Plugin lifecycle traits.
//
// Defines the hooks a plugin may implement to handle installation and removal.
// These are called by the host runtime (fsn-plugin-runtime) at the appropriate
// points in the plugin lifecycle.
//
// Implementing these traits is **optional** — plugins that only need to handle
// commands can rely solely on [`PluginImpl`][crate::PluginImpl].

use crate::{InstanceInfo, PluginResponse};

// ── LifecycleEvent ────────────────────────────────────────────────────────────

/// Lifecycle event passed to install/remove hooks.
///
/// Contains the service instance being installed or removed plus any
/// extra key/value parameters the host supplies.
#[derive(Debug, Clone)]
pub struct LifecycleEvent {
    /// The service instance being operated on.
    pub instance: InstanceInfo,

    /// Extra parameters supplied by the host for this event
    /// (e.g. `upgrade_from = "0.4.2"` on updates).
    pub params: std::collections::HashMap<String, String>,
}

impl LifecycleEvent {
    /// Create a new lifecycle event for `instance` with no extra params.
    pub fn new(instance: InstanceInfo) -> Self {
        Self { instance, params: Default::default() }
    }

    /// Add an extra parameter.
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

// ── PluginInstall ─────────────────────────────────────────────────────────────

/// Lifecycle hook called when a plugin is **installed**.
///
/// The plugin may use this to generate initial configuration files, create
/// directories, register service keys, or emit setup shell commands.
///
/// # Example
///
/// ```rust,ignore
/// use fsn_plugin_sdk::traits::{LifecycleEvent, PluginInstall};
/// use fsn_plugin_sdk::{PluginResponse, ShellCommand};
///
/// struct ZentinelPlugin;
///
/// impl PluginInstall for ZentinelPlugin {
///     fn on_install(&self, event: &LifecycleEvent) -> Result<PluginResponse, String> {
///         let mut resp = PluginResponse::default();
///         resp.commands.push(ShellCommand {
///             cmd: format!("mkdir -p {}/zentinel", event.instance.data_root),
///             cwd: None,
///             env: Default::default(),
///         });
///         Ok(resp)
///     }
/// }
/// ```
pub trait PluginInstall: Send + Sync {
    /// Called once when the plugin/service is installed on a host.
    ///
    /// Return `Ok(response)` with any shell commands, file outputs, or log
    /// messages the host should execute/write.  Return `Err(message)` to abort
    /// the installation with a human-readable error.
    fn on_install(&self, event: &LifecycleEvent) -> Result<PluginResponse, String>;
}

// ── PluginRemove ──────────────────────────────────────────────────────────────

/// Lifecycle hook called when a plugin is **removed** (uninstalled).
///
/// The plugin may use this to clean up generated files, revoke tokens, or
/// emit teardown shell commands.
///
/// # Example
///
/// ```rust,ignore
/// use fsn_plugin_sdk::traits::{LifecycleEvent, PluginRemove};
/// use fsn_plugin_sdk::{PluginResponse, ShellCommand};
///
/// struct ZentinelPlugin;
///
/// impl PluginRemove for ZentinelPlugin {
///     fn on_remove(&self, event: &LifecycleEvent) -> Result<PluginResponse, String> {
///         let mut resp = PluginResponse::default();
///         resp.commands.push(ShellCommand {
///             cmd: format!("rm -rf {}/zentinel", event.instance.data_root),
///             cwd: None,
///             env: Default::default(),
///         });
///         Ok(resp)
///     }
/// }
/// ```
pub trait PluginRemove: Send + Sync {
    /// Called once when the plugin/service is removed from a host.
    ///
    /// Return `Ok(response)` with teardown commands or `Err(message)` to
    /// indicate a cleanup failure (the host will log but continue removal).
    fn on_remove(&self, event: &LifecycleEvent) -> Result<PluginResponse, String>;
}

// ── PluginUpgrade ─────────────────────────────────────────────────────────────

/// Lifecycle hook called when a plugin is **upgraded** to a newer version.
///
/// The `event.params` map will contain `"upgrade_from"` with the previous
/// version string.
pub trait PluginUpgrade: Send + Sync {
    /// Called once when the plugin/service is upgraded.
    fn on_upgrade(&self, event: &LifecycleEvent) -> Result<PluginResponse, String>;
}

// ── PluginLifecycle ───────────────────────────────────────────────────────────

/// Convenience supertrait that combines all lifecycle hooks.
///
/// Implement this instead of the individual traits when a plugin handles
/// install, remove, and upgrade.
pub trait PluginLifecycle: PluginInstall + PluginRemove + PluginUpgrade {}

// Blanket implementation so any type implementing all three also implements the supertrait.
impl<T: PluginInstall + PluginRemove + PluginUpgrade> PluginLifecycle for T {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{InstanceInfo, LogLevel, LogLine, PluginResponse, ShellCommand};

    use super::*;

    fn dummy_instance() -> InstanceInfo {
        InstanceInfo {
            name: "zentinel".into(),
            class_key: "proxy/zentinel".into(),
            domain: "zentinel.example.com".into(),
            project: "example".into(),
            project_domain: "example.com".into(),
            data_root: "/data/zentinel".into(),
            env: HashMap::new(),
        }
    }

    struct DummyPlugin;

    impl PluginInstall for DummyPlugin {
        fn on_install(&self, event: &LifecycleEvent) -> Result<PluginResponse, String> {
            let mut resp = PluginResponse::default();
            resp.commands.push(ShellCommand {
                cmd: format!("mkdir -p {}", event.instance.data_root),
                cwd: None,
                env: HashMap::new(),
            });
            Ok(resp)
        }
    }

    impl PluginRemove for DummyPlugin {
        fn on_remove(&self, event: &LifecycleEvent) -> Result<PluginResponse, String> {
            let mut resp = PluginResponse::default();
            resp.commands.push(ShellCommand {
                cmd: format!("rm -rf {}", event.instance.data_root),
                cwd: None,
                env: HashMap::new(),
            });
            Ok(resp)
        }
    }

    impl PluginUpgrade for DummyPlugin {
        fn on_upgrade(&self, event: &LifecycleEvent) -> Result<PluginResponse, String> {
            let from = event.params.get("upgrade_from").map(|s| s.as_str()).unwrap_or("unknown");
            let mut resp = PluginResponse::default();
            resp.logs.push(LogLine {
                level: LogLevel::Info,
                message: format!("upgraded from {from}"),
            });
            Ok(resp)
        }
    }

    #[test]
    fn install_hook_emits_shell_command() {
        let plugin = DummyPlugin;
        let event = LifecycleEvent::new(dummy_instance());
        let resp = plugin.on_install(&event).unwrap();
        assert_eq!(resp.commands.len(), 1);
        assert!(resp.commands[0].cmd.contains("/data/zentinel"));
    }

    #[test]
    fn remove_hook_emits_rm_command() {
        let plugin = DummyPlugin;
        let event = LifecycleEvent::new(dummy_instance());
        let resp = plugin.on_remove(&event).unwrap();
        assert!(resp.commands[0].cmd.starts_with("rm -rf"));
    }

    #[test]
    fn upgrade_hook_logs_version() {
        let plugin = DummyPlugin;
        let event = LifecycleEvent::new(dummy_instance())
            .with_param("upgrade_from", "0.3.1");
        let resp = plugin.on_upgrade(&event).unwrap();
        assert!(resp.logs[0].message.contains("0.3.1"));
    }

    #[test]
    fn lifecycle_event_params() {
        let event = LifecycleEvent::new(dummy_instance())
            .with_param("key", "value");
        assert_eq!(event.params.get("key").unwrap(), "value");
    }
}
