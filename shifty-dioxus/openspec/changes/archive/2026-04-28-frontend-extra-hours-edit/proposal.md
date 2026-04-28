## Why

The backend now supports updating extra hours entries via `PUT /extra-hours/{id}` (logical-id pattern with optimistic locking). The Dioxus frontend, however, only knows how to create and delete extra hours — to fix a typo, wrong amount, or wrong date, users must delete the entry and re-create it, which is awkward and loses the logical identity that the backend now preserves.

## What Changes

- Add an "Edit" button next to the existing delete button on every entry row inside the extra hours list (both in the HR-facing employee view and the user's own "My Employee Details" page).
- Reuse and extend the existing `ExtraHoursModal` so that it can run in two modes: **create** (current behavior, POST) and **edit** (new, PUT). An optional `editing` prop carrying the existing `ExtraHours` switches the mode, prefills the form, and changes the submit verb.
- In edit mode, the `vacation_days` category option is hidden from the category select. `VacationDays` is a bulk-create shorthand and never appears as an entry in the list anyway; in-place editing of a single entry uses the regular hours UI variant.
- Add a frontend `update_extra_hour` API call that issues `PUT /extra-hours/{id}` with the full `ExtraHoursTO` (carrying the last-read `$version`).
- Map the backend's `409 Conflict` (stale `$version`) to a refresh of the employee data plus a translated user-facing notice via the existing `ErrorView`. The user is then expected to retry against the refreshed entry.
- Wire a new `EmployeeAction::UpdateExtraHours(ExtraHoursTO)` through the employee service coroutine, mirroring the existing `DeleteExtraHours` path.
- Add i18n keys for the edit-button label, the edit-mode dialog title, and the conflict notice in all three locales (En, De, Cs).

## Capabilities

### New Capabilities
- `extra-hours-edit-frontend`: Frontend lifecycle for editing an existing extra hours entry — entry-row edit affordance, dual-mode modal (create vs. edit), category constraints in edit mode, optimistic-lock conflict handling, and the user/HR permission surface as observed by the UI.

### Modified Capabilities
<!-- None. There is no pre-existing frontend spec covering the extra hours UI; the editing lifecycle is documented for the first time as part of this change. -->

## Impact

- **Frontend code**:
  - `src/api.rs`: add `update_extra_hour(config, ExtraHoursTO) -> Result<ExtraHoursTO, ShiftyError>`.
  - `src/component/extra_hours_modal.rs`: add optional `editing: Option<ExtraHours>` prop, prefill state from it, route submit through PUT when set, hide `vacation_days` option in edit mode, switch the dialog title.
  - `src/component/employee_view.rs`: add edit affordance to `ExtraHoursCategorySection` (per-entry button) and a new `on_edit: EventHandler<ExtraHours>` prop on `ExtraHoursViewProps` / `EmployeeViewProps` propagated up.
  - `src/page/employee_details.rs` and `src/page/my_employee_details.rs`: add an `editing_extra_hours: Signal<Option<ExtraHours>>` plus `OpenEditExtraHours(ExtraHours)` and `CloseExtraHours` actions; pass the signal into `ExtraHoursModal`.
  - `src/service/employee.rs`: add `EmployeeAction::UpdateExtraHours(ExtraHoursTO)` and the corresponding async handler that calls the new API and re-loads on success or conflict.
  - `src/i18n/mod.rs`, `src/i18n/{en,de,cs}.rs`: new `EditExtraHourLabel`, `EditExtraHoursFormTitle`, `ExtraHoursConflictNotice` keys with translations in all three locales.
- **Tests**: extend `extra_hours_modal.rs` unit tests for the `editing` mode (prefill + absence of `vacation_days` option), add a render test for the edit button in `employee_view.rs`, and a service-level test for `UpdateExtraHours` (success + 409 → refresh) where the existing test scaffolding allows.
- **Out of scope**: history view of past versions, bulk edit, vacation-range edit, `CustomExtraHours` definition editing (already exists via separate flow).
- **Backend**: no changes — relies on already-shipped `PUT /extra-hours/{id}` from the `extra-hours-update` change in `shifty-backend`.
- **Permissions**: relies on backend's existing rule (`HR_PRIVILEGE` OR own `sales_person_id`); the UI shows the edit affordance unconditionally in both pages and trusts the backend to reject unauthorized edits with 403, surfaced via `ErrorView`.
