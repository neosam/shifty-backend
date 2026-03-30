use std::rc::Rc;

use dioxus::prelude::*;
use rest_types::ShiftplanTO;
use uuid::Uuid;

use crate::api;
use crate::component::modal::Modal;
use crate::state::Config;

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
    let mut show_create_modal = use_signal(|| false);
    let mut create_name = use_signal(|| String::new());
    let mut delete_confirm_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut editing_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut edit_name = use_signal(|| String::new());

    rsx! {
        div { class: "flex border-b border-gray-300 mb-2 items-center",
            for shiftplan in props.shiftplans.iter() {
                {
                    let is_active = props.selected_id == Some(shiftplan.id);
                    let id = shiftplan.id;
                    let is_editing = *editing_id.read() == Some(id);
                    let shiftplan_clone = shiftplan.clone();
                    rsx! {
                        div { class: "flex items-center",
                            if is_editing {
                                input {
                                    class: "px-2 py-1 text-sm border border-blue-400 rounded outline-none",
                                    value: "{edit_name}",
                                    autofocus: true,
                                    oninput: move |e: Event<FormData>| {
                                        edit_name.set(e.value().to_string());
                                    },
                                    onkeydown: {
                                        let config = props.config.clone();
                                        let on_catalog_changed = props.on_catalog_changed;
                                        let shiftplan_clone = shiftplan_clone.clone();
                                        move |e: Event<KeyboardData>| {
                                            if e.key() == Key::Enter {
                                                let config = config.clone();
                                                let on_catalog_changed = on_catalog_changed;
                                                let mut updated = shiftplan_clone.clone();
                                                let new_name = edit_name.read().clone();
                                                editing_id.set(None);
                                                spawn(async move {
                                                    updated.name = new_name.into();
                                                    let _ = api::update_shiftplan(config, updated).await;
                                                    on_catalog_changed.call(Some(id));
                                                });
                                            } else if e.key() == Key::Escape {
                                                editing_id.set(None);
                                            }
                                        }
                                    },
                                    onblur: {
                                        let config = props.config.clone();
                                        let on_catalog_changed = props.on_catalog_changed;
                                        let shiftplan_clone = shiftplan_clone.clone();
                                        move |_| {
                                            let config = config.clone();
                                            let on_catalog_changed = on_catalog_changed;
                                            let mut updated = shiftplan_clone.clone();
                                            let new_name = edit_name.read().clone();
                                            editing_id.set(None);
                                            spawn(async move {
                                                updated.name = new_name.into();
                                                let _ = api::update_shiftplan(config, updated).await;
                                                on_catalog_changed.call(Some(id));
                                            });
                                        }
                                    },
                                }
                            } else {
                                button {
                                    class: if is_active {
                                        "px-4 py-2 text-sm font-medium border-b-2 border-blue-500 text-blue-600"
                                    } else {
                                        "px-4 py-2 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                    },
                                    onclick: move |_| props.on_select.call(id),
                                    ondoubleclick: {
                                        let planning_mode = props.planning_mode;
                                        let name = shiftplan.name.to_string();
                                        move |_| {
                                            if planning_mode {
                                                editing_id.set(Some(id));
                                                edit_name.set(name.clone());
                                            }
                                        }
                                    },
                                    "{shiftplan.name}"
                                }
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
                        create_name.set(String::new());
                        show_create_modal.set(true);
                    },
                    "+"
                }
            }
        }

        // Create modal
        if *show_create_modal.read() {
            Modal {
                div { class: "space-y-4",
                    h2 { class: "text-lg font-semibold", "Neuen Shiftplan erstellen" }
                    input {
                        class: "w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-400",
                        placeholder: "Name",
                        value: "{create_name}",
                        autofocus: true,
                        oninput: move |e: Event<FormData>| {
                            create_name.set(e.value().to_string());
                        },
                        onkeydown: {
                            let config = props.config.clone();
                            let on_catalog_changed = props.on_catalog_changed;
                            move |e: Event<KeyboardData>| {
                                if e.key() == Key::Enter {
                                    let config = config.clone();
                                    let on_catalog_changed = on_catalog_changed;
                                    let name = create_name.read().clone();
                                    show_create_modal.set(false);
                                    spawn(async move {
                                        if let Ok(created) = api::create_shiftplan(config, &name).await {
                                            on_catalog_changed.call(Some(created.id));
                                        }
                                    });
                                } else if e.key() == Key::Escape {
                                    show_create_modal.set(false);
                                }
                            }
                        },
                    }
                    div { class: "flex justify-end space-x-2",
                        button {
                            class: "px-4 py-2 text-sm text-gray-600 hover:text-gray-800",
                            onclick: move |_| {
                                show_create_modal.set(false);
                            },
                            "Abbrechen"
                        }
                        button {
                            class: "px-4 py-2 text-sm bg-blue-500 text-white rounded hover:bg-blue-600",
                            onclick: {
                                let config = props.config.clone();
                                let on_catalog_changed = props.on_catalog_changed;
                                move |_| {
                                    let config = config.clone();
                                    let on_catalog_changed = on_catalog_changed;
                                    let name = create_name.read().clone();
                                    show_create_modal.set(false);
                                    spawn(async move {
                                        if let Ok(created) = api::create_shiftplan(config, &name).await {
                                            on_catalog_changed.call(Some(created.id));
                                        }
                                    });
                                }
                            },
                            "Erstellen"
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
