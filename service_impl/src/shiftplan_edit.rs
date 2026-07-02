use std::sync::Arc;

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    absence::AbsenceService,
    booking::{Booking, BookingService},
    carryover::{Carryover, CarryoverService},
    employee_work_details::EmployeeWorkDetailsService,
    extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService},
    permission::{Authentication, SHIFTPLANNER_PRIVILEGE},
    reporting::ReportingService,
    sales_person::SalesPersonService,
    sales_person_unavailable::{SalesPersonUnavailable, SalesPersonUnavailableService},
    shiftplan_edit::{BookingCreateResult, CopyWeekResult, ShiftplanEditService},
    slot::{Slot, SlotService},
    toggle::ToggleService,
    warning::Warning,
    week_status::WeekStatusService,
    PermissionService, ServiceError,
};
use tokio::join;
use uuid::Uuid;

use crate::gen_service_impl;

gen_service_impl! {
    struct ShiftplanEditServiceImpl: ShiftplanEditService = ShiftplanEditServiceDeps {
        PermissionService: service::PermissionService<Context = Self::Context> = permission_service,
        SlotService: service::slot::SlotService<Transaction = Self::Transaction> = slot_service,
        BookingService: service::booking::BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        CarryoverService: service::carryover::CarryoverService<Context = Self::Context, Transaction = Self::Transaction> = carryover_service,
        ReportingService: service::reporting::ReportingService<Context = Self::Context, Transaction = Self::Transaction> = reporting_service,
        SalesPersonService: service::sales_person::SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        SalesPersonUnavailableService: SalesPersonUnavailableService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_unavailable_service,
        EmployeeWorkDetailsService: service::employee_work_details::EmployeeWorkDetailsService<Context = Self::Context, Transaction = Self::Transaction> = employee_work_details_service,
        ExtraHoursService: ExtraHoursService<Context = Self::Context, Transaction = Self::Transaction> = extra_hours_service,
        UuidService: service::uuid_service::UuidService = uuid_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao,
        // NEU für Phase 3 (D-Phase3-06): Reverse-Warning konsumiert AbsenceService
        AbsenceService: service::absence::AbsenceService<Context = Self::Context, Transaction = Self::Transaction> = absence_service,
        // D-24-08: ToggleService für paid_limit_hard_enforcement-Prüfung
        ToggleService: service::toggle::ToggleService<Context = Self::Context, Transaction = Self::Transaction> = toggle_service,
        // NEU für Phase 40 (D-40-01): WeekStatusService liefert den Lock-Status
        // für das Wochen-Sperre-Gate. Basic-Tier-Dep in Business-Logic-Service
        // (CLAUDE.md § Service-Tier-Konventionen).
        WeekStatusService: service::week_status::WeekStatusService<Context = Self::Context, Transaction = Self::Transaction> = week_status_service
    }
}

#[async_trait]
impl<Deps: ShiftplanEditServiceDeps> ShiftplanEditService for ShiftplanEditServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn modify_slot(
        &self,
        slot: &Slot,
        change_year: u32,
        change_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Slot, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission("shiftplan.edit", context.clone())
            .await?;
        // Phase 40 (D-40-01): Wochen-Sperre-Gate (Scaffold, blockiert noch nicht).
        self.assert_week_not_locked(change_year, change_week, context.clone(), tx.clone())
            .await?;

        let mut stored_slot = self
            .slot_service
            .get_slot(&slot.id, Authentication::Full, tx.clone().into())
            .await?;

        if stored_slot.version != slot.version {
            return Err(ServiceError::EntityConflicts(
                slot.id,
                stored_slot.version,
                slot.version,
            ));
        }

        let new_slot_valid_from =
            time::Date::from_iso_week_date(change_year as i32, change_week, time::Weekday::Monday)?;
        let old_slot_valid_to = new_slot_valid_from - time::Duration::days(1);
        let bookings = self
            .booking_service
            .get_for_slot_id_since(
                slot.id,
                change_year,
                change_week,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let original_valid_to = stored_slot.valid_to;

        stored_slot.valid_to = Some(old_slot_valid_to);

        if stored_slot.valid_to.unwrap() < stored_slot.valid_from {
            self.slot_service
                .delete_slot(&stored_slot.id, Authentication::Full, tx.clone().into())
                .await?;
        } else {
            self.slot_service
                .update_slot(&stored_slot, Authentication::Full, tx.clone().into())
                .await?;
        }

        let mut new_slot = stored_slot;
        new_slot.valid_from = new_slot_valid_from;
        new_slot.valid_to = original_valid_to;
        new_slot.id = Uuid::nil();
        new_slot.version = Uuid::nil();
        new_slot.min_resources = slot.min_resources;
        new_slot.max_paid_employees = slot.max_paid_employees;
        new_slot.from = slot.from;
        new_slot.to = slot.to;

        let new_slot = self
            .slot_service
            .create_slot(&new_slot, Authentication::Full, tx.clone().into())
            .await?;

        for booking in bookings.iter() {
            self.booking_service
                .delete(booking.id, Authentication::Full, tx.clone().into())
                .await?;

            let mut new_booking = booking.clone();
            new_booking.id = Uuid::nil();
            new_booking.version = Uuid::nil();
            new_booking.slot_id = new_slot.id;
            new_booking.year = booking.year;
            new_booking.calendar_week = booking.calendar_week;
            new_booking.created = None;
            // created_by = None → BookingService::create's fallback chain stamps
            // "system" (Authentication::Full). The original booker's authorship
            // survives in the soft-deleted predecessor row in bookings_view.
            new_booking.created_by = None;

            self.booking_service
                .create(&new_booking, Authentication::Full, tx.clone().into())
                .await?;
        }

        self.transaction_dao.commit(tx).await?;
        Ok(new_slot)
    }

    async fn remove_slot(
        &self,
        slot_id: Uuid,
        change_year: u32,
        change_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission("shiftplan.edit", context.clone())
            .await?;
        // Phase 40 (D-40-01): Wochen-Sperre-Gate (Scaffold, blockiert noch nicht).
        self.assert_week_not_locked(change_year, change_week, context.clone(), tx.clone())
            .await?;

        let mut stored_slot = self
            .slot_service
            .get_slot(&slot_id, Authentication::Full, tx.clone().into())
            .await?;

        let new_slot_valid_from =
            time::Date::from_iso_week_date(change_year as i32, change_week, time::Weekday::Monday)?;
        let old_slot_valid_to = new_slot_valid_from - time::Duration::days(1);
        let bookings = self
            .booking_service
            .get_for_slot_id_since(
                slot_id,
                change_year,
                change_week,
                Authentication::Full,
                None,
            )
            .await?;

        stored_slot.valid_to = Some(old_slot_valid_to);

        if stored_slot.valid_to.unwrap() < stored_slot.valid_from {
            self.slot_service
                .delete_slot(&stored_slot.id, Authentication::Full, tx.clone().into())
                .await?;
        } else {
            self.slot_service
                .update_slot(&stored_slot, Authentication::Full, tx.clone().into())
                .await?;
        }

        for booking in bookings.iter() {
            self.booking_service
                .delete(booking.id, Authentication::Full, Some(tx.clone()))
                .await?;
        }

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn modify_slot_single_week(
        &self,
        slot: &Slot,
        change_year: u32,
        change_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Slot, ServiceError> {
        // D-35-04: EINE Transaktion — alle Schritte innerhalb dieser tx
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // D-35-06: Permission-Gate als erster Aufruf (vor jeder Mutation)
        self.permission_service
            .check_permission("shiftplan.edit", context.clone())
            .await?;
        // Phase 40 (D-40-01): Wochen-Sperre-Gate (Scaffold, blockiert noch nicht).
        self.assert_week_not_locked(change_year, change_week, context.clone(), tx.clone())
            .await?;

        let mut stored_slot = self
            .slot_service
            .get_slot(&slot.id, Authentication::Full, tx.clone().into())
            .await?;

        // Versionskonflikt-Check (T-35-04)
        if stored_slot.version != slot.version {
            return Err(ServiceError::EntityConflicts(
                slot.id,
                stored_slot.version,
                slot.version,
            ));
        }

        // Datumsgrenzen (ISO-Woche)
        let new_slot_valid_from =
            time::Date::from_iso_week_date(change_year as i32, change_week, time::Weekday::Monday)?;
        let old_slot_valid_to = new_slot_valid_from - time::Duration::days(1); // Sonntag KW-1
        let seg2_valid_to = new_slot_valid_from + time::Duration::days(6); // Sonntag KW
        let seg3_valid_from = new_slot_valid_from + time::Duration::days(7); // Montag KW+1

        // Buchungen ab change_week abrufen (werden in Rust-Code partitioniert, D-35-03)
        let bookings = self
            .booking_service
            .get_for_slot_id_since(
                slot.id,
                change_year,
                change_week,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;

        let original_valid_to = stored_slot.valid_to;

        // Pitfall 2: Snapshot VOR jeder Mutation aufnehmen (für Segment 3)
        let original_snapshot = stored_slot.clone();

        // Segment 1: Original-Slot auf valid_to = Sonntag KW-1 verkürzen
        stored_slot.valid_to = Some(old_slot_valid_to);
        if stored_slot.valid_to.unwrap() < stored_slot.valid_from {
            // Erste-KW-Edge: Segment 1 wäre leer → delete_slot (Pitfall 5)
            self.slot_service
                .delete_slot(&stored_slot.id, Authentication::Full, tx.clone().into())
                .await?;
        } else {
            self.slot_service
                .update_slot(&stored_slot, Authentication::Full, tx.clone().into())
                .await?;
        }

        // Segment 2: Ausnahme-Woche (Mon KW → Son KW) mit neuen Werten aus `slot`
        // Pitfall 1: valid_to = Some(seg2_valid_to) statt original_valid_to (geschlossen!)
        // WR-01: Klemme seg2_valid_to auf original_valid_to, falls der Slot begrenzt ist —
        // damit überschreitet Seg2 nie das ursprüngliche Slot-Ende.
        let mut seg2 = stored_slot;
        seg2.valid_from = new_slot_valid_from;
        seg2.valid_to = Some(match original_valid_to {
            Some(vt) => seg2_valid_to.min(vt),
            None => seg2_valid_to,
        }); // geschlossen bei Sonntag KW (oder früherem Slot-Ende bei bounded)
        seg2.id = Uuid::nil();
        seg2.version = Uuid::nil();
        seg2.min_resources = slot.min_resources;
        seg2.max_paid_employees = slot.max_paid_employees;
        seg2.from = slot.from;
        seg2.to = slot.to;
        let seg2_slot = self
            .slot_service
            .create_slot(&seg2, Authentication::Full, tx.clone().into())
            .await?;

        // Segment 3: Wiederherstellung ab Montag KW+1 mit Original-Werten (NEU)
        // WR-01: Nur anlegen wenn noch eine nicht-leere Restspanne existiert.
        // Bei bounded Slots, deren valid_to in der Ausnahme-KW liegt, wäre
        // seg3_valid_from > original_valid_to → create_slot würde DateOrderWrong
        // melden und die gesamte Methode würde fehlschlagen.
        let seg3_slot_id: Option<Uuid> =
            if original_valid_to.is_none_or(|vt| seg3_valid_from <= vt) {
                let mut seg3 = original_snapshot;
                seg3.valid_from = seg3_valid_from;
                seg3.valid_to = original_valid_to; // None = unbegrenzt bleibt None
                seg3.id = Uuid::nil();
                seg3.version = Uuid::nil();
                // min_resources, max_paid_employees, from, to = Original (aus snapshot)
                let seg3_slot = self
                    .slot_service
                    .create_slot(&seg3, Authentication::Full, tx.clone().into())
                    .await?;
                Some(seg3_slot.id)
            } else {
                // Ausnahme-KW ist die letzte Woche des Slots → keine Restspanne
                None
            };

        // Booking-Re-Point: partitioniert nach Ausnahme-KW (D-35-03, Pitfall 3/4)
        for booking in bookings.iter() {
            self.booking_service
                .delete(booking.id, Authentication::Full, tx.clone().into())
                .await?;

            // Pitfall 3: calendar_week ist i32, change_week ist u8 → expliziter Cast
            let target_slot_id =
                if booking.year == change_year && booking.calendar_week == change_week as i32 {
                    seg2_slot.id // Buchung ist IN der Ausnahme-KW → Segment 2
                } else {
                    // Buchung ist NACH der Ausnahme-KW → Segment 3.
                    // Invariante: post-exception Buchungen implizieren, dass der Slot
                    // über die Ausnahme-KW hinausgeht → seg3_slot_id ist Some(_).
                    seg3_slot_id
                        .expect("post-exception Buchung impliziert, dass Segment 3 existiert")
                };

            let mut new_booking = booking.clone();
            new_booking.id = Uuid::nil();
            new_booking.version = Uuid::nil();
            new_booking.slot_id = target_slot_id;
            new_booking.year = booking.year;
            new_booking.calendar_week = booking.calendar_week;
            new_booking.created = None;
            // created_by = None → ursprünglicher Ersteller bleibt in soft-deleted Vorgänger-Row
            new_booking.created_by = None;

            self.booking_service
                .create(&new_booking, Authentication::Full, tx.clone().into())
                .await?;
        }

        // D-35-04: GENAU EIN commit am Ende (kein Zwischen-commit)
        self.transaction_dao.commit(tx).await?;
        Ok(seg2_slot) // Ausnahme-Slot zurückgeben (Edit-Ziel)
    }

    async fn update_carryover(
        &self,
        sales_person_id: Uuid,
        year: u32,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let until_week = time::util::weeks_in_year(year as i32);

        let employee_report = self
            .reporting_service
            .get_report_for_employee(
                &sales_person_id,
                year,
                until_week,
                Authentication::Full,
                tx.clone().into(),
            )
            .await?;

        let new_carryover_hours = employee_report.balance_hours;
        let new_vacation_entitlement =
            (employee_report.vacation_entitlement - employee_report.vacation_days).floor() as i32;

        let now = time::OffsetDateTime::now_utc();
        let created = time::PrimitiveDateTime::new(now.date(), now.time());

        let new_carryover = Carryover {
            sales_person_id,
            year,
            carryover_hours: new_carryover_hours,
            vacation: new_vacation_entitlement,
            created,
            deleted: None,
            version: uuid::Uuid::nil(),
        };

        self.carryover_service
            .set_carryover(&new_carryover, Authentication::Full, tx.clone().into())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn update_carryover_all_employees(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Acquire (or reuse) a transaction
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Make sure the caller is allowed to edit shift plans
        self.permission_service
            .check_permission("shiftplan.edit", context.clone())
            .await?;

        // Retrieve all sales persons
        let sales_persons = self
            .sales_person_service
            .get_all(context.clone(), tx.clone().into())
            .await?;

        // Call update_carryover for each sales person
        for sp in sales_persons.iter() {
            // Pass the same transaction along so everything is done in a single transaction.
            // Alternatively, if you want each carryover update to be committed separately,
            // you could pass None here. But typically we want one big transaction.
            self.update_carryover(sp.id, year, context.clone(), Some(tx.clone()))
                .await?;
        }

        // Commit everything at the end
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn add_vacation(
        &self,
        sales_person_id: Uuid,
        from: time::Date,
        to: time::Date,
        description: Arc<str>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        // Permission check is done by the service calls

        let (from_year, from_week, from_weekday) = from.to_iso_week_date();

        let employee_work_details = self
            .employee_work_details_service
            .find_for_week(
                sales_person_id,
                from_week,
                from_year as u32,
                context.clone(),
                tx.clone().into(),
            )
            .await?;

        let mut date = from;
        let mut current_year = from_year as u32;
        let mut current_week = from_week;
        let mut current_weekday = from_weekday;
        let mut vacation_days_for_week = 0;
        while date <= to {
            if employee_work_details.has_day_of_week(date.weekday()) {
                vacation_days_for_week += 1;
            }

            let (year, week, weekday) = date.to_iso_week_date();
            if (current_year, current_week) != (year as u32, week) {
                let amount = (employee_work_details.hours_per_day()
                    * vacation_days_for_week as f32)
                    .min(employee_work_details.expected_hours);
                let date = time::Date::from_iso_week_date(
                    current_year as i32,
                    current_week,
                    current_weekday,
                )?;
                let date_time = time::PrimitiveDateTime::new(date, time::Time::MIDNIGHT);

                let vacation = ExtraHours {
                    id: Uuid::nil(),
                    sales_person_id,
                    amount,
                    category: ExtraHoursCategory::Vacation,
                    description: description.clone(),
                    date_time,
                    created: None,
                    deleted: None,
                    version: Uuid::nil(),
                };

                let _ = self
                    .extra_hours_service
                    .create(&vacation, context.clone(), tx.clone().into())
                    .await?;

                current_year = year as u32;
                current_week = week;
                current_weekday = weekday;
                vacation_days_for_week = 0;
            }

            let employee_unavailable = SalesPersonUnavailable {
                sales_person_id,
                id: Uuid::nil(),
                year: year as u32,
                calendar_week: week,
                day_of_week: weekday.into(),
                created: None,
                deleted: None,
                version: Uuid::nil(),
            };
            match self
                .sales_person_unavailable_service
                .create(&employee_unavailable, context.clone(), tx.clone().into())
                .await
            {
                // Ignore if the day is already blocked.
                Err(ServiceError::EntityAlreadyExists(_)) => Ok(()),
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }?;
            date += time::Duration::days(1);
        }
        let amount = (employee_work_details.hours_per_day() * vacation_days_for_week as f32)
            .min(employee_work_details.expected_hours);
        let date = time::Date::from_iso_week_date(
            current_year as i32,
            current_week,
            current_weekday,
        )?;
        let date_time = time::PrimitiveDateTime::new(date, time::Time::MIDNIGHT);

        let vacation = ExtraHours {
            id: Uuid::nil(),
            sales_person_id,
            amount,
            category: ExtraHoursCategory::Vacation,
            description: description.clone(),
            date_time,
            created: None,
            deleted: None,
            version: Uuid::nil(),
        };

        let _ = self
            .extra_hours_service
            .create(&vacation, context.clone(), tx.clone().into())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn book_slot_with_conflict_check(
        &self,
        booking: &Booking,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BookingCreateResult, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Permission Shiftplanner ∨ self (D-24-04): gate korrigiert von HR → Shiftplanner.
        // Shiftplanner darf andere Personen buchen; ein Mitarbeiter trägt sich selbst ein.
        // Admin behält Zugriff via admin-auto-grant-Trigger (alle Privilegien).
        let (sp_perm, self_perm) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                booking.sales_person_id,
                context.clone(),
                tx.clone().into(),
            ),
        );
        // WR-01: Capture is_shiftplanner from the already-computed sp_perm result before
        // consuming it with .or(). This avoids the redundant second check_permission call
        // inside the enforcement guard below.
        let is_shiftplanner = sp_perm.is_ok();
        sp_perm.or(self_perm)?;

        // Phase 40 (D-40-01/02): Wochen-Sperre-Gate für Nicht-Schichtplaner
        // (Scaffold, blockiert noch nicht). Schichtplaner umgehen den Read.
        if !is_shiftplanner {
            self.assert_week_not_locked(
                booking.year,
                booking.calendar_week as u8,
                context.clone(),
                tx.clone(),
            )
            .await?;
        }

        // Slot-Lookup für day_of_week (Pattern aus modify_slot).
        let slot = self
            .slot_service
            .get_slot(&booking.slot_id, Authentication::Full, tx.clone().into())
            .await?;

        // D-24-02/D-24-08: hard-enforcement pre-persist block. Toggle wird pro Buchung
        // frisch gelesen (is_enabled ist auth-only → Authentication::Full, konsistent
        // mit den anderen inneren Cross-Service-Lookups in dieser Methode).
        // Nur bezahlte Personen zählen (D-24-Grenzregel, strikt-größer wie Soft-Warning).
        // Shiftplanner bypassen (D-24-02). Existing Bookings werden NIE rückwirkend
        // angefasst (D-07 / D-24).
        if let Some(max) = slot.max_paid_employees {
            let hard = self
                .toggle_service
                .is_enabled(
                    "paid_limit_hard_enforcement",
                    Authentication::Full,
                    tx.clone().into(),
                )
                .await?;
            if hard && !is_shiftplanner {
                // Zähle bereits persistierte paid-Bookings in (slot_id, year, week).
                // Das neue Booking ist noch nicht persistiert, daher ist dies der
                // pre-persist-Count. Block wenn (existing + this_if_paid) > max.
                // CR-02: Reuse the paid_ids from the same fetch to determine
                // booked_is_paid, avoiding a second get_all_paid DAO round-trip.
                let (existing_paid, paid_ids) = self
                    .count_paid_bookings_in_slot_week(
                        booking.slot_id,
                        booking.year,
                        booking.calendar_week as u8,
                        tx.clone(),
                    )
                    .await?;
                let booked_is_paid = paid_ids.contains(&booking.sales_person_id);
                let prospective =
                    if booked_is_paid { existing_paid.saturating_add(1) } else { existing_paid };
                // Strikt-größer, deckungsgleich mit Soft-Warning (current > max).
                if prospective > max {
                    return Err(ServiceError::PaidLimitExceeded {
                        current: prospective,
                        max,
                    });
                }
            }
        }

        // Date-Konversion: Booking trägt nur (year, calendar_week, slot_id);
        // wir resolven den exakten Tag via Slot.day_of_week.
        let booking_date: time::Date = time::Date::from_iso_week_date(
            booking.year as i32,
            booking.calendar_week as u8,
            slot.day_of_week.into(),
        )?;
        let single_day_range = shifty_utils::DateRange::new(booking_date, booking_date)
            .map_err(|_| ServiceError::DateOrderWrong(booking_date, booking_date))?;

        // AbsencePeriod-Lookup (cross-Kategorie, soft-delete-gefiltert im DAO).
        let absence_periods = self
            .absence_service
            .find_overlapping_for_booking(
                booking.sales_person_id,
                single_day_range,
                Authentication::Full,
                tx.clone().into(),
            )
            .await?;

        // ManualUnavailable-Lookup pro Woche.
        let manual_unavailables = self
            .sales_person_unavailable_service
            .get_by_week_for_sales_person(
                booking.sales_person_id,
                booking.year,
                booking.calendar_week as u8,
                Authentication::Full,
                tx.clone().into(),
            )
            .await?;

        // Persist via Basic-Service — BookingService::create UNVERÄNDERT
        // (D-Phase3-18 Regression-Lock). Wir reichen den ursprünglichen User
        // via booking.created_by durch, damit das Audit-Feld nicht NULL bleibt
        // (BookingService::create läuft hier mit Authentication::Full und kann
        // den User sonst nicht ableiten).
        let creator = self
            .permission_service
            .current_user_id(context.clone())
            .await?;
        let booking_with_creator = Booking {
            created_by: creator,
            ..booking.clone()
        };
        let persisted_booking = self
            .booking_service
            .create(&booking_with_creator, Authentication::Full, tx.clone().into())
            .await?;

        // Warnings mit echter persistierter Booking-ID. KEINE De-Dup zwischen
        // Quellen (D-Phase3-15) — pro Quelle ein eigener Warning-Eintrag.
        let mut warnings: Vec<Warning> = Vec::new();
        for ap in absence_periods.iter() {
            // Phase 8.3 (D-08.3-05 / ROADMAP SC #6) — Halbtag-Absences werden
            // schweigend toleriert: ein Booking am selben Tag wie eine
            // day_fraction=Half-Absence ist ein legitimer Workflow (Mitarbeiter
            // arbeitet die andere Tageshaelfte). Wir filtern Half-Absences vor
            // der Warning-Emission AUS — kein neuer WarningTO-Variant (CONTEXT.md
            // <domain> "Liefert NICHT" + <deferred> "Konflikt-Warning fuer
            // Halbtag-Booking-Overlap: schweigend toleriert").
            if ap.day_fraction == service::absence::DayFraction::Half {
                continue;
            }
            warnings.push(Warning::BookingOnAbsenceDay {
                booking_id: persisted_booking.id,
                date: booking_date,
                absence_id: ap.id,
                category: ap.category,
            });
        }
        for mu in manual_unavailables.iter() {
            // Soft-Delete-Filter (Pitfall 1 / SC4) + Day-of-Week-Match.
            if mu.deleted.is_none() && mu.day_of_week == slot.day_of_week {
                warnings.push(Warning::BookingOnUnavailableDay {
                    booking_id: persisted_booking.id,
                    year: booking.year,
                    week: booking.calendar_week as u8,
                    day_of_week: slot.day_of_week,
                });
                // Eine Warning pro day_of_week-Match reicht — die DAO liefert
                // typischerweise nur einen Eintrag pro (sp, year, week, dow).
                break;
            }
        }

        // Phase 5 (D-04, D-06, D-07, D-08, D-15, D-16): Paid-Employee-Limit-
        // Soft-Warning. Slot wurde oben (line 419-422) bereits geladen;
        // `slot.max_paid_employees: Option<u8>` ist in-hand.
        // - D-15 (NULL = no limit): nur prüfen, wenn `Some(max)` gesetzt.
        // - D-06 (strikt-größer): Warning nur bei `current > max`, NICHT bei
        //   `current == max`.
        // - D-07 (kein Rollback): Buchung bleibt persistiert; Warning ist
        //   informativ.
        // - D-04 (Count-Regel): aktive Bookings im (slot_id, year, week) mit
        //   `sales_person.is_paid = true`, soft-deletes ausgefiltert.
        // - D-05 (Absence orthogonal): Absence-Status der gebuchten Person
        //   ist irrelevant — eingetragen ist eingetragen.
        // - D-16 (Endpoint-Scope): ausschließlich auf diesem conflict-aware
        //   Pfad. Legacy `POST /booking` (BookingService::create) bleibt
        //   unverändert.
        // Cast: `booking.calendar_week` ist `i32`; mirroring der existierenden
        // `as u8`-Konvention (siehe BookingOnUnavailableDay-Emission oben).
        if let Some(max) = slot.max_paid_employees {
            // CR-02: destructure; the paid_ids set is not needed for the soft-warning
            // path (we only need the count post-persist), so we discard it.
            let (current_paid_count, _paid_ids) = self
                .count_paid_bookings_in_slot_week(
                    booking.slot_id,
                    booking.year,
                    booking.calendar_week as u8,
                    tx.clone(),
                )
                .await?;
            if current_paid_count > max {
                warnings.push(Warning::PaidEmployeeLimitExceeded {
                    slot_id: booking.slot_id,
                    booking_id: persisted_booking.id,
                    year: booking.year,
                    week: booking.calendar_week as u8,
                    current_paid_count,
                    max_paid_employees: max,
                });
            }
        }

        self.transaction_dao.commit(tx).await?;
        Ok(BookingCreateResult {
            booking: persisted_booking,
            warnings: Arc::from(warnings),
        })
    }

    async fn copy_week_with_conflict_check(
        &self,
        from_calendar_week: u8,
        from_year: u32,
        to_calendar_week: u8,
        to_year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CopyWeekResult, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Bulk-Operation auf Schichtplan-Ebene → shiftplan.edit-Permission
        // (analog modify_slot/remove_slot). KEIN HR ∨ self pro Source-Booking,
        // weil die Operation pro Aufruf alle Bookings einer Woche umfasst.
        self.permission_service
            .check_permission("shiftplan.edit", context.clone())
            .await?;
        // Phase 40 (D-40-01): Wochen-Sperre-Gate — AUSSCHLIESSLICH die Ziel-Woche
        // (Lesen der Quelle ist nie gesperrt). Scaffold, blockiert noch nicht.
        self.assert_week_not_locked(to_year, to_calendar_week, context.clone(), tx.clone())
            .await?;

        // Source-Bookings via BookingService (Basic, unangetastet).
        let source_bookings = self
            .booking_service
            .get_for_week(
                from_calendar_week,
                from_year,
                Authentication::Full,
                tx.clone().into(),
            )
            .await?;

        let mut copied_bookings: Vec<Booking> = Vec::new();
        let mut all_warnings: Vec<Warning> = Vec::new();

        for source in source_bookings.iter() {
            // Konstruiere Ziel-Booking — id/version werden vom BookingService
            // beim create() neu vergeben; calendar_week/year werden überschrieben.
            let target = Booking {
                id: Uuid::nil(),
                version: Uuid::nil(),
                created: None,
                deleted: None,
                deleted_by: None,
                created_by: None,
                calendar_week: to_calendar_week as i32,
                year: to_year,
                ..source.clone()
            };

            let result = self
                .book_slot_with_conflict_check(&target, context.clone(), Some(tx.clone()))
                .await?;
            copied_bookings.push(result.booking);
            all_warnings.extend(result.warnings.iter().cloned());
        }

        self.transaction_dao.commit(tx).await?;
        Ok(CopyWeekResult {
            copied_bookings: Arc::from(copied_bookings),
            warnings: Arc::from(all_warnings),
        })
    }

    async fn delete_booking(
        &self,
        booking_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Booking laden um (year, calendar_week) zu lesen — VOR dem Delete, sonst
        // ist die Entity soft-deleted und der Lock-Read hätte keine Woche mehr.
        let booking = self
            .booking_service
            .get(booking_id, Authentication::Full, Some(tx.clone()))
            .await?;

        // Phase 40 (D-40-02): 6. Schreibpfad — Wochen-Sperre-Gate (Scaffold,
        // blockiert noch nicht). Reihenfolge get → assert → delete ist zwingend.
        self.assert_week_not_locked(
            booking.year,
            booking.calendar_week as u8,
            context.clone(),
            tx.clone(),
        )
        .await?;

        // Delegation an Basic-Tier-BookingService::delete erhält die Permission
        // (Shiftplanner ∨ Self) — nur der Lock-Gate wird hier addiert.
        self.booking_service
            .delete(booking_id, context, Some(tx.clone()))
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}

// Phase 5 (D-04, D-05, D-12) — private Helpers für die Paid-Employee-Limit-
// Warning-Emission. Lebt im Business-Logic-Tier (`ShiftplanEditServiceImpl`),
// NICHT auf dem Basic-Tier `BookingService` (CLAUDE.md § "Service-Tier-
// Konventionen" + v1.0 D-Phase3-18 Regression-Lock: BookingService bleibt
// strikt Basic-Tier).
impl<Deps: ShiftplanEditServiceDeps> ShiftplanEditServiceImpl<Deps> {
    /// Phase 40 (D-40-01) — Wochen-Sperre-Gate. SCAFFOLD-Variante (Plan 40-01):
    /// liest den Wochen-Status in DERSELBEN Transaktion wie der Write (kein
    /// TOCTOU), gibt aber IMMER `Ok(())` zurück. Die eigentliche Blockier-Logik
    /// (`WeekStatus::Locked` → `ServiceError::WeekLocked`) und der
    /// `shiftplan.edit`-Bypass folgen RED-first in Plan 40-03. Der Read „benutzt"
    /// die neue `WeekStatusService`-Dep, damit kein dead-field-Lint entsteht.
    async fn assert_week_not_locked(
        &self,
        year: u32,
        calendar_week: u8,
        _context: Authentication<Deps::Context>,
        tx: Deps::Transaction,
    ) -> Result<(), ServiceError> {
        let _status = self
            .week_status_service
            .get_week_status(year, calendar_week, Authentication::Full, Some(tx))
            .await?;
        Ok(())
    }

    /// Phase 5 (D-04, D-05): zählt aktive Bookings im (slot_id, year, week)
    /// deren Sales Person aktuell `is_paid = true` hat.
    ///
    /// Filter-Predikat (mirrors `service_impl::shiftplan::build_shiftplan_day`
    /// aus Plan 05-04 für behavioural consistency):
    /// - `bookings.deleted IS NULL` — wird upstream im `booking_dao` gefiltert;
    ///   die zusätzliche `b.deleted.is_none()`-Prüfung hier ist
    ///   belt-and-suspenders.
    /// - `sales_person.is_paid.unwrap_or(false) == true` UND
    ///   `sales_person.deleted IS NULL` — wird upstream durch
    ///   `SalesPersonService::get_all_paid` gefiltert (DAO `all_paid` =
    ///   `WHERE deleted IS NULL AND is_paid = 1`).
    /// - Absence-Status der gebuchten Person ist **irrelevant** (D-05):
    ///   eingetragen ist eingetragen.
    ///
    /// Verwendet `Authentication::Full` für die inneren Cross-Service-
    /// Lookups — die Permission des äußeren Aufrufers wurde bereits in
    /// `book_slot_with_conflict_check` validiert (Shiftplanner ∨ self, D-24-04).
    ///
    /// Saturating-Cast `count.min(u8::MAX as usize) as u8` analog zu
    /// Plan 05-04's `current_paid_count`-Derivation in `build_shiftplan_day`.
    ///
    /// CR-02: Returns `(count, paid_ids)` so callers can determine `booked_is_paid`
    /// from the same `get_all_paid` fetch, avoiding a redundant DAO round-trip.
    async fn count_paid_bookings_in_slot_week(
        &self,
        slot_id: Uuid,
        year: u32,
        week: u8,
        tx: Deps::Transaction,
    ) -> Result<(u8, std::collections::HashSet<Uuid>), ServiceError> {
        let bookings = self
            .booking_service
            .get_for_week(week, year, Authentication::Full, Some(tx.clone()))
            .await?;
        let paid_persons = self
            .sales_person_service
            .get_all_paid(Authentication::Full, Some(tx.clone()))
            .await?;
        let paid_ids: std::collections::HashSet<Uuid> =
            paid_persons.iter().map(|sp| sp.id).collect();
        let count = bookings
            .iter()
            .filter(|b| b.slot_id == slot_id && b.deleted.is_none())
            .filter(|b| paid_ids.contains(&b.sales_person_id))
            .count();
        Ok((count.min(u8::MAX as usize) as u8, paid_ids))
    }
}
