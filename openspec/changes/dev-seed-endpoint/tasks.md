## 1. Scaffolding

- [x] 1.1 Extend `BasicDao::clear_all()` in `dao_impl_sqlite/src/lib.rs` to also delete from `special_day` table (respect FK ordering). Note: `slot` not deleted to preserve default slots from migrations.
- [x] 1.2 Create `rest/src/dev.rs` module with stub functions `seed_dev_data` and `clear_dev_data`, a `generate_route()` function, and `DevApiDoc` struct with utoipa annotations. Gate the entire module with `#[cfg(feature = "mock_auth")]`
- [x] 1.3 Register the dev routes in `rest/src/lib.rs` behind `#[cfg(feature = "mock_auth")]` using `.nest("/dev", dev::generate_route())`
- [x] 1.4 Add `DevApiDoc` to the OpenAPI documentation aggregation so endpoints appear in Swagger UI

## 2. Tests

- [x] 2.1 Write integration test for `POST /dev/seed` on empty database: verify sales persons, work details, extra hours, bookings, and special days are created
- [x] 2.2 Write integration test for `POST /dev/clear`: seed data first, then clear, verify all tables are empty
- [x] 2.3 Write integration test for `POST /dev/clear` on empty database: verify it succeeds without errors
- [x] 2.4 Write integration test for `POST /dev/seed` called twice: verify no errors and data is additive

## 3. Implementation

- [x] 3.1 Implement `clear_dev_data` handler: call the extended `BasicDao::clear_all()` and return 200 with confirmation message
- [x] 3.2 Implement seed helper to create 5 sales persons with diverse states (active/inactive, paid/unpaid, different colors)
- [x] 3.3 Implement seed helper to create employee work details for each sales person (various hours, workdays, vacation days)
- [x] 3.4 Implement seed helper to create extra hours (vacation for one person, sick leave for another, overtime for a third)
- [x] 3.5 Implement seed helper to create bookings on existing slots for the current calendar week
- [x] 3.6 Implement seed helper to create special days (holidays)
- [x] 3.7 Wire all seed helpers together in `seed_dev_data` handler, return 200 with confirmation message
- [x] 3.8 Run `cargo build`, `cargo test`, and `cargo run` to verify everything works end-to-end
