use std::rc::Rc;

use dioxus::prelude::*;
use rest_types::ShiftplanTO;
use uuid::Uuid;

use crate::api;
use crate::base_types::ImStr;
use crate::component::atoms::btn::{Btn, BtnVariant};
use crate::component::dialog::{Dialog, DialogVariant};
use crate::component::form::{Field, FormCheckbox, TextInput};
use crate::i18n::Key;
use crate::service::i18n::I18N;
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

/// Returns the active-tab class string.
pub(crate) fn active_tab_class() -> &'static str {
    "px-4 py-2 text-[13px] font-semibold border-b-2 border-accent text-accent -mb-px"
}

/// Returns the inactive-tab class string.
pub(crate) fn inactive_tab_class() -> &'static str {
    "px-4 py-2 text-[13px] font-medium border-b-2 border-transparent text-ink-muted hover:text-ink hover:border-border-strong -mb-px"
}

#[component]
pub fn ShiftplanTabBar(props: ShiftplanTabBarProps) -> Element {
    let i18n = I18N.read().clone();
    let mut dialog_mode: Signal<ShiftplanDialogMode> = use_signal(|| ShiftplanDialogMode::Hidden);
    let mut dialog_name = use_signal(|| String::new());
    let mut dialog_is_planning = use_signal(|| false);
    let mut delete_confirm_id: Signal<Option<Uuid>> = use_signal(|| None);

    let create_title: ImStr = i18n.t(Key::ShiftplanCreateTitle).as_ref().into();
    let edit_title: ImStr = i18n.t(Key::ShiftplanEditTitle).as_ref().into();
    let delete_title: ImStr = i18n.t(Key::ShiftplanDeleteConfirmTitle).as_ref().into();
    let delete_body: ImStr = i18n.t(Key::ShiftplanDeleteConfirmBody).as_ref().into();
    let name_label: ImStr = i18n.t(Key::Name).as_ref().into();
    let planning_label: ImStr = i18n.t(Key::ShiftplanIsPlanningLabel).as_ref().into();

    rsx! {
        div { class: "flex border-b border-border items-center",
            for shiftplan in props.shiftplans.iter() {
                {
                    let is_active = props.selected_id == Some(shiftplan.id);
                    let id = shiftplan.id;
                    let shiftplan_clone = shiftplan.clone();
                    let tab_class = if is_active { active_tab_class() } else { inactive_tab_class() };
                    rsx! {
                        div { class: "flex items-center",
                            button {
                                class: tab_class,
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
                                    class: "ml-1 text-bad-soft hover:text-bad text-xs px-1",
                                    "aria-label": "Delete shiftplan",
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
                    class: "ml-2 px-3 py-2 text-[13px] font-medium text-accent hover:bg-accent-soft rounded-md",
                    "aria-label": "Create shiftplan",
                    onclick: move |_| {
                        dialog_name.set(String::new());
                        dialog_is_planning.set(false);
                        dialog_mode.set(ShiftplanDialogMode::Create);
                    },
                    "+"
                }
            }
        }

        // Create / edit dialog
        if *dialog_mode.read() != ShiftplanDialogMode::Hidden {
            {
                let mode = dialog_mode.read().clone();
                let is_create = matches!(mode, ShiftplanDialogMode::Create);
                let title = if is_create { create_title.clone() } else { edit_title.clone() };
                let confirm_label_str = if is_create {
                    i18n.t(Key::Create).to_string()
                } else {
                    i18n.t(Key::Save).to_string()
                };
                let cancel_label_str = i18n.t(Key::Cancel).to_string();
                let config = props.config.clone();
                let on_catalog_changed = props.on_catalog_changed;
                let mode_for_submit = mode.clone();
                let footer = rsx! {
                    Btn {
                        variant: BtnVariant::Secondary,
                        on_click: move |_| {
                            dialog_mode.set(ShiftplanDialogMode::Hidden);
                        },
                        "{cancel_label_str}"
                    }
                    Btn {
                        variant: BtnVariant::Primary,
                        on_click: move |_| {
                            let config = config.clone();
                            let on_catalog_changed = on_catalog_changed;
                            let name = dialog_name.read().clone();
                            let is_planning = *dialog_is_planning.read();
                            let mode = mode_for_submit.clone();
                            dialog_mode.set(ShiftplanDialogMode::Hidden);
                            spawn(async move {
                                match mode {
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
                        },
                        "{confirm_label_str}"
                    }
                };

                rsx! {
                    Dialog {
                        open: true,
                        on_close: move |_| {
                            dialog_mode.set(ShiftplanDialogMode::Hidden);
                        },
                        title,
                        variant: DialogVariant::Auto,
                        width: 460,
                        footer: Some(footer),
                        div { class: "flex flex-col gap-3",
                            Field { label: name_label.clone(),
                                TextInput {
                                    value: ImStr::from(dialog_name.read().as_str()),
                                    on_change: move |value: ImStr| {
                                        dialog_name.set(value.as_str().to_string());
                                    },
                                }
                            }
                            Field { label: planning_label.clone(),
                                FormCheckbox {
                                    value: *dialog_is_planning.read(),
                                    on_change: move |checked: bool| {
                                        dialog_is_planning.set(checked);
                                    },
                                    label: rsx! { "{planning_label}" },
                                }
                            }
                        }
                    }
                }
            }
        }

        // Delete confirm dialog
        if let Some(delete_id) = *delete_confirm_id.read() {
            {
                let cancel_label_str = i18n.t(Key::Cancel).to_string();
                let delete_label_str = i18n.t(Key::Delete).to_string();
                let config = props.config.clone();
                let on_catalog_changed = props.on_catalog_changed;
                let footer = rsx! {
                    Btn {
                        variant: BtnVariant::Secondary,
                        on_click: move |_| {
                            delete_confirm_id.set(None);
                        },
                        "{cancel_label_str}"
                    }
                    Btn {
                        variant: BtnVariant::Danger,
                        on_click: move |_| {
                            let config = config.clone();
                            let on_catalog_changed = on_catalog_changed;
                            delete_confirm_id.set(None);
                            spawn(async move {
                                let _ = api::delete_shiftplan(config, delete_id).await;
                                on_catalog_changed.call(None);
                            });
                        },
                        "{delete_label_str}"
                    }
                };
                rsx! {
                    Dialog {
                        open: true,
                        on_close: move |_| {
                            delete_confirm_id.set(None);
                        },
                        title: delete_title.clone(),
                        variant: DialogVariant::Auto,
                        width: 420,
                        footer: Some(footer),
                        p { class: "text-ink", "{delete_body}" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_tab_class_carries_accent_tokens() {
        let c = active_tab_class();
        assert!(c.contains("border-accent"), "missing border-accent: {c}");
        assert!(c.contains("text-accent"), "missing text-accent: {c}");
        assert!(c.contains("border-b-2"), "missing border-b-2: {c}");
    }

    #[test]
    fn inactive_tab_class_carries_muted_tokens() {
        let c = inactive_tab_class();
        assert!(
            c.contains("border-transparent"),
            "missing border-transparent: {c}"
        );
        assert!(c.contains("text-ink-muted"), "missing text-ink-muted: {c}");
        assert!(c.contains("border-b-2"), "missing border-b-2: {c}");
    }

    #[test]
    fn tab_classes_have_no_legacy_color_tokens() {
        for c in [active_tab_class(), inactive_tab_class()] {
            for forbidden in ["border-blue-", "text-blue-", "border-gray-", "text-gray-"] {
                assert!(
                    !c.contains(forbidden),
                    "legacy class `{}` in tab class `{}`",
                    forbidden,
                    c
                );
            }
        }
    }

    #[test]
    fn shiftplan_tab_bar_no_legacy_classes_in_source() {
        let source = include_str!("shiftplan_tab_bar.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
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
            "border-blue-",
            "border-gray-",
            "border-black",
        ] {
            assert!(
                !production.contains(forbidden),
                "non-test source contains legacy class `{}`",
                forbidden
            );
        }
    }
}
