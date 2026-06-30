# Phase 28: Urlaubsanspruch-Korrektur via Offset (HR, BE+FE) - Research

**Researched:** 2026-06-29
**Domain:** Rust layered backend (dao → dao_impl_sqlite → service → service_impl → rest → rest-types) + Dioxus/WASM frontend; soft-delete CRUD aggregate, billing-snapshot versioning, proration math, auth-context-conditional DTO field hiding.
**Confidence:** HIGH (every claim grounded in real file:line coordinates in this repo)

## Summary

This is an implementation-research task on an existing codebase. All six questions resolve to concrete, verified file:line coordinates. The new `vacation_entitlement_offset` aggregate is a per-(person, year) soft-delete row; the closest structural twin in the repo is **`carryover` (`employee_yearly_carryover`)** — same `(sales_person_id, year)` key, same `created/deleted/update_version` soft-delete columns, same `find_by_sales_person_id_and_year` + `upsert ON CONFLICT(sales_person_id, year)` DAO shape. The REST/`#[utoipa::path]`/`ToSchema`/`error_handler` shape and the HR-permission gate come from `special_day` (full CRUD REST exemplar) and `vacation_balance` (HR-gate exemplar). The new Basic service should use the `gen_service_impl!` macro like `sales_person_unavailable`.

One **load-bearing correction to the CONTEXT** surfaced during tracing: `vacation_days_for_year` (`reporting.rs:803`) feeds `report.vacation_entitlement` (`reporting.rs:853`), which is persisted as **`BillingPeriodValueType::VacationEntitlement`** (`billing_period_report.rs:266-271`) — **not** `VacationDays`. `VacationDays` (`billing_period_report.rs:256-263`) is fed by `report.vacation_days` (actual taken vacation from `week.vacation_days()`), which the off-by-one fix does NOT touch. The CONTEXT's D-28-05 *conclusion* (bump 11→12 is mandatory) is still correct, because `vacation_days_for_year` does feed a persisted value_type — only the value_type *name* in the CONTEXT is wrong. The planner must reference `VacationEntitlement` in the version-bump rationale.

The off-by-one (D-28-04) is confirmed: the year-START branch subtracts `vacation_days * ordinal()/days_in_year` where `ordinal()-1` is correct (Jan-1 start should subtract 0). The year-END branch is **already symmetric and correct** (`1.0 - ordinal/days_in_year` → Dec-31 end gives 0 subtraction) — no off-by-one there.

**Primary recommendation:** Clone `carryover` for the DAO+sqlite layer (add `id` PK + `version` + a partial unique index on `(sales_person_id, year) WHERE deleted IS NULL` per D-28-01), `special_day` for the REST CRUD shape, `vacation_balance` for the HR-gate; make the offset service a `gen_service_impl!` Basic service; construct it in `main.rs` right before `vacation_balance_service` (line 873); bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` to 12 and update the guard test at `billing_period_snapshot_locking.rs:28`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Persist signed offset per person+year | `dao_impl_sqlite` (SQLite) | `dao` (trait) | New table + soft-delete row; mirrors `carryover` |
| Offset CRUD + HR-gate + validation | `service_impl` (Basic service) | `service` (trait) | Entity-manager: only DAO/Permission/Transaction deps (D-28-06) |
| Add offset to entitlement, populate breakdown | `service_impl::vacation_balance` (Business-Logic) | — | Cross-entity; already consumes Carryover, now also Offset (D-28-02/03) |
| Off-by-one proration fix | `service` (`employee_work_details.rs`, pure domain fn) | — | Pure calc on `EmployeeWorkDetails`; feeds both vacation_balance + reporting |
| HR-gated REST endpoints | `rest` (Axum) | `rest-types` (DTO) | Thin wrapper; `#[utoipa::path]` + `error_handler`; perm enforced in service |
| API-level field hiding (offset only for HR) | `service_impl::vacation_balance` | `rest-types` (`Option` fields) | Decided server-side via auth context (D-28-03) |
| Snapshot version bump | `service_impl::billing_period_report` | test guard | Computation of a persisted `value_type` changes (D-28-05) |
| Inline editor + effective-value display | Dioxus `page/absences.rs` | `state/`, `api.rs`, `i18n/` | FE-only; `is_hr` flag gates field render (D-28-07) |

## Standard Stack

No new external dependencies. This phase is built entirely from in-repo patterns and already-present crates (`sqlx`, `axum`, `utoipa`, `mockall`, `time`, `uuid`, `async-trait`, `tokio`; frontend `dioxus`, `reqwest`). **Package Legitimacy Audit and Environment Availability sections are therefore N/A** (no `cargo add`, no new tools). The only "installation" step is applying the new migration (see Migration Application below).

## Architecture Patterns

### System Architecture Diagram

```
                          ┌──────────────────────────────────────────────┐
  HR sets offset  ──PUT──►│ rest/src/vacation_entitlement_offset.rs (NEW) │
  (signed int)            │   #[utoipa::path] + error_handler + ToSchema  │
                          └───────────────┬──────────────────────────────┘
                                          │ context.into()
                                          ▼
                    ┌─────────────────────────────────────────────────────┐
                    │ service_impl::vacation_entitlement_offset (NEW,Basic)│
                    │   gen_service_impl! { Dao, Permission, Clock, Uuid,  │
                    │                        Transaction }                  │
                    │   check_permission(HR_PRIVILEGE) → upsert/soft-delete │
                    └───────────────┬─────────────────────────────────────┘
                                    │ upsert ON CONFLICT(sales_person_id, year)
                                    ▼
                    ┌─────────────────────────────────────────────────────┐
                    │ dao_impl_sqlite::vacation_entitlement_offset (NEW)   │
                    │   TryFrom<&...Db>, WHERE deleted IS NULL, version     │
                    │   table: migrations/sqlite/<ts>_create-...offset.sql  │
                    └─────────────────────────────────────────────────────┘

  GET balance ──►  rest/src/vacation_balance.rs (existing)
   (HR ∨ self)            │ svc.get(sp,year,context,None)
                          ▼
        service_impl::vacation_balance::get  ── HR? = check_permission(HR).is_ok()
                          │   compute_balance():
                          │     entitled = round(Σ vacation_days_for_year)      ← off-by-one FIX here flows
                          │     offset   = OffsetService.read(sp,year)          ← NEW dep (Business-Logic)
                          │     entitled_effective = entitled + offset
                          │     if HR  → offset_days=Some, computed_entitled=Some(entitled)
                          │     else    → both None ; entitled_days = effective for BOTH roles
                          ▼
        VacationBalanceTO { ..., offset_days: Option<i32>, computed_entitled_days: Option<f32> }

  Reporting path (SEPARATE, snapshot-relevant):
        reporting.rs:803 vacation_days_for_year ──► report.vacation_entitlement (:853)
                          └──► billing_period_report.rs:266 BillingPeriodValueType::VacationEntitlement (PERSISTED)
                               ⇒ off-by-one fix changes this value_type ⇒ BUMP 11→12
```

### Recommended Project Structure (new/edited files)

```
dao/src/vacation_entitlement_offset.rs            # NEW: Entity + Dao trait (clone carryover.rs)
dao_impl_sqlite/src/vacation_entitlement_offset.rs # NEW: Db row + TryFrom + impl (clone carryover.rs)
dao/src/lib.rs                                     # register pub mod
dao_impl_sqlite/src/lib.rs                         # register pub mod
migrations/sqlite/<ts>_create-vacation-entitlement-offset.sql  # NEW additive migration
service/src/vacation_entitlement_offset.rs         # NEW: domain + Service trait (#[automock])
service_impl/src/vacation_entitlement_offset.rs    # NEW: gen_service_impl! Basic service + HR gate
service/src/vacation_balance.rs                    # EDIT: VacationBalance += offset_days, computed_entitled_days
service_impl/src/vacation_balance.rs               # EDIT: add Offset dep, add offset, HR-conditional fields
service/src/employee_work_details.rs:158-191       # EDIT: off-by-one fix (ordinal()-1)
service_impl/src/billing_period_report.rs:108      # EDIT: bump 11 → 12 + history doc entry
service_impl/src/test/billing_period_snapshot_locking.rs:28  # EDIT: assert == 12 + new rationale
rest/src/vacation_entitlement_offset.rs            # NEW: HR-gated CRUD REST + ApiDoc
rest/src/vacation_balance.rs                       # (unchanged routes; TO gains fields automatically)
rest-types/src/lib.rs:2042-2080                    # EDIT: TO + 2 From-impls (gated by feature="service-impl")
rest/src/lib.rs                                    # EDIT: mount route + register ApiDoc
shifty_bin/src/main.rs:715,864-873                 # EDIT: offset_dao + offset_service BEFORE vacation_balance
# Frontend
shifty-dioxus/src/state/vacation_balance.rs        # EDIT: += offset/computed
shifty-dioxus/src/page/absences.rs:408,460-491,534 # EDIT: pass is_hr into SelfBody, inline field at VacationStatContract
shifty-dioxus/src/api.rs:669                        # EDIT: HR offset-save call
shifty-dioxus/src/i18n/{mod,en,de,cs}.rs            # EDIT: new labels
```

### Pattern 1: Per-(person, year) soft-delete DAO (CLONE TARGET = `carryover`)

**What:** The new entity is one signed `offset_days` per `(sales_person_id, year)` — structurally identical to `CarryoverEntity`.
**Why this exemplar:** Smallest aggregate whose KEY matches (per person+year). `special_day` is keyed by date; `sales_person_unavailable` is per-week multi-row. Carryover is the only per-(person,year) soft-delete aggregate.

DAO trait — `dao/src/carryover.rs` (34 lines, the whole file):
```rust
// Source: dao/src/carryover.rs:5-34
#[derive(Clone, Debug, PartialEq)]
pub struct CarryoverEntity {
    pub sales_person_id: Uuid,
    pub year: u32,
    pub carryover_hours: f32,
    pub vacation: i32,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait CarryoverDao {
    type Transaction: crate::Transaction;
    async fn find_by_sales_person_id_and_year(&self, sales_person_id: Uuid, year: u32,
        tx: Self::Transaction) -> Result<Option<CarryoverEntity>, DaoError>;
    async fn upsert(&self, entity: &CarryoverEntity, process: &str,
        tx: Self::Transaction) -> Result<(), DaoError>;
}
```
For the offset, replace `carryover_hours: f32, vacation: i32` with `offset_days: i32`, and add `id: Uuid` (D-28-01 wants an `id` column that `carryover` lacks — see "version optimistic-lock" note below).

sqlite impl — `dao_impl_sqlite/src/carryover.rs:24-115` shows the three required mechanics:
- **`TryFrom<&CarryoverDb>` row mapping** (`:24-42`): `Uuid::from_slice(&db.sales_person_id)?`, `db.year as u32`, `PrimitiveDateTime::parse(&db.created, &Iso8601::DATE_TIME)?`, `deleted` via `.map(...).transpose()?`, `Uuid::from_slice(&db.update_version)?`.
- **`WHERE ... deleted IS NULL`** in the read (`:67-69`): `SELECT ... WHERE sales_person_id = ? AND year = ? AND deleted IS NULL`.
- **sqlx compile-time `query!`/`query_as!`** (`:65`, `:96`): macros checked against the local DB at compile time → migration MUST be applied before `cargo build` (see Migration Application).
- **upsert** (`:96-99`): `INSERT INTO ... VALUES(...) ON CONFLICT(sales_person_id, year) DO UPDATE SET ...` — the natural write model for "one offset per person+year".

### Pattern 2: Basic service via `gen_service_impl!` with permission gate (EXEMPLAR = `sales_person_unavailable`, gate from `special_day`/`vacation_balance`)

**What:** New Basic service `VacationEntitlementOffsetService` — Entity-Manager, only DAO/Permission/Clock/Uuid/Transaction deps (D-28-06), no domain-service deps → no cycle.

`gen_service_impl!` Basic skeleton — `service_impl/src/sales_person_unavailable.rs:17-26`:
```rust
// Source: service_impl/src/sales_person_unavailable.rs:17-26
gen_service_impl! {
    struct SalesPersonUnavailableServiceImpl: SalesPersonUnavailableService = SalesPersonUnavailableServiceDeps {
        SalesPersonUnavailableDao: SalesPersonUnavailableDao<Transaction = Self::Transaction> = sales_person_unavailable_dao,
        SalesPersonService: ...,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```
For the offset service, DROP `SalesPersonService` (no self-check needed — HR-only writes) and keep `Dao, Permission, Clock, Uuid, Transaction`.

`Option<Transaction>` + `use_transaction`/`commit` pattern — `service_impl/src/vacation_balance.rs:102-129`:
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
// ... work ...
self.transaction_dao.commit(tx).await?;
```

HR-gate — `service_impl/src/vacation_balance.rs:46,113-114`:
```rust
use service::permission::{Authentication, HR_PRIVILEGE};
self.permission_service.check_permission(HR_PRIVILEGE, context.clone()).await?;  // returns Result<(),_>
```
Create/delete guard + version stamping pattern — `service_impl/src/special_days.rs:84-93,119-123`:
```rust
if !entity.id.is_nil() { return Err(ServiceError::IdSetOnCreate); }
if !entity.version.is_nil() { return Err(ServiceError::VersionSetOnCreate); }
entity.id = self.uuid_service.new_uuid("...::create id");
entity.version = self.uuid_service.new_uuid("...::create version");
// delete = soft delete:
entity.deleted = Some(self.clock_service.date_time_now());
entity.version = self.uuid_service.new_uuid("...");
```

### Pattern 3: HR-conditional DTO fields decided in the service (D-28-03)

**What:** `offset_days`/`computed_entitled_days` populated ONLY when caller is HR; `None` for self-only callers; `entitled_days` is ALWAYS the effective value.

`get` already computes the HR result — `service_impl/src/vacation_balance.rs:112-121`:
```rust
let (hr, sp) = join!(
    self.permission_service.check_permission(HR_PRIVILEGE, context.clone()),
    self.sales_person_service.verify_user_is_sales_person(sales_person_id, context, tx.clone().into()),
);
hr.or(sp)?;                 // authorization gate (HR ∨ self)
```
**The HR flag is simply `hr.is_ok()`.** Capture it before `hr.or(sp)?` consumes it, e.g. `let is_hr = hr.is_ok(); hr.or(sp)?;`, then pass `is_hr` into `compute_balance` so it can set the two `Option` fields. `get_team` (`:131-157`) is HR-only, so there `is_hr = true` unconditionally.

### Anti-Patterns to Avoid
- **Putting offset logic in the offset DAO/Service that reaches into VacationBalance** — would create a cycle. Offset service stays Basic; only `VacationBalanceService` (Business-Logic) reads the offset (D-28-06).
- **Adding offset BEFORE `.round()`** — D-28-02 mandates `round(sum) + offset` (integer day correction), not `round(sum + offset_as_float)`. Add after the `.round()` at `vacation_balance.rs:186-191`.
- **Leaking the offset to self-callers via the DTO** — never serialize raw offset for non-HR; decide server-side (D-28-03).
- **Bumping the snapshot version for the offset mechanism** — the offset touches only `vacation_balance.rs`, which is NOT in the billing snapshot path. The bump is triggered SOLELY by the off-by-one fix to `vacation_days_for_year`.
- **Hardcoding `12` in multiple places** — the constant `CURRENT_SNAPSHOT_SCHEMA_VERSION` is the single source; only the guard test literal at `billing_period_snapshot_locking.rs:28` must be updated alongside it.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Per-(person,year) soft-delete row | Custom table/DAO from scratch | Clone `carryover` (`dao*/carryover.rs`) | Exact key + soft-delete + upsert already proven |
| HR-gated REST CRUD | Hand-rolled status mapping | `error_handler(...)` + `#[utoipa::path]` (`rest/src/special_day.rs`) | Consistent error→HTTP, OpenAPI, ToSchema |
| Auth HR check | Re-deriving roles | `check_permission(HR_PRIVILEGE, ctx)` (`vacation_balance.rs:113`) | Existing RBAC; `.is_ok()` = HR flag |
| Service DI wiring | Manual trait plumbing | `gen_service_impl!` macro (`sales_person_unavailable.rs:17`) | Repo convention; deterministic deps |
| Transaction mgmt | Manual begin/commit | `use_transaction`/`commit` (`vacation_balance.rs:109,127`) | `Option<Transaction>` contract |
| Snapshot drift detection | New mechanism | Bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` + guard test | CLAUDE.md mandated mechanism |

**Key insight:** Every layer of this phase has a 1:1 in-repo exemplar. The work is mechanical mirroring, not design.

## Runtime State Inventory

> This phase is greenfield-additive (new table + new fields + one bug fix), not a rename/refactor/migration. No stored keys, service config, OS-registrations, secrets, or build artifacts carry a renamed string.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — new table only; existing rows unaffected (offset defaults to absent → 0) | none |
| Live service config | None — verified: no external service holds offset state | none |
| OS-registered state | None | none |
| Secrets/env vars | None new (uses existing `DATABASE_URL`) | none |
| Build artifacts | sqlx query cache (`.sqlx/` if offline mode) and compiled binary regenerate on build; new migration must be applied to local DB before `cargo build` (compile-time `query!`) | apply migration |

**Note on existing snapshots:** Bumping 11→12 is the mechanism that lets validators treat pre-bump (v11) `VacationEntitlement` snapshots as "older schema" and skip re-validation — no data migration of existing snapshot rows is needed (write-once; the version column records the rules under which each was written).

## Common Pitfalls

### Pitfall 1: Wrong persisted value_type in the bump rationale
**What goes wrong:** CONTEXT D-28-05 says the off-by-one fix flows into `BillingPeriodValueType::VacationDays`. It actually flows into **`VacationEntitlement`**.
**Why it happens:** `reporting.rs` has TWO similarly-named locals: `vacation_days` (taken vacation, `:787-796` via `week.vacation_days()`) and `vacation_entitlement` (contract entitlement, `:801-805` via `vacation_days_for_year`). They map to two different persisted value_types (`billing_period_report.rs:256-263` vs `:266-271`).
**How to avoid:** In the bump doc-comment and guard-test rationale, name `VacationEntitlement` (and note `VacationDays` is unaffected). The bump itself is still mandatory.
**Warning signs:** A guard-test message citing `VacationDays` as the changed type.

### Pitfall 2: `id` column absent in the carryover twin
**What goes wrong:** Carryover uses a composite PK `(sales_person_id, year)` and has NO `id` column (`migrations/sqlite/20241215063132_...:10`). D-28-01 specifies an `id` for the offset table.
**Why it happens:** Blindly cloning carryover's schema drops the `id`.
**How to avoid:** Offset table = carryover columns PLUS `id BLOB(16) NOT NULL PRIMARY KEY` and `offset_days INTEGER NOT NULL`, with a **partial unique index** `CREATE UNIQUE INDEX ... ON vacation_entitlement_offset(sales_person_id, year) WHERE deleted IS NULL` (Claude's-discretion item in D-28). Keep `update_version BLOB NOT NULL`, `update_process TEXT NOT NULL`, `created TEXT NOT NULL`, `deleted TEXT`.
**Warning signs:** sqlx error on a SELECT that references `id` not in the table.

### Pitfall 3: `From` impls for the TO are feature-gated
**What goes wrong:** Adding the two new `Option` fields only to the struct but forgetting the `#[cfg(feature = "service-impl")]`-gated `From` impls breaks the backend build; conversely editing them without the gate breaks the WASM frontend build.
**Why it happens:** `rest-types/src/lib.rs:2053,2068` gate both `From`s behind `feature = "service-impl"` so the WASM build (`default-features = false`) doesn't pull `service` (documented at `:2036`).
**How to avoid:** Edit all THREE: the `VacationBalanceTO` struct (`:2042`), `From<&VacationBalance>` (`:2053`), `From<&VacationBalanceTO>` (`:2068`). New fields are `Option` so the FE-only build round-trips fine.
**Warning signs:** `cargo build --target wasm32-unknown-unknown` fails pulling `service`.

### Pitfall 4: Off-by-one only at year START, not year END
**What goes wrong:** Symmetrically "fixing" the year-END branch would introduce a new bug.
**Why it happens:** The two branches use different formulas (see Off-by-one analysis below).
**How to avoid:** Fix ONLY `:173` (`ordinal()` → `ordinal()-1`). Leave `:182-184` as-is. Add a regression test pinning Dec-31 end → no subtraction (already correct) to prevent a future "symmetry" regression.

### Pitfall 5: Forgetting clippy gate
**What goes wrong:** `cargo test`/`cargo build` pass but `nix build`/CI fail.
**How to avoid:** Run `cargo clippy --workspace -- -D warnings` before commit (CLAUDE.md hard gate). Frontend clippy runs from the BACKEND shell (E0514 in dioxus shell).

## Code Examples

### Off-by-one fix (D-28-04) — `service/src/employee_work_details.rs:158-191`
Current (buggy) year-START branch:
```rust
// Source: service/src/employee_work_details.rs:171-178  (BUG at :173)
if from_year == year {
    if let Ok(from_date) = self.from_date() {
        let relation = from_date.to_date().ordinal() as f32          // BUG: ordinal()==1 for Jan-1
            / time::util::days_in_year(year as i32) as f32;
        days -= self.vacation_days as f32 * relation as f32;          // subtracts ~vacation*1/365 too much
    }
}
```
Fix: the days strictly BEFORE the start are `ordinal()-1` (Jan-1 → 0 days before → 0 subtraction):
```rust
let relation = (from_date.to_date().ordinal() as f32 - 1.0)
    / time::util::days_in_year(year as i32) as f32;
days -= self.vacation_days as f32 * relation;
```
Year-END branch is **already correct** — `service/src/employee_work_details.rs:180-188`:
```rust
if to_year == year {
    if let Ok(to_date) = self.to_date() {
        let relation = 1.0
            - to_date.to_date().ordinal() as f32                      // Dec-31 → ordinal==365
                / time::util::days_in_year(year as i32) as f32;       // relation = 1 - 365/365 = 0 ✓
        days -= self.vacation_days as f32 * relation as f32;          // subtracts days AFTER end; Dec-31 → 0 ✓
    }
}
```
**Full-year contract (1.1.–31.12.) expected result after fix:** start branch subtracts 0, end branch subtracts 0 → `vacation_days_for_year == vacation_days` (no proration). Before the fix it returned `vacation_days * (1 − 1/365)` ≈ `vacation_days − 0.05·vacation_days`, which is what occasionally tipped the `.round()` from 18 to 17.

### Snapshot bump (D-28-05) — `service_impl/src/billing_period_report.rs:108`
```rust
// CURRENT (verified):
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 11;   // line 108
// → change to 12, and ADD a history doc line above it, e.g.:
// - v12: Phase 28 (VAC-OFFSET-01) — off-by-one fix in
//   EmployeeWorkDetails::vacation_days_for_year (Jan-1 start no longer subtracts
//   ~1/365). This changes BillingPeriodValueType::VacationEntitlement
//   (reporting.rs:853 ← :803). Validators MUST treat v11 snapshots as older schema
//   and skip vacation-entitlement re-validation. NOTE: VacationDays (taken) is
//   UNAFFECTED — only VacationEntitlement (contract aliquot) changes.
```
Persisted mapping that justifies the bump — `service_impl/src/billing_period_report.rs:265-272`:
```rust
billing_period_values.insert(
    BillingPeriodValueType::VacationEntitlement,
    BillingPeriodValue {
        value_delta:    report_delta.vacation_entitlement,      // ← from vacation_days_for_year
        value_ytd_from: report_start.vacation_entitlement,
        value_ytd_to:   report_end.vacation_entitlement,
        value_full_year: report_end_of_year.vacation_entitlement,
    },
);
```

### Guard test to update — `service_impl/src/test/billing_period_snapshot_locking.rs:25-38`
```rust
#[test]
fn test_snapshot_schema_version_pinned() {
    assert_eq!(
        CURRENT_SNAPSHOT_SCHEMA_VERSION, 11,   // ← change to 12 + rewrite the message rationale to Phase 28
        "CURRENT_SNAPSHOT_SCHEMA_VERSION muss 11 sein nach Phase 25 ...");
}
```
(`test_billing_period_value_type_surface_locked` at `:46-69` needs NO change — `VacationEntitlement` is already an enumerated arm at `:63`; no new value_type is added.)

### Offset addition (D-28-02) — `service_impl/src/vacation_balance.rs:186-191,256-267`
```rust
let entitled_days: f32 = work_details.iter()
    .filter(|wd| wd.deleted.is_none())
    .map(|wd| wd.vacation_days_for_year(year))
    .sum::<f32>()
    .round();                                  // :191 — computed (pre-offset) value
// NEW: read offset (one read per person+year), add after round:
let offset_days: i32 = /* OffsetService.find(sales_person_id, year).map(|o| o.offset_days).unwrap_or(0) */;
let entitled_effective = entitled_days + offset_days as f32;
// remaining_days uses the EFFECTIVE entitlement (:256-257):
let remaining_days = entitled_effective + carryover_days as f32 - (used_days + planned_days);
Ok(VacationBalance {
    sales_person_id, year,
    entitled_days: entitled_effective,                              // ALWAYS effective (both roles)
    offset_days: if is_hr { Some(offset_days) } else { None },      // NEW (D-28-03)
    computed_entitled_days: if is_hr { Some(entitled_days) } else { None }, // NEW pre-offset value
    carryover_days, used_days, planned_days, remaining_days,
})
```
Wire the new dep into the `gen_service_impl!` block at `service_impl/src/vacation_balance.rs:57-67` (add `VacationEntitlementOffsetService: ... = vacation_entitlement_offset_service,`).

### DI ordering (Question 5) — `shifty_bin/src/main.rs`
Verified construction order: `carryover_service` is built at **`main.rs:864`**, `vacation_balance_service` at **`main.rs:873`** (carryover before vacation_balance, as the comment at `:867-872` documents). The offset DAO is constructed alongside other DAOs (e.g. `carryover_dao` at `main.rs:715`). **Insert the new offset service between line 866 (end of carryover_service) and line 873 (start of vacation_balance_service)**, then add `vacation_entitlement_offset_service: ...clone()` into the `VacationBalanceServiceImpl { ... }` initializer. The offset service is Basic (only DAO/Permission/Clock/Uuid/Transaction — all available well before line 864) → no forward-reference, no cycle. Also add the `type VacationEntitlementOffsetService = ...` Deps wiring near the other Deps impls (pattern at `:267-308`), and add the offset_dao at the DAO block (`:715` neighbourhood).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Absolute entitlement override | Signed delta offset, survives contract changes | This phase (D-28-01) | Offset "travels" with recomputation |
| UI-only hiding | Server-side (API-level) field hiding via auth context | D-28-03 | Self-callers never receive raw offset |
| (n/a) snapshot v11 (Phase 25 holiday auto-credit) | v12 (off-by-one entitlement fix) | This phase (D-28-05) | Validators skip v11 entitlement re-validation |

**Deprecated/outdated:** SEED open-point #3 ("Urlaub is probably not a billing value_type → no bump") is now **resolved as WRONG** — `vacation_days_for_year` feeds `VacationEntitlement`, which IS a persisted value_type. The CONTEXT correctly overrode this; planner should ignore the SEED's tentative "kein Bump" wording.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The offset table should carry an `id` PK + partial unique index in addition to carryover's columns (D-28-01 names `id`; exact index form is Claude's-discretion) | Pitfall 2 / Structure | Low — both forms work; planner picks. If composite-PK-only chosen, drop `id`. |
| A2 | `upsert ON CONFLICT(sales_person_id, year)` is the chosen write model (vs. full id-based CRUD) | Pattern 1 | Low — operationally simplest; either matches D-28 "dediziertes CRUD" discretion |

All other claims are `[VERIFIED]` against file:line coordinates in this session. No external/registry lookups were needed (no new dependencies).

## Open Questions

1. **`id` PK vs composite PK for the offset table**
   - What we know: carryover uses composite `(sales_person_id, year)` PK with no `id`; D-28-01 lists `id` as a column.
   - What's unclear: whether to keep `id` PK + partial-unique-index, or composite PK like carryover.
   - Recommendation: `id BLOB(16) PRIMARY KEY` + `UNIQUE INDEX (sales_person_id, year) WHERE deleted IS NULL` — keeps row identity for REST `DELETE /{id}` while enforcing one active offset per person+year. Either is acceptable per D-28 discretion.

2. **REST route shape** (Claude's discretion in CONTEXT)
   - Recommendation (matches CONTEXT's own recommendation): a small dedicated `vacation_entitlement_offset` CRUD (`POST` upsert / `DELETE /{id}` or `DELETE /{sp}/{year}`), HR-gated; the GET breakdown stays on the existing `vacation-balance` path (D-28-03). Mirror `rest/src/special_day.rs` for handler shape and `rest/src/lib.rs` for mount + ApiDoc registration.

## Validation Architecture

> `.planning/config.json` not inspected for `nyquist_validation`; treating as enabled (default).

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]`/`#[tokio::test]` + `mockall` (`#[automock]`); in-memory SQLite for integration |
| Config file | none (cargo workspaces) |
| Quick run command | `cargo test -p service_impl vacation` |
| Full suite command | `cargo test --workspace` then `cargo clippy --workspace -- -D warnings` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| VAC-OFFSET-01 | computed 17, offset +1 → entitled 18; −2 → 15; remaining follows | unit | `cargo test -p service_impl offset_calc` | ❌ Wave 0 |
| VAC-OFFSET-01 | Delta survives contract change (17→20 base ⇒ 21 effective) | unit | `cargo test -p service_impl offset_delta` | ❌ Wave 0 |
| D-28-03 | Self-caller → offset/computed = None; HR-caller → Some | unit | `cargo test -p service_impl offset_api_hiding` | ❌ Wave 0 |
| D-28-06b | Non-HR set offset → permission error | unit | `cargo test -p service_impl offset_hr_gate` | ❌ Wave 0 |
| D-28-04 | Full-year contract → `vacation_days_for_year == vacation_days`; Jan-1 start no subtraction; mid-year start = correct fraction; Dec-31 end no subtraction | unit | `cargo test -p service vacation_days_for_year` | ❌ Wave 0 |
| D-28-05 | `CURRENT_SNAPSHOT_SCHEMA_VERSION == 12` pinned | unit | `cargo test -p service_impl test_snapshot_schema_version_pinned` | ✅ exists (`billing_period_snapshot_locking.rs`), update literal |
| D-28-07 | HR detail shows "berechnet + Offset"; user detail shows only effective | manual/browser | dx serve + roundtrip | ❌ manual (per CLAUDE.local note: date/signal inputs hard to drive; verify display via cargo where possible) |

### Sampling Rate
- **Per task commit:** `cargo test -p service_impl vacation && cargo clippy --workspace -- -D warnings`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** full suite green + clippy clean + frontend `cargo build --target wasm32-unknown-unknown` (backend shell) before `/gsd-verify-work`.

### Wave 0 Gaps
- [ ] `service_impl/src/test/vacation_entitlement_offset.rs` — offset CRUD + HR-gate (mockall)
- [ ] `service_impl/src/test/vacation_balance.rs` (or new file) — offset addition + API-hiding cases
- [ ] `service/src/employee_work_details.rs` `#[cfg(test)] mod` — off-by-one regression (full-year/start/end)
- [ ] Update `service_impl/src/test/billing_period_snapshot_locking.rs:28` literal 11→12 + rationale
- [ ] (Framework already present — no install needed)

## Security Domain

> `security_enforcement` not explicitly configured; included for completeness.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes (indirect) | Existing OIDC/mock; `Authentication<Context>` passed through |
| V4 Access Control | **yes** | `check_permission(HR_PRIVILEGE)` on all offset writes (D-28-06b) AND on the offset-breakdown read (D-28-03); HR ∨ self on the balance GET |
| V5 Input Validation | yes | `offset_days: i32` — bounded integer; reject non-integer; existing `ServiceError::IdSetOnCreate`/`VersionSetOnCreate` guards |
| V6 Cryptography | no | none |

### Known Threat Patterns
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Self-caller reads another's / own offset via raw API | Information Disclosure | Server-side `Option` nulling for non-HR (D-28-03) — never UI-only |
| Non-HR sets/edits offset | Elevation of Privilege | `HR_PRIVILEGE` gate on every write path (D-28-06b) |
| IDOR on balance GET | Information Disclosure | Existing `verify_user_is_sales_person` HR ∨ self at `vacation_balance.rs:115` |
| SQL injection | Tampering | sqlx compile-time `query!`/`query_as!` parameterized (carryover exemplar) |

## Project Constraints (from CLAUDE.md)

- **Clippy is a hard gate:** `cargo clippy --workspace -- -D warnings` MUST pass before commit (nix build / CI enforce `--deny warnings`); `cargo test` does NOT run clippy.
- **Snapshot version bump rule:** changing the computation of a persisted `value_type` REQUIRES bumping `CURRENT_SNAPSHOT_SCHEMA_VERSION` (applies here: off-by-one changes `VacationEntitlement`).
- **Service tier:** Basic services consume only DAOs/Permission/Transaction; offset service is Basic, `VacationBalanceService` is Business-Logic and may consume it (no cycle). DI in `main.rs`: Basic before Business-Logic.
- **Transactions:** every service method takes `Option<Transaction>`; use `use_transaction`/`commit`.
- **OpenAPI:** new REST handlers need `#[utoipa::path]`; new DTOs need `ToSchema`; register in an `ApiDoc`.
- **Migrations:** `migrations/sqlite/`; create via `sqlx migrate add <name>`; sqlx compile-time checking needs an up-to-date local DB.
- **i18n:** add new labels to En, De, Cs; `de.rs` must use `Locale::De`.
- **VCS:** jj-managed; do NOT `git commit`; user controls commits (`commit_docs: false`). Build+test+clippy+run before considering work done.
- **Local dev:** `nix develop` (not `nix-shell`); `sqlx database reset` is DESTRUCTIVE — for this phase use additive `sqlx migrate run` (NEVER reset; requires user confirmation).

## Migration Application (Question 6)

- **Env/DB:** `DATABASE_URL=sqlite:./localdb.sqlite3` (from `env.example:3`; copy to `.env`). sqlx reads `DATABASE_URL` for compile-time query verification and for the migrator.
- **Additive workflow (D-28-01):** create with `sqlx migrate add create-vacation-entitlement-offset` (produces `migrations/sqlite/<timestamp>_create-vacation-entitlement-offset.sql`), then **apply with `sqlx migrate run --source migrations/sqlite`** (NOT `sqlx database reset` — reset is destructive and needs explicit user confirmation per CLAUDE.local).
- **Compile-time gate:** the new `query!`/`query_as!` in `dao_impl_sqlite::vacation_entitlement_offset` are validated against the LOCAL DB schema at compile time. **The migration MUST be applied to `localdb.sqlite3` before `cargo build` will pass.** Order of operations: write migration → `sqlx migrate run` → write DAO impl → `cargo build`.
- **Migration format reference** — `migrations/sqlite/20260330000000_add-shiftplan-table.sql` (plain `CREATE TABLE`, BLOB(16) ids, `update_version`/`update_process`/`deleted` columns) and the carryover schema `migrations/sqlite/20241215063132_add_employee-yearly-carryover.sql` (`PRIMARY KEY (sales_person_id, year)`, `FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)`). Filename timestamp must sort after the latest existing migration (`20260628000001_...`).
- **Run from a `nix develop` shell** if `sqlx` is not on PATH (NixOS).

## Sources

### Primary (HIGH confidence — verified file:line in this repo)
- `service/src/employee_work_details.rs:158-191` — `vacation_days_for_year` off-by-one (start) + correct (end)
- `service_impl/src/vacation_balance.rs:46,57-67,102-129,186-191,256-267` — HR-gate, deps, entitlement, return struct
- `service/src/vacation_balance.rs:44-109` — `VacationBalance` domain + trait
- `service_impl/src/reporting.rs:787-805,852-853` — `vacation_days` vs `vacation_entitlement` locals
- `service_impl/src/billing_period_report.rs:108,256-272` — version constant + value_type mapping
- `service_impl/src/test/billing_period_snapshot_locking.rs:25-69` — guard tests
- `service/src/billing_period.rs:39-88` — `BillingPeriodValueType` enum (incl. `VacationEntitlement`)
- `dao/src/carryover.rs` + `dao_impl_sqlite/src/carryover.rs` — clone target (DAO twin)
- `migrations/sqlite/20241215063132_add_employee-yearly-carryover.sql`, `20260330000000_add-shiftplan-table.sql` — migration format
- `service_impl/src/sales_person_unavailable.rs:17-26` — `gen_service_impl!` Basic skeleton
- `service_impl/src/special_days.rs:72-127` — create/delete guards + soft-delete + permission gate
- `rest/src/special_day.rs` — full CRUD REST exemplar (`#[utoipa::path]`, `error_handler`, ApiDoc)
- `rest/src/vacation_balance.rs` — HR-gate REST + `context.into()` + route ordering
- `rest-types/src/lib.rs:2036-2080` — `VacationBalanceTO` + 2 feature-gated `From` impls
- `shifty_bin/src/main.rs:715,864-873` — DI ordering (carryover before vacation_balance)
- `env.example:3` — `DATABASE_URL`
- FE: `shifty-dioxus/src/state/vacation_balance.rs:11`, `src/page/absences.rs:408,433-466,491,534`, `src/api.rs:669`

### Secondary / Tertiary
- None — no external lookups required.

## Metadata

**Confidence breakdown:**
- Clone target & layer exemplars: HIGH — exact files read end-to-end.
- Snapshot value_type correction: HIGH — traced `vacation_days_for_year → vacation_entitlement → VacationEntitlement` through three files.
- Off-by-one analysis: HIGH — both branches read; year-end confirmed already correct.
- Auth-context HR flag: HIGH — `hr.is_ok()` from the existing `join!` result.
- DI ordering: HIGH — line numbers confirmed in `main.rs`.
- Migration workflow: HIGH — env + migration format + sqlx compile-time dependency confirmed.

**Research date:** 2026-06-29
**Valid until:** ~2026-07-29 (stable internal codebase; re-verify line numbers if files churn before planning).
