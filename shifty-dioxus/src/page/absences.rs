//! `AbsencesPage` — Top-Level Route `/absences` (Phase 8 Wave 5, Plan 05).
//!
//! Composes the full Absence-CRUD UI on top of the Plan-04 foundation
//! (api / state / loader / coroutine-services / i18n / proxy):
//!
//! - `AbsencesPage` (HR vs Employee branch via `auth.has_privilege("hr")`, D-09).
//! - `AbsenceModal` (Center-Dialog, range-picker D-05, 422 SelfOverlapBanner D-11,
//!   409 VersionConflictBanner D-08, Forward-Warning list D-12).
//! - `WarningList`, `CategoryBadge`, `StatusPill`,
//!   `VacationEntitlementCard`, `VacationPerPersonList`,
//!   `AbsenceList`, `AbsenceFilterBar`, `StatsGrid`, `DeleteConfirmDialog`.
//!
//! All inline per the Plan-05 component-inventory contract — these components
//! are domain-specific to /absences and not re-used elsewhere in v1.3.
//!
//! Snapshot / pure-function tests live in the `#[cfg(test)]` module at the
//! end of this file (Plan-05 Task 3).

use std::rc::Rc;

use dioxus::prelude::*;
use time::macros::{date, format_description};
use uuid::Uuid;

use rest_types::{AbsenceCategoryTO, AbsencePeriodTO, WarningTO};

use crate::base_types::ImStr;
use crate::component::atoms::{Btn, BtnVariant};
use crate::component::error_view::ErrorView;
use crate::component::form::{Field, SelectInput, TextInput, TextareaInput};
use crate::component::{Dialog, DialogVariant, TopBar};
use crate::i18n::Key;
use crate::loader;
use crate::component::absence_convert_modal::AbsenceConvertModal;
use crate::component::extra_hours_modal::ExtraHoursModal;
use crate::component::warning_list::{WarningList, WarningsList};
use crate::service::absence::{
    AbsenceAction, AbsenceModalEvent, ABSENCE_HOURLY_STORE, ABSENCE_MODAL_EVENT, ABSENCE_REFRESH,
    ABSENCE_STORE,
};
use crate::service::auth::AUTH;
use crate::service::config::CONFIG;
use crate::service::i18n::I18N;
use crate::service::vacation_balance::{
    VacationBalanceAction, VACATION_BALANCE_STORE, VACATION_TEAM_STORE,
};
use crate::state::absence_period::{AbsenceCategory, AbsencePeriod, DayFraction, ExtraHoursMarker};
use crate::state::employee::{ExtraHours, WorkingHoursCategory};
use crate::state::shiftplan::SalesPerson;
use crate::state::vacation_balance::VacationBalance;

// ─── Time helpers ──────────────────────────────────────────────────────────
//
// Production code calls `js::current_datetime()` (wasm_bindgen → `Date`),
// which panics outside a JS environment. Native unit tests therefore use a
// fixed reference date — Plan-05 W-9 constrains this to `#[cfg(...)]` and
// `#[cfg(test)]` boundaries; the production render path never sees the
// hard-coded date.

#[cfg(target_arch = "wasm32")]
fn current_date_for_init() -> time::Date {
    crate::js::current_datetime().date()
}

#[cfg(not(target_arch = "wasm32"))]
fn current_date_for_init() -> time::Date {
    // Native tests only — production WASM build uses `js::current_datetime()`.
    date!(2026 - 05 - 08)
}

#[cfg(target_arch = "wasm32")]
fn current_year_for_init() -> u32 {
    crate::js::get_current_year()
}

#[cfg(not(target_arch = "wasm32"))]
fn current_year_for_init() -> u32 {
    2026
}

// ─── Absence status (D-06, Pitfall 8 — pure function) ─────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbsenceStatus {
    Active,
    Planned,
    Finished,
}

/// Pure function — `today` is injected so unit tests can pin it.
pub fn compute_status(from: time::Date, to: time::Date, today: time::Date) -> AbsenceStatus {
    if to < today {
        AbsenceStatus::Finished
    } else if from > today {
        AbsenceStatus::Planned
    } else {
        AbsenceStatus::Active
    }
}

/// Pure function — a sales person is selectable in the absences person dropdowns
/// only if they are paid (`is_paid`) and active (`!inactive`). The loaded
/// `sales_persons` list itself stays complete so existing absences of inactive /
/// unpaid persons can still resolve their names; only the selection dropdowns
/// are narrowed.
pub fn is_selectable_employee(sales_person: &SalesPerson) -> bool {
    sales_person.is_paid && !sales_person.inactive
}

/// Pure function — filter a vacation-balance slice to only balances whose
/// `sales_person_id` maps to a selectable (paid && active) `SalesPerson`.
///
/// Semantics:
/// - A balance is KEPT only if `sales_persons` contains an entry with a
///   matching `id` for which `is_selectable_employee(sp)` is true.
/// - A balance whose sales person is absent from `sales_persons` is DROPPED
///   (unknown ≡ not selectable, consistent with the dropdown contract).
/// - The order of kept balances is preserved; no sorting is applied here.
///   `VacationPerPersonList` applies its own remaining-days sort downstream.
pub fn selectable_balances(
    rows: &[VacationBalance],
    sales_persons: &[SalesPerson],
) -> Vec<VacationBalance> {
    rows.iter()
        .filter(|b| {
            sales_persons
                .iter()
                .any(|sp| sp.id == b.sales_person_id && is_selectable_employee(sp))
        })
        .cloned()
        .collect()
}

/// Temporär: Krankheitstage sind auf der Absences-Seite ausgeblendet, weil die
/// fachliche Behandlung noch unvollständig ist (Quick-Task 260612-svs). Gilt
/// NUR für diese Seite — Reports zeigen Krankheitstage weiterhin. Reaktivierung:
/// Konstante auf `true` setzen.
pub const SICK_LEAVE_ENABLED: bool = false;

/// Parametrischer Kern, damit Tests beide Schalterstellungen prüfen können,
/// ohne an den aktuellen Wert von `SICK_LEAVE_ENABLED` gekoppelt zu sein.
fn category_visible_with(sick_leave_enabled: bool, category: AbsenceCategory) -> bool {
    sick_leave_enabled || category != AbsenceCategory::SickLeave
}

/// Pure function — ist eine Absence-Kategorie auf der Absences-Seite sichtbar?
pub fn is_visible_category(category: AbsenceCategory) -> bool {
    category_visible_with(SICK_LEAVE_ENABLED, category)
}

/// Pure function — ist ein stundenbasierter Marker mit dieser Kategorie auf der
/// Absences-Seite sichtbar? Unmappbare Kategorien bleiben sichtbar (bestehendes
/// Verhalten der Marker-Pipeline); nur SickLeave wird ausgeblendet.
pub fn is_visible_marker_category(category: &rest_types::ExtraHoursCategoryTO) -> bool {
    match map_marker_category(category) {
        Some(cat) => is_visible_category(cat),
        None => true,
    }
}

/// Map an `ExtraHoursCategoryTO` to an `AbsenceCategory` where possible.
///
/// Only Vacation, SickLeave, and UnpaidLeave have direct counterparts.
/// All other variants (ExtraWork, Holiday, Unavailable, VolunteerWork, Custom)
/// are not representable as an `AbsenceCategory` and return `None`.
fn map_marker_category(c: &rest_types::ExtraHoursCategoryTO) -> Option<AbsenceCategory> {
    match c {
        rest_types::ExtraHoursCategoryTO::Vacation => Some(AbsenceCategory::Vacation),
        rest_types::ExtraHoursCategoryTO::SickLeave => Some(AbsenceCategory::SickLeave),
        rest_types::ExtraHoursCategoryTO::UnpaidLeave => Some(AbsenceCategory::UnpaidLeave),
        _ => None,
    }
}

/// Pure filter predicate for `ExtraHoursMarker` entries.
///
/// Analogous to the inline `Range-Absence` filter closure in `AbsencesPage`,
/// but operates on a single-day marker. `today` is injected so unit tests can
/// pin it without calling into JS.
fn marker_matches_filters(
    marker: &ExtraHoursMarker,
    category_filter: Option<AbsenceCategory>,
    person_filter: Option<Uuid>,
    status_filter: Option<AbsenceStatus>,
    show_past: bool,
    today: time::Date,
) -> bool {
    if let Some(cat) = category_filter {
        match map_marker_category(&marker.category) {
            Some(marker_cat) => {
                if marker_cat != cat {
                    return false;
                }
            }
            None => return false, // unmappable category never matches a concrete filter
        }
    }
    if let Some(person) = person_filter {
        if marker.sales_person_id != person {
            return false;
        }
    }
    let status = compute_status(marker.when, marker.when, today);
    if let Some(s) = status_filter {
        if status != s {
            return false;
        }
    }
    if !show_past && status == AbsenceStatus::Finished {
        return false;
    }
    true
}

// ─── Modal mode ────────────────────────────────────────────────────────────

// Kein `Eq` — enthält `AbsencePeriod`, das wegen `derived_days: f32` nur
// `PartialEq` implementiert.
#[derive(Clone, Debug, PartialEq)]
enum ModalMode {
    Create,
    Edit(AbsencePeriod),
}

// ─── CategoryBadge (Pitfall 5 — STATIC Tailwind match arms) ────────────────

#[derive(Props, Clone, PartialEq)]
pub struct CategoryBadgeProps {
    pub category: AbsenceCategory,
}

#[component]
pub fn CategoryBadge(props: CategoryBadgeProps) -> Element {
    let i18n = I18N.read().clone();
    // Pitfall 5: STATIC Tailwind classes per category. NEVER use `format!`.
    let (text_class, bg_class, key) = match props.category {
        AbsenceCategory::Vacation => ("text-good", "bg-good-soft", Key::AbsenceCategoryVacation),
        AbsenceCategory::SickLeave => ("text-warn", "bg-warn-soft", Key::AbsenceCategorySickLeave),
        AbsenceCategory::UnpaidLeave => ("text-ink-muted", "bg-surface-2", Key::AbsenceCategoryUnpaidLeave),
    };
    let label = i18n.t(key);
    rsx! {
        span {
            class: "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-small font-semibold {text_class} {bg_class}",
            // 7×7 px dot indicator — `bg-current` inherits the badge text colour token.
            span { class: "w-1.5 h-1.5 rounded-full bg-current" }
            "{label}"
        }
    }
}

// ─── StatusPill (Pitfall 5 — STATIC Tailwind match arms) ──────────────────

#[derive(Props, Clone, PartialEq)]
pub struct StatusPillProps {
    pub status: AbsenceStatus,
}

#[component]
pub fn StatusPill(props: StatusPillProps) -> Element {
    let i18n = I18N.read().clone();
    let (text_class, bg_class, key) = match props.status {
        AbsenceStatus::Active => ("text-accent", "bg-accent-soft", Key::AbsenceStatusActive),
        AbsenceStatus::Planned => ("text-ink-soft", "bg-surface-2", Key::AbsenceStatusPlanned),
        AbsenceStatus::Finished => (
            "text-ink-muted",
            "bg-surface-alt",
            Key::AbsenceStatusFinished,
        ),
    };
    let label = i18n.t(key);
    rsx! {
        span {
            class: "inline-flex items-center rounded-full px-2 py-0.5 text-micro font-semibold {text_class} {bg_class}",
            "{label}"
        }
    }
}

// ─── VacationEntitlementCard ───────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct VacationEntitlementCardProps {
    pub is_hr: bool,
    pub year: u32,
    #[props(!optional, default = None)]
    pub vacation_self: Option<VacationBalance>,
    pub vacation_team: Rc<[VacationBalance]>,
    pub sales_persons: Rc<[SalesPerson]>,
    /// D2: When HR selects a Sales Person, render that person's self-view
    /// (hero + 5 StatBoxes) instead of the team aggregate + per-person list.
    /// When None, HR sees the team aggregate as before. Employee path ignores this.
    #[props(!optional, default = None)]
    pub selected_person: Option<Uuid>,
}

#[component]
pub fn VacationEntitlementCard(props: VacationEntitlementCardProps) -> Element {
    let i18n = I18N.read().clone();
    let year_str = props.year.to_string();
    let prev_year_str = (props.year.saturating_sub(1)).to_string();

    // D2: HR with a selected person shows the self-view for that person.
    // Look up the matching VacationBalance in vacation_team (None = not found → zero default).
    let forced_self: Option<VacationBalance> = props
        .selected_person
        .and_then(|id| props.vacation_team.iter().find(|b| b.sales_person_id == id).cloned());

    // show_self is true when: (a) this is not an HR user, OR (b) HR has selected a person.
    let show_self = !props.is_hr || forced_self.is_some();

    let title_template;
    let subtitle_template;
    if show_self {
        title_template = i18n.t(Key::VacationCardSelfTitle).as_ref().to_string();
        subtitle_template = i18n.t(Key::VacationCardSelfSubtitle).as_ref().to_string();
    } else {
        title_template = i18n
            .t(Key::VacationCardTeamTitle)
            .as_ref()
            .replace("{count}", &props.vacation_team.len().to_string());
        subtitle_template = i18n.t(Key::VacationCardTeamSubtitle).as_ref().to_string();
    }

    rsx! {
        section { class: "bg-surface border border-border rounded-lg overflow-hidden",
            div { class: "px-6 py-4 flex flex-col gap-1 border-b border-border",
                h3 { class: "text-lg font-semibold text-ink", "{title_template}" }
                div { class: "text-small text-ink-muted", "{subtitle_template}" }
            }
            if show_self {
                VacationEntitlementSelfBody {
                    year: props.year,
                    year_str: year_str.clone(),
                    prev_year_str: prev_year_str.clone(),
                    // For HR-with-selection: use the selected person's balance.
                    // For employee: fall back to their own vacation_self.
                    vacation_self: forced_self.clone().or(props.vacation_self.clone()),
                }
            } else {
                VacationEntitlementHrBody {
                    year: props.year,
                    prev_year_str: prev_year_str.clone(),
                    vacation_team: props.vacation_team.clone(),
                    sales_persons: props.sales_persons.clone(),
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct VacationEntitlementSelfBodyProps {
    year: u32,
    year_str: String,
    prev_year_str: String,
    #[props(!optional, default = None)]
    vacation_self: Option<VacationBalance>,
}

#[component]
fn VacationEntitlementSelfBody(props: VacationEntitlementSelfBodyProps) -> Element {
    let i18n = I18N.read().clone();
    let _ = props.year;
    let balance = props.vacation_self.clone().unwrap_or(VacationBalance {
        sales_person_id: Uuid::nil(),
        year: 0,
        entitled_days: 0.0,
        carryover_days: 0,
        used_days: 0.0,
        planned_days: 0.0,
        remaining_days: 0.0,
    });
    let hero_label = i18n
        .t(Key::VacationEntitlementHero)
        .as_ref()
        .replace("{year}", &props.year_str);
    let remaining = format_decimal(balance.remaining_days);
    let entitled_total = balance.entitled_days + (balance.carryover_days as f32);
    let entitled_total_str = format_decimal(entitled_total);
    let used_str = format_decimal(balance.used_days);
    let planned_str = format_decimal(balance.planned_days);
    let entitled_contract_str = format_decimal(balance.entitled_days);
    let carryover_str = format!("{}", balance.carryover_days);
    let carryover_label = i18n
        .t(Key::VacationStatCarryover)
        .as_ref()
        .replace("{year-1}", &props.prev_year_str);
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-[180px_1fr] gap-0",
            div { class: "bg-good-soft p-6 flex flex-col items-center justify-center gap-1",
                div { class: "text-micro uppercase text-ink-soft", "{hero_label}" }
                div { class: "text-display font-mono text-good font-bold leading-none",
                    "{remaining}/{entitled_total_str}"
                }
                div { class: "text-small text-ink-muted",
                    "{i18n.t(Key::VacationDaysRemaining)}"
                }
            }
            // Plan 08-07 Task 5: 5 Stat-Boxes — auf Mobile gestapelt 1-col,
            // ab sm 2-col (paarweise), ab md alle 5 nebeneinander für das
            // Desktop-Hero-Layout neben der Hero-Zahl.
            div { class: "p-4 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-5 gap-2.5",
                StatBox {
                    label: ImStr::from(i18n.t(Key::VacationStatContract).as_ref()),
                    value: ImStr::from(entitled_contract_str.as_str()),
                }
                StatBox {
                    label: ImStr::from(carryover_label.as_str()),
                    value: ImStr::from(carryover_str.as_str()),
                }
                StatBox {
                    label: ImStr::from(i18n.t(Key::VacationStatUsed).as_ref()),
                    value: ImStr::from(used_str.as_str()),
                }
                StatBox {
                    label: ImStr::from(i18n.t(Key::VacationStatPending).as_ref()),
                    value: ImStr::from(planned_str.as_str()),
                }
                StatBox {
                    label: ImStr::from(i18n.t(Key::VacationStatRemaining).as_ref()),
                    value: ImStr::from(remaining.as_str()),
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct VacationEntitlementHrBodyProps {
    year: u32,
    prev_year_str: String,
    vacation_team: Rc<[VacationBalance]>,
    sales_persons: Rc<[SalesPerson]>,
}

#[component]
fn VacationEntitlementHrBody(props: VacationEntitlementHrBodyProps) -> Element {
    let i18n = I18N.read().clone();
    let _ = props.year;
    let team = props.vacation_team.clone();
    let sum_remaining: f32 = team.iter().map(|b| b.remaining_days).sum();
    let sum_entitled: f32 = team
        .iter()
        .map(|b| b.entitled_days + (b.carryover_days as f32))
        .sum();
    let sum_used: f32 = team.iter().map(|b| b.used_days).sum();
    let sum_planned: f32 = team.iter().map(|b| b.planned_days).sum();
    let sum_carryover: i32 = team.iter().map(|b| b.carryover_days).sum();
    let carryover_label = i18n
        .t(Key::VacationStatCarryover)
        .as_ref()
        .replace("{year-1}", &props.prev_year_str);
    rsx! {
        div { class: "p-4 flex flex-col gap-3",
            div { class: "flex flex-row items-baseline gap-2",
                div { class: "text-h1 font-mono text-good font-semibold",
                    "{format_decimal(sum_remaining)}"
                }
                div { class: "text-small text-ink-muted",
                    "/ {format_decimal(sum_entitled)} {i18n.t(Key::VacationDaysRemaining)}"
                }
            }
            // Plan 08-07 Task 5: HR-Aggregate-Stat-Boxes — analog Self-Variante
            // als deterministisches 1/2/5-cols-Stepping.
            div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-5 gap-2.5",
                StatBox {
                    label: ImStr::from(i18n.t(Key::VacationStatContract).as_ref()),
                    value: ImStr::from(format_decimal(sum_entitled - sum_carryover as f32).as_str()),
                }
                StatBox {
                    label: ImStr::from(carryover_label.as_str()),
                    value: ImStr::from(format!("{}", sum_carryover).as_str()),
                }
                StatBox {
                    label: ImStr::from(i18n.t(Key::VacationStatUsed).as_ref()),
                    value: ImStr::from(format_decimal(sum_used).as_str()),
                }
                StatBox {
                    label: ImStr::from(i18n.t(Key::VacationStatPending).as_ref()),
                    value: ImStr::from(format_decimal(sum_planned).as_str()),
                }
                StatBox {
                    label: ImStr::from(i18n.t(Key::VacationStatRemaining).as_ref()),
                    value: ImStr::from(format_decimal(sum_remaining).as_str()),
                }
            }
            VacationPerPersonList {
                rows: team.clone(),
                sales_persons: props.sales_persons.clone(),
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct StatBoxProps {
    label: ImStr,
    value: ImStr,
}

#[component]
fn StatBox(props: StatBoxProps) -> Element {
    rsx! {
        div { class: "bg-surface border border-border rounded-md p-3 flex flex-col gap-1",
            div { class: "text-micro uppercase text-ink-muted", "{props.label}" }
            div { class: "text-h1 font-mono text-ink", "{props.value}" }
        }
    }
}

fn format_decimal(value: f32) -> String {
    if (value - value.trunc()).abs() < 0.05 {
        format!("{}", value.trunc() as i32)
    } else {
        format!("{:.1}", value)
    }
}

// ─── VacationPerPersonList (HR-only) ──────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct VacationPerPersonListProps {
    pub rows: Rc<[VacationBalance]>,
    pub sales_persons: Rc<[SalesPerson]>,
}

#[component]
pub fn VacationPerPersonList(props: VacationPerPersonListProps) -> Element {
    let i18n = I18N.read().clone();
    let mut show_all = use_signal(|| false);
    // Filter first — only paid & active persons are shown in the per-person list.
    // This must happen before the empty-guard so that a non-empty `rows` that
    // contains only inactive/unpaid persons collapses to the empty state.
    let filtered = selectable_balances(&props.rows, &props.sales_persons);
    if filtered.is_empty() {
        return rsx! {};
    }
    // Sort ascending by remaining_days; filtered is already a Vec<VacationBalance>.
    let mut sorted: Vec<VacationBalance> = filtered;
    sorted.sort_by(|a, b| {
        a.remaining_days
            .partial_cmp(&b.remaining_days)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let total = sorted.len();
    let limit = if *show_all.read() { total } else { 4.min(total) };
    let visible: Vec<VacationBalance> = sorted.iter().take(limit).cloned().collect();
    let toggle_label = if *show_all.read() {
        i18n.t(Key::VacationPerPersonShowLess).as_ref().to_string()
    } else {
        i18n
            .t(Key::VacationPerPersonShowAll)
            .as_ref()
            .replace("{count}", &total.to_string())
    };
    let header_label = i18n.t(Key::VacationPerPersonHeader);
    rsx! {
        section { class: "border-t border-border bg-surface-alt rounded-md p-3 flex flex-col gap-3",
            div { class: "flex items-center justify-between gap-2",
                div { class: "text-micro uppercase text-ink-muted font-semibold", "{header_label}" }
                if total > 4 {
                    button {
                        r#type: "button",
                        class: "text-small text-accent font-semibold hover:underline",
                        onclick: move |_| {
                            let v = *show_all.read();
                            show_all.set(!v);
                        },
                        "{toggle_label}"
                    }
                }
            }
            // Plan 08-07 Task 5: Per-Person-Karten — Mobile 1-col, ab sm 2-col,
            // ab lg ebenfalls 2-col (Plan-Anker; auto-fill würde sonst auf
            // breiten Bildschirmen 4 Spalten zeigen, was die Information
            // dünn streckt).
            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-2 gap-2",
                for row in visible.iter() {
                    PersonVacationCard {
                        balance: row.clone(),
                        sales_persons: props.sales_persons.clone(),
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct PersonVacationCardProps {
    balance: VacationBalance,
    sales_persons: Rc<[SalesPerson]>,
}

#[component]
fn PersonVacationCard(props: PersonVacationCardProps) -> Element {
    let person = props
        .sales_persons
        .iter()
        .find(|sp| sp.id == props.balance.sales_person_id);
    let name: ImStr = match &person {
        Some(p) => ImStr::from(p.name.as_ref()),
        None => ImStr::from("?"),
    };
    let bg_color: ImStr = match &person {
        Some(p) => ImStr::from(p.background_color.as_ref()),
        None => ImStr::from("#cccccc"),
    };
    // Pitfall 5: `text-warn` low-indicator is a STATIC class — picked via
    // a match on a small bucket, not interpolated.
    let low = props.balance.remaining_days <= 3.0;
    let (remaining_class, bar_class) = if low {
        ("text-warn", "bg-warn")
    } else {
        ("text-good", "bg-good")
    };
    let total = props.balance.entitled_days + (props.balance.carryover_days as f32);
    let used_pct: u32 = if total > 0.01 {
        ((props.balance.used_days / total) * 100.0).clamp(0.0, 100.0) as u32
    } else {
        0
    };
    let bar_style = format!("width:{}%", used_pct);
    rsx! {
        div { class: "bg-surface border border-border rounded-md p-2 px-3 flex flex-col gap-1.5",
            div { class: "flex items-center gap-2",
                span {
                    class: "w-[22px] h-[22px] rounded-full flex-shrink-0",
                    style: "background:{bg_color};",
                    "aria-hidden": "true",
                }
                span { class: "text-body font-semibold truncate flex-1", "{name}" }
                span { class: "text-body font-mono font-bold {remaining_class}",
                    "{format_decimal(props.balance.remaining_days)}"
                }
            }
            div { class: "h-1 rounded-full bg-surface-alt overflow-hidden",
                div {
                    class: "h-full rounded-full {bar_class}",
                    style: "{bar_style}",
                }
            }
        }
    }
}

// ─── Banners (D-08 / D-11) ────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct VersionConflictBannerProps {
    on_reload: EventHandler<()>,
}

#[component]
fn VersionConflictBanner(props: VersionConflictBannerProps) -> Element {
    let i18n = I18N.read().clone();
    let on_reload = props.on_reload;
    rsx! {
        div { class: "border-l-[3px] border-warn bg-warn-soft rounded-md p-3 flex items-start justify-between gap-3",
            div { class: "flex flex-col gap-1",
                div { class: "text-micro uppercase font-semibold text-warn",
                    "{i18n.t(Key::AbsenceErrorVersionConflictHeader)}"
                }
                div { class: "text-body text-ink",
                    "{i18n.t(Key::AbsenceErrorVersionConflictBody)}"
                }
            }
            Btn {
                variant: BtnVariant::Ghost,
                on_click: move |_| on_reload.call(()),
                "{i18n.t(Key::AbsenceErrorVersionConflictReload)}"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SelfOverlapBannerProps {
    raw_payload: String,
}

#[component]
fn SelfOverlapBanner(props: SelfOverlapBannerProps) -> Element {
    let i18n = I18N.read().clone();
    rsx! {
        div { class: "border-l-[3px] border-bad bg-bad-soft rounded-md p-3 flex flex-col gap-1",
            div { class: "text-micro uppercase font-semibold text-bad",
                "{i18n.t(Key::AbsenceErrorSelfOverlapHeader)}"
            }
            div { class: "text-body text-ink",
                // Backend returns a free-form string body for 422; we surface the
                // i18n template plus the raw payload as fallback context. Auto-
                // escape applies per T-8-XSS-01; never use raw HTML injection.
                "{i18n.t(Key::AbsenceErrorSelfOverlapBody)}"
                if !props.raw_payload.is_empty() {
                    span { class: "block text-small text-ink-muted mt-1",
                        "{props.raw_payload}"
                    }
                }
            }
        }
    }
}

// ─── DeleteConfirmDialog (D-07) ────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct DeleteConfirmDialogProps {
    pub open: bool,
    pub on_close: EventHandler<()>,
    pub on_confirm: EventHandler<()>,
}

#[component]
pub fn DeleteConfirmDialog(props: DeleteConfirmDialogProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let i18n = I18N.read().clone();
    let title = ImStr::from(i18n.t(Key::AbsenceDeleteConfirmTitle).as_ref());
    let cancel_label = ImStr::from(i18n.t(Key::AbsenceDeleteCancelBtn).as_ref());
    let confirm_label = ImStr::from(i18n.t(Key::AbsenceDeleteConfirmBtn).as_ref());
    let on_close = props.on_close;
    let on_close_for_cancel = props.on_close;
    let on_confirm = props.on_confirm;
    let footer = rsx! {
        Btn {
            variant: BtnVariant::Ghost,
            on_click: move |_| on_close_for_cancel.call(()),
            "{cancel_label}"
        }
        Btn {
            variant: BtnVariant::Danger,
            on_click: move |_| on_confirm.call(()),
            "{confirm_label}"
        }
    };
    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_close.call(()),
            title: title,
            variant: DialogVariant::Center,
            width: 360,
            footer: Some(footer),
            div { class: "text-body text-ink",
                "{i18n.t(Key::AbsenceDeleteConfirmBody)}"
            }
        }
    }
}

// ─── AbsenceModal ─────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct AbsenceModalProps {
    pub open: bool,
    pub mode: AbsenceModalMode,
    pub is_hr: bool,
    pub sales_persons: Rc<[SalesPerson]>,
    #[props(!optional, default = None)]
    pub current_sp_id: Option<Uuid>,
    pub on_close: EventHandler<()>,
    pub on_delete_request: EventHandler<()>,
}

// Kein `Eq` — enthält `AbsencePeriod`, das wegen `derived_days: f32` nur
// `PartialEq` implementiert.
#[derive(Clone, Debug, PartialEq)]
pub enum AbsenceModalMode {
    Create,
    Edit(AbsencePeriod),
}

impl From<AbsenceModalMode> for ModalMode {
    fn from(m: AbsenceModalMode) -> Self {
        match m {
            AbsenceModalMode::Create => ModalMode::Create,
            AbsenceModalMode::Edit(a) => ModalMode::Edit(a),
        }
    }
}

#[component]
pub fn AbsenceModal(props: AbsenceModalProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let i18n = I18N.read().clone();
    let date_format = format_description!("[year]-[month]-[day]");
    let absence_service = try_consume_context::<Coroutine<AbsenceAction>>();

    let mode_clone = props.mode.clone();
    let is_edit = matches!(mode_clone, AbsenceModalMode::Edit(_));
    let editing_period: Option<AbsencePeriod> = match &mode_clone {
        AbsenceModalMode::Edit(a) => Some(a.clone()),
        AbsenceModalMode::Create => None,
    };

    let initial_employee = match &editing_period {
        Some(a) => a.sales_person_id,
        None => props.current_sp_id.unwrap_or(Uuid::nil()),
    };
    let initial_category = match &editing_period {
        Some(a) => a.category,
        None => AbsenceCategory::Vacation,
    };
    // Phase 8.3: Tageshälfte — Create defaults to Full; Edit reads from the
    // editing AbsencePeriod's day_fraction state mirror (Plan 01 wired this
    // through `AbsencePeriod::from(&AbsencePeriodTO)`).
    let initial_day_fraction: DayFraction = match &editing_period {
        Some(a) => a.day_fraction,
        None => DayFraction::Full,
    };
    let today = current_date_for_init();
    let initial_from = match &editing_period {
        Some(a) => a.from_date.format(&date_format).unwrap_or_default(),
        None => today.format(&date_format).unwrap_or_default(),
    };
    let initial_to = match &editing_period {
        Some(a) => a.to_date.format(&date_format).unwrap_or_default(),
        None => today.format(&date_format).unwrap_or_default(),
    };
    let initial_description = editing_period
        .as_ref()
        .map(|a| a.description.as_ref().to_string())
        .unwrap_or_default();

    let mut employee_id = use_signal(|| initial_employee);
    let mut category = use_signal(|| initial_category);
    let mut from_date = use_signal(|| initial_from.clone());
    let mut to_date = use_signal(|| initial_to.clone());
    let mut description = use_signal(|| initial_description.clone());
    // Phase 8.3: Tageshälfte form signal.
    let mut day_fraction = use_signal(|| initial_day_fraction);

    // Re-seed when the props change between Create/Edit or between
    // different Edit-targets without unmounting (analog
    // `extra_hours_modal.rs:140-148`).
    let editing_key = editing_period.as_ref().map(|a| a.id);
    let mut last_editing_key = use_signal(|| editing_key);
    if *last_editing_key.peek() != editing_key {
        last_editing_key.set(editing_key);
        employee_id.set(initial_employee);
        category.set(initial_category);
        from_date.set(initial_from.clone());
        to_date.set(initial_to.clone());
        description.set(initial_description.clone());
        day_fraction.set(initial_day_fraction);
    }

    // Modal-local outcome state — reset on every open.
    let mut conflict_open = use_signal(|| false);
    let mut validation_payload = use_signal(|| None::<String>);
    let mut warnings_state = use_signal(WarningsList::empty);

    // Subscribe to ABSENCE_MODAL_EVENT side-channel; the service writes the
    // outcome of Create/Update there. We acknowledge by writing `None`.
    let modal_event = ABSENCE_MODAL_EVENT.read().clone();
    use_effect(move || {
        let event = ABSENCE_MODAL_EVENT.read().clone();
        if let Some(ev) = event {
            match ev {
                AbsenceModalEvent::Created(result) | AbsenceModalEvent::Updated(result) => {
                    if result.warnings.is_empty() {
                        // No warnings → close immediately.
                        warnings_state.set(WarningsList::empty());
                        conflict_open.set(false);
                        validation_payload.set(None);
                        // Acknowledge.
                        *ABSENCE_MODAL_EVENT.write() = None;
                        // Trigger close via on_close.
                        // (Stored as captured handler below.)
                    } else {
                        let rc: Rc<[WarningTO]> = Rc::from(result.warnings.as_slice());
                        warnings_state.set(WarningsList(rc));
                        conflict_open.set(false);
                        validation_payload.set(None);
                        *ABSENCE_MODAL_EVENT.write() = None;
                    }
                }
                AbsenceModalEvent::VersionConflict => {
                    conflict_open.set(true);
                    validation_payload.set(None);
                    warnings_state.set(WarningsList::empty());
                    *ABSENCE_MODAL_EVENT.write() = None;
                }
                AbsenceModalEvent::Validation(text) => {
                    validation_payload.set(Some(text));
                    conflict_open.set(false);
                    warnings_state.set(WarningsList::empty());
                    *ABSENCE_MODAL_EVENT.write() = None;
                }
                AbsenceModalEvent::Network(_) => {
                    *ABSENCE_MODAL_EVENT.write() = None;
                }
                AbsenceModalEvent::Deleted => {
                    *ABSENCE_MODAL_EVENT.write() = None;
                }
            }
        }
    });
    let _ = modal_event;

    // Close-effect: when Created/Updated cleared `warnings_state` to empty
    // and the validation/conflict flags are off, treat this as "submit done,
    // no warnings" → invoke on_close. We detect that via a separate effect on
    // the warnings signal AFTER the user explicitly acknowledges, see below.

    let date_iso_format = date_format;
    let title_rc = if is_edit {
        i18n.t(Key::AbsenceModalEditSubtitle)
    } else {
        i18n.t(Key::AbsenceModalCreateSubtitle)
    };
    let title = ImStr::from(title_rc.as_ref());
    let dialog_title_rc = i18n.t(Key::AbsenceNewBtn);
    let dialog_title = ImStr::from(dialog_title_rc.as_ref());

    let parsed_from = time::Date::parse(&from_date.read(), &date_iso_format).ok();
    let parsed_to = time::Date::parse(&to_date.read(), &date_iso_format).ok();
    let range_invalid = match (parsed_from, parsed_to) {
        (Some(f), Some(t)) => t < f,
        _ => false,
    };
    let from_empty = from_date.read().is_empty();
    let to_empty = to_date.read().is_empty();
    let employee_required = *employee_id.read() == Uuid::nil();
    let has_warnings = !warnings_state.read().is_empty();
    let submit_disabled =
        from_empty || to_empty || range_invalid || employee_required;

    let cancel_label = ImStr::from(i18n.t(Key::AbsenceModalCancelBtn).as_ref());
    let delete_label = ImStr::from(i18n.t(Key::AbsenceModalDeleteBtn).as_ref());
    let submit_label = if has_warnings {
        ImStr::from(i18n.t(Key::AbsenceWarningAcknowledgeBtn).as_ref())
    } else if is_edit {
        ImStr::from(i18n.t(Key::AbsenceModalSaveBtn).as_ref())
    } else {
        ImStr::from(i18n.t(Key::AbsenceModalCreateBtn).as_ref())
    };
    let employee_label = ImStr::from(i18n.t(Key::AbsenceFieldEmployee).as_ref());
    let category_label = ImStr::from(i18n.t(Key::AbsenceFieldCategory).as_ref());
    let from_label = ImStr::from(i18n.t(Key::AbsenceFieldFrom).as_ref());
    let to_label = ImStr::from(i18n.t(Key::AbsenceFieldTo).as_ref());
    let description_label = ImStr::from(i18n.t(Key::AbsenceFieldDescription).as_ref());
    let description_hint = ImStr::from(i18n.t(Key::AbsenceFieldDescriptionHint).as_ref());
    // Phase 8.3: Tageshälfte field + reactive hint.
    let day_fraction_label = ImStr::from(i18n.t(Key::AbsenceFieldDayFraction).as_ref());
    let day_fraction_hint: ImStr = match *day_fraction.read() {
        DayFraction::Full => ImStr::from(i18n.t(Key::AbsenceFieldDayFractionFullHint).as_ref()),
        DayFraction::Half => ImStr::from(i18n.t(Key::AbsenceFieldDayFractionHalfHint).as_ref()),
    };
    let day_fraction_full_label = i18n.t(Key::AbsenceDayFractionFull);
    let day_fraction_half_label = i18n.t(Key::AbsenceDayFractionHalf);
    let range_error: Option<ImStr> = if range_invalid {
        Some(ImStr::from(i18n.t(Key::AbsenceErrorRangeInverted).as_ref()))
    } else {
        None
    };

    let on_close = props.on_close;
    let on_close_for_cancel = props.on_close;
    let on_close_for_dialog = props.on_close;
    let on_close_for_warn_ack = props.on_close;
    let on_delete_request = props.on_delete_request;

    let mode_for_submit = mode_clone.clone();
    let editing_period_for_submit = editing_period.clone();
    let absence_service_for_submit = absence_service.clone();
    let on_submit = move |_| {
        // If we already have warnings → user clicks "Verstanden"; close.
        if !warnings_state.read().is_empty() {
            warnings_state.set(WarningsList::empty());
            *ABSENCE_MODAL_EVENT.write() = None;
            on_close_for_warn_ack.call(());
            return;
        }
        if submit_disabled {
            return;
        }
        let from_parsed =
            time::Date::parse(&from_date.read(), &date_iso_format).unwrap_or(date!(1970 - 01 - 01));
        let to_parsed =
            time::Date::parse(&to_date.read(), &date_iso_format).unwrap_or(date!(1970 - 01 - 01));
        // Pitfall 9 / W-7: `id` and `version` MUST be `Uuid::nil()` on Create
        // (the api-layer also defends this; we keep it explicit here for
        // documentation + grep-able audit).
        let id = match &editing_period_for_submit {
            Some(a) => a.id,
            None => Uuid::nil(),
        };
        let version = match &editing_period_for_submit {
            Some(a) => a.version,
            None => Uuid::nil(),
        };
        let category_to: AbsenceCategoryTO = (&*category.read()).into();
        let body = AbsencePeriodTO {
            id,
            sales_person_id: *employee_id.read(),
            category: category_to,
            from_date: from_parsed,
            to_date: to_parsed,
            description: description.read().clone().into(),
            created: None,
            deleted: None,
            version,
            // Phase 8.3 — Halbtag-Support. Form-Signal wird hier durchgereicht.
            day_fraction: (&*day_fraction.read()).into(),
            // Read-only Anzeige-Feld; auf Create/Update vom Backend ignoriert.
            derived_days: 0.0,
        };
        let action = match &mode_for_submit {
            AbsenceModalMode::Create => AbsenceAction::Create(body),
            AbsenceModalMode::Edit(_) => AbsenceAction::Update(body),
        };
        if let Some(svc) = &absence_service_for_submit {
            svc.send(action);
        }
    };

    let absence_service_for_reload = absence_service.clone();
    let editing_for_reload = editing_period.clone();
    let on_reload = move |_| {
        // 409 reload — re-fetch the affected list. For HR variant we do not
        // know the SalesPerson list here (would require a prop / global
        // signal); the page-level refresh-token (bumped via ABSENCE_REFRESH)
        // handles the global reload. We dispatch a per-sales-person fetch if
        // we are editing a known absence so the user sees the latest version
        // immediately.
        conflict_open.set(false);
        if let Some(absence) = &editing_for_reload {
            if let Some(svc) = &absence_service_for_reload {
                svc.send(AbsenceAction::LoadForSalesPerson(absence.sales_person_id));
            }
        }
    };

    let date_iso_format_clone1 = date_iso_format;
    let date_iso_format_clone2 = date_iso_format;

    let footer = rsx! {
        if is_edit {
            Btn {
                variant: BtnVariant::Danger,
                on_click: move |_| on_delete_request.call(()),
                "{delete_label}"
            }
        }
        span { class: "flex-1" }
        Btn {
            variant: BtnVariant::Ghost,
            on_click: move |_| on_close_for_cancel.call(()),
            "{cancel_label}"
        }
        Btn {
            variant: BtnVariant::Primary,
            on_click: on_submit.clone(),
            disabled: submit_disabled && !has_warnings,
            "{submit_label}"
        }
    };

    let conflict_open_now = *conflict_open.read();
    let validation_payload_now = validation_payload.read().clone();
    let warnings_now = warnings_state.read().clone();
    let form_disabled = has_warnings;

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_close_for_dialog.call(()),
            title: dialog_title,
            subtitle: Some(title),
            variant: DialogVariant::Center,
            width: 520,
            footer: Some(footer),

            div { class: "grid grid-cols-2 gap-3",
                if conflict_open_now {
                    div { class: "col-span-2",
                        VersionConflictBanner { on_reload: on_reload.clone() }
                    }
                }
                if let Some(payload) = validation_payload_now.clone() {
                    div { class: "col-span-2",
                        SelfOverlapBanner { raw_payload: payload }
                    }
                }
                Field {
                    label: employee_label,
                    span: Some(2u8),
                    SelectInput {
                        disabled: form_disabled || (!props.is_hr),
                        on_change: move |value: ImStr| {
                            if let Ok(parsed) = Uuid::parse_str(value.as_str()) {
                                employee_id.set(parsed);
                            }
                        },
                        for sp in props.sales_persons.iter().filter(|sp| is_selectable_employee(sp)) {
                            option {
                                value: "{sp.id}",
                                selected: sp.id == *employee_id.read(),
                                "{sp.name}"
                            }
                        }
                    }
                }
                Field {
                    label: category_label,
                    span: Some(2u8),
                    SelectInput {
                        disabled: form_disabled,
                        on_change: move |value: ImStr| {
                            let next = match value.as_str() {
                                "vacation" => AbsenceCategory::Vacation,
                                "sick_leave" => AbsenceCategory::SickLeave,
                                "unpaid_leave" => AbsenceCategory::UnpaidLeave,
                                _ => AbsenceCategory::Vacation,
                            };
                            category.set(next);
                        },
                        option {
                            value: "vacation",
                            selected: *category.read() == AbsenceCategory::Vacation,
                            "{i18n.t(Key::AbsenceCategoryVacation)}"
                        }
                        if SICK_LEAVE_ENABLED {
                            option {
                                value: "sick_leave",
                                selected: *category.read() == AbsenceCategory::SickLeave,
                                "{i18n.t(Key::AbsenceCategorySickLeave)}"
                            }
                        }
                        option {
                            value: "unpaid_leave",
                            selected: *category.read() == AbsenceCategory::UnpaidLeave,
                            "{i18n.t(Key::AbsenceCategoryUnpaidLeave)}"
                        }
                    }
                }
                // Phase 8.3: Tageshälfte (Full / Half). Full-width Field —
                // semantically a data property like Kategorie, not a CTA.
                Field {
                    label: day_fraction_label,
                    span: Some(2u8),
                    hint: Some(day_fraction_hint.clone()),
                    SelectInput {
                        disabled: form_disabled,
                        on_change: move |value: ImStr| {
                            let next = match value.as_str() {
                                "Half" => DayFraction::Half,
                                _ => DayFraction::Full,
                            };
                            day_fraction.set(next);
                        },
                        option {
                            value: "Full",
                            selected: *day_fraction.read() == DayFraction::Full,
                            "{day_fraction_full_label}"
                        }
                        option {
                            value: "Half",
                            selected: *day_fraction.read() == DayFraction::Half,
                            "{day_fraction_half_label}"
                        }
                    }
                }
                Field {
                    label: from_label,
                    TextInput {
                        value: ImStr::from(from_date.read().as_str()),
                        input_type: ImStr::from("date"),
                        disabled: form_disabled,
                        on_change: move |value: ImStr| from_date.set(value.as_str().to_string()),
                    }
                }
                Field {
                    label: to_label,
                    error: range_error.clone(),
                    TextInput {
                        value: ImStr::from(to_date.read().as_str()),
                        input_type: ImStr::from("date"),
                        disabled: form_disabled,
                        on_change: move |value: ImStr| to_date.set(value.as_str().to_string()),
                    }
                }
                Field {
                    label: description_label,
                    span: Some(2u8),
                    hint: Some(description_hint),
                    TextareaInput {
                        value: ImStr::from(description.read().as_str()),
                        disabled: form_disabled,
                        on_change: move |value: ImStr| description.set(value.as_str().to_string()),
                    }
                }
                if !warnings_now.is_empty() {
                    div { class: "col-span-2",
                        WarningList { warnings: warnings_now.clone(), dense: false }
                    }
                }
            }
        }
    }
}

// ─── AbsenceFilterBar ─────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct AbsenceFilterBarProps {
    pub is_hr: bool,
    pub sales_persons: Rc<[SalesPerson]>,
    #[props(!optional, default = None)]
    pub category_filter: Option<AbsenceCategory>,
    pub on_category_change: EventHandler<Option<AbsenceCategory>>,
    #[props(!optional, default = None)]
    pub person_filter: Option<Uuid>,
    pub on_person_change: EventHandler<Option<Uuid>>,
    #[props(!optional, default = None)]
    pub status_filter: Option<AbsenceStatus>,
    pub on_status_change: EventHandler<Option<AbsenceStatus>>,
    pub show_past: bool,
    pub on_show_past_change: EventHandler<bool>,
    pub filtered_count: usize,
    pub total_count: usize,
}

#[component]
pub fn AbsenceFilterBar(props: AbsenceFilterBarProps) -> Element {
    let i18n = I18N.read().clone();
    let category_value = match props.category_filter {
        None => "all",
        Some(AbsenceCategory::Vacation) => "vacation",
        Some(AbsenceCategory::SickLeave) => "sick_leave",
        Some(AbsenceCategory::UnpaidLeave) => "unpaid_leave",
    };
    let status_value = match props.status_filter {
        None => "all",
        Some(AbsenceStatus::Active) => "active",
        Some(AbsenceStatus::Planned) => "planned",
        Some(AbsenceStatus::Finished) => "finished",
    };
    let person_value: String = match props.person_filter {
        None => "all".to_string(),
        Some(uuid) => uuid.to_string(),
    };
    let counter = i18n
        .t(Key::AbsenceFilterCounter)
        .as_ref()
        .replace("{n}", &props.filtered_count.to_string())
        .replace("{m}", &props.total_count.to_string());
    let on_category = props.on_category_change;
    let on_person = props.on_person_change;
    let on_status = props.on_status_change;
    let on_show_past = props.on_show_past_change;
    let show_past = props.show_past;
    // Plan 08-07 Task 5: Filter-Bar — Mobile vertikal-stacked (jeder Filter
    // füllt die volle Breite), ab `md` in eine Zeile mit `flex-wrap`-Fallback
    // für sehr schmale Desktop-Fenster.
    rsx! {
        div { class: "bg-surface border border-border rounded-lg px-3.5 py-2.5 flex flex-col gap-2 md:flex-row md:flex-wrap md:items-center md:gap-2.5",
            label { class: "flex items-center gap-2",
                span { class: "text-micro uppercase text-ink-muted font-semibold",
                    "{i18n.t(Key::AbsenceFilterCategoryLabel)}"
                }
                SelectInput {
                    on_change: move |value: ImStr| {
                        let next = match value.as_str() {
                            "vacation" => Some(AbsenceCategory::Vacation),
                            "sick_leave" => Some(AbsenceCategory::SickLeave),
                            "unpaid_leave" => Some(AbsenceCategory::UnpaidLeave),
                            _ => None,
                        };
                        on_category.call(next);
                    },
                    option { value: "all", selected: category_value == "all",
                        "{i18n.t(Key::AbsenceFilterCategoryAll)}"
                    }
                    option { value: "vacation", selected: category_value == "vacation",
                        "{i18n.t(Key::AbsenceCategoryVacation)}"
                    }
                    if SICK_LEAVE_ENABLED {
                        option { value: "sick_leave", selected: category_value == "sick_leave",
                            "{i18n.t(Key::AbsenceCategorySickLeave)}"
                        }
                    }
                    option { value: "unpaid_leave", selected: category_value == "unpaid_leave",
                        "{i18n.t(Key::AbsenceCategoryUnpaidLeave)}"
                    }
                }
            }
            span { class: "w-px h-[22px] bg-border mx-1" }
            if props.is_hr {
                label { class: "flex items-center gap-2",
                    span { class: "text-micro uppercase text-ink-muted font-semibold",
                        "{i18n.t(Key::AbsenceFilterPersonLabel)}"
                    }
                    SelectInput {
                        on_change: move |value: ImStr| {
                            let next = if value.as_str() == "all" {
                                None
                            } else {
                                Uuid::parse_str(value.as_str()).ok()
                            };
                            on_person.call(next);
                        },
                        option { value: "all", selected: person_value == "all",
                            "{i18n.t(Key::AbsenceFilterPersonAll)}"
                        }
                        for sp in props.sales_persons.iter().filter(|sp| is_selectable_employee(sp)) {
                            option {
                                value: "{sp.id}",
                                selected: person_value == sp.id.to_string(),
                                "{sp.name}"
                            }
                        }
                    }
                }
                span { class: "w-px h-[22px] bg-border mx-1" }
            }
            label { class: "flex items-center gap-2",
                span { class: "text-micro uppercase text-ink-muted font-semibold",
                    "{i18n.t(Key::AbsenceFilterStatusLabel)}"
                }
                SelectInput {
                    on_change: move |value: ImStr| {
                        let next = match value.as_str() {
                            "active" => Some(AbsenceStatus::Active),
                            "planned" => Some(AbsenceStatus::Planned),
                            "finished" => Some(AbsenceStatus::Finished),
                            _ => None,
                        };
                        on_status.call(next);
                    },
                    option { value: "all", selected: status_value == "all",
                        "{i18n.t(Key::AbsenceFilterStatusAll)}"
                    }
                    option { value: "active", selected: status_value == "active",
                        "{i18n.t(Key::AbsenceStatusActive)}"
                    }
                    option { value: "planned", selected: status_value == "planned",
                        "{i18n.t(Key::AbsenceStatusPlanned)}"
                    }
                    option { value: "finished", selected: status_value == "finished",
                        "{i18n.t(Key::AbsenceStatusFinished)}"
                    }
                }
            }
            label { class: "ml-auto flex items-center gap-2 text-body text-ink-soft",
                input {
                    r#type: "checkbox",
                    checked: show_past,
                    onchange: move |e| {
                        let v = e.value() == "true";
                        on_show_past.call(v);
                    },
                }
                span { "{i18n.t(Key::AbsenceFilterShowPast)}" }
            }
            span { class: "text-small text-ink-muted",
                "{counter}"
            }
        }
    }
}

// ─── StatsGrid ────────────────────────────────────────────────────────────

/// Pure function — compute (sick_days, unpaid_days, active_count) for a slice
/// of AbsencePeriod rows, optionally scoped to a single person (D1).
///
/// When `person_filter` is `Some(uuid)`, only rows with a matching
/// `sales_person_id` are counted. Category and status filters are NOT applied
/// here — D1 mandates person-dimension ONLY so that the stat boxes remain
/// meaningful independent metrics.
pub fn stats_for_person(
    absences: &[AbsencePeriod],
    person_filter: Option<Uuid>,
    year: u32,
    today: time::Date,
) -> (i64, i64, usize) {
    let mut sick_days: i64 = 0;
    let mut unpaid_days: i64 = 0;
    let mut active_count: usize = 0;
    for absence in absences.iter() {
        if let Some(p) = person_filter {
            if absence.sales_person_id != p {
                continue;
            }
        }
        let inclusive_days =
            (absence.to_date - absence.from_date).whole_days().max(0) + 1;
        let in_year = absence.from_date.year() == year as i32
            || absence.to_date.year() == year as i32;
        match absence.category {
            AbsenceCategory::SickLeave if in_year => sick_days += inclusive_days,
            AbsenceCategory::UnpaidLeave if in_year => unpaid_days += inclusive_days,
            _ => {}
        }
        if compute_status(absence.from_date, absence.to_date, today) == AbsenceStatus::Active {
            active_count += 1;
        }
    }
    (sick_days, unpaid_days, active_count)
}

#[derive(Props, Clone, PartialEq)]
pub struct StatsGridProps {
    pub absences: Rc<[AbsencePeriod]>,
    pub year: u32,
    pub today: time::Date,
    #[props(!optional, default = None)]
    pub person_filter: Option<Uuid>,
}

#[component]
pub fn StatsGrid(props: StatsGridProps) -> Element {
    let i18n = I18N.read().clone();
    let (sick_days, unpaid_days, active_count) =
        stats_for_person(&props.absences, props.person_filter, props.year, props.today);
    let year_str = props.year.to_string();
    let sick_label = i18n
        .t(Key::AbsenceStatSickLeaveDays)
        .as_ref()
        .replace("{year}", &year_str);
    let unpaid_label = i18n
        .t(Key::AbsenceStatUnpaidDays)
        .as_ref()
        .replace("{year}", &year_str);
    let active_label = i18n.t(Key::AbsenceStatActive).as_ref().to_string();
    // Plan 08-07 Task 5: Explizite Breakpoint-Steps statt auto-fit, damit das
    // Desktop-Layout deterministisch zwei oder drei Spalten zeigt — der
    // existierende `auto-fit,minmax(160px,1fr)` faltete je nach Container-
    // Breite zu unschönen 1-col-Fallbacks.
    // STATIC Tailwind (Pitfall 5): beide Klassenstrings müssen wörtlich im
    // Quelltext stehen, damit der Tailwind-Scanner sie findet.
    let grid_class = if SICK_LEAVE_ENABLED {
        "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2.5"
    } else {
        "grid grid-cols-1 sm:grid-cols-2 gap-2.5"
    };
    rsx! {
        div { class: "{grid_class}",
            if SICK_LEAVE_ENABLED {
                StatBox {
                    label: ImStr::from(sick_label.as_str()),
                    value: ImStr::from(format!("{}", sick_days).as_str()),
                }
            }
            StatBox {
                label: ImStr::from(unpaid_label.as_str()),
                value: ImStr::from(format!("{}", unpaid_days).as_str()),
            }
            StatBox {
                label: ImStr::from(active_label.as_str()),
                value: ImStr::from(format!("{}", active_count).as_str()),
            }
        }
    }
}

// ─── AbsenceList ───────────────────────────────────────────────────────────

/// Gemischtes List-Item: Range-basierte Abwesenheitsperiode oder
/// stundenbasierter Marker (noch nicht konvertiert).
enum AbsenceListItem {
    Range(AbsencePeriod),
    Marker(ExtraHoursMarker),
}

impl AbsenceListItem {
    /// Datum für die chronologische Sortierung.
    fn sort_date(&self) -> time::Date {
        match self {
            AbsenceListItem::Range(a) => a.from_date,
            AbsenceListItem::Marker(m) => m.when,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AbsenceListProps {
    pub rows: Rc<[AbsencePeriod]>,
    /// Stundenbasierte Marker (aus `ABSENCE_HOURLY_STORE`).
    pub hourly_markers: Rc<[ExtraHoursMarker]>,
    pub is_hr: bool,
    pub today: time::Date,
    pub filter_active: bool,
    pub on_row_click: EventHandler<AbsencePeriod>,
    /// „Stunden bearbeiten" (self+hr) — öffnet ExtraHoursModal.
    pub on_edit_hours: EventHandler<ExtraHoursMarker>,
    /// „In Zeitraum umwandeln" (HR-only) — öffnet AbsenceConvertModal.
    pub on_convert: EventHandler<ExtraHoursMarker>,
}

#[component]
pub fn AbsenceList(props: AbsenceListProps) -> Element {
    let i18n = I18N.read().clone();

    // Empty-state: nur wenn BEIDE Quellen leer sind.
    if props.rows.is_empty() && props.hourly_markers.is_empty() {
        let (heading_key, body_key) = if props.filter_active {
            (Key::AbsenceEmptyFilterHeading, Key::AbsenceEmptyFilterBody)
        } else if props.is_hr {
            // HR with no entries at all + no filter → reuse the filter copy
            // (no dedicated "no entries at all for HR" key in UI-SPEC).
            (Key::AbsenceEmptyFilterHeading, Key::AbsenceEmptyFilterBody)
        } else {
            (Key::AbsenceEmptySelfHeading, Key::AbsenceEmptySelfBody)
        };
        return rsx! {
            div { class: "bg-surface border border-border rounded-lg overflow-hidden",
                div { class: "py-12 px-6 text-center",
                    div { class: "text-lg text-ink font-semibold mb-2", "{i18n.t(heading_key)}" }
                    div { class: "text-body text-ink-muted", "{i18n.t(body_key)}" }
                }
            }
        };
    }

    // Chronologischer Merge beider Quellen.
    let mut items: Vec<AbsenceListItem> = Vec::new();
    for row in props.rows.iter() {
        items.push(AbsenceListItem::Range(row.clone()));
    }
    for marker in props.hourly_markers.iter() {
        items.push(AbsenceListItem::Marker(marker.clone()));
    }
    items.sort_by_key(|item| item.sort_date());

    rsx! {
        div { class: "bg-surface border border-border rounded-lg overflow-hidden",
            // Plan 08-07 Task 5: Header bleibt nur ab `md` sichtbar — auf
            // Mobile zeigt jede Row die Felder gestapelt mit eigenen Labels
            // (kein redundanter Spaltenkopf).
            div { class: "hidden md:grid bg-surface-alt border-b border-border px-4 py-2 grid-cols-[1.5fr_170px_140px_90px_70px] gap-3.5 text-micro text-ink-muted uppercase font-semibold",
                div { "{i18n.t(Key::AbsenceColEmployee)}" }
                div { "{i18n.t(Key::AbsenceColRange)}" }
                div { "{i18n.t(Key::AbsenceColCategory)}" }
                div { "{i18n.t(Key::AbsenceColStatus)}" }
                div { "{i18n.t(Key::AbsenceColWarnings)}" }
            }
            for item in items.iter() {
                match item {
                    AbsenceListItem::Range(row) => rsx! {
                        AbsenceListRow {
                            absence: row.clone(),
                            today: props.today,
                            on_click: props.on_row_click,
                        }
                    },
                    AbsenceListItem::Marker(marker) => rsx! {
                        HourlyMarkerRow {
                            marker: marker.clone(),
                            is_hr: props.is_hr,
                            on_edit: props.on_edit_hours,
                            on_convert: props.on_convert,
                        }
                    },
                }
            }
        }
    }
}

// ─── HourlyMarkerRow ───────────────────────────────────────────────────────
//
// Read-only Row für noch nicht konvertierte stundenbasierte extra_hours-Rows
// (Vacation/SickLeave/UnpaidLeave), die inline in der Absence-Liste erscheinen.
// Zeigt Datum, Stunden, Kategorie-Badge + „stundenbasiert"-Badge.
// Zwei Affordances:
//   - „Stunden bearbeiten" (self+hr, immer sichtbar) → on_edit
//   - „In Zeitraum umwandeln" (HR-only) → on_convert

#[derive(Props, Clone, PartialEq)]
pub struct HourlyMarkerRowProps {
    pub marker: ExtraHoursMarker,
    pub is_hr: bool,
    pub on_edit: EventHandler<ExtraHoursMarker>,
    pub on_convert: EventHandler<ExtraHoursMarker>,
}

#[component]
pub fn HourlyMarkerRow(props: HourlyMarkerRowProps) -> Element {
    use rest_types::ExtraHoursCategoryTO;

    let i18n = I18N.read().clone();
    let marker = props.marker.clone();
    let marker_for_edit = marker.clone();
    let marker_for_convert = marker.clone();

    // Pitfall 5: STATIC Tailwind match arms — KEIN format!() für text-/bg-Klassen.
    let (text_class, bg_class) = match &marker.category {
        ExtraHoursCategoryTO::Vacation => ("text-good", "bg-good-soft"),
        ExtraHoursCategoryTO::SickLeave => ("text-warn", "bg-warn-soft"),
        ExtraHoursCategoryTO::UnpaidLeave => ("text-ink-muted", "bg-surface-2"),
        // Sonstige Kategorien (ExtraWork, Holiday etc.) — neutrale Farbe.
        _ => ("text-ink-muted", "bg-surface-2"),
    };
    let category_label = match &marker.category {
        ExtraHoursCategoryTO::Vacation => i18n.t(Key::AbsenceCategoryVacation),
        ExtraHoursCategoryTO::SickLeave => i18n.t(Key::AbsenceCategorySickLeave),
        ExtraHoursCategoryTO::UnpaidLeave => i18n.t(Key::AbsenceCategoryUnpaidLeave),
        _ => i18n.t(Key::CategoryExtraWork),
    };

    let badge_label = i18n.t(Key::AbsenceHourlyBadge);
    let amount_label = i18n.t(Key::AbsenceHourlyAmountLabel);
    let edit_label = i18n.t(Key::AbsenceEditHoursAction);
    let convert_label = i18n.t(Key::AbsenceConvertToRangeAction);
    let when_str = marker.when.to_string();
    let amount_str = format!("{:.2}", marker.amount);
    let person_name = marker.person_name.as_ref();
    let description = marker.description.as_ref();

    rsx! {
        // Gleiche Grid-Struktur wie AbsenceListRow: md:grid-cols-[1.5fr_170px_140px_90px_70px]
        div {
            class: "w-full flex flex-col gap-2 md:grid md:grid-cols-[1.5fr_170px_140px_90px_70px] md:gap-3.5 px-4 py-3.5 border-t border-border bg-surface-alt/30",
            // Spalte 1: Person + Description (T-8-XSS-01: via RSX auto-escape)
            div { class: "flex flex-col gap-0.5 min-w-0",
                span { class: "text-body font-semibold text-ink truncate",
                    if person_name.is_empty() {
                        "—"
                    } else {
                        "{person_name}"
                    }
                }
                if !description.is_empty() {
                    span { class: "text-small text-ink-muted truncate",
                        "{description}"
                    }
                }
            }
            // Spalte 2: Datum + Stunden
            div { class: "text-body text-ink font-mono flex flex-col gap-0.5",
                span { "{when_str}" }
                span { class: "text-small text-ink-muted",
                    "{amount_str} {amount_label}"
                }
            }
            // Spalte 3: Kategorie-Badge + „stundenbasiert"-Badge
            div { class: "flex flex-col gap-1",
                span {
                    class: "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-small font-semibold {text_class} {bg_class}",
                    span { class: "w-1.5 h-1.5 rounded-full bg-current" }
                    "{category_label}"
                }
                span {
                    class: "inline-flex items-center rounded-full px-2 py-0.5 text-micro font-semibold text-ink-muted bg-surface-2",
                    "{badge_label}"
                }
            }
            // Spalte 4: leer (kein Status für Marker)
            div {}
            // Spalte 5: Aktions-Buttons
            div { class: "flex flex-col gap-1 items-end",
                // „Stunden bearbeiten" (self+hr, D-08) — immer sichtbar
                button {
                    r#type: "button",
                    class: "text-small text-accent hover:underline whitespace-nowrap",
                    onclick: move |_| props.on_edit.call(marker_for_edit.clone()),
                    "{edit_label}"
                }
                // „In Zeitraum umwandeln" (HR-only, D-09)
                if props.is_hr {
                    button {
                        r#type: "button",
                        class: "text-small text-accent hover:underline whitespace-nowrap",
                        onclick: move |_| props.on_convert.call(marker_for_convert.clone()),
                        "{convert_label}"
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct AbsenceListRowProps {
    absence: AbsencePeriod,
    today: time::Date,
    on_click: EventHandler<AbsencePeriod>,
}

#[component]
fn AbsenceListRow(props: AbsenceListRowProps) -> Element {
    let i18n = I18N.read().clone();
    let absence = props.absence.clone();
    let status = compute_status(absence.from_date, absence.to_date, props.today);
    // Echte Urlaubs-/Abwesenheitstage (aktive Arbeitstage im Range × Day-Fraction)
    // vom Backend (derive_hours_for_range) statt der reinen Kalenderspanne. So
    // zeigt ein Mo–Mi-Zeitraum bei 2 Arbeitstagen/Woche "2 Tage", nicht "3".
    let derived_days = absence.derived_days;
    let days_label = format_decimal(derived_days);
    let unit_key = if (derived_days - 1.0).abs() < f32::EPSILON {
        Key::AbsenceDayUnit
    } else {
        Key::AbsenceDaysUnit
    };
    let on_click = props.on_click;
    let absence_for_click = absence.clone();
    let from_str = format!("{}", absence.from_date);
    let to_str = format!("{}", absence.to_date);
    let person_name = absence.person_name.as_ref();
    rsx! {
        // Plan 08-07 Task 5: Auf Mobile vertikal-stacked, ab `md` als 5-spaltiges
        // Grid (passend zum Header oben). Gap auf Mobile knapp, ab `md` größer.
        button {
            r#type: "button",
            class: "w-full text-left flex flex-col gap-2 md:grid md:grid-cols-[1.5fr_170px_140px_90px_70px] md:gap-3.5 px-4 py-3.5 border-t border-border hover:bg-surface-alt focus:bg-surface-alt focus:outline-none",
            onclick: move |_| on_click.call(absence_for_click.clone()),
            div { class: "flex flex-col gap-0.5 min-w-0",
                span { class: "text-body font-semibold text-ink truncate",
                    if person_name.is_empty() {
                        "—"
                    } else {
                        "{person_name}"
                    }
                }
                if !absence.description.as_ref().is_empty() {
                    // T-8-XSS-01: description is rendered via rsx interpolation
                    // (Dioxus auto-escape). NEVER use raw HTML injection.
                    span { class: "text-small text-ink-muted truncate",
                        "{absence.description}"
                    }
                }
            }
            div { class: "text-body text-ink font-mono flex flex-col gap-0.5",
                span { "{from_str} – {to_str}" }
                span { class: "text-small text-ink-muted",
                    "{days_label} {i18n.t(unit_key)}"
                }
            }
            div { CategoryBadge { category: absence.category } }
            div { StatusPill { status: status } }
            div { class: "text-small text-ink-muted text-right", "›" }
        }
    }
}

// ─── AbsencesPage (Top-Level component) ───────────────────────────────────

#[component]
pub fn AbsencesPage() -> Element {
    let auth = AUTH.read().clone();
    let i18n = I18N.read().clone();

    // Pitfall 4 — AUTH-loading-Gate before we branch on `is_hr`. Without this,
    // the page can dispatch one action against the Employee API and a second
    // against the HR API (Auth resolves in two render-cycles).
    if !auth.loading_done {
        let loading_label = i18n.t(Key::SearchPlaceholder);
        let _ = loading_label;
        return rsx! {
            TopBar {}
            div { class: "p-md text-ink-muted", "Loading..." }
        };
    }
    let is_hr = auth
        .auth_info
        .as_ref()
        .map(|a| a.has_privilege("hr"))
        .unwrap_or(false);

    let absence_service = use_coroutine_handle::<AbsenceAction>();
    let vacation_service = use_coroutine_handle::<VacationBalanceAction>();
    let today = current_date_for_init();
    let mut selected_year = use_signal(current_year_for_init);
    let year = *selected_year.read();

    // Resolved Self-User (only relevant for the Employee variant).
    let mut current_sp_id = use_signal(|| None::<Uuid>);
    let mut sales_persons = use_signal(|| Rc::<[SalesPerson]>::from([]));

    // Bootstrap loaders (sales-person list + current-user) at mount.
    use_effect(move || {
        spawn(async move {
            let config = CONFIG.read().clone();
            if let Ok(persons) = loader::load_sales_persons(config.clone()).await {
                sales_persons.set(persons);
            }
            if let Ok(Some(sp)) = loader::load_current_sales_person(config).await {
                current_sp_id.set(Some(sp.id));
            }
        });
    });

    // Refresh-token-driven absence + vacation re-fetch (D-09 branch + D-12
    // forward-warnings flow). The token is bumped from the absence service
    // on every successful POST/PUT/DELETE/Convert.
    //
    // CRITICAL (debug convert-absence-name-and-refresh): every signal the
    // effect must react to MUST be read INSIDE the closure. In Dioxus 0.6 a
    // `use_effect` only subscribes to signals read within its closure body;
    // reading them outside and capturing the value (a snapshot) does NOT
    // subscribe. Previously `ABSENCE_REFRESH`, `sales_persons` and
    // `current_sp_id` were read outside, so the effect only re-fired on
    // `selected_year` changes. That broke two flows:
    //   - convert (and any mutation) bumped ABSENCE_REFRESH but the list never
    //     reloaded (no auto-refresh), and
    //   - the bootstrap loaded `sales_persons` asynchronously, but the effect
    //     had already fired LoadAll with an empty list, so the name-join
    //     failed and employee names showed as "—".
    // Reading all four signals inside the closure subscribes the effect to
    // each, so it re-fires on refresh bumps, sales-person load, self-user
    // resolution, and year changes alike.
    use_effect(move || {
        // Subscribe to all reactive sources by reading them in-closure.
        let _ = *ABSENCE_REFRESH.read();
        let sales_persons_for_effect = sales_persons.read().clone();
        let current_sp_for_effect = *current_sp_id.read();
        let year_for_effect = *selected_year.read();
        if is_hr {
            // HR: load all absences + team vacation aggregate.
            absence_service.send(AbsenceAction::LoadAll(sales_persons_for_effect));
            vacation_service.send(VacationBalanceAction::LoadTeam(year_for_effect));
        } else if let Some(sp) = current_sp_for_effect {
            absence_service.send(AbsenceAction::LoadForSalesPerson(sp));
            vacation_service.send(VacationBalanceAction::LoadSelf(sp, year_for_effect));
        }
    });

    // Modal + filter state (page-local).
    let mut modal_open = use_signal(|| false);
    let mut modal_mode = use_signal(|| AbsenceModalMode::Create);
    let mut delete_open = use_signal(|| false);
    let mut delete_target = use_signal(|| None::<Uuid>);
    let mut category_filter = use_signal(|| None::<AbsenceCategory>);
    let mut person_filter = use_signal(|| None::<Uuid>);
    let mut status_filter = use_signal(|| None::<AbsenceStatus>);
    let mut show_past = use_signal(|| true);

    let absences = ABSENCE_STORE.read().clone();
    let hourly_markers = ABSENCE_HOURLY_STORE.read().clone();
    let vacation_self = VACATION_BALANCE_STORE.read().clone();
    let vacation_team = VACATION_TEAM_STORE.read().clone();

    // Signale für die Marker-Modals (Task 3 verdrahtet die vollständige Logik).
    let mut convert_target = use_signal(|| None::<ExtraHoursMarker>);
    let mut edit_hours_target = use_signal(|| None::<ExtraHoursMarker>);

    let category_filter_val = *category_filter.read();
    let person_filter_val = *person_filter.read();
    let status_filter_val = *status_filter.read();
    let show_past_val = *show_past.read();

    // Single-Choke-Point: ausgeblendete Kategorien (aktuell SickLeave, siehe
    // SICK_LEAVE_ENABLED) werden VOR der User-Filter-Pipeline entfernt, damit
    // Liste, Stats und Zähler („X von Y") konsistent nur sichtbare Einträge sehen.
    let visible_absences: Vec<AbsencePeriod> = absences
        .iter()
        .filter(|a| is_visible_category(a.category))
        .cloned()
        .collect();
    let visible_absences_rc: Rc<[AbsencePeriod]> = Rc::from(visible_absences);
    let visible_markers: Vec<ExtraHoursMarker> = hourly_markers
        .iter()
        .filter(|m| is_visible_marker_category(&m.category))
        .cloned()
        .collect();

    let filtered: Vec<AbsencePeriod> = visible_absences_rc
        .iter()
        .filter(|a| {
            if let Some(cat) = category_filter_val {
                if a.category != cat {
                    return false;
                }
            }
            if let Some(person) = person_filter_val {
                if a.sales_person_id != person {
                    return false;
                }
            }
            let status = compute_status(a.from_date, a.to_date, today);
            if let Some(s) = status_filter_val {
                if status != s {
                    return false;
                }
            }
            if !show_past_val && status == AbsenceStatus::Finished {
                return false;
            }
            true
        })
        .cloned()
        .collect();
    let filtered_rc: Rc<[AbsencePeriod]> = Rc::from(filtered);
    let filtered_markers: Vec<ExtraHoursMarker> = visible_markers
        .iter()
        .filter(|m| {
            marker_matches_filters(
                m,
                category_filter_val,
                person_filter_val,
                status_filter_val,
                show_past_val,
                today,
            )
        })
        .cloned()
        .collect();
    let filtered_markers_rc: Rc<[ExtraHoursMarker]> = Rc::from(filtered_markers);
    let total_count = visible_absences_rc.len() + visible_markers.len();
    let filtered_count = filtered_rc.len() + filtered_markers_rc.len();
    let filter_active = category_filter_val.is_some()
        || person_filter_val.is_some()
        || status_filter_val.is_some()
        || !show_past_val;

    let new_btn_label = ImStr::from(i18n.t(Key::AbsenceNewBtn).as_ref());
    let page_title = i18n.t(Key::AbsencePageTitle);
    let page_subtitle = i18n.t(Key::AbsencePageSubtitle);

    let sales_persons_for_modal = sales_persons.read().clone();
    let absence_service_for_delete = absence_service.clone();

    let on_new = move |_| {
        modal_mode.set(AbsenceModalMode::Create);
        modal_open.set(true);
    };
    let on_row_click = move |absence: AbsencePeriod| {
        modal_mode.set(AbsenceModalMode::Edit(absence));
        modal_open.set(true);
    };
    let on_delete_request = move |_| {
        if let AbsenceModalMode::Edit(a) = &*modal_mode.read() {
            delete_target.set(Some(a.id));
            delete_open.set(true);
        }
    };
    let on_delete_confirm = move |_| {
        let target = *delete_target.read();
        if let Some(id) = target {
            absence_service_for_delete.send(AbsenceAction::Delete(id));
            delete_open.set(false);
            delete_target.set(None);
            modal_open.set(false);
        }
    };

    // D-09: AbsenceConvertModal — vorbereitet vor rsx! (rsx! erlaubt keine
    // let-Bindungen in if-let-Blöcken).
    let convert_modal_element: Element = if let Some(m) = convert_target.read().clone() {
        // ExtraHoursCategoryTO → AbsenceCategoryTO Mapping für die 3 relevanten
        // Kategorien. Nur Vacation/SickLeave/UnpaidLeave können als Marker
        // auftreten; sonstige Kategorien fallen auf Vacation zurück (Backend
        // enforced dasselbe Gate).
        let modal_category = match &m.category {
            rest_types::ExtraHoursCategoryTO::SickLeave => rest_types::AbsenceCategoryTO::SickLeave,
            rest_types::ExtraHoursCategoryTO::UnpaidLeave => rest_types::AbsenceCategoryTO::UnpaidLeave,
            _ => rest_types::AbsenceCategoryTO::Vacation,
        };
        rsx! {
            AbsenceConvertModal {
                extra_hours_id: m.extra_hours_id,
                initial_date: m.when,
                amount: m.amount,
                category: modal_category,
                on_submit: move |(id, start, end, day_fraction)| {
                    absence_service.send(AbsenceAction::ConvertExtraHours {
                        extra_hours_id: id,
                        start,
                        end,
                        day_fraction,
                    });
                    convert_target.set(None);
                },
                on_cancel: move |_| convert_target.set(None),
            }
        }
    } else {
        rsx! {}
    };

    // D-08: Inline-Stunden-Edit — ExtraHoursModal in Edit-Mode.
    // Reuse des bestehenden extra_hours-Update-Pfads (PUT /extra-hours/{id}),
    // kein neuer API-Pfad (T-8.5-06c Mitigation).
    let edit_hours_modal_element: Element = if let Some(m) = edit_hours_target.read().clone() {
        // ExtraHoursCategoryTO → WorkingHoursCategory für ExtraHoursModal.
        let edit_category: WorkingHoursCategory = (&m.category).into();
        // Minimalen ExtraHours-Record für Edit-Mode konstruieren.
        // date_time: Marker hat nur `when: time::Date`; Midnight als Placeholder.
        let edit_entry = ExtraHours {
            id: m.extra_hours_id,
            sales_person_id: m.sales_person_id,
            amount: m.amount,
            category: edit_category,
            description: Rc::from(m.description.as_ref()),
            date_time: time::PrimitiveDateTime::new(m.when, time::Time::MIDNIGHT),
            version: Uuid::nil(),
        };
        rsx! {
            ExtraHoursModal {
                open: true,
                sales_person_id: m.sales_person_id,
                editing: Some(edit_entry),
                on_saved: move |_| {
                    // D-15 analog: ABSENCE_REFRESH bumpen damit die Liste neu lädt.
                    crate::service::absence::bump_absence_refresh();
                    edit_hours_target.set(None);
                },
                on_cancel: move |_| edit_hours_target.set(None),
            }
        }
    } else {
        rsx! {}
    };

    rsx! {
        TopBar {}
        ErrorView {}
        div { class: "p-4 md:p-6 flex flex-col gap-3",
            header { class: "flex items-start justify-between gap-3 flex-wrap",
                div { class: "flex flex-col gap-1 min-w-0",
                    h1 { class: "text-h1 font-semibold text-ink", "{page_title}" }
                    div { class: "text-body text-ink-muted", "{page_subtitle}" }
                }
                Btn {
                    variant: BtnVariant::Primary,
                    on_click: on_new.clone(),
                    "{new_btn_label}"
                }
            }
            // Year navigation — ◀ {year} ▶ — both HR and employee see it.
            // Clicking prev/next updates selected_year signal which re-fires
            // the use_effect above to reload vacation + stats for the new year.
            div { class: "flex items-center gap-2",
                button {
                    r#type: "button",
                    class: "px-2 py-1 rounded-md border border-border text-ink hover:bg-surface-alt",
                    title: "{i18n.t(Key::AbsenceYearNavPrev)}",
                    "aria-label": "{i18n.t(Key::AbsenceYearNavPrev)}",
                    onclick: move |_| {
                        let y = *selected_year.read();
                        selected_year.set(y.saturating_sub(1));
                    },
                    "◀"
                }
                span { class: "text-body font-semibold font-mono text-ink min-w-[3.5rem] text-center", "{year}" }
                button {
                    r#type: "button",
                    class: "px-2 py-1 rounded-md border border-border text-ink hover:bg-surface-alt",
                    title: "{i18n.t(Key::AbsenceYearNavNext)}",
                    "aria-label": "{i18n.t(Key::AbsenceYearNavNext)}",
                    onclick: move |_| {
                        let y = *selected_year.read();
                        selected_year.set(y + 1);
                    },
                    "▶"
                }
            }
            VacationEntitlementCard {
                is_hr: is_hr,
                year: year,
                vacation_self: vacation_self.clone(),
                vacation_team: vacation_team.clone(),
                sales_persons: sales_persons.read().clone(),
                selected_person: person_filter_val,
            }
            StatsGrid {
                absences: visible_absences_rc.clone(),
                year: year,
                today: today,
                person_filter: person_filter_val,
            }
            AbsenceFilterBar {
                is_hr: is_hr,
                sales_persons: sales_persons.read().clone(),
                category_filter: category_filter_val,
                on_category_change: move |v: Option<AbsenceCategory>| category_filter.set(v),
                person_filter: person_filter_val,
                on_person_change: move |v: Option<Uuid>| person_filter.set(v),
                status_filter: status_filter_val,
                on_status_change: move |v: Option<AbsenceStatus>| status_filter.set(v),
                show_past: show_past_val,
                on_show_past_change: move |v: bool| show_past.set(v),
                filtered_count: filtered_count,
                total_count: total_count,
            }
            AbsenceList {
                rows: filtered_rc.clone(),
                hourly_markers: filtered_markers_rc.clone(),
                is_hr: is_hr,
                today: today,
                filter_active: filter_active,
                on_row_click: on_row_click.clone(),
                on_edit_hours: move |m: ExtraHoursMarker| edit_hours_target.set(Some(m)),
                on_convert: move |m: ExtraHoursMarker| convert_target.set(Some(m)),
            }
        }
        if *modal_open.read() {
            AbsenceModal {
                open: true,
                mode: modal_mode.read().clone(),
                is_hr: is_hr,
                sales_persons: sales_persons_for_modal.clone(),
                current_sp_id: *current_sp_id.read(),
                on_close: move |_| modal_open.set(false),
                on_delete_request: on_delete_request.clone(),
            }
        }
        if *delete_open.read() {
            DeleteConfirmDialog {
                open: true,
                on_close: move |_| {
                    delete_open.set(false);
                    delete_target.set(None);
                },
                on_confirm: on_delete_confirm.clone(),
            }
        }
        // D-09: AbsenceConvertModal — HR-only, öffnet wenn convert_target Some ist.
        // Die Kategorie-Konvertierung und der ExtraHours-Record werden VOR dem
        // rsx!-Makro im konventionellen Rust-Code berechnet (rsx! unterstützt
        // keine let-Bindungen in if-let-Blöcken).
        {convert_modal_element}
        // D-08: Inline-Stunden-Edit — öffnet ExtraHoursModal wenn edit_hours_target Some ist.
        {edit_hours_modal_element}
    }
}


// ──────────────────────────────────────────────────────────────────────────
// Tests (Plan 05 Task 3) — 11 snapshot / pure-function tests covering
// CategoryBadge (3 categories), StatusPill (3 statuses), compute_status
// (3 boundary cases), and AbsenceFilterBar (HR + Employee variants).
//
// Render pattern is the verified one from `component/dialog.rs:461` —
// VirtualDom::new + rebuild_in_place + dioxus_ssr::render. The components
// pull copy from the global `I18N` signal; we set it to `Locale::De` once
// per test before rendering so reference strings (Urlaub, Krankheit, Aktiv,
// Person, …) match. The signal is process-global, so `cargo test` is
// allowed to run these in parallel safely only because each test rebuilds
// the VirtualDom synchronously from a closure — but to remove any chance
// of cross-test bleed we set the locale unconditionally inside each render.
// ──────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{generate, Locale};
    use std::sync::Arc;

    /// Render a snapshot component. Before rendering we install Locale::De
    /// into the global I18N signal via `use_hook`. The hook runs inside the
    /// Dioxus runtime (which `VirtualDom::new` provides), unlike a direct
    /// `*I18N.write() = …` outside any reactive scope which panics with a
    /// `RuntimeError`.
    ///
    /// Tests embed the locale-setter at the top of their `app` function:
    /// ```ignore
    /// fn app() -> Element {
    ///     pin_de_locale();
    ///     rsx! { CategoryBadge { … } }
    /// }
    /// ```
    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    /// Hook-based locale pin — must be invoked from inside a `#[component]`
    /// or rsx-generating function so it runs in a Dioxus reactive scope.
    fn pin_de_locale() {
        use_hook(|| {
            *I18N.write() = generate(Locale::De);
        });
    }

    // ── compute_status — pure function (Pitfall 8) ─────────────────────

    #[test]
    fn compute_status_today_before_from_returns_planned() {
        let today = date!(2026 - 05 - 08);
        let from = date!(2026 - 05 - 09);
        let to = date!(2026 - 05 - 13);
        assert_eq!(compute_status(from, to, today), AbsenceStatus::Planned);
    }

    #[test]
    fn compute_status_today_in_range_returns_active() {
        let today = date!(2026 - 05 - 08);
        let from = date!(2026 - 05 - 07);
        let to = date!(2026 - 05 - 09);
        assert_eq!(compute_status(from, to, today), AbsenceStatus::Active);
    }

    #[test]
    fn compute_status_today_after_to_returns_finished() {
        let today = date!(2026 - 05 - 08);
        let from = date!(2026 - 05 - 03);
        let to = date!(2026 - 05 - 07);
        assert_eq!(compute_status(from, to, today), AbsenceStatus::Finished);
    }

    // ── is_selectable_employee — pure function ─────────────────────────

    fn sales_person(is_paid: bool, inactive: bool) -> SalesPerson {
        SalesPerson {
            is_paid,
            inactive,
            ..Default::default()
        }
    }

    #[test]
    fn selectable_paid_and_active_returns_true() {
        assert!(is_selectable_employee(&sales_person(true, false)));
    }

    #[test]
    fn selectable_paid_but_inactive_returns_false() {
        assert!(!is_selectable_employee(&sales_person(true, true)));
    }

    #[test]
    fn selectable_unpaid_but_active_returns_false() {
        assert!(!is_selectable_employee(&sales_person(false, false)));
    }

    #[test]
    fn selectable_unpaid_and_inactive_returns_false() {
        assert!(!is_selectable_employee(&sales_person(false, true)));
    }

    // ── Sichtbarkeit von Kategorien (SICK_LEAVE_ENABLED) — pure functions ─

    #[test]
    fn category_visible_disabled_hides_sick_leave_only() {
        assert!(!category_visible_with(false, AbsenceCategory::SickLeave));
        assert!(category_visible_with(false, AbsenceCategory::Vacation));
        assert!(category_visible_with(false, AbsenceCategory::UnpaidLeave));
    }

    #[test]
    fn category_visible_enabled_shows_all_categories() {
        assert!(category_visible_with(true, AbsenceCategory::SickLeave));
        assert!(category_visible_with(true, AbsenceCategory::Vacation));
        assert!(category_visible_with(true, AbsenceCategory::UnpaidLeave));
    }

    #[test]
    fn marker_visibility_hides_sick_leave_keeps_vacation_and_unmappable() {
        use rest_types::ExtraHoursCategoryTO;
        assert!(!is_visible_marker_category(
            &ExtraHoursCategoryTO::SickLeave
        ));
        assert!(is_visible_marker_category(&ExtraHoursCategoryTO::Vacation));
        assert!(is_visible_marker_category(
            &ExtraHoursCategoryTO::UnpaidLeave
        ));
        // Unmappbare Kategorien (kein AbsenceCategory-Gegenstück) bleiben
        // sichtbar — bestehendes Verhalten der Marker-Pipeline.
        assert!(is_visible_marker_category(&ExtraHoursCategoryTO::ExtraWork));
    }

    // ── CategoryBadge snapshots (Pitfall 5: STATIC Tailwind) ───────────

    #[test]
    fn category_badge_renders_vacation_label() {
        fn app() -> Element {
            pin_de_locale();
            rsx! { CategoryBadge { category: AbsenceCategory::Vacation } }
        }
        let html = render(app);
        assert!(html.contains("Urlaub"), "missing label: {html}");
        assert!(html.contains("text-good"), "missing text-good: {html}");
        assert!(html.contains("bg-good-soft"), "missing bg-good-soft: {html}");
    }

    #[test]
    fn category_badge_renders_sick_leave_label() {
        fn app() -> Element {
            pin_de_locale();
            rsx! { CategoryBadge { category: AbsenceCategory::SickLeave } }
        }
        let html = render(app);
        assert!(html.contains("Krankheit"), "missing label: {html}");
        assert!(html.contains("text-warn"), "missing text-warn: {html}");
        assert!(html.contains("bg-warn-soft"), "missing bg-warn-soft: {html}");
    }

    #[test]
    fn category_badge_renders_unpaid_leave_label() {
        fn app() -> Element {
            pin_de_locale();
            rsx! { CategoryBadge { category: AbsenceCategory::UnpaidLeave } }
        }
        let html = render(app);
        assert!(html.contains("Unbezahlt"), "missing label: {html}");
        assert!(
            html.contains("text-ink-muted"),
            "missing text-ink-muted: {html}"
        );
        assert!(html.contains("bg-surface-2"), "missing bg-surface-2: {html}");
    }

    // ── StatusPill snapshots ───────────────────────────────────────────

    #[test]
    fn status_pill_renders_active() {
        fn app() -> Element {
            pin_de_locale();
            rsx! { StatusPill { status: AbsenceStatus::Active } }
        }
        let html = render(app);
        assert!(html.contains("Aktiv"), "missing label: {html}");
        assert!(html.contains("text-accent"), "missing text-accent: {html}");
    }

    #[test]
    fn status_pill_renders_planned() {
        fn app() -> Element {
            pin_de_locale();
            rsx! { StatusPill { status: AbsenceStatus::Planned } }
        }
        let html = render(app);
        assert!(html.contains("Geplant"), "missing label: {html}");
        assert!(
            html.contains("text-ink-soft"),
            "missing text-ink-soft: {html}"
        );
    }

    #[test]
    fn status_pill_renders_finished() {
        fn app() -> Element {
            pin_de_locale();
            rsx! { StatusPill { status: AbsenceStatus::Finished } }
        }
        let html = render(app);
        assert!(html.contains("Beendet"), "missing label: {html}");
        assert!(
            html.contains("text-ink-muted"),
            "missing text-ink-muted: {html}"
        );
    }

    // ── AbsenceFilterBar HR vs Employee variants ───────────────────────

    #[test]
    fn absence_filter_bar_hr_variant_renders_person_dropdown() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                AbsenceFilterBar {
                    is_hr: true,
                    sales_persons: Rc::<[SalesPerson]>::from([]),
                    category_filter: None,
                    on_category_change: |_| {},
                    person_filter: None,
                    on_person_change: |_| {},
                    status_filter: None,
                    on_status_change: |_| {},
                    show_past: true,
                    on_show_past_change: |_| {},
                    filtered_count: 0,
                    total_count: 0,
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Person"),
            "HR filter bar must render the Person dropdown label: {html}"
        );
    }

    #[test]
    fn absence_filter_bar_employee_variant_omits_person_dropdown() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                AbsenceFilterBar {
                    is_hr: false,
                    sales_persons: Rc::<[SalesPerson]>::from([]),
                    category_filter: None,
                    on_category_change: |_| {},
                    person_filter: None,
                    on_person_change: |_| {},
                    status_filter: None,
                    on_status_change: |_| {},
                    show_past: true,
                    on_show_past_change: |_| {},
                    filtered_count: 0,
                    total_count: 0,
                }
            }
        }
        let html = render(app);
        // Locale::De translates AbsenceFilterPersonLabel to "Person". The
        // employee variant must NOT render it.
        assert!(
            !html.contains(">Person<"),
            "Employee filter bar must NOT render the Person dropdown label: {html}"
        );
    }

    // ── SickLeave ausgeblendet (SICK_LEAVE_ENABLED=false) — SSR-Snapshots ─

    #[test]
    fn stats_grid_omits_sick_leave_card_when_disabled() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                StatsGrid {
                    absences: Rc::<[AbsencePeriod]>::from([]),
                    year: 2026u32,
                    today: date!(2026 - 06 - 12),
                    person_filter: None,
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("Krank"),
            "StatsGrid must NOT render the sick-leave card while SICK_LEAVE_ENABLED=false: {html}"
        );
    }

    #[test]
    fn absence_filter_bar_omits_sick_leave_option_when_disabled() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                AbsenceFilterBar {
                    is_hr: true,
                    sales_persons: Rc::<[SalesPerson]>::from([]),
                    category_filter: None,
                    on_category_change: |_| {},
                    person_filter: None,
                    on_person_change: |_| {},
                    status_filter: None,
                    on_status_change: |_| {},
                    show_past: true,
                    on_show_past_change: |_| {},
                    filtered_count: 0,
                    total_count: 0,
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("sick_leave"),
            "FilterBar must NOT render a sick_leave option while SICK_LEAVE_ENABLED=false: {html}"
        );
        assert!(
            html.contains("vacation") && html.contains("unpaid_leave"),
            "FilterBar must still offer vacation and unpaid_leave: {html}"
        );
    }

    #[test]
    fn absence_modal_omits_sick_leave_option_when_disabled() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                AbsenceModal {
                    open: true,
                    mode: AbsenceModalMode::Create,
                    is_hr: true,
                    sales_persons: Rc::<[SalesPerson]>::from([]),
                    current_sp_id: None,
                    on_close: |_| {},
                    on_delete_request: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("sick_leave"),
            "AbsenceModal must NOT render a sick_leave option while SICK_LEAVE_ENABLED=false: {html}"
        );
        assert!(
            html.contains("vacation") && html.contains("unpaid_leave"),
            "AbsenceModal must still offer vacation and unpaid_leave: {html}"
        );
    }

    // ── AbsenceModal Halbtag-Field snapshots (Phase 8.3 — Plan 06) ─────
    //
    // Verify the new `day_fraction` Field renders the Full/Half options
    // and the reactive hint text, plus that Edit-Mode pre-selects the
    // editing AbsencePeriod's day_fraction state.

    #[test]
    fn absence_modal_renders_day_fraction_select_with_full_option_active_by_default() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                AbsenceModal {
                    open: true,
                    mode: AbsenceModalMode::Create,
                    is_hr: true,
                    sales_persons: Rc::<[SalesPerson]>::from([]),
                    current_sp_id: None,
                    on_close: |_| {},
                    on_delete_request: |_| {},
                }
            }
        }
        let html = render(app);
        // Both Tageshälfte i18n De-labels are rendered as <option>-text.
        assert!(html.contains("Ganztag"), "Expected Ganztag label in: {html}");
        assert!(html.contains("Halber Tag"), "Expected Halber Tag label in: {html}");
        // value="Full" must be present and `selected` in the default Create-Mode.
        let full_option_idx = html
            .find("value=\"Full\"")
            .expect("Full option missing in rendered HTML");
        let full_window =
            &html[full_option_idx..(full_option_idx + 80).min(html.len())];
        assert!(
            full_window.contains("selected"),
            "Expected Full option to be selected by default: window={full_window}"
        );
        // Reactive hint — Full variant.
        assert!(
            html.contains("vollen Vertrags-Stundensatz"),
            "Expected Full hint text in: {html}"
        );
    }

    #[test]
    fn absence_modal_in_edit_mode_with_half_period_preselects_half() {
        fn app() -> Element {
            pin_de_locale();
            let editing = AbsencePeriod {
                id: Uuid::nil(),
                sales_person_id: Uuid::nil(),
                category: AbsenceCategory::Vacation,
                from_date: date!(2026 - 12 - 24),
                to_date: date!(2026 - 12 - 24),
                description: Arc::<str>::from("Heiligabend"),
                version: Uuid::nil(),
                day_fraction: DayFraction::Half,
                derived_days: 0.5,
                person_name: Arc::<str>::from(""),
                background_color: Arc::<str>::from(""),
            };
            rsx! {
                AbsenceModal {
                    open: true,
                    mode: AbsenceModalMode::Edit(editing),
                    is_hr: true,
                    sales_persons: Rc::<[SalesPerson]>::from([]),
                    current_sp_id: None,
                    on_close: |_| {},
                    on_delete_request: |_| {},
                }
            }
        }
        let html = render(app);
        let half_option_idx = html
            .find("value=\"Half\"")
            .expect("Half option missing in rendered HTML");
        let half_window =
            &html[half_option_idx..(half_option_idx + 80).min(html.len())];
        assert!(
            half_window.contains("selected"),
            "Expected Half option to be selected in Edit-Mode: window={half_window}"
        );
        // Reactive hint — Half variant.
        assert!(
            html.contains("0,5 Urlaubstage"),
            "Expected Half hint text in: {html}"
        );
    }

    // ── HourlyMarkerRow SSR-Tests (Phase 8.5 Plan 06) ─────────────────
    //
    // Render `HourlyMarkerRow` direkt mit Beispiel-Props (is_hr=true / false).
    // Deckt D-07 (Badge), D-08 (Edit-Button für alle), D-09 (Convert nur HR).

    fn sample_marker() -> ExtraHoursMarker {
        use rest_types::ExtraHoursCategoryTO;
        use std::sync::Arc;
        ExtraHoursMarker {
            extra_hours_id: Uuid::nil(),
            sales_person_id: Uuid::nil(),
            when: date!(2026 - 06 - 15),
            amount: 8.0_f32,
            category: ExtraHoursCategoryTO::Vacation,
            description: Arc::<str>::from("Jahresurlaub"),
            person_name: Arc::<str>::from("Max Mustermann"),
        }
    }

    /// D-07: HourlyMarkerRow rendert „stundenbasiert"-Badge + Stundenanzahl.
    #[test]
    fn hourly_marker_row_renders_badge() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                HourlyMarkerRow {
                    marker: sample_marker(),
                    is_hr: false,
                    on_edit: |_| {},
                    on_convert: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("stundenbasiert"),
            "HourlyMarkerRow muss 'stundenbasiert'-Badge rendern: {html}"
        );
        assert!(
            html.contains("8.00"),
            "HourlyMarkerRow muss die Stundenanzahl rendern: {html}"
        );
        assert!(
            html.contains("Std."),
            "HourlyMarkerRow muss den Stunden-Suffix rendern: {html}"
        );
    }

    /// D-08: „Stunden bearbeiten"-Button ist auch bei is_hr=false sichtbar.
    #[test]
    fn hourly_marker_row_renders_edit_action() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                HourlyMarkerRow {
                    marker: sample_marker(),
                    is_hr: false,
                    on_edit: |_| {},
                    on_convert: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Stunden bearbeiten"),
            "Stunden-bearbeiten-Button muss auch bei is_hr=false sichtbar sein: {html}"
        );
        // Convert-Button darf NICHT vorhanden sein (non-HR)
        assert!(
            !html.contains("In Zeitraum umwandeln"),
            "Convert-Button darf bei is_hr=false NICHT vorhanden sein: {html}"
        );
    }

    /// D-09: „In Zeitraum umwandeln"-Button nur bei is_hr=true.
    #[test]
    fn convert_action_visible_to_hr_only() {
        // HR: Button vorhanden.
        fn app_hr() -> Element {
            pin_de_locale();
            rsx! {
                HourlyMarkerRow {
                    marker: sample_marker(),
                    is_hr: true,
                    on_edit: |_| {},
                    on_convert: |_| {},
                }
            }
        }
        let html_hr = render(app_hr);
        assert!(
            html_hr.contains("In Zeitraum umwandeln"),
            "Convert-Button muss bei is_hr=true vorhanden sein: {html_hr}"
        );

        // Non-HR: Button nicht vorhanden.
        fn app_non_hr() -> Element {
            pin_de_locale();
            rsx! {
                HourlyMarkerRow {
                    marker: sample_marker(),
                    is_hr: false,
                    on_edit: |_| {},
                    on_convert: |_| {},
                }
            }
        }
        let html_non_hr = render(app_non_hr);
        assert!(
            !html_non_hr.contains("In Zeitraum umwandeln"),
            "Convert-Button darf bei is_hr=false NICHT vorhanden sein: {html_non_hr}"
        );
    }

    // ── marker_matches_filters — pure function tests (260612-nlv) ─────────

    fn test_marker(
        category: rest_types::ExtraHoursCategoryTO,
        sp: Uuid,
        when: time::Date,
    ) -> ExtraHoursMarker {
        ExtraHoursMarker {
            extra_hours_id: Uuid::nil(),
            sales_person_id: sp,
            when,
            amount: 8.0_f32,
            category,
            description: std::sync::Arc::<str>::from(""),
            person_name: std::sync::Arc::<str>::from(""),
        }
    }

    #[test]
    fn marker_category_match() {
        use rest_types::ExtraHoursCategoryTO;
        let today = date!(2026 - 06 - 12);
        let sp = Uuid::nil();
        let m = test_marker(ExtraHoursCategoryTO::Vacation, sp, today);
        assert!(
            marker_matches_filters(&m, Some(AbsenceCategory::Vacation), None, None, true, today),
            "Vacation marker must pass Vacation filter"
        );
    }

    #[test]
    fn marker_category_mismatch() {
        use rest_types::ExtraHoursCategoryTO;
        let today = date!(2026 - 06 - 12);
        let sp = Uuid::nil();
        let m = test_marker(ExtraHoursCategoryTO::SickLeave, sp, today);
        assert!(
            !marker_matches_filters(&m, Some(AbsenceCategory::Vacation), None, None, true, today),
            "SickLeave marker must NOT pass Vacation filter"
        );
    }

    #[test]
    fn marker_unmappable_category_with_active_filter() {
        use rest_types::ExtraHoursCategoryTO;
        let today = date!(2026 - 06 - 12);
        let sp = Uuid::nil();
        let m = test_marker(ExtraHoursCategoryTO::ExtraWork, sp, today);
        assert!(
            !marker_matches_filters(&m, Some(AbsenceCategory::Vacation), None, None, true, today),
            "ExtraWork (unmappable) must NOT pass Vacation filter"
        );
    }

    #[test]
    fn marker_no_category_filter_passes() {
        use rest_types::ExtraHoursCategoryTO;
        let today = date!(2026 - 06 - 12);
        let sp = Uuid::nil();
        let m = test_marker(ExtraHoursCategoryTO::ExtraWork, sp, today);
        assert!(
            marker_matches_filters(&m, None, None, None, true, today),
            "ExtraWork marker must pass when no category filter is active"
        );
    }

    #[test]
    fn marker_person_match() {
        use rest_types::ExtraHoursCategoryTO;
        let today = date!(2026 - 06 - 12);
        let sp = Uuid::from_u128(1);
        let m = test_marker(ExtraHoursCategoryTO::Vacation, sp, today);
        assert!(
            marker_matches_filters(&m, None, Some(sp), None, true, today),
            "Marker must pass person filter with matching sales_person_id"
        );
    }

    #[test]
    fn marker_person_mismatch() {
        use rest_types::ExtraHoursCategoryTO;
        let today = date!(2026 - 06 - 12);
        let sp = Uuid::from_u128(1);
        let other_sp = Uuid::from_u128(2);
        let m = test_marker(ExtraHoursCategoryTO::Vacation, sp, today);
        assert!(
            !marker_matches_filters(&m, None, Some(other_sp), None, true, today),
            "Marker must NOT pass person filter with different sales_person_id"
        );
    }

    #[test]
    fn marker_status_filter_single_day_active() {
        use rest_types::ExtraHoursCategoryTO;
        let today = date!(2026 - 06 - 12);
        let sp = Uuid::nil();
        // when == today → Active
        let m = test_marker(ExtraHoursCategoryTO::Vacation, sp, today);
        assert!(
            marker_matches_filters(&m, None, None, Some(AbsenceStatus::Active), true, today),
            "Marker on today must match Active status filter"
        );
        assert!(
            !marker_matches_filters(&m, None, None, Some(AbsenceStatus::Planned), true, today),
            "Marker on today must NOT match Planned status filter"
        );
    }

    #[test]
    fn marker_show_past_hides_finished() {
        use rest_types::ExtraHoursCategoryTO;
        let today = date!(2026 - 06 - 12);
        let past = date!(2026 - 06 - 01); // before today → Finished
        let sp = Uuid::nil();
        let m = test_marker(ExtraHoursCategoryTO::Vacation, sp, past);
        assert!(
            !marker_matches_filters(&m, None, None, None, false, today),
            "Finished marker must be hidden when show_past=false"
        );
        assert!(
            marker_matches_filters(&m, None, None, None, true, today),
            "Finished marker must be visible when show_past=true"
        );
    }

    // ── VacationEntitlementCard selected_person snapshot tests (260612-o7t Task 2) ──

    fn make_vacation_balance(sales_person_id: Uuid, remaining_days: f32) -> VacationBalance {
        VacationBalance {
            sales_person_id,
            year: 2026,
            entitled_days: 30.0,
            carryover_days: 0,
            used_days: 30.0 - remaining_days,
            planned_days: 0.0,
            remaining_days,
        }
    }

    #[test]
    fn vacation_card_hr_with_selected_person_renders_self_body() {
        let person_a = Uuid::from_u128(10);
        let person_b = Uuid::from_u128(20);
        let bal_a = make_vacation_balance(person_a, 17.0);
        let bal_b = make_vacation_balance(person_b, 3.0);
        let team: Rc<[VacationBalance]> = Rc::from(vec![bal_a, bal_b]);
        fn app() -> Element {
            let person_a = Uuid::from_u128(10);
            let person_b = Uuid::from_u128(20);
            let bal_a = make_vacation_balance(person_a, 17.0);
            let bal_b = make_vacation_balance(person_b, 3.0);
            let team: Rc<[VacationBalance]> = Rc::from(vec![bal_a, bal_b]);
            pin_de_locale();
            rsx! {
                VacationEntitlementCard {
                    is_hr: true,
                    year: 2026,
                    vacation_team: team,
                    sales_persons: Rc::<[SalesPerson]>::from([]),
                    selected_person: Some(person_a),
                }
            }
        }
        let html = render(app);
        // Self-view hero label (VacationEntitlementHero key): "Urlaubsanspruch 2026"
        assert!(
            html.contains("Urlaubsanspruch"),
            "self-hero label must appear when HR selects a person: {html}"
        );
        // Person A's remaining days
        assert!(
            html.contains("17"),
            "person A's remaining days (17) must appear in self-view: {html}"
        );
        // Per-person list header must NOT appear (team aggregate is gone)
        assert!(
            !html.contains("Pro Person"),
            "team aggregate header must NOT appear when a person is selected: {html}"
        );
        // Person B's distinct remaining days must NOT appear
        assert!(
            !html.contains(">3<"),
            "person B's remaining days must NOT appear when person A is selected: {html}"
        );
    }

    #[test]
    fn vacation_card_hr_without_selection_renders_team_aggregate() {
        fn app() -> Element {
            let person_a = Uuid::from_u128(10);
            let person_b = Uuid::from_u128(20);
            let bal_a = make_vacation_balance(person_a, 17.0);
            let bal_b = make_vacation_balance(person_b, 3.0);
            let team: Rc<[VacationBalance]> = Rc::from(vec![bal_a, bal_b]);
            // Provide selectable sales persons so selectable_balances keeps both
            // balances and the per-person list section renders.
            let sp_a = SalesPerson { id: person_a, is_paid: true, inactive: false, ..Default::default() };
            let sp_b = SalesPerson { id: person_b, is_paid: true, inactive: false, ..Default::default() };
            pin_de_locale();
            rsx! {
                VacationEntitlementCard {
                    is_hr: true,
                    year: 2026,
                    vacation_team: team,
                    sales_persons: Rc::from(vec![sp_a, sp_b]),
                    selected_person: None,
                }
            }
        }
        let html = render(app);
        // Per-person list header must appear (team aggregate view)
        assert!(
            html.contains("Pro Person"),
            "team aggregate header must appear when no person is selected: {html}"
        );
    }

    #[test]
    fn vacation_card_hr_selected_person_missing_from_team_falls_back() {
        let unknown = Uuid::from_u128(999);
        fn app() -> Element {
            let person_a = Uuid::from_u128(10);
            let unknown = Uuid::from_u128(999);
            let bal_a = make_vacation_balance(person_a, 17.0);
            let team: Rc<[VacationBalance]> = Rc::from(vec![bal_a]);
            pin_de_locale();
            rsx! {
                VacationEntitlementCard {
                    is_hr: true,
                    year: 2026,
                    vacation_team: team,
                    sales_persons: Rc::<[SalesPerson]>::from([]),
                    selected_person: Some(unknown),
                }
            }
        }
        // Should render without panic — self-body with zero/empty balance
        let html = render(app);
        // Hero label must still appear (self-view fallback)
        assert!(
            html.contains("Urlaubsanspruch"),
            "self-hero label must appear even when selected person is not in team: {html}"
        );
        let _ = unknown; // suppress unused variable warning
    }

    // ── stats_for_person — pure function tests (260612-o7t Task 1) ────────

    fn make_absence(
        sales_person_id: Uuid,
        category: AbsenceCategory,
        from_date: time::Date,
        to_date: time::Date,
    ) -> AbsencePeriod {
        AbsencePeriod {
            id: Uuid::nil(),
            sales_person_id,
            category,
            from_date,
            to_date,
            description: std::sync::Arc::<str>::from(""),
            version: Uuid::nil(),
            day_fraction: DayFraction::Full,
            derived_days: 0.0,
            person_name: std::sync::Arc::<str>::from(""),
            background_color: std::sync::Arc::<str>::from(""),
        }
    }

    #[test]
    fn stats_for_person_none_counts_all_persons() {
        let person_a = Uuid::from_u128(1);
        let person_b = Uuid::from_u128(2);
        let today = date!(2026 - 06 - 12);
        // A: 3-day SickLeave (active — today in range)
        let a_sick = make_absence(
            person_a,
            AbsenceCategory::SickLeave,
            date!(2026 - 06 - 10),
            date!(2026 - 06 - 12),
        );
        // B: 2-day UnpaidLeave (planned — future)
        let b_unpaid = make_absence(
            person_b,
            AbsenceCategory::UnpaidLeave,
            date!(2026 - 06 - 20),
            date!(2026 - 06 - 21),
        );
        let absences = vec![a_sick, b_unpaid];
        let (sick_days, unpaid_days, active_count) =
            stats_for_person(&absences, None, 2026, today);
        // Both persons counted: sick=3, unpaid=2, active=1 (only a_sick spans today)
        assert_eq!(sick_days, 3, "sick_days should span person A's range");
        assert_eq!(unpaid_days, 2, "unpaid_days should span person B's range");
        assert_eq!(active_count, 1, "active_count should be 1 (only a_sick is active)");
    }

    #[test]
    fn stats_for_person_some_scopes_to_that_person() {
        let person_a = Uuid::from_u128(1);
        let person_b = Uuid::from_u128(2);
        let today = date!(2026 - 06 - 12);
        // A: 3-day SickLeave (active)
        let a_sick = make_absence(
            person_a,
            AbsenceCategory::SickLeave,
            date!(2026 - 06 - 10),
            date!(2026 - 06 - 12),
        );
        // B: 5-day SickLeave (planned)
        let b_sick = make_absence(
            person_b,
            AbsenceCategory::SickLeave,
            date!(2026 - 06 - 20),
            date!(2026 - 06 - 24),
        );
        // B: 2-day UnpaidLeave (planned)
        let b_unpaid = make_absence(
            person_b,
            AbsenceCategory::UnpaidLeave,
            date!(2026 - 06 - 25),
            date!(2026 - 06 - 26),
        );
        let absences = vec![a_sick, b_sick, b_unpaid];
        let (sick_days, unpaid_days, active_count) =
            stats_for_person(&absences, Some(person_a), 2026, today);
        // Only A's records: sick=3, unpaid=0, active=1
        assert_eq!(sick_days, 3, "sick_days should only count person A");
        assert_eq!(unpaid_days, 0, "unpaid_days should be 0 — person A has none");
        assert_eq!(active_count, 1, "active_count should be 1 — person A's sick leave is active");
    }

    #[test]
    fn stats_for_person_sick_and_unpaid_inclusive_day_count() {
        let person_a = Uuid::from_u128(1);
        let today = date!(2026 - 01 - 20);
        // 5-day SickLeave (from 01-01 to 01-05 inclusive = 5 days)
        let sick = make_absence(
            person_a,
            AbsenceCategory::SickLeave,
            date!(2026 - 01 - 01),
            date!(2026 - 01 - 05),
        );
        // 3-day UnpaidLeave (from 01-10 to 01-12 inclusive = 3 days)
        let unpaid = make_absence(
            person_a,
            AbsenceCategory::UnpaidLeave,
            date!(2026 - 01 - 10),
            date!(2026 - 01 - 12),
        );
        let absences = vec![sick, unpaid];
        let (sick_days, unpaid_days, _active_count) =
            stats_for_person(&absences, None, 2026, today);
        assert_eq!(sick_days, 5, "5-day SickLeave must count as 5 inclusive days");
        assert_eq!(unpaid_days, 3, "3-day UnpaidLeave must count as 3 inclusive days");
    }

    #[test]
    fn stats_for_person_active_count_uses_compute_status() {
        let person_a = Uuid::from_u128(1);
        let today = date!(2026 - 06 - 12);
        // Active: today is inside the range
        let active_abs = make_absence(
            person_a,
            AbsenceCategory::Vacation,
            date!(2026 - 06 - 10),
            date!(2026 - 06 - 15),
        );
        // Finished: to_date before today
        let finished_abs = make_absence(
            person_a,
            AbsenceCategory::SickLeave,
            date!(2026 - 06 - 01),
            date!(2026 - 06 - 05),
        );
        let absences = vec![active_abs, finished_abs];
        let (_sick_days, _unpaid_days, active_count) =
            stats_for_person(&absences, None, 2026, today);
        assert_eq!(active_count, 1, "only the active range should be counted");
    }

    #[test]
    fn stats_for_person_excludes_out_of_year() {
        let person_a = Uuid::from_u128(1);
        let today = date!(2026 - 06 - 12);
        // Range entirely in a different year (2025)
        let past_year = make_absence(
            person_a,
            AbsenceCategory::SickLeave,
            date!(2025 - 03 - 01),
            date!(2025 - 03 - 10),
        );
        // Range in the target year 2026
        let this_year = make_absence(
            person_a,
            AbsenceCategory::SickLeave,
            date!(2026 - 02 - 01),
            date!(2026 - 02 - 03),
        );
        let absences = vec![past_year, this_year];
        let (sick_days, _unpaid_days, _active_count) =
            stats_for_person(&absences, None, 2026, today);
        // Only the 2026 range (3 days) should be counted
        assert_eq!(sick_days, 3, "out-of-year ranges must be excluded from sick_days count");
    }

    // ── selectable_balances — pure function tests (260613-jxe Task 1) ────────

    #[test]
    fn selectable_balances_keeps_paid_and_active_person() {
        let id = Uuid::from_u128(1);
        let sp = SalesPerson { id, is_paid: true, inactive: false, ..Default::default() };
        let balance = make_vacation_balance(id, 10.0);
        let result = selectable_balances(&[balance.clone()], &[sp]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].sales_person_id, id);
    }

    #[test]
    fn selectable_balances_drops_paid_but_inactive_person() {
        let id = Uuid::from_u128(2);
        let sp = SalesPerson { id, is_paid: true, inactive: true, ..Default::default() };
        let balance = make_vacation_balance(id, 5.0);
        let result = selectable_balances(&[balance], &[sp]);
        assert!(result.is_empty(), "paid but inactive person must be dropped");
    }

    #[test]
    fn selectable_balances_drops_unpaid_but_active_person() {
        let id = Uuid::from_u128(3);
        let sp = SalesPerson { id, is_paid: false, inactive: false, ..Default::default() };
        let balance = make_vacation_balance(id, 5.0);
        let result = selectable_balances(&[balance], &[sp]);
        assert!(result.is_empty(), "unpaid but active person must be dropped");
    }

    #[test]
    fn selectable_balances_drops_balance_with_unknown_id() {
        let known_id = Uuid::from_u128(4);
        let unknown_id = Uuid::from_u128(99);
        let sp = SalesPerson { id: known_id, is_paid: true, inactive: false, ..Default::default() };
        let balance = make_vacation_balance(unknown_id, 7.0);
        let result = selectable_balances(&[balance], &[sp]);
        assert!(result.is_empty(), "balance for unknown id must be dropped");
    }

    #[test]
    fn selectable_balances_preserves_order_of_kept_balances() {
        let id_a = Uuid::from_u128(10);
        let id_b = Uuid::from_u128(20);
        let sp_a = SalesPerson { id: id_a, is_paid: true, inactive: false, ..Default::default() };
        let sp_b = SalesPerson { id: id_b, is_paid: true, inactive: false, ..Default::default() };
        let bal_a = make_vacation_balance(id_a, 15.0);
        let bal_b = make_vacation_balance(id_b, 5.0);
        // b comes first in the input slice
        let result = selectable_balances(&[bal_b.clone(), bal_a.clone()], &[sp_a, sp_b]);
        assert_eq!(result.len(), 2);
        // Order must be preserved: b (15 remaining after 30-5=25 used, remaining=5) first
        assert_eq!(result[0].sales_person_id, id_b);
        assert_eq!(result[1].sales_person_id, id_a);
    }

    // ── Regression: refetch-effect must subscribe to ABSENCE_REFRESH ──────
    //
    // Debug session `convert-absence-name-and-refresh` (Bug 2 + Bug 3): the
    // AbsencesPage refetch `use_effect` previously read `ABSENCE_REFRESH`
    // OUTSIDE its closure and captured a snapshot, so it never re-fired when a
    // mutation (e.g. convert-to-absence) bumped the token — the view stayed
    // stale (Bug 3) and stale-bootstrap snapshots produced empty names (Bug 2).
    //
    // This test locks the Dioxus-0.6 reactivity contract the fix relies on: an
    // effect that reads `ABSENCE_REFRESH` *inside* its closure re-runs when
    // `bump_absence_refresh()` is called. If someone moves the read back
    // outside the closure, the effect will stop re-firing and this test fails.

    static EFFECT_FIRE_COUNT: GlobalSignal<u32> = Signal::global(|| 0);

    #[test]
    fn refetch_effect_reruns_when_absence_refresh_bumps() {
        use crate::service::absence::{bump_absence_refresh, ABSENCE_REFRESH};

        fn subscriber_app() -> Element {
            // Reset the shared counter on first mount so the run is
            // deterministic regardless of other tests sharing the process.
            use_hook(|| {
                *EFFECT_FIRE_COUNT.write() = 0;
            });
            // Mirror the page's pattern: read ABSENCE_REFRESH INSIDE the effect
            // closure so Dioxus subscribes the effect to it.
            use_effect(|| {
                let _ = *ABSENCE_REFRESH.read();
                *EFFECT_FIRE_COUNT.write() += 1;
            });
            rsx! {}
        }

        let mut vdom = VirtualDom::new(subscriber_app);
        vdom.rebuild_in_place();
        // Effects run when the vdom processes its work queue.
        vdom.render_immediate(&mut dioxus_core::NoOpMutations);
        let after_initial = vdom.in_runtime(|| *EFFECT_FIRE_COUNT.read());
        assert!(
            after_initial >= 1,
            "effect must fire at least once on initial build (got {after_initial})"
        );

        // Bump the refresh token from within the runtime, then let the vdom
        // process the resulting effect re-run.
        vdom.in_runtime(|| {
            bump_absence_refresh();
        });
        vdom.render_immediate(&mut dioxus_core::NoOpMutations);

        let after_bump = vdom.in_runtime(|| *EFFECT_FIRE_COUNT.read());
        assert!(
            after_bump > after_initial,
            "effect must re-fire after ABSENCE_REFRESH bump \
             (subscribed in-closure); before={after_initial} after={after_bump}. \
             If this fails, the refetch effect likely reads ABSENCE_REFRESH \
             OUTSIDE its closure again (regression of \
             convert-absence-name-and-refresh)."
        );
    }
}
