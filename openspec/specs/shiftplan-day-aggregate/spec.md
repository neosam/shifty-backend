# shiftplan-day-aggregate Specification

## Purpose
TBD - created by archiving change shiftplan-day-view. Update Purpose after archive.
## Requirements
### Requirement: Query day view across all shiftplans
The system SHALL provide an endpoint `GET /shiftplan-day/{year}/{week}/{day_of_week}` that returns a consolidated view of all shiftplans for a specific day, with slots grouped by shiftplan.

The `day_of_week` path parameter SHALL accept `DayOfWeekTO` string values (`Monday`, `Tuesday`, `Wednesday`, `Thursday`, `Friday`, `Saturday`, `Sunday`).

The response SHALL include all non-deleted shiftplans regardless of their `is_planning` flag.

#### Scenario: Successful day query with multiple plans
- **WHEN** a user requests `GET /shiftplan-day/2026/14/Monday` and there are two shiftplans ("Morning" and "Evening") with slots on Monday
- **THEN** the response status SHALL be 200 and the body SHALL contain a `ShiftplanDayAggregateTO` with `year: 2026`, `calendar_week: 14`, `day_of_week: Monday`, and `plans` containing two entries — one per shiftplan — each with the shiftplan details and its Monday slots with bookings

#### Scenario: Day with no slots
- **WHEN** a user requests a day where no shiftplan has slots (e.g., Sunday with no Sunday slots configured)
- **THEN** the response SHALL contain all shiftplans in the `plans` array, each with an empty `slots` list

#### Scenario: Invalid week number
- **WHEN** a user requests a week number that does not exist (e.g., week 54)
- **THEN** the response SHALL return an appropriate error status

### Requirement: Holiday filtering on day view
The system SHALL exclude all slots for a day that is marked as a holiday via special days, consistent with the existing week view behavior.

#### Scenario: Day is a holiday
- **WHEN** a user requests a day that is marked as a `Holiday` special day
- **THEN** all shiftplans SHALL return empty slot lists for that day

### Requirement: Short day filtering on day view
The system SHALL filter slots on short days so that only slots ending at or before the early closing time are included, consistent with the existing week view behavior.

#### Scenario: Day is a short day
- **WHEN** a user requests a day that is marked as a `ShortDay` with `time_of_day` set to 14:00
- **THEN** only slots with `to <= 14:00` SHALL be included in the response for each shiftplan

### Requirement: Booking assignment with self_added
Each slot in the day view response SHALL include its bookings with the `self_added` field computed, consistent with the existing week view behavior. The `self_added` field SHALL be present only when the requesting user has the `SHIFTPLANNER` privilege.

#### Scenario: Bookings with self_added for shiftplanner
- **WHEN** a user with `SHIFTPLANNER` privilege requests a day view
- **THEN** each booking SHALL include `self_added` indicating whether the booking was created by the assigned sales person themselves

#### Scenario: Bookings without self_added for regular user
- **WHEN** a user without `SHIFTPLANNER` privilege requests a day view
- **THEN** each booking SHALL have `self_added` as `None`

### Requirement: Shared day-building logic
The day-building logic (slot filtering by day, special day application, booking assignment, self_added calculation, time sorting) SHALL be extracted into a shared function used by both `get_shiftplan_week` and `get_shiftplan_day`. The refactoring SHALL NOT change the behavior of the existing week endpoint.

#### Scenario: Week endpoint behavior unchanged after refactoring
- **WHEN** `get_shiftplan_week` is called after the refactoring
- **THEN** the result SHALL be identical to the result before the refactoring for all inputs

