# Phase 52 Jahresübergang-Regression — Diagnose

**Datum:** 2026-07-06
**Session:** `.planning/debug/phase-52-jahresuebergang.md`
**Reporter:** User (Test-Env)
**Kernproblem:** KW 1 (Y+1) `paid` **UND** `required` divergieren; KW 53 mindestens `required`.
**Verdikt (kurz):** **REGRESSION durch Phase 52.** Bestätigt für `paid_hours`. `required_hours`-Diff nicht in Isolation reproduziert — braucht ggf. echte Test-Env-Daten.

---

## 1. Zusammenfassung Root Cause

**`ExtraHoursDao::find_by_year(Y)` filtert kalendarisch** (`date_time IN [Y-01-01, (Y+1)-01-01)`), aber der Konsument `ReportingServiceImpl::assemble_weeks` **buckets** die Rows pro `(year, week)` **ISO-basiert** (via `ExtraHours::to_date().as_shifty_week()` → `ShiftyDate::from_date` → `time::Date::to_iso_week_date`).

Am Jahresübergang gehen Rows **verloren**, deren Kalender-Jahr ≠ ISO-Wochenjahr ist:

- **KW 1 (Y)**: enthält ISO-Kalendertage die kalendarisch noch in Y-1 liegen (z.B. 2020-W1 startet Mo 2019-12-30). ExtraHours mit `date_time = 2019-12-30..12-31` sind in `find_by_year(2019)`, nicht in `find_by_year(2020)`. Der In-Memory-Filter würde sie aber der ISO-Woche 2020-W1 zuordnen.
- **KW 53 (Y)** oder **KW 52 (Y)**: enthält ISO-Kalendertage die schon in Y+1 liegen (z.B. 2020-W53 endet So 2021-01-03). ExtraHours mit `date_time = 2021-01-01..01-03` sind in `find_by_year(2021)`, nicht in `find_by_year(2020)`. Der In-Memory-Filter würde sie aber der ISO-Woche 2020-W53 zuordnen.

**Konsequenz für `get_weekly_summary` (der Weekly-Overview-Report):** `paid_hours` in weekly summary wird über `sum(report.dynamic_hours)` aus `reporting_service.get_year(Y)` gefüttert. `dynamic_hours` in `assemble_weeks` (reporting.rs:799) = `weight × planned_hours − abense_hours_for_balance − absence_derived_balance_total`, wobei `abense_hours_for_balance` alle unavailability-kategorien aus ExtraHours summiert (Vacation, SickLeave, Holiday, UnpaidLeave, Unavailable). **Fehlt eine Row am Jahresübergang → `abense_hours` zu niedrig → `dynamic_hours` zu hoch → `paid_hours` in KW 1 / KW 53 zu hoch verglichen mit dem Legacy-`get_summery_for_week`-Pfad (der `find_by_week` verwendet — dieser ist ISO-basiert und findet die Row korrekt).**

---

## 2. Betroffene Dateien & Zeilen

| Datei | Zeilen | Rolle |
|---|---|---|
| `dao_impl_sqlite/src/extra_hours.rs` | `find_by_year` (ca. Z.170-200) | **BUG-QUELLE:** kalendarische Range statt ISO-Wochenjahr |
| `service_impl/src/extra_hours.rs` | `find_by_year` (Z.180-205) | Thin wrapper, keine Semantik |
| `service_impl/src/reporting.rs` | `get_year` (Z.1568-1647), `assemble_weeks` (Z.484-845) | **KONSUMENT:** ruft `find_by_year`, filtert dann per ISO |
| `service_impl/src/booking_information.rs` | `get_weekly_summary` (Z.267-632) | **AUSWIRKUNG:** liest `paid_hours` aus dem verkrüppelten `year_reports` |

## 3. Introducing Commit

- **Commit:** `cd44dc7` — `feat(52-03): add ExtraHoursService::find_by_year (WOP-01)`
- **Vorher:** `find_by_year` existierte **nicht**. Es gab nur `find_by_week`, das ISO-basiert war (und ist).
- Der Kommentar in Z.183 des DAOs sagt explizit *"kalendarisches Jahr, nicht ISO-Woche"* — das war eine **bewusste Design-Wahl** (D-52-06), aber die Wechselwirkung mit dem ISO-basierten Bucket-Filter in `assemble_weeks` wurde nicht bedacht.
- **Verdikt:** **Regression, eingeführt in Phase 52 (Wave 3)**. Weder in v1.x noch in v2.0–v2.4 existierte dieser Pfad.

## 4. Reproduktions-Tests (green als Doku des Bug-Zustands)

- `service_impl/src/test/reporting_year_boundary.rs`:
  - `get_year_vs_get_week_diverges_for_extra_hours_at_iso_kw1_boundary` — ExtraHours am 2019-12-30 mit Kategorie `Vacation`:
    - Legacy `get_week(2020, 1).vacation_hours = 4.5`.
    - Bulk `get_year(2020)[0].1.vacation_hours = 0.0`.
    - **Diskrepanz: 4.5h in KW 1 / 2020.**
  - `get_year_vs_get_week_diverges_for_extra_hours_at_iso_kw53_boundary` — ExtraHours am 2021-01-02 mit Kategorie `Vacation`:
    - Legacy `get_week(2020, 53).vacation_hours = 3.0`.
    - Bulk `get_year(2020)[52].1.vacation_hours = 0.0`.
    - **Diskrepanz: 3.0h in KW 53 / 2020.**
- `service_impl/src/test/booking_information_weekly_summary_year_boundary.rs`:
  - Vier grüne Tests. Prüfen `get_weekly_summary(Y)` vs. `get_summery_for_week(Y, w)` mit **immer aktivem Slot** (valid_to=None) und **kalender-jahr-endendem Slot** (valid_to=2020-12-31). Beide zeigen KEINEN `required_hours`-Diff → Slot-Filter-Semantik ist konsistent zwischen Bulk und Legacy. **Der User-`required_hours`-Diff in KW 53 ist mit diesen Tests NICHT reproduziert.**

Alle Tests sind in service_impl integriert und laufen in <1s.

## 5. Zum User-`required_hours`-Diff (offen)

- Weder mein "always-active"-Slot noch der "valid_to=2020-12-31"-Slot triggern in isolierten Mock-Tests einen Diff.
- Der In-Memory-Slot-Filter in `booking_information.rs:529-548` verwendet dieselbe ISO-Monday/Sunday-Semantik wie die Legacy-DAO `slot.rs:151-168`.
- **Mögliche Trigger, die ich NICHT reproduzieren konnte:**
  1. Slot mit einem `shiftplan_id`, dessen Shiftplan zwischen Bulk-Load (Z.376-383) und Legacy-Aufruf einen `is_planning`-Wechsel hat — theoretisch möglich, praktisch unwahrscheinlich in einer einzelnen `get_weekly_summary`-Response.
  2. SpecialDay mit `year` als **Kalender-Jahr** in der DB gespeichert (Frontend-Bug, kein Backend-Bug) — würde erklären, wenn `find_by_year(Y)` weniger Rows liefert als erwartet und einige Slots deshalb nicht gefiltert werden.
  3. Deleted-SpecialDay-Race: SpecialDay wurde in KW 53 zwischen Bulk-Load und Legacy-Aufruf soft-gelöscht.
  4. **Interaktion mit `shiftplan_report.calendar_week`-Bucket-Filter (In-Memory Z.469-474):** wenn ein Booking (via `shiftplan_report_row`) mit einem `year/calendar_week`-Wert existiert, der aus einer alten pre-ISO-Speicherung stammt (theoretisch), würde er aus der ISO-Woche fallen. Ebenfalls unwahrscheinlich.
- **Empfehlung:** User bitten, im Test-Env für KW 53 die genauen Daten zu zeigen — welche Slots, welche SpecialDays, welche Bookings. Ggf. ein E2E-Sanity-Check mit realem SQLite-Dump gegen einen Repro-Test.

**Trotz nicht-reproduziertem `required_hours`-Diff:** der bestätigte `paid_hours`-Bug ist so schwerwiegend, dass er alleine schon ein Blocker ist.

## 6. Vorgeschlagener Fix (NOCH NICHT IMPLEMENTIERT — wartet auf User-Greenlight)

**Option A (bevorzugt, chirurgisch): DAO-Range erweitern.**

Ändere `ExtraHoursDaoImpl::find_by_year` in `dao_impl_sqlite/src/extra_hours.rs`, sodass die Range **die gesamten ISO-Wochen** abdeckt, die Y berühren:

```rust
// Statt: date_time IN [Y-01-01, (Y+1)-01-01)
// Neu:   date_time IN [ISO-Mo(Y, 1), ISO-Su(Y, weeks_in_year(Y)) + 1 day)
let iso_week_start = time::Date::from_iso_week_date(year as i32, 1, time::Weekday::Monday)?;
let iso_week_end_plus_one = time::Date::from_iso_week_date(
    year as i32,
    time::util::weeks_in_year(year as i32),
    time::Weekday::Sunday,
)?.next_day().unwrap();
// ... format zu ISO-8601 String und SELECT ... WHERE date_time >= ? AND < ? ...
```

Damit ist die Range um **max. 3 Tage vorne und 3 Tage hinten** größer als die kalendarische. Der In-Memory-ISO-Filter in `assemble_weeks` klassifiziert die Rows dann korrekt. **Effektiv keine Performance-Kosten** (max. 6 Extra-Rows pro Jahr Extra-Fetch).

**Bump `snapshot_schema_version` nötig?** NEIN — snapshot_schema_version betrifft nur `billing_period_report.rs::CURRENT_SNAPSHOT_SCHEMA_VERSION` (siehe `shifty-backend/CLAUDE.md`). Der Fix ändert weder eine `value_type` noch die Menge der Inputs, die die Snapshot-Werte produzieren. **Aber:** er ändert **welche ExtraHours-Rows** an Jahresübergängen in den `by_week`-Slice fließen — d.h. bereits geschriebene Snapshots könnten mit dem Fix drift zeigen. **→ Doch: bumpen.** Sicherer Weg (weil die Legacy-Semantik von `find_by_week` bereits ISO-basiert ist, sind Legacy-Snapshots vor Phase 52 nicht betroffen; Snapshots aus Phase-52-Zeit sind fehlerhaft und werden nach dem Fix "richtiger" — das ist genau der Trigger für einen Bump laut CLAUDE.md).

**Option B (fallback): Consumer statt DAO fixen.**

In `assemble_weeks`: statt `find_by_year(Y)` → zusätzlich `find_by_year(Y-1)` **und** `find_by_year(Y+1)` laden, mergen und dann per ISO-Filter bucketen. Nachteil: 3× so viele Rows und Zwei zusätzliche DAO-Roundtrips (statt 1 → 3). Widerspricht dem Performance-Ziel von Phase 52.

**→ Empfehlung: Option A.**

## 7. Zusätzlicher Handlungsbedarf

- **Byte-identity-Fixtures erweitern:** Die 8 Fixtures in `booking_information_weekly_summary_year_batch.rs` decken keinen Jahresübergangs-Fall mit ExtraHours ab (Fixture 8 hat Spillover, aber keine ExtraHours-Rows an der Grenze). Nach dem Fix: Add-Fixture mit ExtraHours an ISO-KW-1-Boundary und ISO-KW-53-Boundary.
- **Docs-Update-Gate:** `docs/features/F07-reporting-balance.md` und `docs/domain/time-accounting.md` sollten die Semantik-Falle dokumentieren.
- **Zwei weitere `find_by_year`-DAOs prüfen — sind sie ISO-konsistent?**
  - `special_day.find_by_year` → **`WHERE year = ?`** auf ISO-Wochenjahr-Feld. ✓ konsistent, kein Fix nötig.
  - `shiftplan_report.extract_raw_shiftplan_report_for_year` → **`WHERE booking.year = ?`** auf ISO-Wochenjahr-Feld. ✓ konsistent, kein Fix nötig.
- **`get_reports_for_all_employees` prüfen (reporting.rs:855+):** benutzt möglicherweise auch `find_by_year` — wenn ja, gleiche Regression. Kurzer Blick unten.

## 8. Regression-Blast-Radius

Alle Callers von `ExtraHoursService::find_by_year`:

- `service_impl/src/reporting.rs::get_year` (Z.1595) — **BETROFFEN**.

Alle Callers von `ReportingService::get_year`:

- `service_impl/src/booking_information.rs::get_weekly_summary` (Z.340, 344) — **BETROFFEN** (via `paid_hours` in weekly summary).
- Test-Konsumenten (nicht production-relevant).

Also: **exakt ein Aufrufer (`booking_information::get_weekly_summary`) ist Sichtbar betroffen**, und zwar auf der Frontend-Seite "Weekly Overview". Alle anderen HR-/Report-Pfade (Employee-Reports, Billing-Period-Snapshots, Balance-Reports) nutzen weiterhin die per-week/`get_report_for_employee`-Pfade und `find_by_week` — **nicht betroffen**.

## 9. Was NICHT vom Bug betroffen ist (Beruhigung)

- `EmployeeReport` / Balance-Reports pro Mitarbeiter (nutzen `get_report_for_employee` → per-week-Aufrufe, ISO-korrekt).
- Billing-Period-Snapshots (nutzen `get_report_for_employee`).
- `get_summery_for_week` (single-week-Legacy-Pfad, ruft `find_by_week` direkt, ISO-korrekt).
- HR-Statistiken (nutzen `get_report_for_employee`).
- Vacation-Balance, Carryover (nutzen `get_report_for_employee` bzw. eigene per-week Aufrufe).

**Der Bug ist auf die Weekly-Overview-Seite (Frontend) beschränkt.**
