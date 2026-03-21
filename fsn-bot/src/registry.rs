// fsn-bot/src/registry.rs — CommandRegistry for looking up BotCommands.

use std::collections::HashMap;
use std::sync::Arc;

use crate::command::BotCommand;

// ── CommandRegistry ───────────────────────────────────────────────────────────

/// Registry mapping command names to [`BotCommand`] implementations.
///
/// # Example
///
/// ```rust,ignore
/// use fsn_bot::{CommandRegistry, commands::PingCommand};
/// use std::sync::Arc;
///
/// let mut registry = CommandRegistry::new();
/// registry.register(Arc::new(PingCommand));
///
/// let cmd = registry.get("ping").unwrap();
/// ```
#[derive(Default)]
pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn BotCommand>>,
}

impl CommandRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a command. If a command with the same name already exists,
    /// it is replaced.
    pub fn register(&mut self, command: Arc<dyn BotCommand>) {
        self.commands.insert(command.name().to_string(), command);
    }

    /// Look up a command by name (without prefix).
    pub fn get(&self, name: &str) -> Option<Arc<dyn BotCommand>> {
        self.commands.get(name).cloned()
    }

    /// Return all registered command names, sorted alphabetically.
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.commands.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    /// Return all registered commands as `(name, command)` pairs, sorted by name.
    pub fn all(&self) -> Vec<(&str, Arc<dyn BotCommand>)> {
        let mut pairs: Vec<_> = self
            .commands
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone()))
            .collect();
        pairs.sort_by_key(|(k, _)| *k);
        pairs
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{BotCommand, CommandResult};
    use crate::context::CommandContext;
    use crate::error::BotError;
    use async_trait::async_trait;

    struct Noop(&'static str);

    #[async_trait]
    impl BotCommand for Noop {
        fn name(&self) -> &str { self.0 }
        fn usage(&self) -> &str { "noop" }
        fn required_permission(&self) -> Option<fsn_auth::Permission> { None }
        async fn execute(&self, _ctx: &CommandContext) -> Result<CommandResult, BotError> {
            Ok(CommandResult::Silent)
        }
    }

    #[test]
    fn register_and_lookup() {
        let mut reg = CommandRegistry::new();
        reg.register(Arc::new(Noop("ping")));
        assert!(reg.get("ping").is_some());
        assert!(reg.get("unknown").is_none());
    }

    #[test]
    fn names_are_sorted() {
        let mut reg = CommandRegistry::new();
        reg.register(Arc::new(Noop("z-cmd")));
        reg.register(Arc::new(Noop("a-cmd")));
        let names = reg.names();
        assert_eq!(names, vec!["a-cmd", "z-cmd"]);
    }
}
