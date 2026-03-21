//! Podman Quadlet management for FreeSynergy.
//!
//! Manages containers **without** a socket or bollard.
//! All container lifecycle operations go through:
//!
//! 1. **Quadlet** — generate `.container` unit files in `~/.config/containers/systemd/`
//! 2. **systemctl** — start / stop / status / daemon-reload
//! 3. **journalctl** — log retrieval
//!
//! # Key Types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`QuadletManager`] | Orchestrates Quadlet file generation + systemctl calls |
//! | [`ServiceConfig`] | Declarative description of a container service |
//! | [`ServiceStatus`] | Runtime status snapshot |
//! | [`Volume`] | Host ↔ container volume mount |
//! | [`PortBinding`] | Host ↔ container port mapping |

pub mod quadlet;
pub mod service;
pub mod systemctl;

pub use quadlet::QuadletManager;
pub use service::{HealthCheck, PortBinding, RestartPolicy, ServiceConfig, Volume};
pub use systemctl::{ServiceStatus, SystemctlManager, UnitActiveState};
