## Context

The multi-shiftplan change added `shiftplan_id` filtering to `SlotDao::get_slots_for_week`. Services like BlockService and BookingInformationService need slots from all non-planning plans for a given week but have no way to request this — they currently pass `Uuid::nil()` which matches nothing.

## Goals / Non-Goals

**Goals:**
- Provide a clean way to query all non-planning slots for a week
- Fix BlockService and BookingInformationService to use correct slot data
- Validate shiftplan_id on slot creation
- Remove all `Uuid::nil()` placeholder usages

**Non-Goals:**
- Changing BlockService to be fully plan-aware (separate iCal per plan) — that's a future enhancement
- Cross-plan hour deduplication — remains out of scope

## Decisions

### 1. New `get_slots_for_week_all_plans` method on SlotDao

Add a method that returns all slots for a week where the associated shiftplan has `is_planning = false` (or `shiftplan_id IS NULL` for backward compatibility).

**Why a new method instead of making shiftplan_id optional?** The existing `get_slots_for_week` with a required `shiftplan_id` is the correct API for plan-scoped views. A separate method makes the intent explicit: "give me everything that counts as operational."

**SQL:**
```sql
SELECT ... FROM slot
LEFT JOIN shiftplan ON slot.shiftplan_id = shiftplan.id
WHERE deleted IS NULL
  AND valid_from <= ?
  AND (valid_to IS NULL OR valid_to >= ?)
  AND (shiftplan.is_planning = 0 OR shiftplan.is_planning IS NULL)
```

### 2. BlockService uses SlotService directly instead of ShiftplanViewService

Currently BlockService calls `shiftplan_service.get_shiftplan_week()` which internally calls `slot_service.get_slots_for_week()`. Instead, BlockService should call `slot_service.get_slots_for_week_all_plans()` and `booking_service.get_for_week()` directly, assembling the data it needs without going through the view service.

**Why?** The view service is designed for a single-plan weekly view. BlockService needs cross-plan data. Coupling through the view service adds an unnecessary indirection and the shiftplan_id requirement.

### 3. Slot creation validation at service level

`SlotService::create_slot` will check `slot.shiftplan_id.is_some()` and return a `ValidationError` if missing.

**Why not at DAO level?** Consistent with existing patterns — the service layer validates, the DAO layer persists.

## Risks / Trade-offs

- **BlockService refactor scope**: Changing from `get_shiftplan_week` to direct slot+booking queries requires understanding what the block service actually needs. Risk: missed functionality. Mitigation: existing tests cover block generation behavior.
- **Performance**: `get_slots_for_week_all_plans` joins the shiftplan table. Impact is negligible given SQLite's query plan and the small table size.
