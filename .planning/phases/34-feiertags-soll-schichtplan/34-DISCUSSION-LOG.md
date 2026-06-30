# Phase 34: Feiertags-Soll im Schichtplan - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-30
**Phase:** 34-feiertags-soll-schichtplan
**Areas discussed:** Welches „Soll" sinkt, Feiertag sichtbar machen, HOL-03-Test neu fassen, Snapshot-Version

---

## Welches „Soll" reduziert wird (HSP-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Nur get_week / per-Mitarbeiter | get_week.expected_hours → WorkingHoursPerSalesPerson.available_hours + holiday_hours; per-Tag-Zeile unberührt; minimaler requirement-treuer Eingriff | ✓ |
| Auch die per-Tag-Zeile (Mo–So) | Zusätzlich booking_information.rs:517 erweitern; mehr Aufwand, HSP-03-Risiko | |
| Erst klären wo es steht | Im laufenden FE ansehen, welche Zahl der Nutzer als „Soll" wahrnimmt | |

**User's choice:** Nur get_week / per-Mitarbeiter.
**Notes:** „Es ist schon im gesamten Report drin aber hier fehlt es noch bzw wurde aus
Versehen so umgesetzt." → Phase 34 = gezielter Konsistenz-Fix einer Auslassung in get_week.

---

## Sichtbarkeit der Feiertagsstunden (HSP-02)

| Option | Description | Selected |
|--------|-------------|----------|
| BE-only: Feld füllen reicht | holiday_hours-DTO korrekt + Soll sinkt; keine FE-Spalte, keine i18n; Test sichert ab | ✓ |
| FE: sichtbarer Feiertags-Indikator | Eigene Spalte/Badge/Tooltip; zieht FE-Scope + i18n (de/en/cs), Phase wäre (BE+FE) | |
| Später / Future | Indikator als deferred idea notieren | |

**User's choice:** BE-only: Feld füllen reicht.
**Notes:** FE rechnet Feiertage bereits aus der Absence-Anzeige heraus
(weekly_overview.rs:52) → kein FE-Regress bei nicht-null holiday_hours.

---

## HOL-03-Regressionstest neu fassen (HSP-03 / HSP-04)

| Option | Description | Selected |
|--------|-------------|----------|
| Umbauen: Band-Guard + neue Soll-Asserts | Mocks setzen; dynamic_hours==40 (Band-Guard) + expected_hours 32 + holiday_hours 8; plus separater vor-Stichtag-Test | ✓ |
| Splitten in zwei Tests | Alter Test = reiner Band-Guard + neuer Test für expected/holiday | |
| Du entscheidest beim Planen | Grobe Richtung festhalten, exakte Struktur dem Planner überlassen | |

**User's choice:** Umbauen: Band-Guard + neue Soll-Asserts.
**Notes:** Separater Test für „Feiertag vor Stichtag → keine Wirkung" + manual-wins ohne
Doppelzählung (Wiederverwendung build_derived_holiday_map).

---

## Snapshot-Schema-Version

| Option | Description | Selected |
|--------|-------------|----------|
| Kein Bump + im Plan verifizieren | Bleibt 12; Snapshots lesen reporting.rs-Pfad, nicht get_week/booking_information; Plan-Task grep-Verifikation | ✓ |
| Sicherheitshalber bumpen | 12→13 vorsorglich; nach Code-Befund unnötig | |

**User's choice:** Kein Bump + im Plan verifizieren.
**Notes:** Default-Erwartung bleibt 12; Verifikation per grep/Lese-Check als Pflicht-Plan-Task.

---

## Claude's Discretion

- Exakte Form/Stelle des 4. Injektionspunkts in get_week (eigener holiday_derived-Term
  vs. Einbau in den bestehenden absence/expected-Reduktionspfad), unter den in CONTEXT.md
  genannten Invarianten (a–d).
- Genaue Fixture-/Test-Struktur im Rahmen von D-34-03.

## Deferred Ideas

- Sichtbarer Feiertags-Indikator/Spalte/Tooltip in der Schichtplan-Tabelle (deckt sich mit
  bereits deferred „Hover-Tooltip auf Feiertags-Zelle", REQUIREMENTS.md §Future).
- Feiertags-Berücksichtigung der per-Tag-Aggregat-Zeile (Mo–So) — eigene Entscheidung
  gegen HSP-03, nicht Teil dieser Phase.
