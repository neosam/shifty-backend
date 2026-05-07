# billing-periods-page Specification

## Purpose
TBD - created by archiving change redesign-07-page-employees. Update Purpose after archive.
## Requirements
### Requirement: Billing periods rendered as a standalone page at Route::BillingPeriods
The system SHALL provide a new `Route::BillingPeriods` rendered by a `BillingPeriods` page component in `src/page/billing_periods.rs`, re-exported from `src/page/mod.rs`. The page SHALL render the redesigned `TopBar` followed by the billing-period management UI (heading from `Key::BillingPeriods`, "create new" button, list of billing periods, create dialog, delete dialog) — all of which were previously embedded in `src/page/employees.rs`. The behavior of loading billing periods, creating, deleting, and listing SHALL be functionally identical to the previous embedded version. The route SHALL be reachable from the new employees page (via a navigation entry or button — the placement is informative).

#### Scenario: Page renders heading and create button
- **WHEN** the page is rendered with at least one billing period in `BILLING_PERIOD_STORE`
- **THEN** the rendered HTML SHALL contain `i18n.t(Key::BillingPeriods)` heading text and a button labeled `i18n.t(Key::CreateNewBillingPeriod)`

#### Scenario: Loading state preserved
- **WHEN** the page is rendered with `BILLING_PERIOD_STORE.read().billing_periods.is_empty()` true
- **THEN** the rendered HTML SHALL contain the `Key::LoadingBillingPeriods` loading message styled with `text-ink-muted`

### Requirement: Billing-period dialogs use the redesigned Dialog atom
The create-billing-period and delete-billing-period dialogs on the new `BillingPeriods` page SHALL use the `Dialog` atom (from `src/component/dialog.rs`) instead of the legacy `Modal` from `src/component/modal.rs`. Each dialog SHALL render its title via the `Dialog`'s `title` prop, its body via children, and its actions via the `footer` slot containing `Btn` atoms (Secondary for Cancel, Primary or Danger for the confirming action).

#### Scenario: Create-period dialog uses Dialog
- **WHEN** the user clicks the "create new billing period" button
- **THEN** the rendered HTML SHALL contain a `Dialog` panel (carrying classes resolving to `bg-surface` and `var(--surface)` background) and SHALL NOT include the legacy `Modal` markup (no `crate::component::Modal` instance)

#### Scenario: Delete-period dialog uses Dialog with Danger button
- **WHEN** the user clicks the delete button for a billing period
- **THEN** the rendered HTML SHALL contain a `Dialog` panel whose footer includes a `Btn` with `variant: BtnVariant::Danger` carrying classes `text-bad` and `border-bad`

#### Scenario: Dialogs use Field atoms for inputs
- **WHEN** the create-period dialog renders the end-date input
- **THEN** the input SHALL be wrapped in a `Field` atom (or use `FormTextInput`) whose `<input>` carries the `form-input` class and `type=date`

### Requirement: Existing BILLING_PERIOD_STORE and Route::BillingPeriodDetails preserved
The redesigned billing-periods page SHALL continue to use `BILLING_PERIOD_STORE` and `BillingPeriodAction` for all data flow. The existing `Route::BillingPeriodDetails { billing_period_id }` route SHALL continue to work and SHALL be reachable from each billing-period list row. No backend changes and no `BillingPeriodAction` surface changes are required.

#### Scenario: List rows link to BillingPeriodDetails
- **WHEN** the page renders a billing-period row
- **THEN** the row SHALL be wrapped in a `<Link>` whose target is `Route::BillingPeriodDetails { billing_period_id }`

#### Scenario: Coroutine actions unchanged
- **WHEN** the page mounts
- **THEN** it SHALL dispatch `BillingPeriodAction::LoadBillingPeriods` exactly as the previous embedded version did

