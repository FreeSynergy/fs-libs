//! `ApiTags` — tags for API protocols and integration patterns.
//!
//! Used on Bridge and App resources to describe what protocols they speak.

use super::{FsTag, TagLibrary};

// ── ApiTags ───────────────────────────────────────────────────────────────────

/// Tags for API protocols, transports, and integration styles.
pub struct ApiTags;

const ALL_KEYS: &[&str] = &[
    // ── Protocols ─────────────────────────────────────────────────────────────
    "api.rest",
    "api.grpc",
    "api.graphql",
    "api.websocket",
    "api.webhook",
    "api.sse",
    // ── Auth protocols ────────────────────────────────────────────────────────
    "api.oidc",
    "api.oauth2",
    "api.saml",
    "api.ldap",
    "api.scim",
    // ── Messaging protocols ───────────────────────────────────────────────────
    "api.matrix",
    "api.smtp",
    "api.imap",
    "api.xmpp",
    "api.activitypub",
    // ── Data formats ──────────────────────────────────────────────────────────
    "api.json",
    "api.xml",
    "api.csv",
    "api.protobuf",
    // ── Integration patterns ──────────────────────────────────────────────────
    "api.bridge",
    "api.federation",
    "api.sync",
];

impl ApiTags {
    #[must_use]
    pub fn rest() -> FsTag {
        FsTag::new("api.rest")
    }
    #[must_use]
    pub fn grpc() -> FsTag {
        FsTag::new("api.grpc")
    }
    #[must_use]
    pub fn graphql() -> FsTag {
        FsTag::new("api.graphql")
    }
    #[must_use]
    pub fn websocket() -> FsTag {
        FsTag::new("api.websocket")
    }
    #[must_use]
    pub fn webhook() -> FsTag {
        FsTag::new("api.webhook")
    }
    #[must_use]
    pub fn sse() -> FsTag {
        FsTag::new("api.sse")
    }
    #[must_use]
    pub fn oidc() -> FsTag {
        FsTag::new("api.oidc")
    }
    #[must_use]
    pub fn oauth2() -> FsTag {
        FsTag::new("api.oauth2")
    }
    #[must_use]
    pub fn saml() -> FsTag {
        FsTag::new("api.saml")
    }
    #[must_use]
    pub fn ldap() -> FsTag {
        FsTag::new("api.ldap")
    }
    #[must_use]
    pub fn scim() -> FsTag {
        FsTag::new("api.scim")
    }
    #[must_use]
    pub fn matrix() -> FsTag {
        FsTag::new("api.matrix")
    }
    #[must_use]
    pub fn smtp() -> FsTag {
        FsTag::new("api.smtp")
    }
    #[must_use]
    pub fn imap() -> FsTag {
        FsTag::new("api.imap")
    }
    #[must_use]
    pub fn xmpp() -> FsTag {
        FsTag::new("api.xmpp")
    }
    #[must_use]
    pub fn activitypub() -> FsTag {
        FsTag::new("api.activitypub")
    }
    #[must_use]
    pub fn json() -> FsTag {
        FsTag::new("api.json")
    }
    #[must_use]
    pub fn xml() -> FsTag {
        FsTag::new("api.xml")
    }
    #[must_use]
    pub fn csv() -> FsTag {
        FsTag::new("api.csv")
    }
    #[must_use]
    pub fn protobuf() -> FsTag {
        FsTag::new("api.protobuf")
    }
    #[must_use]
    pub fn bridge() -> FsTag {
        FsTag::new("api.bridge")
    }
    #[must_use]
    pub fn federation() -> FsTag {
        FsTag::new("api.federation")
    }
    #[must_use]
    pub fn sync() -> FsTag {
        FsTag::new("api.sync")
    }
}

impl TagLibrary for ApiTags {
    fn all_keys() -> &'static [&'static str] {
        ALL_KEYS
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_tags_known() {
        assert!(ApiTags::contains("api.rest"));
        assert!(ApiTags::contains("api.oidc"));
        assert!(ApiTags::contains("api.matrix"));
    }

    #[test]
    fn factory_fns_in_library() {
        let tags = vec![
            ApiTags::rest(),
            ApiTags::grpc(),
            ApiTags::oidc(),
            ApiTags::matrix(),
            ApiTags::federation(),
        ];
        for tag in &tags {
            assert!(ApiTags::contains(tag.key()), "{}", tag.key());
        }
    }
}
