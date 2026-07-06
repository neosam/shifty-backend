# Phase 53 — Discussion Log

**Date:** 2026-07-06
**Mode:** discuss (Textform)
**Owner:** neosam

## Gray Areas Discussed

### G1 — Backend-Träger: Neues DTO-Feld vs. `WorkingHoursPerSalesPerson` erweitern

**Options presented:**
- a) Neues Feld `sales_person_absences: Arc<[SalesPersonAbsence]>` auf `WeeklySummary` + `WeeklySummaryTO`
- b) `WorkingHoursPerSalesPerson` um Freiwillige erweitern (0-Padding)
- c) Claude's Discretion → (a)

**User selection:** **a**

**Notes:** REQUIREMENTS.md nennt das Feld explizit im DTO. Einzige Variante die Fat-Backend-Thin-Client strukturell erfüllt. FE-Mapper-Refactor akzeptiert. Locked as D-53-01.

---

### G2 — Angezeigter Stunden-Wert

**Options presented:**
- a) `committed_voluntary` cap-gated (Kandidat aus REQUIREMENTS)
- b) Absence-Tage × pro-Tag-Anteil (Umrechnung willkürlich)
- c) 0.0 (nur Name)
- d) Claude's Discretion → (a)

**User selection:** **a**

**Notes:** Semantisch die verlorene Zusage, konsistent mit VFA-01 Band-1-Formel. Formel-Präzedenz `booking_information.rs:495–503`. Randfall Zusage=0 → 0.0 im Backend, FE-Filter `>= 0.1` regelt Sichtbarkeit. Locked as D-53-02.

---

### G3 — Whole-Week-Out vs. partiell

**Options presented:**
- a) Ja, whole-week-out spiegeln (identisch zu VFA-01)
- b) Nein, partiell anteilig
- c) Claude's Discretion → (a)

**User selection:** **a**

**Notes:** Symmetrie zwischen Berechnung (VFA-01) und Anzeige (VAA). Consumer-Konsistenz: „welche Zusage fällt weg?" → volle Wochen-Zusage. Locked as D-53-03.

---

### G4 — Visuelle Präsentation

**Options presented:**
- a) Ein Topf, sortiert nach Name (kein visueller Unterschied)
- b) Bezahlte oben, Freiwillige unten
- c) Suffix „(F)" oder Icon 🤝
- d) Claude's Discretion → (a)

**User selection:** **a**

**Notes:** VAA-04 „kein Redesign, visuell konsistent" respektiert. Minimaler FE-Diff, spätere Aufstockung offen. FE-Mapper baut Union + sortiert nach Name (`str::to_lowercase().cmp()`, kein Dep). Locked as D-53-04.

---

### G5 — Fill-Site: `get_weekly_summary` vs. `assemble_weeks`

**Options presented:**
- a) Fill-in in `get_weekly_summary` direkt (D-52-09-konform)
- b) `assemble_weeks` erweitern (verletzt D-52-09)
- c) Claude's Discretion

**User selection:** **c** → Claude entschied **a**

**Notes:** D-52-09 hält `assemble_weeks` clean von Business-Logic-Cross-Cutting. Freiwilligen-Absencen-Anzeige ist `BookingInformationService`-DTO-Assembly, keine `ReportingService`-Semantik. Alle Inputs (`absent_volunteer_ids`, `sales_person_service.get_all`, `all_work_details`) sind bereits im Loop verfügbar. Locked as D-53-05.

---

### G6 — `get_summery_for_week` mitziehen

**Options presented:**
- a) Ja, mitziehen (konsistent, +15–25 LOC)
- b) Nein, Feld leer lassen
- c) Claude's Discretion → (a)

**User selection:** **a**

**Notes:** Beide Endpoints geben denselben `WeeklySummary`-Typ zurück; das neue Feld muss an beiden Fill-Sites ehrlich befüllt werden. `absent_volunteer_ids` wird in der Single-Week-Methode aktuell nicht berechnet — Planner entscheidet über In-Line vs. Helper-Extraktion (Empfehlung: In-Line Wave 1, Cleanup optional). Locked as D-53-06.

---

## Scope Creep — Redirected

Keine Scope-Creep-Momente aufgetreten. Discussion blieb strikt auf VAA-Anzeige.

## Claude's Discretion — Applied

- G5 → Fill-in in `get_weekly_summary` direkt (D-52-09 MUST-preserve als
  Entscheidungsgrund).

## Deferred Ideas

- Konstante für FE-Filter-Threshold `>= 0.1` (Cleanup, nicht scope-blockierend).
- Helper-Extraktion `absent_volunteer_ids_for_week` — optional wenn Planner
  Duplikation vermeiden will.
