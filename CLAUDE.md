# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Database Setup
```bash
# Copy environment template and configure database
cp env.example .env
# Setup database and run migrations
sqlx setup --source migrations/sqlite
```

### Build and Run
```bash
# Standard build
cargo build

# Run development server with hot reload
cargo watch -x run

# Standard run
cargo run
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for specific service
cargo test booking

# Run specific test
cargo test test_create_booking_success
```

### Version Management
```bash
# Update all crate versions consistently
./update_versions.sh 1.2.3
```

## Architecture Overview

**Shifty Backend** is an employee shift planning and HR management system built with clean layered architecture using Rust. The system manages employee schedules, working hours, overtime, vacation, and leave tracking.

### Multi-Crate Workspace Structure

- **`shifty_bin`** - Main executable with dependency injection
- **`service`** - Business logic trait definitions  
- **`service_impl`** - Concrete service implementations
- **`dao`** - Data access trait definitions
- **`dao_impl_sqlite`** - SQLite-specific implementations
- **`rest`** - HTTP API endpoints (Axum framework)
- **`rest-types`** - Transport objects (DTOs) with OpenAPI schemas
- **`shifty-utils`** - Shared utilities

### Layered Architecture Pattern

```
REST Layer (Axum) → Service Layer (Business Logic) → DAO Layer (Data Access) → SQLite Database
```

**Key Principles:**
- Services defined as traits, implemented with dependency injection via `gen_service_impl!` macro
- All service methods accept `Option<Transaction>` for transaction management
- Authentication context passed through all service calls
- Comprehensive error handling with `ServiceError` mapped to HTTP responses

### Transaction Management

Every service method follows this pattern:
```rust
async fn do_something(&self, tx: Option<Self::Transaction>) -> Result<T, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // ... business logic and DAO calls ...
    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

### Authentication & Authorization

- **Development**: Mock authentication with auto-created admin user
- **Production**: OIDC integration  
- **RBAC**: Role-based access control with privilege checking
- Context: `Authentication<Context>` passed to all service methods

## Core Domain Concepts

### Shift Management
- **Shift Plans**: Employee work schedules and time slots
- **Bookings**: Assignments of employees to specific slots
- **Sales Persons**: Employee entities with working hour contracts
- **Slots**: Time slots defining when work can be scheduled

### Time Tracking
- **Working Hours**: Contract-defined expected hours per employee
- **Extra Hours**: Overtime, vacation, holidays, sick leave
- **Balance Hours**: Calculated surplus/deficit of worked vs expected hours
- **Carryover Hours**: Year-end balance persistence for performance optimization

### Special Features
- **Special Days**: Holidays and company events affecting calculations
- **Reporting**: Complex time calculations and balance reporting
- **Permissions**: Fine-grained access control for different user roles

## Implementation Patterns

### Service Implementation
```rust
// Use gen_service_impl! macro for dependency injection
gen_service_impl! {
    struct SomeServiceImpl: service::SomeService = SomeServiceDeps {
        SomeDao: dao::SomeDao = some_dao,
        PermissionService: service::PermissionService = permission_service,
        TransactionDao: dao::TransactionDao = transaction_dao
    }
}
```

### DAO Implementation
- Database interactions use SQLx with compile-time query checking
- Entities converted from database rows via `TryFrom` trait
- All queries include soft delete checks (`WHERE deleted IS NULL`)

### REST API
- Axum framework with modular routing per domain area
- OpenAPI documentation with `utoipa` crate - always add `#[utoipa::path]` annotations
- Transport objects (TOs) in `rest-types` with `ToSchema` derive for OpenAPI
- Consistent error handling via `error_handler` wrapper

### Testing
- **Unit Tests**: Mock-based testing with `mockall` crate
- **Integration Tests**: Full-stack testing with in-memory SQLite
- Test structure in `service_impl/src/test/` with domain-specific modules
- Helper traits like `NoneTypeExt` for authentication in tests

## Key Development Notes

### OpenAPI Documentation
When adding new REST endpoints, always include:
- `#[utoipa::path(...)]` annotation on handler functions
- Proper parameter and response body documentation
- Add endpoints to the ApiDoc struct for Swagger UI inclusion
- Ensure DTOs have `ToSchema` derive attribute

### Database Migrations
- Migrations in `migrations/sqlite/` directory
- Use `sqlx migrate add <name>` to create new migrations
- SQLx compile-time checking requires up-to-date local database

### Feature Flags
- `mock_auth` - Development authentication bypass
- `oidc` - Production OIDC integration
- `local_logging`/`json_logging` - Logging format control

### Error Handling
- Services return `Result<T, ServiceError>`
- DAOs return `Result<T, DaoError>`
- REST layer maps errors to appropriate HTTP status codes
- Comprehensive error contexts throughout the stack

## Business Logic Complexity

The system handles sophisticated time calculations:
- Employee balance hours (worked vs expected)
- Year-end carryover to avoid recalculating historical data
- Overlapping time ranges and booking conflicts
- Multiple absence types (vacation, sick leave, holidays)
- Special day handling affecting working hour expectations

This architecture ensures clean separation of concerns, comprehensive testing, and production-ready deployment capabilities while maintaining developer productivity through hot reload and mock authentication.
- Always execute cargo build, cargo test and cargo run (with some timeout) when you implement new features.