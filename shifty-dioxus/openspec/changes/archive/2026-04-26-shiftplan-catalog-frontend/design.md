## Context

The backend has a new `shiftplan_catalog` module with CRUD endpoints under `/shiftplan`. The existing endpoints `/shiftplan-info/{year}/{week}` and `/slot/week/{year}/{week}` now require a `shiftplan_id`. The frontend must reflect these changes.

Current data flow:
```
ShiftPlan page → use_resource(load_shift_plan(config, week, year))
                   → api::get_shiftplan_week(config, year, week)
                      → GET /shiftplan-info/{year}/{week}
```

New data flow:
```
ShiftPlan page → use_resource(load_shiftplan_catalog(config))
                   → api::get_all_shiftplans(config)
                      → GET /shiftplan
               → use_resource(load_shift_plan(config, shiftplan_id, week, year))
                   → api::get_shiftplan_week(config, shiftplan_id, year, week)
                      → GET /shiftplan-info/{shiftplan_id}/{year}/{week}
```

## Goals / Non-Goals

**Goals:**
- Load the shiftplan catalog from the backend and display it as a tab bar
- Automatically select the first shiftplan on startup
- Use the selected `shiftplan_id` in all API calls
- Log a warning and ignore slots with `shiftplan_id == None`
- New slots receive the `shiftplan_id` of the active shiftplan

**Non-Goals:**
- CRUD UI for shiftplans (creating/editing/deleting shiftplans)
- Persisting the tab selection (URL, LocalStorage)
- Visual indication of `is_planning` status
- Shiftplan management page

## Decisions

### 1. Tab bar as a Dioxus component above the WeekView

The tab bar is implemented as a standalone `ShiftplanTabBar` component, placed in `src/page/shiftplan.rs` between the navigation bar (week selector) and the `WeekView`.

**Rationale:** Minimally invasive, follows the existing component pattern. The tab bar is purely presentational and receives data via props.

### 2. Shiftplan catalog as `use_resource` in the shiftplan page

The catalog is loaded once (independent of week/year). The selection is kept as a `Signal<Option<Uuid>>` in the page state.

**Alternative considered:** A dedicated service with GlobalSignal — would be overkill since the catalog is only needed on the shiftplan page.

### 3. Extend existing loader/API functions instead of creating new ones

`api::get_shiftplan_week` and `api::get_slots` receive an additional `shiftplan_id: Uuid` parameter. The `loader::load_shift_plan` call is updated accordingly.

**Rationale:** Follows the backend — the old endpoints no longer exist.

### 4. Extend SlotTO and SlotEditItem with shiftplan_id

`rest-types::SlotTO` gets `shiftplan_id: Option<Uuid>`. `SlotEditItem` also gets `shiftplan_id: Option<Uuid>`. When creating a new slot, the ID from the active tab is used.

### 5. Log and ignore slots without shiftplan_id

In the loader, slots with `shiftplan_id == None` are logged via `warn!()` and filtered out from the display.

## Risks / Trade-offs

- **Backend must be running**: The catalog must contain at least one shiftplan, otherwise there is nothing to display → The UI should handle the empty state gracefully.
- **Breaking API change**: The old endpoints no longer exist → Frontend and backend must be deployed simultaneously.
