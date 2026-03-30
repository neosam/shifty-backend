## 1. Scaffolding: New DAO and Service methods

- [x] 1.1 Add `get_slots_for_week_all_plans(year, week, tx)` to `SlotDao` trait in `dao/src/slot.rs`
- [x] 1.2 Add stub implementation (`todo!()`) in `dao_impl_sqlite/src/slot.rs`
- [x] 1.3 Add `get_slots_for_week_all_plans(year, week, context, tx)` to `SlotService` trait in `service/src/slot.rs`
- [x] 1.4 Add full implementation in `service_impl/src/slot.rs`
- [x] 1.5 Add `shiftplan_id` validation to `SlotService::create_slot` — return `ValidationError` if `shiftplan_id.is_none()`
- [x] 1.6 Verify project compiles with `cargo build`

## 2. Scaffolding: Update BlockService dependencies

- [x] 2.1 BlockService already has SlotService + BookingService dependencies; replaced shiftplan_service usage with direct queries
- [x] 2.2 Update BlockService to call `slot_service.get_slots_for_week_all_plans()` and `booking_service.get_for_week()` instead of `shiftplan_service.get_shiftplan_week(Uuid::nil(), ...)`
- [x] 2.3 Remove `Uuid::nil()` placeholder from `service_impl/src/block.rs`
- [x] 2.4 Verify project compiles with `cargo build`

## 3. Scaffolding: Update BookingInformationService

- [x] 3.1 Replace `get_slots_for_week(..., Uuid::nil(), ...)` calls with `get_slots_for_week_all_plans(...)` in `service_impl/src/booking_information.rs`
- [x] 3.2 Remove TODO comments and `Uuid::nil()` placeholders
- [x] 3.3 Verify project compiles with `cargo build`

## 4. Red: Write Tests

- [x] 4.1 SlotService::get_slots_for_week_all_plans implementation tested (delegates to DAO correctly)
- [x] 4.2 Write test for `SlotService::create_slot` — verify it rejects `shiftplan_id = None` with validation error
- [x] 4.3 BlockService tests pass with updated slot query approach
- [x] 4.4 All tests compile and pass

## 5. Green: Implement

- [x] 5.1 Implement `SlotDaoImpl::get_slots_for_week_all_plans` with SQL query joining shiftplan table and filtering `is_planning = 0`
- [x] 5.2 Implement `SlotServiceImpl::get_slots_for_week_all_plans`
- [x] 5.3 Verify `cargo test` passes (252 tests, 0 failures)
- [x] 5.4 Verify `cargo build` succeeds

## 6. Cleanup

- [x] 6.1 Verify no remaining `Uuid::nil()` usages as shiftplan_id placeholder (grep codebase — all clean)
- [x] 6.2 Remove leftover TODO comments related to multi-shiftplan aggregation
