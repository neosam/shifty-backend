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

The complete list lives in `migrations/sqlite/` — file names are sorted
by their timestamp prefix.

## Where the data model "speaks"

- **Basic-service DAOs** know their table directly.
- **Business-Logic Services** know no DAOs beyond what is transitively
  reached through the Basic Services they consume.
- **Reporting** reads across several aggregates and is therefore the
  place where schema changes hit the balance calculation the fastest.

## Related edge cases

- Soft-delete consistency → [`../domain/edge-cases.md#8-soft-delete-konsistenz`](../domain/edge-cases.md#8-soft-delete-konsistenz)
- `.sqlx` cache and migrations → [`../domain/edge-cases.md#10-migrations--sqlx-offline-cache`](../domain/edge-cases.md#10-migrations--sqlx-offline-cache)
