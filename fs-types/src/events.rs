// fs-types/src/events.rs — Payload structs for standard FreeSynergy bus events.
//
// Every struct here maps 1:1 to a topic constant in `fs-bus::topics`.
// Structs are serialized as JSON and carried in `Event::payload`.
//
// # Naming convention
//
//   <Domain><Entity>Event
//
//   e.g.  RegistryServiceRegisteredEvent  → topic "registry::service::registered"
//         SessionUserLoginEvent           → topic "session::user::login"

use serde::{Deserialize, Serialize};

// ── registry:: ────────────────────────────────────────────────────────────────

/// Payload for `registry::service::registered`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryServiceRegisteredEvent {
    /// Unique service identifier (e.g. `"fs-store"`).
    pub service_id: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Primary gRPC endpoint, e.g. `"http://localhost:50051"`.
    pub grpc_endpoint: String,
    /// Optional REST endpoint, e.g. `"http://localhost:8080"`.
    pub rest_endpoint: Option<String>,
    /// Capabilities this service provides, e.g. `["render.engine.iced"]`.
    pub capabilities: Vec<String>,
    /// Unix timestamp (seconds) when the service started.
    pub started_at: i64,
}

/// Payload for `registry::service::stopped`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryServiceStoppedEvent {
    /// Unique service identifier.
    pub service_id: String,
    /// Unix timestamp (seconds) when the service stopped.
    pub stopped_at: i64,
}

/// Payload for `registry::capability::added`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryCapabilityAddedEvent {
    /// Service that provides this capability.
    pub service_id: String,
    /// Capability identifier, e.g. `"db.engine.sqlite"`.
    pub capability_id: String,
}

/// Payload for `registry::capability::removed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryCapabilityRemovedEvent {
    /// Service that previously provided this capability.
    pub service_id: String,
    /// Capability identifier.
    pub capability_id: String,
}

// ── session:: ─────────────────────────────────────────────────────────────────

/// Payload for `session::user::login`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionUserLoginEvent {
    /// Unique user identifier (UUID or username, IAM-issued).
    pub user_id: String,
    /// Display name of the user.
    pub username: String,
    /// Session token (opaque, short-lived).
    pub session_id: String,
    /// Unix timestamp (seconds).
    pub logged_in_at: i64,
}

/// Payload for `session::user::logout`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionUserLogoutEvent {
    /// Unique user identifier.
    pub user_id: String,
    /// Session token that ended.
    pub session_id: String,
    /// Unix timestamp (seconds).
    pub logged_out_at: i64,
}

/// Payload for `session::app::opened`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAppOpenedEvent {
    /// Session in which the app was opened.
    pub session_id: String,
    /// Package ID of the app, e.g. `"fs-store"`.
    pub app_id: String,
    /// Window or instance identifier (opaque).
    pub window_id: String,
    /// Unix timestamp (seconds).
    pub opened_at: i64,
}

/// Payload for `session::app::closed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAppClosedEvent {
    /// Session in which the app was closed.
    pub session_id: String,
    /// Package ID of the app.
    pub app_id: String,
    /// Window or instance identifier.
    pub window_id: String,
    /// Unix timestamp (seconds).
    pub closed_at: i64,
}

// ── inventory:: ───────────────────────────────────────────────────────────────

/// Payload for `inventory::package::installed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageInstalledEvent {
    /// Package identifier, e.g. `"fs-store"`.
    pub package_id: String,
    /// Installed version string, e.g. `"0.2.1"`.
    pub version: String,
    /// Package type: `"program"`, `"adapter"`, `"artifact"`, `"bundle"`.
    pub package_type: String,
    /// Unix timestamp (seconds).
    pub installed_at: i64,
}

/// Payload for `inventory::package::removed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageRemovedEvent {
    /// Package identifier.
    pub package_id: String,
    /// Version that was removed.
    pub version: String,
    /// Unix timestamp (seconds).
    pub removed_at: i64,
}

/// Payload for `inventory::package::updated`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageUpdatedEvent {
    /// Package identifier.
    pub package_id: String,
    /// Previous version.
    pub from_version: String,
    /// New version.
    pub to_version: String,
    /// Unix timestamp (seconds).
    pub updated_at: i64,
}

// ── system:: ──────────────────────────────────────────────────────────────────

/// Payload for `system::health::degraded`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemHealthDegradedEvent {
    /// Component that is degraded, e.g. `"cpu"`, `"memory"`, `"disk"`.
    pub component: String,
    /// Current metric value (e.g. CPU usage %).
    pub current_value: f64,
    /// Configured threshold that was exceeded.
    pub threshold: f64,
    /// Unix timestamp (seconds).
    pub detected_at: i64,
}

/// Payload for `system::health::restored`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemHealthRestoredEvent {
    /// Component that recovered.
    pub component: String,
    /// Current (restored) metric value.
    pub current_value: f64,
    /// Unix timestamp (seconds).
    pub restored_at: i64,
}

/// Payload for `system::node::started`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemNodeStartedEvent {
    /// Node identifier (hostname or UUID).
    pub node_id: String,
    /// `FreeSynergy` version running on this node.
    pub fs_version: String,
    /// Unix timestamp (seconds).
    pub started_at: i64,
}

/// Payload for `system::node::stopping`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemNodeStoppingEvent {
    /// Node identifier.
    pub node_id: String,
    /// Unix timestamp (seconds).
    pub stopping_at: i64,
}

// ── auth:: ────────────────────────────────────────────────────────────────────

/// Payload for `auth::user::created`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthUserCreatedEvent {
    /// IAM-assigned user ID.
    pub user_id: String,
    /// Username / login name.
    pub username: String,
    /// Unix timestamp (seconds).
    pub created_at: i64,
}

/// Payload for `auth::user::deleted`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthUserDeletedEvent {
    /// IAM-assigned user ID.
    pub user_id: String,
    /// Unix timestamp (seconds).
    pub deleted_at: i64,
}

/// Payload for `auth::user::updated`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthUserUpdatedEvent {
    /// IAM-assigned user ID.
    pub user_id: String,
    /// Human-readable description of what changed, e.g. `"password"`.
    pub changed_field: String,
    /// Unix timestamp (seconds).
    pub updated_at: i64,
}

/// Payload for `auth::token::issued`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthTokenIssuedEvent {
    /// IAM-assigned user ID.
    pub user_id: String,
    /// Token type, e.g. `"access"`, `"refresh"`.
    pub token_type: String,
    /// Unix timestamp (seconds) when the token expires.
    pub expires_at: i64,
}

/// Payload for `auth::token::revoked`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthTokenRevokedEvent {
    /// IAM-assigned user ID.
    pub user_id: String,
    /// Token type that was revoked.
    pub token_type: String,
    /// Unix timestamp (seconds).
    pub revoked_at: i64,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_service_registered_round_trips() {
        let ev = RegistryServiceRegisteredEvent {
            service_id: "fs-store".into(),
            display_name: "Store".into(),
            grpc_endpoint: "http://localhost:50051".into(),
            rest_endpoint: Some("http://localhost:8080".into()),
            capabilities: vec!["store.install".into()],
            started_at: 1_000_000,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: RegistryServiceRegisteredEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn package_installed_round_trips() {
        let ev = PackageInstalledEvent {
            package_id: "fs-store".into(),
            version: "0.2.1".into(),
            package_type: "program".into(),
            installed_at: 1_000_000,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: PackageInstalledEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn system_health_degraded_round_trips() {
        let ev = SystemHealthDegradedEvent {
            component: "cpu".into(),
            current_value: 95.5,
            threshold: 90.0,
            detected_at: 1_000_000,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: SystemHealthDegradedEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn session_user_login_round_trips() {
        let ev = SessionUserLoginEvent {
            user_id: "user-1".into(),
            username: "kal".into(),
            session_id: "sess-abc".into(),
            logged_in_at: 1_000_000,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: SessionUserLoginEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn auth_token_issued_round_trips() {
        let ev = AuthTokenIssuedEvent {
            user_id: "user-1".into(),
            token_type: "access".into(),
            expires_at: 2_000_000,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: AuthTokenIssuedEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }
}
