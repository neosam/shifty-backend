use crate::{
    component::{EmployeeShort, TopBar},
    js, loader,
    router::Route,
    service::{
        billing_period::{BillingPeriodAction, BILLING_PERIOD_STORE},
        config::CONFIG,
    },
};
use dioxus::prelude::*;
use futures_util::StreamExt;

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

        div { class: "ml-1 mr-1 pt-4 md:m-8",
            // Employees Section
            div { class: "mb-8",
                h2 { class: "text-2xl font-bold mb-4", "Employees" }
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
                h2 { class: "text-2xl font-bold mb-4", "Billing Periods" }
                if billing_periods.billing_periods.is_empty() {
                    div { class: "text-gray-500", "Loading billing periods..." }
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
                                                "Period: {billing_period.start_date} - {billing_period.end_date}"
                                            }
                                            p { class: "text-sm text-gray-600", 
                                                "Created: {billing_period.created_at.date()}"
                                            }
                                            p { class: "text-sm text-gray-600", 
                                                "Created by: {billing_period.created_by.as_ref()}"
                                            }
                                            if !billing_period.sales_persons.is_empty() {
                                                p { class: "text-sm text-gray-500 mt-1", 
                                                    "{billing_period.sales_persons.len()} sales persons included"
                                                }
                                            }
                                        }
                                        div { class: "flex items-center space-x-2",
                                            if billing_period.deleted_at.is_none() {
                                                span { class: "px-2 py-1 bg-green-100 text-green-800 text-xs rounded-full", 
                                                    "Active" 
                                                }
                                            } else {
                                                span { class: "px-2 py-1 bg-red-100 text-red-800 text-xs rounded-full", 
                                                    "Deleted" 
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
