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
        div { class: "fsn-table-container {props.class}",
            table { class: "fsn-table",
                if let Some(cap) = &props.caption {
                    caption { class: "fsn-table__caption", "{cap}" }
                }
                thead {
                    tr {
                        for col in &props.columns {
                            th { class: "fsn-table__th", scope: "col", "{col}" }
                        }
                    }
                }
                tbody {
                    for row in &props.rows {
                        tr { class: "fsn-table__row",
                            for cell in row {
                                td { class: "fsn-table__td", "{cell}" }
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
