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
    pub fn database() -> FsTag {
        FsTag::new("package.database")
    }
    pub fn cache() -> FsTag {
        FsTag::new("package.cache")
    }
    pub fn storage() -> FsTag {
        FsTag::new("package.storage")
    }
    pub fn backup() -> FsTag {
        FsTag::new("package.backup")
    }
    pub fn search() -> FsTag {
        FsTag::new("package.search")
    }
    pub fn security() -> FsTag {
        FsTag::new("package.security")
    }
    pub fn auth() -> FsTag {
        FsTag::new("package.auth")
    }
    pub fn iam() -> FsTag {
        FsTag::new("package.iam")
    }
    pub fn sso() -> FsTag {
        FsTag::new("package.sso")
    }
    pub fn vpn() -> FsTag {
        FsTag::new("package.vpn")
    }
    pub fn proxy() -> FsTag {
        FsTag::new("package.proxy")
    }
    pub fn firewall() -> FsTag {
        FsTag::new("package.firewall")
    }
    pub fn chat() -> FsTag {
        FsTag::new("package.chat")
    }
    pub fn email() -> FsTag {
        FsTag::new("package.email")
    }
    pub fn calendar() -> FsTag {
        FsTag::new("package.calendar")
    }
    pub fn video() -> FsTag {
        FsTag::new("package.video")
    }
    pub fn notifications() -> FsTag {
        FsTag::new("package.notifications")
    }
    pub fn wiki() -> FsTag {
        FsTag::new("package.wiki")
    }
    pub fn docs() -> FsTag {
        FsTag::new("package.docs")
    }
    pub fn git() -> FsTag {
        FsTag::new("package.git")
    }
    pub fn ci() -> FsTag {
        FsTag::new("package.ci")
    }
    pub fn tasks() -> FsTag {
        FsTag::new("package.tasks")
    }
    pub fn kanban() -> FsTag {
        FsTag::new("package.kanban")
    }
    pub fn notes() -> FsTag {
        FsTag::new("package.notes")
    }
    pub fn files() -> FsTag {
        FsTag::new("package.files")
    }
    pub fn ai() -> FsTag {
        FsTag::new("package.ai")
    }
    pub fn llm() -> FsTag {
        FsTag::new("package.llm")
    }
    pub fn ml() -> FsTag {
        FsTag::new("package.ml")
    }
    pub fn media() -> FsTag {
        FsTag::new("package.media")
    }
    pub fn photos() -> FsTag {
        FsTag::new("package.photos")
    }
    pub fn music() -> FsTag {
        FsTag::new("package.music")
    }
    pub fn video_streaming() -> FsTag {
        FsTag::new("package.video-streaming")
    }
    pub fn monitoring() -> FsTag {
        FsTag::new("package.monitoring")
    }
    pub fn metrics() -> FsTag {
        FsTag::new("package.metrics")
    }
    pub fn logging() -> FsTag {
        FsTag::new("package.logging")
    }
    pub fn tracing() -> FsTag {
        FsTag::new("package.tracing")
    }
    pub fn alerting() -> FsTag {
        FsTag::new("package.alerting")
    }
    pub fn dev() -> FsTag {
        FsTag::new("package.dev")
    }
    pub fn registry() -> FsTag {
        FsTag::new("package.registry")
    }
    pub fn package_manager() -> FsTag {
        FsTag::new("package.package-manager")
    }
    pub fn desktop() -> FsTag {
        FsTag::new("package.desktop")
    }
    pub fn widget() -> FsTag {
        FsTag::new("package.widget")
    }
    pub fn theme() -> FsTag {
        FsTag::new("package.theme")
    }
    pub fn browser() -> FsTag {
        FsTag::new("package.browser")
    }
    pub fn system() -> FsTag {
        FsTag::new("package.system")
    }
    pub fn network() -> FsTag {
        FsTag::new("package.network")
    }
    pub fn dns() -> FsTag {
        FsTag::new("package.dns")
    }
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
