# Layered Architecture

Shifty follows a classic three-layer architecture with clear boundaries.

```
┌───────────────────────────────────────────────┐
│  REST layer      rest/         (Axum)         │  HTTP handlers, DTO mapping,
│                                               │  Error → HTTP status
├───────────────────────────────────────────────┤
│  Service layer   service/, service_impl/      │  Business logic, auth gates,
│                                               │  TX management, cross-domain
├───────────────────────────────────────────────┤
│  DAO layer       dao/, dao_impl_sqlite/       │  SQLx queries, row → entity
│                                               │  conversion
├───────────────────────────────────────────────┤
│  Storage         SQLite (migrations/sqlite/)  │  Schema, views, indices
└───────────────────────────────────────────────┘
```

Cross-cutting concerns like DTOs (`rest-types`) and utilities (`shifty-utils`)
sit alongside the layers.

## Why this separation?

- The **REST layer** knows nothing about the database. It is freely
  replaceable (today Axum, tomorrow actix-web).
- The **service layer** knows nothing about HTTP. Via its trait interfaces
  it is callable from a CLI, a job runner, or a test harness.
- The **DAO layer** knows no business rules. It returns rows and entities;
  auth, validation, and composition are the services' job.
- **Fat backend, thin client** (see root README): all business rules live
  in the service layer. The frontend renders results — it does not compute
  balances on its own.

## Trait-first principle

Every service and every DAO is first a **trait** in `service/` respectively
`dao/`. The implementation lives in `service_impl/` respectively
`dao_impl_sqlite/`.

Consequences:

- **Testability.** Unit tests mock at the trait boundary (via `mockall`),
  without a database or HTTP.
- **Replaceability.** A Postgres DAO would be a parallel implementation
  next to `dao_impl_sqlite/`, without touching any service code.
- **Explicit dependencies.** The trait declaration lists
  `type Context`, `type Transaction`, and the return error type. Nothing is
  implicit.

## The `gen_service_impl!` macro

Service implementations are not wired up by hand as structs. The
`gen_service_impl!` macro (declared in `service_impl/src/macros.rs`)
takes care of it:

```rust
gen_service_impl! {
    struct BookingServiceImpl: service::BookingService = BookingServiceDeps {
        BookingDao: dao::BookingDao = booking_dao,
        PermissionService: service::PermissionService = permission_service,
        TransactionDao: dao::TransactionDao = transaction_dao
    }
}
```

This generates:

- A `struct BookingServiceImpl<Deps: BookingServiceDeps>` with typed
  fields for every dependency.
- A trait `BookingServiceDeps` that consumers in `main.rs` implement, so
  the DI container can plug in the concrete implementations.
- Consistent `Arc` and `Clone` handling wherever the async boundary
  requires it.

## Error mapping

- **`DaoError`** for DAO calls (SQL errors, DB constraints).
- **`ServiceError`** for service calls; wraps `DaoError`, adds business
  errors (Forbidden, Conflict, Validation).
- The **HTTP status** is derived in the REST layer via a central
  `error_handler` wrapper from `ServiceError`.

The effect: one error convention per layer, no conversion explosion.

## Auth propagation

`Authentication<Context>` is the auth context that REST handlers pass to
services. It flows through the whole service chain down to the permission
checks. Details in [`04-auth.md`](./04-auth.md).

## Transactions

Every service method takes `Option<Self::Transaction>`. If `None`, the
service opens a transaction itself. If `Some`, it continues inside the
outer TX. Details in [`05-transactions.md`](./05-transactions.md).

## Deeper reading

- [02-service-tiers.md](./02-service-tiers.md) — Basic vs Business-Logic,
  dependency rules.
- [03-data-model.md](./03-data-model.md) — How DAOs interact with the
  schema.
- [07-testing.md](./07-testing.md) — How the trait-first principle
  carries the tests.
