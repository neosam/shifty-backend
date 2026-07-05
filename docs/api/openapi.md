# OpenAPI & Swagger UI

## Where the schema lives

The OpenAPI schema is generated from Rust code at compile time via
[utoipa](https://docs.rs/utoipa):

- **Route annotations** — `#[utoipa::path(...)]` on every REST handler.
- **Schema derivation** — `#[derive(ToSchema)]` on every DTO in
  `rest-types/src/`.
- **Aggregation** — `ApiDoc` in `rest/src/lib.rs` or equivalent gathers
  all annotated handlers and schemas.

## Swagger UI

Reachable on a running backend at:

```
http://<host>:<port>/swagger-ui
```

**[To verify]** — exact path.

For second-client developers, Swagger UI is the authoritative reference
for endpoint signatures, DTOs, and examples.

## API authentication

- **`mock_auth` build:** no real auth — every request is treated as
  admin. Dev only.
- **`oidc` build:** OpenID Connect. Bearer token in the `Authorization`
  header.

Details: [`../architecture/04-auth.md`](../architecture/04-auth.md).

## Error mapping

All REST handlers route `ServiceError` through a central `error_handler`
wrapper. Mapping table:

| ServiceError | HTTP status | When |
| --- | --- | --- |
| `Unauthorized` | 401 | Missing / invalid auth token |
| `Forbidden` | 403 | Auth ok, but role insufficient |
| `NotFound` | 404 | Entity does not exist (or is soft-deleted) |
| `ValidationError(...)` | 400 | Bad request body / params |
| `Conflict(...)` | 409 | Duplicate, overlap, race |
| `InternalError(...)` | 500 | Everything else |

**[To verify]** — exact enum variants in `service/src/lib.rs` or
`service_impl/src/lib.rs`.

## DTO conventions

- **Naming:** DTOs end with `TO` (transport object), e.g. `BookingTO`,
  `SalesPersonTO`.
- **Wire format:** JSON.
- **UUIDs:** as string (hyphenated).
- **Dates:** ISO 8601 (`YYYY-MM-DD`).
- **Timestamps:** ISO 8601 (`YYYY-MM-DDTHH:MM:SSZ`) — **[To verify]**
  whether with timezone.
- **Enums:** as string (variant name).

## Pagination

**[To verify]** — whether Shifty uses offset-based or cursor-based
pagination. Many endpoints currently appear to have none (full list).

## Idempotency

**[To verify]** — which endpoints are idempotent (safe to retry) and
which are not.

## Per-feature endpoint overview

Each feature cluster has its own REST table in the corresponding feature
doc:

- [F01 Employee Management](../features/F01-employee-management.md#5-rest-endpoints)
- [F02 Shiftplan Core](../features/F02-shiftplan-core.md#5-rest-endpoints)
- [F03 Booking](../features/F03-booking.md#5-rest-endpoints)
- [F04 Extra Hours](../features/F04-extra-hours.md#5-rest-endpoints)
- [F05 Absence System](../features/F05-absence-system.md#5-rest-endpoints)
- [F06 Vacation Management](../features/F06-vacation-management.md#5-rest-endpoints)
- [F07 Reporting & Balance](../features/F07-reporting-balance.md#5-rest-endpoints)
- [F08 Billing Period](../features/F08-billing-period.md#5-rest-endpoints)
- [F09 Week Metadata](../features/F09-week-metadata.md#5-rest-endpoints)
- [F10 Templates & Communication](../features/F10-templates-communication.md#5-rest-endpoints)
- [F11 Export](../features/F11-export.md#5-rest-endpoints)
- [F12 Auth & Session](../features/F12-auth-session.md#5-rest-endpoints)
- [F13 System Infrastructure](../features/F13-system-infrastructure.md#5-rest-endpoints)

## For second-client developers

If you build your own client (mobile, CLI, automation):

1. Read [`../README.md`](../README.md), especially the "Fat Backend,
   Thin Client" principle.
2. Generate client code from the OpenAPI schema (most languages have
   generator tooling).
3. Note that balance calculation, conflict checks, and snapshot
   semantics are **not** to be duplicated in the client — always go
   through the backend API.
4. For current state (live report), use an endpoint with a sufficiently
   specific date-range selection.
5. For history (billing), ALWAYS use the billing-period endpoints —
   never a live recomputation, since it drifts.
