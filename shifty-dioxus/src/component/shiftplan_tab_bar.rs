use std::rc::Rc;

use dioxus::prelude::*;
use rest_types::ShiftplanTO;
use uuid::Uuid;

use crate::api;
use crate::component::modal::Modal;
use crate::state::Config;

#[derive(Clone, PartialEq)]
enum ShiftplanDialogMode {
    Hidden,
    Create,
    Edit(ShiftplanTO),
}

#[derive(Clone, PartialEq, Props)]
pub struct ShiftplanTabBarProps {
    pub shiftplans: Rc<[ShiftplanTO]>,
    pub selected_id: Option<Uuid>,
    pub on_select: EventHandler<Uuid>,
    #[props(default = false)]
    pub planning_mode: bool,
    pub config: Config,
    pub on_catalog_changed: EventHandler<Option<Uuid>>,
}

#[component]
pub fn ShiftplanTabBar(props: ShiftplanTabBarProps) -> Element {
    let mut dialog_mode: Signal<ShiftplanDialogMode> = use_signal(|| ShiftplanDialogMode::Hidden);
    let mut dialog_name = use_signal(|| String::new());
    let mut dialog_is_planning = use_signal(|| false);
    let mut delete_confirm_id: Signal<Option<Uuid>> = use_signal(|| None);

    rsx! {
        div { class: "flex border-b border-gray-300 mb-2 items-center",
            for shiftplan in props.shiftplans.iter() {
                {
                    let is_active = props.selected_id == Some(shiftplan.id);
                    let id = shiftplan.id;
                    let shiftplan_clone = shiftplan.clone();
                    rsx! {
                        div { class: "flex items-center",
                            button {
                                class: if is_active {
                                    "px-4 py-2 text-sm font-medium border-b-2 border-blue-500 text-blue-600"
                                } else {
                                    "px-4 py-2 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                },
                                onclick: move |_| props.on_select.call(id),
                                ondoubleclick: {
                                    let planning_mode = props.planning_mode;
                                    let shiftplan_clone = shiftplan_clone.clone();
                                    move |_| {
                                        if planning_mode {
                                            dialog_name.set(shiftplan_clone.name.to_string());
                                            dialog_is_planning.set(shiftplan_clone.is_planning);
                                            dialog_mode.set(ShiftplanDialogMode::Edit(shiftplan_clone.clone()));
                                        }
                                    }
                                },
                                "{shiftplan.name}"
                            }
                            if props.planning_mode {
                                button {
                                    class: "ml-1 text-red-400 hover:text-red-600 text-xs",
                                    onclick: move |_| {
                                        delete_confirm_id.set(Some(id));
                                    },
                                    "✕"
                                }
                            }
                        }
                    }
                }
            }
            if props.planning_mode {
                button {
                    class: "px-3 py-2 text-sm font-medium text-green-600 hover:text-green-800 hover:bg-green-50 rounded",
                    onclick: move |_| {
                        dialog_name.set(String::new());
                        dialog_is_planning.set(false);
                        dialog_mode.set(ShiftplanDialogMode::Create);
                    },
                    "+"
                }
            }
        }

        // Create/Edit modal
        if *dialog_mode.read() != ShiftplanDialogMode::Hidden {
            {
                let is_create = *dialog_mode.read() == ShiftplanDialogMode::Create;
                let title = if is_create { "Neuen Shiftplan erstellen" } else { "Shiftplan bearbeiten" };
                let confirm_label = if is_create { "Erstellen" } else { "Speichern" };
                rsx! {
                    Modal {
                        div { class: "space-y-4",
                            h2 { class: "text-lg font-semibold", "{title}" }
                            input {
                                class: "w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-400",
                                placeholder: "Name",
                                value: "{dialog_name}",
                                autofocus: true,
                                oninput: move |e: Event<FormData>| {
                                    dialog_name.set(e.value().to_string());
                                },
                                onkeydown: move |e: Event<KeyboardData>| {
                                    if e.key() == Key::Escape {
                                        dialog_mode.set(ShiftplanDialogMode::Hidden);
                                    }
                                },
                            }
                            label { class: "flex items-center space-x-2",
                                input {
                                    r#type: "checkbox",
                                    checked: *dialog_is_planning.read(),
                                    onchange: move |e: Event<FormData>| {
                                        dialog_is_planning.set(e.checked());
                                    },
                                }
                                span { "Nur Planung" }
                            }
                            div { class: "flex justify-end space-x-2",
                                button {
                                    class: "px-4 py-2 text-sm text-gray-600 hover:text-gray-800",
                                    onclick: move |_| {
                                        dialog_mode.set(ShiftplanDialogMode::Hidden);
                                    },
                                    "Abbrechen"
                                }
                                button {
                                    class: "px-4 py-2 text-sm bg-blue-500 text-white rounded hover:bg-blue-600",
                                    onclick: {
                                        let config = props.config.clone();
                                        let on_catalog_changed = props.on_catalog_changed;
                                        let current_mode = dialog_mode.read().clone();
                                        move |_| {
                                            let config = config.clone();
                                            let on_catalog_changed = on_catalog_changed;
                                            let name = dialog_name.read().clone();
                                            let is_planning = *dialog_is_planning.read();
                                            let current_mode = current_mode.clone();
                                            dialog_mode.set(ShiftplanDialogMode::Hidden);
                                            spawn(async move {
                                                match current_mode {
                                                    ShiftplanDialogMode::Create => {
                                                        if let Ok(created) = api::create_shiftplan(config, &name, is_planning).await {
                                                            on_catalog_changed.call(Some(created.id));
                                                        }
                                                    }
                                                    ShiftplanDialogMode::Edit(mut original) => {
                                                        original.name = name.into();
                                                        original.is_planning = is_planning;
                                                        let id = original.id;
                                                        let _ = api::update_shiftplan(config, original).await;
                                                        on_catalog_changed.call(Some(id));
                                                    }
                                                    ShiftplanDialogMode::Hidden => {}
                                                }
                                            });
                                        }
                                    },
                                    "{confirm_label}"
                                }
                            }
                        }
                    }
                }
            }
        }

        // Delete confirmation modal
        if let Some(delete_id) = *delete_confirm_id.read() {
            Modal {
                div { class: "space-y-4",
                    h2 { class: "text-lg font-semibold", "Shiftplan löschen?" }
                    p { "Soll dieser Shiftplan wirklich gelöscht werden?" }
                    div { class: "flex justify-end space-x-2",
                        button {
                            class: "px-4 py-2 text-sm text-gray-600 hover:text-gray-800",
                            onclick: move |_| {
                                delete_confirm_id.set(None);
                            },
                            "Abbrechen"
                        }
                        button {
                            class: "px-4 py-2 text-sm bg-red-500 text-white rounded hover:bg-red-600",
                            onclick: {
                                let config = props.config.clone();
                                let on_catalog_changed = props.on_catalog_changed;
                                move |_| {
                                    let config = config.clone();
                                    let on_catalog_changed = on_catalog_changed;
                                    delete_confirm_id.set(None);
                                    spawn(async move {
                                        let _ = api::delete_shiftplan(config, delete_id).await;
                                        on_catalog_changed.call(None);
                                    });
                                }
                            },
                            "Löschen"
                        }
                    }
                }
            }
        }
    }
}
