---
status: diagnosed
trigger: "KW 1 (Y+1): paid+required differ zwischen Old/New. KW 53: required differ. User im Test-Env geprüft. Ungewiss: Regression durch Phase 52 oder pre-existing?"
created: 2026-07-06T00:00:00Z
updated: 2026-07-06T00:00:00Z
followup: "sessions/2026-07-06-phase-52-jahresuebergang/DIAGNOSIS-2026-blindsweep.md — required_hours-Bug BESTÄTIGT (SpecialDay-Bucket-Wahl, disjunkt vom ExtraHours-Bug)"
---

## Current Focus

hypothesis: In-Memory-Slot-Filter `slot.valid_from <= sunday_of_week AND (slot.valid_to IS NULL OR slot.valid_to >= monday_of_week)` mit ISO-Wochenjahr-basiertem Monday/Sunday hat ein Off-by-one oder falsches Jahr bei KW 53 / KW 1 im Vergleich zur DAO-Semantik. ODER: `weeks_in_year(outer_year)` deckt KW 53 des Vorjahres im Loop-Iterations-Modell nicht ab.
test: Fixture in einem KW-53-Jahr (2020) mit UNBEGRENZT-gültigem Slot (`valid_from = 2019-01-01`, `valid_to = None`) und Bookings in KW 52, KW 53, KW 1 2021. `get_weekly_summary(2020)` gegen `get_summery_for_week(2020, 52)`, `..(2020,53)`, `..(2021,1)` byte-vergleichen. Weitere Reproduktion: `reporting_service.get_year(2020)[52]` vs. `get_week(2020, 53)`.
expecting: Diff im `required_hours`- (Slot-Filter) oder `paid_hours`- (week_report/get_year-Off-by-one) Feld an KW 53 / KW 1.
next_action: Schreibe integration-Test Fixture in test/booking_information_weekly_summary_year_batch.rs mit realistischem "immer-aktivem" Slot; call `get_weekly_summary(2020)`; parallel Loop 1..53 mit `get_summery_for_week(2020, w)`; vergleiche.

## Symptoms

expected: `get_weekly_summary(Y)[w-1]` (Neu, Phase 52 Bulk) == `get_summery_for_week(Y, w)` (Legacy Per-Week-DAO) für alle w ∈ 1..=weeks_in_year(Y), gleiche `paid_hours` und `required_hours`.
actual: 
  - KW 1 (Y+1): `paid` UND `required` unterscheiden sich Old vs. New.
  - KW 53: mindestens `required` unterscheidet sich.
errors: (keine Fehlermeldung — stille Semantik-Diskrepanz)
reproduction: Test-Umgebung, KW 1 und KW 53 mit User-Auge geprüft.
started: Nach Phase-52-Ship (Commit 831ade4).

## Evidence

- timestamp: 2026-07-06T00:00:00Z
  checked: `time::util::weeks_in_year` semantics
  found: `time_core::util::weeks_in_year(year)` = 52 or 53, ISO-8601-year-basiert. `weeks_in_year(2020)=53`, `weeks_in_year(2021)=52`, `weeks_in_year(2026)=53`.
  implication: Loop `1..=(weeks_in_year(outer_year)+3)` deckt in 2020: 1..=56 Iterationen ab. Für week=54..=56 → (2021, 1..3). Off-by-One-Test-Case verwendbar.

- timestamp: 2026-07-06T00:00:00Z
  checked: `special_day.year`/`calendar_week` DB-Semantik (via `special_days.rs::create` line 140)
  found: `weeks_in_year(special_day.year)` als Validierung → SpecialDay-year ist ISO-Wochenjahr.
  implication: SpecialDay-Filter `d.year == year && d.calendar_week == week` in In-Memory-Bulk-Load-Filter matcht die DAO-`WHERE year=? AND calendar_week=?`-Semantik korrekt.

- timestamp: 2026-07-06T00:00:00Z
  checked: `booking.year`/`booking.calendar_week` DB-Semantik (via ShiftyDate::from_ymd nutzt `to_iso_week_date`)
  found: Alle year/calendar_week-Felder im Codebase sind ISO-Wochenjahr (siehe shifty-utils/src/date_utils.rs:192).
  implication: `shiftplan_report.year/calendar_week` sind ISO-basiert. Der In-Memory-Filter `r.year == year && r.calendar_week == week` matcht die DAO-Semantik.

- timestamp: 2026-07-06T00:00:00Z
  checked: Slot-DAO `get_slots_for_week_all_plans` in `dao_impl_sqlite/src/slot.rs:151-168`
  found: DAO nutzt `Date::from_iso_week_date(year as i32, week, ...)` für Monday/Sunday → ISO-basiert. In-Memory-Filter in `booking_information.rs:529-548` nutzt exakt dieselben Bounds → Semantik matcht strukturell.
  implication: Slot-Filter *sollte* byte-identisch sein. Diskrepanz muss woanders liegen ODER: die Legacy-Filter-DAO-Semantik verhält sich in 2020-W53/2021-W1 anders als vermutet (z.B. Cross-Year-Slots).

- timestamp: 2026-07-06T00:00:00Z
  checked: Fixture-8 (Spillover 2020-W53→2021-W2) im existierenden Test.
  found: Fixture 8 setzt Slots künstlich mit `valid_from = Monday(target_week)` und `valid_to = Some(Sunday(target_week))` (line 466-469) — nur EINE Woche gültig. Kein "immer-gültig"-Slot (valid_to=None, valid_from früher). Auch keine Legacy-Vergleichs-Assertion (nur gegen manuelle Erwartung).
  implication: Fixtures decken den Produktions-Fall "Slot valid_from=2019, valid_to=None, gültig in ALLEN Wochen inkl. KW 53" NICHT ab. Genau hier könnte die Diskrepanz liegen.

- timestamp: 2026-07-06T00:00:00Z
  checked: `get_weekly_summary` Slot-Filter Zeile 529-548 vs. Legacy `get_slots_for_week_all_plans` DAO-Query.
  found: In-Memory-Filter fehlt der `deleted IS NULL`-Check **nicht** (Zeile 535). ABER: der DAO liefert NUR nicht-gelöschte Slots. In `slot_service.get_slots()` (jahresagnostisch) — was liefert der? Deleted enthalten oder nicht?
  implication: **Neue Hypothese A:** Wenn `slot_service.get_slots()` gelöschte Slots liefert (im Gegensatz zum DAO `get_slots_for_week_all_plans` das im DAO-Level schon filtert), kann das für Wochen mit `deleted` Zeitpunkt in der Zielperiode zu Diskrepanzen führen. Aber der Filter auf Zeile 535 fängt das ab, sofern `deleted` das `Option<Datetime>`-Flag ist.
  implication: **Neue Hypothese B:** Was ist mit `slot.shiftplan_id`? Wenn ein Slot einen `shiftplan_id` hat, dessen Shiftplan ist_planning-Flag zwischen Y und Y+1 wechselt (unwahrscheinlich, aber theoretisch), kann der Filter Zeile 542-546 der Legacy-DAO widersprechen. Legacy-DAO liest den aktuellen Zustand, Bulk-Load auch — hier sollte kein Drift sein.

- timestamp: 2026-07-06T00:00:00Z
  checked: Zeile 386-391 Loop-Reindex
  found: `for week in 1..=(weeks_in_year + 3) { let (year, week) = if week > weeks_in_year { (outer_year+1, week - weeks_in_year) } else { (outer_year, week) }; ... }`. Für outer_year=2020 (weeks_in_year=53): week=1..53 → (2020, 1..53). week=54..56 → (2021, 1..3). ✓
  implication: Loop-Reindex ist korrekt für den KW-53-Jahr-Fall.

## Eliminated

- hypothesis: `weeks_in_year` liefert falschen Wert für KW-53-Jahre.
  evidence: `time_core::util::weeks_in_year` implementiert ISO-8601 Match-Table korrekt (verified in `/nix/store/687cpwyvpzh1gl1dc8ypdx87k6mwsbxw-time-core-0.1.9/src/util.rs`).
  timestamp: 2026-07-06

- hypothesis: `SpecialDay.year` ist Kalender-Jahr (nicht ISO).
  evidence: `special_days.rs::create` line 140 validiert gegen `weeks_in_year(special_day.year as i32)` → ISO-Jahr-Semantik.
  timestamp: 2026-07-06

## Resolution

root_cause: |
  `ExtraHoursDao::find_by_year(Y)` in `dao_impl_sqlite/src/extra_hours.rs`
  filtert kalendarisch (`date_time IN [Y-01-01, (Y+1)-01-01)`), aber der Konsument
  `ReportingServiceImpl::assemble_weeks` buckets die Rows per ISO-Wochenjahr
  (`ExtraHours::to_date().as_shifty_week()`). Am Jahresübergang gehen Rows
  verloren, deren Kalender-Jahr ≠ ISO-Wochenjahr ist — z.B. eine ExtraHours-Row
  vom 2019-12-30 gehört ISO-technisch in KW 1 / 2020, wird aber von
  find_by_year(2020) nicht geliefert (nur von find_by_year(2019)).

  Konsequenz für weekly_summary: `dynamic_hours` in `assemble_weeks` (Z.799)
  subtrahiert `abense_hours_for_balance` (Vacation/SickLeave/Holiday/UnpaidLeave/
  Unavailable). Fehlt eine Row → Subtraktion fällt aus → `dynamic_hours` zu hoch
  → `paid_hours` in weekly_summary KW1/KW53 zu hoch vs. Legacy `get_summery_for_week`.

  Regression, eingeführt in Commit cd44dc7 (Phase 52 Plan 03 WOP-01).
  Vor Phase 52 gab es kein `find_by_year` — nur `find_by_week` (ISO-basiert).

  required_hours-Diff in KW 53 (User-Report): NICHT in isolierten Tests
  reproduziert. Slot-Filter-Semantik ist konsistent zwischen Bulk und Legacy.
  Braucht ggf. echten Test-Env-Datenblick oder ist Sekundär-Effekt einer
  anderen Datenanomalie.
fix: |
  Bevorzugt: DAO-Range in extra_hours.rs::find_by_year erweitern auf
  ISO-Wochenjahr-Bereich statt Kalender-Jahr. Nur ±3 Tage extra pro Jahr.
  NOCH NICHT IMPLEMENTIERT — wartet auf User-Greenlight.
  Details siehe `.planning/debug/sessions/2026-07-06-phase-52-jahresuebergang/DIAGNOSIS.md`.
verification: |
  Reproduktions-Tests in service_impl/src/test/reporting_year_boundary.rs
  (2 Tests green, dokumentieren den Bug-Zustand).
  Cross-Check-Tests in service_impl/src/test/booking_information_weekly_summary_year_boundary.rs
  (4 Tests green, prüfen Slot-Filter-Konsistenz — kein Bug an der Slot-Ebene).
files_changed:
  - service_impl/src/test/reporting_year_boundary.rs (neu)
  - service_impl/src/test/booking_information_weekly_summary_year_boundary.rs (neu)
  - service_impl/src/test/mod.rs (module registrations)
  - .planning/debug/sessions/2026-07-06-phase-52-jahresuebergang/DIAGNOSIS.md (neu)
