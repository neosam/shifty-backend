//! `NavBtn` — square 28×28 mono-glyph button used for prev/next navigation
//! in week and year contexts.
//!
//! Distinct from [`super::btn::Btn`] because its layout constraints (no
//! padding, fixed glyph-only square) are incompatible with the regular
//! button's padding/typography defaults.

use dioxus::prelude::*;

use crate::base_types::ImStr;

const BASE: &str = "w-7 h-7 inline-flex items-center justify-center border border-border-strong rounded-md font-mono text-ink-soft hover:bg-surface-alt";

/// Builds the class string for a `NavBtn`.
pub(crate) fn build_class(disabled: bool) -> String {
    let mut out = String::with_capacity(160);
    out.push_str(BASE);
    if disabled {
        out.push(' ');
        out.push_str("opacity-50 cursor-not-allowed");
    }
    out
}

#[derive(Props, Clone, PartialEq)]
pub struct NavBtnProps {
    /// The mono glyph rendered inside the button (`"‹"`, `"›"`, `"▾"`, etc.).
    pub glyph: ImStr,

    #[props(default = false)]
    pub disabled: bool,

    #[props(!optional, default = None)]
    pub on_click: Option<EventHandler<()>>,

    /// Optional accessible label. When `None`, no `aria-label` attribute
    /// is rendered.
    #[props(!optional, default = None)]
    pub aria_label: Option<ImStr>,
}

#[component]
pub fn NavBtn(props: NavBtnProps) -> Element {
    let class = build_class(props.disabled);
    let disabled = props.disabled;
    let on_click = props.on_click.clone();

    rsx! {
        button {
            class: "{class}",
            disabled,
            "aria-label": props.aria_label.as_ref().map(|s| s.as_str()),
            onclick: move |evt| {
                evt.prevent_default();
                if disabled {
                    return;
                }
                if let Some(handler) = &on_click {
                    handler.call(());
                }
            },
            "{props.glyph}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_class_includes_size_and_layout() {
        let c = build_class(false);
        assert!(c.contains("w-7"), "missing w-7: {c}");
        assert!(c.contains("h-7"), "missing h-7: {c}");
        assert!(c.contains("inline-flex"));
        assert!(c.contains("items-center"));
        assert!(c.contains("justify-center"));
    }

    #[test]
    fn build_class_includes_token_styling() {
        let c = build_class(false);
        assert!(
            c.contains("border-border-strong"),
            "missing strong border: {c}"
        );
        assert!(c.contains("rounded-md"));
        assert!(c.contains("font-mono"));
        assert!(c.contains("text-ink-soft"));
        assert!(c.contains("hover:bg-surface-alt"));
    }

    #[test]
    fn build_class_disabled_adds_opacity_and_cursor() {
        let c = build_class(true);
        assert!(c.contains("opacity-50"), "missing opacity-50: {c}");
        assert!(
            c.contains("cursor-not-allowed"),
            "missing cursor-not-allowed: {c}"
        );
    }

    #[test]
    fn build_class_enabled_omits_disabled_classes() {
        let c = build_class(false);
        assert!(!c.contains("opacity-50"));
        assert!(!c.contains("cursor-not-allowed"));
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn renders_glyph_in_button() {
        fn app() -> Element {
            rsx! { NavBtn { glyph: ImStr::from("‹") } }
        }
        let html = render(app);
        assert!(html.starts_with("<button"), "expected button root: {html}");
        assert!(html.contains("‹"), "glyph not rendered: {html}");
    }

    #[test]
    fn aria_label_propagates_when_provided() {
        fn app() -> Element {
            rsx! {
                NavBtn {
                    glyph: ImStr::from("‹"),
                    aria_label: Some(ImStr::from("Previous week")),
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains(r#"aria-label="Previous week""#),
            "aria-label missing: {html}"
        );
    }

    #[test]
    fn aria_label_omitted_when_none() {
        fn app() -> Element {
            rsx! { NavBtn { glyph: ImStr::from("›") } }
        }
        let html = render(app);
        assert!(
            !html.contains("aria-label"),
            "unexpected aria-label: {html}"
        );
    }

    #[test]
    fn disabled_render_includes_disabled_attr_and_classes() {
        fn app() -> Element {
            rsx! {
                NavBtn { glyph: ImStr::from("›"), disabled: true }
            }
        }
        let html = render(app);
        assert!(
            html.contains("disabled"),
            "missing disabled attribute: {html}"
        );
        assert!(html.contains("opacity-50"));
        assert!(html.contains("cursor-not-allowed"));
    }
}
