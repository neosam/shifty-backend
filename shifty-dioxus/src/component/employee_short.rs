use dioxus::prelude::*;

use crate::{
    i18n::{self, Key},
    state::employee::Employee,
};

#[derive(Clone, Props, PartialEq)]
pub struct EmployeeShortProps {
    pub employee: Employee,
}

#[component]
pub fn EmployeeShort(props: EmployeeShortProps) -> Element {
    let i18n = use_context::<i18n::I18n<Key, i18n::Locale>>();
    let hours_str = i18n.t(Key::Hours);

    rsx! {
        div {
            class: "flex items-center p-2 border-b border-gray-200",
            div {
                class: "flex items-center",
                div {
                    class: "ml-4",
                    div {
                        class: "text-sm font-medium text-gray-900",
                        "{props.employee.sales_person.name}"
                    }
                    div {
                        class: "text-sm text-gray-500",
                        {format!("{:.1} {hours_str}", props.employee.balance)}
                    }
                }
            }
        }
    }
}
