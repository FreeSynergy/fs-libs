//! `WidgetResource` — a desktop widget with role requirements.

use super::meta::{ResourceMeta, Role};
use serde::{Deserialize, Serialize};

// ── RequireMode ───────────────────────────────────────────────────────────────

/// How a set of roles must be satisfied for the widget to be installable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequireMode {
    /// At least one role in the group must be provided by an installed service.
    Any,
    /// Every role in the group must be provided.
    All,
}

// ── RoleRequirement ───────────────────────────────────────────────────────────

/// A role group with an ANY/ALL satisfaction mode.
///
/// A widget may declare multiple `RoleRequirement` groups; every group must
/// be satisfied independently before installation is allowed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleRequirement {
    /// The roles in this requirement group.
    pub roles: Vec<Role>,
    /// Whether ANY or ALL roles in the group must be present.
    pub mode: RequireMode,
}

// ── DataSource ────────────────────────────────────────────────────────────────

/// A data source the widget reads from at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// Human-readable label, e.g. `"Unread Messages"`.
    pub label: String,
    /// The role that provides this data, e.g. `"chat"`.
    pub role: Role,
    /// The Bus topic subscribed to, e.g. `"chat.unread.count"`.
    pub bus_topic: String,
}

// ── WidgetSize ────────────────────────────────────────────────────────────────

/// A 2-dimensional grid size (columns × rows).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WidgetSize {
    pub columns: u32,
    pub rows: u32,
}

impl WidgetSize {
    #[must_use]
    pub fn new(columns: u32, rows: u32) -> Self {
        Self { columns, rows }
    }
}

// ── WidgetResource ────────────────────────────────────────────────────────────

/// A desktop widget — a resizable, draggable UI component placed on the desktop.
///
/// **Installation rule:** if any `required_roles` group is unsatisfied (no
/// matching service in the Inventory), installation is blocked.  The installer
/// shows: *"Install a service with role X first."*
/// In a Bundle that includes a matching service, installation is allowed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetResource {
    /// Shared metadata present on every resource.
    pub meta: ResourceMeta,
    /// Role groups that must be satisfied for this widget to function.
    pub required_roles: Vec<RoleRequirement>,
    /// Data sources the widget reads at runtime.
    pub data_sources: Vec<DataSource>,
    /// Minimum grid size.
    pub min_size: WidgetSize,
    /// Maximum grid size.
    pub max_size: WidgetSize,
    /// Default grid size at first placement.
    pub default_size: WidgetSize,
    /// How often the widget requests fresh data (in seconds). `None` = event-driven.
    pub refresh_interval_secs: Option<u64>,
}
