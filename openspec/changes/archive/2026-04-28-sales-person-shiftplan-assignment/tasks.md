## 1. Scaffolding — Database & DAO

- [x] 1.1 Create SQLite migration for `sales_person_shiftplan` join table with composite PK `(sales_person_id, shiftplan_id)` and audit columns
- [x] 1.2 Define `SalesPersonShiftplanDao` trait in `dao` crate with methods: `get_by_sales_person`, `get_by_shiftplan`, `set_for_sales_person`, `has_any_assignment`, `is_assigned`
- [x] 1.3 Stub `SalesPersonShiftplanDao` SQLite implementation in `dao_impl_sqlite` with `todo!()` bodies

## 2. Scaffolding — Service Layer

- [x] 2.1 Define `SalesPersonShiftplanService` trait in `service` crate with methods: `get_shiftplans_for_sales_person`, `set_shiftplans_for_sales_person`, `get_bookable_sales_persons`, `is_eligible`
- [x] 2.2 Stub `SalesPersonShiftplanServiceImpl` in `service_impl` using `gen_service_impl!` macro with `todo!()` bodies
- [x] 2.3 Add `SalesPersonShiftplanService` dependency to the booking service impl (prepare for eligibility check in `create`)

## 3. Scaffolding — REST Layer

- [x] 3.1 Add DTOs to `rest-types`: request/response types for shiftplan assignment endpoints (using Vec<Uuid> and existing SalesPersonTO directly)
- [x] 3.2 Stub REST handlers for `GET /api/sales-person/{id}/shiftplans`, `PUT /api/sales-person/{id}/shiftplans`, and `GET /api/sales-person/by-shiftplan/{shiftplan_id}` with `todo!()` bodies
- [x] 3.3 Register routes and add `#[utoipa::path]` annotations
- [x] 3.4 Wire up dependency injection in `shifty_bin`

## 4. Red — Write Tests

- [x] 4.1 Write unit tests for `SalesPersonShiftplanService`: setting assignments, getting assignments, clearing assignments
- [x] 4.2 Write unit tests for `get_bookable_sales_persons` with permissive logic: no assignments returns all, mixed assignments returns correct subset
- [x] 4.3 Write unit tests for `is_eligible`: no assignments = eligible, assigned to plan = eligible, assigned to other plan = not eligible
- [x] 4.4 Write unit tests for booking creation: eligible booking succeeds, ineligible booking is rejected
- [x] 4.5 Write unit tests for permission checks: `shiftplanner` privilege required for assignment management

## 5. Green — DAO Implementation

- [x] 5.1 Implement `SalesPersonShiftplanDao` SQLite methods: `get_by_sales_person`, `get_by_shiftplan`, `set_for_sales_person` (DELETE + INSERT in transaction), `has_any_assignment`, `is_assigned`
- [x] 5.2 Update SQLx query files (`cargo sqlx prepare`)

## 6. Green — Service Implementation

- [x] 6.1 Implement `SalesPersonShiftplanServiceImpl` methods with permission checks (`shiftplanner` privilege) and transaction management
- [x] 6.2 Implement `get_bookable_sales_persons` with permissive logic: query all sales persons, filter by eligibility
- [x] 6.3 Extend booking `create()` method: load slot to get `shiftplan_id`, call `is_eligible`, reject with error if not eligible

## 7. Green — REST Implementation

- [x] 7.1 Implement REST handlers: `GET /api/sales-person/{id}/shiftplans` returns assigned shiftplan IDs
- [x] 7.2 Implement REST handler: `PUT /api/sales-person/{id}/shiftplans` accepts array of shiftplan IDs, calls service
- [x] 7.3 Implement REST handler: `GET /api/sales-person/by-shiftplan/{shiftplan_id}` returns bookable sales persons

## 8. Verification

- [x] 8.1 Run full test suite (`cargo test`) and verify all tests pass
- [x] 8.2 Run `cargo build` and `cargo run` to verify compilation and startup
