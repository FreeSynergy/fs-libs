use dioxus::prelude::*;

/// Props for [`Table`].
#[derive(Props, Clone, PartialEq)]
pub struct TableProps {
    /// Column header labels.
    pub columns: Vec<String>,
    /// Row data — each inner `Vec<String>` maps to one row.
    pub rows: Vec<Vec<String>>,
    /// Optional accessible caption for the table.
    pub caption: Option<String>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A data table with column headers and row data.
#[component]
pub fn Table(props: TableProps) -> Element {
    rsx! {
        div { class: "fs-table-container {props.class}",
            table { class: "fs-table",
                if let Some(cap) = &props.caption {
                    caption { class: "fs-table__caption", "{cap}" }
                }
                thead {
                    tr {
                        for col in &props.columns {
                            th { class: "fs-table__th", scope: "col", "{col}" }
                        }
                    }
                }
                tbody {
                    for row in &props.rows {
                        tr { class: "fs-table__row",
                            for cell in row {
                                td { class: "fs-table__td", "{cell}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
