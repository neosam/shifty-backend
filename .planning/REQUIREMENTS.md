---
milestone: v2.5
milestone_name: Weekly-Overview Performance & Freiwilligen-Abwesenheiten
created: 2026-07-05
categories: [WOP, VAA]
status: planning
---

# Requirements — v2.5: Weekly-Overview Performance & Freiwilligen-Abwesenheiten

## Kontext

Die Jahresübersicht (`/weekly_overview/` im Frontend, Backend
`GET /booking-information/weekly-resource-report/{year}`) hat zwei bekannte
Schwächen, die in v2.5 adressiert werden:

**Performance:** `BookingInformationServiceImpl::get_weekly_summary`
(`service_impl/src/booking_information.rs:259`) iteriert über ~55 Wochen und
ruft pro Woche drei Domain-Services sequenziell auf (`reporting_service.get_week`,
`special_day_service.get_by_week`, `shiftplan_report_service.extract_shiftplan_report_for_week`).
Ergibt ~165 sequenzielle Service-Calls pro Abruf. Load-once für
`all_work_details` und `all_absences` ist schon vorhanden — dasselbe Muster
fehlt für die drei anderen. Der Endpoint braucht dadurch ein paar Sekunden,
während der Rest des Systems in Millisekunden antwortet.

**Anzeige-Lücke Freiwilligen-Abwesenheiten:** In `sales_person_absences` der
Jahresansicht werden Abwesenheiten nur für **bezahlte** Mitarbeiter angezeigt
(Backend-Assembly bei `booking_information.rs:449` unter `is_shiftplanner`,
via `working_hours_per_sales_person`, das aus `reporting.get_week` kommt und
Freiwillige nicht enthält). Die Verfügbarkeit von Freiwilligen wird bereits
korrekt reduziert (VFA-01 whole-week-out aus v1.7), aber der Nutzer sieht
nicht namentlich, **wer** in der Woche fehlt. Fehlt also nur die Anzeige,
nicht die Berechnung.

Vorarbeit: Explore-Session 2026-07-05 (siehe
`notes/weekly-overview-perf-analyse.md`, `seeds/weekly-overview-perf.md`,
`research/questions.md` Q-02).

## Nicht-Ziele

- **Kein HTTP-Caching / kein ETag / kein Snapshot-Layer.** Live-Korrektheit
  ist Pflicht — User schauen sofort nach ihrer eigenen Absence-Eintragung in
  die Übersicht.
- **Kein Snapshot-Schema-Bump.** Keine Änderung am persistierten
  `BillingPeriodValueType`; `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt bei 12.
- **Keine Migration.** Alles rein Rust-seitiger Refactor plus DTO-Erweiterung
  (falls für VAA-01/02 nötig — additiv).
- **Keine Änderung an der Verfügbarkeits-Berechnung für Freiwillige.**
  VFA-01 whole-week-out greift bereits korrekt; VAA erweitert reine Anzeige.
- **Kein Frontend-Redesign der Jahresansicht.** Nur die Absencen-Liste wird
  erweitert.
- **Kein neuer Cargo-Dep.**
- **Keine `get_year`-Erweiterung anderer Services** außerhalb des
  Weekly-Summary-Use-Case (kein Trait-Aufblähen auf Verdacht).

## Requirements

### Weekly-Overview Performance (WOP)

- [ ] **WOP-01**: `get_weekly_summary` lädt `special_days` und
  `shiftplan_reports` einmalig fürs ganze Jahr (Bulk-Load, analog zum
  existierenden `all_work_details`- und `all_absences`-Load-once-Muster).
  Die pro-Woche-Iteration konsumiert vorgeladene Kollektionen statt der
  bisherigen ~110 wochenweisen Service-Calls. Ergebnis unverändert.

- [ ] **WOP-02**: `reporting_service` bekommt eine
  `get_year(year, ...)`-Aggregation (oder eine äquivalente Batch-Variante),
  die die ~55 sequenziellen `get_week`-Calls in `get_weekly_summary` ersetzt.
  Alle bestehenden Semantik-Invarianten bleiben erhalten (Balance-Formel,
  CVC-06 Cap-Gating, Chain-C-Legacy-Filter unter
  `shortday_gate.active_from`, ShortDay-Slot-Clipping). Die alte
  `get_week`-Methode bleibt für andere Call-Sites bestehen (Signatur nicht
  entfernen).

- [ ] **WOP-03**: Regressions-/Property-Test in `service_impl/src/test/`
  beweist byte-identisches Ergebnis: Für einen generierten Jahres-Datensatz
  liefert die neue `get_weekly_summary`-Implementierung dieselbe
  `Arc<[WeeklySummary]>` wie die alte Wochen-Iteration. Diff-Toleranz **0**
  (bit-exakter Vergleich f32-serialisiert). Test muss mehrere Szenarien
  abdecken: Feiertage, ShortDays, Freiwilligen-Absencen, CVC-06-Cap aktiv,
  `shortday_gate.active_from` on und off.

- [ ] **WOP-04**: End-to-End-Latenzziel — `GET
  /booking-information/weekly-resource-report/{year}` antwortet in <500ms
  auf einer Dev-DB mit repräsentativem Datensatz (heute mehrere Sekunden).
  Messmethode und Referenz-Datensatz werden im PLAN dokumentiert.

- [ ] **WOP-05**: Alle bestehenden Tests bleiben grün, insbesondere
  `service_impl/src/test/booking_information.rs`,
  `service_impl/src/test/booking_information_chain_c.rs`, alle
  Reporting-Tests. Backend `cargo test --workspace` + `cargo clippy
  --workspace -- -D warnings` grün.

### Freiwilligen-Abwesenheiten in Jahresansicht (VAA)

- [ ] **VAA-01**: In `sales_person_absences` der Jahresansicht (DTO im
  `WeeklySummaryTO`, gerendert im Frontend
  `page/weekly_overview.rs:121`) erscheinen zusätzlich zu bezahlten
  Mitarbeitern auch Freiwillige mit aktiver Vacation/SickLeave/UnpaidLeave-
  Absence-Period in der jeweiligen Kalenderwoche. Backend liefert Name +
  Stunden-Wert fertig geformt im DTO (Fat Backend, Thin Client — kein
  Absencen-Merge im Frontend).

- [ ] **VAA-02**: Der für einen Freiwilligen angezeigte Stunden-Wert
  repräsentiert sinnvoll den Anteil, der aus der Wochen-Verfügbarkeit fällt.
  Kandidat: `committed_voluntary` der Person für die Woche (Zusage) — die
  Zahl, die auch aus `committed_voluntary_hours` in der Zwei-Band-
  Berechnung fällt, wenn der Freiwillige abwesend ist. Die exakte Semantik
  (`committed_voluntary` vs. anderer Wert) wird in discuss-phase fixiert
  und in einem Decision-Log verankert.

- [ ] **VAA-03**: Backend-Test verifiziert VAA-01/02:
  - Freiwilliger mit `Vacation`-Period, die Kalenderwoche N überlappt →
    erscheint in `sales_person_absences` von Woche N mit dem in VAA-02
    fixierten Stunden-Wert.
  - Freiwilliger ohne aktive Period in Woche N → **nicht** in der Liste.
  - Bezahlter Mitarbeiter bleibt unverändert (keine Regression an
    bestehender Anzeige).

- [ ] **VAA-04**: Frontend-Anzeige: bestehende Rendering-Zeile in
  `page/weekly_overview.rs:121` (`"{name}: {hours} h"`) zeigt Freiwilligen-
  Einträge visuell konsistent mit bezahlten Einträgen. Falls eine visuelle
  Unterscheidung (Farbe, Icon, Suffix) gewünscht ist, wird sie in
  discuss-phase entschieden. Kein Frontend-Redesign.

## Traceability

| REQ-ID | Phase |
| --- | --- |
| WOP-01 | 52 |
| WOP-02 | 52 |
| WOP-03 | 52 |
| WOP-04 | 52 |
| WOP-05 | 52 |
| VAA-01 | 53 |
| VAA-02 | 53 |
| VAA-03 | 53 |
| VAA-04 | 53 |
