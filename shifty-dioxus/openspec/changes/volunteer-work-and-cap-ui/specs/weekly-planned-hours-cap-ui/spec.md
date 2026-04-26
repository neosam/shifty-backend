## ADDED Requirements

### Requirement: cap_planned_hours_to_expected field on EmployeeWorkDetails state

The frontend `EmployeeWorkDetails` state struct SHALL include a `cap_planned_hours_to_expected: bool` field. The field MUST round-trip through both `TryFrom<&EmployeeWorkDetailsTO>` and `TryFrom<&EmployeeWorkDetails> for EmployeeWorkDetailsTO`. The default value produced by `EmployeeWorkDetails::blank_standard()` MUST be `false`.

#### Scenario: Default for newly initialised work details

- **WHEN** `EmployeeWorkDetails::blank_standard(<sales_person_id>)` is called
- **THEN** the resulting struct has `cap_planned_hours_to_expected == false`

#### Scenario: Conversion preserves the flag from TO to state

- **GIVEN** an `EmployeeWorkDetailsTO` with `cap_planned_hours_to_expected = true`
- **WHEN** it is converted to `EmployeeWorkDetails` via `TryFrom`
- **THEN** the resulting `EmployeeWorkDetails.cap_planned_hours_to_expected` equals `true`

#### Scenario: Conversion preserves the flag from state to TO

- **GIVEN** an `EmployeeWorkDetails` with `cap_planned_hours_to_expected = true`
- **WHEN** it is converted to `EmployeeWorkDetailsTO` via `TryFrom`
- **THEN** the resulting `EmployeeWorkDetailsTO.cap_planned_hours_to_expected` equals `true`

### Requirement: Cap flag toggle in the work-details form

The `EmployeeWorkDetailsForm` component SHALL expose `cap_planned_hours_to_expected` as a `Checkbox` control with a localised label and a localised helper text. The control MUST be editable when the form mode is `New` or `Edit` and MUST be disabled when the form mode is `ReadOnly`. Changing the checkbox MUST emit the updated `EmployeeWorkDetails` via the `on_update_employee_work_details` event handler in the same shape as every other field in the form.

#### Scenario: Cap toggle visible in New mode

- **WHEN** the form is rendered with `EmployeeWorkDetailsFormType::New`
- **THEN** the cap checkbox is rendered enabled
- **AND** the localised helper text is shown alongside it

#### Scenario: Cap toggle disabled in ReadOnly mode

- **WHEN** the form is rendered with `EmployeeWorkDetailsFormType::ReadOnly`
- **THEN** the cap checkbox is rendered disabled
- **AND** any user attempt to toggle it does not emit an update event

#### Scenario: Toggling the checkbox emits an updated record

- **GIVEN** the form is in `Edit` mode with `cap_planned_hours_to_expected = false`
- **WHEN** the user clicks the checkbox
- **THEN** the `on_update_employee_work_details` handler is called with an `EmployeeWorkDetails` carrying `cap_planned_hours_to_expected = true`
- **AND** all other fields in the emitted record match the prior state unchanged

### Requirement: i18n keys for the cap flag in all supported locales

The i18n key set SHALL include `CapPlannedHoursLabel` (the form label) and `CapPlannedHoursHelp` (the helper text). Both keys MUST have translations in English, German, and Czech.

#### Scenario: English label and help

- **WHEN** the locale is `Locale::En`
- **THEN** `CapPlannedHoursLabel` resolves to "Cap planned hours at expected"
- **AND** `CapPlannedHoursHelp` resolves to a sentence describing that hours beyond expected are recorded as volunteer work and do not affect the balance

#### Scenario: German label and help

- **WHEN** the locale is `Locale::De`
- **THEN** `CapPlannedHoursLabel` resolves to a German translation conveying "Cap planned hours at expected"
- **AND** `CapPlannedHoursHelp` resolves to a German sentence describing the volunteer-work behaviour

#### Scenario: Czech label and help

- **WHEN** the locale is `Locale::Cs`
- **THEN** `CapPlannedHoursLabel` resolves to a Czech translation conveying "Cap planned hours at expected"
- **AND** `CapPlannedHoursHelp` resolves to a Czech sentence describing the volunteer-work behaviour
