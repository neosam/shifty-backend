//! Phase 54 (VOL-STAT + VOL-ACCT): HR-Only Voluntary-Stundenkonto-Sicht
//! pro `SalesPerson` + Date-Range.
//!
//! Business-Logic-Tier-Service. Konsumiert `ExtraHoursService`,
//! `EmployeeWorkDetailsService`, `SalesPersonService`, `PermissionService`
//! und `TransactionDao` (siehe `service_impl::voluntary_stats`).
//!
//! Die Berechnung nutzt drei Range-basierte pure fns in
//! `service_impl::reporting`:
//! - `voluntary_ist_total_in_range` (F1/F2-Ist, Manual-only, Range-Cutoff)
//! - `contract_weeks_count_in_range` (F1-Nenner, D-F1-01)
//! - `committed_voluntary_target_in_range` (F2-Soll, tages-basierte D-F2-01)
//!
//! Zusaetzlich existiert `committed_voluntary_prorata_for_week` als
//! internal per-week Baustein (Debug-Tests).
//!
//! Phase 54 Gap-Closure G1: Analog `ReportingService::get_report_for_employee_range`
//! akzeptiert diese Methode eine echte Date-Range; die Aggregation
//! berücksichtigt nur Tage im Range (Pro-Rata tages-basiert). Grund: die
//! Employee-Report-Chain rechnet bis zur aktuellen KW / Range-Ende — die
//! alte `year`-Semantik ueberschoss den Report-Zeitraum um ~4x fuer den
//! 5h/Woche-Fall (177h vs realistisch ~50h fuer Jan-Juli).
//!
//! Permissionsmodell: HR-Only per API-Level-Redaktion (VOL-STAT-02, VOL-ACCT-02).
//! Non-HR-Aufrufer erhalten ein `VoluntaryStats` mit lauter `None`-Feldern
//! (Praezedenz VAC-OFFSET-01 v1.8 — kein 403).

use std::fmt::Debug;

use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use shifty_utils::ShiftyDate;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

/// HR-Only Voluntary-Stundenkonto-Sicht pro SalesPerson + Date-Range.
///
/// - F1 = `ist_per_contract_week` = `ist_total / contract_weeks`
/// - F2 = `soll_total` = `committed_voluntary_target_in_range` (tages-basierte pro-rata)
/// - Delta = `ist_total − soll_total`
///
/// Alle Felder sind `Option` — bei Non-HR-Zugriff sind alle `None`
/// (VOL-STAT-02, VOL-ACCT-02).
#[derive(Clone, Debug, PartialEq)]
pub struct VoluntaryStats {
    /// F1: Ø Freiwillig-Stunden pro Vertragswoche.
    pub ist_per_contract_week: Option<f32>,
    /// F2-Ist: absolute Summe der Manual-VolunteerWork-Hours im Range.
    pub ist_total: Option<f32>,
    /// F2-Soll: Σ pro-rata `committed_voluntary` ueber alle Tage im Range
    /// (tages-basierte D-F2-01).
    pub soll_total: Option<f32>,
    /// F2-Delta = ist_total − soll_total.
    pub delta: Option<f32>,
    /// Nenner fuer F1: Anzahl Vertragswochen mit gueltiger `EmployeeWorkDetails`-Row
    /// im Range (`expected_hours == 0` zaehlt MIT, D-F1-01).
    pub contract_weeks: Option<u32>,
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait VoluntaryStatsService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// F1+F2 kombiniert. HR-Gate im Impl: Non-HR liefert `VoluntaryStats`
    /// mit lauter `None`-Feldern (VOL-STAT-02, VOL-ACCT-02).
    ///
    /// Analog `ReportingService::get_report_for_employee_range` akzeptiert
    /// diese Methode eine echte Date-Range; die Aggregation berücksichtigt
    /// nur Tage im Range (Pro-Rata tages-basiert). Edge-Weeks tragen
    /// tages-genau bei.
    async fn get_voluntary_stats(
        &self,
        sales_person_id: Uuid,
        from_date: ShiftyDate,
        to_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<VoluntaryStats, ServiceError>;
}
