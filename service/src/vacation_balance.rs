//! Vacation-Balance-Domain für Phase 8.
//!
//! Stellt das Service-Trait [`VacationBalanceService`] sowie die Domain-
//! Struktur [`VacationBalance`] für den Resturlaubs-Endpoint bereit, der
//! die Frontend-Komponenten `VacationEntitlementCard` und
//! `VacationPerPersonList` (UI-SPEC Phase 8) befeuert.
//!
//! Tier-Klassifizierung: **Business-Logic-Service** (D-04 in
//! `08-CONTEXT.md`). Der Service kombiniert Cross-Entity-Daten
//! (`EmployeeWorkDetailsService.vacation_days_for_year`,
//! `CarryoverService.get_carryover().vacation`,
//! `AbsenceService.find_by_sales_person`) zu einem Resturlaubs-Aggregat
//! pro Mitarbeiter und Jahr. Die konkrete Service-Impl entsteht in
//! Plan 08-02; dieser Plan (08-01) liefert ausschließlich Trait + Domain
//! + DTO als Foundation, gegen die Plan 08-02 baut.
//!
//! Permissionsmodell:
//! - `get(sales_person_id, year, ...)`: HR ∨ self
//!   (analog `AbsenceService::find_by_sales_person`, D-04 + D-09 in
//!   `08-CONTEXT.md`).
//! - `get_team(year, ...)`: HR-only (Aggregatsicht über alle bezahlten
//!   Mitarbeiter — Frontend `VacationPerPersonList`).
//!
//! `automock` erzeugt `MockVacationBalanceService` für Plan 08-02-Tests.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::{permission::Authentication, ServiceError};

/// Resturlaubs-Aggregat eines Mitarbeiters für ein konkretes Kalenderjahr.
///
/// Felder korrespondieren 1:1 mit den fünf Stats-Kacheln in der UI-SPEC
/// `VacationEntitlementCard` (i18n-Keys `VacationStatContract`,
/// `VacationStatCarryover`, `VacationStatUsed`, `VacationStatPending`,
/// `VacationStatRemaining`).
///
/// Berechnung (Plan 08-02 implementiert):
/// `remaining_days = entitled_days + carryover_days − (used_days + planned_days)`.
#[derive(Clone, Debug, PartialEq)]
pub struct VacationBalance {
    /// Eindeutiger Bezug auf den Mitarbeiter (`SalesPerson.id`).
    pub sales_person_id: Uuid,
    /// Kalenderjahr (4-stellig, z. B. 2026), für das der Resturlaub
    /// berechnet wird.
    pub year: u32,
    /// Vertragsanspruch in Tagen — Quelle:
    /// `EmployeeWorkDetailsService::vacation_days_for_year` (aliquot über
    /// alle Vertragsabschnitte des Jahres). Anteilig in Tagen,
    /// daher `f32`.
    pub entitled_days: f32,
    /// Übertrag aus dem Vorjahr in Tagen — Quelle:
    /// `CarryoverService::get_carryover(sales_person_id, year).vacation`
    /// (Domain-Wert ist ganzzahlig in Tagen).
    pub carryover_days: i32,
    /// Bereits genommene Vacation-Tage in `year` — Summe aller aktiven
    /// `AbsencePeriod`s der Kategorie `Vacation` mit `to_date < today`,
    /// gewichtet mit Vertragsstunden pro Tag (Backend liefert Tag-
    /// Äquivalent als `f32`, daher mögliche Halbtage durch
    /// Special-Days/Conflict-Resolve).
    pub used_days: f32,
    /// Beantragte aber noch zukünftige Vacation-Tage in `year` — Summe
    /// aller aktiven `AbsencePeriod`s der Kategorie `Vacation` mit
    /// `from_date >= today`. Gleiche Tag-Äquivalent-Logik wie
    /// `used_days`.
    pub planned_days: f32,
    /// Verbleibende Tage =
    /// `entitled_days + carryover_days − (used_days + planned_days)`.
    /// Wird vom Service berechnet, um Frontend-Drift zu vermeiden.
    pub remaining_days: f32,
    /// HR-only Breakdown (D-28-03): der angewendete signierte Offset in
    /// ganzen Tagen. Der *effektive* Wert (`round(base) + offset`) steckt
    /// IMMER in [`entitled_days`]; dieses Feld ist `Some(n)` nur für
    /// HR-Aufrufer und `None` für self-only-Aufrufer (Server-seitiges
    /// API-Hiding, niemals nur im Frontend).
    pub offset_days: Option<i32>,
    /// HR-only Breakdown (D-28-03): der gerundete Vertragsanspruch VOR
    /// Offset-Korrektur (`round(Σ vacation_days_for_year)`). `Some(..)` nur
    /// für HR-Aufrufer, `None` für self-only — analog [`offset_days`].
    pub computed_entitled_days: Option<f32>,
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait VacationBalanceService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Liefert das Resturlaubs-Aggregat für genau einen Mitarbeiter und
    /// ein Kalenderjahr.
    ///
    /// Permission: HR ∨ self (`verify_user_is_sales_person`) — analog
    /// `AbsenceService::find_by_sales_person` (D-04 in
    /// `08-CONTEXT.md`).
    async fn get(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<VacationBalance, ServiceError>;

    /// Liefert das Resturlaubs-Aggregat für ALLE bezahlten Mitarbeiter
    /// eines Kalenderjahrs. Speist die Frontend-`VacationPerPersonList`
    /// (HR-Übersicht aus UI-SPEC Phase 8).
    ///
    /// Permission: HR-only (kein self-Override — die Liste enthält
    /// fremde Mitarbeiter).
    async fn get_team(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[VacationBalance]>, ServiceError>;
}
