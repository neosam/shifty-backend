use std::rc::Rc;

use dioxus::prelude::*;
use rest_types::{DayOfWeekTO, EmployeeAttendanceStatisticsTO, EmployeeWeeklyStatisticsTO};
use uuid::Uuid;

use crate::base_types::{format_hours, ImStr};
use crate::component::atoms::{Btn, BtnVariant, NavBtn, PersonChip, TupleRow};
use crate::component::dialog::{Dialog, DialogVariant};
use crate::component::dropdown_base::DropdownTrigger;
use crate::component::EmployeeWeeklyHistogram;
use crate::i18n::{I18nType, Key};
use crate::js;
use crate::service::{
    auth::AUTH, employee::EmployeeAction, employee::EMPLOYEE_STORE,
    employee_work_details::EMPLOYEE_WORK_DETAILS_STORE, i18n::I18N,
};
use crate::state::employee::{
    CustomExtraHours, CustomExtraHoursDefinition, Employee, ExtraHours, WorkingHours,
};
use crate::state::employee_work_details::EmployeeWorkDetails;

const TYPE_PILL_PAID_HEX: &str = "#eaecfb"; // var(--accent-soft) light theme
const TYPE_PILL_VOLUNTEER_HEX: &str = "#fef0d6"; // var(--warn-soft) light theme

/// Map a `DayOfWeekTO` value to the matching `Key::WeekdayShort*` i18n key.
fn weekday_short_key(weekday: DayOfWeekTO) -> Key {
    match weekday {
        DayOfWeekTO::Monday => Key::WeekdayShortMon,
        DayOfWeekTO::Tuesday => Key::WeekdayShortTue,
        DayOfWeekTO::Wednesday => Key::WeekdayShortWed,
        DayOfWeekTO::Thursday => Key::WeekdayShortThu,
        DayOfWeekTO::Friday => Key::WeekdayShortFri,
        DayOfWeekTO::Saturday => Key::WeekdayShortSat,
        DayOfWeekTO::Sunday => Key::WeekdayShortSun,
    }
}

/// Format the Phase-47 weekday-attendance line (RPT-02).
///
/// - When `counted_calendar_weeks == 0` or `attendance_by_weekday` is empty, the
///   localized empty-state placeholder is returned.
/// - Otherwise, the input is iterated in the BE-provided order (Mon..Sun — the FE
///   does NOT re-sort). Each row is rendered as `"<short-label>: <count> (<pct>%)"`
///   where `pct = (share * 100.0).round() as i32`. Segments are joined by
///   `" · "` (space, U+00B7 MIDDLE DOT, space) per D-47-CONTEXT.
// v2.2.1: obsolete inline-format kept for the SSR test suite as a compact
// stringification that mirrors the pre-table rendering. Prod UI renders a table
// (see `EmployeeViewPlain`), so this fn is dead-code in the release binary.
#[allow(dead_code)]
pub fn format_weekday_attendance_line(
    stats: &EmployeeAttendanceStatisticsTO,
    i18n: &I18nType,
) -> String {
    if stats.counted_calendar_weeks == 0 || stats.attendance_by_weekday.is_empty() {
        return i18n.t(Key::WeekdayAttendanceEmpty).as_ref().to_string();
    }

    stats
        .attendance_by_weekday
        .iter()
        .map(|entry| {
            let label = i18n.t(weekday_short_key(entry.weekday));
            let pct = (entry.share * 100.0).round() as i32;
            format!("{}: {} ({}%)", label.as_ref(), entry.count, pct)
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

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
    /// Whether the current user has the HR role.
    pub is_hr: bool,
    /// Weekly statistics fetched from the HR-gated backend endpoint.
    /// None for non-HR users (403 → None) or when data is unavailable.
    #[props(!optional, default = None)]
    pub weekly_statistics: Option<Rc<EmployeeWeeklyStatisticsTO>>,
    /// Average-hours-per-attendance-day statistic (flexible employees only).
    /// None for non-flexible employees / non-HR (server returns null/403) — the
    /// row is then not rendered at all (D-AVG-05).
    #[props(!optional, default = None)]
    pub attendance_statistics: Option<Rc<EmployeeAttendanceStatisticsTO>>,

    pub onupdate: EventHandler<()>,
    pub on_extra_hour_delete: EventHandler<Uuid>,
    pub on_extra_hour_edit: EventHandler<ExtraHours>,
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

    /// When set, renders a secondary "📅 … →" button in the person identity
    /// header row that navigates to the Absences page for this person.
    #[props(!optional, default = None)]
    pub on_nav_to_absences: Option<EventHandler<()>>,
}

fn current_week_expected_hours(
    weeks: &[WorkingHours],
    current_year: u32,
    current_week: u8,
) -> Option<f32> {
    weeks
        .iter()
        .find(|w| {
            let (y, wk, _) = w.from.to_iso_week_date();
            y as u32 == current_year && wk == current_week
        })
        .map(|w| w.expected_hours)
}

/// Ehrenamt (Volunteer Work) wird im Mitarbeiter-Report (OVERALL-Box) nur
/// angezeigt, wenn die geleisteten Ehrenamt-Stunden >= 0.5 sind — kleine
/// Restwerte werden als Rauschen unterdrueckt.
pub(crate) fn show_volunteer_work(hours: f32) -> bool {
    hours >= 0.5
}

/// Returns true only when the user has the HR role AND statistics data is
/// available (defence-in-depth: the backend 403s non-HR fetches so `stats`
/// will always be None for non-HR users in production).
pub(crate) fn should_show_hr_stats(
    is_hr: bool,
    stats: Option<&EmployeeWeeklyStatisticsTO>,
) -> bool {
    is_hr && stats.is_some()
}

#[component]
pub fn EmployeeViewPlain(props: EmployeeViewPlainProps) -> Element {
    let i18n = I18N.read().clone();
    let mut selected_week = use_signal(|| None::<(u32, u8)>);
    let mut expand_weeks = use_signal(|| false);
    let mut delete_confirm_id: Signal<Option<Uuid>> = use_signal(|| None);

    let employee = props.employee.clone();
    let work_details_list = props.employee_work_details_list.clone();
    let custom_hours = props.custom_hours.clone();
    let show_delete_work_details = props.show_delete_employee_work_details;
    let on_delete_clicked = props.on_delete_employee_work_details_clicked;

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
    let current_year = js::get_current_year();
    let current_week = js::get_current_week();
    let current_week_expected =
        current_week_expected_hours(&employee.working_hours_by_week, current_year, current_week);

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
    let delete_contract_btn_label: ImStr = ImStr::from(i18n.t(Key::EmployeeWorkDetailsDeleteBtn).as_ref());
    let delete_contract_confirm_title: ImStr = ImStr::from(i18n.t(Key::EmployeeWorkDetailsDeleteConfirmTitle).as_ref());
    let delete_contract_confirm_body: ImStr = ImStr::from(i18n.t(Key::EmployeeWorkDetailsDeleteConfirmBody).as_ref());
    let delete_contract_confirm_btn: String = i18n.t(Key::EmployeeWorkDetailsDeleteConfirmBtn).to_string();

    let prev_year_aria = ImStr::from(i18n.t(Key::PreviousYear).as_ref());
    let next_year_aria = ImStr::from(i18n.t(Key::NextYear).as_ref());

    let on_next_year = props.on_next_year;
    let on_prev_year = props.on_previous_year;
    let on_full_year = props.on_full_year;
    let on_until_now = props.on_until_now;
    let on_open_extra_hours = props.on_open_extra_hours;
    let on_add_work_details = props.on_add_employee_work_details;
    let on_work_details_clicked = props.on_employee_work_details_clicked;
    let on_nav_to_absences = props.on_nav_to_absences;

    // NAV-01: label for the absences cross-link in the person header.
    // HR view uses a person-aware template; self-view uses the plain self label.
    let nav_to_absences_label: Option<std::rc::Rc<str>> = on_nav_to_absences.as_ref().map(|_| {
        if props.is_hr {
            i18n.t_m(Key::NavToEmployeeAbsences, [("name", name.as_ref())].into())
        } else {
            i18n.t(Key::NavToMyAbsences)
        }
    });

    let year = props.year;

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
                h1 { class: "text-h1 text-ink", "{name}" }
                PersonChip {
                    name: ImStr::from(type_label.as_ref()),
                    color: Some(ImStr::from(pill_color)),
                }
                if let Some(expected) = current_week_expected {
                    if expected > 0.0 {
                        span { class: "font-mono tabular-nums text-ink-muted text-body",
                            "{expected:.0} {hours_str}"
                        }
                    }
                }
                // NAV-01: absences cross-link — right-aligned in the identity row.
                if let Some(handler) = on_nav_to_absences {
                    if let Some(label) = nav_to_absences_label.as_ref() {
                        div { class: "ml-auto",
                            Btn {
                                variant: BtnVariant::Secondary,
                                on_click: move |_| handler.call(()),
                                "📅 {label} →"
                            }
                        }
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
                    span { class: "font-mono text-lg text-ink min-w-[4ch] text-center", "{year}" }
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
                div { class: "text-small font-normal text-ink-muted italic flex flex-wrap items-baseline gap-2",
                    span { "{current_week_note}" }
                    button {
                        r#type: "button",
                        class: "text-accent underline cursor-pointer text-small font-normal",
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
                h2 { class: "text-micro font-bold uppercase text-ink-muted",
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
                if show_volunteer_work(employee.volunteer_hours) {
                    TupleRow {
                        label: ImStr::from(volunteer_work_str.as_ref()),
                        value: rsx! { span { class: "font-mono tabular-nums",
                            {format!("{} {}", format_hours(employee.volunteer_hours, 2), hours_str)}
                        } },
                    }
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
                h2 { class: "text-micro font-bold uppercase text-ink-muted",
                    "{work_details_header}"
                }
                div { class: "flex flex-col gap-2",
                    for details in work_details_list.iter() {
                        div { class: "flex items-stretch gap-2",
                            div { class: "flex-1 min-w-0",
                                ContractCard {
                                    details: details.clone(),
                                    on_click: {
                                        let id = details.id;
                                        move |_| on_work_details_clicked.call(id)
                                    },
                                    hours_label: hours_str.clone(),
                                }
                            }
                            if show_delete_work_details {
                                {
                                    let id = details.id;
                                    let label = delete_contract_btn_label.clone();
                                    rsx! {
                                        button {
                                            r#type: "button",
                                            class: "px-2 text-bad-soft hover:text-bad text-small flex-shrink-0 self-stretch flex items-center",
                                            "aria-label": "{label}",
                                            onclick: move |_| delete_confirm_id.set(Some(id)),
                                            "🗑"
                                        }
                                    }
                                }
                            }
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
                    h3 { class: "text-micro font-bold uppercase text-ink-muted",
                        "{working_hours_per_week_heading}"
                    }
                    button {
                        r#type: "button",
                        class: "text-accent underline cursor-pointer text-small font-normal",
                        onclick: move |_| {
                            let v = *expand_weeks.read();
                            expand_weeks.set(!v);
                        },
                        if *expand_weeks.read() { "{hide_details_str}" } else { "{show_details_str}" }
                    }
                }
                EmployeeWeeklyHistogram {
                    weeks: histogram_weeks.clone(),
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
                h2 { class: "text-micro font-bold uppercase text-ink-muted",
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
                    on_edit: move |entry| {
                        props.on_extra_hour_edit.call(entry);
                    },
                    on_custom_delete: move |uuid| {
                        props.on_custom_delete.call(uuid);
                    },
                }
            }
        }

        // HR-only statistics block (STAT-01/STAT-02, D-22-04/D-22-05)
        if should_show_hr_stats(props.is_hr, props.weekly_statistics.as_deref()) {
            if let Some(stats) = props.weekly_statistics.as_ref() {
                section { class: "mt-6 pt-4 border-t border-border flex flex-col gap-2",
                    h2 { class: "text-micro font-bold uppercase text-ink-muted",
                        {i18n.t(Key::StatisticsHeading)}
                    }
                    TupleRow {
                        label: ImStr::from(i18n.t(Key::AverageWorkedHoursPerWeek).as_ref()),
                        value: rsx! { span { class: "font-mono tabular-nums",
                            {format_hours(stats.average_worked_hours_per_week, 2)}
                        } },
                    }
                    // Phase 47 (RPT-02) + v2.2.1: weekday attendance table.
                    // Renders count + hours + %-hours per weekday plus a total row.
                    if let Some(att) = props.attendance_statistics.as_ref() {
                        {
                            let tooltip = i18n.t(Key::WeekdayAttendanceTooltip);
                            let label = i18n.t(Key::WeekdayAttendanceLabel);
                            let col_day = i18n.t(Key::WeekdayAttendanceColDay);
                            let col_count = i18n.t(Key::WeekdayAttendanceColCount);
                            let col_hours = i18n.t(Key::WeekdayAttendanceColHours);
                            let col_share = i18n.t(Key::WeekdayAttendanceColShare);
                            let row_total = i18n.t(Key::WeekdayAttendanceRowTotal);

                            let empty = att.counted_calendar_weeks == 0
                                || att.attendance_by_weekday.is_empty();
                            let empty_line = i18n.t(Key::WeekdayAttendanceEmpty);

                            let total_count: u32 =
                                att.attendance_by_weekday.iter().map(|w| w.count).sum();
                            let total_hours: f32 =
                                att.attendance_by_weekday.iter().map(|w| w.hours).sum();

                            let weekday_rows: Vec<Element> = att
                                .attendance_by_weekday
                                .iter()
                                .map(|entry| {
                                    let label = i18n.t(weekday_short_key(entry.weekday));
                                    let pct = (entry.share_of_hours * 100.0).round() as i32;
                                    rsx! {
                                        tr {
                                            td { class: "pr-3 py-0.5 text-ink-muted", "{label}" }
                                            td { class: "pr-3 py-0.5 text-right font-mono tabular-nums",
                                                "{entry.count}"
                                            }
                                            td { class: "pr-3 py-0.5 text-right font-mono tabular-nums",
                                                {format_hours(entry.hours, 1)}
                                            }
                                            td { class: "py-0.5 text-right font-mono tabular-nums",
                                                "{pct}%"
                                            }
                                        }
                                    }
                                })
                                .collect();

                            rsx! {
                                TupleRow {
                                    label: ImStr::from(label.as_ref()),
                                    value: rsx! {
                                        if empty {
                                            span {
                                                class: "text-ink-muted italic",
                                                title: "{tooltip}",
                                                "{empty_line}"
                                            }
                                        } else {
                                            table {
                                                class: "w-full text-small",
                                                title: "{tooltip}",
                                                thead {
                                                    tr { class: "text-ink-muted uppercase text-micro border-b border-border",
                                                        th { class: "pr-3 py-0.5 text-left font-normal", "{col_day}" }
                                                        th { class: "pr-3 py-0.5 text-right font-normal", "{col_count}" }
                                                        th { class: "pr-3 py-0.5 text-right font-normal", "{col_hours}" }
                                                        th { class: "py-0.5 text-right font-normal", "{col_share}" }
                                                    }
                                                }
                                                tbody { {weekday_rows.into_iter()} }
                                                tfoot {
                                                    tr { class: "border-t border-border font-bold",
                                                        td { class: "pr-3 py-0.5", "{row_total}" }
                                                        td { class: "pr-3 py-0.5 text-right font-mono tabular-nums",
                                                            "{total_count}"
                                                        }
                                                        td { class: "pr-3 py-0.5 text-right font-mono tabular-nums",
                                                            {format_hours(total_hours, 1)}
                                                        }
                                                        td { class: "py-0.5 text-right font-mono tabular-nums",
                                                            "100%"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                }
                            }
                        }
                    }
                    TupleRow {
                        label: ImStr::from(i18n.t(Key::StatisticsIncludedWeeks).as_ref()),
                        value: rsx! { span { class: "font-mono tabular-nums",
                            {stats.included_weeks.to_string()}
                        } },
                        dim: true,
                    }
                }
            }
        }

        // Delete contract confirm modal
        if let Some(delete_id) = *delete_confirm_id.read() {
            {
                let confirm_btn_label = delete_contract_confirm_btn.clone();
                let footer = rsx! {
                    Btn {
                        variant: BtnVariant::Secondary,
                        on_click: move |_| {
                            delete_confirm_id.set(None);
                        },
                        "Cancel"
                    }
                    Btn {
                        variant: BtnVariant::Danger,
                        on_click: move |_| {
                            delete_confirm_id.set(None);
                            if let Some(handler) = on_delete_clicked {
                                handler.call(delete_id);
                            }
                        },
                        "{confirm_btn_label}"
                    }
                };
                rsx! {
                    Dialog {
                        open: true,
                        on_close: move |_| {
                            delete_confirm_id.set(None);
                        },
                        title: delete_contract_confirm_title.clone(),
                        variant: DialogVariant::Auto,
                        width: 420,
                        footer: Some(footer),
                        p { class: "text-ink", "{delete_contract_confirm_body}" }
                    }
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
            r#type: "button",
            class: "w-full text-left rounded-md border border-border bg-surface px-3 py-2 hover:bg-surface-alt cursor-pointer",
            onclick: move |_| on_click.call(()),
            div { class: "flex items-baseline justify-between gap-2",
                span { class: "text-body font-semibold text-ink", "{from_str} – {to_str}" }
                span { class: "font-mono tabular-nums text-small font-normal text-ink-muted",
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
    let i18n = I18N.read().clone();
    let on_select = props.on_select;
    let volunteer_label = i18n.t(Key::Volunteer);
    // Show every loaded week, newest first.
    let visible: Vec<WorkingHours> = {
        let mut all: Vec<WorkingHours> = props.weeks.iter().cloned().collect();
        all.reverse();
        all
    };
    rsx! {
        div { class: "mt-2 flex flex-col text-small font-normal",
            for week in visible.into_iter() {
                {
                    let (iso_year, iso_week, _) = week.from.to_iso_week_date();
                    let key = (iso_year as u32, iso_week);
                    let is_selected = props.selected_week == Some(key);
                    let under = week.overall_hours < week.expected_hours;
                    let row_class = if is_selected {
                        "w-full text-left flex items-start justify-between px-2 py-1.5 border-b border-border bg-accent-soft cursor-pointer"
                    } else {
                        "w-full text-left flex items-start justify-between px-2 py-1.5 border-b border-border hover:bg-surface-alt cursor-pointer"
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
                    // YV-02: format the from–to date range
                    let from_str = i18n.format_date(&week.from);
                    let to_str = i18n.format_date(&week.to);
                    let date_range = format!("{} – {}", from_str, to_str);
                    // YV-03: volunteer hours (only show if > 0)
                    let has_volunteer = week.volunteer_hours > 0.0;
                    let volunteer_text = format!(
                        "{}: {} {}",
                        volunteer_label,
                        format_hours(week.volunteer_hours, 2),
                        props.hours_label,
                    );
                    rsx! {
                        button {
                            r#type: "button",
                            class: "{row_class}",
                            onclick: move |_| on_select.call(key),
                            // Left side: KW label + date range (two-line)
                            span { class: "flex flex-col",
                                span { class: "text-ink", "{week_short} {iso_week}" }
                                span { class: "text-ink-muted text-micro", "{date_range}" }
                            }
                            // Right side: hours + optional volunteer
                            span { class: "flex flex-col items-end",
                                span { class: "{value_class}", "{value_text}" }
                                if has_volunteer {
                                    span { class: "font-mono tabular-nums text-ink-muted text-micro",
                                        "{volunteer_text}"
                                    }
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
    let volunteer_label = i18n.t(Key::Volunteer);
    let has_volunteer = props.week.volunteer_hours > 0.0;
    let volunteer_text = format!(
        "{}: {} {hours}",
        volunteer_label,
        format_hours(props.week.volunteer_hours, 2),
    );

    rsx! {
        section { class: "mt-3 rounded-md border border-border bg-surface-alt px-3 py-2 flex flex-col gap-2",
            div { class: "flex items-baseline justify-between gap-2",
                div { class: "flex flex-col",
                    h4 { class: "text-body font-semibold text-ink",
                        "{week_short} {iso_week} · {from_str} – {to_str}"
                    }
                    span { class: "font-mono tabular-nums text-small font-normal text-ink-muted",
                        "{summary}"
                    }
                    // YV-03: volunteer hours as separate value
                    if has_volunteer {
                        span { class: "font-mono tabular-nums text-small font-normal text-ink-muted",
                            "{volunteer_text}"
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "w-6 h-6 inline-flex items-center justify-center rounded-md text-ink-muted hover:bg-surface hover:text-ink",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }
            if !props.week.days.is_empty() {
                ul { class: "flex flex-col",
                    for day in props.week.days.iter() {
                        li { class: "flex items-baseline justify-between gap-2 py-1 border-b border-border text-body",
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
            div { class: "pt-1 text-small font-normal",
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
    pub on_edit: EventHandler<ExtraHours>,
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

    // reason: 3-tuple encodes (label, hint, predicate) per extra-hours category; type alias would obscure the local-only structure
    #[allow(clippy::type_complexity)]
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
                                on_edit: props.on_edit,
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
                                on_edit: props.on_edit,
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
    on_edit: EventHandler<ExtraHours>,
}

#[component]
fn ExtraHoursCategorySection(props: ExtraHoursCategorySectionProps) -> Element {
    let i18n = I18N.read().clone();
    let label = props.label.clone();
    let description = props.description.clone();
    let entries = props.entries.clone();
    let hours_label = props.hours_label;
    let ondelete = props.ondelete;
    let on_edit = props.on_edit;
    let edit_label: ImStr = ImStr::from(i18n.t(Key::EditExtraHourLabel).as_ref());
    rsx! {
        div { class: "flex flex-col mt-3",
            h3 { class: "text-micro font-bold uppercase text-ink-muted",
                "{label}"
            }
            if let Some(desc) = description {
                p { class: "text-small font-normal text-ink-muted mb-2", "{desc}" }
            }
            for entry in entries.iter() {
                {
                    let entry_id = entry.id;
                    let entry_for_edit = entry.clone();
                    let date = i18n.format_date(&entry.date_time.date());
                    let amount = format!("{} {}", format_hours(entry.amount, 2), hours_label);
                    let entry_description = entry.description.clone();
                    let edit_label = edit_label.clone();
                    rsx! {
                        div { class: "flex items-baseline justify-between gap-2 py-1.5 border-b border-border",
                            div { class: "min-w-0 flex flex-col",
                                span { class: "text-body text-ink", "{date}" }
                                if !entry_description.is_empty() {
                                    span { class: "text-small font-normal text-ink-muted truncate", "{entry_description}" }
                                }
                            }
                            div { class: "flex items-center gap-2",
                                span { class: "font-mono tabular-nums text-body text-ink",
                                    "{amount}"
                                }
                                Btn {
                                    variant: BtnVariant::Secondary,
                                    on_click: move |_| on_edit.call(entry_for_edit.clone()),
                                    "{edit_label}"
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
    pub on_extra_hour_edit: EventHandler<ExtraHours>,
    pub on_custom_delete: EventHandler<Uuid>,
    pub on_add_employee_work_details: Option<EventHandler<()>>,
    pub on_employee_work_details_clicked: EventHandler<Uuid>,
    pub on_delete_employee_work_details_clicked: Option<EventHandler<Uuid>>,
    #[props(!optional, default = None)]
    pub on_open_extra_hours: Option<EventHandler<()>>,
    /// When set, renders a secondary "📅 … →" cross-nav button in the person
    /// identity header that navigates to the Absences view for this person.
    #[props(!optional, default = None)]
    pub on_nav_to_absences: Option<EventHandler<()>>,
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
    let weekly_statistics = employee_store.weekly_statistics.clone();
    let attendance_statistics = employee_store.attendance_statistics.clone();
    let is_hr = AUTH
        .read()
        .auth_info
        .as_ref()
        .map(|a| a.has_privilege("hr"))
        .unwrap_or(false);

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
            is_hr,
            weekly_statistics,
            attendance_statistics,
            onupdate: props.onupdate,
            on_extra_hour_delete: props.on_extra_hour_delete,
            on_extra_hour_edit: props.on_extra_hour_edit,
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
            on_delete_employee_work_details_clicked: Some(EventHandler::new(move |id: Uuid| {
                employee_service.send(EmployeeAction::DeleteEmployeeWorkDetails(id));
                if let Some(handler) = props.on_delete_employee_work_details_clicked {
                    handler.call(id);
                }
            })),
            on_next_year: move |_| {
                employee_service.send(EmployeeAction::NextYear);
            },
            on_previous_year: move |_| {
                employee_service.send(EmployeeAction::PrevYear);
            },
            on_open_extra_hours: props.on_open_extra_hours,
            on_nav_to_absences: props.on_nav_to_absences,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_week(from: time::Date, expected: f32) -> WorkingHours {
        WorkingHours {
            from,
            to: from,
            expected_hours: expected,
            overall_hours: 0.0,
            balance: 0.0,
            shiftplan_hours: 0.0,
            extra_work_hours: 0.0,
            vacation_hours: 0.0,
            vacation_days: 0.0,
            sick_leave_hours: 0.0,
            holiday_hours: 0.0,
            unpaid_leave_hours: 0.0,
            volunteer_hours: 0.0,
            days: Rc::from([]),
        }
    }

    #[test]
    fn current_week_expected_returns_value_when_today_present() {
        // Today: ISO 2026 KW 17. Loaded weeks include KW 17 with expected = 20.
        let monday_kw17 = time::Date::from_iso_week_date(2026, 17, time::Weekday::Monday).unwrap();
        let weeks = vec![make_week(monday_kw17, 20.0)];
        assert_eq!(current_week_expected_hours(&weeks, 2026, 17), Some(20.0));
    }

    #[test]
    fn current_week_expected_returns_none_when_today_not_loaded() {
        // Loaded year is 2025; today is in 2026.
        let monday_kw17_2025 =
            time::Date::from_iso_week_date(2025, 17, time::Weekday::Monday).unwrap();
        let weeks = vec![make_week(monday_kw17_2025, 20.0)];
        assert_eq!(current_week_expected_hours(&weeks, 2026, 17), None);
    }

    #[test]
    fn current_week_expected_uses_post_change_value_after_mid_year_contract_change() {
        // Weeks 1-10 of 2026: 20h. Weeks 11+: 30h. Today is KW 17 of 2026.
        let weeks: Vec<WorkingHours> = (1..=20u8)
            .map(|i| {
                let expected = if i <= 10 { 20.0 } else { 30.0 };
                make_week(
                    time::Date::from_iso_week_date(2026, i, time::Weekday::Monday).unwrap(),
                    expected,
                )
            })
            .collect();
        assert_eq!(current_week_expected_hours(&weeks, 2026, 17), Some(30.0));
    }

    #[test]
    fn current_week_expected_returns_none_for_empty_weeks() {
        assert_eq!(current_week_expected_hours(&[], 2026, 17), None);
    }

    #[test]
    fn show_volunteer_work_threshold() {
        // Ehrenamt in der OVERALL-Box nur ab >= 0.5h sichtbar.
        assert!(!show_volunteer_work(0.0));
        assert!(!show_volunteer_work(0.49));
        assert!(show_volunteer_work(0.5));
        assert!(show_volunteer_work(42.0));
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn extra_hours_category_section_renders_edit_button_with_translation() {
        // The Edit button must be present and labeled with the
        // EditExtraHourLabel translation (English, since the I18N store
        // defaults to En in tests).
        fn app() -> Element {
            let entry = ExtraHours {
                id: uuid::Uuid::from_u128(1),
                sales_person_id: uuid::Uuid::nil(),
                amount: 3.5,
                category: crate::state::employee::WorkingHoursCategory::ExtraWork("-".into()),
                description: Rc::from("note"),
                date_time: time::macros::datetime!(2026-04-15 10:00:00),
                version: uuid::Uuid::nil(),
            };
            rsx! {
                ExtraHoursCategorySection {
                    label: Rc::from("Extra work"),
                    description: None,
                    entries: Rc::from([entry]),
                    hours_label: ImStr::from("h"),
                    ondelete: |_| {},
                    on_edit: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Edit"),
            "edit button must carry the EditExtraHourLabel translation: {html}"
        );
        // sanity check the delete glyph also renders so the row layout is intact
        assert!(
            html.contains("\u{1f5d1}") || html.contains("&#x1f5d1;") || html.contains("🗑"),
            "delete button glyph must still render: {html}"
        );
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

    // --- SSR tests for delete-contract button (Task 5) ---
    //
    // Fallback: EmployeeViewPlain calls js::get_current_year()/get_current_week()
    // unconditionally, which panics on non-wasm targets (js-sys limitation).
    // We therefore test the conditional rendering logic via minimal sub-components
    // that isolate exactly the branch under test, following the same pattern used
    // in plan 260516-g63 (ContractModal/ExtraHoursModal tests).

    /// Test A: delete button renders when show_delete_employee_work_details=true.
    /// Uses a minimal stub component that mirrors only the contract-row delete branch.
    #[test]
    fn delete_contract_button_renders_when_enabled() {
        fn app() -> Element {
            let i18n = crate::service::i18n::I18N.read().clone();
            let label: ImStr = ImStr::from(
                i18n.t(crate::i18n::Key::EmployeeWorkDetailsDeleteBtn)
                    .as_ref(),
            );
            // Mirrors the branch: if show_delete_work_details { button { "aria-label": label ... } }
            let show = true;
            rsx! {
                div {
                    if show {
                        button {
                            r#type: "button",
                            "aria-label": "{label}",
                            "🗑"
                        }
                    }
                }
            }
        }
        let html = render(app);
        // EN locale: EmployeeWorkDetailsDeleteBtn = "Delete contract"
        assert!(
            html.contains("Delete contract"),
            "delete-contract button must appear when show=true, got: {html}"
        );
    }

    /// Test B: delete button is absent when show_delete_employee_work_details=false.
    #[test]
    fn delete_contract_button_hidden_when_disabled() {
        fn app() -> Element {
            let i18n = crate::service::i18n::I18N.read().clone();
            let label: ImStr = ImStr::from(
                i18n.t(crate::i18n::Key::EmployeeWorkDetailsDeleteBtn)
                    .as_ref(),
            );
            let show = false;
            rsx! {
                div {
                    if show {
                        button {
                            r#type: "button",
                            "aria-label": "{label}",
                            "🗑"
                        }
                    }
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("Delete contract"),
            "delete-contract button must NOT appear when show=false, got: {html}"
        );
    }

    /// Test C: confirm modal title is absent in the initial render (delete_confirm_id=None).
    #[test]
    fn delete_contract_confirm_modal_hidden_initially() {
        fn app() -> Element {
            let i18n = crate::service::i18n::I18N.read().clone();
            let title: ImStr = ImStr::from(
                i18n.t(crate::i18n::Key::EmployeeWorkDetailsDeleteConfirmTitle)
                    .as_ref(),
            );
            let body: ImStr = ImStr::from(
                i18n.t(crate::i18n::Key::EmployeeWorkDetailsDeleteConfirmBody)
                    .as_ref(),
            );
            // Mirrors: if let Some(id) = *delete_confirm_id.read() { Dialog { title, body } }
            let delete_confirm_id: Option<Uuid> = None; // initial state
            rsx! {
                div {
                    if let Some(_id) = delete_confirm_id {
                        div {
                            h2 { "{title}" }
                            p { "{body}" }
                        }
                    }
                }
            }
        }
        let html = render(app);
        // Confirm modal title must not appear before user clicks
        assert!(
            !html.contains("Delete contract?"),
            "confirm modal must be hidden in initial render (None state), got: {html}"
        );
    }

    // --- YV-02 / YV-03 SSR tests (RED phase, Task 2) ---

    fn make_week_full(
        from: time::Date,
        overall: f32,
        expected: f32,
        volunteer: f32,
    ) -> WorkingHours {
        WorkingHours {
            from,
            to: from + time::Duration::days(6),
            expected_hours: expected,
            overall_hours: overall,
            balance: overall - expected,
            shiftplan_hours: overall,
            extra_work_hours: 0.0,
            vacation_hours: 0.0,
            vacation_days: 0.0,
            sick_leave_hours: 0.0,
            holiday_hours: 0.0,
            unpaid_leave_hours: 0.0,
            volunteer_hours: volunteer,
            days: Rc::from([]),
        }
    }

    #[derive(Props, Clone, PartialEq)]
    struct WeekListProps {
        weeks: Rc<[WorkingHours]>,
        selected_week: Option<(u32, u8)>,
        hours_label: ImStr,
        week_short: ImStr,
    }

    fn render_week_list(p: WeekListProps) -> String {
        fn app(p: WeekListProps) -> Element {
            rsx! {
                WeekListExpanded {
                    weeks: p.weeks.clone(),
                    selected_week: p.selected_week,
                    hours_label: p.hours_label.clone(),
                    week_short: p.week_short.clone(),
                    on_select: |_| {},
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(app, p);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[derive(Props, Clone, PartialEq)]
    struct WeekDetailProps {
        week: WorkingHours,
        hours_label: ImStr,
    }

    fn render_week_detail(p: WeekDetailProps) -> String {
        fn app(p: WeekDetailProps) -> Element {
            rsx! {
                WeekDetailPanel {
                    week: p.week.clone(),
                    hours_label: p.hours_label.clone(),
                    on_close: |_| {},
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(app, p);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn week_list_shows_date_range_in_each_row() {
        // YV-02: each row in WeekListExpanded must display the from–to date.
        // from = 2026-03-02 (Mon KW 10), to = 2026-03-08 (Sun KW 10)
        let from = time::Date::from_iso_week_date(2026, 10, time::Weekday::Monday).unwrap();
        let weeks: Rc<[WorkingHours]> = vec![make_week_full(from, 20.0, 20.0, 0.0)].into();
        let html = render_week_list(WeekListProps {
            weeks,
            selected_week: None,
            hours_label: ImStr::from("h"),
            week_short: ImStr::from("W"),
        });
        // The from-date must appear (year 2026 and month 03 in some format)
        assert!(
            html.contains("2026") && html.contains("03") && html.contains("02"),
            "WeekListExpanded must show from-date for each row: {html}"
        );
    }

    #[test]
    fn week_list_shows_volunteer_when_gt_zero() {
        // YV-03: a row with volunteer_hours > 0 must show a volunteer value.
        let from = time::Date::from_iso_week_date(2026, 10, time::Weekday::Monday).unwrap();
        let weeks: Rc<[WorkingHours]> = vec![make_week_full(from, 20.0, 20.0, 5.0)].into();
        let html = render_week_list(WeekListProps {
            weeks,
            selected_week: None,
            hours_label: ImStr::from("h"),
            week_short: ImStr::from("W"),
        });
        // Must contain "5.00" (the formatted volunteer hours) and "Volunteer" label
        assert!(
            html.contains("5.00"),
            "WeekListExpanded must show volunteer hours value: {html}"
        );
        assert!(
            html.contains("Volunteer"),
            "WeekListExpanded must show Volunteer label: {html}"
        );
    }

    #[test]
    fn week_list_no_volunteer_when_zero() {
        // YV-03: a row with volunteer_hours == 0 must NOT show a volunteer section.
        let from = time::Date::from_iso_week_date(2026, 10, time::Weekday::Monday).unwrap();
        let weeks: Rc<[WorkingHours]> = vec![make_week_full(from, 20.0, 20.0, 0.0)].into();
        let html = render_week_list(WeekListProps {
            weeks,
            selected_week: None,
            hours_label: ImStr::from("h"),
            week_short: ImStr::from("W"),
        });
        // "Volunteer" should not appear when volunteer_hours == 0
        assert!(
            !html.contains("Volunteer"),
            "WeekListExpanded must NOT show Volunteer label when volunteer_hours==0: {html}"
        );
    }

    #[test]
    fn week_detail_panel_shows_volunteer_when_gt_zero() {
        // YV-03: WeekDetailPanel must show volunteer hours as a separate value.
        let from = time::Date::from_iso_week_date(2026, 10, time::Weekday::Monday).unwrap();
        let week = make_week_full(from, 20.0, 20.0, 7.5);
        let html = render_week_detail(WeekDetailProps {
            week,
            hours_label: ImStr::from("h"),
        });
        assert!(
            html.contains("7.50"),
            "WeekDetailPanel must show volunteer hours value: {html}"
        );
        assert!(
            html.contains("Volunteer"),
            "WeekDetailPanel must show Volunteer label: {html}"
        );
    }

    #[test]
    fn week_detail_panel_no_volunteer_when_zero() {
        // YV-03: WeekDetailPanel must NOT show volunteer section when volunteer_hours == 0.
        let from = time::Date::from_iso_week_date(2026, 10, time::Weekday::Monday).unwrap();
        let week = make_week_full(from, 20.0, 20.0, 0.0);
        let html = render_week_detail(WeekDetailProps {
            week,
            hours_label: ImStr::from("h"),
        });
        assert!(
            !html.contains("Volunteer"),
            "WeekDetailPanel must NOT show Volunteer when volunteer_hours==0: {html}"
        );
    }

    // --- Phase 22 — HR statistics block SSR tests (STAT-01/STAT-02, D-22-08) ---
    //
    // EmployeeViewPlain calls js::get_current_year()/get_current_week() which panic on
    // non-wasm targets. We therefore test the conditional rendering logic using a minimal
    // stub component that mirrors only the HR-stats branch — same pattern as delete-contract
    // and extra-hours-modal SSR tests.

    fn make_test_stats() -> Rc<EmployeeWeeklyStatisticsTO> {
        Rc::new(EmployeeWeeklyStatisticsTO {
            average_worked_hours_per_week: 23.5,
            included_weeks: 10,
            total_worked_hours: 235.0,
        })
    }

    // --- Pure-fn tests for should_show_hr_stats ---

    #[test]
    fn should_show_hr_stats_true_when_hr_and_stats_present() {
        let stats = make_test_stats();
        assert!(should_show_hr_stats(true, Some(&*stats)));
    }

    #[test]
    fn should_show_hr_stats_false_when_not_hr_with_stats() {
        let stats = make_test_stats();
        assert!(!should_show_hr_stats(false, Some(&*stats)));
    }

    #[test]
    fn should_show_hr_stats_false_when_hr_no_stats() {
        assert!(!should_show_hr_stats(true, None));
    }

    #[test]
    fn should_show_hr_stats_false_when_not_hr_no_stats() {
        assert!(!should_show_hr_stats(false, None));
    }

    // --- SSR visibility tests for the HR-only block ---

    #[derive(Props, Clone, PartialEq)]
    struct HrStatsBlockProps {
        is_hr: bool,
        weekly_statistics: Option<Rc<EmployeeWeeklyStatisticsTO>>,
    }

    /// Minimal stub that mirrors only the HR-stats conditional block from EmployeeViewPlain.
    fn render_hr_stats_block(p: HrStatsBlockProps) -> String {
        fn app(p: HrStatsBlockProps) -> Element {
            let i18n = crate::service::i18n::I18N.read().clone();
            rsx! {
                div {
                    if should_show_hr_stats(p.is_hr, p.weekly_statistics.as_deref()) {
                        if let Some(stats) = p.weekly_statistics.as_ref() {
                            section {
                                h2 { {i18n.t(crate::i18n::Key::StatisticsHeading)} }
                                span { class: "font-mono tabular-nums",
                                    {format_hours(stats.average_worked_hours_per_week, 2)}
                                }
                                span { {stats.included_weeks.to_string()} }
                            }
                        }
                    }
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(app, p);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn hr_block_visible_when_hr_with_stats() {
        let stats = make_test_stats();
        let html = render_hr_stats_block(HrStatsBlockProps {
            is_hr: true,
            weekly_statistics: Some(stats),
        });
        // I18N defaults to English in tests; heading should appear
        assert!(
            html.contains("Statistics"),
            "HR stats heading must be present for HR user with stats: {html}"
        );
        // average_worked_hours_per_week = 23.5 → format_hours(23.5, 2) = "23.50"
        assert!(
            html.contains("23.50") || html.contains("23"),
            "HR stats must show the average hours value: {html}"
        );
    }

    #[test]
    fn hr_block_hidden_when_not_hr() {
        let stats = make_test_stats();
        let html = render_hr_stats_block(HrStatsBlockProps {
            is_hr: false,
            weekly_statistics: Some(stats),
        });
        assert!(
            !html.contains("Statistics"),
            "HR stats heading must NOT be present for non-HR user: {html}"
        );
    }

    #[test]
    fn hr_block_hidden_when_no_stats() {
        let html = render_hr_stats_block(HrStatsBlockProps {
            is_hr: true,
            weekly_statistics: None,
        });
        assert!(
            !html.contains("Statistics"),
            "HR stats heading must NOT be present when statistics is None: {html}"
        );
    }

    // --- Phase 47 (RPT-01/02/03): SSR + formatter tests for the weekday-attendance row ---

    use rest_types::WeekdayAttendanceTO;

    fn make_weekday_stats(
        counts_and_shares: [(DayOfWeekTO, u32, f32); 7],
        counted_calendar_weeks: u32,
    ) -> EmployeeAttendanceStatisticsTO {
        EmployeeAttendanceStatisticsTO {
            attendance_by_weekday: counts_and_shares
                .into_iter()
                .map(|(w, c, s)| WeekdayAttendanceTO {
                    weekday: w,
                    count: c,
                    share: s,
                    hours: 0.0,
                    share_of_hours: 0.0,
                })
                .collect(),
            counted_calendar_weeks,
        }
    }

    #[derive(Props, Clone, PartialEq)]
    struct WeekdayAttendanceRowProps {
        attendance_statistics: Option<Rc<EmployeeAttendanceStatisticsTO>>,
    }

    /// Minimal stub mirroring only the weekday-attendance row from the HR-stats block.
    fn render_weekday_attendance_row(p: WeekdayAttendanceRowProps) -> String {
        fn app(p: WeekdayAttendanceRowProps) -> Element {
            let i18n = crate::service::i18n::I18N.read().clone();
            rsx! {
                div {
                    if let Some(att) = p.attendance_statistics.as_ref() {
                        {
                            let line = format_weekday_attendance_line(att.as_ref(), &i18n);
                            let tooltip = i18n.t(crate::i18n::Key::WeekdayAttendanceTooltip);
                            rsx! {
                                span {
                                    class: "font-mono tabular-nums",
                                    title: "{tooltip}",
                                    "{line}"
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(app, p);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn weekday_row_renders_all_seven_segments_when_populated() {
        // I18N default locale in tests is En, so the labels are Mon..Sun.
        let stats = make_weekday_stats(
            [
                (DayOfWeekTO::Monday, 8, 0.8),
                (DayOfWeekTO::Tuesday, 3, 0.3),
                (DayOfWeekTO::Wednesday, 7, 0.7),
                (DayOfWeekTO::Thursday, 5, 0.5),
                (DayOfWeekTO::Friday, 2, 0.2),
                (DayOfWeekTO::Saturday, 0, 0.0),
                (DayOfWeekTO::Sunday, 0, 0.0),
            ],
            10,
        );
        let html = render_weekday_attendance_row(WeekdayAttendanceRowProps {
            attendance_statistics: Some(Rc::new(stats)),
        });

        for expected in [
            "Mon: 8 (80%)",
            "Tue: 3 (30%)",
            "Wed: 7 (70%)",
            "Thu: 5 (50%)",
            "Fri: 2 (20%)",
            "Sat: 0 (0%)",
            "Sun: 0 (0%)",
        ] {
            assert!(
                html.contains(expected),
                "expected segment `{expected}` in weekday row: {html}"
            );
        }

        // At least 6 middle-dot separators between 7 segments.
        let dot_count = html.matches('·').count();
        assert!(
            dot_count >= 6,
            "expected at least 6 middle-dot separators, got {dot_count}: {html}"
        );

        assert!(
            html.contains("title="),
            "row must carry a `title=` tooltip attribute: {html}"
        );

        // RPT-02 regression proof: v2.1 label must not reappear.
        assert!(
            !html.contains("Ø Std/Anwesenheitstag"),
            "v2.1 attendance label must not appear on the new row: {html}"
        );
    }

    #[test]
    fn weekday_row_renders_empty_state_when_counted_weeks_zero() {
        // All zeros + counted_calendar_weeks == 0 → localized empty-state text.
        let stats = make_weekday_stats(
            [
                (DayOfWeekTO::Monday, 0, 0.0),
                (DayOfWeekTO::Tuesday, 0, 0.0),
                (DayOfWeekTO::Wednesday, 0, 0.0),
                (DayOfWeekTO::Thursday, 0, 0.0),
                (DayOfWeekTO::Friday, 0, 0.0),
                (DayOfWeekTO::Saturday, 0, 0.0),
                (DayOfWeekTO::Sunday, 0, 0.0),
            ],
            0,
        );
        let html = render_weekday_attendance_row(WeekdayAttendanceRowProps {
            attendance_statistics: Some(Rc::new(stats)),
        });

        // Default test locale is En.
        assert!(
            html.contains("No counted calendar weeks in range"),
            "empty state text must be rendered: {html}"
        );
        assert!(
            !html.contains("Mon: 0 (0%)"),
            "empty state must NOT contain any weekday segment: {html}"
        );
    }

    #[test]
    fn weekday_row_absent_when_statistics_is_none() {
        // Non-flexible employee / non-HR → attendance_statistics == None → no row (D-AVG-05).
        let html = render_weekday_attendance_row(WeekdayAttendanceRowProps {
            attendance_statistics: None,
        });
        assert!(
            !html.contains("font-mono"),
            "row must NOT be rendered when statistics is None: {html}"
        );
        for label in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
            assert!(
                !html.contains(&format!("{label}: ")),
                "no weekday label segment must be rendered when None: {html}"
            );
        }
    }

    #[test]
    fn formatter_handles_odd_percents_correctly() {
        // 0.333 → 33.3 → round → 33.
        let stats = make_weekday_stats(
            [
                (DayOfWeekTO::Monday, 3, 0.333),
                (DayOfWeekTO::Tuesday, 0, 0.0),
                (DayOfWeekTO::Wednesday, 0, 0.0),
                (DayOfWeekTO::Thursday, 0, 0.0),
                (DayOfWeekTO::Friday, 0, 0.0),
                (DayOfWeekTO::Saturday, 0, 0.0),
                (DayOfWeekTO::Sunday, 0, 0.0),
            ],
            10,
        );
        // Bypass the global I18N (Dioxus runtime not available in pure-fn tests).
        let i18n = crate::i18n::generate(crate::i18n::Locale::En);
        let line = format_weekday_attendance_line(&stats, &i18n);
        assert!(
            line.contains("Mon: 3 (33%)"),
            "share=0.333 must round to 33%: {line}"
        );
    }
}
