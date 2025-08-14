use futures_util::StreamExt;

use crate::{
    component::TopBar,
    loader,
    service::{
        billing_period::{BillingPeriodAction, BILLING_PERIOD_STORE},
        config::CONFIG,
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
                    "Invalid billing period id: {err}" 
                }
            };
        }
    };

    let billing_period_service = use_coroutine_handle::<BillingPeriodAction>();
    let billing_periods = BILLING_PERIOD_STORE.read().clone();

    // Load sales persons to show names instead of IDs
    let config = CONFIG.read().clone();
    let sales_persons = use_resource(move || loader::load_sales_persons(config.clone()));

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
                            "Billing Period Details"
                        }
                        div { class: "flex items-center space-x-4",
                            span { class: "text-lg text-gray-600",
                                "{billing_period.start_date} - {billing_period.end_date}"
                            }
                            if billing_period.deleted_at.is_none() {
                                span { class: "px-3 py-1 bg-green-100 text-green-800 text-sm rounded-full", 
                                    "Active" 
                                }
                            } else {
                                span { class: "px-3 py-1 bg-red-100 text-red-800 text-sm rounded-full", 
                                    "Deleted" 
                                }
                            }
                        }
                    }

                    // Basic Information Card
                    div { class: "bg-white shadow rounded-lg p-6 mb-6",
                        h2 { class: "text-xl font-semibold mb-4", "Basic Information" }
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "Start Date" }
                                p { class: "text-sm text-gray-900", "{billing_period.start_date}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "End Date" }
                                p { class: "text-sm text-gray-900", "{billing_period.end_date}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "Created At" }
                                p { class: "text-sm text-gray-900", "{billing_period.created_at}" }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "Created By" }
                                p { class: "text-sm text-gray-900", "{billing_period.created_by.as_ref()}" }
                            }
                            if let Some(deleted_at) = billing_period.deleted_at {
                                div {
                                    label { class: "block text-sm font-medium text-gray-700 mb-1", "Deleted At" }
                                    p { class: "text-sm text-gray-900", "{deleted_at}" }
                                }
                            }
                            if let Some(deleted_by) = &billing_period.deleted_by {
                                div {
                                    label { class: "block text-sm font-medium text-gray-700 mb-1", "Deleted By" }
                                    p { class: "text-sm text-gray-900", "{deleted_by.as_ref()}" }
                                }
                            }
                        }
                    }

                    // Sales Persons Section
                    div { class: "bg-white shadow rounded-lg p-6 mb-6",
                        h2 { class: "text-xl font-semibold mb-4", 
                            "Sales Persons ({billing_period.sales_persons.len()})"
                        }
                        
                        if billing_period.sales_persons.is_empty() {
                            p { class: "text-gray-500 italic", "No sales persons in this billing period." }
                        } else {
                            div { class: "space-y-4",
                                for sales_person in billing_period.sales_persons.iter() {
                                    div { class: "border border-gray-200 rounded-lg p-4",
                                        div { class: "flex justify-between items-start mb-3",
                                            div {
                                                h3 { class: "text-lg font-medium text-blue-600", 
                                                    "{get_sales_person_name(sales_person.sales_person_id)}" 
                                                }
                                                p { class: "text-xs text-gray-500", "ID: {sales_person.sales_person_id}" }
                                                p { class: "text-sm text-gray-600", "Created: {sales_person.created_at}" }
                                                p { class: "text-sm text-gray-600", "Created by: {sales_person.created_by.as_ref()}" }
                                            }
                                            if sales_person.deleted_at.is_none() {
                                                span { class: "px-2 py-1 bg-green-100 text-green-800 text-xs rounded-full", "Active" }
                                            } else {
                                                span { class: "px-2 py-1 bg-red-100 text-red-800 text-xs rounded-full", "Deleted" }
                                            }
                                        }
                                        
                                        // Values/Metrics
                                        if !sales_person.values.is_empty() {
                                            div {
                                                h4 { class: "text-sm font-medium text-gray-700 mb-2", "Values" }
                                                div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3",
                                                    for (key, value) in sales_person.values.iter() {
                                                        div { class: "bg-gray-50 p-3 rounded",
                                                            div { class: "text-xs font-medium text-gray-600 uppercase tracking-wide", "{key}" }
                                                            div { class: "mt-1",
                                                                p { class: "text-sm", "Delta: {value.value_delta}" }
                                                                p { class: "text-sm", "YTD From: {value.value_ytd_from}" }
                                                                p { class: "text-sm", "YTD To: {value.value_ytd_to}" }
                                                                p { class: "text-sm", "Full Year: {value.value_full_year}" }
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
                        p { class: "text-gray-500", "Loading billing period details..." }
                    }
                }
            }
        }
    }
}