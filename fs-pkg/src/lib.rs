// fs-pkg — Package manager for FreeSynergy.
//
// Handles the full package lifecycle:
//   install → validate → write files → run hooks → register events
//   remove  → run pre-remove hooks → delete files → unregister events
//   update  → remove + install (with rollback on failure)
//
// Design:
//   InstallPaths          — configurable base directories for all resource types
//   InstallerRegistry     — maps ResourceType → Installer/Uninstaller (Strategy)
//   Updater               — atomic update orchestrator (Template Method + rollback)
//   OciRef                — Strategy: OCI image reference (registry/repo:tag@digest)
//   ApiManifest           — TOML manifest for a FreeSynergy package
//   PackageInstaller      — low-level file installer (Strategy: local | OCI | store)
//   InstallEvent          — Observer: install/remove events broadcast to listeners
//   EventBus              — simple in-process event router for install hooks
//   SetupFlow             — Chain of Responsibility: ordered setup steps per package
//   SetupContributor      — Visitor: cross-package setup step injection
//
// Patterns: Strategy (PackageSource, InstallerRegistry, SetupStep kinds),
//           Observer (InstallEvent + EventBus),
//           Template Method (Updater, SetupStep defaults),
//           Registry (InstallerRegistry, SetupContributorRegistry),
//           Chain of Responsibility (SetupFlow),
//           Context (SetupContext, persisted to disk)

pub mod capability_match;
pub mod channel;
pub mod dependency_resolver;
pub mod event;
pub mod install_paths;
pub mod installer;
pub mod installer_registry;
pub mod installers;
pub mod manageable;
pub mod manifest;
pub mod package;
pub mod oci;
pub mod scaling;
pub mod setup_flow;
pub mod setup_step;
pub mod setup_contributor;
pub mod signing;
pub mod updater;
pub mod variable_roles;
pub mod variable_types;
pub mod versioning;

pub use capability_match::{CapabilityMatch, CapabilityMatcher, CapabilityRegistry, ServiceCapabilities};
pub use channel::ReleaseChannel;
pub use dependency_resolver::{DepGraph, PackageDep, ResolutionError};
pub use event::{EventBus, InstallEvent, InstallHook};
pub use installer::{
    FetchStrategy, GithubReleaseFetch, LocalFetch, NoOpFetch, OciFetch,
    fetch_strategy_for, InstallOptions, InstallOutcome, PackageInstaller,
};
pub use manageable::{
    ConfigField, ConfigFieldKind, ConfigValue, HealthCheck, InstanceRef,
    Manageable, PackageHealth, RunStatus, SelectOption,
};
pub use manifest::{
    ApiManifest, AppManifest, BundleManifest, ContainerHealthCheck, ContainerManifest,
    ContractManifest, ContractRoute, FileMapping, ManifestFieldType, ManifestVariable,
    PackageFiles, PackageHooks, PackageId, PackageMeta, PackageRequires, PackageSource,
    PackageType, SetupField, SetupManifest,
};
pub use setup_flow::{ServiceRef, SetupContext, SetupFlow, StepExecution};
pub use setup_step::{
    CommandStep, DisplayOutputStep, InputField, InputStep, SetupTrigger,
    StepOutput, StepState, WaitForServiceStep, WriteConfigStep, generate_secret,
};
pub use setup_contributor::{Contribution, SetupContributor, SetupContributorRegistry};
pub use package::{InstalledRecord, Package};
pub use oci::OciRef;
pub use scaling::{InstanceRole, ScalingDialog, ScalingManifest, WorkerMode};
pub use signing::{SignaturePolicy, SignatureVerifier, VerifyOutcome};
pub use variable_roles::{RoleMeta, RoleRegistry, VariableRole, KNOWN_ROLES};
pub use variable_types::{ValidationError, VariableKind, VariableSpec};
pub use versioning::{RollbackError, VersionManager, VersionRecord};

// ── Phase U exports ───────────────────────────────────────────────────────────

pub use install_paths::{InstallPaths, MoveOutcome, PathMigrator};
pub use installer_registry::InstallerRegistry;
pub use installers::{InstallReport, Installer, UninstallOptions, Uninstaller};
pub use updater::{BatchUpdateOutcome, UpdateOutcome, Updater, record_version};
