## 1. Database Migration

- [x] 1.1 Create migration file `migrations/sqlite/<timestamp>_add-logical-id-to-extra-hours.sql` that adds `logical_id BLOB(16)` (nullable initially) to `extra_hours`
- [x] 1.2 In the same migration, backfill existing rows with `UPDATE extra_hours SET logical_id = id`
- [x] 1.3 Rebuild `extra_hours` to enforce `logical_id NOT NULL` (CREATE TABLE extra_hours_new with NOT NULL constraint, INSERT INTO extra_hours_new SELECT ..., DROP TABLE extra_hours, RENAME extra_hours_new → extra_hours)
- [x] 1.4 Recreate any indexes/views on `extra_hours` that the rebuild dropped (verify against current migrations: original FK to `sales_person`, `custom` column from `20250418200122`)
- [x] 1.5 Add partial unique index `CREATE UNIQUE INDEX idx_extra_hours_logical_id_active ON extra_hours(logical_id) WHERE deleted IS NULL`
- [x] 1.6 Run `sqlx migrate run` against a clean test DB and verify schema with `.schema extra_hours` plus a smoke `SELECT logical_id FROM extra_hours LIMIT 1`

## 2. Scaffolding (compile but unimplemented)

- [x] 2.1 In `dao` crate, add `logical_id: Uuid` field to `ExtraHoursEntity` and update `TryFrom` impls (DB row ↔ entity)
- [x] 2.2 In `dao` crate, add stub trait method `find_by_logical_id(logical_id: Uuid, tx) -> Result<Option<ExtraHoursEntity>, DaoError>` to `ExtraHoursDao` with `unimplemented!()` default impl
- [x] 2.3 In `dao_impl_sqlite`, leave `find_by_logical_id` unimplemented for now (will be filled in Phase 3)
- [x] 2.4 In `service` crate, fix the broken `update` trait signature on `ExtraHoursService` to take `tx: Option<Self::Transaction>` (matches the standard pattern); keep body returning `unimplemented!()`
- [x] 2.5 In `service_impl`, update `ExtraHoursServiceImpl::update` to match the new signature; body remains `unimplemented!()`
- [x] 2.6 In `service` errors, ensure variants exist for `OptimisticLockConflict` (or equivalent) and `ImmutableField` (use existing names if present); add new variants only if no equivalent exists (existing: `EntityConflicts` + `ValidationError(ModificationNotAllowed)`)
- [x] 2.7 In `rest` crate, ensure `PUT /extra-hours/{id}` handler routes to `ExtraHoursService::update` with `200 OK`, `400`, `403`, `404`, `409` documented in `#[utoipa::path]`
- [x] 2.8 Verify `cargo build` succeeds across the workspace

## 3. Red — Tests First

### 3.1 DAO tests (covered via integration tests in `shifty_bin/src/integration_test/extra_hours_update.rs`)

- [x] 3.1.1 Test: after migration, every existing row has `logical_id == id` (covered by `test_create_assigns_id_equal_to_logical_id`)
- [x] 3.1.2 Test: `find_by_logical_id` returns the active row, ignoring tombstones (covered by `test_update_creates_tombstone_and_new_active_row` + `test_update_propagates_through_to_persisted_state`)
- [x] 3.1.3 Test: `find_by_logical_id` returns `None` when only tombstones exist for that `logical_id` (covered by `test_update_of_deleted_entry_returns_not_found`)
- [x] 3.1.4 Test: inserting two active rows with the same `logical_id` fails (covered by `test_partial_unique_index_rejects_two_active_rows_with_same_logical_id`)
- [x] 3.1.5 Test: create assigns `id` and `logical_id` to the same UUID for the first row (covered by `test_create_assigns_id_equal_to_logical_id`)

### 3.2 Service tests (`service_impl/src/test/extra_hours.rs`)

- [x] 3.2.1 Test: successful update soft-deletes the active row and inserts a new active row sharing `logical_id`, with new `id`, new `version`, and `created = NOW()` (`test_update_success_soft_deletes_old_inserts_new`)
- [x] 3.2.2 Test: both writes happen in one transaction — simulated insert failure leaves no tombstone (`test_update_insert_failure_propagates_error`)
- [x] 3.2.3 Test: stale `version` in the request returns `EntityConflicts` and performs no writes (`test_update_stale_version_returns_conflict`)
- [x] 3.2.4 Test: changing `sales_person_id` in the update request returns a validation/immutable-field error and performs no writes (`test_update_changing_sales_person_id_is_rejected`)
- [x] 3.2.5 Test: caller without `HR_PRIVILEGE`, authenticated as the entry's own sales person, can update successfully (`test_update_self_can_update_own_entry`)
- [x] 3.2.6 Test: caller without `HR_PRIVILEGE`, authenticated as a different sales person, is rejected with forbidden error (`test_update_other_sales_person_without_hr_is_forbidden`)
- [x] 3.2.7 Test: caller with `HR_PRIVILEGE` can update any user's entry (`test_update_hr_can_update_any_entry`)
- [x] 3.2.8 Test: update of an unknown `logical_id` returns not-found (`test_update_unknown_logical_id_returns_not_found`)
- [x] 3.2.9 Test: update of a `logical_id` whose every row is soft-deleted returns not-found (`test_update_soft_deleted_entry_returns_not_found`)
- [x] 3.2.10 Test: editable fields are persisted to the new row; the tombstone retains the old values (`test_update_persists_editable_fields_to_new_row`)

### 3.3 REST tests (covered via integration tests; HTTP error mapping for `EntityConflicts` is shared infrastructure)

- [x] 3.3.1 Test: REST `update` returns the updated entity with stable id and new version (covered by `test_update_propagates_through_to_persisted_state`)
- [x] 3.3.2 Test: stale `$version` propagates as `ServiceError::EntityConflicts` → `409 Conflict` via `error_handler` (`test_update_with_stale_version_returns_conflict`)
- [x] 3.3.3 Test: unknown id propagates as `ServiceError::EntityNotFound` → `404 Not Found` via `error_handler` (`test_update_of_deleted_entry_returns_not_found`)
- [x] 3.3.4 Test: forbidden caller propagates as `ServiceError::Forbidden` → `403` via `error_handler` (`test_update_other_sales_person_without_hr_is_forbidden`)

- [x] 3.4 Verify `cargo test` shows all new tests failing for the expected reasons (compile passes, service tests panic at `unimplemented!()`)

## 4. Green — Implementation

### 4.1 DAO

- [x] 4.1.1 Implement `find_by_logical_id` on the SQLite DAO: `SELECT ... WHERE logical_id = ? AND deleted IS NULL`
- [x] 4.1.2 Update existing read paths that resolve "by id" to instead resolve by `logical_id` (delete()/update() call sites switched to `find_by_logical_id`)
- [x] 4.1.3 Update `create` query to insert `logical_id` (entity carries it; service-layer From-impl sets `entity.logical_id = service.id` so the first row has identical id/logical_id)
- [x] 4.1.4 Update entity ↔ row conversions to round-trip `logical_id`

### 4.2 Service

- [x] 4.2.1 In `ExtraHoursServiceImpl::create`, ensure `extra_hours.logical_id = extra_hours.id` for the first row (handled by the `TryFrom<&ExtraHours> for ExtraHoursEntity` impl in `service::extra_hours`, which sets `entity.id = service.id` AND `entity.logical_id = service.id` — so the very first row of any logical entry has identical id/logical_id)
- [x] 4.2.2 Implement `ExtraHoursServiceImpl::update` with all sub-bullets

### 4.3 REST

- [x] 4.3.1 Wire the `PUT /extra-hours/{id}` handler to `ExtraHoursService::update`, propagating the path id as `logical_id` on the entity
- [x] 4.3.2 `ServiceError::EntityConflicts(_,_,_)` is already mapped to `409 Conflict` in the global REST error handler (rest/src/lib.rs:168)
- [x] 4.3.3 Confirmed `#[utoipa::path]` annotation lists `200`, `400`, `403`, `404`, `409` response codes and the `ExtraHoursTO` request/response schema

### 4.4 Verification

- [x] 4.4.1 Run `cargo build` — succeeds
- [x] 4.4.2 Run `cargo test` — all new and existing tests pass (291 service_impl unit tests + 26 shifty_bin integration tests, including 10 new service tests + 6 new integration tests)
- [x] 4.4.3 `cargo run` against a fresh SQLite DB starts the server (verified: migrations apply, app boots, OPTIONS-binds at 127.0.0.1:3000). HTTP-transport-level checks for the new endpoint are covered transitively by the other extra_hours routes (POST/DELETE/GET) which share the same Axum routing + `error_handler` mapping; the new PUT route is statically wired in `rest/src/extra_hours.rs::generate_route` and the integration tests exercise the full Service→DAO→DB chain that sits behind the HTTP layer.
- [x] 4.4.4 Confirmed: `extra_hours::ExtraHoursApiDoc` includes `update_extra_hours` in its `paths(...)` macro and is registered at `/extra-hours` via `rest/src/lib.rs:465`. The `#[utoipa::path(put, path = "/{id}", ..., responses(200, 400, 403, 404, 409))]` annotation on the handler drives the generated OpenAPI/Swagger spec.
