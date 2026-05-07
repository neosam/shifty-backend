## 1. Scaffolding — State Layer

- [x] 1.1 Add `pub cap_planned_hours_to_expected: bool` field to `EmployeeWorkDetails` in `src/state/employee_work_details.rs`
- [x] 1.2 Default `cap_planned_hours_to_expected` to `false` in `EmployeeWorkDetails::blank_standard()`
- [x] 1.3 Wire `cap_planned_hours_to_expected` through `TryFrom<&EmployeeWorkDetailsTO>` and `TryFrom<&EmployeeWorkDetails> for EmployeeWorkDetailsTO`
- [x] 1.4 Add a parameterless `VolunteerWork` variant to `WorkingHoursCategory` in `src/state/employee.rs`
- [x] 1.5 Add `is_volunteer_work()` helper to `WorkingHoursCategory`
- [x] 1.6 Extend `WorkingHoursCategory::identifier()` with the arm `VolunteerWork => "volunteer_work".into()`
- [x] 1.7 Extend `WorkingHoursCategory::from_identifier()` with the arm `"volunteer_work" => WorkingHoursCategory::VolunteerWork`
- [x] 1.8 Extend `WorkingHoursCategory::to_i18n_key()` with the arm `VolunteerWork => i18n::Key::CategoryVolunteerWork`
- [x] 1.9 Extend the `Display` impl for `WorkingHoursCategory` with the arm for `VolunteerWork`
- [x] 1.10 Extend `From<&ExtraHoursReportCategoryTO> for WorkingHoursCategory` with the `VolunteerWork` arm
- [x] 1.11 Extend `From<&WorkingHoursCategory> for ExtraHoursCategoryTO` with the `VolunteerWork` arm
- [x] 1.12 Extend `From<&ExtraHoursCategoryTO> for WorkingHoursCategory` with the `VolunteerWork` arm
- [x] 1.13 Add `pub volunteer_hours: f32` field to `WorkingHours` in `src/state/employee.rs`
- [x] 1.14 Populate `volunteer_hours` in `From<&WorkingHoursReportTO> for WorkingHours`
- [x] 1.15 Add `pub volunteer_hours: f32` field to `Employee` in `src/state/employee.rs`
- [x] 1.16 Populate `volunteer_hours` in `From<&EmployeeReportTO> for Employee`
- [x] 1.17 Default `volunteer_hours` to `0.0` in `From<&ShortEmployeeReportTO> for Employee`
- [x] 1.18 Run `cargo check` and confirm no compile errors

## 2. i18n keys and translations

- [x] 2.1 Add `CategoryVolunteerWork` variant to the `Key` enum in `src/i18n/mod.rs`
- [x] 2.2 Add `CapPlannedHoursLabel` variant to the `Key` enum in `src/i18n/mod.rs`
- [x] 2.3 Add `CapPlannedHoursHelp` variant to the `Key` enum in `src/i18n/mod.rs`
- [x] 2.4 Add English translations for the three new keys in `src/i18n/en.rs`
- [x] 2.5 Add German translations for the three new keys in `src/i18n/de.rs` (using `Locale::De`, not `Locale::En`)
- [x] 2.6 Add Czech translations for the three new keys in `src/i18n/cs.rs`
- [x] 2.7 Verify all three locales resolve every new key (no fallback warnings on `cargo run`)

## 3. Tests (Red) — `volunteer-work-category-ui` capability

- [x] 3.1 Unit test: `WorkingHoursCategory::VolunteerWork.identifier() == "volunteer_work"`
- [x] 3.2 Unit test: `WorkingHoursCategory::from_identifier("volunteer_work")` yields `VolunteerWork`
- [x] 3.3 Unit test: `WorkingHoursCategory::VolunteerWork.is_volunteer_work() == true`; every other variant returns `false`
- [x] 3.4 Unit test: round-trip `WorkingHoursCategory::VolunteerWork` → `ExtraHoursCategoryTO::VolunteerWork` → `WorkingHoursCategory::VolunteerWork`
- [x] 3.5 Unit test: `From<&ExtraHoursReportCategoryTO::VolunteerWork>` yields `WorkingHoursCategory::VolunteerWork`
- [x] 3.6 Unit test: `From<&WorkingHoursReportTO>` with `volunteer_hours = 7.5` produces `WorkingHours.volunteer_hours == 7.5`
- [x] 3.7 Unit test: `From<&EmployeeReportTO>` with `volunteer_hours = 12.0` produces `Employee.volunteer_hours == 12.0`
- [x] 3.8 Unit test: `From<&ShortEmployeeReportTO>` produces `Employee.volunteer_hours == 0.0`

## 4. Tests (Red) — `weekly-planned-hours-cap-ui` capability

- [x] 4.1 Unit test: `EmployeeWorkDetails::blank_standard(<id>)` yields `cap_planned_hours_to_expected == false`
- [x] 4.2 Unit test: `TryFrom<&EmployeeWorkDetailsTO>` with `cap_planned_hours_to_expected = true` produces a state with `cap_planned_hours_to_expected == true`
- [x] 4.3 Unit test: `TryFrom<&EmployeeWorkDetails>` with `cap_planned_hours_to_expected = true` produces a TO with `cap_planned_hours_to_expected == true`
- [x] 4.4 Run `cargo test` and confirm all Phase 3 + Phase 4 tests fail or are absent of implementation that the next phases will provide

## 5. Implementation (Green) — Form changes

- [x] 5.1 Add a `FormPair` for the cap flag to `EmployeeWorkDetailsForm` (`src/component/employee_work_details_form.rs`) parallel to the existing `dynamic` row, using the new `CapPlannedHoursLabel` for the label
- [x] 5.2 Render a `<span class="text-xs text-gray-500">` (or equivalent) below the checkbox showing the localised `CapPlannedHoursHelp` text
- [x] 5.3 Disable the checkbox in `EmployeeWorkDetailsFormType::ReadOnly`; enable it for `New` and `Edit`
- [x] 5.4 Wire the `on_change` handler to clone the current `EmployeeWorkDetails`, mutate `cap_planned_hours_to_expected`, and call `props.on_update_employee_work_details`
- [x] 5.5 Add a `<option value="volunteer_work">` to the dropdown in `src/component/add_extra_hours_form.rs`, positioned directly after the `extra_work` option
- [x] 5.6 Confirm the dropdown change handler routes `"volunteer_work"` through `WorkingHoursCategory::from_identifier` to produce `VolunteerWork`

## 6. Implementation (Green) — View changes

- [x] 6.1 Add a `<li><TupleView label=… value=…/></li>` for `volunteer_hours` in the per-week section of `EmployeeView` (`src/component/employee_view.rs` around line 360–411), using the localised `CategoryVolunteerWork` label and the `{:.2} {hours_str}` formatting
- [x] 6.2 Add the analogous line item in the per-period section (around line 600–650) using `props.employee.volunteer_hours`
- [x] 6.3 Render both rows unconditionally (including when the value is `0.00`)

## 7. Implementation (Green) — Billing-period page

- [x] 7.1 In `src/page/billing_period_details.rs`, extend the `match key.to_uppercase().as_str()` block (around line 442) with the arm `"VOLUNTEER" => i18n.t(Key::CategoryVolunteerWork).to_string()`

## 8. Final verification

- [x] 8.1 Run `cargo fmt` and confirm no diff remains
- [x] 8.2 Run `cargo clippy` and resolve any new warnings introduced by this change
- [x] 8.3 Run `cargo test` and confirm all tests pass (including the new ones from Phases 3 and 4)
- [ ] 8.4 Manual run: start `shifty-backend` (`cargo run`) and `shifty-dioxus` (`dx serve --hot-reload` plus the Tailwind watcher); open the work-details form on a sales person, toggle the cap flag, save, reload, and confirm the flag persists
- [ ] 8.5 Manual run: enter a `VolunteerWork` extra-hours record via the add-extra-hours form for a person with `cap = true` exceeding `expected_hours` for a week; confirm `EmployeeView` shows `volunteer_hours = (auto-attributed) + (manual)` and the balance is consistent with the backend calculation
- [ ] 8.6 Manual run: open a billing period that contains volunteer hours; confirm the row labelled with the localised "Volunteer Work" header renders with the correct delta / YTD / full-year figures
- [ ] 8.7 Switch the locale to German and Czech in turn; confirm every new label and helper text resolves to a translated string (no English fallback)
