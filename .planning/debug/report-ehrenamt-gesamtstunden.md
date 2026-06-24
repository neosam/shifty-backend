---
slug: report-ehrenamt-gesamtstunden
status: resolved
trigger: "Im detaillierten Report der Employees-Seite scheint Ehrenamt (committed_voluntary) in die Gesamtstunden eingerechnet zu werden; die gedeckelten Stunden der Mitarbeiterlisten-Zusammenfassung passen dadurch mit dem Report zusammen."
created: 2026-06-24
updated: 2026-06-24
---

# Debug Session: Report rechnet Ehrenamt in Gesamtstunden ein

## Symptoms

- **Expected behavior:** Ehrenamt (committed_voluntary) darf NICHT in die Gesamtstunden
  des detaillierten Reports einfließen. Gesamtstunden = bezahlte Arbeit + relevante
  Extra-Stunden, ohne committed_voluntary.
- **Actual behavior:** Auf der Employees-Seite stimmen die (gedeckelten) Stunden der
  Mitarbeiterlisten-Zusammenfassung mit den Stunden im detaillierten Report überein.
  Es sieht so aus, als ob im Report Ehrenamt in die Gesamtstunden eingerechnet wird.
- **Error messages:** Keine — Logik-/Berechnungsdiskrepanz, kein Crash.
- **Timeline:** Vermutlich seit der Phase-17-CVC-Arbeit (committed_voluntary contribution).
  Relevante jüngste Commits: 3f3dc44 (committed_voluntary read-gate auf
  expected_hours==0), 2d24074 (paid-only work-details gate), CVC-09/CVC-10.
- **Reproduction:** Employees-Seite öffnen, Mitarbeiter mit committed_voluntary-Stunden
  (Ehrenamt) betrachten; Zusammenfassung in der Mitarbeiterliste mit detailliertem
  Report vergleichen, insbesondere bei gedeckelten ("gedeckelt") Stunden.

## Current Focus

- hypothesis: Im detaillierten Report (`get_report_for_employee_range`,
  reporting.rs:502) wird `overall_hours` aus dem ROHEN, UNGEDECKELTEN
  `shiftplan_hours` (Z.577-587) berechnet (`overall_hours = shiftplan_hours +
  overall_extra_work_hours`, Z.659). Der per-Woche GEDECKELTE Wert
  `shiftplan_hours_by_week` (Z.605, via `apply_weekly_cap`) wird berechnet, aber
  nur geloggt — NICHT für overall/balance genutzt. Folge: bei
  `cap_planned_hours_to_expected=true` fließt der Cap-Überlauf
  (= auto_volunteer/Ehrenamt-Anteil) fälschlich in overall_hours UND balance ein.
  Die Summary (`get_reports_for_all_employees`, Z.454) nutzt dagegen den
  gedeckelten `weekly_hours.shiftplan_hours`.
- test: cargo-Test, der für einen cap=true-Mitarbeiter mit shiftplan > expected
  prüft, dass detail-report.overall_hours == expected (gedeckelt) und der Überlauf
  in volunteer_hours landet — Vergleich mit get_reports_for_all_employees.
- expecting: detail-overall_hours == raw shiftplan (zu hoch) BEWEIST Bug;
  summary-overall_hours == capped (korrekt) zeigt die Diskrepanz.
- next_action: Fix anwenden — overall_hours/balance_hours/shiftplan_hours auf den
  gedeckelten Wert (shiftplan_hours_by_week) umstellen.
- reasoning_checkpoint:
    hypothesis: "get_report_for_employee_range nutzt rohes ungedeckeltes
      shiftplan_hours (Z.577) fuer overall_hours/balance/shiftplan_hours, statt den
      bereits berechneten gedeckelten shiftplan_hours_by_week (Z.605). Bei cap=true
      leakt der Cap-Ueberlauf (auto_volunteer) in die Gesamtstunden."
    confirming_evidence:
      - "cargo-Test reporting_cap_overflow::capped_overflow_does_not_leak_into_overall_hours:
        overall_hours == 50 (roh), erwartet 40 (gedeckelt). RED beweist Leak."
      - "Negativ-Kontrolle uncapped_overflow_stays_in_overall_hours: PASS."
      - "get_reports_for_all_employees (Summary, Z.454) nutzt den GEDECKELTEN
        weekly_hours.shiftplan_hours -> Detail muss konsistent sein."
    falsification_test: "Wenn overall_hours bei cap=true bereits == expected waere,
      gaebe es kein Leak und der Test waere GREEN ohne Fix."
    fix_rationale: "Root cause = Detail-Report liest den falschen (rohen) shiftplan-
      Wert. Fix stellt overall/balance/shiftplan_hours auf shiftplan_hours_by_week um
      (Wert existiert bereits, wurde nur geloggt). Cap-Ueberlauf landet bereits
      korrekt in volunteer_hours (by_week, Z.693). Adressiert Ursache, nicht Symptom."
    blind_spots: "Multi-Wochen-Ranges + dynamische Vertraege (expected==0) — by_week
      behandelt diese pro Woche; Regressionstests (reporting_additive_merge,
      billing_period_report) muessen gruen bleiben."

## Evidence

- timestamp: 2026-06-24
  checked: committed_voluntary-Konsumenten im Backend (grep)
  found: `committed_voluntary_for_calendar_week` (reporting.rs:101) wird NUR in
    booking_information.rs konsumiert (Achse B / Verfügbarkeit), NICHT in
    reporting.rs für overall/balance.
  implication: committed_voluntary fließt nicht DIREKT in overall_hours — der Leak
    muss über die Cap-/auto_volunteer-Mechanik kommen. Deckt sich mit Phase-15
    D-01 (committed_voluntary = reine Achse-B-Kapazität, NICHT Achse A/reporting).

- timestamp: 2026-06-24
  checked: get_report_for_employee_range (reporting.rs:502-707)
  found: Zwei shiftplan-Werte. Z.577 `shiftplan_hours` = ROH ungedeckelt; Z.605
    `shiftplan_hours_by_week` = gedeckelt (apply_weekly_cap). overall_hours (Z.659)
    + balance_hours (Z.657) nutzen den ROHEN Wert; shiftplan_hours_by_week wird nur
    geloggt (Z.606).
  implication: Cap-Überlauf (auto_volunteer) leakt in overall_hours/balance des
    Detail-Reports. volunteer_hours (Z.693) zählt korrekt by_week.

- timestamp: 2026-06-24
  checked: get_reports_for_all_employees (reporting.rs:454)
  found: overall_hours = weekly_hours.shiftplan_hours (per-Woche GEDECKELT via
    apply_weekly_cap, Z.289-290 + fold Z.411).
  implication: Summary deckelt korrekt, Detail-Report nicht → Inkonsistenz.

## Eliminated

## Resolution

- root_cause: Der detaillierte Employee-Report (`get_report_for_employee_range` in
  `service_impl/src/reporting.rs`) berechnete `overall_hours`, `balance_hours` und
  `shiftplan_hours` aus dem ROHEN, UNGEDECKELTEN shiftplan-Summenwert (alte Z.577).
  Der per-Woche GEDECKELTE Wert `shiftplan_hours_by_week` (via `apply_weekly_cap`)
  existierte bereits, wurde aber nur geloggt. Folge: bei
  `cap_planned_hours_to_expected = true` floss der Cap-Ueberlauf (= auto_volunteer /
  Ehrenamt-Anteil) faelschlich in die Gesamtstunden + Balance des Detail-Reports ein.
  Die Summary (`get_reports_for_all_employees`) nutzte dagegen schon den gedeckelten
  Wert → Inkonsistenz. committed_voluntary selbst fliesst NICHT direkt ein (Phase-15
  D-01: reine Achse-B-Groesse); der Leak kam ausschliesslich ueber die Cap-Mechanik.
- fix: `overall_hours`, `balance_hours` und `shiftplan_hours` im EmployeeReport auf
  `shiftplan_hours_by_week` (gedeckelt) umgestellt; das rohe `shiftplan_hours` entfernt.
  Cap-Ueberlauf zaehlt weiterhin korrekt in `volunteer_hours` (by_week). Frontend
  unveraendert — es mappt den korrigierten Backend-Wert 1:1 (state/employee.rs:359).
- verification: cargo test -p service_impl reporting (40 passed inkl. neuer
  Regressionstest + Negativ-Kontrolle); cargo test -p service_impl (447 passed);
  cargo test --workspace (alle gruen, inkl. billing_period_report-Mocks);
  cargo build --workspace ohne Warnings.
  BROWSER-VERIFIKATION (2026-06-24): Tom Bauer (cap=1) testweise expected_hours
  10->4 gesenkt (7h gebucht KW21 -> 3h Ueberlauf). Altes Backend (vor Fix):
  Actual=7, Shiftplan=7, Volunteer=3, Balance=-617 (Ueberlauf leakt doppelt).
  Neues Backend (mit Fix): Actual=4, Shiftplan=4, Volunteer=3, Balance=-620
  (gedeckelt, Ueberlauf nur noch in Volunteer). expected_hours danach auf 10
  zurueckgesetzt; Dev-DB im Ausgangszustand.
- files_changed:
    - service_impl/src/reporting.rs (overall/balance/shiftplan_hours -> gedeckelt)
    - service_impl/src/test/reporting_cap_overflow.rs (neuer Regressionstest)
    - service_impl/src/test/mod.rs (Test-Modul registriert)
