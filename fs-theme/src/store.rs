// fs-theme — Store theme helpers: validation and CSS variable prefixing.

/// CSS variable names (without `--` and without prefix) that every Store theme must define.
pub const REQUIRED_VARS: &[&str] = &[
    "bg-base", "bg-surface", "bg-elevated", "bg-card", "bg-input",
    "text-primary", "text-secondary", "text-muted",
    "primary", "primary-hover", "primary-text",
    "accent",
    "success", "warning", "error",
    "border", "border-focus",
];

/// Injects a CSS variable prefix into all `--` declarations in `css`.
///
/// `--bg-base` → `--{prefix}-bg-base`.
/// Variables that already start with `--{prefix}-` are left untouched.
///
/// # Example
/// ```
/// let store_css = ":root { --bg-base: #0c1222; --text-primary: #e8edf5; }";
/// let desktop_css = fs_theme::prefix_theme_css(store_css, "fsn");
/// assert!(desktop_css.contains("--fs-bg-base: #0c1222"));
/// ```
pub fn prefix_theme_css(css: &str, prefix: &str) -> String {
    let mut out = String::with_capacity(css.len() + css.len() / 4);
    let mut chars = css.chars().peekable();
    let guard_inner = format!("{prefix}-");

    while let Some(c) = chars.next() {
        out.push(c);
        if c == '-' && chars.peek() == Some(&'-') {
            out.push(chars.next().unwrap()); // second `-`
            // Read ahead to check if already prefixed.
            let mut ahead = String::new();
            for _ in 0..guard_inner.len() {
                if let Some(nc) = chars.next() {
                    ahead.push(nc);
                } else {
                    break;
                }
            }
            if ahead == guard_inner {
                out.push_str(&ahead);
            } else {
                out.push_str(&guard_inner);
                out.push_str(&ahead);
            }
        }
    }
    out
}

/// Returns the names of required variables missing from `css` (without prefix).
///
/// An empty result means the CSS is valid for Store upload.
pub fn validate_theme_vars(css: &str) -> Vec<&'static str> {
    REQUIRED_VARS
        .iter()
        .copied()
        .filter(|var| !css.contains(&format!("--{var}")))
        .collect()
}
