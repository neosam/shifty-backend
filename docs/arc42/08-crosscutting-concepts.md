# 8. Cross-cutting Concepts

Each concept is summarized here with a pointer to its authoritative reference
document; the reference wins on detail questions.

## 8.1 Authentication & Authorization

Reference: [04-auth](../architecture/04-auth.md), [F12](../features/F12-auth-session.md).

- Two compile-time modes: `mock_auth` (dev, hard-wired admin) and `oidc`
  (prod, external IdP via `axum-oidc`). Both yield the same unified context
  type `Option<Arc<str>>` (username), so services are mode-agnostic.
- RBAC: users → roles → privileges (`admin`, `hr`, `sales`, `shiftplanner`,
  `shiftplan.edit`, `toggle_admin`, `feature_flag_admin`, …). Services gate via
  `PermissionService::check_permission`; handlers never gate on their own.
- `Authentication<Context>` is either `Context(user)` or `Full`. **`Full`
  bypasses all checks** and is reserved for internal aggregate reads after the
  outer call has authenticated the user; it must never appear in a REST
  handler.
- Sessions are server-side (`session` table) referenced by an
  httpOnly/Secure/SameSite=Strict cookie. Admin **impersonation** sets
  `impersonate_user_id` on the session; the context extractor then presents
  the target identity downstream (audited).
- Middleware chain: cookies → `context_extractor` → `forbid_unauthenticated`
  (401; exemptions: iCal feed, authenticate endpoint).

## 8.2 Transactions

Reference: [05-transactions](../architecture/05-transactions.md).

Every service method takes `Option<Transaction>` as its last parameter:
`use_transaction(None)` opens and owns a new transaction,
`use_transaction(Some(tx))` joins the caller's. Commit is reference-counted —
only the outermost owner actually commits; on error the chain breaks and
SQLite rolls back on drop. Rule: data moves (slot split, booking re-point)
happen in **one** transaction or reports double-count. SQLite serializes
writers (`BUSY` under contention), so long reads should not sit in write
transactions unnecessarily.

## 8.3 Persistence Conventions

Reference: [03-data-model](../architecture/03-data-model.md).

- UUIDs (v4) as 16-byte blob primary keys; generation via mockable `UuidService`.
- **Soft-delete**: `deleted` timestamp, readers filter `WHERE deleted IS NULL`;
  DELETE endpoints soft-delete. Enforced by convention, not schema.
- Optimistic concurrency via `update_version` (UUID rotated per update);
  audit columns `update_process` / `update_timestamp`, plus `created_by` /
  `deleted_by` where user attribution matters (bookings).
- Views (`bookings_view`) denormalize for read paths and must be re-created in
  migrations when underlying columns change.
- SQLx compile-time query checking with committed `.sqlx/` offline cache.

## 8.4 Error Handling

`ServiceError` is the single error currency of the service layer
(`Unauthorized` → 401, `Forbidden` → 403, `NotFound` → 404,
`ValidationError` → 400, `Conflict`/`PaidLimitExceeded` → 409, else 500),
mapped centrally by `rest`'s `error_handler`. DAO errors (`DaoError`) are
wrapped, never leaked. Distinct from errors: **warnings** (e.g.
`BookingOnAbsenceDay`) are derived, non-blocking, travel inside success
responses, and are never persisted ([F09](../features/F09-week-metadata.md)).

## 8.5 Domain Time Model

Reference: [time-accounting](../domain/time-accounting.md),
[edge-cases](../domain/edge-cases.md) §1/§4/§5.

ISO 8601 weeks everywhere (`year` + `calendar_week` 1..=53, clamped via
`weeks_in_year`); year-boundary weeks handled with overshoot logic. Hours are
`f32` — convention: **always display backend-computed totals, never re-sum
rounded client-side values**. Absence hours are derived at read time from the
contract active on each day (no denormalized hour rows). The balance perimeter
per category (what counts as worked vs reduces expectation — UnpaidLeave
asymmetry!) is normative in [F07 §2.2](../features/F07-reporting-balance.md).

## 8.6 Feature Evolution: Flags, Toggles, Cutovers

Reference: [F13](../features/F13-system-infrastructure.md), edge-cases §9.

Two mechanisms: `feature_flag` (static boolean, admin-gated) and `toggle`
(user- and/or date-aware, supports effective-date "Stichtag" rollouts). Big
semantic migrations run as **cutovers** with both data sources coexisting
(extra-hours → absence-period being the prototype: readers aggregate both
tables; conversion is explicit and audited via
`absence_period_migration_source`).

## 8.7 Internationalization

Reference: [08-i18n](../architecture/08-i18n.md).

Three locales (En/De/Cs), mandatory for every key. Translations live in the
frontend (`shifty-dioxus/src/i18n/`); the backend returns enum codes, not
translated strings — except server-rendered templates (PDF/report/email
texts). Full-sentence templates with placeholders; locale-specific plural
rules (Cs has three forms) and number/date formats.

## 8.8 Testing

Reference: [07-testing](../architecture/07-testing.md).

Unit tests mock every trait boundary (mockall, `MockClockService` for time);
integration tests run real services against in-memory SQLite with migrations.
Mandatory coverage: deny-path tests for every permission gate (dev mock is
always admin, so RBAC denies are otherwise never exercised), double-count
tests for re-point operations, old-snapshot-read tests on schema-version
bumps. The full gate is `cargo build` → `cargo test` → clippy `-D warnings` →
offline-mode test → `nix build`; `cargo test` alone is insufficient.

## 8.9 API & DTO Discipline

Reference: [api/conventions](../api/conventions.md), [api/openapi](../api/openapi.md).

DTOs (`*TO`) live only in `rest-types` and are shared with the frontend —
renaming a field is deliberately a compile error on both sides. Handlers
convert TO ↔ domain type at the boundary. OpenAPI is generated at compile
time (utoipa) and served via Swagger UI; there is no hand-maintained schema.
Kebab-case URLs, no version prefix, UUID string IDs, ISO dates, enum variants
as exact wire strings, DELETE = soft-delete.

## 8.10 Scheduling & Export

Reference: [F11](../features/F11-export.md).

Two in-process cron mechanisms (intentional, historical): legacy `tokio-cron`
drives the carryover updates ([runtime 6.4](06-runtime-view.md#64-carryover-update-scheduled-job));
`tokio-cron-scheduler` drives the PDF export pipeline (render weekly plans
with printpdf → push via WebDAV/rustls to Nextcloud; config, horizon, and
cron expression stored in `pdf_export_config` with last-success/error
telemetry). iCal feeds are generated on request. Report/block templates render
server-side via Tera or MiniJinja (engine selectable per template).

## 8.11 Logging & Observability

`tracing` + `tracing-subscriber`; production builds use the `json_logging`
feature (structured logs for the systemd journal), dev uses `local_logging`.
The packaged `start.sh` pipes output through a duration-normalizing awk
filter for log parsing. No metrics endpoint; scheduler telemetry (PDF export
last success/error) is persisted in config rather than exported.
