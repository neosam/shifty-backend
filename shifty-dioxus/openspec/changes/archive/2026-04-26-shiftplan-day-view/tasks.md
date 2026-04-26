## 1. REST Types and API Layer

- [x] 1.1 Add frontend REST types mirroring `ShiftplanDayAggregateTO`, `PlanDayViewTO`, `ShiftplanSlotTO`, `ShiftplanBookingTO` in `rest-types`
- [x] 1.2 Add API function `get_shiftplan_day(year, week, day_of_week)` in `src/api.rs` calling `GET /shiftplan-info/day/{year}/{week}/{day_of_week}`

## 2. State and Data Transformation

- [x] 2.1 Add `ViewMode` enum (Week/Day) and day view state types in `src/state/shiftplan.rs`
- [x] 2.2 Add loader function to transform `ShiftplanDayAggregateTO` into structures compatible with `ColumnView` input format (mapping each plan to a column with its slots)

## 3. Day Aggregate View Component

- [x] 3.1 Create `DayAggregateView` component that renders all shiftplan columns for a single day, reusing `ColumnView`, `ColumnViewSlot`, and `TimeView`
- [x] 3.2 Add day button bar component with weekday buttons (Mo–Sa, optionally So) and left/right arrow buttons
- [x] 3.3 Implement smart day selection logic: today's weekday for current week, Monday otherwise

## 4. Shiftplan Page Integration

- [x] 4.1 Add view mode toggle (Week/Day) to the shiftplan page header
- [x] 4.2 Add view mode state and selected day signal to the shiftplan page
- [x] 4.3 Conditionally render `WeekView` or `DayAggregateView` based on view mode
- [x] 4.4 Hide shiftplan tab bar when in day view
- [x] 4.5 Add day data loading to the shiftplan page coroutine with new action variants (LoadDay, SelectDay, NextDay, PreviousDay)

## 5. Day Navigation

- [x] 5.1 Implement day arrow navigation with week boundary wrapping (Monday left → previous week Saturday, Saturday right → next week Monday)
- [x] 5.2 Implement Sunday visibility logic: only show Sunday button when Sunday slots exist

## 6. Booking in Day View

- [x] 6.1 Wire booking add/remove events from `DayAggregateView` to existing `AddUserToSlot`/`RemoveUserFromSlot` actions
- [x] 6.2 Add auto-refresh of day aggregate data after booking mutations

## 7. Tests

- [x] 7.1 Add tests for day navigation logic (day stepping, week boundary wrapping, smart default selection)
- [x] 7.2 Add tests for data transformation from `ShiftplanDayAggregateTO` to `ColumnView` input format
- [x] 7.3 Add tests for Sunday visibility logic
