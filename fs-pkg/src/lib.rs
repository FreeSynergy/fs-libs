// fs-pkg — Package manager for FreeSynergy.
//
// Handles the full package lifecycle:
//   install → validate → write files → run hooks → register events
//   remove  → run pre-remove hooks → delete files → unregister events
//   update  → remove + install (with rollback on failure)
//
// Design:
//   OciRef          — Strategy: OCI image reference (registry/repo:tag@digest)
//   ApiManifest     — TOML manifest for a FreeSynergy package
//   PackageInstaller — Installer (Strategy pattern: local | OCI | store)
//   InstallEvent    — Observer: install/remove events broadcast to listeners
//   EventBus        — simple in-process event router for install hooks
//
// Pattern: Strategy (PackageSource), Observer (InstallEvent + EventBus)

pub mod capability_match;
pub mod channel;
pub mod dependency_resolver;
pub mod event;
pub mod installer;
pub mod manifest;
pub mod oci;
pub mod scaling;
pub mod signing;
pub mod variable_roles;
pub mod variable_types;
pub mod versioning;

pub use capability_match::{CapabilityMatch, CapabilityMatcher, CapabilityRegistry, ServiceCapabilities};
pub use channel::ReleaseChannel;
pub use dependency_resolver::{DepGraph, PackageDep, ResolutionError};
pub use event::{EventBus, InstallEvent, InstallHook};
pub use installer::{InstallOptions, InstallOutcome, PackageInstaller};
pub use manifest::{ApiManifest, BundleManifest, PackageFiles, PackageHooks, PackageMeta, PackageRequires, PackageType};
pub use oci::OciRef;
pub use scaling::{InstanceRole, ScalingDialog, ScalingManifest, WorkerMode};
pub use signing::{SignaturePolicy, SignatureVerifier, VerifyOutcome};
pub use variable_roles::{RoleMeta, RoleRegistry, VariableRole, KNOWN_ROLES};
pub use variable_types::{ValidationError, VariableKind, VariableSpec};
pub use versioning::{RollbackError, VersionManager, VersionRecord};
