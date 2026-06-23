---
phase: 15-reporting-no-double-count-snapshot-bump-same-commit
plan: 02
subsystem: api
tags: [booking_information, reporting, committed_voluntary, no-double-count, formula-b, cvc-04, cvc-05, cvc-06, tests, doc-reconciliation]

# Dependency graph
requires:
  - phase: 15-plan-01
    provides: "volunteer_surplus_above_committed(actual, committed) -> f32 pub(crate) helper"
  - phase: 14-data-model-foundation-backend
    provides: "committed_voluntary field + EmployeeWorkDetails"
provides:
  - "D-05 two-band fixture suite (9 tests: cvc04_*, cvc06_*) in service_impl/src/test/booking_information.rs"
  - "snapshot_schema_version_unchanged_at_7 regression test (D-01 / CVC-05)"
  - "ROADMAP Phase-15 reconciled from 'bump 7→8' to no-bump justification"
  - "REQUIREMENTS CVC-05 reconciled from 'bump 7→8' to explicit no-bump justification (Audit-Trail)"
affects: [phase-16-weekly-summary-to-mapping]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pure-function unit test style: epsilon approx() helper (a-b).abs() < 0.001; no f32 == comparisons"
    - "No-double-count invariant encoded per test: band1 + band2 == max(committed, actual)"
    - "Regression test asserting pub const version value stays unchanged (sentinel pattern)"

key-files:
  created:
    - service_impl/src/test/booking_information.rs
  modified:
    - service_impl/src/test/mod.rs
    - service_impl/src/billing_period_report.rs
    - .planning/ROADMAP.md
    - .planning/REQUIREMENTS.md

key-decisions:
  - "D-05 FORMULA B multi-person fixture pinned: committed_voluntary_hours=5.0, volunteer_hours=3.0, grand_total=8.0 (supersedes Formula A=5.0)"
  - "D-01 / CVC-05 no-bump assertion: CURRENT_SNAPSHOT_SCHEMA_VERSION regression test pins version=7; ROADMAP+REQUIREMENTS reconciled"
  - "cap=false fixtures (cvc06_cap_false_zero) assert committed_after_cap_gate=0.0 at helper level with comment documenting upstream cap gating"

patterns-established:
  - "Two-band invariant test pattern: assert both band1 and band2, then assert band1+band2==max(committed,actual)"
  - "backward_compat lock: committed=0 case asserts band2==Σactual==plain_sum"

requirements-completed: [CVC-04, CVC-05, CVC-06]

# Metrics
duration: 25min
completed: 2026-06-24
---

# Phase 15 Plan 02: D-05 Two-Band Fixture Suite + No-Bump Reconciliation Summary

**Maximalabdeckung fur die Zwei-Band-Dekomposition (FORMULA B) via 9 deterministische Fixtures und Regressionstest fur Version 7; ROADMAP und REQUIREMENTS von "bump 7→8" auf explizite no-bump-Begruendung (Audit-Trail) umgestellt.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-06-24T06:00:00Z
- **Completed:** 2026-06-24T06:25:00Z
- **Tasks:** 2 (Task 1 Fixture-Suite + Task 2 No-Bump-Reconciliation)
- **Files modified:** 5

## Accomplishments

### Task 1: D-05 Zwei-Band Fixture Suite

Neues Testmodul `service_impl/src/test/booking_information.rs` mit 10 `#[test]`-Funktionen (9 Fixtures + 1 Regressionstest):

| Test-Name | Band 1 `committed_voluntary_hours` | Band 2 `volunteer_hours` | Pins |
|-----------|-----------------------------------|--------------------------|------|
| `cvc04_band2_surplus` | 5.0 | 2.0 | surplus = max(7-5,0)=2; Invariante band1+band2=max(5,7)=7 |
| `cvc04_band2_pledge_covers` | 5.0 | 0.0 | surplus floored at 0; Invariante band1+band2=max(5,3)=5 |
| `cvc04_boundary_equal` | 5.0 | 0.0 | c==a Boundary → surplus=0; Invariante gepinnt |
| `cvc04_zero_actual` | 5.0 | 0.0 | forward-looking pledge, keine Actuals → Band 2=0 |
| `cvc06_cap_false_zero` | 0.0 | 7.0 | cap=false → committed nach cap-Gate=0; Band 2=volle Actuals |
| `cvc04_multi_week_sum` | 10.0 (5+5) | 2.0 (2+0) | per-Woche-vor-Summe: W1(c=5,a=7)+W2(c=5,a=3) |
| `cvc04_multi_person` | 5.0 | 3.0 (gesamt 8.0) | FORMULA B: A(c=5,a=0) + B(cap=false,a=3) = 5+3=8 |
| `cvc04_paid_capped_band2_zero` | 5.0 | 0.0 | paid Person: actual_vol=0 in Achse B → Band 2=0 |
| `cvc06_committed_zero_backward_compat` | 0.0 | 7.0 (=Σactual) | committed=0 ⇒ Band 2 bit-identisch zu pre-v1.4 |
| `snapshot_schema_version_unchanged_at_7` | — | — | Regressionstest: Version==7 (D-01/CVC-05) |

Alle Fixtures nutzen Epsilon-Vergleich `(a-b).abs() < 0.001` — kein `==` auf f32.

Modul in `service_impl/src/test/mod.rs` als `pub mod booking_information` registriert (alphabetisch nach `absence`).

### Task 2: No-Bump Regressionstest + ROADMAP/REQUIREMENTS Reconciliation

**Regressionstest:** `snapshot_schema_version_unchanged_at_7` assertiert `crate::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION == 7`.

**ROADMAP.md** — Phase-15-Eintraege auf drei Ebenen umgestellt:
- Kurzeintrag (Phases-Liste): Titel von "Reporting no-double-count + snapshot bump (SAME commit)" zu "Reporting no-double-count (Achse B only, KEIN Snapshot-Bump)"; Bump-Klausel durch no-bump-Begruendung ersetzt.
- Detail-Block Goal: "Atomar damit wird 7→8 gebumpt…"-Satz durch no-bump-Begruendung ersetzt (Achse-B-only, kein persistierter value_type).
- SC#3 (Phases-Liste + Detail-Block): "7→8 gebumpt…" ersetzt durch "Version bleibt 7 (KEIN Bump)…Regressionstest…".
- Changelog-Footer: "Snapshot-Bump 7→8 same-commit" zu "Achse-B-only KEIN Snapshot-Bump (D-01 revidiert 2026-06-23)".

**REQUIREMENTS.md** — CVC-05:
- Wording von "wird 7→8 gebumpt — im selben Commit…" zu "(revidiert per D-01 — KEIN Bump): `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 7…no-bump justification, Audit-Trail. Ein Regressionstest pinnt die unveränderte Version 7."

**billing_period_report.rs** — optionaler Kommentar (Step 4):
- Eine Zeile im Versionshistorien-Block: `/// - Phase 15 (committed_voluntary Zwei-Band): KEIN Bump — Achse-B-only, kein persistierter value_type berührt.`

## Task Commits

Kein Commit — jj-managed Repository; User committet manuell.

## Files Created/Modified

- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/test/booking_information.rs` — NEU: 10 Tests (9 D-05-Fixtures + 1 Regressionstest), Epsilon-Vergleiche, no-double-count-Invariante, FORMULA-B-Kommentar, cap-gate-upstream-Kommentar, backward-compat-assertion
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/test/mod.rs` — `pub mod booking_information` hinzugefuegt
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/billing_period_report.rs` — Ein Kommentar-Eintrag fuer Phase 15 im Versionshistorien-Block
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/ROADMAP.md` — Phase-15-Eintraege (Kurzform + Detail-Block Goal + SC#3 + Changelog-Footer) von bump auf no-bump umgestellt
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/REQUIREMENTS.md` — CVC-05 von "bump 7→8" auf no-bump justification (Audit-Trail) umgestellt

## Decisions Made

### FORMULA B Multi-Person Pinning (D-05)

Der kritische `cvc04_multi_person`-Test pinnt explizit die FORMULA-B-Semantik:
- Person A (cap=true, c=5, a=0): Band 1 += 5, Band 2 += surplus(0,5) = 0
- Person B (cap=false → c gated to 0, a=3): Band 1 += 0, Band 2 += surplus(3,0) = 3
- Gesamt: `committed_voluntary_hours = 5.0`, `volunteer_hours = 3.0`, Summe = **8.0**

Kommentar im Test erklaert explizit, warum FORMULA A (max-of-sums = 5) falsch ist und dass die User-Klaerung D-05 FORMULA B (per-person) mandatiert.

### cap=false Gate-Dokumentation (CVC-06)

Der `cvc06_cap_false_zero`-Test empfaengt `committed_after_cap_gate = 0.0` und kommentiert, dass die cap-Filterung upstream in `get_weekly_summary` via `.filter(cap_planned_hours_to_expected)` stattfindet. Auf Helper-Ebene praezentiert eine cap=false-Person immer committed=0.0.

### No-Bump Justification als Code-Kommentar (D-01 / CVC-05)

Der Regressionstest traegt den vollstaendigen Audit-Trail als Kommentar:
- Phase 15 beruehrt keinen persistierten value_type
- WeeklySummary wird von billing_period_report.rs nie konsumiert
- CLAUDE.md "purely additive changes…" Regel trifft zu
- Deshalb kein Bump, Version bleibt 7

## Deviations from Plan

None — Plan ausgefuehrt wie beschrieben. Die Fixture-Tabelle aus dem Plan wurde 1:1 umgesetzt (9 Fixtures + 1 Regressionstest = 10 Tests statt der im Plan genannten 9, weil der Regressionstest als eigenstaendige Funktion zaehlt).

## Issues Encountered

None — alle Tests im ersten Durchlauf gruen.

## User Setup Required

None — reine Test- und Dokumentationsphase, kein DB-Migration, keine externe Konfiguration.

## Next Phase Readiness

- Phase 16: `committed_voluntary_hours: f32` ist auf `WeeklySummary` (Band 1) vorhanden und getestet; `WeeklySummaryTO` + `From`-Mapping + Frontend-Display als naechster Schritt
- Die zwei Baender sind durch 9 Fixtures strukturell verankert; Phase 16 kann darauf aufbauen ohne Semantik-Drift

## Threat Surface Scan

Nur Test-Code und Planungsdoku-Reconciliation. Kein neuer Netzwerk-Endpunkt, keine Auth-Pfade, kein Schema-Change, keine Eingabe-Verarbeitungs-Surface. T-15-03 (no-double-count invariant) durch die Fixtures vollstaendig mitigiert. T-15-04 (Audit-Integritaet) durch den Regressionstest und die ROADMAP/REQUIREMENTS-Reconciliation abgeschlossen.

## Known Stubs

None — dieses Plan produziert nur Tests und Doku-Korrekturen.

## Self-Check

- `grep -c "#[test]" service_impl/src/test/booking_information.rs` → 10: FOUND (>= 9 Bedingung erfuellt)
- `grep -n "fn cvc04_multi_person" ...` → Zeile 163: FOUND; Test assertiert 5.0 (Band 1) + 3.0 (Band 2) + 8.0 (Gesamt)
- `grep -n "fn cvc04_band2_surplus" ...` → Zeile 22: FOUND; assertiert 2.0 surplus
- `grep -n "fn cvc06_committed_zero_backward_compat" ...` → Zeile 219: FOUND; assertiert Band 2 == Σ actual
- `grep -n " == " service_impl/src/test/booking_information.rs | grep -v "//"` → nur Kommentar; kein f32-==
- `grep -n "pub mod booking_information" service_impl/src/test/mod.rs` → Zeile 4: FOUND
- `grep -n "CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 7" service_impl/src/billing_period_report.rs` → Zeile 75: FOUND (bleibt 7)
- `grep -ni "7→8|7->8" .planning/ROADMAP.md` → keine Treffer: CLEAN
- `grep -ni "no-bump|KEIN.*Bump" .planning/REQUIREMENTS.md` → CVC-05 reconciled: FOUND
- `cargo test --workspace` → 435 service_impl tests, 0 failures: PASSED

## Self-Check: PASSED

---
*Phase: 15-reporting-no-double-count-snapshot-bump-same-commit*
*Completed: 2026-06-24*
