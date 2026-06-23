# Architecture Research — v1.4 Committed Voluntary Capacity

**Domain:** Threading a new time-versioned field `committed_voluntary: f32` (D-01 / Variante B) through the existing layered Rust backend (REST → Service-trait → DAO-trait → SQLite) plus the Dioxus frontend, with no-double-count reporting integration and a billing-period snapshot bump.
**Researched:** 2026-06-22
**Confidence:** HIGH (every integration point below was located in this repo by reading the actual source; line numbers are real and current)

This is an **integration / build-order** research document, not a greenfield architecture survey. The existing layered architecture (see `CLAUDE.md` § Layered Architecture, § Service-Tier-Konventionen, § Billing Period Snapshot Schema Versioning) is taken as fixed and is **not** redesigned. The job is to map the field onto the existing layers and order the phases against compile dependencies.

---

## Standard Architecture (existing, do not redesign)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  shifty-dioxus/ (WASM frontend)                                              │
│   page/weekly_overview.rs ─ state/weekly_overview.rs ─ loader.rs ─ api.rs    │
│   component/contract_modal.rs (EmployeeWorkDetailsForm) ─ state/employee_…   │
│                          ▲ consumes rest-types DTOs (shared crate)           │
└──────────────────────────│──────────────────────────────────────────────────┘
                           │ HTTP/REST (JSON)
┌──────────────────────────│──────────────────────────────────────────────────┐
│  Backend workspace        ▼                                                   │
│  rest/  ── rest-types/ (EmployeeWorkDetailsTO, WeeklySummaryTO, …)            │
│    │           ▲ From / TryFrom                                               │
│    ▼           │                                                              │
│  service/ (traits)  ── service_impl/ (reporting.rs, booking_information.rs,   │
│    │                    billing_period_report.rs ← SNAPSHOT VERSION)          │
│    ▼                                                                          │
│  dao/ (traits) ── dao_impl_sqlite/ (employee_work_details.rs)                 │
│    ▼                                                                          │
│  SQLite (migrations/sqlite/*.sql + compile-time-checked .sqlx/)              │
└───────────────────────────────────────────────────────────────────────────────┘
```

### Field lifecycle (where `committed_voluntary` lives at each layer)

| Layer | Type that gains the field | File |
|-------|---------------------------|------|
| SQLite | new column `committed_voluntary REAL NOT NULL DEFAULT 0` | new `migrations/sqlite/<ts>_add-committed-voluntary-to-employee-work-details.sql` |
| DAO row | `EmployeeWorkDetailsDb.committed_voluntary: f64` + 4 `query!`/`query_as!` blocks + `TryFrom` | `dao_impl_sqlite/src/employee_work_details.rs` |
| DAO entity | `EmployeeWorkDetailsEntity.committed_voluntary: f32` | `dao/src/employee_work_details.rs` |
| Service domain | `EmployeeWorkDetails.committed_voluntary: f32` + two conversions | `service/src/employee_work_details.rs` |
| DTO | `EmployeeWorkDetailsTO.committed_voluntary: f32` + two `From` impls | `rest-types/src/lib.rs` |
| Frontend state | `state::employee_work_details::EmployeeWorkDetails.committed_voluntary` + two `TryFrom` | `shifty-dioxus/src/state/employee_work_details.rs` |

---

## Integration Points — Layer-by-Layer Touch List (Q1)

### 1. SQLite migration + `.sqlx`

- **New migration file** `migrations/sqlite/<timestamp>_add-committed-voluntary-to-employee-work-details.sql`. Template directly from the existing cap-flag migration (`migrations/sqlite/20260426120000_add-cap-flag-to-employee-work-details.sql`):
  ```sql
  ALTER TABLE employee_work_details
  ADD COLUMN committed_voluntary REAL NOT NULL DEFAULT 0;
  ```
  Use `REAL` (not `INTEGER`) — `expected_hours` is already `REAL`/`f64` in the DB and `f32` in the entity; mirror that exactly so the `as f32` / `as f64` casts in the DAO stay consistent.
- **`.sqlx` regeneration is mandatory and is the first compile gate.** SQLx uses compile-time `query_as!`/`query!` macros. Adding the column to the four SELECT lists and the INSERT/UPDATE in `dao_impl_sqlite/src/employee_work_details.rs` changes the query fingerprints, so `cargo sqlx prepare` must regenerate `.sqlx/query-*.json`. Run after the DB has the migration applied (`sqlx database reset` is DESTRUCTIVE — per MEMORY use `sqlx migrate run` to apply additively, then `cargo sqlx prepare`). This is on NixOS → wrap sqlx calls in `nix develop`.

### 2. DAO trait + sqlite impl + entity `TryFrom`

- **`dao/src/employee_work_details.rs`** — add `pub committed_voluntary: f32` to `EmployeeWorkDetailsEntity` (struct only; the trait method signatures already pass `&EmployeeWorkDetailsEntity`, so no trait-method changes).
- **`dao_impl_sqlite/src/employee_work_details.rs`** — five edits:
  1. `EmployeeWorkDetailsDb` struct: add `pub committed_voluntary: f64` (DB is `f64`, mirrors `expected_hours`).
  2. `TryFrom<&EmployeeWorkDetailsDb> for EmployeeWorkDetailsEntity`: `committed_voluntary: working_hours.committed_voluntary as f32`.
  3. `all()`, `find_by_id()`, `find_by_sales_person_id()`, `find_for_week()` — add `committed_voluntary` to each of the four SELECT column lists.
  4. `create()` — add a `let committed_voluntary = entity.committed_voluntary as f64;`, the column in the INSERT list, a `?` placeholder, and the bind in the `query!` arg list.
  5. `update()` — add `committed_voluntary = ?` to the SET clause + the bind. **Note:** the existing `update()` is a *partial* update (it does NOT write `monday..sunday`, `sales_person_id`, etc.). `committed_voluntary` is mutable over time → it MUST be in the UPDATE set clause, alongside `expected_hours` and `cap_planned_hours_to_expected` which are already there.

### 3. Service domain struct + `From` impls

- **`service/src/employee_work_details.rs`** — three edits, all mechanical:
  1. `EmployeeWorkDetails` struct: `pub committed_voluntary: f32`.
  2. `From<&EmployeeWorkDetailsEntity> for EmployeeWorkDetails`: copy the field.
  3. `TryFrom<&EmployeeWorkDetails> for dao::…::EmployeeWorkDetailsEntity`: copy the field.
  The `EmployeeWorkDetailsService` trait itself needs **no** signature change (create/update already take `&EmployeeWorkDetails`).

### 4. rest-types DTO(s) — which EmployeeWorkDetails DTO family member?

- **Only `EmployeeWorkDetailsTO`** (rest-types/src/lib.rs:597) gets the field. It is the *single* DTO for this entity — used by create, update, find-for-week, and find-for-sales-person handlers alike. There is no separate "create vs read" DTO split for this entity.
  - Add `#[serde(default)] pub committed_voluntary: f32` (mirror the `#[serde(default)]` already on `cap_planned_hours_to_expected` so old frontend payloads without the field still deserialize → safe rollout).
  - Update both conversion impls: `From<&EmployeeWorkDetails> for EmployeeWorkDetailsTO` (rest-types/src/lib.rs:636) and `From<&EmployeeWorkDetailsTO> for EmployeeWorkDetails` (rest-types/src/lib.rs:674).
- **Do NOT touch** `WorkingHoursReportTO` (l.460), `EmployeeReportTO` (l.523), `WorkingHoursDayTO` (l.443), `WeeklySummaryTO` (l.901) at the *data-model* phase — those are reporting/summary DTOs and only change later when the reporting output surfaces committed capacity (see Q2/Q4).

### 5. REST handler(s) + OpenAPI surface — **important finding: NO `#[utoipa::path]` impact**

- **Handlers that read/write `EmployeeWorkDetails`:** all five in `rest/src/employee_work_details.rs` — `create_working_hours`, `update_working_hours`, `delete_employee_work_details`, `get_working_hours_for_week`, `get_working_hours_for_sales_person`. They serialize/deserialize `EmployeeWorkDetailsTO` directly via `serde_json`, so once the DTO has the field they transport it **with zero handler-body changes**.
- **OpenAPI surface test impact: NONE.** Verified directly:
  - `EmployeeWorkDetailsTO` has **no `ToSchema` derive** (rest-types/src/lib.rs:596 is `#[derive(Debug, Serialize, Deserialize)]` only — contrast with `WeeklySummaryTO` and the report TOs which also lack it, vs. `SpecialDayTypeTO` which has it).
  - The handlers carry **no `#[utoipa::path]` annotations**.
  - The routes are mounted (`rest/src/lib.rs:588` `/working-hours`, `:590` `/employee-work-details`) but are **not registered in any `ApiDoc`** struct nest list.
  - **Consequence:** This contradicts the general CLAUDE.md guideline ("REST endpoints require `#[utoipa::path]`"). For *this specific endpoint family* the OpenAPI integration does not exist today, so adding the field does **not** require touching utoipa schemas and will **not** break any OpenAPI/Swagger surface snapshot test. Do not let a planner add a phantom "update OpenAPI" task here. (If the team wants to *retrofit* OpenAPI annotations, that is an orthogonal cleanup, explicitly out of v1.4 scope.)
- **Integration test that WILL flex:** `shifty_bin/src/integration_test/employee_work_details_update.rs` roundtrips `EmployeeWorkDetails` through the real SQLite DAO (it already guards the `expected_hours` f32→f64 cast bug). Extend `ewd_template()` with `committed_voluntary` and add an assertion that a fractional value roundtrips — this is the natural home for the data-model test (satisfies the global "always have tests" rule).

---

## Reporting Integration — the no-double-count formula (Q2)

### Where the computation belongs

The no-double-count logic belongs in **`service_impl/src/reporting.rs`**, at the **per-week `volunteer_hours` aggregation sites**. There are **three** of them and all three must change identically (they are parallel implementations of the same per-week math):

| Method | volunteer_hours site | line |
|--------|----------------------|------|
| `get_reports_for_all_employees` | `manual VolunteerWork sum + auto_volunteer_hours` | reporting.rs:362–367 |
| `get_reports_for_employee` (detailed) | `manual_volunteer_hours + auto_volunteer_hours` | reporting.rs:854 |
| (the per-employee ShortEmployeeReport push) | same pattern | reporting.rs:781–788, 852–854 |

Today `volunteer_hours = manual_VolunteerWork_extra_hours + auto_volunteer_hours` (where `auto_volunteer_hours` is the auto-cap overflow from `apply_weekly_cap`, reporting.rs:94). This is the **actual_volunteer** quantity from the todo.

The D-01 formula maps onto the existing variables as:

- **Available/committed capacity** = `expected + committed_voluntary`. `expected_hours` is computed per-week at reporting.rs:851 (`planned_hours - absence - derived`). The new committed capacity is a **separate display axis**, not folded into `expected_hours` (D-01 explicitly decouples them — no `committed >= expected` invariant).
- **Surplus / over-commitment** = `max(0, actual_volunteer − committed_voluntary)`. I.e. replace the raw `volunteer_hours` that flows into the report with `max(0.0, (manual + auto_volunteer) − committed_voluntary_for_week)`, and expose `committed_voluntary_for_week` as its own field so the year view can show "5 committed (covered) + 2 surplus".
- `committed_voluntary_for_week` is read from the same `working_hours` (`Arc<[EmployeeWorkDetails]>`) the method already loads via `employee_work_details_service` (reporting.rs:129) and already filters per week via `find_working_hours_for_calendar_week` (reporting.rs:77) — **scope-gate it on `cap_planned_hours_to_expected == true`** per the todo (only capped/volunteer persons). Pattern mirrors the existing `cap_active = …any(|wh| wh.cap_planned_hours_to_expected)` (reporting.rs:265, 814, 1001).

### Is reporting the right (Business-Logic) tier? — YES

`ReportingServiceImpl` is squarely a **Business-Logic service** per CLAUDE.md § Service-Tier-Konventionen: it consumes other domain services (`ExtraHoursService`, `ShiftplanReportService`, `EmployeeWorkDetailsService`, `SalesPersonService`, `CarryoverService`, `AbsenceService` — reporting.rs:59–74). A cross-entity invariant ("committed vs actual volunteer, no double count") is exactly what the business-logic tier exists for. The `EmployeeWorkDetailsService` is a **Basic** service (entity manager) and correctly stays dumb about reporting.

### New service dependencies needed? — NO

The computation **reuses what `reporting.rs` already pulls in**. `committed_voluntary` rides along on the `EmployeeWorkDetails` records that `employee_work_details_service.all()` / `.find_…` already return (reporting.rs:129–158). No new DI dependency, no change to the `ReportingServiceDeps` macro block, no change to construction order in `shifty_bin/src/main.rs`. This keeps the change low-risk and contained.

### Report output struct(s) that gain a field

To surface committed capacity *separately* (todo req 4), add a field (e.g. `committed_voluntary_hours: f32`) to the report structs and propagate to their TOs:
- `service::reporting::EmployeeReport` / `GroupedReportHours` / `ShortEmployeeReport` (whichever the year view consumes — `WeeklySummary` is built from `ReportingService.get_week` reports, see Q4).
- Corresponding TOs: `EmployeeReportTO`, `WorkingHoursReportTO` only if the detail view needs it; **`WeeklySummaryTO` is the one the Jahresansicht actually reads** (Q4).

---

## Snapshot Versioning — bump confirmed (Q3)

**`CURRENT_SNAPSHOT_SCHEMA_VERSION` MUST bump from 7 → 8.** Confirmed against CLAUDE.md § Billing Period Snapshot Schema Versioning and the actual writer.

- The constant lives at `service_impl/src/billing_period_report.rs:74` (`pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 7;`).
- The affected `value_type` is **`BillingPeriodValueType::Volunteer`** (billing_period_report.rs:240–248). It is written from `report_delta.volunteer_hours` / `report_start/end/end_of_year.volunteer_hours` — i.e. **directly from the `ReportingService` `volunteer_hours` field this feature changes**.
- This is **not a new `BillingPeriodValueType`** — it is a **changed computation of an existing one**. The trigger in CLAUDE.md that fires: *"Change the computation that produces an existing value_type … anything that would make a fresh re-computation disagree with an older snapshot for the same period"* and *"Change the input set the computation reads from."* Both apply: the volunteer value now subtracts `committed_voluntary` (new input from `EmployeeWorkDetails`) and clamps at 0.
- A validator re-running the live computation on a v7 snapshot would get a different `Volunteer` value the moment any person has `committed_voluntary > 0`. Without the bump, that drift is indistinguishable from a real data bug — exactly the failure mode the version guards.
- **Add a `- v8:` history entry** to the doc comment above the constant (billing_period_report.rs:38–73), matching the existing v3..v7 prose style, naming the changed value_type (`Volunteer`) and the new input (`EmployeeWorkDetails.committed_voluntary`).

**Placement rule (Q5):** the bump MUST land in the **same commit** as the reporting-formula switch. The version stamp's whole purpose is "was this snapshot written under the same rules I'm using now" — splitting them creates a window where the new formula writes snapshots still stamped v7. Do not bump early (before the formula change) and do not defer.

---

## Frontend Integration (Q4)

### Jahresansicht (`weekly_overview`) — separate committed display

The year view does **not** read `EmployeeWorkDetails` directly. It consumes `WeeklySummaryTO` via:
`page/weekly_overview.rs` → `state/weekly_overview.rs::WeeklySummary` ← `loader.rs::load_weekly_summary_for_year` ← `api::get_weekly_overview` ← backend `GET /booking-information…` → `WeeklySummaryTO` (rest-types/src/lib.rs:901) ← `service::booking_information::WeeklySummary`.

So the committed capacity must be **plumbed up through the summary**, not the contract DTO:

1. **Backend `service_impl/src/booking_information.rs`** — `get_weekly_summary…` (l.100) builds `WeeklySummary` from per-week `ReportingService` reports (`week_report`, l.133). It currently sums `paid_hours` (l.176–178) and `volunteer_hours` (l.141–153/197). Add a `committed_voluntary_hours` accumulation from the reports' new field and put it on the `WeeklySummary` struct (`service/src/booking_information.rs:38`).
2. **`service/src/booking_information.rs`** — add `pub committed_voluntary_hours: f32` to `WeeklySummary`.
3. **`rest-types/src/lib.rs`** — add `committed_voluntary_hours` to `WeeklySummaryTO` (l.901) + its `From<&WeeklySummary>` impl (l.917).
4. **Frontend `state/weekly_overview.rs`** — add `committed_voluntary_hours: f32` to the `WeeklySummary` state struct (l.12) + its `From` mapping (l.30ff).
5. **Frontend `page/weekly_overview.rs`** — render committed **separately** (todo req 4: not vermischt with paid/volunteer). The current cell renders `"💰{paid} | 🤝{volunteer}"` (weekly_overview.rs:103/108). Add a third token (e.g. a `📌`/committed glyph or a labelled column) for committed capacity. Add an i18n key in all three locales (En/De/Cs) per project rule.
6. **`loader.rs::load_weekly_summary_for_year` (l.611)** and `api.rs` need no logic change beyond the DTO carrying the new field (serde-driven).

### Mitarbeiteransicht — "alle"-Filter + unpaid-volunteer EmployeeWorkDetails path

- **Contract editor (where the field is entered):** `shifty-dioxus/src/component/contract_modal.rs` hosts the `EmployeeWorkDetailsForm`; the existing `cap_planned_hours_to_expected` toggle is at contract_modal.rs:382. The `committed_voluntary` numeric input goes **right next to it** (only meaningful when cap is on → can be gated/shown conditionally). The form binds to `state::employee_work_details::EmployeeWorkDetails`, which needs the new field (state/employee_work_details.rs:44 struct + both `TryFrom` at l.145ff and the `→ TO` direction). Browser-test caveat from MEMORY: prefer cargo tests over programmatic input for verifying form state.
- **"alle"-Filter / unpaid-volunteer visibility:** today the employee list and the year-view reporting are **paid-only**:
  - `loader.rs:474` hard-codes `is_paid: Some(true)` in `load_working_hours_minified_for_week`.
  - `reporting.rs:139–142` (`get_reports_for_all_employees`) filters `employees.filter(|e| e.is_paid.unwrap_or(false))`.
  - `booking_information.rs:118–125` builds `volunteer_ids` from **un**paid persons (`!is_paid`) but only for the shiftplan-report volunteer sum — unpaid persons still need an `EmployeeWorkDetails` record to carry `committed_voluntary`.
  - `component/employees_list.rs` filters the list (search + `!inactive`, l.82–88) — no paid filter there, but the data source upstream is paid-gated.
  - **Required:** an "alle"-toggle that, when on, (a) includes unpaid persons in the list/year-view, and (b) lets a pure-volunteer (unpaid) person get an `EmployeeWorkDetails` record so `committed_voluntary` is editable. This is the most design-open part of the milestone — it touches the paid-only assumption baked into reporting and loaders. Flag for deeper phase-level design (see PITFALLS).

---

## Suggested Phase Decomposition + Build Order (Q5)

Ordered strictly by **compile dependency** (each phase compiles green before the next; backend foundation precedes any frontend that consumes it). 4 phases, with the snapshot bump fused into the reporting phase.

```
Phase A  Data-model foundation (backend, bottom-up)
   migration + .sqlx  →  dao entity  →  dao_impl_sqlite  →  service struct+conversions
   →  EmployeeWorkDetailsTO + From impls  →  extend employee_work_details_update integ test
   Gate: cargo build + cargo test + cargo sqlx prepare green. Field transported end-to-end,
         persisted, roundtrip-tested — but inert (read nowhere yet).
   Compile dep: nothing above depends on it failing; .sqlx regen is the first hard gate.

Phase B  Reporting no-double-count  +  Snapshot bump  (SAME phase, SAME commit)
   reporting.rs: all 3 volunteer_hours sites → max(0, actual_volunteer − committed_voluntary),
   gated on cap_planned_hours_to_expected; add committed_voluntary_hours to report structs.
   billing_period_report.rs: CURRENT_SNAPSHOT_SCHEMA_VERSION 7→8 + v8 history note.
   Tests: extend reporting fixtures (service_impl/src/test/reporting_*; billing_period_report test).
   Gate: cargo test green; snapshot validator tests acknowledge v8.
   Compile dep: needs Phase A's field on EmployeeWorkDetails. MUST be one commit (snapshot
                stamp must match the formula that wrote it).

Phase C  Jahresansicht display (booking_information → WeeklySummary(TO) → frontend year view)
   service WeeklySummary + WeeklySummaryTO + state::weekly_overview + page render + i18n×3.
   Gate: cargo test (backend) + cargo build --target wasm32-unknown-unknown (frontend).
   Compile dep: needs Phase B's committed_voluntary_hours on the reports.

Phase D  Contract editor input + "alle"-filter / unpaid-volunteer EWD path
   contract_modal.rs committed_voluntary input + state TryFrom both directions;
   "alle"-toggle relaxing the paid-only filter in list/loader/reporting so unpaid
   volunteers get an EWD record and are selectable. i18n×3.
   Gate: cargo build --target wasm32 + cargo test; manual UAT (frontend-in-scope per PROJECT.md).
   Compile dep: input editing needs the DTO field (Phase A); the "alle" path is the most
                design-open — split out so its risk doesn't block C.
```

### Ordering rationale / compile-dependency notes

- **A before everything:** the field must exist on `EmployeeWorkDetails` (service) before reporting can read it and before the DTO can transport it. `.sqlx` regeneration is the gating step — get it green first.
- **B fuses snapshot bump with the formula switch** — non-negotiable per the snapshot-versioning contract (same commit).
- **C before D:** the *display* (read path, Phase C) is lower-risk and self-contained; the *editing + "alle"-filter* (Phase D) carries the open paid-only-assumption design question. Shipping C first means the year view shows committed capacity (zero for everyone until D lets it be entered) without being blocked by D's design work.
- **Frontend always trails its backend DTO change** in each phase — the consolidated single `rest-types` crate (PROJECT.md: resolved in v1.2) means a missing field breaks the WASM compile, so the frontend edits are forced to stay in sync within the same phase.
- VCS: jj-only commits, GSD auto-commit disabled — user commits each phase manually (CLAUDE.local.md / MEMORY).

---

## Anti-Patterns (specific to this change)

### Anti-Pattern 1: Folding `committed_voluntary` into `expected_hours`
**What people do:** add committed capacity to `expected_hours` so it "just shows up" in available hours.
**Why wrong:** breaks D-01 (Variante B is explicitly additive/separate, no `committed >= expected` invariant), inflates the balance computation (`balance = overall − expected`), and corrupts every existing `Balance`/`ExpectedHours` snapshot value_type — a far bigger snapshot-versioning blast radius.
**Instead:** keep it a separate axis; only the `Volunteer` value_type computation changes.

### Anti-Pattern 2: Changing only one of the three reporting volunteer sites
**What people do:** patch `get_reports_for_all_employees` and forget `get_reports_for_employee` / the ShortEmployeeReport push.
**Why wrong:** the year view and the detail view diverge; snapshots (built from one path) disagree with the live UI (built from another).
**Instead:** change all three volunteer_hours sites (reporting.rs:362, 781–788/852–854) identically; cover with a fixture test that exercises both report entry points.

### Anti-Pattern 3: Adding a phantom OpenAPI/utoipa task for the EWD endpoint
**What people do:** dutifully "add `#[utoipa::path]` / `ToSchema`" per the generic CLAUDE.md rule.
**Why wrong:** this endpoint family is *not* in the OpenAPI surface today (no `ToSchema`, no `#[utoipa::path]`, not in any `ApiDoc`). Adding it is scope creep and risks new surface-snapshot test churn unrelated to v1.4.
**Instead:** only add `committed_voluntary` to the plain serde `EmployeeWorkDetailsTO`. Leave OpenAPI retrofit out of scope.

### Anti-Pattern 4: Splitting the snapshot bump from the reporting change
**What people do:** bump version in a "prep" commit, change the formula in a later commit.
**Why wrong:** snapshots written in between are stamped with a version whose rules they don't actually follow.
**Instead:** one commit, Phase B.

---

## Integration Points Summary Table

| Boundary | File(s) | Change |
|----------|---------|--------|
| DB schema | new `migrations/sqlite/<ts>_add-committed-voluntary-…sql` + `.sqlx/` regen | add `REAL` column |
| DAO entity ↔ row | `dao/src/employee_work_details.rs`, `dao_impl_sqlite/src/employee_work_details.rs` | field + 4 SELECT + INSERT + UPDATE + TryFrom |
| Service domain | `service/src/employee_work_details.rs` | field + 2 conversions |
| DTO | `rest-types/src/lib.rs` (`EmployeeWorkDetailsTO` l.597) | field + 2 From impls |
| REST handlers | `rest/src/employee_work_details.rs` | **no body change**; **no utoipa** |
| Reporting | `service_impl/src/reporting.rs` (l.362, 781, 852) | no-double-count, 3 sites, no new dep |
| Snapshot | `service_impl/src/billing_period_report.rs` (l.74, 240) | bump 7→8, same commit as reporting |
| Year-view backend | `service_impl/src/booking_information.rs` (l.100, 197), `service/src/booking_information.rs` (l.38) | `committed_voluntary_hours` on `WeeklySummary` |
| Year-view DTO | `rest-types/src/lib.rs` (`WeeklySummaryTO` l.901) | field + From |
| Year-view frontend | `shifty-dioxus/src/state/weekly_overview.rs`, `src/page/weekly_overview.rs` (l.103/108) | field + separate render + i18n×3 |
| Contract editor | `shifty-dioxus/src/component/contract_modal.rs` (l.382), `src/state/employee_work_details.rs` | input + 2 TryFrom + i18n×3 |
| "alle"-filter | `shifty-dioxus/src/loader.rs` (l.474), `src/component/employees_list.rs`, `service_impl/src/reporting.rs` (l.139) | relax paid-only; design-open |
| Tests | `shifty_bin/src/integration_test/employee_work_details_update.rs`, `service_impl/src/test/reporting_*`, `…/billing_period_report.rs` | roundtrip + formula + snapshot |

## Sources

- This repo, read 2026-06-22: the 14 files listed above (line numbers verified by `grep`/`Read`).
- `CLAUDE.md` § Layered Architecture, § Service-Tier-Konventionen, § Billing Period Snapshot Schema Versioning.
- `.planning/PROJECT.md` (v1.4 milestone), `.planning/todos/pending/2026-06-22-committed-voluntary-capacity-jahresansicht.md` (D-01 + reqs 1–5).
- MEMORY: sqlx reset-is-destructive / `migrate run` additive, `nix develop`, jj-only commits, Dioxus date-input test caveat.

---
*Architecture / integration research for: v1.4 Committed Voluntary Capacity*
*Researched: 2026-06-22 — Confidence HIGH*
