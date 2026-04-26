## Context

The `ShiftplanTabBar` component currently has two separate UI patterns for shiftplan management:
1. A create modal with only a name input field
2. Inline rename on double-click (text input replaces tab text)

The `ShiftplanTO` type already has an `is_planning` field, and `api::update_shiftplan` sends the full `ShiftplanTO` to the backend. There is no UI to toggle `is_planning`.

The existing `Modal` component from `src/component/modal.rs` is used for both the create and delete confirmation dialogs.

## Goals / Non-Goals

**Goals:**
- Single modal component for both create and edit operations
- Expose `name` and `is_planning` fields in the dialog
- Only accessible in structure mode
- Reuse existing `Modal` component and API functions

**Non-Goals:**
- Adding new backend endpoints or modifying `ShiftplanTO`
- Separate service/store pattern (complexity not warranted for this scope)
- i18n for the dialog (existing modals in `ShiftplanTabBar` use hardcoded German strings)

## Decisions

### Keep dialog state local to ShiftplanTabBar

The dialog state (mode, name draft, is_planning draft) will be managed with `use_signal` inside `ShiftplanTabBar`, matching the current pattern for the create modal and delete confirmation. No GlobalSignal store needed.

**Why not a service like SlotEdit?** The SlotEdit service exists because slot editing involves complex async loading (fetching slot details from API) and is triggered from multiple places. Shiftplan editing is simpler — all data is already available in the `shiftplans` prop — and is only triggered from the tab bar.

### Use an enum for dialog mode

```rust
enum ShiftplanDialogMode {
    Hidden,
    Create,
    Edit(ShiftplanTO),
}
```

This replaces the current `show_create_modal: Signal<bool>` and `editing_id: Signal<Option<Uuid>>` with a single signal. The `Edit` variant carries the original `ShiftplanTO` for version tracking and preserving the `id`.

### Extend create_shiftplan API to accept is_planning

Currently `api::create_shiftplan(config, &name)` hardcodes `is_planning: false`. Change the signature to accept `is_planning: bool` so the create modal can set it.

## Risks / Trade-offs

- **Removing inline rename**: Users familiar with double-click-to-rename will now get a modal instead. This is intentional — the modal provides a better UX for multiple fields, and the rename capability is preserved within it.
- **Hardcoded German strings**: Follows existing pattern in the tab bar. i18n can be added later when the rest of the tab bar is localized.
