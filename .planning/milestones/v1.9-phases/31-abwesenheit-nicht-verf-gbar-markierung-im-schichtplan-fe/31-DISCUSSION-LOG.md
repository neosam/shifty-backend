# Phase 31: Abwesenheit → Nicht-Verfügbar-Markierung (FE) - Discussion Log

> **Audit trail only.** Decisions are captured in CONTEXT.md.

**Date:** 2026-06-29
**Phase:** 31-Abwesenheit → Nicht-Verfügbar-Markierung im Schichtplan (FE)
**Areas discussed:** Kategorie-Whitelist, Personen-Scope

---

## Kategorie-Whitelist (D-NN → D-31-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Alle 3, exakt wie die Buchungs-Warnung | Vacation + SickLeave + UnpaidLeave, Halbtags ausgenommen — identisch zu BookingOnAbsenceDay (cross-Kategorie, nur Ganztags). Null Drift (SC2). | ✓ |
| Nur Vacation | Nur Urlaub; erzeugt Drift zur Warnung (SC2-Verletzung). | |
| Alle 3, auch Halbtags | Strenger als die Warnung (die Halbtags toleriert) → umgekehrter Drift. | |

**User's choice:** Alle 3, exakt wie die Buchungs-Warnung (→ D-31-01)
**Notes:** Code-Scout bestätigte: `shiftplan_edit.rs:530-545` warnt cross-Kategorie, einziger
Filter `day_fraction == Half`. Der Marker spiegelt exakt dieses Set.

---

## Personen-Scope (D-31-02)

| Option | Description | Selected |
|--------|-------------|----------|
| Nur eigene Person (aktueller User) | per-current-user, analog reload_unavailable_days. | (frei präzisiert) |
| Alle Personen mit Urlaub in der Woche | größeres Feature, weicht von SC1 ab. | |

**User's choice (free text):** „Das soll sich EXAKT so verhalten wie wenn jemand einen
Unavailable Tag hinterlegt hat. Wenn der Schichtplaner die Person auswählt oder die Person
selber soll die Markierung sehen." → D-31-02: Scope = `current_sales_person` (die im
Schichtplan gewählte/aktive Person; Default eingeloggter User, per Dropdown umstellbar) —
exakte Spiegelung des bestehenden `unavailable_days`-Markers.
**Notes:** Code-Scout bestätigte `current_sales_person` (`shiftplan.rs:199/349/538/838`) als
die EINE Quell-Person, die heute schon `unavailable_days` treibt → der Absence-Marker hängt
sich an dieselbe Person.

## Claude's Discretion
- Exakter Per-Woche-Absence-Loader/Endpoint (`/absence-period`, bestehender Aggregat in
  booking_information.rs); pure-Helfer-Signatur; wo der 3-Kategorie-+-Ganztags-Filter sitzt
  (eine Stelle, testbar).

## Deferred Ideas
- Hover-Tooltip auf discourage-Zelle (Future, REQUIREMENTS.md).
- „Alle Personen der Woche"-Scope (verworfen).
