//! Token-based form input atoms used inside [`Field`](super::Field).
//!
//! All three atoms (`TextInput`, `SelectInput`, `TextareaInput`) share the
//! `form-input` class so the global focus rule in `input.css` applies the
//! accent focus ring without per-component styling.

use dioxus::prelude::*;

use crate::base_types::ImStr;

const SHARED_INPUT_CLASSES: &str =
    "h-[34px] px-[10px] border border-border-strong rounded-md bg-surface text-ink text-body w-full min-w-0 form-input";

#[derive(Props, Clone, PartialEq)]
pub struct TextInputProps {
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

    /// Optional `step` attribute for `type="number"` inputs.
    /// When `None` (the default), the attribute is omitted and the browser
    /// uses its default (`1` for `type="number"`).
    #[props(!optional, default = None)]
    pub step: Option<ImStr>,
}

#[component]
pub fn TextInput(props: TextInputProps) -> Element {
    let placeholder_attr = props.placeholder.as_ref().map(|p| p.to_string());
    let step_attr = props.step.as_ref().map(|s| s.to_string());
    let input_type = props.input_type.clone();
    let on_change = props.on_change;
    let disabled = props.disabled;

    rsx! {
        input {
            class: "{SHARED_INPUT_CLASSES}",
            r#type: "{input_type}",
            value: "{props.value}",
            disabled,
            placeholder: placeholder_attr,
            step: step_attr,
            oninput: move |event| {
                if let Some(handler) = &on_change {
                    handler.call(ImStr::from(event.data.value()));
                }
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SelectInputProps {
    pub children: Element,

    #[props(default = false)]
    pub disabled: bool,

    #[props(!optional, default = None)]
    pub placeholder: Option<ImStr>,

    #[props(!optional, default = None)]
    pub on_change: Option<EventHandler<ImStr>>,

    /// Optional controlled value for the select element (D-05/D-07).
    ///
    /// When `Some`, the underlying `<select>` `value` attribute is set so the DOM
    /// selection always tracks the driving signal (controlled mode).  When `None`
    /// (the default), the attribute is omitted and the element behaves as it did
    /// before — uncontrolled — keeping every existing caller backward-compatible.
    #[props(!optional, default = None)]
    pub value: Option<ImStr>,
}

const SELECT_EXTRA_STYLE: &str =
    "appearance:none;-webkit-appearance:none;padding-right:28px;\
     background-image:url(\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'><path d='M1 1l4 4 4-4' stroke='%236b7382' stroke-width='1.5' fill='none' stroke-linecap='round'/></svg>\");\
     background-repeat:no-repeat;background-position:right 10px center;";

#[component]
pub fn SelectInput(props: SelectInputProps) -> Element {
    let on_change = props.on_change;
    let disabled = props.disabled;
    let placeholder_attr = props.placeholder.as_ref().map(|p| p.to_string());
    // When `value` is `Some`, pass it as the controlled value attribute so the
    // DOM selection always tracks the driving signal.  When `None`, omit the
    // attribute entirely so the element stays uncontrolled (D-07 backward-compat).
    let value_attr = props.value.as_ref().map(|v| v.to_string());

    rsx! {
        select {
            class: "{SHARED_INPUT_CLASSES}",
            style: "{SELECT_EXTRA_STYLE}",
            disabled,
            "data-placeholder": placeholder_attr,
            value: value_attr,
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
pub struct TextareaInputProps {
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
    "min-h-[68px] px-[10px] py-2 border border-border-strong rounded-md bg-surface text-ink text-body w-full min-w-0 form-input leading-[1.45]";

#[component]
pub fn TextareaInput(props: TextareaInputProps) -> Element {
    let placeholder_attr = props.placeholder.as_ref().map(|p| p.to_string());
    let on_change = props.on_change;
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

    // ─── TextInput ──────────────────────────────────────────────────

    #[test]
    fn text_input_renders_input_with_form_input_class() {
        fn app() -> Element {
            rsx! { TextInput { value: ImStr::from("hello") } }
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
            rsx! { TextInput { value: ImStr::from("") } }
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
        assert!(
            html.contains("text-body"),
            "missing text-body token: {html}"
        );
    }

    #[test]
    fn text_input_value_attribute_renders() {
        fn app() -> Element {
            rsx! { TextInput { value: ImStr::from("hello") } }
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
            rsx! { TextInput { value: ImStr::from(""), disabled: true } }
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
                TextInput {
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
            rsx! { TextInput { value: ImStr::from("") } }
        }
        let html = render(app);
        assert!(html.contains(r#"type="text""#), "missing type=text: {html}");
    }

    #[test]
    fn text_input_custom_type_propagates() {
        fn app() -> Element {
            rsx! {
                TextInput {
                    value: ImStr::from(""),
                    input_type: ImStr::from("date"),
                }
            }
        }
        let html = render(app);
        assert!(html.contains(r#"type="date""#), "missing type=date: {html}");
    }

    #[test]
    fn text_input_step_propagates_when_provided() {
        fn app() -> Element {
            rsx! {
                TextInput {
                    value: ImStr::from(""),
                    input_type: ImStr::from("number"),
                    step: Some(ImStr::from("0.01")),
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains(r#"step="0.01""#),
            "missing step attribute: {html}"
        );
    }

    #[test]
    fn text_input_step_absent_when_none() {
        fn app() -> Element {
            rsx! { TextInput { value: ImStr::from("") } }
        }
        let html = render(app);
        assert!(!html.contains("step="), "step should not appear without prop: {html}");
    }

    // ─── SelectInput ────────────────────────────────────────────────

    /// D-05: SelectInput with `value: Some("holiday")` renders the value attribute on
    /// the `<select>` element so the DOM selection follows the signal (controlled).
    #[test]
    fn select_input_controlled_value_non_empty_reflected() {
        fn app() -> Element {
            rsx! {
                SelectInput {
                    value: Some(ImStr::from("holiday")),
                    option { value: "a", "A" }
                    option { value: "b", "B" }
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains(r#"value="holiday""#),
            "controlled value 'holiday' not reflected on <select>: {html}"
        );
    }

    /// D-05: SelectInput with `value: Some("")` reflects the empty/placeholder
    /// selection — this is the post-reset state that re-enables the Anlegen button.
    #[test]
    fn select_input_controlled_empty_value_reflected() {
        fn app() -> Element {
            rsx! {
                SelectInput {
                    value: Some(ImStr::from("")),
                    option { value: "one", "One" }
                    option { value: "two", "Two" }
                }
            }
        }
        let html = render(app);
        // value="" appears on the <select> (children have "one"/"two", not "")
        assert!(
            html.contains(r#"value="""#),
            "empty controlled value not reflected on <select>: {html}"
        );
    }

    /// D-07: Without the `value` prop the `<select>` stays uncontrolled — no value
    /// attribute is emitted, preserving backward compatibility for all existing callers.
    #[test]
    fn select_input_uncontrolled_when_no_value_prop() {
        fn app() -> Element {
            rsx! {
                SelectInput {
                    option { value: "x", "X" }
                }
            }
        }
        let html = render(app);
        // Inspect the <select ...> opening tag (before first <option>) to confirm
        // no value= attribute was added by the component.
        let before_first_option = html.find("<option").unwrap_or(html.len());
        let select_tag = &html[..before_first_option];
        assert!(
            !select_tag.contains("value="),
            "select must not carry value= attr in uncontrolled mode: {select_tag}"
        );
    }

    #[test]
    fn select_input_renders_select_with_form_input_class() {
        fn app() -> Element {
            rsx! {
                SelectInput {
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
            rsx! { SelectInput { option { value: "a", "A" } } }
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
                SelectInput { disabled: true,
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
                SelectInput {
                    option { value: "k", "Kraków" }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Kraków"), "child option missing: {html}");
    }

    // ─── TextareaInput ──────────────────────────────────────────────

    #[test]
    fn textarea_renders_with_form_input_class_and_min_height() {
        fn app() -> Element {
            rsx! { TextareaInput { value: ImStr::from("") } }
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
            rsx! { TextareaInput { value: ImStr::from("") } }
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
            rsx! { TextareaInput { value: ImStr::from("first line") } }
        }
        let html = render(app);
        assert!(html.contains("first line"), "value missing in body: {html}");
    }

    #[test]
    fn textarea_disabled_propagates() {
        fn app() -> Element {
            rsx! { TextareaInput { value: ImStr::from(""), disabled: true } }
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
                TextareaInput {
                    value: ImStr::from(""),
                    placeholder: Some(ImStr::from("z.B. Inventur")),
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Inventur"), "placeholder missing: {html}");
    }
}
