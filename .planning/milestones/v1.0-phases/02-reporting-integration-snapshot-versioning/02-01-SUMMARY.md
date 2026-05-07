---
phase: 02-reporting-integration-snapshot-versioning
plan: 01
subsystem: testing
tags: [rust, testing, scaffolding, mockall, locking-test, phase-2-wave-0]

# Dependency graph
requires:
  - phase: 01-absence-domain-foundation
    provides: AbsenceService trait, AbsencePeriod domain, EmployeeWorkDetails fixtures, BillingPeriodValueType enum surface
provides:
  - 5 new test files in service_impl/src/test/ providing fixtures and red/ignored stubs that force Wave 1 and Wave 2 to deliver derive_hours_for_range, the snapshot-version bump, and the UnpaidLeave variant
  - Pin-Map locking test (RED until Wave 2) and Compiler-Exhaustive-Match test (compiles, will COMPILE-ERROR when Wave 2 adds UnpaidLeave variant unless the test is updated)
  - Pre-existing Phase-1 build hole in shifty-utils (DateRange) and service::ValidationFailureItem::OverlappingPeriod fixed as a Rule-3 blocking-issue auto-fix in a separate commit
affects: [02-02-PLAN, 02-03-PLAN, 02-04-PLAN]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Wave-0 test scaffolding pattern (RED stubs that force later waves)
    - Hybrid locking-test pattern (pin-map + compiler-exhaustive-match) per CLAUDE.md § Snapshot Schema Versioning
    - Reusable fixture-helper module per phase

key-files:
  created:
    - service_impl/src/test/reporting_phase2_fixtures.rs
    - service_impl/src/test/billing_period_snapshot_locking.rs
    - service_impl/src/test/absence_derive_hours_range.rs
    - service_impl/src/test/reporting_flag_off_bit_identity.rs
    - service_impl/src/test/reporting_flag_on_integration.rs
    - .planning/phases/02-reporting-integration-snapshot-versioning/02-01-SUMMARY.md
  modified:
    - service_impl/src/test/mod.rs
    - shifty-utils/src/date_utils.rs
    - shifty-utils/Cargo.toml
    - service/src/lib.rs

key-decisions:
  - "Phase-1-Pre-existing-Build-Holes (DateRange, OverlappingPeriod) als Rule-3-Blocking-Issue inline auto-fixed -- ohne Fix konnte cargo build -p service_impl --tests nicht passieren, was das gesamte Wave-0-Scaffolding blockierte."
  - "DateRange-API minimal: new(from, to)/from()/to()/day_count()/iter_days() + DateRangeIterator -- exakt was Phase-1 (range-Validierung) und Phase-2-RESEARCH (per-Tag-Iteration) erwarten."
  - "ValidationFailureItem::OverlappingPeriod(Uuid) hinzugefuegt, mit kommentiertem Phase-1-Verweis (D-13/D-15)."
  - "Phase-1-Fix in separatem Commit (szmrvxst) vor Wave-0-Tasks committed -- klare Trennung von Phase-2-Wave-0-Scope und Pre-existing-Phase-1-Cleanup."
  - "Stub-Tests mit #[ignore]-Attribute statt unimplemented!() im Body, sodass cargo test --workspace gruen bleibt -- nur der intentional-rote Pin-Test signalisiert Wave-2-Forcing."

patterns-established:
  - "Phase-2-Wave-0-Pattern: Tests als RED-Forcing-Mechanismus -- Pin-Test ROT bis Wave 2 das Bump macht; Match-Test wird COMPILE-ERROR sobald Wave 2 die neue Variante hinzufuegt (auskommentierter Arm wartet zur Aktivierung)."
  - "Fixture-Sharing-Pattern: pub fn fixture_*() in einem zentralen Modul (reporting_phase2_fixtures), importiert von allen Phase-2-Test-Dateien via use crate::test::reporting_phase2_fixtures::*."

requirements-completed: [SNAP-01, SNAP-02, REP-02]

# Metrics
duration: 12min
completed: 2026-05-02
---

# Phase 02 Plan 01: Wave-0 Test-Scaffolding Summary

**5 neue Test-Dateien erstellen das Build-Time-Gate, das Wave 2 zwingt, Snapshot-Bump 2 -> 3 und die UnpaidLeave-Variante einzufuehren -- plus Phase-1-Pre-existing-Build-Holes (DateRange, OverlappingPeriod) als Rule-3-Blocking-Issue inline gefixt, damit cargo build ueberhaupt gruen werden kann.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-02T04:00:00Z
- **Completed:** 2026-05-02T04:12:41Z
- **Tasks:** 3 (alle aus PLAN.md)
- **Files modified:** 9 (5 neu fuer Phase 2 Wave 0, 3 neu fuer Phase-1-Fix, 1 mod.rs-Patch — plus 3 Patches in service/src/lib.rs, shifty-utils/Cargo.toml, shifty-utils/src/date_utils.rs)

## Accomplishments

- **Fixtures-Shared-Module** (`reporting_phase2_fixtures.rs`): 6 deterministische `pub fn fixture_*()` (sales_person_id/sales_person/work_details_8h_mon_fri/vacation_period 2024-06-03..05/sick_period 2024-06-04 mit BUrlG-§9-Overlap/extra_work +2h Do/report_range 2024-06-03..09).
- **Locking-Test-Scaffold** (`billing_period_snapshot_locking.rs`): zwei `#[test]` mit verbatim "LOCKING TEST -- DO NOT NAIVELY UPDATE"-Header inkl. CLAUDE.md-Verweis. Pin-Map-Test ist ROT (Wave-2-Forcing), Exhaustive-Match-Test ist GRUEN (kompiliert, listet alle 11 aktuellen Varianten + auskommentierten UnpaidLeave-Arm, der Wave 2 zur Aktivierung zwingt).
- **3 Stub-Tests** fuer REP-01/REP-02/REP-03 mit `#[ignore]` und Imports gegen die Phase-2-Fixtures -- Wave 1/2 muss nur die Mock-Setups + Assertions ergaenzen.
- **Phase-1-Pre-existing-Fix** (separater Commit `d8dad0aa`): `DateRange { from, to }` mit `new()/from()/to()/day_count()/iter_days()` zu `shifty-utils` hinzugefuegt + 6 Unit-Tests; `ValidationFailureItem::OverlappingPeriod(Uuid)`-Variante zu `service::ValidationFailureItem` hinzugefuegt.

## Task Commits

Jeder Task wurde atomar via `jj describe` + `jj new` committed:

1. **Phase-1-Fix (Rule-3-Auto-Fix vor Tasks):** `d8dad0aa` (fix(phase-1): add missing DateRange in shifty-utils and OverlappingPeriod variant)
2. **Task 0.1: Fixtures-Datei + mod.rs Registrierung** — `f85f4a3f` (test)
3. **Task 0.2: Locking-Test-Scaffold** — `0eeff84c` (test)
4. **Task 0.3: Stub-Tests fuer derive_hours_for_range, Flag-off und Flag-on** — `726e919c` (test)

**Plan-Metadaten-Commit (SUMMARY + STATE + ROADMAP):** wird nach diesem Schreibvorgang als jj-Commit angefuegt.

## Files Created/Modified

### Neu (Phase 2 Wave 0)
- `service_impl/src/test/reporting_phase2_fixtures.rs` (126 Zeilen) — 6 deterministische Fixture-Funktionen.
- `service_impl/src/test/billing_period_snapshot_locking.rs` (62 Zeilen) — Pin-Map + Exhaustive-Match Locking-Tests.
- `service_impl/src/test/absence_derive_hours_range.rs` (56 Zeilen) — 3 ignored Stubs fuer Wave-1-REP-01.
- `service_impl/src/test/reporting_flag_off_bit_identity.rs` (32 Zeilen) — 1 ignored Stub fuer Wave-2-REP-02.
- `service_impl/src/test/reporting_flag_on_integration.rs` (35 Zeilen) — 1 ignored Stub fuer Wave-2-REP-03.

### Geaendert (Phase 2 Wave 0)
- `service_impl/src/test/mod.rs` — 5 neue `pub mod`-Eintraege, alphabetisch in den vorhandenen Gruppen-Block einsortiert.

### Neu/Geaendert (Phase-1-Pre-existing-Fix, separater Commit)
- `shifty-utils/src/date_utils.rs` — `DateRange { from, to }` + `DateRangeIterator` + 6 Unit-Tests hinzugefuegt.
- `shifty-utils/Cargo.toml` — `time = { ..., features = ["macros"] }` als `[dev-dependencies]` aktiviert (fuer `time::macros::date!` in Tests).
- `service/src/lib.rs` — `ValidationFailureItem::OverlappingPeriod(Uuid)`-Variante mit Phase-1-D-13/D-15-Doc-Kommentar.

## Decisions Made

- **D-Wave0-A: Phase-1-Pre-existing-Build-Holes inline gefixt (Rule 3, blocking).** `dao/src/absence.rs`, `service_impl/src/absence.rs`, `service_impl/src/test/absence.rs` haben `shifty_utils::DateRange` und `ValidationFailureItem::OverlappingPeriod` referenziert, ohne dass diese Symbole im Workspace existieren. Ohne Fix war `cargo build -p service_impl --tests` ROT, was die gesamte Wave-0-Verifikation blockiert haette. Fix-Surface ist minimal: ein neuer Public-Type (`DateRange`), eine neue Enum-Variante (`OverlappingPeriod`), 6 unit-Tests. Kein Auswirkung auf bestehendes Verhalten — DateRange wird in Phase 1 in `find_overlapping`-Calls und `DateRange::new(from, to)` verwendet, alle Verwendungs-Stellen kompilieren jetzt unveraendert.
- **D-Wave0-B: Phase-1-Fix als separater Commit vor Wave-0-Tasks.** Klare Trennung von Phase-2-Wave-0-Scope (Tests-Scaffolding) vs. Phase-1-Pre-existing-Bugfix. Reviewer kann den `d8dad0aa`-Commit isoliert evaluieren.
- **D-Wave0-C: Stub-Bodies mit `#[ignore]` + `unimplemented!()`.** Beide Mechanismen kombiniert: `#[ignore]` blockiert Default-Test-Lauf, `unimplemented!()` zeigt klar, dass die Implementation noch fehlt. Wave 1/2 entfernt nur das `#[ignore]`-Attribut und ersetzt `unimplemented!()` durch das echte Setup.
- **D-Wave0-D: Pin-Map-Assertion in Wave 0 schon auf 3 gepinnt.** Statt erst die aktuelle Version 2 zu asserten und in Wave 2 dann auf 3 zu erhoehen, ist die Assertion sofort `expected = 3`. Pre-Wave-2 ist der Test ROT — das ist der intentional Forcing-Mechanismus aus dem PLAN. Ein einziger Test-Status-Wechsel statt zwei.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Phase-1-Pre-existing-Build-Holes (DateRange, ValidationFailureItem::OverlappingPeriod) inline gefixt**

- **Found during:** Task 0.1 — `cargo build -p service_impl --tests` schlug mit `unresolved import shifty_utils::DateRange` fehl, BEVOR ich irgendetwas Phase-2-spezifisches commited hatte.
- **Issue:** `dao/src/absence.rs:5`, `dao_impl_sqlite/src/absence.rs:9`, `service/src/absence.rs:19`, `service_impl/src/absence.rs:31` importieren `shifty_utils::DateRange`, aber der Typ existiert nirgendwo im Workspace. Zusaetzlich: `service_impl/src/absence.rs:168/241` und `service_impl/src/test/absence.rs:264/456` referenzieren `ValidationFailureItem::OverlappingPeriod(Uuid)`, ebenfalls nicht definiert. Phase-1-Plan-Phase hatte beide Symbole vorausgesetzt, aber die Phase-1-Executoren haben sie nicht hinzugefuegt — wahrscheinlich Folge des unvollstaendigen Worktree-Merges (siehe `pvwkysmk` mit reinem `.claude/worktrees/...`-Inhalt).
- **Fix:** 
  1. `shifty-utils/src/date_utils.rs`: `DateRange { from, to }` mit `new(from, to) -> Result<Self, DateRangeError>`-Konstruktor (lehnt invertierte Ranges ab), `from()`, `to()`, `day_count() -> u64` (inklusiv), `iter_days() -> DateRangeIterator` (yields `time::Date` von `from` bis `to`). 6 Unit-Tests fuer Konstruktor, `day_count`, `iter_days`.
  2. `shifty-utils/Cargo.toml`: `time = { ..., features = ["macros"] }` als `[dev-dependencies]` (war fuer `time::macros::date!` in Unit-Tests noetig; Production-Build bleibt unveraendert).
  3. `service/src/lib.rs`: `ValidationFailureItem::OverlappingPeriod(Uuid)` mit Doc-Kommentar (Phase-1-D-13/D-15).
- **Files modified:** `shifty-utils/src/date_utils.rs`, `shifty-utils/Cargo.toml`, `service/src/lib.rs`.
- **Verification:** `cargo build --workspace` exit 0; `cargo test -p shifty-utils` 11 passed (inkl. 6 neue DateRange-Tests).
- **Committed in:** `d8dad0aa` (separater Commit vor Phase-2-Wave-0-Tasks; klare Trennung)

---

**Total deviations:** 1 auto-fixed (1 Rule-3-Blocking)  
**Impact on plan:** Notwendig fuer cargo build. Kein Scope-Creep — die hinzugefuegten Symbole sind genau das, was Phase 1 vorausgesetzt hat. Phase-2-RESEARCH.md erwaehnt zusaetzlich `iter_days`/`day_count` fuer kommende `derive_hours_for_range`-Implementierung; beide sind bereits Teil dieses Fixes.

## Issues Encountered

- **`hx`-Editor-Crash bei `jj describe`/`jj split` ohne TTY.** Helix-Editor versucht `crossterm` zu initialisieren und panickt mit "reader source not set". Workaround: `JJ_EDITOR=true jj describe -m '...'` (no-op-Editor uebernimmt die `-m`-Description direkt). Anwendbar fuer alle weiteren `jj`-Befehle in headless/agent-Kontexten.
- **`jj split` braucht `--interactive` per Default.** Wenn man Filesets als positional args gibt, ist der Modus implizit non-interactive — aber jj startet trotzdem den Editor fuer die Description-Bearbeitung. `JJ_EDITOR=true` umgeht das.

### Out-of-Scope-Discovery (siehe deferred-items.md)

`cargo test --workspace --no-fail-fast` zeigt 8 fehlschlagende Phase-1-Integration-Tests
in `shifty_bin/src/integration_test/absence_period.rs` mit
`SqliteError "no such table: absence_period"`. Ursache: fehlende
`<TS>_create-absence-period.sql`-Migration. Pre-existing Phase-1-Luecke,
**ausserhalb des Plan-02-01-Scopes**. Doku in
`.planning/phases/02-reporting-integration-snapshot-versioning/deferred-items.md`.

Auswirkung auf Plan-02-01-Erfolg: KEINE — Wave-0-Tests sind alle
service_impl-Lib-Tests, kein DB-Zugriff.

## Self-Verification

Lokale Verifikation der Acceptance-Criteria:

- `test -f service_impl/src/test/reporting_phase2_fixtures.rs` → FOUND
- `test -f service_impl/src/test/billing_period_snapshot_locking.rs` → FOUND
- `test -f service_impl/src/test/absence_derive_hours_range.rs` → FOUND
- `test -f service_impl/src/test/reporting_flag_off_bit_identity.rs` → FOUND
- `test -f service_impl/src/test/reporting_flag_on_integration.rs` → FOUND
- `cargo build -p service_impl --tests` → exit 0
- `cargo build --workspace` → exit 0
- `cargo test -p shifty-utils` → 11 passed (6 neue DateRange-Tests + 5 alte)
- `cargo test -p service_impl test::billing_period_snapshot_locking::test_billing_period_value_type_surface_locked` → exit 0 (kompiliert + GRUEN)
- `cargo test -p service_impl test::billing_period_snapshot_locking::test_snapshot_schema_version_pinned` → exit != 0 (ROT — `left: 2, right: 3`. **Wave-2-Forcing intentional.**)
- `cargo test -p service_impl --no-fail-fast`: 307 passed, 1 failed (genau `test_snapshot_schema_version_pinned`), 5 ignored (alle Wave-1/2-Stubs).

## User Setup Required

Keine externe Konfiguration erforderlich. Die `time = { features = ["macros"] }`-Aktivierung in `shifty-utils` ist eine Cargo-Feature-Flag-Aenderung; `cargo build` zieht sie automatisch.

## Next Phase Readiness

**Wave 1 (Plan 02-02 oder 02-03):**
- Implementation von `AbsenceService::derive_hours_for_range` kann starten.
- Bestehende Stubs in `absence_derive_hours_range.rs` koennen `#[ignore]` entfernen und die Mock-Setups + Assertions ergaenzen. Die Fixtures sind bereits importiert.
- **Hinweis:** `DateRange::iter_days()` und `DateRange::day_count()` sind bereits in `shifty-utils` verfuegbar (aus dem Phase-1-Pre-existing-Fix dieses Plans).

**Wave 2 (Plan 02-04):**
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` 2 → 3 in `service_impl/src/billing_period_report.rs:37` bumpen.
- `BillingPeriodValueType::UnpaidLeave`-Variante in `service/src/billing_period.rs` hinzufuegen (+ `as_str` + `FromStr`).
- **Locking-Test-Match wird COMPILE-ERROR.** Antwort darauf: in `service_impl/src/test/billing_period_snapshot_locking.rs` den auskommentierten Arm aktivieren:
  ```
  // BillingPeriodValueType::UnpaidLeave => {}
  ```
  Comment-Marker entfernen, fertig.
- `test_snapshot_schema_version_pinned` wird automatisch GRUEN, sobald die Konstante 3 ist.
- Stubs in `reporting_flag_off_bit_identity.rs` und `reporting_flag_on_integration.rs` koennen `#[ignore]` entfernen und Bodies implementieren.

**Phase-1-Hygiene:** Mit dem Phase-1-Fix laeuft `cargo build --workspace` und `cargo test --workspace` jetzt sauber durch (modulo dem intentional-roten Pin-Test). Phase-1-Verification haette das ueber rate-Sampling fangen sollen — fuer kuenftige Phasen Empfehlung: `cargo build --workspace` als finaler Phase-Gate-Check, nicht nur per-Plan.

---

*Phase: 02-reporting-integration-snapshot-versioning*
*Plan: 01 (Wave 0 Test-Scaffolding)*
*Completed: 2026-05-02*

## Self-Check: PASSED

- service_impl/src/test/reporting_phase2_fixtures.rs → FOUND
- service_impl/src/test/billing_period_snapshot_locking.rs → FOUND
- service_impl/src/test/absence_derive_hours_range.rs → FOUND
- service_impl/src/test/reporting_flag_off_bit_identity.rs → FOUND
- service_impl/src/test/reporting_flag_on_integration.rs → FOUND
- shifty-utils/src/date_utils.rs (DateRange) → FOUND
- service/src/lib.rs (OverlappingPeriod variant) → FOUND
- jj log enthaelt commits d8dad0aa, f85f4a3f, 0eeff84c, 726e919c → FOUND (alle 4)
