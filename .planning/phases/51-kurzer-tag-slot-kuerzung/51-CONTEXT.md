---
phase: 51
phase_name: Kurzer-Tag-Slot-Kürzung
milestone: v2.4
created: 2026-07-04
type: context
requirements: [SHC-01, SHC-02, SHC-03, SHC-04, SHC-05]
---

# Phase 51 — Kurzer-Tag-Slot-Kürzung — CONTEXT

## Domain

An Kurzen Tagen (`special_day.ShortDay` mit `time_of_day: Time` als Cutoff)
werden Slots, die den Cutoff überlappen, **dynamisch (view-layer)** am
Cutoff gekürzt — sowohl im Rendering (Schichtplan-WeekView + PDF) als
auch bei der Ist-Stunden-Berechnung (Reporting + Booking-Information).
Slots komplett hinter dem Cutoff verschwinden. Soll-Stunden bleiben
unverändert. Kein DB-Change, kein Snapshot-Bump, keine Migration.

## Requirements (locked)

Vollständige Definition: `.planning/REQUIREMENTS.md` — SHC-01…SHC-05.
Kurzfassung (das WAS ist fixiert, discuss klärt das WIE):

- **SHC-01** — Kanonische Clip-Funktion `(Slot, cutoff) -> Option<Slot>`
  mit den vier D-04-Fällen.
- **SHC-02** — Reporting-Ist-Stunden + Booking-Information-Aggregate
  konsumieren die geclippte Dauer.
- **SHC-03** — WeekView im FE zeigt geclippte Länge; Post-Cutoff-Slots
  fehlen.
- **SHC-04** — PDF-Renderer bleibt visuell konsistent zum WeekView.
- **SHC-05** — Zukünftiger ShortDay auf Datum mit bestehenden Bookings:
  Bookings überleben, Ist-Stunden schrumpfen automatisch, kein Rewrite.

## Semantik-Anker (locked im Explore 2026-07-04)

Vollständig: `.planning/notes/shortday-slot-clipping-semantics.md` (D-01…D-06).

- **D-01** Cutoff-Modell: eine Uhrzeit pro Datum — nutzt existierendes
  `SpecialDay.time_of_day: Option<time::Time>` (`service/src/special_days.rs:41`).
  Kein Modell-Change.
- **D-02** Kürzung dynamisch (view-layer), Slot-DB unangetastet.
  ShortDay löschen → Slots wieder in voller Länge.
- **D-03** Nur zukünftig, keine historische Rückrechnung. Snapshot-Schema
  bleibt Version 12.
- **D-04** Cutoff-Regeln pro Slot:

  | Slot vs. Cutoff | Verhalten |
  |---|---|
  | `slot.end <= cutoff` | unverändert |
  | `slot.end == cutoff` (exakt) | unverändert (kein Sonderfall) |
  | `slot.start >= cutoff` | komplett raus (nicht gerendert, 0 h) |
  | `slot.start < cutoff < slot.end` | gekürzt: `[slot.start, cutoff]` |

- **D-05** Wirkt nur auf Ist, nicht auf Soll. Mitarbeiter mit gekürztem
  Slot sammelt ggf. Minusstunden im Balance-Konto.
- **D-06** Verkürzter Slot wird verkürzt gerendert (Länge signalisiert
  die Kürzung; Zusatz-Markierung explizit *nicht* gewünscht — siehe D-51-04).

## Decisions (aus discuss-phase 2026-07-04)

### D-51-01 — Ort der kanonischen Clip-Funktion: `service::slot::Slot`

Method `Slot::clip_to(cutoff: time::Time) -> Option<Slot>` (oder
äquivalente freie Fn im selben Modul), lebt in
`service/src/slot.rs:13`.

**Rationale:** Slot-Clipping ist reine Fachlogik (Verhalten wandert zur
Struct, die die Daten hält). Kein Persistenz-, kein Transport-Concern.
Da B-51-02 das Frontend die Clip-Fn gar nicht konsumieren lässt, gibt
es keinen Bedarf für eine Shared-Crate wie `shifty-utils` oder
`rest-types`. Minimal-invasive Wahl, in der Layer wo die Konsumenten
leben.

**Verworfen:** `shifty-utils` (unnötige Shared-Crate, wenn nur BE
konsumiert); `rest-types` (FE braucht die Fn nicht mehr wegen B-51-02);
`service_impl`-Helper (Slot-Logik gehört ans Slot-Objekt, nicht in eine
freie Utility-Funktion).

### D-51-02 — DTO liefert bereits geclippte Slots (Fat-Backend-Konsequenz)

Der Backend-Response für Shiftplan-View + Block-Aggregate enthält bereits
die geclippten Slot-Fenster. Das FE bekommt fertige `effective_end`-Werte
(oder equivalent) auf `SlotTO` in der ShiftplanWeek-Response und muss
keine Clip-Logik implementieren.

**Rationale:** Fat-Backend-Prinzip aus PROJECT.md („Architektur-Prinzipien
→ Fat Backend, Thin Client"). Alle zukünftigen Clients (Mobile-App,
alternative UI) bekommen die gekürzten Zeiten kostenlos.

**Konkrete DTO-Änderung:** In `rest-types` bekommt der Slot-Container
der ShiftplanWeek-Response entweder ein Feld `effective_end: Time` (bei
Cutoff-Kürzung ≠ `end`, sonst == `end`) *oder* der `SlotTO` wird direkt
mit den geclippten Zeiten befüllt. Endgültige Feldbenennung → planner.

### D-51-03 — Booking-Create bleibt unangefasst

Post-Cutoff-Slot-Buchungen werden nicht abgelehnt. View-Layer-Semantik
(D-02) bleibt konsistent: die Buchung existiert, zählt 0 h, wird nicht
gerendert. Kein neuer Validation-Pfad.

**Rationale:** ShortDay ist ein reversibler Zustand — würde ein User
den ShortDay wieder löschen, wäre die Buchung sofort wieder gültig.
Blockende Validation würde D-02 verletzen. Falls das UX-Problem wird,
ist eine Warnung (Banner, nicht Dialog — siehe
`feedback_warnings_inline_not_dialog`) die spätere Verschärfung.

### D-51-04 — Keine visuelle Zusatz-Markierung verkürzter Slots

Im WeekView + PDF signalisiert die tatsächliche Länge die Kürzung. Kein
Icon, kein Extra-Rahmen, keine Farbschattierung pro Slot.

**Rationale:** UI-Noise vermeiden. Der Tageskontext (ShortDay ist als
special_day sichtbar) reicht als Kontext.

**Deferred (nicht in v2.4):** Falls sich zeigt, dass User die
Kürzungsursache nicht erkennen, kann eine Tageskopf-Markierung (z. B.
„Cutoff 14:30") als Follow-up nachgezogen werden.

### D-51-05 — iCal nicht als eigene Anpassung, sondern via BlockService

iCal-Export ist keine separate Call-Site. `BlockService::get_blocks_for_
next_weeks_as_ical` (`service/src/block.rs:91`) konsumiert `Block`s,
die aus `Slot`s aufgebaut werden. **Anpassungspunkt ist die
`BlockService`-Konstruktion:** beim Bauen der `Block`s werden die
enthaltenen `Slot`s per ShortDay-Lookup pro Datum geclippt.

**Konsequenzen (automatisch mit-korrekt):**
- iCal (`get_blocks_for_next_weeks_as_ical`)
- Reporting/Balance (`service_impl/src/block_report.rs`)
- Booking-Information-Aggregate
- Employee-Sicht `MyBlockService` (`service/src/my_block.rs`)
- `Block::datetime_to()` (`service/src/block.rs:65`) für iCal-DTSTART/DTEND

**Kein separates SHC-06 für iCal nötig** — SHC-02 + SHC-05 decken es
strukturell ab.

### D-51-06 — Zwei BE-Aggregat-Punkte (statt einem)

Es gibt zwei getrennte BE-Aggregat-Ketten, an denen der Clip greifen muss:

1. **BlockService** (aggregiert Bookings) — Reporting, Booking-Info,
   iCal, MyBlock. Bereits identifiziert.
2. **ShiftplanWeek-DTO-Bauer** (aggregiert Slot-Grid pro Datum ohne
   Bookings) — WeekView + PDF-Renderer.

Beide konsumieren `Slot::clip_to`. Der ShiftplanWeek-Pfad ist Q-01 im
Researcher-Auftrag (siehe `open_research`).

## Locked Constraints (cross-cutting)

- **Fat Backend, Thin Client** (siehe PROJECT.md → Architektur-Prinzipien):
  FE bekommt fertige geclippte Werte im DTO, rechnet nichts selbst.
- **Snapshot-Schema-Version bleibt 12.** Kein `BillingPeriodValueType`
  angefasst, keine Bump-Pflicht nach
  `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Regel (`shifty-backend/CLAUDE.md`).
- **Keine DB-Migration**, keine neue Cargo-Dep.
- **Soll-Stunden unberührt.** Nur Ist-Seite der Balance.
- **Nur zukünftig** — kein historischer Rewrite, historische Snapshots
  bleiben unangetastet.

## Code Context (verifizierte Anker)

- `service/src/special_days.rs:41` — `SpecialDay.time_of_day: Option<time::Time>`
  hält den Cutoff. Bereits vorhanden, kein Modell-Change nötig.
- `service/src/slot.rs:13` — `pub struct Slot` — Ort der geplanten
  `clip_to`-Method (D-51-01).
- `service/src/block.rs:18` — `pub struct Block { slots: Arc<[Slot]>, from, to }`
  — der Aggregatpunkt für BlockService-Konsumenten (D-51-05).
- `service/src/block.rs:73` — `trait BlockService` mit `get_blocks_for_
  sales_person_week` (WeekView-Feed), `get_blocks_for_next_weeks_as_ical`,
  `get_unsufficiently_booked_blocks`, `get_blocks_for_current_user`.
- `service/src/my_block.rs:17` — `trait MyBlockService` (Employee-Sicht).
- `service_impl/src/block_report.rs:10` — Reporting-Konsum von `Block`.
- `service_impl/src/block.rs:29` — `BlockServiceImpl` (Konstruktions-Ort
  für die Clip-Einbindung).
- `rest-types/src/lib.rs:308` — `SlotTO` (potenzieller Träger von
  `effective_end` per D-51-02).
- `rest-types/src/lib.rs:1041` + `service/src/special_days.rs:15` —
  `ShortDay`-Enum-Variante existiert bereits.
- `service_impl/src/pdf_render.rs` — v2.3-PDF-Renderer, konsumiert das
  Shiftplan-Aggregat für SHC-04.

## Open Research (für gsd-phase-researcher)

**Q-01** (aus `.planning/research/questions.md`) — kanonischer Ort der
Slot-Auflösung + Call-Sites im ShiftplanWeek-Pfad. Konkret:

1. Wer baut heute das Slot-Grid pro Datum für WeekView-Rendering?
   `SlotService::slots_for_date` oder eigenes Aggregat in `service_impl`?
2. Wo werden `special_day`s heute pro Datum gelookupt? (Es gibt einen
   Feiertag-Konsumenten im Reporting — dessen Lookup-Pfad
   wiederverwenden.)
3. PDF-Renderer-Input: konsumiert er dasselbe Aggregat wie WeekView oder
   hat er einen eigenen ShiftplanWeek-Bauer? (Siehe `service_impl/src/
   pdf_render.rs`, v2.3-Rewrite mit 5-Parameter-Signatur.)
4. Verifikation: Ist `BlockService` wirklich der einzige BE-Aggregat-Punkt
   für Reporting/Ist-Stunden + iCal + Booking-Info, oder gibt es
   direkte Slot-Time-Berechnungen an anderen Call-Sites?
5. `SlotTO`-Feld-Design: gibt es Präzedenz für „effektive Werte" auf DTOs
   (z. B. bei anderen View-Layer-Berechnungen), an die wir uns anlehnen
   können?

**Output-Ziel:** 1-Seiter mit Datei:Zeile-Liste der Call-Sites, konkrete
Empfehlung für die Signatur/Feld-Benennung auf `SlotTO`, Skizze des
ShiftplanWeek-Aggregat-Umbaus.

## Vermutliche Wave-Struktur (final in plan-phase)

- **Wave 1** — `Slot::clip_to(cutoff) -> Option<Slot>` in
  `service/src/slot.rs` + Unit-Tests für alle vier D-04-Fälle. (SHC-01)
- **Wave 2** — BE-Aggregat-Anpassung an zwei Punkten:
  - `BlockServiceImpl` clippt Slots während der Block-Konstruktion, mit
    ShortDay-Lookup pro Datum → deckt Reporting, Booking-Info, iCal,
    MyBlock ab. (SHC-02, SHC-05)
  - `ShiftplanWeek`-DTO-Bauer clippt Slots pro Datum und liefert
    geclippte Zeiten (via `effective_end` o. ä. auf `SlotTO`) → Basis
    für WeekView + PDF. (Vorbereitung SHC-03/04)
- **Wave 3** — FE-Konsum: WeekView + PDF-Renderer nutzen die geclippten
  Slot-Fenster aus dem DTO. Kein FE-eigenes Clipping (D-51-02 / Fat-Backend).
  (SHC-03, SHC-04)

## Deferred Ideas (nicht in v2.4)

- **Tageskopf-Kennzeichnung von ShortDays im WeekView** (z. B. „Cutoff
  14:30" im Header) — falls User später Kürzungsursache nicht erkennen.
- **Warnung bei Booking-Create auf Post-Cutoff-Slot** (Banner, kein
  Dialog) — falls unbeabsichtigte 0h-Buchungen zum UX-Problem werden.

## Canonical Refs

**MUST READ vor planning:**

- `.planning/REQUIREMENTS.md` — SHC-01…SHC-05 vollständige Definition +
  Metadata-Tabelle
- `.planning/notes/shortday-slot-clipping-semantics.md` — D-01…D-06
  Semantik-Decisions
- `.planning/seeds/shortday-slot-clipping.md` — konsumierte Seed mit
  Feature-Skizze
- `.planning/research/questions.md` — Q-01 kanonische Slot-Clip-Funktion
- `.planning/PROJECT.md` — Architektur-Prinzipien (Fat Backend, Thin
  Client), GSD-Scope-Regel (Backend+Frontend), Quellen-Hierarchie

**Codebase-Anker:**

- `CLAUDE.md` (Repo-Root) — Backend-Konventionen (Service-Tier-Konventionen
  Basic vs. Business-Logic Service beachten: `BlockService` ist heute
  Business-Logic-Service; wenn Clip-Logik dort einzieht, bleibt das ok
  solange keine neue Domain-Service-Dep entsteht.)
- `shifty-dioxus/CLAUDE.md` — Frontend-Konventionen (für Wave 3)
- `.planning/codebase/frontend/` — FE-Codebase-Map (für Q-01/WeekView-Pfad)

**Nicht relevant für diese Phase:** keine ADR-Verweise im Repo für dieses
Feature, keine externen Specs.
