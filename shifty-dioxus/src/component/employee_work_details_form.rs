use dioxus::prelude::*;
use uuid::Uuid;

use crate::{
    base_types::ImStr,
    component::base_components::{
        Button, Checkbox, DateInput, FloatInput, Form, FormGroup, FormPair, Header, IntegerInput,
        Label, TextInput,
    },
    service::{EmployeeWorkDetailsAction, EMPLOYEE_WORK_DETAILS_STORE},
    state::{
        employee_work_details::{self, EmployeeWorkDetails},
        shiftplan::SalesPerson,
    },
};
use web_sys::console::error;

#[derive(PartialEq, Clone, Copy)]
pub enum EmployeeWorkDetailsFormType {
    New,
    Edit,
    ReadOnly,
}

#[derive(PartialEq, Clone, Props)]
pub struct WorkingHoursFormPlainProps {
    pub employee_work_details: EmployeeWorkDetails,
    pub sales_person: SalesPerson,
    pub employee_work_details_form_type: EmployeeWorkDetailsFormType,

    pub cancel_label: ImStr,

    pub show_save_button: bool,
    pub on_update_employee_work_details: EventHandler<EmployeeWorkDetails>,
    pub on_save: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
}

#[component]
pub fn EmployeeWorkDetailsFormPlain(props: WorkingHoursFormPlainProps) -> Element {
    let employee_work_details: EmployeeWorkDetails = props.employee_work_details.clone();
    let (from_year, from_week, from_day) = employee_work_details.from_as_calendar_week();
    let (to_year, to_week, to_day) = employee_work_details.to_as_calendar_week();
    rsx! {
        Header { "Employee Work Details for {props.sales_person.name}" }
        match props.employee_work_details_form_type {
            EmployeeWorkDetailsFormType::New => "New",
            EmployeeWorkDetailsFormType::Edit => "Edit",
            EmployeeWorkDetailsFormType::ReadOnly => "ReadOnly",
        },
        Form {
            FormPair { label: "From".into(),
                div { class: "flex flex-col",
                    DateInput {
                        value: employee_work_details.from.clone(),
                        disabled: props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New,
                        on_change: {
                            to_owned![employee_work_details];
                            move |date: time::Date| {
                                if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New
                                {
                                    return ();
                                }
                                let mut employee_work_details = employee_work_details.clone();
                                employee_work_details.from = date.into();
                                props.on_update_employee_work_details.call(employee_work_details);
                            }
                        }
                    }
                    span { class: "text-xs text-gray-500", "({from_day}, Week {from_week}, {from_year})" }
                }
            }
            FormPair { label: "To".into(),
                div { class: "flex flex-col",
                    DateInput {
                        value: employee_work_details.to.clone(),
                        disabled: props.employee_work_details_form_type == EmployeeWorkDetailsFormType::ReadOnly,
                        on_change: {
                            to_owned![employee_work_details];
                            move |date: time::Date| {
                                if props.employee_work_details_form_type
                                    == EmployeeWorkDetailsFormType::ReadOnly
                                {
                                    return ();
                                }
                                let mut employee_work_details = employee_work_details.clone();
                                employee_work_details.to = date.into();
                                props.on_update_employee_work_details.call(employee_work_details);
                            }
                        }
                    }
                    span { class: "text-xs text-gray-500", "({to_day}, Week {to_week}, {to_year})" }
                }
            }
            FormPair { label: "Workdays".into(),
                Checkbox {
                    value: employee_work_details.monday,
                    disabled: props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: bool| {
                            if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.monday = value;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    },
                    "Monday"
                }
                Checkbox {
                    value: employee_work_details.tuesday,
                    disabled: props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: bool| {
                            if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.tuesday = value;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    },
                    "Tuesday"
                }
                Checkbox {
                    value: employee_work_details.wednesday,
                    disabled: props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: bool| {
                            if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.wednesday = value;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    },
                    "Wednesday"
                }
                Checkbox {
                    value: employee_work_details.thursday,
                    disabled: props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: bool| {
                            if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.thursday = value;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    },
                    "Thursday"
                }
                Checkbox {
                    value: employee_work_details.friday,
                    disabled: props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: bool| {
                            if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.friday = value;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    },
                    "Friday"
                }
                Checkbox {
                    value: employee_work_details.saturday,
                    disabled: props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: bool| {
                            if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.saturday = value;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    },
                    "Saturday"
                }
                Checkbox {
                    value: employee_work_details.sunday,
                    disabled: props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: bool| {
                            if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.sunday = value;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    },
                    "Sunday"
                }
            }
            FormPair { label: "Vacation Days".into(),
                IntegerInput {
                    value: employee_work_details.vacation_days as i32,
                    disabled: props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: i32| {
                            if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.vacation_days = value as u8;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    }
                }
            }
            FormPair { label: "Days per Week".into(),
                IntegerInput {
                    value: employee_work_details.workdays_per_week as i32,
                    disabled: props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: i32| {
                            if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.workdays_per_week = value as u8;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    }
                }
            }
            FormPair { label: "Expected hours per week".into(),
                FloatInput {
                    value: employee_work_details.expected_hours,
                    disabled: props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New,
                    step: 1.0,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: f32| {
                            if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::New
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.expected_hours = value;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    }
                }
            }
            FormGroup {
                span {
                    "Holiday in hours: "
                    "{employee_work_details.holiday_hours()}"
                }
            }
            FormGroup {
                span {
                    "Workday in hours: "
                    "{employee_work_details.vacation_day_in_hours()}"
                }
            }
            FormGroup {
                Button {
                    on_click: {
                        move |_| {
                            props.on_cancel.call(());
                        }
                    },
                    "{props.cancel_label}"
                }
                if props.show_save_button {
                    Button {
                        on_click: {
                            move |_| {
                                props.on_save.call(());
                            }
                        },
                        "Save"
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct EmployeeWorkDetailsFormPlainProps {
    pub employee_work_details_form_type: EmployeeWorkDetailsFormType,

    pub on_save: Option<EventHandler<()>>,
    pub on_cancel: EventHandler<()>,
}

#[component]
pub fn EmployeeWorkDetailsForm(props: EmployeeWorkDetailsFormPlainProps) -> Element {
    let employee_work_details = EMPLOYEE_WORK_DETAILS_STORE.read().clone();
    let employee_work_details_service = use_coroutine_handle::<EmployeeWorkDetailsAction>();
    rsx! {
        EmployeeWorkDetailsFormPlain {
            employee_work_details: employee_work_details.selected_employee_work_details,
            sales_person: employee_work_details.selected_sales_person.clone(),
            employee_work_details_form_type: props.employee_work_details_form_type,
            cancel_label: if props.on_save.is_some() { "Cancel".into() } else { "Close".into() },
            on_update_employee_work_details: move |working_hours| {
                employee_work_details_service
                    .send(EmployeeWorkDetailsAction::UpdateWorkingHours(working_hours));
            },
            on_save: move |()| {
                if let Some(on_save) = &props.on_save {
                    match props.employee_work_details_form_type {
                        EmployeeWorkDetailsFormType::New => {
                            employee_work_details_service.send(EmployeeWorkDetailsAction::Save);
                        }
                        EmployeeWorkDetailsFormType::Edit => {
                            employee_work_details_service
                                .send(EmployeeWorkDetailsAction::Update);
                        }
                        _ => {
                            tracing::error!("Cannot save read only employee work details");
                        }
                    }
                    on_save.call(());
                }
            },
            show_save_button: props.on_save.is_some(),
            on_cancel: props.on_cancel.clone()
        }
    }
}
