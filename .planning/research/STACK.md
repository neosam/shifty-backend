# Stack Research

**Domain:** Rust/Axum/SQLx/Dioxus shift-planning app — v2.1 feature additions
**Researched:** 2026-07-01
**Confidence:** HIGH (verified against actual workspace source, no web research needed — all patterns are already proven in this codebase)

## Verdict: Zero new dependencies required

Both WST-01 and AVG-01 are achievable entirely within the existing workspace. Every pattern they need already exists and is proven across 38+ phases.

---

## Feature-by-Feature Stack Analysis

### WST-01 — Calendar-Week Status

**What it needs:** a per-(year, ISO-week) workflow status enum `{None, InPlanning, Planned, Locked}` persisted to SQLite, with a Shiftplanner-only lock gate injected into booking/slot write paths.

#### Enum persistence in SQLite — use TEXT, manual match

The codebase uses a single consistent pattern for all enum columns: store as a `TEXT` column in SQLite, convert with a hand-written `match` block in the `TryFrom<&XxxDb>` impl and the `create`/`update` DAO methods.

Evidence: `ExtraHoursCategoryEntity` in `dao_impl_sqlite/src/extra_hours.rs`, `SpecialDayTypeEntity` in `dao_impl_sqlite/src/special_day.rs`. Both use `String` on the `XxxDb` struct and `match entity.day_type.as_str() { "Holiday" => ..., "ShortDay" => ..., value => Err(DaoError::EnumValueNotFound(value.into())) }`.

Do NOT use `sqlx::Type` derive or `strum` for enum-string conversion. Those are valid approaches in other codebases but would be inconsistent here and are not needed.

#### ISO-week composite key in SQLite — follow week_message exactly

`week_message` (created in `migrations/sqlite/20250123000000_add-week-message-table.sql`) already solves this:

```sql
CREATE TABLE week_message (
    id BLOB(16) NOT NULL,
    year INTEGER NOT NULL,
    calendar_week INTEGER NOT NULL,
    ...
    UNIQUE (year, calendar_week)
);
```

The calendar-week status table should follow this schema exactly. Use a partial unique index (`WHERE deleted IS NULL`) to allow soft-delete history, mirroring the `vacation_entitlement_offset` pattern if historical status rows should be kept, or a plain `UNIQUE` constraint like `week_message` if only the current row matters. Given that status transitions are auditable data, the partial-unique-index / soft-delete approach (as in `vacation_entitlement_offset`) is preferable.

Rust-side types: `year: u32` and `calendar_week: u8` in the entity; stored as `i64` in the `XxxDb` struct (SQLx maps SQLite `INTEGER` to `i64`); cast on both sides with `entity.year as i64` and `db.year as u32`. This is the exact pattern in `week_message.rs`, `special_day.rs`, `employee_work_details.rs`.

No separate `chrono` or ISO-week library is needed. The `time` crate (`v0.3.36`, already in `dao_impl_sqlite` and `service_impl`) provides `Date::from_iso_week_date(year, week, weekday)` and `Date::to_iso_week_date()`, which are already used throughout the codebase (verified in `extra_hours.rs`, `shiftplan.rs`, `absence.rs`).

#### Lock gate — use existing SHIFTPLANNER_PRIVILEGE

The existing `SHIFTPLANNER_PRIVILEGE` constant (`"shiftplanner"`, defined in `service/src/permission.rs`) and the `check_permission(SHIFTPLANNER_PRIVILEGE, context)` call pattern (used in `service_impl/src/week_message.rs`, `booking_log.rs`, `shiftplan_edit.rs`) are the correct integration points. No new permission infrastructure is needed.

The gate logic in the booking/slot write paths follows the same pattern already used for the `PaidLimitExceeded` check in `shiftplan_edit.rs`: query the status first, check the role, return a typed `ServiceError` variant mapped to an HTTP error code.

#### Service tier classification

`CalendarWeekStatusService` is a **Basic Service** (manages one aggregate, depends only on its DAO + `PermissionService` + `TransactionDao`). It does not need to depend on `BookingService` or `SlotService`. The lock check belongs in `ShiftplanEditService` (business-logic tier), which already has the booking and slot write paths and can consume the new `CalendarWeekStatusService` as a dep.

---

### AVG-01 — Average actual attendance (flexible-hours employees, vacation excluded)

**What it needs:** a new computation over existing booking/absence data, producing an average-hours-per-week figure with vacation weeks removed from the denominator.

#### Computation home — extend ReportingService

`ReportingService` is already the business-logic tier aggregator for worked-hours data. It already consumes `ShiftplanReportDao` (week-range queries using `year * 100 + calendar_week` arithmetic), `AbsenceService` (for per-week absence data, including vacation), and `EmployeeWorkDetailsService` (for `expected_hours`, used to identify flexible-hour employees). No new service-tier wiring is needed; the new average-attendance method lives directly in `ReportingService`.

The denominator exclusion (drop vacation weeks) reuses the same per-week absence bucketing that `absence.rs` already implements via `day.to_iso_week_date()`. The `time` crate handles all date arithmetic.

#### Numeric computation — no new math crate

The existing `f32` arithmetic throughout the reporting stack is sufficient. Averaging hours over N weeks is `total_hours / denominator_weeks as f32`. No statistical library is needed.

#### Snapshot-schema-version impact

If AVG-01 produces a **read-only aggregate** (computed on request, not persisted to `billing_period_sales_person`) → **no bump** to `CURRENT_SNAPSHOT_SCHEMA_VERSION`.

If discuss-phase decides to persist it as a new `BillingPeriodValueType` row → **bump required** (current baseline: 12). This decision must be made explicit in the discuss-phase before implementation begins.

---

## Recommended Stack (unchanged from existing workspace)

### Core Technologies

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust / Cargo workspace | edition 2021 | Backend language | Already in use; all features implementable within it |
| SQLx | 0.8.2 | Compile-time-checked SQLite queries | Already in `dao_impl_sqlite`; use `query!` / `query_as!` macros |
| Axum | 0.8.7 | REST HTTP layer | Already in `rest`; new endpoints follow existing router pattern |
| utoipa | 5 | OpenAPI annotations | All new endpoints need `#[utoipa::path]` — already available |
| time | 0.3.36 | Date/week arithmetic | ISO week support via `from_iso_week_date` / `to_iso_week_date` — already used |
| Dioxus | 0.6.1 | Frontend WASM UI | Already pinned to 0.6.x; new status badge/selector follows existing patterns |
| serde | 1.0 | JSON serialization for REST | Already in `rest-types`; new DTOs get `#[derive(Serialize, Deserialize, ToSchema)]` |

### Supporting Libraries (already present, reused as-is)

| Library | Version | Purpose | Used for v2.1 |
|---------|---------|---------|--------------|
| mockall | 0.13 | Mock generation for unit tests | New service traits get `#[automock]`; integration tests use in-memory SQLite |
| uuid | 1.8.0 | Row identity | New `calendar_week_status` table rows carry `id BLOB(16)` PK |
| async-trait | 0.1.80 | Async trait definitions | New service/DAO traits use `#[async_trait]` |
| thiserror | 2.0 | Typed error variants | New `ServiceError::CalendarWeekLocked` (or similar) variant |
| chrono | 0.4.39 | Already in service_impl | Not needed for new features; `time` crate handles ISO weeks |

## Installation

No new packages. Both features are implementable with zero `Cargo.toml` changes across all crates.

After adding any new `query!` or `query_as!` macro call, run:

```bash
# From shifty-backend/ root, inside the nix develop shell:
cargo sqlx prepare --workspace
# Then commit the updated .sqlx/ directory.
```

Skipping this step causes CI to fail with `SQLX_OFFLINE=true` even when the local build is green (established in project memory: `reference_sqlx_prepare_after_new_query.md`).

## Alternatives Considered

| Recommended | Alternative | Why Not |
|-------------|-------------|---------|
| TEXT discriminant + manual `match` for enum | `sqlx::Type` derive on enum | Inconsistent with all existing enum columns in this codebase; mixing approaches creates confusion |
| TEXT discriminant + manual `match` for enum | `strum` crate for `Display`/`FromStr` | New dependency with zero benefit; the match blocks are 4-8 lines and are already the established pattern |
| `time` crate ISO week methods | `chrono` for ISO week arithmetic | `chrono` is already in `service_impl` but only for legacy use; `time` is the primary date lib and fully covers ISO week needs |
| Partial unique index (`WHERE deleted IS NULL`) for week status | Plain `UNIQUE (year, calendar_week)` | Soft-delete history is valuable for audit; partial index is the established pattern in `vacation_entitlement_offset` |
| Extend `ReportingService` for AVG-01 | New dedicated `AttendanceService` | Unnecessary tier; `ReportingService` already aggregates exactly this kind of week-range worked-hours data |

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `sqlx::Type` derive on `CalendarWeekStatus` enum | Not used anywhere in codebase; SQLx's SQLite `TEXT` type inference with derive is available but inconsistent here | Manual `match` in `TryFrom` + DAO write methods |
| `strum` / `strum_macros` | No existing usage; adds a new proc-macro dep for a 4-line match block | Manual `match` |
| Any new numeric/stats crate (`statrs`, `nalgebra`, etc.) | AVG-01 is a simple mean over f32 values | `f32` arithmetic in the service method |
| `chrono`'s `IsoWeek` for week-status keying | Would introduce a second date-type system for the same data that `time` already handles | `time::Date::from_iso_week_date()` and `Date::to_iso_week_date()` |
| A separate REST crate just for the new endpoints | All domain REST modules live in `rest/src/` with a sub-module per domain | Add `rest/src/calendar_week_status.rs` and register in `rest/src/lib.rs` |

## Integration Points (patterns to copy verbatim)

| Pattern needed | Copy from |
|----------------|-----------|
| Migration with soft-delete + partial unique index on `(year, calendar_week)` | `migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql` |
| `XxxDb` struct with `i64` year/week + TEXT status, `TryFrom` impl | `dao_impl_sqlite/src/week_message.rs` + `dao_impl_sqlite/src/special_day.rs` |
| `UNIQUE (year, calendar_week)` table constraint | `migrations/sqlite/20250123000000_add-week-message-table.sql` |
| `find_by_year_and_week` DAO query | `dao_impl_sqlite/src/week_message.rs` |
| `SHIFTPLANNER_PRIVILEGE` check in service method | `service_impl/src/week_message.rs` (lines ~79ff) |
| `is_shiftplanner` capture + conditional hard-block in write path | `service_impl/src/shiftplan_edit.rs` (lines ~571ff) |
| Basic-service `gen_service_impl!` with DAO + Permission + TransactionDao only | `service_impl/src/week_message.rs` |
| Business-logic consumer of new Basic service | `service_impl/src/shiftplan_edit.rs` (add `CalendarWeekStatusService` dep) |
| Week-range aggregation over booking hours | `dao_impl_sqlite/src/shiftplan_report.rs` (`extract_shiftplan_report`) |
| Vacation-week extraction per sales person | `service_impl/src/absence.rs` (the `to_iso_week_date()` loops) |
| Dioxus enum selector (badge/dropdown) | Existing `SpecialDay` day-type selector or `ExtraHours` category selector in the frontend |
| `#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, ToSchema)]` on new DTO enum | `rest-types/src/lib.rs` line 242 (`ExtraHoursCategory`) |

## Version Compatibility

No changes to existing dep versions. All new code uses the already-locked versions. The `sqlx prepare --workspace` step is the only post-implementation gate that touches the lockfile area (`.sqlx/` offline cache, not `Cargo.lock`).

## Sources

All findings are verified directly from the workspace source files — no web research was required. Sources:

- `dao_impl_sqlite/src/extra_hours.rs` — TEXT discriminant enum persistence pattern (HIGH)
- `dao_impl_sqlite/src/special_day.rs` — TEXT discriminant + `(year, calendar_week, day_of_week)` keying (HIGH)
- `dao_impl_sqlite/src/week_message.rs` — `(year, calendar_week)` unique composite key, full DAO pattern (HIGH)
- `migrations/sqlite/20250123000000_add-week-message-table.sql` — week table schema (HIGH)
- `migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql` — soft-delete + partial unique index pattern (HIGH)
- `service/src/permission.rs` — `SHIFTPLANNER_PRIVILEGE` constant (HIGH)
- `service_impl/src/week_message.rs` — Basic-service permission gate pattern (HIGH)
- `service_impl/src/shiftplan_edit.rs` — `is_shiftplanner` + hard-block pattern in write path (HIGH)
- `service_impl/src/absence.rs` — `to_iso_week_date()` / vacation-week bucketing (HIGH)
- `dao_impl_sqlite/src/shiftplan_report.rs` — week-range worked-hours aggregation (HIGH)
- `dao_impl_sqlite/Cargo.toml`, `service_impl/Cargo.toml`, `rest/Cargo.toml`, `rest-types/Cargo.toml`, `shifty-dioxus/Cargo.toml` — full dependency inventory (HIGH)

---
*Stack research for: Shifty v2.1 — WST-01 Calendar-Week Status + AVG-01 Avg Attendance*
*Researched: 2026-07-01*
