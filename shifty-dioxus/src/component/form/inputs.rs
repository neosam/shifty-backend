//! Token-based form input atoms used inside [`Field`](super::Field).
//!
//! All three atoms (`FormTextInput`, `FormSelectInput`, `FormTextareaInput`)
//! share the `form-input` class so the global focus rule in `input.css`
//! applies the accent focus ring without per-component styling. The
//! `Form*` prefix keeps these names from colliding with the legacy
//! components in [`crate::component::base_components`] until the cleanup
//! change drops both the prefix and the legacy versions.

use dioxus::prelude::*;

use crate::base_types::ImStr;

const SHARED_INPUT_CLASSES: &str =
    "h-[34px] px-[10px] border border-border-strong rounded-md bg-surface text-ink text-[13px] w-full min-w-0 form-input";

#[derive(Props, Clone, PartialEq)]
pub struct FormTextInputProps {
    pub value: ImStr,

    #[props(!optional, default = None)]
    pub on_change: Option<EventHandler<ImStr>>,

    #[props(default = false)]
    pub disabled: bool,

    #[props(!optional, default = None)]
    pub placeholder: Option<ImStr>,

    /// Native input `type` attribute. Defaults to `"text"`.
    #[props(default = ImStr::from("text"))]
    pub input_type: ImStr,
}

#[component]
pub fn FormTextInput(props: FormTextInputProps) -> Element {
    let placeholder_attr = props.placeholder.as_ref().map(|p| p.to_string());
    let input_type = props.input_type.clone();
    let on_change = props.on_change.clone();
    let disabled = props.disabled;

    rsx! {
        input {
            class: "{SHARED_INPUT_CLASSES}",
            r#type: "{input_type}",
            value: "{props.value}",
            disabled,
            placeholder: placeholder_attr,
            oninput: move |event| {
                if let Some(handler) = &on_change {
                    handler.call(ImStr::from(event.data.value()));
                }
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct FormSelectInputProps {
    pub children: Element,

    #[props(default = false)]
    pub disabled: bool,

    #[props(!optional, default = None)]
    pub placeholder: Option<ImStr>,

    #[props(!optional, default = None)]
    pub on_change: Option<EventHandler<ImStr>>,
}

const SELECT_EXTRA_STYLE: &str =
    "appearance:none;-webkit-appearance:none;padding-right:28px;\
     background-image:url(\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'><path d='M1 1l4 4 4-4' stroke='%236b7382' stroke-width='1.5' fill='none' stroke-linecap='round'/></svg>\");\
     background-repeat:no-repeat;background-position:right 10px center;";

#[component]
pub fn FormSelectInput(props: FormSelectInputProps) -> Element {
    let on_change = props.on_change.clone();
    let disabled = props.disabled;
    let placeholder_attr = props.placeholder.as_ref().map(|p| p.to_string());

    rsx! {
        select {
            class: "{SHARED_INPUT_CLASSES}",
            style: "{SELECT_EXTRA_STYLE}",
            disabled,
            "data-placeholder": placeholder_attr,
            onchange: move |event| {
                if let Some(handler) = &on_change {
                    handler.call(ImStr::from(event.data.value()));
                }
            },
            { props.children }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct FormTextareaInputProps {
    pub value: ImStr,

    #[props(!optional, default = None)]
    pub on_change: Option<EventHandler<ImStr>>,

    #[props(default = false)]
    pub disabled: bool,

    #[props(!optional, default = None)]
    pub placeholder: Option<ImStr>,

    #[props(default = 3u8)]
    pub rows: u8,
}

const TEXTAREA_CLASSES: &str =
    "min-h-[68px] px-[10px] py-2 border border-border-strong rounded-md bg-surface text-ink text-[13px] w-full min-w-0 form-input leading-[1.45]";

#[component]
pub fn FormTextareaInput(props: FormTextareaInputProps) -> Element {
    let placeholder_attr = props.placeholder.as_ref().map(|p| p.to_string());
    let on_change = props.on_change.clone();
    let disabled = props.disabled;
    let rows = props.rows.to_string();

    rsx! {
        textarea {
            class: "{TEXTAREA_CLASSES}",
            style: "resize:vertical;",
            rows: "{rows}",
            disabled,
            placeholder: placeholder_attr,
            oninput: move |event| {
                if let Some(handler) = &on_change {
                    handler.call(ImStr::from(event.data.value()));
                }
            },
            "{props.value}"
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

    // ─── FormTextInput ──────────────────────────────────────────────────

    #[test]
    fn text_input_renders_input_with_form_input_class() {
        fn app() -> Element {
            rsx! { FormTextInput { value: ImStr::from("hello") } }
        }
        let html = render(app);
        assert!(html.starts_with("<input"), "expected <input> root: {html}");
        assert!(
            html.contains("form-input"),
            "missing form-input class: {html}"
        );
    }

    #[test]
    fn text_input_uses_token_classes() {
        fn app() -> Element {
            rsx! { FormTextInput { value: ImStr::from("") } }
        }
        let html = render(app);
        assert!(html.contains("h-[34px]"), "missing 34px height: {html}");
        assert!(html.contains("px-[10px]"), "missing 10px padding: {html}");
        assert!(
            html.contains("border-border-strong"),
            "missing strong border: {html}"
        );
        assert!(html.contains("rounded-md"), "missing rounded-md: {html}");
        assert!(html.contains("bg-surface"), "missing bg-surface: {html}");
        assert!(html.contains("text-ink"), "missing text-ink: {html}");
        assert!(html.contains("text-[13px]"), "missing 13px text: {html}");
    }

    #[test]
    fn text_input_value_attribute_renders() {
        fn app() -> Element {
            rsx! { FormTextInput { value: ImStr::from("hello") } }
        }
        let html = render(app);
        assert!(
            html.contains(r#"value="hello""#),
            "missing value attribute: {html}"
        );
    }

    #[test]
    fn text_input_disabled_propagates() {
        fn app() -> Element {
            rsx! { FormTextInput { value: ImStr::from(""), disabled: true } }
        }
        let html = render(app);
        assert!(
            html.contains("disabled"),
            "missing disabled attribute: {html}"
        );
    }

    #[test]
    fn text_input_placeholder_propagates_when_provided() {
        fn app() -> Element {
            rsx! {
                FormTextInput {
                    value: ImStr::from(""),
                    placeholder: Some(ImStr::from("Search…")),
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Search"), "placeholder missing: {html}");
        assert!(
            html.contains("placeholder"),
            "placeholder attr missing: {html}"
        );
    }

    #[test]
    fn text_input_default_type_is_text() {
        fn app() -> Element {
            rsx! { FormTextInput { value: ImStr::from("") } }
        }
        let html = render(app);
        assert!(html.contains(r#"type="text""#), "missing type=text: {html}");
    }

    #[test]
    fn text_input_custom_type_propagates() {
        fn app() -> Element {
            rsx! {
                FormTextInput {
                    value: ImStr::from(""),
                    input_type: ImStr::from("date"),
                }
            }
        }
        let html = render(app);
        assert!(html.contains(r#"type="date""#), "missing type=date: {html}");
    }

    // ─── FormSelectInput ────────────────────────────────────────────────

    #[test]
    fn select_input_renders_select_with_form_input_class() {
        fn app() -> Element {
            rsx! {
                FormSelectInput {
                    option { value: "a", "A" }
                    option { value: "b", "B" }
                }
            }
        }
        let html = render(app);
        assert!(
            html.starts_with("<select"),
            "expected <select> root: {html}"
        );
        assert!(
            html.contains("form-input"),
            "missing form-input class: {html}"
        );
    }

    #[test]
    fn select_input_has_appearance_none_and_chevron_background() {
        fn app() -> Element {
            rsx! { FormSelectInput { option { value: "a", "A" } } }
        }
        let html = render(app);
        assert!(
            html.contains("appearance:none"),
            "missing appearance:none: {html}"
        );
        assert!(
            html.contains("background-image:url("),
            "missing chevron background: {html}"
        );
        assert!(
            html.contains("background-position:right 10px center"),
            "missing chevron alignment: {html}"
        );
    }

    #[test]
    fn select_input_disabled_propagates() {
        fn app() -> Element {
            rsx! {
                FormSelectInput { disabled: true,
                    option { value: "a", "A" }
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("disabled"),
            "missing disabled attribute: {html}"
        );
    }

    #[test]
    fn select_input_renders_children_options() {
        fn app() -> Element {
            rsx! {
                FormSelectInput {
                    option { value: "k", "Kraków" }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Kraków"), "child option missing: {html}");
    }

    // ─── FormTextareaInput ──────────────────────────────────────────────

    #[test]
    fn textarea_renders_with_form_input_class_and_min_height() {
        fn app() -> Element {
            rsx! { FormTextareaInput { value: ImStr::from("") } }
        }
        let html = render(app);
        assert!(
            html.starts_with("<textarea"),
            "expected <textarea> root: {html}"
        );
        assert!(
            html.contains("form-input"),
            "missing form-input class: {html}"
        );
        assert!(html.contains("min-h-[68px]"), "missing min height: {html}");
        assert!(html.contains("leading-[1.45]"), "missing leading: {html}");
    }

    #[test]
    fn textarea_resizes_vertically_only() {
        fn app() -> Element {
            rsx! { FormTextareaInput { value: ImStr::from("") } }
        }
        let html = render(app);
        assert!(
            html.contains("resize:vertical"),
            "missing vertical resize: {html}"
        );
    }

    #[test]
    fn textarea_value_appears_in_body() {
        fn app() -> Element {
            rsx! { FormTextareaInput { value: ImStr::from("first line") } }
        }
        let html = render(app);
        assert!(html.contains("first line"), "value missing in body: {html}");
    }

    #[test]
    fn textarea_disabled_propagates() {
        fn app() -> Element {
            rsx! { FormTextareaInput { value: ImStr::from(""), disabled: true } }
        }
        let html = render(app);
        assert!(
            html.contains("disabled"),
            "missing disabled attribute: {html}"
        );
    }

    #[test]
    fn textarea_placeholder_propagates_when_provided() {
        fn app() -> Element {
            rsx! {
                FormTextareaInput {
                    value: ImStr::from(""),
                    placeholder: Some(ImStr::from("z.B. Inventur")),
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Inventur"), "placeholder missing: {html}");
    }
}
