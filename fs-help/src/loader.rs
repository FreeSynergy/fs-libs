// fs-help/loader.rs — HelpLoader: reads help/*.toml files into HelpTopic objects.
//
// Directory layout expected per package:
//
//   help/
//     overview.toml      — general topics + links (TOML structure)
//     fields.toml        — per-field context topics
//     en/
//       overview.ftl     — English Fluent translations
//       fields.ftl
//     de/
//       overview.ftl     — German Fluent translations
//       fields.ftl
//
// The TOML files define structure (topic IDs, URLs, keywords).
// The .ftl files provide translated text (resolved at render time via fs-i18n).
//
// TOML format example:
//
//   package_kind = "external"
//
//   [[topic]]
//   id          = "kanidm"
//   title_key   = "help-kanidm-title"
//   content_key = "help-kanidm-body"
//   keywords    = ["kanidm", "iam", "identity"]
//   search_query = "kanidm identity provider tutorial setup"
//
//   [[topic.link]]
//   kind = "website"
//   url  = "https://kanidm.com"
//
//   [[topic.link]]
//   kind = "git"
//   url  = "https://github.com/kanidm/kanidm"

use std::path::Path;
use std::sync::Arc;

use serde::Deserialize;

use crate::kind::{ExternalHelp, HelpKindArc, InternalHelp};
use crate::link::{HelpLink, HelpLinkKind};
use crate::topic::HelpTopic;

// ── TOML data model ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct HelpFile {
    /// Applies to all topics in this file. Defaults to `internal`.
    #[serde(default)]
    package_kind: PackageKindTag,

    #[serde(default, rename = "topic")]
    topics: Vec<TopicDef>,
}

#[derive(Debug, Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum PackageKindTag {
    #[default]
    Internal,
    External,
}

#[derive(Debug, Deserialize)]
struct TopicDef {
    id:          String,
    title_key:   String,
    content_key: String,

    #[serde(default)]
    keywords:    Vec<String>,

    #[serde(default)]
    related:     Vec<String>,

    /// Links specific to this topic (overrides file-level links when set).
    #[serde(default, rename = "link")]
    links:       Vec<LinkDef>,

    /// Tutorial search query (external packages only).
    #[serde(default)]
    search_query: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LinkDef {
    kind:      LinkKindTag,
    url:       String,
    #[serde(default)]
    label_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LinkKindTag {
    Website,
    Docs,
    Git,
}

// ── HelpLoader ────────────────────────────────────────────────────────────────

/// Loads help topics from a package's `help/` directory.
///
/// Call [`HelpLoader::load_dir`] once per installed package and register the
/// returned topics into a [`crate::HelpSystem`].
pub struct HelpLoader;

impl HelpLoader {
    /// Load all `.toml` files from `help_dir` and return the parsed topics.
    ///
    /// Sub-directories (language directories) are ignored.
    /// Unreadable or malformed files are silently skipped.
    pub fn load_dir(help_dir: &Path) -> Vec<HelpTopic> {
        let Ok(entries) = std::fs::read_dir(help_dir) else {
            return Vec::new();
        };

        let mut topics = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            // Only top-level .toml files (skip lang subdirectories)
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    topics.extend(Self::parse_toml(&text));
                }
            }
        }

        topics
    }

    /// Parse a single TOML string (content of one help file) into topics.
    ///
    /// Returns an empty `Vec` when the TOML is malformed.
    pub fn parse_toml(text: &str) -> Vec<HelpTopic> {
        let Ok(file) = toml::from_str::<HelpFile>(text) else {
            return Vec::new();
        };

        let pkg_kind = file.package_kind;

        file.topics.into_iter().map(|def| {
            let links = build_links(def.links);
            let kind: HelpKindArc = build_kind(pkg_kind, links, def.search_query);

            HelpTopic::new(def.id, def.title_key, def.content_key)
                .related(def.related)
                .keywords(def.keywords)
                .with_kind(kind)
        }).collect()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_links(defs: Vec<LinkDef>) -> Vec<HelpLink> {
    defs.into_iter().map(|l| {
        let kind = match l.kind {
            LinkKindTag::Website => HelpLinkKind::Website,
            LinkKindTag::Docs    => HelpLinkKind::Docs,
            LinkKindTag::Git     => HelpLinkKind::Git,
        };
        HelpLink::new(kind, l.url).with_label_key(l.label_key)
    }).collect()
}

fn build_kind(
    tag:   PackageKindTag,
    links: Vec<HelpLink>,
    search: Option<String>,
) -> HelpKindArc {
    match tag {
        PackageKindTag::External => Arc::new(
            ExternalHelp::new()
                .with_links(links)
                .with_search_opt(search),
        ),
        PackageKindTag::Internal => Arc::new(
            InternalHelp::new().with_links(links),
        ),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EXTERNAL_TOML: &str = r#"
package_kind = "external"

[[topic]]
id          = "kanidm"
title_key   = "help-kanidm-title"
content_key = "help-kanidm-body"
keywords    = ["kanidm", "iam"]
search_query = "kanidm identity provider tutorial"

[[topic.link]]
kind = "website"
url  = "https://kanidm.com"

[[topic.link]]
kind = "git"
url  = "https://github.com/kanidm/kanidm"
"#;

    const INTERNAL_TOML: &str = r#"
package_kind = "internal"

[[topic]]
id          = "store"
title_key   = "help-store-title"
content_key = "help-store-body"
keywords    = ["store", "install"]

[[topic.link]]
kind = "website"
url  = "https://freesynergy.net"

[[topic.link]]
kind = "docs"
url  = "https://docs.freesynergy.net/store"

[[topic]]
id          = "store.install"
title_key   = "help-store-install-title"
content_key = "help-store-install-body"

[[topic]]
id          = "store.install.package-id"
title_key   = "help-store-field-id-title"
content_key = "help-store-field-id-body"
"#;

    #[test]
    fn parses_external_topic() {
        let topics = HelpLoader::parse_toml(EXTERNAL_TOML);
        assert_eq!(topics.len(), 1);

        let t = &topics[0];
        assert_eq!(t.id, "kanidm");
        assert_eq!(t.kind.kind_name(), "external");
        assert_eq!(t.search_query(), Some("kanidm identity provider tutorial"));
        assert_eq!(t.links().len(), 2);
    }

    #[test]
    fn parses_internal_topics() {
        let topics = HelpLoader::parse_toml(INTERNAL_TOML);
        assert_eq!(topics.len(), 3);

        let root = topics.iter().find(|t| t.id == "store").unwrap();
        assert_eq!(root.kind.kind_name(), "internal");
        assert_eq!(root.search_query(), None);
        assert_eq!(root.links().len(), 2);

        // Field topic has no links
        let field = topics.iter().find(|t| t.id == "store.install.package-id").unwrap();
        assert_eq!(field.links().len(), 0);
    }

    #[test]
    fn returns_empty_on_malformed_toml() {
        let result = HelpLoader::parse_toml("not valid toml ===");
        assert!(result.is_empty());
    }
}
