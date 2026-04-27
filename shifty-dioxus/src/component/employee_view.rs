use std::rc::Rc;

use dioxus::prelude::*;
use uuid::Uuid;

use crate::base_types::{format_hours, ImStr};
use crate::component::atoms::{Btn, BtnVariant, NavBtn, PersonChip, TupleRow};
use crate::component::dropdown_base::DropdownTrigger;
use crate::component::EmployeeWeeklyHistogram;
use crate::i18n::Key;
use crate::js;
use crate::service::{
    employee::EmployeeAction, employee::EMPLOYEE_STORE,
    employee_work_details::EMPLOYEE_WORK_DETAILS_STORE, i18n::I18N,
};
use crate::state::employee::{
    CustomExtraHours, CustomExtraHoursDefinition, Employee, ExtraHours, WorkingHours,
};
use crate::state::employee_work_details::EmployeeWorkDetails;

const TYPE_PILL_PAID_HEX: &str = "#eaecfb"; // var(--accent-soft) light theme
const TYPE_PILL_VOLUNTEER_HEX: &str = "#fef0d6"; // var(--warn-soft) light theme

#[derive(Props, Clone, PartialEq)]
pub struct EmployeeViewPlainProps {
    pub employee: Employee,
    pub extra_hours: Rc<[ExtraHours]>,
    pub employee_work_details_list: Rc<[EmployeeWorkDetails]>,
    pub show_delete_employee_work_details: bool,
    pub year: u32,
    pub show_vacation: bool,
    pub full_year: bool,
    pub custom_hours: Rc<[CustomExtraHours]>,
    pub custom_extra_hours_definitions: Rc<[CustomExtraHoursDefinition]>,

    pub onupdate: EventHandler<()>,
    pub on_extra_hour_delete: EventHandler<Uuid>,
    pub on_custom_delete: EventHandler<Uuid>,
    pub on_full_year: EventHandler<()>,
    pub on_until_now: EventHandler<()>,
    pub on_add_employee_work_details: Option<EventHandler<()>>,
    pub on_employee_work_details_clicked: EventHandler<Uuid>,
    pub on_delete_employee_work_details_clicked: Option<EventHandler<Uuid>>,
    pub on_next_year: EventHandler<()>,
    pub on_previous_year: EventHandler<()>,

    #[props(!optional, default = None)]
    pub on_open_extra_hours: Option<EventHandler<()>>,
}

fn most_recent_expected(work_details: &[EmployeeWorkDetails]) -> f32 {
    work_details
        .iter()
        .max_by_key(|d| d.from)
        .map(|d| d.expected_hours)
        .unwrap_or(0.0)
}

#[component]
pub fn EmployeeViewPlain(props: EmployeeViewPlainProps) -> Element {
    let i18n = I18N.read().clone();
    let mut selected_week = use_signal(|| None::<(u32, u8)>);
    let mut expand_weeks = use_signal(|| false);

    let employee = props.employee.clone();
    let work_details_list = props.employee_work_details_list.clone();
    let custom_hours = props.custom_hours.clone();

    // Header text
    let name = employee.sales_person.name.clone();
    let color = employee.sales_person.background_color.clone();
    let is_paid = employee.sales_person.is_paid;
    let type_label = if is_paid {
        i18n.t(Key::Paid)
    } else {
        i18n.t(Key::Volunteer)
    };
    let pill_color = if is_paid {
        TYPE_PILL_PAID_HEX
    } else {
        TYPE_PILL_VOLUNTEER_HEX
    };
    let expected_per_week = most_recent_expected(&work_details_list);

    // i18n labels
    let overall_header_str = i18n.t(Key::OverallHeading);
    let work_details_header = i18n.t(Key::WorkDetailsHeading);
    let working_hours_per_week_heading = i18n.t(Key::WorkingHoursPerWeekHeading);
    let extra_hours_heading = i18n.t(Key::ExtraHoursHeading);
    let balance_str = i18n.t(Key::Balance);
    let overall_str = i18n.t(Key::Overall);
    let required_str = i18n.t(Key::Required);
    let carryover_balance_str = i18n.t(Key::CarryoverBalance);
    let shiftplan_str = i18n.t(Key::CategoryShiftplan);
    let extra_work_str = i18n.t(Key::CategoryExtraWork);
    let vacation_str = i18n.t(Key::CategoryVacation);
    let sick_leave_str = i18n.t(Key::CategorySickLeave);
    let holidays_str = i18n.t(Key::CategoryHolidays);
    let unpaid_leave_str = i18n.t(Key::CategoryUnpaidLeave);
    let volunteer_work_str = i18n.t(Key::CategoryVolunteerWork);
    let hours_str: ImStr = ImStr::from(i18n.t(Key::Hours).as_ref());
    let _hours_short_str = i18n.t(Key::HoursShort);
    let _actions_label: ImStr = i18n.t(Key::ActionsLabel).into();
    let show_full_year_label: ImStr = i18n.t(Key::ShowFullYearLabel).into();
    let show_until_now_label: ImStr = i18n.t(Key::ShowUntilNowLabel).into();
    let other_hours_str = i18n.t(Key::OtherHours);
    let more_str = i18n.t(Key::More);
    let show_details_str = i18n.t(Key::ShowDetails);
    let hide_details_str = i18n.t(Key::HideDetails);
    let week_short_str = i18n.t(Key::WeekShort);
    let add_work_details_label: ImStr = i18n.t(Key::AddWorkDetailsLabel).into();
    let vacation_days_str: ImStr = i18n.t(Key::VacationDaysLabel).into();
    let vacation_carryover_str: ImStr = i18n.t(Key::VacationCarryoverLabel).into();
    let current_week_note = i18n.t(Key::CurrentWeekNote);

    let prev_year_aria = ImStr::from(i18n.t(Key::PreviousYear).as_ref());
    let next_year_aria = ImStr::from(i18n.t(Key::NextYear).as_ref());

    let on_next_year = props.on_next_year;
    let on_prev_year = props.on_previous_year;
    let on_full_year = props.on_full_year;
    let on_until_now = props.on_until_now;
    let on_open_extra_hours = props.on_open_extra_hours;
    let on_add_work_details = props.on_add_employee_work_details;
    let on_work_details_clicked = props.on_employee_work_details_clicked;

    let year = props.year;
    let current_year = js::get_current_year();
    let current_week = js::get_current_week();

    let dot_style = format!("background-color: {}; width: 32px; height: 32px;", color);

    // Histogram data: full year
    let histogram_weeks: Rc<[WorkingHours]> = employee.working_hours_by_week.clone();

    let selected_week_data = selected_week.read().and_then(|(year, week)| {
        employee
            .working_hours_by_week
            .iter()
            .find(|w| {
                let (y, wk, _) = w.from.to_iso_week_date();
                y as u32 == year && wk == week
            })
            .cloned()
    });

    rsx! {
        // Header
        section { class: "flex flex-col gap-3 pb-4 border-b border-border",
            div { class: "flex flex-wrap items-center gap-3",
                span {
                    class: "rounded-full inline-block flex-shrink-0",
                    style: "{dot_style}",
                }
                h1 { class: "text-xl font-semibold text-ink", "{name}" }
                PersonChip {
                    name: ImStr::from(type_label.as_ref()),
                    color: Some(ImStr::from(pill_color)),
                }
                if expected_per_week > 0.0 {
                    span { class: "font-mono tabular-nums text-ink-muted text-sm",
                        "{expected_per_week:.0} {hours_str}"
                    }
                }
            }
            div { class: "flex flex-wrap items-center gap-3 print:hidden",
                div { class: "flex items-center gap-2",
                    NavBtn {
                        glyph: ImStr::from("‹"),
                        aria_label: Some(prev_year_aria),
                        on_click: Some(EventHandler::new(move |_| on_prev_year.call(()))),
                    }
                    span { class: "font-mono text-base text-ink min-w-[4ch] text-center", "{year}" }
                    NavBtn {
                        glyph: ImStr::from("›"),
                        aria_label: Some(next_year_aria),
                        on_click: Some(EventHandler::new(move |_| on_next_year.call(()))),
                    }
                }
                if let Some(handler) = on_open_extra_hours {
                    Btn {
                        variant: BtnVariant::Primary,
                        on_click: move |_| handler.call(()),
                        "{other_hours_str}"
                    }
                }
                DropdownTrigger {
                    entries: [
                        (
                            show_full_year_label.clone(),
                            Box::new(move |_| on_full_year.call(())),
                            props.full_year,
                        ).into(),
                        (
                            show_until_now_label,
                            Box::new(move |_| on_until_now.call(())),
                            !props.full_year,
                        ).into(),
                    ].into(),
                    Btn { variant: BtnVariant::Secondary, "{more_str} ▾" }
                }
            }
            if !props.full_year {
                div { class: "text-xs text-ink-muted italic flex flex-wrap items-baseline gap-2",
                    span { "{current_week_note}" }
                    button {
                        r#type: "button",
                        class: "text-accent underline cursor-pointer text-xs",
                        onclick: move |_| on_full_year.call(()),
                        "{show_full_year_label}"
                    }
                }
            }
        }

        // 3-column sub-grid
        section {
            class: "grid gap-6 mt-6",
            style: "grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));",

            // Gesamtansicht column
            div { class: "flex flex-col gap-2",
                h2 { class: "text-sm font-semibold uppercase tracking-wide text-ink-muted",
                    "{overall_header_str}"
                }
                TupleRow {
                    label: ImStr::from(balance_str.as_ref()),
                    value: rsx! { span { class: "font-mono tabular-nums",
                        {format!("{} {}", format_hours(employee.balance, 2), hours_str)}
                    } },
                }
                TupleRow {
                    label: ImStr::from(overall_str.as_ref()),
                    value: rsx! { span { class: "font-mono tabular-nums",
                        {format!("{} {}", format_hours(employee.overall_working_hours, 2), hours_str)}
                    } },
                }
                TupleRow {
                    label: ImStr::from(required_str.as_ref()),
                    value: rsx! { span { class: "font-mono tabular-nums",
                        {format!("{} {}", format_hours(employee.expected_working_hours, 2), hours_str)}
                    } },
                }
                div { class: "border-t border-border my-2" }
                TupleRow {
                    label: ImStr::from(shiftplan_str.as_ref()),
                    value: rsx! { span { class: "font-mono tabular-nums",
                        {format!("{} {}", format_hours(employee.shiftplan_hours, 2), hours_str)}
                    } },
                    dim: true,
                }
                TupleRow {
                    label: ImStr::from(extra_work_str.as_ref()),
                    value: rsx! { span { class: "font-mono tabular-nums",
                        {format!("{} {}", format_hours(employee.extra_work_hours, 2), hours_str)}
                    } },
                    dim: true,
                }
                TupleRow {
                    label: ImStr::from(vacation_str.as_ref()),
                    value: rsx! { span { class: "font-mono tabular-nums",
                        {format!("{} {}", format_hours(employee.vacation_hours, 2), hours_str)}
                    } },
                    dim: true,
                }
                TupleRow {
                    label: ImStr::from(sick_leave_str.as_ref()),
                    value: rsx! { span { class: "font-mono tabular-nums",
                        {format!("{} {}", format_hours(employee.sick_leave_hours, 2), hours_str)}
                    } },
                    dim: true,
                }
                TupleRow {
                    label: ImStr::from(holidays_str.as_ref()),
                    value: rsx! { span { class: "font-mono tabular-nums",
                        {format!("{} {}", format_hours(employee.holiday_hours, 2), hours_str)}
                    } },
                    dim: true,
                }
                TupleRow {
                    label: ImStr::from(unpaid_leave_str.as_ref()),
                    value: rsx! { span { class: "font-mono tabular-nums",
                        {format!("{} {}", format_hours(employee.unpaid_leave_hours, 2), hours_str)}
                    } },
                    dim: true,
                }
                TupleRow {
                    label: ImStr::from(volunteer_work_str.as_ref()),
                    value: rsx! { span { class: "font-mono tabular-nums",
                        {format!("{} {}", format_hours(employee.volunteer_hours, 2), hours_str)}
                    } },
                    dim: true,
                }
                TupleRow {
                    label: ImStr::from(carryover_balance_str.as_ref()),
                    value: rsx! { span { class: "font-mono tabular-nums",
                        {format!("{} {}", format_hours(employee.carryover_balance, 2), hours_str)}
                    } },
                    dim: true,
                }
                for custom_hour in custom_hours.iter() {
                    TupleRow {
                        label: ImStr::from(custom_hour.name.as_ref()),
                        value: rsx! { span { class: "font-mono tabular-nums",
                            {format!("{} {}", format_hours(custom_hour.hours, 2), hours_str)}
                        } },
                        dim: true,
                    }
                }
                if props.show_vacation {
                    TupleRow {
                        label: vacation_days_str,
                        value: rsx! { span { class: "font-mono tabular-nums",
                            {format!("{} / {}", employee.vacation_days, employee.vacation_entitlement)}
                        } },
                        dim: true,
                    }
                    TupleRow {
                        label: vacation_carryover_str,
                        value: rsx! { span { class: "font-mono tabular-nums",
                            {format!("{}", employee.vacation_carryover)}
                        } },
                        dim: true,
                    }
                }
            }

            // Arbeitsverträge + Stunden pro Woche column
            div { class: "flex flex-col gap-2",
                h2 { class: "text-sm font-semibold uppercase tracking-wide text-ink-muted",
                    "{work_details_header}"
                }
                div { class: "flex flex-col gap-2",
                    for details in work_details_list.iter() {
                        ContractCard {
                            details: details.clone(),
                            on_click: {
                                let id = details.id;
                                move |_| on_work_details_clicked.call(id)
                            },
                            hours_label: hours_str.clone(),
                        }
                    }
                    if let Some(handler) = on_add_work_details {
                        Btn {
                            variant: BtnVariant::Secondary,
                            icon: Some(ImStr::from("+")),
                            on_click: move |_| handler.call(()),
                            "{add_work_details_label}"
                        }
                    }
                }
                div { class: "flex items-baseline justify-between mt-3 gap-2",
                    h3 { class: "text-xs font-semibold uppercase tracking-wide text-ink-muted",
                        "{working_hours_per_week_heading}"
                    }
                    button {
                        r#type: "button",
                        class: "text-accent underline cursor-pointer text-xs",
                        onclick: move |_| {
                            let v = *expand_weeks.read();
                            expand_weeks.set(!v);
                        },
                        if *expand_weeks.read() { "{hide_details_str}" } else { "{show_details_str}" }
                    }
                }
                EmployeeWeeklyHistogram {
                    weeks: histogram_weeks.clone(),
                    expected_per_week,
                    current_year,
                    current_week,
                    selected_week: *selected_week.read(),
                    on_select: move |pair: (u32, u8)| {
                        let current = *selected_week.read();
                        if current == Some(pair) {
                            selected_week.set(None);
                        } else {
                            selected_week.set(Some(pair));
                        }
                    },
                }
                if let Some(week) = selected_week_data {
                    WeekDetailPanel {
                        week,
                        hours_label: hours_str.clone(),
                        on_close: move |_| selected_week.set(None),
                    }
                }
                if *expand_weeks.read() {
                    WeekListExpanded {
                        weeks: histogram_weeks.clone(),
                        selected_week: *selected_week.read(),
                        hours_label: hours_str.clone(),
                        week_short: ImStr::from(week_short_str.as_ref()),
                        on_select: move |pair: (u32, u8)| {
                            let current = *selected_week.read();
                            if current == Some(pair) {
                                selected_week.set(None);
                            } else {
                                selected_week.set(Some(pair));
                            }
                        },
                    }
                }
            }

            // Zusatzarbeit column
            div { class: "flex flex-col gap-2",
                h2 { class: "text-sm font-semibold uppercase tracking-wide text-ink-muted",
                    "{extra_hours_heading}"
                }
                ExtraHoursView {
                    extra_hours: props.extra_hours.clone(),
                    custom_hours: props.custom_hours.clone(),
                    custom_extra_hours_definitions: props.custom_extra_hours_definitions.clone(),
                    ondelete: move |uuid| {
                        props.on_extra_hour_delete.call(uuid);
                        props.onupdate.call(());
                    },
                    on_custom_delete: move |uuid| {
                        props.on_custom_delete.call(uuid);
                    },
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ContractCardProps {
    details: EmployeeWorkDetails,
    on_click: EventHandler<()>,
    hours_label: ImStr,
}

#[component]
fn ContractCard(props: ContractCardProps) -> Element {
    let i18n = I18N.read().clone();
    let from_str = i18n.format_date(&props.details.from);
    let to_str = i18n.format_date(&props.details.to);
    let on_click = props.on_click;
    let hours_label = props.hours_label;
    rsx! {
        button {
            class: "w-full text-left rounded-md border border-border bg-surface px-3 py-2 hover:bg-surface-alt cursor-pointer",
            onclick: move |_| on_click.call(()),
            div { class: "flex items-baseline justify-between gap-2",
                span { class: "text-sm font-semibold text-ink", "{from_str} – {to_str}" }
                span { class: "font-mono tabular-nums text-xs text-ink-muted",
                    "{props.details.expected_hours} {hours_label}/Woche"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct WeekListExpandedProps {
    weeks: Rc<[WorkingHours]>,
    selected_week: Option<(u32, u8)>,
    hours_label: ImStr,
    week_short: ImStr,
    on_select: EventHandler<(u32, u8)>,
}

#[component]
fn WeekListExpanded(props: WeekListExpandedProps) -> Element {
    let on_select = props.on_select;
    // Show every loaded week, newest first.
    let visible: Vec<WorkingHours> = {
        let mut all: Vec<WorkingHours> = props.weeks.iter().cloned().collect();
        all.reverse();
        all
    };
    rsx! {
        div { class: "mt-2 flex flex-col text-xs",
            for week in visible.into_iter() {
                {
                    let (iso_year, iso_week, _) = week.from.to_iso_week_date();
                    let key = (iso_year as u32, iso_week);
                    let is_selected = props.selected_week == Some(key);
                    let under = week.overall_hours < week.expected_hours;
                    let row_class = if is_selected {
                        "w-full text-left flex items-center justify-between px-2 py-1.5 border-b border-border bg-accent-soft cursor-pointer"
                    } else {
                        "w-full text-left flex items-center justify-between px-2 py-1.5 border-b border-border hover:bg-surface-alt cursor-pointer"
                    };
                    let value_class = if under {
                        "font-mono tabular-nums text-warn"
                    } else {
                        "font-mono tabular-nums text-ink-soft"
                    };
                    let value_text = format!(
                        "{} / {} {}",
                        format_hours(week.overall_hours, 2),
                        format_hours(week.expected_hours, 2),
                        props.hours_label,
                    );
                    let week_short = props.week_short.clone();
                    rsx! {
                        button {
                            r#type: "button",
                            class: "{row_class}",
                            onclick: move |_| on_select.call(key),
                            span { class: "text-ink", "{week_short} {iso_week}" }
                            span { class: "{value_class}", "{value_text}" }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct WeekDetailPanelProps {
    week: WorkingHours,
    hours_label: ImStr,
    on_close: EventHandler<()>,
}

#[component]
fn WeekDetailPanel(props: WeekDetailPanelProps) -> Element {
    let i18n = I18N.read().clone();
    let week_short = i18n.t(Key::WeekShort);
    let (_iso_year, iso_week, _) = props.week.from.to_iso_week_date();
    let from_str = i18n.format_date(&props.week.from);
    let to_str = i18n.format_date(&props.week.to);
    let on_close = props.on_close;
    let hours = props.hours_label.clone();
    let summary = format!(
        "{} / {} {hours}",
        format_hours(props.week.overall_hours, 2),
        format_hours(props.week.expected_hours, 2),
    );
    let diff = props.week.overall_hours - props.week.expected_hours;
    let (status_class, status_text) = if diff < 0.0 {
        (
            "text-warn font-semibold",
            format!(
                "−{} {hours} {}",
                format_hours(diff.abs(), 1),
                i18n.t(Key::HoursUnderTarget),
            ),
        )
    } else if diff > 0.0 {
        (
            "text-good font-semibold",
            format!(
                "+{} {hours} {}",
                format_hours(diff, 1),
                i18n.t(Key::HoursOverTarget),
            ),
        )
    } else {
        (
            "text-good font-semibold",
            i18n.t(Key::TargetReached).to_string(),
        )
    };
    rsx! {
        section { class: "mt-3 rounded-md border border-border bg-surface-alt px-3 py-2 flex flex-col gap-2",
            div { class: "flex items-baseline justify-between gap-2",
                div { class: "flex flex-col",
                    h4 { class: "text-sm font-semibold text-ink",
                        "{week_short} {iso_week} · {from_str} – {to_str}"
                    }
                    span { class: "font-mono tabular-nums text-xs text-ink-muted",
                        "{summary}"
                    }
                }
                button {
                    class: "w-6 h-6 inline-flex items-center justify-center rounded-md text-ink-muted hover:bg-surface hover:text-ink",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }
            if !props.week.days.is_empty() {
                ul { class: "flex flex-col",
                    for day in props.week.days.iter() {
                        li { class: "flex items-baseline justify-between gap-2 py-1 border-b border-border text-sm",
                            span { class: "font-mono text-ink", {i18n.format_date(&day.date)} }
                            span { class: "text-ink-muted",
                                {i18n.t(day.category.to_i18n_key())}
                            }
                            span { class: "font-mono tabular-nums text-ink",
                                {format!("{} {hours}", format_hours(day.hours, 2))}
                            }
                        }
                    }
                }
            }
            div { class: "pt-1 text-xs",
                span { class: "{status_class}", "{status_text}" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ExtraHoursViewProps {
    pub extra_hours: Rc<[ExtraHours]>,
    pub custom_hours: Rc<[CustomExtraHours]>,
    pub custom_extra_hours_definitions: Rc<[CustomExtraHoursDefinition]>,
    pub ondelete: EventHandler<Uuid>,
    pub on_custom_delete: EventHandler<Uuid>,
}

#[component]
pub fn ExtraHoursView(props: ExtraHoursViewProps) -> Element {
    let i18n = I18N.read().clone();
    let extra_work_str = i18n.t(Key::CategoryExtraWork);
    let vacation_str = i18n.t(Key::CategoryVacation);
    let sick_leave_str = i18n.t(Key::CategorySickLeave);
    let holidays_str = i18n.t(Key::CategoryHolidays);
    let unavailable_str = i18n.t(Key::CategoryUnavailable);
    let unpaid_leave_str = i18n.t(Key::CategoryUnpaidLeave);
    let volunteer_work_str = i18n.t(Key::CategoryVolunteerWork);
    let hours_str: ImStr = ImStr::from(i18n.t(Key::Hours).as_ref());
    let work_hours_description_str = i18n.t(Key::WorkHoursDescription);
    let unavailable_description_str = i18n.t(Key::UnavailableDescription);

    let category_predicates: [(Rc<str>, Option<Rc<str>>, Box<dyn Fn(&ExtraHours) -> bool>); 7] = [
        (
            vacation_str,
            None,
            Box::new(|eh: &ExtraHours| eh.category.is_vacation()),
        ),
        (
            holidays_str,
            None,
            Box::new(|eh: &ExtraHours| eh.category.is_holiday()),
        ),
        (
            sick_leave_str,
            None,
            Box::new(|eh: &ExtraHours| eh.category.is_sick_leave()),
        ),
        (
            extra_work_str,
            Some(work_hours_description_str),
            Box::new(|eh: &ExtraHours| eh.category.is_extra_work()),
        ),
        (
            unavailable_str,
            Some(unavailable_description_str),
            Box::new(|eh: &ExtraHours| eh.category.is_unavailable()),
        ),
        (
            unpaid_leave_str,
            None,
            Box::new(|eh: &ExtraHours| eh.category.is_unpaid_leave()),
        ),
        (
            volunteer_work_str,
            None,
            Box::new(|eh: &ExtraHours| eh.category.is_volunteer_work()),
        ),
    ];

    rsx! {
        div { class: "flex flex-col gap-1",
            for (label, description, predicate) in category_predicates.into_iter() {
                {
                    let entries: Vec<&ExtraHours> = props
                        .extra_hours
                        .iter()
                        .filter(|eh| predicate(eh))
                        .collect();
                    if entries.is_empty() {
                        rsx! {}
                    } else {
                        rsx! {
                            ExtraHoursCategorySection {
                                label: label.clone(),
                                description: description.clone(),
                                entries: entries.iter().map(|e| (*e).clone()).collect(),
                                hours_label: hours_str.clone(),
                                ondelete: props.ondelete,
                            }
                        }
                    }
                }
            }
            for custom_category in props.custom_hours.iter() {
                {
                    let entries: Vec<ExtraHours> = props
                        .extra_hours
                        .iter()
                        .filter(|eh| eh.category.is_custom_with_id(custom_category.id))
                        .cloned()
                        .collect();
                    let description = props
                        .custom_extra_hours_definitions
                        .iter()
                        .find(|def| def.id == custom_category.id)
                        .and_then(|def| def.description.clone());
                    if entries.is_empty() {
                        rsx! {}
                    } else {
                        rsx! {
                            ExtraHoursCategorySection {
                                label: custom_category.name.clone(),
                                description,
                                entries: entries.into(),
                                hours_label: hours_str.clone(),
                                ondelete: props.ondelete,
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ExtraHoursCategorySectionProps {
    label: Rc<str>,
    description: Option<Rc<str>>,
    entries: Rc<[ExtraHours]>,
    hours_label: ImStr,
    ondelete: EventHandler<Uuid>,
}

#[component]
fn ExtraHoursCategorySection(props: ExtraHoursCategorySectionProps) -> Element {
    let i18n = I18N.read().clone();
    let label = props.label.clone();
    let description = props.description.clone();
    let entries = props.entries.clone();
    let hours_label = props.hours_label;
    let ondelete = props.ondelete;
    rsx! {
        div { class: "flex flex-col mt-3",
            h3 { class: "text-xs uppercase tracking-wide font-semibold text-ink-muted",
                "{label}"
            }
            if let Some(desc) = description {
                p { class: "text-xs text-ink-muted mb-2", "{desc}" }
            }
            for entry in entries.iter() {
                {
                    let entry_id = entry.id;
                    let date = i18n.format_date(&entry.date_time.date());
                    let amount = format!("{} {}", format_hours(entry.amount, 2), hours_label);
                    let entry_description = entry.description.clone();
                    rsx! {
                        div { class: "flex items-baseline justify-between gap-2 py-1.5 border-b border-border",
                            div { class: "min-w-0 flex flex-col",
                                span { class: "text-sm text-ink", "{date}" }
                                if !entry_description.is_empty() {
                                    span { class: "text-xs text-ink-muted truncate", "{entry_description}" }
                                }
                            }
                            div { class: "flex items-center gap-2",
                                span { class: "font-mono tabular-nums text-sm text-ink",
                                    "{amount}"
                                }
                                Btn {
                                    variant: BtnVariant::Danger,
                                    on_click: move |_| ondelete.call(entry_id),
                                    "🗑"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct EmployeeViewProps {
    pub show_delete_employee_work_details: bool,
    pub show_vacation: bool,
    pub onupdate: EventHandler<()>,
    pub on_extra_hour_delete: EventHandler<Uuid>,
    pub on_custom_delete: EventHandler<Uuid>,
    pub on_add_employee_work_details: Option<EventHandler<()>>,
    pub on_employee_work_details_clicked: EventHandler<Uuid>,
    pub on_delete_employee_work_details_clicked: Option<EventHandler<Uuid>>,
    #[props(!optional, default = None)]
    pub on_open_extra_hours: Option<EventHandler<()>>,
}

#[component]
pub fn EmployeeView(props: EmployeeViewProps) -> Element {
    let employee_store = EMPLOYEE_STORE.read();
    let employee = employee_store.employee.clone();
    let extra_hours = employee_store.extra_hours.clone();
    let employee_work_details_list = EMPLOYEE_WORK_DETAILS_STORE
        .read()
        .employee_work_details
        .clone();
    let employee_service = use_coroutine_handle::<EmployeeAction>();
    let year = employee_store.year;
    let full_year = employee_store.until_week >= time::util::weeks_in_year(year as i32);
    let custom_hours = employee_store.employee.custom_extra_hours.clone();
    let custom_extra_hours_definitions = employee_store.custom_extra_hours_definitions.clone();

    rsx! {
        EmployeeViewPlain {
            employee,
            extra_hours,
            year,
            employee_work_details_list,
            full_year,
            show_vacation: props.show_vacation,
            show_delete_employee_work_details: props.show_delete_employee_work_details,
            custom_hours,
            custom_extra_hours_definitions,
            onupdate: props.onupdate,
            on_extra_hour_delete: props.on_extra_hour_delete,
            on_custom_delete: move |uuid| {
                employee_service.send(EmployeeAction::DeleteCustomExtraHour(uuid));
                props.on_custom_delete.call(uuid);
            },
            on_full_year: move |_| {
                employee_service.send(EmployeeAction::FullYear);
            },
            on_until_now: move |_| {
                employee_service.send(EmployeeAction::UntilNow);
            },
            on_add_employee_work_details: props.on_add_employee_work_details,
            on_employee_work_details_clicked: props.on_employee_work_details_clicked,
            on_delete_employee_work_details_clicked: props.on_delete_employee_work_details_clicked,
            on_next_year: move |_| {
                employee_service.send(EmployeeAction::NextYear);
            },
            on_previous_year: move |_| {
                employee_service.send(EmployeeAction::PrevYear);
            },
            on_open_extra_hours: props.on_open_extra_hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn make_details(from: time::Date, expected: f32) -> EmployeeWorkDetails {
        EmployeeWorkDetails {
            id: Uuid::nil(),
            sales_person_id: Uuid::nil(),
            expected_hours: expected,
            from,
            to: from,
            workdays_per_week: 5,
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
            sunday: false,
            dynamic: false,
            cap_planned_hours_to_expected: false,
            vacation_days: 0,
            created: None,
            deleted: None,
            version: Uuid::nil(),
        }
    }

    #[test]
    fn most_recent_expected_picks_latest_from() {
        let a = make_details(date!(2025 - 01 - 01), 20.0);
        let b = make_details(date!(2026 - 03 - 01), 35.0);
        let arr = vec![a, b];
        assert_eq!(most_recent_expected(&arr), 35.0);
    }

    #[test]
    fn most_recent_expected_zero_when_empty() {
        assert_eq!(most_recent_expected(&[]), 0.0);
    }

    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("employee_view.rs");
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
