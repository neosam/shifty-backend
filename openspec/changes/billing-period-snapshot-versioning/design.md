## Context

`BillingPeriodSalesPerson` rows are write-once snapshots produced inside `build_and_persist_billing_period_report()` (see `service_impl/src/billing_period_report.rs`). Each row carries a `value_type` (`balance`, `vacation`, `sick_leave`, …) and the corresponding numeric values. The set of `value_type`s present in a snapshot is defined implicitly by the reporting code that was current when the snapshot was written.

Two concrete consumers will need to know which schema a given snapshot was written against:

- A planned snapshot validation feature that re-runs the live computation and diffs against the stored snapshot. Without an explicit schema marker per snapshot, a missing `value_type` in an old snapshot is indistinguishable from a missing `value_type` in a buggy new snapshot.
- The next change in flight, `weekly-planned-hours-cap`, which adds a `volunteer` `value_type`. Once that lands, every snapshot created before it will be missing this row — for legitimate reasons.

The change is intentionally minimal: it adds a single `INTEGER` column on `billing_period`, a single source-of-truth constant in code, one write site, and entries in two documentation files. No reading code interprets the version yet.

The reason this small change deserves a `design.md` at all is the bump-enforcement question: once the constant exists, how do we make sure future authors bump it when they need to? That question has more than one defensible answer and is decided here.

## Goals / Non-Goals

**Goals:**

- Stamp every newly persisted `BillingPeriod` row with the schema version that was current at write time.
- Backfill existing rows to version `1` via a column DEFAULT — no separate data migration.
- Make the "you must bump on schema change" rule visible at the points where authors typically work: in the project-wide `CLAUDE.md` (general code work) and in the OpenSpec `config.yaml` (spec-driven changes).
- Keep the change purely additive: no behaviour change, no new dependency, no new pattern.

**Non-Goals:**

- Reading or interpreting the version. That is the job of the snapshot validation feature.
- Per-`value_type` versioning. The version applies to the whole snapshot record. Finer granularity is not required by either consumer.
- Backfilling versions for rows where the "true" schema differed from version `1`. There is no such case today; versioning starts here.
- Build- or CI-time enforcement that the constant gets bumped (see Decisions §4).

## Decisions

### 1. Version lives on `billing_period`, not on `billing_period_sales_person`

Add `snapshot_schema_version INTEGER NOT NULL DEFAULT 1` to the `billing_period` table.

**Why on the parent and not on each sales-person row:** A snapshot is a single point-in-time event for the whole billing period; all `billing_period_sales_person` rows for the same period are written together by `build_and_persist_billing_period_report()` and necessarily share the same schema. Storing the version on each child row would duplicate the value `N` times per period without adding any expressible state.

**Alternative considered:** Column on `billing_period_sales_person`. Rejected as redundant.

### 2. `INTEGER` with monotonic increments, starting at `1`

The version is a plain monotonically increasing integer, not a semantic version, not a content hash.

**Why a plain integer:** A snapshot's schema is either equal to a previous version or different from it; intermediate notions (major/minor) add no useful information for the validator, which only needs an exact lookup of "what value_types and what semantics were current at version N".

**Alternative considered:** Content hash of the schema. Rejected because hashes do not order, are unreadable in queries, and would force the validator to maintain a hash-to-schema map without a stable identifier to refer to.

### 3. Single source of truth in code: `CURRENT_SNAPSHOT_SCHEMA_VERSION`

Define one constant in the service layer next to where snapshots are produced (`service_impl/src/billing_period_report.rs` or a dedicated `snapshot_schema.rs` sibling — exact module placement is a tasks-level detail, but the constant is a single `pub const u32`). `build_and_persist_billing_period_report()` reads this constant and writes its value into the new column.

**Why a single constant rather than a per-call argument:** The version describes the code, not the call. Threading it through call sites would invite drift between writers in different modules. A single `const` keeps the invariant *"every write of this code goes to this version"* mechanically true.

**Why in `service_impl` rather than `service`:** The constant is an implementation detail of the persisting code, not part of the trait contract. Service consumers do not need to know about it.

### 4. Bump enforcement via documentation, not via test

The rule "if you change persisted `value_type`s or their semantics, bump `CURRENT_SNAPSHOT_SCHEMA_VERSION`" is enforced by entries in:

- `shifty-backend/CLAUDE.md` — a new "Billing Period Snapshot Schema Versioning" section under "Key Development Notes".
- `shifty-backend/openspec/config.yaml` — an additional sentence in `context:` (so every opsx invocation sees it) and an additional rule under `rules.tasks` (so every implementation task list is reminded).

**Why documentation and not a `cargo test` lock:** In this project a substantial share of code is written or reviewed with Claude in the loop. Prompt-time reminders catch the mistake *before* the change is written; a test would only catch it after the fact and would still require the author to translate "test failed" into "bump the constant". The documentation lock also explains *why* the rule exists, which a failing assertion cannot.

**Alternative considered:** A two-part `cargo test` lock — a structural test asserting the set of `value_type`s matches a hardcoded list, and a semantic test asserting a reference snapshot matches a hardcoded reference output. Rejected as redundant: any legitimate bump requires updating the hardcoded lists *anyway*, so the test mostly enforces "you remembered to update the test", not "you remembered to bump the version". The marginal value over a documentation lock did not justify the maintenance surface.

**Bounded downside of relying on documentation:** A missed bump produces a snapshot that *says* it is version `N` but is actually version `N+1`. The planned snapshot validation feature will detect this on its first run against such a snapshot — the failure mode is delayed, not undetectable. This is acceptable given how rare schema-touching changes are.

### 5. TO exposure as read-only metadata

`BillingPeriodTO` gains a `snapshot_schema_version: u32` field. The field is read-only from the API perspective (no client ever writes it; `POST /billing_period` continues to take no version input). Exposing it in OpenAPI is preparation for the validation feature's UI and lets API consumers reason about which snapshots they can compare safely.

**Why expose now rather than wait for the consumer:** Adding a field is an additive, non-breaking API change; deferring it would force `weekly-planned-hours-cap` or the validation feature to ship a coupled API change. Better to land the metadata once.

### 6. No backfill beyond the column DEFAULT

All existing `billing_period` rows receive `snapshot_schema_version = 1` via the column's `DEFAULT 1`. We do *not* attempt to retroactively determine whether any of them "really" used a different schema, because the constant did not exist before this change — by definition, every prior snapshot was written against what we now retroactively call version 1.

## Risks / Trade-offs

- **[Documentation lock can be missed by an inattentive author]** → Mitigation: redundancy across CLAUDE.md and openspec config maximises the chance that whichever workflow the author is in (general coding, spec-driven change) surfaces the rule. The planned snapshot validator catches misses on first run against an affected snapshot.
- **[Constant lives in `service_impl`; downstream consumers needing to know "what schema version is current" cannot import it without reaching into the impl crate]** → Acceptable: the only legitimate consumer that needs the *current* version is the writer itself. Validation features compare against *stored* versions read from the database, not against the live constant.
- **[Adding the column requires a SQLx schema regeneration locally before the next `cargo build`]** → Standard project workflow; documented in `shifty-backend/CLAUDE.md`. No new burden.
- **[Write path becomes a touch more verbose]** → One extra field in one struct; trivial.

## Migration Plan

1. **Schema migration:** add `snapshot_schema_version INTEGER NOT NULL DEFAULT 1` to `billing_period`. Existing rows automatically receive `1`.
2. **Constant:** introduce `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 1;` in `service_impl` near the snapshot writer. Module-level placement is a task-level detail.
3. **Entity / service / TO threading:** add the field on `BillingPeriodEntity`, the `BillingPeriod` service type, and `BillingPeriodTO`. Update SQLite read/write mappings in `dao_impl_sqlite`.
4. **Write site:** `build_and_persist_billing_period_report()` writes `CURRENT_SNAPSHOT_SCHEMA_VERSION` into the new field when constructing the `BillingPeriod` to persist.
5. **Documentation lock:** add the new section to `shifty-backend/CLAUDE.md` and the corresponding entries to `openspec/config.yaml` (`context:` and `rules.tasks`).
6. **Tests:** an integration test that creates a billing period and asserts the persisted row carries `snapshot_schema_version = CURRENT_SNAPSHOT_SCHEMA_VERSION`. (This is a one-line correctness test for the write path, not the rejected schema-lock test.)
7. **Rollback:** drop the column via a follow-up migration. No data is lost (the field had no consumers in this change).

## Open Questions

None. The placement of the constant inside `service_impl` (single file vs. dedicated `snapshot_schema.rs` sibling) is left as a task-level implementation detail.

While editing `openspec/config.yaml`, the existing `rules.spec` key — which the CLI rejects with `Unknown artifact ID in rules: "spec". Valid IDs for schema "spec-driven": design, proposal, specs, tasks` — should be renamed to `rules.specs`. This is an incidental fix, not part of this change's intent, but is mentioned here so the implementation task can pick it up rather than leaving a known-broken config in place.
