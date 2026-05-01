## Why

Extra hours entries (overtime, vacation, sick leave, etc.) currently support only create and soft-delete; `Service::update()` is `unimplemented!()`. To correct a typo or wrong amount, users must delete the entry and create a new one — which works but loses the connection between the original entry and its correction. Users have asked for a real update flow, but only if no historical data is overwritten (the soft-delete invariant must hold).

## What Changes

- Add an `update()` operation on the extra hours service that preserves history by soft-deleting the previous row and inserting a new row that shares the same logical identity.
- Introduce a stable `logical_id` for extra hours entries that survives across updates, so that REST clients see a stable resource id over the lifetime of a logical entry.
- **BREAKING (internal trait only)**: fix the broken `Service::update` trait signature on extra hours, which is currently missing the `tx: Option<Self::Transaction>` parameter. No external/REST callers are affected.
- Use optimistic locking via the existing `version` field — concurrent updates that race on a stale version receive a 409 Conflict.
- Wire the existing `PUT /extra-hours/{id}` REST endpoint to the new service method (the endpoint is exposed but currently returns `unimplemented!()` indirectly).
- Define which fields are editable (`amount`, `category`, `description`, `date_time`, `custom_extra_hours_id`) and which are immutable on update (`sales_person_id`, `logical_id`).
- Snapshots and billed periods are not given special treatment — consistent with how `delete()` behaves today; eingefroren snapshots plus drift-detecting validators remain the system-wide invariant.

## Capabilities

### New Capabilities
- `extra-hours-update`: Mutability lifecycle for extra hours entries. Defines the logical-id pattern, the editable/immutable field surface, optimistic locking semantics, soft-delete preservation across updates, and the permission model for editing.

### Modified Capabilities
<!-- None. There is no pre-existing extra-hours capability spec to modify; the lifecycle is documented for the first time as part of this change. -->

## Impact

- **Database**: New migration adds `logical_id BLOB(16) NOT NULL` to `extra_hours` plus a partial unique index on `(logical_id) WHERE deleted IS NULL`. Existing rows are backfilled with `logical_id = id` (one-to-one, preserving the current public id).
- **DAO** (`dao` + `dao_impl_sqlite`): `extra_hours` DAO gains `logical_id` awareness — read paths look up active rows by `logical_id`, the create path stamps `logical_id` on new rows, and the update path inserts a new row carrying the supplied `logical_id`.
- **Service** (`service` + `service_impl`): `extra_hours` service trait `update` signature is corrected to include `tx: Option<Self::Transaction>`; concrete impl performs the soft-delete-old + insert-new flow inside a single transaction with optimistic-lock check on `version`.
- **REST** (`rest` + `rest-types`): `PUT /extra-hours/{id}` becomes functional. The DTO's `id` field is the `logical_id` from the caller's perspective; the underlying physical row id is an internal implementation detail. A new `409 Conflict` response variant is added for the optimistic-lock case.
- **Frontend (out of scope here)**: Existing PUT call sites become functional; no new UI required for this change.
- **Snapshot schema versioning**: No bump required — no `value_type` on `billing_period_sales_person` is added, removed, renamed, or recomputed by this change.
- **Out of scope**: `CustomExtraHours` already has working in-place updates and is intentionally left untouched. Frontend changes beyond the now-functional endpoint are not part of this change.
