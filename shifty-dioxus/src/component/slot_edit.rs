use std::rc::Rc;

use crate::i18n::Key;
use dioxus::prelude::*;

use crate::{
    base_types::ImStr,
    component::{
        base_components::{Button, Form, FormGroup, FormPair, IntegerInput, Select, TimeInput},
        Modal,
    },
    service::{SlotEditAction, I18N, SLOT_EDIT_STORE},
    state::{
        slot_edit::{SlotEditItem, SlotEditType},
        Weekday,
    },
};

#[derive(Clone, PartialEq, Debug, Props)]
pub struct SlotEditProps {
    pub visible: bool,
    pub slot: Rc<SlotEditItem>,
    pub slot_edit_type: SlotEditType,
    pub year: u32,
    pub week: u8,
    pub has_errors: bool,

    pub on_save: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
    pub on_update_slot: EventHandler<SlotEditItem>,
}

#[component]
pub fn SlotEditInner(props: SlotEditProps) -> Element {
    let i18n = I18N.read().clone();

    let title = if props.slot_edit_type == SlotEditType::New {
        i18n.t(Key::SlotNewTitle)
    } else {
        i18n.t(Key::SlotEditTitle)
    };

    let weekday_label = i18n.t(Key::WeekdayLabel);
    let from_label = i18n.t(Key::FromLabel);
    let to_label = i18n.t(Key::ToLabel);
    let min_persons_label = i18n.t(Key::MinPersonsLabel);
    let save_str = i18n.t(Key::SaveLabel);
    let cancel_str = i18n.t(Key::CancelLabel);
    let error_str = i18n.t(Key::SlotEditSaveError);

    let explanation_str = i18n.t_m_rc(
        Key::SlotEditExplanation,
        [
            ("year", props.year.to_string().into()),
            ("week", props.week.to_string().into()),
        ]
        .into(),
    );
    let valid_to_date = props
        .slot
        .valid_to
        .as_ref()
        .map(|valid_to| ImStr::from(i18n.format_date(&valid_to)))
        .unwrap_or("".into());
    let explanation_valid_to_str = i18n.t_m_rc(
        Key::SlotEditValidUntilExplanation,
        [("date", valid_to_date)].into(),
    );

    rsx! {
        if props.visible {
            Modal {
                Form {
                    h1 { class: "text-2xl font-bold", "{title}" }
                    FormGroup {
                        ul {
                            li { "ℹ️ {explanation_str}" }
                            if props.slot.valid_to.is_some() {
                                li { "⚠️ {explanation_valid_to_str}" }
                            }
                        }
                    }
                    FormPair { label: weekday_label.into(),
                        Select {
                            on_change: {
                                let slot = props.slot.to_owned();
                                move |value: ImStr| {
                                    let mut slot = slot.as_ref().clone();
                                    slot.day_of_week = Weekday::from_num_from_monday(
                                        value.as_str().parse::<u8>().unwrap(),
                                    );
                                    props.on_update_slot.call(slot);
                                }
                            },
                            disabled: props.slot_edit_type == SlotEditType::Edit,
                            children: {
                                let slot = props.slot.to_owned();
                                rsx! {
                                    for day in vec![
                                        Weekday::Monday,
                                        Weekday::Tuesday,
                                        Weekday::Wednesday,
                                        Weekday::Thursday,
                                        Weekday::Friday,
                                        Weekday::Saturday,
                                        Weekday::Sunday,
                                    ]
                                    {
                                        option { selected: day == slot.day_of_week, value: "{day.num_from_monday()}",
                                            {day.i18n_string(&i18n)}
                                        }
                                    }
                                }
                            },
                        }
                    }
                    FormPair { label: from_label.into(),
                        TimeInput {
                            value: props.slot.from,
                            on_change: {
                                let slot = props.slot.to_owned();
                                move |value| {
                                    let mut slot = slot.as_ref().clone();
                                    slot.from = value;
                                    props.on_update_slot.call(slot);
                                }
                            },
                            disabled: props.slot_edit_type == SlotEditType::Edit,
                        }
                    }
                    FormPair { label: to_label.into(),
                        TimeInput {
                            value: props.slot.to,
                            on_change: {
                                let slot = props.slot.to_owned();
                                move |value| {
                                    let mut slot = slot.as_ref().clone();
                                    slot.to = value;
                                    props.on_update_slot.call(slot);
                                }
                            },
                            disabled: props.slot_edit_type == SlotEditType::Edit,
                        }
                    }
                    FormPair { label: min_persons_label.into(),
                        IntegerInput {
                            value: props.slot.min_resources as i32,
                            on_change: {
                                let slot = props.slot.to_owned();
                                move |value| {
                                    let mut slot = slot.as_ref().clone();
                                    slot.min_resources = value as u8;
                                    props.on_update_slot.call(slot);
                                }
                            },
                        }
                    }
                    if props.has_errors {
                        FormGroup {
                            p { class: "text-red-500", "{error_str}" }
                        }
                    }

                    Button { on_click: props.on_save, "{save_str}" }
                    Button { on_click: props.on_cancel, "{cancel_str}" }
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
            slot_edit_type: slot_edit.slot_edit_type,
            year: slot_edit.year,
            week: slot_edit.week,
            has_errors: slot_edit.has_errors,
            on_save: move |_| slot_service.send(SlotEditAction::SaveSlot),
            on_cancel: move |_| slot_service.send(SlotEditAction::Cancel),
            on_update_slot: move |slot| slot_service.send(SlotEditAction::UpdateSlot(slot)),
        }
    }
}
