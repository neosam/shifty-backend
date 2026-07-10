//! Phase 54 (VOL-STAT + VOL-ACCT): HR-Only Voluntary-Stundenkonto-Sicht
//! pro `SalesPerson` + Date-Range.
//!
//! Business-Logic-Tier-Service. Konsumiert `ReportingService` (fuer das
//! Ist-Aggregat), `EmployeeWorkDetailsService`, `SalesPersonService`,
//! `PermissionService` und `TransactionDao` (siehe
//! `service_impl::voluntary_stats`).
//!
//! Aggregations-Modell (Gap-Closure 54-09-Ist-Fix):
//! - Ist (F1/F2-Ist): `EmployeeReport::volunteer_hours` aus
//!   `ReportingService::get_report_for_employee_range` — deckt manuelle
//!   VolunteerWork-ExtraHours PLUS Shiftplan-Cap-Ueberlauf PLUS
//!   no_contract-Shiftplan-Stunden konsistent zum OVERALL-"Ehrenamt"-Wert
//!   der Employee-Detail-Seite ab.
//! - Soll (F2-Soll): `committed_voluntary_target_in_range` — tages-basierte
//!   Pro-Rata (D-F2-01) ueber `EmployeeWorkDetails`.
//! - Contract-weeks-Nenner: `contract_weeks_count_in_range` (D-F1-01,
//!   expected_hours=0 zaehlt MIT).
//!
//! `committed_voluntary_prorata_for_week` existiert weiterhin als
//! internal per-week Baustein (Debug-Tests).
//!
//! Phase 54 Gap-Closure G1: Analog `ReportingService::get_report_for_employee_range`
//! akzeptiert diese Methode eine echte Date-Range; die Aggregation
//! berücksichtigt nur Tage im Range (Pro-Rata tages-basiert). Grund: die
//! Employee-Report-Chain rechnet bis zur aktuellen KW / Range-Ende — die
//! alte `year`-Semantik ueberschoss den Report-Zeitraum um ~4x fuer den
//! 5h/Woche-Fall (177h vs realistisch ~50h fuer Jan-Juli).
//!
//! Rebooking-Neutralitaets-Filter (`source == 'manual'`) ist in Phase 54
//! nicht aktiv; er wird ab Phase 55 zentral im `ReportingService` greifen
//! und fliesst dann automatisch in dieses Aggregat.
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
    /// Erfuellungsgrad `ist_total / soll_total * 100` in Prozent. `None`, wenn
    /// `soll_total` ~= 0 (Nicht-Freiwillige oder Range komplett in
    /// Absence-Wochen). Bei Ist > Soll >100%.
    pub ist_per_soll_pct: Option<f32>,
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
