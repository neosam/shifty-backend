## Context

The shiftplan page currently shows a single shiftplan in a weekly grid layout. Users switch between plans using a tab bar. A backend endpoint `/shiftplan-info/day/{year}/{week}/{day_of_week}` already exists that returns all shiftplans aggregated for a single day as `ShiftplanDayAggregateTO`.

The frontend uses a component hierarchy: `WeekView` → `DayView` → `ColumnView` → `ColumnViewSlot`, with `TimeView` as a sticky left sidebar. The `ColumnView` and `ColumnViewSlot` components are already generic — they render columns of time-based items with booking controls, independent of what a "column" represents.

## Goals / Non-Goals

**Goals:**
- Add a toggleable day view that shows all shiftplans side by side for one day
- Reuse existing `ColumnView`, `ColumnViewSlot`, and `TimeView` components
- Support full booking interactions (add/remove) identical to the week view
- Provide intuitive day navigation with weekday buttons and arrow keys
- Smart default day selection based on current week detection

**Non-Goals:**
- Dedicated URL routing for the day view (stays within existing shiftplan route)
- Editing slot structure in day view (change structure mode is week-view only)
- Conflict detection across plans in day view (this could be a future enhancement)
- Print-specific styling for day view

## Decisions

### 1. New `DayAggregateView` component alongside existing `WeekView`

The shiftplan page will conditionally render either `WeekView` or a new `DayAggregateView` based on view mode state. The new component follows the same pattern as `WeekView` but maps plans to columns instead of weekdays to columns.

**Alternative considered:** Extending `WeekView` to handle both modes internally. Rejected because the data sources are fundamentally different (`ShiftplanWeekTO` vs `ShiftplanDayAggregateTO`) and mixing them would complicate the already complex `WeekView` component.

### 2. View mode state lives in the shiftplan page, not in a global signal

The `view_mode` (Week/Day) and `selected_day` state will be local to the shiftplan page component. This avoids polluting global state for what is purely a UI concern within one page.

**State shape:**
```rust
enum ViewMode {
    Week,
    Day,
}
// selected_day: Signal<Weekday> — updated on toggle and navigation
```

### 3. Day navigation wraps across week boundaries

When pressing left on Monday, the view navigates to the previous week's Saturday. When pressing right on the last visible day, it navigates to the next week's Monday. This reuses the existing week navigation logic (`NextWeek`/`PreviousWeek` actions) combined with updating `selected_day`.

### 4. Transform `ShiftplanDayAggregateTO` to reuse `ColumnView` input format

The API response needs to be transformed into the `ColumnViewItem<Slot>` format that `ColumnView` already consumes. Each `PlanDayViewTO` becomes a column, with its slots mapped to `ColumnViewItem` entries. This transformation happens in a new loader function.

### 5. Tab bar hidden in day view, day button bar shown instead

In day view, the shiftplan tab bar is hidden since all plans are visible as columns. Instead, a day button bar is shown with weekday buttons (Mo–Sa, optionally So). The active day is highlighted. This is a simple conditional render in the shiftplan page.

### 6. Sunday visibility based on slot data

Sunday appears in the day button bar only when the day aggregate response contains plans with Sunday slots. This requires loading Sunday data to check, or alternatively checking the shiftplan catalog's slot definitions. The simpler approach: always include Sunday in the button bar but visually disable it if there are no Sunday slots — or just omit it like the week view does. We follow the week view pattern: only show Sunday if slots exist.

### 7. Booking actions reuse existing coroutine

The existing `ShiftPlanAction::AddUserToSlot` and `ShiftPlanAction::RemoveUserFromSlot` actions work with slot IDs and sales person IDs, which are available in the day view data. After a mutation, the day view data is refreshed by re-fetching from the day endpoint.

## Risks / Trade-offs

**[Multiple API calls for Sunday detection]** → To know if Sunday has slots, we either need to load Sunday data separately or derive it from another source. Mitigation: Check shiftplan catalog slot definitions which are already loaded, or simply load the day data on demand when Sunday is clicked.

**[Data refresh after booking]** → The day endpoint returns data for all plans. A booking mutation in one plan requires refreshing the entire day aggregate. This is slightly less efficient than the week view (which refreshes one plan). Mitigation: The endpoint is lightweight and this trade-off is acceptable for simplicity.

**[Week navigation in day mode]** → When wrapping across weeks, the new week's data must be loaded before displaying. This introduces a brief loading state. Mitigation: Show a loading indicator, same as when navigating weeks in the week view.
