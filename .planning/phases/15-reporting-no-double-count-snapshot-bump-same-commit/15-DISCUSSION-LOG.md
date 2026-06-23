# Phase 15: Reporting no-double-count + snapshot bump (SAME commit) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-23
**Phase:** 15-reporting-no-double-count-snapshot-bump-same-commit
**Areas discussed:** Snapshot-Bump & Achse, no-double-count Test-Fixture

---

## Snapshot-Bump & Achse (CVC-04/CVC-05-Widerspruch)

Vorab beim Code-Scout festgestellt: Der Billing-Snapshot speist `BillingPeriodValueType::Volunteer` aus `reporting_service`-Reports (Achse A); `WeeklySummary` (Achse B / `get_weekly_summary`) wird vom Snapshot **nicht** konsumiert. CVC-04 schreibt "nur Achse B" fest, CVC-05 fordert einen Bump wegen "geänderter persistierter Volunteer-Computation" — die zwei widersprechen sich.

Erste Frage war zu jargon-lastig ("Zusage" unklar) → neu erklärt mit Beispiel "5h bezahlt + 5h freiwillig zugesagt"; Zusage = `committed_voluntary` = vorausschauende Plan-/Kapazitätszahl, keine geleistete Stunde.

| Option | Description | Selected |
|--------|-------------|----------|
| Nur Jahresansicht-Anzeige | Zusage nur in get_weekly_summary; gespeicherte Bilanz/Snapshot unberührt; kein Bump, Version bleibt 7; CVC-05 entfällt | ✓ |
| Auch in die Abrechnungs-Bilanz | Zusage fließt in Balance/Billing → Snapshot ändert sich → Bump 7→8 nötig; Nachteil: noch-nicht-geleistete Stunden in der Bilanz | |
| Nochmal anders erklären | (Zwischenschritt — Begriff "Zusage" geklärt) | |

**User's choice:** Nur Jahresansicht-Anzeige (Empfohlen).
**Notes:** committed_voluntary ist reine year-view-Verfügbarkeitskapazität. Phase 15 berührt nur Achse B (`booking_information.rs::get_weekly_summary`), nicht `reporting.rs` (Achse A). Kein persistierter `value_type` ändert sich → kein Snapshot-Bump; `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 7. CVC-05 + Phasentitel "+ snapshot bump (SAME commit)" sind überholt → Planner zieht ROADMAP/REQUIREMENTS nach (no-bump-Begründung statt Bump).

---

## no-double-count Test-Fixture

| Option | Description | Selected |
|--------|-------------|----------|
| Kern: max pro Woche | committed=5/actual=7→7, committed=5/actual=3→5 | ✓ |
| Summe über Jahr (nie max(Σ,Σ)) | mehrere Wochen → pro Woche max, dann summieren | ✓ |
| cap=false → 0.0 Backward-Compat | Zusage trägt 0.0 für nicht-gedeckelte Personen bei | ✓ |
| Mehrere Personen aggregiert | 2+ Personen, per-Person-max korrekt mit Aggregation | ✓ |

**User's choice:** Alle vier + "Alles was geht! Mehr Tests sind immer gut!"
**Notes:** Maximale Coverage gewünscht. Zusätzlich sinnvolle Fälle (Single-Week, leere Woche → 0.0, Boundary committed==actual, committed=0 ⇒ keine Wirkung) als Claude's Discretion mit aufnehmen. Float-Vergleiche via Epsilon.

---

## Claude's Discretion

- Test-Modul-/Datei-Platzierung in `booking_information.rs`.
- Ob ein privater Helper für die per-Woche-max-vor-Summe eingeführt wird (Wiederverwendung von `committed_voluntary_for_calendar_week` aus Phase 14) oder inline.
- **Teilwochen-Gewichtung (D-03):** nicht separat besprochen — Research-Default übernommen: flat pro aktiver ISO-Woche, kein Pro-Rating.
- **Phase-15/16-Schnitt (D-04):** nicht separat besprochen — Default übernommen: Service-Struct `WeeklySummary` + Berechnung in 15; `WeeklySummaryTO` + Frontend in 16.

## Deferred Ideas

- WeeklySummaryTO + Frontend „zugesagt"-Token + Überschuss + i18n → Phase 16.
- Editor-Input + „alle"-Filter + unpaid-volunteer-Record → Phase 17.
- Inline-Banner + committed-Band im Chart → v1.5.
- **Research-Flag:** „actual_volunteer" für eine bezahlt-gedeckelte Person (is_paid=true, cap=true) in Achse B ist undefiniert — get_weekly_summary zählt heute nur is_paid=false-Personen. Höchst-Leverage-Gap; gsd-phase-researcher muss die Formel-Mechanik klären (siehe CONTEXT Research Flag).
