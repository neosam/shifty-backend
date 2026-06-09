//! Phase 8.5 (D-03) — Dedizierter BL-Tier-Service fuer den per-Row-Convert
//! von `extra_hours` (Vacation/SickLeave/UnpaidLeave) nach `absence_period`.
//!
//! Dieser Service teilt NICHTS mit der Cutover-Maschinerie — er extrahiert nur
//! die drei Kern-Writes (absence_dao.create + migration_source_dao.upsert +
//! extra_hours_service.soft_delete_bulk) in eine saubere, hr-gated Heimat.
//! Phase 8.6 loescht CutoverServiceImpl rein subtraktiv ohne diesen Service
//! anpassen zu muessen.

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::{permission::Authentication, ServiceError};

/// Service fuer den atomaren HR-gated Convert einer lebenden `extra_hours`-Row
/// (Kategorie Vacation/SickLeave/UnpaidLeave) in eine `absence_period`.
///
/// Privileg: `hr` (D-05). Die drei Writes (create + backlink + soft-delete)
/// laufen in einer gemeinsamen Transaktion (atomar, D-04).
///
/// KEIN Snapshot-Bump erforderlich (D-16): Convert ist ein per-Row-Daten-Umzug;
/// Reporting summiert beide Quellen seit 8.4 additiv per-row.
#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait AbsenceConversionService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Convert a single living extra_hours row (Vacation/SickLeave/UnpaidLeave)
    /// into an absence_period over the HR-supplied [from_date, to_date] range.
    /// hr-privileged. Atomic: create absence_period + write backlink + soft-delete extra_hours.
    ///
    /// Fehlerverhalten:
    /// - `start > end` → `ServiceError::DateOrderWrong(start, end)`
    /// - Overlap mit bestehender absence_period → `ServiceError::ValidationError([OverlappingPeriod(..)])`
    /// - Kein HR-Privileg → `ServiceError::Forbidden`
    /// - Row nicht gefunden / bereits soft-deleted / falsche Kategorie → `ServiceError::EntityNotFoundGeneric`
    async fn convert_extra_hours_to_absence(
        &self,
        extra_hours_id: Uuid,
        from_date: time::Date,
        to_date: time::Date,
        day_fraction: Option<crate::absence::DayFraction>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<crate::absence::AbsencePeriod, ServiceError>;
}
