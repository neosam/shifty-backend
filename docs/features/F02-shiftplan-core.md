# Feature: Shiftplan Core (Slots, catalog, editor, view)

> **Short form:** The business core of Shifty — defines when work can be
> performed (Slots), groups them into shift plans (catalog), renders the
> weekly/daily view with Bookings (view), and provides the atomic write
> aggregate for all Slot/Booking mutations (edit).

**Cluster ID:** F02
**Status:** production
**First introduced:** Slots since Milestone 0 (migration `20240502113031`);
catalog / multi-plan support since v2.x (migration `20260330000000`).
**Responsible crates:**
`service::slot`, `service::shiftplan`, `service::shiftplan_catalog`,
`service::shiftplan_edit`, `service::shiftplan_report`, `service_impl::…`,
`dao::slot`, `dao::shiftplan`, `dao::shiftplan_report`,
`rest::slot`, `rest::shiftplan_catalog`, `rest::shiftplan_edit`,
`rest::shiftplan` (`shiftplan-info`).

---

## 1. What is it? (Business context)

A **Slot** is a weekly recurring time window on a specific weekday
(e.g. "Tuesday 10:00–12:00") in which employees can be booked. A Slot
only defines the *possibility* of working, not a specific person — that
is done by Booking (see [`F03-booking.md`](./F03-booking.md)).

A **Shiftplan** (shift plan) is a named collection of Slots. Since
v2.x Shifty supports **multiple parallel shift plans** per instance
(e.g. "Store shift" and "Office shift"), each with its own Slot
structure. The default plan is called `main` (hard-coded UUID from
migration `20260330000000_add-shiftplan-table.sql`).

The **catalog** (`ShiftplanService` / `shiftplan_catalog`) manages the
metadata of the shift plans (name, `is_planning` flag, versioning).
The **weekly/daily view** (`ShiftplanViewService`) aggregates Slot
definitions + Bookings + Sales Persons into a renderable structure for
the frontend. The **edit aggregate** (`ShiftplanEditService`) is the
sole write path for structural changes (modify/delete Slot, copy week,
create Booking with conflict awareness, respect week lock, record
Vacation).

**Example workflow from a user's perspective:**

1. HR/Shiftplanner opens the Shiftplan page, selects a shift plan and
   an ISO calendar week.
2. FE loads the week via `GET /shiftplan-info/{shiftplan_id}/{year}/{week}`;
   Slots and existing Bookings are rendered side by side.
3. Clicking on a Slot opens the Slot editor
   (`component/slot_edit.rs`); the Shiftplanner changes `min_resources` /
   `max_paid_employees` or the time window.
4. Save → `PUT /shiftplan-edit/slot/{year}/{week}` → backend splits
   the Slot via `valid_from`/`valid_to` into two segments and migrates
   open Bookings atomically to the new version (`modify_slot`).
5. An employee signs up on the free Slot → FE calls
   `POST /shiftplan-edit/booking` → backend checks the week lock,
   the paid-employee limit, and AbsencePeriod/manual-unavailable
   conflicts and returns warnings (see [`F03-booking.md`](./F03-booking.md),
   [`F05-absence-system.md`](./F05-absence-system.md)).

## 2. Business rules

**Slot rules** (`service::slot::Slot`, verified in
`service_impl/src/slot.rs`):

- **Time window:** `from <= to`; violation → `ServiceError::TimeOrderWrong`
  (`service_impl/src/slot.rs:225-227`).
- **Weekday:** `day_of_week` is immutable after creation; changing via
  `update_slot` yields `ValidationFailureItem::ModificationNotAllowed`
  (`service_impl/src/slot.rs:314-318`). The same applies to `from`, `to`,
  `valid_from` (`service_impl/src/slot.rs:319-329`).
- **Validity:** `valid_from` (inclusive) and `valid_to` (optional,
  inclusive). `valid_to < valid_from` → `ServiceError::DateOrderWrong`.
- **Slot overlap:** Within the same shift plan and the same
  weekday, two active Slots may not overlap when their
  `valid_from`/`valid_to` ranges collide
  (`service_impl/src/slot.rs::test_overlapping_slots`, lines 55-60;
  call in `create_slot`, lines 234-246). Overlap definition:
  strict overlap OR exact coincidence
  (`slot_1.from == slot_2.from && slot_1.to == slot_2.to`) — edge
  contact (`slot_1.to == slot_2.from`) is allowed.
- **`min_resources`:** Expected minimum staffing per Slot (default 2
  from migration `20240813080347_add-column-min-resources.sql`). Currently
  used primarily as a UI hint; FE renders a warning on under-coverage.
  **[To verify]** whether the backend hard-blocks this.
- **`max_paid_employees`:** Optional. Cap on how many **paid** persons
  may be booked per Slot+week. Soft limit — takes effect only with the
  toggle `paid_limit_hard_enforcement`
  (`service_impl/src/shiftplan_edit.rs:618-660`, D-24-02/-08). Without
  the toggle a warning is returned; Shiftplanners always bypass the
  hard limit.
- **Availability window across weekdays:** A Slot exists on exactly one
  weekday (`day_of_week`). The distribution "one Slot per weekday
  9-10 am" is modeled by **multiple Slot rows** (see the default Slot
  set, chapter 3).
- **Optimistic locking:** Each Slot has a `version` UUID; update with
  wrong version → `ServiceError::EntityConflicts`
  (`service_impl/src/slot.rs:300-306`). On successful update a new
  version UUID is issued (`slot.rs:340-343`).
- **Soft delete:** `deleted` is a timestamp; `delete_slot` sets it,
  does not delete the row (`slot.rs:270-283`). All DAO queries filter
  `WHERE deleted IS NULL`.

**Shiftplan catalog rules** (`service_impl/src/shiftplan_catalog.rs`):

- Creation/update require `shiftplanner` (`SHIFTPLANNER_PRIVILEGE`).
- `Shiftplan.is_planning` marks a plan as "in planning" (FE flag,
  effect **[To verify]** in detail).
- `Shiftplan.name` freely chosen; the default row `main` with fixed UUID
  `00000000-0000-4000-8000-000000000001` is created by the migration
  and must not be deleted (reference of all legacy Slots).

**Shiftplan edit rules** (`service_impl/src/shiftplan_edit.rs`):

- **Permission `shiftplan.edit`** for all Slot mutations (`modify_slot`,
  `modify_slot_single_week`, `remove_slot`) — separate privilege from
  `shiftplanner` (separated role since migration
  `20241118165756_add-role-shiftplan-edit.sql`).
- **Week-lock gate** (`assert_week_not_locked`,
  `shiftplan_edit.rs:908+`) runs BEFORE every mutation. The
  `book_slot_with_conflict_check` path also gates against it —
  `shiftplanner` alone is not enough, `shiftplan.edit` is used as bypass
  (see CR-01 comment `shiftplan_edit.rs:591-604`).
- **Slot split semantics** (`modify_slot`, `shiftplan_edit.rs:56-151`):
  changing a Slot from `change_year`/`change_week` creates a **new
  Slot** (with the new values and `valid_from = Monday of change_week`)
  and closes the old one with `valid_to = Sunday of change_week-1`.
  All Bookings from `change_week` on are re-pointed to the new Slot
  (old Booking row soft-deleted, new row created) — all in one
  transaction (see chapter 7 "Edge cases").
- **`modify_slot_single_week`** (D-35-01 Approach B, since Phase 35):
  3-segment split for a **one-time** exception in exactly one calendar
  week. Creates segment 1 (original until CW-1), segment 2 (exception
  in CW), and segment 3 (restoration from CW+1). Bookings of the CW →
  segment 2, later Bookings → segment 3.
- **Copy week** (`copy_week_with_conflict_check`,
  `shiftplan_edit.rs:788+`): iterates Bookings of the source week and
  calls `book_slot_with_conflict_check` per Booking. Accumulates
  cross-source warnings without dedup (D-Phase3-15).

## 3. Data model

### Tables

| Table | Purpose | Important columns |
| --- | --- | --- |
| `slot` | Recurring time Slot | `id`, `day_of_week` (1=Mo … 7=Su), `time_from`, `time_to`, `valid_from`, `valid_to`, `min_resources`, `max_paid_employees`, `shiftplan_id`, `deleted`, `update_version` |
| `shiftplan` | Metadata catalog | `id`, `name`, `is_planning`, `deleted`, `update_version` |
| `sales_person_shiftplan` | N:M mapping person→plan | `sales_person_id`, `shiftplan_id`, `permission_level` (`available` / `planner_only`) |
| `bookings_view` (view) | Denormalized read optimization | Bookings + `sales_person.name` + `slot.day_of_week` / `time_from` / `time_to` + `shiftplan.name` |

### Migrations

Chronological build-up history of the Slot/Shiftplan tables:

- `20240502113031_add-slot.sql` — base table `slot` (without
  `min_resources`, without `shiftplan_id`, without `max_paid_employees`).
- `20240619085745_default-slots.sql` — **default Slot set:** 63 Slots
  (Mo-Sa, mostly 09:00-19:30 in 1h blocks; last Slot 19:00-19:30).
  All with `valid_from = 2020-01-01`, no `valid_to`. Migrations inserts
  them with fixed IDs+versions.
- `20240813080347_add-column-min-resources.sql` — `min_resources INTEGER
  DEFAULT 2 NOT NULL`.
- `20260330000000_add-shiftplan-table.sql` — multi-plan support: creates
  `shiftplan` table, adds `slot.shiftplan_id` FK, backfills all
  legacy Slots to `main` (UUID `…0001`), extends `bookings_view` with
  `shiftplan_name`.
- `20260331000000_add-sales-person-shiftplan.sql` — N:M table
  `sales_person_shiftplan`.
- `20260402000000_add-permission-level-to-sales-person-shiftplan.sql` —
  adds `permission_level`.
- `20260503221640_add-max-paid-employees-to-slot.sql` —
  `max_paid_employees INTEGER` (nullable).

### Relationships

```
shiftplan ──1:N── slot ──1:N── booking
     │
     └──N:M── sales_person (via sales_person_shiftplan)
```

A Slot belongs (since v2.x) to exactly one shift plan; historically
(prior to migration `20260330000000`) `slot.shiftplan_id NULL` was
allowed and got backfilled to `main` via the migration's UPDATE
statement.

## 4. Service API

### Traits

**Basic-Tier** (only DAO + Permission + Transaction as deps):

- `service::slot::SlotService` (`service/src/slot.rs:98-154`) — CRUD +
  read-by-week. Consumes only `SlotDao`, `PermissionService`,
  `ClockService`, `UuidService`, `TransactionDao`.
- `service::shiftplan_catalog::ShiftplanService`
  (`service/src/shiftplan_catalog.rs:45-84`) — CRUD for the catalog rows.
  Consumes only `ShiftplanDao`, `PermissionService`, `ClockService`,
  `UuidService`, `TransactionDao`
  (`service_impl/src/shiftplan_catalog.rs:15-23`).

**Business-Logic-Tier** (consume other services):

- `service::shiftplan::ShiftplanViewService`
  (`service/src/shiftplan.rs:86-143`) — read aggregate. Deps:
  `SlotService`, `BookingService`, `SalesPersonService`,
  `SpecialDayService`, `ShiftplanService`, `AbsenceService`,
  `SalesPersonUnavailableService`, `ToggleService`
  (`service_impl/src/shiftplan.rs:216-231`).
- `service::shiftplan_edit::ShiftplanEditService`
  (`service/src/shiftplan_edit.rs:35-157`) — write aggregate. Deps:
  `SlotService`, `BookingService`, `CarryoverService`, `ReportingService`,
  `SalesPersonService`, `SalesPersonUnavailableService`,
  `EmployeeWorkDetailsService`, `ExtraHoursService`, `AbsenceService`,
  `ToggleService`, `WeekStatusService` +
  utility deps (`service_impl/src/shiftplan_edit.rs:27-48`).
- `service::shiftplan_report::ShiftplanReportService`
  (`service/src/shiftplan_report.rs:33-63`) — aggregates hours per
  person from raw Booking times. Deps: `ShiftplanReportDao`,
  `SpecialDayService`, `ToggleService`, `TransactionDao`
  (`service_impl/src/shiftplan_report.rs:36-43`).

### Most important method signatures

```rust
// SlotService
async fn create_slot(&self, slot: &Slot, ctx, tx) -> Result<Slot, ServiceError>;
async fn update_slot(&self, slot: &Slot, ctx, tx) -> Result<(), ServiceError>;
async fn delete_slot(&self, id: &Uuid, ctx, tx) -> Result<(), ServiceError>;
async fn get_slots_for_week(&self, year, week, shiftplan_id: Uuid, ctx, tx)
    -> Result<Arc<[Slot]>, ServiceError>;

// ShiftplanViewService
async fn get_shiftplan_week(&self, shiftplan_id, year, week, ctx, tx)
    -> Result<ShiftplanWeek, ServiceError>;
async fn get_shiftplan_week_for_sales_person(&self, shiftplan_id, year, week,
    sales_person_id, ctx, tx) -> Result<ShiftplanWeek, ServiceError>;

// ShiftplanEditService (excerpt)
async fn modify_slot(&self, slot: &Slot, change_year, change_week, ctx, tx)
    -> Result<Slot, ServiceError>;
async fn modify_slot_single_week(&self, slot: &Slot, change_year, change_week,
    ctx, tx) -> Result<Slot, ServiceError>;
async fn remove_slot(&self, slot: Uuid, change_year, change_week, ctx, tx)
    -> Result<(), ServiceError>;
async fn book_slot_with_conflict_check(&self, booking: &Booking, ctx, tx)
    -> Result<BookingCreateResult, ServiceError>;
async fn copy_week_with_conflict_check(&self, from_cw, from_year, to_cw,
    to_year, ctx, tx) -> Result<CopyWeekResult, ServiceError>;
async fn delete_booking(&self, booking_id, ctx, tx) -> Result<(), ServiceError>;
```

### Auth gates

| Method | Privilege |
| --- | --- |
| `SlotService::get_*` | `shiftplanner` **or** `sales` (`slot.rs:83-88, 110-115, 138-143, 167-172`) |
| `SlotService::create_slot` / `update_slot` / `delete_slot` | `shiftplanner` (`slot.rs:211, 269, 292`) |
| `ShiftplanService::create` / `update` / `delete` | `shiftplanner` (`shiftplan_catalog.rs:66-67, 100, 138`) **[To verify]** exact lines |
| `ShiftplanViewService::get_shiftplan_week` | see internally bundled Slot+Booking+Sales reads; effectively `shiftplanner ∨ sales` |
| `ShiftplanViewService::get_shiftplan_*_for_sales_person` | HR **or** `verify_user_is_sales_person(sales_person_id)` (D-Phase3-12) |
| `ShiftplanEditService::modify_slot` / `modify_slot_single_week` / `remove_slot` | `shiftplan.edit` (`shiftplan_edit.rs:66, 163, 223`) + week-lock gate |
| `ShiftplanEditService::book_slot_with_conflict_check` | Shiftplanner ∨ self (D-24-04, `shiftplan_edit.rs:573-589`) |
| `ShiftplanEditService::copy_week_with_conflict_check` | `shiftplan.edit` (bulk operation) |
| `ShiftplanEditService::delete_booking` | delegates to `BookingService::delete` (Shiftplanner ∨ self) + week lock |
| `ShiftplanReportService::extract_*` | **[To verify]** — auth gate lives internally; SpecialDay/Toggle reads run under the passed-in context |

### TX behavior

All methods follow the standard pattern `use_transaction(tx).await?` →
business logic → `commit(tx).await?`. Specifically:

- **`modify_slot`** (`shiftplan_edit.rs:56-151`): opens ONE TX,
  update+create of the Slot + Booking re-point + commit. On error
  anywhere → rollback of the entire chain (critical, see chapter 7).
- **`modify_slot_single_week`** (D-35-04): 3-segment split entirely in
  one TX.
- **`remove_slot`** (`shiftplan_edit.rs:153-208`): sets `valid_to` to
  Sunday of CW-1 (or deletes the Slot if the range disappears entirely)
  and soft-deletes all Bookings from `change_week` on — in one TX.
- **`copy_week_with_conflict_check`** (`shiftplan_edit.rs:788+`): iterates
  Bookings of the source week, calls `book_slot_with_conflict_check`
  internally per Booking — warnings are accumulated, TX spans the whole
  thing.

### Important Fat-Backend point

`ShiftplanEditService` is the example of a Business-Logic
service that orchestrates 12 other services and bundles all
cross-aggregate rules. The FE has to reproduce **none** of it — it calls
`POST /shiftplan-edit/booking` and receives the full warning list back
(Fat Backend / Thin Client, see
[Memory feedback](../../../CLAUDE.md)).

## 5. REST endpoints

Mounts (`rest/src/lib.rs:638-672`):

| Prefix | Module |
| --- | --- |
| `/slot` | `rest::slot` |
| `/shiftplan-catalog` | `rest::shiftplan_catalog` |
| `/shiftplan-edit` | `rest::shiftplan_edit` |
| `/shiftplan-info` | `rest::shiftplan` (view endpoints) |
| `/shiftplan` | PDF export (see `F11-export.md`) |

### `/slot`

| Method | Path | Description | DTO In | DTO Out | Errors |
| --- | --- | --- | --- | --- | --- |
| GET | `/` | All Slots | — | `Vec<SlotTO>` | 401 |
| GET | `/{id}` | One Slot | — | `SlotTO` | 404 |
| GET | `/week/{year}/{month}/{shiftplan_id}` | Slots of a week (path name `month` is historical, actually `week`) | — | `Vec<SlotTO>` | 401 |
| POST | `/` | Create new Slot | `SlotTO` | `SlotTO` | 403, 422 (`OverlappingTimeRange`, `TimeOrderWrong`) |
| PUT | `/{id}` | Update Slot | `SlotTO` | `SlotTO` | 403, 409 (`EntityConflicts`), 422 (`ModificationNotAllowed`) |

### `/shiftplan-catalog`

| Method | Path | Description | DTO |
| --- | --- | --- | --- |
| GET | `/` | All shift plans | `Vec<ShiftplanTO>` |
| GET | `/{id}` | One shift plan | `ShiftplanTO` |
| POST | `/` | Create | `ShiftplanTO` |
| PUT | `/{id}` | Update | `ShiftplanTO` |
| DELETE | `/{id}` | Soft delete | — |

### `/shiftplan-info`

| Method | Path | Description | DTO Out |
| --- | --- | --- | --- |
| GET | `/{shiftplan_id}/{year}/{week}` | Weekly view of a plan | `ShiftplanWeekTO` |
| GET | `/day/{year}/{week}/{day_of_week}` | Daily aggregate across all plans | `ShiftplanDayAggregateTO` |
| GET | `/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}` | Weekly view with `unavailable` markers for 1 person | `ShiftplanWeekTO` |
| GET | `/day/{year}/{week}/{day_of_week}/sales-person/{sales_person_id}` | Daily aggregate with markers | `ShiftplanDayAggregateTO` |

The per-sales-person variants set `ShiftplanDayTO.unavailable` to
`AbsencePeriod` / `ManualUnavailable` / `Both` (D-Phase3-10; see
[`F05-absence-system.md`](./F05-absence-system.md)).

### `/shiftplan-edit`

| Method | Path | Description | DTO In | Errors |
| --- | --- | --- | --- | --- |
| PUT | `/slot/{year}/{week}` | Modify Slot from CW on (Slot split) | `SlotTO` | 403, 409, 423 (Week locked) |
| PUT | `/slot/{year}/{week}/single-week` | Change Slot for exactly 1 CW (D-35) | `SlotTO` | 403, 409, 423 |
| DELETE | `/slot/{slot_id}/{year}/{week}` | Close Slot from CW on | — | 403, 423 |
| PUT | `/vacation` | Legacy: enter Vacation (extra_hours + unavailable) | `VacationPayloadTO` | 403 |
| POST | `/booking` | Create Booking with conflict awareness | `BookingTO` | 403, 409 (paid limit), 422, 423 |
| POST | `/copy-week` | Copy week with conflict check | `CopyWeekRequest` | 403, 423 |

DTOs see `rest-types/src/lib.rs`: `SlotTO` (`:308`), `ShiftplanTO`
(`:15`), `ShiftplanWeekTO` (`:1103`), `ShiftplanDayTO` (`:1092`),
`ShiftplanSlotTO` (`:1070`), `ShiftplanBookingTO` (`:1063`).
`BookingCreateResultTO` / `CopyWeekResultTO` (Phase-3 warning aggregates).

**Important on toggle semantics of `ShiftplanSlotTO`:** the field
`slot.to` stays raw (bidirectional DTO rule P07); the day-effective end
time lives in `effective_to`
(`service/src/shiftplan.rs:52-60`) — only the ShortDay cutoff of the
D-51-07 cutoff-date gate shifts it (Phase 51).

## 6. Frontend integration

- **Pages:**
  `shifty-dioxus/src/page/shiftplan.rs` (2194 lines — the central page
  for the entire shift plan workflow: catalog selection, week
  navigation, Slot rendering, Booking actions, week message, week
  status, PDF export button).
- **Components:**
  - `component/shiftplan_tab_bar.rs` — catalog selection.
  - `component/slot_edit.rs` — Slot editor modal (opens for create,
    modify_slot, modify_slot_single_week and remove_slot).
- **Services / coroutines:**
  `service/slot_edit.rs::SlotEditAction` — bundles Slot changes.
- **Loader:** `loader::load_shift_plan`, `loader::load_shiftplan_catalog`,
  `loader::load_day_aggregate`, `loader::register_user_to_slot_with_conflict_check`,
  `loader::remove_user_from_slot` — thin HTTP wrappers around the backend endpoints.
- **Proxy:** `shifty-dioxus/Dioxus.toml` maps (verified via grep):
  - `/slot` → `http://localhost:3000/slot`
  - `/shiftplan-edit` → `…/shiftplan-edit`
  - `/shiftplan-info` → `…/shiftplan-info`
  - `/shiftplan-catalog` → `…/shiftplan-catalog`
  - `/shiftplan` → `…/shiftplan` (PDF)
  - `/sales-person-shiftplan` → `…/sales-person-shiftplan`

## 7. Edge cases

For the central edge-case reference see
[`../domain/edge-cases.md`](../domain/edge-cases.md), sections
"Atomicity + re-point tests" and "Verify backend roundtrip e2e".

- **Slot split without Booking re-point in the same TX** —
  `modify_slot` (`shiftplan_edit.rs:56-151`) must run the Booking
  migration in the same transaction as the Slot duplication. If this is
  broken, Bookings are either duplicated (on old and new Slot, report
  counts 2x) or orphaned (on the deleted old Slot, report doesn't
  count at all). Regression guard: see
  `service_impl/src/test/shiftplan_edit.rs`.
- **`modify_slot` swallows `max_paid_employees`** — historical bug from
  Phase 23: the update cascade did not carry over the new `max_paid_employees`.
  The fix is visible today in `shiftplan_edit.rs:117-120`
  (line `new_slot.max_paid_employees = slot.max_paid_employees`). See
  also [MemPalace feedback "Verify backend roundtrip
  e2e"](../domain/edge-cases.md#backend-roundtrip-e2e-pruefen). Regression:
  create path ≠ edit path → always click through both manually when
  testing.
- **Weekday rollout** — A new Slot with `day_of_week = Sunday`
  becomes effective from Monday of the `valid_from` week; if the ISO
  week is chosen tightly (e.g. `valid_from = Sunday`), the first day
  is lost because the Slot semantics always need a full CW. In
  practice `create_slot` prefers a Monday date as `valid_from`.
  **[To verify]** whether the backend validates this or whether only
  the report reacts.
- **`update_slot` forbids time-window changes** —
  `service_impl/src/slot.rs:319-329` blocks `from`/`to`/`valid_from`/
  `day_of_week` mutations with `ModificationNotAllowed`. The usual way
  is instead `shiftplan_edit::modify_slot` (with Slot split), running
  through the REST route `PUT /shiftplan-edit/slot/{year}/{week}`.
  The frontend accordingly never uses `PUT /slot/{id}` directly for
  structural changes.
- **Week-lock gate** (Phase 40) blocks ALL write paths in
  `ShiftplanEditService` (including Bookings). `shiftplan.edit` is
  bypass; `shiftplanner` alone is not enough. Without this bypass a
  Shiftplanner would be silently blocked in a locked week (CR-01,
  fixed 2026-07-02).
- **Default Slot set** — the 63 migration Slots from
  `20240619085745_default-slots.sql` are **not necessarily** present in
  production — Nix/CI databases get them; dev DBs that once received
  `sqlx database reset`, too. For new customers the set is the
  business-sensible starting point (store 9-19:30).
- **`shiftplan_id NULL` on legacy Slots** — migration
  `20260330000000` adds `shiftplan_id` as a nullable column and
  immediately backfills to `main`. Creation via
  `SlotService::create_slot` requires `shiftplan_id.is_some()`
  (`slot.rs:220-224`) — practically no more NULL rows should be
  produced.
- **PDF button visibility** — `page/shiftplan.rs::should_show_pdf_button`
  (lines 95-97) requires a chosen shift plan + status `Planned`
  or `Locked` (see [`F11-export.md`](./F11-export.md) for the
  export flow).

## 8. Tests

- **Unit (Slot):** `service_impl/src/test/slot.rs` (1186 lines). Covers:
  `test_get_slots{,_sales_role,_no_permission}` (lines 156-211),
  `test_get_slot{,_sales_role,_not_found,_no_permission}` (212-263),
  `test_create_slot{,_no_permission,_non_zero_id,_non_zero_version,
  _intersects,_time_order,_date_order}` (264-580),
  `test_delete_slot` (580+). `test_overlapping_slots` has the overlap
  algorithm inline (`service_impl/src/slot.rs:55-60`) plus the
  regression suite. `clip_to` business logic: `service/src/slot.rs:176-245`.
- **Unit (view / edit):**
  - `service_impl/src/test/shiftplan.rs` (1659 lines) — weekly/daily
    aggregates including `unavailable` markers.
  - `service_impl/src/test/shiftplan_edit.rs` (1997 lines) — Slot split,
    Booking re-point, copy-week, conflict warnings, paid-limit
    enforcement.
  - `service_impl/src/test/shiftplan_edit_lock.rs` (565 lines) —
    week-lock regression suite (Phase 40).
  - `service_impl/src/test/shiftplan_catalog.rs` (290 lines) — CRUD +
    auth.
  - `service_impl/src/test/shiftplan_report.rs` (612 lines) — raw-row
    aggregation + ShortDay gate (Phase 51 Chain D).
- **Integration:** `shifty_bin/src/integration_test/booking_absence_conflict.rs`
  runs `book_slot_with_conflict_check` and
  `get_shiftplan_week_for_sales_person` end-to-end against in-memory
  SQLite.
- **Known gaps:**
  - `min_resources` under-staffing is currently not hard-validated
    by the backend (only FE hint). **[To verify]**
  - Explicit tests that `modify_slot` performs a rollback on failure
    are only implicitly covered by the transaction boundary.

## 9. History & context

- **Slots since Milestone 0** (2024-05-02). The model was extended
  iteratively: `min_resources` (Aug 2024), `shiftplan_id` +
  multi-plan catalog (March 2026), `max_paid_employees` (May 2026).
- **Phase 23 (v1.2-ish)** — fix: `modify_slot` did not retain new
  `max_paid_employees`. Trigger: frontend edit saw no effect,
  because the edit path ≠ create path. Since then MemPalace feedback
  "Verify backend roundtrip e2e".
- **Phase 35 (D-35)** — `modify_slot_single_week` introduced as a
  3-segment split (one-off exception for exactly one CW), so
  Shiftplanners no longer need to manually create the Slot version +
  rollback Slot.
- **Phase 40 (D-40)** — week lock / week lock. All write paths
  in `ShiftplanEditService` gate against `WeekStatusService`;
  `shiftplan.edit` is bypass. CR-01 (2026-07-02): the plain
  Booking path must also gate, `shiftplanner` alone is not a bypass.
- **Phase 3 / Phase 3 conflict warnings** — extension of the
  Booking paths with `BookingCreateResult` including `warnings: Arc<[Warning]>`
  (cross-source: AbsencePeriod, manual unavailable, without dedup per day).
  `ShiftplanViewService::get_*_for_sales_person` variants set
  `unavailable` markers (D-Phase3-10/-12).
- **Phase 51 Chain D** — `ShiftplanReportService` reads raw rows
  (`ShiftplanReportRawRow`) instead of SQL aggregate; `Slot::clip_to`
  business logic + `shortday_gate` aggregate in Rust (D-51-08).
  Prerequisite for the D-51-07 cutoff-date toggle. The old
  `Shiftplan{Report,QuickOverview}Entity` were dropped
  (`dao/src/shiftplan_report.rs:9-12`).
- **Toggle context:** `paid_limit_hard_enforcement`
  (`shiftplan_edit.rs:618-660`) and `shortday_active_from`
  (`shiftplan_report.rs:83-95`) are the active rollout toggles in this
  cluster.
- **References to planning artifacts:** see `.planning/phases/23-*`,
  `.planning/phases/35-*`, `.planning/phases/40-*`, `.planning/phases/51-*`
  for the full context reads.

---

*Last verified against code:* see git blame of this file.
