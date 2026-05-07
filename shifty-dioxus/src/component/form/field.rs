//! `Field` — semantic `<label>` wrapper for the redesigned form atoms.
//!
//! Layout: column-flex with the uppercase 11 px label on top, the input
//! slot in the middle, and an optional hint or error line below the input.
//! Hint and error are mutually exclusive — when both are provided, the
//! error preempts the hint. Use the `span` prop in grid layouts to make
//! the field span two columns.

use dioxus::prelude::*;

use crate::base_types::ImStr;

#[derive(Props, Clone, PartialEq)]
pub struct FieldProps {
    pub label: ImStr,
    pub children: Element,

    #[props(!optional, default = None)]
    pub hint: Option<ImStr>,

    #[props(!optional, default = None)]
    pub error: Option<ImStr>,

    /// Number of grid columns this field should span. Only `Some(2)` has
    /// an effect; any other value is rendered as a single-column field.
    #[props(!optional, default = None)]
    pub span: Option<u8>,
}

const LABEL_CLASSES: &str = "text-micro text-ink-soft uppercase";

#[component]
pub fn Field(props: FieldProps) -> Element {
    let span_style = match props.span {
        Some(2) => "grid-column:span 2;display:flex;flex-direction:column;gap:4px;min-width:0;",
        _ => "display:flex;flex-direction:column;gap:4px;min-width:0;",
    };

    let show_error = props.error.is_some();

    rsx! {
        label {
            style: "{span_style}",
            span { class: "{LABEL_CLASSES}", "{props.label}" }
            { props.children }

            if let Some(error) = props.error.as_ref() {
                span { class: "text-micro text-bad", "{error}" }
            } else if let Some(hint) = props.hint.as_ref() {
                if !show_error {
                    span { class: "text-micro text-ink-muted", "{hint}" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn label_text_appears_in_uppercase_inksoft_class() {
        fn app() -> Element {
            rsx! {
                Field { label: ImStr::from("Email"),
                    input { "type": "text" }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Email"), "missing label text: {html}");
        assert!(html.contains("uppercase"), "label not uppercase: {html}");
        assert!(
            html.contains("text-ink-soft"),
            "label missing token color: {html}"
        );
        // text-micro bakes in font-weight: 600 (semibold) and font-size: 11px,
        // so the label inherits both from the token.
        assert!(html.contains("text-micro"), "label wrong size: {html}");
    }

    #[test]
    fn renders_label_element() {
        fn app() -> Element {
            rsx! {
                Field { label: ImStr::from("X"),
                    input { "type": "text" }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("<label"), "expected <label> element: {html}");
    }

    #[test]
    fn hint_renders_when_no_error() {
        fn app() -> Element {
            rsx! {
                Field {
                    label: ImStr::from("Name"),
                    hint: Some(ImStr::from("Optional")),
                    input { "type": "text" }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Optional"), "hint missing: {html}");
        assert!(html.contains("text-ink-muted"), "hint colour wrong: {html}");
    }

    #[test]
    fn error_preempts_hint() {
        fn app() -> Element {
            rsx! {
                Field {
                    label: ImStr::from("Name"),
                    hint: Some(ImStr::from("Optional")),
                    error: Some(ImStr::from("Required")),
                    input { "type": "text" }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Required"), "error missing: {html}");
        assert!(
            !html.contains("Optional"),
            "hint should be hidden when error: {html}"
        );
        assert!(html.contains("text-bad"), "error colour missing: {html}");
    }

    #[test]
    fn error_alone_renders_in_bad_colour() {
        fn app() -> Element {
            rsx! {
                Field {
                    label: ImStr::from("Name"),
                    error: Some(ImStr::from("Required")),
                    input { "type": "text" }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Required"), "error missing: {html}");
        assert!(html.contains("text-bad"), "error colour missing: {html}");
    }

    #[test]
    fn no_hint_and_no_error_renders_neither() {
        fn app() -> Element {
            rsx! {
                Field { label: ImStr::from("Name"),
                    input { "type": "text" }
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("text-ink-muted"),
            "unexpected hint markup: {html}"
        );
        assert!(
            !html.contains("text-bad"),
            "unexpected error markup: {html}"
        );
    }

    #[test]
    fn span_two_adds_grid_column_span() {
        fn app() -> Element {
            rsx! {
                Field { label: ImStr::from("X"), span: Some(2u8),
                    input { "type": "text" }
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("grid-column:span 2"),
            "missing span 2 style: {html}"
        );
    }

    #[test]
    fn no_span_does_not_emit_grid_column() {
        fn app() -> Element {
            rsx! {
                Field { label: ImStr::from("X"),
                    input { "type": "text" }
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("grid-column:span 2"),
            "unexpected span style: {html}"
        );
    }
}
