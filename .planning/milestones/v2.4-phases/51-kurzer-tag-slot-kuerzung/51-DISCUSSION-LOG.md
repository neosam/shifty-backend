---
phase: 51
date: 2026-07-04
type: discussion_log
---

# Phase 51 — Discussion Log

Human-lesbare Zusammenfassung des `/gsd-discuss-phase 51`-Verlaufs.
Kanonische Decisions leben in `51-CONTEXT.md`.

## Kontext-Snapshot am Start

- Semantik bereits im Explore 2026-07-04 fixiert (D-01…D-06)
- Requirements SHC-01…SHC-05 bereits definiert (das WAS)
- Discussion sollte nur HOW klären
- gsd-tools roster kaputt → manueller Workflow

## Präsentierte Gray Areas

Sechs Gray Areas identifiziert und als Textliste (User-Präferenz laut
`feedback_prefer_text_questions`) präsentiert:

### A) Ort der kanonischen Clip-Funktion
Optionen: (a) `rest-types` / (b) `shifty-utils` / (c) `service_impl`-Helper
oder Method auf `service::slot::Slot` / (d) Egal, Researcher-Empfehlung

**User-Nachfrage:** Was sind „Call-Sites"?
**Erläuterung:** Aufruf-Stellen im Code (Reporting-Service, BlockService,
WeekView, PDF-Renderer, iCal, ggf. Booking-Validation). Der Researcher
liefert die exakte Datei:Zeile-Liste (Q-01). Deine Entscheidung ist der
architektonische Rahmen.

Erst nach Wahl bei B geklärt: Da B=(a) das FE die Fn nicht mehr braucht,
schrumpft A auf reine BE-Frage. `rest-types` fällt raus, `shifty-utils`
ist "sauber" aber unnötig, `service_impl`/Method-auf-Slot ist minimal.

**Wahl:** A(c) — Method auf `service::slot::Slot`.
**Rationale User:** „Es gehört zum Fachobjekt."
**Bestätigung Claude:** Ja, klassisches Domain-Modeling — Verhalten wandert
zur Struct, die die Daten hält. Keine Persistenz-/Transport-Concerns.

### B) FE-Konsum-Modell
Optionen: (a) BE liefert geclippte Slots im DTO / (b) BE liefert rohe
Slots + Cutoff, FE clippt selbst

**Wahl:** B(a).

### C) Booking-Validation an ShortDays
Optionen: (a) Blockieren / (b) Erlauben, Slot zählt 0h / (c) Warnen

**Wahl:** C(b) — konsistent mit D-02 view-layer-Semantik.

### D) Visuelle Markierung verkürzter Slots
Optionen: (a) Nichts extra / (b) Icon / (c) Farbe/Rahmen / (d) Tageskopf

**Wahl:** D(a) — Länge signalisiert es.

### E) iCal-Export als eigene Anpassung?
Optionen: (a) In v2.4 mitnehmen / (b) Deferred

**Erst-Wahl:** E(a).
**User-Rückfrage:** „Ist es nicht eigentlich der Block Service?"

**Recherche:** Ja. `service/src/block.rs:91` — `get_blocks_for_next_weeks_
as_ical` konsumiert `Block`s. Wenn `BlockService`-Konstruktion die
Slots clippt, ist iCal automatisch mit-korrekt. Auch Reporting
(`block_report.rs`), Booking-Info und MyBlock sind Block-Konsumenten.

**Konsequenz:** E fällt als eigene Gray Area weg — iCal ist Nebeneffekt
der BlockService-Anpassung. Kein SHC-06 nötig. Rearchitektur: der
eigentliche „View-Aggregat"-Punkt (F) ist tatsächlich zweigeteilt —
BlockService (für Bookings-Konsumenten) + ShiftplanWeek-DTO-Bauer (für
Grid-Konsumenten).

### F) PDF-Renderer-Konsum-Modell
Fällt aus B(a) automatisch auf F(b) (gemeinsames View-Aggregat).

## Ergänzendes Prinzip (User-getrieben)

User hat nach den sechs Gray Areas explizit gebeten: „generell noch
festhalten, dass sämtliche Businesslogik im Backend stecken soll. Das
Frontend soll möglichst nur die Daten anzeigen. Ich will ggf. andere
Frontends wie Apps ermöglichen und nicht überall Businesslogik verbauen
müssen."

**Persistiert an drei Orten:**
1. `.planning/PROJECT.md` → neuer Abschnitt „Architektur-Prinzipien →
   Fat Backend, Thin Client"
2. Auto-Memory: `feedback_fat_backend_thin_client.md` (feedback-Typ,
   damit ich es in jeder zukünftigen discuss-phase heranziehe)
3. `51-CONTEXT.md` → Locked Constraint + Rationale in D-51-02

Formulierung nach User-Feedback abgestimmt und ohne weitere Justierung
freigegeben.

## Aggregierte Decisions (siehe CONTEXT.md für Details)

| Ref | Decision | Auswahl aus | Rationale |
|-----|----------|-------------|-----------|
| D-51-01 | Clip-Fn als Method auf `service::slot::Slot` | A(c) | Gehört zum Fachobjekt |
| D-51-02 | DTO liefert geclippte Slots (Fat-Backend) | B(a) | Zweit-Client-Fähigkeit |
| D-51-03 | Booking-Create nicht abgelehnt | C(b) | Konsistent mit D-02 |
| D-51-04 | Keine visuelle Zusatz-Markierung | D(a) | UI-Noise vermeiden |
| D-51-05 | iCal via BlockService abgedeckt (revidiert in D-51-06) | E gestrichen | Block-Aggregat als View-Punkt |
| D-51-06 | **Vier** BE-Aggregat-Ketten (Research-Update) | Research | Chain A'/B/C/D, siehe RESEARCH.md |
| D-51-07 | Admin-Stichtag via Toggle `shortday_slot_clipping_active_from` | Nachdisk. | Balance-Historie schützen, HCFG-02-Muster |
| D-51-08 | Chain D: Rust-Layer-Clipping, nicht SQL-Erweiterung | Nachdisk. | Kanonische Clip-Fn, testability |
| D-51-09 | `effective_to` am `ShiftplanSlotTO`-Wrapper | Research | SlotTO ist bidirektional |

## Nachdiskussion (2026-07-04, post-research)

Nach `/gsd-plan-phase 51`-Start hat der `gsd-phase-researcher` in
`51-RESEARCH.md` zwei kritische Funde gemeldet, die vor plan-phase geklärt
werden mussten:

**Fund A — Chain-D-Discovery:** Balance/Ist-Stunden fließt nicht durch
`BlockService`, sondern durch rohem SQL in
`dao_impl_sqlite/src/shiftplan_report.rs`. Plus drei direkte
`slot.to - slot.from`-Sites in `booking_information.rs`. Vier statt zwei
Aggregat-Ketten.

**Fund B — pre-existing Bug:** `shiftplan.rs:62-66` +
`booking_information.rs:394-401, 512-519` verwerfen Slots komplett statt
zu clippen (verletzt D-04). Wird bei Feature-Implementierung mit-gefixt.

**User-Frage:** „Wie legen wir das Startdatum für diese neue Regelung
fest? In den Einstellungen?"

**Rationale User:** „Historisch würde sonst die Stunden neu berechnen und
sämtliche Stundenkonten wären verändert."

**Wahl:** Config-Datum in Settings (analog HCFG-02 aus v1.7) — Toggle
`shortday_slot_clipping_active_from` (ISO-8601-Date). Ohne Wert → Kürzung
aus (Rollout-Default); mit Wert → gate `booking_date >= active_from` in
allen vier Ketten. → **D-51-07** in CONTEXT.md, **SHC-06** in
REQUIREMENTS.md.

**Sub-Entscheidung (Default-Annahme in Auto-Mode):** Eigenes Toggle-Feld,
nicht an existierenden HCFG-Stichtag gebunden. Rationale: verschiedene
Semantiken, unterschiedliche Rollout-Zeitpunkte möglich, spätere
Konsolidierung trivial.

**Zweite Frage (Chain D):** SQL-Change vs. Rust-Layer vs. Chain-D
vorerst nicht anfassen?

**Wahl:** Option 1 — Rust-Layer-Clipping. DAO liefert Rohdaten,
`ShiftplanReportServiceImpl` aggregiert per `Slot::clip_to` +
Stichtag-Gate. Kanonische Clip-Fn für alle vier Ketten, Stichtag-Gate an
einer Stelle, testbar in Rust. → **D-51-08**.

**Verworfen:** SQL-JOIN + `MIN()`-Ansatz (Clip-Semantik in SQL dupliziert,
Snapshot-Immunität schwerer zu argumentieren, schwerer testbar).

## Deferred Ideas

- Tageskopf-Kennzeichnung ShortDay im WeekView (falls Kürzungsursache
  unklar wird)
- Warnung bei Booking-Create auf Post-Cutoff-Slot (Banner, nicht Dialog)

## Nicht diskutiert (bewusst)

- Q-01 (kanonische Slot-Auflösung + Call-Sites) → Researcher-Auftrag,
  nicht User-Entscheidung
- Semantik-Fragen D-01…D-06 → im Explore bereits gelockt, hier nur
  referenziert

## Next Steps

`/clear` und dann:

```
/gsd-plan-phase 51
```

Oder mit Researcher-Vorlauf für Q-01:

```
/gsd-plan-phase 51
```

(mit gsd-phase-researcher als Wave-1-Agent, der die Call-Site-Liste
liefert bevor Wave-1-Plan geschrieben wird).
