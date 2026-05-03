---
phase: 4
slug: migration-cutover
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-03
revised: 2026-05-03
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source: derived from `04-RESEARCH.md § Validation Architecture` (Z. 846-938).
> Revision (2026-05-03): added `soft_delete_bulk_forbidden_for_unprivileged_user` row
> per Plan-Checker BLOCKER #4; Threat-Model T-04-04-01 now has its automated test.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) + `tokio::test` (async) + `mockall` (Service-Mocks) + `insta = "1.47.2"` (NEU für OpenAPI-Snapshot, dev-dependency in `rest/Cargo.toml`) |
| **Config file** | none — pro-Crate `Cargo.toml` (existing); insta-Snapshots unter `rest/tests/snapshots/` |
| **Quick run command** | `cargo test -p service_impl test::cutover` (Service-Layer-Mock-Tests) |
| **Full suite command** | `cargo test --workspace` (alle Crates inkl. `shifty_bin/src/integration_test/cutover.rs`) |
| **OpenAPI-Snapshot run** | `cargo test -p rest --test openapi_snapshot` |
| **Estimated runtime** | ~5s (quick), ~60s (full), ~2s (snapshot) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p service_impl test::cutover` (< 5s)
- **After OpenAPI-Surface-Änderungen (Wave 2):** zusätzlich `cargo test -p rest --test openapi_snapshot` (< 2s)
- **After every plan wave:** Run `cargo build --workspace && cargo test --workspace` (< 60s)
- **Before `/gsd:verify-phase 04`:** Full suite green + `cargo run` boot-Smoke mit 30s-Timeout (verifiziert dass Bin nach allen Migrationen + neuem `CutoverService`-DI bootet)
- **Max feedback latency:** < 60s

---

## Per-Task Verification Map

> Plan-IDs werden in Plan-Phase final vergeben (z.B. `04-00-..` für Wave-0). Diese Map listet die Tests pro Requirement; sie wird beim Plan-Schreiben in die jeweilige `<automated>`-Sektion der Tasks übernommen.

| Req ID | Behavior | Wave | Test Type | Automated Command | File Exists | Status |
|--------|----------|------|-----------|-------------------|-------------|--------|
| MIG-01 | Heuristik-Cluster: konsekutive Werktage gleicher (sp,kat) mit `amount==contract_hours_at(day)` → 1 absence_period | 1 | unit (mockall) | `cargo test -p service_impl test::cutover::cluster_merges_consecutive_workdays_with_exact_match` | ❌ W1 | ⬜ pending |
| MIG-01 | Quarantäne: `amount_below_contract_hours` | 1 | unit | `cargo test -p service_impl test::cutover::quarantine_amount_below_contract` | ❌ W1 | ⬜ pending |
| MIG-01 | Quarantäne: `amount_above_contract_hours` | 1 | unit | `cargo test -p service_impl test::cutover::quarantine_amount_above_contract` | ❌ W1 | ⬜ pending |
| MIG-01 | Quarantäne: `weekend_entry_with_workday_only_contract` | 1 | unit | `cargo test -p service_impl test::cutover::quarantine_weekend_entry_workday_contract` | ❌ W1 | ⬜ pending |
| MIG-01 | Quarantäne: `contract_not_active_at_date` | 1 | unit | `cargo test -p service_impl test::cutover::quarantine_contract_not_active` | ❌ W1 | ⬜ pending |
| MIG-01 | Quarantäne: `iso_53_week_gap` (falls überhaupt produzierbar) | 1 | unit | `cargo test -p service_impl test::cutover::quarantine_iso_53_gap` | ❌ W1 | ⬜ pending |
| MIG-01 | Re-Run-Idempotenz: zweiter Cutover-Run skippt bereits gemappte `extra_hours.id` | 1 + 3 | unit + integration | `cargo test -p service_impl test::cutover::idempotent_rerun_skips_mapped` und `cargo test -p shifty_bin --test integration_test cutover::test_idempotence_rerun_no_op` | ❌ W1 + W3 | ⬜ pending |
| MIG-02 | Gate-Berechnung benutzt `derive_hours_for_range` (kein Re-Implement) | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_gate_uses_derive_hours_for_range_path` | ❌ W3 | ⬜ pending |
| MIG-02 | Gate-Drift Toleranz `< 0.01h`: 0.005h-Drift = pass | 2 | unit | `cargo test -p service_impl test::cutover::gate_tolerance_pass_below_threshold` | ❌ W2 | ⬜ pending |
| MIG-02 | Gate-Drift Toleranz `< 0.01h`: 0.02h-Drift = fail | 2 | unit | `cargo test -p service_impl test::cutover::gate_tolerance_fail_above_threshold` | ❌ W2 | ⬜ pending |
| MIG-02 | Diff-Report-JSON-File-Schema: `gate_run_id`, `run_at`, `dry_run`, `drift_threshold`, `total_drift_rows`, `drift[]`, `passed` vorhanden | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_diff_report_json_schema` | ❌ W3 | ⬜ pending |
| MIG-03 | REST `POST /admin/cutover/gate-dry-run` HR-permission | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_gate_dry_run_endpoint_success` | ❌ W3 | ⬜ pending |
| MIG-03 | REST `POST /admin/cutover/gate-dry-run` 403 für non-HR | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_gate_dry_run_forbidden_for_unprivileged` | ❌ W3 | ⬜ pending |
| MIG-03 | REST `POST /admin/cutover/gate-dry-run` 200 mit `gate_passed:false` bei Quarantäne-Fixture | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_gate_dry_run_returns_failure_with_quarantine` | ❌ W3 | ⬜ pending |
| MIG-03 | REST `POST /admin/cutover/commit` requires `cutover_admin`-Privilege (HR-only ist 403) | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_commit_forbidden_for_hr_only` | ❌ W3 | ⬜ pending |
| MIG-03 | REST `POST /admin/cutover/commit` success für `cutover_admin` (gate-pass + state-change) | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_commit_success_for_cutover_admin` | ❌ W3 | ⬜ pending |
| MIG-04 | Atomic-Tx: Sub-Service-Error → komplette Rollback (Flag bleibt false, extra_hours unverändert) | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_atomic_rollback_on_subservice_error` | ❌ W3 | ⬜ pending |
| MIG-04 | Carryover-Refresh-Scope: nur `(sp, year)`-Tupel mit non-zero Vac/Sick/UnpaidLeave-Sum | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_carryover_refresh_scope_only_affected_tuples` | ❌ W3 | ⬜ pending |
| MIG-04 | Pre-Cutover-Backup: alle gateskopierten Tupel vor UPDATE in `employee_yearly_carryover_pre_cutover_backup` | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_pre_cutover_backup_populated_before_update` | ❌ W3 | ⬜ pending |
| MIG-04 | Soft-Delete legacy: migrierte Rows haben `deleted IS NOT NULL` + `update_process='phase-4-cutover-migration'`; Quarantäne-Rows aktiv | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_soft_delete_migrated_rows_only` | ❌ W3 | ⬜ pending |
| MIG-04 | Flag-Flip: `feature_flag.absence_range_source_active = 1` nach Commit | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_feature_flag_set_to_true_on_commit` | ❌ W3 | ⬜ pending |
| MIG-05 | `/extra-hours` POST flag-gated: vor Cutover 200, nach Cutover 403 für Vac/Sick/UnpaidLeave; ExtraWork bleibt 200 | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_extra_hours_post_flag_gated_before_after` | ❌ W3 | ⬜ pending |
| MIG-05 | `ServiceError::ExtraHoursCategoryDeprecated` → 403 mit Body `{"error":"extra_hours_category_deprecated","category":...,"message":...}` | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_403_body_format_for_deprecated_category` | ❌ W3 | ⬜ pending |
| MIG-05 | OpenAPI-Snapshot lockt API-Surface (D-Phase4-11): neue `/admin/cutover/*`-Endpunkte (3) + 6 Cutover-Schemas + `ExtraHoursCategoryDeprecated`-Schema | 0 + 2 | snapshot | `cargo test -p rest --test openapi_snapshot openapi_snapshot_locks_full_api_surface` | ❌ W0 (skeleton) + W2 (accept) | ⬜ pending |
| SC-1 | Production-Data-Profile via REST `POST /admin/cutover/profile` (Histogramm `(sp, category, year)` + Bruchstunden-Quote + Wochenend-Count + ISO-53-Indicator) — exercises full HR-auth → handler → service → JSON-file → response-body path | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_profile_generates_json_with_histograms` | ❌ W3 | ⬜ pending |
| SC-5 | Per-`(sp, kategorie, jahr)`-Invariant: Pre-Migration-Sum == Post-Migration-derived-Sum (≤ 0.001h Drift) | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::per_sales_person_per_year_per_category_invariant` | ❌ W3 | ⬜ pending |
| SC-3 (Atomarität) | Bei Gate-Fail bleibt gesamte Tx zurück (kein soft-delete, kein Backup-Insert, Flag bleibt 0) | 3 | integration | `cargo test -p shifty_bin --test integration_test cutover::test_gate_fail_no_state_change` | ❌ W3 | ⬜ pending |
| Forbidden | `_forbidden`-Tests pro public service method auf `CutoverService` (HR ∨ `cutover_admin`) | 1 | unit | `cargo test -p service_impl test::cutover::run_forbidden_for_unprivileged_user` und `..::run_forbidden_for_hr_only_when_committing` | ❌ W1 | ⬜ pending |
| Forbidden | `_forbidden`-Test für `CarryoverRebuildService::rebuild_for_year` (HR oder Admin — Plan-Phase entscheidet exakte Surface) | 1 | unit | `cargo test -p service_impl test::carryover_rebuild::rebuild_forbidden_for_unprivileged` | ❌ W1 | ⬜ pending |
| Forbidden | `_forbidden`-Test für `ExtraHoursService::soft_delete_bulk` (CUTOVER_ADMIN_PRIVILEGE) — verifiziert dass Permission-Check VOR dem DAO-Call sitzt via `MockExtraHoursDao::expect_soft_delete_bulk().times(0)`. Schließt Threat-Model T-04-04-01 (Plan 04-04). | 1 | unit | `cargo test -p service_impl test::extra_hours::soft_delete_bulk_forbidden_for_unprivileged_user` | ❌ W1 | ⬜ pending |
| Wave-0 Hygiene | Standalone `cargo test -p dao` und `cargo test -p dao_impl_sqlite` grün (D-Phase4-15) | 0 | unit | `cargo test -p dao && cargo test -p dao_impl_sqlite` | ✅ existing tests; W0-Cargo.toml-Patch macht sie grün | ⬜ pending |

*Status legend: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements (Pre-Implementation)

- [ ] `dao/Cargo.toml` — `uuid = { version = "1.8", features = ["v4"] }` (D-Phase4-15)
- [ ] `dao_impl_sqlite/Cargo.toml` — `uuid = { version = "1.8.0", features = ["v4"] }` (D-Phase4-15)
- [ ] `migrations/sqlite/<TS>_create-absence-migration-quarantine.sql` (D-Phase4-03)
- [ ] `migrations/sqlite/<TS+1>_create-absence-period-migration-source.sql` (D-Phase4-04)
- [ ] `migrations/sqlite/<TS+2>_create-employee-yearly-carryover-pre-cutover-backup.sql` (D-Phase4-13)
- [ ] `migrations/sqlite/<TS+3>_add-cutover-admin-privilege.sql` (D-Phase4-07 + C-Phase4-08)
- [ ] `rest/Cargo.toml` — `[dev-dependencies] insta = { version = "1.47.2", features = ["json"] }` (D-Phase4-11)
- [ ] `rest/tests/openapi_snapshot.rs` — Skeleton-Test mit `#[ignore]` (Wave 2 macht `cargo insta accept`)
- [ ] `.planning/migration-backup/` Verzeichnis (mit `.gitkeep`) — für Diff-Report-JSON-Files (D-Phase4-06)
- [ ] `.planning/phases/04-migration-cutover/deferred-items.md` — `localdb.sqlite3`-Drift-Hinweis (D-Phase4-15)
- [ ] `service/src/cutover.rs` Trait-Stub + `service/src/carryover_rebuild.rs` Trait-Stub + `service/src/lib.rs` Mod-Imports + `ExtraHoursCategoryDeprecated`-Variante in `ServiceError`-Enum
- [ ] `service_impl/src/test/cutover.rs` Skeleton mit `#[ignore]`-Tests pro Verification-Map-Eintrag (Test-First-Stubs, ermöglichen Sampling während Wave 1)

*Wave 0 ist atomar mit Wave 1-3 (Phase 4 ist atomar laut ROADMAP), aber W0-Tasks sind so klein und additiv, dass sie als separater Wave-Block behandelt werden für Plan-Lesbarkeit.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `cargo insta accept` für `rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap` (Wave 2) | MIG-05 | Snapshot-Inhalt muss von Mensch reviewt werden — Insta-Convention. Der Test schreibt automatisch `.snap.new`-Files, aber das Pin-File wird erst durch explizite Mensch-Bestätigung committet. Reviewer prüft: 3 `/admin/cutover/*`-Pfade + 6 Cutover-DTOs (Gate + Profile + 403-Error). | 1) `cargo test -p rest --test openapi_snapshot` 2) Mensch reviewt `.snap.new`-Diff via `git diff` (kein `cargo insta` global installiert — Memory-Note) 3) Falls OK: rename `.snap.new` → `.snap` ODER `cargo insta accept --workspace-root rest` (lokale `cargo install cargo-insta` nur falls explizit erlaubt) 4) jj-Commit |
| Production-Data-Profile-Run + Diff-Report-Review (SC-1, MIG-02) im Live-System | SC-1 + MIG-02 | Echtes Production-Data-Volume + HR-Sichtprüfung der Quarantäne-Reasons können nur im Operations-Kontext erfolgen. Tests decken Schema + Logik ab; Operations-Run ist die letzte Konsistenz-Prüfung. | 1) HR ruft `POST /admin/cutover/profile` (per Plan 04-06 wired) UND `POST /admin/cutover/gate-dry-run` 2) Reviewt Profile-JSON in `.planning/migration-backup/profile-{ts}.json` UND Diff-Report-JSON in `.planning/migration-backup/cutover-gate-{ts}.json` 3) Reviewt Quarantäne-Rows aus DB (manuell SQL oder zukünftiger GET-Endpunkt) 4) Bei OK: ruft `POST /admin/cutover/commit` |
| Bin-Boot-Smoke (`cargo run` mit 30s-Timeout) — verifiziert DI-Verdrahtung | MIG-04 (Atomic Tx requires correct service tree) | `cargo test --workspace` validiert Logik, aber nicht ob `shifty_bin/src/main.rs` mit dem neuen `CutoverService` + `CarryoverRebuildService` korrekt verdrahtet ist (DI-Order: FeatureFlagService MUSS vor ExtraHoursService konstruiert werden — Research-Finding) | `timeout 30 cargo run` mit `RUST_LOG=info`; Erwartung: "Server listening on …"-Log; Exit-Code 124 (Timeout durch Server-Run) ist OK, beliebiger anderer Exit-Code = Boot-Fehler |

---

## Validation Sign-Off

- [ ] Alle Tasks haben `<automated>` verify-command oder Wave 0 Stub-Dependency
- [ ] Sampling continuity: keine 3 konsekutiven Tasks ohne automated verify
- [ ] Wave 0 deckt alle ❌-MISSING-References aus Per-Task-Map ab
- [ ] Keine watch-mode-Flags (`cargo watch`-Subaufrufe verboten)
- [ ] Feedback latency < 60s (Full Suite); < 5s (Quick) — verifiziert via `time cargo test ...`
- [ ] OpenAPI-Snapshot-Determinismus: 3× hintereinander `cargo test -p rest --test openapi_snapshot` produziert keine `.snap.new`-Files
- [ ] Atomic-Rollback-Verhalten manuell verifiziert via Test `test_atomic_rollback_on_subservice_error`
- [ ] Threat-Model T-04-04-01 (`soft_delete_bulk` Elevation-of-Privilege) abgedeckt durch `soft_delete_bulk_forbidden_for_unprivileged_user`
- [ ] `nyquist_compliant: true` in Frontmatter setzen, sobald die obigen Boxen alle ✓ sind

**Approval:** pending
