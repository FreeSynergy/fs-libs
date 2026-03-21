//! Dynamic bridge executor for FreeSynergy.
//!
//! A bridge maps standardized role API calls (e.g. `user.create` for IAM)
//! to the concrete HTTP API of a specific service (e.g. Kanidm).
//!
//! # Architecture
//!
//! ```text
//! caller → BridgeDispatcher → Inventory (find bridge for role)
//!                           → BridgeExecutor  (apply FieldMapping + HTTP call)
//!                           → concrete service (Kanidm / Outline / Forgejo …)
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use fsn_bridge::BridgeDispatcher;
//! use serde_json::json;
//!
//! let dispatcher = BridgeDispatcher::new(inventory);
//! let result = dispatcher.execute("iam", "user.list", json!({})).await?;
//! ```

pub mod catalog;
pub mod dispatcher;
pub mod error;
pub mod executor;

pub use catalog::{forgejo_git_bridge, kanidm_iam_bridge, outline_wiki_bridge};
pub use dispatcher::BridgeDispatcher;
pub use error::BridgeError;
pub use executor::BridgeExecutor;
