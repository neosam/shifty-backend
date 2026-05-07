use dioxus::prelude::*;

use crate::{
    component::{
        add_extra_days_form::{AddExtraDaysForm, AddExtraDaysType},
        base_components::Header,
    },
    i18n::Key,
    service::i18n::I18N,
};

#[allow(dead_code)]
pub enum Choice {
    Main,
    Vacation,
    SickLeave,
    Holiday,
    ExtraWork,
}

#[component]
pub fn AddExtraHoursChoice() -> Element {
    let i18n = I18N.read().clone();

    let form_title = i18n.t(Key::AddExtraHoursChoiceTitle);
    let add_vacation_str = i18n.t(Key::CategoryVacation);
    let add_sick_leave_str = i18n.t(Key::CategorySickLeave);
    let add_holiday_str = i18n.t(Key::CategoryHolidays);
    let add_extra_work_str = i18n.t(Key::CategoryExtraWork);

    let mut choice = use_signal(|| Choice::Main);

    let result = match *choice.read() {
        Choice::Main => rsx! {
            div {
                Header { {form_title.clone()} }

                div { class: "grid gap-4 md:grid-cols-2",
                    button {
                        class: "border-2 border-gray-200 p-2",
                        onclick: move |_| *choice.write() = Choice::Vacation,
                        "{add_vacation_str}"
                    }
                    button { class: "border-2 border-gray-200 p-2", "{add_sick_leave_str}" }
                    button { class: "border-2 border-gray-200 p-2", "{add_holiday_str}" }
                    button { class: "border-2 border-gray-200 p-2", "{add_extra_work_str}" }
                }
            }
        },
        Choice::Vacation => rsx! {
            AddExtraDaysForm { extra_hours_type: AddExtraDaysType::Vacation }
        },
        Choice::Holiday => rsx! {
            AddExtraDaysForm { extra_hours_type: AddExtraDaysType::Holiday }
        },
        _ => rsx! { "Not implemented" },
    };
    result
}
