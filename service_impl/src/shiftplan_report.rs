//! Phase 51 Chain D — `ShiftplanReportService` mit Rust-Layer-Clip + Gate
//! (D-51-06 / D-51-08).
//!
//! Der DAO liefert Roh-Zeilen pro Booking (`ShiftplanReportRawRow`). Dieser
//! Service aggregiert nach `(sales_person_id, year, week, day_of_week)` und
//! wendet vorher pro Row den ShortDay-Clip + Stichtag-Gate an. Damit sehen
//! Balance-Views + Reporting geclippte Ist-Stunden (SHC-02), ohne dass
//! Bookings umgeschrieben werden (SHC-05), und ohne Umzüge von historischen
//! Ist-Stunden vor `active_from` (SHC-06, D-51-07).
//!
//! Snapshot-Immunität (D-03): `billing_period_report.rs` liest persistierte
//! Snapshot-Rows unverändert weiter. `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt
//! 12.

use crate::gen_service_impl;
use crate::shortday_gate;
use async_trait::async_trait;
use dao::{
    shiftplan_report::{ShiftplanReportDao, ShiftplanReportRawRow},
    TransactionDao,
};
use service::{
    permission::Authentication,
    shiftplan_report::{ShiftplanQuickOverview, ShiftplanReportDay, ShiftplanReportService},
    slot::Slot,
    special_days::{SpecialDay, SpecialDayService},
    toggle::ToggleService,
    ServiceError,
};
use shifty_utils::{DayOfWeek, ShiftyDate, ShiftyWeek};
use std::collections::HashMap;
use std::sync::Arc;
use time::Date;
use uuid::Uuid;

gen_service_impl! {
    struct ShiftplanReportServiceImpl: service::shiftplan_report::ShiftplanReportService = ShiftplanReportServiceDeps {
        ShiftplanReportDao: dao::shiftplan_report::ShiftplanReportDao<Transaction = Self::Transaction> = shiftplan_report_dao,
        SpecialDayService: SpecialDayService<Context = Self::Context> = special_day_service,
        ToggleService: ToggleService<Context = Self::Context, Transaction = Self::Transaction> = toggle_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

/// Baut einen ephemeren `Slot` aus einer Roh-Row.
///
/// Nur `from`, `to`, `day_of_week` werden vom Clip-Pfad gelesen; alle anderen
/// Felder sind Dummies. Keine DB-Zugriffe, keine Seiteneffekte.
fn slot_from_row(row: &ShiftplanReportRawRow) -> Slot {
    Slot {
        id: Uuid::nil(),
        day_of_week: row.day_of_week,
        from: row.time_from,
        to: row.time_to,
        min_resources: 0,
        max_paid_employees: None,
        valid_from: Date::from_iso_week_date(row.year as i32, row.calendar_week, row.day_of_week.into())
            .unwrap_or_else(|_| Date::from_calendar_date(2000, time::Month::January, 1).unwrap()),
        valid_to: None,
        deleted: None,
        version: Uuid::nil(),
        shiftplan_id: None,
    }
}

/// Wendet Clip + Gate pro Row an und liefert die verbleibenden Stunden.
///
/// Semantik (D-51-06 Chain D):
/// - Gate inaktiv (booking_date < active_from oder active_from == None)
///   → `hours` aus rohen Zeiten.
/// - Kein `ShortDay`-Cutoff für den Wochentag → `hours` aus rohen Zeiten.
/// - Cutoff greift → `Slot::clip_to(cutoff)`:
///   - `Some(clipped)` → `hours` aus geclippten Zeiten (D-04 Zeilen 1/2/4).
///   - `None` → 0.0 (Slot komplett hinter Cutoff, D-04 Zeile 3).
fn hours_for_row(
    row: &ShiftplanReportRawRow,
    special_days: &[SpecialDay],
    active_from: Option<Date>,
) -> f32 {
    let slot = slot_from_row(row);
    match shortday_gate::clip_slot_for_week(
        &slot,
        special_days,
        row.year,
        row.calendar_week,
        active_from,
    ) {
        shortday_gate::ClipOutcome::Keep(clipped) => {
            let secs = (clipped.to - clipped.from).as_seconds_f32();
            secs / 3600.0
        }
        shortday_gate::ClipOutcome::Drop => 0.0,
    }
}

#[async_trait]
impl<Deps: ShiftplanReportServiceDeps> ShiftplanReportService for ShiftplanReportServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn extract_shiftplan_report(
        &self,
        sales_person_id: Uuid,
        from_date: ShiftyDate,
        to_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanReportDay]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Toggle einmal fürs ganze Range (Rollout-Default None → Legacy).
        let toggle_raw = self
            .toggle_service
            .get_toggle_value(shortday_gate::TOGGLE_NAME, context.clone(), None)
            .await?;
        let active_from = shortday_gate::parse_active_from(toggle_raw.as_deref());

        // SpecialDays pro Woche cachen (Muster: reporting.rs:186-198).
        let mut special_days_by_week: HashMap<ShiftyWeek, Arc<[SpecialDay]>> = HashMap::new();
        let from_week = from_date.as_shifty_week();
        let to_week = to_date.as_shifty_week();
        for week in from_week.iter_until(&to_week) {
            let sds = self
                .special_day_service
                .get_by_week(week.year, week.week, context.clone())
                .await?;
            special_days_by_week.insert(week, sds);
        }

        let raw_rows = self
            .shiftplan_report_dao
            .extract_raw_shiftplan_report(
                sales_person_id,
                from_date.year(),
                from_date.week(),
                to_date.year(),
                to_date.week(),
                tx.clone(),
            )
            .await?;

        // Filter: Buchungs-Datum muss im [from_date, to_date]-Range liegen
        // (Match zum Verhalten von vor Phase 51 — Range-Query alignt lose auf
        // year*100+week, Datum-Filter zieht sauber nach).
        let filtered: Vec<&ShiftplanReportRawRow> = raw_rows
            .iter()
            .filter(|row| {
                match ShiftyDate::new(row.year, row.calendar_week, row.day_of_week) {
                    Ok(d) => d >= from_date && d <= to_date,
                    Err(_) => false,
                }
            })
            .collect();

        // Aggregation: (sales_person_id, year, week, day_of_week_number) → hours.
        // `DayOfWeek` selbst impl. weder `Hash` noch `Eq`, deshalb rutscht die
        // Nummer (`u8`) rein und wird beim Emit zurück-konvertiert.
        let mut agg: HashMap<(Uuid, u32, u8, u8), f32> = HashMap::new();
        for row in filtered {
            let week_key = ShiftyWeek::new(row.year, row.calendar_week);
            let empty: Arc<[SpecialDay]> = Arc::from(Vec::<SpecialDay>::new());
            let sds = special_days_by_week.get(&week_key).unwrap_or(&empty);
            let hours = hours_for_row(row, sds, active_from);
            *agg.entry((
                row.sales_person_id,
                row.year,
                row.calendar_week,
                row.day_of_week.to_number(),
            ))
                .or_insert(0.0) += hours;
        }

        let mut result: Vec<ShiftplanReportDay> = agg
            .into_iter()
            .filter_map(|((sales_person_id, year, calendar_week, dow_num), hours)| {
                DayOfWeek::from_number(dow_num).map(|day_of_week| ShiftplanReportDay {
                    sales_person_id,
                    hours,
                    year,
                    calendar_week,
                    day_of_week,
                })
            })
            .collect();
        result.sort_by(|a, b| {
            a.sales_person_id
                .cmp(&b.sales_person_id)
                .then_with(|| a.year.cmp(&b.year))
                .then_with(|| a.calendar_week.cmp(&b.calendar_week))
                .then_with(|| a.day_of_week.cmp(&b.day_of_week))
        });

        self.transaction_dao.commit(tx).await?;
        Ok(result.into())
    }

    async fn extract_quick_shiftplan_report(
        &self,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanQuickOverview]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let toggle_raw = self
            .toggle_service
            .get_toggle_value(shortday_gate::TOGGLE_NAME, context.clone(), None)
            .await?;
        let active_from = shortday_gate::parse_active_from(toggle_raw.as_deref());

        // SpecialDays für alle Wochen 1..=until_week (D-51-07 Gate-Check pro
        // Row braucht die Woche).
        let mut special_days_by_week: HashMap<u8, Arc<[SpecialDay]>> = HashMap::new();
        for week in 1..=until_week {
            let sds = self
                .special_day_service
                .get_by_week(year, week, context.clone())
                .await?;
            special_days_by_week.insert(week, sds);
        }

        let raw_rows = self
            .shiftplan_report_dao
            .extract_raw_quick_shiftplan_report(year, until_week, tx.clone())
            .await?;

        // Jahres-Rollup: (sales_person_id, year) → hours.
        let mut agg: HashMap<(Uuid, u32), f32> = HashMap::new();
        for row in raw_rows.iter() {
            let empty: Arc<[SpecialDay]> = Arc::from(Vec::<SpecialDay>::new());
            let sds = special_days_by_week
                .get(&row.calendar_week)
                .unwrap_or(&empty);
            let hours = hours_for_row(row, sds, active_from);
            *agg.entry((row.sales_person_id, row.year)).or_insert(0.0) += hours;
        }

        let mut result: Vec<ShiftplanQuickOverview> = agg
            .into_iter()
            .map(|((sales_person_id, year), hours)| ShiftplanQuickOverview {
                sales_person_id,
                hours,
                year,
            })
            .collect();
        result.sort_by(|a, b| {
            a.sales_person_id
                .cmp(&b.sales_person_id)
                .then_with(|| a.year.cmp(&b.year))
        });

        self.transaction_dao.commit(tx).await?;
        Ok(result.into())
    }

    async fn extract_shiftplan_report_for_week(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanReportDay]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let toggle_raw = self
            .toggle_service
            .get_toggle_value(shortday_gate::TOGGLE_NAME, context.clone(), None)
            .await?;
        let active_from = shortday_gate::parse_active_from(toggle_raw.as_deref());

        let special_days = self
            .special_day_service
            .get_by_week(year, calendar_week, context.clone())
            .await?;

        let raw_rows = self
            .shiftplan_report_dao
            .extract_raw_shiftplan_report_for_week(year, calendar_week, tx.clone())
            .await?;

        // Aggregation pro (sales_person_id, day_of_week_number) — die Woche ist fix.
        let mut agg: HashMap<(Uuid, u8), f32> = HashMap::new();
        for row in raw_rows.iter() {
            let hours = hours_for_row(row, &special_days, active_from);
            *agg.entry((row.sales_person_id, row.day_of_week.to_number()))
                .or_insert(0.0) += hours;
        }

        let mut result: Vec<ShiftplanReportDay> = agg
            .into_iter()
            .filter_map(|((sales_person_id, dow_num), hours)| {
                DayOfWeek::from_number(dow_num).map(|day_of_week| ShiftplanReportDay {
                    sales_person_id,
                    hours,
                    year,
                    calendar_week,
                    day_of_week,
                })
            })
            .collect();
        result.sort_by(|a, b| {
            a.sales_person_id
                .cmp(&b.sales_person_id)
                .then_with(|| a.day_of_week.cmp(&b.day_of_week))
        });

        self.transaction_dao.commit(tx).await?;
        Ok(result.into())
    }
}

