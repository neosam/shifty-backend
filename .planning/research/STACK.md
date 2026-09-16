# Stack Research

**Domain:** Rust/Axum/SQLx backend — additive schema migration (new `person` aggregate table + nullable FK backfill) and additive REST DTO field (`planning target hours`)
**Researched:** 2026-09-16
**Confidence:** HIGH (every claim below is grounded in a file this agent actually read in this repo)

## Headline Finding: No New Dependencies Needed

Both v2.7 features are expressible entirely with what's already in the workspace. This
is the single most important fact for the roadmapper: **do not add an ORM, a migration
framework, a UUID crate, or a schema-diffing tool.** Everything required —
raw-bytes UUID generation, nullable FK columns, additive backfill SQL, additive
DTO fields — has a direct, working precedent already committed in this repo (cited
below). Confirmed via:

- `Cargo.toml` (repo root, lines 1–10) — workspace members are `rest`, `service`,
  `service_impl`, `shifty_bin`, `dao`, `dao_impl_sqlite`, `rest-types`,
  `shifty-utils`. No ORM crate (diesel, sea-orm) anywhere in the workspace.
- `dao_impl_sqlite/Cargo.toml`, `dao/Cargo.toml`, `rest-types/Cargo.toml`,
  `service_impl/Cargo.toml`, `shifty_bin/Cargo.toml` — read in full; `sqlx = "0.8.2"`,
  `uuid = "1.8.0/1.8"` (feature `v4`, and `serde` in `rest-types`), `serde = "1.0"`,
  `utoipa = "5"`, `time = "0.3.36"` are already present in every crate that would
  touch this feature.
- `shifty_bin/src/main.rs:1651`, `dao_impl_sqlite/src/migration_source.rs:111`,
  `shifty_bin/src/integration_test.rs:283` — all three call
  `sqlx::migrate!("./../migrations/sqlite")` / `("../migrations/sqlite")`. Migrations
  run automatically at startup via the `sqlx` macro; there is no separate
  migration-runner tool to introduce.
- `.sqlx/` directory exists at repo root (query cache files, e.g.
  `.sqlx/query-00173731e94f6d830f379f558dbe3778f20c997a82bfa357f65b3a7624b322e2.json`)
  — confirms `sqlx-cli`'s offline-mode workflow (`cargo sqlx prepare`) is already
  the established tool for compile-time-checked queries. Any new hand-written
  `query!`/`query_as!` calls in a `PersonDao` must be followed by re-running
  `cargo sqlx prepare` (or the Nix build's `SQLX_OFFLINE` step will fail) — this is
  existing workflow, not a new tool.

## Recommended Approach (no new packages)

### 1. Adding nullable FK columns to `sales_person` and `user`

**Established technique, directly reusable, zero risk:** a plain
`ALTER TABLE ... ADD COLUMN ... REFERENCES ...(id)` with no `NOT NULL`. SQLite's
`ALTER TABLE ADD COLUMN` restriction against `NOT NULL` columns without a
non-NULL default does **not** apply here, because the new `person_id` columns
are nullable per the seed decision (`.planning/seeds/personenverwaltung-zusammenlegen.md`,
"Variante A": `sales_person.person_id BLOB(16) NULL`, `user.person_id BLOB(16) NULL`).

Two migrations already in this repo do exactly this — add a nullable, FK-bearing
column to a live, populated table, in a single `ALTER TABLE` statement, no table
rebuild:

- `migrations/sqlite/20260330000000_add-shiftplan-table.sql:25` —
  `ALTER TABLE slot ADD COLUMN shiftplan_id blob(16) REFERENCES shiftplan(id);`
  (nullable BLOB(16) FK onto an existing populated table — the exact shape needed
  for `sales_person.person_id`).
- `migrations/sqlite/20251017044013_add-session-tracking-to-user-invitation.sql:5-6` —
  `ALTER TABLE user_invitation ADD COLUMN session_id TEXT REFERENCES session(id)
  ON DELETE SET NULL;` (nullable TEXT FK — the shape needed for `user.person_id`,
  modulo the column type).

**Conclusion:** `sales_person.person_id` and `user.person_id` can each be added
with a one-line `ALTER TABLE ADD COLUMN`. No `CREATE TABLE new / INSERT SELECT /
DROP / RENAME` rebuild is required, because the new columns are nullable — SQLite's
rebuild-forcing restrictions (no `PRIMARY KEY`/`UNIQUE` on the new column, `NOT
NULL` needs a non-NULL default) simply don't trigger.

### 2. Creating the `person` table

Plain `CREATE TABLE person (id BLOB(16) NOT NULL PRIMARY KEY, ...)` following the
existing audit-column convention on `sales_person`
(`migrations/sqlite/20240506114107_add_sales_person.sql`): `update_timestamp TEXT`,
`update_process TEXT NOT NULL`, `update_version blob(16) NOT NULL`. No new
column types or extensions needed — SQLite's dynamic typing already stores
`BLOB(16)` UUIDs the same way `sales_person.id` and `extra_hours.id` do today.

### 3. Dissolving `sales_person_user`

A plain `DROP TABLE sales_person_user;` at the end of the migration, once its
data has been folded into the `person` backfill (see below). No FK-enforcement
blocker was found: no other table's schema in `migrations/sqlite/*.sql`
references `sales_person_user`, and no `PRAGMA foreign_keys = ON` /
`SqliteConnectOptions::foreign_keys(...)` call exists anywhere in
`shifty_bin/src/main.rs` or `dao_impl_sqlite/src/*.rs` (grepped, no hits) — so
SQLite FK constraints are not being actively enforced by this app's connection
setup in the first place. This is a minor observation, not load-bearing for the
migration technique, but worth a note for the phase's discuss-step: it means
orphaned FK values wouldn't be rejected at the DB layer even if introduced by
mistake — validation discipline in the backfill SQL itself matters more than DB
enforcement here.

### 4. The backfill (person rows + merge of paired rows)

**Do this as raw SQL inside the migration file, not as an application-code
migration step.** This repo has both patterns in its history, and the
distinguishing factor is business-logic complexity, not table count:

- **Simple, mechanical backfills run as SQL in the migration file itself.**
  Precedent: `migrations/sqlite/20260428101456_add-logical-id-to-extra-hours.sql`
  — add nullable column, `UPDATE extra_hours SET logical_id = id;`, then rebuild
  the table to enforce `NOT NULL`. Also
  `migrations/sqlite/20260707000001_add-source-column-to-extra-hours.sql` — a
  column with a `DEFAULT` value stands in for a backfill sweep entirely ("kein
  Backfill-Sweep noetig").
- **Backfills with real business logic (categorization, ambiguity resolution,
  date-range splitting) run as Rust application/service code**, with dedicated
  bookkeeping tables. Precedent: the `extra_hours` → `absence_period` cutover
  used `service_impl/src/absence_conversion.rs` plus
  `dao/src/migration_source.rs` / `dao_impl_sqlite/src/migration_source.rs`, and
  two dedicated SQL-only bookkeeping tables
  (`migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql`,
  `...20260503000001_create-absence-period-migration-source.sql`) to record what
  happened and quarantine ambiguous rows for manual HR resolution.

**The `person` backfill belongs in the first camp.** The merge rule is
mechanical and already provably unambiguous — the seed doc states this
explicitly: *"Die beidseitigen UNIQUE-Constraints der alten Tabelle garantieren,
dass dabei keine Mehrdeutigkeit entstehen kann"* (`sales_person_user` has
`UNIQUE(sales_person_id)` **and** `UNIQUE(user_id)`, confirmed at
`migrations/sqlite/20240506114107_add_sales_person.sql:14-22`). There is no
categorization, no date splitting, no manual-review case — every pair maps to
exactly one shared `person` row, every unpaired `sales_person`/`user` gets its
own. That means a quarantine table and a Rust cutover service (the pattern used
for the harder absence-migration problem) would be over-engineering here; three
or four `INSERT ... SELECT` / `UPDATE ... SET person_id = (SELECT ...)`
statements in the migration file are sufficient and match the repo's own
proportionality precedent (`extra_hours.logical_id` did its backfill in three
lines of SQL because the rule was equally mechanical: `logical_id = id`).

**UUID generation inside the migration:** use `randomblob(16)` directly (raw 16
bytes), **not** the `V_UUID_V4` view defined in
`migrations/sqlite/20240426150045_user-roles.sql:90-98`. That view formats a
*text* UUID (`lower(hex(...) || '-' || ...)`, 36-char dashed string) and is
never referenced by any other migration or any Rust file in the repo (grepped
`--include` for `V_UUID_V4` across `.sql` and `.rs`, zero hits outside its own
definition) — it appears to be dead code from the early schema, not the
established convention. The actual convention, confirmed in
`dao_impl_sqlite/src/sales_person.rs:34` and
`dao_impl_sqlite/src/billing_period_sales_person.rs:94-110`, is: Rust holds
`uuid::Uuid`, persists it via `.as_bytes().to_vec()` into a `BLOB(16)` column,
and reads it back via `Uuid::from_slice(&bytes).unwrap()`. `randomblob(16)`
produces exactly the 16 raw bytes that `Uuid::from_slice` expects — no dashes,
no hex-string parsing needed. (The generated bytes won't carry RFC 4122 version/
variant bits the way `Uuid::new_v4()` does, but nothing in the DAO layer
validates that — `from_slice` only checks length.) If strict RFC 4122
conformance is wanted for defensiveness, note it as an open question for
discuss-phase rather than pulling in a UUID-generation library — SQLite's
`randomblob` is sufficient for this backfill's purposes and needs zero new
dependencies.

### 5. The "planning target hours" additive DTO field

Directly reuse the additive-field pattern already used three times in
`rest-types/src/lib.rs`. Concretely, `ShortEmployeeReportTO`
(`rest-types/src/lib.rs:376-389`) already carries this exact shape:

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShortEmployeeReportTO {
    pub sales_person: SalesPersonTO,
    pub balance_hours: f32,
    pub expected_hours: f32,
    pub dynamic_hours: f32,
    pub overall_hours: f32,
    #[serde(default)]
    pub volunteer_hours: f32,
    #[serde(default)]
    pub has_pending_rebooking: bool,
    #[serde(default)]
    pub pending_rebooking_id: Option<Uuid>,
}
```

`has_pending_rebooking` / `pending_rebooking_id` were added in Phase 55
specifically as additive, `#[serde(default)]`-guarded fields to keep wire
compatibility with old clients/payloads (comment at `rest-types/src/lib.rs:381-384`
cites this explicitly as "Praezedenz VAA-04"). A new `planning_target_hours: f32`
(or similarly named) field on the report DTO follows the identical recipe: add
the field with `#[serde(default)]`, compute it in the corresponding
`impl From<&service::reporting::...> for ...TO` block
(`rest-types/src/lib.rs:392-406` shows the exact `From` pattern for
`ShortEmployeeReportTO`), and populate it in `service_impl/src/reporting.rs`
(the file already listed as a Docs-Freshness-Gate trigger in `CLAUDE.md` for
exactly this kind of change). No `utoipa`/`serde` version bump needed — `f32`
and `Option<Uuid>` fields already have `ToSchema` derives working today via the
same crate versions.

**No snapshot-schema-version bump applies to either feature** — confirmed by the
project's own decision log: `.planning/PROJECT.md` states `person` "kommt
obendrauf... kein neuer persistierter value_type... CURRENT_SNAPSHOT_SCHEMA_VERSION
bleibt 13," and the planning-target-hours field is explicitly a *display* field
computed from the already-live `expected_hours`/dynamic-contract logic — it does
not touch `billing_period_sales_person` or `BillingPeriodValueType` at all
(neither the `person` seed doc nor the sollstunden-bug todo referenced in
`PROJECT.md` mentions billing-period snapshots).

## Supporting Libraries

| Library | Version (as pinned in repo) | Purpose | When to Use |
|---------|------------------------------|---------|-------------|
| `uuid` | 1.8.0/1.8, feature `v4` (`dao/Cargo.toml`, `dao_impl_sqlite/Cargo.toml`, `service_impl/Cargo.toml`, `shifty_bin/Cargo.toml`); + `serde` feature in `rest-types/Cargo.toml` | Rust-side UUID type for `person.id` in DAO/service/DTO layers | Already the workspace-wide convention for every `BLOB(16)` PK; use `Uuid::new_v4()` for any *runtime* (non-migration) person-creation path, exactly like `sales_person`/`extra_hours` today |
| `sqlx` | 0.8.2, features `runtime-tokio`, `sqlite` (`dao_impl_sqlite/Cargo.toml`, `shifty_bin/Cargo.toml`) | Compile-time-checked queries + `sqlx::migrate!` migration runner | Already wired for every DAO; a new `PersonDao` follows the same `query!`/`query_as!` pattern seen in `dao_impl_sqlite/src/sales_person.rs` |
| `serde` | 1.0 (all crates) | DTO (de)serialization, `#[serde(default)]` for additive-field wire compat | Use for the new `planning_target_hours` field exactly as done for `volunteer_hours`/`has_pending_rebooking` |
| `utoipa` | 5, features `time`, `rc_schema`, `uuid` (`rest-types/Cargo.toml`) | OpenAPI schema derive (`ToSchema`) | Already covers `f32`, `Option<Uuid>`, no new feature flags needed |

No entries need to be added to this table — it documents what already exists
and is sufficient.

## Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `sqlx migrate add <name>` (sqlx-cli) | Scaffolds a new timestamped file under `migrations/sqlite/` | Documented in `CLAUDE.md:184`; use for both the `person`/backfill migration and any follow-up cleanup migration |
| `cargo sqlx prepare` (sqlx-cli, offline mode) | Regenerates `.sqlx/query-*.json` cache after adding/changing any `query!`/`query_as!` call | Required after adding a `PersonDao` with new hand-written queries, or the Nix `SQLX_OFFLINE` build step will fail to compile. This is existing project workflow (`.sqlx/` already populated), not a new tool. |
| `sqlx setup --source migrations/sqlite` | Local dev DB bootstrap + migration run | `CLAUDE.md:12`, unchanged by this milestone |

## Installation

```bash
# Nothing to install. Every crate/version needed already exists in the
# workspace Cargo.toml files listed above. No `cargo add` calls are required
# for either the Person migration+DAO+service work or the planning-target-hours
# DTO field.
```

## Alternatives Considered

| Recommended | Alternative | Why Not |
|-------------|-------------|---------|
| Plain `ALTER TABLE ADD COLUMN ... REFERENCES ...` for `sales_person.person_id` / `user.person_id` | Full table rebuild (`CREATE TABLE new` / `INSERT SELECT` / `DROP` / `RENAME`, as used for `extra_hours.logical_id`) | Rebuild is only needed in this repo's own history when a `NOT NULL` constraint must be retroactively enforced after backfill. `person_id` is nullable by the seed's own "Variante A" decision, so the simpler one-line `ALTER TABLE` (precedent: `slot.shiftplan_id`, `user_invitation.session_id`) is sufficient — no reason to pay the rebuild's extra risk/complexity. |
| Pure-SQL backfill inside the migration file | Rust application-code cutover service + quarantine/migration-source bookkeeping tables (the `absence_conversion.rs` pattern) | That pattern exists in this repo specifically because the extra_hours→absence_period migration had real disambiguation logic (categorization, date-range splitting, manual-review cases). The person merge has no such ambiguity — the pre-existing `UNIQUE(sales_person_id)` + `UNIQUE(user_id)` constraints on `sales_person_user` already prove every pair is 1:1. Introducing a Rust cutover service here would be scope creep relative to the repo's own proportionality precedent. |
| `randomblob(16)` for new `person.id` values inside the migration SQL | The `V_UUID_V4` view (`migrations/sqlite/20240426150045_user-roles.sql:90`) | `V_UUID_V4` produces a dashed hex *string*, not raw bytes, and is unused anywhere else in the codebase (grep confirmed zero references) — it doesn't match the `BLOB(16)` + `Uuid::from_slice` convention every other table uses. `randomblob(16)` produces the correct raw-byte shape directly. |
| New DTO field with `#[serde(default)]`, computed in the existing `From<&service::...>` impl | A wrapper/versioned DTO, or a new API version | The repo has an established, working additive-field convention (`ShortEmployeeReportTO.has_pending_rebooking`/`pending_rebooking_id`, `sales_person_absences`) that keeps wire compatibility without versioning machinery. No reason to deviate. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Any ORM (diesel, sea-orm, sqlx's `Model`-style helpers) | Not used anywhere in this workspace; would require introducing a parallel data-access paradigm alongside every existing `dao_impl_sqlite/src/*.rs` file | Plain `sqlx::query!`/`query_as!` following the existing `SalesPersonDaoImpl` pattern |
| A standalone migration framework (refinery, diesel_migrations, sqlx-cli replaced by something else) | `sqlx::migrate!` already runs every migration in `migrations/sqlite/` at startup (`shifty_bin/src/main.rs:1651`) and is the sole mechanism in this codebase | Keep using `sqlx migrate add` + the existing `migrations/sqlite/` directory |
| A UUID-generation crate/extension for SQLite (e.g. a custom SQLite function, `sqlite-uuid` extension) | SQLite's built-in `randomblob(16)` already produces the exact byte shape the Rust layer expects (`Uuid::from_slice`); no extension loading infrastructure exists in this repo | `randomblob(16)` in migration SQL; `Uuid::new_v4()` in Rust runtime code |
| The `V_UUID_V4` view for the person backfill | Wrong output shape (dashed text string vs. raw BLOB(16)) and unreferenced/dead elsewhere in the codebase | `randomblob(16)` |
| A new API version / breaking DTO change for `planning_target_hours` | The existing additive-field + `#[serde(default)]` convention already solves wire compatibility without versioning | Add the field additively, exactly like `has_pending_rebooking` |

## Stack Patterns by Variant

**If the `person_id` columns must eventually become `NOT NULL`** (not part of
v2.7 per the seed doc, which keeps them nullable to support the "nur User" /
"nur SalesPerson" cases permanently — this is not a transitional nullable, it's
a permanent one):
- No rebuild needed now or later for this reason, because these FKs are
  intentionally, permanently optional (one of the three "Ausprägungen" has no
  `sales_person` row, another has no `user` row).

**If a future phase needs `PersonDao`/`PersonService` at the Basic-Service tier**
(per `CLAUDE.md`'s Basic-vs-Business-Logic-Service convention):
- `PersonService` is a Basic Service (CRUD + validation over its own aggregate,
  consumes only `PersonDao` + `PermissionService` + `TransactionDao`) if it only
  manages the `person` row itself.
- Any cross-entity operation (e.g., "create a fully-linked person with both a
  sales_person and a user row in one transaction") belongs one tier up, as a
  Business-Logic Service that consumes `PersonService`, `SalesPersonService`,
  and a user-management service — matching the existing `AbsenceService` /
  `BookingInformationService` precedent in `CLAUDE.md`.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|------------------|-------|
| `sqlx 0.8.2` | `uuid 1.8.0` via `Uuid::from_slice`/`.as_bytes()` (not sqlx's native `uuid` feature) | This repo does **not** use sqlx's built-in `uuid` Cargo feature — it manually converts `Vec<u8>` ⇄ `Uuid` in every DAO (see `dao_impl_sqlite/src/sales_person.rs:34`, `dao_impl_sqlite/src/billing_period_sales_person.rs:94`). A new `PersonDao` should follow the same manual-conversion style for consistency, not introduce sqlx's `uuid` feature as a shortcut. |
| SQLite `ALTER TABLE ADD COLUMN` | Nullable FK columns only, without rebuild | Confirmed by two real precedents in this repo (`slot.shiftplan_id`, `user_invitation.session_id`); do not attempt `ADD COLUMN ... NOT NULL ... REFERENCES ...` without a rebuild — that combination is what forced the `extra_hours_new` rebuild for `logical_id`, and it doesn't apply here since `person_id` stays nullable. |

## Sources

- `Cargo.toml` (repo root) — workspace member list, confirms no ORM crate.
- `dao/Cargo.toml`, `dao_impl_sqlite/Cargo.toml`, `rest-types/Cargo.toml`, `service_impl/Cargo.toml`, `shifty_bin/Cargo.toml` — read in full for exact pinned versions (`sqlx 0.8.2`, `uuid 1.8/1.8.0`, `serde 1.0`, `utoipa 5`, `time 0.3.36`).
- `migrations/sqlite/20240506114107_add_sales_person.sql` — original `sales_person` + `sales_person_user` schema, confirms `UNIQUE(sales_person_id)` + `UNIQUE(user_id)` and the audit-column convention.
- `migrations/sqlite/20260330000000_add-shiftplan-table.sql:25` — nullable BLOB(16) FK added via plain `ALTER TABLE ADD COLUMN`.
- `migrations/sqlite/20251017044013_add-session-tracking-to-user-invitation.sql:5-6` — nullable TEXT FK added via plain `ALTER TABLE ADD COLUMN`.
- `migrations/sqlite/20260428101456_add-logical-id-to-extra-hours.sql` — precedent for simple SQL-only backfill, and for when a full table rebuild is actually required (`NOT NULL` enforcement after backfill).
- `migrations/sqlite/20260707000001_add-source-column-to-extra-hours.sql` — precedent for `DEFAULT`-only "backfill" (no sweep needed).
- `migrations/sqlite/20240426150045_user-roles.sql:90-98` — `V_UUID_V4` view definition, confirmed unused elsewhere via grep.
- `migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql`, `...20260503000001_create-absence-period-migration-source.sql` — precedent for the *other* backfill style (Rust cutover + bookkeeping tables), used here to justify why the person merge does NOT need this heavier pattern.
- `dao_impl_sqlite/src/sales_person.rs:1-60` — confirms raw-bytes UUID convention (`Uuid::from_slice`).
- `dao_impl_sqlite/src/billing_period_sales_person.rs:42-155` — confirms `.as_bytes().to_vec()` write-side convention and `Uuid::new_v4()` for new IDs at runtime.
- `rest-types/src/lib.rs:376-406` — `ShortEmployeeReportTO` additive-field pattern (`#[serde(default)]`), the direct precedent for the planning-target-hours DTO field.
- `shifty_bin/src/main.rs:1651`, `dao_impl_sqlite/src/migration_source.rs:111`, `shifty_bin/src/integration_test.rs:283` — `sqlx::migrate!` call sites, confirming migration-runner mechanism.
- `.planning/PROJECT.md` (current-milestone section, "v2.7 Personenverwaltung & Planungs-Soll") — snapshot-version-stays-13 decision, phase numbering, docs-freshness triggers.
- `.planning/seeds/personenverwaltung-zusammenlegen.md` — Variante A schema decision, nullable FK, "Namen bleiben, wo sie sind", unambiguity-via-UNIQUE-constraints argument.
- `CLAUDE.md` (repo root) — `sqlx setup`/`sqlx migrate add` commands, Basic-vs-Business-Logic-Service convention, Docs-Freshness-Gate trigger table.
- Confidence: HIGH for all claims — every statement above cites a specific file/line this agent read directly in this repo; no external web search was needed because the question was entirely "what does this repo already do," not "what does the ecosystem offer."

---
*Stack research for: Person aggregate migration + planning-target-hours DTO field (v2.7, Shifty backend)*
*Researched: 2026-09-16*
