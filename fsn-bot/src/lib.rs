//! Bot command framework for FreeSynergy.
//!
//! Provides a command registry, permission-checked dispatch, and a set of
//! built-in commands. Bridges fsn-bus events to command handlers and integrates
//! with fsn-channel for replies.
//!
//! # Architecture
//!
//! ```text
//! fsn-bus Router
//!   └─ BotRouter (TopicHandler for "channel.message.incoming")
//!        └─ CommandRegistry
//!             ├─ PingCommand
//!             ├─ HelpCommand
//!             ├─ StatusCommand
//!             ├─ DeployCommand → fires "deploy.requested" on bus
//!             └─ HealthQueryCommand
//! ```
//!
//! # Quick start
//!
//! ```rust,ignore
//! use fsn_bot::{CommandRegistry, BotRouter, AllowAllPermissions, commands::*};
//! use std::sync::Arc;
//!
//! let mut registry = CommandRegistry::new();
//! registry.register(Arc::new(PingCommand));
//! registry.register(Arc::new(StatusCommand::default()));
//!
//! let router = BotRouter::new("!", registry, channel, Arc::new(AllowAllPermissions));
//! bus_router.register(Arc::new(router));
//! ```

pub mod command;
pub mod commands;
pub mod context;
pub mod error;
pub mod registry;
pub mod router;

// ── Flat re-exports ───────────────────────────────────────────────────────────

pub use command::{BotCommand, CommandResult};
pub use context::CommandContext;
pub use error::BotError;
pub use registry::CommandRegistry;
pub use router::{AllowAllPermissions, BotRouter, DenyAllPermissions, PermissionResolver};
