//! `fs-inventory` — local inventory of installed FreeSynergy resources.
//!
//! The Inventory answers the question *"What is installed on this node?"*.
//! It is the **single source of truth** for:
//!
//! - Which resources are installed and at what version
//! - Which service instances are running and on which ports
//! - Which bridge instances are active and serving which roles
//!
//! # Database
//!
//! Uses its own SQLite file: `fs-inventory.db`.
//! No other component may maintain a parallel list.
//!
//! # Usage
//!
//! ```no_run
//! use fs_inventory::Inventory;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let inv = Inventory::open("fs-inventory.db").await?;
//! let services = inv.services_with_role("iam").await?;
//! # Ok(())
//! # }
//! ```

pub mod entity;
pub mod error;
pub mod models;
pub mod repo;

pub use error::InventoryError;
pub use models::{
    BridgeInstance, BridgeStatus, InstalledResource, ResourceStatus, ServiceInstance, ServiceStatus,
};
pub use repo::Inventory;
