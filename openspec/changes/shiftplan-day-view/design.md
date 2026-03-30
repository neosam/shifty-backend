## Context

The shiftplan view layer currently has a single entry point: `ShiftplanViewService::get_shiftplan_week`, which loads slots, bookings, sales persons, special days, and user assignments for one shiftplan and one week, then assembles a `ShiftplanWeek` containing 7 `ShiftplanDay` entries.

The day-assembly logic (filter slots by day, apply special day rules, assign bookings, compute self_added, sort by time) is embedded inline in a loop inside `get_shiftplan_week`. This logic needs to be shared with the new day-aggregate feature.

The `ShiftplanViewServiceImpl` currently depends on `SlotService`, `BookingService`, `SalesPersonService`, `SpecialDayService`, `PermissionService`, and `TransactionDao`. The new method additionally needs `ShiftplanService` (catalog) to enumerate all plans.

## Goals / Non-Goals

**Goals:**
- Provide a REST endpoint that returns all shiftplans for a specific day, grouped by plan
- Extract the day-building logic into a reusable helper function
- Refactor `get_shiftplan_week` to use the extracted helper (no behavior change)

**Non-Goals:**
- Ordering/sorting of shiftplans (separate future change)
- Filtering which plans to include (all non-deleted plans are returned)
- Changes to the existing week endpoint's API contract

## Decisions

### 1. Extract `build_shiftplan_day` as a free function

**Decision**: Extract the per-day logic into a standalone function in `service_impl/src/shiftplan.rs`:

```rust
fn build_shiftplan_day(
    day_of_week: DayOfWeek,
    slots: &[Slot],
    bookings: &[Booking],
    sales_persons: &[SalesPerson],
    special_days: &[SpecialDay],
    user_assignments: Option<&HashMap<Uuid, Arc<str>>>,
) -> Result<ShiftplanDay, ServiceError>
```

**Why**: A free function (not a method) keeps it pure — no async, no DAO access, easily testable. Both `get_shiftplan_week` and `get_shiftplan_day` call it after loading their data independently.

**Alternative considered**: Making it an `&self` method on the impl. Rejected because the function needs no access to `self` — all data is passed in.

### 2. Add `ShiftplanService` dependency to `ShiftplanViewServiceImpl`

**Decision**: Add `ShiftplanService` to the `gen_service_impl!` macro call for `ShiftplanViewServiceImpl`.

**Why**: `get_shiftplan_day` needs to enumerate all shiftplans. The catalog service already exists and provides `get_all()`.

**Alternative considered**: Adding a new DAO method to get all shiftplan IDs. Rejected because the service already exists and handles permissions.

### 3. New domain types for the aggregate response

**Decision**: Add to `service/src/shiftplan.rs`:

```rust
pub struct ShiftplanDayAggregate {
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
    pub plans: Vec<PlanDayView>,
}

pub struct PlanDayView {
    pub shiftplan: shiftplan_catalog::Shiftplan,
    pub slots: Vec<ShiftplanSlot>,
}
```

And corresponding TOs in `rest-types`.

**Why**: Clean separation — the aggregate wraps multiple plans' day views. `PlanDayView` pairs a shiftplan with its slots for that day.

### 4. REST endpoint path: `/shiftplan-day/{year}/{week}/{day_of_week}`

**Decision**: Use `DayOfWeekTO` string values (`Monday`, `Tuesday`, etc.) as the path parameter for `day_of_week`.

**Why**: Consistent with the existing `DayOfWeekTO` enum which already implements `Serialize`/`Deserialize`. Axum can deserialize path parameters using serde.

### 5. Data loading strategy in `get_shiftplan_day`

**Decision**: Load shared data (special days, sales persons, bookings, user assignments) once, then loop over all plans loading only slots per plan.

```
1. Load special_days, sales_persons, bookings (for the week), user_assignments — once
2. Load all shiftplans via ShiftplanService::get_all()
3. For each plan: load slots via get_slots_for_week, then build_shiftplan_day for the target day
4. Assemble ShiftplanDayAggregate
```

**Why**: Bookings, sales persons, and special days are not plan-specific — loading them once avoids redundant queries. Only slots are plan-scoped.

## Risks / Trade-offs

- **Additional dependency on ShiftplanService**: Increases coupling of `ShiftplanViewServiceImpl`. → Acceptable trade-off since the catalog is a lightweight, read-only service.
- **Loading full week of slots per plan, then filtering to one day**: Slightly wasteful, but `get_slots_for_week` is the existing API and adding a `get_slots_for_day` DAO method would be premature optimization for now. → Can be optimized later if needed.
- **Refactoring `get_shiftplan_week`**: Risk of subtle behavior change. → Mitigated by existing tests; the extraction is mechanical.
