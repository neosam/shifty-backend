## Why

Some sales persons have a genuine employment contract for a small number of hours per week (e.g. 5h) but additionally wish to contribute unpaid, volunteer hours on top — they help beyond their contractual obligation without expecting the extra time to accrue as overtime on their balance account.

The current system treats every booked shiftplan hour identically: hours booked beyond `expected_hours` automatically produce a positive balance (overtime credit). There is no way to express "up to contract = payable, beyond that = volunteer". As a consequence:

- Shift planners cannot cleanly separate payable shiftplan hours from volunteer contribution.
- The weekly planning report mixes paid and volunteer work into a single overtime number.
- There is no hour category that documents performed work in a balance-neutral way (existing `UnpaidLeave` models absence, not volunteer presence).

The capping behaviour must be opt-in per employee work detail record; the default behaviour (overtime credit for hours beyond expected) must remain unchanged for all existing users.

## What Changes

- Introduce a new boolean field `cap_planned_hours_to_expected` on `EmployeeWorkDetails` (default `false`, time-versioned along with existing from/to range).
- Introduce a new `ExtraHoursCategory` variant `VolunteerWork` representing documented work that is balance-neutral (neither counted as paid work nor as overtime).
- Introduce a new `ReportType` variant for balance-neutral hour entries (existing variants `WorkingHours`, `ExtraWork`, `AbsenceHours` all affect balance — none fits volunteer semantics).
- In the weekly report aggregation (`hours_per_week` in `reporting.rs`): when the active work details for a week have the cap flag set and weekly shiftplan hours exceed `expected_hours`, the overflow is reported as volunteer hours and excluded from the balance calculation. `ExtraHours` entries (overtime, sick leave, vacation, etc.) are never capped.
- The cap is one-sided: it limits payable hours upward but does not compensate downward. If a person with the cap flag plans **fewer** shiftplan hours than `expected_hours` in a given week, the resulting negative balance is intentional. Compensation, when desired, is performed manually by a shift planner through an `ExtraWork` entry — no automatic carryover, no balancing across weeks.
- Hybrid model for the new category: volunteer hours can be **automatically derived** from the cap logic *and* be **manually entered** as an `ExtraHours` record (analogous to how overtime can be both planned and manually added).
- Extend report transport objects with a dedicated `volunteer_hours` figure and make volunteer hours visible per week in the weekly planner report.

## Capabilities

### New Capabilities

- `volunteer-work-hours`: New `ExtraHoursCategory` representing performed work that is documented on the employee but neutral with respect to balance calculations. Manually enterable like other extra-hour categories.
- `weekly-planned-hours-cap`: Opt-in per-`EmployeeWorkDetails` flag that caps the payable shiftplan hours at `expected_hours` on a strict per-week basis. Shiftplan hours beyond the cap are attributed to the volunteer category and do not contribute to balance.

### Modified Capabilities

*(none — this change only adds behaviour behind an opt-in flag; default behaviour is preserved)*

## Impact

- **Database**: Migration adds `cap_planned_hours_to_expected INTEGER NOT NULL DEFAULT 0` to the `employee_work_details` table. The `extra_hours.category` TEXT column gains a new recognised value `"VolunteerWork"`.
- **DAO layer**: `EmployeeWorkDetailsEntity` extended with the new boolean field; `ExtraHoursCategoryEntity` extended with a new variant; SQLite mapping updated in `dao_impl_sqlite`.
- **Service layer**: `ExtraHoursCategory::as_report_type()` and `availability()` handle the new variant; new `ReportType` variant for balance-neutral entries; `hours_per_week()` and the per-week aggregation paths in `reporting.rs` apply the cap when the flag is set; `EmployeeWorkDetails` service type updated.
- **REST layer**: `EmployeeWorkDetailsTO` gains the new field; report transport objects (`EmployeeReportTO`, `WorkingHoursReportTO`, `ShortEmployeeReportTO`, `ExtraHoursReportCategoryTO`) expose volunteer hours; OpenAPI schemas regenerated.
- **i18n**: None in this backend change. The backend exposes `VolunteerWork` as an enum string via the API; user-facing labels for the three locales (En, De, Cs) are owned by `shifty-dioxus` and added in the same frontend follow-up that ships the UI controls.
- **Frontend (out of scope for this change)**: Dioxus UI will later need a toggle for the cap flag on the work details form and a display column for volunteer hours in the weekly report; manual entry form for the new category.

## Open Design Questions

These are intentionally deferred to `design.md` once the proposal is accepted. They do not affect whether the change should happen, only how it is implemented.

1. **Naming of the balance-neutral `ReportType` variant.** Candidates: `Documented`, `Recorded`, `BalanceNeutral`, `Informational`. To be decided during design.
2. **Interaction with existing `ExtraHoursCategory` filters and aggregations.** Should volunteer hours appear in generic "extra hours" lists the same way as overtime, or be segregated into their own section in report outputs?
3. **Display of volunteer hours when the cap flag is not set.** If a shift planner manually enters a `VolunteerWork` extra hours record for a person *without* the cap flag, should it simply be balance-neutral like any other volunteer entry, or should there be validation preventing this combination?
