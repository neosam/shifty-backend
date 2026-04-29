use dioxus::prelude::*;
use futures_util::StreamExt;
use uuid::Uuid;

use crate::{
    base_types::ImStr,
    component::{
        atoms::{Btn, BtnVariant},
        SelectInput, TextInput, TextareaInput, TopBar,
    },
    i18n::Key,
    service::{
        i18n::I18N,
        text_template::{handle_text_template_action, TextTemplateAction, TEXT_TEMPLATE_STORE},
    },
    state::text_template::{TemplateEngine, TextTemplate},
};

#[component]
pub fn TextTemplateManagement() -> Element {
    let mut show_form = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<Uuid>);
    let mut form_name = use_signal(|| "".to_string());
    let mut form_template_type = use_signal(|| "billing-period".to_string());
    let mut form_template_text = use_signal(|| "".to_string());
    let mut form_template_engine = use_signal(|| TemplateEngine::Tera);

    let i18n = I18N.read().clone();
    let store = TEXT_TEMPLATE_STORE.read().clone();

    // Load templates when component mounts
    // Using use_hook ensures this runs every time the component is mounted
    use_hook(|| {
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

    let action_coroutine = use_coroutine(
        move |mut rx: UnboundedReceiver<TextTemplateAction>| async move {
            while let Some(action) = rx.next().await {
                handle_text_template_action(action).await;
            }
        },
    );

    let mut reset_form = move || {
        show_form.set(false);
        editing_id.set(None);
        form_name.set("".to_string());
        form_template_type.set("billing-period".to_string());
        form_template_text.set("".to_string());
        form_template_engine.set(TemplateEngine::Tera);
    };

    let mut save_template = move |_: ()| {
        let name = form_name.read().clone();
        let template_type = form_template_type.read().clone();
        let template_text = form_template_text.read().clone();
        let template_engine = form_template_engine.read().clone();

        if template_type.trim().is_empty() || template_text.trim().is_empty() {
            return;
        }

        let name_rc = if name.trim().is_empty() {
            None
        } else {
            Some(name.into())
        };

        if let Some(id) = *editing_id.read() {
            // Update existing template
            let template = TextTemplate {
                id,
                name: name_rc,
                template_type: template_type.into(),
                template_text: template_text.into(),
                template_engine,
                created_at: None,
                created_by: None,
            };
            action_coroutine.send(TextTemplateAction::UpdateTemplate(id, template));
        } else {
            // Create new template
            let template = TextTemplate {
                id: Uuid::nil(),
                name: name_rc,
                template_type: template_type.into(),
                template_text: template_text.into(),
                template_engine,
                created_at: None,
                created_by: None,
            };
            action_coroutine.send(TextTemplateAction::SaveTemplate(template));
        }
        reset_form();
    };

    let mut edit_template = move |template: TextTemplate| {
        form_name.set(
            template
                .name
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default(),
        );
        form_template_type.set(template.template_type.to_string());
        form_template_text.set(template.template_text.to_string());
        form_template_engine.set(template.template_engine.clone());
        editing_id.set(Some(template.id));
        show_form.set(true);
    };

    let delete_template = move |template_id: Uuid| {
        action_coroutine.send(TextTemplateAction::DeleteTemplate(template_id));
    };

    rsx! {
        TopBar {}

        div { class: "ml-1 mr-1 pt-4 md:m-8",
            h1 { class: "text-h1 mb-4 text-ink", "{title}" }

            if !*show_form.read() {
                div { class: "mb-4",
                    Btn {
                        variant: BtnVariant::Primary,
                        on_click: move |_| show_form.set(true),
                        "{add_new_str}"
                    }
                }
            }

            if *show_form.read() {
                div { class: "bg-surface-alt border border-border p-4 rounded-lg mb-6",
                    h2 { class: "text-h2 mb-4 text-ink",
                        if editing_id.read().is_some() {
                            "{i18n.t(Key::EditTemplate)}"
                        } else {
                            "{i18n.t(Key::AddNewTemplate)}"
                        }
                    }

                    div { class: "mb-4",
                        label { class: "block text-body font-medium text-ink-soft mb-2", "{i18n.t(Key::TemplateName)}" }
                        TextInput {
                            value: ImStr::from(form_name.read().as_str()),
                            placeholder: Some(ImStr::from("Enter template name (optional)...")),
                            on_change: move |value: ImStr| form_name.set(value.to_string()),
                        }
                    }

                    div { class: "mb-4",
                        label { class: "block text-body font-medium text-ink-soft mb-2", "{template_type_str}" }
                        SelectInput {
                            on_change: move |value: ImStr| form_template_type.set(value.to_string()),
                            option {
                                value: "billing-period",
                                selected: form_template_type.read().as_str() == "billing-period",
                                "Billing Period"
                            }
                            option {
                                value: "shiftplan-report",
                                selected: form_template_type.read().as_str() == "shiftplan-report",
                                "Shiftplan Report"
                            }
                        }
                    }

                    div { class: "mb-4",
                        label { class: "block text-body font-medium text-ink-soft mb-2", "{i18n.t(Key::TemplateEngine)}" }
                        SelectInput {
                            on_change: move |value: ImStr| {
                                let engine = match value.as_str() {
                                    "minijinja" => TemplateEngine::MiniJinja,
                                    _ => TemplateEngine::Tera,
                                };
                                form_template_engine.set(engine);
                            },
                            option {
                                value: "tera",
                                selected: matches!(*form_template_engine.read(), TemplateEngine::Tera),
                                "{i18n.t(Key::TemplateEngineTera)}"
                            }
                            option {
                                value: "minijinja",
                                selected: matches!(*form_template_engine.read(), TemplateEngine::MiniJinja),
                                "{i18n.t(Key::TemplateEngineMiniJinja)}"
                            }
                        }
                    }

                    div { class: "mb-4",
                        label { class: "block text-body font-medium text-ink-soft mb-2", "{template_text_str}" }
                        TextareaInput {
                            value: ImStr::from(form_template_text.read().as_str()),
                            placeholder: Some(ImStr::from("Enter your template text here...")),
                            rows: 6u8,
                            on_change: move |value: ImStr| form_template_text.set(value.to_string()),
                        }
                    }

                    div { class: "flex gap-2",
                        Btn {
                            variant: BtnVariant::Primary,
                            on_click: move |_| save_template(()),
                            "{save_str}"
                        }
                        Btn {
                            variant: BtnVariant::Secondary,
                            on_click: move |_| reset_form(),
                            "{cancel_str}"
                        }
                    }
                }
            }

            div { class: "overflow-x-auto",
                table { class: "min-w-full bg-surface border border-border",
                    thead { class: "bg-surface-alt",
                        tr {
                            th { class: "px-6 py-3 text-left text-micro font-bold text-ink-muted uppercase", "ID" }
                            th { class: "px-6 py-3 text-left text-micro font-bold text-ink-muted uppercase", "{i18n.t(Key::TemplateName)}" }
                            th { class: "px-6 py-3 text-left text-micro font-bold text-ink-muted uppercase", "{template_type_str}" }
                            th { class: "px-6 py-3 text-left text-micro font-bold text-ink-muted uppercase", "Preview" }
                            th { class: "px-6 py-3 text-left text-micro font-bold text-ink-muted uppercase", "{actions_str}" }
                        }
                    }
                    tbody { class: "divide-y divide-border",
                        for template in store.filtered_templates.iter() {
                            tr { key: "{template.id}",
                                td { class: "px-6 py-4 whitespace-nowrap text-body text-ink", "{template.id}" }
                                td { class: "px-6 py-4 whitespace-nowrap text-body text-ink",
                                    if let Some(ref name) = template.name {
                                        "{name}"
                                    } else {
                                        span { class: "text-ink-muted italic", "No name" }
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-body text-ink", "{template.template_type}" }
                                td { class: "px-6 py-4 text-body text-ink max-w-xs truncate",
                                    "{template.template_text}"
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-body font-medium",
                                    div { class: "flex gap-2",
                                        Btn {
                                            variant: BtnVariant::Primary,
                                            on_click: {
                                                let template = template.clone();
                                                move |_| edit_template(template.clone())
                                            },
                                            "{edit_str}"
                                        }
                                        Btn {
                                            variant: BtnVariant::Danger,
                                            on_click: {
                                                let template_id = template.id;
                                                move |_| delete_template(template_id)
                                            },
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

#[cfg(test)]
mod tests {
    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("text_template_management.rs");
        let test_module_start = src
            .find("#[cfg(test)]")
            .expect("test module marker missing");
        let prefix = &src[..test_module_start];
        for forbidden in [
            "bg-gray-",
            "bg-white",
            "text-gray-",
            "text-blue-",
            "text-red-",
            "text-green-",
            "bg-blue-",
            "bg-green-",
            "bg-red-",
            "border-black",
            "border-gray-",
        ] {
            assert!(
                !prefix.contains(forbidden),
                "legacy class `{forbidden}` found in source"
            );
        }
    }
}
