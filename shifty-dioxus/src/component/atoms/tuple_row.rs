//! `TupleRow` — label-value layout used in detail panels and summaries.
//!
//! The value slot is a generic [`Element`], so callers can render mono-font
//! numbers, color dots, or any other inline content. The mono font for
//! numeric values is intentionally the caller's responsibility; this atom
//! only provides the row layout.

use dioxus::prelude::*;

use crate::base_types::ImStr;

const ROW_BASE: &str = "flex items-baseline justify-between gap-3 py-1.5 border-b border-border";

/// Builds the class string for the outer row container.
pub(crate) fn row_class(dim: bool) -> String {
    let mut out = String::with_capacity(96);
    out.push_str(ROW_BASE);
    out.push(' ');
    if dim {
        out.push_str("text-[13px] text-ink-muted");
    } else {
        out.push_str("text-[13px]");
    }
    out
}

/// Builds the class string for the label span.
pub(crate) fn label_class(dim: bool) -> &'static str {
    if dim {
        // The whole row already carries text-ink-muted; keep label inheriting.
        "text-ink-muted"
    } else {
        "text-ink-soft"
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TupleRowProps {
    pub label: ImStr,
    pub value: Element,

    /// Render the row in a lower-contrast `text-ink-muted` color for
    /// secondary fields.
    #[props(default = false)]
    pub dim: bool,

    /// Optional description rendered below the label/value row in
    /// `text-xs text-ink-muted`.
    #[props(!optional, default = None)]
    pub description: Option<Element>,
}

#[component]
pub fn TupleRow(props: TupleRowProps) -> Element {
    let row = row_class(props.dim);
    let label = label_class(props.dim);

    rsx! {
        div { class: "flex flex-col",
            div { class: "{row}",
                span { class: "{label}", "{props.label}" }
                span { class: "tuple-row-value", {props.value} }
            }
            if let Some(description) = props.description {
                div { class: "text-xs text-ink-muted", {description} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_class_default_includes_layout_and_border() {
        let c = row_class(false);
        assert!(c.contains("flex"));
        assert!(c.contains("items-baseline"));
        assert!(c.contains("justify-between"));
        assert!(c.contains("gap-3"));
        assert!(c.contains("py-1.5"));
        assert!(c.contains("border-b"));
        assert!(c.contains("border-border"));
        assert!(c.contains("text-[13px]"));
    }

    #[test]
    fn row_class_dim_adds_ink_muted() {
        let c = row_class(true);
        assert!(c.contains("text-ink-muted"), "dim row missing text-ink-muted: {c}");
    }

    #[test]
    fn row_class_default_omits_ink_muted() {
        let c = row_class(false);
        assert!(!c.contains("text-ink-muted"));
    }

    #[test]
    fn label_class_default_uses_ink_soft() {
        assert_eq!(label_class(false), "text-ink-soft");
    }

    #[test]
    fn label_class_dim_uses_ink_muted() {
        assert_eq!(label_class(true), "text-ink-muted");
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn description_renders_when_present() {
        fn app() -> Element {
            rsx! {
                TupleRow {
                    label: ImStr::from("Total"),
                    value: rsx! { span { "42" } },
                    description: Some(rsx! { "context note" }),
                }
            }
        }
        let html = render(app);
        assert!(html.contains("text-xs"));
        assert!(html.contains("text-ink-muted"));
        assert!(html.contains("context note"), "description not rendered: {html}");
    }

    #[test]
    fn description_omitted_when_none() {
        fn app() -> Element {
            rsx! {
                TupleRow {
                    label: ImStr::from("Total"),
                    value: rsx! { span { "42" } },
                }
            }
        }
        let html = render(app);
        assert!(!html.contains("text-xs"), "description div leaked: {html}");
    }

    #[test]
    fn renders_label_and_value() {
        fn app() -> Element {
            rsx! {
                TupleRow {
                    label: ImStr::from("Hours"),
                    value: rsx! { span { class: "font-mono", "12.5" } },
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Hours"));
        assert!(html.contains("12.5"));
        assert!(html.contains("font-mono"));
        // Layout classes present
        assert!(html.contains("border-b"));
        assert!(html.contains("text-[13px]"));
    }
}
