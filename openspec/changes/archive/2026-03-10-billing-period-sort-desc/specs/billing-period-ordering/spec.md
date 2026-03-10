## ADDED Requirements

### Requirement: Billing periods ordered descending by start date

The `BillingPeriodDao` trait SHALL provide an `all_ordered_desc` method that returns all non-deleted billing periods sorted by `start_date` in descending order (newest first).

#### Scenario: Multiple billing periods returned in descending order
- **WHEN** there are billing periods with start dates 2025-01-01, 2025-04-01, and 2025-07-01
- **THEN** `all_ordered_desc` SHALL return them in order: 2025-07-01, 2025-04-01, 2025-01-01

#### Scenario: Single billing period
- **WHEN** there is exactly one billing period
- **THEN** `all_ordered_desc` SHALL return it as the only element

#### Scenario: No billing periods
- **WHEN** there are no non-deleted billing periods
- **THEN** `all_ordered_desc` SHALL return an empty collection

### Requirement: Billing period overview uses descending order

The `get_billing_period_overview` service method SHALL return billing periods sorted by `start_date` descending by using `all_ordered_desc`.

#### Scenario: REST API returns newest billing period first
- **WHEN** a client requests the billing period overview
- **THEN** the response SHALL contain billing periods ordered with the most recent period first
