---
phase: 1
slug: absence-domain-foundation
status: planned
nyquist_compliant: true
wave_0_complete: false
created: 2026-05-01
updated: 2026-05-01
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source of truth: `01-RESEARCH.md` § Validation Architecture (Allen-inclusive overlap, `_forbidden` pro Methode, Additivitäts-Beweis).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `tokio` 1.44 + `mockall` 0.13 + Rust-Standard-Test-Harness |
| **Config file** | none (Cargo defaults) |
| **Quick run command** | `cargo test -p service_impl test::absence` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~5 s quick / ~60–120 s full (workspace) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p service_impl test::absence`
- **After every plan wave:** Run `cargo test -p service_impl && cargo test -p shifty_bin integration_test::absence_period`
- **Before `/gsd-verify-work`:** `cargo test --workspace` must be green (CC-08)
- **Max feedback latency:** 5 s for quick run

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 0.1 | 01-00 | 0 | ABS-01 | T-01-00-01, T-01-00-02 | DB CHECK + partial unique invariant | integration (DB-schema) | `nix-shell -p sqlx-cli --run "sqlx migrate run --source migrations/sqlite"` | ❌ W0 (creates) | ⬜ pending |
| 0.2 | 01-00 | 0 | ABS-03 | — | Inclusive Allen overlap correctness | unit | `cargo test -p shifty-utils date_range` | ❌ W0 (creates) | ⬜ pending |
| 0.3 | 01-00 | 0 | ABS-03 | — | Validation channel for OverlappingPeriod | regression | `cargo build --workspace` | ✅ existing (extends `service::ValidationFailureItem`) | ⬜ pending |
| 0.4 | 01-00 | 0 | (additivity) | — | Wave 0 build/test gate | regression | `cargo test --workspace` | ✅ existing | ⬜ pending |
| 1.1 | 01-01 | 1 | ABS-01, ABS-02 | T-01-01-01 | Trait + automock; enum-Surface compiletime-bounded | unit (compile) | `cargo build -p dao` | ❌ W0 (creates) | ⬜ pending |
| 1.2 | 01-01 | 1 | ABS-02 | T-01-01-02, T-01-01-03 | `WHERE deleted IS NULL` invariant + Two-Branch find_overlapping | unit (compile) + sqlx-prepare | `cargo build -p dao_impl_sqlite` | ❌ W0 (creates) | ⬜ pending |
| 1.3 | 01-01 | 1 | (additivity) | — | DAO-Smoke; existing tests stable | regression | `cargo build --workspace && cargo test -p dao -p dao_impl_sqlite -p shifty-utils` | ✅ existing | ⬜ pending |
| 2.1 | 01-02 | 2 | ABS-01, ABS-03 | — | Domain model + automock | unit (compile) | `cargo build -p service` | ❌ W0 (creates) | ⬜ pending |
| 2.2 | 01-02 | 2 | ABS-03, ABS-05 | T-01-02-01..04 | gen_service_impl! DI; HR ∨ self; logical_id-Update; Self-Overlap with exclude | unit (compile) | `cargo build -p service_impl` | ❌ W0 (creates) | ⬜ pending |
| 2.3 | 01-02 | 2 | ABS-03, ABS-05 | T-01-02-01..04 | `_forbidden` per public method; OverlappingPeriod assertion; D-15 mock predicate | unit (mock) | `cargo test -p service_impl test::absence` | ❌ W0 (creates) | ⬜ pending |
| 2.4 | 01-02 | 2 | (additivity) | — | Service-Layer-Smoke | regression | `cargo build --workspace && cargo test -p service_impl` | ✅ existing | ⬜ pending |
| 3.1 | 01-03 | 3 | ABS-04 | — | DTO Schema + bidirektional roundtrip | unit (compile, both feature flags) | `cargo build -p rest-types && cargo build -p rest-types --features service-impl` | ✅ existing (lib.rs) | ⬜ pending |
| 3.2 | 01-03 | 3 | ABS-04 | T-01-03-01, T-01-03-04 | utoipa-Annotation per handler; path-id wins; error_handler wrapper | unit (compile) | `cargo build -p rest` | ❌ W0 (creates) | ⬜ pending |
| 3.3 | 01-03 | 3 | ABS-04 | T-01-03-02 | RestStateDef extension; ApiDoc-Nest; Router-Nest `/absence-period` | unit (compile) | `cargo build -p rest && cargo build --workspace` (shifty_bin will fail until plan 04) | ✅ existing (lib.rs) | ⬜ pending |
| 3.4 | 01-03 | 3 | (additivity) | — | REST-Smoke; service_impl tests stable | regression | `cargo build -p rest -p rest-types -p service_impl && cargo test -p service_impl test::absence` | ✅ existing | ⬜ pending |
| 4.1 | 01-04 | 4 | ABS-01..05 | T-01-04-01 | DI verdrahtet; Workspace baut komplett | unit (compile) | `cargo build --workspace` | ✅ existing (main.rs) | ⬜ pending |
| 4.2 | 01-04 | 4 | ABS-01..04 | T-01-04-02, T-01-04-03 | E2E CRUD + DB-CHECK + Partial-Unique + logical_id-Update + D-12 + D-15 + Soft-Delete | integration | `cargo test -p shifty_bin integration_test::absence_period` | ❌ W0 (creates) | ⬜ pending |
| 4.3 | 01-04 | 4 | (all) | T-01-04-04 | Phase-1-Final-Gate; Additivität gewahrt; CC-07 unverletzt | regression + manual diff | `cargo build --workspace && cargo test --workspace && git diff service_impl/src/billing_period_report.rs service_impl/src/reporting.rs service_impl/src/extra_hours.rs service_impl/src/booking.rs` | ✅ existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

### Phase Requirements → Test Skeleton (from RESEARCH.md)

The planner expanded each row below into one or more concrete tasks; every test below is now bound to a task ID:

| Req ID | Behavior | Test Type | Automated Command | Bound to Task |
|--------|----------|-----------|-------------------|---------------|
| ABS-01 | Entity persistiert, Soft-Delete-Spalte, `logical_id`-Spalte | integration | `cargo test -p shifty_bin integration_test::absence_period::test_create_assigns_id_equal_to_logical_id` | 4.2 |
| ABS-01 | DB-CHECK lehnt invertierten Range ab | integration | `cargo test -p shifty_bin integration_test::absence_period::test_check_constraint_rejects_inverted_range` | 4.2 |
| ABS-01 | Partial unique index erzwingt max. 1 aktive Row pro `logical_id` | integration | `cargo test -p shifty_bin integration_test::absence_period::test_partial_unique_index_enforces_one_active_per_logical_id` | 4.2 |
| ABS-02 | DAO `find_by_logical_id` filtert `deleted IS NULL` | unit + integration | `cargo test -p service_impl test::absence::test_update_unknown_logical_id_returns_not_found` | 2.3, 4.2 |
| ABS-02 | DAO `find_overlapping` Allen-inclusive | integration | `cargo test -p shifty_bin integration_test::absence_period::test_create_overlapping_same_category_returns_validation_error` | 4.2 |
| ABS-02 | DAO `find_overlapping` honors `exclude_logical_id` | integration | `cargo test -p shifty_bin integration_test::absence_period::test_update_can_extend_range_without_self_collision` | 4.2 |
| ABS-03 | Service Range-Validierung (`from > to` → `DateOrderWrong`) | unit | `cargo test -p service_impl test::absence::test_create_inverted_range_returns_date_order_wrong` | 2.3 |
| ABS-03 | Service Self-Overlap auf Same-Category | unit | `cargo test -p service_impl test::absence::test_create_self_overlap_same_category_returns_validation` | 2.3 |
| ABS-03 | Service Self-Overlap exkludiert eigene Row beim Update | unit (mock predicate) | `cargo test -p service_impl test::absence::test_update_self_overlap_excludes_self` | 2.3 |
| ABS-03 | Service Cross-Category darf überlappen (D-12) | unit + integration | `cargo test -p service_impl test::absence::test_create_self_overlap_different_category_succeeds` und `cargo test -p shifty_bin integration_test::absence_period::test_create_overlapping_different_category_succeeds` | 2.3, 4.2 |
| ABS-03 | Update `logical_id`-Pattern (tombstone + neue Row) | integration | `cargo test -p shifty_bin integration_test::absence_period::test_update_creates_tombstone_and_new_active_row` | 4.2 |
| ABS-04 | REST Routes vorhanden, OpenAPI registriert | unit (compile) + manual smoke | `cargo build -p rest` (Compile-Gate) — Swagger-UI manuell via `cargo run` | 3.2, 3.3 |
| ABS-04 | DTO Round-Trip serializes/deserializes (compile-time + service-impl roundtrip) | unit (compile both feature flags) | `cargo build -p rest-types && cargo build -p rest-types --features service-impl` | 3.1 |
| ABS-05 | `_forbidden`-Test pro public service method | unit (mock) | `cargo test -p service_impl test::absence -- forbidden` | 2.3 |
| ABS-05 | HR ∨ Self-Pattern beim Create | unit | `cargo test -p service_impl test::absence::test_create_other_sales_person_without_hr_is_forbidden` | 2.3 |
| (additivity) | Bestehende Tests bleiben grün | regression | `cargo test --workspace` | 4.3 |
| (additivity) | Snapshot-Schema-Versioning unverändert | manual gate (CC-07) | `git diff -- service_impl/src/billing_period_report.rs \| grep -i CURRENT_SNAPSHOT_SCHEMA_VERSION` MUSS leer sein | 4.3 |

---

## Wave 0 Requirements

> Files/scaffolding that must exist before any test can compile. Wave 0 of the plan installs them.

- [ ] `migrations/sqlite/<timestamp>_create-absence-period.sql` — Plan 01-00 Task 0.1
- [ ] `shifty-utils/src/date_range.rs` + re-export in `shifty-utils/src/lib.rs` — Plan 01-00 Task 0.2
- [ ] `service::ValidationFailureItem::OverlappingPeriod(Uuid)`-Variante — Plan 01-00 Task 0.3
- [ ] `dao/src/absence.rs` — Plan 01-01 Task 1.1
- [ ] `dao_impl_sqlite/src/absence.rs` — Plan 01-01 Task 1.2
- [ ] `service/src/absence.rs` — Plan 01-02 Task 2.1
- [ ] `service_impl/src/absence.rs` — Plan 01-02 Task 2.2
- [ ] `service_impl/src/test/absence.rs` + patch in `service_impl/src/test/mod.rs` — Plan 01-02 Task 2.3
- [ ] `rest-types/src/lib.rs` (inline AbsencePeriodTO + AbsenceCategoryTO; **NICHT** als per-domain TO-File — Repo-Konvention) — Plan 01-03 Task 3.1
- [ ] `rest/src/absence.rs` (handlers + ApiDoc + generate_route) — Plan 01-03 Task 3.2
- [ ] `rest/src/lib.rs` patches (mod, RestStateDef, ApiDoc-Nest, Router-Nest) — Plan 01-03 Task 3.3
- [ ] `shifty_bin/src/main.rs` patches (DI-Block, RestStateImpl-Erweiterung) — Plan 01-04 Task 4.1
- [ ] `shifty_bin/src/integration_test/absence_period.rs` + module patch in `shifty_bin/src/integration_test.rs` (NICHT `mod.rs` — verifiziert) — Plan 01-04 Task 4.2

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions | Bound to Task |
|----------|-------------|------------|-------------------|---------------|
| Snapshot-Schema-Versioning unverändert | additivity invariant (CC-07) | `CURRENT_SNAPSHOT_SCHEMA_VERSION` darf in Phase 1 NICHT bumpen — wird in Phase 2 gebumpt, sobald Reporting konsumiert | Vor PR/Merge: `git diff -- service_impl/src/billing_period_report.rs` darf den Konstantenwert nicht verändern. | 4.3 |
| Reporting-/Booking-/Snapshot-Pfade additiv unberührt | Phase-1-Erfolgskriterium 5 | Bit-Identität messen ist mit Mocks teuer; einfacher: keine Diffs in den entsprechenden Dateien | `git diff -- service_impl/src/{reporting,booking,billing_period_report,extra_hours}.rs` muss leer sein | 4.3 |
| OpenAPI Swagger-UI zeigt `/absence-period`-Routen | ABS-04 (UX-Aspekt) | Swagger-UI-Rendering ist kein Test-Target; Smoke-Run bestätigt es visuell | `cargo run` (mit Timeout) → http://localhost:3000/swagger-ui prüfen, `AbsencePeriodTO` und 6 Routen sichtbar | 4.3 |

---

## Pinned Discretion Items

The planner has resolved the three open items from RESEARCH.md (A1, A2, A3) plus the TO-file convention anomaly:

| ID | Decision | Rationale | Where Pinned |
|----|----------|-----------|--------------|
| A1 | **`ValidationFailureItem::OverlappingPeriod(Uuid)`** als neue Variante (NICHT `Duplicate`-Reuse mit Kontext-String). | Sprechende Variante; UI kann konflikthafte logical_id direkt verlinken. Phase 3 (BOOK-01) kann analog `OverlappingBooking(Uuid)` ergänzen. | Plan 01-00 Task 0.3 |
| A2 | **Option A** für D-10 Read-Sicht in Phase 1: HR ∨ self only; "Schichtplan-Kollege"-Erweiterung deferred to Phase 3. | Vermeidet Phase-1-Scope-Creep auf einen nicht-existenten Service-API; Phase 3 (PLAN-01/BOOK-01) braucht das Konzept ohnehin. KEIN `SalesPersonShiftplanService`-Dependency in `AbsenceServiceImpl`. | Plan 01-02 Task 2.2 (gen_service_impl!-Block, Permission-Pattern in find_by_id und find_by_sales_person) |
| A3 | **`cargo sqlx prepare --workspace`** läuft als Pflicht-Schritt nach Migration und nach DAO-Impl. `.sqlx/`-Cache existiert im Repo (verifiziert: `ls .sqlx/ \| wc -l > 0`). | Repo nutzt Offline-Build; ohne Cache-Update bricht CI. | Plan 01-00 Task 0.1 + Plan 01-01 Task 1.2 |
| TO-File | **Inline DTOs in `rest-types/src/lib.rs`** (NICHT `rest-types/src/absence_period_to.rs`). | Repo-Konvention verifiziert: `ls rest-types/src/` → nur `lib.rs`; alle bestehenden DTOs sind inline. RESEARCH-Vorschlag wird zugunsten der Repo-Konvention überschrieben. | Plan 01-03 Task 3.1 |
| Integration-Test-Module-Pfad | Module declaration in `shifty_bin/src/integration_test.rs` (NICHT `integration_test/mod.rs` — verifiziert: bestehende Module wie `extra_hours_update` sind dort als `mod extra_hours_update;` deklariert). | Repo-Konvention verifiziert. | Plan 01-04 Task 4.2 |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies (every task has `<verify><automated>...</automated></verify>` block).
- [x] Sampling continuity: no 3 consecutive tasks without automated verify (every task has its own automated check).
- [x] Wave 0 covers all MISSING references (Plan 01-00 erzeugt Migration + DateRange + ValidationFailureItem-Variante).
- [x] No watch-mode flags (no `cargo watch` in CI gates).
- [x] Feedback latency < 5 s for the per-task quick run (`cargo test -p service_impl test::absence`).
- [x] `nyquist_compliant: true` set in frontmatter — per-task verification map filled.

**Approval:** planner-approved 2026-05-01
