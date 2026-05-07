---
plan: 04-06-cutover-rest-and-openapi
phase: 4
wave: 2
depends_on: [04-05-cutover-gate-and-diff-report, 04-04-extra-hours-flag-gate-and-soft-delete, 04-03-carryover-rebuild-service]
requirements: [MIG-03, MIG-05, SC-1]
files_modified:
  - rest-types/src/lib.rs
  - rest/src/cutover.rs
  - rest/src/lib.rs
  - shifty_bin/src/main.rs
  - rest/tests/openapi_snapshot.rs
  - rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap
autonomous: false
must_haves:
  truths:
    - "6 new DTOs exist in `rest-types/src/lib.rs` (inline per Phase-3 convention): `CutoverRunResultTO`, `CutoverGateDriftRowTO`, `CutoverGateDriftReportTO`, `ExtraHoursCategoryDeprecatedErrorTO`, `CutoverProfileTO`, `CutoverProfileBucketTO` — all `Serialize, Deserialize, ToSchema`."
    - "`From<&service::cutover::CutoverRunResult>` impls (and others) gated by `#[cfg(feature = \"service-impl\")]` per Phase-3 inline-DTO precedent."
    - "`rest/src/cutover.rs` contains 3 axum handlers (`POST /admin/cutover/gate-dry-run`, `POST /admin/cutover/commit`, `POST /admin/cutover/profile`) with `#[utoipa::path]` annotations."
    - "`rest/src/lib.rs` patched: `mod cutover;` + ApiDoc nest entry + Router nest + `error_handler` arm for `ServiceError::ExtraHoursCategoryDeprecated` → HTTP 403 with the documented JSON body."
    - "`RestStateDef` extended with `type CutoverService` + `cutover_service()` accessor."
    - "`shifty_bin/src/main.rs` patched: DI for 1 new DAO (CutoverDao); CarryoverRebuildServiceDependencies + CutoverServiceDependencies marker structs + verbatim `*ServiceDeps` trait impls (one assoc-type alias per dep field — pattern from `AbsenceServiceDependencies`); ExtraHoursServiceDependencies extended with `FeatureFlagService` assoc type. **DI re-order: FeatureFlagService MUST be constructed BEFORE ExtraHoursService** (Wave-1-Plan 04-04 hand-off)."
    - "OpenAPI snapshot is generated and committed: `rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap`."
    - "`#[ignore]` removed from openapi_snapshot test."
    - "`cargo test -p rest --test openapi_snapshot openapi_snapshot_locks_full_api_surface` GREEN (3 consecutive runs produce no `.snap.new` file)."
  artifacts:
    - path: "rest/src/cutover.rs"
      provides: "3 REST handlers (gate-dry-run, commit, profile) + CutoverApiDoc + generate_route()"
    - path: "rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap"
      provides: "Pin file for OpenAPI surface (committed to repo)"
    - path: "shifty_bin/src/main.rs"
      provides: "DI: CutoverServiceImpl + CarryoverRebuildServiceImpl + 1 new DAO (CutoverDaoImpl); ExtraHoursServiceDependencies extended with FeatureFlagService assoc type; FeatureFlagService re-ordered before ExtraHoursService"
  key_links:
    - from: "POST /admin/cutover/gate-dry-run"
      to: "CutoverService::run(dry_run=true)"
      via: "REST handler: permission HR (service-layer enforces); body returns CutoverRunResultTO"
    - from: "POST /admin/cutover/commit"
      to: "CutoverService::run(dry_run=false)"
      via: "Permission cutover_admin (service-layer enforces); body returns CutoverRunResultTO"
    - from: "POST /admin/cutover/profile"
      to: "CutoverService::profile(...)"
      via: "Permission HR (service-layer enforces — profile is read-only); body returns CutoverProfileTO; persists JSON file under `.planning/migration-backup/profile-{ts}.json`"
    - from: "ServiceError::ExtraHoursCategoryDeprecated"
      to: "rest::error_handler -> 403 JSON body"
      via: "Match arm produces `{\"error\": \"extra_hours_category_deprecated\", \"category\": \"vacation\", \"message\": \"...\"}`"
---

<objective>
Wave 2 (second half) — Wire the REST surface for `/admin/cutover/gate-dry-run`, `/admin/cutover/commit`, and `/admin/cutover/profile`, map `ServiceError::ExtraHoursCategoryDeprecated` to HTTP 403, register the new services in `shifty_bin/src/main.rs` (with the **mandatory FeatureFlagService-before-ExtraHoursService re-order**), and generate + accept the OpenAPI snapshot pin file.

This plan is `autonomous: false` because the OpenAPI snapshot accept-step is a `checkpoint:human-verify` task — the user reviews the `.snap.new` content before promoting it to `.snap`.

Purpose: Finish the externally-visible API surface and lock it via insta. After this plan, the entire workspace boots, REST works end-to-end (permission-gated), profile() is reachable from Production by HR via REST (SC-1 acceptance criterion), and OpenAPI surface is pinned.

Output: 6 files modified (1 new file: `rest/src/cutover.rs`; 1 new snapshot file; 4 patches).
</objective>

<execution_context>
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/workflows/execute-plan.md
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/phases/04-migration-cutover/04-CONTEXT.md
@.planning/phases/04-migration-cutover/04-RESEARCH.md
@.planning/phases/04-migration-cutover/04-VALIDATION.md
@.planning/phases/04-migration-cutover/04-04-SUMMARY.md
@.planning/phases/04-migration-cutover/04-05-SUMMARY.md

@rest-types/src/lib.rs
@rest/src/lib.rs
@rest/src/absence.rs
@rest/src/feature_flag.rs
@rest/src/extra_hours.rs
@shifty_bin/src/main.rs
@rest/tests/openapi_snapshot.rs

<interfaces>
<!-- Verbatim contracts. -->

From `rest-types/src/lib.rs:1779-1794` (Phase-3 inline-DTO + From-impl gated pattern):
```rust
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct BookingCreateResultTO {
    pub booking: BookingTO,
    pub warnings: Vec<WarningTO>,
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan_edit::BookingCreateResult> for BookingCreateResultTO {
    fn from(r: &service::shiftplan_edit::BookingCreateResult) -> Self { ... }
}
```

From `rest/src/absence.rs` (Phase-1 handler shape — pattern for Cutover handlers):
```rust
#[utoipa::path(
    post, path = "/",
    request_body = AbsencePeriodTO,
    responses((status = 201, body = AbsencePeriodCreateResultTO)),
)]
pub async fn create_absence_period_handler<RestState: RestStateDef>(...) -> Response { ... }

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> { ... }

#[derive(OpenApi)]
#[openapi(paths(...))]
pub struct AbsenceApiDoc;
```

From `rest/src/lib.rs:296-354` (RestStateDef trait — extend with CutoverService entry):
```rust
type AbsenceService: service::absence::AbsenceService<Context = Context> + Send + Sync + 'static;
fn absence_service(&self) -> Arc<Self::AbsenceService>;
```

From `rest/src/lib.rs:121-249` (existing error_handler arms — add the new variant):
```rust
match err {
    ServiceError::Forbidden => Response::builder().status(403).body(...).unwrap(),
    ServiceError::Unauthorized => Response::builder().status(401)...,
    // ... add new arm before generic InternalError fallback
}
```

From `shifty_bin/src/main.rs:227-269` (existing **marker-struct + assoc-type-alias DI pattern** — verbatim template, this is what CarryoverRebuildServiceDependencies + CutoverServiceDependencies must mirror):

```rust
pub struct AbsenceServiceDependencies;
impl service_impl::absence::AbsenceServiceDeps for AbsenceServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type AbsenceDao = AbsenceDao;
    type PermissionService = PermissionService;
    type SalesPersonService = SalesPersonService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type SpecialDayService = SpecialDayService;
    type EmployeeWorkDetailsService = WorkingHoursService;
    type TransactionDao = TransactionDao;
    type BookingService = BookingService;
    type SalesPersonUnavailableService = SalesPersonUnavailableService;
    type SlotService = SlotService;
}
type AbsenceService =
    service_impl::absence::AbsenceServiceImpl<AbsenceServiceDependencies>;

pub struct ExtraHoursServiceDependencies;
impl service_impl::extra_hours::ExtraHoursServiceDeps for ExtraHoursServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ExtraHoursDao = ExtraHoursDao;
    type PermissionService = PermissionService;
    type SalesPersonService = SalesPersonService;
    type CustomExtraHoursService = CustomExtraHoursService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type ExtraHoursService =
    service_impl::extra_hours::ExtraHoursServiceImpl<ExtraHoursServiceDependencies>;

pub struct FeatureFlagServiceDependencies;
impl service_impl::feature_flag::FeatureFlagServiceDeps for FeatureFlagServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type FeatureFlagDao = FeatureFlagDao;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type FeatureFlagService =
    service_impl::feature_flag::FeatureFlagServiceImpl<FeatureFlagServiceDependencies>;
```

**Pattern:** marker struct (`pub struct XServiceDependencies;`) + `impl service_impl::x::XServiceDeps for XServiceDependencies` with one `type Y = Z;` per dep field — NO struct fields, NO `#[derive(Clone)]` data, NO Arc fields (the `gen_service_impl!` macro generates the actual `XServiceImpl<Deps>` struct that owns the Arcs). Phase 4 adds two new marker-struct + impl blocks following this exact pattern.

From CONTEXT.md `<specifics>` (verbatim 403 body schema):
```json
{ "error": "extra_hours_category_deprecated", "category": "vacation", "message": "Use POST /absence-period for this category" }
```

From C-Phase4-05 (Profile-Bucket field schema — verbatim):
- `sales_person_id`, `sales_person_name`, `category`, `year`
- `row_count`, `sum_amount` (= `sum_hours`)
- `fractional_count` (count of rows where `amount != contract_hours_at(day)`)
- `weekend_on_workday_only_contract_count`
- `iso_53_indicator: bool`
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: rest-types/src/lib.rs — 6 new inline DTOs + From impls (4 cutover-result + 2 cutover-profile)</name>
  <read_first>
    - rest-types/src/lib.rs (Z. 1620-1800 — Phase-3 wrapper-DTO precedent)
    - service/src/cutover.rs (Plan 04-01 source structs: CutoverRunResult, DriftRow, GateResult, CutoverProfile, CutoverProfileBucket)
    - service/src/extra_hours.rs (ExtraHoursCategory enum — for ExtraHoursCategoryTO mapping; verify TO already exists in rest-types)
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "<canonical_refs> rest-types/src/lib.rs"
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "C-Phase4-05" (Profile-bucket field list)
  </read_first>
  <action>
**Append to `rest-types/src/lib.rs`** (after the existing Phase-3 wrapper-DTO block ~line 1800):

```rust
// =====================
// Phase 4 — Cutover DTOs
// =====================

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CutoverGateDriftRowTO {
    pub sales_person_id: uuid::Uuid,
    pub sales_person_name: String,
    pub category: AbsenceCategoryTO,    // already exists from Phase-1
    pub year: u32,
    pub legacy_sum: f32,
    pub derived_sum: f32,
    pub drift: f32,
    pub quarantined_extra_hours_count: u32,
    pub quarantine_reasons: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CutoverGateDriftReportTO {
    pub gate_run_id: uuid::Uuid,
    pub run_at: String,           // ISO-8601 string for OpenAPI portability
    pub dry_run: bool,
    pub drift_threshold: f32,
    pub total_drift_rows: u32,
    pub drift: Vec<CutoverGateDriftRowTO>,
    pub passed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CutoverRunResultTO {
    pub run_id: uuid::Uuid,
    pub ran_at: String,
    pub dry_run: bool,
    pub gate_passed: bool,
    pub total_clusters: u32,
    pub migrated_clusters: u32,
    pub quarantined_rows: u32,
    pub gate_drift_rows: u32,
    pub diff_report_path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ExtraHoursCategoryDeprecatedErrorTO {
    pub error: String,        // always "extra_hours_category_deprecated"
    pub category: String,     // lowercase variant name (e.g., "vacation")
    pub message: String,      // user-facing hint
}

/// Per-(sp, category, year) profile bucket per C-Phase4-05.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CutoverProfileBucketTO {
    pub sales_person_id: uuid::Uuid,
    pub sales_person_name: String,
    pub category: AbsenceCategoryTO,
    pub year: u32,
    pub row_count: u32,
    pub sum_hours: f32,                                    // = sum_amount
    pub fractional_count: u32,
    pub weekend_on_workday_only_count: u32,
    pub iso_53_indicator: bool,
}

/// Production-data profile envelope — wraps every bucket plus run metadata.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CutoverProfileTO {
    pub profile_run_id: uuid::Uuid,
    pub run_at: String,                                    // ISO-8601
    pub total_buckets: u32,
    pub buckets: Vec<CutoverProfileBucketTO>,
    pub output_path: String,                               // .planning/migration-backup/profile-{ts}.json
}

// From impls (gated for binary builds that include service crate)

#[cfg(feature = "service-impl")]
impl From<&service::cutover::DriftRow> for CutoverGateDriftRowTO {
    fn from(r: &service::cutover::DriftRow) -> Self {
        Self {
            sales_person_id: r.sales_person_id,
            sales_person_name: r.sales_person_name.to_string(),
            category: AbsenceCategoryTO::from(&r.category),
            year: r.year,
            legacy_sum: r.legacy_sum,
            derived_sum: r.derived_sum,
            drift: r.drift,
            quarantined_extra_hours_count: r.quarantined_extra_hours_count,
            quarantine_reasons: r.quarantine_reasons.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::cutover::CutoverRunResult> for CutoverRunResultTO {
    fn from(r: &service::cutover::CutoverRunResult) -> Self {
        Self {
            run_id: r.run_id,
            ran_at: r.ran_at.assume_utc()
                .format(&time::format_description::well_known::Iso8601::DEFAULT)
                .unwrap_or_default(),
            dry_run: r.dry_run,
            gate_passed: r.gate_passed,
            total_clusters: r.total_clusters,
            migrated_clusters: r.migrated_clusters,
            quarantined_rows: r.quarantined_rows,
            gate_drift_rows: r.gate_drift_rows,
            diff_report_path: r.diff_report_path.as_ref().map(|s| s.to_string()),
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::cutover::CutoverProfileBucket> for CutoverProfileBucketTO {
    fn from(b: &service::cutover::CutoverProfileBucket) -> Self {
        Self {
            sales_person_id: b.sales_person_id,
            sales_person_name: b.sales_person_name.to_string(),
            category: AbsenceCategoryTO::from(&b.category),
            year: b.year,
            row_count: b.row_count,
            sum_hours: b.sum_amount,
            fractional_count: b.fractional_count,
            weekend_on_workday_only_count: b.weekend_on_workday_only_contract_count,
            iso_53_indicator: b.iso_53_indicator,
        }
    }
}

#[cfg(feature = "service-impl")]
impl From<&service::cutover::CutoverProfile> for CutoverProfileTO {
    fn from(p: &service::cutover::CutoverProfile) -> Self {
        Self {
            profile_run_id: p.run_id,
            run_at: p.generated_at.assume_utc()
                .format(&time::format_description::well_known::Iso8601::DEFAULT)
                .unwrap_or_default(),
            total_buckets: p.buckets.len() as u32,
            buckets: p.buckets.iter().map(CutoverProfileBucketTO::from).collect(),
            output_path: p.profile_path.to_string(),
        }
    }
}
```

Verify `AbsenceCategoryTO` already exists in `rest-types/src/lib.rs` (added in Phase 1) — if not, the executor must add it.

**Verify `feature = "service-impl"`** is the gating flag used by the existing `BookingCreateResultTO` From impl at line 1779. If a different gate name is used (e.g. `service`), use that.
  </action>
  <acceptance_criteria>
    - `grep -q 'pub struct CutoverGateDriftRowTO' rest-types/src/lib.rs` exits 0
    - `grep -q 'pub struct CutoverGateDriftReportTO' rest-types/src/lib.rs` exits 0
    - `grep -q 'pub struct CutoverRunResultTO' rest-types/src/lib.rs` exits 0
    - `grep -q 'pub struct ExtraHoursCategoryDeprecatedErrorTO' rest-types/src/lib.rs` exits 0
    - `grep -q 'pub struct CutoverProfileTO' rest-types/src/lib.rs` exits 0
    - `grep -q 'pub struct CutoverProfileBucketTO' rest-types/src/lib.rs` exits 0
    - `cargo build -p rest-types` exits 0
    - `cargo build --workspace` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p rest-types</automated>
  </verify>
  <done>
    All 6 DTOs exist with `ToSchema` derive (utoipa-visible) and From-impls.
  </done>
</task>

<task type="auto">
  <name>Task 2: rest/src/cutover.rs — 3 axum handlers (gate-dry-run, commit, profile) + utoipa annotations + CutoverApiDoc + generate_route</name>
  <read_first>
    - rest/src/absence.rs (full file — Phase-1 handler/ApiDoc/route pattern)
    - rest/src/feature_flag.rs (if exists — Phase-2 admin-style endpoint pattern)
    - rest-types/src/lib.rs (Task 1 above — DTOs)
    - service/src/cutover.rs (run + profile signatures)
    - rest/src/lib.rs (Z. 296-354 RestStateDef — see what's needed for the new accessor)
  </read_first>
  <action>
**Create `rest/src/cutover.rs`:**

```rust
//! Phase 4 — Cutover REST surface.
//!
//! Three endpoints + one ApiDoc + one route generator.
//! Permission enforcement happens at the service layer (D-Phase4-07 + Pattern 3):
//!   - gate-dry-run: HR
//!   - commit:       cutover_admin
//!   - profile:      HR (read-only — full extra_hours scan, deliberately Tx-rolled-back)

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
use utoipa::OpenApi;

use rest_types::{CutoverProfileTO, CutoverRunResultTO};

use crate::{error_handler, RestStateDef, Context};

/// Empty placeholder — kept as a typed body so future field additions remain
/// backward-compatible. Adding a non-Option field here IS a snapshot-breaking
/// change (the OpenAPI schema diff will fire) and requires explicit review.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CutoverGateDryRunRequest {}

/// Empty placeholder — see `CutoverGateDryRunRequest` for snapshot-impact note.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CutoverCommitRequest {}

/// Empty placeholder — see `CutoverGateDryRunRequest` for snapshot-impact note.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CutoverProfileRequest {}

#[utoipa::path(
    post,
    path = "/gate-dry-run",
    responses(
        (status = 200, body = CutoverRunResultTO, description = "Dry-run cutover (rolls back).",),
        (status = 403, description = "Caller lacks HR privilege."),
    ),
)]
pub async fn cutover_gate_dry_run_handler<RestState: RestStateDef>(
    State(state): State<RestState>,
) -> impl IntoResponse {
    let svc = state.cutover_service();
    let ctx = state.context();   // verify exact accessor; mirror absence handler pattern
    let result = svc.run(true, ctx, None).await;
    error_handler::<RestState, _, _>(result.map(|r| (StatusCode::OK, Json(CutoverRunResultTO::from(&r)))))
}

#[utoipa::path(
    post,
    path = "/commit",
    responses(
        (status = 200, body = CutoverRunResultTO, description = "Committed cutover. Feature flag flipped + Migration applied + Carryover refreshed.",),
        (status = 403, description = "Caller lacks cutover_admin privilege."),
    ),
)]
pub async fn cutover_commit_handler<RestState: RestStateDef>(
    State(state): State<RestState>,
) -> impl IntoResponse {
    let svc = state.cutover_service();
    let ctx = state.context();
    let result = svc.run(false, ctx, None).await;
    error_handler::<RestState, _, _>(result.map(|r| (StatusCode::OK, Json(CutoverRunResultTO::from(&r)))))
}

/// SC-1 surface — Production-Data-Profile. Read-only from a state perspective
/// (the cutover Tx is rolled back inside `profile()`); persists a JSON file
/// under `.planning/migration-backup/profile-{ts}.json` for HR review.
/// Permission: HR (matches gate-dry-run; profile is non-destructive).
#[utoipa::path(
    post,
    path = "/profile",
    responses(
        (status = 200, body = CutoverProfileTO, description = "Production-data profile generated; JSON file persisted under .planning/migration-backup/.",),
        (status = 403, description = "Caller lacks HR privilege."),
    ),
)]
pub async fn cutover_profile_handler<RestState: RestStateDef>(
    State(state): State<RestState>,
) -> impl IntoResponse {
    let svc = state.cutover_service();
    let ctx = state.context();
    let result = svc.profile(ctx, None).await;
    error_handler::<RestState, _, _>(result.map(|p| (StatusCode::OK, Json(CutoverProfileTO::from(&p)))))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        cutover_gate_dry_run_handler,
        cutover_commit_handler,
        cutover_profile_handler,
    ),
    components(schemas(
        CutoverRunResultTO,
        CutoverProfileTO,
        rest_types::CutoverProfileBucketTO,
    ))
)]
pub struct CutoverApiDoc;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/gate-dry-run", post(cutover_gate_dry_run_handler::<RestState>))
        .route("/commit", post(cutover_commit_handler::<RestState>))
        .route("/profile", post(cutover_profile_handler::<RestState>))
}
```

**Verify** the `error_handler` signature and `state.context()` access pattern match how `rest/src/absence.rs` uses them — adapt if needed. The exact `error_handler<RestState, ResponseBody, ErrorBody>` generic surface might differ; mirror the existing handler pattern verbatim.
  </action>
  <acceptance_criteria>
    - File `rest/src/cutover.rs` exists; `grep -q 'pub fn generate_route' rest/src/cutover.rs` exits 0
    - 3 utoipa handlers present: `[ "$(grep -c '#\[utoipa::path' rest/src/cutover.rs)" -ge 3 ]`
    - `grep -q 'pub struct CutoverApiDoc' rest/src/cutover.rs` exits 0
    - `grep -q 'svc.run(true' rest/src/cutover.rs` exits 0 (dry-run handler)
    - `grep -q 'svc.run(false' rest/src/cutover.rs` exits 0 (commit handler)
    - `grep -q 'svc.profile' rest/src/cutover.rs` exits 0 (profile handler)
    - `grep -q '"/profile"' rest/src/cutover.rs` exits 0 (profile route registered)
    - `cargo build -p rest` exits 0 (will fail until Task 3 + Task 4 complete; verify only after all wave-2 tasks land)
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p rest</automated>
  </verify>
  <done>
    REST handlers (3) + utoipa annotations + ApiDoc + route generator all in place. Profile endpoint reachable from HR via REST per SC-1.
  </done>
</task>

<task type="auto">
  <name>Task 3: rest/src/lib.rs — mod cutover; ApiDoc nest; Router nest; RestStateDef extension; error_handler 403 mapping</name>
  <read_first>
    - rest/src/lib.rs (full file)
    - rest/src/cutover.rs (Task 2 above)
    - rest-types/src/lib.rs (ExtraHoursCategoryDeprecatedErrorTO from Task 1)
    - service/src/lib.rs (ServiceError::ExtraHoursCategoryDeprecated variant from Plan 04-01)
  </read_first>
  <action>
**Patch `rest/src/lib.rs`:**

a) Add `mod cutover;` near the top (alphabetical order with the existing `mod absence; mod billing_period; ...` block at line 3).

b) Extend the `RestStateDef` trait (Z. 296-354) with the new associated type and accessor:
```rust
type CutoverService: service::cutover::CutoverService<Context = Context> + Send + Sync + 'static;
fn cutover_service(&self) -> Arc<Self::CutoverService>;
```

c) Add to the ApiDoc nest list (Z. 460-486):
```rust
(path = "/admin/cutover", api = cutover::CutoverApiDoc),
```
(Insert in alphabetical order; should land after `/absence-period` and before `/billing-period` — or wherever alphabetical fits given the existing order.)

d) Add to the Router nest list (Z. 540-570):
```rust
.nest("/admin/cutover", cutover::generate_route())
```

e) Patch the `error_handler` (Z. 121-249) — add a new match arm BEFORE the catch-all `_ => /* InternalError */`:
```rust
Err(crate::RestError::ServiceError(service::ServiceError::ExtraHoursCategoryDeprecated(category))) => {
    let body = rest_types::ExtraHoursCategoryDeprecatedErrorTO {
        error: "extra_hours_category_deprecated".to_string(),
        category: format!("{:?}", category).to_lowercase(),
        message: "Use POST /absence-period for this category".to_string(),
    };
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap_or_default()))
        .unwrap()
}
```

**Important:**
- The exact match-arm structure depends on how `error_handler` is currently shaped. Read the existing arms (Forbidden, Unauthorized, EntityNotFound, etc.) and follow the same pattern verbatim.
- `format!("{:?}", category).to_lowercase()` produces `"vacation"` from `ExtraHoursCategory::Vacation`. If the enum has a `CustomExtraHours(...)`-variant or other complex variants, the lowercase format may produce something like `"customextrahours(...)"` which is acceptable for this error path (Vacation/SickLeave/UnpaidLeave are the only categories that hit this code path per Plan 04-04 gate logic).
  </action>
  <acceptance_criteria>
    - `grep -q 'mod cutover' rest/src/lib.rs` exits 0
    - `grep -q 'cutover_service' rest/src/lib.rs` exits 0
    - `grep -q '"/admin/cutover"' rest/src/lib.rs` exits 0
    - `grep -q 'ExtraHoursCategoryDeprecated' rest/src/lib.rs` exits 0
    - `grep -q 'extra_hours_category_deprecated' rest/src/lib.rs` exits 0
    - `cargo build -p rest` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p rest</automated>
  </verify>
  <done>
    `rest` crate compiles with new module, RestStateDef extension, ApiDoc nest, router nest, and error mapping.
  </done>
</task>

<task type="auto">
  <name>Task 4: shifty_bin/src/main.rs — DI re-order + verbatim CutoverServiceDependencies + CarryoverRebuildServiceDependencies + ExtraHoursServiceDependencies extension + RestStateImpl::cutover_service</name>
  <read_first>
    - shifty_bin/src/main.rs (full file — focus on Z. 227-269 marker-struct + impl pattern, Z. 270-280 type aliases, Z. 462-471 FeatureFlagServiceDependencies, Z. 495-501 RestStateImpl struct, Z. 574-593 trait impl methods, Z. 729-810 the DI construction block, Z. 956-962 final RestStateImpl assembly)
    - service_impl/src/extra_hours.rs (Plan 04-04 — confirmed new field name `feature_flag_service` added to `gen_service_impl!` block)
    - service_impl/src/cutover.rs (Plan 04-02 / 04-05 — DI deps: 10 fields enumerated in 04-05 SUMMARY hand-off)
    - service_impl/src/carryover_rebuild.rs (Plan 04-03 — DI deps: 4 fields)
    - .planning/phases/04-migration-cutover/04-04-SUMMARY.md (DI re-order MANDATORY note)
  </read_first>
  <action>
**Patch `shifty_bin/src/main.rs`:**

a) **Patch the existing `ExtraHoursServiceDependencies` impl** (current Z. 256-267) — add the new `FeatureFlagService` assoc type that Plan 04-04 introduced:

```rust
pub struct ExtraHoursServiceDependencies;
impl service_impl::extra_hours::ExtraHoursServiceDeps for ExtraHoursServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ExtraHoursDao = ExtraHoursDao;
    type PermissionService = PermissionService;
    type SalesPersonService = SalesPersonService;
    type CustomExtraHoursService = CustomExtraHoursService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type FeatureFlagService = FeatureFlagService;   // NEW Phase-4 (D-Phase4-09)
    type TransactionDao = TransactionDao;
}
```

(The `FeatureFlagService` type alias is already defined further down at Z. 470-471 — Rust's name-resolution for `type X = ...;` aliases doesn't require declaration-before-use within the same module, so referring to it from above is fine. Verify by `cargo build` after editing.)

b) **Add the two new marker-struct + assoc-type-alias DI blocks** at the bottom of the existing dependency-struct cluster (i.e., after `FeatureFlagServiceDependencies` at Z. 462 — append in alphabetical or logical order; Plan-Phase recommends grouping with the other Phase-4-touched deps for grep-locatability):

```rust
// Phase 4 / Plan 04-03 — Cycle-breaking carryover rebuild service.
// Business-Logic-Tier service (consumes ReportingService + CarryoverService).
pub struct CarryoverRebuildServiceDependencies;
impl service_impl::carryover_rebuild::CarryoverRebuildServiceDeps
    for CarryoverRebuildServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type ReportingService = ReportingService;
    type CarryoverService = CarryoverService;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type CarryoverRebuildService =
    service_impl::carryover_rebuild::CarryoverRebuildServiceImpl<CarryoverRebuildServiceDependencies>;

// Phase 4 / Plan 04-02+04-05 — Cutover orchestration.
// Business-Logic-Tier service (consumes 7 sub-services + 2 DAOs + transaction_dao).
pub struct CutoverServiceDependencies;
impl service_impl::cutover::CutoverServiceDeps for CutoverServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type CutoverDao = CutoverDao;
    type AbsenceDao = AbsenceDao;
    type AbsenceService = AbsenceService;
    type ExtraHoursService = ExtraHoursService;
    type CarryoverRebuildService = CarryoverRebuildService;
    type FeatureFlagService = FeatureFlagService;
    type EmployeeWorkDetailsService = WorkingHoursService;   // matches AbsenceServiceDependencies row
    type SalesPersonService = SalesPersonService;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type CutoverService =
    service_impl::cutover::CutoverServiceImpl<CutoverServiceDependencies>;
```

(Pattern verbatim from existing `AbsenceServiceDependencies` at Z. 227-251 and `FeatureFlagServiceDependencies` at Z. 462-471. NO `#[derive(Clone)]`, NO struct fields — these are zero-sized marker types; the actual `Arc`-holding struct is generated by `gen_service_impl!` inside `service_impl::cutover::CutoverServiceImpl` and `service_impl::carryover_rebuild::CarryoverRebuildServiceImpl`.)

c) **Add the new DAO type alias** near where the other DAOs are aliased (find the existing `type AbsenceDao = ...;` and `type ExtraHoursDao = ...;` block; add):
```rust
type CutoverDao = dao_impl_sqlite::cutover::CutoverDaoImpl;
```

d) **Construct the new DAO + services in the runtime DI block** (the `let absence_service = Arc::new(...)` etc. block around Z. 729-810). Place these AFTER all basic services AND AFTER `feature_flag_service` AND AFTER `extra_hours_service`:

```rust
let cutover_dao = Arc::new(dao_impl_sqlite::cutover::CutoverDaoImpl);

let carryover_rebuild_service = Arc::new(service_impl::carryover_rebuild::CarryoverRebuildServiceImpl {
    reporting_service: reporting_service.clone(),
    carryover_service: carryover_service.clone(),
    permission_service: permission_service.clone(),
    transaction_dao: transaction_dao.clone(),
});

let cutover_service = Arc::new(service_impl::cutover::CutoverServiceImpl {
    cutover_dao: cutover_dao.clone(),
    absence_dao: absence_dao.clone(),
    absence_service: absence_service.clone(),
    extra_hours_service: extra_hours_service.clone(),
    carryover_rebuild_service: carryover_rebuild_service.clone(),
    feature_flag_service: feature_flag_service.clone(),
    employee_work_details_service: working_hours_service.clone(),   // verify variable name
    sales_person_service: sales_person_service.clone(),
    permission_service: permission_service.clone(),
    transaction_dao: transaction_dao.clone(),
});
```

(Verify each Arc-name matches the actual variable names in main.rs — `working_hours_service` may be aliased differently. The bind-name for the ImplStruct's field comes from the `gen_service_impl!` macro snake_case'd from the trait name in the Plan 04-02 DI block.)

e) **DI re-order (MANDATORY per Plan 04-04 hand-off):**
- Currently `feature_flag_service` is constructed AFTER `extra_hours_service` (Plan-04-04 hand-off documents Z. 770 vs Z. 795).
- **Move the `let feature_flag_service = Arc::new(FeatureFlagServiceImpl{...});` line to BEFORE the `let extra_hours_service = Arc::new(ExtraHoursServiceImpl{...});` line.**
- Pass `feature_flag_service.clone()` as a new field into `ExtraHoursServiceImpl { ... }` (the field is named `feature_flag_service` per `gen_service_impl!` snake_case rule).

f) **Extend `RestStateImpl` (Z. 495-501)** with the new field:
```rust
cutover_service: Arc<CutoverService>,
```

(Optionally also `carryover_rebuild_service: Arc<CarryoverRebuildService>` if any REST endpoint needs it — Phase 4 does not, so leave it out. The cutover service holds a clone internally via the DI block above.)

g) **Implement `RestStateDef::cutover_service()`** (mirror `extra_hours_service()` at Z. 592):
```rust
fn cutover_service(&self) -> Arc<Self::CutoverService> {
    self.cutover_service.clone()
}
```
And add `type CutoverService = CutoverService;` to the `impl RestStateDef for RestStateImpl` block.

h) **Add to the final RestStateImpl assembly (Z. 956-962):**
```rust
cutover_service,
```

After patching, run `cargo build --workspace` and `cargo run` with a 30-second timeout (per VALIDATION.md "Manual-Only Verifications" Bin-Boot-Smoke):
```bash
timeout 30 cargo run 2>&1 | head -40
```
Expected: log `"Server listening on ..."` appears within 30s; non-124 exit code = boot failure.
  </action>
  <acceptance_criteria>
    - `grep -q 'cutover_service' shifty_bin/src/main.rs` exits 0 (multiple matches)
    - `grep -q 'pub struct CarryoverRebuildServiceDependencies;' shifty_bin/src/main.rs` exits 0
    - `grep -q 'pub struct CutoverServiceDependencies;' shifty_bin/src/main.rs` exits 0
    - `grep -q 'impl service_impl::carryover_rebuild::CarryoverRebuildServiceDeps for CarryoverRebuildServiceDependencies' shifty_bin/src/main.rs` exits 0
    - `grep -q 'impl service_impl::cutover::CutoverServiceDeps for CutoverServiceDependencies' shifty_bin/src/main.rs` exits 0
    - `grep -q 'type FeatureFlagService = FeatureFlagService;' shifty_bin/src/main.rs` exits 0 (added to ExtraHoursServiceDependencies impl block)
    - `grep -q 'type CutoverDao = dao_impl_sqlite::cutover::CutoverDaoImpl' shifty_bin/src/main.rs` exits 0
    - **DI re-order verified (tightened awk pattern targets `Arc::new(<ImplStruct>` constructor calls — first occurrence each):**
      ```bash
      awk '!ff && /Arc::new\(FeatureFlagServiceImpl/{ff=NR} !eh && /Arc::new\(ExtraHoursServiceImpl/{eh=NR} END{exit (ff && eh && ff < eh ? 0 : 1)}' shifty_bin/src/main.rs
      ```
      Exits 0 iff both lines exist AND `feature_flag_service` constructor sits before `extra_hours_service` constructor.
    - `cargo build --workspace` exits 0
    - `cargo test --workspace` exits 0 (no test should regress)
    - `timeout 30 cargo run 2>&1 | grep -q "Server listening"` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build --workspace && timeout 30 cargo run 2>&1 | head -40 | grep -q "listening"</automated>
  </verify>
  <done>
    `shifty_bin` boots with the full DI tree; CutoverService is reachable via REST including the new profile endpoint.
  </done>
</task>

<task type="auto">
  <name>Task 5: rest/tests/openapi_snapshot.rs — un-#[ignore] + run to generate .snap.new</name>
  <read_first>
    - rest/tests/openapi_snapshot.rs (Plan 04-00 Task 3 scaffold)
    - rest/Cargo.toml (verify insta dev-dep present)
  </read_first>
  <action>
**Patch `rest/tests/openapi_snapshot.rs`:** remove the `#[ignore = "wave-2-accepts-snapshot"]` attribute. The test:
```rust
#[test]
fn openapi_snapshot_locks_full_api_surface() {
    let openapi = ApiDoc::openapi();
    insta::with_settings!({ sort_maps => true }, {
        insta::assert_json_snapshot!(openapi);
    });
}
```

Run the test:
```bash
cargo test -p rest --test openapi_snapshot
```

**Expected outcome:** the test FAILS on first run because no `.snap` file exists; insta writes `.snap.new` next to the test (path: `rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap.new`).

The executor must NOT auto-accept (that requires human review of the diff). Move on to Task 6 (checkpoint:human-verify) which gates the user's review and rename of `.snap.new → .snap`.
  </action>
  <acceptance_criteria>
    - `grep -q '#\[ignore' rest/tests/openapi_snapshot.rs` exits 1 (no longer present)
    - File `rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap.new` exists after `cargo test -p rest --test openapi_snapshot` runs
    - The `.snap.new` file contains valid JSON (run `cat .../*.snap.new | tail -5` to inspect — should look like a serialized OpenApi spec)
  </acceptance_criteria>
  <verify>
    <automated>cargo test -p rest --test openapi_snapshot; test -f rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap.new</automated>
  </verify>
  <done>
    `.snap.new` file generated with the live OpenAPI surface; ready for human review in Task 6.
  </done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 6: Human review + accept of OpenAPI snapshot file (3 endpoints + 6 schemas)</name>
  <what-built>
    Plan 04-06 has wired the `/admin/cutover/gate-dry-run`, `/admin/cutover/commit`, AND `/admin/cutover/profile` endpoints into the OpenAPI ApiDoc. The insta-snapshot test produced a `.snap.new` file containing the full live OpenAPI spec.

    The user must review this snapshot to ensure:
    1. **All 3 new `/admin/cutover/*` endpoints** appear with expected request/response schemas (gate-dry-run, commit, profile).
    2. **All 6 new schemas** are present: `CutoverRunResultTO`, `CutoverGateDriftRowTO`, `CutoverGateDriftReportTO`, `ExtraHoursCategoryDeprecatedErrorTO`, `CutoverProfileTO`, `CutoverProfileBucketTO`.
    3. The profile-endpoint response wraps `CutoverProfileTO` (with `profile_run_id`, `run_at`, `total_buckets`, `buckets[]`, `output_path`); the bucket schema (`CutoverProfileBucketTO`) carries the C-Phase4-05 fields (sales_person_id, sales_person_name, category, year, row_count, sum_hours, fractional_count, weekend_on_workday_only_count, iso_53_indicator).
    4. No unexpected or unintended changes to existing endpoints (e.g., field renames, removed endpoints).
    5. The snapshot is byte-identical across 3 consecutive runs (determinism check).
  </what-built>
  <how-to-verify>
    1. **Review the diff manually** (no global `cargo insta` install per MEMORY.md):
       ```bash
       diff -u rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap{,.new} 2>/dev/null \
         || cat rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap.new | head -150
       ```
       (First time: the `.snap` doesn't exist yet, so `diff` exits non-zero; review the `.snap.new` content directly.)

    2. **Verify all 3 cutover endpoints are present:**
       ```bash
       grep -E '"/admin/cutover/(gate-dry-run|commit|profile)"' rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap.new | wc -l
       ```
       Output must be `3`.

    3. **Verify all 6 new schemas (including the two profile DTOs):**
       ```bash
       grep -E '"(CutoverRunResultTO|CutoverGateDriftRowTO|CutoverGateDriftReportTO|ExtraHoursCategoryDeprecatedErrorTO|CutoverProfileTO|CutoverProfileBucketTO)"' rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap.new | wc -l
       ```
       Output must be `>= 6` (each schema appears at least once as a definition).

    4. **Determinism check (3 consecutive identical runs):**
       ```bash
       # Move .snap.new to .snap to lock the baseline:
       mv rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap{.new,}

       # Now run 3x — each must exit 0 with NO new .snap.new file:
       for i in 1 2 3; do
         cargo test -p rest --test openapi_snapshot openapi_snapshot_locks_full_api_surface || echo "FAIL on run $i"
       done

       # Expected: 0 .snap.new files exist after the loop
       ls rest/tests/snapshots/*.snap.new 2>/dev/null | wc -l   # must print 0
       ```

    5. **Optional alternative:** if `cargo insta` is locally installed (per user permission only), run `cargo insta review` for an interactive workflow.

    6. **Reject signal:** if any unexpected endpoint/field changes appear (e.g., a Phase-3 endpoint accidentally lost a parameter), describe the unexpected diff and abort — Plan-Phase or Wave-3 must reconcile before proceeding.
  </how-to-verify>
  <resume-signal>Type "approved" once `.snap.new` is renamed to `.snap`, all 3 endpoints + 6 schemas verified present, and 3-run determinism check passes. Or describe issues for replanning.</resume-signal>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| HTTP `/admin/cutover/*` ← Internet | All 3 endpoints require authenticated session + permission gate at the service layer. |
| OpenAPI snapshot ← future PRs | Pin file is committed to repo; CI tests fail on any unintended schema change. |
| `error_handler` 403 body ← service error variant | The Display-string parsing pattern is tightly coupled to `ExtraHoursCategory` enum names. |
| Profile endpoint file write to `.planning/migration-backup/` | HR-triggered file write of potentially-large JSON; server-controlled filename (timestamp); subject to disk-IO DoS on pathological data sizes. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-06-01 | Elevation of Privilege | `POST /admin/cutover/commit` could be invoked by an HR-only user | mitigate | `CutoverService::run(false)` enforces `CUTOVER_ADMIN_PRIVILEGE` (Plan 04-02 + 04-05). REST handler does NOT pre-check (centralizes auth at service layer). Wave-3 forbidden test verifies. |
| T-04-06-02 | Spoofing | A malicious client could trigger `/extra-hours` POST hoping the handler hasn't been flag-gated yet | mitigate | Service-layer gate (Plan 04-04) catches both REST + non-REST callers; REST handler is unchanged but still flows through `ExtraHoursServiceImpl::create`. |
| T-04-06-03 | Tampering | OpenAPI snapshot drift could mask a stealth API break | mitigate | Insta pin file (Task 6); CI fails any non-reviewed change. Determinism verified by 3-run check. |
| T-04-06-04 | Information Disclosure | `CutoverGateDriftReportTO.drift[].sales_person_name` returned in REST response = PII over HTTP | mitigate | All endpoints require authenticated session + privilege gate; PII visible only to authorized HR/cutover_admin. Plus per Phase-3 PII handling: same scope as existing `/sales-person` endpoints already accessible to HR. |
| T-04-06-05 | DI mis-construction | DI ordering mistake (FeatureFlagService AFTER ExtraHoursService) → crash at runtime | mitigate | Task 4 acceptance criterion uses tightened awk to verify line ordering of `Arc::new(...Impl)` constructor calls (first occurrence each, not last); Bin-Boot-Smoke (`timeout 30 cargo run`) verifies actual construction. |
| T-04-06-06 | Repudiation | No audit log of who triggered cutover or profile | accept | Session is logged via existing tower-sessions middleware; cutover_run_id ties DB rows to a single attempt. profile() does NOT write DB rows; the JSON file under `.planning/migration-backup/` carries `profile_run_id` for cross-reference. Full audit trail (signed events) is deferred. |
| T-04-06-07 | Spoofing — Profile endpoint | Unprivileged caller hits `POST /admin/cutover/profile` hoping for early disclosure of legacy data | mitigate | `CutoverService::profile()` gates on HR per Plan 04-07 Task 1 (matches gate-dry-run permission). Wave-3 integration test #15 verifies via REST. |
| T-04-06-08 | Denial of Service — Profile file size | Profile JSON for an installation with 100k+ buckets could exhaust filesystem or block the server thread | accept | profile() runs synchronously inside one HTTP handler; for the current installation scale (well below 100k buckets) this is acceptable. Mitigation if scale grows: stream the JSON or limit bucket count — deferred. Disk-space monitoring is an Operations responsibility. |
| T-04-06-09 | Tampering — Profile file path | Pathological input could traverse outside `.planning/migration-backup/` | mitigate | Filename is server-controlled (`format!("{}.json", unix_timestamp)`) — no user input enters the filesystem path. Fixed prefix; no `..`-traversal possible. |
</threat_model>

<verification>
- `cargo build --workspace` GREEN
- `cargo test --workspace` GREEN (no regressions across all crates)
- `cargo test -p rest --test openapi_snapshot` GREEN with deterministic .snap (3-run check passes)
- `timeout 30 cargo run` boots and logs `"Server listening on..."`
- Tightened `awk` check confirms `Arc::new(FeatureFlagServiceImpl...)` line < `Arc::new(ExtraHoursServiceImpl...)` line in main.rs
- ApiDoc nest list includes `/admin/cutover`
- Router nest list includes `/admin/cutover` (3 routes — gate-dry-run, commit, profile)
- `error_handler` has the new `ExtraHoursCategoryDeprecated` arm
- 6 new DTOs in `rest-types/src/lib.rs` are `ToSchema`-derived (visible in OpenAPI spec)
- Snapshot contains all 3 new endpoints + 6 new schemas (verified via grep in Task 6)
- `CarryoverRebuildServiceDependencies` and `CutoverServiceDependencies` follow the verbatim marker-struct + assoc-type-alias pattern from `AbsenceServiceDependencies` — no Arc fields, no `#[derive(Clone)]`, all dep types declared via `type X = Y;` aliases.
</verification>

<success_criteria>
1. REST surface for cutover endpoints fully wired with utoipa — 3 endpoints (gate-dry-run, commit, profile).
2. SC-1 acceptance satisfied: HR can trigger `POST /admin/cutover/profile` from Production via REST and receives `CutoverProfileTO`; JSON file persisted under `.planning/migration-backup/profile-{ts}.json`.
3. `ServiceError::ExtraHoursCategoryDeprecated` maps to HTTP 403 with the documented JSON body.
4. DI re-order enforced (FeatureFlagService Arc::new before ExtraHoursService Arc::new — verified by tightened awk).
5. Verbatim `*Dependencies` marker-struct + assoc-type-alias pattern used for both new DI blocks (matches `AbsenceServiceDependencies`).
6. OpenAPI snapshot pin file committed; future API changes require explicit review.
7. Bin boots with the full new service tree; ready for Wave-3 E2E integration tests.
</success_criteria>

<output>
After completion, create `.planning/phases/04-migration-cutover/04-06-SUMMARY.md` listing:
- Files modified and the verbatim signature of `cutover_service()` accessor on RestStateDef
- DI re-order: line-numbers before/after for `Arc::new(FeatureFlagServiceImpl...)` and `Arc::new(ExtraHoursServiceImpl...)`
- Verbatim CarryoverRebuildServiceDependencies + CutoverServiceDependencies blocks (assoc-type lists)
- ApiDoc nest entry verbatim (3 paths)
- error_handler arm verbatim
- Snapshot file size + line count + verified schema count (>= 6)
- Bin-boot-smoke result (PASS / FAIL with timestamp + log snippet)
- Hand-off note for Plan 04-07: "REST endpoints are reachable; integration tests can use TowerService::oneshot or full Server-Test-Helper to issue HTTP requests against the wired RestStateImpl. The CutoverServiceImpl is ready for E2E exercise; the profile endpoint is wired (POST /admin/cutover/profile, HR permission), so test #15 in Plan 04-07 must call it via REST — not via service-method-direct."
</output>
