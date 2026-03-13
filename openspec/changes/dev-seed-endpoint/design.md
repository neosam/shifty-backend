## Context

Local development currently requires manually creating test data through individual API calls or direct database manipulation. The system already has patterns for feature-gated code (`mock_auth`), a `BasicDao::clear_all()` method used in integration tests, and services that accept `Authentication::Full` for admin-level bypass.

## Goals / Non-Goals

**Goals:**
- Provide a single endpoint to populate the database with realistic test data
- Provide a single endpoint to clear all data for a clean reset
- Ensure these endpoints never exist in production builds
- Make endpoints accessible via Swagger UI for easy use

**Non-Goals:**
- Configurable or parameterized test data (fixed dataset is sufficient)
- Seeding data for billing periods or text templates (user creates these manually)
- Creating a separate service layer for dev tooling (direct service calls from REST handler)

## Decisions

### 1. Feature-gate with `mock_auth` instead of a new feature flag
**Decision**: Use the existing `mock_auth` feature flag.
**Rationale**: `mock_auth` already represents "development mode" and is never enabled in production. Adding a separate feature flag would increase configuration complexity for no benefit.
**Alternative considered**: New `dev_tools` feature flag — rejected because it would need to be coordinated with `mock_auth` anyway (dev endpoints without mock auth are useless).

### 2. No new service layer — REST handler calls existing services directly
**Decision**: The seed endpoint in `rest/src/dev.rs` calls existing services (`SalesPersonService`, `ExtraHoursService`, etc.) with `Authentication::Full`.
**Rationale**: This is dev-only throwaway code. Adding a `DevService` trait + implementation + DI wiring would be over-engineering for code that should stay simple and disposable. The REST handler orchestrates the creation sequence directly.
**Alternative considered**: New `DevService` in the service layer — rejected as unnecessary abstraction for dev tooling.

### 3. Use `Authentication::Full` for all service calls
**Decision**: All service calls in the seed endpoint use `Authentication::Full` which bypasses permission checks.
**Rationale**: This is the established pattern in integration tests. Since the endpoints are feature-gated to dev-only, there is no security concern.

### 4. Extend `BasicDao::clear_all()` to cover all tables
**Decision**: Extend the existing `clear_all` method to also delete from `special_day`, `slot`, `booking`, and other tables not currently covered.
**Rationale**: The current `clear_all` misses several tables. A comprehensive clear is needed for a true reset. The deletion order must respect foreign key constraints.
**Alternative considered**: Separate `dev_clear_all` method — rejected because the existing `clear_all` is already only used in tests and should be comprehensive.

### 5. Seed data uses current calendar week
**Decision**: Bookings and time-sensitive data are created relative to the current date (obtained via `ClockService::date_now()`).
**Rationale**: Static dates would quickly become stale. Using the current week ensures the seeded data is always immediately visible and relevant in the UI.

### 6. Seed endpoint is additive, not idempotent
**Decision**: Each call to `POST /dev/seed` adds new data. For a clean state, call `POST /dev/clear` first.
**Rationale**: Simplest implementation. Idempotency would require checking for existing data, adding complexity for no real benefit in a dev tool.

## Risks / Trade-offs

- **[Risk] `clear_all` deletes everything including manually created data** → Acceptable for dev tooling; documented in Swagger description.
- **[Risk] Seed data may conflict with existing data on repeated calls** → Mitigated by recommending clear-then-seed workflow. UUID generation ensures no ID conflicts.
- **[Trade-off] No service layer means less testability** → Accepted because this is dev tooling; integration test of the endpoint itself is sufficient.
