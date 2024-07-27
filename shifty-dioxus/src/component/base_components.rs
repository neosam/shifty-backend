use std::rc::Rc;

use dioxus::prelude::*;

use crate::base_types::ImStr;

#[derive(Props, Clone, PartialEq)]
pub struct ChildrenProps {
    pub children: Element,
}

#[component]
pub fn Header(props: ChildrenProps) -> Element {
    rsx! {
        h1 {
            class: "text-2xl font-bold mb-6",
            {props.children},
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct OptionProps {
    pub children: Element,

    #[props(default = false)]
    pub selected: bool,
    pub value: ImStr,
}

#[component]
pub fn Option(props: OptionProps) -> Element {
    rsx! { option {
        value: "{props.value}",
        selected: props.selected,
        {props.children}
    } }
}

#[derive(Props, Clone, PartialEq)]
pub struct SelectProps {
    pub children: Element,

    #[props(!optional, default = None)]
    pub on_change: Option<EventHandler<ImStr>>,
}

#[component]
pub fn Select(props: SelectProps) -> Element {
    rsx! { select {
        class: "border-2 border-gray-200 p-2",
        onchange: move |event| {
            if let Some(on_change) = &props.on_change {
                let value = event.data.value();
                on_change.call(ImStr::from(value));
            }
        },
        {props.children}
    } }
}

#[derive(Clone, PartialEq)]
pub struct SimpleOption {
    pub key: ImStr,
    pub text: ImStr,
}
#[derive(Props, Clone, PartialEq)]
pub struct SimpleSelectProps {
    pub options: Rc<[SimpleOption]>,

    pub selected_key: Option<ImStr>,
    pub on_change: Option<EventHandler<ImStr>>,
}

#[component]
pub fn SimpleSelect(props: SimpleSelectProps) -> Element {
    rsx! {
        Select {
            on_change: props.on_change.clone(),
            for option in props.options.iter() {
                Option {
                    value: option.key.clone(),
                    selected: props.selected_key.as_ref() == Some(&option.key),
                    span { "{option.text.clone()}" }
                }
            }
        }
    }
}

#[component]
pub fn Label(props: ChildrenProps) -> Element {
    rsx! { label {
        class: "",
        {props.children}
    } }
}

#[derive(Props, Clone, PartialEq)]
pub struct FormPairProps {
    pub children: Element,

    pub label: Rc<str>,
}

#[component]
pub fn FormPair(props: FormPairProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col justify-between md:flex-row border-b-2 border-gray-200 border-dashed p-2",
            Label {
                {props.label}
            }
            {props.children}
        }
    }
}

#[component]
pub fn FormItem(props: ChildrenProps) -> Element {
    rsx! { div {
        class: "border-b-2 border-gray-200 border-dashed p-2",
        {props.children}
    } }
}

#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    pub children: Element,

    pub value: Option<bool>,
    pub on_change: Option<EventHandler<bool>>,
}

#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let value = props.value.unwrap_or(false);
    rsx! {
        div {
            onclick: move |_| {
                if let Some(on_change) = &props.on_change {
                    on_change.call(!value);
                }
            },
            input {
                class: "border-2 border-gray-200 p-2 mr-2",
                "type": "checkbox",
                "checked": props.value,
            }
            {props.children}
        }
    }
}

#[component]
pub fn Form(props: ChildrenProps) -> Element {
    rsx! { form {
        {props.children}
    } }
}

#[component]
pub fn FormGroup(props: ChildrenProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col justify-between md:flex-row border-b-2 border-gray-200 border-dashed p-2",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    pub children: Element,

    pub on_click: Option<EventHandler<()>>,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    rsx! {
        button {
            class: "border-2 border-gray-200 p-2",
            onclick: move |_| {
                if let Some(on_click) = &props.on_click {
                    on_click.call(());
                }
            },
            {props.children}
        }
    }
}
