//! `PlatformTags` — system and platform requirement tags.
//!
//! These tags describe what platform or system feature a package needs.
//! The Store combines them with `SysInfo` to show compatibility warnings.
//!
//! # Naming convention
//!
//! Platform/OS tags:  `platform.<os>`
//! Feature tags:      `requires.<feature>`

use super::{FsTag, TagLibrary};

// ── PlatformTags ──────────────────────────────────────────────────────────────

/// Tags for OS platforms and required system features.
pub struct PlatformTags;

const ALL_KEYS: &[&str] = &[
    // ── Platforms ─────────────────────────────────────────────────────────────
    "platform.linux",
    "platform.macos",
    "platform.windows",
    "platform.any",
    // ── Required system features ──────────────────────────────────────────────
    "requires.systemd",
    "requires.pam",
    "requires.launchd",
    "requires.windows-services",
    "requires.podman",
    "requires.docker",
    "requires.git",
    "requires.ssh",
    "requires.smartctl",
];

impl PlatformTags {
    pub fn linux() -> FsTag {
        FsTag::new("platform.linux")
    }
    pub fn macos() -> FsTag {
        FsTag::new("platform.macos")
    }
    pub fn windows() -> FsTag {
        FsTag::new("platform.windows")
    }
    pub fn any() -> FsTag {
        FsTag::new("platform.any")
    }
    pub fn requires_systemd() -> FsTag {
        FsTag::new("requires.systemd")
    }
    pub fn requires_pam() -> FsTag {
        FsTag::new("requires.pam")
    }
    pub fn requires_launchd() -> FsTag {
        FsTag::new("requires.launchd")
    }
    pub fn requires_windows_services() -> FsTag {
        FsTag::new("requires.windows-services")
    }
    pub fn requires_podman() -> FsTag {
        FsTag::new("requires.podman")
    }
    pub fn requires_docker() -> FsTag {
        FsTag::new("requires.docker")
    }
    pub fn requires_git() -> FsTag {
        FsTag::new("requires.git")
    }
    pub fn requires_ssh() -> FsTag {
        FsTag::new("requires.ssh")
    }
    pub fn requires_smartctl() -> FsTag {
        FsTag::new("requires.smartctl")
    }
}

impl TagLibrary for PlatformTags {
    fn all_keys() -> &'static [&'static str] {
        ALL_KEYS
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_tags_known() {
        assert!(PlatformTags::contains("platform.linux"));
        assert!(PlatformTags::contains("platform.macos"));
        assert!(PlatformTags::contains("platform.windows"));
        assert!(PlatformTags::contains("requires.systemd"));
        assert!(PlatformTags::contains("requires.podman"));
    }

    #[test]
    fn factory_fns_in_library() {
        let tags = vec![
            PlatformTags::linux(),
            PlatformTags::macos(),
            PlatformTags::requires_systemd(),
            PlatformTags::requires_podman(),
            PlatformTags::requires_git(),
        ];
        for tag in &tags {
            assert!(PlatformTags::contains(tag.key()), "{}", tag.key());
        }
    }
}
