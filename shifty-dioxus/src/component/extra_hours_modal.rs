//! `ExtraHoursModal` — token-based dialog wrapping the extra-hours form.
//!
//! Replaces the legacy `Modal { AddExtraHoursForm }` mounting in
//! `employee_details.rs` with the redesigned `Dialog` shell and form atoms.

use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use time::macros::{date, format_description};
use uuid::Uuid;

use crate::api;
use crate::base_types::ImStr;
use crate::component::atoms::{Btn, BtnVariant};
use crate::component::form::{Field, FormSelectInput, FormTextInput};
use crate::component::{Dialog, DialogVariant};
use crate::error::{result_handler, ShiftyError};
use crate::i18n::Key;
use crate::js;
use crate::service::{config::CONFIG, i18n::I18N};
use crate::state::employee::{CustomExtraHoursDefinition, WorkingHoursCategory};

#[derive(Props, Clone, PartialEq)]
pub struct ExtraHoursModalProps {
    pub open: bool,
    pub sales_person_id: Uuid,
    pub on_saved: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
}

enum ExtraHoursModalAction {
    Submit,
    LoadCustomExtraHours,
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
    let title = ImStr::from(i18n.t(Key::AddExtraHoursFormTitle).as_ref());
    let cancel_str = ImStr::from(i18n.t(Key::Cancel).as_ref());
    let submit_str = ImStr::from(i18n.t(Key::Submit).as_ref());

    let on_cancel = props.on_cancel;
    let on_saved = props.on_saved;
    let sales_person_id = props.sales_person_id;

    let format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    let date_format = format_description!("[year]-[month]-[day]");
    let mut category = use_signal(|| WorkingHoursCategory::ExtraWork("".into()));
    let mut amount = use_signal(|| 0.0_f32);
    let mut description = use_signal(String::new);
    let mut when = use_signal(|| js::current_datetime().format(&format).unwrap_or_default());
    let mut from = use_signal(|| {
        js::current_datetime()
            .date()
            .format(&date_format)
            .unwrap_or_default()
    });
    let mut to = use_signal(|| {
        js::current_datetime()
            .date()
            .format(&date_format)
            .unwrap_or_default()
    });
    let mut custom_extra_hours = use_signal(|| Rc::<[CustomExtraHoursDefinition]>::from([]));

    let cr = use_coroutine(
        move |mut rx: UnboundedReceiver<ExtraHoursModalAction>| async move {
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
                        let category: WorkingHoursCategory = category.read().clone();
                        let amount_value = *amount.read();
                        let description_value = description.read().clone();
                        let when_value = when.read().clone();
                        let config = CONFIG.read().clone();

                        if category == WorkingHoursCategory::VacationDays {
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
                        } else {
                            result_handler(
                                api::add_extra_hour(
                                    config,
                                    sales_person_id,
                                    amount_value,
                                    (&category).into(),
                                    description_value,
                                    when_value,
                                )
                                .await,
                            );
                        }
                        on_saved.call(());
                    }
                }
            }
        },
    );

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

    let current_category_id = category_identifier(&category.read());
    let is_vacation_days = *category.read() == WorkingHoursCategory::VacationDays;

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
                    FormSelectInput {
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
                        option {
                            value: "vacation_days",
                            selected: current_category_id == "vacation_days",
                            "{vacation_days_str}"
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
                        if !custom_extra_hours.read().is_empty() {
                            option { disabled: true, "──────────" }
                            for custom_hour in custom_extra_hours.read().iter() {
                                option {
                                    value: "custom_{custom_hour.id}",
                                    selected: current_category_id == format!("custom_{}", custom_hour.id),
                                    "{custom_hour.name}"
                                }
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
                    FormTextInput {
                        value: ImStr::from(description.read().as_str()),
                        on_change: move |value: ImStr| description.set(value.as_str().to_string()),
                    }
                }

                if is_vacation_days {
                    Field { label: from_label,
                        FormTextInput {
                            value: ImStr::from(from.read().as_str()),
                            input_type: ImStr::from("date"),
                            on_change: move |value: ImStr| from.set(value.as_str().to_string()),
                        }
                    }
                    Field { label: to_label,
                        FormTextInput {
                            value: ImStr::from(to.read().as_str()),
                            input_type: ImStr::from("date"),
                            on_change: move |value: ImStr| to.set(value.as_str().to_string()),
                        }
                    }
                } else {
                    Field { label: amount_label,
                        FormTextInput {
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
                        FormTextInput {
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
