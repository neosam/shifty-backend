## Context

The system currently has no concept of multiple shift plans. Slots, bookings, and sales persons all live in a single global namespace. The shift plan "view" is dynamically assembled from these entities by `ShiftplanService::get_shiftplan_week()`. There is no `shiftplan` table or entity.

The organization needs team-based shift plans (e.g., main store schedule, baking schedule, cleaning schedule) where some plans are purely organizational ("planning calendars") and should not count toward hour calculations or billing.

## Goals / Non-Goals

**Goals:**
- Introduce a `shiftplan` entity with CRUD operations
- Associate every slot with exactly one shift plan
- Support a `is_planning` flag to exclude planning-only calendars from hour reports
- Migrate existing data seamlessly (all current slots → default "main" plan)
- Rename `ShiftplanService` → `ShiftplanViewService` to free the name for the new CRUD service

**Non-Goals:**
- Per-plan permissions (permissions remain global for now)
- Deduplication of hours when an employee appears in multiple plans (out of scope)
- Frontend changes (co-deployed, handled separately)

## Decisions

### 1. New `shiftplan` table with FK on `slot`

A new `shiftplan` table stores plan metadata. The `slot` table gets a `shiftplan_id` foreign key.

**Why not a join table?** Slots belong to exactly one plan (1:N relationship). A join table would add complexity for an M:N relationship that isn't needed.

**Why not group by a string tag on slot?** A proper entity allows versioning, soft delete, and the `is_planning` flag -- consistent with the rest of the data model.

### 2. Nullable FK with backfill migration (Option A)

The migration adds `shiftplan_id` as a nullable column, then backfills it with the default plan's UUID.

**Why nullable?** SQLite does not support `ALTER TABLE ADD COLUMN ... NOT NULL` without a default value. Using a static UUID default is fragile. The nullable approach with an immediate `UPDATE` is clean and the service layer enforces non-null on all new slots.

**Alternative considered:** Recreating the slot table with NOT NULL constraint. This would require recreating all indexes and is more complex for no practical benefit since the service layer validates.

### 3. Rename ShiftplanService → ShiftplanViewService

The existing `ShiftplanService` (which assembles the weekly view) is renamed to `ShiftplanViewService`. The new `ShiftplanService` handles CRUD for the shiftplan entity.

**Why rename the existing one?** The CRUD service for the core entity deserves the canonical name. The view assembly is a derived operation and the "View" suffix makes this clear.

### 4. Report queries filter by `is_planning`

All `ShiftplanReportDao` queries join through `slot → shiftplan` and add `WHERE shiftplan.is_planning = 0`. This is transparent to the service layer.

**Why at the DAO level?** The filtering is a data concern. Keeping it in SQL means no performance overhead from fetching data only to discard it.

### 5. ShiftplanViewService and SlotService gain `shiftplan_id` parameter

`get_shiftplan_week` and `get_slots_for_week` require a `shiftplan_id` to scope queries. The REST endpoint changes from `/{year}/{week}` to `/{shiftplan_id}/{year}/{week}`.

**ShiftplanEditService does not need changes** -- it operates on individual slots which are already associated with a plan via their `shiftplan_id`.

### 6. New ShiftplanService follows existing patterns

The new service uses:
- `gen_service_impl!` macro for dependency injection
- `ShiftplanDao` trait with standard CRUD operations
- `ShiftplanEntity` in the DAO layer
- `Shiftplan` struct in the service layer with `From` conversions
- `ShiftplanTO` in rest-types with `ToSchema` derive
- `#[utoipa::path]` annotations on all REST endpoints

## Risks / Trade-offs

- **Nullable FK in production** -- The `shiftplan_id` column remains nullable at the DB level. Risk: a bug could insert a slot without a plan. Mitigation: service layer validation on create/update. The existing pattern (e.g., `min_resources` was also added as nullable via ALTER TABLE) shows this works in practice.

- **Breaking API change** -- The shiftplan week endpoint path changes. Risk: any external consumers break. Mitigation: frontend is co-deployed, and no external API consumers exist currently.

- **Double-counting hours** -- An employee booked in two non-planning plans has hours counted from both. Risk: inflated hour reports. Mitigation: explicitly out of scope; the current use case has distinct employee groups per plan.

## Migration Plan

1. Deploy migration: create `shiftplan` table, insert "main" plan, add `shiftplan_id` to slot, backfill all existing slots.
2. Deploy backend with new code (CRUD service, renamed view service, updated queries).
3. Deploy frontend simultaneously with updated API calls.
4. Rollback: revert backend + frontend, migration is backward-compatible (new column is nullable, old code ignores it).
