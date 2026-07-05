# Testing — Conventions & Gates

## Two Test Levels

- **Unit tests** — Mock the trait boundaries. No database, no HTTP
  access. Test runtime in milliseconds. Focus: a single business rule.
- **Integration tests** — In-memory SQLite, real DAOs, real services.
  Test entire aggregate flows: create booking → log written → report
  shows it correctly.

## Unit Tests with `mockall`

Every trait in `service/` and `dao/` has a `#[cfg_attr(test, automock)]`.
This generates `MockFooService` etc.

**Pattern:**

```rust
#[tokio::test]
async fn create_booking_denies_without_role() {
    let mut mock_perm = MockPermissionService::new();
    mock_perm.expect_require_role()
        .returning(|_, _| Err(ServiceError::Forbidden));

    let service = BookingServiceImpl::new(Deps {
        permission_service: mock_perm,
        // ...
    });

    let result = service.create(dto, auth, None).await;
    assert!(matches!(result, Err(ServiceError::Forbidden)));
}
```

Test files live under `service_impl/src/test/` — one module per
domain (`test/booking.rs`, `test/absence.rs`, …).

## Integration Tests

**In-memory SQLite** is started via SQLx setup with `sqlite::memory:`.
Migrations run at test startup. After that: real services over real
DAOs.

**Helper traits** like `NoneTypeExt` compose `Authentication` contexts
for tests concisely.

## Test Coverage Expectation

- **New business rule:** Always a unit test that checks the rule in
  isolation (including the deny cases).
- **New REST endpoint:** OpenAPI annotation + at least one integration
  test for the happy path plus one error case.
- **New migration with semantic change:** Regression test that proves
  old data continues to work after the migration.
- **Re-point op (slot split, booking migration):** Test against
  double-counting in the report. **Mandatory**, see
  [`../domain/edge-cases.md#7-transaktionen--atomarität`](../domain/edge-cases.md#7-transaktionen--atomarität).
- **Snapshot generation with a new `value_type`:** Bump
  `CURRENT_SNAPSHOT_SCHEMA_VERSION` and add a test that the old
  snapshot can still be read.

## Test Gates (what CI and `nix build` enforce)

The pipeline consists of several stages. **Only the last one is
binding:**

| Stage | What runs | Enough? |
| --- | --- | --- |
| `cargo build` | Compiles | **No** |
| `cargo test` | Runs unit + integration | **No** |
| `cargo clippy --workspace -- -D warnings` | Lint check | **No** alone |
| `SQLX_OFFLINE=true cargo test` | Test with offline cache | **No** alone |
| **`nix build`** | All stages + reproducibility check | **Yes** |

**Important:** `cargo test` alone is **not** enough. `nix build`
enforces `cargo clippy -- --deny warnings`. Every phase gate (including
autonomous phase execution) MUST run `cargo clippy --workspace -- -D warnings`
alongside, otherwise the final build fails.

See `.github/workflows/rust.yml` for the CI definition.

## sqlx Offline Cache

CI runs with `SQLX_OFFLINE=true`. SQLx then falls back to the
`.sqlx/` cache instead of a real database.

**Rule:** After every new `query!`/`query_as!` usage, run
`cargo sqlx prepare --workspace` and commit the `.sqlx/` cache.

If you forget:

- Incremental build may be green (cache still there).
- Clean build (CI) fails.
- `cargo test --doc` fails (uses a different target).
- Phase 33 found this with "why is CI red even though everything is green".

## Toolchain Split (Backend vs Frontend)

The backend workspace uses a different Rust toolchain and a different
clippy level than `shifty-dioxus/`:

- Backend: Strict, `clippy -D warnings`.
- Frontend: **Excluded** from the backend CI clippy — contains ~198
  pre-existing lints tracked as backlog.
- Clippy in the dioxus shell is additionally functionally broken (E0514)
  and must be run from the backend shell if you need it.

**Consequence:** New frontend lints drift unnoticed. If you work in
the frontend, run clippy manually — no gate runs it.

## Test Isolation

- **Do not parallelize if DB fixtures are shared.** SQLite in-memory
  DBs are isolated per test; only when a test fixture is explicitly
  shared (which is not the case in Shifty) can race-condition tests
  arise.
- **Time-sensitive tests:** Use `MockClockService` to set "today"
  deterministically.

## What is NOT Tested

- **RBAC deny paths in dev.** Because `mock_auth` is always Admin,
  deny paths never fire in E2E dev. **[Convention]** Explicit unit
  tests for "not admin, only role X" are mandatory.
- **UI E2E.** There is no automated browser test suite. Frontend
  changes are verified manually; for critical flows, use browser
  automation when in doubt (see [`06-frontend.md`](./06-frontend.md)).
- **Load / concurrency.** No systematic load test.
