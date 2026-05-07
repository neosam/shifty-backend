---
phase: 02-reporting-integration-snapshot-versioning
verified: 2026-05-02T11:00:00Z
status: passed
score: 6/6 must-haves verified
overrides_applied: 0
re_verification: null
---

# Phase 2: Reporting Integration & Snapshot Versioning — Verification Report

**Phase Goal:** Reporting kann Absence-derived Stunden zusätzlich zu ExtraHours summieren (Feature-Flag-gesteuert), per-Tag gegen den am jeweiligen Tag gültigen Vertrag berechnet, mit korrekter Feiertags-Orthogonalität. `CURRENT_SNAPSHOT_SCHEMA_VERSION` ist auf 3 gebumpt — im **selben Commit** wie der Reporting-Switch.

**Verified:** 2026-05-02T11:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `AbsenceService::derive_hours_for_range(from, to, sales_person_id)` liefert pro Tag im Range die Vertragsstunden des am Tag gültigen `EmployeeWorkDetails`-Vertrages; Feiertage liefern 0 Urlaubsstunden; Vertragswechsel produziert pro Tag den jeweils gültigen Wert. | VERIFIED | Trait in `service/src/absence.rs:180`; Impl in `service_impl/src/absence.rs:331-465` mit Per-Tag-Vertragsauswahl (`active_contract` via `from_date()/to_date()`), Holiday-Skip (`if holidays.contains(&day) { continue; }`), Cross-Category-Resolver SickLeave>Vacation>UnpaidLeave. 3 Tests grün: `test_derive_hours_for_range_basic`, `test_derive_hours_holiday_is_zero`, `test_derive_hours_contract_change`. |
| 2 | Solange `absence.range_source_active` aus ist, liefern Reports/Bilanzen/Snapshots **bit-identische** Werte wie vor Phase 2. | VERIFIED | `service_impl/src/test/reporting_flag_off_bit_identity.rs::test_flag_off_produces_identical_values_to_pre_phase2` grün — verifiziert dass mit Flag=off `vacation_hours==8.0, sick_leave_hours==4.0, unpaid_leave_hours==2.0` (aus ExtraHours, identisch pre-Phase-2) und `MockAbsenceService::expect_derive_hours_for_range().times(0)` (DARF NICHT aufgerufen werden). Reporting-Switch in `service_impl/src/reporting.rs:489-505,642-674` ist if-else-Branch — Flag=off-Pfad ist 1:1 pre-Phase-2-Code. |
| 3 | Ist der Flag an, wechselt der Reporting-Pfad atomar zur neuen Quelle für Vacation/Sick/UnpaidLeave. | VERIFIED | `service_impl/src/test/reporting_flag_on_integration.rs::test_flag_on_uses_absence_source` grün — verifiziert dass mit Flag=on und ExtraHours mit 999h-Markern (Vacation/Sick/UnpaidLeave) plus Absence-Map (Mo Vacation 8h, Di SickLeave 8h, Mi Vacation 8h): `vacation_hours==16.0, sick_leave_hours==8.0, unpaid_leave_hours==0.0, extra_work_hours==2.0`. ExtraHours-999h werden ignoriert (Filter Schritt 1, `service_impl/src/reporting.rs:489-505`); Override Schritt 3 (`service_impl/src/reporting.rs:642-674`). |
| 4 | `CURRENT_SNAPSHOT_SCHEMA_VERSION = 3`; Build-Time-Locking-Test schlägt fehl bei Drift. | VERIFIED | `service_impl/src/billing_period_report.rs:37` enthält `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 3;`. Locking-Tests grün: `test_snapshot_schema_version_pinned` (Pin-Map auf 3) und `test_billing_period_value_type_surface_locked` (Compiler-Exhaustive-Match listet alle 12 Varianten inkl. UnpaidLeave). Pin-Map-Test `test_snapshot_v3_pinned_values` deckt alle 12 BillingPeriodValueType-Varianten ab (Balance, ExpectedHours, ExtraWork, Holiday, Overall, SickLeave, UnpaidLeave, VacationDays, VacationEntitlement, VacationHours, Volunteer + CustomExtraHours-Surface-Check via `values.keys().any(|k| matches!(k, BillingPeriodValueType::CustomExtraHours(_)))`). |
| 5 | Bestehende Snapshots der Version 2 bleiben lesbar; neue Snapshots tragen Version 3; Validatoren erkennen den Unterschied. | VERIFIED | `BillingPeriodSalesPerson` (`service/src/billing_period.rs:107-118`) und `BillingPeriod` (`service/src/billing_period.rs:23`) tragen `snapshot_schema_version: u32`. `BillingPeriodValueType::FromStr` (`service/src/billing_period.rs:72-97`) ignoriert ungültige Strings via `Err`, neue `unpaid_leave`-Variante wird unterstützt; v2-Snapshots ohne `unpaid_leave`-Zeile produzieren `None` aus der `values: BTreeMap`-Lookup. Test `test_from_entities_missing_unpaid_leave_is_zero` grün — verifiziert v2-Snapshot ohne `unpaid_leave`-Zeile bleibt lesbar. |
| 6 | Atomarer Single-Commit (CLAUDE.md D-Phase2-10): Snapshot-Bump 2→3 + UnpaidLeave-Variante + Snapshot-Insert + Reporting-Switch in **EINEM** jj-Commit. | VERIFIED | `jj show 39be1b73 --stat` zeigt EINEN Commit `39be1b73a10a07c6cf0c66224a005ddc9e7eb2ea` mit 11 Dateien (927 insertions, 92 deletions): `service/src/billing_period.rs` (UnpaidLeave-Variante), `service_impl/src/billing_period_report.rs` (Bump 2→3 + UnpaidLeave-Insert), `service_impl/src/reporting.rs` (Switch), `service_impl/src/feature_flag.rs` (Auth-Full-Bypass), 6 Test-Dateien, `shifty_bin/src/main.rs` (DI). Commit-Description: "feat(02-04): atomic Wave-2 -- snapshot bump v2->v3 + UnpaidLeave variant + Reporting-Switch + locking-tests". |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `service/src/absence.rs` | Trait method `derive_hours_for_range` + `ResolvedAbsence` struct | VERIFIED | Lines 113-119 (`ResolvedAbsence { category, hours }`), 180-188 (Trait-Methode, async, returns `BTreeMap<Date, ResolvedAbsence>`). |
| `service_impl/src/absence.rs` | Impl `derive_hours_for_range` mit Cross-Category-Resolver, Holiday-Skip, Per-Tag-Vertragsauswahl | VERIFIED | Lines 331-465: vollständige Impl mit Range-Validation, Permission-Check (HR ∨ self), Batch-Fetch (Absences + WorkDetails), Wochen-Set für SpecialDayService, Holiday-Set, Per-Tag-Loop mit `find_active_contract`, `has_day_of_week`, `holidays.contains`, dominant via `max_by_key(absence_category_priority)`. |
| `service_impl/src/billing_period_report.rs` | `CURRENT_SNAPSHOT_SCHEMA_VERSION = 3` + UnpaidLeave-Insert | VERIFIED | Line 37: `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 3;`. Lines 167-174: UnpaidLeave-Insert mit `report_delta.unpaid_leave_hours`/`report_start.unpaid_leave_hours`/`report_end.unpaid_leave_hours`/`report_end_of_year.unpaid_leave_hours`. Line 301: `snapshot_schema_version: CURRENT_SNAPSHOT_SCHEMA_VERSION` in Persist-Pfad. |
| `service/src/billing_period.rs` | `BillingPeriodValueType::UnpaidLeave` (12. Variante) + as_str + FromStr | VERIFIED | Line 45: `UnpaidLeave,`-Variante. Line 61: `BillingPeriodValueType::UnpaidLeave => "unpaid_leave".into(),`. Line 84: `"unpaid_leave" => Ok(BillingPeriodValueType::UnpaidLeave),`. Round-Trip `as_str` → `from_str` ist Identität. |
| `service_impl/src/reporting.rs` | Reporting-Switch: einmaliger Flag-Read, Filter, Override | VERIFIED | Lines 70-72 (gen_service_impl mit FeatureFlagService + AbsenceService Deps). Lines 475-482: `is_enabled("absence_range_source_active", Authentication::Full, tx.clone())` einmaliger Read. Lines 489-505: Filter-Branch (Schritt 1) — entfernt Vacation/SickLeave/UnpaidLeave bei Flag=on. Lines 513-541: Override-Setup (Schritt 3) — `derive_hours_for_range` Aufruf bei Flag=on. Lines 642-674: EmployeeReport-Felder if-else (vacation_hours/sick_leave_hours/unpaid_leave_hours). |
| `service_impl/src/feature_flag.rs` | FeatureFlagService Trait + Impl mit Authentication::Full-Bypass | VERIFIED | Lines 25-47: `is_enabled` mit `if let Authentication::Context(_) = &context { current_user_id check }` — Authentication::Full bypasst User-Check. Lines 49-67: `set` mit admin-only `check_permission(FEATURE_FLAG_ADMIN_PRIVILEGE, ...)`. |
| `service_impl/src/test/billing_period_snapshot_locking.rs` | Compiler-Match alle 12 Varianten + Pin-Map auf 3 | VERIFIED | 2 Tests grün. Match-Block listet alle 12 Varianten (Overall, Balance, ExpectedHours, ExtraWork, VacationHours, SickLeave, UnpaidLeave, Holiday, Volunteer, VacationDays, VacationEntitlement, CustomExtraHours(_)). |
| `service_impl/src/test/billing_period_report.rs` | `test_snapshot_v3_pinned_values` Pin-Map alle 12 Varianten | VERIFIED | Test grün. Deckt 11 explizite Varianten (Balance/ExpectedHours/ExtraWork/Holiday/Overall/SickLeave/UnpaidLeave/VacationDays/VacationEntitlement/VacationHours/Volunteer) und 12. Variante CustomExtraHours via Surface-Check (`values.keys().any(matches!CustomExtraHours)`) plus `assert_eq!(values.len(), 11)`. |
| `service_impl/src/test/billing_period.rs` | `test_from_entities_missing_unpaid_leave_is_zero` (v2-Lesbarkeit) | VERIFIED | Test grün. Verifiziert dass v2-Snapshot ohne `unpaid_leave`-Zeile ein lesbares `BillingPeriodSalesPerson` produziert mit `values.get(&UnpaidLeave) == None`. |
| `service_impl/src/test/reporting_flag_off_bit_identity.rs` | Bit-Identitäts-Test bei Flag=off | VERIFIED | Test `test_flag_off_produces_identical_values_to_pre_phase2` grün. `expect_is_enabled => Ok(false)`, `expect_derive_hours_for_range.times(0)`. |
| `service_impl/src/test/reporting_flag_on_integration.rs` | Switch-Test bei Flag=on | VERIFIED | Test `test_flag_on_uses_absence_source` grün. `expect_is_enabled => Ok(true)` + `expect_derive_hours_for_range` returning fixture-Map. |
| `service_impl/src/test/absence_derive_hours_range.rs` | 3 Tests (basic, holiday, contract-change) | VERIFIED | Alle 3 grün. |
| `shifty_bin/src/main.rs` | DI-Wiring FeatureFlagService + AbsenceService | VERIFIED | Lines 314-316 (ReportingServiceDependencies type-Aliasse). Lines 778-790 (ReportingServiceImpl-Konstruktor mit `feature_flag_service: feature_flag_service.clone(), absence_service: absence_service.clone()`). Lines 772-777 (FeatureFlagServiceImpl-Konstruktion VOR reporting_service). |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `service_impl/src/reporting.rs::get_report_for_employee_range` | `FeatureFlagService::is_enabled` | `self.feature_flag_service.is_enabled("absence_range_source_active", Authentication::Full, tx.clone())` | WIRED | Line 475-482. |
| `service_impl/src/reporting.rs::get_report_for_employee_range` | `AbsenceService::derive_hours_for_range` | `self.absence_service.derive_hours_for_range(from_date.to_date(), to_date.to_date(), *sales_person_id, context.clone(), tx.clone())` | WIRED | Lines 518-527. |
| `service_impl/src/billing_period_report.rs` | `EmployeeReport::unpaid_leave_hours` | `report_delta.unpaid_leave_hours` etc. in UnpaidLeave-Insert | WIRED | Lines 168-173. |
| `service_impl/src/test/billing_period_snapshot_locking.rs` | `BillingPeriodValueType::UnpaidLeave` | Match-Arm in exhaustive match | WIRED | Compiler-Locking aktiv — Test compiliert nur mit allen 12 Varianten. |
| `shifty_bin/src/main.rs::ReportingServiceImpl-Konstruktor` | `feature_flag_service + absence_service` | DI-Konstruktor mit `.clone()` | WIRED | Lines 787-788. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `EmployeeReport.vacation_hours/sick_leave_hours/unpaid_leave_hours` (Flag=on) | `absence_derived_*` aus `derive_hours_for_range` | `AbsenceService::derive_hours_for_range` mit echtem DAO + WorkDetailsService + SpecialDayService — Tests bewiesen liefert Per-Tag-Stunden | FLOWING | `test_flag_on_uses_absence_source` zeigt `vacation_hours=16.0` (Mo+Mi via AbsencePeriod), `sick_leave_hours=8.0` (Di), `unpaid_leave_hours=0.0`. |
| `EmployeeReport.vacation_hours/sick_leave_hours/unpaid_leave_hours` (Flag=off) | `extra_hours.iter().filter(category).map(amount).sum()` | `ExtraHoursService::find_by_sales_person_id_and_year_range` | FLOWING | `test_flag_off_produces_identical_values_to_pre_phase2` zeigt `vacation_hours=8.0/4.0/2.0` (aus ExtraHours-Quelle). |
| `BillingPeriodValue::value_delta` für UnpaidLeave | `report_delta.unpaid_leave_hours` | EmployeeReport (Flag-gated source) | FLOWING | `test_snapshot_v3_pinned_values` zeigt UnpaidLeave-Eintrag in values-Map mit korrektem Wert. |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Workspace builds clean | `cargo build --workspace` | Finished `dev` profile (0 errors) | PASS |
| All service_impl lib tests pass | `cargo test -p service_impl --lib` | 321 passed; 0 failed; 0 ignored | PASS |
| Phase-2 derive_hours tests pass | `cargo test -p service_impl --lib test::absence_derive_hours_range` | 3/3 passed | PASS |
| Bit-Identity test passes | `cargo test -p service_impl --lib test::reporting_flag_off_bit_identity` | 1/1 passed | PASS |
| Flag-on Switch test passes | `cargo test -p service_impl --lib test::reporting_flag_on_integration` | 1/1 passed | PASS |
| Locking tests pass | `cargo test -p service_impl --lib test::billing_period_snapshot_locking` | 2/2 passed | PASS |
| Pin-map test passes | `cargo test -p service_impl --lib test_snapshot_v3_pinned_values` | 1/1 passed | PASS |
| v2 readability test passes | `cargo test -p service_impl --lib test_from_entities_missing_unpaid_leave_is_zero` | 1/1 passed | PASS |
| Atomic commit verification | `jj show 39be1b73 --stat` | 1 commit, 11 files changed | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description (per ROADMAP) | Status | Evidence |
|-------------|-------------|---------------------------|--------|----------|
| REP-01 | 02-02-PLAN | derive_hours_for_range mit Per-Tag-Vertrag, Holiday=0, Vertragswechsel | SATISFIED | SC-1 verifiziert. 3 Tests grün. |
| REP-02 | 02-01-PLAN, 02-04-PLAN | Bit-Identität Reports/Bilanzen/Snapshots bei Flag=off | SATISFIED | SC-2 verifiziert. `test_flag_off_produces_identical_values_to_pre_phase2` grün. |
| REP-03 | 02-03-PLAN, 02-04-PLAN | Switch zur neuen Quelle Vacation/Sick/UnpaidLeave bei Flag=on | SATISFIED | SC-3 verifiziert. `test_flag_on_uses_absence_source` grün. FeatureFlagService implementiert. |
| REP-04 | 02-03-PLAN, 02-04-PLAN | Flag-Infrastruktur (Service + DAO + Migration + Privileg) | SATISFIED | FeatureFlagService Trait+Impl in `service{,_impl}/src/feature_flag.rs`; DAO in `dao{,_impl_sqlite}/src/feature_flag.rs`; Migration `migrations/sqlite/20260501000000_add-feature-flag-table.sql`. 6 Service-Tests grün (inkl. Authentication::Full-Bypass-Test). |
| SNAP-01 | 02-01-PLAN, 02-04-PLAN | Snapshot-Schema-Version 3 + atomarer Bump im selben Commit wie Reporting-Switch | SATISFIED | SC-4 + SC-6 verifiziert. CURRENT_SNAPSHOT_SCHEMA_VERSION=3. Atomic-Commit `39be1b73`. |
| SNAP-02 | 02-01-PLAN, 02-04-PLAN | Locking-Test: Build-Time-Gate gegen Drift | SATISFIED | SC-4 verifiziert. `test_billing_period_value_type_surface_locked` (Compiler-Match) + `test_snapshot_schema_version_pinned` (Pin-Map) + `test_snapshot_v3_pinned_values` (alle 12 Varianten gepinnt). |

Alle 6 Phase-2-Requirements (REP-01..04, SNAP-01..02) sind durch Code+Tests SATISFIED. Keine ORPHANED Requirements.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (keine) | — | — | — | Code ist sauber. Keine TODO/FIXME/STUB-Pattern in den Phase-2-modifizierten Dateien. Keine `unimplemented!()`/`return Err(NotImplemented)` Pattern in den neuen Code-Dateien. Filter+Override-Pattern dokumentiert per Inline-Kommentaren mit Verweis auf D-Phase2-08-A, T-02-04-03, T-02-04-04, Pitfall 5. |

### Human Verification Required

(keine — alle Success Criteria sind programmatisch verifiziert via Tests)

### Gaps Summary

Keine Gaps. Phase-2-Goal ist vollständig erreicht.

**Atomarer Single-Commit** (CLAUDE.md-Pflicht, ROADMAP.md Goal-Statement: "im **selben Commit**") ist via `jj show 39be1b73 --stat` einwandfrei verifiziert: 1 Commit, 11 Dateien — Snapshot-Bump 2→3, UnpaidLeave-Variante, Snapshot-Builder-Insert, Reporting-Switch und Locking-Tests sind alle in `39be1b73` gemeinsam committed.

**Locking-Test-Surface** deckt alle 12 BillingPeriodValueType-Varianten:
- `test_billing_period_value_type_surface_locked`: Compiler-Exhaustive-Match listet alle 12 (Overall, Balance, ExpectedHours, ExtraWork, VacationHours, SickLeave, UnpaidLeave, Holiday, Volunteer, VacationDays, VacationEntitlement, CustomExtraHours(_)).
- `test_snapshot_v3_pinned_values`: 11 explizite Asserts + Surface-Check für CustomExtraHours + `values.len() == 11` Surface-Lock.

**Bekannte Pre-existing Issues** (von Phase 1, nicht Phase-2-relevant):
- 8 fehlschlagende `shifty_bin::integration_test::absence_period`-Tests (`SqliteError "no such table: absence_period"`) — Phase-1-Carry-Over (fehlende Migration), dokumentiert in `deferred-items.md`, Adressierung in Phase 4 geplant. Pro Verifications-Anfrage explizit ausgeschlossen.
- Pre-existing `localdb.sqlite3`-Drift (lokaler Dev-State, nicht checked-in) — `cargo run --bin shifty_bin` panickt mit `VersionMissing(20260428101456)`. Auf frischer DB laufen alle 41 Migrationen sauber durch.

---

*Verified: 2026-05-02T11:00:00Z*
*Verifier: Claude (gsd-verifier)*
