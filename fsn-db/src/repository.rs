/// CRUD repositories for FreeSynergy database entities.
///
/// Each repository wraps a reference to a [`sea_orm::DatabaseConnection`] and
/// provides typed async methods for the corresponding table.
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    Order, QueryFilter, QueryOrder, QuerySelect,
};

use crate::entities::{audit_log, host, installed_package, module, permission, plugin, project, resource, service_registry};
use fsn_error::FsnError;

// ── helpers ───────────────────────────────────────────────────────────────────

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ── ResourceRepo ──────────────────────────────────────────────────────────────

/// Repository for the `resources` table.
pub struct ResourceRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> ResourceRepo<'a> {
    /// Create a new repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert a new resource. Returns the model with its generated `id`.
    pub async fn insert(
        &self,
        kind: impl Into<String>,
        name: impl Into<String>,
        project_id: Option<i64>,
        parent_id: Option<i64>,
        meta: Option<String>,
    ) -> Result<resource::Model, FsnError> {
        let now = unix_now();
        let active = resource::ActiveModel {
            kind: Set(kind.into()),
            name: Set(name.into()),
            project_id: Set(project_id),
            parent_id: Set(parent_id),
            meta: Set(meta),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ResourceRepo::insert: {e}")))
    }

    /// Find a resource by its primary key.
    pub async fn find_by_id(&self, id: i64) -> Result<Option<resource::Model>, FsnError> {
        resource::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ResourceRepo::find_by_id: {e}")))
    }

    /// List all resources of a given kind.
    pub async fn find_by_kind(&self, kind: &str) -> Result<Vec<resource::Model>, FsnError> {
        resource::Entity::find()
            .filter(resource::Column::Kind.eq(kind))
            .all(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ResourceRepo::find_by_kind: {e}")))
    }

    /// List all resources.
    pub async fn find_all(&self) -> Result<Vec<resource::Model>, FsnError> {
        resource::Entity::find()
            .all(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ResourceRepo::find_all: {e}")))
    }

    /// Update the `name` and `meta` of an existing resource.
    pub async fn update(
        &self,
        id: i64,
        name: impl Into<String>,
        meta: Option<String>,
    ) -> Result<resource::Model, FsnError> {
        let active = resource::ActiveModel {
            id: Set(id),
            name: Set(name.into()),
            meta: Set(meta),
            updated_at: Set(unix_now()),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ResourceRepo::update: {e}")))
    }

    /// Delete a resource by its primary key.
    pub async fn delete_by_id(&self, id: i64) -> Result<(), FsnError> {
        resource::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("ResourceRepo::delete_by_id: {e}")))
    }
}

// ── PermissionRepo ────────────────────────────────────────────────────────────

/// Repository for the `permissions` table.
pub struct PermissionRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> PermissionRepo<'a> {
    /// Create a new permission repository.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Grant a permission to a subject for an action, optionally scoped to a resource.
    pub async fn grant(
        &self,
        subject: impl Into<String>,
        action: impl Into<String>,
        resource_id: Option<i64>,
        expires_at: Option<i64>,
    ) -> Result<permission::Model, FsnError> {
        let active = permission::ActiveModel {
            subject: Set(subject.into()),
            action: Set(action.into()),
            resource_id: Set(resource_id),
            granted_at: Set(unix_now()),
            expires_at: Set(expires_at),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("PermissionRepo::grant: {e}")))
    }

    /// List all non-expired permissions for a subject.
    pub async fn find_for_subject(&self, subject: &str) -> Result<Vec<permission::Model>, FsnError> {
        let now = unix_now();
        permission::Entity::find()
            .filter(permission::Column::Subject.eq(subject))
            .filter(
                sea_orm::Condition::any()
                    .add(permission::Column::ExpiresAt.is_null())
                    .add(permission::Column::ExpiresAt.gt(now)),
            )
            .all(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("PermissionRepo::find_for_subject: {e}")))
    }

    /// Revoke a specific permission by its primary key.
    pub async fn revoke(&self, id: i64) -> Result<(), FsnError> {
        permission::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("PermissionRepo::revoke: {e}")))
    }
}

// ── AuditRepo ─────────────────────────────────────────────────────────────────

/// Append-only repository for the `audit_logs` table.
pub struct AuditRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> AuditRepo<'a> {
    /// Create a new audit repository.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Append an audit log entry. Returns the inserted row.
    pub async fn log(
        &self,
        actor: impl Into<String>,
        action: impl Into<String>,
        resource_id: Option<i64>,
        resource_kind: Option<String>,
        payload: Option<String>,
        source: Option<String>,
        outcome: impl Into<String>,
    ) -> Result<audit_log::Model, FsnError> {
        let active = audit_log::ActiveModel {
            actor: Set(actor.into()),
            action: Set(action.into()),
            resource_id: Set(resource_id),
            resource_kind: Set(resource_kind),
            payload: Set(payload),
            source: Set(source),
            outcome: Set(outcome.into()),
            created_at: Set(unix_now()),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("AuditRepo::log: {e}")))
    }

    /// Return the most recent `limit` audit log entries (newest first).
    pub async fn recent(&self, limit: u64) -> Result<Vec<audit_log::Model>, FsnError> {
        audit_log::Entity::find()
            .order_by(audit_log::Column::CreatedAt, Order::Desc)
            .limit(Some(limit))
            .all(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("AuditRepo::recent: {e}")))
    }

    /// Return all audit entries for a specific resource.
    pub async fn find_for_resource(&self, resource_id: i64) -> Result<Vec<audit_log::Model>, FsnError> {
        audit_log::Entity::find()
            .filter(audit_log::Column::ResourceId.eq(resource_id))
            .order_by(audit_log::Column::CreatedAt, Order::Desc)
            .all(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("AuditRepo::find_for_resource: {e}")))
    }
}

// ── PluginRepo ────────────────────────────────────────────────────────────────

/// Repository for the `plugins` table.
pub struct PluginRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> PluginRepo<'a> {
    /// Create a new plugin repository.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Register a new plugin or update its version/path if it already exists.
    pub async fn upsert(
        &self,
        name: impl Into<String>,
        version: impl Into<String>,
        kind: impl Into<String>,
        path: Option<String>,
        wasm_hash: Option<String>,
        meta: Option<String>,
    ) -> Result<plugin::Model, FsnError> {
        let name = name.into();
        let existing = plugin::Entity::find()
            .filter(plugin::Column::Name.eq(&name))
            .one(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("PluginRepo::upsert find: {e}")))?;

        if let Some(existing) = existing {
            let active = plugin::ActiveModel {
                id: Set(existing.id),
                version: Set(version.into()),
                kind: Set(kind.into()),
                path: Set(path),
                wasm_hash: Set(wasm_hash),
                meta: Set(meta),
                ..Default::default()
            };
            active
                .update(self.conn)
                .await
                .map_err(|e| FsnError::internal(format!("PluginRepo::upsert update: {e}")))
        } else {
            let active = plugin::ActiveModel {
                name: Set(name),
                version: Set(version.into()),
                kind: Set(kind.into()),
                path: Set(path),
                wasm_hash: Set(wasm_hash),
                enabled: Set(true),
                meta: Set(meta),
                installed_at: Set(unix_now()),
                ..Default::default()
            };
            active
                .insert(self.conn)
                .await
                .map_err(|e| FsnError::internal(format!("PluginRepo::upsert insert: {e}")))
        }
    }

    /// List all enabled plugins.
    pub async fn find_enabled(&self) -> Result<Vec<plugin::Model>, FsnError> {
        plugin::Entity::find()
            .filter(plugin::Column::Enabled.eq(true))
            .all(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("PluginRepo::find_enabled: {e}")))
    }

    /// Find a plugin by name.
    pub async fn find_by_name(&self, name: &str) -> Result<Option<plugin::Model>, FsnError> {
        plugin::Entity::find()
            .filter(plugin::Column::Name.eq(name))
            .one(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("PluginRepo::find_by_name: {e}")))
    }

    /// Enable or disable a plugin by its primary key.
    pub async fn set_enabled(&self, id: i64, enabled: bool) -> Result<(), FsnError> {
        let active = plugin::ActiveModel {
            id: Set(id),
            enabled: Set(enabled),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("PluginRepo::set_enabled: {e}")))
    }

    /// Remove a plugin registration by its primary key.
    pub async fn delete_by_id(&self, id: i64) -> Result<(), FsnError> {
        plugin::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("PluginRepo::delete_by_id: {e}")))
    }
}

// ── HostRepo ──────────────────────────────────────────────────────────────────

/// Repository for the `hosts` table.
pub struct HostRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> HostRepo<'a> {
    /// Create a new host repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert a new host record. Returns the inserted model with its generated `id`.
    pub async fn insert(
        &self,
        name: impl Into<String>,
        fqdn: impl Into<String>,
        ip_address: impl Into<String>,
        ssh_port: i32,
        project_id: Option<i64>,
    ) -> Result<host::Model, FsnError> {
        let now = unix_now();
        let active = host::ActiveModel {
            name: Set(name.into()),
            fqdn: Set(fqdn.into()),
            ip_address: Set(ip_address.into()),
            ssh_port: Set(ssh_port),
            status: Set("unknown".into()),
            os: Set(None),
            architecture: Set(None),
            agent_version: Set(None),
            project_id: Set(project_id),
            joined_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("HostRepo::insert: {e}")))
    }

    /// Find a host by its primary key.
    pub async fn find_by_id(&self, id: i64) -> Result<Option<host::Model>, FsnError> {
        host::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("HostRepo::find_by_id: {e}")))
    }

    /// List all hosts.
    pub async fn find_all(&self) -> Result<Vec<host::Model>, FsnError> {
        host::Entity::find()
            .all(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("HostRepo::find_all: {e}")))
    }

    /// Update the operational status of a host.
    pub async fn update_status(&self, id: i64, status: impl Into<String>) -> Result<(), FsnError> {
        let active = host::ActiveModel {
            id: Set(id),
            status: Set(status.into()),
            updated_at: Set(unix_now()),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("HostRepo::update_status: {e}")))
    }

    /// Delete a host by its primary key.
    pub async fn delete_by_id(&self, id: i64) -> Result<(), FsnError> {
        host::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("HostRepo::delete_by_id: {e}")))
    }
}

// ── ProjectRepo ───────────────────────────────────────────────────────────────

/// Repository for the `projects` table.
pub struct ProjectRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> ProjectRepo<'a> {
    /// Create a new project repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert a new project. Returns the inserted model with its generated `id`.
    pub async fn insert(
        &self,
        name: impl Into<String>,
        domain: Option<String>,
        description: Option<String>,
    ) -> Result<project::Model, FsnError> {
        let now = unix_now();
        let active = project::ActiveModel {
            name: Set(name.into()),
            domain: Set(domain),
            description: Set(description),
            status: Set("draft".into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ProjectRepo::insert: {e}")))
    }

    /// Find a project by its primary key.
    pub async fn find_by_id(&self, id: i64) -> Result<Option<project::Model>, FsnError> {
        project::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ProjectRepo::find_by_id: {e}")))
    }

    /// List all projects.
    pub async fn find_all(&self) -> Result<Vec<project::Model>, FsnError> {
        project::Entity::find()
            .all(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ProjectRepo::find_all: {e}")))
    }

    /// Update a project's name, domain, description, and status.
    pub async fn update(
        &self,
        id: i64,
        name: impl Into<String>,
        domain: Option<String>,
        description: Option<String>,
        status: impl Into<String>,
    ) -> Result<project::Model, FsnError> {
        let active = project::ActiveModel {
            id: Set(id),
            name: Set(name.into()),
            domain: Set(domain),
            description: Set(description),
            status: Set(status.into()),
            updated_at: Set(unix_now()),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ProjectRepo::update: {e}")))
    }

    /// Delete a project by its primary key.
    pub async fn delete_by_id(&self, id: i64) -> Result<(), FsnError> {
        project::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("ProjectRepo::delete_by_id: {e}")))
    }
}

// ── ModuleRepo ────────────────────────────────────────────────────────────────

/// Repository for the `modules` table.
pub struct ModuleRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> ModuleRepo<'a> {
    /// Create a new module repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert a new module instance. Returns the inserted model with its generated `id`.
    pub async fn insert(
        &self,
        name: impl Into<String>,
        module_type: impl Into<String>,
        host_id: i64,
        project_id: Option<i64>,
        version: Option<String>,
        config: Option<String>,
    ) -> Result<module::Model, FsnError> {
        let now = unix_now();
        let active = module::ActiveModel {
            name: Set(name.into()),
            module_type: Set(module_type.into()),
            host_id: Set(host_id),
            project_id: Set(project_id),
            status: Set("stopped".into()),
            version: Set(version),
            config: Set(config),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ModuleRepo::insert: {e}")))
    }

    /// Find a module by its primary key.
    pub async fn find_by_id(&self, id: i64) -> Result<Option<module::Model>, FsnError> {
        module::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ModuleRepo::find_by_id: {e}")))
    }

    /// List all modules running on a specific host.
    pub async fn find_by_host(&self, host_id: i64) -> Result<Vec<module::Model>, FsnError> {
        module::Entity::find()
            .filter(module::Column::HostId.eq(host_id))
            .all(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ModuleRepo::find_by_host: {e}")))
    }

    /// List all modules belonging to a specific project.
    pub async fn find_by_project(&self, project_id: i64) -> Result<Vec<module::Model>, FsnError> {
        module::Entity::find()
            .filter(module::Column::ProjectId.eq(project_id))
            .all(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ModuleRepo::find_by_project: {e}")))
    }

    /// Update the operational status of a module.
    pub async fn update_status(
        &self,
        id: i64,
        status: impl Into<String>,
    ) -> Result<(), FsnError> {
        let active = module::ActiveModel {
            id: Set(id),
            status: Set(status.into()),
            updated_at: Set(unix_now()),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("ModuleRepo::update_status: {e}")))
    }

    /// Delete a module by its primary key.
    pub async fn delete_by_id(&self, id: i64) -> Result<(), FsnError> {
        module::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("ModuleRepo::delete_by_id: {e}")))
    }
}

// ── InstalledPackageRepo ──────────────────────────────────────────────────────

/// Repository for the `installed_packages` table.
///
/// Tracks all installed package versions, supporting multi-version coexistence
/// and rollback by toggling the `active` flag.
pub struct InstalledPackageRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> InstalledPackageRepo<'a> {
    /// Create a new repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert a new installed package record. Returns the auto-generated `id`.
    ///
    /// The new record is always inserted as `active = true`. Call
    /// [`set_active`](Self::set_active) on any previous version to deactivate it.
    pub async fn insert(
        &self,
        package_id: impl Into<String>,
        version: impl Into<String>,
        channel: impl Into<String>,
        package_type: impl Into<String>,
        signature: Option<String>,
        trust_unsigned: bool,
    ) -> Result<i64, FsnError> {
        let now = unix_now();
        let active = installed_package::ActiveModel {
            package_id:    Set(package_id.into()),
            version:       Set(version.into()),
            channel:       Set(channel.into()),
            package_type:  Set(package_type.into()),
            active:        Set(true),
            signature:     Set(signature),
            trust_unsigned: Set(trust_unsigned),
            installed_at:  Set(now),
            updated_at:    Set(now),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map(|m| m.id)
            .map_err(|e| FsnError::internal(format!("InstalledPackageRepo::insert: {e}")))
    }

    /// Find the currently active version for `package_id`.
    pub async fn find_active(
        &self,
        package_id: &str,
    ) -> Result<Option<installed_package::Model>, FsnError> {
        installed_package::Entity::find()
            .filter(installed_package::Column::PackageId.eq(package_id))
            .filter(installed_package::Column::Active.eq(true))
            .one(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("InstalledPackageRepo::find_active: {e}")))
    }

    /// List every installed package record across all versions.
    pub async fn list_all(&self) -> Result<Vec<installed_package::Model>, FsnError> {
        installed_package::Entity::find()
            .all(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("InstalledPackageRepo::list_all: {e}")))
    }

    /// Set the `active` flag for a specific record by `id`.
    pub async fn set_active(&self, id: i64, active: bool) -> Result<(), FsnError> {
        let model = installed_package::ActiveModel {
            id:         Set(id),
            active:     Set(active),
            updated_at: Set(unix_now()),
            ..Default::default()
        };
        model
            .update(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("InstalledPackageRepo::set_active: {e}")))
    }

    /// Remove a package record by its primary key.
    pub async fn remove(&self, id: i64) -> Result<(), FsnError> {
        installed_package::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("InstalledPackageRepo::remove: {e}")))
    }
}

// ── ServiceRegistryRepo ───────────────────────────────────────────────────────

/// Repository for the `service_registry` table.
pub struct ServiceRegistryRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> ServiceRegistryRepo<'a> {
    /// Create a new service registry repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert or update a service registry entry for `module_id`.
    ///
    /// If a row for `module_id` already exists it is updated; otherwise a new
    /// row is inserted.
    pub async fn upsert(
        &self,
        module_id: i64,
        module_name: impl Into<String>,
        capabilities: impl Into<String>,
        endpoint_url: Option<String>,
    ) -> Result<service_registry::Model, FsnError> {
        let existing = service_registry::Entity::find()
            .filter(service_registry::Column::ModuleId.eq(module_id))
            .one(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ServiceRegistryRepo::upsert find: {e}")))?;

        if let Some(row) = existing {
            let active = service_registry::ActiveModel {
                id: Set(row.id),
                module_name: Set(module_name.into()),
                capabilities: Set(capabilities.into()),
                endpoint_url: Set(endpoint_url),
                ..Default::default()
            };
            active
                .update(self.conn)
                .await
                .map_err(|e| FsnError::internal(format!("ServiceRegistryRepo::upsert update: {e}")))
        } else {
            let active = service_registry::ActiveModel {
                module_id: Set(module_id),
                module_name: Set(module_name.into()),
                capabilities: Set(capabilities.into()),
                endpoint_url: Set(endpoint_url),
                healthy: Set(false),
                last_check: Set(None),
                ..Default::default()
            };
            active
                .insert(self.conn)
                .await
                .map_err(|e| FsnError::internal(format!("ServiceRegistryRepo::upsert insert: {e}")))
        }
    }

    /// Find the registry entry for a specific module.
    pub async fn find_by_module(
        &self,
        module_id: i64,
    ) -> Result<Option<service_registry::Model>, FsnError> {
        service_registry::Entity::find()
            .filter(service_registry::Column::ModuleId.eq(module_id))
            .one(self.conn)
            .await
            .map_err(|e| FsnError::internal(format!("ServiceRegistryRepo::find_by_module: {e}")))
    }

    /// List all registry entries whose `capabilities` JSON contains `capability`.
    ///
    /// Uses a SQL `LIKE` search — suitable for simple string matching.
    pub async fn find_by_capability(
        &self,
        capability: &str,
    ) -> Result<Vec<service_registry::Model>, FsnError> {
        service_registry::Entity::find()
            .filter(service_registry::Column::Capabilities.contains(capability))
            .all(self.conn)
            .await
            .map_err(|e| {
                FsnError::internal(format!("ServiceRegistryRepo::find_by_capability: {e}"))
            })
    }

    /// Mark a registry entry as healthy or unhealthy, updating `last_check`.
    pub async fn set_healthy(&self, id: i64, healthy: bool) -> Result<(), FsnError> {
        let active = service_registry::ActiveModel {
            id: Set(id),
            healthy: Set(healthy),
            last_check: Set(Some(unix_now())),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsnError::internal(format!("ServiceRegistryRepo::set_healthy: {e}")))
    }
}
