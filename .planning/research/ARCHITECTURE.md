# Architecture Research

**Domain:** Shifty v2.1 — Feature integration into existing layered Rust + Dioxus monorepo
**Researched:** 2026-07-01
**Confidence:** HIGH (direct codebase inspection; no external sources needed)

## Standard Architecture

### System Overview (existing, unchanged)

```
┌──────────────────────────────────────────────────────────────────┐
│  shifty-dioxus (Dioxus/WASM)                                     │
│  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌────────────────┐  │
│  │  Pages   │  │Components │  │  State   │  │  api.rs fns    │  │
│  └────┬─────┘  └─────┬─────┘  └────┬─────┘  └───────┬────────┘  │
└───────┴──────────────┴─────────────┴────────────────┴────────────┘
                                                        │ HTTP/REST
┌───────────────────────────────────────────────────────┴────────────┐
│  shifty-backend (Axum / Cargo workspace)                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  REST Layer  (rest/)  #[utoipa::path], error_handler wrap  │   │
│  └───────────────────────────────┬─────────────────────────────┘   │
│                                  │ calls                           │
│  ┌────────────────────────────────┴──────────────────────────────┐  │
│  │  Service Layer  (service/ + service_impl/)                   │  │
│  │  ┌──────────────────────────────────────────────────────┐    │  │
│  │  │  Business-Logic Tier (may consume domain services)   │    │  │
│  │  │  ShiftplanEditService, ReportingService,             │    │  │
│  │  │  BookingInformationService, AbsenceService, …        │    │  │
│  │  └──────────────────────────┬───────────────────────────┘    │  │
│  │                             │ calls                          │  │
│  │  ┌──────────────────────────┴───────────────────────────┐    │  │
│  │  │  Basic Tier (entity managers — DAOs only)            │    │  │
│  │  │  BookingService, SlotService, SalesPersonService,    │    │  │
│  │  │  SpecialDayService, WeekStatusService (NEW)          │    │  │
│  │  └──────────────────────────┬───────────────────────────┘    │  │
│  └────────────────────────────┬┴───────────────────────────────┘   │
│                               │ calls                              │
│  ┌────────────────────────────┴──────────────────────────────────┐  │
│  │  DAO Layer  (dao/ + dao_impl_sqlite/)                        │  │
│  └────────────────────────────┬──────────────────────────────────┘  │
│                               │                                     │
│  ┌────────────────────────────┴──────────────────────────────────┐  │
│  │  SQLite  (SQLx compile-time checked)                         │  │
│  └───────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

---

## WST-01: KW-Status — Integration Architecture

### Tier Classification

**WeekStatusService → Basic tier (entity manager)**

Rationale: It manages exactly one aggregate (the `week_status` table). It only needs `WeekStatusDao + PermissionService + TransactionDao`. It does NOT need to call any other domain service to do CRUD on week statuses. Per CLAUDE.md: "Nur DAOs + Permission + Transaction → basic."

**Lock gate enforcement → Business-Logic tier, inside ShiftplanEditService**

Rationale: Lock gate is a cross-entity invariant (a status on one entity constrains writes to other entities — bookings and slots). Cross-entity invariants live in the business-logic tier. `ShiftplanEditService` is already the business-logic write aggregate for shiftplan mutations.

### New Components per Layer

**Database layer — Migration (migrations/sqlite/)**

```sql
-- New file: 20260701000000_create-week-status.sql
CREATE TABLE week_status (
    id          TEXT NOT NULL PRIMARY KEY,
    year        INTEGER NOT NULL,
    calendar_week INTEGER NOT NULL,
    status      TEXT NOT NULL DEFAULT 'None',  -- 'None' | 'InPlanning' | 'Planned' | 'Locked'
    created     TEXT NOT NULL,
    deleted     TEXT,
    version     TEXT NOT NULL,
    UNIQUE(year, calendar_week, deleted)        -- one live row per (year, week)
);
```

**DAO layer**

New trait `dao::week_status::WeekStatusDao` (mirrors `dao::week_message::WeekMessageDao` pattern):
- `find_by_year_and_week(year, week, tx) -> Option<WeekStatusEntity>`
- `find_by_year(year, tx) -> Vec<WeekStatusEntity>`
- `create(entity, process, tx) -> ()`
- `update(entity, process, tx) -> ()`
- `delete(id, process, tx) -> ()`

New impl `dao_impl_sqlite::week_status::WeekStatusDaoImpl`.

**Service layer — Basic tier**

New trait `service::week_status::WeekStatusService`:
- `get(year, week, context, tx) -> Option<WeekStatus>` (returns `WeekStatus::None` if no row exists)
- `get_by_year(year, context, tx) -> Arc<[WeekStatus]>`
- `set(year, week, status, context, tx) -> WeekStatus` (upsert — create or update)
- `delete(year, week, context, tx) -> ()`
- Convenience: `is_locked(year, week, tx) -> bool` (internal helper, takes `Transaction` not `Option<Transaction>`)

`WeekStatus` domain struct:
```rust
pub enum WeekStatusEnum { None, InPlanning, Planned, Locked }
pub struct WeekStatus {
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub status: WeekStatusEnum,
    pub version: Uuid,
}
```

Permission gate inside `WeekStatusService::set`: only shiftplanner may change status. Readable by all authenticated users.

New impl `service_impl::week_status::WeekStatusServiceImpl` via `gen_service_impl!`:
```rust
gen_service_impl! {
    struct WeekStatusServiceImpl: WeekStatusService = WeekStatusServiceDeps {
        WeekStatusDao: dao::week_status::WeekStatusDao<Transaction = Self::Transaction> = week_status_dao,
        PermissionService: service::PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao,
        ClockService: service::clock::ClockService = clock_service,
        UuidService: service::uuid_service::UuidService = uuid_service
    }
}
```

**New ServiceError variant:**
```rust
#[error("Week {year}-W{week} is locked")]
WeekLocked { year: u32, week: u8 },
```

Maps to HTTP 423 (Locked) in the REST error handler.

**Service layer — Business-Logic tier (modifications to ShiftplanEditService)**

`ShiftplanEditService` gains `WeekStatusService` as a new dependency:
```rust
gen_service_impl! {
    struct ShiftplanEditServiceImpl: ShiftplanEditService = ShiftplanEditServiceDeps {
        // ... existing deps ...
        WeekStatusService: service::week_status::WeekStatusService<
            Context = Self::Context,
            Transaction = Self::Transaction
        > = week_status_service,  // NEW
    }
}
```

**REST layer**

New module `rest::week_status` with `#[utoipa::path]` annotations:
- `GET /week-status/{year}/{week}` → get current status (or None)
- `PUT /week-status/{year}/{week}` → set status (shiftplanner only)
- `DELETE /week-status/{year}/{week}` → reset to None (shiftplanner only)
- `GET /week-status/year/{year}` → all statuses for a year (for frontend bulk-load)

**rest-types (shared DTOs)**

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, ToSchema)]
pub enum WeekStatusEnumTO { None, InPlanning, Planned, Locked }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, ToSchema)]
pub struct WeekStatusTO {
    #[serde(default)] pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub status: WeekStatusEnumTO,
    #[serde(rename = "$version", default)] pub version: Uuid,
}
```

### Lock Gate Injection Sites (exhaustive)

The gate logic pattern (applied at the top of each write method, after acquiring the transaction, before any mutation):

```rust
// pseudo-code — applied in ShiftplanEditServiceImpl
let locked = self.week_status_service
    .is_locked(year, week, tx.clone())
    .await?;
if locked && !is_shiftplanner {
    return Err(ServiceError::WeekLocked { year, week });
}
```

**Complete list of write paths and gate injection sites:**

| REST Endpoint | Calls | Gate injection | Gate checks week | Shiftplanner bypass |
|---|---|---|---|---|
| `PUT /shiftplan-edit/slot/{year}/{week}` | `ShiftplanEditService::modify_slot` | top of method, before slot lookup | `(change_year, change_week)` | Yes — shiftplanner is the only caller via `shiftplan.edit` permission already |
| `PUT /shiftplan-edit/slot/{year}/{week}/single-week` | `ShiftplanEditService::modify_slot_single_week` | top of method, after permission check | `(change_year, change_week)` | Yes — same |
| `DELETE /shiftplan-edit/slot/{slot_id}/{year}/{week}` | `ShiftplanEditService::remove_slot` | top of method, after permission check | `(change_year, change_week)` | Yes — same |
| `POST /shiftplan-edit/booking` | `ShiftplanEditService::book_slot_with_conflict_check` | after is_shiftplanner check, before slot lookup | `(booking.year, booking.calendar_week as u8)` | Yes — is_shiftplanner already computed |
| `POST /shiftplan-edit/copy-week` | `ShiftplanEditService::copy_week_with_conflict_check` | top of method, after permission check | `(to_year, to_calendar_week)` | Yes — propagates to inner `book_slot_with_conflict_check` calls which also re-check |
| `DELETE /booking/{id}` | currently `BookingService::delete` directly | **MUST be re-routed** — see below | `(booking.year, booking.calendar_week as u8)` | Conditional |

**The `DELETE /booking/{id}` bypass — required fix:**

Currently `rest/src/booking.rs::delete_booking` calls `booking_service.delete(id, ...)` directly, bypassing all business-logic gates. This is the only ungated write path that a non-shiftplanner user can reach (they can delete their own bookings).

Fix: Add a new method `ShiftplanEditService::delete_booking(booking_id, context, tx)` that:
1. Loads the booking via `BookingService::get(booking_id)` to extract `(year, calendar_week)`
2. Checks lock status for that week
3. Delegates to `BookingService::delete(booking_id, context, tx)` for the actual deletion

Change `rest/src/booking.rs::delete_booking` to route through `shiftplan_edit_service.delete_booking(...)` instead. The `BookingService::delete` method itself is unchanged (Regression-Lock: service method signature and behavior unmodified). Only the REST routing changes.

**Legacy paths — documented bypass (known gaps):**

| REST Endpoint | Status | Recommendation |
|---|---|---|
| `POST /booking` (legacy create) | Unguarded bypass | Document as known gap. Frontend already uses `POST /shiftplan-edit/booking`. Acceptable because: (a) only shiftplanner or self-booking is allowed at service level; (b) Locked-week gate for non-shiftplanners is the critical path; a shiftplanner using the legacy path is explicitly allowed through the lock anyway. |
| `POST /booking/copy` (legacy copy-week) | Unguarded bypass | Same rationale — only shiftplanner can call copy-week (it checks `shiftplan.edit` permission). No non-shiftplanner bypass risk. |

The conclusion is: only `DELETE /booking/{id}` is a genuine non-shiftplanner bypass that must be fixed. The two legacy `POST /booking` endpoints are shiftplanner-or-self-only at the service level and therefore no non-shiftplanner can bypass the lock via them.

**Note on `copy_week_with_conflict_check` inner loop:** The outer method checks `(to_year, to_calendar_week)` at entry. The inner calls to `book_slot_with_conflict_check` re-check each booking's `(year, calendar_week)`. Since all target bookings share the same `to_year / to_calendar_week`, both checks are consistent. No duplicate-check concern — the inner check is belt-and-suspenders.

### DI Construction Order (shifty_bin/src/main.rs)

```
1. (existing) permission_service, clock_service, uuid_service
2. (existing) slot_service, sales_person_service, booking_service  [basic tier]
3. (NEW) week_status_dao = WeekStatusDaoImpl::new(pool)
4. (NEW) week_status_service = WeekStatusServiceImpl { week_status_dao, permission_service,
         clock_service, uuid_service, transaction_dao }            [basic tier]
5. (existing) absence_service, carryover_service, reporting_service [business-logic tier]
6. (modified) shiftplan_edit_service = ShiftplanEditServiceImpl { ...,
         week_status_service }                                      [business-logic tier]
```

---

## AVG-01: Average Attendance — Integration Architecture

### Tier Classification

**Computation: ReportingService (existing business-logic tier)**

Rationale:
- AVG-01 aggregates data from multiple domain entities: bookings (via `ShiftplanReportService`), worked hours (via `ExtraHoursService`), absence periods (via `AbsenceService`), employee work details (via `EmployeeWorkDetailsService` to identify flexible employees).
- All these sources are already in `ReportingService`'s existing dependency set.
- No new domain service dep is required — this is a new method on the existing `ReportingService` trait, not a new service.
- "Flexible hours" = employees whose active `EmployeeWorkDetails` has `is_dynamic == true` (already a field on the domain struct).

### Snapshot Version Bump: NO

**Verdict: AVG-01 does NOT require a `CURRENT_SNAPSHOT_SCHEMA_VERSION` bump.**

Rationale per CLAUDE.md rules:
- AVG-01 does not add a new persisted `BillingPeriodValueType` to `billing_period_sales_person`.
- AVG-01 does not change the computation of any existing persisted `value_type`.
- AVG-01 is a pure read-aggregate: it computes on-the-fly from existing source data and returns a result via REST. Nothing is written to the billing period snapshot tables.
- The existing `average_worked_hours_per_week` function in `service/src/reporting.rs` (formula A-22-1) is a pure function returning `EmployeeWeeklyStatistics` — it is already implemented and not persisted.
- AVG-01 extends this pattern for the filtered set of flexible employees, producing a new read-only endpoint.

CLAUDE.md: "You do NOT need to bump for: purely additive changes that do not touch the snapshot's value_types."

### Data Sources Read by AVG-01

| Source | Service | Purpose |
|---|---|---|
| `EmployeeWorkDetails` | `EmployeeWorkDetailsService` | Identify flexible employees (`is_dynamic == true`); get expected-hours contract per week |
| `SalesPerson` | `SalesPersonService` | Employee identity, `is_paid` filter if needed |
| Bookings / shiftplan hours | `ShiftplanReportService` | Actual worked hours per week per employee |
| `ExtraHours` | `ExtraHoursService` | Overtime, sick leave, etc. that count as worked |
| `AbsencePeriod` | `AbsenceService` | Identify fully-absent weeks to exclude from denominator (vacation / sick-leave / unpaid-leave) |

All these are already in `ReportingServiceImpl`'s dep set. No new dependencies needed.

**Open definition questions (for discuss-phase, noted here for completeness):**

- Which absence categories exclude a week from the denominator? The existing A-22-1 formula (already coded) excludes weeks where `vacation_hours + sick_leave_hours + unpaid_leave_hours + holiday_hours > 0` AND `overall_hours == 0`. Confirm this is the right rule for AVG-01.
- Report granularity: per-employee or aggregate? The discuss-phase todo says "Schnitt der tatsächlich geleisteten Anwesenheit" but doesn't specify per-employee vs fleet average.
- Time range: year-to-date, full year, billing period?

### New Components per Layer

**Service layer**

New method on `service::reporting::ReportingService` trait:
```rust
async fn get_attendance_average_for_flexible_employees(
    &self,
    year: u32,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[EmployeeAttendanceAverage]>, ServiceError>;
```

New struct in `service::reporting`:
```rust
pub struct EmployeeAttendanceAverage {
    pub sales_person: Arc<SalesPerson>,
    pub average_attended_hours_per_week: f32,
    pub included_weeks: u32,
    pub total_attended_hours: f32,
    pub is_dynamic: bool,
}
```

No new deps on `ReportingServiceImpl` — delegates to existing `get_report_for_employee` and uses the existing `average_worked_hours_per_week` pure function.

**REST layer**

New endpoint in `rest/src/report.rs`:
- `GET /report/attendance-average/{year}` → returns `Arc<[EmployeeAttendanceAverageTO]>`, HR-gated

**rest-types (shared DTOs)**

```rust
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct EmployeeAttendanceAverageTO {
    pub sales_person_id: Uuid,
    pub sales_person_name: Arc<str>,
    pub average_attended_hours_per_week: f32,
    pub included_weeks: u32,
    pub total_attended_hours: f32,
    pub is_dynamic: bool,
}
```

---

## Dioxus Frontend Wiring Points

### WST-01 Frontend

**api.rs — new functions:**
```rust
pub async fn get_week_status(config, year, week) -> Option<WeekStatusTO>
pub async fn get_week_statuses_for_year(config, year) -> Vec<WeekStatusTO>
pub async fn set_week_status(config, year, week, status: WeekStatusEnumTO) -> WeekStatusTO
```

**state/shiftplan.rs — new Signal:**
```rust
week_status: Signal<Option<WeekStatusTO>>   // current week's status
```

**page/shiftplan.rs or component/shiftplan_tab_bar.rs — rendering:**
- Fetch `week_status` on week-navigation (alongside existing week-message fetch).
- Render a status badge next to the week header (e.g., coloured pill: grey=None, yellow=InPlanning, green=Planned, red=Locked).
- For shiftplanners: dropdown/button group to cycle through the four statuses.
- For non-shiftplanners: badge is read-only.
- Disable booking buttons and slot-edit triggers when `status == Locked && !is_shiftplanner`.
- Handle HTTP 423 response from any write endpoint: show non-blocking inline banner "Diese Woche ist gesperrt" (+ de/en/cs i18n keys).

**Error handling pattern (inline banner, per UX constraint from memory):**
No modal dialogs — the locked-week error uses the existing non-blocking inline warning banner pattern (same as paid-limit warnings).

### AVG-01 Frontend

**api.rs — new function:**
```rust
pub async fn get_attendance_averages(config, year) -> Vec<EmployeeAttendanceAverageTO>
```

**New page or section in existing report:** Either a new page route `Route::AttendanceAverageReport { year }` or a new tab/section in the existing employee view. Exact placement is an open discussion-phase question.

**i18n keys needed (de/en/cs for all):**
- `attendance_average_report_title`
- `average_attended_hours_per_week`
- `included_weeks`
- `flexible_employees_only`
- Plus week-status keys: `week_status_none`, `week_status_in_planning`, `week_status_planned`, `week_status_locked`

---

## Recommended Build Order

The build order follows data-dependency: schema before DAO, DAO before service, service before REST, REST before frontend.

### WST-01 Build Order

1. **Migration** — `week_status` table, UNIQUE constraint on `(year, calendar_week, deleted IS NULL)`.
2. **DAO trait + impl** — `dao::week_status` + `dao_impl_sqlite::week_status`. Run `cargo sqlx prepare --workspace`.
3. **Service trait** — `service::week_status::WeekStatusService` trait + `WeekStatus`/`WeekStatusEnum` structs + `ServiceError::WeekLocked` variant.
4. **Service impl (Basic tier)** — `service_impl::week_status::WeekStatusServiceImpl`.
5. **DI wiring in main.rs** — Instantiate in basic-tier block; pass to `ShiftplanEditService` in business-logic block.
6. **Lock gate injection in ShiftplanEditService** — Add `week_status_service` dep, inject gate into the five methods listed above.
7. **`DELETE /booking/{id}` re-routing** — Add `ShiftplanEditService::delete_booking` method; update REST handler.
8. **REST CRUD for week-status** — `rest::week_status` module with `#[utoipa::path]` annotations; register in `ApiDoc`.
9. **rest-types DTOs** — `WeekStatusTO`, `WeekStatusEnumTO` (with `ToSchema`). Add `From<>` impls (service → DTO).
10. **Tests** — Unit tests for lock gate (mock WeekStatusService returning Locked; verify write paths return WeekLocked error). Integration tests for CRUD + lock enforcement e2e.
11. **Frontend** — `api.rs` functions, Signal in shiftplan state, badge rendering, disabled controls, inline banner for 423.
12. **i18n** — Add week-status keys to de.rs / en.rs / cs.rs.

### AVG-01 Build Order

1. **Service trait method** — `ReportingService::get_attendance_average_for_flexible_employees` + `EmployeeAttendanceAverage` struct. (No new deps or migration needed.)
2. **Service impl** — Implement on `ReportingServiceImpl` using existing data sources and `average_worked_hours_per_week` pure function.
3. **rest-types DTO** — `EmployeeAttendanceAverageTO` with `ToSchema`.
4. **REST endpoint** — `GET /report/attendance-average/{year}` in `rest/src/report.rs` with `#[utoipa::path]`, HR-gated.
5. **Tests** — Unit tests using existing test data setup patterns in `service_impl/src/test/`.
6. **Frontend** — `api.rs` function, new page or section, i18n keys.

**Dependency between WST-01 and AVG-01:** Independent. Either can be built first. WST-01 is more impactful (changes existing write paths) and should be built and verified before AVG-01 (which is purely additive/read-only).

---

## Component Responsibilities Summary

| Component | Status | Tier | Communicates With |
|---|---|---|---|
| `dao::week_status::WeekStatusDao` | NEW | DAO | SQLite via SQLx |
| `dao_impl_sqlite::week_status::WeekStatusDaoImpl` | NEW | DAO impl | SQLite pool |
| `service::week_status::WeekStatusService` | NEW | Basic | WeekStatusDao, PermissionService, TransactionDao |
| `service_impl::week_status::WeekStatusServiceImpl` | NEW | Basic impl | As above |
| `service_impl::shiftplan_edit::ShiftplanEditServiceImpl` | MODIFIED | Business-Logic | Adds WeekStatusService dep; lock gate in 5 methods + new `delete_booking` |
| `service::reporting::ReportingService` | MODIFIED (trait) | Business-Logic | Adds new method signature |
| `service_impl::reporting::ReportingServiceImpl` | MODIFIED (impl) | Business-Logic | Implements AVG-01 using existing deps |
| `service::ServiceError` | MODIFIED | — | New `WeekLocked` variant → HTTP 423 |
| `rest::week_status` | NEW | REST | `WeekStatusService` |
| `rest::booking::delete_booking` handler | MODIFIED | REST | Routes to `ShiftplanEditService::delete_booking` |
| `rest::report` | MODIFIED | REST | New attendance-average endpoint |
| `rest-types` | MODIFIED | Shared DTO | New `WeekStatusTO`, `WeekStatusEnumTO`, `EmployeeAttendanceAverageTO` |
| `shifty_bin::main.rs` | MODIFIED | DI root | WeekStatusServiceImpl instantiation + wiring |
| `shifty-dioxus::api.rs` | MODIFIED | Frontend API | New week-status + AVG-01 fetch functions |
| `shifty-dioxus::state::shiftplan` | MODIFIED | Frontend state | `week_status: Signal<Option<WeekStatusTO>>` |
| `shifty-dioxus::page::shiftplan` / tab bar | MODIFIED | Frontend UI | Status badge, controls, 423 inline banner |
| `shifty-dioxus::i18n` | MODIFIED | i18n | New keys in de/en/cs for week status + AVG-01 |

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Lock Gate in BasicTier (WeekStatusService or BookingService)

**What people do:** Put the lock check in `BookingService::create` or `BookingService::delete` by adding `WeekStatusService` as a basic-tier dependency.

**Why it's wrong:** CLAUDE.md is explicit: "Basic services konsumieren KEINE anderen Domain-Services." Adding `WeekStatusService` (a domain service, even if also basic) as a dependency of `BookingService` violates the tier rule and creates a potential for cycles. It also breaks the test isolation of basic services (which use simple DAO mocks only).

**Do this instead:** Gate in `ShiftplanEditService` (business-logic tier) and add a new `ShiftplanEditService::delete_booking` method to cover the `DELETE /booking/{id}` REST path.

### Anti-Pattern 2: Adding AVG-01 Computation to BillingPeriodReportService

**What people do:** Because AVG-01 is an "average" it might seem like billing-period data. Someone might add it as a new `BillingPeriodValueType` and persist it in the snapshot.

**Why it's wrong:** AVG-01 is defined as a pure read-aggregate ("reines Read-Aggregat"). Persisting it would: (a) require a snapshot version bump, (b) require the complex snapshot invalidation/re-run machinery, (c) add brittleness for a display-only metric that is cheap to recompute.

**Do this instead:** Implement as a new `ReportingService` method that computes on-the-fly. No persistence, no bump.

### Anti-Pattern 3: Checking Lock Status in the REST Layer Handler

**What people do:** To avoid adding `WeekStatusService` to `ShiftplanEditService`, add a pre-check in the Axum handler that fetches the week status before calling the service.

**Why it's wrong:** The REST layer should not contain business logic. The lock invariant is a business rule. If it lives in the REST handler, it is bypassed by any internal caller that goes directly to the service (e.g., tests, future scheduled jobs). Business rules must live in the service layer.

**Do this instead:** Lock gate in `ShiftplanEditService` where all mutation paths converge.

### Anti-Pattern 4: Separate "LockGateService" Wrapping ShiftplanEditService

**What people do:** Create a new `LockedShiftplanEditService` that wraps `ShiftplanEditService` and adds the gate around each method.

**Why it's wrong:** Indirection without benefit. `ShiftplanEditService` already is the natural owner; adding a wrapper doubles the mock surface and makes DI more complex.

**Do this instead:** Add `WeekStatusService` as a new dep to `ShiftplanEditServiceImpl` and inject the gate inline. This is the exact pattern used for `ToggleService` (D-24-08: paid-limit toggle is checked inline in `book_slot_with_conflict_check`).

---

## Sources

- Direct codebase inspection: `service_impl/src/shiftplan_edit.rs`, `service_impl/src/booking.rs`, `rest/src/shiftplan_edit.rs`, `rest/src/booking.rs`, `service/src/shiftplan_edit.rs`, `service/src/toggle.rs`, `service/src/reporting.rs`, `dao/src/week_message.rs`, `shifty_bin/src/main.rs`
- Project conventions: `CLAUDE.md` (service-tier rules, snapshot versioning rules, transaction pattern)
- Project charter: `.planning/PROJECT.md` (WST-01/AVG-01 feature descriptions, Regression-Lock references, snapshot version history)
- Frontend: `shifty-dioxus/src/api.rs`, `shifty-dioxus/CLAUDE.md`

---
*Architecture research for: Shifty v2.1 WST-01 + AVG-01 integration into existing layered Rust + Dioxus monorepo*
*Researched: 2026-07-01*
