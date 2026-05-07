## Context

The shifty-backend crate just shipped `extra-hours-update`, making `PUT /extra-hours/{id}` functional with the following contract:

- The path id is the **logical id** — stable across edits.
- Optimistic locking via `$version`. A stale version returns `409 Conflict`.
- Editable fields: `amount`, `category`, `description`, `date_time`, `custom_extra_hours_id`. `sales_person_id` is immutable; sending a different value returns a validation error.
- Permission: `HR_PRIVILEGE` OR the caller is the entry's own sales person.

On the frontend, the existing flow is:

- `ExtraHoursModal` (`src/component/extra_hours_modal.rs`) — POST-only create dialog, opened from `employee_details.rs` and `my_employee_details.rs` via the "Other hours" button on the employee view.
- `ExtraHoursView` / `ExtraHoursCategorySection` (`src/component/employee_view.rs`) — renders entries grouped by category. Each entry currently has a delete button only.
- `EmployeeAction::DeleteExtraHours(Uuid)` (`src/service/employee.rs`) — the existing pattern for entry mutations.
- `ExtraHours` state (`src/state/employee.rs`) already carries `id`, `version`, `sales_person_id`, `amount`, `category`, `description`, `date_time` — every field needed to build a PUT body.
- `ExtraHoursTO` (`rest-types`) already has the `$version` field; no DTO change is needed.

The user has confirmed the scope: extend the existing modal (rather than fork a separate edit modal), allow category change in edit mode (with `vacation_days` excluded), refresh on 409, and show the edit affordance unconditionally because the backend enforces permissions.

## Goals / Non-Goals

**Goals:**
- A user with permission can click an edit button on any extra hours entry and modify `amount`, `category`, `description`, `date_time` through the same dialog they use to create new entries.
- The edit submit issues `PUT /extra-hours/{id}` with the entry's last-read `$version`; on success the employee data is reloaded so the list reflects the new active row.
- A `409 Conflict` response triggers an automatic refresh of the employee data plus a translated user-visible notice (i.e. "the entry was modified elsewhere; please retry").
- The same dialog component handles both create and edit. The mode is controlled by a single optional prop (`editing: Option<ExtraHours>`), keeping the test surface small.
- Both the HR-facing employee detail page and the user's own "My Employee Details" page expose the edit affordance.

**Non-Goals:**
- No history view of past versions (the backend preserves them via the logical-id pattern, but surfacing them is a separate change).
- No bulk edit across multiple entries.
- No vacation-range edit. `VacationDays` is a bulk-create shorthand and is intentionally not editable as a unit; individual `Vacation` rows are edited like any other entry.
- No changes to `CustomExtraHours` definitions (already has its own management page).
- No new permission gating in the UI — the backend rejects unauthorized PUTs with 403 and the existing `ErrorView` surfaces it.

## Decisions

### Decision 1: Extend `ExtraHoursModal` with an `editing` prop, do not fork a second modal

The create modal already contains 95 % of the form surface that edit needs: category select, description input, amount input, datetime input, custom-hours loading, footer with cancel + submit buttons. A separate `EditExtraHoursModal` would duplicate all of that. Instead:

- Add `editing: Option<ExtraHours>` to `ExtraHoursModalProps`. `None` = create (current behavior). `Some(eh)` = edit.
- Initial signal values (`category`, `amount`, `description`, `when`) are derived from `editing` if present, otherwise from the existing defaults.
- The dialog title switches between `AddExtraHoursFormTitle` (existing) and a new `EditExtraHoursFormTitle`.
- The submit handler branches: `editing.is_none()` → POST via existing `add_extra_hour`; `editing.is_some()` → PUT via the new `update_extra_hour`.
- The `vacation_days` `<option>` is rendered only when `editing.is_none()`. In edit mode the bulk-vacation flow is unreachable and the conditional `from`/`to` fields are dead code we never enter.

Alternative considered: a sibling `EditExtraHoursModal` component. Rejected because it would duplicate the custom-hours loading effect, the parse/identifier helpers, and the form-field markup, and any future change (e.g. a new field) would need to be mirrored.

### Decision 2: API call returns the updated DTO, but caller reloads instead of patching state

`update_extra_hour` returns `Result<ExtraHoursTO, ShiftyError>`. The natural temptation is to patch the in-memory `ExtraHours` list with the returned DTO and skip the round trip. We **do not** do that:

- The employee report (overall hours, balance, vacation_days, custom_extra_hours totals) is recomputed server-side from the underlying entries. Patching only the entry list would leave the aggregate fields stale.
- The reload path (`EmployeeAction::Refresh` → `get_short_employee_report`) already exists and is used after `DeleteExtraHours`. Reusing it keeps the post-mutation invariant identical.
- Round-trip cost is one HTTP call against a local backend; not a concern in this UI.

So the service handler simply: call `update_extra_hour` → on `Ok(_)`, send `EmployeeAction::Refresh` and propagate a saved-event upward; on `Err`, surface via `ErrorView` (existing path) and refresh anyway when the error is a 409.

### Decision 3: 409 conflict → refresh + translated notice via `ErrorView`

The user explicitly chose "refresh" for conflict handling. Implementation:

- `update_extra_hour` distinguishes 409 from other errors by mapping `reqwest::StatusCode::CONFLICT` to a dedicated `ShiftyError::Conflict` variant (or the closest existing variant — to be confirmed when implementing; no new error category should be invented if one fits).
- On 409, the service handler triggers `EmployeeAction::Refresh` so the user sees the up-to-date data, AND pushes a translated message ("Entry was modified elsewhere; please retry") into the existing error-display channel so it appears in `ErrorView`.
- The modal closes on 409 (the `editing` snapshot is now stale; reopening with fresh data is a deliberate user action).

Alternative considered: keep the modal open and re-prefill from the refreshed entry. Rejected for the first iteration: it requires the modal to listen to store changes mid-edit, which is a non-trivial behavior change for a rare race. We can revisit if the conflict happens often in practice.

### Decision 4: Edit-button placement and label inside the entry row

The entry row in `ExtraHoursCategorySection` currently has the shape:

```
[date · description]                           [amount]   [🗑 Delete]
```

The edit button goes between the amount and the delete button:

```
[date · description]                  [amount]   [✎ Edit]   [🗑 Delete]
```

Both buttons use the `Btn` atom; the edit button uses `BtnVariant::Secondary` (delete already uses `Danger`). The edit button is icon-only with an ARIA label from the new `EditExtraHourLabel` i18n key, matching the delete button's compact style.

### Decision 5: Event propagation — `on_edit: EventHandler<ExtraHours>` from row to page

The page owns the `editing_extra_hours: Signal<Option<ExtraHours>>`. The row needs to push the **whole** `ExtraHours` (not just the id) up so the modal can prefill without an extra round trip. The handler chain:

```
ExtraHoursCategorySection
  on_edit (per row click) ──► ExtraHoursViewProps.on_edit
                                ──► EmployeeViewPlainProps.on_edit_extra_hour
                                      ──► EmployeeViewProps.on_edit_extra_hour
                                            ──► employee_details.rs / my_employee_details.rs
                                                  set editing.set(Some(eh))
                                                  set show_extra_hours_dialog.set(true)
```

Each layer adds one prop with the same shape. The pages already manage `show_extra_hours_dialog`, so adding a sibling `editing_extra_hours` signal is a one-line change there.

### Decision 6: Service action mirrors `DeleteExtraHours`

`EmployeeAction` gets a new variant `UpdateExtraHours(ExtraHoursTO)`. The `ExtraHoursTO` is built in the modal at submit time from the current form signals plus the `editing` snapshot (for `id`, `sales_person_id`, `version`). The handler in `service/employee.rs` calls `api::update_extra_hour` then dispatches `Refresh`. This matches the shape of `DeleteExtraHours(Uuid)` exactly — no architectural deviation.

### Decision 7: i18n keys added in all three locales atomically

The CLAUDE.md guidance is explicit: every new translation key must be added to En, De, AND Cs in the same change (a previous bug had German falling back to English). The new keys are:

- `EditExtraHourLabel` — ARIA label / button text. En: "Edit", De: "Bearbeiten", Cs: "Upravit".
- `EditExtraHoursFormTitle` — modal title in edit mode. En: "Edit extra hours entry", De: "Eintrag bearbeiten", Cs: "Upravit záznam".
- `ExtraHoursConflictNotice` — 409 message. En: "Entry was modified elsewhere; the data has been refreshed. Please retry your edit.", De: "Eintrag wurde inzwischen geändert. Die Daten wurden aktualisiert. Bitte erneut versuchen.", Cs: "Záznam byl mezitím změněn. Data byla aktualizována. Zkuste úpravu znovu."

Final wording can be polished during implementation, but the keys must land in all three locales.

## Risks / Trade-offs

- **Risk: Modal conditional logic grows complex.** Adding `editing` toggles title, vacation_days option, submit verb, and initial values. Mitigation: keep the toggle in one obvious place (an `is_edit` boolean derived once at the top of the component) and reference it consistently. Add a unit test covering both modes.

- **Risk: `vacation_days` option still selectable in some race.** A user could open the create modal, switch to vacation_days, then somehow reopen it as edit — leaving the old selection in a stale signal. Mitigation: when the modal opens (`open` transitions false→true), reset signals from `editing` if present. The current code already evaluates initial values when `open` is true; using `use_effect` keyed on `open` + `editing` covers re-open cases.

- **Risk: 409 race feels invisible to the user if `ErrorView` is dismissed too eagerly.** The `ErrorView` is the standard surface; if its dismissal timing is too short the conflict notice might be missed. Mitigation: this is a general `ErrorView` UX concern, not specific to this change. Document the assumption that `ErrorView` shows its messages long enough to read.

- **Risk: Custom extra hours category in edit mode shows a deleted definition.** If a `Custom(uuid)` category was edited and the underlying definition was later deleted, the category select might not contain that uuid. Mitigation: in edit mode, if the current `editing.category` is `Custom(uuid)` and the loaded `custom_extra_hours_definitions` does not include that uuid, render an inert `<option selected disabled>` for it so the form does not silently drop the value. This is a small addition to the existing `option { ... }` rendering loop.

- **Trade-off: Refresh on 409 instead of in-place re-prefill.** Simpler to implement and test; rare in practice. Acceptable for v1; revisit if conflict logs show it happening often.

- **Trade-off: Edit button always visible.** Simpler than checking permissions client-side; relies on backend to reject 403. Cost is users without permission see a button that fails — but in this app the only relevant cases are HR-on-others (allowed) and self-on-self (allowed), so the practical conflict surface is empty.

## Migration Plan

1. Ship the API + service + modal + view + page changes in one PR. They are mutually dependent (modal cannot submit without API; rows cannot trigger without service action; pages cannot wire signals without props).
2. Verify `cargo test` passes for the new modal mode test, the new edit-button render test, and the service test.
3. Manually exercise create + edit + delete in a dev session against a backend with `extra-hours-update` deployed. Confirm 409 path by editing the same entry in two browser tabs.
4. **Rollback**: revert the frontend PR. The backend is unaffected and continues to expose `PUT /extra-hours/{id}` for any other client.
