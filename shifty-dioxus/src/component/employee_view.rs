use std::rc::Rc;

use dioxus::prelude::*;
use uuid::Uuid;

use crate::component::add_extra_hours_choice::AddExtraHoursChoice;
use crate::i18n::{self, Key};
use crate::service::I18N;
use crate::state::employee::{Employee, ExtraHours, WorkingHours};

use crate::component::{AddExtraHoursForm, Modal};

#[derive(Props, Clone, PartialEq)]
pub struct EmployeeViewProps {
    pub employee: Employee,
    pub extra_hours: Rc<[ExtraHours]>,
    pub onupdate: EventHandler<()>,
    pub on_extra_hour_delete: EventHandler<Uuid>,
}

#[derive(Props, Clone, PartialEq)]
pub struct WorkingHoursViewProps {
    pub working_hours: WorkingHours,
}

#[derive(Props, Clone, PartialEq)]
pub struct TupleViewProps {
    pub label: Rc<str>,
    pub value: Rc<str>,
    pub ondelete: Option<EventHandler<()>>,
}
#[component]
pub fn TupleView(props: TupleViewProps) -> Element {
    rsx! {
        div { class: "flex justify-between border-b-2 border-gray-200 border-dashed pl-2 gap-4",
            div { "{props.label}" }
            div { class: "flex flow-row gap-2",
                div { "{props.value}" }
                if let Some(ondelete) = props.ondelete {
                    button {
                        class: "border-2 border-gray-200 pl-1 pr-1 shrink h-6 font-small",
                        onclick: move |_| {
                            ondelete.call(());
                        },
                        "🗑️"
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TripleViewProps {
    pub label: Rc<str>,
    pub value: Rc<str>,
    pub description: Rc<str>,
    pub ondelete: Option<EventHandler<()>>,
}
#[component]
pub fn TripleView(props: TripleViewProps) -> Element {
    rsx! {
        div { class: "flex justify-between border-b-2 border-gray-200 border-dashed pl-2 gap-2",
            div { class: "flex flex-col",
                div { "{props.label}" }
                div { class: "text-sm text-gray-500", "{props.description}" }
            }
            div { class: "flex flow-row gap-2",
                div { class: "flex flex-col",
                    div { "{props.value}" }
                }
                if let Some(ondelete) = props.ondelete {
                    button {
                        class: "border-2 border-gray-200 pl-1 pr-1 shrink h-6 font-small",
                        onclick: move |_| {
                            ondelete.call(());
                        },
                        "🗑️"
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ExtraHoursViewProps {
    pub extra_hours: Rc<[ExtraHours]>,
    pub ondelete: EventHandler<Uuid>,
}

#[component]
pub fn ExtraHoursView(props: ExtraHoursViewProps) -> Element {
    let i18n = I18N.read().clone();
    let extra_work_str = i18n.t(Key::CategoryExtraWork);
    let vacation_str = i18n.t(Key::CategoryVacation);
    let sick_leave_str = i18n.t(Key::CategorySickLeave);
    let holidays_str = i18n.t(Key::CategoryHolidays);
    let hours_str = i18n.t(Key::Hours);
    let work_hours_description_str = i18n.t(Key::WorkHoursDescription);

    rsx! {
        div { class: "flex flex-col",
            h2 { class: "text-lg font-bold mt-8", "{vacation_str}" }

            ul {
                for extra_hours in props.extra_hours.iter().filter(|eh| eh.category.is_vacation()) {
                    {
                        let extra_hours_id = extra_hours.id;
                        rsx! { li {
                            class: "mb-1",
                            TripleView {
                                label: i18n.format_date(&extra_hours.date_time.date()),
                                value: format!("{:.3} {hours_str}", extra_hours.amount).into(),
                                description: format!("{}", extra_hours.description).into(),
                                ondelete: move |_| props.ondelete.call(extra_hours_id),
                            }
                        } }
                    }
                }
            }

            h2 { class: "text-lg font-bold mt-8", "{holidays_str}" }

            ul {
                for extra_hours in props.extra_hours.iter().filter(|eh| eh.category.is_holiday()) {
                    {
                        let extra_hours_id = extra_hours.id;
                        rsx! { li {
                            class: "mb-1",
                            TripleView {
                                label: i18n.format_date(&extra_hours.date_time.date()),
                                value: format!("{:.3} {hours_str}", extra_hours.amount).into(),
                                description: format!("{}", extra_hours.description).into(),
                                ondelete: move |_| props.ondelete.call(extra_hours_id),
                            }
                        } }
                    }
                }
            }

            h2 { class: "text-lg font-bold mt-8", "{sick_leave_str}" }

            ul {
                for extra_hours in props.extra_hours.iter().filter(|eh| eh.category.is_sick_leave()) {
                    {
                        let extra_hours_id = extra_hours.id;
                        rsx! { li {
                            class: "mb-1",
                            TripleView {
                                label: i18n.format_date(&extra_hours.date_time.date()),
                                value: format!("{:.3} {hours_str}", extra_hours.amount).into(),
                                description: format!("{}", extra_hours.description).into(),
                                ondelete: move |_| props.ondelete.call(extra_hours_id),
                            }
                        } }
                    }
                }
            }

            h2 { class: "text-lg font-bold mt-8", "{extra_work_str}" }
            p { class: "text-sm text-gray-500 mb-4", "{work_hours_description_str}" }

            ul {
                for extra_hours in props.extra_hours.iter().filter(|eh| eh.category.is_extra_work()) {
                    {
                        let extra_hours_id = extra_hours.id;
                        rsx! { li {
                            key: "{extra_hours_id}",
                            class: "mb-1",
                            TripleView {
                                label: i18n.format_date(&extra_hours.date_time.date()),
                                value: format!("{:.3} {hours_str}", extra_hours.amount).into(),
                                description: format!("{}", extra_hours.description).into(),
                                ondelete: move |_| props.ondelete.call(extra_hours_id),
                            }
                        } }
                    }
                }
            }
        }
    }
}

#[component]
pub fn WorkingHoursView(props: WorkingHoursViewProps) -> Element {
    let mut expand_days = use_signal(|| false);
    let mut expand_details = use_signal(|| false);

    let i18n = I18N.read().clone();
    let working_hours_per_day_heading = i18n.t(Key::WorkingHoursPerDayHeading);
    let balance_str = i18n.t(Key::Balance);
    let overall_str = i18n.t(Key::Overall);
    let required_str = i18n.t(Key::Required);
    let shiftplan_str = i18n.t(Key::CategoryShiftplan);
    let extra_work_str = i18n.t(Key::CategoryExtraWork);
    let vacation_str = i18n.t(Key::CategoryVacation);
    let sick_leave_str = i18n.t(Key::CategorySickLeave);
    let holidays_str = i18n.t(Key::CategoryHolidays);
    let show_details_str = i18n.t(Key::ShowDetails);
    let hide_details_str = i18n.t(Key::HideDetails);
    let hours_str = i18n.t(Key::Hours);

    rsx! {
        div {
            div { class: "flex flex-row mt-4 justify-between gap-2",
                h3 { class: "text-l font-bold",
                    "{i18n.format_date(&props.working_hours.from)} - {i18n.format_date(&props.working_hours.to)}"
                }
                div { { format!("{:.2} {}", props.working_hours.balance, hours_str) } }
                if *expand_details.read() {
                    div {
                        class: "cursor-pointer underline",
                        onclick: move |_| {
                            *expand_details.write() = false;
                        },
                        "{hide_details_str}"
                    }
                } else {
                    div {
                        class: "cursor-pointer underline",
                        onclick: move |_| {
                            *expand_details.write() = true;
                        },
                        "{show_details_str}"
                    }
                }
            }
            ul {
                if *expand_details.read() {
                    li {
                        TupleView {
                            label: balance_str.clone(),
                            value: format!("{:.2} {hours_str}", props.working_hours.balance).into()
                        }
                    }
                    li {
                        TupleView {
                            label: overall_str.clone(),
                            value: format!("{:.2} {hours_str}", props.working_hours.overall_hours).into()
                        }
                    }
                    li {
                        TupleView {
                            label: required_str.clone(),
                            value: format!("{:.2} {hours_str}", props.working_hours.expected_hours).into()
                        }
                    }
                    li { class: "mt-2",
                        TupleView {
                            label: shiftplan_str.clone(),
                            value: format!("{:.2} {hours_str}", props.working_hours.shiftplan_hours).into()
                        }
                    }
                    li {
                        TupleView {
                            label: extra_work_str.clone(),
                            value: format!("{:.2} {hours_str}", props.working_hours.extra_work_hours).into()
                        }
                    }
                    li {
                        TupleView {
                            label: vacation_str.clone(),
                            value: format!("{:.2} {hours_str}", props.working_hours.vacation_hours).into()
                        }
                    }
                    li {
                        TupleView {
                            label: sick_leave_str.clone(),
                            value: format!("{:.2} {hours_str}", props.working_hours.sick_leave_hours).into()
                        }
                    }
                    li {
                        TupleView {
                            label: holidays_str.clone(),
                            value: format!("{:.2} {hours_str}", props.working_hours.holiday_hours).into()
                        }
                    }
                }
                li { class: "mt-4",
                    if *expand_details.read() {
                        div {
                            class: "cursor-pointer underline",
                            onclick: move |_| {
                                *expand_details.write() = false;
                            },
                            "{hide_details_str}"
                        }
                    } else {
                    }
                }
            }

            if *expand_details.read() {
                div { class: "flex flex-row mt-6 justify-between",
                    h4 { class: "text-lg font-bold", "{working_hours_per_day_heading}" }
                    if !*expand_days.read() {
                        div {
                            class: "cursor-pointer underline",
                            onclick: move |_| {
                                *expand_days.write() = true;
                            },
                            "{show_details_str}"
                        }
                    } else {
                        div {
                            class: "cursor-pointer underline",
                            onclick: move |_| {
                                *expand_days.write() = false;
                            },
                            "{hide_details_str}"
                        }
                    }
                }

                if *expand_days.read() {
                    ul {
                        for working_hours in props.working_hours.days.iter() {
                            li {
                                TripleView {
                                    label: i18n.format_date(&working_hours.date),
                                    value: format!("{} {hours_str}", working_hours.hours).into(),
                                    description: format!("{}", i18n.t(working_hours.category.to_i18n_key())).into()
                                }
                            }
                        }
                    }
                }

                div { class: "mb-12" }
            }
        }
    }
}

#[component]
pub fn EmployeeView(props: EmployeeViewProps) -> Element {
    let i18n = I18N.read().clone();
    let mut expand_weeks = use_signal(|| false);
    //let mut expand_months = use_signal(|| false);
    //let mut expand_details = use_signal(|| false);
    let mut show_add_entry_dialog = use_signal(|| false);

    let overall_header_str = i18n.t(Key::OverallHeading);
    let working_hours_per_week_heading = i18n.t(Key::WorkingHoursPerWeekHeading);
    let extra_hours_heading = i18n.t(Key::ExtraHoursHeading);
    let balance_str = i18n.t(Key::Balance);
    let overall_str = i18n.t(Key::Overall);
    let required_str = i18n.t(Key::Required);
    let shiftplan_str = i18n.t(Key::CategoryShiftplan);
    let extra_work_str = i18n.t(Key::CategoryExtraWork);
    let vacation_str = i18n.t(Key::CategoryVacation);
    let sick_leave_str = i18n.t(Key::CategorySickLeave);
    let holidays_str = i18n.t(Key::CategoryHolidays);
    let show_details_str = i18n.t(Key::ShowDetails);
    let hide_details_str = i18n.t(Key::HideDetails);
    let hours_str = i18n.t(Key::Hours);
    let add_entry_str = i18n.t(Key::AddEntry);

    rsx! {
        if *show_add_entry_dialog.read() {
            Modal {
                AddExtraHoursForm {
                    sales_person_id: props.employee.sales_person.id,
                    onabort: move |_| {
                        *show_add_entry_dialog.write() = false;
                    },
                    onsaved: move |_| {
                        props.onupdate.call(());
                        *show_add_entry_dialog.write() = false;
                    }
                }
            }
        }

        div { class: "flex justify-between",
            h1 { class: "text-2xl font-bold", {props.employee.sales_person.name.clone()} }
            button {
                class: "border-2 border-gray-200 p-2",
                onclick: move |_| {
                    *show_add_entry_dialog.write() = true;
                },
                "{add_entry_str}"
            }
        }
        div { class: "flex flex-col lg:flex-row lg:justify-between lg:gap-4",
            div {
                div { class: "flex flex-col",
                    h2 { class: "text-lg font-bold mt-8", "{overall_header_str}" }

                    ul {
                        li {
                            TupleView {
                                label: balance_str.clone(),
                                value: format!("{:.2} {}", props.employee.balance, hours_str.clone()).into()
                            }
                        }

                        li {
                            TupleView {
                                label: overall_str.clone(),
                                value: format!("{:.2} {}", props.employee.overall_working_hours, hours_str.clone()).into()
                            }
                        }
                        li {
                            TupleView {
                                label: required_str.clone(),
                                value: format!("{:.2} {}", props.employee.expected_working_hours, hours_str.clone()).into()
                            }
                        }
                        li { class: "mt-2",
                            TupleView {
                                label: shiftplan_str.clone(),
                                value: format!("{:.2} {}", props.employee.shiftplan_hours, hours_str.clone()).into()
                            }
                        }
                        li {
                            TupleView {
                                label: extra_work_str.clone(),
                                value: format!("{:.2} {}", props.employee.extra_work_hours, hours_str.clone()).into()
                            }
                        }
                        li {
                            TupleView {
                                label: vacation_str.clone(),
                                value: format!("{:.2} {}", props.employee.vacation_hours, hours_str.clone()).into()
                            }
                        }
                        li {
                            TupleView {
                                label: sick_leave_str.clone(),
                                value: format!("{:.2} {}", props.employee.sick_leave_hours, hours_str.clone()).into()
                            }
                        }
                        li {
                            TupleView {
                                label: holidays_str.clone(),
                                value: format!("{:.2} {}", props.employee.holiday_hours, hours_str.clone()).into()
                            }
                        }
                    }
                }
            }

            div { class: "border-t-2 border-gray-200 border-double mt-8 lg:pl-4 lg:flex-grow lg:ml-4 lg:border-t-0 lg:border-l-2 lg:mt-0",
                div { class: "flex flex-row mt-8 justify-between",
                    h2 { class: "text-lg font-bold", "{working_hours_per_week_heading}" }
                    if !*expand_weeks.read() {
                        div {
                            class: "cursor-pointer underline",
                            onclick: move |_| {
                                *expand_weeks.write() = true;
                            },
                            "{show_details_str}"
                        }
                    } else {
                        div {
                            class: "cursor-pointer underline",
                            onclick: move |_| {
                                *expand_weeks.write() = false;
                            },
                            "{hide_details_str}"
                        }
                    }
                }

                if *expand_weeks.read() {
                    for working_hours in props.employee.working_hours_by_week.iter() {
                        WorkingHoursView { working_hours: working_hours.clone() }
                    }
                }
            }

            div { class: "border-t-2 border-gray-200 border-double mt-8 lg:pl-4 lg:flex-grow lg:ml-4 lg:border-t-0 lg:border-l-2 lg:mt-0",

                h2 { class: "text-lg font-bold mt-8", "{extra_hours_heading}" }

                ExtraHoursView {
                    extra_hours: props.extra_hours.clone(),
                    ondelete: move |uuid| {
                        props.on_extra_hour_delete.call(uuid);
                        props.onupdate.call(());
                    }
                }
            }
        }
    }
}
