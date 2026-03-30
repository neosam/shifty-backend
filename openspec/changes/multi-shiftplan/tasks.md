## 1. Scaffolding: Database Migration

- [x] 1.1 Create SQLite migration: `shiftplan` table (id, name, is_planning, deleted, update_timestamp, update_process, update_version)
- [x] 1.2 In the same migration: insert default "main" plan with `is_planning = 0`
- [x] 1.3 In the same migration: `ALTER TABLE slot ADD COLUMN shiftplan_id blob(16) REFERENCES shiftplan(id)`
- [x] 1.4 In the same migration: `UPDATE slot SET shiftplan_id = <main-plan-uuid>`
- [x] 1.5 In the same migration: drop and recreate `bookings_view` to include shiftplan name

## 2. Scaffolding: Rename ShiftplanService to ShiftplanViewService

- [x] 2.1 Rename `service/src/shiftplan.rs` trait `ShiftplanService` → `ShiftplanViewService` and update all references across all crates (service, service_impl, rest, shifty_bin)
- [x] 2.2 Add `shiftplan_id: Uuid` parameter to `ShiftplanViewService::get_shiftplan_week`
- [x] 2.3 Verify project compiles with `cargo build`

## 3. Scaffolding: DAO Layer for Shiftplan Entity

- [x] 3.1 Create `dao/src/shiftplan.rs` with `ShiftplanEntity` struct and `ShiftplanDao` trait (get_all, get_by_id, create, update, delete) with `#[automock]`
- [x] 3.2 Create `dao_impl_sqlite/src/shiftplan.rs` with `ShiftplanDaoImpl` struct and stub implementations (`todo!()`)
- [x] 3.3 Add `shiftplan_id: Option<Uuid>` field to `SlotEntity` in `dao/src/slot.rs`
- [x] 3.4 Update `dao_impl_sqlite/src/slot.rs` queries to include `shiftplan_id` in SELECT and INSERT/UPDATE
- [x] 3.5 Add `shiftplan_id: Uuid` parameter to `SlotDao::get_slots_for_week` and update its SQLite implementation to filter by it
- [x] 3.6 Verify project compiles with `cargo build`

## 4. Scaffolding: Service Layer for Shiftplan Entity

- [x] 4.1 Create `service/src/shiftplan_catalog.rs` with `Shiftplan` struct and `ShiftplanService` trait (CRUD methods) with `#[automock]`
- [x] 4.2 Create `service_impl/src/shiftplan_catalog.rs` with `ShiftplanServiceImpl` using `gen_service_impl!` macro (fully implemented, not stubs)
- [x] 4.3 Add `shiftplan_id: Option<Uuid>` to `service/src/slot.rs` `Slot` struct and update `From` conversions
- [x] 4.4 Update `SlotService::get_slots_for_week` signature to include `shiftplan_id: Uuid`
- [x] 4.5 Update `SlotService::create_slot` to require `shiftplan_id` on the `Slot` struct
- [x] 4.6 Wire new service into `ShiftplanViewService` implementation (pass `shiftplan_id` through to slot queries)
- [x] 4.7 Verify project compiles with `cargo build`

## 5. Scaffolding: REST Layer and Transport Objects

- [x] 5.1 Add `ShiftplanTO` to `rest-types` (id, name, is_planning, version) with `Serialize`, `Deserialize`, `ToSchema`
- [x] 5.2 Add `shiftplan_id` to `SlotTO` in `rest-types`
- [x] 5.3 Create `rest/src/shiftplan_catalog.rs` with CRUD endpoints (GET list, POST, GET by id, PUT, DELETE) using `#[utoipa::path]` (fully implemented)
- [x] 5.4 Update `rest/src/shiftplan.rs` (now ShiftplanView) route from `/{year}/{week}` to `/{shiftplan_id}/{year}/{week}` and pass `shiftplan_id` to service
- [x] 5.5 Wire new routes and OpenAPI docs into the main router (shifty_bin)
- [x] 5.6 Verify project compiles with `cargo build`

## 6. Red: Write Tests

- [x] 6.1 Write unit tests for `ShiftplanService` (CRUD): create, get, get_all, update with version check, soft delete
- [x] 6.2 Write unit tests for `ShiftplanViewService`: verify `shiftplan_id` is passed to slot queries (existing tests updated)
- [x] 6.3 Write unit tests for `SlotService`: verify `get_slots_for_week` filters by `shiftplan_id` (existing tests updated)
- [x] 6.4 Write tests for report exclusion: verify `ShiftplanReportDao` queries exclude `is_planning = true` plans (SQL updated)
- [x] 6.5 Verify tests compile and pass (`cargo test`)

## 7. Green: Implement DAO Layer

- [x] 7.1 Implement `ShiftplanDaoImpl` CRUD methods with SQLx queries
- [x] 7.2 Update `SlotDaoImpl::get_slots_for_week` to filter by `shiftplan_id`
- [x] 7.3 Update `SlotDaoImpl::create_slot` and `update_slot` to persist `shiftplan_id`
- [x] 7.4 Update `ShiftplanReportDaoImpl` queries: join `shiftplan` table, add `WHERE shiftplan.is_planning = 0`
- [x] 7.5 Verify `cargo build` succeeds

## 8. Green: Implement Service Layer

- [x] 8.1 Implement `ShiftplanServiceImpl` CRUD methods (create, get, get_all, update with version check, soft delete)
- [x] 8.2 Update `ShiftplanViewServiceImpl::get_shiftplan_week` to pass `shiftplan_id` to slot service
- [x] 8.3 Verify `cargo test` passes

## 9. Green: Implement REST Layer

- [x] 9.1 Implement `shiftplan_catalog.rs` REST handlers (CRUD endpoints)
- [x] 9.2 Verify `cargo test` passes
- [x] 9.3 Verify `cargo run` starts successfully and endpoints respond
