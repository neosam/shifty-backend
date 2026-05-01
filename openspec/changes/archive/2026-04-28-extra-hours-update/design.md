## Context

The `extra_hours` table tracks per-sales-person time entries (overtime, vacation, sick leave, holidays, custom categories). Today the lifecycle is:

- `create`: insert new row with a fresh `id`, fresh `version`, `created = NOW()`.
- `delete`: in-place mutation that sets `deleted = NOW()` (soft delete). The `dao::ExtraHoursDao::update` DAO method is reused for that single field flip.
- `update` (the user-facing concept): not implemented — `Service::update` returns `unimplemented!()` and its trait signature is malformed (missing `tx` parameter).

Users want to correct mistakes (typo in description, wrong amount, wrong date) without losing the original. The system-wide invariant is: **soft-delete only — never overwrite historical data in place**. That invariant must hold for the new update flow too.

A second constraint: persisted billing period snapshots are intentionally frozen, and validators compare them against live re-computation to surface drift. This means it is acceptable for an update to silently disagree with an older snapshot — that is how snapshot drift is supposed to be reported. No special blocking is required.

## Goals / Non-Goals

**Goals:**
- Users can edit a previously-created extra hours entry through `PUT /extra-hours/{id}` and observe a stable resource id across edits.
- Every prior version of an entry remains in the table as a soft-deleted row, recoverable for audit and forensics.
- Concurrent updates on the same entry are detected and rejected with a clear error (409 Conflict), not silently overwritten.
- The on-the-wire DTO shape stays unchanged — `id` and `$version` already exist on `ExtraHoursTO`.

**Non-Goals:**
- No history view / audit UI — that is a separate feature; we only ensure the data is preserved for it.
- No cross-row "undo" or version pinning — the active row is always the one with `deleted IS NULL` for a given `logical_id`.
- No re-snapshotting of billing periods after an update; drift detection stays the responsibility of existing validators.
- No changes to `CustomExtraHours`, which uses a different (in-place + reactivation) update model and is left as-is.
- No cross-user "transfer" semantics; changing `sales_person_id` is not an edit.

## Decisions

### Decision 1: Logical-id pattern over alternatives

Three patterns were considered for "update without losing history":

- **A: Append-only with new `id` per edit.** Simplest. But the externally-visible REST id changes on every edit, breaking the `PUT /resource/{id}` contract.
- **B: In-place mutation with a separate history table.** Stable id, but doubles the storage paths (writes to two tables) and breaks the system's existing "one table, soft-delete" pattern.
- **C: Logical id (chosen).** Stable external id via a `logical_id` column. Active row = the one row per `logical_id` with `deleted IS NULL`. All other rows for the same `logical_id` are tombstones representing prior versions.

C wins because it keeps a single table, preserves the soft-delete invariant exactly as the rest of the codebase uses it, and gives the REST layer a stable id to surface as `id` in the DTO. The cost is one new column plus a partial unique index.

### Decision 2: Externally exposed `id` IS the `logical_id`

The DTO field `id` carries the `logical_id`. The physical row id is internal. This means:

- `GET /extra-hours/{id}` resolves `id` against `logical_id WHERE deleted IS NULL`.
- `PUT /extra-hours/{id}` resolves the active row by `logical_id`, soft-deletes it, inserts a new row carrying the same `logical_id`.
- `DELETE /extra-hours/{id}` resolves the active row by `logical_id` and sets `deleted = NOW()`.
- `POST /extra-hours` mints a fresh UUID and stores it as both `id` and `logical_id` of the first row.

For backfilled rows, `logical_id = id` already holds, so existing GET/DELETE callers see no behavior change.

### Decision 3: Optimistic locking via existing `version`

`extra_hours.version` (UUID) already exists and is rotated on `create`. The update flow rotates `version` again for the new row. The client's `PUT` body must carry the `$version` it last read; the service compares this against the active row's current `version`. On mismatch, return `ServiceError::OptimisticLockConflict` (or analogous existing error) → REST maps to `409 Conflict`. On match, proceed.

This matches the pattern already established in `CustomExtraHours::update`, so no new error-handling category needs to be invented.

### Decision 4: Editable vs immutable fields

- **Editable**: `amount`, `category`, `description`, `date_time`, `custom_extra_hours_id`.
- **Immutable**: `sales_person_id` (a different sales person is a different booking, not an edit), `logical_id` (identity), `id` (per-row, server-assigned), `created` (per-row, server-assigned), `version` (server-rotated).

If the request DTO carries a different `sales_person_id` than the active row, the service rejects with `ServiceError::ImmutableField` (or equivalent), not silently with a successful re-write.

### Decision 5: `created` of the new row = `NOW()`, not preserved from the predecessor

The `created` column means "when was this row written". Re-using the predecessor's `created` would conflate "first time the user logged this entry" with "when this specific row was inserted" and would defeat the audit usefulness of `created`. The first row's `created` (on the original tombstone, found by `WHERE logical_id = ? ORDER BY created ASC LIMIT 1`) remains the source of truth for "when did this entry first appear", but it is not propagated to subsequent rows.

### Decision 6: No special handling for billed / snapshotted periods

`delete()` today does not check whether the entry sits in a billed period. The system's contract is: snapshots are frozen at write time and validators surface drift. Update follows the same rule. If users want to forbid edits to billed periods, that is a separate, system-wide policy decision that should be applied uniformly to delete and update — not introduced as a one-off here.

### Decision 7: Permission model identical to `create`

`HR_PRIVILEGE` OR the user is the same sales person as `extra_hours.sales_person_id`. This matches `create` and existing `delete`. Editing your own entry is allowed; editing someone else's requires HR.

### Decision 8: Single transaction for the soft-delete + insert

Both row writes happen in the same transaction obtained via `transaction_dao.use_transaction(tx)`. If the insert fails, the soft-delete rolls back. This prevents the orphan state of "old row tombstoned, no replacement active row".

### Decision 9: Migration backfill strategy

`logical_id` is added as `NOT NULL`. Backfill in the same migration: `UPDATE extra_hours SET logical_id = id`. SQLite's `ALTER TABLE ADD COLUMN` cannot add a `NOT NULL` column without a default — so the migration adds the column nullable, backfills, then enforces non-null via table rebuild (`CREATE TABLE extra_hours_new ... ; INSERT INTO extra_hours_new SELECT ... ; DROP TABLE extra_hours; ALTER TABLE extra_hours_new RENAME TO extra_hours;`) plus recreating dependent indexes/views. The partial unique index `(logical_id) WHERE deleted IS NULL` is created after the backfill so it does not blow up on duplicate-tombstone scenarios that could appear in pre-existing data.

## Risks / Trade-offs

- **Risk: Backfill of existing rows produces conflicting tombstones for the partial unique index.** Pre-existing data has each row with its own unique `id`, so `logical_id = id` cannot collide. Mitigation: the partial unique index restricts to `WHERE deleted IS NULL`; tombstones are excluded from uniqueness anyway, so there is nothing to clash with. Verified by inspection of the migration script in tasks.

- **Risk: Future code paths look up by physical `id` instead of `logical_id` and silently miss.** Read paths after this change resolve external `id` against `logical_id`, so a stray `find_by_physical_id` call could return a tombstone or skip the active row. Mitigation: name the DAO method clearly (`find_by_logical_id`), and keep `find_by_id` either deleted or repurposed; document in the spec that the DAO contract for "the entry called X" is "active row with `logical_id = X`".

- **Risk: Optimistic-lock check confuses physical-row version with logical-entry version.** Each row has its own `version`. The check must read the *active* row (by `logical_id`, `deleted IS NULL`) and compare its `version` to the request's `$version`. Mitigation: the service first resolves the active row, then compares — never compares against a tombstone.

- **Trade-off: Snapshot drift is allowed.** Updating an entry inside a billed period silently disagrees with the snapshot. This is the existing system contract (validators detect drift). It is a feature, not a bug, but it must be communicated in the spec so reviewers understand the choice.

- **Trade-off: Storage growth.** Every edit adds a new row instead of mutating in place. For typical usage (occasional corrections), the growth is negligible compared to the create-rate of bookings; no archival strategy is required at this stage.

## Migration Plan

1. Ship the SQL migration (column add + backfill + table rebuild for `NOT NULL` + partial unique index recreation).
2. Ship the DAO/service/REST changes in the same release; the new column is read by both the rebuilt DAO query and the active-row resolution code.
3. No data backfill outside the migration is needed.
4. **Rollback**: revert the application binary; the column remains in place but is harmless (nullable equivalents are not part of any other code path). A second migration to drop the column is not necessary unless an explicit rollback is desired.
