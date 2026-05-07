//! `ExtraHoursModal` — token-based dialog wrapping the extra-hours form.
//!
//! Replaces the legacy `Modal { AddExtraHoursForm }` mounting in
//! `employee_details.rs` with the redesigned `Dialog` shell and form atoms.
//!
//! Runs in two modes:
//! - **Create** (`editing == None`): default values, submit issues `POST /extra-hours`
//!   via the legacy direct-API path.
//! - **Edit** (`editing == Some(entry)`): prefill from the entry, submit issues
//!   `PUT /extra-hours/{id}` via `EmployeeAction::UpdateExtraHours` so the service
//!   layer handles 409 → refresh + translated notice.

use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use time::macros::{date, format_description};
use uuid::Uuid;

use crate::api;
use crate::base_types::ImStr;
use crate::component::atoms::{Btn, BtnVariant};
use crate::component::form::{Field, SelectInput, TextInput};
use crate::component::{Dialog, DialogVariant};
use crate::error::{result_handler, ShiftyError};
use crate::i18n::Key;
use crate::js;
use crate::service::{
    config::CONFIG,
    employee::{build_update_payload, EmployeeAction},
    i18n::I18N,
};
use crate::state::employee::{CustomExtraHoursDefinition, ExtraHours, WorkingHoursCategory};

#[derive(Props, Clone, PartialEq)]
pub struct ExtraHoursModalProps {
    pub open: bool,
    pub sales_person_id: Uuid,
    #[props(!optional, default = None)]
    pub editing: Option<ExtraHours>,
    pub on_saved: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
}

enum ExtraHoursModalAction {
    Submit,
    LoadCustomExtraHours,
}

#[cfg(target_arch = "wasm32")]
fn current_datetime_for_init() -> time::PrimitiveDateTime {
    js::current_datetime()
}

#[cfg(not(target_arch = "wasm32"))]
fn current_datetime_for_init() -> time::PrimitiveDateTime {
    use time::macros::datetime;
    datetime!(2026-01-01 00:00:00)
}

fn category_identifier(category: &WorkingHoursCategory) -> String {
    match category {
        WorkingHoursCategory::Custom(id) => format!("custom_{id}"),
        _ => category.identifier().to_string(),
    }
}

fn parse_category(identifier: &str) -> WorkingHoursCategory {
    if let Some(stripped) = identifier.strip_prefix("custom_") {
        if let Ok(uuid) = Uuid::parse_str(stripped) {
            return WorkingHoursCategory::Custom(uuid);
        }
        return WorkingHoursCategory::ExtraWork("".into());
    }
    WorkingHoursCategory::from_identifier(identifier)
}

#[component]
pub fn ExtraHoursModal(props: ExtraHoursModalProps) -> Element {
    if !props.open {
        return rsx! {};
    }

    let i18n = I18N.read().clone();
    let is_edit = props.editing.is_some();

    let title_str = if is_edit {
        i18n.t(Key::EditExtraHoursFormTitle)
    } else {
        i18n.t(Key::AddExtraHoursFormTitle)
    };
    let title = ImStr::from(title_str.as_ref());
    let cancel_str = ImStr::from(i18n.t(Key::Cancel).as_ref());
    let submit_str = ImStr::from(i18n.t(Key::Submit).as_ref());

    let on_cancel = props.on_cancel;
    let on_saved = props.on_saved;
    let sales_person_id = props.sales_person_id;

    let format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    let date_format = format_description!("[year]-[month]-[day]");

    let (init_category, init_amount, init_description, init_when) = match &props.editing {
        Some(eh) => (
            eh.category.clone(),
            eh.amount,
            eh.description.as_ref().to_string(),
            eh.date_time.format(&format).unwrap_or_default(),
        ),
        None => (
            WorkingHoursCategory::ExtraWork("".into()),
            0.0_f32,
            String::new(),
            current_datetime_for_init()
                .format(&format)
                .unwrap_or_default(),
        ),
    };

    let mut category = use_signal(|| init_category.clone());
    let mut amount = use_signal(|| init_amount);
    let mut description = use_signal(|| init_description.clone());
    let mut when = use_signal(|| init_when.clone());
    let mut from = use_signal(|| {
        current_datetime_for_init()
            .date()
            .format(&date_format)
            .unwrap_or_default()
    });
    let mut to = use_signal(|| {
        current_datetime_for_init()
            .date()
            .format(&date_format)
            .unwrap_or_default()
    });
    let mut custom_extra_hours = use_signal(|| Rc::<[CustomExtraHoursDefinition]>::from([]));

    // Re-seed signals when the dialog re-opens for a different entry (or
    // toggles between create and edit) without being unmounted.
    let editing_key = props.editing.as_ref().map(|e| e.id);
    let mut last_editing_key = use_signal(|| editing_key);
    if *last_editing_key.peek() != editing_key {
        last_editing_key.set(editing_key);
        category.set(init_category.clone());
        amount.set(init_amount);
        description.set(init_description.clone());
        when.set(init_when.clone());
    }

    let editing_for_submit = props.editing.clone();
    // `try_consume_context` rather than `use_coroutine_handle` so that
    // SSR-render unit tests can exercise the modal without registering the
    // EmployeeAction service. In real runs the coroutine is always present.
    let employee_service = try_consume_context::<Coroutine<EmployeeAction>>();

    let cr = use_coroutine(move |mut rx: UnboundedReceiver<ExtraHoursModalAction>| {
        let editing_for_submit = editing_for_submit.clone();
        async move {
            while let Some(action) = rx.next().await {
                match action {
                    ExtraHoursModalAction::LoadCustomExtraHours => {
                        let config = CONFIG.read().clone();
                        if let Ok(hours) =
                            api::get_custom_extra_hours_by_sales_person(config, sales_person_id)
                                .await
                        {
                            let definitions: Rc<[CustomExtraHoursDefinition]> =
                                hours.iter().map(|h| h.into()).collect();
                            custom_extra_hours.set(definitions);
                        }
                    }
                    ExtraHoursModalAction::Submit => {
                        let category_value: WorkingHoursCategory = category.read().clone();
                        let amount_value = *amount.read();
                        let description_value = description.read().clone();
                        let when_value = when.read().clone();
                        let config = CONFIG.read().clone();

                        if let Some(editing) = editing_for_submit.as_ref() {
                            // Edit path — PUT via service so 409 handling lives in one place.
                            let date_time = js::date_time_str_to_primitive_date_time(&when_value);
                            let to = build_update_payload(
                                editing,
                                amount_value,
                                (&category_value).into(),
                                description_value.into(),
                                date_time,
                            );
                            if let Some(svc) = &employee_service {
                                svc.send(EmployeeAction::UpdateExtraHours(to));
                            }
                            on_saved.call(());
                        } else if category_value == WorkingHoursCategory::VacationDays {
                            let amount_value = amount_value as i32;
                            let from_date = time::Date::parse(&*from.read(), &date_format)
                                .unwrap_or(date!(1970 - 01 - 01));
                            let to_date = time::Date::parse(&*to.read(), &date_format)
                                .unwrap_or(date!(1970 - 01 - 01));
                            result_handler(
                                api::add_vacation(
                                    config.to_owned(),
                                    sales_person_id,
                                    from_date,
                                    to_date,
                                    description_value.into(),
                                )
                                .await
                                .map_err(ShiftyError::from),
                            );
                            let _ = amount_value;
                            on_saved.call(());
                        } else {
                            result_handler(
                                api::add_extra_hour(
                                    config,
                                    sales_person_id,
                                    amount_value,
                                    (&category_value).into(),
                                    description_value,
                                    when_value,
                                )
                                .await,
                            );
                            on_saved.call(());
                        }
                    }
                }
            }
        }
    });

    use_effect(move || {
        cr.send(ExtraHoursModalAction::LoadCustomExtraHours);
    });

    let category_str = ImStr::from(i18n.t(Key::Category).as_ref());
    let amount_label = ImStr::from(i18n.t(Key::AmountOfHours).as_ref());
    let description_label = ImStr::from(i18n.t(Key::Description).as_ref());
    let when_label = ImStr::from(i18n.t(Key::When).as_ref());
    let from_label = ImStr::from(i18n.t(Key::FromLabel).as_ref());
    let to_label = ImStr::from(i18n.t(Key::ToLabel).as_ref());

    let extra_work_str = i18n.t(Key::CategoryExtraWork);
    let volunteer_work_str = i18n.t(Key::CategoryVolunteerWork);
    let holidays_str = i18n.t(Key::CategoryHolidays);
    let sick_leave_str = i18n.t(Key::CategorySickLeave);
    let vacation_days_str = i18n.t(Key::CategoryVacationDays);
    let unavailable_str = i18n.t(Key::CategoryUnavailable);
    let unpaid_leave_str = i18n.t(Key::CategoryUnpaidLeave);
    let vacation_str = i18n.t(Key::CategoryVacationHours);

    let current_category = category.read().clone();
    let current_category_id = category_identifier(&current_category);
    let is_vacation_days = current_category == WorkingHoursCategory::VacationDays;
    let custom_known = custom_extra_hours.read();
    let stale_custom_id: Option<Uuid> = match &current_category {
        WorkingHoursCategory::Custom(uuid) if !custom_known.iter().any(|def| &def.id == uuid) => {
            Some(*uuid)
        }
        _ => None,
    };

    let footer = rsx! {
        Btn {
            variant: BtnVariant::Secondary,
            on_click: move |_| on_cancel.call(()),
            "{cancel_str}"
        }
        Btn {
            variant: BtnVariant::Primary,
            on_click: move |_| cr.send(ExtraHoursModalAction::Submit),
            "{submit_str}"
        }
    };

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_cancel.call(()),
            title: title,
            variant: DialogVariant::Auto,
            width: 460,
            footer: Some(footer),

            div { class: "flex flex-col gap-3",
                Field { label: category_str,
                    SelectInput {
                        on_change: move |value: ImStr| {
                            category.set(parse_category(value.as_str()));
                        },
                        option {
                            value: "extra_work",
                            selected: current_category_id == "extra_work",
                            "{extra_work_str}"
                        }
                        option {
                            value: "volunteer_work",
                            selected: current_category_id == "volunteer_work",
                            "{volunteer_work_str}"
                        }
                        option {
                            value: "holiday",
                            selected: current_category_id == "holiday",
                            "{holidays_str}"
                        }
                        option {
                            value: "sick_leave",
                            selected: current_category_id == "sick_leave",
                            "{sick_leave_str}"
                        }
                        if !is_edit {
                            option {
                                value: "vacation_days",
                                selected: current_category_id == "vacation_days",
                                "{vacation_days_str}"
                            }
                        }
                        option {
                            value: "unavailable",
                            selected: current_category_id == "unavailable",
                            "{unavailable_str}"
                        }
                        option {
                            value: "unpaid_leave",
                            selected: current_category_id == "unpaid_leave",
                            "{unpaid_leave_str}"
                        }
                        if !custom_known.is_empty() {
                            option { disabled: true, "──────────" }
                            for custom_hour in custom_known.iter() {
                                option {
                                    value: "custom_{custom_hour.id}",
                                    selected: current_category_id == format!("custom_{}", custom_hour.id),
                                    "{custom_hour.name}"
                                }
                            }
                        }
                        if let Some(stale_id) = stale_custom_id {
                            option {
                                value: "custom_{stale_id}",
                                selected: true,
                                "custom_{stale_id}"
                            }
                        }
                        option { disabled: true, "──────────" }
                        option {
                            value: "vacation",
                            selected: current_category_id == "vacation",
                            "{vacation_str}"
                        }
                    }
                }

                Field { label: description_label,
                    TextInput {
                        value: ImStr::from(description.read().as_str()),
                        on_change: move |value: ImStr| description.set(value.as_str().to_string()),
                    }
                }

                if is_vacation_days && !is_edit {
                    Field { label: from_label,
                        TextInput {
                            value: ImStr::from(from.read().as_str()),
                            input_type: ImStr::from("date"),
                            on_change: move |value: ImStr| from.set(value.as_str().to_string()),
                        }
                    }
                    Field { label: to_label,
                        TextInput {
                            value: ImStr::from(to.read().as_str()),
                            input_type: ImStr::from("date"),
                            on_change: move |value: ImStr| to.set(value.as_str().to_string()),
                        }
                    }
                } else {
                    Field { label: amount_label,
                        TextInput {
                            value: ImStr::from(amount.read().to_string()),
                            input_type: ImStr::from("number"),
                            on_change: move |value: ImStr| {
                                if let Ok(n) = value.as_str().parse::<f32>() {
                                    amount.set(n);
                                }
                            },
                        }
                    }
                    Field { label: when_label,
                        TextInput {
                            value: ImStr::from(when.read().as_str()),
                            input_type: ImStr::from("datetime-local"),
                            on_change: move |value: ImStr| when.set(value.as_str().to_string()),
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use time::macros::datetime;

    #[test]
    fn category_identifier_for_built_ins() {
        assert_eq!(
            category_identifier(&WorkingHoursCategory::Vacation),
            "vacation"
        );
        assert_eq!(
            category_identifier(&WorkingHoursCategory::Holiday),
            "holiday"
        );
        assert_eq!(
            category_identifier(&WorkingHoursCategory::ExtraWork("".into())),
            "extra_work"
        );
    }

    #[test]
    fn category_identifier_for_custom_uses_uuid_prefix() {
        let id = Uuid::nil();
        assert_eq!(
            category_identifier(&WorkingHoursCategory::Custom(id)),
            format!("custom_{id}")
        );
    }

    #[test]
    fn parse_category_recognises_built_ins() {
        assert!(matches!(
            parse_category("vacation"),
            WorkingHoursCategory::Vacation
        ));
        assert!(matches!(
            parse_category("vacation_days"),
            WorkingHoursCategory::VacationDays
        ));
    }

    #[test]
    fn parse_category_recognises_custom_uuid_prefix() {
        let id = Uuid::nil();
        let parsed = parse_category(&format!("custom_{id}"));
        assert!(matches!(
            parsed,
            WorkingHoursCategory::Custom(parsed_id) if parsed_id == id
        ));
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    fn sample_extra_hours(category: WorkingHoursCategory) -> ExtraHours {
        ExtraHours {
            id: Uuid::from_u128(0x1234_5678_90ab_cdef_1234_5678_90ab_cdef),
            sales_person_id: Uuid::from_u128(0xaaaa_bbbb_cccc_dddd_eeee_ffff_0000_1111),
            amount: 4.5,
            category,
            description: Rc::from("note from existing entry"),
            date_time: datetime!(2026-04-15 10:30:00),
            version: Uuid::from_u128(0x9999_8888_7777_6666_5555_4444_3333_2222),
        }
    }

    #[test]
    fn closed_modal_renders_nothing() {
        fn app() -> Element {
            rsx! {
                ExtraHoursModal {
                    open: false,
                    sales_person_id: Uuid::nil(),
                    on_saved: |_| {},
                    on_cancel: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("shifty-dialog-title"),
            "closed modal should not render dialog: {html}"
        );
    }

    #[test]
    fn create_mode_renders_vacation_days_option() {
        fn app() -> Element {
            rsx! {
                ExtraHoursModal {
                    open: true,
                    sales_person_id: Uuid::nil(),
                    on_saved: |_| {},
                    on_cancel: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("value=\"vacation_days\""),
            "create-mode dialog must include vacation_days option: {html}"
        );
    }

    #[test]
    fn edit_mode_omits_vacation_days_option_and_prefills() {
        fn app() -> Element {
            rsx! {
                ExtraHoursModal {
                    open: true,
                    sales_person_id: Uuid::nil(),
                    editing: Some(sample_extra_hours(WorkingHoursCategory::ExtraWork("-".into()))),
                    on_saved: |_| {},
                    on_cancel: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("value=\"vacation_days\""),
            "edit-mode dialog must NOT include vacation_days option: {html}"
        );
        assert!(
            html.contains("note from existing entry"),
            "edit-mode dialog must prefill description from editing entry: {html}"
        );
        assert!(
            html.contains("4.5"),
            "edit-mode dialog must prefill amount from editing entry: {html}"
        );
    }

    #[test]
    fn edit_mode_keeps_unknown_custom_category_as_selected_option() {
        let stale_uuid = Uuid::from_u128(0xfeed_face_dead_beef_cafe_babe_1234_5678);
        // capture into a const-style closure: render() takes fn() -> Element so we
        // resolve the entry inline with literal values to avoid closures.
        fn app() -> Element {
            let stale_uuid = Uuid::from_u128(0xfeed_face_dead_beef_cafe_babe_1234_5678);
            rsx! {
                ExtraHoursModal {
                    open: true,
                    sales_person_id: Uuid::nil(),
                    editing: Some(ExtraHours {
                        id: Uuid::from_u128(0x1111_2222_3333_4444_5555_6666_7777_8888),
                        sales_person_id: Uuid::nil(),
                        amount: 2.0,
                        category: WorkingHoursCategory::Custom(stale_uuid),
                        description: Rc::from("orphan-custom"),
                        date_time: time::macros::datetime!(2026-04-01 09:00:00),
                        version: Uuid::nil(),
                    }),
                    on_saved: |_| {},
                    on_cancel: |_| {},
                }
            }
        }
        let html = render(app);
        let expected_value = format!("custom_{stale_uuid}");
        assert!(
            html.contains(&format!("value=\"{expected_value}\"")),
            "edit-mode dialog must preserve unknown Custom uuid as a selectable option: {html}"
        );
    }

    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("extra_hours_modal.rs");
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
