## Why

Setting up a local development environment requires manually creating test data through multiple API calls. This is tedious and error-prone, slowing down frontend development, demos, and debugging. A single endpoint to seed realistic test data (and another to clear it) would make local development significantly faster.

## What Changes

- Add a `POST /dev/seed` endpoint that creates a complete set of test data: sales persons with various states (active/inactive, paid/unpaid), employee work details (contracts with different hours), extra hours (vacation, sick leave, overtime), bookings for the current calendar week, and special days (holidays).
- Add a `POST /dev/clear` endpoint that wipes all data from the database for a clean reset.
- Both endpoints are feature-gated behind `mock_auth` so they are never compiled into production builds.
- Both endpoints are documented in Swagger/OpenAPI for easy access.

## Capabilities

### New Capabilities
- `dev-seed`: REST endpoints for seeding and clearing test data in development environments, gated behind the `mock_auth` feature flag.

### Modified Capabilities
<!-- No existing capabilities are modified — this is purely additive dev tooling. -->

## Impact

- **Code**: New `rest/src/dev.rs` module with two endpoints, conditionally compiled. Route registration in `rest/src/lib.rs` behind `#[cfg(feature = "mock_auth")]`. May need to extend `BasicDao::clear_all()` to cover additional tables (special_days, slots, etc.).
- **APIs**: Two new REST endpoints under `/dev/` — only available in dev builds.
- **Dependencies**: No new dependencies. Uses existing services and `Authentication::Full` for admin-level access.
- **Systems**: No production impact — code does not exist in release builds.
