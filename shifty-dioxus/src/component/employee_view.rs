use std::rc::Rc;

use dioxus::prelude::*;

use crate::state::employee::{Employee, WorkingHours};

#[derive(Props, Clone, PartialEq)]
pub struct EmployeeViewProps {
    pub employee: Employee,
}

#[derive(Props, Clone, PartialEq)]
pub struct WorkingHoursViewProps {
    pub working_hours: WorkingHours,
}

#[derive(Props, Clone, PartialEq)]
pub struct TupleViewProps {
    pub label: Rc<str>,
    pub value: Rc<str>,
}
#[component]
pub fn TupleView(props: TupleViewProps) -> Element {
    rsx! {
        div {
            class: "flex justify-between border-b-2 border-gray-200 border-dashed pl-2 gap-4",
            div {
                "{props.label}"
            }
            div {
                "{props.value}"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TripleViewProps {
    pub label: Rc<str>,
    pub value: Rc<str>,
    pub description: Rc<str>,
}
#[component]
pub fn TripleView(props: TripleViewProps) -> Element {
    rsx! {
        div {
            class: "flex justify-between border-b-2 border-gray-200 border-dashed pl-2 gap-2",
            div {
                class: "flex flex-col",
                div {
                    "{props.label}"
                }
                div {
                    class: "text-sm text-gray-500",
                    "{props.description}"
                }
            }
            div {
                class: "flex flex-col",
                div {
                    "{props.value}"
                }
            }
        }
    }
}

#[component]
pub fn WorkingHoursView(props: WorkingHoursViewProps) -> Element {
    let mut expand_days = use_signal(|| false);
    let mut expand_details = use_signal(|| false);

    rsx! {
        div {
            div {
                class: "flex flex-row mt-4 justify-between gap-2",
                h3 {
                    class: "text-l font-bold",
                    "{props.working_hours.from} - {props.working_hours.to}"
                }
                div {
                    "{props.working_hours.balance} hours"
                }
                    if *expand_details.read() {
                       div {
                            class: "cursor-pointer underline",
                            onclick: move |_| {
                                *expand_details.write() = false;
                            },
                            "Less"
                        }
                    } else {
                        div {
                            class: "cursor-pointer underline",
                            onclick: move |_| {
                                *expand_details.write() = true;
                            },
                            "More"
                        }
                    }
            }
            ul {
                if *expand_details.read() {
                    li {
                        TupleView {
                            label: "Balance".into(),
                            value: format!("{:.2} hours", props.working_hours.balance).into()
                        }
                    }
                    li {
                        TupleView {
                            label: "Overall".into(),
                            value: format!("{} hours", props.working_hours.overall_hours).into()
                        }
                    }
                    li {
                        TupleView {
                            label: "Required".into(),
                            value: format!("{} hours", props.working_hours.expected_hours).into()
                        }
                    }
                    li {
                        class: "mt-2",
                        TupleView {
                            label: "Shiftplan".into(),
                            value: format!("{} hours", props.working_hours.shiftplan_hours).into()
                        }
                    }
                    li {
                        TupleView {
                            label: "Extra work".into(),
                            value: format!("{} hours", props.working_hours.extra_work_hours).into()
                        }
                    }
                    li {
                        TupleView {
                            label: "Vacation".into(),
                            value: format!("{} hours = {} days", props.working_hours.vacation_hours, props.working_hours.vacation_hours / 8.0).into()
                        }
                    }
                    li {
                        TupleView {
                            label: "Sick leave".into(),
                            value: format!("{} hours = {} days", props.working_hours.sick_leave_hours, props.working_hours.sick_leave_hours / 8.0).into()
                        }
                    }
                    li {
                        TupleView {
                            label: "Holidays".into(),
                            value: format!("{} hours = {} days", props.working_hours.holiday_hours, props.working_hours.holiday_hours / 8.0).into()
                        }
                    }
                }
                li {
                    class: "mt-4",
                    if *expand_details.read() {
                       div {
                            class: "cursor-pointer",
                            onclick: move |_| {
                                *expand_details.write() = false;
                            },
                            "Hide details"
                        }
                    } else {

                    }
                }

            }

            if *expand_details.read() {
                div {
                    class: "flex flex-row mt-6 justify-between",
                    h4 {
                        class: "text-lg font-bold",
                        "Working hours per day"
                    }
                    if !*expand_days.read() {
                        div {
                            class: "cursor-pointer",
                            onclick: move |_| {
                                *expand_days.write() = true;
                            },
                            "Show"
                        }
                    } else {
                        div {
                            class: "cursor-pointer",
                            onclick: move |_| {
                                *expand_days.write() = false;
                            },
                            "Hide"
                        }
                    }
                }

                if *expand_days.read() {
                    ul {
                        for working_hours in props.working_hours.days.iter() {
                            li {
                                TripleView {
                                    label: format!("{}", working_hours.date).into(),
                                    value: format!("{} hours", working_hours.hours).into(),
                                    description: format!("{}", working_hours.category).into()
                                }
                            }
                        }
                    }
                }

                div {
                    class: "mb-12",
                }
            }
        }
    }
}

#[component]
pub fn EmployeeView(props: EmployeeViewProps) -> Element {
    let mut expand_weeks = use_signal(|| false);
    let mut expand_months = use_signal(|| false);
    let mut expand_details = use_signal(|| false);

    rsx! {
        h1 {
            class: "text-2xl font-bold",
            {props.employee.sales_person.name.clone()}
        }
        div {
            class: "flex flex-col lg:flex-row lg:justify-between lg:gap-4",
            div {

                div {
                    class: "flex flex-col",
                    h2 {
                        class: "text-lg font-bold mt-8",
                        "Overall"
                    }

                    ul {
                        li {
                            TupleView {
                                label: "Balance".into(),
                                value: format!("{:.2} hours", props.employee.balance).into()
                            }
                        }
                        if *expand_details.read() {

                            li {
                                TupleView {
                                    label: "Overall".into(),
                                    value: format!("{} hours", props.employee.overall_working_hours).into()
                                }
                            }
                            li {
                                TupleView {
                                    label: "Required".into(),
                                    value: format!("{} hours", props.employee.expected_working_hours).into()
                                }
                            }
                            li {
                                class: "mt-2",
                                TupleView {
                                    label: "Shiftplan".into(),
                                    value: format!("{} hours", props.employee.shiftplan_hours).into()
                                }
                            }
                            li {
                                TupleView {
                                    label: "Extra work".into(),
                                    value: format!("{} hours", props.employee.extra_work_hours).into()
                                }
                            }
                            li {
                                TupleView {
                                    label: "Vacation".into(),
                                    value: format!("{} hours = {} days", props.employee.vacation_hours, props.employee.vacation_hours / 8.0).into()
                                }
                            }
                            li {
                                TupleView {
                                    label: "Sick leave".into(),
                                    value: format!("{} hours = {} days", props.employee.sick_leave_hours, props.employee.sick_leave_hours / 8.0).into()
                                }
                            }
                            li {
                                TupleView {
                                    label: "Holidays".into(),
                                    value: format!("{} hours = {} days", props.employee.holiday_hours, props.employee.holiday_hours / 8.0).into()
                                }
                            }
                        }
                        li {
                            class: "mt-4",
                            if *expand_details.read() {
                               div {
                                    class: "cursor-pointer underline",
                                    onclick: move |_| {
                                        *expand_details.write() = false;
                                    },
                                    "Hide details"
                                }
                            } else {
                                div {
                                    class: "cursor-pointer underline",
                                    onclick: move |_| {
                                        *expand_details.write() = true;
                                    },
                                    "Show details"
                                }

                            }
                        }
                    }
                }
            }

            div {
                class: "border-t-2 border-gray-200 border-double mt-8 lg:pl-4 lg:flex-grow lg:ml-4 lg:border-t-0 lg:border-l-2 lg:mt-0",
                div {
                    class: "flex flex-row mt-8 justify-between",
                    h2 {
                        class: "text-lg font-bold",
                        "Working hours per week"
                    }
                    if !*expand_weeks.read() {
                        div {
                            class: "cursor-pointer underline",
                            onclick: move |_| {
                                *expand_weeks.write() = true;
                            },
                            "Show"
                        }
                    } else {
                        div {
                            class: "cursor-pointer underline",
                            onclick: move |_| {
                                *expand_weeks.write() = false;
                            },
                            "Hide"
                        }
                    }
                }

                if *expand_weeks.read() {
                    for working_hours in props.employee.working_hours_by_week.iter() {
                        WorkingHoursView {
                            working_hours: working_hours.clone()
                        }
                    }
                }
            }

            div {
                class: "border-t-2 border-gray-200 border-solid mt-8 lg:pl-4 lg:flex-grow lg:ml-4 lg:border-t-0 lg:border-l-2 lg:mt-0",
                div {
                    class: "flex flex-row mt-8 justify-between",
                    h2 {
                        class: "text-lg font-bold",
                        "Working hours per month"
                    }
                    if !*expand_months.read() {
                        div {
                            class: "cursor-pointer underline",
                            onclick: move |_| {
                                *expand_months.write() = true;
                            },
                            "Show"
                        }
                    } else {
                        div {
                            class: "cursor-pointer underline",
                            onclick: move |_| {
                                *expand_months.write() = false;
                            },
                            "Hide"
                        }
                    }
                }

                if *expand_months.read() {
                    for working_hours in props.employee.working_hours_by_month.iter() {
                        WorkingHoursView {
                            working_hours: working_hours.clone()
                        }
                    }
                }
            }
        }
    }
}
