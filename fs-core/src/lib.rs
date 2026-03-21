// fs-core — abstract, renderer-independent types for FreeSynergy.
//
// Zero external dependencies. Safe to use from any backend (TUI, GUI, WGUI).
//
// Re-exports everything for ergonomic `use fs_core::*` usage.

pub mod action;
pub mod repository;

pub use action::{FormAction, SelectionResult};
pub use repository::{Repository, RepositoryError, RepositoryManager};
