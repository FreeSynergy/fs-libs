//! `PackageTags` — standard tags for store packages.
//!
//! These are the content and function tags used to describe what a package
//! does.  Every tag key maps to an i18n key `"tag.<key>"` that provides
//! the translated display name in the Store UI.
//!
//! # Adding new tags
//!
//! Add the key to `ALL_KEYS` and the corresponding i18n entries to all
//! language snippet files (`en`, `de`, …).  The naming convention is:
//! `package.<domain>` — lowercase, no underscores.

use super::{FsTag, TagLibrary};

// ── PackageTags ───────────────────────────────────────────────────────────────

/// Standard tags that describe the content and function of store packages.
pub struct PackageTags;

const ALL_KEYS: &[&str] = &[
    // ── Data & Storage ────────────────────────────────────────────────────────
    "package.database",
    "package.cache",
    "package.storage",
    "package.backup",
    "package.search",
    // ── Identity & Security ───────────────────────────────────────────────────
    "package.security",
    "package.auth",
    "package.iam",
    "package.sso",
    "package.vpn",
    "package.proxy",
    "package.firewall",
    // ── Communication ─────────────────────────────────────────────────────────
    "package.chat",
    "package.email",
    "package.calendar",
    "package.video",
    "package.notifications",
    // ── Collaboration & Content ───────────────────────────────────────────────
    "package.wiki",
    "package.docs",
    "package.git",
    "package.ci",
    "package.tasks",
    "package.kanban",
    "package.notes",
    "package.files",
    // ── AI & ML ───────────────────────────────────────────────────────────────
    "package.ai",
    "package.llm",
    "package.ml",
    // ── Media ─────────────────────────────────────────────────────────────────
    "package.media",
    "package.photos",
    "package.music",
    "package.video-streaming",
    // ── Monitoring & Operations ───────────────────────────────────────────────
    "package.monitoring",
    "package.metrics",
    "package.logging",
    "package.tracing",
    "package.alerting",
    // ── Development ───────────────────────────────────────────────────────────
    "package.dev",
    "package.registry",
    "package.package-manager",
    // ── Desktop & UI ──────────────────────────────────────────────────────────
    "package.desktop",
    "package.widget",
    "package.theme",
    "package.browser",
    // ── System ────────────────────────────────────────────────────────────────
    "package.system",
    "package.network",
    "package.dns",
    "package.maps",
];

impl PackageTags {
    #[must_use]
    pub fn database() -> FsTag {
        FsTag::new("package.database")
    }
    #[must_use]
    pub fn cache() -> FsTag {
        FsTag::new("package.cache")
    }
    #[must_use]
    pub fn storage() -> FsTag {
        FsTag::new("package.storage")
    }
    #[must_use]
    pub fn backup() -> FsTag {
        FsTag::new("package.backup")
    }
    #[must_use]
    pub fn search() -> FsTag {
        FsTag::new("package.search")
    }
    #[must_use]
    pub fn security() -> FsTag {
        FsTag::new("package.security")
    }
    #[must_use]
    pub fn auth() -> FsTag {
        FsTag::new("package.auth")
    }
    #[must_use]
    pub fn iam() -> FsTag {
        FsTag::new("package.iam")
    }
    #[must_use]
    pub fn sso() -> FsTag {
        FsTag::new("package.sso")
    }
    #[must_use]
    pub fn vpn() -> FsTag {
        FsTag::new("package.vpn")
    }
    #[must_use]
    pub fn proxy() -> FsTag {
        FsTag::new("package.proxy")
    }
    #[must_use]
    pub fn firewall() -> FsTag {
        FsTag::new("package.firewall")
    }
    #[must_use]
    pub fn chat() -> FsTag {
        FsTag::new("package.chat")
    }
    #[must_use]
    pub fn email() -> FsTag {
        FsTag::new("package.email")
    }
    #[must_use]
    pub fn calendar() -> FsTag {
        FsTag::new("package.calendar")
    }
    #[must_use]
    pub fn video() -> FsTag {
        FsTag::new("package.video")
    }
    #[must_use]
    pub fn notifications() -> FsTag {
        FsTag::new("package.notifications")
    }
    #[must_use]
    pub fn wiki() -> FsTag {
        FsTag::new("package.wiki")
    }
    #[must_use]
    pub fn docs() -> FsTag {
        FsTag::new("package.docs")
    }
    #[must_use]
    pub fn git() -> FsTag {
        FsTag::new("package.git")
    }
    #[must_use]
    pub fn ci() -> FsTag {
        FsTag::new("package.ci")
    }
    #[must_use]
    pub fn tasks() -> FsTag {
        FsTag::new("package.tasks")
    }
    #[must_use]
    pub fn kanban() -> FsTag {
        FsTag::new("package.kanban")
    }
    #[must_use]
    pub fn notes() -> FsTag {
        FsTag::new("package.notes")
    }
    #[must_use]
    pub fn files() -> FsTag {
        FsTag::new("package.files")
    }
    #[must_use]
    pub fn ai() -> FsTag {
        FsTag::new("package.ai")
    }
    #[must_use]
    pub fn llm() -> FsTag {
        FsTag::new("package.llm")
    }
    #[must_use]
    pub fn ml() -> FsTag {
        FsTag::new("package.ml")
    }
    #[must_use]
    pub fn media() -> FsTag {
        FsTag::new("package.media")
    }
    #[must_use]
    pub fn photos() -> FsTag {
        FsTag::new("package.photos")
    }
    #[must_use]
    pub fn music() -> FsTag {
        FsTag::new("package.music")
    }
    #[must_use]
    pub fn video_streaming() -> FsTag {
        FsTag::new("package.video-streaming")
    }
    #[must_use]
    pub fn monitoring() -> FsTag {
        FsTag::new("package.monitoring")
    }
    #[must_use]
    pub fn metrics() -> FsTag {
        FsTag::new("package.metrics")
    }
    #[must_use]
    pub fn logging() -> FsTag {
        FsTag::new("package.logging")
    }
    #[must_use]
    pub fn tracing() -> FsTag {
        FsTag::new("package.tracing")
    }
    #[must_use]
    pub fn alerting() -> FsTag {
        FsTag::new("package.alerting")
    }
    #[must_use]
    pub fn dev() -> FsTag {
        FsTag::new("package.dev")
    }
    #[must_use]
    pub fn registry() -> FsTag {
        FsTag::new("package.registry")
    }
    #[must_use]
    pub fn package_manager() -> FsTag {
        FsTag::new("package.package-manager")
    }
    #[must_use]
    pub fn desktop() -> FsTag {
        FsTag::new("package.desktop")
    }
    #[must_use]
    pub fn widget() -> FsTag {
        FsTag::new("package.widget")
    }
    #[must_use]
    pub fn theme() -> FsTag {
        FsTag::new("package.theme")
    }
    #[must_use]
    pub fn browser() -> FsTag {
        FsTag::new("package.browser")
    }
    #[must_use]
    pub fn system() -> FsTag {
        FsTag::new("package.system")
    }
    #[must_use]
    pub fn network() -> FsTag {
        FsTag::new("package.network")
    }
    #[must_use]
    pub fn dns() -> FsTag {
        FsTag::new("package.dns")
    }
    #[must_use]
    pub fn maps() -> FsTag {
        FsTag::new("package.maps")
    }
}

impl TagLibrary for PackageTags {
    fn all_keys() -> &'static [&'static str] {
        ALL_KEYS
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_factory_fns_return_known_keys() {
        let tags = vec![
            PackageTags::database(),
            PackageTags::cache(),
            PackageTags::security(),
            PackageTags::auth(),
            PackageTags::chat(),
            PackageTags::ai(),
            PackageTags::llm(),
            PackageTags::monitoring(),
            PackageTags::git(),
        ];
        for tag in &tags {
            assert!(
                PackageTags::contains(tag.key()),
                "Key not in library: {}",
                tag.key()
            );
        }
    }

    #[test]
    fn contains_known_and_rejects_unknown() {
        assert!(PackageTags::contains("package.database"));
        assert!(PackageTags::contains("package.ai"));
        assert!(!PackageTags::contains("package.unknown"));
        assert!(!PackageTags::contains("platform.linux")); // wrong library
    }

    #[test]
    fn all_returns_all_keys() {
        let tags = PackageTags::all();
        assert_eq!(tags.len(), ALL_KEYS.len());
    }
}
