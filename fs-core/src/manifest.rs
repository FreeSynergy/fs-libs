// Shared manifest parsing infrastructure for FreeSynergy managers.
//
// All managers that read TOML-style manifests with `[[section]]` blocks
// use the same parsing loop. Concrete managers implement `ManifestBuilder`
// to provide the section header and field mapping; the generic
// `parse_manifest_sections` function handles the loop.
//
// This is the Rust equivalent of an abstract base class with an extension
// point: SetBase holds the common data, ManifestBuilder is the abstract
// "apply extra fields" method that each concrete manager overrides.

// ── SetBase ───────────────────────────────────────────────────────────────────

/// Common fields shared by every set-type manifest entry
/// (icon sets, cursor sets, theme sets, …).
///
/// Embed this in your concrete proto type and delegate base-field
/// parsing to [`SetBase::apply_field`].
#[derive(Debug, Default, Clone)]
pub struct SetBase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source_repo_id: String,
    pub builtin: bool,
}

impl SetBase {
    /// Tries to apply a field that belongs to the common base.
    ///
    /// Returns `true` if the key was recognised and applied.
    /// Returns `false` for unknown keys — those are extra fields that the
    /// concrete builder must handle itself.
    pub fn apply_field(&mut self, key: &str, val: String) -> bool {
        match key {
            "id" => self.id = val,
            "name" => self.name = val,
            "description" => self.description = val,
            "source_repo_id" | "source" => self.source_repo_id = val,
            "builtin" => self.builtin = val == "true",
            _ => return false,
        }
        true
    }

    /// `true` if the minimum required field `id` is non-empty.
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
    }
}

// ── ManifestBuilder ───────────────────────────────────────────────────────────

/// Extension point for manifest parsers — the abstract "derived class" hook.
///
/// Implement this on your builder type. The generic parsing loop in
/// [`parse_manifest_sections`] calls `apply_field` for every `key = value`
/// line and `build` at the end of each `[[section]]` block.
///
/// Handle your extra fields first; fall through to `base.apply_field` for
/// the common ones (`id`, `name`, `description`, `source_repo_id`, `builtin`).
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Default)]
/// struct IconSetBuilder {
///     base:              SetBase,
///     has_dark_variants: bool,
/// }
///
/// impl ManifestBuilder for IconSetBuilder {
///     type Output = IconSetProto;
///
///     fn apply_field(&mut self, key: &str, val: String) {
///         match key {
///             "has_dark_variants" => self.has_dark_variants = val == "true",
///             _                   => { self.base.apply_field(key, val); }
///         }
///     }
///
///     fn build(self) -> Option<IconSetProto> {
///         self.base.is_valid().then(|| IconSetProto {
///             base:              self.base,
///             has_dark_variants: self.has_dark_variants,
///         })
///     }
/// }
/// ```
pub trait ManifestBuilder: Default {
    /// The fully constructed output type produced by [`build`](Self::build).
    type Output;

    /// Apply one `key = value` pair from the manifest.
    ///
    /// Handle extra (type-specific) keys here; delegate unknown keys to
    /// `self.base.apply_field(key, val)`.
    fn apply_field(&mut self, key: &str, val: String);

    /// Finalise the current section into an output value.
    ///
    /// Return `None` to silently discard incomplete sections (e.g. missing `id`).
    fn build(self) -> Option<Self::Output>;
}

// ── Generic parser ────────────────────────────────────────────────────────────

/// Parses a manifest string with `[[section_header]]` blocks.
///
/// One `B::Output` is produced per valid section. Incomplete sections
/// (where `B::build` returns `None`) are silently skipped.
///
/// ```toml
/// [[set]]
/// id   = "fs-icons"
/// name = "FreeSynergy Icons"
///
/// [[set]]
/// id   = "community-icons"
/// name = "Community Icons"
/// ```
pub fn parse_manifest_sections<B: ManifestBuilder>(
    content: &str,
    section_header: &str,
) -> Vec<B::Output> {
    let mut out = Vec::new();
    let mut current: Option<B> = None;

    for raw in content.lines() {
        let line = raw.trim();
        if line == section_header {
            if let Some(b) = current.take() {
                if let Some(item) = b.build() {
                    out.push(item);
                }
            }
            current = Some(B::default());
            continue;
        }
        if let Some(ref mut b) = current {
            if let Some((key, val)) = kv(line) {
                b.apply_field(key, val);
            }
        }
    }

    if let Some(b) = current {
        if let Some(item) = b.build() {
            out.push(item);
        }
    }

    out
}

// ── Key-value parser ──────────────────────────────────────────────────────────

/// Parses a single `key = "value"` or `key = value` manifest line.
///
/// Handles any amount of whitespace around `=` and optional quotes on the
/// value. Returns `None` for blank lines, comments (`#`), and section
/// headers (`[…]`).
pub fn kv(line: &str) -> Option<(&str, String)> {
    let (lhs, rhs) = line.split_once('=')?;
    let key = lhs.trim();
    let val = rhs.trim().trim_matches('"').to_string();
    Some((key, val))
}
