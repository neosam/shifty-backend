---
plan: 04-00-foundation-and-migrations
phase: 4
wave: 0
depends_on: []
requirements: [MIG-01, MIG-02, MIG-03, MIG-04, MIG-05]
files_modified:
  - dao/Cargo.toml
  - dao_impl_sqlite/Cargo.toml
  - rest/Cargo.toml
  - migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql
  - migrations/sqlite/20260503000001_create-absence-period-migration-source.sql
  - migrations/sqlite/20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql
  - migrations/sqlite/20260503000003_add-cutover-admin-privilege.sql
  - .planning/migration-backup/.gitkeep
  - .planning/phases/04-migration-cutover/deferred-items.md
  - .gitignore
  - rest/tests/openapi_snapshot.rs
autonomous: true
must_haves:
  truths:
    - "Standalone `cargo test -p dao` and `cargo test -p dao_impl_sqlite` are GREEN (D-Phase4-15)."
    - "Three new write-once tables (`absence_migration_quarantine`, `absence_period_migration_source`, `employee_yearly_carryover_pre_cutover_backup`) exist and `cargo build --workspace` compiles."
    - "Privilege `cutover_admin` is seeded in the privilege table (D-Phase4-07 + C-Phase4-08)."
    - "Crate `insta = 1.47.2` (json feature) is a dev-dependency of the `rest` crate."
    - "Directory `.planning/migration-backup/` exists, is git-tracked via `.gitkeep`, and is in `.gitignore` for `*.json` files (PII-Schutz pro Security Domain)."
    - "OpenAPI snapshot test skeleton compiles but is `#[ignore]`d until Wave 2 accepts the snapshot."
  artifacts:
    - path: "migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql"
      provides: "Quarantine table per D-Phase4-03"
    - path: "migrations/sqlite/20260503000001_create-absence-period-migration-source.sql"
      provides: "Mapping table per D-Phase4-04"
    - path: "migrations/sqlite/20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql"
      provides: "Carryover backup table per D-Phase4-13"
    - path: "migrations/sqlite/20260503000003_add-cutover-admin-privilege.sql"
      provides: "cutover_admin privilege seed (analog feature_flag_admin)"
    - path: "rest/tests/openapi_snapshot.rs"
      provides: "OpenAPI snapshot harness skeleton (Wave 0); accepted in Wave 2"
    - path: ".planning/migration-backup/.gitkeep"
      provides: "Diff-Report-Verzeichnis exists for Wave 2/3 runs"
    - path: ".planning/phases/04-migration-cutover/deferred-items.md"
      provides: "localdb-Drift-Hinweis (D-Phase4-15)"
  key_links:
    - from: "dao_impl_sqlite/Cargo.toml"
      to: "uuid v4 feature"
      via: "features=[\"v4\"] enables Uuid::new_v4() — used by Wave 1 cutover_dao"
    - from: "rest/Cargo.toml"
      to: "insta = 1.47.2 (json) dev-dep"
      via: "Wave 0 dev-dependency block — Wave 2 accept-step relies on it"
    - from: "migrations/sqlite/20260503000003_add-cutover-admin-privilege.sql"
      to: "Wave 2 PermissionService::check_permission(CUTOVER_ADMIN_PRIVILEGE, ...)"
      via: "INSERT INTO privilege (name, update_process)"
---

<objective>
Wave 0 — Foundation & Hygiene. Land the four SQLite migrations (3 new tables + 1 privilege insert), patch the `uuid` feature in `dao` + `dao_impl_sqlite` Cargo.toml (D-Phase4-15 hygiene), add `insta = 1.47.2` as a dev-dependency in `rest/Cargo.toml`, scaffold the OpenAPI snapshot test (gated `#[ignore]` until Wave 2 accepts the file), and create the `.planning/migration-backup/` directory with a `.gitignore` rule for the JSON diff reports (PII protection per Security Domain).

Purpose: Establish the on-disk and dependency surface that every later wave reads. Every Wave-1 task that opens a migration-related DAO trait, every Wave-2 task that calls insta, and every Wave-3 task that writes a diff-report file depends on this wave being green first.

Output: 4 migrations, 3 Cargo.toml patches, 1 ignored snapshot test, 1 directory placeholder, 1 deferred-items doc, 1 gitignore patch.
</objective>

<execution_context>
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/workflows/execute-plan.md
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/ROADMAP.md
@.planning/phases/04-migration-cutover/04-CONTEXT.md
@.planning/phases/04-migration-cutover/04-RESEARCH.md
@.planning/phases/04-migration-cutover/04-VALIDATION.md
@.planning/phases/03-booking-shift-plan-konflikt-integration/deferred-items.md
@migrations/sqlite/20260501000000_add-feature-flag-table.sql
@migrations/sqlite/20260105000000_app-toggles.sql
@migrations/sqlite/20260502170000_create-absence-period.sql

<interfaces>
<!-- Existing patterns the executor must replicate verbatim. -->

From `migrations/sqlite/20260501000000_add-feature-flag-table.sql` (Phase-2 privilege seed pattern):
```sql
INSERT INTO privilege (name, update_process)
VALUES ('feature_flag_admin', 'initial');
```

From `migrations/sqlite/20260502170000_create-absence-period.sql` (Phase-1 schema column conventions):
```sql
id              BLOB(16) NOT NULL PRIMARY KEY,
sales_person_id BLOB(16) NOT NULL,
created         TEXT NOT NULL,
update_timestamp TEXT,
update_process  TEXT NOT NULL,
update_version  BLOB(16) NOT NULL,
```

From `dao/Cargo.toml:14`:
```toml
uuid = "1.8"
```

From `dao_impl_sqlite/Cargo.toml:11`:
```toml
uuid = "1.8.0"
```

From `rest/Cargo.toml` (no `[dev-dependencies]` block exists yet — Wave 0 introduces it):
```toml
# (Add a new section)
[dev-dependencies]
insta = { version = "1.47.2", features = ["json"] }
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: uuid v4 hygiene + insta dev-dep + .gitignore + migration-backup dir + deferred-items doc</name>
  <read_first>
    - dao/Cargo.toml
    - dao_impl_sqlite/Cargo.toml
    - rest/Cargo.toml
    - .gitignore (from repo root)
    - .planning/phases/03-booking-shift-plan-konflikt-integration/deferred-items.md
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "D-Phase4-15"
    - .planning/phases/04-migration-cutover/04-RESEARCH.md § "Security Domain → Diff-Report mit PII"
  </read_first>
  <action>
1. Patch `dao/Cargo.toml`: replace `uuid = "1.8"` with `uuid = { version = "1.8", features = ["v4"] }`.
2. Patch `dao_impl_sqlite/Cargo.toml`: replace `uuid = "1.8.0"` with `uuid = { version = "1.8.0", features = ["v4"] }`.
3. Patch `rest/Cargo.toml`: append a new `[dev-dependencies]` block at the end of file (or extend existing if any) with the line `insta = { version = "1.47.2", features = ["json"] }`. Do NOT touch the existing `[dependencies]` section.
4. Create `.planning/migration-backup/.gitkeep` (empty file).
5. Patch repo-root `.gitignore`: add a section header `# Phase 4 — Cutover diff reports (contain PII like sales_person_name)` and the rule `.planning/migration-backup/*.json`. The `.gitkeep` file MUST remain tracked — verify with the negation rule `!.planning/migration-backup/.gitkeep` immediately after the wildcard rule.
6. Create `.planning/phases/04-migration-cutover/deferred-items.md` with a single bullet documenting the local `localdb.sqlite3` drift hint per D-Phase4-15. Reference: pattern from `.planning/phases/03-booking-shift-plan-konflikt-integration/deferred-items.md`. Content (verbatim):
   - "**Lokale `localdb.sqlite3`-Provisionierung** (D-Phase4-15): Beim Updaten auf Phase 4 müssen lokale Dev-Datenbanken neu provisioniert werden (`rm localdb.sqlite3 && nix-shell --run 'sqlx setup --source migrations/sqlite'`). Die alte localdb fehlt noch alle Phase-1..4-Migrations und alle Phase-2-Seeds. Kein Code-Fix nötig — lokal pro Dev."
  </action>
  <acceptance_criteria>
    - `grep -q 'features = \["v4"\]' dao/Cargo.toml` exits 0
    - `grep -q 'features = \["v4"\]' dao_impl_sqlite/Cargo.toml` exits 0
    - `grep -q 'insta = { version = "1.47.2", features = \["json"\] }' rest/Cargo.toml` exits 0
    - File `.planning/migration-backup/.gitkeep` exists
    - `grep -q '.planning/migration-backup/\*.json' .gitignore` exits 0
    - `grep -q '!.planning/migration-backup/.gitkeep' .gitignore` exits 0
    - File `.planning/phases/04-migration-cutover/deferred-items.md` exists and contains the substring `localdb.sqlite3`
    - `cargo build --workspace` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo test -p dao && cargo test -p dao_impl_sqlite</automated>
  </verify>
  <done>
    Workspace builds, `cargo test -p dao` and `cargo test -p dao_impl_sqlite` are green standalone; insta dev-dep available for later waves; PII-safe gitignore rule in place.
  </done>
</task>

<task type="auto">
  <name>Task 2: Four SQLite migrations (quarantine + mapping + carryover backup + cutover_admin privilege)</name>
  <read_first>
    - migrations/sqlite/20260501000000_add-feature-flag-table.sql (privilege seed pattern)
    - migrations/sqlite/20260105000000_app-toggles.sql (Z. 30: privilege INSERT pattern)
    - migrations/sqlite/20260502170000_create-absence-period.sql (BLOB(16) + soft-delete column pattern)
    - migrations/sqlite/20241215063132_add_employee-yearly-carryover.sql (employee_yearly_carryover schema for backup-table column-isomorphism)
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "D-Phase4-03 / D-Phase4-04 / D-Phase4-13 / D-Phase4-07 / C-Phase4-01 / C-Phase4-08"
  </read_first>
  <action>
Create FOUR new migrations under `migrations/sqlite/` with the timestamps below. Each migration is a single `CREATE TABLE` (or single `INSERT`) — keep them small and atomic per C-Phase4-01.

**1. `migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql` (D-Phase4-03):**
```sql
-- Phase 4 (Migration & Cutover) — Quarantine table for ambiguous extra_hours rows.
-- Write-once audit table; NO soft-delete (HR resolves manually).

CREATE TABLE absence_migration_quarantine (
    extra_hours_id  BLOB(16) NOT NULL PRIMARY KEY,
    reason          TEXT NOT NULL,
    sales_person_id BLOB(16) NOT NULL,
    category        TEXT NOT NULL,
    date_time       TEXT NOT NULL,
    amount          REAL NOT NULL,
    cutover_run_id  BLOB(16) NOT NULL,
    migrated_at     TEXT NOT NULL,

    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id),
    FOREIGN KEY (extra_hours_id)  REFERENCES extra_hours(id)
);

CREATE INDEX idx_absence_migration_quarantine_run
    ON absence_migration_quarantine(cutover_run_id);

CREATE INDEX idx_absence_migration_quarantine_sp_cat
    ON absence_migration_quarantine(sales_person_id, category);
```

**2. `migrations/sqlite/20260503000001_create-absence-period-migration-source.sql` (D-Phase4-04):**
```sql
-- Phase 4 — Mapping from legacy extra_hours.id to migrated absence_period.id.
-- Idempotency key: extra_hours_id PK. Re-run skips already-mapped rows.
-- Write-once audit table; NO soft-delete.

CREATE TABLE absence_period_migration_source (
    extra_hours_id    BLOB(16) NOT NULL PRIMARY KEY,
    absence_period_id BLOB(16) NOT NULL,
    cutover_run_id    BLOB(16) NOT NULL,
    migrated_at       TEXT NOT NULL,

    FOREIGN KEY (absence_period_id) REFERENCES absence_period(id),
    FOREIGN KEY (extra_hours_id)    REFERENCES extra_hours(id)
);

CREATE INDEX idx_absence_period_migration_source_period
    ON absence_period_migration_source(absence_period_id);

CREATE INDEX idx_absence_period_migration_source_run
    ON absence_period_migration_source(cutover_run_id);
```

**3. `migrations/sqlite/20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql` (D-Phase4-13):**
```sql
-- Phase 4 — Pre-cutover snapshot of employee_yearly_carryover for safe rollback.
-- Schema-isomorph to employee_yearly_carryover (sales_person_id, year, carryover_hours, vacation,
-- created, deleted, update_process, update_version) plus cutover_run_id + backed_up_at.
-- PK is composite (cutover_run_id, sales_person_id, year) so multiple cutover runs can coexist.

CREATE TABLE employee_yearly_carryover_pre_cutover_backup (
    cutover_run_id  BLOB(16) NOT NULL,
    sales_person_id BLOB(16) NOT NULL,
    year            INTEGER NOT NULL,
    carryover_hours REAL NOT NULL,
    vacation        INTEGER NOT NULL,
    created         TEXT NOT NULL,
    deleted         TEXT,
    update_process  TEXT NOT NULL,
    update_version  BLOB(16) NOT NULL,
    backed_up_at    TEXT NOT NULL,

    PRIMARY KEY (cutover_run_id, sales_person_id, year),
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);

CREATE INDEX idx_employee_yearly_carryover_pre_cutover_backup_sp_year
    ON employee_yearly_carryover_pre_cutover_backup(sales_person_id, year);
```

**4. `migrations/sqlite/20260503000003_add-cutover-admin-privilege.sql` (D-Phase4-07 + C-Phase4-08):**
```sql
-- Phase 4 — New privilege for the destructive cutover commit endpoint.
-- Pattern verbatim from 20260501000000_add-feature-flag-table.sql:22-23.

INSERT INTO privilege (name, update_process)
VALUES ('cutover_admin', 'phase-4-migration');
```

After writing all four files, run `nix-shell --run 'sqlx migrate info --source migrations/sqlite'` to verify all four migrations are listed (use `nix-shell` per CLAUDE.local.md). If `sqlx-cli` is not on `nix-shell` PATH by default, run `cargo build --workspace` instead — the `.sqlx` cache will refuse to build if compile-time queries reference non-existent tables. Wave 0 has no compile-time queries, so `cargo build` MUST be green.
  </action>
  <acceptance_criteria>
    - All four migration files exist under `migrations/sqlite/` with the exact filenames above
    - `grep -q "CREATE TABLE absence_migration_quarantine" migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql` exits 0
    - `grep -q "CREATE TABLE absence_period_migration_source" migrations/sqlite/20260503000001_create-absence-period-migration-source.sql` exits 0
    - `grep -q "CREATE TABLE employee_yearly_carryover_pre_cutover_backup" migrations/sqlite/20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql` exits 0
    - `grep -q "'cutover_admin'" migrations/sqlite/20260503000003_add-cutover-admin-privilege.sql` exits 0
    - `cargo build --workspace` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build --workspace</automated>
  </verify>
  <done>
    Migrations land in workspace; on next `sqlx setup` (locally) all four tables/seeds materialize. The Wave-1 cutover_dao traits will compile against these schemas.
  </done>
</task>

<task type="auto">
  <name>Task 3: OpenAPI snapshot skeleton (gated #[ignore] until Wave 2 accept)</name>
  <read_first>
    - rest/Cargo.toml (verify Task 1 added insta dev-dep)
    - rest/src/lib.rs (Z. 460-486: ApiDoc struct — must be `pub`)
    - .planning/phases/04-migration-cutover/04-RESEARCH.md § "Pattern 4: Insta-OpenAPI-Snapshot mit Sort-Maps-Belt-and-Suspenders" + § "Operation 3: Insta OpenAPI Snapshot Test"
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "D-Phase4-11 / OpenAPI-Snapshot-Test"
  </read_first>
  <action>
Create `rest/tests/openapi_snapshot.rs` containing the following test (gated with `#[ignore]` for Wave 0; Wave 2 removes the `#[ignore]` and runs `cargo test ... -- --ignored` to generate the `.snap` file, then a human reviews and renames `.snap.new → .snap`):

```rust
//! Phase 4 OpenAPI snapshot lock (D-Phase4-11).
//!
//! Wave 0: scaffold-only — the test is `#[ignore]`'d until Wave 2 has added the
//! `/admin/cutover/*` endpoints + `ExtraHoursCategoryDeprecatedErrorTO` schema.
//! Wave 2: removes the `#[ignore]`, runs `cargo test -p rest --test openapi_snapshot`
//! once to generate `rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap.new`,
//! a human reviews via `git diff` (no global `cargo insta` install per MEMORY.md),
//! then renames `.snap.new → .snap` (or runs `cargo insta accept` if locally installed).

use rest::ApiDoc;
use utoipa::OpenApi;

#[test]
#[ignore = "wave-2-accepts-snapshot"]
fn openapi_snapshot_locks_full_api_surface() {
    let openapi = ApiDoc::openapi();
    insta::with_settings!({ sort_maps => true }, {
        insta::assert_json_snapshot!(openapi);
    });
}
```

Verify the test compiles by running `cargo test -p rest --test openapi_snapshot --no-run`. The test must NOT execute (the `#[ignore]` keeps it out of the default `cargo test` run). The compile step proves the `insta` dev-dependency from Task 1 is wired and that `rest::ApiDoc` is a public, exportable type.

If `rest::ApiDoc` is not `pub`, abort and report — Wave 2 will need it `pub` anyway, but Wave 0 keeps the surface change minimal. **Do not** modify `rest/src/lib.rs` in this task — verify only.
  </action>
  <acceptance_criteria>
    - File `rest/tests/openapi_snapshot.rs` exists
    - `grep -q '#\[ignore = "wave-2-accepts-snapshot"\]' rest/tests/openapi_snapshot.rs` exits 0
    - `grep -q 'insta::assert_json_snapshot!(openapi)' rest/tests/openapi_snapshot.rs` exits 0
    - `cargo test -p rest --test openapi_snapshot --no-run` exits 0
    - `cargo test -p rest --test openapi_snapshot` runs and reports `0 passed; 0 failed; 1 ignored` (the test is gated)
  </acceptance_criteria>
  <verify>
    <automated>cargo test -p rest --test openapi_snapshot</automated>
  </verify>
  <done>
    Snapshot harness compiles, is `#[ignore]`'d so it does not block `cargo test --workspace`. Wave 2 will toggle the `#[ignore]` and accept the snapshot.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Filesystem (.planning/migration-backup/) | Diff-Report JSONs may contain PII (sales_person_name); the directory is in `.gitignore` so reports never accidentally land in git. |
| SQLite migrations | New tables increase the attack surface; FKs + write-once design prevent unauthorized mutation. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-00-01 | Information Disclosure | `.planning/migration-backup/*.json` | mitigate | `.gitignore` rule `*.json` under that directory + `.gitkeep` exception (Task 1). Operations is responsible for filesystem-level read restrictions. |
| T-04-00-02 | Tampering | `absence_migration_quarantine` / `absence_period_migration_source` rows | mitigate | Tables are write-once (no UPDATE SQL paths in any plan); FKs to `extra_hours.id` and `absence_period.id` prevent dangling references. |
| T-04-00-03 | Elevation of Privilege | `cutover_admin` privilege seeded via migration | mitigate | Privilege exists in DB only after migration runs; any role binding requires DB write — out of band of Phase 4. Wave 2 service-layer enforces `check_permission` before any state change. |
| T-04-00-04 | Repudiation | Backup-table rows | accept | `cutover_run_id` + `backed_up_at` provides audit trail; full repudiation defense (signed audit logs) is out of scope for v1. |
</threat_model>

<verification>
- `cargo build --workspace` green
- `cargo test -p dao` green standalone (uuid v4 feature)
- `cargo test -p dao_impl_sqlite` green standalone (uuid v4 feature)
- `cargo test -p rest --test openapi_snapshot` reports `1 ignored` (Wave-0 gate)
- All four migration files exist with the exact `BLOB(16)` + soft-delete-column conventions
- `.gitignore` protects diff reports from accidental commits
- No code in `service/`, `service_impl/`, `rest/src/` touched (Wave 0 is hygiene only — no traits, no impls)
</verification>

<success_criteria>
1. Standalone tests on `dao` and `dao_impl_sqlite` are green (D-Phase4-15 closed).
2. Four migration files exist; the next `sqlx setup` will materialize three new tables and one privilege row.
3. `insta` is available as a dev-dep in `rest`; OpenAPI snapshot harness compiles but is `#[ignore]`d.
4. `.planning/migration-backup/` exists and is PII-protected via `.gitignore`.
5. Wave 1 can start immediately (all DAO/Service trait stubs in Plan 04-01 land into a workspace where the underlying schemas exist).
</success_criteria>

<output>
After completion, create `.planning/phases/04-migration-cutover/04-00-SUMMARY.md` listing:
- Cargo.toml diffs (3 files)
- Migration filenames + table/index names created
- `.gitignore` patch verbatim
- Verification: `cargo build --workspace` exit code, `cargo test -p dao` exit code, `cargo test -p dao_impl_sqlite` exit code
- Hand-off note for Plan 04-01 (Wave 1): "DAO Cargo.toml has uuid v4 feature; migrations exist; cutover_dao trait can be authored against the new schemas."
</output>
