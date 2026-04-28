## 1. Scaffolding

- [x] 1.1 Add domain types `ShiftplanDayAggregate` and `PlanDayView` to `service/src/shiftplan.rs`
- [x] 1.2 Add `get_shiftplan_day` method to `ShiftplanViewService` trait with `year: u32`, `week: u8`, `day_of_week: DayOfWeek`, returning `Result<ShiftplanDayAggregate, ServiceError>`
- [x] 1.3 Add `ShiftplanService` dependency to `ShiftplanViewServiceImpl` via `gen_service_impl!`
- [x] 1.4 Extract `build_shiftplan_day` free function signature in `service_impl/src/shiftplan.rs` with a `todo!()` body
- [x] 1.5 Stub `get_shiftplan_day` implementation with `todo!()` and refactor `get_shiftplan_week` to call `build_shiftplan_day` (still `todo!()`)
- [x] 1.6 Add `ShiftplanDayAggregateTO` and `PlanDayViewTO` to `rest-types/src/lib.rs` with `ToSchema`, `Serialize`, `Deserialize` and `From` conversions
- [x] 1.7 Add REST handler `get_shiftplan_day` in `rest/src/shiftplan.rs` with `#[utoipa::path]` annotation and register route
- [x] 1.8 Wire `ShiftplanService` into `ShiftplanViewServiceImpl` in `shifty_bin` dependency injection
- [x] 1.9 Verify project compiles with `cargo build`

## 2. Tests (Red)

- [x] 2.1 Add unit test: `build_shiftplan_day` filters slots by day_of_week and assigns bookings correctly
- [x] 2.2 Add unit test: `build_shiftplan_day` excludes all slots when day is a holiday
- [x] 2.3 Add unit test: `build_shiftplan_day` filters slots on short days (only slots with `to <= time_of_day`)
- [x] 2.4 Add unit test: `build_shiftplan_day` computes `self_added` when user_assignments is provided
- [x] 2.5 Add unit test: `build_shiftplan_day` sets `self_added` to `None` when user_assignments is `None`
- [x] 2.6 Add unit test: `build_shiftplan_day` sorts slots by `from` time
- [x] 2.7 Add service test: `get_shiftplan_day` returns all shiftplans with their day slots aggregated
- [x] 2.8 Add service test: `get_shiftplan_day` returns error for invalid week number
- [x] 2.9 Add service test: `get_shiftplan_week` still produces identical results after refactoring (existing tests pass)
- [x] 2.10 Verify all new tests compile but fail with `cargo test`

## 3. Implementation (Green)

- [x] 3.1 Implement `build_shiftplan_day`: slot filtering by day, special day handling (holiday/short day), booking assignment, self_added calculation, time sorting
- [x] 3.2 Refactor `get_shiftplan_week` to use `build_shiftplan_day` in the day loop — verify existing tests still pass
- [x] 3.3 Implement `get_shiftplan_day`: load shared data once, iterate all plans, call `build_shiftplan_day` for each, assemble `ShiftplanDayAggregate`
- [x] 3.4 Verify all tests pass with `cargo test`
- [x] 3.5 Verify server starts with `cargo run` (smoke test)
