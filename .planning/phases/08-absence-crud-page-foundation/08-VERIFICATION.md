---
phase: 08-absence-crud-page-foundation
verified: 2026-05-08T20:33:23Z
status: human_needed
score: 16/16 must-haves verifiziert (automated)
overrides_applied: 0
re_verification: null
human_verification:
  - test: "HR-User UAT-Smoke (20 Schritte)"
    expected: "Login HR → TopBar 'Verwaltung'-Submenu → AbsencesPage rendert → VacationEntitlementCard Team-Variante mit VacationPerPersonList → AbsenceFilterBar 3 Filter → AbsenceList 5-Spalten → Modal Create (incl. Cross-Field-Validation 'Bis<Von', WarningList bei Booking-Konflikt) → AbsenceList refresh → Edit existing → Delete via DeleteConfirmDialog (NICHT window.confirm) → Self-Overlap 422 inline rendert SelfOverlapBanner ohne Modal-Close"
    why_human: "Browser-Interaktion mit HR-Auth-Rolle gegen Integrations-Backend; visuelle Verifikation Modal-Layout, Banner-Styles, Refresh-Verhalten — siehe 08-HUMAN-UAT.md Schritte 1–20"
  - test: "Employee-User UAT-Smoke (15 Schritte)"
    expected: "Login Employee → AbsencesPage Top-Level (kein Verwaltung-Submenu) → VacationEntitlementCard Self-Variante (Hero-Layout) → kein Person-Dropdown → AbsenceList nur eigene Einträge → Modal Mitarbeiter-Dropdown DISABLED, vorgefüllt → Anlage / Edit / Delete eigene Einträge → Forbidden-Test (T-8-AUTH-01 + T-8-IDOR-01): GET /absence-period/by-sales-person/{andere-uuid} → 403 → Locale-Switch En/Cs/De"
    why_human: "Browser-Interaktion mit Employee-Auth-Rolle; Defense-in-Depth-Test 403 erfordert manuellen DevTools-Console-Aufruf; Locale-Switch-Verifikation visuell — siehe 08-HUMAN-UAT.md Schritte 21–35"
  - test: "Forward-Warnings AbsencePeriodCreateResultTO.warnings[] Rendering bei Booking-Konflikt"
    expected: "Vacation-Range über existierende Booking legen → POST/PUT triggern → WarningList rendert vor Modal-Close mit Acknowledge-Btn → click → Modal schließt"
    why_human: "Erfordert seeded Konflikt-Daten + Browser-Interaktion; Forward-Warning-Logik ist Backend-emittiert, Render ist UI-Layer"
  - test: "409 Version-Konflikt-Banner (D-08)"
    expected: "Zwei Browser-Tabs öffnen → in beiden gleicher Eintrag laden → Tab 1 speichern → Tab 2 speichern → VersionConflictBanner mit 'Erneut laden?'-Button"
    why_human: "Konkurrente Edit-Session, nicht automatisierbar in cargo test"
deferred_uat_blocked_by: phase-9-cutover-migration-ui
deferred_uat_source: 08-HUMAN-UAT.md
---

# Phase 8: Absence-CRUD-Page Foundation — Verification Report

**Phase Goal:** Frontend-AbsencesPage mit CRUD über `/absence-period`, Resturlaubs-Anzeige (Self + Team), Forward-Warnings, vollständig integriert + WASM-grün.

**Verified:** 2026-05-08T20:33:23Z
**Status:** human_needed (automated gates pass; UAT formell deferred zu Phase 9)
**Re-verification:** No — initial verification

---

## Goal Achievement Overview

Phase 8 ist **funktional fertig codiert**: Backend-Resturlaubs-Endpoint + REST-Wiring + DI, Frontend-Foundation (api/state/loader/coroutine/i18n/proxy), AbsencesPage mit 12 Inline-Komponenten + 11 ssr-Tests, Route-Registrierung, TopBar-Eintrag, Cutover-Gate, responsive Layout, Cutover-UX-Polish (Plans 08-08 inline drift report + 08-09 Wochenpauschalen-Heuristik). Automatisierte Regression-Gates re-verifiziert: Backend `cargo test --workspace` 488+ Tests grün, Frontend `cargo test` 509 Tests grün, WASM `cargo check --target wasm32-unknown-unknown` grün.

Die int-UAT (HR + Employee Browser-Smoke gegen Integrations-Backend) wurde bewusst zu Phase 9 (Cutover-Migration-UI) verschoben. Begründung: Cutover-Auto-Heuristik (Plan 08-09) deckt nicht alle realen Buchungs-Patterns ab; manuelle Drift-Resolution gehört in eine UI, nicht in immer komplexere Heuristiken. Siehe `08-HUMAN-UAT.md` (status: partial, blocked_by: phase-9).

---

## Observable Truths (Goal-Backward)

| #  | Truth | Status | Evidence |
| -- | ----- | ------ | -------- |
| 1  | Route `/absences` ist via Top-Bar-Menü erreichbar (FUI-A-01, SC-1) | VERIFIED | `shifty-dioxus/src/router.rs:58` Route::Absences `#[route("/absences/")]`, `top_bar.rs:54+409` nav_visibility.absences gerendert (cutover-gated) |
| 2  | HR-Privileg schaltet Filter über alle Mitarbeiter (FUI-A-02, SC-1) | VERIFIED | `page/absences.rs:1457` `is_hr = auth.has_privilege("hr")`, branching in AbsenceFilterBar/VacationEntitlementCard/AbsenceList |
| 3  | AbsencesPage rendert HR-Variante (Liste über alle + VacationPerPersonList) | VERIFIED | `page/absences.rs:262-281` `if props.is_hr` branch in VacationEntitlementCard; `page/absences.rs:381` VacationPerPersonList HR-only |
| 4  | AbsencesPage rendert Employee-Variante (eigene Liste + Self-Card) | VERIFIED | `page/absences.rs:262` `if !props.is_hr` Hero-Layout; AbsenceList filtert auf eigene Einträge |
| 5  | Modal bietet Range-Picker + Kategorie-Dropdown + Description (FUI-A-03, SC-2) | VERIFIED | `page/absences.rs:729` AbsenceModal mit native `<input type="date">` + Cross-Field-Validation `to >= from` (D-05); Kategorie-Dropdown Vacation/SickLeave/UnpaidLeave; Textarea |
| 6  | Self-Overlap-422 wird als Validation-Banner gerendert (FUI-A-03, SC-2) | VERIFIED | `error.rs:39` ShiftyError::Validation; `page/absences.rs:629` SelfOverlapBanner inline |
| 7  | 409 Version-Konflikt-Banner mit Reload-Btn (D-08) | VERIFIED | `error.rs:36` ShiftyError::Conflict; `page/absences.rs:601` VersionConflictBanner |
| 8  | AbsencePeriodCreateResultTO.warnings[] als nicht-blockierende Liste (FUI-A-04, SC-3) | VERIFIED | `page/absences.rs:194` WarningList component; `page/absences.rs:789` warnings_state Acknowledge-Btn vor Modal-Close |
| 9  | Backend Resturlaubs-Endpoint GET /vacation-balance/{sp}/{year} (HR ∨ self) | VERIFIED | `service/src/vacation_balance.rs` Trait + Domain; `service_impl/src/vacation_balance.rs:110-111` `tokio::join!(check_permission(HR), verify_user_is_sales_person)`; `rest/src/vacation_balance.rs:45-79` utoipa-Path |
| 10 | Backend Resturlaubs-Endpoint GET /vacation-balance/team/{year} (HR-only) | VERIFIED | `service_impl/src/vacation_balance.rs:137` `check_permission(HR_PRIVILEGE, ...)`; `rest/src/vacation_balance.rs:81-111` utoipa-Path |
| 11 | VacationBalanceTO mit ToSchema in rest-types | VERIFIED | `rest-types/src/lib.rs` `pub struct VacationBalanceTO` mit `#[derive(...,ToSchema)]`; From-Impls feature-gated |
| 12 | DI-Wiring in shifty_bin/main.rs (BL-Tier-Reihenfolge) | VERIFIED | `shifty_bin/src/main.rs:884+` VacationBalanceServiceImpl konstruiert NACH absence_service/working_hours/carryover; RestStateImpl-Field + Getter |
| 13 | Frontend Coroutines registriert, Stores mit DataFlow | VERIFIED | `app.rs:27-32` use_coroutine für absence/vacation_balance/feature_flag; `page/absences.rs:1464+1495+1499` LoadAll/LoadForSalesPerson/LoadSelf/LoadTeam Sends; ABSENCE_STORE/VACATION_BALANCE_STORE.read() in render |
| 14 | Cutover-Gate im FE-Menü (Plan 08-07) | VERIFIED | `top_bar.rs:54` `absences: logged_in && cutover_active`; `top_bar.rs:350-359` FEATURE_FLAGS_STORE.read() + nav_visibility |
| 15 | Cutover Drift-Report inline (Plan 08-08) — interpretierbare Diagnostik | VERIFIED | `service_impl/cutover.rs:929` build_gate_drift_report; `service/src/cutover.rs:48,71` QuarantineReason::human_text + suggested_action; `rest-types/src/lib.rs:1943` CutoverQuarantineEntryTO + CutoverGateDriftReportTO |
| 16 | Cutover Wochenpauschalen-Heuristik (Plan 08-09) | VERIFIED | `service_impl/cutover.rs:1064` detect_weekly_lump_sum; `service_impl/cutover.rs:1130` lookup_active_contract; `service_impl/cutover.rs:1153` iso_week_range; 7 unit-tests + 1 integration-test (alle grün) |

**Score:** 16/16 must-haves verifiziert (automated)

---

## Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `service/src/vacation_balance.rs` | Trait + Domain-Struct + automock | VERIFIED | 4551 bytes; `pub trait VacationBalanceService` mit `get` + `get_team` |
| `service_impl/src/vacation_balance.rs` | gen_service_impl! BL-Tier | VERIFIED | 9301 bytes; gen_service_impl + Permission HR ∨ self via tokio::join |
| `service_impl/src/test/vacation_balance.rs` | ≥6 Unit-Tests mit mockall | VERIFIED | 17048 bytes; 7 `#[tokio::test]` |
| `rest/src/vacation_balance.rs` | REST-Handler + utoipa + ApiDoc | VERIFIED | 4193 bytes; 2× `#[utoipa::path]`, VacationBalanceApiDoc, generate_route |
| `rest-types/src/lib.rs` | VacationBalanceTO + FeatureFlagTO | VERIFIED | Strukturen vorhanden, From-Impls feature-gated `service-impl` |
| `shifty_bin/src/main.rs` | DI-Wiring | VERIFIED | VacationBalanceServiceDeps + Konstruktor + RestStateImpl + Getter |
| `rest/src/feature_flag.rs` | GET /feature-flag/{key} | VERIFIED | 3454 bytes; utoipa-typed, fail-safe enabled=false für unknown keys |
| `migrations/sqlite/20260508120000_admin-auto-grant-privilege.sql` | AFTER-INSERT-Trigger + Backfill | VERIFIED | 1715 bytes (Datum 2026-05-08) |
| `shifty-dioxus/src/page/absences.rs` | Page + 12 Komponenten + 11 Tests | VERIFIED | 75192 bytes (~1685 LOC); AbsencesPage + AbsenceModal + WarningList + CategoryBadge + StatusPill + VacationEntitlementCard + VacationPerPersonList + AbsenceList + AbsenceFilterBar + StatsGrid + DeleteConfirmDialog + VersionConflictBanner + SelfOverlapBanner |
| `shifty-dioxus/src/state/absence_period.rs` | AbsencePeriod state + AbsenceCategory + From-TO | VERIFIED | 3906 bytes |
| `shifty-dioxus/src/state/vacation_balance.rs` | VacationBalance state + From-TO | VERIFIED | 1751 bytes |
| `shifty-dioxus/src/service/absence.rs` | Coroutine + AbsenceAction + Stores | VERIFIED | 8713 bytes |
| `shifty-dioxus/src/service/vacation_balance.rs` | Coroutine + VacationBalanceAction | VERIFIED | 2097 bytes |
| `shifty-dioxus/src/state/feature_flag.rs` | FeatureFlagsState + Loader | VERIFIED | 3195 bytes |
| `shifty-dioxus/src/service/feature_flag.rs` | feature_flag_service Coroutine | VERIFIED | 2284 bytes |
| `shifty-dioxus/Dioxus.toml` | Proxy /absence-period + /vacation-balance | VERIFIED | beide Einträge present |

---

## Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `service/src/lib.rs` | `service/src/vacation_balance.rs` | `pub mod vacation_balance;` | WIRED | service/src/lib.rs:44 |
| `rest-types/src/lib.rs` | `service::vacation_balance::VacationBalance` | feature-gated From-Impl | WIRED | beide From-Impls hinter `#[cfg(feature = "service-impl")]` |
| `rest/src/lib.rs` | `vacation_balance::generate_route` | `.nest("/vacation-balance", ...)` | WIRED | rest/src/lib.rs:591 |
| `rest/src/lib.rs` | VacationBalanceApiDoc | utoipa nest entry | WIRED | rest/src/lib.rs:526 `(path = "/vacation-balance", api = vacation_balance::VacationBalanceApiDoc)` |
| `shifty_bin/src/main.rs` | `service_impl::vacation_balance::VacationBalanceServiceImpl` | type alias + Konstruktor + DI | WIRED | main.rs:263+276+884 |
| `shifty-dioxus/src/router.rs` | `page::AbsencesPage` | Route::Absences {} → AbsencesPage | WIRED | router.rs:58 |
| `shifty-dioxus/src/component/top_bar.rs` | Route::Absences | nav_items push (Key::AbsenceMenuLabel) | WIRED | top_bar.rs:407-409 (cutover-gated via visibility.absences) |
| `shifty-dioxus/src/page/absences.rs::AbsenceModal` | `service::absence::AbsenceAction::Create` | absence_service.send | WIRED | absences.rs:936 |
| `shifty-dioxus/src/page/absences.rs::VacationEntitlementCard` | `VACATION_BALANCE_STORE` | .read() | WIRED | absences.rs:1514 |
| `shifty-dioxus/src/app.rs` | absence_service + vacation_balance_service + feature_flag_service | use_coroutine | WIRED | app.rs:27-32 |

---

## Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| AbsencesPage | absences (ABSENCE_STORE) | absence_service Coroutine via api::list_absence_periods → `/absence-period` (proxy → backend SQLite) | Yes — Backend liefert echte Liste aus `absence_period`-Tabelle (DAO-impl seit v1.0 Phase 1) | FLOWING |
| VacationEntitlementCard | vacation_self (VACATION_BALANCE_STORE) | vacation_balance_service::LoadSelf → api::get_vacation_balance → `/vacation-balance/{sp}/{year}` → VacationBalanceServiceImpl.get | Yes — Service aggregiert AbsenceService + EmployeeWorkDetails + Carryover (Plan 08-02 7 Unit-Tests grün) | FLOWING |
| VacationPerPersonList | vacation_team (VACATION_TEAM_STORE) | vacation_balance_service::LoadTeam → api::get_team_vacation_balance → `/vacation-balance/team/{year}` → VacationBalanceServiceImpl.get_team | Yes — get_team iteriert über alle paid sales_persons; HR-only-Gate enforced | FLOWING |
| AbsenceModal warnings | warnings_state | api::create_absence_period / update_absence_period Response (AbsencePeriodCreateResultTO.warnings[]) | Yes — Backend emitted warnings (Forward-Warnings aus v1.0 Phase 3) | FLOWING |
| TopBar nav_items.absences | visibility.absences | FEATURE_FLAGS_STORE.read().absence_range_source_active && logged_in | Yes — feature_flag_service lädt absence_range_source_active beim App-Start | FLOWING |

Alle Render-Komponenten haben echte Datenquellen verdrahtet — keine HOLLOW oder STATIC.

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Backend full-suite | `nix develop --command cargo test --workspace` | 488+ Tests passed, 0 failed (output zeigt Crates mit 396, 68, 11, 10, 8, 3 Tests etc., gesamt > 488 wie in 08-06-SUMMARY) | PASS |
| Frontend cargo test | `nix develop --command bash -c "cd shifty-dioxus && cargo test"` | 509 passed; 0 failed | PASS |
| WASM compile-gate | `nix develop --command bash -c "cd shifty-dioxus && cargo check --target wasm32-unknown-unknown"` | Finished `dev` profile in 15.11s — 0 errors, 38 warnings | PASS |
| OpenAPI Surface assertion | `cargo test -p rest --test openapi_surface` (Teil der workspace-Suite) | passed (im `cargo test --workspace` enthalten); pinnt VacationBalance-Tag + Pfade | PASS |
| Vacation balance unit tests | `cargo test -p service_impl vacation_balance` (Teil der workspace-Suite) | 7 #[tokio::test], alle passed | PASS |

**Note:** Direktes `cargo build --target wasm32-unknown-unknown` schlug in der aktuellen Sandbox-nix-shell mit "linker `lld` not found" fehl, aber `cargo check --target wasm32-unknown-unknown` lief grün durch. Die existierenden WASM-Builds in `shifty-dioxus/target/wasm32-unknown-unknown/{debug,wasm-dev}/shifty-dioxus.wasm` (Datum 2026-05-08) sowie 08-06-SUMMARY belegen den voll-grünen Build auf Stand `ddf60fd8`. Linker-Issue ist ein Sandbox-Toolchain-Detail, kein Code-Problem.

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ----------- | ----------- | ------ | -------- |
| FUI-A-01 | 08-04, 08-05, 08-07 | Top-Level-Route `absences` mit CRUD über `/absence-period` | SATISFIED (auto) / NEEDS HUMAN (UAT) | Route + Page + Modal + 8 api-Funktionen verdrahtet; UAT-Bestätigung blockiert auf Phase 9 |
| FUI-A-02 | 08-04, 08-05, 08-07 | HR-Sicht über alle Mitarbeiter, Employee-Sicht nur eigen, aus Auth-Context | SATISFIED (auto) / NEEDS HUMAN (UAT) | `is_hr = auth.has_privilege("hr")` Branching im Page+Modal+List+Card; Defense-in-Depth backend `verify_user_is_sales_person` — UAT-Schritt T-8-AUTH-01/IDOR-01 deferred |
| FUI-A-03 | 08-04, 08-05 | Form mit Range-Picker + Kategorie-Dropdown + Description; Self-Overlap-422 als Validation-Error | SATISFIED (auto) / NEEDS HUMAN (UAT) | AbsenceModal mit Cross-Field-Validation, ShiftyError::Validation Variant, SelfOverlapBanner; UAT-Schritt 19 deferred |
| FUI-A-04 | 08-04, 08-05 | AbsencePeriodCreateResultTO.warnings[] als nicht-blockierende Liste | SATISFIED (auto) / NEEDS HUMAN (UAT) | WarningList component + warnings_state + Acknowledge-Btn; UAT-Schritt 11 (Booking-Konflikt-Trigger) deferred |

**ORPHANED Requirements:** Keine — REQUIREMENTS.md mappt FUI-A-01..04 → Phase 8, alle vier sind in mindestens einem Plan-Frontmatter `requirements`-Feld claimed (siehe 08-04/08-05/08-06).

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| (none) | — | — | — | — |

Skill-Scan auf TODO/FIXME/XXX/HACK/placeholder in den Schlüsseldateien (`page/absences.rs`, `service_impl/vacation_balance.rs`, `rest/vacation_balance.rs`) liefert 0 Hits. Keine `dangerous_inner_html` (XSS-Mitigation T-8-XSS-01 ok). Keine `window.confirm` (DeleteConfirmDialog ist Dioxus-Center-Dialog, D-07). Statische Tailwind-match-Arme (Pitfall 5) — keine `format!("text-{}", ...)`-Hits.

---

## Human Verification Required

Phase 8 ist code-side fertig, aber der int-UAT ist deferred. Folgende Tests sind der Integrations-Verifikations-Backlog:

### 1. HR-User UAT-Smoke (20 Schritte)

**Test:** Backend (Port 3000) + Frontend (Port 8080) + Cutover committed (`absence_range_source_active = true`). Login HR-User mit `hr`-Privileg.
**Expected:** Siehe `08-HUMAN-UAT.md` Schritte 1–20: TopBar Verwaltung-Submenu zeigt Abwesenheiten → AbsencesPage rendert → VacationEntitlementCard Team-Variante → AbsenceFilterBar 3 Filter → AbsenceList 5-Spalten → Modal Anlage / Edit / Delete via DeleteConfirmDialog → Self-Overlap-422 inline → Range korrigieren → Save.
**Why human:** Browser-Interaktion + visuelle Verifikation Modal-Layout, Banner-Styles, Refresh-Verhalten gegen Integrations-Backend; nicht automatisierbar in cargo test.
**Blocked by:** Phase 9 — Cutover-Migration-UI (Plan 08-09 Auto-Heuristik deckt nicht alle realen Patterns ab; manuelle Drift-Resolution via UI nötig).

### 2. Employee-User UAT-Smoke (15 Schritte)

**Test:** Login Employee-User OHNE `hr`-Privileg, MIT `sales_person_id`.
**Expected:** Siehe `08-HUMAN-UAT.md` Schritte 21–35: AbsencesPage Top-Level (kein Submenu) → VacationEntitlementCard Self-Variante mit Hero-Layout → kein Person-Dropdown → AbsenceList nur eigene Einträge → Modal Mitarbeiter-Dropdown DISABLED → Anlage/Edit/Delete eigene → Forbidden-Test 403 für fremde sales_person_id → Locale-Switch En/Cs/De.
**Why human:** Browser-Interaktion + Defense-in-Depth-Test (T-8-AUTH-01 + T-8-IDOR-01) erfordert manuelle DevTools-Console; Locale-Switch-Verifikation visuell.
**Blocked by:** Phase 9 — Cutover-Migration-UI.

### 3. Forward-Warnings (FUI-A-04) — Booking-Konflikt-Render

**Test:** Vacation-Range über existierende Booking legen → POST/PUT triggern.
**Expected:** WarningList rendert vor Modal-Close mit Acknowledge-Btn; click → Modal schließt.
**Why human:** Erfordert seeded Konflikt-Daten; Forward-Warning-Logik ist Backend-emittiert (Implementation seit v1.0 Phase 3 verifiziert), Render auf UI-Layer manuell zu verifizieren.

### 4. 409 Version-Konflikt-Banner (D-08)

**Test:** Zwei Browser-Tabs öffnen → in beiden gleichen Eintrag laden → Tab 1 speichern → Tab 2 speichern.
**Expected:** VersionConflictBanner mit "Erneut laden?"-Button.
**Why human:** Konkurrente Edit-Session, nicht automatisierbar.

---

## Gaps Summary

**Keine Code-Gaps.** Alle 16 must-have-Truths sind in der Codebase verifiziert: Backend-Service + REST + DI + Tests + OpenAPI surface assertion + Frontend-API-Layer + State + Service-Coroutine + Page mit 12 Komponenten + Routing + TopBar + Cutover-Gate + Cutover-Drift-Report + Wochenpauschalen-Heuristik. Alle automatisierten Regression-Gates re-verifiziert grün.

**Verbleibende Items sind End-to-End-UAT** (Browser-Smoke gegen Integrations-Backend), die strategisch zu Phase 9 (Cutover-Migration-UI) verschoben wurden, weil das int-Migrations-Setup hängt: die Cutover-Auto-Heuristik (Plan 08-09) deckt etablierte reale Buchungs-Patterns nicht ab (Teil-Wochen-Pauschalen, Feiertag-Inkonsistenz Pre-Check vs Gate, echte Datenprobleme — siehe `08-HUMAN-UAT.md` gap-1). Statt die Auto-Heuristik weiter zu erweitern, wird die manuelle Drift-Resolution durch eine UI in Phase 9 gelöst. Phase 8 Code ist fertig und auf Mock-Backend bereit; der Browser-Smoke folgt sobald migrierte int-Daten verfügbar sind.

**Status `human_needed`** ist die korrekte Klassifikation: keine Code-Lücken (status `passed` falsch, weil 35 UAT-Schritte gentl. menschlich verifiziert werden müssen), keine Implementation-Gaps (status `gaps_found` falsch, weil Code/Tests/Build alle grün).

---

## Phase-8-Goal-Success-Criteria-Mapping

| SC | Source | Status | Notes |
| -- | ------ | ------ | ----- |
| SC-1 | FUI-A-01 — Route via Menü, HR-Privileg schaltet Filter | SATISFIED (auto) / NEEDS HUMAN (UAT-Schritte 2+5+22) | Route + TopBar + Privileg-Branching im Code verifiziert |
| SC-2 | FUI-A-02+03 — CRUD AbsencePeriodTO + 422 Self-Overlap inline | SATISFIED (auto) / NEEDS HUMAN (UAT-Schritte 8-11+19-20) | AbsenceModal CRUD + ShiftyError::Validation + SelfOverlapBanner verdrahtet |
| SC-3 | FUI-A-04 — Forward-Warnings als nicht-blockierende Liste | SATISFIED (auto) / NEEDS HUMAN (UAT-Schritt 11) | WarningList + Acknowledge-Btn verdrahtet |
| SC-4 | D-03/D-04 — Backend-Resturlaubs-Endpoint + FE-Konsumption | SATISFIED (auto) / NEEDS HUMAN (UAT-Schritt 4 HR + 24 Employee) | Service + REST + DI + State-Coroutine + 7 Unit-Tests; visuelle UAT pending |
| SC-5 | WASM grün, Backend Full-Suite grün, UAT erfolgreich | PARTIAL — automated gates grün, UAT deferred | cargo test --workspace + frontend cargo test + cargo check --target wasm32 alle grün; UAT siehe 08-HUMAN-UAT.md |

---

_Verified: 2026-05-08T20:33:23Z_
_Verifier: Claude (gsd-verifier)_
