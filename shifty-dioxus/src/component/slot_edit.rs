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
    "h-[34px] px-[10px] border border-border-strong rounded-md bg-surface text-ink text-body w-full min-w-0 form-input";

#[derive(Clone, PartialEq, Debug, Props)]
pub struct SlotEditProps {
    pub visible: bool,
    pub slot: Rc<SlotEditItem>,
    pub slot_edit_type: SlotEditType,
    pub year: u32,
    pub week: u8,
    pub has_errors: bool,
    /// Display-only live count of paid bookings for this slot's view-week.
    /// Drives the non-blocking overage banner (D-23-02); never written back.
    pub current_paid_count: u8,
    /// Whether "nur diese Woche" mode is active (SWO-01).  Default false = "ab dieser Woche".
    pub single_week: bool,

    pub on_save: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
    pub on_update_slot: EventHandler<SlotEditItem>,
    pub on_set_single_week: EventHandler<bool>,
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
    let max_paid_label: ImStr = i18n.t(Key::MaxPaidEmployeesLabel).as_ref().into();
    let max_paid_hint: ImStr = i18n.t(Key::MaxPaidEmployeesHint).as_ref().into();
    let max_paid_value = props
        .slot
        .max_paid_employees
        .map(|n| n.to_string())
        .unwrap_or_default();
    let show_overage = props
        .slot
        .max_paid_employees
        .is_some_and(|n| props.current_paid_count > n);
    let overage_str = i18n.t_m_rc(
        Key::MaxPaidEmployeesOverageHint,
        [
            ("current", props.current_paid_count.to_string().into()),
            (
                "limit",
                props
                    .slot
                    .max_paid_employees
                    .map(|n| n.to_string())
                    .unwrap_or_default()
                    .into(),
            ),
        ]
        .into(),
    );
    let save_str = i18n.t(Key::SaveLabel).to_string();
    let cancel_str = i18n.t(Key::CancelLabel).to_string();
    let error_str = i18n.t(Key::SlotEditSaveError).to_string();

    let scope_label = i18n.t(Key::SlotEditModeScopeLabel).to_string();
    let from_this_week_label = i18n.t(Key::SlotEditModeFromThisWeek).to_string();
    let this_week_only_label = i18n.t(Key::SlotEditModeThisWeekOnly).to_string();
    let single_week_hint = i18n.t_m_rc(
        Key::SlotEditModeThisWeekOnlyHint,
        [
            ("week", props.week.to_string().into()),
            ("year", props.year.to_string().into()),
        ]
        .into(),
    );

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
                ul { class: "list-disc pl-5 text-small font-normal text-ink-muted space-y-1",
                    li { "ℹ️ {explanation_str}" }
                    if props.slot.valid_to.is_some() {
                        li { class: "text-warn", "⚠️ {explanation_valid_to_str}" }
                    }
                }

                if props.slot_edit_type == SlotEditType::Edit {
                    div { class: "flex flex-col gap-2",
                        span { class: "text-small font-medium text-ink-muted", "{scope_label}" }
                        div { class: "flex gap-4",
                            label { class: "inline-flex items-center gap-2 cursor-pointer text-body text-ink",
                                input {
                                    r#type: "radio",
                                    name: "slot_edit_mode",
                                    class: "h-4 w-4 border border-border-strong accent-accent form-input",
                                    checked: !props.single_week,
                                    onchange: move |_| props.on_set_single_week.call(false),
                                }
                                "{from_this_week_label}"
                            }
                            label { class: "inline-flex items-center gap-2 cursor-pointer text-body text-ink",
                                input {
                                    r#type: "radio",
                                    name: "slot_edit_mode",
                                    class: "h-4 w-4 border border-border-strong accent-accent form-input",
                                    checked: props.single_week,
                                    onchange: move |_| props.on_set_single_week.call(true),
                                }
                                "{this_week_only_label}"
                            }
                        }
                        if props.single_week {
                            p { class: "text-small font-normal text-ink-muted mt-1",
                                "{single_week_hint}"
                            }
                        }
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

                Field { label: max_paid_label.clone(), hint: Some(max_paid_hint.clone()),
                    input {
                        class: FORM_INPUT_CLASSES,
                        r#type: "number",
                        min: "0",
                        value: "{max_paid_value}",
                        oninput: {
                            let slot = props.slot.clone();
                            move |event: Event<FormData>| {
                                let raw = event.value();
                                let mut updated = slot.as_ref().clone();
                                if raw.is_empty() {
                                    updated.max_paid_employees = None;
                                    props.on_update_slot.call(updated);
                                } else if let Ok(value) = raw.parse::<u8>() {
                                    updated.max_paid_employees = Some(value);
                                    props.on_update_slot.call(updated);
                                }
                                // parse failure (non-empty, non-u8): silently ignore.
                            }
                        },
                    }
                }

                if show_overage {
                    div { class: "border-l-[3px] border-warn bg-warn-soft rounded-md p-2.5 text-body text-ink",
                        "{overage_str}"
                    }
                }

                if props.has_errors {
                    p { class: "text-bad text-small font-normal", "{error_str}" }
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
            current_paid_count: slot_edit.current_paid_count,
            single_week: slot_edit.single_week,
            on_save: move |_| slot_service.send(SlotEditAction::SaveSlot),
            on_cancel: move |_| slot_service.send(SlotEditAction::Cancel),
            on_update_slot: move |slot| slot_service.send(SlotEditAction::UpdateSlot(slot)),
            on_set_single_week: move |val| slot_service.send(SlotEditAction::SetSingleWeek(val)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{generate, Locale};
    use crate::service::i18n::I18N;

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    fn pin_de_locale() {
        use_hook(|| {
            *I18N.write() = generate(Locale::De);
        });
    }

    fn slot_with_max(max: Option<u8>) -> Rc<SlotEditItem> {
        let mut item = SlotEditItem::empty();
        item.max_paid_employees = max;
        Rc::new(item)
    }

    fn props_with(max: Option<u8>, current_paid_count: u8) -> SlotEditProps {
        SlotEditProps {
            visible: true,
            slot: slot_with_max(max),
            slot_edit_type: SlotEditType::Edit,
            year: 2026,
            week: 26,
            has_errors: false,
            current_paid_count,
            single_week: false,
            on_save: EventHandler::new(|_| {}),
            on_cancel: EventHandler::new(|_| {}),
            on_update_slot: EventHandler::new(|_| {}),
            on_set_single_week: EventHandler::new(|_| {}),
        }
    }

    #[test]
    fn slot_edit_renders_max_paid_employees_field_with_value() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                SlotEditInner { ..props_with(Some(5), 0) }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Max. bezahlte Mitarbeiter"),
            "missing max-paid label: {html}"
        );
        assert!(
            html.contains("value=\"5\"") || html.contains(">5"),
            "max-paid input should carry value 5: {html}"
        );
    }

    #[test]
    fn slot_edit_renders_empty_max_paid_when_none() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                SlotEditInner { ..props_with(None, 0) }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Max. bezahlte Mitarbeiter"),
            "missing max-paid label: {html}"
        );
        // None must render an empty value attribute for the max-paid input.
        assert!(
            html.contains("value=\"\""),
            "max-paid input should be empty when None: {html}"
        );
    }

    #[test]
    fn slot_edit_shows_overage_hint_when_limit_below_count() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                SlotEditInner { ..props_with(Some(2), 3) }
            }
        }
        let html = render(app);
        assert!(
            html.contains("bg-warn-soft"),
            "overage banner div missing: {html}"
        );
        // Interpolated De text "Aktuell 3 bezahlt (Limit: 2)".
        assert!(
            html.contains("Aktuell 3 bezahlt") && html.contains("Limit: 2"),
            "overage banner text missing/incorrect: {html}"
        );
    }

    #[test]
    fn slot_edit_no_overage_hint_when_limit_ok() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                SlotEditInner { ..props_with(Some(5), 3) }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("bg-warn-soft"),
            "overage banner should be absent when limit >= count: {html}"
        );
    }

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

    // ---- Mode radio group SSR tests (SWO-01 / Plan 35-03) ----

    #[test]
    fn slot_edit_edit_mode_shows_radio_group_both_labels_no_hint() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                SlotEditInner { ..props_with(None, 0) }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Ab dieser Woche"),
            "Edit mode should show 'Ab dieser Woche' label: {html}"
        );
        assert!(
            html.contains("Nur diese Woche"),
            "Edit mode should show 'Nur diese Woche' label: {html}"
        );
        assert!(
            html.contains("slot_edit_mode"),
            "Edit mode should contain radio name 'slot_edit_mode': {html}"
        );
        // Hint paragraph must NOT be present when single_week=false (default).
        // "ausschließlich" is unique to the hint text; the existing explanation bullet does not contain it.
        assert!(
            !html.contains("ausschließlich"),
            "Hint paragraph must be absent when single_week=false: {html}"
        );
    }

    #[test]
    fn slot_edit_edit_mode_single_week_shows_hint() {
        fn app() -> Element {
            pin_de_locale();
            let p = SlotEditProps {
                single_week: true,
                ..props_with(None, 0)
            };
            rsx! {
                SlotEditInner { ..p }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Nur diese Woche"),
            "Edit+single_week mode should show 'Nur diese Woche': {html}"
        );
        // Interpolated hint must contain the week and year values.
        assert!(
            html.contains("26/2026") || (html.contains("26") && html.contains("2026")),
            "Hint should contain week 26 and year 2026: {html}"
        );
        assert!(
            html.contains("Folgewoche"),
            "Hint should contain 'Folgewoche': {html}"
        );
    }

    #[test]
    fn slot_edit_new_mode_hides_radio_group() {
        fn app() -> Element {
            pin_de_locale();
            let p = SlotEditProps {
                slot_edit_type: SlotEditType::New,
                visible: true,
                ..props_with(None, 0)
            };
            rsx! {
                SlotEditInner { ..p }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("slot_edit_mode"),
            "New mode must NOT render radio group (no radio name 'slot_edit_mode'): {html}"
        );
        assert!(
            !html.contains("Ab dieser Woche"),
            "New mode must NOT render 'Ab dieser Woche' label: {html}"
        );
        assert!(
            !html.contains("Geltungsbereich"),
            "New mode must NOT render scope label 'Geltungsbereich': {html}"
        );
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
