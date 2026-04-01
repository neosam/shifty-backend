## Context

The `ShiftplanTabBar` component currently displays tabs for each shiftplan with click-to-select. The shiftplan page has a `change_structure_mode` signal that controls whether slot editing UI is shown. This same mode will gate the shiftplan CRUD controls.

The backend provides:
- `POST /shiftplan-catalog` — create (accepts `ShiftplanTO`, returns created `ShiftplanTO`)
- `PUT /shiftplan-catalog/{id}` — update (accepts `ShiftplanTO`, returns updated `ShiftplanTO`)
- `DELETE /shiftplan-catalog/{id}` — delete

## Goals / Non-Goals

**Goals:**
- Add `+` button to tab bar for creating shiftplans (modal with name input)
- Add `✕` button on each tab for deleting (with confirmation dialog)
- Add inline rename via double-click on tab name
- All three actions only visible/available when `change_structure_mode` is active
- Refresh the catalog after any mutation

**Non-Goals:**
- Editing `is_planning` from the UI (set to `false` by default on create)
- Drag-and-drop reordering of tabs
- Dedicated shiftplan management page

## Decisions

### 1. Extend ShiftplanTabBar with a `planning_mode` prop

The tab bar receives a boolean `planning_mode` prop from the shiftplan page (sourced from `change_structure_mode`). When true, `✕` buttons appear on tabs and `+` button appears at the end.

**Rationale:** Keeps the tab bar self-contained. The page just passes one extra prop.

### 2. Modal for creation, inline edit for rename, confirm dialog for delete

Three different UI patterns for three different actions:
- **Create**: Modal dialog with a text input and submit button. Cleanest for a "new entity" workflow.
- **Rename**: Double-click turns tab name into an `<input>`, press Enter or blur to save. Fast and intuitive.
- **Delete**: Browser-style confirm or a small inline confirmation. Prevents accidental deletion.

**Alternative considered:** Modal for all three — would be consistent but slower for rename.

### 3. API functions in api.rs, no separate service/loader

The CRUD operations are simple HTTP calls. They go directly in `api.rs`. The tab bar component or the shiftplan page calls them via `spawn()` and refreshes the catalog resource afterward.

**Rationale:** No need for a dedicated service with GlobalSignal — the catalog `use_resource` on the shiftplan page can just be restarted after mutations. This matches the existing pattern for slot editing.

### 4. After mutation: restart catalog resource and auto-select

- After **create**: restart catalog, select the newly created shiftplan
- After **rename**: restart catalog, keep current selection
- After **delete**: restart catalog, select first remaining shiftplan (or None if empty)

### 5. Use existing Modal component for creation dialog

The project already has a `Modal` component in `src/component/modal.rs`. Reuse it for the create dialog.

## Risks / Trade-offs

- **Inline rename UX**: Double-click to edit may not be discoverable. Acceptable since it's behind planning mode which is a power-user feature.
- **Concurrent edits**: No optimistic locking on the tab bar level — the `version` field on `ShiftplanTO` handles conflicts at the API level (409 response).
