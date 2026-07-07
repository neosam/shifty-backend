# Data Model

## Physical schema (ER)

The current ER model is generated automatically from the
`migrations/sqlite/` files and lives as a Mermaid diagram in
[`diagrams/db-schema-er.mmd`](./diagrams/db-schema-er.mmd).

## Logical model (aggregates)

The aggregate view — which objects belong together from a domain
perspective and how business rules draw the boundaries — is in
[`diagrams/domain-aggregates.mmd`](./diagrams/domain-aggregates.mmd).

Core aggregates:

- **Employee** — Sales Person + Work Details (Contract) + Unavailable + Vacation Offset.
- **Shiftplan** — Slots + Catalog + Special Days.
- **Booking** — Booking + Booking Log.
- **Absence** (v1.0+) — range-based absences.
- **Legacy Extra Hours** (pre-cutover) — single-day rows + custom categories.
- **Accounting** — Carryover + Vacation Balance + Report.
- **Billing Period** — snapshot with `snapshot_schema_version`.
- **Rebooking** (v2.6, Phase 54) — audit-traceable batches that
  reconcile capped-employee voluntary hours; see feature
  [F14](../features/F14-rebooking.md).
- **Week Metadata** — Week Status + Week Message.
- **Auth & Session** — User + Role + Session + Invitation.
- **System** — Feature Flag + Toggle + Scheduler.

## Schema conventions

### Soft delete

Almost every table has a `deleted` (nullable timestamp) column. Readers
**always** filter `WHERE deleted IS NULL`. This is a convention, not a
DB constraint — meaning reviewers must confirm that every new
`query!` / `query_as!` sets that filter.

### Time columns

- **`from` / `to` (Date):** Half-open ranges? Closed ranges? The
  convention varies per table. **[To verify]** in `absence`,
  `employee_work_details`, `sales_person`.
- **Timestamps:** stored SQLite-native. Timezone: **[To verify]**
  (UTC vs local Berlin).

### Foreign keys

SQLite enforces FKs only if `PRAGMA foreign_keys = ON` is active. Whether
Shifty sets this globally: **[To verify]** in the startup path.

### Views

Some aggregate reads go through views (`bookings_view` — migration
`20240728155625_add-bookings-view.sql`, extended with user tracking in
`20250115000001_update-bookings-view-add-user-tracking.sql`). Views are
convenient but shift with their underlying tables — mind the
migration order.

### Enum columns

Categories like `ExtraHoursCategory` are persisted as `TEXT` with the
enum names (e.g. `"ExtraWork"`, `"Vacation"`). New variants require both
a code change (enum extension + conversion) and a DB migration if a
constraint / check restricts the values.

**[To verify]** — whether there are explicit CHECK constraints on
category columns.

## Migration history (high level)

Chronologically ordered — details in the individual feature docs:

| Period | Milestone |
| --- | --- |
| 2024-04 | Users & roles, initial RBAC. |
| 2024-05 – 2024-07 | Slot + Sales Person + Booking + Booking view. |
| 2024-08 | `min_resources` column. |
| 2024-10 | Special-days table, weekday + vacation on working days. |
| 2024-11 | Session table, constraint tightening. |
| 2024-12 | Yearly carryover (hours + vacation). |
| 2025-01 | User tracking on bookings + view, week message. |
| 2025-04 | Custom extra hours. |
| 2025-08 | Billing period, text template. |
| 2025-10 | User invitation, session tracking. |
| 2026+ | Absence-system cutover, multi-plan support. |
| 2026-07 | v2.6 Phase 54 — `rebooking_batch` + `rebooking_batch_entry` tables, `extra_hours.source` marker column, voluntary-rebooking toggle seed. |

The complete list lives in `migrations/sqlite/` — file names are sorted
by their timestamp prefix.

## Where the data model "speaks"

- **Basic-service DAOs** know their table directly.
- **Business-Logic Services** know no DAOs beyond what is transitively
  reached through the Basic Services they consume.
- **Reporting** reads across several aggregates and is therefore the
  place where schema changes hit the balance calculation the fastest.

## Phase 54 (v2.6) — Rebooking data-model additions

Migration set `20260707000000..02` introduces the data-model
foundation for feature [F14](../features/F14-rebooking.md).

### `rebooking_batch` (parent)

Audit-traceable batch of paired `extra_hours` rows that reconcile
voluntary hours for a capped employee in one ISO-week.

| Column | Type | Notes |
| --- | --- | --- |
| `id` | BLOB(16) PK | UUID v4. |
| `sales_person_id` | BLOB(16) | Employee this batch reconciles. |
| `iso_year`, `iso_week` | INT | ISO-year + ISO-week of the reconciliation window. |
| `kind` | TEXT | `Manual` \| `HrSuggestion` \| `AutoCron` \| `AutoCronBackfill`. |
| `state` | TEXT | `Pending` \| `Approved` \| `Rejected` \| `SkippedLocked`. |
| `created`, `approved`, `approved_by` | TEXT | audit timestamps + username; `approved*` NULL until `state = Approved`. |
| `deleted` | TEXT nullable | soft-delete marker. |
| `update_process`, `update_version` | audit columns |

**Constraint [D-54-DM-01]** — partial UNIQUE index
`rebooking_batch_week_unique_idx` on
`(sales_person_id, iso_year, iso_week) WHERE deleted IS NULL` —
*global across all `kind`s*. This is Claim-on-Suggest: once HR opens a
Pending batch for week X, the Phase-56 F4 cron cannot race in a second
`AutoCron` batch for the same week. Two performance indices exist
alongside (`rebooking_batch_state_idx` on `state`,
`rebooking_batch_entry_sp_idx` on `sales_person_id` in the child
table).

### `rebooking_batch_entry` (child)

Per-slot payload for one batch, one row per hour bundle rebooked.

| Column | Type | Notes |
| --- | --- | --- |
| `id` | BLOB(16) PK | |
| `batch_id` | BLOB(16) FK → `rebooking_batch(id)` | No CASCADE — soft-delete pattern. |
| `sales_person_id` | BLOB(16) | Denormalised for query performance. |
| `hours` | REAL | Absolute hour count to rebook. |
| `balance_before` | REAL | Balance snapshot at suggestion time (audit). |
| `voluntary_actual`, `voluntary_committed` | REAL | Snapshot of F1 numerator + F2 pro-rata target at suggestion time. |
| `extra_hours_out_id`, `extra_hours_in_id` | BLOB(16) nullable | FKs into `extra_hours` — filled atomically at state → Approved (Phase 55 writers). |
| `created`, `deleted`, `update_process`, `update_version` | audit columns |

### `extra_hours.source` marker column ([D-54-DM-02])

Additive column `source TEXT NOT NULL DEFAULT 'manual'`. Two active
domain values:

- **`manual`** — every row from HR-CRUD, absence-convert, dev-seed and
  REST-TO writers. Existing rows migrate in via the column DEFAULT.
- **`rebooking`** — reserved for F3/F4/F5 writers (Phase 55+). In
  Phase 54 no writer sets this value.

Reader consequence: aggregates that must remain balance-neutral under
future rebooking pairs filter `source = 'manual'`. First live consumer
is `voluntary_ist_total_for_year(..)` in
`service_impl/src/reporting.rs`.

### Toggle seed `voluntary_rebooking_auto_active_from`

Migration `20260707000002_seed-voluntary-rebooking-toggle.sql`
idempotently seeds `INSERT OR IGNORE INTO toggle` with
`enabled = 0, value = NULL`. Dormant in Phase 54 — Phase 56's F4-cron
reads the ISO-date value to activate the AutoCron writer chain.

## Related edge cases

- Soft-delete consistency → [`../domain/edge-cases.md#8-soft-delete-konsistenz`](../domain/edge-cases.md#8-soft-delete-konsistenz)
- `.sqlx` cache and migrations → [`../domain/edge-cases.md#10-migrations--sqlx-offline-cache`](../domain/edge-cases.md#10-migrations--sqlx-offline-cache)
