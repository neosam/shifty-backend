//! `ContractModal` — token-based dialog wrapping the contract form.
//!
//! Replaces the legacy `Modal { EmployeeWorkDetailsForm }` mounting in
//! `employee_details.rs` with the redesigned `Dialog` shell and form atoms.

use dioxus::prelude::*;
use time::macros::format_description;

use crate::base_types::ImStr;
use crate::component::atoms::{Btn, BtnVariant};
use crate::component::employee_work_details_form::EmployeeWorkDetailsFormType;
use crate::component::form::{Field, FormCheckbox, TextInput};
use crate::component::{Dialog, DialogVariant};
use crate::i18n::Key;
use crate::service::{
    employee_work_details::EmployeeWorkDetailsAction,
    employee_work_details::EMPLOYEE_WORK_DETAILS_STORE, i18n::I18N,
};
use crate::state::employee_work_details::EmployeeWorkDetails;

const PILL_BASE: &str = "min-w-[38px] h-8 px-2 inline-flex items-center justify-center rounded-md text-xs cursor-pointer disabled:cursor-not-allowed disabled:opacity-50";
const PILL_ACTIVE: &str = "bg-accent text-accent-ink border border-accent font-semibold";
const PILL_INACTIVE_WEEKDAY: &str = "bg-surface text-ink border border-border-strong font-medium";
const PILL_INACTIVE_WEEKEND: &str =
    "bg-surface text-ink-muted border border-border-strong font-medium";

/// Builds the class string for a weekday pill.
///
/// - Active pills always use the accent token regardless of weekday/weekend.
/// - Inactive weekdays render in `text-ink`; inactive weekend days render in
///   `text-ink-muted` so the weekend reads as the lower-emphasis default.
pub(crate) fn weekday_pill_class(active: bool, weekend: bool) -> String {
    let mut out = String::with_capacity(160);
    out.push_str(PILL_BASE);
    out.push(' ');
    if active {
        out.push_str(PILL_ACTIVE);
    } else if weekend {
        out.push_str(PILL_INACTIVE_WEEKEND);
    } else {
        out.push_str(PILL_INACTIVE_WEEKDAY);
    }
    out
}

#[derive(Props, Clone, PartialEq)]
pub struct ContractModalProps {
    pub open: bool,
    pub form_type: EmployeeWorkDetailsFormType,
    pub on_save: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
}

#[component]
pub fn ContractModal(props: ContractModalProps) -> Element {
    if !props.open {
        return rsx! {};
    }

    let i18n = I18N.read().clone();
    let title: ImStr = ImStr::from(
        i18n.t_m(
            Key::AddWorkDetailsFormTitle,
            [(
                "name",
                EMPLOYEE_WORK_DETAILS_STORE
                    .read()
                    .selected_sales_person
                    .name
                    .as_ref(),
            )]
            .into(),
        )
        .as_ref(),
    );
    let read_only = props.form_type == EmployeeWorkDetailsFormType::ReadOnly;
    let cancel_label = if read_only {
        ImStr::from(i18n.t(Key::Cancel).as_ref())
    } else {
        ImStr::from(i18n.t(Key::Cancel).as_ref())
    };
    let save_label = ImStr::from(i18n.t(Key::Save).as_ref());

    let on_cancel = props.on_cancel;
    let on_save = props.on_save;

    let footer = rsx! {
        Btn {
            variant: BtnVariant::Secondary,
            on_click: move |_| on_cancel.call(()),
            "{cancel_label}"
        }
        if !read_only {
            Btn {
                variant: BtnVariant::Primary,
                on_click: move |_| on_save.call(()),
                "{save_label}"
            }
        }
    };

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_cancel.call(()),
            title: title,
            variant: DialogVariant::Auto,
            width: 520,
            footer: Some(footer),
            ContractModalBody { form_type: props.form_type }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ContractModalBodyProps {
    form_type: EmployeeWorkDetailsFormType,
}

#[component]
fn ContractModalBody(props: ContractModalBodyProps) -> Element {
    let i18n = I18N.read().clone();
    let store = EMPLOYEE_WORK_DETAILS_STORE.read();
    let details = store.selected_employee_work_details.clone();
    let work_details_service = use_coroutine_handle::<EmployeeWorkDetailsAction>();

    let date_format = format_description!("[year]-[month]-[day]");
    let from_str = details.from.format(&date_format).unwrap_or_default();
    let to_str = details.to.format(&date_format).unwrap_or_default();

    let read_only = props.form_type == EmployeeWorkDetailsFormType::ReadOnly;
    let editable_dates = props.form_type == EmployeeWorkDetailsFormType::New;
    let editable_to = !read_only;

    let from_label = ImStr::from(i18n.t(Key::FromLabel).as_ref());
    let to_label = ImStr::from(i18n.t(Key::ToLabel).as_ref());
    let workdays_label = ImStr::from(i18n.t(Key::WorkdaysLabel).as_ref());
    let expected_hours_label = ImStr::from(i18n.t(Key::ExpectedHoursPerWeekLabel).as_ref());
    let days_per_week_label = ImStr::from(i18n.t(Key::DaysPerWeekLabel).as_ref());
    let vacation_days_label = ImStr::from(i18n.t(Key::VacationEntitlementsPerYearLabel).as_ref());
    let dynamic_label = ImStr::from(i18n.t(Key::DynamicHourLabel).as_ref());
    let cap_label = ImStr::from(i18n.t(Key::CapPlannedHoursLabel).as_ref());
    let cap_help = ImStr::from(i18n.t(Key::CapPlannedHoursHelp).as_ref());
    let holiday_in_hours_label = i18n.t(Key::HolidaysInHoursLabel);
    let workday_in_hours_label = i18n.t(Key::WorkdaysInHoursLabel);

    let monday_label = i18n.t(Key::Monday);
    let tuesday_label = i18n.t(Key::Tuesday);
    let wednesday_label = i18n.t(Key::Wednesday);
    let thursday_label = i18n.t(Key::Thursday);
    let friday_label = i18n.t(Key::Friday);
    let saturday_label = i18n.t(Key::Saturday);
    let sunday_label = i18n.t(Key::Sunday);

    let dispatch = move |updated: EmployeeWorkDetails| {
        work_details_service.send(EmployeeWorkDetailsAction::UpdateWorkingHours(updated));
    };

    let parse_date = |value: &str| -> Option<time::Date> {
        time::Date::parse(value, &format_description!("[year]-[month]-[day]")).ok()
    };

    let holiday_hours_text = format!("{}: {:.2}", holiday_in_hours_label, details.holiday_hours());
    let vacation_day_text = format!(
        "{}: {:.2}",
        workday_in_hours_label,
        details.vacation_day_in_hours()
    );

    rsx! {
        div { class: "flex flex-col gap-3",
            // Date range
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                Field {
                    label: from_label,
                    TextInput {
                        value: ImStr::from(from_str.as_str()),
                        input_type: ImStr::from("date"),
                        disabled: !editable_dates,
                        on_change: {
                            let details = details.clone();
                            move |value: ImStr| {
                                if !editable_dates {
                                    return;
                                }
                                if let Some(d) = parse_date(value.as_str()) {
                                    let mut next = details.clone();
                                    next.from = d;
                                    dispatch(next);
                                }
                            }
                        },
                    }
                }
                Field {
                    label: to_label,
                    TextInput {
                        value: ImStr::from(to_str.as_str()),
                        input_type: ImStr::from("date"),
                        disabled: !editable_to,
                        on_change: {
                            let details = details.clone();
                            move |value: ImStr| {
                                if !editable_to {
                                    return;
                                }
                                if let Some(d) = parse_date(value.as_str()) {
                                    let mut next = details.clone();
                                    next.to = d;
                                    dispatch(next);
                                }
                            }
                        },
                    }
                }
            }

            // Weekday pill buttons
            Field { label: workdays_label,
                div { class: "flex flex-wrap gap-1 mt-1",
                    {
                        let pills: [(bool, &str, bool, fn(&mut EmployeeWorkDetails, bool)); 7] = [
                            (
                                details.monday,
                                monday_label.as_ref(),
                                false,
                                |d, v| d.monday = v,
                            ),
                            (
                                details.tuesday,
                                tuesday_label.as_ref(),
                                false,
                                |d, v| d.tuesday = v,
                            ),
                            (
                                details.wednesday,
                                wednesday_label.as_ref(),
                                false,
                                |d, v| d.wednesday = v,
                            ),
                            (
                                details.thursday,
                                thursday_label.as_ref(),
                                false,
                                |d, v| d.thursday = v,
                            ),
                            (
                                details.friday,
                                friday_label.as_ref(),
                                false,
                                |d, v| d.friday = v,
                            ),
                            (
                                details.saturday,
                                saturday_label.as_ref(),
                                true,
                                |d, v| d.saturday = v,
                            ),
                            (
                                details.sunday,
                                sunday_label.as_ref(),
                                true,
                                |d, v| d.sunday = v,
                            ),
                        ];
                        rsx! {
                            for (active, label, weekend, setter) in pills.into_iter() {
                                {
                                    let label = label.to_string();
                                    let class = weekday_pill_class(active, weekend);
                                    let details = details.clone();
                                    rsx! {
                                        button {
                                            r#type: "button",
                                            class: "{class}",
                                            disabled: !editable_dates,
                                            onclick: move |evt| {
                                                evt.prevent_default();
                                                if !editable_dates {
                                                    return;
                                                }
                                                let mut next = details.clone();
                                                setter(&mut next, !active);
                                                dispatch(next);
                                            },
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Numeric fields
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                Field {
                    label: expected_hours_label,
                    TextInput {
                        value: ImStr::from(details.expected_hours.to_string()),
                        input_type: ImStr::from("number"),
                        disabled: read_only,
                        on_change: {
                            let details = details.clone();
                            move |value: ImStr| {
                                if read_only {
                                    return;
                                }
                                if let Ok(n) = value.as_str().parse::<f32>() {
                                    let mut next = details.clone();
                                    next.expected_hours = n;
                                    dispatch(next);
                                }
                            }
                        },
                    }
                }
                Field {
                    label: days_per_week_label,
                    TextInput {
                        value: ImStr::from(details.workdays_per_week.to_string()),
                        input_type: ImStr::from("number"),
                        disabled: read_only,
                        on_change: {
                            let details = details.clone();
                            move |value: ImStr| {
                                if read_only {
                                    return;
                                }
                                if let Ok(n) = value.as_str().parse::<u8>() {
                                    let mut next = details.clone();
                                    next.workdays_per_week = n;
                                    dispatch(next);
                                }
                            }
                        },
                    }
                }
            }

            if !read_only {
                Field {
                    label: vacation_days_label,
                    TextInput {
                        value: ImStr::from(details.vacation_days.to_string()),
                        input_type: ImStr::from("number"),
                        on_change: {
                            let details = details.clone();
                            move |value: ImStr| {
                                if let Ok(n) = value.as_str().parse::<u8>() {
                                    let mut next = details.clone();
                                    next.vacation_days = n;
                                    dispatch(next);
                                }
                            }
                        },
                    }
                }
            }

            // Toggle fields
            FormCheckbox {
                value: details.dynamic,
                disabled: read_only,
                on_change: {
                    let details = details.clone();
                    move |v: bool| {
                        if read_only {
                            return;
                        }
                        let mut next = details.clone();
                        next.dynamic = v;
                        dispatch(next);
                    }
                },
                label: rsx! { "{dynamic_label}" },
            }
            div { class: "flex flex-col gap-1",
                FormCheckbox {
                    value: details.cap_planned_hours_to_expected,
                    disabled: read_only,
                    on_change: {
                        let details = details.clone();
                        move |v: bool| {
                            if read_only {
                                return;
                            }
                            let mut next = details.clone();
                            next.cap_planned_hours_to_expected = v;
                            dispatch(next);
                        }
                    },
                    label: rsx! { "{cap_label}" },
                }
                span { class: "text-xs text-ink-muted", "{cap_help}" }
            }

            // Derived hours info
            div { class: "border-t border-border pt-3 flex flex-col gap-1 text-xs text-ink-muted",
                span { "{holiday_hours_text}" }
                span { "{vacation_day_text}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::employee_work_details_form::EmployeeWorkDetailsFormType;

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn closed_modal_renders_nothing() {
        fn app() -> Element {
            rsx! {
                ContractModal {
                    open: false,
                    form_type: EmployeeWorkDetailsFormType::New,
                    on_save: |_| {},
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
    fn weekday_pill_active_uses_accent_tokens() {
        let c = weekday_pill_class(true, false);
        assert!(c.contains("bg-accent"), "missing bg-accent: {c}");
        assert!(
            c.contains("text-accent-ink"),
            "missing text-accent-ink: {c}"
        );
        assert!(c.contains("border-accent"), "missing border-accent: {c}");
        assert!(c.contains("font-semibold"), "missing font-semibold: {c}");
    }

    #[test]
    fn weekday_pill_active_overrides_weekend_dimming() {
        // Active weekend pills still use the accent tokens, not the muted ones.
        let c = weekday_pill_class(true, true);
        assert!(c.contains("bg-accent"));
        assert!(
            !c.contains("text-ink-muted"),
            "active pill must not be ink-muted: {c}"
        );
    }

    #[test]
    fn weekday_pill_inactive_weekday_uses_ink() {
        let c = weekday_pill_class(false, false);
        assert!(c.contains("bg-surface"), "missing bg-surface: {c}");
        assert!(c.contains("text-ink"), "missing text-ink: {c}");
        assert!(
            !c.contains("text-ink-muted"),
            "inactive weekday must not be muted: {c}"
        );
        assert!(c.contains("border-border-strong"));
    }

    #[test]
    fn weekday_pill_inactive_weekend_is_muted() {
        let c = weekday_pill_class(false, true);
        assert!(c.contains("text-ink-muted"), "missing text-ink-muted: {c}");
        assert!(c.contains("bg-surface"));
        assert!(c.contains("border-border-strong"));
        assert!(!c.contains("bg-accent"));
    }

    #[test]
    fn weekday_pill_includes_layout_classes() {
        for (active, weekend) in [(false, false), (true, false), (false, true), (true, true)] {
            let c = weekday_pill_class(active, weekend);
            assert!(c.contains("min-w-[38px]"), "missing min-w pill width: {c}");
            assert!(c.contains("h-8"), "missing pill height: {c}");
            assert!(c.contains("rounded-md"), "missing rounded-md: {c}");
            assert!(c.contains("inline-flex"));
        }
    }

    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("contract_modal.rs");
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
