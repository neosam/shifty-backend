---
phase: 61-personservice-aufl-sung-von-sales-person-user
verified: 2026-09-17T00:00:00Z
status: passed
score: 5/5 must-haves verified
behavior_unverified: 0
overrides_applied: 0
---

# Phase 61: PersonService + Auflösung von `sales_person_user` — Verification Report

**Phase Goal:** Alle Lese- und Schreibpfade, die heute über `sales_person_user`
laufen, laufen über die geteilte `person`-Identität; die Verknüpfungstabelle ist
aufgelöst.

**Verified:** 2026-09-17
**Status:** passed
**Re-verification:** No — initial verification (first VERIFICATION.md for this phase)

## Goal Achievement

This phase ran in two rounds: 61-01…61-06 (original build) and 61-07…61-10
(gap closure against `61-REVIEW.md`, which found 2 critical + 7 warning + 7 info
findings). Both rounds were audited against the actual codebase, not against
SUMMARY.md prose. Every claim below was independently re-derived from source
files, the delivered migration, and — where the finding was security- or
data-integrity-relevant — a live single-named-test run (not the full suite,
not a re-narration of the executor's own gate output).

### Observable Truths (mapped to ROADMAP Success Criteria 1–5)

| # | Truth (Roadmap SC) | Status | Evidence |
|---|---|---|---|
| 1 | SC1 — Ein Mitarbeiter sieht/bucht nach der Umstellung genau dieselben eigenen Daten wie vorher; Vorher/Nachher-Matrix (self/foreign/unlinked) belegt es | ✓ VERIFIED | `service_impl/src/test/person_self_gate.rs` implements `self_may_access_own_sales_person`, `foreign_sales_person_rejected`, `unlinked_user_rejected`, plus the fail-open regression `null_person_id_does_not_match_null_person_id`. Ran `null_person_id_does_not_match_null_person_id` standalone (`cargo test -p service_impl --lib test::person_self_gate::null_person_id_does_not_match_null_person_id -- --exact`) → **1 passed**. The byte-identity old-vs-new comparison test existed in Plan 61-03 (`byte_identity_old_vs_new_query`, confirmed in `61-03-SUMMARY.md`) and was deliberately removed in the DROP wave (61-05) per the Nyquist validation plan's own instruction ("seine Alt-Query-Hälfte muss in der DROP-Welle wieder entfernt werden") — not a silent drop. |
| 2 | SC2 — `sales_person_user` existiert nicht mehr; grep über `dao_impl_sqlite/`, `service_impl/`, `dao/` findet den Namen nur noch in historischen Migrationen | ✓ VERIFIED | `grep -rln "sales_person_user" dao/src dao_impl_sqlite/src service_impl/src service/src` → **no results**. `DROP TABLE IF EXISTS sales_person_user;` present in `migrations/sqlite/20260917000000_drop-sales-person-user.sql:233`. `docs/` and `AUTHENTICATION.md` also clean after the WR-06 fix (`grep -rn "sales_person_user" docs/ AUTHENTICATION.md` → no results). |
| 3 | SC3 — `SalesPersonService`-Signaturen unverändert, ~50 Aufrufstellen in `service_impl/` unberührt im Diff | ✓ VERIFIED | `git diff --stat 17ba8f7c^..ab8b4d8d -- service_impl/` shows only `person.rs` (new), `sales_person.rs` (+59/-, the `set_user`/`assign_to_user`/`discard_assigned_user` rewrite itself, in-scope), `lib.rs` (+1, module registration), and five `test/person_*.rs` files (new/replaced test infra). No caller of `SalesPersonService` elsewhere in `service_impl/` appears in the diff. `service/src/sales_person.rs` and `dao/src/sales_person.rs` show no signature changes (only `dao/src/lib.rs` +1 for the new `PersonDao` module registration). |
| 4 | SC4 — Ambiguous merge → HTTP 409 (distinguishable), complementary merge of two unlinked objects → exactly one shared person | ✓ VERIFIED | `service_impl/src/test/person_merge.rs` covers `merge_zweier_freier_objekte_ergibt_eine_person`, `konflikt_ziel_benutzer_haengt_an_anderer_sales_person`, `konflikt_sales_person_haengt_an_anderem_benutzer`, `komplementaeres_verschmelzen_ist_erlaubt`, `konflikt_wird_als_409_sichtbar`. The gap-closure fix for WR-04 (split path previously surfaced as 500 instead of 409) is present in `service_impl/src/sales_person.rs:327-335` and independently confirmed passing: `cargo test -p service_impl --lib test::sales_person::set_user_none_uebersetzt_split_konflikt_nach_409 -- --exact` → **1 passed**. |
| 5 | SC5 — `PersonService` is Basic-Tier (only `PersonDao`+Permission+Clock/Uuid+Transaction), constructed in the Basic block, `cargo clippy -- -D warnings` green, Service-Tier + Auth docs bilingually updated in the same commit | ✓ VERIFIED | `service_impl/src/person.rs:37-43` `PersonServiceDeps` lists exactly `PersonDao`, `PermissionService`, `ClockService`, `UuidService`, `TransactionDao` — no domain service. `shifty_bin/src/main.rs:1237-1250` constructs `person_service` with an explicit comment ("Basic-Tier Person-Service ... Basic-Block, strikt vor der Business-Logic-Schicht"), before `reporting_service`/`voluntary_stats_service`/`rebooking_reconciliation_service`. Clippy/tests already confirmed green by the orchestrator at `ab8b4d8d` (0 warnings, 0 failed, 1082 passed) — not re-run here per the task brief. Docs pairing confirmed by commit inspection (see Docs Pairing section below). |

**Score:** 5/5 truths verified (0 present-but-behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `dao/src/person.rs`, `dao_impl_sqlite/src/person.rs` | Basic-Tier `PersonDao` trait + SQLite impl, LEFT-JOIN anti-join candidate reads (D-61-02) | ✓ VERIFIED | `sales_persons_without_user`/`users_without_sales_person` use `LEFT JOIN ... WHERE u.name IS NULL` / `sp.id IS NULL` — not `person_id IS NULL` — matching D-61-02 exactly. |
| `service/src/person.rs`, `service_impl/src/person.rs` | `PersonService` trait + Basic-Tier impl | ✓ VERIFIED | See SC5 above. |
| `dao_impl_sqlite/src/sales_person.rs` | Four read paths rewritten as SQL joins on `person_id` (D-61-08); merge/split as person operations (D-61-03/D-61-05) | ✓ VERIFIED | `find_by_user`, `find_sales_person_by_user_id`, `get_assigned_user`, `get_all_user_assignments` all use `JOIN user u ON u.person_id = sp.person_id` (SQL NULL≠NULL semantics — fail-closed by construction, never a Rust `Option<Uuid>` comparison). `assign_to_user` merges onto the sales person's identity and deletes the absorbed person (2× `NOT EXISTS` guard); `discard_assigned_user` creates a fresh person for the displaced user. |
| `migrations/sqlite/20260917000000_drop-sales-person-user.sql` | 7-statement Block A (converges, doesn't diverge) + Block B (LEFT-JOIN assertion with 2 global coverage terms) + Block C (DROP) | ✓ VERIFIED | Statement order confirmed by direct file read: A3 (decouple, lines 77-101) precedes A5 (merge, lines 109-141); A7 (carrier sweep, lines 166-179) follows A6 (person INSERT, lines 143-164). Block B (lines 212-228) uses two `LEFT JOIN`s plus both global `COUNT(*) ... IS NULL = 0` terms. All 27 `person_link_migration_test` cases pass (see Behavioral Spot-Checks). |
| `dao_impl_sqlite/src/lib.rs::clear_all` | No orphaned `user.person_id` after clear (CR-02 fix) | ✓ VERIFIED | `DELETE FROM person WHERE NOT EXISTS (...sales_person...) AND NOT EXISTS (...user...)` — wording is near-identical to migration statement A7, as the plan required. Regression test `clear_all_laesst_keinen_benutzer_ohne_person_zurueck` confirmed passing standalone. |
| `AUTHENTICATION.md` | No longer documents dropped table as current schema (WR-06 fix) | ✓ VERIFIED | `grep -n "sales_person_user" AUTHENTICATION.md` → no results. Commit `dc43e472` rewrote the schema section. |
| `service_impl/src/billing_period_report.rs::CURRENT_SNAPSHOT_SCHEMA_VERSION` | Stays 13 | ✓ VERIFIED | `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13;` — unchanged. |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `migrations/.../20260917000000_drop-sales-person-user.sql` (Block A) | `dao_impl_sqlite/src/sales_person.rs` (`assign_to_user`/`discard_assigned_user`) | Same winner logic (D-61-03/D-61-05) applied in both the migration's catch-up backfill and the runtime path | ✓ WIRED | Migration comment explicitly states this equivalence (lines 18-24); code review confirmed both use "sales person keeps/generates identity, user follows" as the rule, cross-checked line-by-line above. |
| `shifty_bin/src/person_link_migration_test.rs` | `migrations/.../20260917000000_drop-sales-person-user.sql` | `include_str!` — tests run the *actual* shipped file | ✓ WIRED | Confirmed via `DROP_MIGRATION_SQL` constant and successful compilation/execution against the real file (all 27 tests pass, including the two spot-checked here independently). |
| `shifty_bin/src/main.rs` (DI) | `service_impl::person::PersonServiceImpl` | Constructed in Basic block, wired into `RestStateImpl` | ⚠️ ORPHANED (accepted, documented) | `RestStateDef::PersonService`/`person_service()` has no caller yet (Review IN-02) — deliberately deferred to Phase 62's first consuming endpoint, recorded in `61-07-PLAN.md` Deferral Record and `STATE.md:41`. Does not block this phase's goal (no endpoint/TO change is in scope per `61-CONTEXT.md`). |

### Behavioral Spot-Checks (independently executed, not re-narrated from SUMMARY)

| Behavior | Command | Result | Status |
|---|---|---|---|
| Fail-open protection (D-61-08 core security claim) | `cargo test -p service_impl --lib test::person_self_gate::null_person_id_does_not_match_null_person_id -- --exact` | `1 passed; 0 failed` | ✓ PASS |
| CR-01 fix — window-state merge converges instead of aborting | `cargo test -p shifty_bin --bin shifty_bin person_link_migration_test::nachzug_verschmilzt_im_fenster_verknuepftes_bestandspaar -- --exact` | `1 passed; 0 failed` | ✓ PASS |
| WR-05 fix — orphaned link row still aborts the migration | `cargo test -p shifty_bin --bin shifty_bin person_link_migration_test::selbst_assertion_bricht_bei_verwaister_verknuepfung_ab -- --exact` | `1 passed; 0 failed` | ✓ PASS |
| CR-02 fix — clear_all leaves no dangling `user.person_id` | `cargo test -p service_impl --lib test::person_creation::clear_all_laesst_keinen_benutzer_ohne_person_zurueck -- --exact` | `1 passed; 0 failed` | ✓ PASS |
| WR-04 fix — split-path conflict now surfaces as 409, not 500 | `cargo test -p service_impl --lib test::sales_person::set_user_none_uebersetzt_split_konflikt_nach_409 -- --exact` | `1 passed; 0 failed` | ✓ PASS |
| Full migration regression suite (existence + count proof, run once) | `cargo test -p shifty_bin --bin shifty_bin person_link_migration` | `27 passed; 0 failed` (matches the plan's acceptance criterion of exactly 4 more than the prior 23) | ✓ PASS |

### Probe Execution

No `scripts/*/tests/probe-*.sh` convention or PLAN/SUMMARY-declared probes found for this phase. **Step 7c: SKIPPED (no declared or conventional probes for this phase — verification relies on named cargo tests instead).**

### Docs Pairing (EN/DE, per commit)

| Commit | EN file(s) | DE file(s) | Paired? |
|---|---|---|---|
| `d51ce9ce` (61-01) | `02-service-tiers.md` | `02-service-tiers_de.md` | ✓ |
| `225d62c1`/`17ba8f7c` (61-01) | `F01-employee-management.md` | `F01-employee-management_de.md` | ✓ |
| `03a41df4` (61-04) | `F01-employee-management.md` | `F01-employee-management_de.md` | ✓ |
| `024eba93` (61-06) | `04-auth.md`, `F12-auth-session.md` | `04-auth_de.md`, `F12-auth-session_de.md` | ✓ (plus `service-graph-runtime.mmd`, diagram-only, no DE counterpart expected) |
| `1a79fc64` (61-06) | `03-data-model.md`, `F01-employee-management.md` | `03-data-model_de.md`, `F01-employee-management_de.md` | ✓ (plus `db-schema-er.mmd`, diagram-only) |
| `633106fc` (61-07) | `03-data-model.md` | `03-data-model_de.md` | ✓ |
| `dc43e472` (61-10) | `AUTHENTICATION.md` | — | N/A — root-level single file, no `_de` counterpart exists or is expected anywhere in the repo for this file |

`docs/arc42/*` is confirmed English-only by repo convention (not drift, per task brief).

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| PERS-05 | All 10 plans (61-01…61-10) | Self-access check works via person link instead of `sales_person_user`, same-data guarantee held | ✓ SATISFIED | REQUIREMENTS.md line 88 marks PERS-05 "Complete"; SC1–SC5 evidence above independently confirms it. No orphaned requirement IDs — PERS-05 is the only ID mapped to Phase 61 in REQUIREMENTS.md, and it appears in every plan's frontmatter. |

### Anti-Patterns Found

No `TBD`/`FIXME`/`XXX` unreferenced debt markers found in the phase's modified files (`dao/src/person.rs`, `dao_impl_sqlite/src/person.rs`, `dao_impl_sqlite/src/sales_person.rs`, `dao_impl_sqlite/src/lib.rs`, `service/src/person.rs`, `service_impl/src/person.rs`, `service_impl/src/sales_person.rs`, the migration file, `AUTHENTICATION.md`). All review findings (CR-01, CR-02, WR-04, WR-05, WR-06) that were scheduled for this gap-closure round are fixed and independently confirmed above. The remaining findings (WR-01, WR-02, WR-03, WR-07, IN-01…IN-07) are explicitly recorded as deferred — see Deferred Items below — not silently dropped.

### Deferred Items

Not phase failures — pre-existing, deliberately triaged, and traceable to a recorded decision. Listed per the task brief's "known open items" so they are not mistakenly re-reported as new gaps.

| # | Item | Addressed In / Status | Evidence |
|---|---|---|---|
| 1 | WR-02 — `users_without_sales_person` misses a user whose only sales person is soft-deleted | Phase 62 | `61-07-PLAN.md` Deferral Record: "Phase 62 besitzt die Kandidatenlisten und das Aggregat-Read; die Semantikentscheidung ... ist dort eine Produktfrage". Confirmed live in `dao_impl_sqlite/src/person.rs:173-189` (unchanged since review). |
| 2 | WR-01, WR-03, WR-07, IN-01…IN-07 | Backlog | `61-07-PLAN.md` Deferral Record table; `STATE.md:41` — "Weiterhin aufgeschoben: WR-02 (Phase 62), WR-01, WR-03, WR-07 und IN-01..IN-07 (Backlog)." Spot-checked WR-03 (`PermissionDaoImpl::create_user` still opens its own pool transaction, `dao_impl_sqlite/src/lib.rs:94-137`) — confirmed present and unfixed, matching the deferral. |
| 3 | `AUTHENTICATION.md` still describes a `MockContext` type | Pre-existing (arc42 D3 second finding) | `grep -rn "struct MockContext\|type MockContext" --include="*.rs"` over the whole repo → no results; the doc reference is stale but was not introduced by this phase and is out of the WR-06 fix's scope (which only addressed the `sales_person_user` schema section). |
| 4 | `openspec/specs/dev-seed/spec.md:79` claims dev-seed reset deletes rows from the dissolved table | Pre-existing | Confirmed line still present; out of this phase's docs-freshness trigger list (openspec is not one of the CLAUDE.md trigger targets for a migration change). |
| 5 | ROADMAP progress row for Phase 60 reads `0/? \| Not started` | Pre-existing drift | Confirmed at `.planning/ROADMAP.md:607`. |

**Environment note (not a phase defect):** `rustfmt` is absent from the Nix dev shell while CI runs `cargo fmt --all -- --check`; the repo already carries formatting drift in ~135 files this phase never touched. The fmt gate could not be verified locally in this phase or in this verification pass, and nothing was reformatted as part of either.

### Human Verification Required

None. Every must-have in this phase resolves to a grep-checkable artifact state, a migration-order invariant, or a passing named test — no visual, real-time, or external-service behavior is in scope (backend-only phase, no frontend/endpoint change per `61-CONTEXT.md`).

### Gaps Summary

None. Both the original build (61-01…61-06) and the gap-closure round (61-07…61-10) were independently re-derived from source:

- The security-critical self-access gate (D-61-08) is confirmed to decide membership entirely in SQL via `person_id` joins in all four read paths — never a Rust `Option<Uuid>` comparison — and the fail-open regression test passes standalone.
- The DROP migration's Block A statement order was read directly from the shipped file and matches the required convergence order (A3 before A5, A7 after A6); Block B uses LEFT JOINs plus both global coverage terms. Both the CR-01 scenario (window-state merge) and the WR-05 scenario (orphaned link abort) were run as standalone named tests against the actual file via `include_str!` and passed.
- `clear_all` (CR-02) and the split-path 409 translation (WR-04) were independently confirmed via standalone test runs, not just SUMMARY narration.
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` is unchanged at 13.
- Docs pairing (EN/DE, same commit) holds for every triggered file across all six documentation commits in this phase.
- SC3's "≈50 call sites untouched" claim is confirmed via `git diff --stat` over the phase's full commit range — no caller of `SalesPersonService` outside `person.rs`/`sales_person.rs`/tests appears in the diff.

The one design-intent nuance worth surfacing (not a gap): `PersonService` has no consumer yet (`RestStateDef::PersonService` is unused, Review IN-02) — this is intentional per `61-CONTEXT.md` ("kein Endpoint neu, kein Frontend-Anteil") and is explicitly recorded as deferred to Phase 62's first consuming endpoint, not an oversight.

Phase goal achieved. Ready to proceed.

---

_Verified: 2026-09-17_
_Verifier: Claude (gsd-verifier)_
