//! Service-Impl der Absence-Domain (Phase 1).
//!
//! Wiring per `gen_service_impl!` (Option A — minimaler Dependency-Set: nur
//! `AbsenceDao`, `PermissionService`, `SalesPersonService`, `ClockService`,
//! `UuidService`, `TransactionDao`; siehe D-08 und D-10 für die ausgeschlossenen
//! Hilfs-Services). Schreib- und Read-Methoden nutzen
//! `tokio::join!(check_permission(HR), verify_user_is_sales_person(...))` mit
//! `or` (D-09). `create` und `update` validieren Range (`DateRange::new` →
//! `DateOrderWrong`, D-14) und Self-Overlap via `find_overlapping`. Der
//! `update`-Pfad folgt 1:1 dem ExtraHours-`logical_id`-Pattern (Tombstone +
//! Insert, D-07) und exkludiert die alte Row beim Self-Overlap-Check
//! (`Some(logical_id)`, D-15). `delete` ist Soft-Delete via
//! `update(tombstone)`.

use crate::gen_service_impl;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    absence::{self, AbsenceDao},
    TransactionDao,
};
use service::{
    absence::{
        AbsenceCategory, AbsencePeriod, AbsencePeriodCreateResult, AbsenceService, ResolvedAbsence,
    },
    booking::BookingService,
    clock::ClockService,
    employee_work_details::EmployeeWorkDetailsService,
    permission::{Authentication, HR_PRIVILEGE},
    sales_person::SalesPersonService,
    sales_person_unavailable::SalesPersonUnavailableService,
    slot::SlotService,
    special_days::{SpecialDayService, SpecialDayType},
    uuid_service::UuidService,
    warning::Warning,
    PermissionService, ServiceError, ValidationFailureItem,
};
use shifty_utils::DateRange;
use time::Date;
use tokio::join;
use uuid::Uuid;

gen_service_impl! {
    struct AbsenceServiceImpl: AbsenceService = AbsenceServiceDeps {
        AbsenceDao: AbsenceDao<Transaction = Self::Transaction> = absence_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        SpecialDayService: SpecialDayService<Context = Self::Context> = special_day_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Context = Self::Context, Transaction = Self::Transaction> = employee_work_details_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
        // Phase-3 (D-Phase3-08): AbsenceService konsumiert BookingService und
        // SalesPersonUnavailableService für den Forward-Warning-Loop in
        // create/update sowie SlotService für Booking → Date-Auflösung
        // (Booking trägt nur slot_id + calendar_week + year).
        BookingService: BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        SalesPersonUnavailableService: SalesPersonUnavailableService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_unavailable_service,
        SlotService: SlotService<Context = Self::Context, Transaction = Self::Transaction> = slot_service,
    }
}

/// Prioritaet fuer den Cross-Category-Resolver (D-Phase2-03, BUrlG §9).
/// `SickLeave > Vacation > UnpaidLeave` — der Tag mit dominanter Kategorie
/// bekommt die Vertragsstunden, andere Kategorien produzieren 0.
fn absence_category_priority(category: &AbsenceCategory) -> u8 {
    match category {
        AbsenceCategory::SickLeave => 3,
        AbsenceCategory::Vacation => 2,
        AbsenceCategory::UnpaidLeave => 1,
    }
}

/// Helfer für `range.contains(date)` — `DateRange` selbst hat kein
/// `contains` (Phase-1-Surface), wir nutzen die invariante
/// `from <= date <= to`.
fn range_contains(range: &DateRange, date: Date) -> bool {
    range.from() <= date && date <= range.to()
}

#[async_trait]
impl<Deps: AbsenceServiceDeps> AbsenceService for AbsenceServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn find_all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let entities = self.absence_dao.find_all(tx.clone()).await?;
        let result: Arc<[AbsencePeriod]> = entities.iter().map(AbsencePeriod::from).collect();
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn find_by_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;

        let entities = self
            .absence_dao
            .find_by_sales_person(sales_person_id, tx.clone())
            .await?;
        let result: Arc<[AbsencePeriod]> = entities.iter().map(AbsencePeriod::from).collect();
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriod, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let active = self
            .absence_dao
            .find_by_logical_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;
        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                active.sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;
        let result = AbsencePeriod::from(&active);
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn create(
        &self,
        request: &AbsencePeriod,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriodCreateResult, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                request.sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;

        let mut entity = request.to_owned();
        if !entity.id.is_nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if !entity.version.is_nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }
        if entity.deleted.is_some() {
            return Err(ServiceError::DeletedSetOnCreate);
        }
        if entity.created.is_some() {
            return Err(ServiceError::CreatedSetOnCreate);
        }

        let new_range = DateRange::new(entity.from_date, entity.to_date)
            .map_err(|_| ServiceError::DateOrderWrong(entity.from_date, entity.to_date))?;

        // exclude_logical_id: None (Create-Pfad — keine eigene Row zu exkludieren).
        let conflicts = self
            .absence_dao
            .find_overlapping(
                entity.sales_person_id,
                (&entity.category).into(),
                new_range,
                None, // exclude_logical_id: None for create — there is no own row yet.
                tx.clone(),
            )
            .await?;
        if !conflicts.is_empty() {
            return Err(ServiceError::ValidationError(Arc::from([
                ValidationFailureItem::OverlappingPeriod(conflicts[0].logical_id),
            ])));
        }

        entity.id = self.uuid_service.new_uuid("absence_service::create::id");
        entity.version = self
            .uuid_service
            .new_uuid("absence_service::create::version");
        entity.created = Some(self.clock_service.date_time_now());

        let dao_entity = absence::AbsencePeriodEntity::try_from(&entity)?;
        self.absence_dao
            .create(&dao_entity, "absence_service::create", tx.clone())
            .await?;

        // Phase 3 — Forward-Warning-Loop (BOOK-01, D-Phase3-04).
        // Läuft NACH dem DAO-Persist + VOR commit, sodass Self-Conflicts
        // bereits validiert sind.
        let warnings = self
            .compute_forward_warnings(
                entity.id,
                entity.sales_person_id,
                new_range,
                tx.clone(),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(AbsencePeriodCreateResult {
            absence: entity,
            warnings,
        })
    }

    async fn update(
        &self,
        request: &AbsencePeriod,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriodCreateResult, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let logical_id = request.id;

        let active = self
            .absence_dao
            .find_by_logical_id(logical_id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(logical_id))?;

        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                active.sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;

        if request.sales_person_id != active.sales_person_id {
            return Err(ServiceError::ValidationError(Arc::from([
                ValidationFailureItem::ModificationNotAllowed("sales_person_id".into()),
            ])));
        }
        if request.version != active.version {
            return Err(ServiceError::EntityConflicts(
                logical_id,
                request.version,
                active.version,
            ));
        }

        let new_range = DateRange::new(request.from_date, request.to_date)
            .map_err(|_| ServiceError::DateOrderWrong(request.from_date, request.to_date))?;

        let conflicts = self
            .absence_dao
            .find_overlapping(
                active.sales_person_id,
                (&request.category).into(),
                new_range,
                Some(logical_id),
                tx.clone(),
            )
            .await?;
        if !conflicts.is_empty() {
            return Err(ServiceError::ValidationError(Arc::from([
                ValidationFailureItem::OverlappingPeriod(conflicts[0].logical_id),
            ])));
        }

        let mut tombstone = active.clone();
        tombstone.deleted = Some(self.clock_service.date_time_now());
        self.absence_dao
            .update(
                &tombstone,
                "absence_service::update::soft_delete",
                tx.clone(),
            )
            .await?;

        let new_id = self.uuid_service.new_uuid("absence_service::update::id");
        let new_version = self
            .uuid_service
            .new_uuid("absence_service::update::version");
        let now = self.clock_service.date_time_now();

        let new_entity = absence::AbsencePeriodEntity {
            id: new_id,
            logical_id: active.logical_id,
            sales_person_id: active.sales_person_id,
            category: (&request.category).into(),
            from_date: request.from_date,
            to_date: request.to_date,
            description: request.description.clone(),
            created: now,
            deleted: None,
            version: new_version,
        };
        self.absence_dao
            .create(&new_entity, "absence_service::update::insert", tx.clone())
            .await?;

        // Phase 3 — Forward-Warning-Loop (BOOK-01, D-Phase3-04: kein Diff —
        // alle Tage in der NEUEN Range, unabhängig vom alten Range).
        // Stable absence-id ist die `logical_id` (D-07) — Plan-3 nutzt sie
        // in den Warnings, damit die UI den logisch persistenten Eintrag
        // referenzieren kann (nicht die rotierte physische `new_id`).
        let warnings = self
            .compute_forward_warnings(
                active.logical_id,
                active.sales_person_id,
                new_range,
                tx.clone(),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(AbsencePeriodCreateResult {
            absence: AbsencePeriod::from(&new_entity),
            warnings,
        })
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let active = self
            .absence_dao
            .find_by_logical_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;
        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                active.sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;

        let mut tombstone = active;
        tombstone.deleted = Some(self.clock_service.date_time_now());
        self.absence_dao
            .update(&tombstone, "absence_service::delete", tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn derive_hours_for_range(
        &self,
        from: Date,
        to: Date,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BTreeMap<Date, ResolvedAbsence>, ServiceError> {
        // Range-Validation zuerst (analog Phase-1 D-14).
        let date_range = DateRange::new(from, to)
            .map_err(|_| ServiceError::DateOrderWrong(from, to))?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Permission: HR ∨ self (analog find_by_sales_person, D-10/D-11).
        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                sales_person_id,
                context.clone(),
                tx.clone().into()
            ),
        );
        hr.or(sp)?;

        // Batch-Fetch: Absences fuer den Mitarbeiter (D-Phase2-02 — keine
        // per-Tag-DAO-Calls; einmal lesen, dann per Tag filtern).
        let absences = self
            .absence_dao
            .find_by_sales_person(sales_person_id, tx.clone())
            .await?;

        // Batch-Fetch: alle Vertraege; per Tag den am Tag aktiven via
        // from_date()/to_date() filtern (analog reporting::find_working_hours_for_calendar_week).
        let work_details = self
            .employee_work_details_service
            .find_by_sales_person_id(sales_person_id, context.clone(), Some(tx.clone()))
            .await?;

        // Kalenderwochen-Set aus dem Range bilden — pro Woche EIN
        // SpecialDayService-Call (deduplizierter batch).
        let mut weeks: BTreeSet<(u32, u8)> = BTreeSet::new();
        for day in date_range.iter_days() {
            let (iso_year, iso_week, _) = day.to_iso_week_date();
            weeks.insert((iso_year as u32, iso_week));
        }

        // Holidays aus SpecialDayService ziehen und auf konkrete time::Date
        // mappen (SpecialDay traegt year/calendar_week/day_of_week).
        let mut holidays: BTreeSet<Date> = BTreeSet::new();
        for (year, week) in weeks.iter() {
            let special = self
                .special_day_service
                .get_by_week(*year, *week, context.clone())
                .await?;
            for sd in special.iter() {
                if sd.deleted.is_some() {
                    continue;
                }
                if sd.day_type != SpecialDayType::Holiday {
                    continue;
                }
                if let Ok(holiday_date) = time::Date::from_iso_week_date(
                    sd.year as i32,
                    sd.calendar_week,
                    sd.day_of_week.into(),
                ) {
                    holidays.insert(holiday_date);
                }
            }
        }

        // Per-Tag-Iteration: Vertragsauswahl, Kalenderfilter, Holiday-Skip,
        // dominante Kategorie via Prio-Reihenfolge.
        let mut result: BTreeMap<Date, ResolvedAbsence> = BTreeMap::new();
        for day in date_range.iter_days() {
            // Aktiven Vertrag fuer den Tag waehlen (from_date()/to_date()
            // sind ShiftyDate-Wrapper — bei Konvertierungs-Fehlern wird der
            // Vertrag uebersprungen).
            let active_contract = work_details.iter().find(|wh| {
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
            if !contract.has_day_of_week(day.weekday()) {
                continue;
            }
            if holidays.contains(&day) {
                continue;
            }
            let hours = contract.hours_per_day();
            if hours <= 0.0 {
                continue;
            }

            // Aktive Absences: alle nicht-getombstoneten Perioden, deren
            // [from_date, to_date] den Tag enthaelt.
            let dominant = absences
                .iter()
                .filter(|ap| {
                    ap.deleted.is_none() && ap.from_date <= day && day <= ap.to_date
                })
                .max_by_key(|ap| absence_category_priority(&(&ap.category).into()));
            let Some(dominant) = dominant else {
                continue;
            };
            result.insert(
                day,
                ResolvedAbsence {
                    category: (&dominant.category).into(),
                    hours,
                },
            );
        }

        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn find_overlapping_for_booking(
        &self,
        sales_person_id: Uuid,
        range: DateRange,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        // Permission HR ∨ self (D-Phase3-12) — gleiche Read-Regel wie in
        // `find_by_sales_person`.
        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;

        let entities = self
            .absence_dao
            .find_overlapping_for_booking(sales_person_id, range, tx.clone())
            .await?;
        let result: Arc<[AbsencePeriod]> = entities.iter().map(AbsencePeriod::from).collect();
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }
}

// =========================================================================
// Phase-3 Forward-Warning-Helper (private to the impl block).
//
// `compute_forward_warnings` läuft NACH dem DAO-Persist von `create`/`update`
// und VOR dem `commit`. Pro Booking-Tag in der neuen Range gibt es genau eine
// `Warning::AbsenceOverlapsBooking` (D-Phase3-15: keine De-Dup); pro
// überlappendem ManualUnavailable genau eine `Warning::AbsenceOverlapsManualUnavailable`
// (D-Phase3-16: KEIN Auto-Cleanup).
//
// Performance-Hinweis (C-Phase3-02): Der Loop iteriert pro Booking-betroffene
// Kalenderwoche genau einmal `BookingService::get_for_week`. Pro Booking
// genau einmal `SlotService::get_slot` für die day_of_week-Auflösung —
// in einer Range mit N Wochen und M Bookings sind das O(N + M) Calls.
// `SalesPersonUnavailableService::get_all_for_sales_person` läuft genau
// einmal — clientseitiger Filter pro Tag. Plan 06 verifiziert Performance
// auf 60-Tage-Ranges.
// =========================================================================
impl<Deps: AbsenceServiceDeps> AbsenceServiceImpl<Deps> {
    async fn compute_forward_warnings(
        &self,
        absence_id: Uuid,
        sales_person_id: Uuid,
        new_range: DateRange,
        tx: <Deps as AbsenceServiceDeps>::Transaction,
    ) -> Result<Arc<[Warning]>, ServiceError> {
        let mut warnings: Vec<Warning> = Vec::new();

        // 1) Bookings — pro betroffener Kalenderwoche genau ein
        // `BookingService::get_for_week`-Call (deduplizierter batch).
        // Authentication::Full umgeht die Permission-Probe innerhalb des
        // Service-internen Loops; die outer Permission ist oben in
        // create/update bereits HR ∨ self verifiziert.
        let mut weeks_seen: BTreeSet<(u32, u8)> = BTreeSet::new();
        for day in new_range.iter_days() {
            let (iso_year, iso_week, _weekday) = day.to_iso_week_date();
            let week_key = (iso_year as u32, iso_week);
            if !weeks_seen.insert(week_key) {
                continue;
            }
            let bookings = self
                .booking_service
                .get_for_week(iso_week, iso_year as u32, Authentication::Full, tx.clone().into())
                .await?;
            for b in bookings.iter() {
                if b.sales_person_id != sales_person_id {
                    continue;
                }
                if b.deleted.is_some() {
                    continue;
                }
                // Booking → Date: braucht `Slot.day_of_week`. `Booking`
                // selbst trägt nur slot_id + calendar_week + year.
                let slot = self
                    .slot_service
                    .get_slot(&b.slot_id, Authentication::Full, tx.clone().into())
                    .await?;
                let booking_date = match time::Date::from_iso_week_date(
                    b.year as i32,
                    b.calendar_week as u8,
                    slot.day_of_week.into(),
                ) {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                if !range_contains(&new_range, booking_date) {
                    continue;
                }
                warnings.push(Warning::AbsenceOverlapsBooking {
                    absence_id,
                    booking_id: b.id,
                    date: booking_date,
                });
            }
        }

        // 2) ManualUnavailables — ein Call, clientseitiger Range-Filter.
        let manual_all = self
            .sales_person_unavailable_service
            .get_all_for_sales_person(
                sales_person_id,
                Authentication::Full,
                tx.clone().into(),
            )
            .await?;
        for mu in manual_all.iter() {
            if mu.deleted.is_some() {
                continue;
            }
            let mu_date = match time::Date::from_iso_week_date(
                mu.year as i32,
                mu.calendar_week,
                mu.day_of_week.into(),
            ) {
                Ok(d) => d,
                Err(_) => continue,
            };
            if !range_contains(&new_range, mu_date) {
                continue;
            }
            warnings.push(Warning::AbsenceOverlapsManualUnavailable {
                absence_id,
                unavailable_id: mu.id,
            });
        }

        Ok(Arc::from(warnings))
    }
}
