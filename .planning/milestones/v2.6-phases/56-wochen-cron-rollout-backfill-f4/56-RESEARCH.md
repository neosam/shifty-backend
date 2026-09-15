# Phase 56: Wochen-Cron + Rollout-Backfill (F4) — Research

**Researched:** 2026-07-26
**Domain:** Rust async scheduler (tokio-cron-scheduler), ISO-week arithmetic, Stichtag-Toggle chain-audit, idempotent write path, Backfill REST endpoint, Admin frontend card
**Confidence:** HIGH — all findings verified against live codebase (Phase 54/55 artifacts in-repo, Phase 48 scheduler in-repo, Phase 51 shortday_gate in-repo)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01 (D-F4-SNAPSHOT-01):** Bei Snapshot-Version 12 bleiben — KEIN Bump auf 13. Begründung: Rebooking ist balance-neutral (`−N` + `+N` in derselben Person/Woche), der Stichtag-Guard schützt die Historie, ein vorsorglicher Bump ohne neuen `value_type` würde jede historische Snapshot-Validierung ohne inhaltlichen Grund invalidieren.
  - **D-01-BEWEISLAST (verbindlich):** Straddling-Golden-Snapshot-Fixture (Muster v2.5 Phase 52 WOP-03, 8 Fixtures) — Billing-Period strikt VOR, strikt NACH und TRAVERSIEREND über `active_from` muss byte-identisch bleiben. Wenn NICHT byte-identisch → Bump 12→13.

- **D-02 (D-F4-STALE-01):** Claim-on-Suggest. Pending HR-Suggestion belegt UNIQUE-Slot sofort (`rebooking_batch(kind=hr_suggestion, state=pending)`); Cron skipt via `ON CONFLICT DO NOTHING`. Kein separater Fingerprint-/409-Mechanismus.

- **D-03 (D-F4-CRON-01):** Montag 03:00, 6-Feld-Cron-Syntax (`0 0 3 * * 1` — Sekunden-Feld voran, analog v2.2 EXP + v2.3.1-Hotfix). Verarbeitet Vorwoche via `ShiftyDate::from_date(today).as_shifty_week().previous()` analog v2.5-`_iso_year`-Helper. Boot am Ende von `main()` nach `pdf_export_scheduler.start()`.

- **D-04 (D-F4-LOCKED-01):** Skip + Audit-Row. Bei `WeekStatus=Locked` überspringt Cron die Woche und persistiert `rebooking_batch(state='skipped_locked')` als Audit-Spur. State-Enum erweitern. Kein Crash, kein hartes Fehler-Log.

- **D-05 (D-F4-BACKFILL-01):** JSON pro Batch-Kandidat mit Preview-Werten (Ist / Soll / committed_voluntary / Excess pro SalesPerson+Woche) + Pagination für mehrjährige Backfills. `dry_run=true` → kein DB-Write; `dry_run=false` → persistiert `rebooking_batch(kind=auto_cron_backfill)`. HR-Admin-Gate. TOs: `BackfillRequestTO`, `BackfillPreviewTO`.

- **D-06 (D-F4-CHAIN-01):** Pro-Chain-Rekonstruktion analog HCFG-02 (v1.7) / SHC-04 (v2.4). Jede Konsumkette (Balance `reporting.rs`, F1-Ist, F2-Soll-Nutzung, F4-Self-Guard, `billing_period.value_type`, `booking_information::get_weekly_summary`) bekommt explizite pre-Stichtag-Legacy-Semantik bei `active_from=None` + Golden-Snapshot-Regression. Drei Neutralitäts-Property-Tests.

### Claude's Discretion

- Exakte Boot-Minute des Cron (nur Konfliktvermeidung mit `pdf_export` bei 03:00 — Empfehlung: 03:15 oder 04:00).
- Interne Struktur der ISO-Wochen-Arithmetik (v2.5-Helper wiederverwenden).
- `state`-Enum-Benennung (`skipped_locked` als Vorschlag).

### Deferred Ideas (OUT OF SCOPE)

None — alle 6 Discuss-Points (D-F4-SNAPSHOT/STALE/CRON/LOCKED/BACKFILL/CHAIN) sind entschieden.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REB-AUTO-01 | Wochen-Cron `VoluntaryRebookingScheduler` läuft montags früh, verarbeitet Vorwoche (ISO-Woche via v2.5 `_iso_year`-Helper), schreibt für gedeckelte SalesPersons mit Excess `rebooking_batch(kind=auto_cron)` | Scheduler-Muster: `pdf_export_scheduler.rs`; ISO-Woche: `ShiftyWeek::next()` + `ShiftyDate::from_date()` |
| REB-AUTO-02 | Admin-Toggle `voluntary_rebooking_auto_active_from` schützt historische Balance-Views; `None`=deaktiviert; mit Datum wirkt Rebooking nur für ISO-Wochen ≥ `active_from`; Legacy-Semantik pro Konsumkette | Toggle-Muster: `shortday_gate.rs` (SHC-04) + `reporting.rs` HCFG-02 Stichtag |
| REB-AUTO-03 | Cron ist idempotent via UNIQUE-Constraint `(sales_person_id, iso_year, iso_week) WHERE deleted IS NULL` + `INSERT ON CONFLICT DO NOTHING`; zweiter Lauf = no-op; Claim-on-Suggest hält | Migration `20260707000000_create-rebooking-batch.sql` line 42-44 |
| REB-AUTO-04 | `POST /admin/rebooking/backfill?from=YYYY-Www&dry_run={true/false}`; `dry_run=true` zeigt Preview ohne DB-Write; `dry_run=false` persistiert `kind=auto_cron_backfill`; HR-Admin-gated; keine CLI | REST-Muster: `rest/src/rebooking.rs` (Phase 55); TOs in `rest-types/src/rebooking.rs` |
| REB-AUTO-05 | Snapshot-Schema-Version-Entscheidung (12 bleibt, D-01); Straddling-Golden-Snapshot-Fixture als Beweislast | Fixture-Muster: `booking_information_weekly_summary_year_batch.rs` (WOP-03, 8 Fixtures) |

</phase_requirements>

---

## Summary

Phase 56 schließt den v2.6-Milestone ab: ein Wochen-Cron (`VoluntaryRebookingScheduler`) spiegelt `PdfExportSchedulerImpl` 1:1, ein Admin-Toggle `voluntary_rebooking_auto_active_from` schützt historische Daten über einen per-Chain-Guard, und ein REST-Backfill-Endpoint mit Dry-Run ermöglicht den einmaligen rückwirkenden Rollout.

Alle sechs Discuss-Points sind entschieden. Die Kernaufgaben sind: (1) Scheduler-Trait + Impl von `pdf_export_scheduler.rs` kopieren und anpassen; (2) `run_for_week` + `backfill_range` in `rebooking_reconciliation.rs` ergänzen; (3) Stichtag-Toggle per `shortday_gate.rs`-Muster in alle ExtraHours-lesenden Chains injizieren; (4) Straddling-Golden-Snapshot-Fixture schreiben, die D-01-Beweislast erbringt; (5) Admin-Card im FE analog `pdf_export_config`-Card + Dioxus.toml-Proxy.

Das größte Implementierungsrisiko ist das Chain-Audit D-06: es gibt sechs separate Konsumketten, die alle den Toggle-Check erhalten müssen — wird eine vergessen, verletzt das die Golden-Snapshot-Regression.

**Primary recommendation:** Strikt nach dem PdfExportScheduler-Blueprint bauen, keine Neuerfindung. ISO-Woche immer via `ShiftyDate::from_date(date).as_shifty_week()` → `week.year` und `week.week` — NIE `date.year()` + `date.iso_week()` direkt.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Wochen-Cron-Trigger | Backend (BL-Service) | — | Scheduler orchestriert RebookingReconciliationService; Cron-Loop = tokio async task |
| Stichtag-Toggle lesen | Backend (BL-Service) | — | ToggleService ist Basic-Tier; jeder BL-Konsument liest selbst analog shortday_gate |
| ISO-Wochen-Arithmetik (Vorwoche) | Backend (BL-Service Cron) | — | ShiftyWeek/ShiftyDate im Backend; FE nie |
| Excess-Berechnung (Ist > Soll + committed_voluntary) | Backend (RebookingReconciliation) | — | Fat Backend / Thin Client |
| Backfill-Endpoint | API (REST, HR-Admin-gated) | — | Wiederverwendet Auth + Audit-Trail; kein CLI |
| Dry-Run-Preview-JSON | API (REST) | — | Backend berechnet BackfillPreviewTO; FE zeigt nur |
| UNIQUE-Constraint-Idempotenz | Database (SQLite index) | Backend (ON CONFLICT) | Phase 54 UNIQUE-Index ist das Gate; DAO mapped Conflict → EntityAlreadyExists |
| Admin-Card (Toggle-Editor + Backfill-Button) | Frontend (Dioxus settings page) | — | Analog pdf_export_config-Card; FE zeigt pre-computed DTO |
| i18n Labels | Frontend | — | 3 Sprachen (de/en/cs) |

---

## Standard Stack

### Core — Verified in-repo
| Library | Version (Cargo.toml) | Purpose | Source |
|---------|---------------------|---------|--------|
| `tokio-cron-scheduler` | 0.15.x (Phase 48 EXP) | Cron-Loop + Job-Registration | [VERIFIED: Cargo.lock] |
| `async-trait` | existing | Trait-Impl für async fns | [VERIFIED: service_impl/Cargo.toml] |
| `tokio` | existing | async runtime | [VERIFIED: Cargo.toml] |
| `time` | existing | ISO-Wochen-Arithmetik | [VERIFIED: shifty-utils/src/date_utils.rs] |
| `shifty-utils` | workspace | ShiftyDate, ShiftyWeek, DateRange | [VERIFIED: codebase] |
| `sqlx` | existing | ON CONFLICT DO NOTHING in DAO | [VERIFIED: dao_impl_sqlite] |

### Existing Services — reuse, no new deps
| Service | Location | Reuse |
|---------|----------|-------|
| `RebookingReconciliationService` | `service_impl/src/rebooking_reconciliation.rs` | Ergänze `run_for_week` + `backfill_range` |
| `RebookingBatchService` | `service_impl/src/rebooking_batch.rs` | Vorhanden (Phase 54); UNIQUE-Constraint nutzen |
| `WeekStatusService` | `service_impl/src/week_status.rs` | `get_week_status(year, week)` → prüfe `WeekStatus::Locked` |
| `ToggleService` | `service/src/toggle.rs` | `get_toggle_value("voluntary_rebooking_auto_active_from", Auth::Full, None)` |
| `VoluntaryStatsService` | `service_impl/src/voluntary_stats.rs` | committed_voluntary-Wert pro Woche |
| `ReportingService` | `service_impl/src/reporting.rs` | balance + F1-Ist per Woche |
| `SalesPersonService` | existing | alle gedeckelten SalesPersons |

**No new Cargo dependencies required for this phase.** [VERIFIED: codebase scan]

---

## Architecture Patterns

### System Architecture Diagram

```
                     ┌──────────────────────────────────┐
                     │           main()                  │
                     │  scheduler_service.start()        │
                     │  pdf_export_scheduler.start()     │
                     │  voluntary_rebooking_scheduler.start() ← NEW
                     └──────────────────────────────────┘
                                    │ Montag 03:15, tokio_cron_scheduler
                                    ▼
                    ┌───────────────────────────────┐
                    │  VoluntaryRebookingScheduler  │
                    │  run_once_now(Auth::Full)     │
                    │  1. read toggle (active_from) │
                    │  2. compute Vorwoche          │
                    │  3. load capped SalesPersons  │
                    │  4. per-SP: check Excess      │
                    │  5. check WeekStatus::Locked  │
                    └──────────┬────────────────────┘
                               │ run_for_week(sp, year, week)
                               ▼
            ┌──────────────────────────────────────────┐
            │  RebookingReconciliationService           │
            │  run_for_week(sp, year, week, kind)       │
            │  - Slot-Check: ON CONFLICT → skip (D-02)  │
            │  - Locked → skipped_locked batch (D-04)   │
            │  - Excess → rebook() → Pair-ExtraHours    │
            │  backfill_range(from, to, dry_run)        │
            └──────────────────────────────────────────┘
                               │
                    ┌──────────┴──────────┐
                    ▼                     ▼
           ┌──────────────┐     ┌──────────────────┐
           │ RebookingBatch│     │ ExtraHoursService │
           │ DAO (UNIQUE)  │     │ −N/+N source=     │
           │ INSERT OR     │     │ Rebooking rows    │
           │ CONFLICT      │     └──────────────────┘
           └──────────────┘

    REST:  POST /admin/rebooking/backfill?from=YYYY-Www&dry_run={true|false}
           HR-Admin-gated → RebookingReconciliationService.backfill_range()

    FE:    settings.rs → Admin-Card (Toggle-Editor + Backfill-Button + Preview)
           Dioxus.toml → [[web.proxy]] backend="http://localhost:3000/admin/rebooking"
```

### Recommended Project Structure (new files only)

```
service/src/
└── voluntary_rebooking_scheduler.rs   # Trait (1:1 PdfExportScheduler)

service_impl/src/
└── voluntary_rebooking_scheduler.rs   # Impl (1:1 PdfExportSchedulerImpl, stripped)
    rebooking_reconciliation.rs        # + run_for_week(), backfill_range()
    voluntary_rebooking_gate.rs        # Analog shortday_gate.rs (NEU — chain-guard)

rest/src/
└── rebooking.rs                       # + POST /admin/rebooking/backfill handler

rest-types/src/
└── rebooking.rs                       # + BackfillRequestTO, BackfillPreviewTO

shifty_bin/src/
└── main.rs                            # DI-Wiring + .start() am Ende

shifty-dioxus/src/page/
└── settings.rs                        # neue Admin-Card unter "Automatische Umbuchung"
shifty-dioxus/Dioxus.toml             # + [[web.proxy]] /admin/rebooking
shifty-dioxus/i18n/{de,en,cs}/*.ftl   # Card-Labels, Backfill-Button, Preview-Header
```

---

### Pattern 1: Scheduler Trait (1:1-Spiegel von `PdfExportScheduler`)

**What:** Service-Trait mit `start()`, `run_once_now()` — kein `reload_from_db()` nötig (Toggle hat keinen Reload-Trigger).

**Source:** `service/src/pdf_export.rs` [VERIFIED: in-repo line 27-52]

```rust
// service/src/voluntary_rebooking_scheduler.rs
#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait VoluntaryRebookingScheduler {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Boot: registriert den Cron-Job. Falls Toggle deaktiviert, startet
    /// Scheduler dormant (kein Cron-Job registriert).
    async fn start(&self) -> Result<(), ServiceError>;

    /// Einzelner Lauf: verarbeitet letzte abgeschlossene ISO-Woche für alle
    /// gedeckelten SalesPersons mit Excess. Authentication::Full vom Cron.
    async fn run_once_now(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
}
```

---

### Pattern 2: Scheduler Impl — gen_service_impl! + tokio_cron_scheduler

**What:** Deps-Struct via `gen_service_impl!` macro, Custom-Fields `scheduler` + `current_job`, Boot via `start()`.

**Source:** `service_impl/src/pdf_export_scheduler.rs` [VERIFIED: in-repo line 62-85, 115-141, 181-215]

```rust
// service_impl/src/voluntary_rebooking_scheduler.rs
gen_service_impl! {
    struct VoluntaryRebookingSchedulerImpl: VoluntaryRebookingScheduler = VoluntaryRebookingSchedulerDeps {
        RebookingReconciliationService: RebookingReconciliationService<...> = rebooking_reconciliation_service,
        SalesPersonService: SalesPersonService<...> = sales_person_service,
        ToggleService: ToggleService<...> = toggle_service,
        WeekStatusService: WeekStatusService<...> = week_status_service,
        ClockService: ClockService = clock_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
    ; custom_fields {
        scheduler: Arc<Mutex<Option<JobScheduler>>> = scheduler,
        current_job: Arc<Mutex<Option<Uuid>>> = current_job,
    }
}
```

**Kritisch:** `clone_for_job()` Helfer (kopiiert alle Arc-Handles in einen leichten Handle-Struct — vermeidet dass `self` in Closure gemoved wird). [VERIFIED: pdf_export_scheduler.rs line 476-493]

---

### Pattern 3: 6-Feld-Cron-Syntax (Pflicht)

**Was:** `tokio_cron_scheduler` 0.15 nutzt `croner 3.0` mit `Seconds::Required` — also 6-Feld-Format `sec min hour dom mon dow`.

**Pitfall 16 (CRITICAL):** v2.3.1-Hotfix war nötig, weil Phase 48 mit 5-Feld geseedet hat (`0 6 * * 1`). Hotfix-Migration `20260704000000_fix-pdf-export-cron-6-field.sql` korrigierte es auf 6-Feld (`'0 ' || cron_schedule`). [VERIFIED: migrations/sqlite/20260704000000_fix-pdf-export-cron-6-field.sql]

**Korrektes Format für Montag 03:15:**
```
"0 15 3 * * 1"
 ^  ^  ^ ^ ^ ^
 |  |  | | | └── Wochentag (1=Montag)
 |  |  | | └──── Monat (*)
 |  |  | └────── Dom (*)
 |  |  └──────── Stunde (3)
 |  └──────────── Minute (15)
 └──────────────── Sekunde (0)
```

**KEIN Konflikt mit pdf_export:** pdf_export läuft Montag 06:00 (`0 0 6 * * 1`). Empfehlung: 03:15 oder 04:00 für VoluntaryRebookingScheduler (Claude's Discretion).

---

### Pattern 4: ISO-Wochen-Arithmetik für "letzte abgeschlossene Vorwoche"

**Was:** `ShiftyDate::from_date(today)` → ShiftyDate mit korrektem ISO-Jahr via `to_iso_week_date()`. Dann `as_shifty_week()` → `ShiftyWeek`. Vorwoche = `ShiftyWeek::new(week.year, week.week - 1)` — aber korrekt über den Jahresübergang.

**Source:** `shifty-utils/src/date_utils.rs` [VERIFIED: in-repo]

```rust
// In run_once_now():
let today = self.clock_service.date_now();
let today_shifty = ShiftyDate::from_date(today);  // nutzt to_iso_week_date() intern
let current_week = today_shifty.as_shifty_week();
// Vorwoche: ShiftyWeek::new() normalisiert Woche 0 → letztes Jahr, Woche 52/53
let prev_week = if current_week.week == 1 {
    ShiftyWeek::new(current_week.year - 1,
        time::util::weeks_in_year(current_week.year as i32 - 1))
} else {
    ShiftyWeek::new(current_week.year, current_week.week - 1)
};
// Dann: iso_year = prev_week.year, iso_week = prev_week.week
```

**KRITISCH (Pitfall 5):** NIEMALS `today.year()` + `today.iso_week()` — das gibt bei Jan 1-3 falsches Jahr. Immer `ShiftyDate::from_date(today)` → ISO-Jahr via `ShiftyDate::year()`. [VERIFIED: Pitfall-5-Analyse + v2.5-WOP-Follow-up-3-Precedent]

**Jahresgrenz-Test:** Cron läuft Mo 2027-01-04 → verarbeitet ISO-Woche (2026, 53). Cron läuft Mo 2027-01-11 → verarbeitet (2027, 1). [CITED: PITFALLS.md §Pitfall 5]

---

### Pattern 5: Stichtag-Toggle-Guard (Chain-Audit D-06)

**Was:** Neues Modul `voluntary_rebooking_gate.rs` analog zu `shortday_gate.rs`. [VERIFIED: service_impl/src/shortday_gate.rs]

```rust
// service_impl/src/voluntary_rebooking_gate.rs
pub const TOGGLE_NAME: &str = "voluntary_rebooking_auto_active_from";

pub fn parse_active_from(raw: Option<&str>) -> Option<time::Date> {
    let s = raw?;
    if s.is_empty() { return None; }
    time::Date::parse(s, &time::format_description::well_known::Iso8601::DEFAULT).ok()
}

/// Gibt true zurück wenn die ISO-Woche mit active_from >= Stichtag beginnt.
/// active_from=None → immer false (Feature dormant).
pub fn should_run_for_week(
    iso_year: u32, iso_week: u8,
    active_from: Option<time::Date>,
) -> bool {
    match active_from {
        None => false,
        Some(cutoff) => {
            // Montag der ISO-Woche als Anker
            let monday = ShiftyDate::new(iso_year, iso_week, DayOfWeek::Monday)
                .ok()
                .map(|d| d.to_date());
            monday.map(|d| d >= cutoff).unwrap_or(false)
        }
    }
}

/// Liest den Toggle, toleriert Unauthorized → None (Feature dormant).
pub async fn read_active_from<T: ToggleService>(
    toggle_service: &T,
    context: Authentication<T::Context>,
) -> Result<Option<time::Date>, ServiceError> { /* ... */ }
```

**Die sechs Konsumketten (D-06 Chain-Audit-Matrix):**

| Chain | Datei | Wo Guard einfügen | Legacy-Semantik (active_from=None) |
|-------|-------|-------------------|-------------------------------------|
| Balance/reporting.rs | `service_impl/src/reporting.rs` | In `get_report_for_employee` + `get_report_for_employee_range` — filter `source==Rebooking` ExtraHours wenn `!should_run_for_week(week, active_from)` | Rebooking-ExtraHours für pre-Stichtag-Wochen = unsichtbar |
| F1-Ist | `service_impl/src/voluntary_stats.rs` | `get_voluntary_stats` — `source==Manual` filter bleibt (Phase 55 D-55-A1); zusätzlich Rebooking-ExtraHours für pre-Stichtag-Wochen = unsichtbar | Identisch Phase 55-Baseline |
| F2-Soll-Nutzung | `service_impl/src/voluntary_stats.rs` | `committed_voluntary_target_in_range` — Soll-Seite ist von ExtraHours unabhängig; guard nur nötig wenn Soll future-Wochen beeinflusst | Kein Guard nötig (Soll basiert nicht auf ExtraHours) |
| F4-Self-Guard | `service_impl/src/voluntary_rebooking_scheduler.rs` | Cron-Entry: `if !should_run_for_week(week, active_from) { skip }` | Cron läuft nie ohne Stichtag |
| billing_period.value_type | `service_impl/src/billing_period_report.rs` | `build_and_persist_billing_period_report` liest ExtraHours direkt — Rebooking-ExtraHours per `source` filter ausschließen ODER via Stichtag — D-01 Entscheidung: da balance-neutral (−N+N), ändert sich `value_type`-Summe nicht; kein separater Guard nötig | Byte-identisch (balance-neutral) |
| booking_information | `service_impl/src/booking_information.rs` | `get_weekly_summary` — `source==Rebooking` filter analog Phase 55 — Rebooking-ExtraHours sind bereits durch `source=Rebooking` von F1-Aggregation separiert | Identisch Phase 55-Baseline |

**WICHTIG:** `read_active_from` in Chains, die keinen User-Context haben (billing_period, shiftplan_report), MUSS `Authentication::Full` verwenden — analog `shortday_gate::read_active_from` mit Unauthorized-Toleranz. [VERIFIED: shortday_gate.rs line 73-99]

---

### Pattern 6: UNIQUE-Constraint + ON CONFLICT DO NOTHING (Idempotenz)

**Was:** Der UNIQUE-Partial-Index aus Phase 54 (`rebooking_batch_week_unique_idx`) ist das einzige Idempotenz-Gate.

**Source:** `migrations/sqlite/20260707000000_create-rebooking-batch.sql` [VERIFIED: line 42-44]

```sql
CREATE UNIQUE INDEX IF NOT EXISTS rebooking_batch_week_unique_idx
    ON rebooking_batch (sales_person_id, iso_year, iso_week)
    WHERE deleted IS NULL;
```

**DAO-Muster:** [VERIFIED: dao_impl_sqlite/src/rebooking_batch.rs line 162-163, 368, 421-442]
```rust
fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db_err) if db_err.is_unique_violation())
}
// Bei UNIQUE-Conflict → DaoError::EntityAlreadyExists
// Im Service → ServiceError::EntityAlreadyExists (→ Cron skipt, kein Crash)
```

**Cron-Idempotenz-Flow:**
1. Cron versucht `create(&batch, kind=auto_cron)`
2. UNIQUE-Konflikt → `DaoError::EntityAlreadyExists` → Service fängt → no-op (continue loop)
3. Claim-on-Suggest (D-02): HR-Suggestion mit `state=pending` belegt denselben Slot → Cron skipt ebenfalls

---

### Pattern 7: `run_for_week` + `backfill_range` in RebookingReconciliationService

**Was:** Neue Methoden im bestehenden `RebookingReconciliationService` (Phase 55 Trait + Impl). Teilen den internen `rebook()`-Pair-Write-Path (Pitfall 15 — single write-path für alle Trigger). [VERIFIED: service_impl/src/rebooking_reconciliation.rs line 82-129]

```rust
// Trait-Erweiterung (service/src/rebooking_reconciliation.rs):

/// F4 Cron-Trigger: verarbeitet eine ISO-Woche für einen SalesPerson.
/// Interne Methode — wird vom Scheduler pro (SP, Woche) aufgerufen.
/// ON CONFLICT DO NOTHING via RebookingBatchService.
/// Locked-Woche → skipped_locked-Batch.
async fn run_for_week(
    &self,
    sales_person_id: Uuid,
    iso_year: u32,
    iso_week: u8,
    kind: RebookingBatchKind,   // AutoCron oder AutoCronBackfill
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<RunForWeekOutcome, ServiceError>;

/// F4 Backfill: iteriert alle ISO-Wochen von `from` bis letzte abgeschlossene Vorwoche.
/// dry_run=true: berechnet Previews ohne DB-Write.
/// dry_run=false: persistiert AutoCronBackfill-Batches.
async fn backfill_range(
    &self,
    from: ShiftyWeek,
    dry_run: bool,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Vec<BackfillPreviewItem>, ServiceError>;

pub enum RunForWeekOutcome {
    Booked { hours: f32 },
    Skipped { reason: SkipReason },
}
pub enum SkipReason {
    NoExcess,
    AlreadyProcessed,   // UNIQUE-Konflikt
    WeekLocked,         // → skipped_locked Batch
    ToggleOff,          // active_from = None oder Woche vor Stichtag
}
```

---

### Pattern 8: Straddling-Golden-Snapshot-Fixture (D-01-Beweislast)

**Was:** Analog WOP-03 8-Fixtures-Muster. Eine Billing-Period wird DREIMAL gerechnet mit dem gleichen Dataset aber unterschiedlichem `active_from`:

1. **Pre-Stichtag:** `active_from = Some(next_month)` → Rebooking-ExtraHours unsichtbar → Snapshot = v2.5-Baseline
2. **Post-Stichtag:** `active_from = Some(last_month)` → Rebooking-ExtraHours sichtbar → Snapshot muss balance-neutral bleiben
3. **Straddling:** `active_from = Some(today)` → hälfte der Period pre, Hälfte post → Snapshot muss konsistent bleiben

**Source-Fixture-Muster:** `service_impl/src/test/booking_information_weekly_summary_year_batch.rs` [VERIFIED: in-repo]

```rust
// Byte-identity assertion (WOP-03-Muster):
fn assert_f32_bit_eq(actual: f32, expected: f32, field: &str, idx: usize) {
    assert!(
        !actual.is_nan(),
        "field `{field}` in billing_period[{idx}] is NaN"
    );
    assert_eq!(
        actual.to_bits(), expected.to_bits(),
        "field `{field}` diverges: actual={actual} (0x{:08x}), expected={expected} (0x{:08x})",
        actual.to_bits(), expected.to_bits(),
    );
}
```

**Wenn Straddling-Fixture nicht byte-identisch liefert → Bump 12→13** (CURRENT_SNAPSHOT_SCHEMA_VERSION in `service_impl/src/billing_period_report.rs:117`).

---

### Pattern 9: Backfill REST-Endpoint + DTO

**Was:** Neuer Handler in `rest/src/rebooking.rs`. HR-Admin-Gate analog Phase 55-Endpoints.

```rust
// rest-types/src/rebooking.rs (NEU):
#[derive(Debug, Deserialize, ToSchema)]
pub struct BackfillRequestTO {
    pub from: String,        // "YYYY-Www" ISO format
    pub dry_run: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BackfillPreviewItemTO {
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
    pub voluntary_ist: f32,
    pub voluntary_soll: f32,
    pub committed_voluntary: f32,
    pub excess: f32,
    pub would_book: f32,
    pub skipped: bool,
    pub skip_reason: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BackfillPreviewTO {
    pub dry_run: bool,
    pub items: Vec<BackfillPreviewItemTO>,
    pub total_candidates: usize,
    pub already_processed: usize,
}
```

**Endpoint:** `POST /admin/rebooking/backfill?from=YYYY-Www&dry_run={true|false}`  
**Pagination:** `page` + `per_page` query-params für mehrjährige Backfills (D-05).

---

### Pattern 10: Admin-Card Frontend (pdf_export_config-Analogie)

**Was:** Die `pdf_export_config`-Card in `settings.rs` (Zeile 955-980) ist das direkte Vorlage-Muster.

**Source:** `shifty-dioxus/src/page/settings.rs` [VERIFIED: in-repo, grep confirmed]

**FE-Pfade:**
- `shifty-dioxus/src/page/settings.rs` → neue Card "Automatische Freiwilligen-Umbuchung"
  - Toggle-Datum-Editor (`voluntary_rebooking_auto_active_from`)
  - Backfill-Formular: `from`-Woche-Picker + Dry-Run-Checkbox + Button
  - Preview-Tabelle (BackfillPreviewTO-Render)
- `shifty-dioxus/src/loader.rs` → `trigger_rebooking_backfill_dry_run()` + `trigger_rebooking_backfill_apply()`
- `shifty-dioxus/Dioxus.toml` → NEU:

```toml
[[web.proxy]]
backend = "http://localhost:3000/admin/rebooking"
```

**KRITISCH (Pitfall 14):** `/admin/rebooking/*` Route fehlt bisher in `Dioxus.toml`. Muss explizit ergänzt werden. [VERIFIED: Dioxus.toml — kein /admin/rebooking entry vorhanden, line 106-114 zeigt /admin/cutover + /admin/impersonate]

---

### Pattern 11: DI-Wiring + Boot in main.rs

**Was:** `VoluntaryRebookingSchedulerImpl` nach allen Business-Logic-Services konstruieren, am Ende von `main()` nach `pdf_export_scheduler.start()` starten.

**Source:** `shifty_bin/src/main.rs` [VERIFIED: line 1463-1528, 1607-1621]

```rust
// In RestStateImpl::new():
let voluntary_rebooking_scheduler = Arc::new(VoluntaryRebookingSchedulerImpl::new(
    rebooking_reconciliation_service.clone(),
    sales_person_service.clone(),
    toggle_service.clone(),
    week_status_service.clone(),
    permission_service.clone(),
    clock_service.clone(),
    transaction_dao.clone(),
));

// In main() nach pdf_export_scheduler.start():
rest_state
    .voluntary_rebooking_scheduler
    .start()
    .await
    .expect("Expected the voluntary rebooking scheduler to start");
```

**Service-Tier-Konvention:** `VoluntaryRebookingScheduler` ist Business-Logic-Tier (konsumiert `RebookingReconciliationService` + Basic-Services). Boot-Reihenfolge: Basic-Services → BL-Services (inkl. RebookingReconciliationService) → Scheduler. [VERIFIED: CLAUDE.md Service-Tier-Konvention]

---

### Pattern 12: State-Enum `skipped_locked` (D-04)

**Was:** `rebooking_batch.state` hat bereits `'skipped_locked'` als validen Wert in der Schema-Migration.

**Source:** `migrations/sqlite/20260707000000_create-rebooking-batch.sql:15` [VERIFIED: line 15]
```sql
state TEXT NOT NULL,  -- 'pending' | 'approved' | 'rejected' | 'skipped_locked'
```

Der Rust-Enum `RebookingBatchState` in `dao/src/rebooking_batch.rs` muss um `SkippedLocked` erweitert werden (falls noch nicht vorhanden — prüfen).

---

### Anti-Patterns to Avoid

- **NIEMALS `time::Date::year()` + `time::Date::iso_week()` kombinieren** für Vorwoche-Berechnung → Jahresgrenz-Bug. Immer `ShiftyDate::from_date(date).as_shifty_week()`. [Pitfall 5]
- **NIEMALS 5-Feld-Cron-Ausdruck** — `tokio_cron_scheduler 0.15` erwartet 6 Felder. [Pitfall 16]
- **KEINE neue ExtraHours-Aggregations-Logik** — bestehenden `rebook()`-Path wiederverwenden. [Pitfall 15]
- **KEINEN Toggle-Check vergessen** in einer der sechs Chains → verletzt D-06 und Golden-Snapshot-Regression. [Pitfall 3]
- **NIE Straddling-Fixture weglassen** — D-01-Beweislast ist nicht optional, sie ist das Gate. [Pitfall 2]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cron-Loop | eigenen tokio::spawn sleep-loop | `tokio_cron_scheduler` (Phase 48 Präzedenz) | Timezone-sicher, `JobScheduler::new()` lazy via `start()`, Reload möglich |
| ISO-Wochen-Arithmetik | `date.year() + date.iso_week()` | `ShiftyDate::from_date(date).as_shifty_week()` | Jahresgrenz-Bug WOP-Follow-up #3 |
| UNIQUE-Idempotenz | eigener Pre-Select-Check | UNIQUE-Index + `is_unique_violation` → `EntityAlreadyExists` | Phase 54 Index ist atomarer Gate |
| Toggle-Parse | eigene ISO-Date-Parse-Fn | `voluntary_rebooking_gate::parse_active_from()` (analog shortday_gate) | Defensiv: bad input → None (kein Crash) |
| DB-Transaktion | manuelle Rollback-Logik | `transaction_dao.use_transaction()` + `commit()` Muster | Vorhanden in allen Service-Impls |

**Key insight:** Alles was Phase 48 (EXP) für den PDF-Scheduler gebaut hat, kann 1:1 gespiegelt werden. Die einzige echte Neuerung ist die Chain-Audit-Matrix (D-06).

---

## Common Pitfalls

### Pitfall 1: ISO-Jahresgrenz-Bug bei Vorwoche-Berechnung
**What goes wrong:** Cron läuft Mo 2027-01-04 und berechnet `2027-01-04.year()=2027` + `2027-01-04.iso_week()=1` → verarbeitet (2027, 0) oder (2026, 1) falsch. Richtig wäre (2026, 53).
**Why it happens:** `time::Date::year()` und `time::Date::iso_week()` stimmen an Jahresgrenzen nicht überein.
**How to avoid:** `ShiftyDate::from_date(date)` nutzt intern `to_iso_week_date()` — `year` ist immer das ISO-Wochenjahr.
**Warning signs:** Fehlende Tests bei Jahreswechsel; direkter `.year()` Aufruf in Scheduler-Code.

### Pitfall 2: 5-Feld-Cron-Ausdruck → Job registriert nicht
**What goes wrong:** `Job::new_async("0 3 * * 1", ...)` parsed als `sec min hour dom mon` — fehlender `dow` → Job-Parse-Fehler.
**Why it happens:** v2.3.1-Hotfix war nötig, weil Phase 48 mit falschem Default geseedet hat.
**How to avoid:** Seed-Migration mit 6-Feld-String: `"0 15 3 * * 1"` (sec min hour dom mon dow).
**Warning signs:** Startup-Log zeigt kein "cron job registered"; Backend startet dormant.

### Pitfall 3: Chain-Audit-Matrix unvollständig (Pitfall 3)
**What goes wrong:** Toggle-Guard in `run_for_week` eingebaut, aber in `reporting.rs` vergessen → `balance_before != balance_after` nach Cron-Run.
**Why it happens:** Sechs verschiedene Konsumketten, jede in einer anderen Datei.
**How to avoid:** Explizite Checkliste vor Code-Review: Alle 6 Chains (Balance, F1-Ist, F2-Soll, F4-Self, billing_period, booking_information) haben `voluntary_rebooking_gate::should_run_for_week` aufgerufen.
**Warning signs:** Property-Test `balance_before == balance_after` schlägt fehl.

### Pitfall 4: Straddling-Fixture schlägt fehl → muss Bump auf 13
**What goes wrong:** Billing-Period straddling `active_from` liefert unterschiedliche Werte pre/post.
**Why it happens:** Rebooking-ExtraHours verändern doch einen `value_type` (z.B. `extra_hours_by_category`) wegen fehlenden `source=Rebooking`-Filters.
**How to avoid:** `source=Rebooking`-Filter in ALLEN billing_period-Aggregationen; Fixture schreiben und ausführen BEVOR PR gemergt wird.
**Warning signs:** Fixture-Assertion fehlschlägt mit Bit-Pattern-Diff → Bump 12→13 ausführen.

### Pitfall 5: `Dioxus.toml`-Proxy fehlt für `/admin/rebooking`
**What goes wrong:** `POST /admin/rebooking/backfill` → 404 in `dx serve` Dev-Mode.
**Why it happens:** Neue Backend-Routes müssen manuell in `[[web.proxy]]` ergänzt werden. [VERIFIED: Pitfall 14 aus PITFALLS.md]
**How to avoid:** Als erste Action im FE-Plan: Dioxus.toml ergänzen, dann testen.
**Warning signs:** `dx serve` zeigt 404 auf Backfill-Button-Klick.

---

## Runtime State Inventory

Keine Rename/Refactor-Phase — kein Runtime State Inventory erforderlich. Alle neuen Entities sind in Phase 54 migriert (Toggle-Seed `voluntary_rebooking_auto_active_from` vorhanden, Tabellen `rebooking_batch`/`rebooking_batch_entry` vorhanden).

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo build` | alle Tests | ja | rust 1.95 (nix develop) | — |
| `cargo test --workspace` | CI Gate | ja | stable | — |
| `cargo clippy --workspace -- -D warnings` | HARD Gate | ja | stable | — |
| `sqlx` (SQLite offline cache) | DAO compile-time | ja | vorhanden | `sqlx prepare` ausführen |
| `tokio-cron-scheduler 0.15` | Scheduler | ja | vorhanden (Phase 48) | — |
| NixOS nix develop | Build-Env | ja | — | `nix develop` statt `nix-shell` |

**No missing dependencies.** Phase 56 fügt keine neuen Cargo-Dependencies hinzu.

---

## Code Examples

### Example 1: ISO-Vorwoche korrekt berechnen
```rust
// Source: shifty-utils/src/date_utils.rs [VERIFIED]
let today = self.clock_service.date_now();
let today_shifty = ShiftyDate::from_date(today); // ISO-Wochenjahr korrekt
let current_week = today_shifty.as_shifty_week();

// Vorwoche — ShiftyWeek::new() normalisiert Woche 0 → vorheriges Jahr
let prev_week = if current_week.week == 1 {
    ShiftyWeek::new(
        current_week.year - 1,
        time::util::weeks_in_year((current_week.year - 1) as i32),
    )
} else {
    ShiftyWeek::new(current_week.year, current_week.week - 1)
};
let (iso_year, iso_week) = (prev_week.year, prev_week.week);
```

### Example 2: Toggle-Check pro Chain (analog shortday_gate)
```rust
// Source: service_impl/src/shortday_gate.rs [VERIFIED line 51-70]
// In jeder Konsumkette:
let raw = toggle_service
    .get_toggle_value(voluntary_rebooking_gate::TOGGLE_NAME, Authentication::Full, None)
    .await
    .ok()
    .flatten();
let active_from = voluntary_rebooking_gate::parse_active_from(raw.as_deref());
if !voluntary_rebooking_gate::should_run_for_week(iso_year, iso_week, active_from) {
    // Legacy-Semantik: Rebooking-ExtraHours unsichtbar für diese Woche
    return Ok(legacy_result);
}
```

### Example 3: ON CONFLICT DO NOTHING im Cron (Idempotenz)
```rust
// Source: dao_impl_sqlite/src/rebooking_batch.rs [VERIFIED line 162-163, 421-442]
match self.rebooking_batch_service.create(&batch, &[], auth, Some(tx.clone())).await {
    Ok(_) => {
        // Erfolgreich — Pair-ExtraHours schreiben
        self.rebook(sp_id, direction, hours, iso_year, iso_week, tx.clone()).await?;
    }
    Err(ServiceError::EntityAlreadyExists(_)) => {
        // Slot bereits belegt (Claim-on-Suggest oder vorheriger Cron-Lauf)
        // → no-op, continue zu nächstem SalesPerson
        info!("rebooking_cron: slot ({sp_id}, {iso_year}, {iso_week}) already claimed — skip");
    }
    Err(e) => return Err(e),
}
```

### Example 4: Woche auf Locked-Status prüfen
```rust
// Source: service/src/week_status.rs [VERIFIED]
use service::week_status::WeekStatus;

let status = self.week_status_service
    .get_week_status(iso_year, iso_week, Authentication::Full, Some(tx.clone()))
    .await?;
if status == WeekStatus::Locked {
    // Audit-Row schreiben (D-04)
    let skipped_batch = RebookingBatchEntity {
        kind: RebookingBatchKind::AutoCron,
        state: RebookingBatchState::SkippedLocked,
        ..
    };
    self.rebooking_batch_service.create(&skipped_batch, &[], auth, Some(tx.clone())).await
        .ok(); // UNIQUE-Konflikt ignorieren (idempotent)
    return Ok(RunForWeekOutcome::Skipped { reason: SkipReason::WeekLocked });
}
```

### Example 5: Byte-Identitäts-Assertion (Straddling-Fixture)
```rust
// Source: booking_information_weekly_summary_year_batch.rs [VERIFIED line 81-97]
#[track_caller]
fn assert_billing_period_bit_eq(
    actual: &BillingPeriodValueRow, expected: &BillingPeriodValueRow, idx: usize
) {
    assert!(
        !actual.value.is_nan(),
        "row[{idx}] value is NaN — golden-snapshot cannot compare deterministically"
    );
    assert_eq!(
        actual.value.to_bits(), expected.value.to_bits(),
        "BillingPeriod value_type={:?} row[{idx}] diverges: actual={} (0x{:08x}), expected={} (0x{:08x})",
        actual.value_type,
        actual.value, actual.value.to_bits(),
        expected.value, expected.value.to_bits(),
    );
}
```

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (rustc built-in + mockall) |
| Config file | no separate config — workspace Cargo.toml |
| Quick run command | `cargo test -p service_impl rebooking 2>&1 \| head -40` |
| Full suite command | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File |
|--------|----------|-----------|-------------------|------|
| REB-AUTO-01 | Cron verarbeitet Vorwoche für gedeckelte SalesPersons mit Excess | Integration | `cargo test -p service_impl test_voluntary_rebooking_cron_run_for_week` | `service_impl/src/test/voluntary_rebooking.rs` (Wave 0) |
| REB-AUTO-01 | ISO-Jahresgrenz-Robustheit (Vorwoche Jan 1-3) | Unit | `cargo test -p service_impl test_cron_year_boundary` | `service_impl/src/test/voluntary_rebooking.rs` |
| REB-AUTO-02 | active_from=None → Cron läuft nie | Property | `cargo test -p service_impl test_toggle_off_no_rebook` | `service_impl/src/test/voluntary_rebooking.rs` |
| REB-AUTO-02 | active_from=Some(date) → nur Wochen ≥ date | Property | `cargo test -p service_impl test_toggle_active_from_gate` | `service_impl/src/test/voluntary_rebooking.rs` |
| REB-AUTO-02 (D-06) | balance_before == balance_after (Neutralitäts-Property-Test 1) | Property | `cargo test -p service_impl test_neutrality_balance` | `service_impl/src/test/rebooking_neutrality.rs` |
| REB-AUTO-02 (D-06) | F1_Ist_before == F1_Ist_after (Neutralitäts-Property-Test 2) | Property | `cargo test -p service_impl test_neutrality_f1_ist` | `service_impl/src/test/rebooking_neutrality.rs` |
| REB-AUTO-02 (D-06) | F2_Soll_before == F2_Soll_after (Neutralitäts-Property-Test 3) | Property | `cargo test -p service_impl test_neutrality_f2_soll` | `service_impl/src/test/rebooking_neutrality.rs` |
| REB-AUTO-03 | Cron-Idempotenz: zweiter Lauf = no-op | Integration | `cargo test -p service_impl test_cron_idempotent_double_run` | `service_impl/src/test/voluntary_rebooking.rs` |
| REB-AUTO-03 | Claim-on-Suggest: pending HR-Suggestion → Cron skipt | Integration | `cargo test -p service_impl test_claim_on_suggest_blocks_cron` | `service_impl/src/test/voluntary_rebooking.rs` |
| REB-AUTO-04 | dry_run=true → kein DB-Write | Integration | `cargo test -p service_impl test_backfill_dry_run_no_write` | `service_impl/src/test/rebooking_backfill.rs` |
| REB-AUTO-04 | dry_run=false → persistiert auto_cron_backfill | Integration | `cargo test -p service_impl test_backfill_apply_persists` | `service_impl/src/test/rebooking_backfill.rs` |
| REB-AUTO-05 (D-01) | Straddling-Golden-Snapshot pre-Stichtag byte-identisch | Golden | `cargo test -p service_impl test_straddling_snapshot_pre` | `service_impl/src/test/billing_period_straddling.rs` (Wave 0) |
| REB-AUTO-05 (D-01) | Straddling-Golden-Snapshot post-Stichtag byte-identisch | Golden | `cargo test -p service_impl test_straddling_snapshot_post` | `service_impl/src/test/billing_period_straddling.rs` |
| REB-AUTO-05 (D-01) | Straddling-Golden-Snapshot straddling byte-identisch | Golden | `cargo test -p service_impl test_straddling_snapshot_straddling` | `service_impl/src/test/billing_period_straddling.rs` |
| D-04 | Locked-Woche → skipped_locked Audit-Row | Integration | `cargo test -p service_impl test_cron_locked_week_audit` | `service_impl/src/test/voluntary_rebooking.rs` |

### Neutralitäts-Property-Tests (D-06) — kritische Spezifikation

Diese drei Tests sind die primäre Absicherung gegen Doppel-Zählung (Pitfall 1) und Stichtag-Regression (Pitfall 3):

```rust
// service_impl/src/test/rebooking_neutrality.rs

#[tokio::test]
async fn test_neutrality_balance() {
    // Setup: SalesPerson mit cap=true, voluntary_ist > soll + committed
    // Messung VOR Cron: balance_before = reporting_service.get_report(...).balance_hours
    // Cron-Run: voluntary_rebooking_scheduler.run_once_now(auth)
    // Messung NACH Cron: balance_after = reporting_service.get_report(...).balance_hours
    // Assert: balance_before.to_bits() == balance_after.to_bits()
    // (Rebooking ist balance-neutral: -N VolunteerWork + N ExtraWork canceln sich)
}

#[tokio::test]
async fn test_neutrality_f1_ist() {
    // Setup analog
    // Messung VOR: f1_ist_before = voluntary_stats_service.get_voluntary_stats(...).ist_total
    // Cron-Run
    // Messung NACH: f1_ist_after = ...
    // Assert: f1_ist_before == f1_ist_after (source=Rebooking Filter greift)
}

#[tokio::test]
async fn test_neutrality_f2_soll() {
    // Setup analog
    // MessungVOR: soll_before = voluntary_stats_service.get_voluntary_stats(...).soll_total
    // Cron-Run
    // Messung NACH: soll_after = ...
    // Assert: soll_before == soll_after (Soll ist unabhängig von ExtraHours)
}
```

### Straddling-Golden-Snapshot-Fixture (D-01-Beweislast)

```
Fixture-Aufbau:
  Period: 2026-07-01 bis 2026-08-31 (straddling um active_from=2026-08-01)
  SalesPerson: cap=true, 1 Rebooking-Batch in KW 2026-W31 (liegt Post-Stichtag)

Test 1 (Pre): active_from = 2026-09-01 → Rebooking für 2026-W31 unsichtbar
              → Snapshot-Werte = Baseline ohne Rebooking
Test 2 (Post): active_from = 2026-07-01 → Rebooking für 2026-W31 sichtbar
              → balance-neutral → Snapshot-Werte = bit-identisch zu Test 1
Test 3 (Straddling): active_from = 2026-08-01
              → Rebooking 2026-W31 liegt nach active_from → sichtbar
              → balance-neutral → bit-identisch zu Test 1 + Test 2

Gate: alle drei Tests MÜSSEN byte-identisch sein.
Falls ein Test abweicht → Bump CURRENT_SNAPSHOT_SCHEMA_VERSION 12→13.
```

### Sampling Rate
- **Per task commit:** `cargo test -p service_impl rebooking && cargo clippy --workspace -- -D warnings`
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Phase gate (vor /gsd-verify-work):** Full suite grün + Straddling-Fixture grün + FE `cargo build --target wasm32-unknown-unknown` + `cargo test -p shifty-dioxus`

### Wave 0 Gaps (neue Testdateien)
- [ ] `service_impl/src/test/voluntary_rebooking.rs` — covers REB-AUTO-01, REB-AUTO-03, D-04
- [ ] `service_impl/src/test/rebooking_neutrality.rs` — covers D-06 (3 Neutralitäts-Property-Tests)
- [ ] `service_impl/src/test/rebooking_backfill.rs` — covers REB-AUTO-04
- [ ] `service_impl/src/test/billing_period_straddling.rs` — covers REB-AUTO-05 (D-01-Beweislast)

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | HR-Admin-Gate via `check_permission(HR_PRIVILEGE, context)` — Präzedenz Phase 55 |
| V3 Session Management | no | Scheduler läuft mit `Authentication::Full` (trusted internal) |
| V4 Access Control | yes | `POST /admin/rebooking/backfill` — Admin-Privilege-Check ZUERST vor Datenabruf (Pitfall 8) |
| V5 Input Validation | yes | `from=YYYY-Www` Format-Validation; `dry_run=bool`; Week-Range-Sanity-Check |
| V6 Cryptography | no | Keine neuen Crypto-Operationen |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Backfill-Range-Exhaustion (sehr langer `from`-Zeitraum) | DoS | Pagination (D-05) + max-range-Limit |
| Double-Approve via Race (Backfill + live Cron gleichzeitig) | Tampering | UNIQUE-Constraint fängt; `EntityAlreadyExists` → no-op |
| HR-Privilege-Bypass auf Backfill-Endpoint | Elevation of Privilege | `check_permission(HR_PRIVILEGE)` ZUERST, Präzedenz Phase 55 |

---

## Project Constraints (from CLAUDE.md)

- **Clippy-Gate (HARD):** `cargo clippy --workspace -- -D warnings` bei JEDEM Commit. `cargo test` allein reicht NICHT. [VERIFIED: CLAUDE.md]
- **Docs-Freshness-Gate:** Trigger-Dateien → Docs-Sync. Phase 56 berührt: `main.rs` → `docs/architecture/02-service-tiers.md` + `service-graph-runtime.mmd`; `service/**/*.rs` (neue Traits) → `docs/features/F14-rebooking.md` + `_de.md`. Falls `reporting.rs` Balance-Chain berührt → `docs/features/F07-reporting-balance.md` + `_de.md`. Beide Sprachen (EN + DE) synchron im gleichen Commit. [VERIFIED: CLAUDE.md Docs-Freshness-Gate-Tabelle]
- **Snapshot-Schema-Versioning:** Wenn Straddling-Fixture scheitert → `CURRENT_SNAPSHOT_SCHEMA_VERSION` in `service_impl/src/billing_period_report.rs:117` auf 13 bumpen + Migration + `docs/features/F08-billing-period.md` + `_de.md`. [VERIFIED: CLAUDE.md §Billing Period Snapshot Schema Versioning]
- **Service-Tier-Konvention:** `VoluntaryRebookingScheduler` = Business-Logic-Tier. Konsumiert `RebookingReconciliationService` (BL) + Basic-Services. Kein Zyklus. [VERIFIED: CLAUDE.md]
- **Fat Backend, Thin Client:** Alle Berechnungen (Excess, Preview-Werte, BackfillPreviewTO) im Backend. FE zeigt vorbereitete DTOs. [VERIFIED: REQUIREMENTS.md]
- **OpenAPI-Dokumentation:** `#[utoipa::path]` auf allen neuen REST-Handlern. `BackfillRequestTO` + `BackfillPreviewTO` mit `ToSchema`. In ApiDoc struct eintragen. [VERIFIED: CLAUDE.md]
- **jj VCS + GSD-Auto-Commit:** commit_docs=true, yolo-Mode. GSD committet automatisch. [VERIFIED: STATE.md]
- **NixOS:** `nix develop` (NICHT `nix-shell`). `sqlx migrate run` für additive Migrationen (NICHT `sqlx database reset`). [VERIFIED: STATE.md]
- **Multi-Sprache i18n:** Alle neuen benutzersichtbaren Texte in de/en/cs.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 5-Feld-Cron `"0 6 * * 1"` | 6-Feld `"0 0 6 * * 1"` | v2.3.1-Hotfix | Migration fix-pdf-export-cron-6-field.sql |
| Ohne Toggle-Guard | `shortday_gate.rs`-Muster (SHC-04) | v2.4 Phase 51 | Pro-Chain-Guard + parse_active_from |
| Snapshot-Bump ohne Straddling-Fixture | Straddling-Fixture als Beweislast | v2.6 Phase 56 (D-01) | Explizite Gate für "kein Bump"-Entscheidung |

**Deprecated/outdated:**
- CLI-Backfill (`clap`): bewusst abgelehnt (REQUIREMENTS.md §Non-Goals). REST-Backfill wiederverwendet Auth + Audit-Trail.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `RebookingBatchState::SkippedLocked` ist noch NICHT im Rust-Enum (nur in DB-Migration als String) | Pattern 12 | Enum-Erweiterung in Phase 55 erledigt → kein Aufwand; sonst Enum-Variant ergänzen |
| A2 | `pdf_export_scheduler` läuft Montag 06:00 — kein Konflikt mit 03:15 | D-03/Code Examples | Wenn pdf_export auch 03:xx → Minute anpassen (Claude's Discretion) |
| A3 | `billing_period_report.rs` liest ExtraHours per `source` nicht gefiltert | Chain-Audit D-06 | Prüfen ob Phase 55 bereits `source=Manual` Filter in billing_period eingebaut hat; falls ja → kein Guard nötig da Rebooking schon separiert |

**A1 + A3 müssen beim Planner verifiziert werden** (per `grep -n "SkippedLocked\|skipped_locked" dao/src/rebooking_batch.rs` und `grep -n "source.*Rebooking" service_impl/src/billing_period_report.rs`).

---

## Open Questions

1. **Conflict-freie Boot-Minute** — Welche Minute läuft `pdf_export_scheduler`? [Empfehlung: `grep "cron_schedule" migrations/sqlite/20260703000000_create-pdf-export-config.sql` → `0 6 * * 1` → VoluntaryRebooking sicher auf `0 15 3 * * 1` oder `0 0 4 * * 1`]

2. **RebookingBatchState-Enum vollständig?** — Prüfen ob `SkippedLocked` bereits in `dao/src/rebooking_batch.rs` vorhanden. Die DB-Migration hat es (`'skipped_locked'`), aber der Rust-Enum ggf. nicht.

3. **billing_period_report ExtraHours-Filter** — Liest `build_and_persist_billing_period_report()` ExtraHours mit oder ohne `source=Rebooking`-Filter? Phase 55 D-55-A1 hat `source==Manual`-Filter im Reporting eingebaut — aber gilt das auch für billing_period? Prüfen via `grep -n "source\|ExtraHoursSource" service_impl/src/billing_period_report.rs`.

---

## Sources

### Primary (HIGH confidence — verified in-repo)
- `service_impl/src/pdf_export_scheduler.rs` — 1:1-Blueprint für VoluntaryRebookingScheduler (Trait, Impl, Boot, clone_for_job)
- `service_impl/src/rebooking_reconciliation.rs` — Phase 55 BL-Tier; `rebook()`-Pair-Write-Path zu erweitern
- `service_impl/src/shortday_gate.rs` — D-06 Chain-Guard-Muster (parse_active_from, should_clip, read_active_from)
- `shifty-utils/src/date_utils.rs` — ShiftyDate, ShiftyWeek API für ISO-Wochen-Arithmetik
- `migrations/sqlite/20260707000000_create-rebooking-batch.sql` — UNIQUE-Index-Definition (line 42-44)
- `migrations/sqlite/20260704000000_fix-pdf-export-cron-6-field.sql` — 6-Feld-Cron-Hotfix
- `migrations/sqlite/20260707000002_seed-voluntary-rebooking-toggle.sql` — Toggle-Seed bestätigt
- `dao_impl_sqlite/src/rebooking_batch.rs` — `is_unique_violation` + `EntityAlreadyExists`-Pattern
- `service_impl/src/test/booking_information_weekly_summary_year_batch.rs` — WOP-03 8-Fixtures-Muster (Golden-Snapshot-Fixture-Blueprint)
- `shifty-dioxus/Dioxus.toml` — Proxy-Liste (kein `/admin/rebooking` → muss ergänzt werden)
- `shifty_bin/src/main.rs` — Boot-Sequenz `pdf_export_scheduler.start()` (line 1618-1621)

### Secondary (HIGH confidence — planning docs)
- `.planning/phases/56-wochen-cron-rollout-backfill-f4/56-CONTEXT.md` — alle 6 Decisions D-01..D-06
- `.planning/REQUIREMENTS.md` §REB-AUTO-01..05 — vollständige Requirements
- `.planning/research/PITFALLS.md` §Pitfall 2,3,4,5,7,13,16 — konkrete Muster
- `.planning/phases/55-manual-rebooking-hr-alert/55-CONTEXT.md` — Phase 55 Design (rebook()-Path, Claim-on-Suggest)

---

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — alle Libraries in-repo verifiziert, keine neuen Dependencies
- Architecture: HIGH — 1:1-Spiegel von Phase 48 PdfExportScheduler + Phase 51 SHC-04 Chain-Guard
- ISO-Wochen-Arithmetik: HIGH — ShiftyDate/ShiftyWeek-API in-repo verifiziert; Jahresgrenz-Pitfall dokumentiert
- Chain-Audit-Matrix: HIGH — shortday_gate.rs als vollständiges Muster; alle 6 Chains identifiziert
- Straddling-Fixture: HIGH — WOP-03-Muster in-repo vorhanden; Fixture-Aufbau konkret spezifiziert
- Backfill REST + FE: MEDIUM — REST-Muster aus Phase 55 vorhanden; FE-Card-Detail aus pdf_export_config-Analogie erschlossen

**Research date:** 2026-07-26
**Valid until:** 2026-09-01 (stabile Codebase; Pitfalls bleiben relevant)
