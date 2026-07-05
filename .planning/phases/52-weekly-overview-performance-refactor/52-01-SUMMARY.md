---
phase: 52-weekly-overview-performance-refactor
plan: 01
subsystem: [backend, testing, performance]
tags: [tdd, golden-snapshot, regression-guard, byte-identity, latency-baseline]
requires: []
provides:
  - "Golden-Snapshot-Fixture-Test für get_weekly_summary (Wave 1)"
  - "Latency-Baseline vor Refactor (Wave 4 Vergleichs-Referenz)"
affects:
  - service_impl/src/test/booking_information_weekly_summary_year_batch.rs
  - service_impl/src/test/mod.rs
  - .planning/phases/52-weekly-overview-performance-refactor/52-01-latency-baseline.txt
tech_stack_added: []
patterns_used:
  - "IEEE-754 sign-of-zero pinning (Rust f32::to_bits() Golden-Snapshot)"
  - "Fixture-Builder mit FixtureConfig-Struct + HashMap<(year, week), Vec<..>>"
key_files_created:
  - service_impl/src/test/booking_information_weekly_summary_year_batch.rs
  - .planning/phases/52-weekly-overview-performance-refactor/52-01-latency-baseline.txt
key_files_modified:
  - service_impl/src/test/mod.rs
decisions:
  - "D-52-11 (8 Fixture-Achsen) 1:1 umgesetzt — jede Achse ein #[tokio::test]"
  - "D-52-12 (f32::to_bits() byte-identity) mit expliziter NaN-Assertion vor to_bits()"
  - "D-52-13/14 (kein neuer Dep) — reine Rust-Literale, kein proptest, kein insta"
  - "Baseline-Erwartung pinnt IEEE-754 -0.0 für required_hours/volunteer_hours/committed_voluntary_hours (sum-of-empty-iter Semantik in Rust)"
metrics:
  duration_minutes: 30
  completed: 2026-07-05
  tasks_completed: 2
  files_created: 3
  files_modified: 1
status: complete
---

# Phase 52 Plan 01: Golden-Snapshot-Fixture + Latenz-Baseline Summary

**One-liner:** Wave-1-Regressions-Gate für den Weekly-Overview-Performance-Refactor
— 8 byte-identische Fixture-Tests gegen die aktuelle `get_weekly_summary`-Impl
plus Pre-Refactor-Latenz-Baseline (Median 2.33 s auf Dev-DB), damit Waves 2-4
Byte-Identität und Speedup nachweisbar sind.

## Was gebaut wurde

### Task 1 — Fixture-Test (`booking_information_weekly_summary_year_batch.rs`)

8 `#[tokio::test]`-Fixtures decken alle WOP-03-Achsen (D-52-11) ab:

| # | Fixture | Achse | Erwartetes Bit-Muster (in Ziel-Woche) |
| - | ------- | ----- | ------------------------------------ |
| 1 | `fixture_1_baseline` | Leeres Setup, Year=2026, 56 Iterationen (weeks_in_year=53 + Spillover W1..W3 in 2027) | Alle Wochen: `required=-0`, `volunteer=-0`, `committed=-0`, `paid=+0`, `overall=+0` |
| 2 | `fixture_2_holiday_week_n` | Holiday Mo in W31 mit Slot Mo 09:00-17:00 | Slot Holiday-gefiltert → `required=-0` (identisch Baseline) |
| 3 | `fixture_3_shortday_week_n` | ShortDay Mo 14:30 in W31, Slot Mo 14:00-15:00, Gate `active_from=2020-01-01` | W31: `required=0.5` (geclippt), sonst Baseline |
| 4 | `fixture_4_volunteer_vacation_period` | Freiwilliger + Vacation-Absence-Period 2026-07-27..08-02 (überlappt W31) | VFA-01 whole-week-out, ohne Contract kein messbarer Effekt → Baseline |
| 5 | `fixture_5_cvc06_cap_active` | Freiwilliger + Cap-Contract (expected=10, committed=5) + Shiftplan-Report Mo 8h in W31 | Alle Wochen `committed=5, overall=5`; W31 zusätzlich `volunteer=3, overall=8` |
| 6 | `fixture_6_gate_off_legacy` | ShortDay 14:30, Slot 14:00-15:00, `active_from=None` | Chain-C Legacy-Drop: `required=-0` (Slot gedroppt weil slot.to > cutoff) |
| 7 | `fixture_7_gate_on_active_from_before_week` | Identisch Fixture 3, alternative Formulierung des Gates | W31: `required=0.5` |
| 8 | `fixture_8_combined_holiday_shortday_volunteer_cap_gate` | year=2020 (weeks_in_year=53), W53 mit Holiday+ShortDay+Slot+Vacation+CVC-06-Cap + Spillover W55=2021-W2 mit eigenem Slot | Baseline: `committed=5, overall=5`; W53 (idx 52): `required=0.5, volunteer=8, committed=-0, overall=8`; W55 (idx 54): `required=2` |

**Field-Coverage-Matrix (per Fixture):**

| Field | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
| ----- | - | - | - | - | - | - | - | - |
| `year` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `week` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `required_hours` | ✓ | ✓ (Holiday-Filter) | ✓ (Clip) | ✓ | ✓ | ✓ (Legacy-Drop) | ✓ (Clip) | ✓ (Kombi + Spillover) |
| `paid_hours` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `volunteer_hours` (Band 2) | ✓ | ✓ | ✓ | ✓ | ✓ (Surplus) | ✓ | ✓ | ✓ |
| `committed_voluntary_hours` (Band 1) | ✓ | ✓ | ✓ | ✓ | ✓ (Cap) | ✓ | ✓ | ✓ (VFA-01) |
| `overall_available_hours` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `working_hours_per_sales_person` | ✓ (leer) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Weekday-Slots (`monday_available_hours`..`sunday_available_hours`) | ✓ (alle +0) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

Alle Werte werden über `assert_weekly_summary_bit_exact()` verglichen, das pro
`f32`-Feld erst `!is_nan()` assertet und dann `to_bits()` (D-52-12).

### Task 2 — Latency-Baseline

`52-01-latency-baseline.txt` enthält 5 Curl-Runs gegen den lokalen Backend-Server
(Dev-Build, `localdb.sqlite3` 454 KiB, reale User-Daten):

```
1: 1.552791 s
2: 2.266341 s
3: 2.330317 s   ← median
4: 2.979340 s
5: 3.151468 s
Median: 2.330317 s
Backend commit: a88ea80e36200debd9cce3ea51493da37253e1b6
```

Median 2.33 s bestätigt "heute mehrere Sekunden". Zielwert WOP-04 (< 0.5 s)
erfordert ~5× Speedup.

## Byte-Identitäts-Mechanismus

Zwei Helper (`assert_f32_bit_eq`, `assert_whps_bit_eq`) bauen auf einer strikten
Regel auf:

1. **NaN ist verboten:** `assert!(!actual.is_nan())` und `assert!(!expected.is_nan())`
   vor jedem `to_bits()`-Vergleich — NaN-Bit-Muster sind nicht kanonisch (D-52-12
   defensiv).
2. **Feld-für-Feld-Vergleich:** `assert_weekly_summary_bit_exact()` iteriert
   `WeeklySummary` und den inneren `Arc<[WorkingHoursPerSalesPerson]>` einzeln,
   sodass die Fehlermeldung sagt: „`field required_hours` in `summary[0]`
   diverges: `actual=-0 (0x80000000)`, `expected=0 (0x00000000)`" — Debug-Grade
   ist damit deutlich besser als ein Vec-`assert_eq!` (Struct-Debug wäre
   unlesbar bei 56 Wochen).

## Design-Entdeckung: IEEE-754 Sign-of-Zero-Pinning

Beim ersten Test-Run ergab sich, dass die Impl konsistent `-0.0` für
`required_hours`, `volunteer_hours` und `committed_voluntary_hours` liefert,
aber `+0.0` für `paid_hours` und `overall_available_hours`. Ursache:

- `paid_hours` nutzt `let mut paid_hours = 0.0` + `paid_hours += ...` →
  Scalar-Akkumulator startet bei `+0.0`.
- `volunteer_hours` (via `volunteer_surplus_band2`) und
  `committed_voluntary_hours` (via `.sum::<f32>()`) nutzen Iterator-Sums, die
  bei leerem Iterator `-0.0` liefern (in Rust IEEE-754 Standard-Verhalten für
  `f32::sum` mit leerem Input in bestimmten Kontexten).
- `required_hours` = `slots.iter().map(...).sum()` → auch `-0.0` bei leerem
  Slot-Vec.
- `overall_available_hours = committed + volunteer + paid = (-0) + (-0) + 0 = +0.0`
  (IEEE-754: `-0 + 0 = +0`, aber `-0 + -0 = -0`).

Diese Bit-Muster sind **Teil der Golden-Snapshot-Definition**. Wave 2-4 dürfen
diese Muster nicht ändern, sonst ist der Refactor nicht byte-identisch (Wenn
die neue Impl `+0.0` produziert wo alt `-0.0` war → Fixture 1 schlägt fehl →
Alarm, muss inspiziert werden).

Docstring in `empty_summary()` erklärt das für zukünftige Leser.

## Test-Ergebnisse

```
cargo test --package service_impl --lib booking_information_weekly_summary_year_batch
  running 8 tests
  test fixture_1_baseline ... ok
  test fixture_2_holiday_week_n ... ok
  test fixture_3_shortday_week_n ... ok
  test fixture_4_volunteer_vacation_period ... ok
  test fixture_5_cvc06_cap_active ... ok
  test fixture_6_gate_off_legacy ... ok
  test fixture_7_gate_on_active_from_before_week ... ok
  test fixture_8_combined_holiday_shortday_volunteer_cap_gate ... ok
  test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 695 filtered out
```

Vollständiger Workspace-Test-Gate:

```
cargo test --workspace   → all pass (703 + 64 + weitere = alle grün)
cargo clippy --workspace -- -D warnings  → 0 warnings
```

**Bestätigung:** „Alle 8 Fixtures grün gegen aktuelle Impl — sind ab jetzt
hartes Regressions-Gate für Waves 2-4."

## Latenz-Baseline

**Median 2.33 s** auf Dev-DB (`localdb.sqlite3`, 454 KiB, User-Daten).

Runs (sortiert): 1.55, 2.27, 2.33 (Median), 2.98, 3.15 Sekunden.

Backend-Commit zum Messzeitpunkt: `a88ea80` (pre-Task-2, also
pre-Refactor-Baseline).

## Deviations from Plan

- **Endpoint antwortete OHNE Authentication-Header 200 OK** — Dev-Setup hat
  `mock_auth`-Feature aktiv, das den Login-Guard bypasst. Kein Deviation aus
  dem Plan; ist wie in `52-01-PLAN.md` Task-2 dokumentiert ("Mock-Auth aktiv"),
  aber der genaue Impersonation-Mechanismus (`user=admin` via Feature-Flag)
  wurde nicht angerufen — der Endpoint war einfach offen im Dev-Modus.
- **Build-Profil:** dev (`cargo run`), nicht release. Der PLAN.md sagt "Backend
  in Dev-Modus starten" — dev-Build ist damit gerechtfertigt, aber der
  Wave-4-Vergleich MUSS mit demselben Profil laufen (nicht release), sonst
  sind die Zahlen nicht vergleichbar. Explizit im Baseline-File notiert.

Sonst: keine Abweichungen. Plan wurde 1:1 umgesetzt.

## Known Stubs

Keine. Der Test ist eigenständig und benötigt keinen Follow-up-Plan.

## Threat Flags

Keine neuen. STRIDE-Register aus PLAN.md (T-52-01/02, beide `accept`) stimmt
weiter.

## Notes for Wave 2+

- Der Test läuft in der Standard-Suite (`cargo test --package service_impl`),
  kein `#[ignore]`. Wave 2/3/4-Executor werden automatisch die Byte-Identität
  prüfen.
- Beim ersten Test-Fehler in Wave 2+: SEHR wahrscheinlich hat der Refactor die
  IEEE-754-Sign-of-Zero-Semantik geändert. Prüfe zuerst, ob die Divergenz nur
  `+0.0 vs -0.0` ist, bevor du eine echte Regression annimmst.
- Fixture 8 pinnt VFA-01 in Kombination mit CVC-06: Absent volunteer →
  `committed_for_person(sp)=0`, aber `per_day_actuals`-Iterator wird NICHT auf
  absent gefiltert. Das ist heutige Semantik (Zeile 373-389 in
  `booking_information.rs`) und **muss** in Wave 2+ so bleiben, sonst schlägt
  Fixture 8 fehl.

## Self-Check: PASSED

- ✓ `service_impl/src/test/booking_information_weekly_summary_year_batch.rs` existiert
- ✓ `service_impl/src/test/mod.rs` referenziert das neue Modul
- ✓ `.planning/phases/52-weekly-overview-performance-refactor/52-01-latency-baseline.txt` existiert
- ✓ Commit `a88ea80` (fixture test) in git-Log
- ✓ Commit `e4e6f3f` (latency baseline) in git-Log
- ✓ `cargo test --package service_impl --lib booking_information_weekly_summary_year_batch` = 8 passed
- ✓ `cargo test --workspace` = alle passed
- ✓ `cargo clippy --workspace -- -D warnings` = clean
