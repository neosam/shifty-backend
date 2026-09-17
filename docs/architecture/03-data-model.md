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
- **Person** (v2.7, Phase 60) — identity bracket above the Employee
  aggregate and the Auth & Session aggregate: one `person` row per real
  human, referenced optionally by `sales_person.person_id` and
  `user.person_id`. It does not replace the Employee aggregate — it spans it.

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

SQLite enforces FKs only if `PRAGMA foreign_keys = ON` is active.
**Enforcement is active in Shifty** — but not because the project sets the
pragma. sqlx turns it on itself: `SqliteConnectOptions::default()` inserts
`foreign_keys = ON` into the pragma set applied to every connection
(`sqlx-sqlite-0.8.6/src/options/mod.rs:185`), and
`shifty_bin/src/main.rs:1645` connects via a plain
`SqlitePool::connect("sqlite:./localdb.sqlite3")` URL without overriding it.

This is exactly why a source grep is misleading here: searching `*.rs` and
`*.sql` for `foreign_keys` really does return zero hits — the pragma comes
from the driver, not from the codebase. Verified empirically against the
production DB file and again inside the test harness: `PRAGMA foreign_keys`
reads `1`, and inserting a `sales_person_user` row with an unresolvable
`sales_person_id` is rejected with `FOREIGN KEY constraint failed` (SQLite
code 787). Both FK declarations on `sales_person_user` parse correctly and
are listed by `PRAGMA foreign_key_list`, despite the missing comma between
them in `migrations/sqlite/20240506114107_add_sales_person.sql:22-23`.

What the FK check does **not** buy you: soft delete. A parent row that is
only marked `deleted` still exists as a row, so the FK stays satisfied while
every live query filtering `WHERE deleted IS NULL` sees a dangling reference.
For that class of invariant an FK is the wrong instrument — see
`docs/domain/edge-cases.md` §8.

Anything that must really hold beyond referential integrity has to be
expressed as a `UNIQUE` index or a `CHECK` — see the Phase-60 section below.

> **Correction (2026-09-17, Plan 60-04):** an earlier version of this section
> claimed FK declarations were "decorative documentation, not an enforced
> constraint [D-60-06]". That conclusion was wrong; it was drawn from the
> source grep alone and missed the sqlx default. D-60-06 — not switching FK
> enforcement on inside Phase 60 — still stands as a scope decision, but its
> premise about the current state does not.

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
| 2026-09 | v2.7 Phase 60 — `20260916000000_create-person-backfill.sql`: `person` identity parent table, nullable `person_id` on `sales_person` + `user`, two full UNIQUE indices, idempotent self-verifying backfill. |

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

## Phase 60 (v2.7) — Person identity parent table

Migration `20260916000000_create-person-backfill.sql` introduces the
identity foundation of the v2.7 person management for features
[F01](../features/F01-employee-management.md) and
[F12](../features/F12-auth-session.md). Phase 60 is schema + backfill
**only** — no DAO, no service, no endpoint, no visible behaviour change.

### `person` (identity parent)

One row per real human. `person` is the bracket that spans the
**Employee** aggregate (`sales_person`) and the **Auth & Session**
aggregate (`user`).

| Column | Type | Notes |
| --- | --- | --- |
| `id` | BLOB(16) PK | 16 random bytes from `randomblob(16)`, stored raw and read back via `Uuid::from_slice`. Deliberately **not** an RFC-4122 UUID — SQLite sets neither the version nibble nor the variant bits, and nothing in the system interprets the value as a versioned UUID. |
| `created` | TEXT | Creation timestamp. |
| `update_process` | TEXT | Origin marker — backfilled rows carry `phase-60-person-backfill`. |
| `update_version` | BLOB(16) | Audit column. |

Those four columns are the **complete** set:

- **No `deleted` column [D-60-04].** No person is deleted in v2.7, and an
  unused soft-delete column invites a half-implemented semantics. Should a
  later phase need "delete a person", the column is added there. This is a
  deliberate exception to the soft-delete convention above; it is locked by
  the schema test `person_table_has_no_deleted_column`.
- **No name columns.** Names stay where they are — `user.name` and
  `sales_person.name`. Real person master data (e-mail, phone, date of
  birth) is future scope (PERS-F-01), not v2.7.

### `sales_person.person_id` and `user.person_id`

Both columns are `BLOB(16)`, **nullable, without DEFAULT**.

The optionality is a **domain promise, not a transitional state
[D-60-03]**: a person may have a user **or** a sales person **or** both.
HR/admin accounts without a sales person and volunteers without a login
are equally valid, permanently supported shapes. Nothing anywhere
auto-creates the missing counterpart. The unique index below forbids
**two** carriers of the same side on one person — "no carrier" is the
normal case for two of the three shapes.

### Uniqueness — `sales_person_person_id_unique_idx`, `user_person_id_unique_idx`

Both indices are **full, not partial**: neither carries a
`WHERE deleted IS NULL` filter clause.

That deliberately deviates from the dominant repo pattern described under
*Schema conventions* (cf. the partial `rebooking_batch_week_unique_idx`).
Reason [D-60-01]: a soft-deleted sales person **keeps** its person identity
permanently. If deletion released the identity, two different humans could
occupy the same person identity over time, and write-once
`billing_period_sales_person` snapshots from years ago would resolve to the
wrong person. The price, deliberately accepted: a returning employee is two
persons at first; merging them is future scope, not v2.7.

**This decision is one-way.** After production use, switching to a partial
index would rebind identities that have already been handed out and
reattribute historical billing snapshots. A later clean-up pass must **not**
align these two indices with the partial-index convention. The regression
guards `sales_person_index_is_not_partial` and `user_index_is_not_partial`
read the stored DDL from `sqlite_master` and fail if the filter clause ever
appears. `user` has no soft-delete column at all, so the question does not
arise there [D-60-02].

### Enforcement: index, not foreign key

The 1:1 guarantee that the two `UNIQUE` constraints on `sales_person_user`
carry today now additionally lives in these two indices. It is enforced
**by the database**, not by an application check [D-60-05] — SQLite forbids
`UNIQUE` in `ALTER TABLE … ADD COLUMN`, hence the separate
`CREATE UNIQUE INDEX`.

The proof runs through the index, **not** through a foreign key — for a
different reason than originally recorded. `person_id` carries no
`FOREIGN KEY` declaration at all, so there is nothing for SQLite to enforce
on that edge; the uniqueness guarantee has to come from the index either way.
FK enforcement as such *is* active in this repo (sqlx sets the pragma per
connection — see *Schema conventions → Foreign keys* above). Adding an FK
declaration on `person_id` was out of Phase-60 scope [D-60-06].

### Backfill

A one-off, idempotent SQL backfill inside the same migration file (and
therefore inside one transaction):

- It deliberately carries **no** soft-delete filter — every historical
  `sales_person` row gets an identity, including soft-deleted ones (PERS-03).
  Otherwise write-once snapshots in `billing_period_sales_person` would later
  resolve to nothing.
- The idempotency key is the still-empty `person_id` column
  (`WHERE person_id IS NULL`); the `person` insert excludes rows that already
  exist. A second run changes neither row counts nor a single assignment
  (PERS-02).
- For a linked pair only the sales-person side draws a `randomblob(16)`; the
  user side **copies** that value back through a correlated subquery, so both
  sides of a pair point at the same person.
- At the end the migration **verifies itself** [D-60-07]: the count check
  `COUNT(person) = COUNT(sales_person) + COUNT(user) − COUNT(sales_person_user)`
  runs as a hard assertion inside the migration transaction (a throw-away
  `TEMP` table with a `CHECK` constraint, since SQLite only knows `RAISE()`
  inside triggers). If it does not hold, the whole migration rolls back and the
  server does not start — better than committing a half-executed backfill.
- Backfilled rows stay recognisable forever by `update_process =
  'phase-60-person-backfill'`.
- There is no down migration and no in-database backup table [D-60-09]. The
  recovery path is the copy of the SQLite file taken before the deploy.

### Scope boundary

- `sales_person_user` **stays** in Phase 60. It remains the running truth the
  backfill is checked against; dissolving it is Phase 61.
- Rows created **after** the migration (login auto-creation, `create_sales_person`)
  get **no** `person_id` in Phase 60 — assignment on creation is Phase-61 scope.
  There is no trigger and no DAO change in this phase [D-60-10]. The count check
  holds for the state *immediately after* the migration.
- `sales_person.id` and `user.name` are not re-keyed; both stay bit-identical
  (PERS-06).

### Snapshot contract

`CURRENT_SNAPSHOT_SCHEMA_VERSION` stays **13** — no bump. `person` is purely
administrative: no new persisted `value_type`, no changed computation, no
changed input set, and `billing_period_sales_person.sales_person_id` is
unchanged. None of the four bump clauses in `CLAUDE.md` applies, and
"bumping just in case" is explicitly wrong here — it would make every older
snapshot look like it was written under different rules.

## Related edge cases

- Soft-delete consistency → [`../domain/edge-cases.md#8-soft-delete-konsistenz`](../domain/edge-cases.md#8-soft-delete-konsistenz)
- `.sqlx` cache and migrations → [`../domain/edge-cases.md#10-migrations--sqlx-offline-cache`](../domain/edge-cases.md#10-migrations--sqlx-offline-cache)
