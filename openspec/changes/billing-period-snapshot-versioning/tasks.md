## 1. Scaffolding (compile only, no logic)

- [x] 1.1 Create new SQLx migration file under `migrations/sqlite/` named `<timestamp>_add-snapshot-schema-version-to-billing-period.sql` that runs `ALTER TABLE billing_period ADD COLUMN snapshot_schema_version INTEGER NOT NULL DEFAULT 1`
- [x] 1.2 Run `sqlx migrate run --source migrations/sqlite` against the local dev database so subsequent compile-time SQLx checks see the new column
- [x] 1.3 Add `pub snapshot_schema_version: u32` field to `BillingPeriodEntity` in `dao/src/billing_period.rs`
- [x] 1.4 Add `pub snapshot_schema_version: u32` field to the `BillingPeriod` service type in `service/src/billing_period.rs`
- [x] 1.5 Add `pub snapshot_schema_version: u32` field to `BillingPeriodTO` in `rest-types/src/lib.rs` (keep `ToSchema` derive so it appears in OpenAPI)
- [x] 1.6 Define `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 1;` in `service_impl/src/billing_period_report.rs` (top-level constant near the snapshot writer)
- [x] 1.7 Extend the SQLx `SELECT` queries in `dao_impl_sqlite/src/billing_period.rs` (and any related `query_as!`/`query!` sites that read `billing_period`) to include the new column, mapping it into the entity field
- [x] 1.8 Extend the SQLx `INSERT` query that creates a `billing_period` row to include `snapshot_schema_version`; bind a placeholder value of `0u32` for now (will be replaced in Phase 3)
- [x] 1.9 Add the field to all `From`/`Into` conversions between `BillingPeriodEntity` ↔ `BillingPeriod` ↔ `BillingPeriodTO`, propagating the value verbatim
- [x] 1.10 Verify `cargo build` from `shifty-backend/` succeeds with no warnings about unused fields

## 2. Tests (Red)

- [x] 2.1 In `service_impl/src/test/billing_period_report.rs` (create the file if absent), add an integration test asserting that creating a billing period via `build_and_persist_billing_period_report()` produces a row whose `snapshot_schema_version` equals `CURRENT_SNAPSHOT_SCHEMA_VERSION` — covers Spec Req 1, Scenario 1
- [x] 2.2 Add a SQLx test that loads a billing-period row that existed before the migration (seeded directly via raw SQL in the test setup) and asserts its `snapshot_schema_version` is `1` — covers Spec Req 1, Scenario 2
- [x] 2.3 Add a test asserting that the writer takes its version exclusively from the constant: feed two distinct billing-period creation calls and assert both rows carry `CURRENT_SNAPSHOT_SCHEMA_VERSION`, with no parameter or configuration switch able to influence the value — covers Spec Req 2
- [x] 2.4 Add a test that creates a billing period, soft-deletes it via the existing delete operation, and asserts `snapshot_schema_version` on the resulting row is unchanged — covers Spec Req 3
- [x] 2.5 Add a REST integration test in `rest/`'s test scaffolding that issues `GET /billing_period/{id}` against an existing billing period and asserts the response body contains `snapshot_schema_version` with the persisted value — covers Spec Req 4, Scenario 1
- [x] 2.6 Add a REST integration test that issues `POST /billing_period` with a request body that attempts to specify `snapshot_schema_version: 999`, then asserts the persisted row's value is `CURRENT_SNAPSHOT_SCHEMA_VERSION` — covers Spec Req 4, Scenario 2
- [x] 2.7 Run `cargo test` and confirm the new tests fail (writer still emits the stub `0` from Phase 1.8)

## 3. Implementation (Green)

- [x] 3.1 Replace the placeholder `0u32` in the snapshot writer (`build_and_persist_billing_period_report()` in `service_impl/src/billing_period_report.rs`) with `CURRENT_SNAPSHOT_SCHEMA_VERSION`
- [x] 3.2 Confirm the REST `POST /billing_period` handler in `rest/src/billing_period.rs` does not pass any client-supplied `snapshot_schema_version` value into the service layer (either the request body lacks the field or the handler explicitly drops it)
- [x] 3.3 Run `cargo test` and confirm all tests from Phase 2 now pass

## 4. Documentation Lock (non-test enforcement)

- [x] 4.1 Add a new `### Billing Period Snapshot Schema Versioning` section to `shifty-backend/CLAUDE.md` under "Key Development Notes". Section MUST name `CURRENT_SNAPSHOT_SCHEMA_VERSION`, list the change types that require a bump (added/removed/renamed `value_type`, changed computation of an existing `value_type`, changed input set), and state the reason (snapshot validation distinguishability) — covers Spec Req 5, Scenario 1
- [x] 4.2 In `shifty-backend/openspec/config.yaml`, add a sentence under `context:` (within the `## Conventions` block) that names `CURRENT_SNAPSHOT_SCHEMA_VERSION` and states the bump condition — covers Spec Req 5, Scenario 2 (context part)
- [x] 4.3 In the same `openspec/config.yaml`, add an entry under `rules.tasks` reminding task authors to include an explicit bump subtask whenever the change touches persisted `value_type`s or their computation — covers Spec Req 5, Scenario 2 (rules part)
- [x] 4.4 Incidental fix from design.md Open Questions: rename the `rules.spec` key to `rules.specs` in `shifty-backend/openspec/config.yaml` so the CLI no longer warns `Unknown artifact ID in rules: "spec"`
- [x] 4.5 Re-run `openspec instructions tasks --change billing-period-snapshot-versioning` and confirm the unknown-artifact warning has disappeared

## 5. Final verification

- [x] 5.1 Run `cargo build` from `shifty-backend/` — succeeds with no warnings
- [x] 5.2 Run `cargo test` from `shifty-backend/` — all tests pass
- [x] 5.3 Run `cargo run` briefly (a few seconds) to confirm the server starts and the migration applies cleanly on an empty database
- [x] 5.4 Manually inspect `GET /billing_period/{id}` via Swagger UI or curl against a freshly created billing period to confirm `snapshot_schema_version` appears in the JSON response
