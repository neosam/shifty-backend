//! `FormCheckbox` — token-based checkbox atom for the redesigned forms.
//!
//! Coexists with the legacy [`crate::component::base_components::Checkbox`]
//! until the cleanup change. New code SHOULD use `FormCheckbox`.

use dioxus::prelude::*;

const WRAPPER: &str = "inline-flex items-center gap-2 cursor-pointer text-[13px] text-ink";
const INPUT: &str = "h-4 w-4 rounded-sm border border-border-strong accent-accent form-input";

#[derive(Props, Clone, PartialEq)]
pub struct FormCheckboxProps {
    pub value: bool,

    #[props(default = false)]
    pub disabled: bool,

    #[props(!optional, default = None)]
    pub on_change: Option<EventHandler<bool>>,

    pub label: Element,
}

#[component]
pub fn FormCheckbox(props: FormCheckboxProps) -> Element {
    let on_change = props.on_change.clone();
    let disabled = props.disabled;
    let checked = props.value;

    rsx! {
        label { class: "{WRAPPER}",
            input {
                class: "{INPUT}",
                r#type: "checkbox",
                checked,
                disabled,
                oninput: move |event| {
                    if disabled {
                        return;
                    }
                    if let Some(handler) = &on_change {
                        let new_value = event.data.value() == "true";
                        handler.call(new_value);
                    }
                },
            }
            { props.label }
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
    fn renders_input_with_type_checkbox() {
        fn app() -> Element {
            rsx! {
                FormCheckbox { value: false, label: rsx! { "x" } }
            }
        }
        let html = render(app);
        assert!(
            html.contains(r#"type="checkbox""#),
            "missing type=checkbox: {html}"
        );
        assert!(html.contains("<input"), "expected input element: {html}");
    }

    #[test]
    fn checked_attribute_reflects_value_true() {
        fn app() -> Element {
            rsx! {
                FormCheckbox { value: true, label: rsx! { "x" } }
            }
        }
        let html = render(app);
        assert!(html.contains("checked"), "missing checked attr: {html}");
    }

    #[test]
    fn unchecked_value_omits_checked_attribute() {
        fn app() -> Element {
            rsx! {
                FormCheckbox { value: false, label: rsx! { "x" } }
            }
        }
        let html = render(app);
        assert!(!html.contains("checked"), "unexpected checked attr: {html}");
    }

    #[test]
    fn disabled_propagates() {
        fn app() -> Element {
            rsx! {
                FormCheckbox { value: false, disabled: true, label: rsx! { "x" } }
            }
        }
        let html = render(app);
        assert!(html.contains("disabled"), "missing disabled attr: {html}");
    }

    #[test]
    fn label_slot_renders_alongside_input() {
        fn app() -> Element {
            rsx! {
                FormCheckbox { value: false, label: rsx! { "Monday" } }
            }
        }
        let html = render(app);
        assert!(html.contains("Monday"), "label text missing: {html}");
        assert!(html.contains("<input"), "input missing: {html}");
    }

    #[test]
    fn input_uses_form_input_class() {
        fn app() -> Element {
            rsx! {
                FormCheckbox { value: false, label: rsx! { "x" } }
            }
        }
        let html = render(app);
        assert!(
            html.contains("form-input"),
            "missing form-input class: {html}"
        );
    }
}
