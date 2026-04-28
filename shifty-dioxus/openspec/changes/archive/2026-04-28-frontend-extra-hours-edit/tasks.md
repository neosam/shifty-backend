## 1. i18n Keys

- [x] 1.1 Add `EditExtraHourLabel`, `EditExtraHoursFormTitle`, `ExtraHoursConflictNotice` to the `Key` enum in `src/i18n/mod.rs`
- [x] 1.2 Add English translations in `src/i18n/en.rs` (`Edit`, `Edit extra hours entry`, `Entry was modified elsewhere; the data has been refreshed. Please retry your edit.`)
- [x] 1.3 Add German translations in `src/i18n/de.rs` (`Bearbeiten`, `Eintrag bearbeiten`, `Eintrag wurde inzwischen geändert. Die Daten wurden aktualisiert. Bitte erneut versuchen.`)
- [x] 1.4 Add Czech translations in `src/i18n/cs.rs` (`Upravit`, `Upravit záznam`, `Záznam byl mezitím změněn. Data byla aktualizována. Zkuste úpravu znovu.`)
- [x] 1.5 Verify with `cargo check` that all locale modules compile and no key is missing

## 2. API Layer

- [x] 2.1 In `src/api.rs`, add `pub async fn update_extra_hour(config: Config, extra_hours: ExtraHoursTO) -> Result<ExtraHoursTO, ShiftyError>` that issues `PUT {backend}/extra-hours/{extra_hours.id}` with the DTO as JSON body
- [x] 2.2 Distinguish 409 Conflict from other errors: map `reqwest::StatusCode::CONFLICT` to a dedicated `ShiftyError` variant (use existing `Conflict`/`OptimisticLock`/`EntityConflict` if present; otherwise add the smallest necessary variant in `src/error.rs`)
- [x] 2.3 On success, parse the response body as `ExtraHoursTO` and return it
- [x] 2.4 On any other non-success status, propagate via `response.error_for_status()?` so the existing `ShiftyError` mapping handles it

## 3. Service Layer

- [x] 3.1 In `src/service/employee.rs`, add `EmployeeAction::UpdateExtraHours(ExtraHoursTO)` to the `EmployeeAction` enum
- [x] 3.2 Add an async helper `update_extra_hours(extra_hours: ExtraHoursTO) -> Result<(), ShiftyError>` that calls `api::update_extra_hour` and returns `Ok(())` on success
- [x] 3.3 In the employee coroutine match arm, handle `UpdateExtraHours(eh)`: call the helper; on `Ok` send `EmployeeAction::Refresh`; on `Err` matching the conflict variant, send `EmployeeAction::Refresh` AND push the `ExtraHoursConflictNotice` translation into the existing error channel; on other errors, propagate via the existing error channel without forcing a refresh

## 4. Modal — Dual Mode (Create + Edit)

- [x] 4.1 In `src/component/extra_hours_modal.rs`, extend `ExtraHoursModalProps` with `pub editing: Option<ExtraHours>` (default `None`)
- [x] 4.2 At the top of the component, derive `let is_edit = props.editing.is_some();`
- [x] 4.3 Initialize the `category`, `amount`, `description`, `when` signals from `props.editing` if present, otherwise from the existing defaults
- [x] 4.4 Use a `last_editing_key` signal (rather than `use_effect`) to re-seed signals when `props.editing.id` changes — same effect, simpler hook surface, matches the existing `last_loaded_id` pattern in `employee_details.rs`
- [x] 4.5 Switch the dialog title between `Key::AddExtraHoursFormTitle` (create) and `Key::EditExtraHoursFormTitle` (edit) based on `is_edit`
- [x] 4.6 In the category `<select>`, render the `vacation_days` `<option>` only when `!is_edit`
- [x] 4.7 If `is_edit` and the editing entry's category is `Custom(uuid)` not present in `custom_extra_hours.read()`, render an extra `<option selected>` for that uuid so the value round-trips
- [x] 4.8 In the submit branch: when `is_edit`, build an `ExtraHoursTO` carrying `id`, `sales_person_id`, `version` from `props.editing` plus the form values, and dispatch `EmployeeAction::UpdateExtraHours(to)` instead of calling `add_extra_hour`
- [x] 4.9 Keep create-mode behavior unchanged (POST via `add_extra_hour`)
- [x] 4.10 Always call `props.on_saved` on a successful submit (both modes) so the page closes the dialog

## 5. List — Per-Row Edit Button

- [x] 5.1 In `src/component/employee_view.rs`, add `pub on_edit: EventHandler<ExtraHours>` to `ExtraHoursViewProps`
- [x] 5.2 Add `pub on_extra_hour_edit: EventHandler<ExtraHours>` to `EmployeeViewPlainProps` and `EmployeeViewProps`, propagated from `ExtraHoursView` up
- [x] 5.3 Add `pub on_edit: EventHandler<ExtraHours>` to `ExtraHoursCategorySectionProps`
- [x] 5.4 In the entry row inside `ExtraHoursCategorySection`, add a `Btn { variant: BtnVariant::Secondary, ... }` between the amount span and the delete button, labeled with `Key::EditExtraHourLabel`
- [x] 5.5 Wire the edit button's `on_click` to `props.on_edit.call(entry.clone())`

## 6. Page Wiring (HR view)

- [x] 6.1 In `src/page/employee_details.rs`, add `let mut editing_extra_hours = use_signal(|| None::<ExtraHours>);`
- [x] 6.2 Add `EmployeeDetailsAction::OpenEditExtraHours(ExtraHours)` to the action enum and a coroutine arm that sets `editing_extra_hours.set(Some(eh))` and `show_extra_hours_dialog.set(true)`
- [x] 6.3 Update `EmployeeDetailsAction::OpenExtraHours` (existing) to also set `editing_extra_hours.set(None)` so re-opening for create clears any stale edit context
- [x] 6.4 Update `EmployeeDetailsAction::ExtraHoursSaved` and `CloseExtraHours` to also clear `editing_extra_hours.set(None)`
- [x] 6.5 Pass `editing: editing_extra_hours.read().clone()` into the `ExtraHoursModal` element
- [x] 6.6 Pass `on_extra_hour_edit: move |eh| cr.send(EmployeeDetailsAction::OpenEditExtraHours(eh))` into `EmployeeView`

## 7. Page Wiring (Self view)

- [x] 7.1 In `src/page/my_employee_details.rs`, replicate the same `editing_extra_hours` signal, `OpenEditExtraHours` action, related state-clearing on close/save, modal `editing` prop, and `on_extra_hour_edit` handler as in 6.1–6.6

## 8. Tests

- [x] 8.1 Render test in modal test module: opens with `editing = Some(ExtraWork)`, asserts prefilled description/amount and absent `vacation_days` option
- [x] 8.2 Render test: `editing = None` keeps `vacation_days` option (regression guard for create mode)
- [x] 8.3 Render test: `editing = Some(Custom(uuid))` with uuid not in known definitions — preserves the orphan custom option as selected
- [x] 8.4 Render test for `ExtraHoursCategorySection` asserting the edit button carries the `EditExtraHourLabel` translation
- [x] 8.5 Service-level: pure unit test for `build_update_payload` (identity carried from snapshot, editable fields from form). End-to-end coroutine handler intentionally not unit-tested — gap documented in the test module: no API mocking layer exists in this crate, and the dispatch wiring is covered by the modal SSR tests + manual verification in §9.5

## 9. Verification

- [x] 9.1 Run `cargo fmt`
- [x] 9.2 Run `cargo clippy` — no new warnings introduced by this change (the only `deref` hint near the new edit-mode submit branch is a stylistic note that already applies to the existing create-mode `&*from.read()` / `&*to.read()` calls and is not regression)
- [x] 9.3 Run `cargo check`
- [x] 9.4 Run `cargo test` — 460 passed, 0 failed
- [x] 9.5 Manual browser exercise (Chromium via Playwright, 2026-04-28):
  - Edit button visible on every entry row (Extra work, Volunteer Work)
  - Modal opens in edit mode with title "Edit extra hours entry", category preselected, description / amount / when prefilled, `vacation_days` option absent
  - Edit + Submit issues `PUT /extra-hours/{logical-id}` with the form values (`amount=15.5`, `description="edited via chrome"`, carried `$version`)
  - On success: list refreshes, entry shows new values, aggregate (Extra work, Balance, Actual) recomputed correctly
  - 409 path verified: when the modal is reopened after a successful save, submitting with the old in-memory `$version` returns 409 → translated `ExtraHoursConflictNotice` displays in `ErrorView`, employee data refreshes
- [x] 9.6 Run `openspec validate frontend-extra-hours-edit --strict` — change is valid
