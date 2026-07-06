use crate::gen_service_impl;
use crate::reporting::find_working_hours_for_calendar_week;
use std::sync::Arc;

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    absence::AbsenceService,
    booking::BookingService,
    booking_information::{
        build_booking_information, BookingInformation, BookingInformationService, WeeklySummary,
        WorkingHoursPerSalesPerson,
    },
    clock::ClockService,
    employee_work_details::EmployeeWorkDetailsService,
    permission::{Authentication, SALES_PRIVILEGE, SHIFTPLANNER_PRIVILEGE},
    reporting::ReportingService,
    sales_person::SalesPersonService,
    sales_person_unavailable::SalesPersonUnavailableService,
    shiftplan_catalog::ShiftplanService,
    shiftplan_report::ShiftplanReportService,
    slot::{Slot, SlotService},
    special_days::SpecialDayService,
    toggle::ToggleService,
    uuid_service::UuidService,
    PermissionService, ServiceError,
};

use crate::shortday_gate;
use crate::shortday_gate::{ClipOutcome, ShortdayMode};
use shifty_utils::DayOfWeek;
use tokio::join;
use uuid::Uuid;

/// D-05 / CVC-04: Band 2 per-person surplus = max(actual − committed, 0).
/// `committed` is the person's OWN cap-gated committed_voluntary for the week
/// (0.0 for cap=false rows — gated at the call site, CVC-06). `committed = 0`
/// ⇒ returns `actual` unchanged ⇒ Band 2 bit-identical to pre-v1.4.
/// NEVER subtract aggregate-from-aggregate — the max is nonlinear, so this MUST
/// be applied per person before summing (person-set overlap is real, D-05).
pub(crate) fn volunteer_surplus_above_committed(actual: f32, committed: f32) -> f32 {
    (actual - committed).max(0.0)
}

/// Band 2 aggregate (D-05 / CVC-04): sum max(weekly_actual_p − committed_p, 0) PER PERSON.
///
/// The per-person weekly actual MUST be summed across the per-day shiftplan-report rows BEFORE
/// the nonlinear max — `extract_shiftplan_report_for_week` returns one row per (person, day)
/// because the DAO query groups by `sales_person_id, year, day_of_week`. Subtracting committed
/// per-day instead of per-week under-counts the surplus (CR-01 BLOCKER):
///
/// Example: committed=5, Mon 3h + Tue 4h (weekly actual=7).
/// - Correct (per-week): max(7 − 5, 0) = 2.0
/// - Buggy (per-day):    max(3 − 5, 0) + max(4 − 5, 0) = 0 + 0 = 0.0  ← CR-01
///
/// `per_day_actuals`: iterator of `(sales_person_id, hours)` for each per-day report row.
/// `committed_for_person`: closure returning the cap-gated weekly committed for a given person.
pub(crate) fn volunteer_surplus_band2(
    per_day_actuals: impl IntoIterator<Item = (uuid::Uuid, f32)>,
    committed_for_person: impl Fn(uuid::Uuid) -> f32,
) -> f32 {
    use std::collections::HashMap;
    let mut weekly: HashMap<uuid::Uuid, f32> = HashMap::new();
    for (person, hours) in per_day_actuals {
        *weekly.entry(person).or_insert(0.0) += hours;
    }
    weekly
        .into_iter()
        .map(|(person, actual)| {
            volunteer_surplus_above_committed(actual, committed_for_person(person))
        })
        .sum()
}

/// VFA-01 (D-26-01 / D-26-03): Returns `true` iff the absence period `[from, to]` overlaps
/// the calendar week `[week_monday, week_sunday]` (all bounds inclusive, per AbsencePeriod D-05).
///
/// Any overlap of the Mon–Sun calendar week counts as a whole-week-out (D-26-03): the test is
/// purely date-based, not proportional per day. All three absence categories (Vacation, SickLeave,
/// UnpaidLeave) use this helper identically — category is not an input (D-26-01 category-agnostic).
pub(crate) fn period_overlaps_week(
    from: time::Date,
    to: time::Date,
    week_monday: time::Date,
    week_sunday: time::Date,
) -> bool {
    from <= week_sunday && to >= week_monday
}

/// v2.2.1: pure conflict predicate for `get_booking_conflicts_for_week`. A booking
/// is a conflict when EITHER the person is manually marked unavailable on this
/// weekday, OR the booking date falls into an active absence period for the same
/// person. `absence_ranges` is an already-filtered list of `(from, to)` pairs for
/// the person; empty for persons without absences overlapping the week.
pub(crate) fn is_booking_conflict(
    unavailable_weekdays: &[DayOfWeek],
    slot_day_of_week: DayOfWeek,
    booking_date: Option<time::Date>,
    absence_ranges: &[(time::Date, time::Date)],
) -> bool {
    if unavailable_weekdays.contains(&slot_day_of_week) {
        return true;
    }
    let Some(date) = booking_date else {
        return false;
    };
    absence_ranges
        .iter()
        .any(|(from, to)| *from <= date && date <= *to)
}

gen_service_impl! {
    struct BookingInformationServiceImpl: BookingInformationService = BookingInformationServiceDeps {
        ShiftplanReportService: ShiftplanReportService<Transaction = Self::Transaction> = shiftplan_report_service,
        SlotService: SlotService<Transaction = Self::Transaction> = slot_service,
        // Phase 52 (WOP-01, D-52-01): ShiftplanService (catalog, Basic-Tier) für den
        // In-Memory-Filter `shiftplan.is_planning`-Semantik in `get_weekly_summary`.
        // Bulk-Load-Ersatz für den DAO-JOIN gegen `shiftplan` in
        // `SlotDao::get_slots_for_week_all_plans` — planning-Shiftplan-Ids werden
        // einmal pro Endpoint-Abruf geladen und im Slot-Filter angewandt. Kein
        // Zyklus: ShiftplanService konsumiert keine BookingInformationService.
        ShiftplanService: ShiftplanService<Transaction = Self::Transaction> = shiftplan_service,
        BookingService: BookingService<Transaction = Self::Transaction> = booking_service,
        SalesPersonService: SalesPersonService<Transaction = Self::Transaction> = sales_person_service,
        SalesPersonUnavailableService: SalesPersonUnavailableService<Transaction = Self::Transaction> = sales_person_unavailable_service,
        ReportingService: ReportingService<Transaction = Self::Transaction> = reporting_service,
        SpecialDayService: SpecialDayService = special_day_service,
        // Phase 51 (D-51-06 Chain C + D-51-07): Stichtag-Toggle für pro-Slot-Clip
        // in `get_weekly_summary` / `get_summery_for_week`. Basic-Tier-Dep — bleibt
        // zyklen-frei (ToggleService konsumiert keine Business-Logic-Services).
        ToggleService: ToggleService<Context = Self::Context, Transaction = Self::Transaction> = toggle_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Transaction = Self::Transaction> = employee_work_details_service,
        // VFA-01 (D-26-01/D-26-03): AbsenceService provides volunteer absences for the year-view.
        // BookingInformationService (business-logic tier) → AbsenceService (business-logic tier):
        // no DI cycle because AbsenceService does NOT consume BookingInformationService
        // (Service-Tier rule; D-Phase3-18 regression-lock preserved).
        AbsenceService: AbsenceService<Context = Self::Context, Transaction = Self::Transaction> = absence_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: BookingInformationServiceDeps> BookingInformationService
    for BookingInformationServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_booking_conflicts_for_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[BookingInformation]>, ServiceError> {
        use shifty_utils::{DateRange, ShiftyDate};
        use std::collections::HashMap;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;
        let bookings = self
            .booking_service
            .get_for_week(week, year, Authentication::Full, tx.clone().into())
            .await?;
        let sales_persons = self
            .sales_person_service
            .get_all(Authentication::Full, tx.clone().into())
            .await?;
        let slots = self
            .slot_service
            .get_slots(Authentication::Full, tx.clone().into())
            .await?;
        let unavailable_entries = self
            .sales_person_unavailable_service
            .get_by_week(year, week, Authentication::Full, tx.clone().into())
            .await?;
        let booking_informations = build_booking_information(slots, bookings, sales_persons);

        // v2.2.1: additionally flag bookings that fall into any active absence
        // period for the same sales person. Fetches per-person absences that
        // overlap the [Mon..Sun] range of this ISO week once, then filters
        // in-memory per booking date.
        let week_from = ShiftyDate::new(year, week, DayOfWeek::Monday).ok();
        let week_to = ShiftyDate::new(year, week, DayOfWeek::Sunday).ok();
        let week_range = match (week_from, week_to) {
            (Some(from), Some(to)) => DateRange::new(from.to_date(), to.to_date()).ok(),
            _ => None,
        };

        let mut absences_by_person: HashMap<Uuid, Arc<[service::absence::AbsencePeriod]>> =
            HashMap::new();
        if let Some(range) = week_range {
            // Collect unique sales-person ids that actually have bookings.
            let mut seen = std::collections::HashSet::<Uuid>::new();
            let unique_ids: Vec<Uuid> = booking_informations
                .iter()
                .filter_map(|bi| {
                    if seen.insert(bi.sales_person.id) {
                        Some(bi.sales_person.id)
                    } else {
                        None
                    }
                })
                .collect();

            for sp_id in unique_ids {
                let overlapping = self
                    .absence_service
                    .find_overlapping_for_booking(
                        sp_id,
                        range,
                        Authentication::Full,
                        tx.clone().into(),
                    )
                    .await?;
                absences_by_person.insert(sp_id, overlapping);
            }
        }

        let conflicts = booking_informations
            .iter()
            .filter(|booking_information| {
                let unavailable_weekdays: Vec<DayOfWeek> = unavailable_entries
                    .iter()
                    .filter(|u| u.sales_person_id == booking_information.sales_person.id)
                    .map(|u| u.day_of_week)
                    .collect();

                // v2.2.1: compute the booking's exact calendar date; used both to check
                // absence-period membership and to skip on bad week/day encoding.
                let cw_u8: Result<u8, _> =
                    booking_information.booking.calendar_week.try_into();
                let booking_date = cw_u8.ok().and_then(|cw| {
                    ShiftyDate::new(
                        booking_information.booking.year,
                        cw,
                        booking_information.slot.day_of_week,
                    )
                    .ok()
                    .map(|d| d.to_date())
                });

                let absence_ranges: Vec<(time::Date, time::Date)> = absences_by_person
                    .get(&booking_information.sales_person.id)
                    .map(|periods| periods.iter().map(|p| (p.from_date, p.to_date)).collect())
                    .unwrap_or_default();

                is_booking_conflict(
                    &unavailable_weekdays,
                    booking_information.slot.day_of_week,
                    booking_date,
                    &absence_ranges,
                )
            })
            .cloned()
            .collect();

        self.transaction_dao.commit(tx).await?;
        Ok(conflicts)
    }

    async fn get_weekly_summary(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[WeeklySummary]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (shiftplanner, sales) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context.clone())
        );
        shiftplanner.or(sales)?;

        let is_shiftplanner = self
            .permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
            .await
            .is_ok();

        let mut weekly_report = vec![];
        let weeks_in_year = time::util::weeks_in_year(year as i32);
        // VAA-01 (D-53-01/05, Pitfall 1): einmal laden, zweimal nutzen. `all_sales_persons`
        // wird im Per-Woche-Loop fuer den Namens-Lookup der Freiwilligen-Absencen
        // wiederverwendet — kein zweiter `get_all`-Aufruf.
        let all_sales_persons = self
            .sales_person_service
            .get_all(Authentication::Full, tx.clone().into())
            .await?;
        let volunteer_ids: Arc<[Uuid]> = all_sales_persons
            .iter()
            .filter(|sales_person| !sales_person.is_paid.unwrap_or(false))
            .map(|sales_person| sales_person.id)
            .collect();
        // Pitfall 4: load work-details ONCE before the per-week loop (not N times)
        let all_work_details = self
            .employee_work_details_service
            .all(Authentication::Full, tx.clone().into())
            .await?;
        // VFA-01 (D-26-01): load all absence periods once before the week loop (load-once pattern,
        // mirrors all_work_details optimisation above). Authentication::Full matches every sibling
        // internal load in this method. find_all is category-agnostic (Vacation + SickLeave +
        // UnpaidLeave all included). We pre-filter to volunteer sales_person_ids to minimise
        // per-week iteration work.
        let all_absences = self
            .absence_service
            .find_all(Authentication::Full, tx.clone().into())
            .await?;
        // Phase 51 (D-51-06 Chain C + D-51-07): Stichtag-Toggle-Wert einmal pro
        // Method-Call holen (nicht pro Woche) — er hängt weder an `year` noch
        // an `week`. `Unauthorized` → None (Legacy off), analog reporting.rs
        // und Chain A' (block.rs).
        // Gap-Closure: zentraler Helper mit `Unauthorized → None` (Legacy off).
        // D-52-09 / R8: MUST-preserve. Toggle-Read bleibt HIER in
        // `get_weekly_summary`, NIE in `assemble_weeks`/`get_year`.
        let active_from =
            shortday_gate::read_active_from(self.toggle_service.as_ref(), context.clone()).await?;

        // ─── Phase 52 (WOP-01/WOP-02, D-52-01/D-52-04/D-52-06 + Follow-up #3) — Bulk-Load-Präambel ───
        //
        // Ersetzt ~55×3 sequenzielle DAO-Roundtrips durch **konstant viele**
        // Bulk-Loads: 2× `get_year` (year + year+1 Spillover), 2×
        // `special_day.get_by_iso_year`, 2× `extract_shiftplan_report_for_iso_year`,
        // 1× `slot_service.get_slots` (jahresagnostisch — In-Memory-Filter
        // im Loop), 1× `shiftplan_service.get_all` (für `is_planning`-Filter
        // in Slot-Selektion, spiegelt DAO-JOIN aus
        // `SlotDao::get_slots_for_week_all_plans`).
        //
        // **Follow-up #3 (ISO-Wochenjahr):** Alle drei Batches (year_reports,
        // special_days, shiftplan_reports) werden per ISO-Wochenjahr geladen —
        // NICHT per Kalender-Jahr. Der In-Memory-Filter im Loop bucketet per
        // `(year == outer_year_iso, week)`; Rows an KW 1 / KW 53, die kalendarisch
        // in ein anderes Jahr fielen (z.B. Feiertag am 2027-01-01 = ISO-2026-W53-Fri),
        // wurden zuvor verschluckt. Die neuen `_iso_year`-Varianten fixen das
        // strukturell: die DAO/Service-Ebene liefert genau die Rows, deren
        // ISO-Wochenjahr == `year` ist.
        //
        // Byte-Identität ist strukturell garantiert:
        // - `year_reports[week - 1].1` == `reporting_service.get_week(year, week)`
        //   (Wave-4 `assemble_weeks`-Delegation, Wave-1-Fixtures 8 grün).
        // - Per-Woche-In-Memory-Filter auf `special_days` / `shiftplan_reports`
        //   entspricht exakt `_for_week`-Filter (kein Semantik-Diff).
        // - Slot-In-Memory-Filter reproduziert DAO-`WHERE`-Klausel aus
        //   `dao_impl_sqlite/src/slot.rs:151-168` bit-genau (R1).
        let year_plus_1 = year + 1;
        let year_reports = self
            .reporting_service
            .get_year(year, Authentication::Full, tx.clone().into())
            .await?;
        let next_year_reports = self
            .reporting_service
            .get_year(year_plus_1, Authentication::Full, tx.clone().into())
            .await?;
        let special_days_this = self
            .special_day_service
            .get_by_iso_year(year, Authentication::Full)
            .await?;
        let special_days_next = self
            .special_day_service
            .get_by_iso_year(year_plus_1, Authentication::Full)
            .await?;
        let shiftplan_reports_this = self
            .shiftplan_report_service
            .extract_shiftplan_report_for_iso_year(year, Authentication::Full, tx.clone().into())
            .await?;
        let shiftplan_reports_next = self
            .shiftplan_report_service
            .extract_shiftplan_report_for_iso_year(
                year_plus_1,
                Authentication::Full,
                tx.clone().into(),
            )
            .await?;
        let all_slots = self
            .slot_service
            .get_slots(Authentication::Full, tx.clone().into())
            .await?;
        // is_planning-Bulk (Set): Slot ohne shiftplan_id ODER Slot mit
        // shiftplan_id, dessen Shiftplan `is_planning=false` ist, wird
        // berücksichtigt — spiegelt `(shiftplan.is_planning = 0 OR
        // shiftplan.is_planning IS NULL)` aus DAO-JOIN.
        let planning_shiftplan_ids: std::collections::HashSet<Uuid> = self
            .shiftplan_service
            .get_all(Authentication::Full, tx.clone().into())
            .await?
            .iter()
            .filter(|sp| sp.is_planning)
            .map(|sp| sp.id)
            .collect();

        let outer_year = year;
        for week in 1..=(weeks_in_year + 3) {
            let (year, week) = if week > weeks_in_year {
                (outer_year + 1, week - weeks_in_year)
            } else {
                (outer_year, week)
            };
            // Wähle Vec-Slice für die Zielwoche pro D-52-04-Spillover-Regel.
            let (year_reports_source, special_days_source, shiftplan_reports_source) =
                if year == outer_year {
                    (
                        &year_reports,
                        &special_days_this,
                        &shiftplan_reports_this,
                    )
                } else {
                    (
                        &next_year_reports,
                        &special_days_next,
                        &shiftplan_reports_next,
                    )
                };
            // VFA-01 (D-26-01 / D-26-03): build the set of volunteers absent in this calendar week.
            // Uses pre-loaded all_absences (load-once pattern). Category-agnostic: find_all returns
            // all three categories (Vacation, SickLeave, UnpaidLeave). Whole-week-out: any overlap
            // of Mon–Sun → full exclusion from both bands for that week (not pro-rated per day).
            // On date construction error (should not happen with valid ISO weeks), skip exclusion
            // for that week — never panic.
            let absent_volunteer_ids: std::collections::HashSet<Uuid> =
                if let (Ok(week_monday), Ok(week_sunday)) = (
                    time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday),
                    time::Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday),
                ) {
                    all_absences
                        .iter()
                        .filter(|period| {
                            volunteer_ids.contains(&period.sales_person_id)
                                && period_overlaps_week(
                                    period.from_date,
                                    period.to_date,
                                    week_monday,
                                    week_sunday,
                                )
                        })
                        .map(|period| period.sales_person_id)
                        .collect()
                } else {
                    std::collections::HashSet::new()
                };
            let mut working_hours_per_sales_person = vec![];
            // Phase 52 (WOP-02, D-52-03/D-52-04): `week_report` per Vec-Index
            // aus dem passenden `get_year`-Ergebnis. R6 Off-by-one-Guard:
            // Wave-4 garantiert `year_reports[week - 1].1 == get_week(year,
            // week)` byte-identisch (assemble_weeks-Delegation). Fixture 8
            // (Spillover W53→W55) fängt Off-by-one am Übergang ab.
            let week_report = year_reports_source
                .get((week - 1) as usize)
                .map(|(_, reports)| reports.clone())
                .unwrap_or_else(|| Arc::from(Vec::new()));
            // Phase 52 (WOP-01, D-52-01): In-Memory-Filter statt
            // `special_day_service.get_by_week(year, week)`. Semantik
            // identisch: die DAO liefert Rows `WHERE year = ? AND
            // calendar_week = ?` (soft-delete-Filter passiert im
            // Service-Layer bereits beim `get_by_year`-Bulk-Load).
            let special_days: Arc<[service::special_days::SpecialDay]> = special_days_source
                .iter()
                .filter(|d| d.year == year && d.calendar_week == week)
                .cloned()
                .collect();
            // Band 2 (D-05 / CVC-04 / CR-01 fix): per-person surplus = Σ max(actual_p − committed_p, 0).
            // CRITICAL: `extract_shiftplan_report_for_week` returns ONE ROW PER (person, day) because
            // the DAO query groups by `sales_person_id, year, day_of_week`. We MUST aggregate
            // per-person weekly actuals BEFORE applying the nonlinear max (CR-01 blocker):
            //   Buggy per-day form: max(3−5,0) + max(4−5,0) = 0 when actual=7, committed=5.
            //   Correct per-week:   max(7−5, 0) = 2.
            // volunteer_surplus_band2 accumulates per-day rows into per-person weekly totals first.
            //
            // Phase 52 (WOP-01, D-52-01): In-Memory-Filter statt
            // `extract_shiftplan_report_for_week(year, week)`. Der Bulk-Load
            // `extract_shiftplan_report_for_year` liefert dieselben
            // `ShiftplanReportDay`-Rows (`(sales_person_id, year,
            // calendar_week, day_of_week, hours)`) — Filter `year ==
            // target_year && calendar_week == target_week` reproduziert die
            // Per-Woche-Auswahl bit-genau.
            let shiftplan_reports: Arc<[service::shiftplan_report::ShiftplanReportDay]> =
                shiftplan_reports_source
                    .iter()
                    .filter(|r| r.year == year && r.calendar_week == week)
                    .cloned()
                    .collect();
            let per_day_actuals = shiftplan_reports
                .iter()
                .filter(|report| volunteer_ids.contains(&report.sales_person_id))
                .map(|report| (report.sales_person_id, report.hours));
            let volunteer_hours = volunteer_surplus_band2(per_day_actuals, |sp_id| {
                // VFA-01 (D-26-03 / D-26-01): absent volunteer's committed pledge drops to 0 for
                // this week (Band-2 consistency decision, 26-CONTEXT): removing the pledge from
                // BOTH bands prevents the surplus math from being overstated. Category-agnostic
                // because absent_volunteer_ids was built from the category-agnostic find_all.
                if absent_volunteer_ids.contains(&sp_id) {
                    return 0.0;
                }
                // Per-person cap-gated weekly committed (CVC-06): sum over the person's active rows.
                find_working_hours_for_calendar_week(&all_work_details, year, week)
                    .filter(|wh| {
                        wh.sales_person_id == sp_id
                            && (wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0) // D-05: cap || rein-freiwillig (expected_hours=0)
                    })
                    .map(|wh| wh.committed_voluntary) // D-03 flat, no weight
                    .sum()
            });
            // Band 1 (D-04 / CVC-04): cap-gated Σ_person committed per week (flat, no weight D-03).
            // Explicit per-row cap filter (Pitfall 5 / CVC-06): non-capped rows contribute 0.
            // VFA-01 (D-26-03): absent volunteers' committed contribution drops to 0 for the whole
            // week — category-agnostic whole-week-out, not pro-rated per day.
            let committed_voluntary_hours: f32 = find_working_hours_for_calendar_week(
                &all_work_details,
                year,
                week,
            )
            .filter(|wh| wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0) // D-05: cap || rein-freiwillig (expected_hours=0), symmetrisch zu D-01 Editor-Sichtbarkeit
            .filter(|wh| !absent_volunteer_ids.contains(&wh.sales_person_id)) // VFA-01 (D-26-03): absent → 0 for whole week
            .map(|wh| wh.committed_voluntary) // D-03 flat, no weight
            .sum();
            // VAA-01/02 (D-53-01/02/03): pro abwesendem Freiwilligen einen
            // Anzeige-Eintrag bauen. Cap-Gate-Formel identisch zu
            // `committed_voluntary_hours` oben, aber OHNE den absent-Filter —
            // hier sind gerade die Abwesenden gesucht. Pitfall 2:
            // `wh.sales_person_id == sp_id`-Filter ist Pflicht, sonst summiert
            // die personenuebergreifende Iteration ueber alle Personen.
            let sales_person_absences: Arc<
                [service::booking_information::SalesPersonAbsence],
            > = absent_volunteer_ids
                .iter()
                .filter_map(|&sp_id| {
                    let name = all_sales_persons
                        .iter()
                        .find(|sp| sp.id == sp_id)
                        .map(|sp| sp.name.clone())?;
                    let hours: f32 = find_working_hours_for_calendar_week(
                        &all_work_details,
                        year,
                        week,
                    )
                    .filter(|wh| {
                        wh.sales_person_id == sp_id
                            && (wh.cap_planned_hours_to_expected
                                || wh.expected_hours == 0.0)
                    })
                    .map(|wh| wh.committed_voluntary)
                    .sum();
                    Some(service::booking_information::SalesPersonAbsence {
                        sales_person_id: sp_id,
                        name,
                        hours,
                    })
                })
                .collect();
            // Phase 51 (D-51-06 Chain C + D-51-07): pro-Slot-Clip statt
            // Filter-Anti-Pattern. Holiday-Filter bleibt hart (kompletter Slot
            // raus); ShortDay-Slots werden via `Slot::clip_to` verkürzt statt
            // verworfen — `slot_hours` unten sieht dann die geclippten Zeiten
            // (D-04 Zeile 4). Legacy-Verhalten (Toggle aus / Gate inaktiv):
            // `clip_slot_for_week` gibt den Slot unverändert weiter.
            //
            // Phase 52 (WOP-01, D-52-01, R1): In-Memory-Filter statt
            // `slot_service.get_slots_for_week_all_plans(year, week)`.
            // Reproduziert exakt die DAO-`WHERE`-Klausel aus
            // `dao_impl_sqlite/src/slot.rs:151-168`:
            //   1. `slot.deleted IS NULL`
            //   2. `slot.valid_from <= sunday_of_week`
            //   3. `slot.valid_to IS NULL OR slot.valid_to >= monday_of_week`
            //   4. `shiftplan.is_planning = 0 OR shiftplan.is_planning IS NULL`
            //      (in-memory via `planning_shiftplan_ids`-Set: slot OHNE
            //      `shiftplan_id` ODER slot mit shiftplan_id, dessen
            //      Shiftplan NICHT planning ist).
            // Bei ISO-Wochen-Datum-Konstruktions-Fehler (praktisch nie): leere
            // Slot-Liste, wie DAO-Fehler-Pfad.
            let slots_for_week: Vec<Slot> = if let (Ok(monday_of_week), Ok(sunday_of_week)) = (
                time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday),
                time::Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday),
            ) {
                all_slots
                    .iter()
                    .filter(|slot| slot.deleted.is_none())
                    .filter(|slot| slot.valid_from <= sunday_of_week)
                    .filter(|slot| {
                        slot.valid_to
                            .map(|vt| vt >= monday_of_week)
                            .unwrap_or(true)
                    })
                    .filter(|slot| {
                        slot.shiftplan_id
                            .map(|sid| !planning_shiftplan_ids.contains(&sid))
                            .unwrap_or(true)
                    })
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            };
            let slots: Arc<[Slot]> = slots_for_week
                .iter()
                .filter(|slot| {
                    !special_days.iter().any(|day| {
                        day.day_of_week == slot.day_of_week
                            && day.day_type
                                == service::special_days::SpecialDayType::Holiday
                    })
                })
                .filter_map(|slot| {
                    // Chain C: Legacy-Mode — Gate aus + ShortDay ⇒ Slot droppen wenn
                    // slot.to > cutoff. Wiederherstellung Pre-Phase-51-Semantik
                    // (Commit 62a2f35^, `.filter(...ShortDay && slot.to > cutoff)`),
                    // damit `required_hours` in historischen Wochen (vor Stichtag /
                    // ohne Stichtag) mit v1.x-Backend identisch bleibt.
                    match shortday_gate::clip_slot_for_week(
                        slot,
                        &special_days,
                        year,
                        week,
                        active_from,
                        ShortdayMode::Legacy,
                    ) {
                        ClipOutcome::Keep(s) => Some(s),
                        ClipOutcome::Drop => None,
                    }
                })
                .collect();
            let slot_hours = slots
                .iter()
                .map(|slot| {
                    (slot.to - slot.from).as_seconds_f32() / 3600.0 * slot.min_resources as f32
                })
                .sum::<f32>();
            let mut paid_hours = 0.0;
            for report in week_report.iter() {
                paid_hours += report.dynamic_hours;
                if is_shiftplanner {
                    let absence_hours = report.vacation_hours
                        + report.sick_leave_hours
                        + report.holiday_hours
                        + report.custom_absence_hours.iter().map(|c| c.hours).sum::<f32>();
                    working_hours_per_sales_person.push(WorkingHoursPerSalesPerson {
                        sales_person_id: report.sales_person.id,
                        sales_person_name: report.sales_person.name.clone(),
                        available_hours: report.expected_hours,
                        absence_hours,
                        vacation_hours: report.vacation_hours,
                        sick_leave_hours: report.sick_leave_hours,
                        holiday_hours: report.holiday_hours,
                        unavailable_hours: report.unavailable_hours,
                        custom_absence_hours: report.custom_absence_hours.clone(),
                    });
                }
            }
            // D-01 (Phase 16): available = paid + committed (Band 1, pledge) + volunteer (Band 2, surplus).
            // No double-count: Band 2 already subtracted committed per-person (Σ max(actual−committed,0)).
            let overall_available_hours =
                committed_voluntary_hours + volunteer_hours + paid_hours;
            weekly_report.push(WeeklySummary {
                year,
                week,
                overall_available_hours,
                paid_hours,
                volunteer_hours,
                committed_voluntary_hours,
                working_hours_per_sales_person: working_hours_per_sales_person.into(),
                // VAA-01/02 (D-53-01/02/05): pro Woche im Assembly-Loop gebautes
                // Feld — Anzeige-Wert je Freiwilligem, cap-gated Wochen-Zusage.
                sales_person_absences,
                required_hours: slot_hours,
                monday_available_hours: 0.0,
                tuesday_available_hours: 0.0,
                wednesday_available_hours: 0.0,
                thursday_available_hours: 0.0,
                friday_available_hours: 0.0,
                saturday_available_hours: 0.0,
                sunday_available_hours: 0.0,
            });
        }

        self.transaction_dao.commit(tx).await?;
        Ok(weekly_report.into())
    }

    async fn get_summery_for_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeeklySummary, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (shiftplanner, sales) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context.clone())
        );
        shiftplanner.or(sales)?;

        let is_shiftplanner = self
            .permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
            .await
            .is_ok();

        let mut working_hours_per_sales_person = vec![];
        // VAA-01 (D-53-01/06, Pitfall 1): einmal laden, mehrfach nutzen —
        // `all_sales_persons` traegt Namen fuer den Freiwilligen-Absencen-
        // Namens-Lookup und wird weiter unten fuer `paid_employees` weiter-
        // verwendet.
        let all_sales_persons = self
            .sales_person_service
            .get_all(Authentication::Full, tx.clone().into())
            .await?;
        let volunteer_ids: Arc<[Uuid]> = all_sales_persons
            .iter()
            .filter(|sales_person| !sales_person.is_paid.unwrap_or(false))
            .map(|sales_person| sales_person.id)
            .collect();
        // VAA-01 (D-53-06): `all_absences` fuer die Single-Week-Variante laden —
        // identisch zu `get_weekly_summary` (Category-agnostisch: Vacation +
        // SickLeave + UnpaidLeave). Muss geladen werden, weil VFA-01 fuer die
        // Wochensicht nicht Standard-Input war (D-53-06 Motivation).
        let all_absences = self
            .absence_service
            .find_all(Authentication::Full, tx.clone().into())
            .await?;
        // VAA-01 (D-53-01/06, Pitfall 6): `absent_volunteer_ids` inline fuer
        // diese Woche bauen — identisches Muster wie `get_weekly_summary` (Zeile
        // 422-442). `volunteer_ids.contains(...)`-Filter ist Pflicht, sonst
        // leaken bezahlte Mitarbeiter mit Absence ins Freiwilligen-Feld.
        let absent_volunteer_ids: std::collections::HashSet<Uuid> =
            if let (Ok(week_monday), Ok(week_sunday)) = (
                time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday),
                time::Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday),
            ) {
                all_absences
                    .iter()
                    .filter(|period| {
                        volunteer_ids.contains(&period.sales_person_id)
                            && period_overlaps_week(
                                period.from_date,
                                period.to_date,
                                week_monday,
                                week_sunday,
                            )
                    })
                    .map(|period| period.sales_person_id)
                    .collect()
            } else {
                std::collections::HashSet::new()
            };

        let week_report = self
            .reporting_service
            .get_week(year, week, Authentication::Full, tx.clone().into())
            .await?;
        let special_days = self
            .special_day_service
            .get_by_week(year, week, Authentication::Full)
            .await?;
        let volunteer_hours = self
            .shiftplan_report_service
            .extract_shiftplan_report_for_week(year, week, Authentication::Full, tx.clone().into())
            .await?
            .iter()
            .filter(|report| volunteer_ids.contains(&report.sales_person_id))
            .map(|report| report.hours)
            .sum::<f32>();
        // Phase 51 (D-51-06 Chain C + D-51-07): pro-Slot-Clip vor Filter-Anti-
        // Pattern. Toggle-Prefetch für dieses (year, week) einmalig; `required_hours_by_day`
        // (unten) foldet auto-korrekt über die geclippte `slots`-Variable.
        // Gap-Closure: zentraler Helper mit `Unauthorized → None` (Legacy off).
        let active_from =
            shortday_gate::read_active_from(self.toggle_service.as_ref(), context.clone()).await?;
        let slots: Arc<[Slot]> = self
            .slot_service
            .get_slots_for_week_all_plans(year, week, Authentication::Full, tx.clone().into())
            .await?
            .iter()
            .filter(|slot| {
                !special_days.iter().any(|day| {
                    day.day_of_week == slot.day_of_week
                        && day.day_type == service::special_days::SpecialDayType::Holiday
                })
            })
            .filter_map(|slot| {
                // Chain C: Legacy-Mode — Gate aus + ShortDay ⇒ Slot droppen wenn
                // slot.to > cutoff. Siehe Kommentar in `get_weekly_summary`.
                match shortday_gate::clip_slot_for_week(
                    slot,
                    &special_days,
                    year,
                    week,
                    active_from,
                    ShortdayMode::Legacy,
                ) {
                    ClipOutcome::Keep(s) => Some(s),
                    ClipOutcome::Drop => None,
                }
            })
            .collect();
        let slot_hours = slots
            .iter()
            .map(|slot| (slot.to - slot.from).as_seconds_f32() / 3600.0 * slot.min_resources as f32)
            .sum::<f32>();
        let mut paid_hours = 0.0;
        for report in week_report.iter() {
            paid_hours += report.dynamic_hours;
            if is_shiftplanner {
                let absence_hours = report.vacation_hours
                    + report.sick_leave_hours
                    + report.holiday_hours
                    + report.custom_absence_hours.iter().map(|c| c.hours).sum::<f32>();
                working_hours_per_sales_person.push(WorkingHoursPerSalesPerson {
                    sales_person_id: report.sales_person.id,
                    sales_person_name: report.sales_person.name.clone(),
                    available_hours: report.expected_hours,
                    absence_hours,
                    vacation_hours: report.vacation_hours,
                    sick_leave_hours: report.sick_leave_hours,
                    holiday_hours: report.holiday_hours,
                    unavailable_hours: report.unavailable_hours,
                    custom_absence_hours: report.custom_absence_hours.clone(),
                });
            }
        }
        let overall_available_hours = volunteer_hours + paid_hours;

        // Calculate available hours per day
        let mut monday_hours = 0.0;
        let mut tuesday_hours = 0.0;
        let mut wednesday_hours = 0.0;
        let mut thursday_hours = 0.0;
        let mut friday_hours = 0.0;
        let mut saturday_hours = 0.0;
        let mut sunday_hours = 0.0;

        // VAA-01 (D-53-06, Pitfall 1): abgeleitet aus dem oben bereits geladenen
        // `all_sales_persons` — kein zweiter `get_all`-Aufruf.
        let paid_employees = all_sales_persons
            .iter()
            .filter(|sales_person| sales_person.is_paid.unwrap_or(false))
            .map(|sp| sp.id)
            .collect::<Vec<_>>();

        let work_details = self
            .employee_work_details_service
            .all(Authentication::Full, tx.clone().into())
            .await?;

        // VAA-01/02 (D-53-01/02/06): Freiwilligen-Absencen fuer die Wochensicht
        // — dieselbe Formel wie in `get_weekly_summary` (Fill-Site 1). Pitfall 2:
        // `wh.sales_person_id == sp_id`-Filter ist Pflicht.
        let sales_person_absences: Arc<
            [service::booking_information::SalesPersonAbsence],
        > = absent_volunteer_ids
            .iter()
            .filter_map(|&sp_id| {
                let name = all_sales_persons
                    .iter()
                    .find(|sp| sp.id == sp_id)
                    .map(|sp| sp.name.clone())?;
                let hours: f32 = find_working_hours_for_calendar_week(
                    &work_details,
                    year,
                    week,
                )
                .filter(|wh| {
                    wh.sales_person_id == sp_id
                        && (wh.cap_planned_hours_to_expected
                            || wh.expected_hours == 0.0)
                })
                .map(|wh| wh.committed_voluntary)
                .sum();
                Some(service::booking_information::SalesPersonAbsence {
                    sales_person_id: sp_id,
                    name,
                    hours,
                })
            })
            .collect();

        let unavailable_days = self
            .sales_person_unavailable_service
            .get_by_week(year, week, Authentication::Full, tx.clone().into())
            .await?;

        // Calculate per-day hours for each paid employee
        for employee_id in paid_employees {
            if let Some(details) = work_details.iter().find(|d| {
                d.sales_person_id == employee_id
                    && (d.from_year < year || (d.from_year == year && d.from_calendar_week <= week))
                    && (d.to_year > year || (d.to_year == year && d.to_calendar_week >= week))
            }) {
                // Check each day if employee is available (not in unavailable_days)
                let is_unavailable = |day: DayOfWeek| {
                    unavailable_days
                        .iter()
                        .any(|ud| ud.sales_person_id == employee_id && ud.day_of_week == day)
                };

                // Count working days excluding unavailable days
                let working_days = details
                    .potential_weekday_list()
                    .iter()
                    .filter(|&day| {
                        let service_day = match day {
                            time::Weekday::Monday => DayOfWeek::Monday,
                            time::Weekday::Tuesday => DayOfWeek::Tuesday,
                            time::Weekday::Wednesday => DayOfWeek::Wednesday,
                            time::Weekday::Thursday => DayOfWeek::Thursday,
                            time::Weekday::Friday => DayOfWeek::Friday,
                            time::Weekday::Saturday => DayOfWeek::Saturday,
                            time::Weekday::Sunday => DayOfWeek::Sunday,
                        };
                        !is_unavailable(service_day)
                    })
                    .count() as f32;

                if working_days > 0.0 {
                    let hours_per_day = details.expected_hours / working_days;

                    // Check each day if employee is available (not in unavailable_days)
                    let is_unavailable = |day: DayOfWeek| {
                        unavailable_days
                            .iter()
                            .any(|ud| ud.sales_person_id == employee_id && ud.day_of_week == day)
                    };

                    // Add hours to each working day if employee is available
                    for day in details.potential_weekday_list().iter() {
                        let service_day = match day {
                            time::Weekday::Monday => DayOfWeek::Monday,
                            time::Weekday::Tuesday => DayOfWeek::Tuesday,
                            time::Weekday::Wednesday => DayOfWeek::Wednesday,
                            time::Weekday::Thursday => DayOfWeek::Thursday,
                            time::Weekday::Friday => DayOfWeek::Friday,
                            time::Weekday::Saturday => DayOfWeek::Saturday,
                            time::Weekday::Sunday => DayOfWeek::Sunday,
                        };

                        if !is_unavailable(service_day) {
                            match service_day {
                                DayOfWeek::Monday => monday_hours += hours_per_day,
                                DayOfWeek::Tuesday => tuesday_hours += hours_per_day,
                                DayOfWeek::Wednesday => wednesday_hours += hours_per_day,
                                DayOfWeek::Thursday => thursday_hours += hours_per_day,
                                DayOfWeek::Friday => friday_hours += hours_per_day,
                                DayOfWeek::Saturday => saturday_hours += hours_per_day,
                                DayOfWeek::Sunday => sunday_hours += hours_per_day,
                            }
                        }
                    }
                }
            }
        }

        // Get volunteer hours per day from shiftplan report
        let volunteer_reports = self
            .shiftplan_report_service
            .extract_shiftplan_report_for_week(year, week, Authentication::Full, tx.clone().into())
            .await?;

        // Accumulate hours by day for volunteers
        let volunteer_hours_by_day = volunteer_reports
            .iter()
            .filter(|report| volunteer_ids.contains(&report.sales_person_id))
            .fold((0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0), |mut acc, report| {
                match report.day_of_week {
                    DayOfWeek::Monday => acc.0 += report.hours,
                    DayOfWeek::Tuesday => acc.1 += report.hours,
                    DayOfWeek::Wednesday => acc.2 += report.hours,
                    DayOfWeek::Thursday => acc.3 += report.hours,
                    DayOfWeek::Friday => acc.4 += report.hours,
                    DayOfWeek::Saturday => acc.5 += report.hours,
                    DayOfWeek::Sunday => acc.6 += report.hours,
                }
                acc
            });

        // Add volunteer hours from each day's available hours
        monday_hours += volunteer_hours_by_day.0;
        tuesday_hours += volunteer_hours_by_day.1;
        wednesday_hours += volunteer_hours_by_day.2;
        thursday_hours += volunteer_hours_by_day.3;
        friday_hours += volunteer_hours_by_day.4;
        saturday_hours += volunteer_hours_by_day.5;
        sunday_hours += volunteer_hours_by_day.6;

        // Calculate required hours per day from slots
        let required_hours_by_day =
            slots
                .iter()
                .fold((0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0), |mut acc, slot| {
                    let hours =
                        (slot.to - slot.from).as_seconds_f32() / 3600.0 * slot.min_resources as f32;
                    match slot.day_of_week {
                        DayOfWeek::Monday => acc.0 += hours,
                        DayOfWeek::Tuesday => acc.1 += hours,
                        DayOfWeek::Wednesday => acc.2 += hours,
                        DayOfWeek::Thursday => acc.3 += hours,
                        DayOfWeek::Friday => acc.4 += hours,
                        DayOfWeek::Saturday => acc.5 += hours,
                        DayOfWeek::Sunday => acc.6 += hours,
                    }
                    acc
                });

        let summary = WeeklySummary {
            year,
            week,
            overall_available_hours,
            paid_hours,
            volunteer_hours,
            // Phase 15: Band 1 is year-view-only (D-04); single-week variant keeps inert 0.0
            // placeholder (see 15-01-SUMMARY.md). volunteer_hours is left at full actual (no
            // per-person surplus reduction) because this variant feeds a per-day consumer.
            committed_voluntary_hours: 0.0,
            working_hours_per_sales_person: working_hours_per_sales_person.into(),
            // VAA-01/02 (D-53-01/02/06): Single-Week-Fill-Site — semantisch
            // identisch zu `get_weekly_summary` (Anzeige-Bruecke Wochensicht ↔
            // Jahressicht).
            sales_person_absences,
            required_hours: slot_hours,

            monday_available_hours: monday_hours - required_hours_by_day.0,
            tuesday_available_hours: tuesday_hours - required_hours_by_day.1,
            wednesday_available_hours: wednesday_hours - required_hours_by_day.2,
            thursday_available_hours: thursday_hours - required_hours_by_day.3,
            friday_available_hours: friday_hours - required_hours_by_day.4,
            saturday_available_hours: saturday_hours - required_hours_by_day.5,
            sunday_available_hours: sunday_hours - required_hours_by_day.6,
        };

        self.transaction_dao.commit(tx).await?;
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use service::booking_information::WeeklySummary;

    // --- volunteer_surplus_above_committed helper tests (Task 1, t1-t3) ---

    #[test]
    fn surplus_over_fulfilled() {
        // t1: committed=5, actual=7 → surplus = max(7-5, 0) = 2
        let result = volunteer_surplus_above_committed(7.0, 5.0);
        assert!((result - 2.0).abs() < 0.001, "expected 2.0, got {result}");
    }

    #[test]
    fn surplus_pledge_covers() {
        // t2: committed=5, actual=3 → surplus = max(3-5, 0) = 0 (floor)
        let result = volunteer_surplus_above_committed(3.0, 5.0);
        assert!((result - 0.0).abs() < 0.001, "expected 0.0, got {result}");
    }

    #[test]
    fn surplus_committed_zero_backward_compat() {
        // t3: committed=0, actual=7 → surplus = max(7-0, 0) = 7 (no-op, identical to pre-v1.4)
        let result = volunteer_surplus_above_committed(7.0, 0.0);
        assert!((result - 7.0).abs() < 0.001, "expected 7.0, got {result}");
    }

    #[test]
    fn weekly_summary_constructs_with_committed_field() {
        // t4: WeeklySummary with committed_voluntary_hours: 0.0 constructs, Clone/Debug/PartialEq work
        let summary = WeeklySummary {
            year: 2026,
            week: 1,
            overall_available_hours: 40.0,
            required_hours: 35.0,
            paid_hours: 40.0,
            volunteer_hours: 5.0,
            committed_voluntary_hours: 0.0,
            monday_available_hours: 8.0,
            tuesday_available_hours: 8.0,
            wednesday_available_hours: 8.0,
            thursday_available_hours: 8.0,
            friday_available_hours: 8.0,
            saturday_available_hours: 0.0,
            sunday_available_hours: 0.0,
            working_hours_per_sales_person: Arc::from(vec![]),
            sales_person_absences: Arc::from(vec![]),
        };
        let cloned = summary.clone();
        assert_eq!(summary, cloned);
        // Debug formatting must not panic
        let _debug = format!("{:?}", summary);
        assert!((summary.committed_voluntary_hours - 0.0).abs() < 0.001);
    }
}
