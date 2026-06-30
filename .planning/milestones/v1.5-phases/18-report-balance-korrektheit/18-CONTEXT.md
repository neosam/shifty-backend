# Phase 18: Report-/Balance-Korrektheit (Backend) - Context

**Gathered:** 2026-06-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Zwei diagnostizierte Backend-Korrektheits-Bugs in der Urlaubs-/Abwesenheits-Auswertung
werden behoben und testabgesichert. **Reine Backend-Phase** — kein Frontend-Anteil
(die FE-Ansichten lesen die korrigierten Werte unverändert über die bestehende REST-API;
begründeter Skip i.S. der GSD-Scope-Regel).

**UV-04 — Carryover-Abweichung:** `VacationBalanceService.compute_balance`
(`service_impl/src/vacation_balance.rs:221-230`) liest `get_carryover(sales_person_id, year, …)`
— also den Übertrag **aus** `year` heraus (Ende-von-`year`-Snapshot). Korrekt ist der
Übertrag **in** `year` hinein, d.h. der Ende-von-(`year-1`)-Snapshot, wie ihn
`ReportingService.get_report_for_employee` über `get_carryover(*sales_person_id, from_date.year() - 1, …)`
(`service_impl/src/reporting.rs:662-672`) liest. **Report-Service = Wahrheit.**

**UV-05 — Urlaubstage = 0 nach Konvertierung:** Im Detail-Employee-Report
(`get_report_for_employee`) kommt `vacation_days` aus
`by_week.iter().fold(… week.vacation_days())` (`reporting.rs:643-653`). `vacation_days()`
(`service/src/reporting.rs:118-123`) = `vacation_hours / hours_per_day()`. Das per-Woche
`GroupedReportHours.vacation_hours` wird in `hours_per_week` (`reporting.rs:1227-1231`)
**ausschließlich** aus `filtered_extra_hours_list` (extra_hours, Kategorie `Vacation`)
berechnet. Die absence-derived Stunden (`derived_absence`) fließen dort nur in
`absence_hours` (Expected-Reduktion, `reporting.rs:1136-1148`) — **nicht** in die per-Woche
Kategorie-Felder `vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours`. Folge: nach
Konvertierung (extra_hours soft-deleted, statt dessen absence_period) ist per-Woche
`vacation_hours = 0` → `vacation_days = 0`, **obwohl** der Top-Level-Display-Wert
`vacation_hours` korrekt ist (er addiert `absence_derived_vacation_hours` als Jahreslumpen,
`reporting.rs:719-724`). Betroffen identisch: `sick_leave_days`, `absence_days`.

**Nicht in dieser Phase:** Frontend-Änderungen; das Convert-Modal-UX (Phase 19);
Statistik (Phase 22).
</domain>

<decisions>
## Implementation Decisions

### UV-04
- **D-18-01 (Carryover-Quelle angleichen):** `compute_balance` in `vacation_balance.rs`
  liest den Carryover künftig für `year - 1` (Ende-von-Vorjahr-Snapshot), exakt wie
  `reporting.rs:662-672`. Damit stimmen Vacation-Balance-Ansicht und Report-Service überein.
- **D-18-02 (Regressionstest):** Ein Test pinnt die Gleichheit „Vacation-Balance-Carryover
  == Report-Service-Carryover" für denselben Mitarbeiter/Jahr (z.B. Carryover in Jahr N
  wird aus dem Ende-N-1-Snapshot gelesen, nicht aus Ende-N).

### UV-05
- **D-18-03 (derived Absences in per-Woche-Kategorien mergen):** In `hours_per_week`
  (`reporting.rs`) werden die `derived_absence`-Stunden der Woche **nach Kategorie
  aufgeteilt** (Vacation/SickLeave/UnpaidLeave) und in die per-Woche-Felder
  `vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours` addiert — damit
  `vacation_days()`/`sick_leave_days()`/`absence_days()` die konvertierten Absences
  mitzählen. (`ResolvedAbsence.category` liefert die Kategorie, vgl. `reporting.rs:483-489`.)
- **D-18-04 (KEINE Doppelzählung — kritisch):** Wenn die derived Stunden jetzt in die
  per-Woche `vacation_hours` einfließen, summiert `get_report_for_employee` sie bereits
  über `by_week`. Der bestehende **Jahreslumpen-Add** `… + absence_derived_vacation_hours`
  für den Top-Level-Display (`reporting.rs:719-724`, analog sick/unpaid) **muss dann
  entfallen / auf die per-Woche-Summe umgestellt werden**, sonst werden die derived
  Stunden doppelt gezählt. Gleiches gilt symmetrisch für `get_reports_for_all_employees`
  (`reporting.rs:480-509`) — dort prüfen, ob die ShortEmployeeReport-Stunden konsistent
  bleiben. Single source of truth: die per-Woche-Felder.
- **D-18-05 (Gating-Konsistenz):** Die `absence_hours`-Expected-Reduktion ist heute auf
  `working_hours_for_week > 0` gegated (`reporting.rs:1139`). Der **Display-/Tage**-Merge
  der derived Kategorie-Stunden bleibt **ungegated** (analog zur bestehenden ungegate
  Display-Logik, Kommentar `reporting.rs:477-479`), damit auch dynamische/vertraglose
  Wochen ihre Urlaubstage korrekt zeigen. Expected-/Balance-Pfad NICHT verändern (nur
  die Display-/Tage-Felder).
- **D-18-06 (Tests):** Regressionstest „Konvertierung erhält Urlaubstage" — vor Konvert.
  (extra_hours Vacation) und nach Konvert. (absence_period) liefert
  `get_report_for_employee` denselben `vacation_days`-Wert (> 0). Plus ein Test gegen
  Doppelzählung (Stunden bleiben stabil). Analog mind. ein `sick_leave_days`-Fall.

### Snapshot-Versioning
- **D-18-07:** UV-04/UV-05 ändern die **Live-Report-Ableitung** (Carryover-Lesefenster,
  Tage-Display), berühren aber voraussichtlich **keinen persistierten
  `BillingPeriodValueType`**. `CURRENT_SNAPSHOT_SCHEMA_VERSION` daher **nicht** bumpen —
  **außer** der Planner stellt fest, dass eine persistierte value_type-Berechnung im
  Snapshot-Builder mitbetroffen ist (dann gemäß `CLAUDE.md` im selben Commit bumpen).

### Claude's Discretion
- Ob D-18-03 die derived-Aufteilung inline in `hours_per_week` oder via Helper macht.
- Genaue Testdatei-Platzierung (Erweiterung bestehender Reporting-Testmodule vs. neu).
</decisions>

<canonical_refs>
## Canonical References

### Code (verifizierte Stellen)
- `service_impl/src/vacation_balance.rs:221-230` — fehlerhafte `get_carryover(year)`-Quelle (UV-04).
- `service_impl/src/reporting.rs:662-672` — korrekte `get_carryover(year-1)`-Referenz.
- `service_impl/src/reporting.rs:1041-1261` — `hours_per_week` (per-Woche-Builder); `1136-1148` derived→absence_hours; `1227-1246` per-Woche Kategorie-Felder (UV-05).
- `service_impl/src/reporting.rs:643-653` — `vacation_days`/`sick_leave_days`/`absence_days`-Fold.
- `service_impl/src/reporting.rs:719-742` — Top-Level-Display-Lumpen (Doppelzählungs-Falle D-18-04).
- `service_impl/src/reporting.rs:480-509` — `get_reports_for_all_employees` (ShortEmployeeReport-Konsistenz).
- `service/src/reporting.rs:104-144` — `GroupedReportHours::hours_per_day()/vacation_days()/sick_leave_days()/absence_days()`.

### Regeln
- `CLAUDE.md` § Snapshot Versioning, § Service-Tier, § Transactions. jj-only Commits (CLAUDE.local.md).
- Debug-Session `.planning/debug/carryover-absence-vs-report.md` (UV-04 Diagnose-Historie).

### Requirements / Roadmap
- `.planning/REQUIREMENTS.md` — UV-04, UV-05.
- `.planning/ROADMAP.md` § Phase 18 — Goal + Success Criteria 1–4.
</canonical_refs>
