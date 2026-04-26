## Why

`BillingPeriodSalesPerson` snapshots persist a set of `value_type` rows (e.g. `balance`, `vacation`, `sick_leave`) computed once at billing-period creation time and never updated afterwards. The set of expected `value_type`s is implicitly defined by whatever the reporting code wrote at the moment the snapshot was created.

Two concrete needs make this implicit contract a problem:

1. **A planned snapshot validation feature** wants to compare each persisted snapshot against a fresh live re-computation, to detect drift caused by retroactive booking edits, contract changes, or other after-the-fact modifications. Without an explicit per-snapshot record of *which schema was current when this snapshot was written*, the validator cannot distinguish between a missing value caused by an actual data problem and a missing value that is correct because the snapshot predates that field's introduction.

2. **A planned `volunteer-work-hours` change** (`weekly-planned-hours-cap` in this repo) will introduce a new `volunteer` `value_type`. Once that ships, every snapshot created before the change will be missing this row — for legitimate reasons. A schema version on the snapshot makes this a clean, self-describing fact instead of a guess based on the creation date or on the presence of specific rows.

Introducing the version *now*, before the new `value_type` lands, means every snapshot from this point forward carries an honest version number. Doing it later would require either backfilling versions from creation timestamps (spröde across deploys, environments, rollbacks) or detecting versions heuristically from row contents (an implicit hack baked into every reader).

This change is intentionally tiny: it adds annotation only. No reading code interprets the version yet. Future changes (validation, schema evolution) consume it.

## What Changes

- Add a new column `snapshot_schema_version INTEGER NOT NULL DEFAULT 1` to the `billing_period` table. Existing rows automatically receive `1` via the DEFAULT.
- Define a single constant `CURRENT_SNAPSHOT_SCHEMA_VERSION` (initial value `1`) in the reporting/billing-period service layer. All future `value_type` set changes bump this constant by one.
- Modify `build_and_persist_billing_period_report()` in `service_impl/src/billing_period_report.rs` to write `CURRENT_SNAPSHOT_SCHEMA_VERSION` into the new column when persisting a new snapshot.
- Extend `BillingPeriodEntity` (DAO) and `BillingPeriod` (service type) with a `snapshot_schema_version: u32` field. Read paths return it as-is from the database.
- Extend the relevant `BillingPeriodTO` transport object with a read-only `snapshot_schema_version` field for OpenAPI consumers.
- No change to runtime behaviour: nothing reads the version yet; the snapshot-validation and `volunteer-work-hours` changes will consume it.

## Capabilities

### New Capabilities

- `billing-period-snapshot-versioning`: Per-snapshot schema version stamped at creation time on `billing_period`, enabling future readers (validation, schema evolution) to interpret each snapshot against the schema that was current when it was written.

### Modified Capabilities

*(none)*

## Impact

- **Database**: Migration adds one `INTEGER NOT NULL DEFAULT 1` column to `billing_period`. No data migration; existing rows get `1` via DEFAULT.
- **DAO layer**: `BillingPeriodEntity` and the SQLite mapping in `dao_impl_sqlite` gain the new field on read and write paths.
- **Service layer**: `BillingPeriod` service type gains the field; `BillingPeriodReport` service writes `CURRENT_SNAPSHOT_SCHEMA_VERSION` on persistence.
- **REST layer**: `BillingPeriodTO` exposes the version in OpenAPI as read-only metadata.
- **i18n**: None (read-only technical metadata, not user-facing).
- **Frontend**: None required for this change (the value can be displayed later by consuming features).
- **Documentation / workflow**: `shifty-backend/CLAUDE.md` gains a "Billing Period Snapshot Schema Versioning" section; `openspec/config.yaml` gains a corresponding entry under `context:` and a rule under `rules.tasks` that reminds future authors to bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` whenever they touch persisted `value_type`s or their computation. These edits ship in the same commit as the schema change so the bump-enforcement is in place before the first downstream change consumes it.
- **Sequencing**: This change is a prerequisite for the planned snapshot-validation feature and is recommended to land before `weekly-planned-hours-cap` so that the first snapshot containing the new `volunteer` `value_type` already carries version `2` rather than version `1`.
