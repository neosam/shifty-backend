# API Reference — REST Endpoints & Conventions

This section targets developers building a **second client** against the
Shifty backend — mobile app, CLI, data export tool, etc.

The "Fat Backend, Thin Client" objective guarantees that all domain rules
(balance calculation, absence conflicts, billing snapshot) live in the backend
and are exposed via REST. A new client does **not** have to duplicate any
domain rule.

## Chapters

- **[openapi.md](./openapi.md)** — How to obtain the OpenAPI schema,
  Swagger UI, authentication, error mapping.
- **[conventions.md](./conventions.md)** — DTO conventions, pagination,
  error formats, transaction semantics across the HTTP boundary,
  field nullability, time and date formats.

## Basics

- **Framework:** Axum (Rust)
- **Docs:** [utoipa](https://docs.rs/utoipa) generates the OpenAPI schema from
  `#[utoipa::path(...)]` annotations and `ToSchema` derives on DTOs.
- **DTOs (Transport Objects, TOs):** All DTOs live in the `rest-types` crate.
  The frontend client consumes the same crate — there is exactly one source
  of truth for field names, types, and optionality.
- **Auth:** Either OIDC (prod) or mock (`mock_auth` feature flag).
  Details in [`../architecture/04-auth.md`](../architecture/04-auth.md).
- **Errors:** `ServiceError` is consistently mapped to HTTP status codes via
  `error_handler` (see `conventions.md`).

## Endpoint Overview

For a semantic overview of which endpoints belong to which domain, see
[`../features/`](../features/README.md) — the associated endpoint list is
documented per feature there.

For the full, machine-readable reference: Swagger UI at `/swagger-ui`
in the running backend.
