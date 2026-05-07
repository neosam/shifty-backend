# extra-hours-edit-frontend Specification

## Purpose
TBD - created by archiving change frontend-extra-hours-edit. Update Purpose after archive.
## Requirements
### Requirement: Per-entry edit affordance in the extra hours list

The extra hours list inside the employee view SHALL render an edit button for every entry shown, alongside the existing delete button. Activating the edit button SHALL open the extra hours dialog prefilled with the entry's current values, in edit mode.

The edit affordance SHALL be present on both the HR-facing employee detail page and the user's own "My Employee Details" page, without any client-side permission gating — the backend is the source of truth for who may edit what.

#### Scenario: Edit button appears next to delete button for every entry
- **WHEN** the employee view renders an extra hours category section with one or more entries
- **THEN** each entry row displays an edit button between the amount and the delete button
- **AND** the edit button has an accessible label sourced from the `EditExtraHourLabel` translation key

#### Scenario: Clicking the edit button opens the dialog in edit mode
- **WHEN** the user clicks the edit button on an entry
- **THEN** the extra hours dialog opens
- **AND** the dialog is in edit mode (carries the clicked entry as `editing`)
- **AND** the dialog title reads the value of the `EditExtraHoursFormTitle` translation key

#### Scenario: Edit affordance present on both the HR and self pages
- **WHEN** the HR employee detail page renders extra hours for a sales person
- **THEN** the edit affordance is shown
- **WHEN** the user's own "My Employee Details" page renders their extra hours
- **THEN** the edit affordance is shown

### Requirement: Dialog runs in two modes (create and edit) under one component

The `ExtraHoursModal` component SHALL accept an optional `editing: Option<ExtraHours>` input that switches its mode. When `editing` is `None` the dialog behaves exactly as it does today (create mode, POST). When `editing` is `Some(entry)` the dialog runs in edit mode (prefill, PUT). The same component SHALL handle both modes; no separate edit-modal component is introduced.

#### Scenario: Create mode preserved when editing is None
- **WHEN** the dialog is opened with `editing = None`
- **THEN** the form fields start at their default values (current behavior)
- **AND** submitting issues `POST /extra-hours` via the existing `add_extra_hour` API

#### Scenario: Edit mode prefills from the editing entry
- **WHEN** the dialog is opened with `editing = Some(entry)`
- **THEN** the category field is prefilled to `entry.category`
- **AND** the description field is prefilled to `entry.description`
- **AND** the amount field is prefilled to `entry.amount`
- **AND** the date/time field is prefilled to `entry.date_time`

#### Scenario: Re-opening after a previous edit resets prefill
- **WHEN** the dialog was previously open in edit mode for entry A and then closed
- **AND** the dialog is then opened in edit mode for entry B
- **THEN** the form fields reflect entry B's values, not entry A's stale values

#### Scenario: Re-opening from edit to create resets prefill
- **WHEN** the dialog was previously open in edit mode for some entry and then closed
- **AND** the dialog is then opened in create mode (`editing = None`)
- **THEN** the form fields show the create-mode defaults, not the previous entry's values

### Requirement: Edit mode hides VacationDays and shows the hours UI variant

In edit mode the dialog SHALL hide the `vacation_days` option from the category select. Individual `Vacation` entries (the per-day rows produced by a previous `VacationDays` bulk-create) remain editable like any other entry — they appear in the list under the Vacation category and use the standard amount + date/time form fields.

#### Scenario: vacation_days option absent in edit mode
- **WHEN** the dialog is opened in edit mode
- **THEN** the category select does not include the `vacation_days` option

#### Scenario: vacation_days option present in create mode
- **WHEN** the dialog is opened in create mode
- **THEN** the category select includes the `vacation_days` option (current behavior preserved)

#### Scenario: Editing a Vacation entry uses the standard hours UI
- **WHEN** a `Vacation` entry is opened for editing
- **THEN** the dialog shows the amount and date/time fields (not the from/to range fields)
- **AND** the user may modify amount, date/time, description, and category through the same fields used for any other category

### Requirement: Edit mode allows changing the category, with custom-category resilience

Edit mode SHALL allow the user to change the entry's category to any of the categories available in create mode except `vacation_days`. If the entry being edited has a `Custom(uuid)` category whose definition is no longer present in the loaded `custom_extra_hours_definitions` (e.g. the definition was deleted after the entry was created), the dialog SHALL still display the current category so that an unintended PUT does not silently rewrite it.

#### Scenario: User can switch from one category to another
- **WHEN** an entry with category `ExtraWork` is opened for editing
- **AND** the user selects `Holiday` from the category select
- **AND** the user submits
- **THEN** the PUT body's category is `Holiday`

#### Scenario: Stale custom category remains selected
- **WHEN** an entry with category `Custom(uuid X)` is opened for editing
- **AND** the loaded custom-extra-hours definitions do not include uuid X
- **THEN** the category select still indicates `Custom(uuid X)` as the selected value
- **AND** the user can still submit without the form silently switching the category to a default

### Requirement: Submit in edit mode issues PUT with optimistic-lock version

Submitting the dialog in edit mode SHALL call `update_extra_hour` (a new function on the API client) which issues `PUT /extra-hours/{id}` with the full `ExtraHoursTO`. The `ExtraHoursTO` SHALL carry the `id`, `sales_person_id`, and `version` from the original `editing` snapshot — these fields are not user-editable in the dialog. The body's `amount`, `category`, `description`, and `date_time` SHALL come from the form signals.

#### Scenario: PUT body uses snapshot identity and form values
- **WHEN** the dialog is in edit mode for an entry with id `X`, sales_person_id `S`, version `V`, and the user has edited the form fields
- **AND** the user submits
- **THEN** a `PUT /extra-hours/X` request is issued
- **AND** the request body has `id = X`, `sales_person_id = S`, `$version = V`
- **AND** the request body's `amount`, `category`, `description`, `date_time` reflect the current form values

#### Scenario: Successful PUT closes the dialog and refreshes the employee data
- **WHEN** the PUT returns success
- **THEN** the dialog closes
- **AND** the employee data is reloaded so the list reflects the new active row

### Requirement: 409 Conflict triggers refresh and a translated user notice

If the PUT returns `409 Conflict` (the optimistic-lock version is stale) the frontend SHALL:

1. Trigger a refresh of the employee data so the list reflects the current server state.
2. Surface a translated, user-visible notice through the existing `ErrorView` channel, sourced from the `ExtraHoursConflictNotice` translation key.
3. Close the dialog (the in-memory `editing` snapshot is now stale; reopening with fresh data is the user's next action).

The frontend SHALL NOT silently retry the PUT.

#### Scenario: 409 refreshes the data
- **WHEN** the PUT returns 409
- **THEN** the employee data is reloaded

#### Scenario: 409 shows a translated notice
- **WHEN** the PUT returns 409
- **THEN** the `ErrorView` displays a message sourced from the `ExtraHoursConflictNotice` translation key in the active locale

#### Scenario: 409 closes the dialog
- **WHEN** the PUT returns 409
- **THEN** the dialog closes

#### Scenario: 409 does not auto-retry
- **WHEN** the PUT returns 409
- **THEN** no second PUT is issued automatically

### Requirement: Edit failures other than 409 surface via ErrorView and do not refresh blindly

For PUT failures other than 409 (e.g. 403 Forbidden, 400 Bad Request, network error), the frontend SHALL surface the error through the existing `ErrorView` channel. The dialog SHALL remain open so the user can adjust their input or close it manually. The employee data SHALL NOT be force-refreshed for these errors (no need to discard the user's edits unless a conflict requires it).

#### Scenario: 403 Forbidden surfaces an error and keeps the dialog open
- **WHEN** the PUT returns 403
- **THEN** the `ErrorView` displays the error
- **AND** the dialog remains open with the user's current form values intact

#### Scenario: Network failure surfaces an error and keeps the dialog open
- **WHEN** the PUT fails with a network-level error
- **THEN** the `ErrorView` displays the error
- **AND** the dialog remains open

### Requirement: i18n keys are added in all three locales atomically

All new translation keys introduced by this change SHALL be added to all three locale modules (`src/i18n/en.rs`, `src/i18n/de.rs`, `src/i18n/cs.rs`) in the same change. The keys are: `EditExtraHourLabel`, `EditExtraHoursFormTitle`, `ExtraHoursConflictNotice`.

#### Scenario: Each new key has a translation in every locale
- **WHEN** the change is shipped
- **THEN** `EditExtraHourLabel`, `EditExtraHoursFormTitle`, and `ExtraHoursConflictNotice` each return a non-empty translated string in En, De, and Cs

#### Scenario: No locale falls back due to a missing key
- **WHEN** the dialog is opened in edit mode under any of the three locales
- **THEN** the dialog title is the De / En / Cs translation of `EditExtraHoursFormTitle` respectively, never an English fallback when the locale is De or Cs

