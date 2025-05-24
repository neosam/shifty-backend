use dioxus::prelude::*;
use futures_util::StreamExt;
use std::rc::Rc;
use tracing::info;
use uuid::Uuid;

use crate::{
    api,
    error::result_handler,
    i18n::Key,
    service::{config::CONFIG, i18n::I18N},
    state::employee::CustomExtraHoursDefinition,
};

use rest_types::CustomExtraHoursTO;

pub enum CustomExtraHoursManagementAction {
    LoadCustomExtraHours,
    CreateCustomExtraHours {
        name: String,
        description: Option<String>,
        modifies_balance: bool,
        assigned_sales_person_ids: Vec<Uuid>,
    },
    UpdateCustomExtraHours {
        id: Uuid,
        name: String,
        description: Option<String>,
        modifies_balance: bool,
        assigned_sales_person_ids: Vec<Uuid>,
    },
    DeleteCustomExtraHours(Uuid),
}

#[component]
pub fn CustomExtraHoursManagement() -> Element {
    let custom_extra_hours = use_signal(|| Rc::<[CustomExtraHoursDefinition]>::from([]));
    let mut show_form = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<Uuid>);
    let mut form_name = use_signal(|| "".to_string());
    let mut form_description = use_signal(|| "".to_string());
    let mut form_modifies_balance = use_signal(|| true);

    let config = CONFIG.read().clone();
    let i18n = I18N.read().clone();

    let title = i18n.t(Key::CustomExtraHoursManagement);
    let name_str = i18n.t(Key::Name);
    let description_str = i18n.t(Key::Description);
    let modifies_balance_str = i18n.t(Key::ModifiesBalance);
    let actions_str = i18n.t(Key::Actions);
    let add_new_str = i18n.t(Key::AddNew);
    let save_str = i18n.t(Key::Save);
    let cancel_str = i18n.t(Key::Cancel);
    let edit_str = i18n.t(Key::Edit);
    let delete_str = i18n.t(Key::Delete);

    let action_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<CustomExtraHoursManagementAction>| {
        to_owned![config, custom_extra_hours];
        async move {
            while let Some(action) = rx.next().await {
                match action {
                    CustomExtraHoursManagementAction::LoadCustomExtraHours => {
                        // For now, we'll load for the current user's sales person
                        // In a real implementation, you might want to load all custom extra hours
                        // or filter by the current user's permissions
                        if let Ok(current_sales_person) = api::get_current_sales_person(config.clone()).await {
                            if let Some(sales_person) = current_sales_person {
                                match api::get_custom_extra_hours_by_sales_person(config.clone(), sales_person.id).await {
                                    Ok(hours) => {
                                        let definitions: Rc<[CustomExtraHoursDefinition]> = hours
                                            .iter()
                                            .map(|h| h.into())
                                            .collect();
                                        *custom_extra_hours.write() = definitions;
                                    }
                                    Err(e) => {
                                        info!("Failed to load custom extra hours: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    CustomExtraHoursManagementAction::CreateCustomExtraHours {
                        name,
                        description,
                        modifies_balance,
                        assigned_sales_person_ids,
                    } => {
                        let custom_extra_hours_to = CustomExtraHoursTO {
                            id: Uuid::nil(),
                            name: name.into(),
                            description: description.map(|d| d.into()),
                            modifies_balance,
                            assigned_sales_person_ids: assigned_sales_person_ids.into(),
                            created: None,
                            deleted: None,
                            version: Uuid::nil(),
                        };

                        let result = api::post_custom_extra_hours(config.clone(), custom_extra_hours_to).await;
                        result_handler(result.map_err(|e| crate::error::ShiftyError::from(e)));
                        // Note: We don't reload here to avoid infinite loops
                    }
                    CustomExtraHoursManagementAction::UpdateCustomExtraHours {
                        id,
                        name,
                        description,
                        modifies_balance,
                        assigned_sales_person_ids,
                    } => {
                        let custom_extra_hours_to = CustomExtraHoursTO {
                            id,
                            name: name.into(),
                            description: description.map(|d| d.into()),
                            modifies_balance,
                            assigned_sales_person_ids: assigned_sales_person_ids.into(),
                            created: None,
                            deleted: None,
                            version: Uuid::nil(),
                        };

                        let result = api::put_custom_extra_hours(config.clone(), custom_extra_hours_to).await;
                        result_handler(result.map_err(|e| crate::error::ShiftyError::from(e)));
                        // Note: We don't reload here to avoid infinite loops
                    }
                    CustomExtraHoursManagementAction::DeleteCustomExtraHours(id) => {
                        let result = api::delete_custom_extra_hours(config.clone(), id).await;
                        result_handler(result.map_err(|e| crate::error::ShiftyError::from(e)));
                        // Note: We don't reload here to avoid infinite loops
                    }
                }
            }
        }
    });

    // Load custom extra hours when component mounts
    use_effect(move || {
        action_coroutine.send(CustomExtraHoursManagementAction::LoadCustomExtraHours);
    });

    let mut start_edit = move |id: Uuid, name: String, description: Option<String>, modifies_balance: bool| {
        *editing_id.write() = Some(id);
        *form_name.write() = name;
        *form_description.write() = description.unwrap_or_default();
        *form_modifies_balance.write() = modifies_balance;
        *show_form.write() = true;
    };

    let mut start_create = move || {
        *editing_id.write() = None;
        *form_name.write() = "".to_string();
        *form_description.write() = "".to_string();
        *form_modifies_balance.write() = true;
        *show_form.write() = true;
    };

    let mut cancel_form = move || {
        *show_form.write() = false;
        *editing_id.write() = None;
    };

    let mut save_form = move || {
        let name = form_name.read().clone();
        let description = if form_description.read().is_empty() {
            None
        } else {
            Some(form_description.read().clone())
        };
        let modifies_balance = *form_modifies_balance.read();

        if let Some(id) = *editing_id.read() {
            action_coroutine.send(CustomExtraHoursManagementAction::UpdateCustomExtraHours {
                id,
                name,
                description,
                modifies_balance,
                assigned_sales_person_ids: vec![], // For now, empty - would need sales person selection UI
            });
        } else {
            action_coroutine.send(CustomExtraHoursManagementAction::CreateCustomExtraHours {
                name,
                description,
                modifies_balance,
                assigned_sales_person_ids: vec![], // For now, empty - would need sales person selection UI
            });
        }

        *show_form.write() = false;
        *editing_id.write() = None;
        
        // Reload data after operation
        action_coroutine.send(CustomExtraHoursManagementAction::LoadCustomExtraHours);
    };

    rsx! {
        div { class: "container mx-auto p-4",
            h1 { class: "text-3xl font-bold mb-6", "{title}" }

            button {
                class: "mb-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600",
                onclick: move |_| start_create(),
                "{add_new_str}"
            }

            if *show_form.read() {
                div { class: "mb-6 p-4 border border-gray-300 rounded bg-gray-50",
                    h2 { class: "text-xl font-semibold mb-4",
                        if editing_id.read().is_some() {
                            "Edit Custom Extra Hours"
                        } else {
                            "Create New Custom Extra Hours"
                        }
                    }

                    div { class: "mb-4",
                        label { class: "block text-sm font-medium text-gray-700 mb-2", "{name_str}" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_name.read()}",
                            onchange: move |event| {
                                *form_name.write() = event.data.value();
                            },
                            placeholder: "Enter name...",
                        }
                    }

                    div { class: "mb-4",
                        label { class: "block text-sm font-medium text-gray-700 mb-2", "{description_str}" }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{form_description.read()}",
                            onchange: move |event| {
                                *form_description.write() = event.data.value();
                            },
                            placeholder: "Enter description...",
                            rows: "3",
                        }
                    }

                    div { class: "mb-4",
                        label { class: "flex items-center",
                            input {
                                "type": "checkbox",
                                class: "mr-2",
                                checked: *form_modifies_balance.read(),
                                onchange: move |event| {
                                    *form_modifies_balance.write() = event.data.value() == "true";
                                },
                            }
                            span { class: "text-sm font-medium text-gray-700", "{modifies_balance_str}" }
                        }
                    }

                    div { class: "flex space-x-2",
                        button {
                            class: "px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600",
                            onclick: move |_| save_form(),
                            "{save_str}"
                        }
                        button {
                            class: "px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600",
                            onclick: move |_| cancel_form(),
                            "{cancel_str}"
                        }
                    }
                }
            }

            div { class: "overflow-x-auto",
                table { class: "min-w-full bg-white border border-gray-300",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-b", "{name_str}" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-b", "{description_str}" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-b", "{modifies_balance_str}" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-b", "{actions_str}" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        for custom_hour in custom_extra_hours.read().iter() {
                            tr { class: "hover:bg-gray-50",
                                td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 border-b", "{custom_hour.name}" }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500 border-b",
                                    if let Some(ref desc) = custom_hour.description {
                                        "{desc}"
                                    } else {
                                        "-"
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500 border-b",
                                    if custom_hour.modifies_balance { "Yes" } else { "No" }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium border-b",
                                    button {
                                        class: "text-blue-600 hover:text-blue-900 mr-2",
                                        onclick: {
                                            let id = custom_hour.id;
                                            let name = custom_hour.name.to_string();
                                            let description = custom_hour.description.clone().map(|d| d.to_string());
                                            let modifies_balance = custom_hour.modifies_balance;
                                            move |_| start_edit(id, name.clone(), description.clone(), modifies_balance)
                                        },
                                        "{edit_str}"
                                    }
                                    button {
                                        class: "text-red-600 hover:text-red-900",
                                        onclick: {
                                            let id = custom_hour.id;
                                            move |_| {
                                                action_coroutine.send(CustomExtraHoursManagementAction::DeleteCustomExtraHours(id));
                                                // Reload data after delete
                                                action_coroutine.send(CustomExtraHoursManagementAction::LoadCustomExtraHours);
                                            }
                                        },
                                        "{delete_str}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if custom_extra_hours.read().is_empty() {
                div { class: "text-center py-8 text-gray-500",
                    "No custom extra hours defined yet. Click 'Add New' to create one."
                }
            }
        }
    }
} 