## Why

The current shiftplan view only shows one plan at a time in a weekly layout. To see what's happening across all plans on a given day, users must manually switch between tabs. This makes it hard to get a quick overview of staffing for a specific day — the most common use case for both planners and regular employees.

A backend endpoint (`/shiftplan-info/day/{year}/{week}/{day_of_week}`) already exists that aggregates all shiftplans for a single day. The frontend just needs a view to consume it.

## What Changes

- Add a **Week/Day toggle** to the shiftplan page that switches between the existing week view and a new day view
- In day view, display **all shiftplans as columns** side by side for a single day, using plan names as column headers
- Add a **day button bar** (Mo, Di, Mi, Do, Fr, Sa, optionally So) for direct day selection, plus arrow buttons for prev/next navigation
- Smart default day selection: today's weekday when viewing the current week, Monday otherwise
- Arrow navigation wraps across week boundaries (Monday left goes to previous week Saturday, Saturday right goes to next week Monday)
- Sunday button only visible when Sunday slots exist
- Booking (add/remove) works the same as in the week view
- Auto-refresh after booking mutations
- Hide the shiftplan tab bar in day view (all plans are visible simultaneously)

## Capabilities

### New Capabilities
- `shiftplan-day-view`: Day-level aggregated view of all shiftplans with day navigation, booking support, and week/day toggle

### Modified Capabilities

## Impact

- **Frontend components**: New day view component reusing existing `ColumnView`, `ColumnViewSlot`, and `TimeView` components
- **Frontend state**: New state for view mode (week/day) and selected day on the shiftplan page
- **API layer**: New API function to call `/shiftplan-info/day/{year}/{week}/{day_of_week}`
- **REST types**: New frontend types mirroring `ShiftplanDayAggregateTO`, `PlanDayViewTO`, `ShiftplanSlotTO`, `ShiftplanBookingTO`
- **No backend changes required** — endpoint already exists
- **No breaking changes**
