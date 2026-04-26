# Volunteer Work Hours

## Purpose

Provides a `VolunteerWork` extra hours category for tracking performed work that is recorded but balance-neutral. Volunteer hours are surfaced as a dedicated figure in employee reports and persisted as a distinct `value_type` in billing period snapshots, so contributions outside of expected working hours can be observed without affecting balance calculations.

## Requirements

### Requirement: VolunteerWork Extra Hours Category

The `ExtraHoursCategoryEntity` enum SHALL include a parameterless variant `VolunteerWork` representing performed work that is recorded but balance-neutral. The variant SHALL be persistable in the SQLite `extra_hours.category` TEXT column using the literal string `"VolunteerWork"`, consistent with the storage convention of other parameterless variants.

#### Scenario: VolunteerWork extra hours record round-trips through persistence

- **WHEN** an extra-hours record with category `VolunteerWork` is created via the DAO
- **AND** the same record is subsequently read back by id
- **THEN** the loaded record carries category `VolunteerWork`

### Requirement: Documented ReportType for Balance-Neutral Hours

The `ReportType` enum SHALL include a variant `Documented`. Hours classified as `Documented` SHALL NOT contribute to `expected_hours`, `overall_hours`, or `balance_hours` in any report aggregation.

#### Scenario: VolunteerWork category maps to Documented report type

- **WHEN** `ExtraHoursCategory::VolunteerWork.as_report_type()` is called
- **THEN** the returned value is `ReportType::Documented`

#### Scenario: Documented hours are excluded from balance calculation

- **GIVEN** a sales person with `expected_hours = 40` for week W
- **AND** `40h` of shiftplan bookings in week W
- **AND** a `5h` `VolunteerWork` extra-hours record dated in week W
- **WHEN** the weekly report is computed for week W
- **THEN** `balance_hours` for week W is `0`
- **AND** `overall_hours` for week W is `40` (the volunteer 5h are excluded)
- **AND** `expected_hours` for week W is `40` (unchanged by the volunteer entry)

### Requirement: VolunteerWork Marks the Person as Available

The `availability()` mapping for `ExtraHoursCategoryEntity::VolunteerWork` SHALL return `Availability::Available`. A volunteer record indicates the person was present and contributing, not absent.

#### Scenario: Volunteer record does not mark the period unavailable

- **WHEN** `ExtraHoursCategoryEntity::VolunteerWork.availability()` is called
- **THEN** the returned value is `Availability::Available`

### Requirement: Manual VolunteerWork Entry is Always Permitted

A shift planner SHALL be able to create a `VolunteerWork` extra-hours record for any sales person, irrespective of whether the active EmployeeWorkDetails record for the affected period has the `cap_planned_hours_to_expected` flag set. The system SHALL NOT reject creation based on that flag.

#### Scenario: Manual VolunteerWork entry succeeds for a non-capped person

- **GIVEN** a sales person whose currently active EmployeeWorkDetails has `cap_planned_hours_to_expected = false`
- **WHEN** a shift planner submits an extra-hours create request with category `VolunteerWork`
- **THEN** the record is persisted successfully

### Requirement: VolunteerWork Appears in Per-Category Report Listings

`ExtraHoursReportCategoryTO` SHALL include a `VolunteerWork` variant. Reports that enumerate per-category extra-hours SHALL surface `VolunteerWork` records under that variant when present.

#### Scenario: Per-category list surfaces volunteer hours

- **GIVEN** a sales person with one `VolunteerWork` extra-hours record in the report period
- **WHEN** an employee report listing per-category extra hours is requested
- **THEN** the response includes a `VolunteerWork` entry in the per-category list with the correct hours value

### Requirement: Volunteer Hours as Dedicated Field in Report Transport Objects

`ShortEmployeeReportTO`, `EmployeeReportTO`, `WorkingHoursReportTO`, and the per-week `GroupedReportHours` structure SHALL each include a dedicated `volunteer_hours: f32` field. The figure SHALL be the sum of all volunteer-hour sources for the corresponding period or week (manual `VolunteerWork` extra-hours records plus any auto-attributed hours produced by the cap mechanism defined in the `weekly-planned-hours-cap` capability).

#### Scenario: Report response exposes volunteer_hours field

- **WHEN** a client retrieves an employee report covering a period that contains volunteer hours
- **THEN** the response body includes a `volunteer_hours` field for the period
- **AND** the per-week breakdown for each week containing volunteer hours includes a `volunteer_hours` figure for that week

### Requirement: Volunteer Hours Persist as a Distinct value_type in Billing Period Snapshots

When a billing period is persisted, volunteer hours for each affected sales person SHALL be stored as a `BillingPeriodSalesPerson` row with a dedicated `value_type` of `"volunteer"`. The persisted value SHALL be the same combined figure exposed on the live report (manual `VolunteerWork` records plus any cap-attributed hours). Sales persons with zero volunteer hours over the period MAY be omitted from the `"volunteer"` `value_type` rows.

#### Scenario: Billing period snapshot contains volunteer rows for affected persons

- **GIVEN** a sales person who accumulated `8h` of volunteer hours (from any combination of manual entries and cap attribution) within a billing period
- **WHEN** the billing period is persisted via `build_and_persist_billing_period_report`
- **THEN** the persisted `billing_period_sales_person` rows for that person include a row with `value_type = "volunteer"` and `value_delta = 8`

#### Scenario: Persisted volunteer rows round-trip through service-layer load

- **GIVEN** a billing period was persisted with `8h` of volunteer hours for a sales person
- **WHEN** the billing period is subsequently loaded via the service layer
- **THEN** the resulting `BillingPeriodSalesPerson.values` map contains the key `BillingPeriodValueType::Volunteer` with `value_delta = 8`
- **AND** no persisted volunteer row is silently dropped during deserialisation

### Requirement: Snapshot Schema Version Bump for the New value_type

This change introduces a new persisted `value_type` (`"volunteer"`) into billing period snapshots and therefore SHALL bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` by one as part of its implementation. New billing period snapshots created after this change ships SHALL carry the bumped version.

#### Scenario: New snapshot carries the bumped version

- **GIVEN** the change has shipped on top of `billing-period-snapshot-versioning`
- **WHEN** a new billing period is created
- **THEN** the persisted `billing_period.snapshot_schema_version` equals the value of `CURRENT_SNAPSHOT_SCHEMA_VERSION` after the bump (which is exactly one greater than the value persisted by `billing-period-snapshot-versioning`)
