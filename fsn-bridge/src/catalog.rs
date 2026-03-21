//! Pre-built bridge definitions for common services.
//!
//! Each function returns a fully-populated `BridgeResource` that can be
//! registered in the Inventory, published to the Store, or used directly.

use fsn_types::resources::{
    bridge::{BridgeMethod, BridgeResource, FieldMapping, HttpMethod},
    meta::{ResourceMeta, ResourceType, Role, ValidationStatus},
};
use std::path::PathBuf;

// ── Shared helpers ────────────────────────────────────────────────────────────

fn meta(id: &str, name: &str, description: &str, target_service: &str) -> ResourceMeta {
    ResourceMeta {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        version: "1.0.0".into(),
        author: "FreeSynergy".into(),
        license: "MIT".into(),
        icon: PathBuf::from("bridge.svg"),
        tags: vec!["bridge".into(), target_service.into()],
        resource_type: ResourceType::Bridge,
        dependencies: vec![],
        signature: None,
        status: ValidationStatus::Ok,
        source: None,
        platform: None,
    }
}

fn method(
    name: &str,
    http_method: HttpMethod,
    endpoint: &str,
    req: FieldMapping,
    res: FieldMapping,
) -> BridgeMethod {
    BridgeMethod {
        standard_name: name.into(),
        http_method,
        endpoint: endpoint.into(),
        request_mapping: req,
        response_mapping: res,
    }
}

// ── Kanidm IAM Bridge ─────────────────────────────────────────────────────────

/// Bridge that maps the standardized `iam` role API to Kanidm's REST API.
///
/// Tested against Kanidm ≥ 1.4.  The Kanidm API base URL is typically
/// `https://kanidm.example.com:8443`.
pub fn kanidm_iam_bridge() -> BridgeResource {
    BridgeResource {
        meta: meta(
            "kanidm-iam-bridge",
            "Kanidm IAM Bridge",
            "Maps the FreeSynergy IAM role API to Kanidm's REST v1 API.",
            "kanidm",
        ),
        target_role: Role::new("iam"),
        target_service: "kanidm".into(),
        methods: vec![
            // user.create → POST /v1/person
            method(
                "user.create", HttpMethod::Post, "/v1/person",
                FieldMapping::identity().map("username", "attrs.name").map("email", "attrs.mail"),
                FieldMapping::identity().map("attrs.uuid", "id").map("attrs.name", "username").map("attrs.mail", "email"),
            ),
            // user.get → GET /v1/person/:id
            method(
                "user.get", HttpMethod::Get, "/v1/person",
                FieldMapping::identity(),
                FieldMapping::identity().map("attrs.uuid", "id").map("attrs.name", "username").map("attrs.mail", "email"),
            ),
            // user.list → GET /v1/person
            method(
                "user.list", HttpMethod::Get, "/v1/person",
                FieldMapping::identity(),
                FieldMapping::identity(),
            ),
            // user.update → PATCH /v1/person/:id
            method(
                "user.update", HttpMethod::Patch, "/v1/person",
                FieldMapping::identity().map("email", "attrs.mail"),
                FieldMapping::identity(),
            ),
            // user.delete → DELETE /v1/person/:id
            method(
                "user.delete", HttpMethod::Delete, "/v1/person",
                FieldMapping::identity(),
                FieldMapping::identity(),
            ),
            // group.create → POST /v1/group
            method(
                "group.create", HttpMethod::Post, "/v1/group",
                FieldMapping::identity().map("name", "attrs.name"),
                FieldMapping::identity().map("attrs.uuid", "id").map("attrs.name", "name"),
            ),
            // group.list → GET /v1/group
            method(
                "group.list", HttpMethod::Get, "/v1/group",
                FieldMapping::identity(),
                FieldMapping::identity(),
            ),
            // group.add_member → POST /v1/group/:id/_attr/member
            method(
                "group.add_member", HttpMethod::Post, "/v1/group/_attr/member",
                FieldMapping::identity().map("user_id", "member"),
                FieldMapping::identity(),
            ),
        ],
    }
}

// ── Outline Wiki Bridge ───────────────────────────────────────────────────────

/// Bridge that maps the standardized `wiki` role API to Outline's REST API.
///
/// Tested against Outline ≥ 0.76.  Requires an Outline API token set as
/// `Authorization: Bearer <token>` in the requests.
pub fn outline_wiki_bridge() -> BridgeResource {
    BridgeResource {
        meta: meta(
            "outline-wiki-bridge",
            "Outline Wiki Bridge",
            "Maps the FreeSynergy wiki role API to Outline's REST API.",
            "outline",
        ),
        target_role: Role::new("wiki"),
        target_service: "outline".into(),
        methods: vec![
            // page.create → POST /api/documents.create
            method(
                "page.create", HttpMethod::Post, "/api/documents.create",
                FieldMapping::identity().map("title", "title").map("content", "text").map("collection_id", "collectionId"),
                FieldMapping::identity().map("data.id", "id").map("data.title", "title").map("data.text", "content").map("data.url", "url").map("data.updatedAt", "updated_at"),
            ),
            // page.get → POST /api/documents.info
            method(
                "page.get", HttpMethod::Post, "/api/documents.info",
                FieldMapping::identity().map("id", "id"),
                FieldMapping::identity().map("data.id", "id").map("data.title", "title").map("data.text", "content").map("data.url", "url").map("data.updatedAt", "updated_at"),
            ),
            // page.list → POST /api/documents.list
            method(
                "page.list", HttpMethod::Post, "/api/documents.list",
                FieldMapping::identity(),
                FieldMapping::identity(),
            ),
            // page.search → POST /api/documents.search
            method(
                "page.search", HttpMethod::Post, "/api/documents.search",
                FieldMapping::identity().map("query", "query"),
                FieldMapping::identity(),
            ),
        ],
    }
}

// ── Forgejo Git Bridge ────────────────────────────────────────────────────────

/// Bridge that maps the standardized `git` role API to Forgejo's REST API.
///
/// Compatible with Forgejo ≥ 7.x and Gitea ≥ 1.21 (identical API).
/// Requires a Forgejo access token set as `Authorization: token <token>`.
pub fn forgejo_git_bridge() -> BridgeResource {
    BridgeResource {
        meta: meta(
            "forgejo-git-bridge",
            "Forgejo Git Bridge",
            "Maps the FreeSynergy git role API to Forgejo's Gitea-compatible REST API.",
            "forgejo",
        ),
        target_role: Role::new("git"),
        target_service: "forgejo".into(),
        methods: vec![
            // repo.create → POST /api/v1/user/repos
            method(
                "repo.create", HttpMethod::Post, "/api/v1/user/repos",
                FieldMapping::identity().map("private", "private").map("description", "description"),
                FieldMapping::identity().map("id", "id").map("name", "name").map("full_name", "full_name").map("clone_url", "clone_url").map("private", "private").map("default_branch", "default_branch"),
            ),
            // repo.list → GET /api/v1/repos/search
            method(
                "repo.list", HttpMethod::Get, "/api/v1/repos/search",
                FieldMapping::identity(),
                FieldMapping::identity(),
            ),
            // repo.get → GET /api/v1/repos/:owner/:repo
            method(
                "repo.get", HttpMethod::Get, "/api/v1/repos",
                FieldMapping::identity(),
                FieldMapping::identity().map("id", "id").map("name", "name").map("full_name", "full_name").map("clone_url", "clone_url").map("private", "private").map("default_branch", "default_branch"),
            ),
            // commit.list → GET /api/v1/repos/:owner/:repo/git/commits
            method(
                "commit.list", HttpMethod::Get, "/api/v1/repos/commits",
                FieldMapping::identity(),
                FieldMapping::identity().map("sha", "sha").map("commit.message", "message").map("commit.author.name", "author").map("commit.author.date", "timestamp"),
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fsn_types::resources::validator::{required_methods_for_role, Validate};

    #[test]
    fn kanidm_bridge_validates_ok() {
        let mut b = kanidm_iam_bridge();
        b.validate();
        assert_eq!(b.meta.status, ValidationStatus::Ok);
    }

    #[test]
    fn outline_bridge_validates_ok() {
        let mut b = outline_wiki_bridge();
        b.validate();
        assert_eq!(b.meta.status, ValidationStatus::Ok);
    }

    #[test]
    fn forgejo_bridge_validates_ok() {
        let mut b = forgejo_git_bridge();
        b.validate();
        assert_eq!(b.meta.status, ValidationStatus::Ok);
    }

    #[test]
    fn kanidm_bridge_has_all_required_iam_methods() {
        let b = kanidm_iam_bridge();
        let required = required_methods_for_role("iam");
        let mapped: std::collections::HashSet<&str> =
            b.methods.iter().map(|m| m.standard_name.as_str()).collect();
        for req in required {
            assert!(mapped.contains(req), "Missing IAM method: {req}");
        }
    }

    #[test]
    fn outline_bridge_has_all_required_wiki_methods() {
        let b = outline_wiki_bridge();
        let required = required_methods_for_role("wiki");
        let mapped: std::collections::HashSet<&str> =
            b.methods.iter().map(|m| m.standard_name.as_str()).collect();
        for req in required {
            assert!(mapped.contains(req), "Missing wiki method: {req}");
        }
    }

    #[test]
    fn forgejo_bridge_has_all_required_git_methods() {
        let b = forgejo_git_bridge();
        let required = required_methods_for_role("git");
        let mapped: std::collections::HashSet<&str> =
            b.methods.iter().map(|m| m.standard_name.as_str()).collect();
        for req in required {
            assert!(mapped.contains(req), "Missing git method: {req}");
        }
    }
}
