## 1. Scaffolding

- [x] 1.1 Add SQLite migration: `ALTER TABLE sales_person_shiftplan ADD COLUMN permission_level TEXT NOT NULL DEFAULT 'available' CHECK(permission_level IN ('available', 'planner_only'))`
- [x] 1.2 Extend DAO trait `SalesPersonShiftplanDao`: add `get_permission_level(sales_person_id, shiftplan_id) -> Option<String>` method; modify `set_for_sales_person` to accept permission levels alongside shiftplan IDs (e.g., `&[(Uuid, String)]` instead of `&[Uuid]`)
- [x] 1.3 Stub DAO implementation in `dao_impl_sqlite`: implement new/modified methods with `todo!()`
- [x] 1.4 Extend service trait `SalesPersonShiftplanService`: add `Authentication<Context>` parameter to `is_eligible`; update `get_shiftplans_for_sales_person` return type to include permission levels; update `set_shiftplans_for_sales_person` to accept permission levels
- [x] 1.5 Stub service implementation: update `SalesPersonShiftplanServiceImpl` method signatures with `todo!()` for new logic
- [x] 1.6 Update `BookingService` to pass authentication context to `is_eligible` calls
- [x] 1.7 Add REST types: create `ShiftplanAssignmentTO` DTO with `shiftplan_id` and `permission_level` fields, add `ToSchema` derive
- [x] 1.8 Update REST endpoints: adjust `set_shiftplans_for_sales_person` to accept `Vec<ShiftplanAssignmentTO>`; adjust `get_shiftplans_for_sales_person` to return `Vec<ShiftplanAssignmentTO>`
- [x] 1.9 Fix all compilation errors from signature changes across the codebase (including existing tests and mocks)

## 2. Tests (Red)

- [x] 2.1 Test: assignment defaults to `available` permission level when not specified
- [x] 2.2 Test: assignment stores `planner_only` permission level correctly
- [x] 2.3 Test: `is_eligible` returns true for person with no assignments (unchanged behavior)
- [x] 2.4 Test: `is_eligible` returns true for `available` assignment regardless of caller role
- [x] 2.5 Test: `is_eligible` returns true for `planner_only` assignment when caller is shiftplanner
- [x] 2.6 Test: `is_eligible` returns false for `planner_only` assignment when caller is non-shiftplanner
- [x] 2.7 Test: `is_eligible` returns false when person has other assignments but not for this plan (unchanged behavior)
- [x] 2.8 Test: `get_bookable_sales_persons` includes `planner_only` persons for shiftplanner caller
- [x] 2.9 Test: `get_bookable_sales_persons` excludes `planner_only` persons for non-shiftplanner caller
- [x] 2.10 Test: booking creation succeeds for shiftplanner with `planner_only` assignment
- [x] 2.11 Test: booking creation fails for non-shiftplanner with `planner_only` assignment
- [x] 2.12 Test: booking deletion denied for non-shiftplanner when booking is in `planner_only` shiftplan
- [x] 2.13 Test: booking deletion succeeds for shiftplanner when booking is in `planner_only` shiftplan

## 3. Implementation (Green)

- [x] 3.1 Implement DAO: `get_permission_level` SQL query, update `set_for_sales_person` to insert with permission level, update `get_by_sales_person` to return permission levels
- [x] 3.2 Implement service: `is_eligible` with role-aware logic (check permission level + caller's shiftplanner privilege)
- [x] 3.3 Implement service: `get_bookable_sales_persons` with role-aware filtering
- [x] 3.4 Implement booking service: pass authentication context through to `is_eligible`; add deletion permission check for `planner_only` assignments
- [x] 3.5 Implement REST: wire up new DTOs and updated endpoint logic
- [x] 3.6 Run full test suite and fix any remaining failures
