## Why

With the introduction of multiple shift plans, any sales person can currently be booked into any plan without restriction. As the number of plans grows, administrators need the ability to control which employees are eligible for which plans. This prevents incorrect bookings and prepares the system for future visibility controls.

## What Changes

- New N:M mapping between sales persons and shift plans via a `sales_person_shiftplan` join table
- **Permissive model**: Sales persons with no assignments can be booked into all plans. Once at least one assignment exists, they are restricted to only those assigned plans.
- Hard validation in booking creation: reject bookings where the sales person is not eligible for the slot's shift plan
- New REST endpoints to manage assignments (CRUD from the sales person side)
- New REST endpoint to query bookable sales persons for a given shift plan (respecting the permissive logic)
- Existing bookings remain untouched — no retroactive enforcement

## Capabilities

### New Capabilities
- `sales-person-shiftplan-assignment`: N:M assignment of sales persons to shift plans, with permissive eligibility model and hard booking validation

### Modified Capabilities

## Impact

- **Database**: New `sales_person_shiftplan` join table (migration)
- **DAO layer**: New `SalesPersonShiftplanDao` trait and SQLite implementation
- **Service layer**: New service for managing assignments; booking service extended with eligibility check
- **REST layer**: New endpoints under `/api/sales-person/{id}/shiftplans` and `/api/sales-person/by-shiftplan/{shiftplan_id}`
- **REST types**: New DTOs for assignment management
- **Frontend** (out of scope for backend): Will need UI in employee management to assign shift plans
