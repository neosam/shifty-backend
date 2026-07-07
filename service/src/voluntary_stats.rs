//! Phase 54 (VOL-STAT + VOL-ACCT): HR-Only Voluntary-Stundenkonto-Sicht
//! pro `SalesPerson` + ISO-Jahr.
//!
//! Business-Logic-Tier-Service. Konsumiert `ExtraHoursService`,
//! `EmployeeWorkDetailsService`, `SalesPersonService`, `PermissionService`
//! und `TransactionDao` (siehe `service_impl::voluntary_stats`).
//!
//! Die Berechnung nutzt vier pure fns in `service_impl::reporting`:
//! - `voluntary_ist_total_for_year` (F2-Ist / F1-Ist Zaehler, Manual-only)
//! - `contract_weeks_count` (F1-Nenner, D-F1-01)
//! - `committed_voluntary_prorata_for_week` (D-F2-01 Tages-Prorata)
//! - `committed_voluntary_target_for_year` (F2-Soll Summe)
//!
//! Permissionsmodell: HR-Only per API-Level-Redaktion (VOL-STAT-02, VOL-ACCT-02).
//! Non-HR-Aufrufer erhalten ein `VoluntaryStats` mit lauter `None`-Feldern
//! (Praezedenz VAC-OFFSET-01 v1.8 — kein 403).

use std::fmt::Debug;

use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

/// HR-Only Voluntary-Stundenkonto-Sicht pro SalesPerson + ISO-Jahr.
///
/// - F1 = `ist_per_contract_week` = `ist_total / contract_weeks`
/// - F2 = `soll_total` = `committed_voluntary_target_for_year` (pro-rata)
/// - Delta = `ist_total − soll_total`
///
/// Alle Felder sind `Option` — bei Non-HR-Zugriff sind alle `None`
/// (VOL-STAT-02, VOL-ACCT-02).
#[derive(Clone, Debug, PartialEq)]
pub struct VoluntaryStats {
    /// F1: Ø Freiwillig-Stunden pro Vertragswoche.
    pub ist_per_contract_week: Option<f32>,
    /// F2-Ist: absolute Summe der Manual-VolunteerWork-Hours im Jahr.
    pub ist_total: Option<f32>,
    /// F2-Soll: Σ pro-rata `committed_voluntary` ueber alle ISO-Wochen des Jahres.
    pub soll_total: Option<f32>,
    /// F2-Delta = ist_total − soll_total.
    pub delta: Option<f32>,
    /// Nenner fuer F1: Anzahl Vertragswochen mit gueltiger `EmployeeWorkDetails`-Row
    /// (`expected_hours == 0` zaehlt MIT, D-F1-01).
    pub contract_weeks: Option<u32>,
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait VoluntaryStatsService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// F1+F2 kombiniert. HR-Gate im Impl: Non-HR liefert `VoluntaryStats`
    /// mit lauter `None`-Feldern (VOL-STAT-02, VOL-ACCT-02).
    async fn get_voluntary_stats(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<VoluntaryStats, ServiceError>;
}
