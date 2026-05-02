---
phase: 02-reporting-integration-snapshot-versioning
plan: 02
subsystem: service
tags: [rust, service, absence, derive, conflict-resolution, di, phase-2-wave-1]

# Dependency graph
requires:
  - phase: 01-absence-domain-foundation
    provides: AbsenceService trait, AbsencePeriod domain, EmployeeWorkDetails service+fixtures, DateRange::iter_days
  - phase: 02-reporting-integration-snapshot-versioning
    plan: 01
    provides: reporting_phase2_fixtures, absence_derive_hours_range stub tests
provides:
  - AbsenceService::derive_hours_for_range trait method (REP-01) returning conflict-resolved BTreeMap<Date, ResolvedAbsence>
  - Cross-Category-Resolver SickLeave > Vacation > UnpaidLeave (BUrlG §9, D-Phase2-03) as single source of truth for Phase-4 migration gate
  - SpecialDayService + EmployeeWorkDetailsService as new AbsenceService deps (8 deps total in gen_service_impl block)
  - Public ResolvedAbsence struct (category + hours)
affects: [02-04-PLAN, phase-04 migration validation gate]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Per-day contract lookup via batch-fetch + iter_days filter (analog reporting::find_working_hours_for_calendar_week)
    - Cross-category priority resolver as inline fn (deterministic, re-runnable for Phase 4 validation)
    - SpecialDayService::get_by_week deduplicated batch fetch per ISO week

key-files:
  created:
    - .planning/phases/02-reporting-integration-snapshot-versioning/02-02-SUMMARY.md
  modified:
    - service/src/absence.rs
    - service_impl/src/absence.rs
    - service_impl/src/test/absence.rs
    - service_impl/src/test/absence_derive_hours_range.rs
    - shifty_bin/src/main.rs

key-decisions:
  - "A1 (RESEARCH open question) confirmed: AbsenceService bekommt SpecialDayService + EmployeeWorkDetailsService als neue Deps. Begruendung: Single source of truth fuer Cross-Category-Resolver; Caller (ReportingService) muss nichts ueber Per-Tag-Vertrags-Lookup oder Holiday-Detection wissen."
  - "Holiday-Tage produzieren KEINEN Map-Eintrag (kein ResolvedAbsence), nicht Eintrag mit hours=0. Caller bridged via .unwrap_or(0.0) — semantisch klar 'diese Absence verbraucht 0 Stunden an diesem Tag'."
  - "Range-Validation laeuft VOR dem Permission-Check, damit DateOrderWrong stabil bei invertierten Ranges zurueckkommt unabhaengig vom Auth-Status (analog Phase-1 D-14 Pattern)."
  - "main.rs constructor order angepasst: working_hours_service let-binding vor absence_service verschoben, weil AbsenceServiceImpl jetzt EmployeeWorkDetailsService als Dep haelt."
  - "DAO-Mock liefert AbsencePeriodEntity, nicht Domain AbsencePeriod — Service-Body iteriert direkt ueber Entities und konvertiert dominante Kategorie via From-Impl. Vermeidet doppelte Conversion und matcht Phase-1-Pattern."

patterns-established:
  - "Phase-2 Wave-1: tdd cycle in einem Plan — Task 1.1 macht service-Crate gruen + service_impl rot (E0046 RED), Task 1.2 macht service_impl gruen + Tests gruen (GREEN), Task 1.3 macht main.rs DI gruen (Workspace GREEN)."
  - "Per-day resolver pattern: outer loop iter_days, inner filter+max_by_key fuer dominante Kategorie."

requirements-completed: [REP-01]

# Metrics
duration: 11min
completed: 2026-05-02
---

# Phase 02 Plan 02: AbsenceService::derive_hours_for_range Summary

**Conflict-resolved BUrlG §9-konformer Tages-Iterator als Single source of truth fuer Phase-4-Migration-Gate — `AbsenceService` erhaelt `SpecialDayService` und `EmployeeWorkDetailsService` als neue Deps (8 Deps total) und liefert pro Tag eine bereits aufgeloeste `ResolvedAbsence` mit Prioritaet `SickLeave > Vacation > UnpaidLeave`.**

## Performance

- **Duration:** ~11 min
- **Started:** 2026-05-02T04:22:02Z
- **Completed:** 2026-05-02T04:33:20Z
- **Tasks:** 3 (alle aus PLAN.md, alle gruen)
- **Files modified:** 5 (1 service, 1 service_impl, 2 test, 1 main.rs)

## Accomplishments

### Trait + Domain (Task 1.1)
- **`service/src/absence.rs`:** `pub struct ResolvedAbsence { category: AbsenceCategory, hours: f32 }` (Clone, Debug, PartialEq).
- **`service/src/absence.rs`:** Trait-Methode `derive_hours_for_range(from, to, sales_person_id, ctx, tx) -> Result<BTreeMap<Date, ResolvedAbsence>, ServiceError>` im `#[automock]`-Block.
- **`service_impl/src/test/absence_derive_hours_range.rs`:** `#[ignore]`-Attribute aller 3 Stubs entfernt — Tests sind RED via `unimplemented!()` bis Task 1.2.

### Implementation + Deps + Tests gruen (Task 1.2)
- **`service_impl/src/absence.rs`:** `gen_service_impl!`-Block um 2 Deps erweitert: `SpecialDayService` + `EmployeeWorkDetailsService`. Module-level `fn absence_category_priority` als Single source of truth fuer D-Phase2-03.
- **`service_impl/src/absence.rs`:** Neue `derive_hours_for_range`-Implementation:
  1. Range-Validation (`DateRange::new` → `DateOrderWrong`)
  2. Permission HR ∨ self via `tokio::join!` + `or` (analog `find_by_sales_person`)
  3. Batch-Fetch: `absence_dao.find_by_sales_person` + `employee_work_details_service.find_by_sales_person_id`
  4. Wochenset (`BTreeSet<(year, week)>`) aus Range; pro Woche EIN `special_day_service.get_by_week`-Call (deduplizierter batch)
  5. Holiday-Set (`BTreeSet<Date>`) aus `SpecialDay::day_type == Holiday`, konvertiert via `time::Date::from_iso_week_date`
  6. Per-Tag-Iteration: aktiven Vertrag ueber `from_date()/to_date()` finden, `has_day_of_week`-Filter, Holiday-Skip, Vertragsstunden-Lookup, dominante Kategorie via `max_by_key(absence_category_priority)`
- **`service_impl/src/test/absence.rs`:** `AbsenceDependencies` um `MockSpecialDayService` + `MockEmployeeWorkDetailsService` erweitert (8 Felder, Visibility auf `pub(crate)` damit `absence_derive_hours_range`-Tests `build_dependencies()` aufrufen koennen).
- **3 Tests gruen:**
  - `test_derive_hours_for_range_basic`: Mo+Mi Vacation 8h, Di SickLeave (BUrlG §9 vor Vacation), Do/Fr/Sa/So leer (3 Eintraege)
  - `test_derive_hours_holiday_is_zero`: Di als Holiday → KEIN Eintrag fuer Di trotz Sick+Vacation; Mo+Mi unveraendert (2 Eintraege)
  - `test_derive_hours_contract_change`: 8h-Vertrag KW22-23, 4h-Vertrag KW24-25 → 06-03..05 = 8h, 06-10..14 = 4h (8 Eintraege)

### DI-Wiring (Task 1.3)
- **`shifty_bin/src/main.rs`:** `AbsenceServiceDependencies`-Block um 2 type-Aliasse erweitert: `type SpecialDayService = SpecialDayService;` und `type EmployeeWorkDetailsService = WorkingHoursService;`.
- **`shifty_bin/src/main.rs`:** `let working_hours_service = ...`-Block VOR `let absence_service = ...` verschoben (Compiler-Reihenfolge); `AbsenceServiceImpl`-Constructor um `special_day_service.clone()` + `employee_work_details_service: working_hours_service.clone()` erweitert.

## Task Commits

Alle Tasks atomar via `jj describe` + `jj new`:

1. **Task 1.1: Trait surface (RED)** — `8fafb6ef` (`feat(02-02): trait surface for AbsenceService::derive_hours_for_range`)
2. **Task 1.2: Implementation + Tests gruen (GREEN)** — `3e371b06` (`feat(02-02): implement AbsenceService::derive_hours_for_range`)
3. **Task 1.3: DI-Wiring (Workspace GREEN)** — `ae7d0642` (`feat(02-02): wire SpecialDayService and EmployeeWorkDetailsService into AbsenceService DI`)

**Plan-Metadaten-Commit (SUMMARY + STATE + ROADMAP):** wird nach diesem Schreibvorgang als jj-Commit angefuegt.

## Files Created/Modified

### Geaendert
- `service/src/absence.rs` — `BTreeMap`-Import, `ResolvedAbsence`-Struct, neue Trait-Methode `derive_hours_for_range`.
- `service_impl/src/absence.rs` — Imports erweitert (`AbsenceCategory`, `ResolvedAbsence`, `EmployeeWorkDetailsService`, `SpecialDayService`, `SpecialDayType`, `BTreeSet`, `Date`); `gen_service_impl!` von 6 auf 8 Deps; module-level `absence_category_priority`-Helper; ~110 Zeilen neuer Methoden-Body.
- `service_impl/src/test/absence.rs` — `AbsenceDependencies` von 6 auf 8 Felder; Visibility auf `pub(crate)` fuer `build_service` und `build_dependencies`.
- `service_impl/src/test/absence_derive_hours_range.rs` — komplette Neu-Implementierung der 3 Tests (von Stubs auf voll-funktionsfaehig); ~280 Zeilen.
- `shifty_bin/src/main.rs` — 2 neue type-Aliasse, 2 neue Constructor-Felder, `working_hours_service`-Binding nach oben verschoben.

## Decisions Made

- **D-02-02-A: Holiday-Tage produzieren KEINEN Map-Eintrag.** Im PLAN als "Behavior-Decision" angedeutet, hier final implementiert. Der Caller in Phase-4-Migration-Gate kann via `.get(&day).map(|r| r.hours).unwrap_or(0.0)` einheitlich abfragen — semantisch klar "diese Absence verbraucht 0 Stunden an diesem Tag". Vorteil: keine speziellen `hours == 0`-Marker noetig, der Map-Eintrag kodiert "Absence aktiv" via Existenz.
- **D-02-02-B: A1-RESEARCH-Frage final mit "AbsenceService bekommt 2 neue Deps" beantwortet.** Alternative aus RESEARCH (Holiday-Erkennung via ExtraHours-Lookup) waere semantisch unsauber gewesen, weil Phase-2-Goal `derive_hours_for_range` als single source of truth fuer Phase-4 etabliert — unabhaengig von ExtraHours-Daten, sodass die Migration-Validation nur auf `(absence_period, employee_work_details, special_day)` lesen muss.
- **D-02-02-C: Range-Validation VOR Permission-Check.** Falls jemand mit gueltiger Auth einen invertierten Range schickt, soll er stabil `DateOrderWrong` bekommen — kein Auth-Roundtrip noetig. Spiegelt Phase-1-D-14 Pattern wider.
- **D-02-02-D: DAO-Mock liefert `AbsencePeriodEntity`, Service iteriert ueber Entities.** Kein Domain-Conversion-Roundtrip im Body — wir konvertieren erst beim `ResolvedAbsence`-Insert via `(&dominant.category).into()`. Konsistent mit Phase-1-Pattern.
- **D-02-02-E: `working_hours_service`-let-Verschiebung in main.rs.** Die alternative Loesung (`absence_service` selbst nach unten verschieben) waere risikoaerm gewesen, aber der Phase-1-`AbsenceServiceImpl`-Block hat klar den festen Platz "nach den UUID-/Clock-Services". Phase-2 bricht diesen Constructor-Order minimal auf und dokumentiert es per Inline-Kommentar.

## Deviations from Plan

**Total deviations:** 0

Der Plan wurde 1:1 wie geschrieben ausgefuehrt. Keine Rule-1/2/3-Auto-Fixes erforderlich. Die im PLAN als A1-Open-Question markierte Architektur-Entscheidung (SpecialDayService als Dep) war bereits per CONTEXT.md final, sodass kein Rule-4-Checkpoint noetig war.

### Anmerkungen

- **`absence_category_priority` als module-level fn statt nested.** PLAN-Sketch hat sie inline in `derive_hours_for_range` gezeigt. Ich habe sie auf module-level extrahiert, damit sie ein eindeutiger Symbol-Anker fuer Phase-4-Migration-Validation wird (gleicher Helper kann von dort wiederverwendet werden ohne Duplikation).

## Issues Encountered

- **Compiler-`unused_imports`-Warning bei `AbsenceDependencies` in `absence_derive_hours_range.rs`.** Der PLAN-Sketch enthielt `use crate::test::absence::{build_dependencies, AbsenceDependencies};`, aber `AbsenceDependencies` wird im Test-Body nicht direkt referenziert (nur ueber `build_dependencies()`-Return-Type). Behoben: Import auf `build_dependencies` reduziert.

- **`SpecialDayService::get_by_week` hat KEINEN `tx`-Parameter.** Das Trait ist `#[automock(type Context=();)]`, NICHT `(type Transaction=...)`. Beim Mock-Setup nur `(year, week, ctx)`-Closure-Args, kein `tx`. Im Service-Body kein `tx.clone().into()`-Argument. Beim Schreiben der Mock-Setups initial nicht beachtet — schnell korrigiert via Trait-Inspektion.

### Out-of-Scope-Discoveries

- **8 fehlschlagende `shifty_bin::integration_test::absence_period`-Tests** mit `SqliteError "no such table: absence_period"`. Bereits in `02-01-SUMMARY` und `deferred-items.md` dokumentiert (fehlende `<TS>_create-absence-period.sql`-Migration aus Phase 1). KEINE Regression durch Plan 02-02 — beide Failing-Sets identisch zu Pre-02-02-State.
- **`test_snapshot_schema_version_pinned` ROT** — intentionales Wave-2-Forcing aus Plan 02-01, Plan 02-04 macht ihn GREEN.

Auswirkung auf Plan-02-02-Erfolg: KEINE — alle 3 PLAN-Tasks sind gruen, Workspace-Build gruen, alle 28 absence-Unit-Tests gruen.

## Self-Verification

Lokale Verifikation der PLAN-Acceptance-Criteria:

### Task 1.1
- `grep -c "pub struct ResolvedAbsence" service/src/absence.rs` → 1 ✓
- `grep -c "pub category: AbsenceCategory" service/src/absence.rs` → 1 ✓
- `grep -c "pub hours: f32" service/src/absence.rs` → 1 ✓
- `grep -c "fn derive_hours_for_range" service/src/absence.rs` → 1 ✓
- `grep -c "BTreeMap<Date, ResolvedAbsence>" service/src/absence.rs` → 1 ✓
- `cargo build -p service` → exit 0 ✓
- `cargo build -p service_impl` (Task 1.1-State) → E0046 ROT (intentional) ✓
- `#[ignore]`-Attribute in Stubs → 0 ✓

### Task 1.2
- `cargo build -p service_impl` → exit 0 ✓
- `grep -c "SpecialDayService" service_impl/src/absence.rs` → 4 ✓
- `grep -c "EmployeeWorkDetailsService" service_impl/src/absence.rs` → 2 ✓
- `grep -c "fn derive_hours_for_range" service_impl/src/absence.rs` → 1 ✓
- `grep -c "AbsenceCategory::SickLeave => 3" service_impl/src/absence.rs` → 1 ✓
- `grep -c "AbsenceCategory::Vacation => 2" service_impl/src/absence.rs` → 1 ✓
- `grep -c "AbsenceCategory::UnpaidLeave => 1" service_impl/src/absence.rs` → 1 ✓
- `grep -c "MockSpecialDayService" service_impl/src/test/absence.rs` → 4 ✓
- `grep -c "MockEmployeeWorkDetailsService" service_impl/src/test/absence.rs` → 4 ✓
- `cargo test -p service_impl test::absence_derive_hours_range` → 3/3 GRUEN ✓
- `cargo test -p service_impl test::absence` → 28/28 GRUEN ✓

### Task 1.3
- `grep -cE "type SpecialDayService\s*=" shifty_bin/src/main.rs` → 4 (1 neu + 3 bestehend) ✓
- `grep -cE "type EmployeeWorkDetailsService\s*=\s*WorkingHoursService" shifty_bin/src/main.rs` → 5 ✓
- `grep -c "special_day_service: special_day_service" shifty_bin/src/main.rs` → 3 ✓
- `grep -c "employee_work_details_service: working_hours_service" shifty_bin/src/main.rs` → 5 ✓
- `cargo build --workspace` → exit 0 ✓
- `cargo test --workspace`: 313 passed in service_impl-lib (modulo erwarteter ROT-Pin-Test); 8 absence_period-Integration-Fails sind out-of-scope (pre-existing Phase-1, in deferred-items.md) ✓

## User Setup Required

Keine externe Konfiguration erforderlich. Die Phase-2-Wave-1-Aenderungen sind vollstaendig in Rust und werden von `cargo build`/`cargo test` automatisch eingelesen.

## Next Phase Readiness

**Wave 2 (Plan 02-04 — Reporting-Switch + UnpaidLeave-Snapshot + Version-Bump + Locking-Tests, atomarer Single-Commit):**
- `AbsenceService::derive_hours_for_range` ist bereit zum Aufruf vom `ReportingService` aus.
- Pattern fuer `ReportingService::get_report_for_employee_range`: nach Permission-Check einmaliger `feature_flag_service.is_enabled("absence_range_source_active", Authentication::Full, tx.clone().into())`-Call; wenn `true` → `absence_service.derive_hours_for_range(from, to, sales_person_id, ctx, Some(tx))` als Quelle fuer `vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours`.
- `ReportingServiceImpl` braucht 2 neue Deps: `FeatureFlagService` und `AbsenceService` (analoges Pattern wie hier in Wave 1).
- Pin-Test `test_snapshot_schema_version_pinned` wird GRUEN sobald `CURRENT_SNAPSHOT_SCHEMA_VERSION` von 2 auf 3 erhoeht.
- Compiler-Exhaustive-Match-Test wird COMPILE-ERROR sobald `BillingPeriodValueType::UnpaidLeave` hinzugefuegt wird; auskommentierter Arm in `billing_period_snapshot_locking.rs` wartet zur Aktivierung.

**Wave 1 ist abgeschlossen.** Plan 02-03 (Feature-Flag-Infra) kann parallel zu/vor Plan 02-04 laufen.

---

*Phase: 02-reporting-integration-snapshot-versioning*
*Plan: 02 (Wave 1 — derive_hours_for_range)*
*Completed: 2026-05-02*

## Self-Check: PASSED

- service/src/absence.rs → FOUND
- service_impl/src/absence.rs → FOUND
- service_impl/src/test/absence.rs → FOUND
- service_impl/src/test/absence_derive_hours_range.rs → FOUND
- shifty_bin/src/main.rs → FOUND
- .planning/phases/02-reporting-integration-snapshot-versioning/02-02-SUMMARY.md → FOUND
- jj log enthaelt commits 8fafb6ef, 3e371b06, ae7d0642 → FOUND (alle 3)
