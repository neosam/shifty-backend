use dioxus::prelude::*;

use crate::{
    base_types::ImStr,
    component::base_components::{
        Button, Checkbox, DateInput, FloatInput, Form, FormGroup, FormPair, Header, IntegerInput,
    },
    i18n,
    service::{
        employee_work_details::EmployeeWorkDetailsAction,
        employee_work_details::EMPLOYEE_WORK_DETAILS_STORE, i18n::I18N,
    },
    state::{employee_work_details::EmployeeWorkDetails, shiftplan::SalesPerson},
};

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
    let i18n = I18N.read().clone();

    let title = i18n.t_m(
        i18n::Key::AddWorkDetailsFormTitle,
        [("name", props.sales_person.name.as_ref())].into(),
    );
    let from_label = i18n.t(i18n::Key::FromLabel);
    let to_label = i18n.t(i18n::Key::ToLabel);
    let workdays_label = i18n.t(i18n::Key::WorkdaysLabel);
    let expected_hours_per_week_label = i18n.t(i18n::Key::ExpectedHoursPerWeekLabel);
    let days_per_week_label = i18n.t(i18n::Key::DaysPerWeekLabel);
    let vacation_days_label = i18n.t(i18n::Key::VacationEntitlementsPerYearLabel);
    let holiday_in_hours_label = i18n.t(i18n::Key::HolidaysInHoursLabel);
    let workday_in_hours_label = i18n.t(i18n::Key::WorkdaysInHoursLabel);

    let monday_label = i18n.t(i18n::Key::Monday);
    let tuesday_label = i18n.t(i18n::Key::Tuesday);
    let wednesday_label = i18n.t(i18n::Key::Wednesday);
    let thursday_label = i18n.t(i18n::Key::Thursday);
    let friday_label = i18n.t(i18n::Key::Friday);
    let saturday_label = i18n.t(i18n::Key::Saturday);
    let sunday_label = i18n.t(i18n::Key::Sunday);

    rsx! {
        Header { "{title}" }
        Form {
            FormPair { label: from_label,
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
                        },
                    }
                    span { class: "text-xs text-gray-500", "({from_day}, Week {from_week}, {from_year})" }
                }
            }
            FormPair { label: to_label,
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
                        },
                    }
                    span { class: "text-xs text-gray-500", "({to_day}, Week {to_week}, {to_year})" }
                }
            }
            FormPair { label: workdays_label,
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
                    "{monday_label}"
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
                    "{tuesday_label}"
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
                    "{wednesday_label}"
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
                    "{thursday_label}"
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
                    "{friday_label}"
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
                    "{saturday_label}"
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
                    "{sunday_label}"
                }
            }
            FormPair { label: expected_hours_per_week_label,
                FloatInput {
                    value: employee_work_details.expected_hours,
                    disabled: props.employee_work_details_form_type == EmployeeWorkDetailsFormType::ReadOnly,
                    step: 1.0,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: f32| {
                            if props.employee_work_details_form_type == EmployeeWorkDetailsFormType::ReadOnly
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.expected_hours = value;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    },
                }
            }
            FormPair { label: days_per_week_label,
                IntegerInput {
                    value: employee_work_details.workdays_per_week as i32,
                    disabled: props.employee_work_details_form_type == EmployeeWorkDetailsFormType::ReadOnly,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: i32| {
                            if props.employee_work_details_form_type
                                == EmployeeWorkDetailsFormType::ReadOnly
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.workdays_per_week = value as u8;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    },
                }
            }
            if props.employee_work_details_form_type != EmployeeWorkDetailsFormType::ReadOnly {
                FormPair { label: vacation_days_label,
                    IntegerInput {
                        value: employee_work_details.vacation_days as i32,
                        disabled: props.employee_work_details_form_type == EmployeeWorkDetailsFormType::ReadOnly,
                        on_change: {
                            to_owned![employee_work_details];
                            move |value: i32| {
                                if props.employee_work_details_form_type
                                    == EmployeeWorkDetailsFormType::ReadOnly
                                {
                                    return ();
                                }
                                let mut employee_work_details = employee_work_details.clone();
                                employee_work_details.vacation_days = value as u8;
                                props.on_update_employee_work_details.call(employee_work_details);
                            }
                        },
                    }
                }
            }
            FormPair { label: "Dynamische Stunden".into(),
                Checkbox {
                    value: employee_work_details.dynamic,
                    disabled: props.employee_work_details_form_type == EmployeeWorkDetailsFormType::ReadOnly,
                    on_change: {
                        to_owned![employee_work_details];
                        move |value: bool| {
                            if props.employee_work_details_form_type == EmployeeWorkDetailsFormType::ReadOnly
                            {
                                return ();
                            }
                            let mut employee_work_details = employee_work_details.clone();
                            employee_work_details.dynamic = value;
                            props.on_update_employee_work_details.call(employee_work_details);
                        }
                    },
                    ""
                }
            }
            FormGroup {
                span {
                    "{holiday_in_hours_label}: "
                    "{employee_work_details.holiday_hours()}"
                }
            }
            FormGroup {
                span {
                    "{workday_in_hours_label}: "
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
            on_cancel: props.on_cancel.clone(),
        }
    }
}
