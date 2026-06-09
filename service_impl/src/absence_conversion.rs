//! Phase 8.5 (D-03) — AbsenceConversionServiceImpl: schlanke 3-write Tx.
//!
//! Extrahiert die drei Kern-Writes aus `convert_quarantine_entry`
//! (cutover.rs:551-582) ohne Quarantaene-/Heuristik-/Drift-Ballast:
//! 1. absence_dao.create
//! 2. migration_source_dao.upsert_migration_source (Backlink)
//! 3. extra_hours_service.soft_delete_bulk
//!
//! BL-Tier-Service (D-03). Privileg-Gate: hr (D-05). Kein Snapshot-Bump (D-16).

use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    absence::{AbsenceDao, AbsencePeriodEntity},
    extra_hours::{ExtraHoursCategoryEntity, ExtraHoursDao},
    migration_source::{MigrationSourceDao, MigrationSourceRow},
    TransactionDao,
};
use service::{
    absence::{AbsencePeriod, DayFraction},
    absence_conversion::AbsenceConversionService,
    extra_hours::ExtraHoursService,
    permission::{Authentication, HR_PRIVILEGE},
    PermissionService, ServiceError, ValidationFailureItem,
};
use shifty_utils::DateRange;
use uuid::Uuid;

use crate::gen_service_impl;

gen_service_impl! {
    struct AbsenceConversionServiceImpl: service::absence_conversion::AbsenceConversionService = AbsenceConversionServiceDeps {
        ExtraHoursDao: dao::extra_hours::ExtraHoursDao<Transaction = Self::Transaction> = extra_hours_dao,
        AbsenceDao: dao::absence::AbsenceDao<Transaction = Self::Transaction> = absence_dao,
        MigrationSourceDao: dao::migration_source::MigrationSourceDao<Transaction = Self::Transaction> = migration_source_dao,
        ExtraHoursService: service::extra_hours::ExtraHoursService<Context = Self::Context, Transaction = Self::Transaction> = extra_hours_service,
        PermissionService: service::PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

/// Map the extra_hours legacy category enum to the absence_period category enum.
/// Caller MUST ensure the input is one of {Vacation, SickLeave, UnpaidLeave};
/// other variants return Err (kein panic in BL-Tier-Service — sicher gegen
/// schlechte Daten).
///
/// Kopiert aus cutover.rs:1470-1479 (A3: eigenstaendig damit Phase 8.6
/// cutover.rs sauber loeschen kann ohne diesen Service anzufassen).
pub(crate) fn extra_hours_category_to_absence(
    c: &ExtraHoursCategoryEntity,
) -> Result<dao::absence::AbsenceCategoryEntity, ServiceError> {
    match c {
        ExtraHoursCategoryEntity::Vacation => Ok(dao::absence::AbsenceCategoryEntity::Vacation),
        ExtraHoursCategoryEntity::SickLeave => Ok(dao::absence::AbsenceCategoryEntity::SickLeave),
        ExtraHoursCategoryEntity::UnpaidLeave => {
            Ok(dao::absence::AbsenceCategoryEntity::UnpaidLeave)
        }
        _other => Err(ServiceError::EntityNotFoundGeneric(Arc::from(
            "extra_hours category not convertible (must be Vacation, SickLeave or UnpaidLeave)",
        ))),
    }
}

#[async_trait]
impl<Deps: AbsenceConversionServiceDeps> AbsenceConversionService
    for AbsenceConversionServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn convert_extra_hours_to_absence(
        &self,
        extra_hours_id: Uuid,
        from_date: time::Date,
        to_date: time::Date,
        day_fraction: Option<DayFraction>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriod, ServiceError> {
        // 1. hr-Gate VOR Tx-Open (D-05, T-8.5-02a)
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        // 2. Tx oeffnen
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // 3. Zeitstempel fuer created / migrated_at
        let now = time::OffsetDateTime::now_utc();
        let migrated_at = time::PrimitiveDateTime::new(now.date(), now.time());

        // 4. Row laden + validieren (T-8.5-02b: find_by_logical_id filtert deleted IS NULL)
        let entity = self
            .extra_hours_dao
            .find_by_logical_id(extra_hours_id, tx.clone())
            .await?
            .ok_or_else(|| {
                ServiceError::EntityNotFoundGeneric(Arc::from(
                    "extra_hours not found or already converted",
                ))
            })?;

        // Kategorie validieren (T-8.5-02c)
        let target_category = extra_hours_category_to_absence(&entity.category)?;

        // 5. DateRange validieren (T-8.5-02d: Overlap-Check)
        let new_range = DateRange::new(from_date, to_date)
            .map_err(|_| ServiceError::DateOrderWrong(from_date, to_date))?;

        // 6. Overlap-Check (exclude_logical_id = None: Create-Pfad)
        let conflicts = self
            .absence_dao
            .find_overlapping(
                entity.sales_person_id,
                target_category.clone(),
                new_range,
                None,
                tx.clone(),
            )
            .await?;
        if !conflicts.is_empty() {
            return Err(ServiceError::ValidationError(Arc::from([
                ValidationFailureItem::OverlappingPeriod(conflicts[0].logical_id),
            ])));
        }

        // Write 1 — absence_dao.create (aus cutover.rs:530-553)
        let absence_period_id = Uuid::new_v4();
        let absence_entity = AbsencePeriodEntity {
            id: absence_period_id,
            logical_id: absence_period_id,
            sales_person_id: entity.sales_person_id,
            category: target_category,
            from_date,
            to_date,
            description: Arc::from(""),
            created: migrated_at,
            deleted: None,
            version: Uuid::new_v4(),
            day_fraction: day_fraction
                .as_ref()
                .map(dao::absence::DayFractionEntity::from)
                .unwrap_or(dao::absence::DayFractionEntity::Full),
        };
        self.absence_dao
            .create(&absence_entity, "absence_conversion::convert", tx.clone())
            .await?;

        // Write 2 — Backlink via MigrationSourceDao (befreit von run-Kopplung: D-04)
        self.migration_source_dao
            .upsert_migration_source(
                &MigrationSourceRow {
                    extra_hours_id,
                    absence_period_id,
                    migrated_at,
                },
                tx.clone(),
            )
            .await?;

        // Write 3 — soft-delete (aus cutover.rs:575-582; Pitfall 5: intern Authentication::Full
        // nach dem hr-Gate — der echte Guard ist das hr-Gate am Anfang dieser Methode)
        self.extra_hours_service
            .soft_delete_bulk(
                Arc::from([extra_hours_id]),
                "absence_conversion::convert",
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok((&absence_entity).into())
    }
}
