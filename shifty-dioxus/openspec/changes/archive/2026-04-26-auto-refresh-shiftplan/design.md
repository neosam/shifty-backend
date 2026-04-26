## Context

The shiftplan page loads slot data via `use_resource` which depends on `week`, `year`, and `selected_shiftplan_id` signals. When bookings change, the page coroutine calls `update_shiftplan()` which triggers `shift_plan_context.restart()` and reloads related data.

However, the `slot_edit_service` runs as a separate global coroutine and has no access to the page-local `shift_plan_context`. After saving or deleting a slot, no reload is triggered.

## Goals / Non-Goals

**Goals:**
- Shiftplan view auto-refreshes after slot create, edit, or delete
- Solution follows existing architectural patterns (GlobalSignal + reactive dependencies)
- Minimal code changes

**Non-Goals:**
- Real-time sync across multiple browser sessions
- Generic pub/sub event system
- Refresh optimization (debouncing, partial updates)

## Decisions

### Use a GlobalSignal refresh counter

Introduce a `GlobalSignal<u64>` (e.g., `SHIFTPLAN_REFRESH`) that acts as a version counter. Any service that mutates shiftplan structure increments it. The shiftplan page reads this signal inside its `use_resource`, making it a reactive dependency.

**Why a counter over a boolean flag?** A counter ensures every increment triggers a new resource run, even if multiple mutations happen in sequence. A boolean toggle could miss updates if flipped back before the resource reacts.

**Why GlobalSignal over ShiftPlanAction?** The `slot_edit_service` is a global coroutine without access to the page's coroutine handle. A GlobalSignal is accessible from anywhere without coupling services to page-level state.

**Alternatives considered:**
- *Route through ShiftPlanAction*: Would require passing the coroutine handle into `slot_edit_service` or restructuring the service — higher coupling, more invasive change.
- *use_effect watching SLOT_EDIT_STORE.visible*: Fragile — ties refresh to UI state rather than data mutation events. Would also trigger on cancel.

### Place the signal in the slot_edit service module

The `SHIFTPLAN_REFRESH` signal will live in `src/service/slot_edit.rs` alongside `SLOT_EDIT_STORE`, since slot editing is currently the only producer. If more producers emerge later, it can be moved to a shared module.

### Read the signal in the shiftplan page's use_resource

Inside the `use_resource` closure for `shift_plan_context`, add `SHIFTPLAN_REFRESH.read()`. Dioxus tracks this as a dependency and re-runs the resource when it changes.

## Risks / Trade-offs

- **Unnecessary reloads on edit-without-change**: If a user opens the edit dialog and saves without changing anything, a reload still occurs. Acceptable — the API call is lightweight and correctness matters more.
- **Signal placement may need moving later**: If other services also need to trigger shiftplan refresh, the signal should move to a shared location. Low risk — trivial refactor.
