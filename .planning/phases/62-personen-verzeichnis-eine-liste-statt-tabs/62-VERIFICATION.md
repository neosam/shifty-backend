---
phase: 62-personen-verzeichnis-eine-liste-statt-tabs
verified: 2026-09-20T13:26:12Z
status: passed
score: 9/9 must-haves verified
behavior_unverified: 0
overrides_applied: 1
overrides:
  - must_have: "Jede Zeile traegt ein sichtbares Auspraegungs-Badge (PADM-01, D-62-01)"
    reason: "Das Badge wurde waehrend des UAT-Durchgangs von Plan 62-07 auf ausdrueckliche Entscheidung des Produktverantwortlichen entfernt (Zitat: 'Ja, die sieht man aber die Badges sind verwirrend... Man sieht ja Verkaeufer und Benutzername in der Tabelle', Freigabe: 'ja, mach das'). Ursache: Farbsprachen-Kollision mit dem Bezahlt/Freiwillig-Chip in derselben Tabellenzelle (beide nutzen accent/warn). Die Ausprägung bleibt vollstaendig ableitbar ueber zwei bestehende Zellen (Chip-Praesenz <=> Verkaeufer-Datensatz vorhanden; Benutzername-Zelle gefuellt <=> Login vorhanden) und ist per Regressionstest `directory_row_encodes_kind_without_badge` gegen beide Richtungen abgesichert. `PersonKindTO` bleibt im Transport-Objekt und im Service vollstaendig erhalten und steuert weiterhin Aktions-Sichtbarkeit (D-62-06) und das Ziel des Bearbeiten-Links (D-62-07) — nur die visuelle Pille entfiel. Der Nutzer hat die Aenderung nach Umsetzung im Browser bestaetigt ('Sieht gut aus')."
    accepted_by: "Nutzer (Produktverantwortlicher, UAT-Durchgang Plan 62-07, 2026-09-20)"
    accepted_at: "2026-09-20T00:00:00Z"
deferred:
  - truth: "Anlegen von Benutzerkonto und Verkaeufer laufen ueber einen vereinheitlichten Einstieg statt zweier getrennter Knoepfe"
    addressed_in: "Phase 63"
    evidence: "ROADMAP.md Phase 63 Success Criterion 2 ('Beim Anlegen erfasst der Admin einen Eintrag und schaltet dabei zu...') und Discuss-Point D-63-03, gepinnt aus der 62-07-UAT-Beobachtung ('Es gibt jetzt trotzdem neuen benutzer und neuen verkaeufer separat. Das checkt niemand.')."
---

# Phase 62: Personen-Verzeichnis — eine Liste statt Tabs Verification Report

**Phase Goal:** Ein Admin öffnet `/user_management/` und sieht **eine** durchsuchbare Liste aller Einträge mit erkennbarer Ausprägung — verknüpfte, reine Benutzer und reine Verkäufer nebeneinander, jeder mit einem fertig vom Backend gelieferten Namen, kein Tab-Wechsel mehr.
**Verified:** 2026-09-20T13:26:12Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

Truths 1–5 are the ROADMAP.md Success Criteria (the contract); 6–9 are hard project gates (CLAUDE.md) and cross-cutting invariants (D-62-05/WR-02) that the plans' `must_haves` require and that this verification independently re-ran rather than trusting SUMMARY claims.

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | SC1: `/user_management/` zeigt eine einzige Liste ohne Tab-Leiste; verknüpfte, reine Benutzerkonten und reine Verkäufer stehen nebeneinander, jeder mit erkennbarer Ausprägung | ✓ VERIFIED (badge item PASSED via override) | `shifty-dioxus/src/component/user_management_tab_bar.rs` deleted (`grep -rl 'UserManagementTabBar' shifty-dioxus/src/` → 0 hits); `PersonDirectoryTable` in `page/user_management.rs` renders one table fed by `entry.kind: PersonKindTO`; Ausprägung is derivable via two independently-tested signals (chip presence, username cell) — see `directory_row_encodes_kind_without_badge` (passing); confirmed in real browser by product owner during 62-07 UAT ("Sieht gut aus" after the badge-removal fix) |
| 2 | SC2: Die Suche findet einen Eintrag über den Namen, den er **hat** — Verkäufername, Username oder beides | ✓ VERIFIED | `matches_search` checked against both `entry.display_name` and `entry.username` (`page/user_management.rs:280,284`); tests `directory_search_matches_both_names`, `matches_search_is_case_insensitive` re-run and pass; confirmed by user in browser during 62-07 UAT ("Ja, wir finden beides") |
| 3 | SC3: Jeder Eintrag ist beschriftet; der Anzeigename kommt fertig aus dem Backend — grep-verifiziert keine Fallback-Regel im Frontend | ✓ VERIFIED | Re-ran independently: `grep -n 'display_name\|username' shifty-dioxus/src/page/user_management.rs \| grep -c 'unwrap_or\|\.or('` → 0; same check on `service/user_management.rs`, `state/user_management.rs`, `api.rs`, `loader.rs` → 0 each; `grep -rln 'display_name' shifty-dioxus/src/` → only `page/user_management.rs` (single source, no duplicated logic); `display_name` is derived once in `service_impl/src/person_directory.rs::entry_from_row` |
| 4 | SC4: Ein Nutzer ohne Admin-Rolle erreicht die Seite nicht (Route-Guard) **und** bekommt vom Aggregat-Endpoint keine Daten (Backend-Gate) — beide getestet | ✓ VERIFIED | Frontend: `is_admin`/`NotAuthorizedView` early-return before any hook/data access (`page/user_management.rs:61-95`); test `non_admin_sees_not_authorized`. Backend: `check_permission("admin", context)` is the first statement of `PersonDirectoryServiceImpl::list`, before `use_transaction` (`service_impl/src/person_directory.rs:110-121`); HTTP integration tests `admin_bekommt_200` / `hr_ohne_admin_bekommt_403` re-run independently and pass (`cargo test -p shifty_bin … integration_test::person_directory` → 2 passed). Runtime 403 additionally reproduced live in 62-07 via impersonation (`GET /person-directory` → 403 while impersonating a non-admin) |
| 5 | SC5: Die Liste wird aus **einem** Aggregat-Request gefüllt, und der Routen-Präfix steht im **selben Commit** in `Dioxus.toml` — per `dx serve`-Smoke verifiziert | ✓ VERIFIED | Unit test `load_directory_ist_ein_request` (source-assertion counting `.await` in the coroutine arm) re-run and passes; `git show --name-only --format= e0d8e8d3` lists both `rest/src/lib.rs` and `shifty-dioxus/Dioxus.toml`; 4-way prefix consistency `grep -rho "person-directory" rest/src/lib.rs shifty-dioxus/Dioxus.toml shifty-dioxus/src/api.rs \| sort -u \| wc -l` → 1; `dx serve` smoke in 62-07-SUMMARY.md shows port 3000 and port 8080 both returning 200 with byte-identical bodies, plus a validated DAO-query-log counter (2→3 on one proxy request) as an independent runtime request-count proof |
| 6 | Docs-Freshness-Gate: F01-employee-management, 02-service-tiers, service-graph-runtime.mmd updated in both EN and DE, matching the actual code | ✓ VERIFIED | `grep -c 'person_directory\|PersonDirectory' docs/features/F01-employee-management.md docs/features/F01-employee-management_de.md` → 25/25; `grep -c 'PersonDirectory' docs/architecture/02-service-tiers.md docs/architecture/02-service-tiers_de.md` → 4/4; `service-graph-runtime.mmd` carries a `PersonDirectory` node placed in the Business-Logic subgraph (not Basic) |
| 7 | D-62-05 / WR-02: A user whose only sales person is soft-deleted remains a re-link candidate | ✓ VERIFIED | `users_without_sales_person` soft-delete filter moved into the `LEFT JOIN`'s `ON` clause (not `WHERE`); regression test `user_mit_nur_soft_geloeschter_sales_person_ist_kandidat` re-run and passes as part of the full workspace suite |
| 8 | Workspace and shifty-dioxus tests are green; clippy is a clean hard gate on both | ✓ VERIFIED | Independently re-run (not taken from SUMMARY): `SQLX_OFFLINE=true cargo test --workspace` → all green (incl. 95 shifty_bin integration tests, 0 failed); `SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings` → no output; `cargo clippy --manifest-path shifty-dioxus/Cargo.toml --all-targets -- -D warnings` → no output; `cargo test --manifest-path shifty-dioxus/Cargo.toml` → 830 passed, 0 failed |
| 9 | No debt markers (TBD/FIXME/XXX) left in phase-touched files; requirement IDs fully accounted for | ✓ VERIFIED | `grep -n 'TBD\|FIXME\|XXX'` across all 11 phase-touched source files → no hits; requirement union across all 7 plan frontmatters = {PADM-01, PADM-06, PADM-08}, exactly matching ROADMAP.md's declared requirement set for Phase 62 — no orphaned requirements |

**Score:** 9/9 truths verified (0 present-but-behavior-unverified; 1 verified via documented override)

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Anlegen von Benutzerkonto und Verkäufer laufen weiterhin über zwei getrennte, kontextlose Buttons statt eines vereinheitlichten Einstiegs | Phase 63 | ROADMAP.md Phase 63 Success Criterion 2; Discuss-Point D-63-03; observation recorded verbatim in `62-07-SUMMARY.md` ("Es gibt jetzt trotzdem neuen benutzer und neuen verkaeufer separat. Das checkt niemand.") — explicitly confirmed by the user as *not* a Phase 62 gap, since unification of creation was never part of Phase 62's `must_haves` |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ------------- | ------ | ------- |
| `dao/src/person_directory.rs` | `PersonDirectoryDao` trait, mockable | ✓ VERIFIED | Exists, registered in `dao/src/lib.rs`, compiled and tested |
| `dao_impl_sqlite/src/person_directory.rs` | Three-way outer join impl | ✓ VERIFIED | Exists, six DB integration tests (`test/person_directory_dao.rs`) pass |
| `service/src/person_directory.rs` | `PersonDirectoryService` trait + domain types | ✓ VERIFIED | Exists, mockable, registered in `service/src/lib.rs` |
| `service_impl/src/person_directory.rs` | Admin-gated business-logic impl | ✓ VERIFIED | Exists; gate is first statement; `entry_from_row` derives kind/display_name once |
| `rest-types/src/lib.rs` | `PersonEntryTO`, `PersonKindTO` | ✓ VERIFIED | Both types present with `ToSchema`, wire-format tests pass |
| `rest/src/person_directory.rs` | `GET /person-directory` endpoint | ✓ VERIFIED | Route registered, OpenAPI documented, no second gate in handler (trust boundary stays in the service) |
| `shifty_bin/src/integration_test/person_directory.rs` | HTTP tests, both gate sides | ✓ VERIFIED | `admin_bekommt_200` / `hr_ohne_admin_bekommt_403` re-run, both pass |
| `shifty-dioxus/src/page/user_management.rs` | Rebuilt single-list page | ✓ VERIFIED | Tab bar removed, aggregate loader wired, admin guard, search, action gating, edit-link routing all present and tested |
| `shifty-dioxus/src/component/atoms/person_kind_badge.rs` | Ausprägungs-Badge atom | ⚠️ DELETED (documented) | Deleted during 62-07 UAT on explicit product-owner decision — see override above; not a stub/orphan, a deliberate removal with replacement signals |
| `shifty-dioxus/src/component/user_management_tab_bar.rs` | Old tab bar | ✓ CORRECTLY REMOVED | Confirmed deleted; zero references anywhere in `shifty-dioxus/src/` |
| `shifty-dioxus/Dioxus.toml` | `[[web.proxy]]` entry for `/person-directory` | ✓ VERIFIED | Present, same commit as the route (`e0d8e8d3`), smoke-tested against a live `dx serve` in 62-07 |
| `docs/features/F01-employee-management.md` (+`_de.md`) | Updated feature docs | ✓ VERIFIED | Section counts match, both languages structurally synced |
| `docs/architecture/02-service-tiers.md` (+`_de.md`) | Tier doc with new BL service | ✓ VERIFIED | `PersonDirectoryService` listed in Business-Logic tier, both languages |
| `docs/architecture/diagrams/service-graph-runtime.mmd` | Runtime graph node | ✓ VERIFIED | `PersonDirectory` node present in Business subgraph, not Basic |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `shifty-dioxus/src/api.rs::get_person_directory` | `rest/src/person_directory.rs` | HTTP GET `{backend}/person-directory` | ✓ WIRED | URL string identical across 4 locations (`.nest`, OpenAPI nest, frontend URL, Dioxus.toml proxy) — verified via `sort -u \| wc -l == 1` |
| `rest/src/person_directory.rs` handler | `service::person_directory::PersonDirectoryService::list` | `person_directory_service().list(context.into(), None)` | ✓ WIRED | Handler contains zero `check_permission` calls (single trust boundary in the service, confirmed) |
| `service_impl/src/person_directory.rs` | `dao::person_directory::PersonDirectoryDao` | `person_directory_dao.list_directory_entries(...)` + `all_user_roles(...)` | ✓ WIRED | Gate precedes DAO call; DAO call confirmed live via query-log counter in 62-07 |
| `shifty_bin/src/main.rs` | `PersonDirectoryServiceImpl` construction | Business-Logic DI block | ✓ WIRED | Constructed at line 1371, strictly after Basic-tier `person_service` (1278) and after `voluntary_stats_service` (1348) — no `OnceLock`/forward-declaration tricks |
| `shifty-dioxus/src/page/user_management.rs` row | Detail route | `sales_person_id.is_some()` → `Route::SalesPersonDetails`, else `Route::UserDetails` | ✓ WIRED | Confirmed in source (D-62-07); deterministic conditional, low complexity — not independently browser-clicked but source-conclusive |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `PersonDirectoryTable` (`page/user_management.rs`) | `user_management.directory` (`Rc<[PersonEntryTO]>`) | `UserManagementAction::LoadPersonDirectory` → `api::get_person_directory` → `GET /person-directory` → `PersonDirectoryService::list` → `PersonDirectoryDao::list_directory_entries` (real `sqlx::query_as!` three-way outer join) | Yes | ✓ FLOWING — confirmed live: `GET /person-directory` returned real seeded rows (`admin`, `DEVUSER`, and after manual seeding, `Rosa Verkauf` / `Béla Verknüpft` / `nur.benutzer`) in 62-07's runtime capture, not a static/empty stub |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Full workspace test suite | `nix develop --command bash -c 'SQLX_OFFLINE=true cargo test --workspace'` | all green, incl. 95 `shifty_bin` integration tests | ✓ PASS |
| shifty-dioxus test suite | `nix develop --command cargo test --manifest-path shifty-dioxus/Cargo.toml` | 830 passed, 0 failed | ✓ PASS |
| Workspace clippy hard gate | `nix develop --command bash -c 'SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings'` | no output | ✓ PASS |
| shifty-dioxus clippy hard gate | `nix develop --command cargo clippy --manifest-path shifty-dioxus/Cargo.toml --all-targets -- -D warnings` | no output | ✓ PASS |
| Named test: single-request proof | `cargo test --manifest-path shifty-dioxus/Cargo.toml -- service::user_management::tests::load_directory_ist_ein_request --exact` | 1 passed | ✓ PASS |
| Named test: admin/HR gate (HTTP) | `cargo test -p shifty_bin --bin shifty_bin integration_test::person_directory -- --nocapture` | 2 passed (`admin_bekommt_200`, `hr_ohne_admin_bekommt_403`) | ✓ PASS |
| Named test: i18n parity | `cargo test --manifest-path shifty-dioxus/Cargo.toml "i18n::tests::i18n_user_management_keys_present_in_all_locales" --exact` | 1 passed | ✓ PASS |

### Probe Execution

No `scripts/*/tests/probe-*.sh` convention or plan-declared probes exist for this phase — SKIPPED (no runnable probes; phase uses `cargo test` + a manual `dx serve` smoke documented in 62-07-SUMMARY.md, both independently re-verified above).

### Requirements Coverage

| Requirement | Source Plan(s) | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| PADM-01 | 62-01…62-07 (all 7) | Admin sees one searchable list with recognizable Ausprägung, no tab switch | ✓ SATISFIED | SC1/SC2/SC5 above; Ausprägung recognizable via documented, accepted override |
| PADM-06 | 62-02, 62-03, 62-04, 62-06, 62-07 | Display name comes ready-made from backend, no frontend fallback | ✓ SATISFIED | SC3 above, `entry_from_row` derivation, zero fallback grep hits |
| PADM-08 | 62-01(advanced), 62-03, 62-04, 62-05, 62-06, 62-07 | Person management fully admin-gated | ✓ SATISFIED | SC4 above, both route-guard and backend-gate independently tested |

**Orphaned requirements:** none — the union of `requirements:` across all 7 PLAN frontmatters ({PADM-01, PADM-06, PADM-08}) matches ROADMAP.md's declared requirement set for Phase 62 exactly.

**Note:** `.planning/REQUIREMENTS.md` traceability table still shows PADM-01/06/08 as `Pending` at verification time. This is a deliberate, documented process choice across all seven SUMMARYs (`requirements.mark-complete` was intentionally not invoked at plan level because the phase-level claim is only true once the whole phase lands) — not a gap. It is expected to be closed as part of phase-completion tooling following this verification, not something this report should silently paper over or unilaterally edit.

### Anti-Patterns Found

Scanned all 11 phase-touched source files (`dao/src/person_directory.rs`, `dao_impl_sqlite/src/person_directory.rs`, `service/src/person_directory.rs`, `service_impl/src/person_directory.rs`, `rest/src/person_directory.rs`, `rest-types/src/lib.rs`, `shifty-dioxus/src/page/user_management.rs`, `shifty-dioxus/src/service/user_management.rs`, `shifty-dioxus/src/api.rs`, `shifty-dioxus/src/loader.rs`, `shifty_bin/src/main.rs`) for `TBD`/`FIXME`/`XXX`/`TODO`/`HACK`/`PLACEHOLDER`/hardcoded-empty patterns.

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | none found | — | Clean. Two `#[allow(dead_code)] // reason: …` annotations exist (`api.rs::get_all_users`, `loader.rs::load_all_users`, `state/user_management.rs::User`) but each carries an explicit, phase-62/63-scoped justification (paths a follow-up phase needs) — not silent debt. |

### Human Verification Required

None outstanding. The phase's human checkpoint (Plan 62-07, Task 2) already ran as a full UAT session with the product owner against the live dev servers (documented in `62-07-SUMMARY.md`, section "UAT-Protokoll Task 2 — wer hat was bestaetigt"). All nine checkpoint steps are resolved: steps requiring genuine visual/interactive judgment (Ausprägung recognizability, search behavior, page reachability) were confirmed directly by the user in the browser; steps that are mechanical (request count, rejection status code, action gating) were confirmed by the orchestrator via runtime HTTP calls and SSR-rendered tests. The one substantive finding from that UAT (badge caused visual confusion) was fixed and re-confirmed by the user in the same session ("Sieht gut aus").

### Gaps Summary

No gaps. All ROADMAP.md Success Criteria for Phase 62 are independently verified against the codebase (not taken on SUMMARY.md's word) — file existence, wiring, a full independent re-run of `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, and the shifty-dioxus-specific equivalents, plus targeted greps for the fat-backend/no-fallback contract and the admin-gate-first-statement contract. The one documented deviation (D-62-01 Ausprägungs-Badge removed) is a deliberate, human-approved product decision with equivalent replacement signals and a regression test guarding it — handled here via a formal override rather than reported as a silent regression. The one deferred item (unified create entry point) is explicitly out of Phase 62's `must_haves` and is correctly pinned to Phase 63 (D-63-03, SC-2) in ROADMAP.md.

---

_Verified: 2026-09-20T13:26:12Z_
_Verifier: Claude (gsd-verifier)_
