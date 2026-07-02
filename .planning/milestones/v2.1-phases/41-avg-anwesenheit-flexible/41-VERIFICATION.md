---
phase: 41-avg-anwesenheit-flexible
verified: 2026-07-02T12:30:00Z
status: passed
score: 13/13 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification: false
---

# Phase 41: Ø-Anwesenheit bei flexiblen Stunden (BE+FE) — Verification Report

**Phase Goal:** HR kann die durchschnittliche tatsächliche Anwesenheit flexibler Mitarbeiter (is_dynamic) über den angezeigten Report-Zeitraum einsehen, wobei Urlaub/jede Absence per Konstruktion aus dem Nenner heraus ist; < 2 Anwesenheitstage → Leerzustand.
**Verified:** 2026-07-02T12:30:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | D-AVG-01: pure fn average_hours_per_attendance_day GETRENNT von A-22-1; tagebasiert | ✓ VERIFIED | `service/src/reporting.rs:277` — eigene Funktion, eigener Input-Typ `&[WorkingHoursDay]`; A-22-1 bei Zeile 217 unverändert |
| 2 | D-AVG-02: Anwesenheitstag = Kalendertag mit Shiftplan/ExtraWork/VolunteerWork + hours > 0; Datum dedupliziert | ✓ VERIFIED | `service/src/reporting.rs:282-288` — BTreeSet-Dedup; 7 Unit-Tests grün incl. mixed_day (selbes Datum 2× mit versch. Kategorien → 1 Tag) |
| 3 | D-AVG-03: Vacation/SickLeave/Holiday/UnpaidLeave/Unavailable/Custom per Konstruktion draußen | ✓ VERIFIED | matches!-Filter schließt alle Nicht-Arbeitskategorien aus; Tests absence_day_not_counted + custom_category_not_attendance grün |
| 4 | D-AVG-06: < 2 Anwesenheitstage → None; 12 Tage 54h → Some(4.5) | ✓ VERIFIED | Tests user_example (Some(4.5)), one_day_returns_none, two_days_returns_some grün; `cargo test -p service_impl reporting_avg_attendance` → 7/7 ok |
| 5 | D-AVG-08: A-22-1 unverändert; CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 12 | ✓ VERIFIED | `grep CURRENT_SNAPSHOT_SCHEMA_VERSION` → Zeile 117: `pub const = 12`; A-22-1-Regressions-Suite 9/9 grün |
| 6 | D-AVG-04: Service aggregiert über Report-Zeitraum (year/until_week), kein separater Datepicker | ✓ VERIFIED | `service_impl/src/reporting.rs:1221` — ruft `get_report_for_employee(year, until_week)`; REST-Endpoint Range-Parameter: `?year=Y&until_week=W` |
| 7 | D-AVG-05: HR_PRIVILEGE-Check ist ERSTE Operation; nicht-flexible MA → Ok(None) server-seitig | ✓ VERIFIED | `service_impl/src/reporting.rs:1203-1214` — check_permission vor jedem Datenabruf; Mock-Test attendance_statistics_requires_hr via .times(0) bewiesen; attendance_statistics_returns_none_for_static grün |
| 8 | D-AVG-05: HR-gated REST-Endpoint GET /report/{id}/attendance-statistics mit utoipa + ReportApiDoc | ✓ VERIFIED | `rest/src/report.rs:201,254,261` — `#[utoipa::path]` vorhanden; ReportApiDoc: paths(get_attendance_statistics) + components(schemas(EmployeeAttendanceStatisticsTO)) |
| 9 | D-AVG-08: reines Read-Aggregat; EmployeeAttendanceStatisticsTO in rest-types; kein Snapshot-Bump | ✓ VERIFIED | `rest-types/src/lib.rs:626` — TO mit ToSchema + From-Impl; Snapshot-Version 12 unverändert; kein BillingPeriodValueType-Eintrag |
| 10 | D-AVG-07: TupleRow direkt nach "Ø Std/Woche", vor "Einbezogene Wochen"; nur bei Some(attendance_statistics) | ✓ VERIFIED | `employee_view.rs:531-565` — Row direkt nach AverageWorkedHoursPerWeek-TupleRow, vor StatisticsIncludedWeeks-Row; if let Some(att) guard |
| 11 | D-AVG-06: EN-DASH "–" (text-ink-muted) bei avg==None; zwei-stufig Option (Row fehlt bei äußerem None) | ✓ VERIFIED | `employee_view.rs:541-558`; SSR-Tests attendance_row_shows_endash_when_inner_none + attendance_row_absent_when_none grün |
| 12 | D-AVG-05: attendance_statistics=None für nicht-flexible MA / Nicht-HR → keine Row gerendert | ✓ VERIFIED | `load_employee_data` setzt attendance_statistics immer neu (auch None); SSR-Test attendance_row_absent_when_none grün |
| 13 | D-AVG-09: i18n de/en/cs (Label, Beschreibung, Leerzustand) vollständig | ✓ VERIFIED | `i18n_attendance_keys_present_in_all_locales` grün (1/1); 3 Keys × 3 Locales mit nicht-leeren Übersetzungen |

**Score:** 13/13 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `service/src/reporting.rs` | EmployeeAttendanceStatistics struct + average_hours_per_attendance_day fn | ✓ VERIFIED | Zeile 251/277; korrekte Feldtypen; pub |
| `service/src/reporting.rs` | get_employee_attendance_statistics in trait ReportingService | ✓ VERIFIED | Zeile 368; korrekte async-Signatur mit Option<EmployeeAttendanceStatistics> |
| `service_impl/src/reporting.rs` | impl get_employee_attendance_statistics (HR-gate → is_dynamic-Filter → report → pure fn) | ✓ VERIFIED | Zeile 1194–1233; Reihenfolge gate→filter→fetch→aggregate korrekt |
| `service_impl/src/test/reporting_avg_attendance.rs` | 7 pure-fn Nyquist-Fälle | ✓ VERIFIED | 7/7 Tests grün |
| `service_impl/src/test/reporting_attendance_gate.rs` | Mock-Tests HR-Gate + is_dynamic-None | ✓ VERIFIED | 2/2 Tests grün; .times(0) beweist Reihenfolge |
| `rest-types/src/lib.rs` | EmployeeAttendanceStatisticsTO (ToSchema) + From-Impl | ✓ VERIFIED | Zeile 626/637; cfg(feature="service-impl") |
| `rest/src/report.rs` | get_attendance_statistics Handler + Route + ApiDoc | ✓ VERIFIED | Zeile 201/254/261; utoipa-Annotation + ReportApiDoc vollständig |
| `shifty-dioxus/src/i18n/` | 3 Keys × de/en/cs + Completeness-Test | ✓ VERIFIED | mod.rs:602/604/606; de/en/cs vollständig; Test grün |
| `shifty-dioxus/src/api.rs` | get_employee_attendance_statistics loader | ✓ VERIFIED | Zeile 404; Result<Option<Rc<TO>>, reqwest::Error> |
| `shifty-dioxus/src/service/employee.rs` | EmployeeStore.attendance_statistics + load_employee_data | ✓ VERIFIED | Zeile 36/75/124/140; always-reset (Pitfall 5) |
| `shifty-dioxus/src/component/employee_view.rs` | TupleRow + attendance_statistics Prop + zwei-stufig Option | ✓ VERIFIED | Zeile 47/541-558; SSR-Tests 3/3 grün |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `service_impl/src/reporting.rs:1194` | `service::reporting::average_hours_per_attendance_day` | flatten by_week[*].days → pure fn | ✓ WIRED | Zeile 1226-1231 |
| `rest/src/report.rs:get_attendance_statistics` | `reporting_service().get_employee_attendance_statistics` | Handler-Aufruf | ✓ WIRED | Zeile 227; error_handler wrapping |
| `shifty-dioxus/src/service/employee.rs:load_employee_data` | `api::get_employee_attendance_statistics` | await + .ok().flatten() | ✓ WIRED | Zeile 124-140; always-reset |
| `employee_view.rs:props.attendance_statistics` | TupleRow rendering | if let Some(att) guard | ✓ WIRED | Zeile 541-558 |
| `ReportApiDoc` | `get_attendance_statistics` + `EmployeeAttendanceStatisticsTO` | paths() + components(schemas()) | ✓ WIRED | Zeile 254/261 |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 7 pure-fn Nyquist-Fälle (inkl. user example 12d/54h→4.5) | `cargo test -p service_impl reporting_avg_attendance` | 7/7 ok in 0.00s | ✓ PASS |
| A-22-1 Regression (9 Fälle) | `cargo test -p service_impl reporting_avg_weekly` | 9/9 ok | ✓ PASS |
| HR-Gate-first + is_dynamic-Filter Mock-Tests | `cargo test -p service_impl reporting_attendance_gate` | 2/2 ok | ✓ PASS |
| i18n Completeness de/en/cs | `cargo test i18n_attendance_keys_present_in_all_locales` | 1/1 ok | ✓ PASS |
| SSR Rendering: Some(avg)/None-inner/None-outer | `cargo test attendance_row` | 3/3 ok | ✓ PASS |
| Clippy Workspace | `cargo clippy --workspace -- -D warnings` | Finished, 0 errors | ✓ PASS |
| Backend Workspace Tests | `cargo test --workspace` | 568+64+… passed; 0 failed | ✓ PASS |
| WASM-Build | `cargo build --target wasm32-unknown-unknown` (shifty-dioxus) | Finished in 1m 23s | ✓ PASS |
| Snapshot-Version bleibt 12 | `grep CURRENT_SNAPSHOT_SCHEMA_VERSION.*12 service_impl/src/billing_period_report.rs` | Zeile 117 match | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| AVG-01 | 41-01, 41-02 | Tagebasierte Ø-Kennzahl für flexible MA, Absence per Konstruktion draußen | ✓ SATISFIED | pure fn + Trait-Methode + 9 Tests beweisen Formel, Kategoriefilter, < 2-Schwelle |
| AVG-02 | 41-02, 41-03 | Auswertung im Frontend sichtbar (REST-Endpoint + HR-Gate) | ✓ SATISFIED | GET /report/{id}/attendance-statistics; ApiDoc; FE-Loader; TupleRow hinter should_show_hr_stats |
| AVG-03 | 41-04 | i18n de/en/cs (Labels, Tooltips, Leerzustand) | ✓ SATISFIED | 3 Keys × 3 Locales; Completeness-Test grün |

### Anti-Patterns Found

Keine in den Phase-41-Dateien. Kein TBD/FIXME/XXX in den modifizierten Dateien gefunden.

### Bekannter Vorläufer-Fehler (nicht Phase 41)

`i18n_impersonation_keys_match_german_reference` schlägt fehl — pre-existing Defekt aus Phase 37-02, nicht durch Phase 41 eingeführt. 745 andere FE-Tests grün; die 3 neuen Phase-41-SSR-Tests und der i18n-Completeness-Test grün.

### Human Verification Required

Keine automatisierten Lücken. Der optionale Browser-Smoke (Zahl neben "Ø Std/Woche" im laufenden Backend) ist als Manual-Only in 41-VALIDATION.md klassifiziert. Alle programmatisch prüfbaren Gates grün.

---

_Verified: 2026-07-02T12:30:00Z_
_Verifier: Claude (gsd-verifier)_
