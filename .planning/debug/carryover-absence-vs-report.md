---
slug: carryover-absence-vs-report
status: awaiting_human_verify
trigger: "Der Urlaub, der vom Vorjahr übertragen wird (Carryover), stimmt in den Abwesenheitszeiträumen (Absence Periods) überhaupt nicht mit dem Report-Service überein. Diese beiden Werte MÜSSEN identisch sein."
created: 2026-06-13
updated: 2026-06-13
---

# Debug Session: carryover-absence-vs-report

## Symptoms

- **Expected:** Der vom Vorjahr übertragene Urlaub (Carryover) in der Abwesenheitszeiträume-Ansicht (Absence Periods) ist identisch mit dem Carryover-Urlaub, den der Report-Service berechnet.
- **Actual:** Die beiden Werte weichen voneinander ab — durchgängig bei JEDEM Mitarbeiter auf der INT-Umgebung, kein Einzelfall.
- **Source of truth:** Der Report-Service (ReportingService) ist korrekt. Die Absence-Periods-Ansicht soll an dessen Berechnung angeglichen werden.
- **Error messages:** Keine — es ist eine stille Wertabweichung, kein Crash/Error.
- **Timeline:** Reproduzierbar auf INT für alle Mitarbeiter (kein bekannter Auslöser-Commit genannt).
- **Reproduction:** Carryover-Urlaub eines beliebigen Mitarbeiters in der Absence-Periods-Ansicht mit dem Wert im Report-Service vergleichen → sie stimmen nicht überein.

## Goal

Root-Cause der Abweichung finden, beide Berechnungspfade angleichen (Report-Service = Wahrheit) und mit Tests absichern, dass der Carryover-Urlaub in beiden Pfaden identisch ist.

## Evidence

- timestamp: 2026-06-13
  checked: Frontend Absence Periods view (absences.rs) + backend service wiring
  found: Das Frontend zeigt den Carryover aus `VACATION_BALANCE_STORE` (geladen über `VacationBalanceAction::LoadSelf/LoadTeam`), das von `/vacation-balance/{sp_id}/{year}` bzw. `/vacation-balance/team/{year}` kommt.
  implication: Der Unterschied liegt im Backend — `VacationBalanceService` vs `ReportingService`.

- timestamp: 2026-06-13
  checked: VacationBalanceService.compute_balance (service_impl/src/vacation_balance.rs Z. 221-230)
  found: `get_carryover(sales_person_id, year, ...)` — verwendet `year` direkt.
  implication: Liest das Ende-von-`year`-Snapshot (Übertrag in nächstes Jahr), NICHT den Übertrag in `year`.

- timestamp: 2026-06-13
  checked: ReportingService.get_employee (service_impl/src/reporting.rs Z. 603-616)
  found: `get_carryover(*sales_person_id, from_date.year() - 1, ...)` — verwendet `year - 1`.
  implication: Liest korrekt den Ende-von-(year-1)-Snapshot (Übertrag eingehend in `year`).

- timestamp: 2026-06-13
  checked: ShiftplanEditService.update_carryover (service_impl/src/shiftplan_edit.rs Z. 228-236)
  found: Schreibt `Carryover { year, carryover_hours, vacation }` nach Berechnung des Reports für `year`. Ein Carryover mit `year = Y` = Ende-von-Jahr-Y-Saldo (geht in Jahr Y+1 ein).
  implication: Die year-Semantik ist klar: year=Y → Saldo am Ende von Y → Übertrag in Y+1. Um den EINGEHENDEN Übertrag für Jahr Y zu lesen, muss man get_carryover(sp, Y-1) aufrufen.

## Eliminated

- hypothesis: Fehler liegt im Frontend (falsche API-Route oder State-Mapping).
  evidence: Beide Pfade (Report und Absence/VacationBalance) nutzen denselben CarryoverService. Das Problem liegt ausschließlich im year-Parameter beim Aufruf von get_carryover im VacationBalanceService.
  timestamp: 2026-06-13

## Current Focus

hypothesis: VacationBalanceService.compute_balance ruft get_carryover(sp_id, year) auf — soll aber get_carryover(sp_id, year - 1) aufrufen, um den eingehenden Übertrag aus dem Vorjahr zu erhalten. ReportingService macht es korrekt mit year - 1.
test: Quellcode beider Pfade gelesen + Carryover-Schreib-Semantik in shiftplan_edit.rs verifiziert.
expecting: Fix in vacation_balance.rs: year → year - 1 beim get_carryover-Aufruf.
next_action: Fix applied, Tests updated und ergänzt, cargo build + cargo test laufen grün.
reasoning_checkpoint:
  hypothesis: "VacationBalanceService liest get_carryover(sp_id, year) statt get_carryover(sp_id, year - 1)"
  confirming_evidence:
    - "ReportingService Z.603-616 verwendet from_date.year() - 1"
    - "ShiftplanEditService schreibt Carryover.year = year (Jahr des berechneten Reports), d.h. year=2024 → Übertrag in 2025"
    - "Abweichung bei JEDEM Mitarbeiter → systematischer Fehler, kein Einzelfall"
  falsification_test: "Nach Änderung zu year-1 stimmen beide Werte überein; bei year stimmen sie nicht überein"
  fix_rationale: "Direktes Anpassen des year-Parameters von `year` zu `year - 1` macht beide Pfade identisch"
  blind_spots: "Mögliche Jahr-0-Unterlauf-Probleme bei year=0 — in der Praxis unwahrscheinlich, da kein Jahr-0 existiert"

## Resolution

root_cause: VacationBalanceService.compute_balance (service_impl/src/vacation_balance.rs) rief `get_carryover(sales_person_id, year, ...)` auf statt `get_carryover(sales_person_id, year - 1, ...)`. Ein Carryover-Eintrag mit year=Y speichert den Ende-von-Jahr-Y-Saldo, der in Jahr Y+1 eingebracht wird. Damit wurde der Übertrag des AKTUELLEN Jahres (→ ins nächste Jahr) gelesen statt des Übertrags des VORJAHRES (→ ins aktuelle Jahr). Der ReportingService verwendete korrekt year - 1.

fix: In service_impl/src/vacation_balance.rs Z. 225 den year-Parameter von `year` auf `year - 1` geändert. Docblock korrigiert. Zwei neue Tests ergänzt in service_impl/src/test/vacation_balance.rs:
  - `get_carryover_is_called_with_previous_year`: prüft über mockall::with-Expectation, dass year-1 übergeben wird.
  - `carryover_from_previous_year_is_included_in_balance`: prüft, dass die Carryover-Row für year-1 korrekt in die Balance eingerechnet wird.

verification: cargo build (clean), cargo test (alle 61 Tests bestanden, davon 13 vacation_balance-Tests).

files_changed:
  - service_impl/src/vacation_balance.rs
  - service_impl/src/test/vacation_balance.rs
