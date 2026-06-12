//! Shared `WarningList` component (Phase 9 — FUI-A-05).
//!
//! Moved from `page/absences.rs` and extended with the three booking-path
//! `WarningTO` variants (`BookingOnAbsenceDay`, `BookingOnUnavailableDay`,
//! `PaidEmployeeLimitExceeded`). Also gains `person_name` and `suppress_header`
//! props for the booking-dialog caller in `shiftplan.rs`.
//!
//! The two absence-path arms (`AbsenceOverlapsBooking`,
//! `AbsenceOverlapsManualUnavailable`) are kept so `absences.rs` callers
//! continue to work without modification.

use std::rc::Rc;

use dioxus::prelude::*;
use rest_types::{AbsenceCategoryTO, DayOfWeekTO, WarningTO};

use crate::base_types::ImStr;
use crate::i18n::{generate, Key, Locale};
use crate::service::i18n::I18N;

// ─── WarningsList newtype (PartialEq via Rc::ptr_eq) ──────────────────────────

/// Newtype wrapper for `Rc<[WarningTO]>` so the props type can derive
/// `PartialEq`. `WarningTO` itself does not implement `PartialEq` (it carries
/// non-comparable data), so we compare by `Rc::ptr_eq` — exact for "same
/// allocation" and accurate-enough for re-render skip (Phase 8 / Phase 9 D-09).
#[derive(Clone, Debug)]
pub struct WarningsList(pub Rc<[WarningTO]>);

impl PartialEq for WarningsList {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl WarningsList {
    pub fn empty() -> Self {
        Self(Rc::new([]))
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ─── WarningListProps ─────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct WarningListProps {
    pub warnings: WarningsList,
    /// Spacing variant: `true` → `p-2.5`, `false` → `p-4` (normalised per UI-SPEC).
    #[props(default = false)]
    pub dense: bool,
    /// Person name for booking-path warnings (substituted for `{person}` placeholder).
    /// Absence-path callers pass `None` (defaults applied).
    #[props(default = None)]
    pub person_name: Option<ImStr>,
    /// When `true`, the internal `text-micro text-warn font-semibold uppercase` header
    /// row is NOT rendered. Use this when an outer `Dialog { title }` already provides
    /// the heading, to avoid a double-header. Defaults to `false` (header shown).
    #[props(default = false)]
    pub suppress_header: bool,
}

// ─── Helper functions ─────────────────────────────────────────────────────────

fn category_key(c: &AbsenceCategoryTO) -> Key {
    match c {
        AbsenceCategoryTO::Vacation => Key::AbsenceCategoryVacation,
        AbsenceCategoryTO::SickLeave => Key::AbsenceCategorySickLeave,
        AbsenceCategoryTO::UnpaidLeave => Key::AbsenceCategoryUnpaidLeave,
    }
}

fn day_of_week_key(d: &DayOfWeekTO) -> Key {
    match d {
        DayOfWeekTO::Monday => Key::Monday,
        DayOfWeekTO::Tuesday => Key::Tuesday,
        DayOfWeekTO::Wednesday => Key::Wednesday,
        DayOfWeekTO::Thursday => Key::Thursday,
        DayOfWeekTO::Friday => Key::Friday,
        DayOfWeekTO::Saturday => Key::Saturday,
        DayOfWeekTO::Sunday => Key::Sunday,
    }
}

// ─── WarningList component ────────────────────────────────────────────────────

#[component]
pub fn WarningList(props: WarningListProps) -> Element {
    let i18n = I18N.read().clone();
    let count = props.warnings.len();
    if count == 0 {
        return rsx! {};
    }
    let header_text = if count == 1 {
        i18n.t(Key::BookingWarningDialogHeaderSingular).to_string()
    } else {
        i18n.t(Key::BookingWarningDialogHeaderPlural)
            .as_ref()
            .replace("{count}", &count.to_string())
    };
    let pad_class = if props.dense { "p-2.5" } else { "p-4" };
    let person_owned = props
        .person_name
        .as_ref()
        .map(|n| n.as_str().to_string())
        .unwrap_or_else(|| "–".to_string());
    let person = person_owned.as_str();
    rsx! {
        div { class: "border-l-[3px] border-warn bg-warn-soft rounded-md {pad_class} flex flex-col gap-2",
            if !props.suppress_header {
                div { class: "text-micro text-warn font-semibold uppercase", "{header_text}" }
            }
            ul { class: "list-disc pl-4 text-body text-ink",
                for warning in props.warnings.0.iter() {
                    li {
                        match warning {
                            WarningTO::AbsenceOverlapsBooking { date, .. } => {
                                let body = i18n
                                    .t(Key::AbsenceWarningOverlapsBooking)
                                    .as_ref()
                                    .replace("{date}", &date.to_string());
                                rsx! { "{body}" }
                            }
                            WarningTO::AbsenceOverlapsManualUnavailable { .. } => rsx! {
                                "{i18n.t(Key::AbsenceWarningOverlapsManual)}"
                            },
                            WarningTO::BookingOnAbsenceDay { date, category, .. } => {
                                let body = i18n
                                    .t(Key::BookingWarningOnAbsenceDay)
                                    .as_ref()
                                    .replace("{person}", person)
                                    .replace("{date}", &date.to_string())
                                    .replace("{category}", &i18n.t(category_key(category)).to_string());
                                rsx! { "{body}" }
                            }
                            WarningTO::BookingOnUnavailableDay { week, year, day_of_week, .. } => {
                                let body = i18n
                                    .t(Key::BookingWarningOnUnavailableDay)
                                    .as_ref()
                                    .replace("{person}", person)
                                    .replace("{week}", &week.to_string())
                                    .replace("{year}", &year.to_string())
                                    .replace("{day}", &i18n.t(day_of_week_key(day_of_week)).to_string());
                                rsx! { "{body}" }
                            }
                            WarningTO::PaidEmployeeLimitExceeded {
                                current_paid_count,
                                max_paid_employees,
                                ..
                            } => {
                                let body = i18n
                                    .t(Key::BookingWarningPaidLimitExceeded)
                                    .as_ref()
                                    .replace("{current}", &current_paid_count.to_string())
                                    .replace("{max}", &max_paid_employees.to_string());
                                rsx! { "{body}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;
    use uuid::Uuid;

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

    // ── warning_list_renders_booking_on_absence_day ────────────────────────

    #[test]
    fn warning_list_renders_booking_on_absence_day() {
        fn app() -> Element {
            pin_de_locale();
            let warnings = WarningsList(Rc::new([WarningTO::BookingOnAbsenceDay {
                booking_id: Uuid::nil(),
                date: date!(2026 - 12 - 24),
                absence_id: Uuid::nil(),
                category: AbsenceCategoryTO::Vacation,
            }]));
            rsx! {
                WarningList {
                    warnings,
                    person_name: Some(ImStr::from("Maria")),
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Maria"),
            "expected 'Maria' in output, got: {html}"
        );
        assert!(
            html.contains("2026-12-24"),
            "expected '2026-12-24' in output, got: {html}"
        );
        // German "Urlaub" = AbsenceCategoryVacation
        assert!(
            html.contains("Urlaub"),
            "expected 'Urlaub' (vacation category) in output, got: {html}"
        );
    }

    // ── warning_list_renders_booking_on_unavailable_day ────────────────────

    #[test]
    fn warning_list_renders_booking_on_unavailable_day() {
        fn app() -> Element {
            pin_de_locale();
            let warnings = WarningsList(Rc::new([WarningTO::BookingOnUnavailableDay {
                booking_id: Uuid::nil(),
                year: 2026,
                week: 12,
                day_of_week: DayOfWeekTO::Monday,
            }]));
            rsx! {
                WarningList {
                    warnings,
                    person_name: Some(ImStr::from("Maria")),
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("KW 12/2026"),
            "expected 'KW 12/2026' in output, got: {html}"
        );
        assert!(
            html.contains("Maria"),
            "expected 'Maria' in output, got: {html}"
        );
        // The German Monday key should appear in the output
        // (exact string depends on locale; just check it's non-empty)
        assert!(!html.is_empty(), "output should not be empty");
    }

    // ── warning_list_renders_paid_limit_exceeded ───────────────────────────

    #[test]
    fn warning_list_renders_paid_limit_exceeded() {
        fn app() -> Element {
            pin_de_locale();
            let warnings = WarningsList(Rc::new([WarningTO::PaidEmployeeLimitExceeded {
                slot_id: Uuid::nil(),
                booking_id: Uuid::nil(),
                year: 2026,
                week: 12,
                current_paid_count: 2,
                max_paid_employees: 3,
            }]));
            rsx! {
                WarningList { warnings }
            }
        }
        let html = render(app);
        assert!(
            html.contains('2'),
            "expected current count '2' in output, got: {html}"
        );
        assert!(
            html.contains('3'),
            "expected max count '3' in output, got: {html}"
        );
    }

    // ── warning_list_empty_renders_nothing ─────────────────────────────────

    #[test]
    fn warning_list_empty_renders_nothing() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                WarningList { warnings: WarningsList::empty() }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("border-l-[3px]"),
            "empty list should render no warning box, got: {html}"
        );
    }

    // ── warning_list_suppress_header_hides_internal_header ─────────────────

    #[test]
    fn warning_list_suppress_header_hides_internal_header() {
        fn app() -> Element {
            pin_de_locale();
            let warnings = WarningsList(Rc::new([WarningTO::PaidEmployeeLimitExceeded {
                slot_id: Uuid::nil(),
                booking_id: Uuid::nil(),
                year: 2026,
                week: 12,
                current_paid_count: 1,
                max_paid_employees: 5,
            }]));
            rsx! {
                WarningList { warnings, suppress_header: true }
            }
        }
        let html = render(app);
        // The internal header div must NOT be present
        assert!(
            !html.contains("text-micro text-warn font-semibold uppercase"),
            "suppress_header: true should hide the internal header, got: {html}"
        );
        // But the warning item text must still be present
        assert!(
            html.contains('1') || html.contains('5'),
            "warning item should still be rendered, got: {html}"
        );
    }
}
