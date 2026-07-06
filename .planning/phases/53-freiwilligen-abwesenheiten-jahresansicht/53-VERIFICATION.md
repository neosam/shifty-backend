---
phase: 53-freiwilligen-abwesenheiten-jahresansicht
verified: 2026-07-06T00:00:00Z
status: human_needed
score: 9/9 must-haves verified
behavior_unverified: 1
overrides_applied: 0
behavior_unverified_items:
  - truth: "Frontend rendert Freiwilligen-Zeilen visuell konsistent mit bezahlten — Union-Liste erscheint sortiert und ohne Farbmarkierung im Browser"
    test: "Backend starten, FE starten, /weekly_overview/ oeffnen, Woche mit Freiwilligem (aktive Vacation-Period) + Bezahltem mit absence_hours pruefen"
    expected: "Beide Namen erscheinen in einer alphabetisch sortierten Liste im Format '{name}: {hours} h', keine Farb-/Icon-Unterschiede"
    why_human: "Dioxus WASM Signal-Rendering kann nur im Browser verifiziert werden. Cargo-Test beweist den Mapper (Union korrekt, sort korrekt), aber ob das DOM-Update live korrekt ausfaellt und die Signals triggern ist nur im Browser beobachtbar (Memory: reference_dioxus_browser_test_date_inputs)."
human_verification:
  - test: "Browser-Sichtkontrolle der Union-Liste in /weekly_overview/"
    expected: "Freiwilliger mit aktiver Absence-Period + Bezahlter mit absence_hours erscheinen beide in einer alphabetisch sortierten Absencen-Zeile; Format '{name}: {hours} h'; kein Icon/Farb-Unterschied zwischen Freiwilligem und Bezahltem"
    why_human: "Dioxus WASM live-render ist headless nicht pruefbar. Cargo-Test (sales_person_absences_union_merges_paid_and_volunteers_sorted_by_name) beweist den Mapper, aber der Browser-Render-Path (Signal-Update via LoadYear-Coroutine -> WeeklySummary -> Rendering-Zeile) erfordert Sichtkontrolle."
---

# Phase 53: Freiwilligen-Abwesenheiten in Jahresansicht — Verification Report

**Phase Goal:** Freiwillige mit aktiver Vacation/SickLeave/UnpaidLeave-Period erscheinen in `sales_person_absences` der Jahresansicht zusaetzlich zu bezahlten Mitarbeitern. Backend liefert Name + Stunden-Wert fertig im DTO. Frontend rendert mit bestehender Zeile.
**Verified:** 2026-07-06
**Status:** HUMAN_NEEDED (1 behavior-unverified truth — Browser-Sichtkontrolle)
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | VAA-01: `SalesPersonAbsence` (service) + `SalesPersonAbsenceTO` (rest-types) Structs existieren mit korrekten Feldern und Derives | VERIFIED | `service/src/booking_information.rs:45` struct exists; `rest-types/src/lib.rs:997` struct exists; both with correct fields (sales_person_id, name, hours) and derives |
| 2 | VAA-01: `WeeklySummary.sales_person_absences` + `WeeklySummaryTO.sales_person_absences` Felder existieren; DTO-Feld hat `#[serde(default)]` | VERIFIED | `service/src/booking_information.rs:73`; `rest-types/src/lib.rs` has `#[serde(default)]` immediately before field; From-impl chains correctly via `SalesPersonAbsenceTO::from` at line 1066 |
| 3 | VAA-01/D-53-05: Fill-Site 1 in `get_weekly_summary` befuellt `sales_person_absences` im Per-Woche-Assembly-Loop | VERIFIED | `service_impl/src/booking_information.rs:528-559` fill-block exists; `668` literal assignment confirmed; `assemble_weeks` in `reporting.rs` untouched |
| 4 | VAA-01/D-53-06: Fill-Site 2 in `get_summery_for_week` befuellt dasselbe Feld analog | VERIFIED | `service_impl/src/booking_information.rs:855-885` fill-block + `1024` literal; `all_absences` + `all_sales_persons` + `absent_volunteer_ids` HashSet inline gebaut (lines 728-760) |
| 5 | VAA-02/D-53-02: Stunden-Wert = Sigma cap-gated committed_voluntary, mit `wh.sales_person_id == sp_id`-Filter (Pitfall 2) | VERIFIED | `booking_information.rs:501-506` (Fill-Site 1): `find_working_hours_for_calendar_week` + `.filter(|wh| wh.sales_person_id == sp_id && (wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0))` + `.map(|wh| wh.committed_voluntary)`; identical at 864-875 (Fill-Site 2) |
| 6 | VAA-03/D-53-03: Sichtbarkeitskriterium exakt `absent_volunteer_ids`; drei Backend-Tests gruen | VERIFIED | `cargo test -p service_impl booking_information_vaa` — 3/3 PASSED: `vaa03_volunteer_with_period_appears_with_correct_hours`, `vaa03_volunteer_without_period_absent_not_in_list`, `vaa03_paid_employee_unchanged_regression_lock` |
| 7 | VAA-03 #3/Regression-Lock: `WorkingHoursPerSalesPerson` + Feld `working_hours_per_sales_person` unveraendert (bezahlten-only Vertrag) | VERIFIED | `service/src/booking_information.rs:24-35` struct unchanged (7 fields, same types); separate from `SalesPersonAbsence`; no paid employee in `sales_person_absences` confirmed by test |
| 8 | VAA-04/D-53-04: FE-Union-Merge in `state::WeeklySummary::from()` — Bezahlten-Loop unveraendert + Freiwilligen-extend + case-insensitive sort_by(name) | VERIFIED | `shifty-dioxus/src/state/weekly_overview.rs:35-67`; tests `sales_person_absences_union_merges_paid_and_volunteers_sorted_by_name` + `bezahlter_bleibt_via_working_hours_pfad_sichtbar` both PASS (4/4 state::weekly_overview tests green) |
| 9 | VAA-04 Rendering-Lock: `page/weekly_overview.rs:126` unveraendert `format!("{}: {} {hours_short}", absence.name, format_hours(absence.absence_hours, 2))` | VERIFIED | `grep -n 'format!("{}: {} {hours_short}", absence.name, format_hours(absence.absence_hours, 2))' shifty-dioxus/src/page/weekly_overview.rs` — exact 1 match at line 126 |
| 10 | VAA-04 visuelle Konsistenz Freiwillige/Bezahlte im Browser | PRESENT_BEHAVIOR_UNVERIFIED | Code present and wired correctly; browser rendering of WASM signals not automatable — see human verification |

**Score:** 9/9 truths verified (1 present, behavior-unverified)

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `service/src/booking_information.rs` | `SalesPersonAbsence` struct + `WeeklySummary.sales_person_absences` field | VERIFIED | Struct at line 45; field at line 73 |
| `rest-types/src/lib.rs` | `SalesPersonAbsenceTO` struct + `WeeklySummaryTO.sales_person_absences` + `#[serde(default)]` + From-impls | VERIFIED | Struct at 997; serde(default) guard confirmed; From<&SalesPersonAbsence> at 1003; mapping in From<&WeeklySummary> at 1066 |
| `service_impl/src/booking_information.rs` | Fill-Site 1 (get_weekly_summary) + Fill-Site 2 (get_summery_for_week) | VERIFIED | 5 occurrences of `sales_person_absences`; 2 fill-blocks + 2 literal-uses + 1 in inline test |
| `service_impl/src/test/booking_information_vaa.rs` | 3 VAA-03 test functions | VERIFIED | File exists (406 lines); all 3 tokio-tests present and green |
| `service_impl/src/test/mod.rs` | `pub mod booking_information_vaa;` registration | VERIFIED | Line 8: `pub mod booking_information_vaa;` confirmed |
| `shifty-dioxus/src/state/weekly_overview.rs` | Union-Merge refactor + make_to() fixture + 2 new tests | VERIFIED | Union-merge at lines 35-67; sort_by at 66; 2 new tests confirmed green |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `absent_volunteer_ids` HashSet | `sales_person_absences` fill-block | `filter_map` over absent IDs in per-week loop | WIRED | `booking_information.rs:530` uses `absent_volunteer_ids` as source; `volunteer_ids.contains` filter present |
| `all_sales_persons` Arc | `name` field in SalesPersonAbsence | `.iter().find(|sp| sp.id == sp_id).map(|sp| sp.name.clone())` | WIRED | Lines 533-537 (Fill-Site 1), 860-864 (Fill-Site 2) |
| `WeeklySummaryTO.sales_person_absences` | `state::WeeklySummary.sales_person_absences` | `From<&WeeklySummaryTO>` mapper | WIRED | `state/weekly_overview.rs:54-62` iterates `.sales_person_absences` from DTO |
| `state::WeeklySummary.sales_person_absences` | Rendering in `page/weekly_overview.rs:126` | `absence.name` + `absence.absence_hours` in format! | WIRED | Rendering-line uses `absence.name` from the union Vec |
| `volunteer_ids.contains` filter | `absent_volunteer_ids` in `get_summery_for_week` | Pitfall-6 guard | WIRED | `booking_information.rs:740`: `volunteer_ids.contains(&period.sales_person_id)` confirmed |

---

## Decision Coverage

| Decision | Requirement | Evidence | Status |
|----------|-------------|----------|--------|
| D-53-01: New `SalesPersonAbsence` + `SalesPersonAbsenceTO` structs with correct fields/derives | VAA-01 | `service/src/booking_information.rs:44-49`; `rest-types/src/lib.rs:997-1008`; derives confirmed | PASS |
| D-53-02: `hours = Sigma filter(sales_person_id==sp_id && cap||expected==0).map(committed_voluntary)` | VAA-02 | `booking_information.rs:501-506` (site 1) + `864-875` (site 2); `wh.sales_person_id == sp_id` filter present at both sites | PASS |
| D-53-03: Visibility criterion = exactly `absent_volunteer_ids` (VFA-01 whole-week-out) | VAA-03 | `booking_information.rs:530` iterates `absent_volunteer_ids`; same HashSet built via `period_overlaps_week` as VFA-01 | PASS |
| D-53-04: FE Union-merge + case-insensitive sort by name + >= 0.1 filter for volunteers | VAA-04 | `state/weekly_overview.rs:53-66`; `sort_by(|x,y| x.name.to_lowercase().cmp(...))` at line 66; `filter(|a| a.hours >= 0.1)` at line 58 | PASS |
| D-53-05: Fill-Site in `get_weekly_summary` assembly loop, NOT `assemble_weeks` | VAA-01 | `reporting.rs` has 0 occurrences of `sales_person_absences`; fill-block in `booking_information.rs:528+` inside `get_weekly_summary` | PASS |
| D-53-06: `get_summery_for_week` fills field analogously; builds `absent_volunteer_ids` inline | VAA-01 | `booking_information.rs:728-760` builds absent_volunteer_ids inline; fill-block 855-885; literal 1024 | PASS |

---

## Requirements Coverage

| Requirement | Description | Status | Evidence |
|-------------|-------------|--------|----------|
| VAA-01 | Volunteers with active Vacation/SickLeave/UnpaidLeave appear in `sales_person_absences`; Backend delivers Name + hours in DTO | SATISFIED | Structs exist; fill-sites wired; From-impl chain complete; both endpoints populate the field |
| VAA-02 | Hours value = `committed_voluntary` cap-gated (D-53-02 formula) | SATISFIED | Formula verified at lines 501-506 and 864-875; `vaa03_volunteer_with_period_appears_with_correct_hours` asserts hours==5.0 for cap-gated fixture |
| VAA-03 | Backend tests: (1) volunteer with period appears, (2) volunteer without period absent, (3) paid employee unchanged | SATISFIED | All 3 tests (`vaa03_*`) PASS in `service_impl/src/test/booking_information_vaa.rs` |
| VAA-04 | FE renders volunteers visually consistent with paid (no color/icon/suffix); existing rendering line unchanged | SATISFIED (automated) / UNVERIFIED (browser) | grep-lock confirms `page/weekly_overview.rs:126` unchanged; union-test PASS; browser render is human-only |

---

## Regression Lock Status

| Lock | Status | Evidence |
|------|--------|----------|
| `committed_voluntary_hours_defaults_to_zero_when_absent` (Legacy-JSON-Wire-Compat) | GREEN | `cargo test -p rest-types committed_voluntary_hours_defaults_to_zero_when_absent` — 1/1 PASSED |
| `booking_information_vfa` (VFA-01 whole-week-out) | GREEN | 2/2 PASSED |
| `booking_information_chain_c` (Phase 52 assembly) | GREEN | 8/8 PASSED |
| `working_hours_per_sales_person` semantics unchanged | GREEN | Struct fields at `service/src/booking_information.rs:25-35` unchanged; `vaa03_paid_employee_unchanged_regression_lock` PASSED |
| `effective_absence = absence_hours - holiday_hours + unavailable_hours` formula | GREEN | `state/weekly_overview.rs:41-43` confirmed unchanged; `bezahlter_bleibt_via_working_hours_pfad_sichtbar` PASSED |
| VAA-04 rendering-lock `format!("{}: {} {hours_short}", absence.name, format_hours(absence.absence_hours, 2))` | GREEN | Exact 1 match at `page/weekly_overview.rs:126` |

---

## Gate Re-Run Results

| Gate | Command | Result | Status |
|------|---------|--------|--------|
| Backend test suite | `cargo test --workspace` | 732 passed, 0 failed (service_impl); all suites green | PASS |
| Backend clippy | `cargo clippy --workspace -- -D warnings` | exit 0, no warnings | PASS |
| FE tests | `cd shifty-dioxus && cargo test` | 802 passed, 0 failed | PASS |
| FE WASM build | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` | exit 0, no warnings | PASS |
| FE clippy | NOT run | Skipped per Memory `reference_dioxus_clippy_not_gated` (~198 pre-existing lints; excluded from CI; toolchain-split E0514 in dioxus-nix-shell) | SKIP (documented) |
| VAA-03 specific tests | `cargo test -p service_impl booking_information_vaa` | 3/3 PASSED | PASS |
| Legacy wire compat | `cargo test -p rest-types committed_voluntary_hours_defaults_to_zero_when_absent` | 1/1 PASSED | PASS |
| VFA-01 regression | `cargo test -p service_impl booking_information_vfa` | 2/2 PASSED | PASS |
| Phase 52 regression | `cargo test -p service_impl booking_information_chain_c` | 8/8 PASSED | PASS |

---

## Deviation Acceptance

| # | Deviation | Semantic Impact | Accepted |
|---|-----------|-----------------|----------|
| P01-a | Import style: `From<&SalesPersonAbsence> for SalesPersonAbsenceTO` header uses short-form (via top-level `use` import) instead of fully-qualified path specified in Plan acceptance criteria | None — same From-impl, same feature gate, same behavior; mirrors exact byte-identical pattern of neighboring `WorkingHoursPerSalesPerson` impl | YES — cosmetic, consistent with codebase pattern |
| P01-b | Extra test fill-site in `service_impl/src/test/booking_information_weekly_summary_year_batch.rs` discovered and fixed during Wave-1-Gate (compile error under `cargo test --workspace --profile test` not caught by `cargo build --workspace`) | None — only adds `sales_person_absences: Arc::from(Vec::new())` to existing golden-value test helper; WOP-03 bit-pattern assertion unaffected | YES — additive compile fix, no semantic drift |
| P02-a | RED baseline: Only 1 of 3 tests FAILED initially (Tests 2+3 are negative/regression-lock assertions that trivially pass against an empty field; only Test 1 is a positive assertion against the empty default) | None — VAA-03 #2 and #3 correctly designed as regression locks; they fail only if the implementation introduces a bug, not when the field is merely empty | YES — by design; semantically correct regression-lock formulation |
| P02-b | Only 1 WeeklySummary literal in `get_summery_for_week` (Plan/PATTERNS.md referenced a second literal at Zeile 960 that does not exist in the actual code) | None — the single existing literal is correctly filled; D-53-06 fulfilled | YES — plan was over-inclusive; actual code has one path |
| P02-c | Second `sales_person_service.get_all()` call in `get_summery_for_week` eliminated (additive cleanup: `paid_employees` derived from `all_sales_persons` instead of second service call) | None — 1 DAO roundtrip instead of 2; semantically identical | YES — performance improvement, neutral semantic |
| P03 | FE Clippy gate skipped per runtime instruction (Memory: `reference_dioxus_clippy_not_gated` — ~198 pre-existing lints; excluded from CI) | None — the Phase 53 change is idiomatic Rust (Vec::extend + sort_by); no new lints introduced | YES — documented memory-based override; no regressions |

---

## Docs-Freshness-Gate Scan

**Hard triggers per CLAUDE.md (none fired):**

| Trigger File | Changed? | Docs Target |
|-------------|----------|-------------|
| `migrations/sqlite/*.sql` | No | N/A |
| `service_impl/src/permission.rs` | No | N/A |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION` | No (stays 12) | N/A |
| `service_impl/src/reporting.rs` (Balance-Formel) | No | N/A |
| `shifty_bin/src/main.rs` (DI-Verdrahtung) | No | N/A |

**Soft recommendation (INFORMATIONAL — not a gate blocker):**

`docs/features/F03-booking.md` (EN) and `docs/features/F03-booking_de.md` (DE) both describe `WeeklySummaryTO` and its fields. The description at EN:329 / DE:330 currently reads "weekly aggregate with paid, volunteer, committed voluntary hours and per-day capacities" — it does not mention the new `sales_person_absences` field added in Phase 53.

`docs/features/F07-reporting-balance.md` (EN:560) and `_de.md` (DE:558) reference `WeeklySummary` state in the FE but do not describe its fields.

**Assessment:** No hard trigger was fired. The CLAUDE.md trigger table does not list a `rest-types/src/lib.rs` DTO-field extension as a mandatory docs trigger. The CONTEXT.md and RESEARCH.md both concluded this is a soft recommendation. An additive sentence in F03-booking.md (+ _de.md) noting that `WeeklySummaryTO` gained `sales_person_absences: Arc<[SalesPersonAbsenceTO]>` (Phase 53, VAA-01) would be a quality improvement but is **not a hard gate for phase completion**.

**Recommendation:** Add a 1-sentence additive note to F03-booking.md and F03-booking_de.md as a follow-up task (both language versions to stay in sync per CLAUDE.md).

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `service_impl/src/booking_information.rs` | 1017 | Comment references "placeholder" | INFO | Pre-existing comment from Phase 15 documenting intentional `committed_voluntary_hours: 0.0` in single-week variant. Not introduced by Phase 53. Not a stub for Phase 53 work. |

No TBD/FIXME/XXX markers in any files modified by Phase 53. No unreferenced debt markers.

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| VAA-03 #1: volunteer with period appears with correct hours | `cargo test -p service_impl vaa03_volunteer_with_period_appears_with_correct_hours` | PASSED | PASS |
| VAA-03 #2: volunteer without period not in list | `cargo test -p service_impl vaa03_volunteer_without_period_absent_not_in_list` | PASSED | PASS |
| VAA-03 #3: paid employee unchanged regression lock | `cargo test -p service_impl vaa03_paid_employee_unchanged_regression_lock` | PASSED | PASS |
| VAA-04 Union-merge sort order | `cargo test -p shifty-dioxus state::weekly_overview::tests::sales_person_absences_union_merges_paid_and_volunteers_sorted_by_name` | PASSED | PASS |
| Legacy wire compat | `cargo test -p rest-types committed_voluntary_hours_defaults_to_zero_when_absent` | PASSED | PASS |

---

## Human Verification Required

### 1. Browser Rendering: Union-Liste in /weekly_overview/

**Test:** Start backend (port 3000) + FE (port 8080). Open `/weekly_overview/`. Select a year/week where a Freiwilliger has an active Vacation/SickLeave/UnpaidLeave Absence-Period AND a paid employee has `absence_hours > 0`. Inspect the Absencen-row for that week.

**Expected:** Both names appear in a single alphabetically sorted list, format `"{name}: {hours} h"` (e.g. "Anna: 8 h" before "Bob: 5 h" if Anna < Bob case-insensitive). No color difference, no icon, no suffix distinguishing volunteer from paid employee.

**Why human:** Dioxus WASM signal propagation after LoadYear coroutine update cannot be driven headlessly. The Cargo tests prove the mapper (Union-merge + sort correct), but whether the rendered DOM reflects the updated `WeeklySummary.sales_person_absences` vec when the data is loaded from a real backend requires live browser observation (Memory: `reference_dioxus_browser_test_date_inputs`, `reference_dioxus_browser_verify_reports`).

---

## Summary

Phase 53 delivers all 4 required artifacts (VAA-01..04) with full test coverage and clean gates. All 9 programmatically verifiable truths pass. The sole remaining item is a human browser sightcheck of the rendered Union-Liste — the code, wiring, and unit tests are definitively green.

**Blockers:** None.
**Warnings:** FE Clippy skipped (documented, pre-existing 198 lints, excluded from CI). Docs F03-booking.md not updated for new `sales_person_absences` field (soft recommendation, not a hard gate).
**Human action required:** Browser sightcheck per human verification item above before phase-complete.

---

_Verified: 2026-07-06_
_Verifier: Claude (gsd-verifier)_
