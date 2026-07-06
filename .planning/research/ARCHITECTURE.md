# Architecture Patterns — v2.6 Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter

**Milestone:** v2.6 (subsequent, additive on shipped v2.5 baseline)
**Researched:** 2026-07-06
**Confidence:** HIGH (all decisions grounded in existing precedents: v1.4 committed_voluntary, v1.7 ToggleService gate, v2.2 tokio_cron_scheduler, v2.3 PDF-Assembler-BL, v2.5 Fat-Backend `sales_person_absences`)

## Guiding Principles for This Milestone

1. **Fat Backend, Thin Client** (PROJECT.md §Architektur-Prinzipien) — every rebooking suggestion, statistics number, and account computation is a pre-computed field in the DTO. Frontend does zero arithmetic.
2. **Service-Tier-Konvention** (CLAUDE.md) — Basic Services own single aggregates and consume only DAOs + Permission + Transaction. Business-Logic Services combine aggregates. Rebooking touches ExtraHours (Basic) + WorkingHours (Basic) + Reporting (BL) — it *must* be BL.
3. **Additive Migrations** — precedent from v1.7 / v1.8 / v2.1 / v2.4 shows new tables + toggles ship without touching `billing_period_sales_person`. v2.6 follows suit.
4. **Cron Scheduler Precedent** — `PdfExportSchedulerImpl` (v2.2/v2.3, `service_impl/src/pdf_export_scheduler.rs`) already owns the pattern: `tokio_cron_scheduler::JobScheduler` behind an `Arc<Mutex<Option<…>>>`, lazy-init in `start()`, DB-driven reload. Copy-paste-adapt shape, do not invent a new one.

---

## Recommended Architecture

### High-Level Data Flow (F1–F5)

```
┌─ Wochen-Cron (tokio_cron_scheduler) ────────────────────────────────┐
│  VoluntaryRebookingScheduler.start()  ───► run_once_for_previous_week
│                                                     │                │
│                                                     ▼                │
└──────────────────────► RebookingReconciliationService (BL) ─────────┘
                          │      │           │           │
                          ▼      ▼           ▼           ▼
                  ExtraHoursService  ReportingService  WorkingHoursService  RebookingBatchService
                        (Basic)         (BL, read)        (Basic)                (Basic, entity-mgr)
                          │                │                │                       │
                          ▼                ▼                ▼                       ▼
                    extra_hours       shiftplan_report  employee_work_details  rebooking_batch(_entry)
                                                                                    (2 new tables)

REST (Axum)
├─ GET  /reporting/employee/{y}/{spid}         → ReportingService (existing, extended DTO)
├─ GET  /reporting/employee/{y}/{spid}/voluntary-stats → VoluntaryStatsService (new, HR-gated)
├─ POST /rebooking/manual                      → RebookingReconciliationService.rebook_manual (F3)
├─ GET  /rebooking-suggestions?state=pending   → RebookingBatchService.list_pending (F5)
├─ POST /rebooking-suggestions/{id}/approve    → RebookingReconciliationService.approve (F5)
├─ POST /rebooking-suggestions/{id}/reject     → RebookingBatchService.reject (F5)
└─ POST /admin/rebooking/backfill?from=YYYY-Www → RebookingReconciliationService.backfill (F4 rollout)
```

### Component Boundaries

| Component | Tier | Responsibility | Consumes | Consumed By |
|-----------|------|----------------|----------|-------------|
| `RebookingBatchService` (NEW) | **Basic** (entity-manager) | CRUD `rebooking_batch` + `rebooking_batch_entry`; state transitions `pending→approved/rejected`; pure list/find queries | `RebookingBatchDao` (new), `PermissionService`, `TransactionDao` | `RebookingReconciliationService`, REST |
| `RebookingReconciliationService` (NEW) | **Business-Logic** | orchestrates: compute account → decide rebooking → within one tx: create ExtraHours pair + create/update batch entry | `RebookingBatchService`, `ExtraHoursService`, `ReportingService`, `WorkingHoursService`, `SalesPersonService`, `PermissionService`, `TransactionDao` | REST + `VoluntaryRebookingScheduler` |
| `VoluntaryStatsService` (NEW) | **Business-Logic** (read-only aggregate) | pure fn `voluntary_hours_per_contract_week(sp_id, year)` + `committed_voluntary_target(sp_id, year)` (Soll-Summe) — HR-gated read | `ExtraHoursService`, `WorkingHoursService`, `SalesPersonService`, `PermissionService`, `TransactionDao` | REST (F1, F2) |
| `VoluntaryRebookingScheduler` (NEW) | **Business-Logic** | Wochen-Cron; delegates work to `RebookingReconciliationService.run_for_week(prev_week)`; DB-driven toggle-reload analog `PdfExportSchedulerImpl` | `RebookingReconciliationService`, `ToggleService`, `ClockService`, `PermissionService`, `TransactionDao` | boot in `shifty_bin/src/main.rs` |
| `RebookingBatchDao` (NEW) | DAO | SQL against 2 new tables | pool | `RebookingBatchService` |
| `ExtraHoursService` (existing, unchanged) | Basic | CRUD `extra_hours` — reused verbatim (F3/F4/F5 all persist the double-entry pair via existing `create`) | | reconciliation, everywhere |
| `WorkingHoursService` (existing, unchanged) | Basic | `committed_voluntary` + `expected_hours` per week — already a Basic entity-manager; helper `committed_voluntary_for_calendar_week` (in `service_impl::reporting`) stays put | | reconciliation, stats |
| `ReportingService` (existing, unchanged) | BL | already exposes `EmployeeReport.balance_hours` (existing balance formula = negative → trigger for F5) and `WorkingHoursPerSalesPerson.volunteer_hours` (per week) | | reconciliation (reads only) |
| `ToggleService` (existing, unchanged) | Basic | `voluntary_rebooking_auto_active_from` toggle key (F4 Stichtag-Gate, precedent HCFG-02 / `shortday_slot_clipping_active_from`) | | scheduler + reconciliation |

**Zyklen-Freiheit-Check.** All new BL services depend on Basic services + `RebookingBatchService` (Basic). No new BL→BL edges (RebookingReconciliation reads `ReportingService`, another BL, but Reporting does not read RebookingReconciliation — same precedent as `BookingInformationService` reading `ReportingService`). Compiles cleanly under existing `gen_service_impl!` DI graph.

### Answers to the Eight Specific Questions

**1. RebookingBatch + RebookingBatchEntry — one service or two?**
Two. `RebookingBatchService` (Basic entity-manager) owns the aggregate lifecycle: create batch, add entries, transition state (`pending → approved | rejected`), list by state / by sales_person / by kind. `RebookingReconciliationService` (BL) sits on top and owns the *domain-orchestrated* actions ("decide + persist double-entry + persist batch"). Split matches the precedent `BillingPeriodService` (Basic) vs. `BillingPeriodReportService` (BL) — same shape, verified pattern.

**2. Freiwilliges Soll = `committed_voluntary × Vertragswochen` — WorkingHoursService (Basic) or ReportingService (BL)?**
Neither directly. Put the pure function `committed_voluntary_target_for_year(&[EmployeeWorkDetails], year: u32) -> f32` next to the existing pure helper `committed_voluntary_for_calendar_week` in `service_impl/src/reporting.rs` (module-level `pub fn`). Then expose it through `VoluntaryStatsService` (new BL). Rationale: pure fn = testable, no auth; the BL service adds auth-gate + DTO assembly. Precedent: `volunteer_surplus_band2` / `volunteer_surplus_above_committed` in `service_impl/src/booking_information.rs` — same shape (pure `pub(crate) fn` + BL service call site).

**3. F1 Statistik — new `VoluntaryStatsService` or extend `ReportingService`?**
New `VoluntaryStatsService`. `ReportingService` is already heavy (extra_hours + shiftplan_report + employee_work_details + carryover + special_day + toggle + absence — 8 deps). Adding a "Ø-freiwillig/Vertragswoche" concern bloats it and mingles bezahlt / freiwillig axes that Fat Backend deliberately keeps separate (v1.4 D-01 / CVC-05 rationale, D-53-02 precedent in v2.5). `VoluntaryStatsService` reuses `ExtraHoursService` + `WorkingHoursService` + a pure fn — clean single-concern BL. Small enough to justify.

**4. F4 Cron-Job — where in DI graph?**
Exact analog of `PdfExportSchedulerImpl` (`service_impl/src/pdf_export_scheduler.rs`). In `shifty_bin/src/main.rs`:
- Basic wave: build `RebookingBatchDao` + `RebookingBatchService` next to other Basic services (e.g. next to `ToggleService` / `WeekStatusService`).
- BL wave: after `ReportingService` + `ExtraHoursService` + `WorkingHoursService` are built, construct `RebookingReconciliationService`, then `VoluntaryStatsService`, then `VoluntaryRebookingScheduler` (needs Reconciliation as dep).
- Boot: at the end of `main()`, after `pdf_export_scheduler.start().await`, call `rest_state.voluntary_rebooking_scheduler.start().await`. Use the same `Arc<Mutex<Option<JobScheduler>>>` + `Arc<Mutex<Option<Uuid>>>` shape.
- **Toggle-Gate** for auto-mode: cron *runs* unconditionally but each invocation reads `voluntary_rebooking_auto_active_from` (analog HCFG-02); if `booking_week < active_from` or toggle unset → skip (kind=`auto_cron` never fires). This protects historical weeks during rollout.

**5. F5 REST-Endpoints — routing + auth.**
Under a new `rest/src/rebooking.rs` module (Axum) plus `RestStateDef::rebooking_batch_service()` / `.rebooking_reconciliation_service()` accessors on `RestStateImpl`. Endpoints:
- `GET  /rebooking-suggestions?state=pending&kind=hr_suggestion` → `HR_PRIVILEGE`
- `GET  /rebooking-suggestions/{id}` → `HR_PRIVILEGE`
- `POST /rebooking-suggestions/{id}/approve` → `HR_PRIVILEGE`
- `POST /rebooking-suggestions/{id}/reject` → `HR_PRIVILEGE`
- `POST /rebooking/manual` (F3) → `HR_PRIVILEGE` (double-entry within one tx)
- `POST /admin/rebooking/backfill` (F4 rollout) → `check_only_full_authentication` (admin path, like `soft_delete_bulk`)

All handlers wrapped in `error_handler` + annotated with `#[utoipa::path]`. Same shape as `pdf_export_config` REST routes.

**6. Migration order + Snapshot-Bump — confirm or refute.**
**Confirm.** Additive: two new tables in one migration file (e.g. `20260707000000_create-rebooking-batch.sql`), plus a second additive file to seed the `voluntary_rebooking_auto_active_from` toggle (F4 Stichtag, precedent `20260704000001_seed-shortday-slot-clipping-toggle.sql`). **No Snapshot-Schema-Version bump** because F1–F5 never write a new `BillingPeriodValueType` to `billing_period_sales_person`. F3/F4/F5 persist *only* two ExtraHours rows (VolunteerWork -N / ExtraWork +N) + one rebooking_batch(_entry) row. ExtraHours already flow into the existing snapshot terms (`working_hours`, `absense_hours`), so downstream billing-period-report picks up the effect *without* new value_types.
  - **BUT** — the `docs/features/F08-billing-period.md` snapshot value-type list should be extended with a *narrative note* that ExtraWork rows created via rebooking flow into `working_hours` unchanged (docs-freshness gate, PROJECT.md).
  - **AND** — bump the *milestone-close checklist* to grep `service_impl/src/billing_period_report.rs` for accidental value-type additions (Pitfall guard).

**7. Frontend structure.**
- **New loader (`shifty-dioxus/src/loader.rs`):**
  - `load_rebooking_suggestions_pending(config)` → `Rc<[RebookingSuggestionTO]>`
  - `load_voluntary_stats(config, sp_id, year)` → `VoluntaryStatsTO` (Ist + Soll + Konto vom Backend)
  - `approve_rebooking_suggestion(config, id)` / `reject_rebooking_suggestion(config, id)` (mutations)
  - `submit_manual_rebooking(config, ManualRebookingRequestTO)`
- **New state (`shifty-dioxus/src/state/`):** `rebooking.rs` (thin `From<&…TO>`-mapper analog to `state::WeeklySummary::from(&WeeklySummaryTO)` — pure union/rename, no math).
- **New component (`shifty-dioxus/src/component/`):**
  - `rebooking_alert_banner.rs` — persistent inline warning row on `page/employees.rs` when a sales_person has negative balance AND has voluntary hours available. Backend flags this in the response, FE only renders.
  - `rebooking_suggestion_modal.rs` — IST vs. DANN table (Konto, Freiw.-Ist, Freiw.-Soll, Freiw.-Konto), Approve/Reject buttons.
  - `voluntary_stats_row.rs` — extension to `page/employee_details.rs` under „Freiwillige Stunden" showing Ist + Soll + Konto (HR-only via existing role gate on the parent page).
- **`Dioxus.toml` proxy** — add `[[web.proxy]]` for `/rebooking-suggestions`, `/rebooking`, `/reporting/employee/**/voluntary-stats` (precedent MEMORY: v1.8 + v2.2 both forgot this and hit 404 in dev).
- **i18n:** de/en/cs keys for banner text, modal columns, approve/reject buttons, manual-rebooking form.
- **No math in FE.** Every displayed number arrives pre-computed. `RebookingSuggestionTO` carries: `current_balance`, `voluntary_actual`, `voluntary_committed`, `voluntary_balance`, `proposed_rebooking_hours`, `projected_balance_after`, `projected_voluntary_balance_after` — all `f32`, backend-computed.

**8. Suggested build order (Wave-Empfehlung).**
Dependencies dictate: statistics computation feeds the modal, modal is the UAT anchor for reconciliation, cron is a wrapper on reconciliation, alerts need the account.

- **Phase 54 — Data-model + statistics (F1 + F2)**
  - Wave 1: Migration (`rebooking_batch`, `rebooking_batch_entry`) + `RebookingBatchDao` + `RebookingBatchService` (Basic, CRUD only, no logic yet — just the aggregate). Toggle-Seed for `voluntary_rebooking_auto_active_from`.
  - Wave 2: Pure fn `committed_voluntary_target_for_year` + `VoluntaryStatsService` (BL) + REST `GET /reporting/employee/{y}/{spid}/voluntary-stats` + `VoluntaryStatsTO` in rest-types.
  - Wave 3: Frontend row „Freiwillige Stunden — Ist / Soll / Konto" in `page/employee_details.rs`. Reads via new loader. F2 sichtbar.
  - Verify: byte-identical to hand-computed reference on a fixture (property test), HR-only auth-gate.
- **Phase 55 — Manual rebooking + HR-Alert + Modal (F3 + F5 without cron)**
  - Wave 1: `RebookingReconciliationService.rebook_manual` — single tx creates 2 ExtraHours + 1 batch (kind=`hr_suggestion`, state=`approved`). Auth: `HR_PRIVILEGE`. `POST /rebooking/manual`.
  - Wave 2: `RebookingReconciliationService.suggest_for_sales_person` (pure decide-logic) + `.approve` / `RebookingBatchService.reject`. REST: `GET /rebooking-suggestions`, `POST .../approve|reject`.
  - Wave 3: Frontend alert banner on `page/employees.rs` (Backend liefert `has_rebooking_suggestion` + `suggestion_id` im existing `ShortEmployeeReport` DTO oder in einer neuen kleinen Sammel-API `GET /rebooking-suggestions/summary`). Modal-Komponente.
  - Verify: Roundtrip create-manual → visible in `EmployeeReport` → visible in banner list. UAT.
- **Phase 56 — Cron + Backfill (F4)**
  - Wave 1: `VoluntaryRebookingScheduler` (analog `PdfExportSchedulerImpl`) — `start()`, cron `0 0 3 * * 1` (Monday 03:00, previous week), delegates to `RebookingReconciliationService.run_for_week(year, week)`. Stichtag-Gate via toggle.
  - Wave 2: `POST /admin/rebooking/backfill?from=YYYY-Www` (admin) — iteriert Wochen und ruft `run_for_week` — creates kind=`auto_cron` batches.
  - Wave 3: Frontend Admin-Card unter Settings analog `pdf_export_config` — Stichtag setzen / Backfill-Button.
  - Verify: End-to-end mit fixed clock + fake previous week; historical protection via active_from.

Rationale for the order: F1+F2 deliver visible value (statistics) with lowest risk; F3+F5 need the account numbers to display in the modal; F4 wraps F3-logic in cron with a Stichtag — no new domain logic, just automation.

## Patterns to Follow

### Pattern 1: Basic vs. Business-Logic Split for Aggregate + Orchestration
**What:** When a new domain object needs *both* CRUD and multi-service orchestration, split into `<Name>Service` (Basic, entity CRUD) + `<Name>OrchestrationService` (BL, wires it into other services).
**When:** Any milestone that introduces a new persistent aggregate consumed by cross-cutting logic.
**Example (v2.6):**
```rust
// Basic
pub struct RebookingBatchServiceImpl<Deps: RebookingBatchServiceDeps> { … }
// BL — consumes Basic + other services
pub struct RebookingReconciliationServiceImpl<Deps: RebookingReconciliationServiceDeps> { … }
```
**Precedent:** `BillingPeriodService` + `BillingPeriodReportService`; `WeekStatusService` + `ShiftplanEditService`.

### Pattern 2: Pure Fn in service_impl + Auth-gated BL Wrapper
**What:** Business calculation lives as `pub fn` (or `pub(crate) fn`) at module top; BL service method loads inputs, calls pure fn, applies auth gate.
**When:** Any calculation you want unit-testable without mock services.
**Example:**
```rust
// service_impl/src/reporting.rs (module-level)
pub fn committed_voluntary_target_for_year(
    work_details: &[EmployeeWorkDetails],
    year: u32,
) -> f32 { … }

// service_impl/src/voluntary_stats.rs
async fn get_stats(&self, sp_id: Uuid, year: u32, ctx, tx) -> Result<VoluntaryStats, ServiceError> {
    self.permission_service.check_permission(HR_PRIVILEGE, ctx).await?;
    let wd = self.working_hours_service.for_sales_person(...).await?;
    let target = committed_voluntary_target_for_year(&wd, year);
    …
}
```
**Precedent:** `committed_voluntary_for_calendar_week`, `apply_weekly_cap`, `volunteer_surplus_band2`, `derive_hours_for_week_pure`.

### Pattern 3: Cron Scheduler as BL Service, Started in main()
**What:** New scheduler = new struct implementing `SchedulerService`-style trait, `start()` lazy-inits `JobScheduler`, called after `RestStateImpl::new`.
**When:** Any recurring task.
**Precedent:** `PdfExportSchedulerImpl` (`service_impl/src/pdf_export_scheduler.rs:60–115`), `SchedulerServiceImpl` (`service_impl/src/scheduler.rs`).

### Pattern 4: Stichtag-Toggle for Retroactive Behaviour Rollout
**What:** New behavior is gated by an admin-set `active_from` date in `ToggleService`; unset toggle = feature dormant (default), set toggle = feature applies only to `booking_date >= active_from`.
**When:** Behavior that changes historical calculations *would* introduce drift.
**Precedent:** `holiday_auto_credit` (v1.7 HCFG-02), `shortday_slot_clipping_active_from` (v2.4 SHC).
**v2.6 usage:** `voluntary_rebooking_auto_active_from` — the F4 cron only auto-books for weeks ≥ toggle date. Rollout-safe.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Frontend computes the rebooking preview
**What:** FE receives `voluntary_ist`, `expected_hours`, `committed_voluntary`, does the arithmetic to show „Wenn Du zustimmst, wird das Konto von –10h auf –4h steigen".
**Why bad:** Every future client (mobile, CLI) has to re-implement the same math; drift from BE guaranteed. Violates Fat Backend.
**Instead:** `RebookingSuggestionTO` carries `projected_balance_after`, `projected_voluntary_balance_after`, `proposed_rebooking_hours` — all backend-computed. FE renders literals.

### Anti-Pattern 2: Reconciliation writes ExtraHours directly via DAO
**What:** Skip `ExtraHoursService.create` and write via `ExtraHoursDao.create`.
**Why bad:** Loses the `CustomExtraHoursService` lazy-load setup, loses permission gate (though `Authentication::Full` internal-caller pattern applies), and — worst — bypasses whatever validation `ExtraHoursService.create` grows in future milestones. Two entries out-of-sync at DAO level is exactly the bug the batch table exists to prevent.
**Instead:** Both ExtraHours rows via `ExtraHoursService.create(..., Authentication::Full, tx.into())` (precedent: internal-service pattern used throughout). All three writes (two ExtraHours + one batch entry) in the same tx via the shared `use_transaction` handle.

### Anti-Pattern 3: Cron modifies weeks locked by `WeekStatus`
**What:** F4 cron creates ExtraHours for a week whose `WeekStatus` is `Locked`.
**Why bad:** Violates the v2.1 wochen-sperre TOCTOU guarantee.
**Instead:** `RebookingReconciliationService.run_for_week` checks `WeekStatusService` before persist and short-circuits with `ValidationError` (records to `rebooking_batch` with state=`skipped_locked`, no ExtraHours).

### Anti-Pattern 4: Bumping snapshot-schema-version without new value_type
**What:** Bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` „just in case" because we touched extra_hours.
**Why bad:** Every bump invalidates every historical snapshot for validators (CLAUDE.md rule). No new `BillingPeriodValueType` = no bump.
**Instead:** Grep-verify `billing_period_sales_person.value_type` writes are unchanged; document the non-bump in ROADMAP explicitly (v2.5-precedent).

### Anti-Pattern 5: Rebooking without atomic tx (double-count risk)
**What:** Create ExtraHours row 1, commit, then create row 2. Between: another tx reads reporting.
**Why bad:** Reporting sees `-N` in VolunteerWork without the matching `+N` in ExtraWork — balance briefly wrong; if second insert fails, permanently wrong. Precedent in MEMORY: `feedback_atomic_repoint_no_double_count`.
**Instead:** Single `use_transaction`-tx across both ExtraHours + batch entry writes. Commit only after all three succeed.

## Scalability Considerations

| Concern | At current scale (~30 employees) | At 300 employees | At 3000 |
|---------|----------------------------------|------------------|---------|
| F1 stats endpoint | trivial, per-request compute over ~50 weeks | still OK; add per-year cache if needed | precompute nightly into new column |
| F4 cron weekly run | ~30 checks × ~1s each → <1min, fine | ~5min, still fine | batch in transaction chunks of 100 |
| F5 alert banner | reads all employees' balance (already done in `page/employees.rs`) | already the existing hot path — reuse | as above |
| Rebooking batches over years | tiny table growth (~30 rows/week) | ~300/week × 52 = ~15k/year — irrelevant | add `deleted IS NULL` + `state != rejected` filter index |

Guidance: don't optimize prematurely. Current baseline is `~30 sales_persons`. Existing v2.5 performance work eliminated the hot path (weekly-overview 2.33s → 0.12s). v2.6 endpoints are per-request small aggregates — no year-batch acrobatics needed. Add DB indices `rebooking_batch(state)` and `rebooking_batch_entry(sales_person_id, batch_id)` in the migration file preemptively (cheap, no downside).

## Data Model (New Tables)

```sql
-- 20260707000000_create-rebooking-batch.sql
CREATE TABLE rebooking_batch (
    id                     BLOB PRIMARY KEY,               -- Uuid
    kind                   TEXT NOT NULL,                  -- 'auto_cron' | 'hr_suggestion' | 'manual'
    state                  TEXT NOT NULL,                  -- 'pending' | 'approved' | 'rejected' | 'skipped_locked'
    booking_year           INTEGER,                        -- NULL for kind='manual'
    booking_week           INTEGER,                        -- NULL for kind='manual'
    created                TIMESTAMP NOT NULL,
    approved               TIMESTAMP,
    approved_by            TEXT,                           -- username
    deleted                TIMESTAMP,                       -- soft delete
    version                BLOB NOT NULL,                  -- Uuid, optimistic lock
    update_process         TEXT NOT NULL
);

CREATE TABLE rebooking_batch_entry (
    id                     BLOB PRIMARY KEY,
    batch_id               BLOB NOT NULL REFERENCES rebooking_batch(id),
    sales_person_id        BLOB NOT NULL,
    hours                  REAL NOT NULL,                  -- positive: moved from Volunteer to ExtraWork
    balance_before         REAL NOT NULL,
    voluntary_actual       REAL NOT NULL,
    voluntary_committed    REAL NOT NULL,
    extra_hours_out_id     BLOB,                           -- the -N VolunteerWork row, NULL until approved
    extra_hours_in_id      BLOB,                           -- the +N ExtraWork row,     NULL until approved
    deleted                TIMESTAMP,
    version                BLOB NOT NULL,
    update_process         TEXT NOT NULL
);

CREATE INDEX rebooking_batch_state_idx ON rebooking_batch(state) WHERE deleted IS NULL;
CREATE INDEX rebooking_batch_entry_sp_idx ON rebooking_batch_entry(sales_person_id) WHERE deleted IS NULL;
```

Precedents for column shape: `week_status` (v2.1), `pdf_export_config` (v2.3), `vacation_entitlement_offset` (v1.8). Same Uuid + version + soft-delete conventions.

## Modified DTOs

| DTO | Change | Reason |
|-----|--------|--------|
| `EmployeeReportTO` (rest-types) | +additive fields (`#[serde(default)]`): `voluntary_hours_per_contract_week: f32`, `committed_voluntary_target: f32`, `voluntary_balance: f32` | F1 + F2 in `page/employee_details.rs` |
| `ShortEmployeeReportTO` (rest-types) | +additive: `has_pending_rebooking: bool`, `pending_rebooking_id: Option<Uuid>` | F5 banner on `page/employees.rs` — Fat Backend flag |
| **new** `VoluntaryStatsTO` | Ist / Soll / Konto zusammengefasst | F1/F2 dedicated endpoint (alternative to bloating `EmployeeReportTO`) |
| **new** `RebookingSuggestionTO` | full modal payload (all pre-computed numbers + IST/DANN pair) | F5 modal |
| **new** `ManualRebookingRequestTO` | `{ sales_person_id, hours, direction, description }` | F3 |
| **new** `RebookingBatchTO` / `RebookingBatchEntryTO` | mirrors DB tables 1:1 | REST list/detail |

Additive-only, `#[serde(default)]` on new fields — wire-compat precedent from v2.5 `sales_person_absences`.

## Sources

- `CLAUDE.md` (repo root) — Service-Tier-Konvention (Basic vs. Business-Logic), Snapshot-Bump-Kontrakt, Docs-Freshness-Gate. **HIGH confidence** — canonical.
- `.planning/PROJECT.md` — Fat Backend, Thin Client principle; v2.5 shipped context; snapshot version = 12. **HIGH**.
- `shifty_bin/src/main.rs` (1466 lines total; DI wiring reference) — every service alias + construction order for the 30+ existing services. Precedents: `PdfExportScheduler` (lines 1307–1315), `ToggleService` (1029–1034), Basic-before-BL discipline. **HIGH**.
- `service_impl/src/pdf_export_scheduler.rs` — `tokio_cron_scheduler` pattern to copy for `VoluntaryRebookingScheduler`. **HIGH**.
- `service_impl/src/reporting.rs` — location for the new pure fn `committed_voluntary_target_for_year`; precedent helpers `committed_voluntary_for_calendar_week` / `apply_weekly_cap`. **HIGH**.
- `service_impl/src/extra_hours.rs` — internal-call pattern via `Authentication::Full` to reuse from reconciliation. **HIGH**.
- `service_impl/src/booking_information.rs` — precedent for BL-service consuming multiple BL-services (Reporting + Absence) without cycle; per-person weekly loop shape. **HIGH**.
- `service/src/extra_hours.rs` — `ExtraHoursCategory::VolunteerWork` + `ExtraWork` enum variants (the F3/F4/F5 double-entry pair uses exactly these). **HIGH**.
- `.planning/milestones/v2.2-ROADMAP.md` line 127+136 — confirms `tokio-cron-scheduler` is already a workspace dep since v2.2; no new Cargo dep needed. **HIGH**.
- `.planning/milestones/v2.5-ROADMAP.md` — precedent for additive DTO extension + no snapshot bump. **HIGH**.
- MEMORY `feedback_atomic_repoint_no_double_count` — atomicity rule for re-point/rebooking-style operations. **HIGH**.
- MEMORY `feedback_dioxus_proxy_for_new_backend_endpoints` — Dioxus.toml proxy checklist. **HIGH**.
- MEMORY `feedback_stichtag_rollout_legacy_semantics` — Stichtag-Rollout pattern (justifies the F4 active_from gate). **HIGH**.
