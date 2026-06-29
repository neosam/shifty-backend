//! Vacation-Entitlement-Offset-Domain für Phase 28 (VAC-OFFSET-01).
//!
//! Stellt das Service-Trait [`VacationEntitlementOffsetService`] sowie die
//! Domain-Struktur [`VacationEntitlementOffset`] bereit: ein vorzeichen-
//! behafteter (signed) Korrektur-Offset pro Mitarbeiter und Kalenderjahr,
//! der vom Business-Logic-Service `VacationBalanceService` (Plan 28-02)
//! NACH der `.round()`-Berechnung des Vertragsanspruchs addiert wird.
//!
//! Tier-Klassifizierung: **Basic-Service (Entity-Manager)** (D-28-06). Der
//! Service hängt ausschließlich von DAO + Permission + Clock + Uuid +
//! Transaction ab — KEIN Domain-Service als Dependency, damit kein Zyklus
//! mit `VacationBalanceService` entsteht.
//!
//! Permissionsmodell: get/set/delete sind allesamt HR-gated
//! (`HR_PRIVILEGE`, D-28-06b). Ein nicht-HR-Aufrufer erhält
//! [`ServiceError::Forbidden`].
//!
//! `automock` erzeugt `MockVacationEntitlementOffsetService` für die
//! Plan-28-02-Tests.

use std::fmt::Debug;

use async_trait::async_trait;
use dao::vacation_entitlement_offset::VacationEntitlementOffsetEntity;
use mockall::automock;
use uuid::Uuid;

use crate::{permission::Authentication, ServiceError};

/// Vorzeichenbehafteter Urlaubsanspruch-Offset eines Mitarbeiters für ein
/// konkretes Kalenderjahr. Genau eine aktive Zeile pro (sales_person_id,
/// year) (Soft-Delete-Historie erlaubt).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VacationEntitlementOffset {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub year: u32,
    /// Signierte Korrektur in ganzen Tagen (kann negativ sein).
    pub offset_days: i32,
    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

impl From<&VacationEntitlementOffsetEntity> for VacationEntitlementOffset {
    fn from(entity: &VacationEntitlementOffsetEntity) -> Self {
        Self {
            id: entity.id,
            sales_person_id: entity.sales_person_id,
            year: entity.year,
            offset_days: entity.offset_days,
            created: Some(entity.created),
            deleted: entity.deleted,
            version: entity.version,
        }
    }
}

impl TryFrom<&VacationEntitlementOffset> for VacationEntitlementOffsetEntity {
    type Error = ServiceError;

    fn try_from(offset: &VacationEntitlementOffset) -> Result<Self, Self::Error> {
        Ok(Self {
            id: offset.id,
            sales_person_id: offset.sales_person_id,
            year: offset.year,
            offset_days: offset.offset_days,
            created: offset.created.ok_or(ServiceError::InternalError)?,
            deleted: offset.deleted,
            version: offset.version,
        })
    }
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait VacationEntitlementOffsetService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Liefert den aktiven Offset für genau einen Mitarbeiter und ein
    /// Kalenderjahr (oder `None`, wenn keiner gesetzt ist). HR-gated
    /// (D-28-06b / D-28-03).
    async fn get(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<VacationEntitlementOffset>, ServiceError>;

    /// Setzt (upsert) den Offset für (sales_person_id, year): existiert
    /// bereits eine aktive Zeile, wird `offset_days` + neue Version
    /// aktualisiert, sonst eine neue Zeile mit frischer id/version/created
    /// angelegt. HR-gated (D-28-06b).
    async fn set(
        &self,
        sales_person_id: Uuid,
        year: u32,
        offset_days: i32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<VacationEntitlementOffset, ServiceError>;

    /// Soft-deleted den aktiven Offset für (sales_person_id, year). Gibt
    /// `EntityNotFound`, wenn keiner existiert. HR-gated (D-28-06b).
    async fn delete(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
