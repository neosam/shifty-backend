## 1. State Layer (src/state/employee.rs)

- [x] 1.1 Add `UnpaidLeave` variant to `WorkingHoursCategory` enum
- [x] 1.2 Add `is_unpaid_leave()` helper method
- [x] 1.3 Add `"unpaid_leave"` to `identifier()` and `from_identifier()`
- [x] 1.4 Add `to_i18n_key()` mapping to `Key::CategoryUnpaidLeave`
- [x] 1.5 Add `Display` impl for `UnpaidLeave`
- [x] 1.6 Add `From<ExtraHoursReportCategoryTO>` mapping for `UnpaidLeave`
- [x] 1.7 Add `From<WorkingHoursCategory>` to `ExtraHoursCategoryTO` mapping for `UnpaidLeave`
- [x] 1.8 Add `From<ExtraHoursCategoryTO>` to `WorkingHoursCategory` mapping for `UnpaidLeave`
- [x] 1.9 Add `unpaid_leave_hours: f32` to `WorkingHours` struct and its `From<WorkingHoursReportTO>` impl
- [x] 1.10 Add `unpaid_leave_hours: f32` to `Employee` struct, `From<EmployeeReportTO>`, and `From<ShortEmployeeReportTO>` (default 0.0)

## 2. i18n Translations

- [x] 2.1 Add `CategoryUnpaidLeave` key to `src/i18n/mod.rs`
- [x] 2.2 Add English translation "Unpaid Leave" in `src/i18n/en.rs`
- [x] 2.3 Add German translation "Unbezahlter Urlaub" in `src/i18n/de.rs`
- [x] 2.4 Add Czech translation "Neplacene volno" in `src/i18n/cs.rs`

## 3. Service Layer (src/service/employee.rs)

- [x] 3.1 Add `unpaid_leave_hours: 0.0` to default `WorkingHours` initialization

## 4. Components

- [x] 4.1 Add "Unpaid Leave" option to category dropdown in `src/component/add_extra_hours_form.rs`
- [x] 4.2 Add unpaid leave entries section to extra hours list in `src/component/employee_view.rs` (ExtraHoursView)
- [x] 4.3 Add unpaid leave hours line to weekly working hours summary in `src/component/employee_view.rs` (WorkingHoursView)
- [x] 4.4 Add unpaid leave hours line to employee overall report in `src/component/employee_view.rs` (EmployeeView)

## 5. Verification

- [x] 5.1 Run `cargo check` to verify compilation with updated rest-types
- [x] 5.2 Run `cargo test` to verify all tests pass
- [x] 5.3 Run `cargo clippy` to verify no linting issues
