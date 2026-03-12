## Why

The backend now supports deleting the last billing period (soft-delete via `DELETE /billing-period/{id}`), but the frontend has no UI for this. HR users need the ability to undo an accidentally created billing period directly from the employees page without requiring database intervention.

## What Changes

- Add a delete button on the last billing period card in the employees list page (`/employees/`)
- Button is only visible to users with HR permissions
- Clicking the button opens a confirmation dialog (using existing Modal component)
- On success, the billing period card disappears from the list
- On error, the error message is shown inside the confirmation dialog
- Add `delete_billing_period` API function to call `DELETE /billing-period/{id}`
- Add `DeleteBillingPeriod` action to the billing period service
- Add i18n keys for button label, confirmation text, and error messages (En, De, Cs)

## Capabilities

### New Capabilities
- `billing-period-delete`: Frontend UI and service layer for deleting the last billing period, including API integration, confirmation dialog, error handling, and i18n support

### Modified Capabilities

## Impact

- **Frontend code**: `src/api.rs`, `src/service/billing_period.rs`, `src/page/employees.rs`, `src/i18n/mod.rs`, `src/i18n/en.rs`, `src/i18n/de.rs`, `src/i18n/cs.rs`
- **Backend**: No changes needed — endpoint already exists
- **API contract**: Uses existing `DELETE /billing-period/{id}` (204 success, 409 conflict, 403 forbidden)
