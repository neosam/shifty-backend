## ADDED Requirements

### Requirement: VolunteerWork variant in WorkingHoursCategory

The `WorkingHoursCategory` enum SHALL include a parameterless `VolunteerWork` variant. It MUST support bidirectional conversion with `ExtraHoursCategoryTO::VolunteerWork` and `ExtraHoursReportCategoryTO::VolunteerWork`. It MUST have the identifier `"volunteer_work"`, map to a dedicated i18n key, and be displayable.

#### Scenario: Identifier round-trip

- **WHEN** `WorkingHoursCategory::VolunteerWork.identifier()` is called
- **THEN** it returns `"volunteer_work"`

#### Scenario: From identifier parsing

- **WHEN** `WorkingHoursCategory::from_identifier("volunteer_work")` is called
- **THEN** it returns `WorkingHoursCategory::VolunteerWork`

#### Scenario: Conversion from ExtraHoursCategoryTO

- **WHEN** an `ExtraHoursCategoryTO::VolunteerWork` is converted to `WorkingHoursCategory`
- **THEN** the result is `WorkingHoursCategory::VolunteerWork`

#### Scenario: Conversion to ExtraHoursCategoryTO

- **WHEN** a `WorkingHoursCategory::VolunteerWork` is converted to `ExtraHoursCategoryTO`
- **THEN** the result is `ExtraHoursCategoryTO::VolunteerWork`

#### Scenario: Conversion from ExtraHoursReportCategoryTO

- **WHEN** an `ExtraHoursReportCategoryTO::VolunteerWork` is converted to `WorkingHoursCategory`
- **THEN** the result is `WorkingHoursCategory::VolunteerWork`

### Requirement: is_volunteer_work helper method

The `WorkingHoursCategory` enum MUST provide an `is_volunteer_work()` method that returns `true` only for the `VolunteerWork` variant.

#### Scenario: Positive match

- **WHEN** `is_volunteer_work()` is called on `WorkingHoursCategory::VolunteerWork`
- **THEN** it returns `true`

#### Scenario: Negative match

- **WHEN** `is_volunteer_work()` is called on any other variant (e.g., `ExtraWork`, `Vacation`)
- **THEN** it returns `false`

### Requirement: Volunteer hours tracked in WorkingHours

The `WorkingHours` state struct SHALL include a `volunteer_hours: f32` field. It MUST be populated from `WorkingHoursReportTO.volunteer_hours` when constructed via `From<&WorkingHoursReportTO>`.

#### Scenario: Mapping from WorkingHoursReportTO

- **WHEN** a `WorkingHoursReportTO` with `volunteer_hours: 7.5` is converted to `WorkingHours`
- **THEN** the resulting `WorkingHours.volunteer_hours` equals `7.5`

### Requirement: Volunteer hours tracked in Employee

The `Employee` state struct SHALL include a `volunteer_hours: f32` field. It MUST be populated from `EmployeeReportTO.volunteer_hours` when constructed via `From<&EmployeeReportTO>` and MUST default to `0.0` when constructed via `From<&ShortEmployeeReportTO>`.

#### Scenario: Mapping from EmployeeReportTO

- **WHEN** an `EmployeeReportTO` with `volunteer_hours: 12.0` is converted to `Employee`
- **THEN** the resulting `Employee.volunteer_hours` equals `12.0`

#### Scenario: Default from ShortEmployeeReportTO

- **WHEN** a `ShortEmployeeReportTO` is converted to `Employee`
- **THEN** the resulting `Employee.volunteer_hours` equals `0.0`

### Requirement: Volunteer Work selectable in extra hours form

The add-extra-hours form dropdown SHALL include a "Volunteer Work" option with value `"volunteer_work"`. It MUST be placed near the `"Extra Work"` option, since both represent presence and work performed.

#### Scenario: User selects volunteer work

- **WHEN** a user opens the add-extra-hours form and selects "Volunteer Work" from the category dropdown
- **THEN** the form category is set to `WorkingHoursCategory::VolunteerWork`

#### Scenario: Submitting volunteer work creates an extra-hours record

- **GIVEN** the form has category `VolunteerWork`, a positive amount, a description, and a date
- **WHEN** the user submits the form
- **THEN** the request issued to `api::add_extra_hour` carries `ExtraHoursCategoryTO::VolunteerWork`

### Requirement: Volunteer hours displayed in employee view

The employee view SHALL display `volunteer_hours` as a labelled line item using the existing `TupleView` rendering, both in the per-week working-hours section and in the per-period summary section. The line MUST be rendered unconditionally — including when the value is `0.00` — consistent with how every other category aggregate is rendered.

#### Scenario: Per-week section renders volunteer hours

- **GIVEN** an employee with at least one week containing `volunteer_hours = 5.0`
- **WHEN** the per-week section is rendered for that week
- **THEN** a line item shows the localised "Volunteer Work" label and the value `5.00 <hours-suffix>`

#### Scenario: Per-period section renders volunteer hours when zero

- **GIVEN** an employee whose period total has `volunteer_hours = 0.0`
- **WHEN** the per-period section is rendered
- **THEN** a line item shows the localised "Volunteer Work" label and the value `0.00 <hours-suffix>`

### Requirement: Volunteer value_type label in billing-period details

The billing-period details page SHALL recognise the persisted `value_type` string `"volunteer"` (compared case-insensitively as `"VOLUNTEER"`) and SHALL render its header using the localised `CategoryVolunteerWork` label, consistent with the existing handling of `"BALANCE"`, `"EXPECTED_HOURS"`, and `"OVERALL"`.

#### Scenario: Volunteer row renders with localised header

- **GIVEN** a billing-period sales-person row with `value_type = "volunteer"` and `value_delta = 8.0`
- **WHEN** the billing-period details page is rendered in any locale
- **THEN** the row's header displays the localised "Volunteer Work" label
- **AND** the existing delta / YTD-from / YTD-to / full-year values render unchanged using the standard formatting

### Requirement: i18n translations for the volunteer-work category

The `CategoryVolunteerWork` i18n key SHALL have translations in English, German, and Czech.

#### Scenario: English label

- **WHEN** the locale is `Locale::En`
- **THEN** the `CategoryVolunteerWork` label resolves to "Volunteer Work"

#### Scenario: German label

- **WHEN** the locale is `Locale::De`
- **THEN** the `CategoryVolunteerWork` label resolves to a German translation conveying "Volunteer Work" (suggested: "Ehrenamt")

#### Scenario: Czech label

- **WHEN** the locale is `Locale::Cs`
- **THEN** the `CategoryVolunteerWork` label resolves to a Czech translation conveying "Volunteer Work" (suggested: "Dobrovolnictví")
