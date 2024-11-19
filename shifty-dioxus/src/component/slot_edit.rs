use std::rc::Rc;

use dioxus::prelude::*;
use tracing::info;

use crate::{
    component::{
        base_components::{Button, Form, FormPair, IntegerInput},
        Modal,
    },
    service::{SlotEditAction, SLOT_EDIT_STORE},
    state::slot_edit::SlotEditItem,
};

#[derive(Clone, PartialEq, Debug, Props)]
pub struct SlotEditProps {
    pub visible: bool,
    pub slot: Rc<SlotEditItem>,

    pub on_save: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
    pub on_update_slot: EventHandler<SlotEditItem>,
}

#[component]
pub fn SlotEditInner(props: SlotEditProps) -> Element {
    rsx! {
        if props.visible {
            Modal {
                div { class: "flex flex-col items-center justify-center w-full",
                    h1 { class: "text-2xl font-bold", "Slot Edit" }
                    Form {
                        FormPair { label: "From".into(), "{props.slot.from}" }
                        FormPair { label: "To".into(), "{props.slot.to}" }
                        FormPair { label: "Min persons".into(),
                            IntegerInput {
                                value: props.slot.min_resources as i32,
                                on_change: move |value| {
                                    let mut slot = props.slot.as_ref().clone();
                                    slot.min_resources = value as u8;
                                    props.on_update_slot.call(slot);
                                }
                            }
                        }

                        Button { on_click: props.on_save, "Save" }
                        Button { on_click: props.on_cancel, "Cancel" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SlotEdit() -> Element {
    let slot_edit = SLOT_EDIT_STORE.read().to_owned();
    let slot_service = use_coroutine_handle::<SlotEditAction>();
    rsx! {
        SlotEditInner {
            visible: slot_edit.visible,
            slot: slot_edit.slot.clone(),
            on_save: move |_| slot_service.send(SlotEditAction::SaveSlot),
            on_cancel: move |_| slot_service.send(SlotEditAction::Cancel),
            on_update_slot: move |slot| slot_service.send(SlotEditAction::UpdateSlot(slot))
        }
    }
}
