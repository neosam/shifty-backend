use dioxus::prelude::*;
use futures_util::StreamExt;
use uuid::Uuid;

use crate::{
    i18n::Key,
    service::{
        i18n::I18N,
        text_template::{handle_text_template_action, TextTemplateAction, TEXT_TEMPLATE_STORE},
    },
    state::text_template::TextTemplate,
};

#[component]
pub fn TextTemplateManagement() -> Element {
    let mut show_form = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<Uuid>);
    let mut form_template_type = use_signal(|| "billing-period".to_string());
    let mut form_template_text = use_signal(|| "".to_string());

    let i18n = I18N.read().clone();
    let store = TEXT_TEMPLATE_STORE.read().clone();

    use_effect(move || {
        spawn(async move {
            handle_text_template_action(TextTemplateAction::LoadTemplates).await;
        });
    });

    let title = i18n.t(Key::TextTemplateManagement);
    let template_type_str = i18n.t(Key::TemplateType);
    let template_text_str = i18n.t(Key::TemplateText);
    let actions_str = i18n.t(Key::Actions);
    let add_new_str = i18n.t(Key::AddNew);
    let save_str = i18n.t(Key::Save);
    let cancel_str = i18n.t(Key::Cancel);
    let edit_str = i18n.t(Key::Edit);
    let delete_str = i18n.t(Key::Delete);

    let action_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<TextTemplateAction>| {
        async move {
            while let Some(action) = rx.next().await {
                handle_text_template_action(action).await;
            }
        }
    });

    let mut reset_form = move || {
        show_form.set(false);
        editing_id.set(None);
        form_template_type.set("billing-period".to_string());
        form_template_text.set("".to_string());
    };

    let save_template = move |_| {
        let template_type = form_template_type.read().clone();
        let template_text = form_template_text.read().clone();

        if template_type.trim().is_empty() || template_text.trim().is_empty() {
            return;
        }

        if let Some(id) = *editing_id.read() {
            // Update existing template
            let template = TextTemplate {
                id,
                template_type: template_type.into(),
                template_text: template_text.into(),
                created_at: None,
                created_by: None,
            };
            action_coroutine.send(TextTemplateAction::UpdateTemplate(id, template));
        } else {
            // Create new template
            let template = TextTemplate {
                id: Uuid::nil(),
                template_type: template_type.into(),
                template_text: template_text.into(),
                created_at: None,
                created_by: None,
            };
            action_coroutine.send(TextTemplateAction::SaveTemplate(template));
        }
        reset_form();
    };

    let mut edit_template = move |template: TextTemplate| {
        form_template_type.set(template.template_type.to_string());
        form_template_text.set(template.template_text.to_string());
        editing_id.set(Some(template.id));
        show_form.set(true);
    };

    let delete_template = move |template_id: Uuid| {
        action_coroutine.send(TextTemplateAction::DeleteTemplate(template_id));
    };

    rsx! {
        div { class: "p-6",
            h1 { class: "text-2xl font-bold mb-4", "{title}" }
            
            if !*show_form.read() {
                button {
                    onclick: move |_| show_form.set(true),
                    class: "mb-4 bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
                    "{add_new_str}"
                }
            }

            if *show_form.read() {
                div { class: "bg-gray-100 p-4 rounded-lg mb-6",
                    h2 { class: "text-lg font-semibold mb-4",
                        if editing_id.read().is_some() {
                            "{i18n.t(Key::EditTemplate)}"
                        } else {
                            "{i18n.t(Key::AddNewTemplate)}"
                        }
                    }
                    
                    div { class: "mb-4",
                        label { class: "block text-sm font-medium mb-2", "{template_type_str}" }
                        select {
                            class: "w-full p-2 border border-gray-300 rounded-md",
                            value: form_template_type.read().clone(),
                            onchange: move |event| form_template_type.set(event.value()),
                            option { value: "billing-period", "Billing Period" }
                            option { value: "employee-report", "Employee Report" }
                            option { value: "shift-plan", "Shift Plan" }
                        }
                    }
                    
                    div { class: "mb-4",
                        label { class: "block text-sm font-medium mb-2", "{template_text_str}" }
                        textarea {
                            class: "w-full p-2 border border-gray-300 rounded-md h-32",
                            value: form_template_text.read().clone(),
                            oninput: move |event| form_template_text.set(event.value()),
                            placeholder: "Enter your template text here..."
                        }
                    }
                    
                    div { class: "flex gap-2",
                        button {
                            onclick: save_template,
                            class: "bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded",
                            "{save_str}"
                        }
                        button {
                            onclick: move |_| reset_form(),
                            class: "bg-gray-500 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded",
                            "{cancel_str}"
                        }
                    }
                }
            }

            div { class: "overflow-x-auto",
                table { class: "min-w-full bg-white border border-gray-300",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "ID" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "{template_type_str}" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "Preview" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "{actions_str}" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        for template in store.filtered_templates.iter() {
                            tr { key: "{template.id}",
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900", "{template.id}" }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900", "{template.template_type}" }
                                td { class: "px-6 py-4 text-sm text-gray-900 max-w-xs truncate", 
                                    "{template.template_text}"
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                    div { class: "flex gap-2",
                                        button {
                                            onclick: {
                                                let template = template.clone();
                                                move |_| edit_template(template.clone())
                                            },
                                            class: "bg-blue-500 hover:bg-blue-700 text-white font-bold py-1 px-2 rounded text-xs",
                                            "{edit_str}"
                                        }
                                        button {
                                            onclick: {
                                                let template_id = template.id;
                                                move |_| delete_template(template_id)
                                            },
                                            class: "bg-red-500 hover:bg-red-700 text-white font-bold py-1 px-2 rounded text-xs",
                                            "{delete_str}"
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