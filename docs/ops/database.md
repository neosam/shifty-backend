# Database — Migrations, Cache, Operations

## SQLite — why and when not

Shifty runs on SQLite in production. Advantages:

- Deploy as a single file — no separate DB server.
- Trivial test isolation (in-memory DB).
- Reproducible migrations via `sqlx`.

Limits:

- **Single writer.** Two concurrent write requests are serialised.
- **No true concurrent read/write** without WAL mode. If load grows,
  thinking about Postgres may become necessary — but the DAO layer is
  trait-based, so a migration would be a DAO impl swap, not a service
  change.

## Migrations

**Location:** `shifty-backend/migrations/sqlite/`.
**Format:** `YYYYMMDDHHMMSS_<name>.sql` — sorted chronologically.

### Create a new migration

```bash
nix develop
sqlx migrate add --source migrations/sqlite <name-in-kebab-case>
```

This produces an empty `.sql` file with the correct timestamp.

### Run migrations

**Incremental (dev + prod):**

```bash
sqlx migrate run --source migrations/sqlite
```

Applies all migrations that have not yet been run.

**Destructive (dev only, when starting fresh):**

```bash
sqlx database reset --source migrations/sqlite
```

⚠️ **Wipes the DB completely.** Never in production. Fine for the local
dev DB, but obtain confirmation from the user before running it (memory
rule: destructive DB ops always require explicit confirmation).

## sqlx offline cache (`.sqlx/`)

CI runs with `SQLX_OFFLINE=true`. SQLx then falls back to the query
cache in the `.sqlx/` directory instead of requiring a real DB.

**Rule:** after every new `query!` / `query_as!` usage you MUST run

```bash
cargo sqlx prepare --workspace
```

That refreshes the cache. The cache must be committed alongside.

**If you forget:**

- Local incremental build stays green (cache still there).
- Clean build fails.
- `cargo test --doc` fails.
- CI fails.

Phase 33 discovered this the painful way with "why is CI red".

## Backup

**[To verify]** — backup strategy in production. Typical for SQLite:
`.backup` command via the `sqlite3` CLI, or file copy while the DB is
quiescent (only if no WAL files are active).

For critical deployments: periodically copy the `.sqlite` file
(including `-wal` and `-shm`) followed by a consistency check.

## Foreign Keys

SQLite only enforces FKs if `PRAGMA foreign_keys = ON` is set. **Verified
(2026-09-17, Plan 60-04): it is on.** Shifty's own code never sets the
pragma, but sqlx does — `SqliteConnectOptions::default()` puts
`foreign_keys = ON` into the pragma set applied to every connection
(`sqlx-sqlite-0.8.6/src/options/mod.rs:185`), and
`shifty_bin/src/main.rs:1645` connects through a plain
`SqlitePool::connect("sqlite:./localdb.sqlite3")` URL that does not override
it. Grepping the repo for `foreign_keys` therefore returns nothing and is
**not** evidence that enforcement is off.

How to re-check on a live file:

```sql
PRAGMA foreign_keys;              -- must return 1 on an app connection
PRAGMA foreign_key_check;         -- must return no rows
```

Operational consequence: a delete that would orphan a child row fails with
`FOREIGN KEY constraint failed` (SQLite code 787) rather than silently
producing an orphan — relevant e.g. for the admin user delete, which
hard-deletes from `user`.

The remaining blind spot is soft delete: a row marked `deleted` still exists,
so the FK check stays satisfied while every live query filtering
`WHERE deleted IS NULL` sees a reference that resolves to nothing. FK
enforcement does not protect that class of invariant — a migration review
should still confirm that new tables express it explicitly.

## Views

Some aggregate reads go through views:

- `bookings_view` — base view, regenerated multiple times whenever
  columns change (2024-07-28 initial, 2025-01-15 user tracking,
  2026-03-30 multi-plan).

**Rule:** when you change a column on a table, check whether views sit
on top. They then need to be recreated (in a migration).

## Soft-delete convention

Almost every table has `deleted` (nullable timestamp). Readers filter
`WHERE deleted IS NULL`. This is a convention, not a constraint.

For new queries in review: **check for `deleted IS NULL`**.

See [`../domain/edge-cases.md#8-soft-delete-konsistenz`](../domain/edge-cases.md#8-soft-delete-konsistenz).

## `PRAGMA journal_mode = WAL`

**[To verify]** — whether WAL mode is active. WAL improves concurrent
read-while-write capability. Without WAL, long reads block writers.

## Debugging & ad-hoc queries

For quick inspection:

```bash
sqlite3 <path-to-db.sqlite>
> .schema sales_person
> SELECT * FROM sales_person WHERE deleted IS NULL LIMIT 5;
> .quit
```

**Caution:** direct writes against the DB bypass business logic and
snapshot semantics. If you MUST write for a data fix: leave an explicit
comment in the change log, use a transaction, and verify reporting
consistency afterwards.
