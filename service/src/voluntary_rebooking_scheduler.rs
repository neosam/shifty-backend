//! F4 Auto-Cron Scheduler-Trait (Phase 56, REB-AUTO-01).
//!
//! Business-Logic-Tier per Service-Tier-Konvention: der Scheduler ist ein
//! duennes Orchestrierungs-Aggregat, das den
//! [`crate::rebooking_reconciliation::RebookingReconciliationService`]
//! (ebenfalls Business-Logic) fuer JEDE gedeckelte `SalesPerson` der letzten
//! abgeschlossenen ISO-Woche (Vorwoche) aufruft. Die eigentliche Schwerarbeit
//! (Ueberschuss-Berechnung, Idempotenz, Locked-Skip, Toggle-Gate) lebt in
//! [`RebookingReconciliationService::run_for_week`]; der Scheduler kapselt nur
//! die Cron-Schleife (`tokio-cron-scheduler`) und die Vorwochen-Arithmetik.
//!
//! Der Scheduler wird in `shifty_bin/src/main.rs` beim Boot mit
//! [`VoluntaryRebookingScheduler::start`] gestartet — NACH dem
//! `pdf_export_scheduler.start()` (konfliktfreier Cron-Slot: Montag 03:15 vs
//! PDF-Export Montag 06:00). Anders als der PDF-Export hat dieser Scheduler
//! KEIN `reload_from_db`: der Stichtag-Toggle (REB-AUTO-05) wird bei jedem Lauf
//! frisch aus dem `ToggleService` gelesen (im Motor `run_for_week`), nicht in
//! einen Cron-Ausdruck kompiliert.
//!
//! Die Cron-Loop laeuft mit [`crate::permission::Authentication::Full`]
//! (trusted interner Aufrufer, keine externe Eingabe).

use std::fmt::Debug;

use async_trait::async_trait;
use mockall::automock;

use crate::{permission::Authentication, ServiceError};

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait VoluntaryRebookingScheduler {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Beim App-Boot: registriert den woechentlichen Cron-Job (6-Feld-Ausdruck
    /// `0 15 3 * * 1` — Montag 03:15). Der Job ist ab Registrierung aktiv; ob
    /// er tatsaechlich bucht, entscheidet der Stichtag-Toggle bei jedem Lauf
    /// (F4-Self-Guard in `run_for_week`).
    async fn start(&self) -> Result<(), ServiceError>;

    /// Loest genau EINEN sofortigen Wochen-Lauf synchron aus. Wird vom
    /// Cron-Trigger (mit `Authentication::Full`) aufgerufen und ist der Ort, an
    /// dem der Scheduler direkt getestet wird (der tokio-cron-Timer selbst wird
    /// nicht in `cargo test` gefeuert).
    ///
    /// Ablauf: berechnet die letzte abgeschlossene ISO-Woche (Vorwoche) aus
    /// `ClockService::date_now()` (jahresgrenzsicherer Step-Back), laedt alle
    /// gedeckelten `SalesPerson` dieser Woche (`EmployeeWorkDetailsService::
    /// all_for_week`, `cap_planned_hours_to_expected=true`) und delegiert pro
    /// Person an [`RebookingReconciliationService::run_for_week`]. Ein Fehler
    /// fuer EINE Person wird geloggt und bricht den Gesamtlauf NICHT ab
    /// (T-56-07). Der Return-Type ist `Ok(())` ausser bei einem harten
    /// Setup-Fehler (Toggle-Read / Contract-Read).
    async fn run_once_now(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
}
