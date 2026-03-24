//! `fs-types` — Shared types and traits for the FreeSynergy ecosystem.
//!
//! Zero renderer dependencies. Safe to use from any crate in the stack.
//!
//! # Design
//!
//! OOP via traits — types carry their own behavior, no external match blocks.
//! New variants belong in the relevant module; callers use trait methods.
//!
//! # Modules
//!
//! | Module | Contents |
//! |---|---|
//! | [`primitives`] | `FsValue` trait, `FsUrl`, `LanguageCode`, `SemVer`, `FsPort` |
//! | [`tags`] | `FsTag`, `TagLibrary`, `PackageTags`, `PlatformTags`, `ApiTags` |
//! | [`resource`] | `Resource` trait, `ResourceKind`, `Meta` |
//! | [`resources`] | `ResourceMeta` struct, `ResourceType` enum, all resource types |
//! | [`host`] | `HostMode`, `HostStatus` |
//! | [`project`] | `ProjectStatus`, `ProjectVisibility` |
//! | [`module`] | `ModuleStatus`, `ModuleSource` |
//! | [`permission`] | `Action`, `Scope` |
//! | [`type_system`] | `ServiceType`, `ContainerPurpose`, `TypeRegistry`, `TypeEntry` |
//! | [`capability`] | `Capability` trait |
//! | [`requirement`] | `Requirement`, `DeclareRequirements` trait |

pub mod capability;
pub mod host;
pub mod label;
pub mod module;
pub mod permission;
pub mod primitives;
pub mod project;
pub mod requirement;
pub mod resource;
pub mod resources;
pub mod tags;
pub mod type_system;

// ── Flat re-exports ───────────────────────────────────────────────────────────

pub use capability::Capability;
pub use host::{HostMode, HostStatus};
pub use label::StrLabel;
pub use module::{ModuleSource, ModuleStatus};
pub use permission::{Action, Scope};
pub use primitives::FsValue;
pub use primitives::{FsPort, FsUrl, LanguageCode, SemVer};
pub use project::{ProjectStatus, ProjectVisibility};
pub use requirement::{DeclareRequirements, Requirement};
pub use resource::{Meta, Resource, ResourceKind};
pub use resources::{
    platform_filter_from_tags, AnimationSet, AppResource, BotResource, BridgeResource,
    BundleResource, ButtonStyle, ColorScheme, ContainerResource, CursorSet, Dependency, FontSet,
    IconSet, OsFamily, PackageSource, PlatformFilter, RequiredFeature, ResourceMeta, ResourceType,
    Role, StyleResource, Validate, ValidationStatus, WidgetResource, WindowChrome,
};
pub use tags::{ApiTags, FsTag, PackageTags, PlatformTags, TagLibrary};
pub use type_system::{ContainerPurpose, ServiceType, TypeEntry, TypeRegistry};

// ── Tracing conventions (doc-only) ────────────────────────────────────────────

/// Tracing span conventions for FreeSynergy crates.
///
/// ## Rules
///
/// 1. **Instrument public async functions** with `#[tracing::instrument(skip(self))]`.
/// 2. **Skip large or sensitive fields** to avoid flooding the log.
/// 3. **Name spans explicitly** when the function name is ambiguous.
///    ```ignore
///    #[tracing::instrument(name = "container.start", skip(self))]
///    ```
/// 4. **Use structured fields** for IDs and key metadata.
///    ```ignore
///    #[tracing::instrument(fields(service_id = %id))]
///    ```
/// 5. **Log level guidance**
///    - `error!` — unrecoverable failure
///    - `warn!`  — degraded / retry
///    - `info!`  — lifecycle events (start, stop, deploy)
///    - `debug!` — per-request detail
///    - `trace!` — high-frequency loops (avoid in library code)
/// 6. **Never instrument private helpers** unless debugging a specific issue.
pub mod tracing_conventions {}
