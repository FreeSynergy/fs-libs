// fs-components/display.rs — Table, Progress, CodeBlock.

use dioxus::prelude::*;

use crate::card::BadgeVariant;

// ── TableColumn ───────────────────────────────────────────────────────────────

/// Column descriptor for `Table`.
#[derive(Clone, PartialEq)]
pub struct TableColumn {
    /// Machine key (used as React-style key in loops).
    pub key: String,
    /// Human-readable header label.
    pub label: String,
    /// Optional CSS width for this column (e.g. `"120px"` or `"20%"`).
    pub width: Option<String>,
}

impl TableColumn {
    /// Shorthand constructor.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self { key: key.into(), label: label.into(), width: None }
    }

    /// Attach a CSS width.
    pub fn with_width(mut self, w: impl Into<String>) -> Self {
        self.width = Some(w.into());
        self
    }
}

// ── Table ─────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct TableProps {
    /// Column definitions.
    pub columns: Vec<TableColumn>,
    /// Row data — each row is a `Vec<String>` aligned with `columns`.
    pub rows: Vec<Vec<String>>,
    /// Alternate row background colours when `true`.
    #[props(default = false)]
    pub striped: bool,
}

/// Styled HTML table with a header row and optional stripe pattern.
///
/// ```no_run
/// Table {
///     columns: vec![TableColumn::new("name", "Name"), TableColumn::new("status", "Status")],
///     rows: vec![vec!["my-app".into(), "running".into()]],
///     striped: true,
/// }
/// ```
#[component]
pub fn Table(props: TableProps) -> Element {
    let header_style = "background: var(--fs-color-bg-sidebar, #0f172a); \
                        border-bottom: 1px solid var(--fs-color-border-default, #334155);";

    rsx! {
        div {
            class: "fs-table-wrap",
            style: "width: 100%; overflow-x: auto;",

            table {
                class: "fs-table",
                style: "width: 100%; border-collapse: collapse; font-size: 13px; \
                        color: var(--fs-color-text-primary, #e2e8f0);",

                thead {
                    tr {
                        style: "{header_style}",
                        for col in &props.columns {
                            th {
                                key: "{col.key}",
                                style: "text-align: left; padding: 9px 12px; \
                                        font-weight: 600; font-size: 12px; \
                                        color: var(--fs-color-text-secondary, #94a3b8); \
                                        white-space: nowrap; \
                                        width: {col.width.as_deref().unwrap_or(\"auto\")};",
                                "{col.label}"
                            }
                        }
                    }
                }

                tbody {
                    for (row_idx, row) in props.rows.iter().enumerate() {
                        {
                            let stripe_bg = if props.striped && row_idx % 2 == 1 {
                                "background: rgba(255,255,255,0.02);"
                            } else {
                                ""
                            };

                            rsx! {
                                tr {
                                    key: "{row_idx}",
                                    style: "border-bottom: 1px solid var(--fs-color-border-default, #334155); \
                                            transition: background 0.1s; {stripe_bg}",

                                    for (col_idx, cell) in row.iter().enumerate() {
                                        td {
                                            key: "{col_idx}",
                                            style: "padding: 9px 12px;",
                                            "{cell}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Progress ──────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ProgressProps {
    /// Progress fraction in `[0.0, 1.0]`.
    pub value: f64,
    /// Optional text label shown above the bar.
    pub label: Option<String>,
    /// Colour variant (reuses `BadgeVariant`).
    #[props(default)]
    pub variant: BadgeVariant,
}

/// Accessible progress bar.
///
/// ```no_run
/// Progress { value: 0.72, label: Some("Deploying…".to_string()), variant: BadgeVariant::Info }
/// ```
#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let pct = (props.value.clamp(0.0, 1.0) * 100.0) as u32;

    let fill_color = props.variant.fill_color();

    rsx! {
        div {
            class: "fs-progress-wrap",
            style: "display: flex; flex-direction: column; gap: 4px;",

            if let Some(lbl) = &props.label {
                div {
                    style: "display: flex; justify-content: space-between; \
                            font-size: 12px; \
                            color: var(--fs-color-text-secondary, #94a3b8);",
                    span { "{lbl}" }
                    span { "{pct}%" }
                }
            }

            div {
                role: "progressbar",
                aria_valuenow: "{pct}",
                aria_valuemin: "0",
                aria_valuemax: "100",
                class: "fs-progress",
                style: "width: 100%; height: 6px; border-radius: 999px; \
                        background: var(--fs-color-border-default, #334155); \
                        overflow: hidden;",

                div {
                    style: "height: 100%; width: {pct}%; border-radius: 999px; \
                            background: {fill_color}; \
                            transition: width 0.3s ease;",
                }
            }
        }
    }
}

// ── CodeBlock ─────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct CodeBlockProps {
    /// The source code string to display.
    pub code: String,
    /// Optional language hint shown as a small label in the corner.
    pub language: Option<String>,
    /// Shows a copy-to-clipboard button when `true`.
    #[props(default = false)]
    pub copyable: bool,
}

/// Preformatted code block with a dark surface, monospace font, and optional
/// copy button.
///
/// This component does not perform syntax highlighting — it renders raw text in
/// a `<pre>` block. Use a dedicated highlighting library at the application layer
/// if rich colouring is required.
///
/// ```no_run
/// CodeBlock {
///     code: "fn main() { println!(\"Hello\"); }",
///     language: Some("rust".to_string()),
///     copyable: true,
/// }
/// ```
#[component]
pub fn CodeBlock(props: CodeBlockProps) -> Element {
    let mut copied = use_signal(|| false);
    let code_clone = props.code.clone();

    rsx! {
        div {
            class: "fs-code-block",
            style: "position: relative; background: var(--fs-color-bg-sidebar, #0f172a); \
                    border: 1px solid var(--fs-color-border-default, #334155); \
                    border-radius: 8px; overflow: hidden;",

            // Top bar
            div {
                style: "display: flex; align-items: center; justify-content: space-between; \
                        padding: 6px 12px; \
                        border-bottom: 1px solid var(--fs-color-border-default, #334155);",

                span {
                    style: "font-size: 11px; \
                            color: var(--fs-color-text-muted, #64748b);",
                    "{props.language.as_deref().unwrap_or(\"code\")}"
                }

                if props.copyable {
                    button {
                        style: "background: none; border: none; cursor: pointer; \
                                font-size: 11px; padding: 2px 8px; border-radius: 4px; \
                                color: var(--fs-color-text-secondary, #94a3b8); \
                                border: 1px solid var(--fs-color-border-default, #334155);",
                        onclick: move |_| {
                            // Clipboard API is handled externally; here we just toggle state.
                            copied.set(true);
                        },
                        if *copied.read() { "Copied!" } else { "Copy" }
                    }
                }
            }

            // Code body
            pre {
                style: "margin: 0; padding: 14px 16px; overflow-x: auto; \
                        font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', \
                                     'Menlo', monospace; \
                        font-size: 12px; line-height: 1.6; \
                        color: var(--fs-color-text-primary, #e2e8f0);",
                code {
                    style: "background: none; padding: 0; border: none;",
                    "{code_clone}"
                }
            }
        }
    }
}
