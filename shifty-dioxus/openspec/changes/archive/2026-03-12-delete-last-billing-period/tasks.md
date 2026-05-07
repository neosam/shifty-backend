## 1. API Layer

- [x] 1.1 Add `delete_billing_period(config, id)` function to `src/api.rs` that sends `DELETE /billing-period/{id}` and returns `Result<(), reqwest::Error>`

## 2. Internationalization

- [x] 2.1 Add i18n keys to `src/i18n/mod.rs`: `DeleteBillingPeriod`, `ConfirmDeleteBillingPeriod`, `DeleteBillingPeriodError`
- [x] 2.2 Add English translations to `src/i18n/en.rs`
- [x] 2.3 Add German translations to `src/i18n/de.rs`
- [x] 2.4 Add Czech translations to `src/i18n/cs.rs`

## 3. Employees Page UI

- [x] 3.1 Add `DeleteBillingPeriod(Uuid)` and `ConfirmDeleteBillingPeriod` and `CancelDeleteBillingPeriod` variants to `EmployeesPageAction` in `src/page/employees.rs`
- [x] 3.2 Add state signals for delete confirmation dialog (show/hide, selected billing period ID, error message)
- [x] 3.3 Add delete button on the first billing period card, visible only when `auth_info.has_privilege("hr")`, with `stop_propagation` to prevent navigation
- [x] 3.4 Add confirmation modal with period date range, Cancel/Delete buttons, and error message area
- [x] 3.5 Implement delete action handler: call `api::delete_billing_period` directly, on success reload billing periods and close modal, on error set error signal

## 4. Testing

- [x] 4.1 Add tests for the delete API function
- [x] 4.2 Add tests for the i18n keys (all three locales have translations)
- [x] 4.3 Add tests for HR privilege visibility logic
