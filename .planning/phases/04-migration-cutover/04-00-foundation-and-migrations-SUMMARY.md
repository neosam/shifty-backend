---
phase: 04-migration-cutover
plan: 00
subsystem: database
tags: [sqlite, sqlx, migrations, uuid, insta, openapi, utoipa, jj]

# Dependency graph
requires:
  - phase: 02-reporting-integration-snapshot-versioning
    provides: feature_flag table + privilege seed pattern (analog `feature_flag_admin`)
  - phase: 01-absence-domain-foundation
    provides: absence_period table (FK target for absence_period_migration_source)
  - phase: 03-booking-shift-plan-konflikt-integration
    provides: deferred-items entry for uuid v4 hygiene (D-Phase4-15 carry-over)
provides:
  - Three new write-once tables (`absence_migration_quarantine`, `absence_period_migration_source`, `employee_yearly_carryover_pre_cutover_backup`)
  - Privilege `cutover_admin` seeded analog `feature_flag_admin`
  - `uuid` `v4` feature on `dao` + `dao_impl_sqlite` so standalone tests are green
  - `insta = 1.47.2` (json) dev-dep on `rest` for D-Phase4-11 OpenAPI snapshot harness
  - `.planning/migration-backup/` dir tracked via `.gitkeep`, JSON-files in `.gitignore` for PII protection
  - OpenAPI snapshot test skeleton (`#[ignore]`d until Wave 2 accepts the .snap)
  - Phase-4 deferred-items doc (localdb provisioning hint)
affects: [04-01-service-traits-and-stubs, 04-02-cutover-service-heuristic, 04-03-carryover-rebuild-service, 04-04-extra-hours-flag-gate-and-soft-delete, 04-05-cutover-gate-and-diff-report, 04-06-cutover-rest-and-openapi, 04-07-integration-tests-and-profile]

# Tech tracking
tech-stack:
  added:
    - "insta 1.47.2 (json feature) — dev-dependency in rest crate (D-Phase4-11)"
    - "uuid v4 feature explicitly enabled on dao + dao_impl_sqlite (D-Phase4-15)"
  patterns:
    - "Write-once audit table pattern (no soft-delete column on quarantine + mapping + backup tables)"
    - "Composite PK (cutover_run_id, sales_person_id, year) on backup table — multiple cutover runs can coexist"
    - "Privilege seed via dedicated migration with `update_process = 'phase-4-migration'` (analog `feature_flag_admin` from Phase 2)"
    - "OpenAPI snapshot harness gated with `#[ignore = \"wave-2-accepts-snapshot\"]` to keep `cargo test --workspace` green during Wave 0/1"

key-files:
  created:
    - "migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql"
    - "migrations/sqlite/20260503000001_create-absence-period-migration-source.sql"
    - "migrations/sqlite/20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql"
    - "migrations/sqlite/20260503000003_add-cutover-admin-privilege.sql"
    - "rest/tests/openapi_snapshot.rs"
    - ".planning/migration-backup/.gitkeep"
    - ".planning/phases/04-migration-cutover/deferred-items.md"
  modified:
    - "dao/Cargo.toml"
    - "dao_impl_sqlite/Cargo.toml"
    - "rest/Cargo.toml"
    - ".gitignore"

key-decisions:
  - "uuid v4 feature explicitly opted-in on dao + dao_impl_sqlite to make standalone-tests green (D-Phase4-15)"
  - "Four discrete migrations, one per concern (3 tables + 1 privilege seed), per C-Phase4-01"
  - "Snapshot test is `#[ignore]`d in Wave 0 — Wave 2 accepts via .snap file rename"
  - "Diff-report directory has `.gitignore` rule for `*.json` + `!.gitkeep` exception (PII protection per Security Domain)"

patterns-established:
  - "jj per-task commits: each Wave-0 task = one jj change, isolated from prior planning-only changes"
  - "Conventional commit prefixes per task type: chore() for hygiene/deps, feat() for new schema, test() for snapshot scaffolding"

requirements-completed: [MIG-01, MIG-02, MIG-03, MIG-04, MIG-05]

# Metrics
duration: ~4min
completed: 2026-05-03
---

# Phase 04 Plan 00: Foundation & Migrations Summary

**Wave-0 hygiene + four phase-4 SQLite migrations + insta dev-dep + OpenAPI snapshot harness — every later wave reads from this surface.**

## Performance

- **Duration:** ~4 min (247s)
- **Started:** 2026-05-03T10:43:07Z
- **Completed:** 2026-05-03T10:47:14Z
- **Tasks:** 3
- **Files modified:** 11 (7 created + 4 modified)

## Accomplishments

- Four phase-4 SQLite migrations land cleanly in workspace; `cargo build --workspace` is green so the schema surface is ready for the Wave-1 cutover_dao traits.
- `uuid v4` feature is now explicitly declared on `dao` + `dao_impl_sqlite` — `cargo test -p dao` (10 passed) and `cargo test -p dao_impl_sqlite` (compiles + 0 tests) are green standalone (D-Phase4-15 closed).
- `insta = 1.47.2 (json)` dev-dep is wired into the `rest` crate; the OpenAPI snapshot test compiles, exposes `rest::ApiDoc` as a public surface, and reports `1 ignored` so it does not block `cargo test --workspace`.
- PII-safe `.gitignore` rule + `.gitkeep` exception protect `.planning/migration-backup/` from accidentally committing diff-report JSONs (Security Domain mitigation T-04-00-01).
- Deferred-items.md captures the localdb provisioning hint per D-Phase4-15.

## Task Commits

Each task committed atomically via jj (no git commit/add):

1. **Task 1: uuid v4 hygiene + insta dev-dep + .gitignore + migration-backup dir + deferred-items doc** — jj change `eac78087` (chore)
2. **Task 2: Four SQLite migrations (quarantine + mapping + carryover backup + cutover_admin privilege)** — jj change `2ec32cd8` (feat)
3. **Task 3: OpenAPI snapshot skeleton (gated #[ignore] until Wave 2 accept)** — jj change `5948d0ee` (test)

_Note: STATE.md / ROADMAP.md updates are intentionally not part of these commits — orchestrator owns that surface (per execution prompt)._

## Files Created/Modified

### Created

- `migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql` — Write-once quarantine table for ambiguous extra_hours rows (D-Phase4-03); FKs to sales_person + extra_hours; 2 indexes (cutover_run_id, sp+category).
- `migrations/sqlite/20260503000001_create-absence-period-migration-source.sql` — Idempotency mapping `extra_hours.id → absence_period.id` (D-Phase4-04); FKs + 2 indexes.
- `migrations/sqlite/20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql` — Pre-cutover snapshot of `employee_yearly_carryover` (D-Phase4-13); composite PK `(cutover_run_id, sales_person_id, year)`; 1 index.
- `migrations/sqlite/20260503000003_add-cutover-admin-privilege.sql` — `INSERT INTO privilege (name, update_process) VALUES ('cutover_admin', 'phase-4-migration')` (D-Phase4-07 + C-Phase4-08).
- `rest/tests/openapi_snapshot.rs` — `insta::assert_json_snapshot!(ApiDoc::openapi())` with `sort_maps`; `#[ignore = "wave-2-accepts-snapshot"]`.
- `.planning/migration-backup/.gitkeep` — empty placeholder so the dir is git-tracked.
- `.planning/phases/04-migration-cutover/deferred-items.md` — localdb provisioning hint per D-Phase4-15.

### Modified (Cargo.toml diffs)

- `dao/Cargo.toml`:
  - `-uuid = "1.8"`
  - `+uuid = { version = "1.8", features = ["v4"] }`
- `dao_impl_sqlite/Cargo.toml`:
  - `-uuid = "1.8.0"`
  - `+uuid = { version = "1.8.0", features = ["v4"] }`
- `rest/Cargo.toml` — appended new section:
  ```toml
  [dev-dependencies]
  insta = { version = "1.47.2", features = ["json"] }
  ```
- `.gitignore` — appended:
  ```
  # Phase 4 — Cutover diff reports (contain PII like sales_person_name)
  .planning/migration-backup/*.json
  !.planning/migration-backup/.gitkeep
  ```

## Migrations Created — Tables / Indexes / Seeds

| File | Object | Type |
| ---- | ------ | ---- |
| `20260503000000_create-absence-migration-quarantine.sql` | `absence_migration_quarantine` | TABLE (PK `extra_hours_id`, 8 cols, 2 FKs) |
| `20260503000000_create-absence-migration-quarantine.sql` | `idx_absence_migration_quarantine_run` | INDEX (cutover_run_id) |
| `20260503000000_create-absence-migration-quarantine.sql` | `idx_absence_migration_quarantine_sp_cat` | INDEX (sales_person_id, category) |
| `20260503000001_create-absence-period-migration-source.sql` | `absence_period_migration_source` | TABLE (PK `extra_hours_id`, 4 cols, 2 FKs) |
| `20260503000001_create-absence-period-migration-source.sql` | `idx_absence_period_migration_source_period` | INDEX (absence_period_id) |
| `20260503000001_create-absence-period-migration-source.sql` | `idx_absence_period_migration_source_run` | INDEX (cutover_run_id) |
| `20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql` | `employee_yearly_carryover_pre_cutover_backup` | TABLE (composite PK, 10 cols, 1 FK) |
| `20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql` | `idx_employee_yearly_carryover_pre_cutover_backup_sp_year` | INDEX (sales_person_id, year) |
| `20260503000003_add-cutover-admin-privilege.sql` | `INSERT INTO privilege` | SEED (`cutover_admin`) |

## Verification

| Step | Command | Result |
| ---- | ------- | ------ |
| Workspace build | `cargo build --workspace` | exit 0 (`Finished dev profile`) |
| dao standalone tests | `cargo test -p dao` | exit 0 — 10 passed, 0 failed |
| dao_impl_sqlite standalone tests | `cargo test -p dao_impl_sqlite` | exit 0 — compiles green, 0 tests in crate |
| Snapshot harness compiles | `cargo test -p rest --test openapi_snapshot --no-run` | exit 0 |
| Snapshot test gated | `cargo test -p rest --test openapi_snapshot` | exit 0 — 0 passed; 0 failed; 1 ignored |

All seven Cargo-/grep-based acceptance criteria across Tasks 1+2+3 verified green prior to each per-task commit.

## Decisions Made

None beyond what the plan locked. All four migrations follow the exact schemas from `04-CONTEXT.md` D-Phase4-03/04/13/07 + C-Phase4-08 verbatim. Cargo.toml patches follow D-Phase4-15. Snapshot scaffold follows RESEARCH.md Pattern 4 verbatim.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Threat Surface Scan

Threat register (T-04-00-01..04) was honored:

- **T-04-00-01 (Information Disclosure):** Mitigated via `.gitignore` `*.json` rule + `!.gitkeep` exception (Task 1).
- **T-04-00-02 (Tampering):** Quarantine + mapping tables are write-once by SQL design (no UPDATE paths added; Wave 1 implements DAOs that only INSERT). FKs prevent dangling references.
- **T-04-00-03 (Elevation of Privilege):** `cutover_admin` privilege seeded via migration only; any role binding is an out-of-band DB operation.
- **T-04-00-04 (Repudiation):** Accepted — `cutover_run_id` + `migrated_at` / `backed_up_at` columns provide audit trail.

No new threat surface introduced beyond the threat register.

## Hand-off Note for Plan 04-01 (Wave 1)

- `dao` Cargo.toml has `uuid v4` feature → standalone-tests green; future `cutover_dao` trait can call `Uuid::new_v4()` without feature drift.
- All four migrations exist on disk; the next `nix-shell --run 'sqlx setup --source migrations/sqlite'` will materialize the three new tables + privilege row.
- Wave 1 can author the `cutover_dao` trait against the new schemas immediately. Wave 2 can call `PermissionService::check_permission(CUTOVER_ADMIN_PRIVILEGE, ...)` once the role binding is in place.
- `insta` is wired into `rest` as a dev-dep — Wave 2 only needs to remove the `#[ignore]` and accept the generated `.snap.new` file.

## Self-Check: PASSED

Verification of summary claims:

- **Files created exist:**
  - migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql — FOUND
  - migrations/sqlite/20260503000001_create-absence-period-migration-source.sql — FOUND
  - migrations/sqlite/20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql — FOUND
  - migrations/sqlite/20260503000003_add-cutover-admin-privilege.sql — FOUND
  - rest/tests/openapi_snapshot.rs — FOUND
  - .planning/migration-backup/.gitkeep — FOUND
  - .planning/phases/04-migration-cutover/deferred-items.md — FOUND
- **Files modified exist with expected content:**
  - dao/Cargo.toml — `features = ["v4"]` present
  - dao_impl_sqlite/Cargo.toml — `features = ["v4"]` present
  - rest/Cargo.toml — `insta = { version = "1.47.2", features = ["json"] }` present
  - .gitignore — `.planning/migration-backup/*.json` + `!.planning/migration-backup/.gitkeep` present
- **jj changes exist:**
  - `eac78087` — Task 1 (chore: uuid v4 + insta + gitignore + migration-backup dir + deferred-items)
  - `2ec32cd8` — Task 2 (feat: four phase-4 SQLite migrations)
  - `5948d0ee` — Task 3 (test: OpenAPI snapshot harness scaffold)

---

*Phase: 04-migration-cutover*
*Plan: 00 (foundation-and-migrations)*
*Completed: 2026-05-03*
