---
milestone: v2.4
milestone_name: Kurzer-Tag-Slot-Kürzung
created: 2026-07-04
categories: [SHC]
---

# Requirements — v2.4: Kurzer-Tag-Slot-Kürzung

## Kontext

Bisher hat das existierende `special_day.ShortDay`-Modell (ein
Cutoff-Zeitpunkt auf einem konkreten Datum) **keine Auswirkung** auf Slots
oder Stundenberechnung. Slots werden voll dargestellt, gebuchte Stunden
zählen mit der vollen Slot-Dauer — auch wenn der Betrieb an dem Tag früher
schließt. v2.4 gleicht das an: an einem Kurzen Tag werden Slots, die den
Cutoff überlappen, auf `[slot.start, cutoff]` gekürzt (Rendering + Ist-
Stunden), und Slots komplett hinter dem Cutoff verschwinden.

Die Semantik wurde am 2026-07-04 in einer `/gsd-explore`-Session mit dem
User fixiert und in
`notes/shortday-slot-clipping-semantics.md` (D-01 bis D-06) als
Decision-Log verankert. Zusammengefasst:

- **Modell:** ShortDay = eine Cutoff-Uhrzeit an genau einem Datum. Kein
  Split-Öffnungszeiten-Modell.

- **Kürzung dynamisch** (view-layer). Slot-DB bleibt unangetastet;
  Rendering + Reporting wenden den Cutoff on-the-fly an. Kein Snapshot-
  Schema-Bump, keine Migration.

- **Nur zukünftig / no rewrite:** Historische Bookings werden nicht
  neuberechnet. Zukünftige ShortDay-Einträge wirken ab dem Zeitpunkt der
  Live-Berechnung.

- **Cutoff-Regeln pro Slot:**
  - `slot.end <= cutoff` → unverändert
  - `slot.end == cutoff` → unverändert (kein Sonderfall)
  - `slot.start >= cutoff` → Slot komplett raus (nicht gerendert, 0 h)
  - `slot.start < cutoff < slot.end` → gekürzt auf `[slot.start, cutoff]`
- **Wirkt nur auf Ist-Stunden**, nicht auf Soll. Vertrag bleibt
  unverändert; Mitarbeiter sammelt ggf. Minusstunden im Balance-Konto.

- **Verkürzte Slots werden verkürzt dargestellt** (Rendering-Länge zeigt
  effektive Dauer, keine gesonderte Markierung nötig).

## Nicht-Ziele

- **Kein Snapshot-Schema-Bump.** Berechnung ändert sich nur auf zukünftig
  gebuchte/gelesene Werte; historische `billing_period`-Snapshots bleiben
  unter Schema-Version 12 gültig.

- **Keine Migration** an der Slot-Tabelle oder anderen Domain-Tabellen.
  **Ausnahme:** Toggle-Seed für `shortday_slot_clipping_active_from`
  (SHC-06) — additive Zeile im `toggle`-Katalog, kein Schema-Change,
  Präzedenz HCFG-02 aus v1.7.

- **Kein Rewrite existierender Bookings** an Tagen, die nachträglich zu
  Kurzen Tagen werden. Ihre Ist-Stunden schrumpfen automatisch durch die
  dynamische Berechnung, aber es wird kein DB-Update ausgelöst.

- **Keine Änderung an Soll-Stunden / Vertragserwartung.** An Kurzen Tagen
  sammelt der Mitarbeiter Minusstunden im Balance-Konto — Alternative
  (Soll schrumpft mit) im Explore bewusst verworfen.

- **Keine erweiterte Cutoff-Semantik** (kein Split-Vormittag/Nachmittag,
  keine mehreren Cutoffs pro Tag). Ein Cutoff-Zeitpunkt pro Datum.

- **Kein Warnhinweis-System** für gekürzte Bookings im FE (Slot wird
  einfach kürzer dargestellt, keine Toast/Banner).

- **Keine Booking-Validation-Änderung an legacy `POST /booking`** — falls
  Validation für ganz-außerhalb-Slots gewünscht wird, im
  Conflict-Aware-Booking-Pfad (`ShiftplanEditService`) diskutieren, nicht
  im legacy Pfad.

## Requirements

### Kurzer-Tag-Slot-Kürzung (SHC)

- [x] **SHC-01**: Es existiert eine kanonische Clip-Funktion, die für ein
  gegebenes `(slot, cutoff_time)` einen `Option<Slot>` zurückgibt — `None`
  wenn der Slot komplett hinter dem Cutoff liegt (`slot.start >= cutoff`),
  ansonsten der (ggf. auf `slot.end = cutoff` gekürzte) Slot. Die Funktion
  ist pure und vollständig unit-getestet für alle vier Cutoff-Fälle (Slot
  vor Cutoff / endet exakt am Cutoff / überlappt / komplett nach Cutoff).

- [x] **SHC-02**: Der Reporting-Pfad (Ist-Stunden-Berechnung — Balance,
  Booking-Information-Aggregate) berücksichtigt die Kürzung. Wenn am
  Buchungstag ein ShortDay existiert, zählt jede Slot-Buchung die effektive
  (geclippte) Dauer statt der Roh-Slot-Dauer. Backend-Test verifiziert:
  Booking auf Slot 14:00–15:00 mit ShortDay-Cutoff 14:30 zählt 0,5 h,
  ohne ShortDay 1 h.

- [x] **SHC-03**: Der Schichtplan-Wochen-View im Frontend (`shifty-dioxus`
  WeekView) zeigt Slots an Kurzen Tagen in der geclippten Länge — Slot
  14:00–15:00 mit Cutoff 14:30 wird als Zelle 14:00–14:30 gerendert. Slots
  komplett hinter dem Cutoff werden gar nicht mehr angezeigt.

- [x] **SHC-04**: Der PDF-Renderer (`service_impl/src/pdf_render.rs`,
  v2.3-Rewrite) wendet dieselbe Clip-Semantik an. PDF und Browser-WeekView
  bleiben visuell konsistent an Kurzen Tagen — Slot 14:00–15:00 im PDF ist
  bei Cutoff 14:30 ebenfalls verkürzt gerendert, Post-Cutoff-Slots fehlen.

- [x] **SHC-05**: Das Setzen oder Ändern eines ShortDays auf einem
  **zukünftigen** Datum mit bereits existierenden Bookings ist erlaubt.
  Existierende Bookings überleben unverändert; ihre Ist-Stunden werden ab
  diesem Moment durch die dynamische Berechnung reduziert. Es wird kein
  Booking-Rewrite und keine Cascade-Warnung ausgelöst.

- [x] **SHC-06**: Es existiert ein admin-konfigurierbarer Stichtag
  `shortday_slot_clipping_active_from` (ISO-8601-Date via
  `ToggleService`, Präzedenz HCFG-02 aus v1.7). Ohne gesetzten Wert
  bleibt die Kürzung deaktiviert (Legacy-Verhalten, Rollout-Default). Bei
  gesetztem Wert wirkt die Kürzung ausschließlich für Bookings mit
  `booking_date >= active_from` — in allen vier Aggregat-Ketten
  (Chain A' BlockService, Chain B ShiftplanWeek/PDF, Chain C
  BookingInformation, Chain D ShiftplanReport). Live-Balance-Views auf
  historische Wochen bleiben dadurch unverändert. Backend-Test verifiziert
  Stichtag-Gate am Grenzfall (`booking_date == active_from - 1` →
  ungeclippt; `== active_from` → geclippt). Admin-Editor in Settings
  analog HCFG-02.

## Constraints

- **Dynamische Kürzung im View-Layer.** Kanonische Clip-Funktion wird an
  jeder Konsum-Stelle aufgerufen (Reporting, Booking-Information, WeekView,
  PDF-Renderer, ggf. iCal). Kein persistierter Cache-Wert; kein
  DAO-Rewrite.

- **Snapshot-Schema-Version bleibt 12.** SHC-02 ändert nur die
  Live-Berechnung, nicht die persistierte
  `billing_period_sales_person.value_type`-Menge oder die
  Snapshot-Writer-Logik.

- **Nur zukünftig.** Historische Snapshots werden nicht neu berechnet,
  historische Bookings nicht angepasst.

- **Soll-Stunden bleiben unberührt.** SHC-02 wirkt ausschließlich auf die
  Ist-Seite der Balance.

- **Ort der Clip-Funktion:** Method auf `service::slot::Slot` (D-51-01,
  bestätigt in discuss-phase).

- **Admin-Stichtag als Gate** (SHC-06 / D-51-07): ohne Wert → keine
  Kürzung; mit Wert → gate `booking_date >= active_from`. Muster
  identisch zu HCFG-02 aus v1.7 (`ToggleService::get_toggle_value`).

- **Vier BE-Aggregat-Ketten** (D-51-06): Chain A' BlockService,
  Chain B ShiftplanWeek/PDF, Chain C BookingInformation,
  Chain D ShiftplanReport (Balance/Ist-Stunden). Chain D wird
  Rust-Layer-refaktoriert, nicht SQL-erweitert (D-51-08).

- **DTO-Feld am Wrapper** (D-51-09): `ShiftplanSlotTO.effective_to` —
  nicht `SlotTO.to` mutieren (bidirektionales DTO). Präzedenz
  `current_paid_count`.

## Traceability

| Requirement | Phase | Plan(s) | Status |
|-------------|-------|---------|--------|
| SHC-01 | 51 | 51-01 (primary) | pending |
| SHC-02 | 51 | 51-03, 51-04, 51-05, 51-06 (primary Chain D) | pending |
| SHC-03 | 51 | 51-07 (primary) | pending |
| SHC-04 | 51 | 51-03 (auto via Chain B), 51-07 (verify) | pending |
| SHC-05 | 51 | 51-03, 51-04, 51-05, 51-06 | pending |
| SHC-06 | 51 | 51-02 (BE), 51-03, 51-04, 51-05, 51-06 (Gate-Konsum), 51-08 (FE) | pending |
