# Feature: Extra Hours — legacy time recording & custom categories

> **Short form:** Single-day time rows for overtime, Vacation, sickness,
> holiday, unavailability, unpaid leave, volunteer work, and arbitrary
> business-defined additional categories — the original time-recording
> aggregate that, for absence categories (Vacation/SickLeave/UnpaidLeave)
> starting with v1.0, **coexists** with the new range-based Absence system
> (F05).

**Cluster ID:** F04
**Status:** production (legacy for Absence categories, still authoritative for
overtime / volunteer / custom)
**First introduced:** initial HR iteration (migration `20240618125847_paid-sales-persons.sql`)
**Responsible crates:**
- `service::extra_hours`, `service::custom_extra_hours`
- `service_impl::extra_hours`, `service_impl::custom_extra_hours`
- `dao::extra_hours`, `dao::custom_extra_hours` (plus the concrete
  SQLite implementation in `dao_impl_sqlite`)
- `rest::extra_hours`, `rest::custom_extra_hours`
- `rest_types::{ExtraHoursTO, ExtraHoursCategoryTO, CustomExtraHoursTO,
  ConvertExtraHoursRequestTO}`
- Frontend: `shifty-dioxus/src/page/custom_extra_hours_management.rs`

---

## 1. What is it? (Business context)

Extra Hours are **single-day time rows** with which HR (or an employee for
themselves) records times outside the regularly planned shift plan:

- Overtime (`ExtraWork`)
- Vacation (`Vacation`) — legacy: individual days; as of v1.0 usually as
  Absence range (F05)
- Sickness (`SickLeave`) — same
- Holiday (`Holiday`)
- Unavailability (`Unavailable`) — pure availability marker, without
  hour impact on the balance
- Unpaid leave (`UnpaidLeave`) — same legacy → Absence range;
  lowers expectation
- Volunteer work (`VolunteerWork`) — documented, but not balance-affecting
- Custom categories (`CustomExtraHours(id)`) — defined by the business itself

**Custom Extra Hours** are a second, orthogonal catalog: HR can define
arbitrarily many custom booking categories (e.g. "Training", "Works Council",
"Emergency on-call"), assign them to a set of Sales Persons, and decide per
category whether it influences the hours balance (`modifies_balance`).
The actual time rows still live in `extra_hours` but reference via
`custom_extra_hours_id` a row from `custom_extra_hours`.

**Example workflow from a user's perspective:**

1. HR opens the employee detail page and enters "05.05.2025 — 4h overtime"
   (`ExtraWork`) — lands directly in the report as positive balance.
2. An employee books "12.05.2025 — 8h Vacation" (`Vacation`) themselves —
   legacy path; as of v1.0 the UI creates an `AbsencePeriod` instead.
3. HR creates a category "Training" in "Custom Extra Hours Management" (see
   frontend), `modifies_balance=true`, assigned to all sales staff.
   From now on "Training" is available as a category when creating an Extra
   Hours row and counts as work hours.

## 2. Business rules

- **Category enum** is pinned in `service::extra_hours::ExtraHoursCategory`
  (`service/src/extra_hours.rs:41-50`) and in the DAO layer as
  `ExtraHoursCategoryEntity` (`dao/src/extra_hours.rs:9-18`). New fixed enum
  values are a breaking change; for new categories the Custom path is
  intended.
- **`as_report_type()`** classifies each category into a ReportType
  (`service/src/extra_hours.rs:51-73`):
  - `ExtraWork` → `WorkingHours` (counts as worked)
  - `Vacation`, `SickLeave`, `Holiday`, `UnpaidLeave` → `AbsenceHours`
    (count as absent, lower expected hours according to category
    semantics)
  - `Unavailable` → `None` (no balance effect, only availability)
  - `VolunteerWork` → `Documented` (neither balance nor expectation)
  - `CustomExtraHours(…)` → `WorkingHours` if `modifies_balance=true`,
    otherwise `None` (falls back to `None` on unloaded `LazyLoad`)
- **`availability()`** decides whether this row marks the Sales Person as
  available or blocked for the day (`service/src/extra_hours.rs:75-96`).
- **`UnpaidLeave` (unpaid leave) — special role:**
  - `as_report_type() == AbsenceHours` and `availability() == Unavailable`
    (verified via tests in `service/src/extra_hours.rs:255-268`).
  - **Lowers expectation**: the reporting path filters `UnpaidLeave`
    explicitly out (`service_impl/src/reporting.rs:562`, `:974`), so that
    the expected hours for this time are removed from the weekly
    expectation — an UnpaidLeave day is thus neither worked nor owed, the
    balance stays neutral.
- **Custom-category effect depends on `modifies_balance`:** only if the
  definition is loaded and `modifies_balance=true`, the row counts into the
  balance — otherwise it is ignored (even for availability). The
  lazy-load semantics are thus "safe by default": unloaded custom
  categories have no effect.
- **Author and self-service:** HR (`HR_PRIVILEGE`) may write for any Sales
  Person; a Sales Person account may write for themselves. This is an
  OR gate (`service_impl/src/extra_hours.rs:118-127` and
  `:187-196`, `:248-257`, `:322-345`).
- **Update = soft-delete + insert:** an update creates a new physical row
  with the same `logical_id` and marks the old one as deleted
  (`service_impl/src/extra_hours.rs:273-309`). The stable ID exposed
  externally is `logical_id`; the physical `id` changes per version. The
  migration `20260428101456_add-logical-id-to-extra-hours.sql` enforces via
  partial index exactly one active row per `logical_id`.
- **Version conflict:** update compares `request.version` against the
  currently active row; on mismatch → `ServiceError::EntityConflicts`
  (`service_impl/src/extra_hours.rs:265-271`).
- **`sales_person_id` is immutable:** an update that changes the Sales
  Person is rejected (`service_impl/src/extra_hours.rs:259-263` →
  `ValidationFailureItem::ModificationNotAllowed`).
- **Delete = soft delete:** sets `deleted = NOW()` on the active row
  (`service_impl/src/extra_hours.rs:315-359`).
- **Bulk soft delete for Cutover:** `soft_delete_bulk` is a special mass
  path used exclusively by the Cutover process (F05 / Phase 4). It
  requires `CUTOVER_ADMIN_PRIVILEGE`, checks **before** any DAO work
  and inherits the transaction from the caller (no commit here —
  `service_impl/src/extra_hours.rs:372-399`). Idempotent at the DAO layer:
  rows already deleted are skipped (`dao/src/extra_hours.rs:87-99`).
- **Cutover convergence with Absence (F05):** After Cutover, for new
  Absence categories (`Vacation`, `SickLeave`, `UnpaidLeave`) primarily
  `AbsencePeriod` rows are written; old `extra_hours` rows of these
  categories remain readable. The earlier write block in `create()` was
  deliberately removed in Phase 8.4
  (`service_impl/src/extra_hours.rs:198-204`), so that coexistence
  model M-01 is possible.
- **Conversion path:** REST endpoint `POST /extra-hours/{id}/convert-to-absence`
  (see §5) delegates to the `AbsenceConversionService` — this marks the
  Extra Hours row internally as deleted and creates an `AbsencePeriod` row.
- **Custom Extra Hours constraints:**
  - `HR_PRIVILEGE` for all CUD operations and `get_all`/`get_by_id`
    (`service_impl/src/custom_extra_hours.rs:38-79`, `:113-216`).
  - `get_by_sales_person_id` allows HR **or** the affected Sales Person
    (`service_impl/src/custom_extra_hours.rs:81-111`).
  - Create requires empty `id`, `version`, no `created`, no `deleted`;
    delete = soft delete via `deleted = NOW()`; update checks version conflict.
  - Assignment (mapping to Sales Persons) runs via
    `custom_extra_hours_sales_person`; the array `assigned_sales_person_ids`
    is part of the aggregate (see data model). **[To verify]** how
    assignments are specifically mapped to the link table on update — the
    trait signature treats them as `Arc<[Uuid]>`, the persistence lives in
    the SQLite DAO.

## 3. Data model

### Tables

| Table | Purpose | Important columns |
| --- | --- | --- |
| `extra_hours` | Single-day time row per Sales Person | `id` (physical), `logical_id`, `sales_person_id`, `amount`, `category`, `custom_extra_hours_id`, `description`, `date_time`, `created`, `deleted`, `update_process`, `update_timestamp`, `update_version` |
| `custom_extra_hours` | Catalog of business-defined additional categories | `id`, `name`, `description`, `modifies_balance`, `created`, `deleted`, `update_version`, `update_process` |
| `custom_extra_hours_sales_person` | N:M mapping custom category ↔ Sales Person | `sales_person_id`, `custom_extra_hours_id` (compound PK), `created`, `deleted`, `update_process` |

### Migrations

Chronological, as the table has grown historically:

- **`20240618125847_paid-sales-persons.sql`** — creates the base table
  `extra_hours` (with `id`, `sales_person_id`, `amount`, `category`,
  `description`, `date_time`, `created`, `deleted`, `update_*`), FK on
  `sales_person`.
- **`20250413073750_add-custom-extra-hours-table.sql`** — introduces custom
  categories: `custom_extra_hours` + linking table
  `custom_extra_hours_sales_person`.
- **`20250418200122_insert-custom-column-to-extra-hours.sql`** — adds
  `extra_hours.custom_extra_hours_id BLOB NOT NULL DEFAULT X'00…00'`.
  The nil-UUID default marks "no custom category" and is the reason
  why the DAO representation `Custom(Uuid)` is serialized as an explicit
  enum value — not as `Option`.
- **`20260428101456_add-logical-id-to-extra-hours.sql`** — introduces
  `logical_id`: nullable-add, backfill (`logical_id = id`), CREATE-new
  with NOT-NULL rebuild, partial unique index
  `idx_extra_hours_logical_id_active ON extra_hours(logical_id) WHERE
  deleted IS NULL`. From here on update follows the "soft-delete + insert-new"
  pattern; the stable API ID is `logical_id`, no longer the physical
  row ID.

Additionally relevant for the **Cutover interaction with F05**:

- **`20260502170000_create-absence-period.sql`** — introduces `absence_period`
  (strictly additive, `extra_hours` untouched).
- **`20260503000000_create-absence-migration-quarantine.sql`** — quarantine
  table for legacy `extra_hours` rows that could not be migrated
  unambiguously (FK on `extra_hours.id`).
- **`20260503000001_create-absence-period-migration-source.sql`** — mapping
  table: `extra_hours_id → absence_period_id`, idempotency key is the
  Extra Hours physical ID.

### Relationships

```
sales_person 1─┬─* extra_hours ──(0..1)── custom_extra_hours  (via custom_extra_hours_id)
               │
               *
               │
       custom_extra_hours_sales_person  (N:M between sales_person & custom_extra_hours)

absence_period_migration_source: (extra_hours.id) ─→ absence_period.id     [Cutover mapping]
absence_migration_quarantine:    (extra_hours.id) ─→ quarantine             [Cutover failed rows]
```

## 4. Service API

### Traits

`service::extra_hours::ExtraHoursService` (`service/src/extra_hours.rs:187-248`):

```rust
#[async_trait]
pub trait ExtraHoursService {
    type Context;
    type Transaction: dao::Transaction;

    async fn find_by_sales_person_id_and_year(&self, sp: Uuid, year: u32, until_week: u8, ctx, tx) -> Arc<[ExtraHours]>;
    async fn find_by_sales_person_id_and_year_range(&self, sp: Uuid, from: ShiftyDate, to: ShiftyDate, ctx, tx) -> Arc<[ExtraHours]>;
    async fn find_by_week(&self, year: u32, week: u8, ctx, tx) -> Arc<[ExtraHours]>;
    async fn create(&self, entity: &ExtraHours, ctx, tx) -> ExtraHours;
    async fn update(&self, entity: &ExtraHours, ctx, tx) -> ExtraHours;
    async fn delete(&self, id: Uuid, ctx, tx) -> ();
    async fn soft_delete_bulk(&self, ids: Arc<[Uuid]>, update_process: &str, ctx, tx) -> ();
}
```

`service::custom_extra_hours::CustomExtraHoursService` (`service/src/custom_extra_hours.rs:60-105`):
`get_all`, `get_by_id`, `get_by_sales_person_id`, `create`, `update`, `delete`.

### Auth gates

| Method | Gate |
| --- | --- |
| `ExtraHoursService::find_by_sales_person_id_and_year(_range)` | HR **or** self (`service_impl/src/extra_hours.rs:118-127`) |
| `ExtraHoursService::find_by_week` | `check_only_full_authentication` — pure internal path (Reporting/Scheduler) (`service_impl/src/extra_hours.rs:162-164`) |
| `ExtraHoursService::create` | HR **or** self for the target Sales Person |
| `ExtraHoursService::update` | HR **or** self for the affected row |
| `ExtraHoursService::delete` | HR **or** self (double-checked, once via `SALES_PRIVILEGE`, once via `verify_user_is_sales_person`) |
| `ExtraHoursService::soft_delete_bulk` | `CUTOVER_ADMIN_PRIVILEGE` **before** any DAO work; only the Cutover commit path calls this |
| `CustomExtraHoursService::get_all` / `get_by_id` / `create` / `update` / `delete` | HR |
| `CustomExtraHoursService::get_by_sales_person_id` | HR **or** self |

### TX behavior

- All methods accept `Option<Self::Transaction>` and, on `None`, pull
  their own via `use_transaction`.
- `create`, `update`, `delete` commit themselves.
- `soft_delete_bulk` does **not** commit — the Cutover chain holds the
  transaction and commits only at the end
  (`service_impl/src/extra_hours.rs:395-398`).
- `update` runs atomically: soft delete of the old row + insert of the new row
  in the same transaction.

### Dependencies

`ExtraHoursServiceImpl` — Business-Logic-Tier (consumes another
domain service):

- DAOs: `ExtraHoursDao`, `TransactionDao`
- Basic services: `PermissionService`, `SalesPersonService`
- Business-Logic service (lazy-load resolution): `CustomExtraHoursService`
  — called internally for loading the custom definition with
  `Authentication::Full` (`service_impl/src/extra_hours.rs:51-54`);
  this Full-context bypass is explicitly documented and intended for
  internal aggregate consumers of the toggle and custom-category reads
  (cf. Memory "ToggleService Full-Context-Bypass").
- Infrastructure: `ClockService`, `UuidService`.

`CustomExtraHoursServiceImpl` — Basic-Tier (only DAO + Permission +
Transaction + `SalesPersonService` for the self check; no further
domain service):

- DAO: `CustomExtraHoursDao`
- Basic services: `PermissionService`, `SalesPersonService`
- Infrastructure: `ClockService`, `UuidService`, `TransactionDao`

## 5. REST endpoints

### Extra Hours

Router: `rest/src/extra_hours.rs:22-35`, mounted under `/extra-hours`
(`rest/src/lib.rs:667`).

| Method | Path | Description | DTO In | DTO Out | Important errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/extra-hours/by-sales-person/{id}?year=…&until_week=…` | All rows of a Sales Person up to CW `until_week` in year `year` | — | `Vec<ExtraHoursTO>` | 401, 404 |
| `POST` | `/extra-hours` | Create new row | `ExtraHoursTO` | `ExtraHoursTO` (status 201) | 400 (validation), 403 |
| `PUT` | `/extra-hours/{id}` | Update row (logical ID in path); versioned | `ExtraHoursTO` | `ExtraHoursTO` | 400, 403, 404, 409 |
| `DELETE` | `/extra-hours/{id}` | Soft delete (logical ID) | — | 204 | 404 |
| `POST` | `/extra-hours/{id}/convert-to-absence` | Convert legacy row into `AbsencePeriod` | `ConvertExtraHoursRequestTO` | `AbsencePeriodTO` | 403 (HR), 404 (soft-deleted/unknown), 422 (`DateOrderWrong`, `OverlappingPeriod`) |

DTOs see `rest_types` (`rest-types/src/lib.rs:797-870, 1859-…`).

### Custom Extra Hours

Router: `rest/src/custom_extra_hours.rs:17-28`, mounted under
`/custom-extra-hours` (`rest/src/lib.rs:644`).

| Method | Path | Description | DTO In | DTO Out |
| --- | --- | --- | --- | --- |
| `GET` | `/custom-extra-hours` | All custom categories | — | `Vec<CustomExtraHoursTO>` |
| `GET` | `/custom-extra-hours/{id}` | Single category | — | `CustomExtraHoursTO` |
| `GET` | `/custom-extra-hours/by-sales-person/{sales_person_id}` | Only those assigned to the Sales Person | — | `Vec<CustomExtraHoursTO>` |
| `POST` | `/custom-extra-hours` | Create | `CustomExtraHoursTO` | `CustomExtraHoursTO` (201) |
| `PUT` | `/custom-extra-hours/{id}` | Update | `CustomExtraHoursTO` | `CustomExtraHoursTO` |
| `DELETE` | `/custom-extra-hours/{id}` | Soft delete | — | 204 |

**Note doc drift:** the utoipa annotation for DELETE names `/custom-extra-hours/{id}`
(`rest/src/custom_extra_hours.rs:212`), which together with the router mount adds up.
The handler uses `Path<Uuid>`; the effective path is unchanged
`DELETE /custom-extra-hours/{id}`.

## 6. Frontend integration

- **Pages:** `shifty-dioxus/src/page/custom_extra_hours_management.rs` — HR page
  for managing custom categories (create/edit/delete). Extra Hours
  individual rows are currently not managed on a dedicated page but
  from the employee details pages.
- **API client:** `shifty-dioxus/src/api.rs` — `get_custom_extra_hours_by_sales_person`,
  `post_custom_extra_hours`, `put_custom_extra_hours`, `delete_custom_extra_hours`
  (call sites in `custom_extra_hours_management.rs:73-140`).
- **State objects:** `shifty-dioxus/src/state/employee.rs` —
  `CustomExtraHoursDefinition` as frontend view model (signature:
  `custom_extra_hours_management.rs:12`).
- **i18n keys** (`custom_extra_hours_management.rs:49-58`):
  `CustomExtraHoursManagement`, `Name`, `Description`, `ModifiesBalance`,
  `Actions`, `AddNew`, `Save`, `Cancel`, `Edit`, `Delete`.
- **Proxy** (`shifty-dioxus/Dioxus.toml:57-58, 71-72`):
  - `/custom-extra-hours` → `http://localhost:3000/custom-extra-hours`
  - `/extra-hours` → `http://localhost:3000/extra-hours`
- **Known frontend gaps:**
  - Assignment "custom category ↔ Sales Person" is currently hard-coded in the
    UI with `assigned_sales_person_ids: vec![]`
    (`custom_extra_hours_management.rs:190,197`); the comment there
    explicitly documents this as an open feature.
  - `Load` is re-triggered after each mutating action
    (`custom_extra_hours_management.rs:205, 330`), to synchronize the
    state.

## 7. Edge cases

Central edge-case reference: [`../domain/edge-cases.md#2-absence--extra-hours`](../domain/edge-cases.md#2-absence--extra-hours)
and [Section 8 "Soft-Delete-Konsistenz"](../domain/edge-cases.md#8-soft-delete-konsistenz).

Feature-specific:

- **Cutover split Vacation/SickLeave/UnpaidLeave (F04 × F05):** After Cutover
  **both** data sources can exist for the same person and the same time
  range — old rows in `extra_hours` (not deleted, but either converted or
  left as-is) and new ones in `absence_period`. Every
  report/balance path must aggregate **both** sources, otherwise the
  balance tips. This is the most prominent edge case of the cluster —
  see `../domain/edge-cases.md#21-cutover-historie`.
- **`UnpaidLeave` lowers expectation:** reporting must filter `UnpaidLeave`
  rows explicitly and reduce the weekly expectation accordingly
  (`service_impl/src/reporting.rs:562`, `:974`). Anyone forgetting the
  category in a new aggregate computes too much expectation — i.e., the
  Sales Person appears with a deficit in the balance that does not
  actually exist.
- **Custom category unloaded → no balance effect:** if
  `load_custom_extra_hours_definitions` fails (definition deleted /
  not found), `LazyLoad.get()` falls back to `None`, and both semantic
  functions (`as_report_type`, `availability`) return `None`. The row is
  effectively invisible for the balance. The log-warn path marks this as an
  integrity issue (`service_impl/src/extra_hours.rs:60-72`), but does not
  stop the query.
- **`Unavailable` rows do not need an amount to take effect:** they are pure
  availability markers; their `amount` does not influence the balance.
  Still, `amount` is written and passed through in reports with `Documented`
  semantics.
- **Snapshot drift on delete/update:** if an `extra_hours` row is
  deleted whose contribution is already included in a persisted `billing_period`
  snapshot, the live view drifts against the snapshot. Without a
  version bump of `CURRENT_SNAPSHOT_SCHEMA_VERSION`, the diff cannot be
  identified as a real delete. See `../domain/edge-cases.md#23-legacy-extra-hours--delete-semantik`.
- **`logical_id` reuse forbidden:** the partial unique index
  `idx_extra_hours_logical_id_active` enforces "one active row per
  logical_id". Setting in a test on a new insert a `logical_id` of a
  soft-deleted row does not collide — on an active one **it does**.
- **`convert-to-absence` requires valid range:** the REST endpoint maps
  `DateOrderWrong` / `OverlappingPeriod` to 422; the actual
  conversion semantics live in `AbsenceConversionService` (F05).

## 8. Tests

- **Unit / service tests:**
  - `service_impl/src/test/extra_hours.rs` (748 lines) covers the
    update semantics "soft-delete + insert", the OR permission flow (HR vs.
    self), version conflict, the reject on changed `sales_person_id`,
    NotFound on unknown/deleted row, and the Phase 4 bulk-delete
    path (happy path + elevation-of-privilege guard, which explicitly
    pins `MockExtraHoursDao::expect_soft_delete_bulk().times(0)` before
    the permission gate rejects).
  - `service_impl/src/test/custom_extra_hours.rs` (620 lines) covers CRUD +
    the Sales Person assignment filters for `get_by_sales_person_id`.
  - `service::extra_hours` internal tests
    (`service/src/extra_hours.rs:254-268`) pin the `UnpaidLeave`
    classification.
  - DAO trait default tests
    (`dao/src/custom_extra_hours.rs:172-221`) prove `find_all` /
    `find_by_id` / `find_by_sales_person_id` including soft-delete filter.
- **Integration:** in-memory SQLite runs of the DAO impl live in
  `dao_impl_sqlite/src/…` (wired via the usual
  `sqlx::sqlite::SqlitePool::connect(":memory:")` harnesses).
- **Known gaps:**
  - **[To verify]** whether there is a dedicated test measuring the
    reporting aggregation across `extra_hours` **and** `absence_period` on
    a Cutover-spanning period; that would be the
    high-value regression guard from edge case §7.
  - **[To verify]** whether the `convert-to-absence` endpoint has an
    end-to-end roundtrip test that reaches into the `AbsenceConversionService`
    impl (not only the mock layer).

## 9. History & context

- **Initial (2024-06):** migration `20240618125847_paid-sales-persons.sql`
  creates `extra_hours` and `working_hours` together — the original
  time-recording building block for HR & Reporting.
- **2025-04 — Custom Extra Hours introduced:**
  - `20250413073750_add-custom-extra-hours-table.sql` (catalog +
    N:M mapping),
  - `20250418200122_insert-custom-column-to-extra-hours.sql` (foreign-key
    column in `extra_hours`).
  - Motivation: business-defined categories without enum extension.
- **v1.0 / Cutover (2026-05, Phase 4):** range-based Absence aggregate
  (`absence_period`) takes over the categories Vacation/SickLeave/UnpaidLeave
  for the new case. Existing `extra_hours` rows of these categories are
  either migrated (`absence_period_migration_source`), quarantined
  (`absence_migration_quarantine`), or — in the coexistence model — left
  as historical rows. `soft_delete_bulk` is the mass path with
  which the Cutover commit finally hides mapped legacy rows from the live
  read.
- **v1.3 / Phase 8.4:** the write block for the deprecated categories
  was removed (`service_impl/src/extra_hours.rs:198-204`) — coexistence
  M-01 is the definitive model decision, not "Absence replaces
  Extra Hours". New rows of these categories can be created without
  feature gate again, e.g. for corrections of historical data.
- **Phase 51 / Toggle bypass:** the internal use of
  `Authentication::Full` when loading the custom definition is
  consistent with the bypass documented in Phase 51 for
  internal aggregate consumers (Memory "ToggleService Full-Context-Bypass").
- **References to `.planning/phases/…`** for the Cutover context:
  `.planning/phases/04-*` (migration & Cutover) and `.planning/phases/08-*`
  (coexistence retuning). [To verify] concrete phase IDs in the current
  milestone cleanup state.

---

**Conclusion:** `extra_hours` is the permanently authoritative data source for
overtime, volunteer and custom categories; for Vacation/SickLeave/UnpaidLeave
it is legacy-coexistent with `absence_period` (F05) — every report must
read both sources, otherwise the balance tips.

*Last verified against code:* see git blame of this file.
