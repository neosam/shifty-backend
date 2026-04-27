use std::rc::Rc;

use dioxus::prelude::*;
use time::macros::format_description;

use crate::base_types::ImStr;
use crate::component::atoms::btn::{Btn, BtnVariant};
use crate::component::dialog::{Dialog, DialogVariant};
use crate::component::form::{Field, SelectInput};
use crate::i18n::Key;
use crate::service::{
    i18n::I18N,
    slot_edit::{SlotEditAction, SLOT_EDIT_STORE},
};
use crate::state::{
    slot_edit::{SlotEditItem, SlotEditType},
    Weekday,
};

const FORM_INPUT_CLASSES: &str =
    "h-[34px] px-[10px] border border-border-strong rounded-md bg-surface text-ink text-[13px] w-full min-w-0 form-input";

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

fn parse_time_input(value: &str) -> Option<time::Time> {
    let format_hm = format_description!("[hour]:[minute]");
    let format_hms = format_description!("[hour]:[minute]:[second]");
    time::Time::parse(value, format_hms)
        .or_else(|_| time::Time::parse(value, format_hm))
        .ok()
}

#[component]
pub fn SlotEditInner(props: SlotEditProps) -> Element {
    let i18n = I18N.read().clone();

    if !props.visible {
        return rsx! {};
    }

    let title: ImStr = if props.slot_edit_type == SlotEditType::New {
        i18n.t(Key::SlotNewTitle).as_ref().into()
    } else {
        i18n.t(Key::SlotEditTitle).as_ref().into()
    };

    let weekday_label: ImStr = i18n.t(Key::WeekdayLabel).as_ref().into();
    let from_label: ImStr = i18n.t(Key::FromLabel).as_ref().into();
    let to_label: ImStr = i18n.t(Key::ToLabel).as_ref().into();
    let min_persons_label: ImStr = i18n.t(Key::MinPersonsLabel).as_ref().into();
    let save_str = i18n.t(Key::SaveLabel).to_string();
    let cancel_str = i18n.t(Key::CancelLabel).to_string();
    let error_str = i18n.t(Key::SlotEditSaveError).to_string();

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
        .map(|valid_to| ImStr::from(i18n.format_date(valid_to)))
        .unwrap_or_else(|| "".into());
    let explanation_valid_to_str = i18n.t_m_rc(
        Key::SlotEditValidUntilExplanation,
        [("date", valid_to_date)].into(),
    );

    let display_format = format_description!("[hour]:[minute]");
    let from_value = props.slot.from.format(&display_format).unwrap_or_default();
    let to_value = props.slot.to.format(&display_format).unwrap_or_default();
    let min_resources_value = props.slot.min_resources as i32;
    let day_disabled = props.slot_edit_type == SlotEditType::Edit;
    let time_disabled = props.slot_edit_type == SlotEditType::Edit;

    let weekday_options = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ];

    let footer = rsx! {
        Btn { variant: BtnVariant::Secondary, on_click: props.on_cancel, "{cancel_str}" }
        Btn { variant: BtnVariant::Primary, on_click: props.on_save, "{save_str}" }
    };

    rsx! {
        Dialog {
            open: true,
            on_close: props.on_cancel,
            title,
            variant: DialogVariant::Auto,
            width: 460,
            footer: Some(footer),
            div { class: "flex flex-col gap-3",
                ul { class: "list-disc pl-5 text-[12px] text-ink-muted space-y-1",
                    li { "ℹ️ {explanation_str}" }
                    if props.slot.valid_to.is_some() {
                        li { class: "text-warn", "⚠️ {explanation_valid_to_str}" }
                    }
                }

                Field { label: weekday_label.clone(),
                    SelectInput {
                        disabled: day_disabled,
                        on_change: {
                            let slot = props.slot.clone();
                            move |value: ImStr| {
                                let mut updated = slot.as_ref().clone();
                                if let Ok(num) = value.as_str().parse::<u8>() {
                                    updated.day_of_week = Weekday::from_num_from_monday(num);
                                    props.on_update_slot.call(updated);
                                }
                            }
                        },
                        for day in weekday_options.iter() {
                            option {
                                value: day.num_from_monday().to_string(),
                                selected: *day == props.slot.day_of_week,
                                {day.i18n_string(&i18n).to_string()}
                            }
                        }
                    }
                }

                Field { label: from_label.clone(),
                    input {
                        class: FORM_INPUT_CLASSES,
                        r#type: "time",
                        value: "{from_value}",
                        disabled: time_disabled,
                        oninput: {
                            let slot = props.slot.clone();
                            move |event: Event<FormData>| {
                                if let Some(parsed) = parse_time_input(&event.value()) {
                                    let mut updated = slot.as_ref().clone();
                                    updated.from = parsed;
                                    props.on_update_slot.call(updated);
                                }
                            }
                        },
                    }
                }

                Field { label: to_label.clone(),
                    input {
                        class: FORM_INPUT_CLASSES,
                        r#type: "time",
                        value: "{to_value}",
                        disabled: time_disabled,
                        oninput: {
                            let slot = props.slot.clone();
                            move |event: Event<FormData>| {
                                if let Some(parsed) = parse_time_input(&event.value()) {
                                    let mut updated = slot.as_ref().clone();
                                    updated.to = parsed;
                                    props.on_update_slot.call(updated);
                                }
                            }
                        },
                    }
                }

                Field { label: min_persons_label.clone(),
                    input {
                        class: FORM_INPUT_CLASSES,
                        r#type: "number",
                        min: "0",
                        value: "{min_resources_value}",
                        oninput: {
                            let slot = props.slot.clone();
                            move |event: Event<FormData>| {
                                if let Ok(value) = event.value().parse::<i32>() {
                                    let mut updated = slot.as_ref().clone();
                                    updated.min_resources = value as u8;
                                    props.on_update_slot.call(updated);
                                }
                            }
                        },
                    }
                }

                if props.has_errors {
                    p { class: "text-bad text-[12px]", "{error_str}" }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_time_accepts_hh_mm() {
        let parsed = parse_time_input("09:30");
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap().hour(), 9);
    }

    #[test]
    fn parse_time_accepts_hh_mm_ss() {
        let parsed = parse_time_input("13:45:00");
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap().minute(), 45);
    }

    #[test]
    fn parse_time_rejects_garbage() {
        assert!(parse_time_input("not a time").is_none());
    }

    #[test]
    fn slot_edit_no_legacy_classes_in_source() {
        let source = include_str!("slot_edit.rs");
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
