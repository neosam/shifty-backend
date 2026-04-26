## ADDED Requirements

### Requirement: Cap Flag on EmployeeWorkDetails

The `EmployeeWorkDetails` entity SHALL include a boolean field `cap_planned_hours_to_expected` with a default value of `false`. The field SHALL participate in the existing time-versioning of `EmployeeWorkDetails` (it is part of the record identified by the same `from_year/from_calendar_week … to_year/to_calendar_week` range as the rest of the record's fields).

#### Scenario: Newly created EmployeeWorkDetails defaults the flag to false

- **WHEN** an `EmployeeWorkDetails` record is created without an explicit `cap_planned_hours_to_expected` value
- **THEN** the persisted record has `cap_planned_hours_to_expected = false`

#### Scenario: Pre-existing records receive the default after migration

- **WHEN** the schema migration that introduces the column is applied to a database that already contains `employee_work_details` rows
- **THEN** every pre-existing row has `cap_planned_hours_to_expected = false`

### Requirement: Weekly Cap Attributes Overflow to Volunteer Hours

When the active EmployeeWorkDetails record for a given week has `cap_planned_hours_to_expected = true` and the sum of shiftplan booking hours for that week exceeds `expected_hours` for that week, the report aggregation SHALL:

- treat only `expected_hours` of the booking sum as `shiftplan_hours` for the balance calculation, and
- attribute the overflow (`shiftplan_sum − expected_hours`) to the per-week `volunteer_hours` figure as auto-attributed hours (in addition to any manual `VolunteerWork` extra-hours records in the same week).

The overflow SHALL NOT contribute to `overall_hours` or `balance_hours`.

#### Scenario: Bookings exceed expected — overflow becomes volunteer hours

- **GIVEN** an `EmployeeWorkDetails` record with `expected_hours = 5` and `cap_planned_hours_to_expected = true`, active for week W
- **AND** the sales person has `10h` of shiftplan bookings in week W and no extra-hours records
- **WHEN** the weekly report is computed for week W
- **THEN** `shiftplan_hours` for the balance calculation in week W is `5`
- **AND** `volunteer_hours` for week W is `5` (auto-attributed)
- **AND** `balance_hours` for week W is `0`

#### Scenario: Auto-attributed and manual volunteer hours combine

- **GIVEN** an `EmployeeWorkDetails` record with `expected_hours = 5` and `cap_planned_hours_to_expected = true`, active for week W
- **AND** the sales person has `10h` of shiftplan bookings in week W
- **AND** the sales person has a `2h` `VolunteerWork` extra-hours record dated in week W
- **WHEN** the weekly report is computed for week W
- **THEN** `volunteer_hours` for week W is `7` (`5` auto-attributed plus `2` manual)
- **AND** `balance_hours` for week W is `0`

### Requirement: Cap is One-Sided — No Compensation Below Expected

When the active EmployeeWorkDetails for a week has the cap flag set and the sum of shiftplan booking hours for that week is **less than** `expected_hours`, the report aggregation SHALL NOT pad upward, attribute negative volunteer hours, or otherwise compensate. The resulting negative balance is intentional.

#### Scenario: Bookings below expected produce negative balance

- **GIVEN** an `EmployeeWorkDetails` record with `expected_hours = 5` and `cap_planned_hours_to_expected = true`, active for week W
- **AND** the sales person has `3h` of shiftplan bookings in week W and no extra-hours records
- **WHEN** the weekly report is computed for week W
- **THEN** `shiftplan_hours` for week W is `3`
- **AND** `volunteer_hours` auto-attributed for week W is `0`
- **AND** `balance_hours` for week W is `−2`

### Requirement: Cap Does Not Affect Extra Hours

`ExtraHours` records (any category — `ExtraWork`, `Vacation`, `SickLeave`, `Holiday`, `Unavailable`, `UnpaidLeave`, `VolunteerWork`, `Custom`) SHALL be unaffected by the cap mechanism. Their contribution to `overall_hours`, `expected_hours`, and `balance_hours` SHALL be identical regardless of the cap flag's value.

#### Scenario: ExtraWork remains credited under cap

- **GIVEN** an `EmployeeWorkDetails` record with `expected_hours = 5` and `cap_planned_hours_to_expected = true`, active for week W
- **AND** the sales person has `5h` of shiftplan bookings and a `3h` `ExtraWork` extra-hours record in week W
- **WHEN** the weekly report is computed for week W
- **THEN** `overall_hours` for week W is `8` (5 capped shiftplan + 3 extra work)
- **AND** `balance_hours` for week W is `+3` (the extra work is credited as overtime)
- **AND** `volunteer_hours` auto-attributed for week W is `0`

### Requirement: Cap Inactive When Flag is False — Existing Behaviour Preserved

When the active EmployeeWorkDetails for a week has `cap_planned_hours_to_expected = false`, the report aggregation SHALL behave identically to the pre-change behaviour: shiftplan hours contribute to `overall_hours` in full, and no auto-attribution to volunteer hours occurs.

#### Scenario: Default flag preserves overtime crediting

- **GIVEN** an `EmployeeWorkDetails` record with `expected_hours = 20` and `cap_planned_hours_to_expected = false`, active for week W
- **AND** the sales person has `25h` of shiftplan bookings in week W
- **WHEN** the weekly report is computed for week W
- **THEN** `shiftplan_hours` for week W is `25` (full amount)
- **AND** `balance_hours` for week W is `+5` (overtime credited as today)
- **AND** `volunteer_hours` auto-attributed for week W is `0`

### Requirement: Cap Evaluation is Time-Versioned per EmployeeWorkDetails Record

The cap behaviour for any given week SHALL be determined by the EmployeeWorkDetails record active for that specific week. A change to the flag in a later EmployeeWorkDetails record (with a later `from_year/from_calendar_week`) SHALL NOT alter the calculation for weeks covered by an earlier record.

#### Scenario: Cap flag flip between consecutive WorkDetails records

- **GIVEN** EmployeeWorkDetails record A active for weeks 1–10 with `cap_planned_hours_to_expected = false` and `expected_hours = 5`
- **AND** EmployeeWorkDetails record B active for weeks 11–20 with `cap_planned_hours_to_expected = true` and `expected_hours = 5`
- **AND** the sales person has `8h` of shiftplan bookings in week 8 and `8h` of shiftplan bookings in week 12
- **WHEN** the weekly report is computed for both weeks
- **THEN** `balance_hours` for week 8 is `+3` (cap inactive — record A controls)
- **AND** `balance_hours` for week 12 is `0` with `volunteer_hours = 3` auto-attributed (cap active — record B controls)
