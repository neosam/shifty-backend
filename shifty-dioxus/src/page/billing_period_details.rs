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
    
    // Custom report states
    let mut selected_template_id = use_signal(|| None::<Uuid>);
    let mut custom_report_result = use_signal(|| None::<String>);
    let mut generating_report = use_signal(|| false);
    
    // New template creation states
    let mut show_new_template_form = use_signal(|| false);
    let mut new_template_type = use_signal(|| "billing-period".to_string());
    let mut new_template_text = use_signal(|| "".to_string());
    let mut saving_template = use_signal(|| false);
    
    // Load billing period templates
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

    // Helper functions for new template creation
    let mut reset_new_template_form = move || {
        show_new_template_form.set(false);
        new_template_type.set("billing-period".to_string());
        new_template_text.set("".to_string());
    };

    let save_new_template = move |_| {
        let template_type = new_template_type.read().clone();
        let template_text = new_template_text.read().clone();

        if template_type.trim().is_empty() || template_text.trim().is_empty() {
            return;
        }

        let template = crate::state::text_template::TextTemplate {
            id: Uuid::nil(),
            template_type: template_type.into(),
            template_text: template_text.into(),
            created_at: None,
            created_by: None,
        };

        spawn(async move {
            saving_template.set(true);
            handle_text_template_action(TextTemplateAction::SaveTemplate(template)).await;
            // Reload templates to include the new one
            handle_text_template_action(TextTemplateAction::LoadTemplatesByType("billing-period".to_string())).await;
            saving_template.set(false);
            reset_new_template_form();
        });
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
                            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                                div {
                                    h3 { class: "text-lg font-medium mb-3", "{i18n.t(Key::GenerateReport)}" }
                                    
                                    // Template Selection
                                    div { class: "mb-4",
                                        label { class: "block text-sm font-medium text-gray-700 mb-2", "{i18n.t(Key::SelectTemplate)}" }
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
                                                    "{template.template_type} - {template.template_text.chars().take(50).collect::<String>()}..."
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
                                
                                // Report Result
                                if let Some(ref report) = *custom_report_result.read() {
                                    div {
                                        h3 { class: "text-lg font-medium mb-3", "{i18n.t(Key::GeneratedReport)}" }
                                        div { class: "bg-gray-50 p-4 rounded-lg border",
                                            pre { class: "whitespace-pre-wrap text-sm font-mono", "{report}" }
                                        }
                                    }
                                }
                            }
                            
                            // Create New Template Section
                            div { class: "border-t pt-6",
                                div { class: "flex justify-between items-center mb-4",
                                    h3 { class: "text-lg font-medium", "{i18n.t(Key::CreateNewTemplate)}" }
                                    if !*show_new_template_form.read() {
                                        button {
                                            onclick: move |_| show_new_template_form.set(true),
                                            class: "bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded",
                                            "{i18n.t(Key::AddNew)}"
                                        }
                                    }
                                }
                                
                                if *show_new_template_form.read() {
                                    div { class: "bg-gray-50 p-4 rounded-lg",
                                        div { class: "mb-4",
                                            label { class: "block text-sm font-medium text-gray-700 mb-2", "{i18n.t(Key::TemplateType)}" }
                                            select {
                                                class: "w-full p-2 border border-gray-300 rounded-md",
                                                value: new_template_type.read().clone(),
                                                onchange: move |event| new_template_type.set(event.value()),
                                                option { value: "billing-period", "Billing Period" }
                                                option { value: "employee-report", "Employee Report" }
                                                option { value: "shift-plan", "Shift Plan" }
                                            }
                                        }
                                        
                                        div { class: "mb-4",
                                            label { class: "block text-sm font-medium text-gray-700 mb-2", "{i18n.t(Key::TemplateText)}" }
                                            textarea {
                                                class: "w-full p-2 border border-gray-300 rounded-md h-32",
                                                value: new_template_text.read().clone(),
                                                oninput: move |event| new_template_text.set(event.value()),
                                                placeholder: "Enter your template text here..."
                                            }
                                        }
                                        
                                        div { class: "flex gap-2",
                                            button {
                                                onclick: save_new_template,
                                                disabled: new_template_type.read().trim().is_empty() || new_template_text.read().trim().is_empty() || *saving_template.read(),
                                                class: if *saving_template.read() {
                                                    "bg-gray-400 text-white font-bold py-2 px-4 rounded cursor-not-allowed"
                                                } else {
                                                    "bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded"
                                                },
                                                if *saving_template.read() {
                                                    "{i18n.t(Key::Saving)}"
                                                } else {
                                                    "{i18n.t(Key::Save)}"
                                                }
                                            }
                                            button {
                                                onclick: move |_| reset_new_template_form(),
                                                class: "bg-gray-500 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded",
                                                "{i18n.t(Key::Cancel)}"
                                            }
                                        }
                                    }
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