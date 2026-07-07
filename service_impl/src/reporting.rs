use crate::gen_service_impl;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    absence::{AbsenceCategory, AbsencePeriod, AbsenceService, ResolvedAbsence},
    carryover::CarryoverService,
    clock::ClockService,
    employee_work_details::{EmployeeWorkDetails, EmployeeWorkDetailsService},
    extra_hours::{
        Availability, ExtraHours, ExtraHoursCategory, ExtraHoursService, ExtraHoursSource,
        ReportType,
    },
    permission::{Authentication, HR_PRIVILEGE},
    reporting::{
        CustomExtraHours, EmployeeReport, ExtraHoursReportCategory, GroupedReportHours,
        ShortEmployeeReport, WorkingHoursDay,
    },
    sales_person::{SalesPerson, SalesPersonService},
    shiftplan_report::{ShiftplanReportDay, ShiftplanReportService},
    special_days::{SpecialDay, SpecialDayService, SpecialDayType},
    toggle::ToggleService,
    uuid_service::UuidService,
    PermissionService, ServiceError,
};
use shifty_utils::{DayOfWeek, ShiftyDate, ShiftyWeek};
use tokio::join;
use tracing::info;
use uuid::Uuid;

pub trait IteratorExt {
    fn collect_to_hash_map_by<K, F>(self, f: F) -> HashMap<K, Arc<[Self::Item]>>
    where
        Self: Iterator + Sized,
        K: Clone + Eq + std::hash::Hash,
        F: Fn(&Self::Item) -> K,
    {
        let vec_map = self.fold(HashMap::new(), |mut map, item| {
            let key = f(&item);
            map.entry(key.clone()).or_insert_with(Vec::new).push(item);
            map
        });
        let vec_map: HashMap<K, Arc<[Self::Item]>> = vec_map
            .into_iter()
            .map(|(key, vec)| (key, vec.into()))
            .collect();
        vec_map
    }
}
impl<T> IteratorExt for T where T: Iterator {}

#[test]
pub fn iterator_test() {
    let vec = [(1, 1), (2, 5), (1, 6)];
    let map = vec.iter().collect_to_hash_map_by(|e| e.0);
    assert_eq!(map.len(), 2);
    let first_sum = map.get(&1).unwrap().iter().map(|e| e.1).sum::<i32>();
    let second_sum = map.get(&2).unwrap().iter().map(|e| e.1).sum::<i32>();
    assert_eq!(first_sum, 7);
    assert_eq!(second_sum, 5);
}

gen_service_impl! {
    struct ReportingServiceImpl: ReportingService = ReportingServiceDeps {
        ExtraHoursService: ExtraHoursService<Transaction = Self::Transaction> = extra_hours_service,
        ShiftplanReportService: ShiftplanReportService<Transaction = Self::Transaction> = shiftplan_report_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Transaction = Self::Transaction, Context = Self::Context> = employee_work_details_service,
        SalesPersonService: SalesPersonService<Transaction = Self::Transaction, Context = Self::Context> = sales_person_service,
        CarryoverService: CarryoverService<Transaction = Self::Transaction> = carryover_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        // Phase 8.4: Additiver Merge — AbsenceService-derived hours werden
        // unbedingt mit den lebenden extra_hours summiert (kein Feature-Flag-
        // Switch mehr; FeatureFlagService-Dep wurde hier entfernt — M-03).
        AbsenceService: AbsenceService<Context = Self::Context, Transaction = Self::Transaction> = absence_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
        // Phase 25: Holiday derive-on-read — SpecialDayService is Basic-tier
        // (no Transaction type), ToggleService is Basic-tier. ReportingService
        // is Business-Logic tier and may consume both — no cycle.
        SpecialDayService: SpecialDayService<Context = Self::Context> = special_day_service,
        ToggleService: ToggleService<Context = Self::Context, Transaction = Self::Transaction> = toggle_service,
    }
}

pub fn find_working_hours_for_calendar_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> impl Iterator<Item = &EmployeeWorkDetails> {
    working_hours.iter().filter(move |wh| {
        (year, week) >= (wh.from_year, wh.from_calendar_week)
            && (year, week) <= (wh.to_year, wh.to_calendar_week)
    })
}

/// CVC-03 / D-OVERLAP-AGG = SUM: Aggregiert `committed_voluntary` über alle
/// in der ISO-Woche aktiven `EmployeeWorkDetails`-Rows per **SUM**.
///
/// Liegen zwei überlappende Rows in derselben Woche (Daten-Anomalie — Versionen
/// sind normalerweise sequenziell, aber `find_working_hours_for_calendar_week`
/// kann mehrere Rows liefern), wird deren `committed_voluntary` summiert —
/// konsistent mit dem `expected_hours`-Präzedenzfall in `reporting.rs` (`.fold`-
/// Pfad, gleiche Selektion). Das Boolean-`.any()`-Pattern des Cap-Flags
/// generalisiert nicht auf einen numerischen Wert und wird hier nicht kopiert.
///
/// **In Phase 14 existiert kein Produktions-Read-Site für diesen Helper** —
/// das Feld ist inert. Phase 15 (Reporting-Integration) konsumiert diesen
/// Helper direkt.
pub fn committed_voluntary_for_calendar_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> f32 {
    find_working_hours_for_calendar_week(working_hours, year, week)
        .map(|wh| wh.committed_voluntary)
        .sum()
}

/// Phase 54 (VOL-STAT-01 / VOL-ACCT-01-Ist, D-54-DM-02):
/// Summe der `VolunteerWork`-`ExtraHours` im ISO-Jahr `year`, gefiltert auf
/// `source == Manual`. Rebooking-Marker-Rows (Pitfall 1: Doppel-Zaehlung
/// bei +N/-N-Paaren) sind ausgeschlossen (VOL-ACCT-03 Property-Test).
///
/// Die ISO-Jahres-Zuordnung erfolgt via `ShiftyDate::from(...).as_shifty_week().year`
/// — dieselbe Semantik wie im uebrigen `reporting.rs`.
pub fn voluntary_ist_total_for_year(extra_hours: &[ExtraHours], year: u32) -> f32 {
    extra_hours
        .iter()
        .filter(|eh| eh.deleted.is_none())
        .filter(|eh| matches!(eh.category, ExtraHoursCategory::VolunteerWork))
        .filter(|eh| eh.source == ExtraHoursSource::Manual)
        .filter(|eh| ShiftyDate::from(eh.date_time).as_shifty_week().year == year)
        .map(|eh| eh.amount)
        .sum()
}

/// Phase 54 (VOL-STAT-01-Nenner / VOL-ACCT-01-Ist_per_week Nenner, D-F1-01):
/// Anzahl ISO-Wochen im Jahr, in denen mindestens eine aktive
/// `EmployeeWorkDetails`-Row den Kalender-Wochen-Slot abdeckt. Eine Row mit
/// `expected_hours == 0` zaehlt MIT (D-F1-01, siehe Test
/// `contract_weeks_zero_expected_counts_d_f1_01`).
///
/// Iteriert `1..=weeks_in_year(year)` und pruept jede Woche via
/// `find_working_hours_for_calendar_week`.
pub fn contract_weeks_count(working_hours: &[EmployeeWorkDetails], year: u32) -> u32 {
    let total = time::util::weeks_in_year(year as i32);
    (1..=total)
        .filter(|&w| find_working_hours_for_calendar_week(working_hours, year, w).next().is_some())
        .count() as u32
}

/// Phase 54 (VOL-ACCT-01-Soll, D-F2-01):
/// Pro-rata-Verteilung von `committed_voluntary` fuer eine ISO-Woche.
/// Iteriert Mo..So, waehlt fuer jeden Tag die aktive `EmployeeWorkDetails`
/// via `find_working_hours_for_calendar_week` (fuer die ISO-Woche des Tages),
/// summiert (`committed_voluntary / 7.0`) je Tag.
///
/// Bei Mid-Week-Vertragswechsel (z.B. Mittwoch=>Donnerstag) ergibt das
/// 3/7*alt + 4/7*neu (siehe Test `f2_soll_prorata_midweek_change_d_f2_01`).
///
/// Guard: Existiert die ISO-Woche fuer das Jahr nicht (z.B. 53 in einem
/// 52-Wochen-Jahr), gibt die Funktion 0.0 zurueck.
pub fn committed_voluntary_prorata_for_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> f32 {
    if week == 0 || week > time::util::weeks_in_year(year as i32) {
        return 0.0;
    }
    let Ok(monday) = time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday)
    else {
        return 0.0;
    };
    (0i64..7)
        .filter_map(|offset| {
            let day = monday + time::Duration::days(offset);
            // Aktiver Vertrag am Tag: matche EmployeeWorkDetails, deren
            // from_date..=to_date den Tag umschliesst (Mid-Week-Wechsel wird
            // so korrekt tagesgenau abgebildet).
            working_hours
                .iter()
                .filter(|wh| wh.deleted.is_none())
                .find(|wh| {
                    let from = match wh.from_date() {
                        Ok(d) => d.to_date(),
                        Err(_) => return false,
                    };
                    let to = match wh.to_date() {
                        Ok(d) => d.to_date(),
                        Err(_) => return false,
                    };
                    from <= day && day <= to
                })
                .map(|wh| wh.committed_voluntary / 7.0)
        })
        .sum()
}

/// Phase 54 (VOL-ACCT-01-Soll, D-F2-01):
/// Summe der pro-rata-Wochen ueber das ISO-Jahr. Fuer ein 53-Wochen-Jahr
/// werden 53 Wochen aufsummiert, fuer ein 52-Wochen-Jahr entsprechend 52
/// (siehe Test `f2_soll_iso_week_53_year_boundary_d_f2_01`).
pub fn committed_voluntary_target_for_year(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
) -> f32 {
    let total = time::util::weeks_in_year(year as i32);
    (1..=total)
        .map(|w| committed_voluntary_prorata_for_week(working_hours, year, w))
        .sum()
}

/// Caps shiftplan hours at expected hours when at least one of the active
/// `EmployeeWorkDetails` records for the week sets `cap_planned_hours_to_expected`.
/// Returns `(capped_shiftplan_hours, auto_volunteer_hours)`. When the cap is
/// inactive or would not bind, the shiftplan hours pass through unchanged and
/// `auto_volunteer_hours` is `0.0`. ExtraHours records are never affected by
/// this function.
pub fn apply_weekly_cap(
    cap_active: bool,
    shiftplan_hours: f32,
    expected_hours_for_week: f32,
) -> (f32, f32) {
    if cap_active && shiftplan_hours > expected_hours_for_week {
        (
            expected_hours_for_week,
            shiftplan_hours - expected_hours_for_week,
        )
    } else {
        (shiftplan_hours, 0.0)
    }
}

/// Phase 52 Follow-Up #2 (WOP-04): Priority function for the cross-category
/// absence resolver. Mirrors `service_impl::absence::absence_category_priority`
/// (D-Phase2-03, BUrlG §9). Kept crate-private so the assemble_weeks pure
/// helper reproduces the exact tie-breaker semantics of
/// `AbsenceService::derive_hours_for_range`.
fn absence_category_priority(category: &AbsenceCategory) -> u8 {
    match category {
        AbsenceCategory::SickLeave => 3,
        AbsenceCategory::Vacation => 2,
        AbsenceCategory::UnpaidLeave => 1,
    }
}

/// Phase 52 Follow-Up #2 (WOP-04): Pure, in-memory replacement for
/// `AbsenceService::derive_hours_for_range` restricted to a single ISO week.
///
/// **Byte-identical to** `derive_hours_for_range(monday, sunday, sales_person_id, ...)`
/// for the week `(year, week)` given the same inputs — same day iteration,
/// same active-contract selection, same dominant-category resolver, same
/// per-week workdays cap with fractional-day support.
///
/// Inputs (all pre-loaded once at the top of `assemble_weeks`, then reused):
/// - `absences_for_person`: slice of `AbsencePeriod` filtered to this person
///   (already `deleted IS NULL` from the DAO). Order must be `by from_date`
///   to match the DAO's `find_by_sales_person` ordering (irrelevant for
///   correctness because we use `max_by_key`, but preserved for parity).
/// - `contracts_for_person`: slice of `EmployeeWorkDetails` for this person.
/// - `holidays_this_week`: `BTreeSet<Date>` of holidays overlapping this week
///   (`SpecialDayType::Holiday`, `deleted IS NULL`). Used to skip holiday
///   days from absence computation.
fn derive_hours_for_week_pure(
    year: u32,
    week: u8,
    absences_for_person: &[&AbsencePeriod],
    contracts_for_person: &[EmployeeWorkDetails],
    holidays_this_week: &BTreeSet<time::Date>,
) -> BTreeMap<time::Date, ResolvedAbsence> {
    let mut result: BTreeMap<time::Date, ResolvedAbsence> = BTreeMap::new();
    // Build the seven days Monday..Sunday of the target ISO week.
    let Ok(monday) = time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday) else {
        return result;
    };
    let Ok(sunday) = time::Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday) else {
        return result;
    };

    // Per-week workdays counter — starts at 0, capped at `workdays_per_week`.
    // Single-week scope, so we don't need a map keyed by Monday.
    let mut week_counted: f32 = 0.0;

    // Iterate Mon..=Sun via day-offset (0..7). Safe across month/year boundaries
    // because `time::Date + time::Duration::days(i64)` is total for values in
    // year range.
    for day_offset in 0i64..7 {
        let day = monday + time::Duration::days(day_offset);
        if day > sunday {
            break;
        }
        // Active contract for this day.
        let active_contract = contracts_for_person.iter().find(|wh| {
            if wh.deleted.is_some() {
                return false;
            }
            let from_date = match wh.from_date() {
                Ok(d) => d.to_date(),
                Err(_) => return false,
            };
            let to_date = match wh.to_date() {
                Ok(d) => d.to_date(),
                Err(_) => return false,
            };
            from_date <= day && day <= to_date
        });
        let Some(contract) = active_contract else {
            continue;
        };
        // Availability: only checked weekdays may bear absence.
        if !contract.has_day_of_week(day.weekday()) {
            continue;
        }
        if holidays_this_week.contains(&day) {
            continue;
        }

        // Dominant active absence for this day (priority ordering,
        // ties broken by `max_by_key` = last-encountered wins — preserved).
        let dominant = absences_for_person
            .iter()
            .filter(|ap| ap.deleted.is_none() && ap.from_date <= day && day <= ap.to_date)
            .max_by_key(|ap| absence_category_priority(&ap.category));
        let Some(dominant) = dominant else {
            continue;
        };

        let workdays = contract.workdays_per_week as f32;
        let hours_per_day = contract.hours_per_day();
        if workdays <= 0.0 || hours_per_day <= 0.0 {
            continue;
        }

        let remaining = workdays - week_counted;
        if remaining <= 0.0 {
            continue;
        }

        let day_fraction_factor: f32 = match dominant.day_fraction {
            service::absence::DayFraction::Half => 0.5,
            service::absence::DayFraction::Full => 1.0,
        };
        let counted = day_fraction_factor.min(remaining);
        week_counted += counted;
        result.insert(
            day,
            ResolvedAbsence {
                category: dominant.category,
                hours: counted * hours_per_day,
                days: counted,
            },
        );
    }

    result
}

/// Phase 52 Follow-Up #2 (WOP-04): Pure, in-memory replacement for
/// `build_derived_holiday_map` restricted to a single ISO week.
///
/// **Byte-identical to** the async `build_derived_holiday_map(monday, sunday, ...)`
/// call for the week `(year, week)` given the same inputs.
///
/// Inputs (pre-loaded once at the top of `assemble_weeks`):
/// - `cutoff`: pre-parsed `holiday_auto_credit` toggle value; `None` disables
///   the auto-credit (D-25-05).
/// - `special_days_this_week`: slice of `SpecialDay` rows overlapping this
///   week (filtered upstream).
/// - `working_hours`: contracts for this person.
/// - `extra_hours`: per-week extra hours (manual-wins gate).
fn build_derived_holiday_map_for_week_pure(
    year: u32,
    week: u8,
    cutoff: Option<time::Date>,
    special_days_this_week: &[SpecialDay],
    working_hours: &[EmployeeWorkDetails],
    extra_hours_for_person: &[ExtraHours],
) -> HashMap<time::Date, f32> {
    let mut result: HashMap<time::Date, f32> = HashMap::new();
    let Some(cutoff) = cutoff else {
        return result;
    };
    for sd in special_days_this_week.iter() {
        if sd.day_type != SpecialDayType::Holiday {
            continue;
        }
        let holiday_date = match time::Date::from_iso_week_date(
            sd.year as i32,
            sd.calendar_week,
            time::Weekday::from(sd.day_of_week),
        ) {
            Ok(d) => d,
            Err(_) => continue,
        };
        if holiday_date < cutoff {
            continue;
        }
        let has_manual = extra_hours_for_person.iter().any(|eh| {
            eh.category == ExtraHoursCategory::Holiday && eh.date_time.date() == holiday_date
        });
        if has_manual {
            continue;
        }
        if let Some(wh) = find_working_hours_for_calendar_week(working_hours, year, week).next() {
            if wh.has_day_of_week(time::Weekday::from(sd.day_of_week)) {
                let hours = wh.holiday_hours();
                if hours > 0.0 {
                    result.insert(holiday_date, hours);
                }
            }
        }
    }
    result
}

impl<Deps: ReportingServiceDeps> ReportingServiceImpl<Deps> {
    /// Phase 25 (HOL-01/02, HCFG-01/03): Build a per-employee derived-holiday map
    /// for a date range. The map is keyed by the concrete holiday date and
    /// contains the credited hours (= `EmployeeWorkDetails::holiday_hours()`).
    ///
    /// Returns an empty map when:
    /// - The `holiday_auto_credit` toggle has no `value` set (automation off, D-25-05).
    /// - A manual `ExtraHours(Holiday)` already covers the same employee+day
    ///   (manual wins, D-25-03 / HCFG-03).
    ///
    /// Respects year-boundary-safe ISO-week date arithmetic (uses
    /// `time::Date::from_iso_week_date`, never manual week math — Pitfall 1).
    async fn build_derived_holiday_map(
        &self,
        from_date: ShiftyDate,
        to_date: ShiftyDate,
        working_hours: &[EmployeeWorkDetails],
        extra_hours: &[ExtraHours],
        context: Authentication<Deps::Context>,
    ) -> Result<std::collections::HashMap<time::Date, f32>, ServiceError> {
        // Step 1: Read cutoff from toggle service (D-25-05).
        // Treat Unauthorized as "no cutoff configured" (automation off) rather than
        // propagating the error — the reporting service is called with various
        // authentication contexts (e.g. mock-auth tests) where the toggle service
        // requires a real user ID but the caller has none.
        let toggle_value = match self
            .toggle_service
            .get_toggle_value("holiday_auto_credit", context.clone(), None)
            .await
        {
            Ok(v) => v,
            Err(ServiceError::Unauthorized) => return Ok(std::collections::HashMap::new()),
            Err(e) => return Err(e),
        };
        let cutoff: time::Date = match toggle_value
            .as_deref()
            .and_then(|s| {
                time::Date::parse(s, &time::format_description::well_known::Iso8601::DEFAULT).ok()
            }) {
            Some(d) => d,
            None => return Ok(std::collections::HashMap::new()), // automation off
        };

        let mut result: std::collections::HashMap<time::Date, f32> =
            std::collections::HashMap::new();

        // Step 2: Iterate over every ISO week in the range, fetch special days.
        let from_week = from_date.as_shifty_week();
        let to_week = to_date.as_shifty_week();
        for week in from_week.iter_until(&to_week) {
            let special_days = self
                .special_day_service
                .get_by_week(week.year, week.week, context.clone())
                .await?;

            for sd in special_days.iter() {
                // Only process Holiday entries (not ShortDay etc.).
                if sd.day_type != service::special_days::SpecialDayType::Holiday {
                    continue;
                }

                // Step 3: Convert (year, calendar_week, day_of_week) → concrete date.
                // Use time::Date::from_iso_week_date to handle year-boundary correctly.
                let holiday_date = match time::Date::from_iso_week_date(
                    sd.year as i32,
                    sd.calendar_week,
                    time::Weekday::from(sd.day_of_week),
                ) {
                    Ok(d) => d,
                    Err(_) => continue, // invalid date — defensive skip
                };

                // Step 4: Cutoff gate (HCFG-01, D-25-05).
                if holiday_date < cutoff {
                    continue;
                }

                // Step 5: Conflict check — manual ExtraHours(Holiday) for same day
                // takes priority (D-25-03 / HCFG-03).
                let has_manual = extra_hours.iter().any(|eh| {
                    eh.category == ExtraHoursCategory::Holiday
                        && eh.date_time.date() == holiday_date
                });
                if has_manual {
                    continue; // manual wins — skip auto-credit for this day
                }

                // Step 6: Find contract valid this week. Credit only when the
                // employee's contract covers this day-of-week (D-25-02).
                if let Some(wh) =
                    find_working_hours_for_calendar_week(working_hours, week.year, week.week).next()
                {
                    if wh.has_day_of_week(time::Weekday::from(sd.day_of_week)) {
                        let hours = wh.holiday_hours();
                        if hours > 0.0 {
                            result.insert(holiday_date, hours);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Phase 52 (WOP-02): Per-week aggregation helper — single source of truth
    /// for the `get_week` / `get_year` output semantics.
    ///
    /// Input contract:
    /// - `weeks`: list of `(year, iso_week)` tuples to aggregate over. Order is
    ///   preserved in the return value.
    /// - `work_details`, `shiftplan_reports`, `extra_hours`: three fully-loaded
    ///   slice references. The helper filters each slice per-week internally.
    ///   Callers own the bulk load (single-week callers may pass the week's own
    ///   query result; year-batch callers pass a year's worth in one shot).
    ///
    /// Per-week semantics (byte-identical to the pre-refactor `get_week` body,
    /// D-52-08/09):
    /// - `find_working_hours_for_calendar_week` selects contract rows.
    /// - `apply_weekly_cap` fires PER-WEEK against the raw shiftplan hours,
    ///   NEVER aggregated across the year (CVC-06).
    /// - `derive_hours_for_range` (async DAO, per-person per-week).
    /// - `build_derived_holiday_map` (async, per-person per-week).
    /// - `sales_person_service.get` + `is_paid` filter run per-person per-week
    ///   (R9 / D-06 / CVC-10). This is intentionally NOT hoisted out.
    /// - No `shortday_gate` toggle read — Chain-C gating remains in
    ///   `booking_information.get_weekly_summary` (D-52-09, R8).
    ///
    /// Phase 52 Follow-Up (WOP-04): callers pass two in-memory indexes to
    /// eliminate O(N_persons × N_weeks) per-iteration cost:
    /// - `sales_person_index`: `HashMap<Uuid, SalesPerson>` — replaces per-person
    ///   per-week `sales_person_service.get(sp_id, ...)` DAO roundtrip with an
    ///   O(1) lookup. Callers build it from `sales_person_service.get_all(...)`.
    /// - `working_hours_by_sp`: `HashMap<Uuid, Arc<[EmployeeWorkDetails]>>` —
    ///   pre-bucketed by sales_person_id ONCE before the week loop, so
    ///   `find_working_hours_for_calendar_week` runs against a small per-person
    ///   slice instead of re-bucketing every week. Callers build it once from
    ///   `work_details` before delegating.
    ///
    /// Both indexes are byte-identical to the pre-follow-up shape (same rows,
    /// same order per-person after the internal grouping). No new DAO calls.
    ///
    /// Phase 52 Follow-Up #2 (WOP-04): three additional year-scope caches are
    /// built ONCE at the top of the helper (before the week loop) and replace
    /// three per-(person × week) chains that dominated post-follow-up latency:
    /// - `holiday_auto_credit` toggle read → ONCE per `assemble_weeks` call
    ///   (was: 9 588 reads per year-request). `Unauthorized → None` preserved
    ///   (D-25-05).
    /// - `special_day_service.get_by_week` per unique (year, week) in `weeks`
    ///   (was: N_persons × N_weeks calls = 11 466 per year-request). Bulk load
    ///   is skipped entirely when there are no absences to filter AND the
    ///   auto-credit toggle is off — preserving test-mock ergonomics.
    /// - `absence_service.derive_hours_for_range` → in-memory computation from
    ///   caller-passed `all_absences` (was: N_persons × N_weeks async calls
    ///   = 4 746 per year-request). Semantics preserved via
    ///   `derive_hours_for_week_pure`. Permission semantics: `all_absences` is
    ///   loaded by the caller with `Authentication::Full` (mirrors the
    ///   pre-follow-up-1 `sales_person_service.get_all` pattern) — the outer
    ///   permission gate on `get_week`/`get_year` remains authoritative.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn assemble_weeks(
        &self,
        weeks: &[(u32, u8)],
        work_details: &[EmployeeWorkDetails],
        shiftplan_reports: &[ShiftplanReportDay],
        extra_hours: &[ExtraHours],
        all_absences: &[AbsencePeriod],
        sales_person_index: &HashMap<Uuid, SalesPerson>,
        working_hours_by_sp: &HashMap<Uuid, Arc<[EmployeeWorkDetails]>>,
        context: Authentication<Deps::Context>,
        tx: Option<Deps::Transaction>,
    ) -> Result<Vec<(u8, Arc<[ShortEmployeeReport]>)>, ServiceError> {
        let _ = work_details; // Retained in signature for byte-identity of caller shape (Wave-2 contract).
        let _ = &tx; // tx is retained for signature compat; per-iteration DAO calls no longer use it.
        let mut assembled: Vec<(u8, Arc<[ShortEmployeeReport]>)> =
            Vec::with_capacity(weeks.len());

        // ─── Follow-Up #2: one-shot year-scope preloads ─────────────────────
        //
        // (a) `holiday_auto_credit` toggle → cutoff date. Read ONCE with the
        //     caller's `context`. Preserves the `Unauthorized → None` shape
        //     from the pre-follow-up per-call implementation (D-25-05).
        let toggle_value_res = self
            .toggle_service
            .get_toggle_value("holiday_auto_credit", context.clone(), None)
            .await;
        let cutoff: Option<time::Date> = match toggle_value_res {
            Ok(v) => v.as_deref().and_then(|s| {
                time::Date::parse(s, &time::format_description::well_known::Iso8601::DEFAULT).ok()
            }),
            Err(ServiceError::Unauthorized) => None,
            Err(e) => return Err(e),
        };

        // (b) Bucketing of `all_absences` by sales_person_id — pure in-memory.
        //     Only referenced from the pure derive helper; ordering preserved
        //     (find_all returns rows ordered by (sales_person_id, from_date),
        //     so per-person entries stay in from_date order).
        let mut absences_by_sp: HashMap<Uuid, Vec<&AbsencePeriod>> = HashMap::new();
        for ap in all_absences.iter() {
            if ap.deleted.is_some() {
                continue;
            }
            absences_by_sp.entry(ap.sales_person_id).or_default().push(ap);
        }
        let has_any_absences = !absences_by_sp.is_empty();

        // (c) Special-day preload per unique (year, week) — only when needed.
        //     Needed when either (i) the holiday-auto-credit toggle is on
        //     (`cutoff.is_some()`) so `build_derived_holiday_map_for_week_pure`
        //     has data, OR (ii) at least one absence exists so
        //     `derive_hours_for_week_pure` can skip holiday days.
        //     Otherwise skip the DAO calls entirely (byte-identical: both pure
        //     helpers return empty when their inputs are empty).
        let need_special_days = cutoff.is_some() || has_any_absences;
        let mut special_days_by_week: HashMap<(u32, u8), Arc<[SpecialDay]>> = HashMap::new();
        if need_special_days {
            let mut unique_weeks: HashSet<(u32, u8)> = HashSet::new();
            for &(y, w) in weeks {
                unique_weeks.insert((y, w));
            }
            for (y, w) in unique_weeks {
                let sds = self
                    .special_day_service
                    .get_by_week(y, w, context.clone())
                    .await?;
                special_days_by_week.insert((y, w), sds);
            }
        }

        // (d) Per-(year, week) holiday-date set for the absence-skip filter.
        //     Precomputed once to keep the per-person loop pure.
        let mut holidays_by_week: HashMap<(u32, u8), BTreeSet<time::Date>> = HashMap::new();
        if has_any_absences {
            for (&(y, w), sds) in special_days_by_week.iter() {
                let mut holidays: BTreeSet<time::Date> = BTreeSet::new();
                for sd in sds.iter() {
                    if sd.deleted.is_some() {
                        continue;
                    }
                    if sd.day_type != SpecialDayType::Holiday {
                        continue;
                    }
                    if let Ok(date) = time::Date::from_iso_week_date(
                        sd.year as i32,
                        sd.calendar_week,
                        sd.day_of_week.into(),
                    ) {
                        holidays.insert(date);
                    }
                }
                holidays_by_week.insert((y, w), holidays);
            }
        }

        let empty_special_days: Arc<[SpecialDay]> = Arc::from(Vec::<SpecialDay>::new());
        let empty_holidays: BTreeSet<time::Date> = BTreeSet::new();

        for &(year, week) in weeks {
            let shiftplan_report = shiftplan_reports
                .iter()
                .filter(|r| r.year == year && r.calendar_week == week)
                .collect_to_hash_map_by(|r| r.sales_person_id);
            let extra_hours_bucket = extra_hours
                .iter()
                .filter(|eh| {
                    let sw = eh.to_date().as_shifty_week();
                    sw.year == year && sw.week == week
                })
                .collect_to_hash_map_by(|eh| eh.sales_person_id);

            let mut result: Vec<ShortEmployeeReport> = Vec::new();

            for (sales_person_id, working_hours) in working_hours_by_sp.iter() {
                let sales_person_id = *sales_person_id;
                let working_hours: &[EmployeeWorkDetails] = working_hours.as_ref();
                let raw_shiftplan_hours = shiftplan_report
                    .get(&sales_person_id)
                    .map(|r| r.iter().map(|r| r.hours).sum::<f32>())
                    .unwrap_or(0.0);
                let employee_extra_hours = extra_hours_bucket.get(&sales_person_id);
                let extra_working_hours = employee_extra_hours
                    .map(|eh| {
                        eh.iter()
                            .filter(|eh| eh.category.availability() == Availability::Available)
                            .map(|eh| eh.amount)
                            .sum::<f32>()
                    })
                    .unwrap_or(0.0);
                let abense_hours = employee_extra_hours
                    .map(|eh| {
                        eh.iter()
                            .filter(|eh| eh.category.availability() == Availability::Unavailable)
                            .map(|eh| eh.amount)
                            .sum::<f32>()
                    })
                    .unwrap_or(0.0);
                let vacation_hours = employee_extra_hours
                    .map(|eh| {
                        eh.iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::Vacation)
                            .map(|eh| eh.amount)
                            .sum::<f32>()
                    })
                    .unwrap_or(0.0);
                let sick_leave_hours = employee_extra_hours
                    .map(|eh| {
                        eh.iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::SickLeave)
                            .map(|eh| eh.amount)
                            .sum::<f32>()
                    })
                    .unwrap_or(0.0);
                let holiday_hours = employee_extra_hours
                    .map(|eh| {
                        eh.iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
                            .map(|eh| eh.amount)
                            .sum::<f32>()
                    })
                    .unwrap_or(0.0);
                let unavailable_hours = employee_extra_hours
                    .map(|eh| {
                        eh.iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::Unavailable)
                            .map(|eh| eh.amount)
                            .sum::<f32>()
                    })
                    .unwrap_or(0.0);
                let unpaid_leave_hours = employee_extra_hours
                    .map(|eh| {
                        eh.iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::UnpaidLeave)
                            .map(|eh| eh.amount)
                            .sum::<f32>()
                    })
                    .unwrap_or(0.0);
                let manual_volunteer_hours = employee_extra_hours
                    .map(|eh| {
                        eh.iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::VolunteerWork)
                            .map(|eh| eh.amount)
                            .sum::<f32>()
                    })
                    .unwrap_or(0.0);
                let custom_absence_hours: Arc<[CustomExtraHours]> = {
                    let mut map: HashMap<(Uuid, Arc<str>), f32> = HashMap::new();
                    if let Some(eh_list) = employee_extra_hours {
                        for eh_entry in eh_list.iter() {
                            if let ExtraHoursCategory::CustomExtraHours(lazy_load_custom_def) =
                                &eh_entry.category
                            {
                                if let Some(custom_def) = lazy_load_custom_def.get() {
                                    let key = (custom_def.id, custom_def.name.clone());
                                    *map.entry(key).or_insert(0.0) += eh_entry.amount;
                                }
                            }
                        }
                    }
                    map.into_iter()
                        .map(|((id, name), hours)| CustomExtraHours { id, name, hours })
                        .collect::<Vec<_>>()
                        .into()
                };
                // See `get_week` for the has_contract_row / no-contract-volunteer
                // rationale (quick-260624-ujk). In the year-batch path the same
                // invariant holds: `all_for_year` returns only contract rows.
                let has_contract_row =
                    find_working_hours_for_calendar_week(working_hours, year, week)
                        .next()
                        .is_some();
                let (planned_hours, dynamic_hours): (f32, f32) =
                    find_working_hours_for_calendar_week(working_hours, year, week)
                        .map(|wh| weight_for_week(year, week, wh))
                        .map(|wfw| (wfw.0, wfw.1))
                        .fold((0.0, 0.0), |(acc_a, acc_b), (a, b)| (acc_a + a, acc_b + b));
                let cap_active = find_working_hours_for_calendar_week(working_hours, year, week)
                    .any(|wh| wh.cap_planned_hours_to_expected);
                // Gap 1 (Phase 8.4 / CR-01) + Gap 2 (WR-01): additiver
                // absence_period-Merge, derived VOR apply_weekly_cap.
                //
                // Phase 52 Follow-Up #2 (WOP-04): the per-(person, week) async
                // `derive_hours_for_range` DAO chain (absence_dao + work_details
                // + special_day_service) has been replaced with the pure
                // `derive_hours_for_week_pure` helper reading from the
                // year-scope preloads built at the top of `assemble_weeks`.
                // Byte-identical to the DAO path — the pure helper mirrors
                // `AbsenceServiceImpl::derive_hours_for_range` for a single
                // ISO week (same iteration, same active-contract selection,
                // same dominant-category resolver, same per-week workdays cap).
                let absences_for_person: Vec<&AbsencePeriod> = absences_by_sp
                    .get(&sales_person_id)
                    .cloned()
                    .unwrap_or_default();
                let holidays_this_week = holidays_by_week
                    .get(&(year, week))
                    .unwrap_or(&empty_holidays);
                let derived = derive_hours_for_week_pure(
                    year,
                    week,
                    &absences_for_person,
                    working_hours,
                    holidays_this_week,
                );
                let mut absence_derived_vacation_hours = 0.0_f32;
                let mut absence_derived_sick_leave_hours = 0.0_f32;
                let mut absence_derived_unpaid_leave_hours = 0.0_f32;
                for resolved in derived.values() {
                    match resolved.category {
                        AbsenceCategory::Vacation => {
                            absence_derived_vacation_hours += resolved.hours
                        }
                        AbsenceCategory::SickLeave => {
                            absence_derived_sick_leave_hours += resolved.hours
                        }
                        AbsenceCategory::UnpaidLeave => {
                            absence_derived_unpaid_leave_hours += resolved.hours
                        }
                    }
                }
                let abense_hours_for_balance = if !has_contract_row || planned_hours <= 0.0 {
                    0.0f32
                } else {
                    abense_hours
                };
                let absence_derived_balance_total = if !has_contract_row || planned_hours <= 0.0 {
                    0.0f32
                } else {
                    absence_derived_vacation_hours
                        + absence_derived_sick_leave_hours
                        + absence_derived_unpaid_leave_hours
                };
                // 4th injection point (Phase 34 / HSP-01/02, D-34-01).
                //
                // Phase 52 Follow-Up #2 (WOP-04): async
                // `build_derived_holiday_map` replaced with pure
                // `build_derived_holiday_map_for_week_pure`. Same inputs
                // (working_hours + per-week extra_hours), same cutoff (read
                // ONCE at the top of `assemble_weeks`), same special_day slice
                // (pre-loaded per unique (year, week)).
                let employee_extra_hours_owned: Vec<ExtraHours> = employee_extra_hours
                    .map(|arc| arc.iter().map(|r| (*r).clone()).collect())
                    .unwrap_or_default();
                let special_days_this_week = special_days_by_week
                    .get(&(year, week))
                    .unwrap_or(&empty_special_days);
                let derived_holiday_map = build_derived_holiday_map_for_week_pure(
                    year,
                    week,
                    cutoff,
                    special_days_this_week,
                    working_hours,
                    &employee_extra_hours_owned,
                );
                let derived_holiday_for_week: f32 = derived_holiday_map.values().sum();
                let holiday_derived_gated = if !has_contract_row || planned_hours <= 0.0 {
                    0.0f32
                } else {
                    derived_holiday_for_week
                };
                let holiday_hours = holiday_hours + holiday_derived_gated;
                // HSP-03 band guard / CR-01 — see `get_week` doc.
                let expected_hours_for_cap =
                    planned_hours - abense_hours_for_balance - absence_derived_balance_total;
                let (shiftplan_hours, auto_volunteer_hours) =
                    apply_weekly_cap(cap_active, raw_shiftplan_hours, expected_hours_for_cap);
                let expected_hours = expected_hours_for_cap - holiday_derived_gated;
                let shiftplan_paid = if has_contract_row { shiftplan_hours } else { 0.0 };
                let no_contract_volunteer = if has_contract_row {
                    0.0
                } else {
                    shiftplan_hours
                };
                let volunteer_hours =
                    manual_volunteer_hours + auto_volunteer_hours + no_contract_volunteer;
                let dynamic_hours =
                    dynamic_hours - abense_hours_for_balance - absence_derived_balance_total;
                let overall_hours = shiftplan_paid + extra_working_hours;
                let balance_hours = overall_hours - expected_hours;
                // D-06 / CVC-10: is_paid-Filter — MUST stay per-person per-week
                // (R9 / T-52-03 mitigation).
                //
                // Phase 52 Follow-Up (WOP-04): resolve the sales person via the
                // caller-provided in-memory index (built once from
                // `sales_person_service.get_all(...)`) instead of a per-person
                // per-week DAO roundtrip. Fallback to the legacy `.get(...)` DAO
                // call preserves byte-identity when a sales_person_id shows up in
                // `working_hours_by_sp` but not in the index (should not happen
                // in practice — both feed from the same underlying tables — but
                // the fallback keeps the pre-follow-up error shape).
                let sales_person = match sales_person_index.get(&sales_person_id) {
                    Some(sp) => sp.clone(),
                    None => {
                        self.sales_person_service
                            .get(sales_person_id, Authentication::Full, tx.clone())
                            .await?
                    }
                };
                if !sales_person.is_paid.unwrap_or(false) {
                    continue;
                }
                result.push(ShortEmployeeReport {
                    sales_person: Arc::new(sales_person),
                    balance_hours,
                    dynamic_hours,
                    expected_hours,
                    overall_hours,
                    vacation_hours: vacation_hours + absence_derived_vacation_hours,
                    sick_leave_hours: sick_leave_hours + absence_derived_sick_leave_hours,
                    holiday_hours,
                    unavailable_hours,
                    unpaid_leave_hours: unpaid_leave_hours + absence_derived_unpaid_leave_hours,
                    volunteer_hours,
                    custom_absence_hours,
                });
            }

            assembled.push((week, result.into()));
        }

        Ok(assembled)
    }
}

#[async_trait]
impl<Deps: ReportingServiceDeps> service::reporting::ReportingService
    for ReportingServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_reports_for_all_employees(
        &self,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError> {
        let until_week = until_week.min(time::util::weeks_in_year(year as i32));

        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let working_hours = self
            .employee_work_details_service
            .all(Authentication::Full, tx.clone())
            .await?;

        let employees = self
            .sales_person_service
            .get_all(Authentication::Full, tx.clone())
            .await?;
        let mut short_employee_report: Vec<ShortEmployeeReport> = Vec::new();
        for paid_employee in employees
            .iter()
            .filter(|employee| employee.is_paid.unwrap_or(false))
        {
            let detailed_shiftplan_report = self
                .shiftplan_report_service
                .extract_shiftplan_report(
                    paid_employee.id,
                    ShiftyDate::first_day_in_year(year),
                    ShiftyWeek::new(year, until_week).as_date(DayOfWeek::Sunday),
                    Authentication::Full,
                    tx.clone(),
                )
                .await?;

            let working_hours: Arc<[EmployeeWorkDetails]> = working_hours
                .iter()
                .filter(|wh| wh.sales_person_id == paid_employee.id)
                .cloned()
                .collect();
            let extra_hours_array = self
                .extra_hours_service
                .find_by_sales_person_id_and_year(
                    paid_employee.id,
                    year,
                    until_week,
                    Authentication::Full,
                    tx.clone(),
                )
                .await?;
            let previous_year_carryover = self
                .carryover_service
                .get_carryover(
                    paid_employee.id,
                    year - 1,
                    Authentication::Full,
                    tx.clone(),
                )
                .await?
                .map(|c| c.carryover_hours)
                .unwrap_or(0.0);

            let additional_weeks = if until_week >= time::util::weeks_in_year(year as i32) {
                1
            } else {
                0
            };
            #[derive(Default)]
            struct WeeklyHours {
                shiftplan_hours: f32,
                extra_working_hours: f32,
                absense_hours: f32,
                planned_hours: f32,
                dynamic_hours: f32,
                vacation_hours: f32,
                sick_leave_hours: f32,
                holiday_hours: f32,
                unavailable_hours: f32,
                unpaid_leave_hours: f32,
                volunteer_hours: f32,
                custom_absence_hours: HashMap<(Uuid, Arc<str>), f32>,
                /// Per-Woche gegatete derived Balance-Reduktion (0.0 bei dynamischen Wochen).
                /// Gap (Phase 8.4 / CR-01): symmetrisch zu absense_hours (Z.263) und zur
                /// Referenz hours_per_week (Z.988). Verhindert dynamic-contract Balance-Asymmetrie (M-02).
                absence_derived_balance_hours: f32,
            }

            // Gap (Phase 8.4 / CR-01): derived-Map VOR dem per-Woche-Fold berechnen, damit
            // jede Woche ihren gegateten Beitrag zur Balance-Reduktion bestimmen kann.
            // Range exakt auf das Report-Jahr begrenzt: [first_day_in_year(year) .. until_week-Sonntag].
            let derived = self
                .absence_service
                .derive_hours_for_range(
                    ShiftyDate::first_day_in_year(year).to_date(),
                    ShiftyWeek::new(year, until_week)
                        .as_date(DayOfWeek::Sunday)
                        .to_date(),
                    paid_employee.id,
                    context.clone(),
                    tx.clone(),
                )
                .await?;

            // Phase 25: Pre-compute per-employee derived-holiday map for the year range.
            // Empty when toggle has no value (automation off, D-25-05).
            let derived_holiday = self
                .build_derived_holiday_map(
                    ShiftyDate::first_day_in_year(year),
                    ShiftyWeek::new(year, until_week).as_date(DayOfWeek::Sunday),
                    &working_hours,
                    &extra_hours_array,
                    context.clone(),
                )
                .await?;

            let weekly_hours = (0..=until_week + additional_weeks)
                .map(|week| {
                    let target_year = year;
                    let year = if week == 0 {
                        year - 1
                    } else if week > time::util::weeks_in_year(year as i32) {
                        year + 1
                    } else {
                        year
                    };
                    let week = if week == 0 {
                        time::util::weeks_in_year(year as i32)
                    } else if week > time::util::weeks_in_year(year as i32) {
                        week - time::util::weeks_in_year(year as i32)
                    } else {
                        week
                    };

                    // User-Regel (quick-260624-ujk): Eine KW OHNE EmployeeWorkDetails-Zeile bedeutet,
                    // dass der Mitarbeiter in dieser Woche KEINEN Vertrag hat. Geleistete Shiftplan-Stunden
                    // sind dann Ehrenamt (volunteer), kein bezahltes Soll=Ist. Das unterscheidet sich vom
                    // dynamischen Vertrag (Zeile vorhanden, expected=0): dort gilt weiterhin Soll=Ist.
                    //
                    // Abgrenzung booking_information-Band-Logik: Die booking_information-Baender
                    // (committed_voluntary Band 1, volunteer_surplus Band 2) sind auf is_paid=false
                    // (unbezahlte Freiwillige) gegated. Dieser Pfad betrifft bezahlte Mitarbeiter
                    // ohne Vertragszeile. Beide Pfade sind disjunkt — keine Doppelzaehlung.
                    let has_contract_row =
                        find_working_hours_for_calendar_week(&working_hours, year, week)
                            .next()
                            .is_some();
                    let (expected_hours, dynamic_hours) =
                        find_working_hours_for_calendar_week(&working_hours, year, week)
                            .map(|wh| weight_for_week(year, week,
                                &wh.with_to_date(
                                    wh.to_date()
                                        .unwrap_or(ShiftyDate::last_day_in_year(target_year))
                                        .min(ShiftyDate::last_day_in_year(target_year))
                                    ).with_from_date(
                                        wh.from_date()
                                            .unwrap_or(ShiftyDate::first_day_in_year(target_year))
                                            .max(ShiftyDate::first_day_in_year(target_year))
                                    )
                                ))
                            .map(|(expected_hours, dynamic_hours, _, _)| (expected_hours, dynamic_hours))
                            .fold((0.0, 0.0), |(acc_a, acc_b), (a, b)| (acc_a + a, acc_b + b));
                    // If expected hours is 0 or less, the planned hours and the working hours are the same
                    // because the balance should never be affected in this case.
                    let raw_shiftplan_hours: f32 = detailed_shiftplan_report
                        .iter()
                        .filter(|shift_plan_item| {
                            shift_plan_item.year == year && shift_plan_item.calendar_week == week && shift_plan_item.to_date().map(|d| d.to_date().year() as u32).ok() == Some(target_year)
                        })
                        .map(|shift_plan_item| shift_plan_item.hours)
                        .sum();
                    let cap_active = find_working_hours_for_calendar_week(&working_hours, year, week)
                        .any(|wh| wh.cap_planned_hours_to_expected);
                    let (shiftplan_hours, auto_volunteer_hours) =
                        apply_weekly_cap(cap_active, raw_shiftplan_hours, expected_hours);
                    // Per-Woche gegatete derived Balance-Reduktion (Referenz: hours_per_week Z.988-996).
                    // Bei dynamischen Wochen (expected_hours <= 0.0) ist der Beitrag 0.0 —
                    // symmetrisch zu absense_hours (Z.263 weiter unten).
                    let absence_derived_balance_hours = if expected_hours <= 0.0 {
                        0.0f32
                    } else {
                        derived
                            .iter()
                            .filter(|(d, _)| {
                                let w = ShiftyDate::from(**d).as_shifty_week();
                                w.year == year && w.week == week
                            })
                            .map(|(_, r)| r.hours)
                            .sum::<f32>()
                    };
                    if !has_contract_row {
                        // Ehrenamt-Pfad (quick-260624-ujk): Keine Vertragszeile fuer diese KW.
                        // Shiftplan-Stunden fliessen als Ehrenamt, NICHT in overall/planned.
                        // ExtraWork bleibt in extra_working_hours (explizit erfasste bezahlte Leistung).
                        let extra_work: f32 = extra_hours_array
                            .iter()
                            .filter(|extra_hours| {
                                extra_hours.category == ExtraHoursCategory::ExtraWork
                                &&
                                extra_hours.date_time.iso_week() == week
                                    && extra_hours.date_time.year() as u32 == year
                            })
                            .map(|extra_hours| extra_hours.amount)
                            .sum();
                        WeeklyHours {
                            shiftplan_hours: 0.0,
                            extra_working_hours: extra_work,
                            absense_hours: 0.0,
                            planned_hours: 0.0,
                            dynamic_hours,
                            vacation_hours: 0.0,
                            sick_leave_hours: 0.0,
                            holiday_hours: 0.0,
                            unavailable_hours: 0.0,
                            unpaid_leave_hours: 0.0,
                            volunteer_hours: auto_volunteer_hours + shiftplan_hours,
                            custom_absence_hours: HashMap::new(),
                            absence_derived_balance_hours: 0.0,
                        }
                    } else if expected_hours <= 0.0 {
                        // Dynamischer Vertrag (Zeile vorhanden, expected=0): Soll=Ist.
                        let extra_work: f32 = extra_hours_array
                            .iter()
                            .filter(|extra_hours| {
                                extra_hours.category == ExtraHoursCategory::ExtraWork
                                &&
                                extra_hours.date_time.iso_week() == week
                                    && extra_hours.date_time.year() as u32 == year
                            })
                            .map(|extra_hours| extra_hours.amount)
                            .sum();
                        let overall_hours = extra_work + shiftplan_hours;
                        WeeklyHours {
                            shiftplan_hours,
                            extra_working_hours: extra_work,
                            absense_hours: 0.0,
                            planned_hours: overall_hours,
                            dynamic_hours,
                            vacation_hours: 0.0,
                            sick_leave_hours: 0.0,
                            holiday_hours: 0.0,
                            unavailable_hours: 0.0,
                            unpaid_leave_hours: 0.0,
                            volunteer_hours: auto_volunteer_hours,
                            custom_absence_hours: HashMap::new(),
                            absence_derived_balance_hours: 0.0,
                        }
                    } else {
                        let week_extra_hours: Vec<_> = extra_hours_array
                            .iter()
                            .filter(|eh| eh.date_time.iso_week() == week
                                && eh.date_time.year() as u32 == year)
                            .collect();
                        let extra_working_hours = week_extra_hours
                            .iter()
                            .filter(|eh| eh.category.as_report_type() == ReportType::WorkingHours)
                            .map(|eh| eh.amount)
                            .sum::<f32>();
                        let absense_hours = week_extra_hours
                            .iter()
                            .filter(|eh| eh.category.as_report_type() == ReportType::AbsenceHours)
                            .map(|eh| eh.amount)
                            .sum::<f32>();
                        let vacation_hours = week_extra_hours
                            .iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::Vacation)
                            .map(|eh| eh.amount)
                            .sum::<f32>();
                        let sick_leave_hours = week_extra_hours
                            .iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::SickLeave)
                            .map(|eh| eh.amount)
                            .sum::<f32>();
                        let manual_holiday_hours = week_extra_hours
                            .iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
                            .map(|eh| eh.amount)
                            .sum::<f32>();
                        // Phase 25 (injection point 1b): derived holiday for this (year, week).
                        // Must also be added to absense_hours (Pitfall 3 — Holiday is AbsenceHours).
                        let derived_holiday_for_week: f32 = derived_holiday
                            .iter()
                            .filter(|(date, _)| {
                                let w = ShiftyDate::from(**date).as_shifty_week();
                                w.year == year && w.week == week
                            })
                            .map(|(_, h)| h)
                            .sum();
                        let holiday_hours = manual_holiday_hours + derived_holiday_for_week;
                        let absense_hours = absense_hours + derived_holiday_for_week;
                        let unavailable_hours = week_extra_hours
                            .iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::Unavailable)
                            .map(|eh| eh.amount)
                            .sum::<f32>();
                        let mut custom_absence_hours: HashMap<(Uuid, Arc<str>), f32> = HashMap::new();
                        for eh_entry in week_extra_hours.iter() {
                            if let ExtraHoursCategory::CustomExtraHours(lazy_load_custom_def) =
                                &eh_entry.category
                            {
                                if let Some(custom_def) = lazy_load_custom_def.get() {
                                    let key = (custom_def.id, custom_def.name.clone());
                                    *custom_absence_hours.entry(key).or_insert(0.0) += eh_entry.amount;
                                }
                            }
                        }
                        let unpaid_leave_hours = week_extra_hours
                            .iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::UnpaidLeave)
                            .map(|eh| eh.amount)
                            .sum::<f32>();
                        let volunteer_hours = week_extra_hours
                            .iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::VolunteerWork)
                            .map(|eh| eh.amount)
                            .sum::<f32>()
                            + auto_volunteer_hours;
                        WeeklyHours {
                            shiftplan_hours,
                            extra_working_hours,
                            absense_hours,
                            planned_hours: expected_hours,
                            dynamic_hours,
                            vacation_hours,
                            sick_leave_hours,
                            holiday_hours,
                            unavailable_hours,
                            unpaid_leave_hours,
                            volunteer_hours,
                            custom_absence_hours,
                            absence_derived_balance_hours,
                        }
                    }
                })
                .fold(
                    WeeklyHours::default(),
                    |mut acc, week| {
                        acc.shiftplan_hours += week.shiftplan_hours;
                        acc.extra_working_hours += week.extra_working_hours;
                        acc.absense_hours += week.absense_hours;
                        acc.planned_hours += week.planned_hours;
                        acc.dynamic_hours += week.dynamic_hours;
                        acc.vacation_hours += week.vacation_hours;
                        acc.sick_leave_hours += week.sick_leave_hours;
                        acc.holiday_hours += week.holiday_hours;
                        acc.unavailable_hours += week.unavailable_hours;
                        acc.unpaid_leave_hours += week.unpaid_leave_hours;
                        acc.volunteer_hours += week.volunteer_hours;
                        acc.absence_derived_balance_hours += week.absence_derived_balance_hours;
                        for ((id, name), hours) in week.custom_absence_hours {
                            *acc.custom_absence_hours.entry((id, name)).or_insert(0.0) += hours;
                        }
                        acc
                    },
                );
            let custom_absence_hours: Arc<[CustomExtraHours]> = weekly_hours.custom_absence_hours
                .into_iter()
                .map(|((id, name), hours)| CustomExtraHours { id, name, hours })
                .collect::<Vec<_>>()
                .into();
            // Ungegate Display-Jahreslumpen (fuer vacation_hours/sick_leave_hours/unpaid_leave_hours).
            // DISPLAY bleibt additiv und UNGEGATED — die derived Stunden erscheinen in den Display-Spalten
            // unabhaengig vom Vertragstyp (Truth 3 & 4, test_all_employees_additive_merge=12h).
            let mut absence_derived_vacation_hours = 0.0_f32;
            let mut absence_derived_sick_leave_hours = 0.0_f32;
            let mut absence_derived_unpaid_leave_hours = 0.0_f32;
            for resolved in derived.values() {
                match resolved.category {
                    AbsenceCategory::Vacation => absence_derived_vacation_hours += resolved.hours,
                    AbsenceCategory::SickLeave => absence_derived_sick_leave_hours += resolved.hours,
                    AbsenceCategory::UnpaidLeave => absence_derived_unpaid_leave_hours += resolved.hours,
                }
            }
            // Gap (Phase 8.4 / CR-01): per-Woche-gegatete Balance-Reduktion. Display bleibt ungegate
            // (Jahreslumpen absence_derived_vacation/sick/unpaid), aber die expected/balance-Reduktion
            // zaehlt nur die derived Stunden der Wochen MIT Vertragsarbeitszeit — symmetrisch zu
            // absense_hours (dynamic-Zweig Z.263) und zur Referenz hours_per_week (Z.988).
            // Verhindert dynamic-contract Balance-Asymmetrie (M-02).
            let expected_hours = weekly_hours.planned_hours - weekly_hours.absense_hours - weekly_hours.absence_derived_balance_hours;
            let dynamic_hours = weekly_hours.dynamic_hours - weekly_hours.absense_hours - weekly_hours.absence_derived_balance_hours;
            let overall_hours = weekly_hours.shiftplan_hours + weekly_hours.extra_working_hours;
            let balance_hours = overall_hours - expected_hours + previous_year_carryover;
            short_employee_report.push(ShortEmployeeReport {
                sales_person: Arc::new(paid_employee.clone()),
                balance_hours,
                dynamic_hours,
                expected_hours,
                overall_hours,
                vacation_hours: weekly_hours.vacation_hours + absence_derived_vacation_hours,
                sick_leave_hours: weekly_hours.sick_leave_hours + absence_derived_sick_leave_hours,
                holiday_hours: weekly_hours.holiday_hours,
                unavailable_hours: weekly_hours.unavailable_hours,
                unpaid_leave_hours: weekly_hours.unpaid_leave_hours + absence_derived_unpaid_leave_hours,
                volunteer_hours: weekly_hours.volunteer_hours,
                custom_absence_hours,
            });
        }
        Ok(short_employee_report.into())
    }

    async fn get_report_for_employee(
        &self,
        sales_person_id: &Uuid,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeReport, ServiceError> {
        let first_day_of_year =
            ShiftyDate::first_day_in_year(year);
        let until_week = until_week.min(time::util::weeks_in_year(year as i32));
        let to_date = if until_week == time::util::weeks_in_year(year as i32) {
            ShiftyDate::last_day_in_year(year)
        } else {
            ShiftyDate::new(year, until_week, DayOfWeek::Sunday)?
        };

        self.get_report_for_employee_range(
            sales_person_id,
            first_day_of_year,
            to_date.min(ShiftyDate::last_day_in_year(year)),
            true,
            context,
            tx,
        )
        .await
    }

    async fn get_report_for_employee_range(
        &self,
        sales_person_id: &Uuid,
        from_date: ShiftyDate,
        to_date: ShiftyDate,
        include_carryover: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeReport, ServiceError> {
        let (hr_permission, user_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                *sales_person_id,
                context.clone(),
                tx.clone()
            ),
        );
        hr_permission.or(user_permission)?;

        let sales_person = self
            .sales_person_service
            .get(*sales_person_id, context.clone(), tx.clone())
            .await?;
        let working_hours = self
            .employee_work_details_service
            .find_by_sales_person_id(*sales_person_id, Authentication::Full, tx.clone())
            .await?;
        let shiftplan_report = self
            .shiftplan_report_service
            .extract_shiftplan_report(
                *sales_person_id,
                from_date,
                to_date,
                Authentication::Full,
                tx.clone(),
            )
            .await?;
        let extra_hours = self
            .extra_hours_service
            .find_by_sales_person_id_and_year_range(
                *sales_person_id,
                from_date,
                to_date,
                Authentication::Full,
                tx.clone(),
            )
            .await?;

        // Additiver Merge (Phase 8.4 / D-01): immer beide Quellen.
        // Die lebenden `extra_hours` fliessen ungefiltert (deleted IS NULL-
        // Filterung passiert bereits im DAO — D-02). Konvertierte extra_hours
        // sind via soft_delete_bulk (Phase 8.2) soft-deleted und fallen aus
        // dem DAO-Load bereits heraus (keine Doppelzaehlung per-row).
        let derived = self
            .absence_service
            .derive_hours_for_range(
                from_date.to_date(),
                to_date.to_date(),
                *sales_person_id,
                context.clone(),
                tx.clone(),
            )
            .await?;
        // D-18-04: year-lump absence_derived_*_hours removed — since Task 1 (UV-05) the
        // per-week GroupedReportHours already includes derived category hours, so the
        // top-level fields are sourced from by_week (single source of truth). The old
        // year-lump fold (absence_derived_vacation_hours etc.) is no longer needed here.

        // Phase 25: Precompute per-employee derived-holiday map for the range.
        // Returns empty map when toggle has no value (automation off, D-25-05).
        let derived_holiday = self
            .build_derived_holiday_map(
                from_date,
                to_date,
                &working_hours,
                &extra_hours,
                context.clone(),
            )
            .await?;

        // Hinweis: Das rohe, ungedeckelte shiftplan_hours wird bewusst NICHT mehr
        // fuer overall/balance/shiftplan_hours verwendet (Debug
        // `report-ehrenamt-gesamtstunden`). Der per-Woche gedeckelte Wert
        // `shiftplan_hours_by_week` (siehe unten) ist die einzige Quelle.
        let overall_extra_work_hours = extra_hours
            .iter()
            .filter(|eh| {
                eh.to_date() >= from_date
                    && eh.to_date() <= to_date
                    && eh.category.as_report_type() == ReportType::WorkingHours
            })
            .map(|eh| eh.amount)
            .sum::<f32>();
        let by_week = hours_per_week(
            &shiftplan_report,
            &extra_hours,
            &working_hours,
            &derived,
            &derived_holiday,
            from_date,
            to_date,
        )?;
        let shiftplan_hours_by_week = by_week.iter().map(|week| week.shiftplan_hours).sum::<f32>();
        tracing::info!("Shiftplan hours: {}", shiftplan_hours_by_week);
        let (vacation_days, sick_leave_days, holiday_days, absence_days) = by_week.iter().fold(
            (0.0, 0.0, 0.0, 0.0),
            |(vacation_days, sick_leave_days, holiday_days, absence_days), week| {
                (
                    vacation_days + week.vacation_days(),
                    sick_leave_days + week.sick_leave_days(),
                    holiday_days + week.holiday_days(),
                    absence_days + week.absence_days(),
                )
            },
        );

        let planned_hours: f32 = by_week.iter().map(|week| week.expected_hours).sum();
        let dynamic_hours: f32 = by_week.iter().map(|week| week.dynamic_hours).sum();
        let vacation_entitlement = working_hours
            .iter()
            .map(|wh| wh.vacation_days_for_year(from_date.year()))
            .sum::<f32>()
            .round();
        let (previous_year_carryover, previous_year_vacation) = if include_carryover { self
            .carryover_service
            .get_carryover(
                *sales_person_id,
                from_date.year() - 1,
                Authentication::Full,
                tx.clone(),
            )
            .await?
            .map(|c| (c.carryover_hours, c.vacation))
            .unwrap_or((0.0, 0))
        } else {
            (0.0, 0)
        };

        let aggregated_custom_extra_hours: Arc<[CustomExtraHours]> = {
            let mut map: HashMap<(Uuid, Arc<str>), f32> = HashMap::new();
            for week_report in by_week.iter() {
                for custom_hour_entry in week_report.custom_extra_hours.iter() {
                    *map.entry((custom_hour_entry.id, custom_hour_entry.name.clone()))
                        .or_insert(0.0) += custom_hour_entry.hours;
                }
            }
            map.into_iter()
                .map(|((id, name), hours)| CustomExtraHours { id, name, hours })
                .collect::<Vec<_>>()
                .into()
        };

        // Debug `report-ehrenamt-gesamtstunden` / Phase-15 D-01: overall_hours,
        // balance_hours und shiftplan_hours muessen den per-Woche GEDECKELTEN
        // Wert (`shiftplan_hours_by_week`, via apply_weekly_cap) verwenden — NICHT
        // das rohe ungedeckelte `shiftplan_hours` (Z.577). Sonst leakt der
        // Cap-Ueberlauf (= auto_volunteer / Ehrenamt-Anteil) in die Gesamtstunden.
        // Der Ueberlauf zaehlt korrekt in volunteer_hours (by_week). Konsistent mit
        // get_reports_for_all_employees, das ebenfalls den gedeckelten Wert nutzt.
        let employee_report = EmployeeReport {
            sales_person: Arc::new(sales_person),
            balance_hours: shiftplan_hours_by_week + overall_extra_work_hours - planned_hours
                + previous_year_carryover,
            overall_hours: shiftplan_hours_by_week + overall_extra_work_hours,
            expected_hours: planned_hours,
            dynamic_hours,
            shiftplan_hours: shiftplan_hours_by_week,
            holiday_days,
            vacation_carryover: previous_year_vacation,
            vacation_days,
            vacation_entitlement: vacation_entitlement + previous_year_vacation as f32,
            sick_leave_days,
            absence_days,
            extra_work_hours: extra_hours
                .iter()
                .filter(|extra_hours| extra_hours.category == ExtraHoursCategory::ExtraWork)
                .map(|extra_hours| extra_hours.amount)
                .sum(),
            // UV-05 / D-18-04: by_week is the SINGLE SOURCE OF TRUTH for category hours.
            // Per-week fields already include extra_hours + derived (Task 1 / hours_per_week).
            // Summing by_week eliminates the old year-lump double-count while keeping the
            // correct additive total (extra_hours-for-week + derived-for-week per week).
            vacation_hours: by_week.iter().map(|w| w.vacation_hours).sum::<f32>(),
            sick_leave_hours: by_week.iter().map(|w| w.sick_leave_hours).sum::<f32>(),
            // Phase 25 (injection point 1c): switch to by_week single source of truth
            // (Option A from RESEARCH). by_week.holiday_hours already includes both
            // manual extra_hours + derived holiday hours (from hours_per_week 1a).
            // The billing-period snapshot reads EmployeeReport.holiday_hours — this
            // is why point 1c matters for HSNAP-01 correctness.
            holiday_hours: by_week.iter().map(|w| w.holiday_hours).sum::<f32>(),
            volunteer_hours: by_week.iter().map(|w| w.volunteer_hours).sum::<f32>(),
            unpaid_leave_hours: by_week.iter().map(|w| w.unpaid_leave_hours).sum::<f32>(),
            carryover_hours: previous_year_carryover,
            by_week,
            by_month: Arc::new([]),
            custom_extra_hours: aggregated_custom_extra_hours,
        };

        Ok(employee_report)
    }

    async fn get_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError> {
        // Phase 52 (WOP-02): thin wrapper. All aggregation semantics live in
        // `assemble_weeks` (single source of truth shared with `get_year` in
        // Wave 4). We load the same three inputs as before, then delegate on a
        // 1-element `weeks` slice. Byte-identity is structural — no code path
        // change.
        //
        // Note: pre-refactor `get_week` did NOT run its own transaction envelope
        // (no `use_transaction` / `commit`) — it forwarded whatever `tx` the
        // caller passed to each DAO call and returned. We preserve that exact
        // shape here so mock-based unit tests (which do not stub `commit`) stay
        // byte-identical.
        //
        // Auth check is done by working_hours_service.
        let work_details = self
            .employee_work_details_service
            .all_for_week(week, year, context.clone(), tx.clone())
            .await?;
        let shiftplan_report = self
            .shiftplan_report_service
            .extract_shiftplan_report_for_week(year, week, Authentication::Full, tx.clone())
            .await?;
        let extra_hours = self
            .extra_hours_service
            .find_by_week(year, week, Authentication::Full, tx.clone())
            .await?;
        info!("Extra hours: {:?}", &extra_hours);

        // Phase 52 Follow-Up (WOP-04): pre-build the two in-memory indexes
        // (sales_person load-once + working_hours bucketed by sp_id) before
        // delegating. Single-week callers pay a small O(N_sp + N_wd) upfront
        // cost but save the per-person DAO roundtrip inside `assemble_weeks`.
        let all_sales_persons = self
            .sales_person_service
            .get_all(Authentication::Full, tx.clone())
            .await?;
        let sales_person_index: HashMap<Uuid, SalesPerson> = all_sales_persons
            .iter()
            .map(|sp| (sp.id, sp.clone()))
            .collect();
        let working_hours_by_sp: HashMap<Uuid, Arc<[EmployeeWorkDetails]>> = work_details
            .iter()
            .cloned()
            .collect_to_hash_map_by(|wh| wh.sales_person_id);
        // Phase 52 Follow-Up #2 (WOP-04): bulk-load all absences ONCE, then
        // pass through to `assemble_weeks` which computes per-person per-week
        // absence hours in-memory (was: N_persons × N_weeks async DAO chains).
        // `Authentication::Full` mirrors the load-once pattern already used
        // for `sales_person_service.get_all(...)` above; the outer permission
        // gate on the REST/service caller remains authoritative.
        let all_absences = self
            .absence_service
            .find_all(Authentication::Full, tx.clone())
            .await?;

        let mut assembled = self
            .assemble_weeks(
                &[(year, week)],
                &work_details,
                &shiftplan_report,
                &extra_hours,
                &all_absences,
                &sales_person_index,
                &working_hours_by_sp,
                context,
                tx,
            )
            .await?;

        Ok(assembled
            .pop()
            .map(|(_, reports)| reports)
            .unwrap_or_else(|| Arc::from(Vec::<ShortEmployeeReport>::new())))
    }

    async fn get_year(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[(u8, Arc<[ShortEmployeeReport]>)]>, ServiceError> {
        // Phase 52 (WOP-02): Batch-Variante von `get_week`. Drei Bulk-Load-
        // Roundtrips für das ganze Jahr statt 55×3, dann Delegation auf den
        // gemeinsamen `assemble_weeks`-Helper mit einem Vec über alle
        // ISO-Wochen des Jahres. Byte-Identität zu 55×`get_week` ist strukturell
        // garantiert (D-52-08 / D-52-09) — beide Pfade rufen denselben Helper mit
        // denselben Slice-Referenzen pro Woche auf.
        //
        // Wie `get_week` läuft dieser Pfad OHNE eigenen Transaction-Envelope
        // (kein `use_transaction`/`commit`): der Consumer entscheidet über
        // die TX-Lebenszeit; `tx.clone()` wird an die drei Bulk-Loads und an
        // `assemble_weeks` durchgereicht.
        let work_details = self
            .employee_work_details_service
            .all(Authentication::Full, tx.clone())
            .await?;
        // Follow-up #3: ISO-Wochenjahr-Batches (nicht Kalender-Jahr).
        // `assemble_weeks` bucketet Rows per ISO-Wochenjahr; die alten
        // `_for_year`/`find_by_year`-Methoden filterten kalendarisch und
        // verschluckten Rows an KW 1 / KW 53. Siehe SUMMARY der Follow-up-3.
        let shiftplan_reports = self
            .shiftplan_report_service
            .extract_shiftplan_report_for_iso_year(year, Authentication::Full, tx.clone())
            .await?;
        let extra_hours = self
            .extra_hours_service
            .find_by_iso_year(year, Authentication::Full, tx.clone())
            .await?;
        info!("Extra hours (year batch): {:?}", &extra_hours);

        // Phase 52 Follow-Up (WOP-04): pre-build the two in-memory indexes
        // ONCE, before iterating ~55 ISO weeks. This eliminates:
        //   1. `sales_person_service.get(sp_id, ...)` per (person × week) —
        //      replaced by `HashMap<Uuid, SalesPerson>` O(1) lookup.
        //   2. `collect_to_hash_map_by` on `work_details` per week + linear
        //      `find_working_hours_for_calendar_week` full-scans per (person,
        //      week) — replaced by pre-bucketed `HashMap<Uuid, Arc<[EmployeeWorkDetails]>>`.
        // Both indexes are pure in-memory rearrangements of data already loaded
        // by the bulk-load preamble above — no new DAO calls, byte-identical.
        let all_sales_persons = self
            .sales_person_service
            .get_all(Authentication::Full, tx.clone())
            .await?;
        let sales_person_index: HashMap<Uuid, SalesPerson> = all_sales_persons
            .iter()
            .map(|sp| (sp.id, sp.clone()))
            .collect();
        let working_hours_by_sp: HashMap<Uuid, Arc<[EmployeeWorkDetails]>> = work_details
            .iter()
            .cloned()
            .collect_to_hash_map_by(|wh| wh.sales_person_id);
        // Phase 52 Follow-Up #2 (WOP-04): year-batch absence bulk load. One
        // DAO roundtrip replaces N_persons × N_weeks per-(person, week)
        // `derive_hours_for_range` chains (~4 700 SQLite queries per request
        // pre-optimisation).
        let all_absences = self
            .absence_service
            .find_all(Authentication::Full, tx.clone())
            .await?;

        let weeks_in_year = time::util::weeks_in_year(year as i32);
        let weeks: Vec<(u32, u8)> = (1..=weeks_in_year).map(|w| (year, w)).collect();

        let assembled = self
            .assemble_weeks(
                &weeks,
                &work_details,
                &shiftplan_reports,
                &extra_hours,
                &all_absences,
                &sales_person_index,
                &working_hours_by_sp,
                context,
                tx,
            )
            .await?;

        Ok(assembled.into())
    }

    async fn get_employee_weekly_statistics(
        &self,
        sales_person_id: &Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<service::reporting::EmployeeWeeklyStatistics, ServiceError> {
        // STAT-01 / D-22-05: HR gate is the FIRST statement — no data fetched before auth.
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        // D-22-01: current year up to current ISO week.
        let today = self.clock_service.date_now();
        let (year, current_week, _) = today.to_iso_week_date();
        let year = year as u32;

        // D-22-06: reuse existing per-week data from get_report_for_employee.
        let report = self
            .get_report_for_employee(
                sales_person_id,
                year,
                current_week,
                context.clone(),
                tx,
            )
            .await?;

        // A-22-1 pure formula.
        let stats = service::reporting::average_worked_hours_per_week(&report.by_week);
        Ok(stats)
    }

    async fn get_employee_attendance_statistics(
        &self,
        sales_person_id: &Uuid,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<service::reporting::EmployeeAttendanceStatistics>, ServiceError> {
        // D-AVG-05: HR gate is the FIRST await — no data fetched before auth.
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        // v2.2 post-ship RPT-02-Fix: is_dynamic-Filter entfernt. Die Wochentag-
        // Anwesenheits-Verteilung wird für ALLE Mitarbeiter berechnet — auch
        // für non-flexible Rollen ist die pro-Wochentag-Verteilung sinnvoll
        // (zeigt die tatsächliche Belegung im Zeitraum, unabhängig vom Contract).

        // D-AVG-04: aggregate over the displayed report range.
        // Note: until_week clamping to weeks_in_year is done inside get_report_for_employee.
        let report = self
            .get_report_for_employee(sales_person_id, year, until_week, context, tx)
            .await?;

        // Flatten all per-week days and apply the RPT-01 pure aggregate fn.
        let all_days: Vec<WorkingHoursDay> = report
            .by_week
            .iter()
            .flat_map(|w| w.days.iter().cloned())
            .collect();
        // D-47-BE: counted_calendar_weeks = number of report weeks (year+until_week
        // clamped inside get_report_for_employee); one row per counted week.
        let counted_calendar_weeks = report.by_week.len() as u32;
        let stats =
            service::reporting::weekday_attendance_distribution(&all_days, counted_calendar_weeks);
        Ok(Some(stats))
    }
}

fn weight_for_week(
    year: u32,
    week: u8,
    employee_work_details: &EmployeeWorkDetails,
) -> (f32, f32, u8, f32) {
    let workdays: Arc<[time::Weekday]> = employee_work_details.potential_weekday_list();
    let all_potential_workdays = workdays.len() as u8;

    // Remove the workdays that are outside of the employee's contract.
    let workdays: Arc<[DayOfWeek]> = if year < employee_work_details.from_year
        || year > employee_work_details.to_year
        || (year == employee_work_details.from_year && week < employee_work_details.from_calendar_week)
        || (year == employee_work_details.to_year && week > employee_work_details.to_calendar_week)
    {
        Arc::new([])
    } else if employee_work_details.from_year == employee_work_details.to_year
        && employee_work_details.from_calendar_week == employee_work_details.to_calendar_week 
    {
        workdays
            .iter()
            .map(|workday| DayOfWeek::from(*workday))
            .filter(|workday| *workday >= employee_work_details.from_day_of_week && *workday <= employee_work_details.to_day_of_week)
            .collect()
    } else if year == employee_work_details.from_year
        && week == employee_work_details.from_calendar_week
    {
        workdays
            .iter()
            .map(|workday| DayOfWeek::from(*workday))
            .filter(|workday| *workday >= employee_work_details.from_day_of_week)
            .collect()
    } else if year == employee_work_details.to_year
        && week == employee_work_details.to_calendar_week
    {
        workdays
            .iter()
            .map(|workday| DayOfWeek::from(*workday))
            .filter(|workday| *workday <= employee_work_details.to_day_of_week)
            .collect()
    } else {
        workdays
            .iter()
            .map(|workday| DayOfWeek::from(*workday))
            .collect()
    };

    let num_potential_workdays_in_week = workdays.iter().count();
    let relation = num_potential_workdays_in_week as f32 / all_potential_workdays as f32;
    (
        if employee_work_details.is_dynamic { 0.0 } else {
            employee_work_details.expected_hours * relation
        },
        employee_work_details.expected_hours * relation,
        num_potential_workdays_in_week as u8,
        employee_work_details.workdays_per_week as f32 * relation,
    )
}

fn hours_per_week(
    shiftplan_hours_list: &Arc<[ShiftplanReportDay]>,
    extra_hours_list: &Arc<[ExtraHours]>,
    working_hours: &[EmployeeWorkDetails],
    derived_absence: &std::collections::BTreeMap<time::Date, service::absence::ResolvedAbsence>,
    derived_holiday: &std::collections::HashMap<time::Date, f32>,
    from_date: ShiftyDate,
    to_date: ShiftyDate,
) -> Result<Arc<[GroupedReportHours]>, ServiceError> {
    let from_week = from_date.as_shifty_week();
    let to_week = to_date.as_shifty_week();

    let mut weeks: Vec<GroupedReportHours> = Vec::new();
    for week in from_week.iter_until(&to_week) {
        tracing::info!("Week: {}, Year: {}", week.week, week.year);
        let filtered_extra_hours_list = extra_hours_list
            .iter()
            .filter(|eh| eh.to_date().as_shifty_week() == week)
            .collect::<Vec<_>>();
        let filtered_shiftplan_hours_list = shiftplan_hours_list
            .iter()
            .filter(|r| {
                if let Ok(date) = r.to_date() {
                    date.as_shifty_week() == week
                } else {
                    false
                }
            })
            .inspect(|r: &&ShiftplanReportDay| {
                tracing::info!("{:?} - {:?}", r.to_date(), r);
            })
            .collect::<Vec<_>>();
        let raw_shiftplan_hours = filtered_shiftplan_hours_list
            .iter()
            .map(|r: &&ShiftplanReportDay| r.hours)
            .sum::<f32>();
        let (working_hours_for_week, dynamic_working_hours_for_week, days_per_week, workdays_per_week) =
            find_working_hours_for_calendar_week(working_hours, week.year, week.week)
                .map(|wh| weight_for_week(week.year, week.week,
                    &wh.with_to_date(
                        wh.to_date()
                            .unwrap_or(to_date)
                            .min(to_date)
                        ).with_from_date(
                            wh.from_date()
                                .unwrap_or(from_date)
                                .max(from_date)
                        )
                    ))
                .fold(
                    (0.0f32, 0.0f32, 0u8, 0f32),
                    |(working_hours_acc, dynamic_working_hours_acc, days_per_week_acc, workdays_per_week_acc),
                     (wh, dwh, dpw, wpw)| {
                        (
                            working_hours_acc + wh,
                            dynamic_working_hours_acc + dwh,
                            days_per_week_acc + dpw,
                            workdays_per_week_acc + wpw,
                        )
                    },
                );
        let cap_active = find_working_hours_for_calendar_week(working_hours, week.year, week.week)
            .any(|wh| wh.cap_planned_hours_to_expected);
        // User-Regel (quick-260624-ujk): Eine KW OHNE EmployeeWorkDetails-Zeile bedeutet,
        // dass der Mitarbeiter in dieser Woche KEINEN Vertrag hat. Geleistete Shiftplan-Stunden
        // sind dann Ehrenamt (volunteer), kein bezahltes Soll=Ist. Das unterscheidet sich vom
        // dynamischen Vertrag (Zeile vorhanden, expected=0): dort gilt weiterhin Soll=Ist.
        //
        // Abgrenzung booking_information-Band-Logik: Die booking_information-Baender
        // (committed_voluntary Band 1, volunteer_surplus Band 2) sind auf is_paid=false
        // (unbezahlte Freiwillige) gegated. Dieser Pfad betrifft bezahlte Mitarbeiter
        // ohne Vertragszeile. Beide Pfade sind disjunkt — keine Doppelzaehlung.
        let has_contract_row =
            find_working_hours_for_calendar_week(working_hours, week.year, week.week)
                .next()
                .is_some();
        let (shiftplan_hours, auto_volunteer_hours) =
            apply_weekly_cap(cap_active, raw_shiftplan_hours, working_hours_for_week);
        // no-contract: shiftplan-Stunden gehen NICHT in overall, sondern als Ehrenamt.
        // dynamic (has_contract_row && working_hours_for_week == 0): shiftplan_paid bleibt shiftplan_hours (Soll=Ist).
        let shiftplan_paid = if has_contract_row { shiftplan_hours } else { 0.0 };
        let no_contract_volunteer = if has_contract_row { 0.0 } else { shiftplan_hours };
        let extra_work_hours = filtered_extra_hours_list
            .iter()
            .filter(|eh| eh.category.as_report_type() == ReportType::WorkingHours)
            .map(|eh| eh.amount)
            .sum::<f32>();
        let absence_hours = if working_hours_for_week <= 0.0 {
            0.0f32
        } else {
            filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category.as_report_type() == ReportType::AbsenceHours)
                .map(|eh| eh.amount)
                .sum::<f32>()
        };
        // Gap 2 (Phase 8.4 / WR-01): absence_period-derived Stunden dieser Woche summieren.
        // Alle drei derived-Kategorien (V/S/U) sind AbsenceHours -> reduzieren expected symmetrisch.
        // Nur wenn working_hours_for_week > 0 (gleiche Bedingung wie absence_hours oben).
        let derived_absence_hours = if working_hours_for_week <= 0.0 {
            0.0f32
        } else {
            derived_absence
                .iter()
                .filter(|(d, _)| ShiftyDate::from(**d).as_shifty_week() == week)
                .map(|(_, r)| r.hours)
                .sum::<f32>()
        };
        let absence_hours = absence_hours + derived_absence_hours;

        // Phase 25 (HOL-01/02 injection point 1a): add derived holiday hours
        // for this ISO week. Holiday is AbsenceHours-typed, so derived hours
        // must be added to BOTH holiday_hours AND absence_hours to correctly
        // reduce expected_hours/balance (Pitfall 3). Gated by working_hours_for_week
        // like absence_hours above (no credit in contract-less/dynamic weeks).
        let derived_holiday_for_week: f32 = if working_hours_for_week <= 0.0 {
            0.0
        } else {
            derived_holiday
                .iter()
                .filter(|(date, _)| ShiftyDate::from(**date).as_shifty_week() == week)
                .map(|(_, h)| h)
                .sum()
        };
        let absence_hours = absence_hours + derived_holiday_for_week;

        let mut day_list = filtered_extra_hours_list
            .iter()
            .map(|eh| {
                Ok(WorkingHoursDay {
                    date: eh.date_time.date(),
                    hours: eh.amount,
                    category: (&eh.category).into(),
                })
            })
            .chain(
                filtered_shiftplan_hours_list
                    .iter()
                    .map(|working_hours_day| {
                        Ok::<WorkingHoursDay, ServiceError>(WorkingHoursDay {
                            date: time::Date::from_iso_week_date(
                                week.year as i32,
                                working_hours_day.calendar_week,
                                time::Weekday::Sunday
                                    .nth_next(working_hours_day.day_of_week.to_number()),
                            )?,
                            hours: working_hours_day.hours,
                            category: ExtraHoursReportCategory::Shiftplan,
                        })
                    }),
            )
            .collect::<Result<Vec<WorkingHoursDay>, ServiceError>>()?;
        day_list.sort_by_key(|day| day.date);
        // Drei Faelle (quick-260624-ujk):
        // 1. !has_contract_row: Ehrenamt-Pfad — expected=0, shiftplan NICHT in overall.
        // 2. has_contract_row && working_hours_for_week == 0 (dynamisch): Soll=Ist — expected = shiftplan + extra.
        // 3. has_contract_row && working_hours_for_week > 0: Normal — expected = Vertragsstunden.
        let expected_hours = if !has_contract_row {
            0.0
        } else if working_hours_for_week == 0.0 {
            shiftplan_hours + extra_work_hours
        } else {
            working_hours_for_week
        };

        let custom_extra_hours: Arc<[service::reporting::CustomExtraHours]> = {
            let mut map: HashMap<(Uuid, Arc<str>), f32> = HashMap::new();
            for eh_entry in filtered_extra_hours_list.iter() {
                if let ExtraHoursCategory::CustomExtraHours(lazy_load_custom_def) =
                    &eh_entry.category
                {
                    if let Some(custom_def) = lazy_load_custom_def.get() {
                        let key = (custom_def.id, custom_def.name.clone());
                        *map.entry(key).or_insert(0.0) += eh_entry.amount;
                    }
                }
            }
            map.into_iter()
                .map(|((id, name), hours)| service::reporting::CustomExtraHours { id, name, hours })
                .collect::<Vec<_>>()
                .into()
        };

        // UV-05 / D-18-03: per-week derived absence hours split by category, UNGATED.
        // These feed the DISPLAY/DAYS fields (vacation_days/sick_leave_days/absence_days),
        // independent of working_hours_for_week — so contract-less/dynamic weeks still show
        // their converted absence days. This is SEPARATE from `derived_absence_hours` (line
        // ~1139) which is gated and only reduces expected/balance (D-18-05: do NOT change that).
        let (derived_vacation_hours, derived_sick_leave_hours, derived_unpaid_leave_hours) =
            derived_absence
                .iter()
                .filter(|(d, _)| ShiftyDate::from(**d).as_shifty_week() == week)
                .fold((0.0f32, 0.0f32, 0.0f32), |(v, s, u), (_, r)| {
                    match r.category {
                        service::absence::AbsenceCategory::Vacation => (v + r.hours, s, u),
                        service::absence::AbsenceCategory::SickLeave => (v, s + r.hours, u),
                        service::absence::AbsenceCategory::UnpaidLeave => (v, s, u + r.hours),
                    }
                });

        weeks.push(GroupedReportHours {
            from: week.as_date(DayOfWeek::Monday).max(from_date),
            to: week.as_date(DayOfWeek::Sunday).min(to_date),
            year: week.year,
            week: week.week,
            contract_weekly_hours: dynamic_working_hours_for_week,
            expected_hours: expected_hours - absence_hours,
            dynamic_hours: dynamic_working_hours_for_week - absence_hours,
            // no-contract (quick-260624-ujk): shiftplan_paid=0 fuer vertraglose Wochen;
            // ExtraWork bleibt in overall (explizit erfasste bezahlte Leistung).
            overall_hours: shiftplan_paid + extra_work_hours,
            balance: shiftplan_paid + extra_work_hours - expected_hours + absence_hours,
            shiftplan_hours: shiftplan_paid,
            days_per_week,
            workdays_per_week,
            extra_work_hours: filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category == ExtraHoursCategory::ExtraWork)
                .map(|eh| eh.amount)
                .sum(),
            vacation_hours: filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category == ExtraHoursCategory::Vacation)
                .map(|eh| eh.amount)
                .sum::<f32>()
                + derived_vacation_hours,
            sick_leave_hours: filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category == ExtraHoursCategory::SickLeave)
                .map(|eh| eh.amount)
                .sum::<f32>()
                + derived_sick_leave_hours,
            // Phase 25 (injection point 1a): manual Holiday hours + derived-for-week.
            // The derived amount was already added to absence_hours above (Pitfall 3).
            holiday_hours: filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
                .map(|eh| eh.amount)
                .sum::<f32>()
                + derived_holiday_for_week,
            unpaid_leave_hours: filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category == ExtraHoursCategory::UnpaidLeave)
                .map(|eh| eh.amount)
                .sum::<f32>()
                + derived_unpaid_leave_hours,
            // no-contract (quick-260624-ujk): no_contract_volunteer traegt die Shiftplan-Stunden
            // vertragloser Wochen als Ehrenamt bei (+ manuelle VolunteerWork + auto_volunteer vom Cap).
            volunteer_hours: filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category == ExtraHoursCategory::VolunteerWork)
                .map(|eh| eh.amount)
                .sum::<f32>()
                + auto_volunteer_hours
                + no_contract_volunteer,
            custom_extra_hours,
            days: day_list.iter().cloned().collect(),
        });
    }
    Ok(weeks.into())
}

#[cfg(test)]
mod test_dynamic_vacation_days {
    use super::*;
    use shifty_utils::DayOfWeek;
    use time::macros::datetime;
    use uuid::Uuid;

    fn create_work_details(is_dynamic: bool) -> EmployeeWorkDetails {
        EmployeeWorkDetails {
            id: Uuid::new_v4(),
            sales_person_id: Uuid::new_v4(),
            expected_hours: 40.0,
            from_day_of_week: DayOfWeek::Monday,
            from_calendar_week: 1,
            from_year: 2024,
            to_day_of_week: DayOfWeek::Sunday,
            to_calendar_week: 52,
            to_year: 2024,
            workdays_per_week: 5,
            is_dynamic,
            cap_planned_hours_to_expected: false,
            committed_voluntary: 0.0,
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
            sunday: false,
            vacation_days: 30,
            created: Some(datetime!(2024-01-01 10:00:00)),
            deleted: None,
            version: Uuid::new_v4(),
        }
    }

    fn create_vacation_extra_hours(date: time::PrimitiveDateTime, amount: f32) -> ExtraHours {
        ExtraHours {
            id: Uuid::new_v4(),
            sales_person_id: Uuid::new_v4(),
            amount,
            category: ExtraHoursCategory::Vacation,
            description: "Vacation".into(),
            date_time: date,
            created: Some(datetime!(2024-01-01 10:00:00)),
            deleted: None,
            version: Uuid::new_v4(),
            source: service::extra_hours::ExtraHoursSource::Manual,
        }
    }

    fn create_shiftplan_day(year: u32, week: u8, day: DayOfWeek, hours: f32) -> ShiftplanReportDay {
        ShiftplanReportDay {
            sales_person_id: Uuid::new_v4(),
            hours,
            year,
            calendar_week: week,
            day_of_week: day,
        }
    }

    /// Dynamic employee takes a full week of vacation (40h).
    /// Expected: 5 vacation days (40h / (40h/5days) = 5).
    #[test]
    fn test_dynamic_employee_full_week_vacation() {
        let work_details = create_work_details(true);
        // Week 10 of 2024: Mon=March 4, Sun=March 10
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();

        let extra_hours: Arc<[ExtraHours]> = Arc::new([
            create_vacation_extra_hours(datetime!(2024-03-04 08:00:00), 8.0),
            create_vacation_extra_hours(datetime!(2024-03-05 08:00:00), 8.0),
            create_vacation_extra_hours(datetime!(2024-03-06 08:00:00), 8.0),
            create_vacation_extra_hours(datetime!(2024-03-07 08:00:00), 8.0),
            create_vacation_extra_hours(datetime!(2024-03-08 08:00:00), 8.0),
        ]);

        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([]);

        let result = hours_per_week(&shiftplan, &extra_hours, &[work_details], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        assert_eq!(result.len(), 1);

        let week = &result[0];
        assert_eq!(week.vacation_hours, 40.0);
        assert!(
            week.vacation_days() > 0.0,
            "Dynamic employee should have vacation_days > 0, got {}",
            week.vacation_days()
        );
        assert!(
            (week.vacation_days() - 5.0).abs() < 0.01,
            "Expected 5.0 vacation days, got {}",
            week.vacation_days()
        );
    }

    /// Dynamic employee takes partial vacation (8h) and works 32h.
    /// Expected: 1 vacation day (8h / (40h/5days) = 1).
    #[test]
    fn test_dynamic_employee_partial_vacation() {
        let work_details = create_work_details(true);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();

        let extra_hours: Arc<[ExtraHours]> = Arc::new([
            create_vacation_extra_hours(datetime!(2024-03-04 08:00:00), 8.0),
        ]);

        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([
            create_shiftplan_day(2024, 10, DayOfWeek::Tuesday, 8.0),
            create_shiftplan_day(2024, 10, DayOfWeek::Wednesday, 8.0),
            create_shiftplan_day(2024, 10, DayOfWeek::Thursday, 8.0),
            create_shiftplan_day(2024, 10, DayOfWeek::Friday, 8.0),
        ]);

        let result = hours_per_week(&shiftplan, &extra_hours, &[work_details], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        assert_eq!(result.len(), 1);

        let week = &result[0];
        assert_eq!(week.vacation_hours, 8.0);
        assert!(
            (week.vacation_days() - 1.0).abs() < 0.01,
            "Expected 1.0 vacation day, got {}",
            week.vacation_days()
        );
    }

    /// Dynamic employee balance should still be forced to 0.
    #[test]
    fn test_dynamic_employee_balance_zero() {
        let work_details = create_work_details(true);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();

        let extra_hours: Arc<[ExtraHours]> = Arc::new([]);

        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([
            create_shiftplan_day(2024, 10, DayOfWeek::Monday, 8.0),
            create_shiftplan_day(2024, 10, DayOfWeek::Tuesday, 8.0),
            create_shiftplan_day(2024, 10, DayOfWeek::Wednesday, 8.0),
        ]);

        let result = hours_per_week(&shiftplan, &extra_hours, &[work_details], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        assert_eq!(result.len(), 1);

        let week = &result[0];
        // For dynamic employees, balance = shiftplan + extra_work - expected + absence
        // where expected = shiftplan + extra_work (since working_hours_for_week == 0)
        // so balance = 24 + 0 - 24 + 0 = 0
        assert!(
            week.balance.abs() < 0.01,
            "Dynamic employee balance should be ~0, got {}",
            week.balance
        );
    }

    /// Non-dynamic employee vacation days should work as before.
    #[test]
    fn test_non_dynamic_employee_vacation_unchanged() {
        let work_details = create_work_details(false);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();

        let extra_hours: Arc<[ExtraHours]> = Arc::new([
            create_vacation_extra_hours(datetime!(2024-03-04 08:00:00), 8.0),
            create_vacation_extra_hours(datetime!(2024-03-05 08:00:00), 8.0),
        ]);

        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([
            create_shiftplan_day(2024, 10, DayOfWeek::Wednesday, 8.0),
            create_shiftplan_day(2024, 10, DayOfWeek::Thursday, 8.0),
            create_shiftplan_day(2024, 10, DayOfWeek::Friday, 8.0),
        ]);

        let result = hours_per_week(&shiftplan, &extra_hours, &[work_details], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        assert_eq!(result.len(), 1);

        let week = &result[0];
        assert_eq!(week.vacation_hours, 16.0);
        assert!(
            (week.vacation_days() - 2.0).abs() < 0.01,
            "Expected 2.0 vacation days for non-dynamic, got {}",
            week.vacation_days()
        );
    }

    fn create_unpaid_leave_extra_hours(date: time::PrimitiveDateTime, amount: f32) -> ExtraHours {
        ExtraHours {
            id: Uuid::new_v4(),
            sales_person_id: Uuid::new_v4(),
            amount,
            category: ExtraHoursCategory::UnpaidLeave,
            description: "Unpaid leave".into(),
            date_time: date,
            created: Some(datetime!(2024-01-01 10:00:00)),
            deleted: None,
            version: Uuid::new_v4(),
            source: service::extra_hours::ExtraHoursSource::Manual,
        }
    }

    #[test]
    fn test_unpaid_leave_tracked_separately() {
        let work_details = create_work_details(false);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();

        let extra_hours: Arc<[ExtraHours]> = Arc::new([
            create_vacation_extra_hours(datetime!(2024-03-04 08:00:00), 8.0),
            create_unpaid_leave_extra_hours(datetime!(2024-03-05 08:00:00), 8.0),
        ]);

        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([]);

        let result = hours_per_week(&shiftplan, &extra_hours, &[work_details], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        let week = &result[0];
        assert_eq!(week.vacation_hours, 8.0);
        assert_eq!(week.unpaid_leave_hours, 8.0);
    }

    #[test]
    fn test_unpaid_leave_does_not_affect_vacation_days() {
        let work_details = create_work_details(false);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();

        let extra_hours: Arc<[ExtraHours]> = Arc::new([
            create_vacation_extra_hours(datetime!(2024-03-04 08:00:00), 24.0),
            create_unpaid_leave_extra_hours(datetime!(2024-03-05 08:00:00), 16.0),
        ]);

        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([]);

        let result = hours_per_week(&shiftplan, &extra_hours, &[work_details], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        let week = &result[0];
        // Vacation days should only consider vacation hours (24h / 8h per day = 3 days)
        assert!(
            (week.vacation_days() - 3.0).abs() < 0.01,
            "Expected 3.0 vacation days, got {}",
            week.vacation_days()
        );
    }

    #[test]
    fn test_unpaid_leave_included_in_absence_days() {
        let work_details = create_work_details(false);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();

        let extra_hours: Arc<[ExtraHours]> = Arc::new([
            create_vacation_extra_hours(datetime!(2024-03-04 08:00:00), 8.0),
            create_unpaid_leave_extra_hours(datetime!(2024-03-05 08:00:00), 8.0),
        ]);

        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([]);

        let result = hours_per_week(&shiftplan, &extra_hours, &[work_details], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        let week = &result[0];
        // absence_days = (vacation 8 + sick 0 + holiday 0 + unpaid_leave 8) / 8 hours_per_day = 2
        assert!(
            (week.absence_days() - 2.0).abs() < 0.01,
            "Expected 2.0 absence days, got {}",
            week.absence_days()
        );
    }

    #[test]
    fn test_unpaid_leave_reduces_expected_hours() {
        let work_details = create_work_details(false);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();

        let extra_hours: Arc<[ExtraHours]> = Arc::new([
            create_unpaid_leave_extra_hours(datetime!(2024-03-04 08:00:00), 8.0),
        ]);

        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([
            create_shiftplan_day(2024, 10, DayOfWeek::Tuesday, 8.0),
            create_shiftplan_day(2024, 10, DayOfWeek::Wednesday, 8.0),
            create_shiftplan_day(2024, 10, DayOfWeek::Thursday, 8.0),
            create_shiftplan_day(2024, 10, DayOfWeek::Friday, 8.0),
        ]);

        let result = hours_per_week(&shiftplan, &extra_hours, &[work_details], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        let week = &result[0];
        // Expected hours: 40 (contract) - 8 (unpaid leave absence) = 32
        assert!(
            (week.expected_hours - 32.0).abs() < 0.01,
            "Expected 32.0 expected hours, got {}",
            week.expected_hours
        );
        // Balance should be neutral: overall(32) - expected(32) = 0
        assert!(
            week.balance.abs() < 0.01,
            "Expected ~0 balance, got {}",
            week.balance
        );
    }
}

#[cfg(test)]
mod test_weekly_planned_hours_cap {
    use super::*;
    use service::extra_hours::{Availability, ExtraHoursCategory, ReportType};
    use shifty_utils::DayOfWeek;
    use time::macros::datetime;
    use uuid::Uuid;

    fn make_work_details(
        expected_hours: f32,
        cap: bool,
        from_year: u32,
        from_week: u8,
        to_year: u32,
        to_week: u8,
    ) -> EmployeeWorkDetails {
        EmployeeWorkDetails {
            id: Uuid::new_v4(),
            sales_person_id: Uuid::new_v4(),
            expected_hours,
            from_day_of_week: DayOfWeek::Monday,
            from_calendar_week: from_week,
            from_year,
            to_day_of_week: DayOfWeek::Sunday,
            to_calendar_week: to_week,
            to_year,
            workdays_per_week: 5,
            is_dynamic: false,
            cap_planned_hours_to_expected: cap,
            committed_voluntary: 0.0,
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
            sunday: false,
            vacation_days: 0,
            created: Some(datetime!(2024-01-01 10:00:00)),
            deleted: None,
            version: Uuid::new_v4(),
        }
    }

    fn make_extra_hours(
        date: time::PrimitiveDateTime,
        amount: f32,
        category: ExtraHoursCategory,
    ) -> ExtraHours {
        ExtraHours {
            id: Uuid::new_v4(),
            sales_person_id: Uuid::new_v4(),
            amount,
            category,
            description: "".into(),
            date_time: date,
            created: Some(datetime!(2024-01-01 10:00:00)),
            deleted: None,
            version: Uuid::new_v4(),
            source: service::extra_hours::ExtraHoursSource::Manual,
        }
    }

    fn make_shiftplan_day(year: u32, week: u8, day: DayOfWeek, hours: f32) -> ShiftplanReportDay {
        ShiftplanReportDay {
            sales_person_id: Uuid::new_v4(),
            hours,
            year,
            calendar_week: week,
            day_of_week: day,
        }
    }

    // --- volunteer-work-hours capability ---

    #[test]
    fn volunteer_work_maps_to_documented_report_type() {
        assert_eq!(
            ExtraHoursCategory::VolunteerWork.as_report_type(),
            ReportType::Documented
        );
    }

    #[test]
    fn volunteer_work_marks_person_available() {
        assert_eq!(
            ExtraHoursCategory::VolunteerWork.availability(),
            Availability::Available
        );
    }

    #[test]
    fn volunteer_extra_hours_excluded_from_balance_and_overall() {
        // Spec volunteer-work-hours Req 2 Scenario 2: 40h shiftplan + 5h volunteer
        // -> balance 0, overall 40, expected 40, volunteer 5
        let wd = make_work_details(40.0, false, 2024, 1, 2024, 52);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();
        let extra: Arc<[ExtraHours]> = Arc::new([make_extra_hours(
            datetime!(2024-03-05 08:00:00),
            5.0,
            ExtraHoursCategory::VolunteerWork,
        )]);
        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([
            make_shiftplan_day(2024, 10, DayOfWeek::Monday, 8.0),
            make_shiftplan_day(2024, 10, DayOfWeek::Tuesday, 8.0),
            make_shiftplan_day(2024, 10, DayOfWeek::Wednesday, 8.0),
            make_shiftplan_day(2024, 10, DayOfWeek::Thursday, 8.0),
            make_shiftplan_day(2024, 10, DayOfWeek::Friday, 8.0),
        ]);

        let result = hours_per_week(&shiftplan, &extra, &[wd], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        let week = &result[0];
        assert!((week.balance - 0.0).abs() < 0.01, "balance was {}", week.balance);
        assert!((week.overall_hours - 40.0).abs() < 0.01, "overall was {}", week.overall_hours);
        assert!((week.expected_hours - 40.0).abs() < 0.01, "expected was {}", week.expected_hours);
        assert!((week.volunteer_hours - 5.0).abs() < 0.01, "volunteer was {}", week.volunteer_hours);
    }

    // --- weekly-planned-hours-cap capability ---

    #[test]
    fn cap_overflow_attributed_to_volunteer() {
        // Spec Req 2 Scenario 1: cap=true, expected=5, 10h bookings
        // -> shiftplan 5, volunteer 5, balance 0
        let wd = make_work_details(5.0, true, 2024, 1, 2024, 52);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();
        let extra: Arc<[ExtraHours]> = Arc::new([]);
        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([
            make_shiftplan_day(2024, 10, DayOfWeek::Monday, 5.0),
            make_shiftplan_day(2024, 10, DayOfWeek::Tuesday, 5.0),
        ]);

        let result = hours_per_week(&shiftplan, &extra, &[wd], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        let week = &result[0];
        assert!((week.shiftplan_hours - 5.0).abs() < 0.01, "shiftplan was {}", week.shiftplan_hours);
        assert!((week.volunteer_hours - 5.0).abs() < 0.01, "volunteer was {}", week.volunteer_hours);
        assert!((week.balance - 0.0).abs() < 0.01, "balance was {}", week.balance);
    }

    #[test]
    fn cap_combined_with_manual_volunteer() {
        // Spec Req 2 Scenario 2: cap=true, expected=5, 10h bookings + 2h manual volunteer
        // -> volunteer 7, balance 0
        let wd = make_work_details(5.0, true, 2024, 1, 2024, 52);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();
        let extra: Arc<[ExtraHours]> = Arc::new([make_extra_hours(
            datetime!(2024-03-05 08:00:00),
            2.0,
            ExtraHoursCategory::VolunteerWork,
        )]);
        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([
            make_shiftplan_day(2024, 10, DayOfWeek::Monday, 5.0),
            make_shiftplan_day(2024, 10, DayOfWeek::Tuesday, 5.0),
        ]);

        let result = hours_per_week(&shiftplan, &extra, &[wd], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        let week = &result[0];
        assert!((week.volunteer_hours - 7.0).abs() < 0.01, "volunteer was {}", week.volunteer_hours);
        assert!((week.balance - 0.0).abs() < 0.01, "balance was {}", week.balance);
    }

    #[test]
    fn cap_below_expected_yields_negative_balance() {
        // Spec Req 3: cap=true, expected=5, 3h bookings
        // -> shiftplan 3, volunteer 0, balance -2
        let wd = make_work_details(5.0, true, 2024, 1, 2024, 52);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();
        let extra: Arc<[ExtraHours]> = Arc::new([]);
        let shiftplan: Arc<[ShiftplanReportDay]> =
            Arc::new([make_shiftplan_day(2024, 10, DayOfWeek::Monday, 3.0)]);

        let result = hours_per_week(&shiftplan, &extra, &[wd], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        let week = &result[0];
        assert!((week.shiftplan_hours - 3.0).abs() < 0.01, "shiftplan was {}", week.shiftplan_hours);
        assert!(week.volunteer_hours.abs() < 0.01, "volunteer was {}", week.volunteer_hours);
        assert!((week.balance - (-2.0)).abs() < 0.01, "balance was {}", week.balance);
    }

    #[test]
    fn cap_does_not_affect_extra_work() {
        // Spec Req 4: cap=true, expected=5, 5h bookings + 3h ExtraWork
        // -> overall 8, balance +3, volunteer 0
        let wd = make_work_details(5.0, true, 2024, 1, 2024, 52);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();
        let extra: Arc<[ExtraHours]> = Arc::new([make_extra_hours(
            datetime!(2024-03-06 08:00:00),
            3.0,
            ExtraHoursCategory::ExtraWork,
        )]);
        let shiftplan: Arc<[ShiftplanReportDay]> =
            Arc::new([make_shiftplan_day(2024, 10, DayOfWeek::Monday, 5.0)]);

        let result = hours_per_week(&shiftplan, &extra, &[wd], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        let week = &result[0];
        assert!((week.overall_hours - 8.0).abs() < 0.01, "overall was {}", week.overall_hours);
        assert!((week.balance - 3.0).abs() < 0.01, "balance was {}", week.balance);
        assert!(week.volunteer_hours.abs() < 0.01, "volunteer was {}", week.volunteer_hours);
    }

    #[test]
    fn no_cap_preserves_overtime() {
        // Spec Req 5: cap=false, expected=20, 25h bookings
        // -> shiftplan 25, balance +5, volunteer 0
        let wd = make_work_details(20.0, false, 2024, 1, 2024, 52);
        let from = ShiftyDate::new(2024, 10, DayOfWeek::Monday).unwrap();
        let to = ShiftyDate::new(2024, 10, DayOfWeek::Sunday).unwrap();
        let extra: Arc<[ExtraHours]> = Arc::new([]);
        let shiftplan: Arc<[ShiftplanReportDay]> = Arc::new([
            make_shiftplan_day(2024, 10, DayOfWeek::Monday, 5.0),
            make_shiftplan_day(2024, 10, DayOfWeek::Tuesday, 5.0),
            make_shiftplan_day(2024, 10, DayOfWeek::Wednesday, 5.0),
            make_shiftplan_day(2024, 10, DayOfWeek::Thursday, 5.0),
            make_shiftplan_day(2024, 10, DayOfWeek::Friday, 5.0),
        ]);

        let result = hours_per_week(&shiftplan, &extra, &[wd], &std::collections::BTreeMap::new(), &std::collections::HashMap::new(), from, to).unwrap();
        let week = &result[0];
        assert!((week.shiftplan_hours - 25.0).abs() < 0.01, "shiftplan was {}", week.shiftplan_hours);
        assert!((week.balance - 5.0).abs() < 0.01, "balance was {}", week.balance);
        assert!(week.volunteer_hours.abs() < 0.01, "volunteer was {}", week.volunteer_hours);
    }

    #[test]
    fn apply_weekly_cap_helper_inactive_passes_through() {
        let (shift, vol) = apply_weekly_cap(false, 25.0, 20.0);
        assert_eq!(shift, 25.0);
        assert_eq!(vol, 0.0);
    }

    #[test]
    fn apply_weekly_cap_helper_active_caps_overflow() {
        let (shift, vol) = apply_weekly_cap(true, 10.0, 5.0);
        assert_eq!(shift, 5.0);
        assert_eq!(vol, 5.0);
    }

    #[test]
    fn apply_weekly_cap_helper_active_below_expected_no_compensation() {
        let (shift, vol) = apply_weekly_cap(true, 3.0, 5.0);
        assert_eq!(shift, 3.0);
        assert_eq!(vol, 0.0);
    }
}

/// CVC-03 / D-OVERLAP-AGG = SUM: Tests fuer `committed_voluntary_for_calendar_week`.
///
/// Pinnt die SUM-Aggregations-Semantik — zwei ueberlappende aktive Rows in
/// derselben ISO-Woche werden summiert (5.0 + 5.0 -> 10.0), nicht mit `.any()`
/// oder einem anderen Bool-Reduktions-Pattern aggregiert.
#[cfg(test)]
mod test_committed_voluntary_for_calendar_week {
    use super::*;
    use shifty_utils::DayOfWeek;
    use time::macros::datetime;
    use uuid::Uuid;

    fn make_work_details_with_committed(
        committed_voluntary: f32,
        from_year: u32,
        from_week: u8,
        to_year: u32,
        to_week: u8,
    ) -> EmployeeWorkDetails {
        EmployeeWorkDetails {
            id: Uuid::new_v4(),
            sales_person_id: Uuid::new_v4(),
            expected_hours: 8.0,
            from_day_of_week: DayOfWeek::Monday,
            from_calendar_week: from_week,
            from_year,
            to_day_of_week: DayOfWeek::Sunday,
            to_calendar_week: to_week,
            to_year,
            workdays_per_week: 5,
            is_dynamic: false,
            cap_planned_hours_to_expected: false,
            committed_voluntary,
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
            sunday: false,
            vacation_days: 0,
            created: Some(datetime!(2026-01-01 10:00:00)),
            deleted: None,
            version: Uuid::new_v4(),
        }
    }

    /// CVC-03 / D-OVERLAP-AGG = SUM: Zwei ueberlappende aktive Rows in KW 10 /
    /// 2026 mit je committed_voluntary = 5.0 aggregieren zu 10.0.
    /// Pinnt explizit, dass NICHT `.any()` (Bool-Anti-Pattern des Cap-Flags)
    /// sondern `.map().sum()` verwendet wird.
    #[test]
    fn committed_voluntary_sum_two_overlapping_rows_in_same_week() {
        let row_a = make_work_details_with_committed(5.0, 2026, 1, 2026, 52);
        let row_b = make_work_details_with_committed(5.0, 2026, 1, 2026, 52);
        let working_hours = vec![row_a, row_b];

        let result = committed_voluntary_for_calendar_week(&working_hours, 2026, 10);
        assert!(
            (result - 10.0).abs() < f32::EPSILON,
            "5.0 + 5.0 must sum to 10.0 (got {})",
            result
        );
    }

    /// CVC-03 Single-Row: eine aktive Row mit committed_voluntary = 5.0 ergibt 5.0.
    #[test]
    fn committed_voluntary_sum_single_row() {
        let row = make_work_details_with_committed(5.0, 2026, 1, 2026, 52);
        let working_hours = vec![row];

        let result = committed_voluntary_for_calendar_week(&working_hours, 2026, 10);
        assert!(
            (result - 5.0).abs() < f32::EPSILON,
            "single row must yield 5.0 (got {})",
            result
        );
    }

    /// CVC-03 Empty: keine aktive Row in der Woche ergibt 0.0 (leere .sum()).
    #[test]
    fn committed_voluntary_sum_no_active_row_in_week_yields_zero() {
        // Row liegt in KW 1-9, KW 10 ist nicht abgedeckt.
        let row = make_work_details_with_committed(5.0, 2026, 1, 2026, 9);
        let working_hours = vec![row];

        let result = committed_voluntary_for_calendar_week(&working_hours, 2026, 10);
        assert!(
            result.abs() < f32::EPSILON,
            "no active row must yield 0.0 (got {})",
            result
        );
    }

    /// CVC-03 Empty-Slice: leeres Slice ergibt 0.0.
    #[test]
    fn committed_voluntary_sum_empty_slice_yields_zero() {
        let working_hours: Vec<EmployeeWorkDetails> = vec![];
        let result = committed_voluntary_for_calendar_week(&working_hours, 2026, 10);
        assert!(
            result.abs() < f32::EPSILON,
            "empty slice must yield 0.0 (got {})",
            result
        );
    }
}
