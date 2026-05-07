## Context

The backend already supports soft-deleting the last billing period via `DELETE /billing-period/{id}` (returns 204 on success, 409 if not the latest, 403 if not HR). The frontend needs a UI to trigger this. The existing codebase uses a `BillingPeriodAction` enum dispatched through a coroutine, and the `Modal` component for dialogs. HR permission checks use `auth_info.has_privilege("hr")`.

## Goals / Non-Goals

**Goals:**
- Allow HR users to delete the last billing period from the employees list page
- Provide confirmation before deletion to prevent accidental data loss
- Show errors inline in the confirmation dialog
- Follow existing patterns (coroutine actions, Modal component, i18n)

**Non-Goals:**
- No toast/notification system — errors stay in the modal
- No deletion of arbitrary billing periods (backend constraint: only the latest)
- No changes to the backend
- No undo after successful deletion (backend soft-deletes, but this is invisible to users)

## Decisions

### 1. Delete button placement: on the billing period card

The delete button appears on the **first card** in the billing periods list (which is the latest, since the backend returns them sorted). Only shown when `auth_info.has_privilege("hr")` is true. The button uses `onclick` with `prevent_default` and `stop_propagation` to prevent navigating to the detail page.

**Alternative considered**: Button on the detail page — rejected because the user wanted it directly on the card for quick access.

### 2. Confirmation via existing Modal component

Reuse the existing `Modal` component (same pattern as the "Create Billing Period" dialog). The modal shows the period date range and has Cancel/Delete buttons. On API error, the error message appears inside the modal without closing it.

**Alternative considered**: Browser `confirm()` dialog — rejected because it can't show error messages and looks inconsistent.

### 3. Delete action through BillingPeriodService coroutine

Add a `DeleteBillingPeriod(Uuid)` variant to `BillingPeriodAction`. The service function calls the API, then reloads the billing periods list on success. This follows the exact same pattern as `CreateBillingPeriod`.

However, unlike create, delete needs to communicate success/failure back to the UI (to keep the modal open on error). This will be handled by introducing a signal for the delete error state, managed within the page component. The page action handler will call the API directly (not through the coroutine) to get the result synchronously, then trigger a reload via the coroutine.

**Alternative considered**: Routing everything through the coroutine — rejected because the coroutine's error handling goes to the global `ERROR_STORE`, which doesn't let us show errors in the modal.

### 4. Direct API call from page action handler

The `EmployeesPageAction` handler will call `api::delete_billing_period` directly in the async block and handle success/error locally. On success, it sends `BillingPeriodAction::LoadBillingPeriods` to refresh the list and closes the modal. On error, it sets an error signal displayed in the modal.

This is a pragmatic deviation from routing everything through the service, but it's necessary for inline error handling.

## Risks / Trade-offs

- **[Risk] Button click propagates to Link** → Mitigation: Use `prevent_default` and `stop_propagation` on the delete button click handler
- **[Risk] Race condition if list changes while modal is open** → Low risk since only HR users can create/delete, and this is a single-user action
- **[Trade-off] Direct API call from page vs service coroutine** → Accepts slight pattern inconsistency for better UX (inline error display)
