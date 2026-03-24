//! Resource validation — checks every resource type for completeness and correctness.
//!
//! The `Validate` trait provides a uniform `validate()` method that updates the
//! resource's `meta.status` in place.  Callers (Store, Builder) call this after
//! loading a resource from disk or network and display the status permanently.

use super::{
    app::AppResource,
    bot::BotResource,
    bridge::BridgeResource,
    bundle::BundleResource,
    container::ContainerResource,
    messenger_adapter::MessengerAdapterResource,
    meta::ValidationStatus,
    theme::{AnimationSet, ButtonStyle, ColorScheme, CursorSet, FontSet, IconSet, StyleResource, TokenSet, WindowChrome},
    widget::WidgetResource,
};

// ── Validate trait ────────────────────────────────────────────────────────────

/// Validate a resource and update its `meta.status` in place.
pub trait Validate {
    fn validate(&mut self);
}

// ── AppResource ───────────────────────────────────────────────────────────────

impl Validate for AppResource {
    fn validate(&mut self) {
        self.meta.validate();
        if matches!(self.meta.status, ValidationStatus::Broken) {
            return;
        }
        let incomplete = self.platforms.is_empty()
            || self.binary_name.trim().is_empty();
        if incomplete {
            self.meta.status = ValidationStatus::Incomplete;
        }
    }
}

// ── ContainerResource ─────────────────────────────────────────────────────

impl Validate for ContainerResource {
    fn validate(&mut self) {
        self.meta.validate();
        if matches!(self.meta.status, ValidationStatus::Broken) {
            return;
        }
        // YAML must be non-empty
        if self.compose_yaml.trim().is_empty() {
            self.meta.status = ValidationStatus::Broken;
            return;
        }
        // At least one service
        if self.services.is_empty() {
            self.meta.status = ValidationStatus::Broken;
            return;
        }
        // Main service must have a healthcheck
        let main_has_healthcheck = self
            .services
            .iter()
            .filter(|s| s.is_main)
            .all(|s| s.healthcheck.is_some());
        // All required variables must have a description
        let vars_ok = self.variables.iter().filter(|v| v.required).all(|v| {
            !v.description.trim().is_empty()
        });
        if !main_has_healthcheck || !vars_ok {
            self.meta.status = ValidationStatus::Incomplete;
        }
    }
}

// ── WidgetResource ────────────────────────────────────────────────────────────

impl Validate for WidgetResource {
    fn validate(&mut self) {
        self.meta.validate();
        if matches!(self.meta.status, ValidationStatus::Broken) {
            return;
        }
        if self.data_sources.is_empty() && self.required_roles.is_empty() {
            self.meta.status = ValidationStatus::Incomplete;
        }
    }
}

// ── BotResource ───────────────────────────────────────────────────────────────

impl Validate for BotResource {
    fn validate(&mut self) {
        self.meta.validate();
        if matches!(self.meta.status, ValidationStatus::Broken) {
            return;
        }
        if self.channels.is_empty() {
            self.meta.status = ValidationStatus::Broken;
            return;
        }
        if self.tokens_required.is_empty() && self.triggers.is_empty() {
            self.meta.status = ValidationStatus::Incomplete;
        }
    }
}

// ── BridgeResource ────────────────────────────────────────────────────────────

impl Validate for BridgeResource {
    fn validate(&mut self) {
        self.meta.validate();
        if matches!(self.meta.status, ValidationStatus::Broken) {
            return;
        }
        if self.target_role.as_str().is_empty() || self.target_service.trim().is_empty() {
            self.meta.status = ValidationStatus::Broken;
            return;
        }
        // Check all required standard methods are mapped
        let required = self.target_role.required_bridge_methods();
        let mapped: std::collections::HashSet<&str> =
            self.methods.iter().map(|m| m.standard_name.as_str()).collect();
        let missing_required = required.iter().any(|m| !mapped.contains(m));
        if missing_required {
            self.meta.status = ValidationStatus::Incomplete;
        }
    }
}

// ── BundleResource ────────────────────────────────────────────────────────────

impl Validate for BundleResource {
    /// Basic structural validation.  Bundle reference resolution (checking that
    /// referenced ids exist in the store) is done at install time by the Store.
    fn validate(&mut self) {
        self.meta.validate();
        if matches!(self.meta.status, ValidationStatus::Broken) {
            return;
        }
        let has_content =
            !self.packages.is_empty() || self.theme.as_ref().is_some_and(|t| {
                t.color_scheme.is_some()
                    || t.style.is_some()
                    || t.font_set.is_some()
                    || t.icon_set.is_some()
                    || t.cursor_set.is_some()
                    || t.button_style.is_some()
                    || t.window_chrome.is_some()
                    || t.animation_set.is_some()
            });
        if !has_content {
            self.meta.status = ValidationStatus::Broken;
        }
    }
}

// ── Theme components ──────────────────────────────────────────────────────────

impl Validate for ColorScheme {
    fn validate(&mut self) {
        self.meta.validate();
        if !self.colors.is_complete() {
            self.meta.status = ValidationStatus::Broken;
        }
    }
}

impl Validate for StyleResource {
    fn validate(&mut self) {
        self.meta.validate();
        if !self.style.is_complete() {
            self.meta.status = ValidationStatus::Broken;
        }
    }
}

impl Validate for ButtonStyle {
    fn validate(&mut self) {
        self.meta.validate();
        if !self.tokens.is_complete() {
            self.meta.status = ValidationStatus::Broken;
        }
    }
}

impl Validate for WindowChrome {
    fn validate(&mut self) {
        self.meta.validate();
        if !self.tokens.is_complete() {
            self.meta.status = ValidationStatus::Broken;
        }
    }
}

impl Validate for AnimationSet {
    fn validate(&mut self) {
        self.meta.validate();
        if !self.tokens.is_complete() {
            self.meta.status = ValidationStatus::Broken;
        }
    }
}

impl Validate for FontSet {
    fn validate(&mut self) {
        self.meta.validate();
        if matches!(self.meta.status, ValidationStatus::Broken) {
            return;
        }
        if self.ui_fonts.is_empty() {
            self.meta.status = ValidationStatus::Incomplete;
        }
    }
}

impl Validate for CursorSet {
    fn validate(&mut self) {
        self.meta.validate();
        if matches!(self.meta.status, ValidationStatus::Broken) {
            return;
        }
        if self.cursors.is_empty() {
            self.meta.status = ValidationStatus::Incomplete;
        }
    }
}

impl Validate for IconSet {
    fn validate(&mut self) {
        self.meta.validate();
        if matches!(self.meta.status, ValidationStatus::Broken) {
            return;
        }
        if self.icons.is_empty() {
            self.meta.status = ValidationStatus::Broken;
        }
    }
}

// ── MessengerAdapterResource ──────────────────────────────────────────────────

impl Validate for MessengerAdapterResource {
    fn validate(&mut self) {
        self.meta.validate();
        if matches!(self.meta.status, ValidationStatus::Broken) {
            return;
        }
        if self.tokens_required.is_empty() {
            self.meta.status = ValidationStatus::Incomplete;
            return;
        }
        if self.supported_features.is_empty() {
            self.meta.status = ValidationStatus::Broken;
            return;
        }
        // Must at least support Send
        let can_send = self
            .supported_features
            .iter()
            .any(|f| matches!(f, super::messenger_adapter::ChannelFeature::Send));
        if !can_send {
            self.meta.status = ValidationStatus::Broken;
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::{
        bridge::{BridgeMethod, FieldMapping, HttpMethod},
        meta::{Dependency, ResourceMeta, ResourceType, Role, ValidationStatus},
    };
    use std::path::PathBuf;

    fn base_meta(rt: ResourceType) -> ResourceMeta {
        ResourceMeta {
            id: "test".into(),
            name: "Test".into(),
            summary: "A sufficiently long summary for store listings.".into(),
            description: "A medium-length description shown in the store detail view.".into(),
            description_file: PathBuf::from("help/en/description.ftl"),
            version: "1.0.0".into(),
            author: "FreeSynergy".into(),
            license: "MIT".into(),
            icon: PathBuf::from("icon.svg"),
            tags: vec!["tag".into()],
            resource_type: rt,
            dependencies: Vec::<Dependency>::new(),
            signature: None,
            status: ValidationStatus::Incomplete,
            source: None,
            platform: None,
        }
    }

    #[test]
    fn bridge_validate_ok_with_all_iam_methods() {
        let methods: Vec<BridgeMethod> = Role::new("iam").required_bridge_methods()
            .iter()
            .map(|name| BridgeMethod {
                standard_name:    name.to_string(),
                http_method:      HttpMethod::Post,
                endpoint:         "/v1/test".into(),
                request_mapping:  FieldMapping::identity(),
                response_mapping: FieldMapping::identity(),
            })
            .collect();

        let mut bridge = BridgeResource {
            meta:           base_meta(ResourceType::Bridge),
            target_role:    Role::new("iam"),
            target_service: "kanidm".into(),
            methods,
        };
        bridge.validate();
        assert_eq!(bridge.meta.status, ValidationStatus::Ok);
    }

    #[test]
    fn bridge_validate_incomplete_when_methods_missing() {
        let mut bridge = BridgeResource {
            meta:           base_meta(ResourceType::Bridge),
            target_role:    Role::new("iam"),
            target_service: "kanidm".into(),
            methods:        vec![],
        };
        bridge.validate();
        assert_eq!(bridge.meta.status, ValidationStatus::Incomplete);
    }

    #[test]
    fn bundle_validate_broken_when_empty() {
        let mut bundle = BundleResource {
            meta:     base_meta(ResourceType::Bundle),
            packages: vec![],
            theme:    None,
        };
        bundle.validate();
        assert_eq!(bundle.meta.status, ValidationStatus::Broken);
    }
}
