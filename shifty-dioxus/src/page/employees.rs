use crate::{
    component::{EmployeeShort, Modal, TopBar},
    i18n::Key,
    js, loader,
    router::Route,
    service::{
        billing_period::{BillingPeriodAction, BILLING_PERIOD_STORE},
        config::CONFIG,
        i18n::I18N,
    },
};
use dioxus::prelude::*;
use futures_util::StreamExt;
use time::macros::format_description;

pub enum EmployeesPageAction {
    ShowCreateBillingPeriodDialog,
    HideCreateBillingPeriodDialog, 
    CreateBillingPeriod(String),
}

#[component]
pub fn Employees() -> Element {
    let year = use_signal(|| js::get_current_year());
    let week_until = if *year.read() == js::get_current_year() {
        js::get_current_week()
    } else {
        52
    };
    let config = CONFIG.read().clone();
    let employees =
        use_resource(move || loader::load_employees(config.to_owned(), *year.read(), week_until));
    
    let billing_period_service = use_coroutine_handle::<BillingPeriodAction>();
    let billing_periods = BILLING_PERIOD_STORE.read().clone();
    let i18n = I18N.read().clone();
    
    // Dialog state
    let mut show_create_billing_period_dialog = use_signal(|| false);
    let mut end_date = use_signal(|| {
        let date_format = format_description!("[year]-[month]-[day]");
        js::current_datetime().date().format(&date_format).unwrap()
    });
    
    let page_action_handler = use_coroutine(
        move |mut rx: UnboundedReceiver<EmployeesPageAction>| async move {
            while let Some(action) = rx.next().await {
                match action {
                    EmployeesPageAction::ShowCreateBillingPeriodDialog => {
                        show_create_billing_period_dialog.set(true);
                    }
                    EmployeesPageAction::HideCreateBillingPeriodDialog => {
                        show_create_billing_period_dialog.set(false);
                    }
                    EmployeesPageAction::CreateBillingPeriod(date_string) => {
                        if let Ok(parsed_date) = time::Date::parse(&date_string, &format_description!("[year]-[month]-[day]")) {
                            billing_period_service.send(BillingPeriodAction::CreateBillingPeriod(parsed_date));
                            show_create_billing_period_dialog.set(false);
                        }
                    }
                }
            }
        }
    );
    
    let _billing_period_loader = use_coroutine({
        move |mut rx: UnboundedReceiver<()>| async move {
            billing_period_service.send(BillingPeriodAction::LoadBillingPeriods);
            while let Some(()) = rx.next().await {
                billing_period_service.send(BillingPeriodAction::LoadBillingPeriods);
            }
        }
    });

    rsx! {
        TopBar {}

        // Modal for creating new billing period
        if *show_create_billing_period_dialog.read() {
            Modal {
                div { class: "space-y-6",
                    h2 { class: "text-xl font-bold mb-4", "{i18n.t(Key::CreateBillingPeriod)}" }
                    
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2", "{i18n.t(Key::EndDate)}" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            r#type: "date",
                            value: "{end_date.read()}",
                            oninput: move |event| end_date.set(event.value()),
                            required: true
                        }
                        p { class: "text-sm text-gray-500 mt-1", 
                            "{i18n.t(Key::SelectEndDateForNewBillingPeriod)}" 
                        }
                    }
                    
                    div { class: "flex justify-end space-x-4",
                        button {
                            class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-gray-500",
                            onclick: move |_| page_action_handler.send(EmployeesPageAction::HideCreateBillingPeriodDialog),
                            "{i18n.t(Key::Cancel)}"
                        }
                        button {
                            class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500",
                            onclick: move |_| {
                                page_action_handler.send(EmployeesPageAction::CreateBillingPeriod(end_date.read().clone()));
                            },
                            "{i18n.t(Key::CreateBillingPeriod)}"
                        }
                    }
                }
            }
        }

        div { class: "ml-1 mr-1 pt-4 md:m-8",
            // Employees Section
            div { class: "mb-8",
                h2 { class: "text-2xl font-bold mb-4", "{i18n.t(Key::Employees)}" }
                match &*employees.read_unchecked() {
                    Some(Ok(employee)) => {
                        let mut employee = employee.iter().cloned().collect::<Vec<_>>();
                        employee.sort_by(|a, b| a.sales_person.name.cmp(&b.sales_person.name));
                        rsx! {
                            for employee in employee.iter() {
                                Link {
                                    to: Route::EmployeeDetails {
                                        employee_id: employee.sales_person.id.to_string(),
                                    },
                                    EmployeeShort { employee: employee.clone() }
                                }
                            }
                        }
                    }
                    Some(Err(err)) => {
                        rsx! {
                        "Error while loading employees: {err}"
                        }
                    }
                    None => {
                        rsx! { "Loading employees." }
                    }
                }
            }

            // Billing Periods Section
            div { class: "mb-8",
                div { class: "flex justify-between items-center mb-4",
                    h2 { class: "text-2xl font-bold mb-4", "{i18n.t(Key::BillingPeriods)}" }
                    button {
                        class: "px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 transition-colors",
                        onclick: move |_| page_action_handler.send(EmployeesPageAction::ShowCreateBillingPeriodDialog),
                        "{i18n.t(Key::CreateNewBillingPeriod)}"
                    }
                }
                if billing_periods.billing_periods.is_empty() {
                    div { class: "text-gray-500", "{i18n.t(Key::LoadingBillingPeriods)}" }
                } else {
                    div { class: "grid gap-4",
                        for billing_period in billing_periods.billing_periods.iter() {
                            Link {
                                to: Route::BillingPeriodDetails {
                                    billing_period_id: billing_period.id.to_string(),
                                },
                                div { 
                                    class: "bg-white shadow rounded-lg p-4 border border-gray-200 hover:shadow-lg hover:border-blue-300 transition-all cursor-pointer",
                                    div { class: "flex justify-between items-center",
                                        div {
                                            h3 { class: "text-lg font-semibold text-blue-600 hover:text-blue-800", 
                                                "{i18n.t(Key::Period)}: {i18n.format_date(&billing_period.start_date)} - {i18n.format_date(&billing_period.end_date)}"
                                            }
                                            p { class: "text-sm text-gray-600", 
                                                "{i18n.t(Key::CreatedAt)}: {i18n.format_date(&billing_period.created_at.date())}"
                                            }
                                            p { class: "text-sm text-gray-600", 
                                                "{i18n.t(Key::CreatedBy)}: {billing_period.created_by.as_ref()}"
                                            }
                                            if !billing_period.sales_persons.is_empty() {
                                                p { class: "text-sm text-gray-500 mt-1", 
                                                    {
                                                        i18n.t(Key::SalesPersonsIncluded).replace("{count}", &billing_period.sales_persons.len().to_string())
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "flex items-center space-x-2",
                                            if billing_period.deleted_at.is_none() {
                                                span { class: "px-2 py-1 bg-green-100 text-green-800 text-xs rounded-full", 
                                                    "{i18n.t(Key::Active)}" 
                                                }
                                            } else {
                                                span { class: "px-2 py-1 bg-red-100 text-red-800 text-xs rounded-full", 
                                                    "{i18n.t(Key::Deleted)}" 
                                                }
                                            }
                                            svg { 
                                                class: "w-4 h-4 text-gray-400",
                                                fill: "none",
                                                stroke: "currentColor",
                                                view_box: "0 0 24 24",
                                                path { 
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round", 
                                                    stroke_width: "2",
                                                    d: "m9 18 6-6-6-6"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
