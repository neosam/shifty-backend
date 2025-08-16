use futures_util::StreamExt;

use crate::{
    component::TopBar,
    i18n::Key,
    loader,
    service::{
        billing_period::{BillingPeriodAction, BILLING_PERIOD_STORE},
        config::CONFIG,
        i18n::I18N,
    },
};
use dioxus::prelude::*;
use uuid::Uuid;

pub enum BillingPeriodDetailsAction {
    LoadBillingPeriod,
}

#[derive(Clone, PartialEq, Props)]
pub struct BillingPeriodDetailsProps {
    pub billing_period_id: String,
}

#[component]
pub fn BillingPeriodDetails(props: BillingPeriodDetailsProps) -> Element {
    let billing_period_id = match Uuid::parse_str(&props.billing_period_id) {
        Ok(billing_period_id) => billing_period_id,
        Err(err) => {
            return rsx! { 
                TopBar {}
                div { class: "ml-1 mr-1 pt-4 md:m-8",
                    "{I18N.read().t(Key::InvalidBillingPeriodId)}: {err}" 
                }
            };
        }
    };

    let billing_period_service = use_coroutine_handle::<BillingPeriodAction>();
    let billing_periods = BILLING_PERIOD_STORE.read().clone();
    let i18n = I18N.read().clone();

    // Load sales persons to show names instead of IDs
    let config = CONFIG.read().clone();
    let sales_persons = use_resource(move || loader::load_sales_persons(config.clone()));

    // Filter state for sales persons
    let mut filter_text = use_signal(|| String::new());

    let _billing_period_loader = use_coroutine({
        to_owned![billing_period_id];
        move |mut rx: UnboundedReceiver<BillingPeriodDetailsAction>| async move {
            // Load the specific billing period when page loads
            billing_period_service.send(BillingPeriodAction::LoadBillingPeriod(billing_period_id));
            
            while let Some(action) = rx.next().await {
                match action {
                    BillingPeriodDetailsAction::LoadBillingPeriod => {
                        billing_period_service.send(BillingPeriodAction::LoadBillingPeriod(billing_period_id));
                    }
                }
            }
        }
    });

    let selected_billing_period = billing_periods.selected_billing_period.as_ref();

    // Helper function to get sales person name by ID
    let get_sales_person_name = |sales_person_id: Uuid| -> String {
        if let Some(Ok(persons)) = sales_persons.read().as_ref() {
            persons
                .iter()
                .find(|person| person.id == sales_person_id)
                .map(|person| person.name.to_string())
                .unwrap_or_else(|| format!("Unknown ({})", sales_person_id))
        } else {
            sales_person_id.to_string()
        }
    };

    rsx! {
        TopBar {}

        div { class: "ml-1 mr-1 pt-4 md:m-8",
            if let Some(billing_period) = selected_billing_period {
                div { class: "max-w-4xl mx-auto",
                    // Header
                    div { class: "mb-6",
                        h1 { class: "text-3xl font-bold mb-2", 
                            "{i18n.t(Key::BillingPeriodDetails)}"
                        }
                        div { class: "flex items-center space-x-4",
                            span { class: "text-lg text-gray-600",
                                "{i18n.format_date(&billing_period.start_date)} - {i18n.format_date(&billing_period.end_date)}"
                            }
                            if billing_period.deleted_at.is_none() {
                                span { class: "px-3 py-1 bg-green-100 text-green-800 text-sm rounded-full", 
                                    "{i18n.t(Key::Active)}" 
                                }
                            } else {
                                span { class: "px-3 py-1 bg-red-100 text-red-800 text-sm rounded-full", 
                                    "{i18n.t(Key::Deleted)}" 
                                }
                            }
                        }
                    }

                    // Basic Information Card
                    div { class: "bg-white shadow rounded-lg p-6 mb-6",
                        h2 { class: "text-xl font-semibold mb-4", "{i18n.t(Key::BasicInformation)}" }
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "{i18n.t(Key::StartDate)}" }
                                p { class: "text-sm text-gray-900", "{i18n.format_date(&billing_period.start_date)}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "{i18n.t(Key::EndDate)}" }
                                p { class: "text-sm text-gray-900", "{i18n.format_date(&billing_period.end_date)}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "{i18n.t(Key::CreatedAt)}" }
                                p { class: "text-sm text-gray-900", "{i18n.format_date(&billing_period.created_at.date())}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "{i18n.t(Key::CreatedBy)}" }
                                p { class: "text-sm text-gray-900", "{billing_period.created_by.as_ref()}" }
                            }
                            if let Some(deleted_at) = billing_period.deleted_at {
                                div {
                                    label { class: "block text-sm font-medium text-gray-700 mb-1", "{i18n.t(Key::DeletedAt)}" }
                                    p { class: "text-sm text-gray-900", "{i18n.format_date(&deleted_at.date())}" }
                                }
                            }
                            if let Some(deleted_by) = &billing_period.deleted_by {
                                div {
                                    label { class: "block text-sm font-medium text-gray-700 mb-1", "{i18n.t(Key::DeletedBy)}" }
                                    p { class: "text-sm text-gray-900", "{deleted_by.as_ref()}" }
                                }
                            }
                        }
                    }

                    // Sales Persons Section
                    div { class: "bg-white shadow rounded-lg p-6 mb-6",
                        div { class: "flex justify-between items-center mb-4",
                            h2 { class: "text-xl font-semibold", 
                                "{i18n.t(Key::SalesPersons)} ({billing_period.sales_persons.len()})"
                            }
                            div { class: "w-80",
                                input {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                    r#type: "text",
                                    placeholder: "{i18n.t(Key::FilterSalesPersonsByName)}",
                                    value: "{filter_text.read()}",
                                    oninput: move |event| filter_text.set(event.value()),
                                }
                            }
                        }
                        
                        if billing_period.sales_persons.is_empty() {
                            p { class: "text-gray-500 italic", "{i18n.t(Key::NoSalesPersonsInBillingPeriod)}" }
                        } else {
                            // Filter and sort sales persons
                            {
                                let filter_text_lower = filter_text.read().to_lowercase();
                                let mut filtered_sales_persons: Vec<_> = billing_period.sales_persons
                                    .iter()
                                    .filter(|sales_person| {
                                        if filter_text_lower.is_empty() {
                                            true
                                        } else {
                                            let sales_person_name = get_sales_person_name(sales_person.sales_person_id).to_lowercase();
                                            sales_person_name.contains(&filter_text_lower)
                                        }
                                    })
                                    .collect();

                                // Sort by sales person name
                                filtered_sales_persons.sort_by(|a, b| {
                                    let name_a = get_sales_person_name(a.sales_person_id).to_lowercase();
                                    let name_b = get_sales_person_name(b.sales_person_id).to_lowercase();
                                    name_a.cmp(&name_b)
                                });

                                if filtered_sales_persons.is_empty() {
                                    rsx! {
                                        p { class: "text-gray-500 italic", 
                                            {
                                                i18n.t(Key::NoSalesPersonsMatchFilter).replace("{filter}", &filter_text.read())
                                            }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        div { class: "space-y-4",
                                            for sales_person in filtered_sales_persons.iter() {
                                                div { class: "border border-gray-200 rounded-lg p-4",
                                                    div { class: "flex justify-between items-start mb-3",
                                                        div {
                                                            h3 { class: "text-lg font-medium text-blue-600", 
                                                                "{get_sales_person_name(sales_person.sales_person_id)}" 
                                                            }
                                                        }
                                                        if sales_person.deleted_at.is_none() {
                                                            span { class: "px-2 py-1 bg-green-100 text-green-800 text-xs rounded-full", "{i18n.t(Key::Active)}" }
                                                        } else {
                                                            span { class: "px-2 py-1 bg-red-100 text-red-800 text-xs rounded-full", "{i18n.t(Key::Deleted)}" }
                                                        }
                                                    }
                                                    
                                                    // Values/Metrics
                                                    if !sales_person.values.is_empty() {
                                                        div {
                                                            h4 { class: "text-sm font-medium text-gray-700 mb-2", "{i18n.t(Key::Values)}" }
                                                            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3",
                                                                for (key, value) in sales_person.values.iter() {
                                                                    div { class: "bg-gray-50 p-3 rounded",
                                                                        div { class: "text-xs font-medium text-gray-600 uppercase tracking-wide", "{key}" }
                                                                        div { class: "mt-1",
                                                                            p { class: "text-sm", "{i18n.t(Key::Delta)}: {value.value_delta:.2}" }
                                                                            p { class: "text-sm", "{i18n.t(Key::YtdFrom)}: {value.value_ytd_from:.2}" }
                                                                            p { class: "text-sm", "{i18n.t(Key::YtdTo)}: {value.value_ytd_to:.2}" }
                                                                            p { class: "text-sm", "{i18n.t(Key::FullYear)}: {value.value_full_year:.2}" }
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
                    }
                }
            } else {
                div { class: "flex items-center justify-center py-12",
                    div { class: "text-center",
                        div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4" }
                        p { class: "text-gray-500", "{i18n.t(Key::LoadingBillingPeriodDetails)}" }
                    }
                }
            }
        }
    }
}