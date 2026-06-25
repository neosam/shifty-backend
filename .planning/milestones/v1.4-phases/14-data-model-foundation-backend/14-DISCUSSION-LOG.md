# Phase 14: Data-model foundation (backend) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-22
**Phase:** 14-data-model-foundation-backend
**Areas discussed:** Overlap-Aggregation (D-OVERLAP-AGG)

---

## Overlap-Aggregation (D-OVERLAP-AGG)

Wie soll `committed_voluntary` aggregiert werden, wenn zwei überlappende aktive `EmployeeWorkDetails`-Rows in derselben ISO-Woche liegen (gelesen über dieselbe `find_working_hours_for_calendar_week`-Selektion wie `expected_hours`)?

| Option | Description | Selected |
|--------|-------------|----------|
| SUM (Empfohlen) | Konsistent mit dem `expected_hours`-Präzedenzfall (gleicher `.fold(acc + a)`-Pfad, gleiche Selektion). Zwei Zusagen addieren sich (5h + 5h → 10h). Folgt dem line-für-line-Prinzip der Phase; minimaler Sonderfall-Code. | ✓ |
| MAX | Defensiv: nimmt die größere Zusage (5h + 5h → 5h). Verhindert anomalie-bedingte Kapazitäts-Aufblähung, weicht aber vom `expected_hours`-Pfad ab (eigener Aggregations-Code). | |
| FIRST | Nimmt die erste matchende Row. Reihenfolge-abhängig, am wenigsten vorhersehbar für numerische Kapazität. | |

**User's choice:** SUM
**Notes:** Konsistenz mit dem bestehenden `expected_hours`-`.fold`-Pfad (`reporting.rs:240-254`) war ausschlaggebend. Das Boolean-`.any()`-Pattern des Cap-Flags generalisiert nicht auf einen numerischen Wert und wird explizit nicht kopiert. Semantik wird per Unit-Test gepinnt, obwohl das Feld in Phase 14 noch inert ist (Produktions-Read-Site in Phase 15).

---

## Claude's Discretion

- Ob die SUM-Aggregation als wiederverwendbarer Accessor/Helper eingeführt oder nur als getestete dokumentierte Semantik gepinnt wird (Planner-Entscheidung).
- Test-Datei-/Modul-Platzierung des Round-Trip- und Carry-Forward-Tests.

## Deferred Ideas

- Reporting-Integration + Snapshot-Bump → Phase 15.
- Jahresansicht-Display → Phase 16.
- Vertrags-Editor-Input + „alle"-Filter + unpaid-volunteer-Record → Phase 17.
- Inline-Banner + committed-Chart-Band → v1.5 (CVC-F-01 / CVC-F-02).

---

*Hinweis: Der Großteil der Phase-14-Entscheidungen war bereits auf Milestone-/ROADMAP-/Research-Ebene gelockt (D-01/Variante B, Migrationsform, serde(default), kein REST/OpenAPI, Feld inert, line-für-line-Präzedenz). Die Diskussion beschränkte sich auf die einzige von der ROADMAP explizit offen gelassene Decision: D-OVERLAP-AGG.*
