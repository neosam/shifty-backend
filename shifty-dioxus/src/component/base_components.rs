use std::rc::Rc;

use dioxus::prelude::*;
use time::macros::format_description;

use crate::base_types::ImStr;

#[derive(Props, Clone, PartialEq)]
pub struct ChildrenProps {
    pub children: Element,
}

#[component]
pub fn Header(props: ChildrenProps) -> Element {
    rsx! {
        h1 { class: "text-2xl font-bold mb-6", {props.children} }
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
    rsx! {
        option { value: "{props.value}", selected: props.selected, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SelectProps {
    pub children: Element,
    #[props(default = false)]
    pub disabled: bool,

    #[props(!optional, default = None)]
    pub on_change: Option<EventHandler<ImStr>>,
}

#[component]
pub fn Select(props: SelectProps) -> Element {
    rsx! {
        select {
            class: "border-2 border-gray-200 p-2 min-w-60",
            onchange: move |event| {
                if let Some(on_change) = &props.on_change {
                    let value = event.data.value();
                    on_change.call(ImStr::from(value));
                }
            },
            disabled: props.disabled,
            {props.children}
        }
    }
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
        Select { on_change: props.on_change.clone(),
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
    rsx! {
        label { class: "", {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct FormPairProps {
    pub children: Element,

    pub label: Rc<str>,
}

#[component]
pub fn FormPair(props: FormPairProps) -> Element {
    rsx! {
        div { class: "flex flex-col justify-between md:flex-row border-b-2 border-gray-200 border-dashed p-2",
            Label { {props.label} }
            {props.children}
        }
    }
}

#[component]
pub fn FormItem(props: ChildrenProps) -> Element {
    rsx! {
        div { class: "border-b-2 border-gray-200 border-dashed p-2", {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    pub children: Element,
    #[props(default = false)]
    pub disabled: bool,

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
                    if !props.disabled {
                        on_change.call(!value);
                    }
                }
            },
            input {
                class: "border-2 border-gray-200 p-2 mr-2 min-w-60",
                "type": "checkbox",
                disabled: props.on_change.is_none() || props.disabled,
                "checked": props.value,
            }
            {props.children}
        }
    }
}

#[component]
pub fn Form(props: ChildrenProps) -> Element {
    rsx! {
        form { {props.children} }
    }
}

#[component]
pub fn FormGroup(props: ChildrenProps) -> Element {
    rsx! {
        div { class: "flex flex-col justify-between md:flex-row border-b-2 border-gray-200 border-dashed p-2",
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
            prevent_default: "onclick",
            onclick: move |_| {
                if let Some(on_click) = &props.on_click {
                    on_click.call(());
                }
            },
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TextInputProps {
    pub value: ImStr,
    #[props(default = false)]
    pub disabled: bool,

    pub on_change: Option<EventHandler<ImStr>>,
}

#[component]
pub fn TextInput(props: TextInputProps) -> Element {
    rsx! {
        input {
            class: "border-2 border-gray-200 p-2 min-w-60",
            "type": "text",
            value: props.value,
            disabled: props.disabled,
            oninput: move |event| {
                if let Some(on_change) = &props.on_change {
                    let value = event.data.value();
                    on_change.call(ImStr::from(value));
                }
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DateInputProps {
    pub value: time::Date,
    #[props(default = false)]
    pub disabled: bool,

    pub on_change: Option<EventHandler<time::Date>>,
}

#[component]
pub fn DateInput(props: DateInputProps) -> Element {
    let format = format_description!("[year]-[month]-[day]");
    let value = props.value.format(&format).unwrap();
    rsx! {
        input {
            class: "border-2 border-gray-200 p-2 min-w-60",
            "type": "date",
            value,
            disabled: props.disabled,
            oninput: move |event| {
                if let Some(on_change) = &props.on_change {
                    let value = event.data.value();
                    if let Ok(value) = time::Date::parse(&value, format) {
                        on_change.call(value);
                    } else {
                        tracing::error!("Invalid date: {}", value);
                    }
                }
            },
        }
    }
}

// Input tag for integer values
#[derive(Props, Clone, PartialEq)]
pub struct IntegerInputProps {
    pub value: i32,
    #[props(default = false)]
    pub disabled: bool,

    pub on_change: Option<EventHandler<i32>>,
}

#[component]
pub fn IntegerInput(props: IntegerInputProps) -> Element {
    rsx! {
        input {
            class: "border-2 border-gray-200 p-2 min-w-60",
            "type": "number",
            value: props.value.to_string(),
            disabled: props.disabled,
            oninput: move |event| {
                if let Some(on_change) = &props.on_change {
                    let value = event.data.value();
                    if let Ok(value) = value.parse() {
                        on_change.call(value);
                    } else {
                        tracing::error!("Invalid number: {}", value);
                    }
                }
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TimeInputProps {
    pub value: time::Time,
    #[props(default = false)]
    pub disabled: bool,

    pub on_change: Option<EventHandler<time::Time>>,
}

#[component]
pub fn TimeInput(props: TimeInputProps) -> Element {
    let time_format = format_description!("[hour]:[minute]:[second]");
    rsx! {
        input {
            class: "border-2 border-gray-200 p-2 min-w-60",
            "type": "time",
            value: props.value.format(&time_format).unwrap(),
            disabled: props.disabled,
            oninput: move |event| {
                if let Some(on_change) = &props.on_change {
                    let value = event.data.value();
                    if let Ok(value) = time::Time::parse(&value, time_format) {
                        on_change.call(value);
                    } else {
                        tracing::error!("Invalid number: {}", value);
                    }
                }
            },
        }
    }
}

// Input tag for float values
#[derive(Props, Clone, PartialEq)]
pub struct FloatInputProps {
    pub value: f32,
    pub step: f32,
    #[props(default = false)]
    pub disabled: bool,

    pub on_change: Option<EventHandler<f32>>,
}

#[component]
pub fn FloatInput(props: FloatInputProps) -> Element {
    rsx! {
        input {
            class: "border-2 border-gray-200 p-2",
            "type": "number",
            value: props.value.to_string(),
            step: props.step.to_string(),
            disabled: props.disabled,
            oninput: move |event| {
                if let Some(on_change) = &props.on_change {
                    let value = event.data.value();
                    if let Ok(value) = value.parse() {
                        on_change.call(value);
                    } else {
                        tracing::error!("Invalid number: {}", value);
                    }
                }
            },
        }
    }
}
