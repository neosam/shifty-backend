use futures_util::StreamExt;

use crate::{
    component::TopBar,
    i18n::Key,
    loader,
    service::{
        billing_period::{BillingPeriodAction, BILLING_PERIOD_STORE},
        config::CONFIG,
        i18n::I18N,
        text_template::{handle_text_template_action, TextTemplateAction, TEXT_TEMPLATE_STORE},
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
    let mut show_paid = use_signal(|| true); // Default: checked - show only paid
    let mut show_active = use_signal(|| true); // Default: checked - show only active
    
    // Custom report states
    let mut selected_template_id = use_signal(|| None::<Uuid>);
    let mut custom_report_result = use_signal(|| None::<String>);
    let mut generating_report = use_signal(|| false);
    let mut copy_status = use_signal(|| None::<String>);
    
    // Load billing period templates for report generation
    use_effect(move || {
        spawn(async move {
            handle_text_template_action(TextTemplateAction::LoadTemplatesByType("billing-period".to_string())).await;
        });
    });

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

    // Helper function to get sales person paid status by ID
    let get_sales_person_is_paid = |sales_person_id: Uuid| -> bool {
        if let Some(Ok(persons)) = sales_persons.read().as_ref() {
            if let Some(person) = persons.iter().find(|person| person.id == sales_person_id) {
                person.is_paid
            } else {
                false
            }
        } else {
            false
        }
    };

    // Helper function to get sales person active status by ID
    let get_sales_person_is_active = |sales_person_id: Uuid| -> bool {
        if let Some(Ok(persons)) = sales_persons.read().as_ref() {
            if let Some(person) = persons.iter().find(|person| person.id == sales_person_id) {
                !person.inactive // Note: inactive field is inverted (inactive = false means active = true)
            } else {
                true // Default to active if not found
            }
        } else {
            true // Default to active if data not loaded
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

                    // Custom Report Section
                    div { class: "bg-white shadow rounded-lg p-6 mb-6",
                        h2 { class: "text-xl font-semibold mb-4", "{i18n.t(Key::CustomReports)}" }
                        
                        div { class: "space-y-6",
                            // Template Selection and Generation
                            div {
                                h3 { class: "text-lg font-medium mb-3", "{i18n.t(Key::GenerateReport)}" }
                                
                                // Template Selection
                                div { class: "mb-4",
                                    label { class: "block text-sm font-medium text-gray-700 mb-2", 
                                        "{i18n.t(Key::SelectTemplate)} ({TEXT_TEMPLATE_STORE.read().filtered_templates.len()} billing period templates available)" 
                                    }
                                    select {
                                        class: "w-full p-2 border border-gray-300 rounded-md",
                                        value: selected_template_id.read().as_ref().map(|id| id.to_string()).unwrap_or_default(),
                                        onchange: move |event| {
                                            if let Ok(uuid) = Uuid::parse_str(&event.value()) {
                                                selected_template_id.set(Some(uuid));
                                            } else {
                                                selected_template_id.set(None);
                                            }
                                        },
                                        option { value: "", "Select a template..." }
                                        for template in TEXT_TEMPLATE_STORE.read().filtered_templates.iter() {
                                            option { 
                                                value: "{template.id}",
                                                if let Some(ref name) = template.name {
                                                    "{name}"
                                                } else {
                                                    "{template.template_text.chars().take(50).collect::<String>()}..."
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                // Generate Button
                                button {
                                    onclick: move |_| {
                                        if let Some(template_id) = *selected_template_id.read() {
                                            let config = CONFIG.read().clone();
                                            let billing_period_id = billing_period_id;
                                            spawn(async move {
                                                generating_report.set(true);
                                                custom_report_result.set(None);
                                                
                                                match loader::generate_custom_report(config, billing_period_id, template_id).await {
                                                    Ok(report) => {
                                                        custom_report_result.set(Some(report));
                                                    }
                                                    Err(e) => {
                                                        custom_report_result.set(Some(format!("Error generating report: {}", e)));
                                                    }
                                                }
                                                
                                                generating_report.set(false);
                                            });
                                        }
                                    },
                                    disabled: selected_template_id.read().is_none() || *generating_report.read(),
                                    class: if *generating_report.read() {
                                        "bg-gray-400 text-white font-bold py-2 px-4 rounded cursor-not-allowed"
                                    } else {
                                        "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                                    },
                                    if *generating_report.read() {
                                        "{i18n.t(Key::GeneratingReport)}"
                                    } else {
                                        "{i18n.t(Key::GenerateReport)}"
                                    }
                                }
                            }
                            
                            // Report Result (moved below generation form for full width)
                            {
                                let report_opt = custom_report_result.read().clone();
                                if let Some(report) = report_opt {
                                    rsx! {
                                        div { class: "border-t pt-6",
                                            div { class: "flex justify-between items-center mb-3",
                                                h3 { class: "text-lg font-medium", "{i18n.t(Key::GeneratedReport)}" }
                                                div { class: "flex items-center gap-2",
                                                    button {
                                                        onclick: move |_| {
                                                            let report_text = report.clone();
                                                            let i18n_copy = I18N.read().clone();
                                                            spawn(async move {
                                                                copy_status.set(None);
                                                                match crate::js::copy_to_clipboard(&report_text).await {
                                                                    Ok(_) => copy_status.set(Some(i18n_copy.t(Key::CopiedToClipboard).to_string())),
                                                                    Err(_) => copy_status.set(Some(i18n_copy.t(Key::CopyFailed).to_string())),
                                                                }
                                                                // Clear status after 3 seconds
                                                                spawn(async move {
                                                                    gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
                                                                    copy_status.set(None);
                                                                });
                                                            });
                                                        },
                                                        class: "bg-green-500 hover:bg-green-700 text-white font-bold py-1 px-3 rounded text-sm",
                                                        "{i18n.t(Key::CopyToClipboard)}"
                                                    }
                                                    if let Some(status) = copy_status.read().clone() {
                                                        span { class: "text-sm text-green-600 font-medium", "{status}" }
                                                    }
                                                }
                                            }
                                            div { class: "bg-gray-50 p-4 rounded-lg border",
                                                pre { class: "whitespace-pre-wrap text-sm font-mono overflow-x-auto", "{report}" }
                                            }
                                        }
                                    }
                                } else {
                                    rsx! { div {} }
                                }
                            }
                        }
                    }

                    // Sales Persons Section
                    div { class: "bg-white shadow rounded-lg p-6 mb-6",
                        div { class: "mb-4",
                            h2 { class: "text-xl font-semibold mb-4", 
                                "{i18n.t(Key::SalesPersons)} ({billing_period.sales_persons.len()} total)"
                            }
                            
                            // Filter controls
                            div { class: "flex flex-col md:flex-row gap-4 items-start md:items-center",
                                // Checkbox filters
                                div { class: "flex gap-6",
                                    label { class: "flex items-center gap-2 text-sm",
                                        input {
                                            r#type: "checkbox",
                                            class: "rounded border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50",
                                            checked: *show_paid.read(),
                                            onchange: move |event| show_paid.set(event.checked()),
                                        }
                                        span { "{i18n.t(Key::ShowPaid)}" }
                                    }
                                    label { class: "flex items-center gap-2 text-sm",
                                        input {
                                            r#type: "checkbox",
                                            class: "rounded border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50",
                                            checked: *show_active.read(),
                                            onchange: move |event| show_active.set(event.checked()),
                                        }
                                        span { "{i18n.t(Key::ShowActive)}" }
                                    }
                                }
                                
                                // Text filter
                                div { class: "w-full md:w-80",
                                    input {
                                        class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                        r#type: "text",
                                        placeholder: "{i18n.t(Key::FilterSalesPersonsByName)}",
                                        value: "{filter_text.read()}",
                                        oninput: move |event| filter_text.set(event.value()),
                                    }
                                }
                            }
                        }
                        
                        if billing_period.sales_persons.is_empty() {
                            p { class: "text-gray-500 italic", "{i18n.t(Key::NoSalesPersonsInBillingPeriod)}" }
                        } else {
                            // Filter and sort sales persons
                            {
                                let filter_text_lower = filter_text.read().to_lowercase();
                                let show_paid_val = *show_paid.read();
                                let show_active_val = *show_active.read();
                                
                                let mut filtered_sales_persons: Vec<_> = billing_period.sales_persons
                                    .iter()
                                    .filter(|sales_person| {
                                        // Name filter
                                        let name_matches = if filter_text_lower.is_empty() {
                                            true
                                        } else {
                                            let sales_person_name = get_sales_person_name(sales_person.sales_person_id).to_lowercase();
                                            sales_person_name.contains(&filter_text_lower)
                                        };
                                        
                                        // Active filter - be very explicit about the logic
                                        let is_active = get_sales_person_is_active(sales_person.sales_person_id);
                                        let active_filter_matches = if show_active_val {
                                            // When "Active" checkbox is checked: only show active employees
                                            // This means: exclude inactive employees
                                            is_active
                                        } else {
                                            // When "Active" checkbox is unchecked: show all employees
                                            true
                                        };
                                        
                                        // Paid filter
                                        let is_paid = get_sales_person_is_paid(sales_person.sales_person_id);
                                        let paid_filter_matches = if show_paid_val {
                                            is_paid // Only show paid when checked
                                        } else {
                                            true // Show all (paid and unpaid) when unchecked
                                        };
                                        
                                        name_matches && active_filter_matches && paid_filter_matches
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
                                            if filter_text.read().is_empty() {
                                                "No sales persons match the current filters."
                                            } else {
                                                {i18n.t(Key::NoSalesPersonsMatchFilter).replace("{filter}", &filter_text.read())}
                                            }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        p { class: "text-sm text-gray-600 mb-4", 
                                            "Showing {filtered_sales_persons.len()} of {billing_period.sales_persons.len()} sales persons"
                                        }
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
                                                                        div { class: "text-xs font-medium text-gray-600 uppercase tracking-wide", 
                                                                            {
                                                                                // Translate known value types
                                                                                let translated = match key.to_uppercase().as_str() {
                                                                                    "BALANCE" => i18n.t(Key::Balance).to_string(),
                                                                                    "EXPECTED_HOURS" => i18n.t(Key::ExpectedHours).to_string(),
                                                                                    "OVERALL" => i18n.t(Key::Overall).to_string(),
                                                                                    _ => key.clone(),
                                                                                };
                                                                                translated
                                                                            }
                                                                        }
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