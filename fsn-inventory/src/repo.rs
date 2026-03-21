//! `Inventory` — the primary interface to `fsn-inventory.db`.

use crate::{
    entity::{bridge_instance, installed_resource, service_instance},
    error::InventoryError,
    models::{BridgeInstance, InstalledResource, ResourceStatus, ServiceInstance, ServiceStatus},
};
use fsn_types::Role;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, Database,
    DatabaseConnection, EntityTrait, QueryFilter,
};
use tracing::instrument;

// ── Schema SQL ────────────────────────────────────────────────────────────────

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS installed_resources (
    id            TEXT PRIMARY KEY NOT NULL,
    resource_type TEXT NOT NULL,
    version       TEXT NOT NULL,
    channel       TEXT NOT NULL DEFAULT 'stable',
    installed_at  TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT '{\"state\":\"active\"}',
    config_path   TEXT NOT NULL DEFAULT '',
    data_path     TEXT NOT NULL DEFAULT '',
    validation    TEXT NOT NULL DEFAULT 'incomplete'
);

CREATE TABLE IF NOT EXISTS service_instances (
    id             TEXT PRIMARY KEY NOT NULL,
    resource_id    TEXT NOT NULL REFERENCES installed_resources(id),
    instance_name  TEXT NOT NULL,
    roles_provided TEXT NOT NULL DEFAULT '[]',
    roles_required TEXT NOT NULL DEFAULT '[]',
    bridges        TEXT NOT NULL DEFAULT '[]',
    variables      TEXT NOT NULL DEFAULT '[]',
    network        TEXT NOT NULL DEFAULT '',
    status         TEXT NOT NULL DEFAULT '{\"state\":\"stopped\"}',
    port           INTEGER,
    s3_paths       TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE IF NOT EXISTS bridge_instances (
    id               TEXT PRIMARY KEY NOT NULL,
    bridge_id        TEXT NOT NULL,
    role             TEXT NOT NULL,
    service_instance TEXT NOT NULL REFERENCES service_instances(id),
    api_base_url     TEXT NOT NULL,
    status           TEXT NOT NULL DEFAULT 'active'
);

CREATE INDEX IF NOT EXISTS idx_si_resource_id ON service_instances(resource_id);
CREATE INDEX IF NOT EXISTS idx_bi_role        ON bridge_instances(role);
CREATE INDEX IF NOT EXISTS idx_bi_service     ON bridge_instances(service_instance);
";

// ── Inventory ─────────────────────────────────────────────────────────────────

/// The local inventory — the single source of truth for installed resources.
pub struct Inventory {
    db: DatabaseConnection,
}

impl Inventory {
    /// Open (or create) the inventory database at the given path.
    #[instrument(name = "inventory.open")]
    pub async fn open(path: &str) -> Result<Self, InventoryError> {
        let url = format!("sqlite://{}?mode=rwc", path);
        let db = Database::connect(&url).await?;
        db.execute_unprepared(SCHEMA)
        .await?;
        Ok(Self { db })
    }

    // ── InstalledResource ─────────────────────────────────────────────────────

    /// Insert a newly installed resource.
    #[instrument(name = "inventory.install", skip(self, resource))]
    pub async fn install(&self, resource: &InstalledResource) -> Result<(), InventoryError> {
        if installed_resource::Entity::find_by_id(&resource.id)
            .one(&self.db)
            .await?
            .is_some()
        {
            return Err(InventoryError::AlreadyInstalled { id: resource.id.clone() });
        }
        installed_resource::ActiveModel {
            id:            Set(resource.id.clone()),
            resource_type: Set(serde_json::to_string(&resource.resource_type)?),
            version:       Set(resource.version.clone()),
            channel:       Set(serde_json::to_string(&resource.channel)?),
            installed_at:  Set(resource.installed_at.clone()),
            status:        Set(serde_json::to_string(&resource.status)?),
            config_path:   Set(resource.config_path.clone()),
            data_path:     Set(resource.data_path.clone()),
            validation:    Set(serde_json::to_string(&resource.validation)?),
        }
        .insert(&self.db)
        .await?;
        Ok(())
    }

    /// Remove an installed resource by id.
    #[instrument(name = "inventory.uninstall", skip(self))]
    pub async fn uninstall(&self, id: &str) -> Result<(), InventoryError> {
        let model = installed_resource::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| InventoryError::NotFound { id: id.to_owned() })?;
        let active: installed_resource::ActiveModel = model.into();
        active.delete(&self.db).await?;
        Ok(())
    }

    /// All installed resources.
    pub async fn all_resources(&self) -> Result<Vec<InstalledResource>, InventoryError> {
        let rows = installed_resource::Entity::find().all(&self.db).await?;
        rows.into_iter().map(model_to_installed_resource).collect()
    }

    /// Find an installed resource by id.
    pub async fn resource(&self, id: &str) -> Result<Option<InstalledResource>, InventoryError> {
        let row = installed_resource::Entity::find_by_id(id).one(&self.db).await?;
        row.map(model_to_installed_resource).transpose()
    }

    // ── ServiceInstance ───────────────────────────────────────────────────────

    /// Register a service instance.
    #[instrument(name = "inventory.add_service", skip(self, svc))]
    pub async fn add_service(&self, svc: &ServiceInstance) -> Result<(), InventoryError> {
        service_instance::ActiveModel {
            id:             Set(svc.id.clone()),
            resource_id:    Set(svc.resource_id.clone()),
            instance_name:  Set(svc.instance_name.clone()),
            roles_provided: Set(serde_json::to_string(&svc.roles_provided)?),
            roles_required: Set(serde_json::to_string(&svc.roles_required)?),
            bridges:        Set(serde_json::to_string(&svc.bridges)?),
            variables:      Set(serde_json::to_string(&svc.variables)?),
            network:        Set(svc.network.clone()),
            status:         Set(serde_json::to_string(&svc.status)?),
            port:           Set(svc.port.map(|p| p as i32)),
            s3_paths:       Set(serde_json::to_string(&svc.s3_paths)?),
        }
        .insert(&self.db)
        .await?;
        Ok(())
    }

    /// All service instances providing a specific role.
    #[instrument(name = "inventory.services_with_role", skip(self))]
    pub async fn services_with_role(
        &self,
        role: &str,
    ) -> Result<Vec<ServiceInstance>, InventoryError> {
        // roles_provided is stored as JSON array; use LIKE for a simple search.
        // For production, a join table would be cleaner — acceptable for Phase G.
        let pattern = format!("%\"{}\"%" , role);
        let rows = service_instance::Entity::find()
            .filter(service_instance::Column::RolesProvided.like(pattern))
            .all(&self.db)
            .await?;
        rows.into_iter().map(model_to_service_instance).collect()
    }

    /// All service instances.
    pub async fn all_services(&self) -> Result<Vec<ServiceInstance>, InventoryError> {
        let rows = service_instance::Entity::find().all(&self.db).await?;
        rows.into_iter().map(model_to_service_instance).collect()
    }

    // ── BridgeInstance ────────────────────────────────────────────────────────

    /// Register a bridge instance.
    #[instrument(name = "inventory.add_bridge", skip(self, bridge))]
    pub async fn add_bridge(&self, bridge: &BridgeInstance) -> Result<(), InventoryError> {
        bridge_instance::ActiveModel {
            id:               Set(bridge.id.clone()),
            bridge_id:        Set(bridge.bridge_id.clone()),
            role:             Set(bridge.role.as_str().to_owned()),
            service_instance: Set(bridge.service_instance.clone()),
            api_base_url:     Set(bridge.api_base_url.clone()),
            status:           Set(serde_json::to_string(&bridge.status)?),
        }
        .insert(&self.db)
        .await?;
        Ok(())
    }

    /// All bridge instances serving a specific role.
    #[instrument(name = "inventory.bridges_for_role", skip(self))]
    pub async fn bridges_for_role(
        &self,
        role: &str,
    ) -> Result<Vec<BridgeInstance>, InventoryError> {
        let rows = bridge_instance::Entity::find()
            .filter(bridge_instance::Column::Role.eq(role))
            .all(&self.db)
            .await?;
        rows.into_iter().map(model_to_bridge_instance).collect()
    }

    /// All bridge instances.
    pub async fn all_bridges(&self) -> Result<Vec<BridgeInstance>, InventoryError> {
        let rows = bridge_instance::Entity::find().all(&self.db).await?;
        rows.into_iter().map(model_to_bridge_instance).collect()
    }

    // ── Status updates ────────────────────────────────────────────────────────

    /// Update the runtime status of an installed resource.
    #[instrument(name = "inventory.set_resource_status", skip(self, status))]
    pub async fn set_resource_status(
        &self,
        id: &str,
        status: &ResourceStatus,
    ) -> Result<(), InventoryError> {
        let model = installed_resource::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| InventoryError::NotFound { id: id.to_owned() })?;
        let mut active: installed_resource::ActiveModel = model.into();
        active.status = Set(serde_json::to_string(status)?);
        active.update(&self.db).await?;
        Ok(())
    }

    /// Update the runtime status of a service instance by instance name.
    #[instrument(name = "inventory.set_service_status_by_name", skip(self, status))]
    pub async fn set_service_status_by_name(
        &self,
        instance_name: &str,
        status: &ServiceStatus,
    ) -> Result<(), InventoryError> {
        let model = service_instance::Entity::find()
            .filter(service_instance::Column::InstanceName.eq(instance_name))
            .one(&self.db)
            .await?
            .ok_or_else(|| InventoryError::NotFound { id: instance_name.to_owned() })?;
        let mut active: service_instance::ActiveModel = model.into();
        active.status = Set(serde_json::to_string(status)?);
        active.update(&self.db).await?;
        Ok(())
    }

    // ── Upsert helpers ────────────────────────────────────────────────────────

    /// Insert a resource or update its status if it is already installed.
    ///
    /// Idempotent: calling this on every deploy is safe.
    #[instrument(name = "inventory.upsert_resource", skip(self, resource))]
    pub async fn upsert_resource(&self, resource: &InstalledResource) -> Result<(), InventoryError> {
        match self.install(resource).await {
            Ok(()) => Ok(()),
            Err(InventoryError::AlreadyInstalled { .. }) => {
                self.set_resource_status(&resource.id, &resource.status).await
            }
            Err(e) => Err(e),
        }
    }

    /// Register a service instance or update an existing one with the same name.
    ///
    /// Idempotent: if a service instance with the same `instance_name` already
    /// exists, its status and roles are updated instead of inserting a duplicate.
    #[instrument(name = "inventory.upsert_service", skip(self, svc))]
    pub async fn upsert_service(&self, svc: &ServiceInstance) -> Result<(), InventoryError> {
        let existing = service_instance::Entity::find()
            .filter(service_instance::Column::InstanceName.eq(&svc.instance_name))
            .one(&self.db)
            .await?;

        if let Some(model) = existing {
            let mut active: service_instance::ActiveModel = model.into();
            active.status         = Set(serde_json::to_string(&svc.status)?);
            active.roles_provided = Set(serde_json::to_string(&svc.roles_provided)?);
            active.roles_required = Set(serde_json::to_string(&svc.roles_required)?);
            active.network        = Set(svc.network.clone());
            active.port           = Set(svc.port.map(|p| p as i32));
            active.update(&self.db).await?;
        } else {
            self.add_service(svc).await?;
        }
        Ok(())
    }
}

// ── Conversion helpers ────────────────────────────────────────────────────────

fn model_to_installed_resource(
    m: installed_resource::Model,
) -> Result<InstalledResource, InventoryError> {
    Ok(InstalledResource {
        id:            m.id,
        resource_type: serde_json::from_str(&m.resource_type)?,
        version:       m.version,
        channel:       serde_json::from_str(&m.channel)?,
        installed_at:  m.installed_at,
        status:        serde_json::from_str(&m.status)?,
        config_path:   m.config_path,
        data_path:     m.data_path,
        validation:    serde_json::from_str(&m.validation)?,
    })
}

fn model_to_service_instance(
    m: service_instance::Model,
) -> Result<ServiceInstance, InventoryError> {
    Ok(ServiceInstance {
        id:             m.id,
        resource_id:    m.resource_id,
        instance_name:  m.instance_name,
        roles_provided: serde_json::from_str(&m.roles_provided)?,
        roles_required: serde_json::from_str(&m.roles_required)?,
        bridges:        serde_json::from_str(&m.bridges)?,
        variables:      serde_json::from_str(&m.variables)?,
        network:        m.network,
        status:         serde_json::from_str(&m.status)?,
        port:           m.port.map(|p| p as u16),
        s3_paths:       serde_json::from_str(&m.s3_paths)?,
    })
}

fn model_to_bridge_instance(
    m: bridge_instance::Model,
) -> Result<BridgeInstance, InventoryError> {
    Ok(BridgeInstance {
        id:               m.id,
        bridge_id:        m.bridge_id,
        role:             Role::new(m.role),
        service_instance: m.service_instance,
        api_base_url:     m.api_base_url,
        status:           serde_json::from_str(&m.status)?,
    })
}
