use dioxus::prelude::*;

use crate::{i18n::Key, service::I18N, state::employee::Employee};

#[derive(Clone, Props, PartialEq)]
pub struct EmployeeShortProps {
    pub employee: Employee,
}

#[component]
pub fn EmployeeShort(props: EmployeeShortProps) -> Element {
    let i18n = I18N.read().clone();
    let hours_str = i18n.t(Key::Hours);

    rsx! {
        div { class: "flex items-center p-2 border-b border-gray-200",
            div { class: "flex items-center",
                div { class: "ml-4",
                    div { class: "text-sm font-medium text-gray-900",
                        "{props.employee.sales_person.name}"
                    }
                    div { class: "text-sm text-gray-500",
                        {format!("{:.2} {hours_str}", props.employee.balance)}
                    }
                }
            }
        }
    }
}
