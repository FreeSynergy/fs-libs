// fs-core — abstract, renderer-independent types for FreeSynergy.
//
// Zero external dependencies. Safe to use from any backend (TUI, GUI, WGUI).
//
// Re-exports everything for ergonomic `use fs_core::*` usage.

pub mod action;
pub mod error;
pub mod manager;
pub mod manifest;
pub mod registry;
pub mod repository;
pub mod store;

pub use action::{FormAction, SelectionResult};
pub use error::ManagerError;
pub use manager::{FsManager, SelectableManager};
pub use manifest::{ManifestBuilder, SetBase, kv, parse_manifest_sections};
pub use registry::{HealthStatus, ManagerRegistry};
pub use repository::{Repository, RepositoryError, RepositoryManager};
pub use store::{ManagerStore, NoopStore};
