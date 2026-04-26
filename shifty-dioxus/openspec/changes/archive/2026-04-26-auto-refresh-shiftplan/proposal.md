## Why

When the shiftplan structure is modified (e.g., adding, editing, or deleting a slot), the changes are not immediately visible in the week view. Users must navigate away (e.g., switch weeks) and back to see updates. This breaks the expected feedback loop and creates confusion about whether changes were saved.

## What Changes

- Introduce a global refresh signal that the `slot_edit_service` triggers after successful save/delete operations
- The shiftplan page's `use_resource` will track this signal as a dependency, causing automatic data reload
- After any structural change (slot create, edit, delete), the week view will immediately reflect the new state

## Capabilities

### New Capabilities
- `shiftplan-refresh`: Global refresh mechanism for the shiftplan view that can be triggered by any service after data mutations

### Modified Capabilities

## Impact

- `src/service/slot_edit.rs`: Trigger refresh signal after SaveSlot and DeleteSlot
- `src/page/shiftplan.rs`: Add refresh signal as dependency to `shift_plan_context` resource
- New service module or addition to existing service for the global refresh signal
