---
phase: 51
phase_name: Kurzer-Tag-Slot-Kürzung
milestone: v2.4
created: 2026-07-04
type: context
requirements: [SHC-01, SHC-02, SHC-03, SHC-04, SHC-05, SHC-06]
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

### D-51-06 — Vier BE-Aggregat-Ketten (Research-Update, ersetzt "zwei")

**Ursprüngliche Annahme (falsch):** Zwei BE-Aggregat-Ketten (BlockService
+ ShiftplanWeek).

**Verifiziert im Research (`51-RESEARCH.md`, 2026-07-04):** Es sind **vier**
BE-Aggregat-Ketten, an denen der Clip greifen muss:

1. **Chain B — ShiftplanWeek-DTO-Bauer** (Read-Aggregat: WeekView + PDF)
   `service_impl/src/shiftplan.rs:42-66` (`build_shiftplan_day`).
   Automatisch mit-korrigiert: `pdf_render.rs` konsumiert dasselbe
   `ShiftplanWeek`-Struct via `PdfShiftplanServiceImpl` (`service_impl/src/
   pdf_shiftplan.rs:149-152`).
2. **Chain A' — BlockService** (iCal + insufficient-booked)
   `service_impl/src/block.rs:87-96, 237-269`. NICHT für Balance/Ist-Stunden
   (siehe Chain D).
3. **Chain C — BookingInformation** (weekly summary + booking-conflicts)
   `service_impl/src/booking_information.rs:388-409, 506-525, 680-697`.
   Direkte `slot.to - slot.from`-Arithmetik, unabhängig von BlockService.
4. **Chain D — ShiftplanReportDao** (Balance / Ist-Stunden, raw SQL)
   `dao_impl_sqlite/src/shiftplan_report.rs:77, 114, 147`. Aggregiert
   `SUM(STRFTIME(slot.time_to) - STRFTIME(slot.time_from))` direkt in SQL.
   Umsetzung siehe D-51-08.

Alle vier konsumieren `Slot::clip_to` und respektieren das Stichtag-Gate
aus D-51-07.

**Bug-Discovery (Research):** `service_impl/src/shiftplan.rs:62-66` und
`booking_information.rs:394-401, 512-519` haben schon heute eine
ShortDay-Logik, aber sie **verwirft** den ganzen Slot statt zu clippen
(verletzt D-04). Die Feature-Implementierung ersetzt diesen Bug durch die
korrekte Clip-Semantik. Der Test `test_get_shiftplan_week_with_special_days`
(`service_impl/src/test/shiftplan.rs:251`) muss auf die D-04-Erwartung
angepasst werden.

### D-51-07 — Admin-konfigurierbarer Stichtag (Toggle-basiert)

Ein neuer `ToggleService`-Wert `shortday_slot_clipping_active_from` steuert,
ab welchem Datum die Slot-Kürzung wirkt. Muster **identisch zu HCFG-02**
(`holiday_auto_credit` in v1.7, siehe `service_impl/src/reporting.rs:164-180`):

- **Wert:** `Option<String>` — ISO-8601-Date-String.
- **`None`/leer:** Kürzung deaktiviert, Legacy-Verhalten (kein Clip an
  keinem Datum). Default beim Rollout.
- **`Some(date)`:** Kürzung greift **nur** für ShortDays mit Datum
  `>= date`. Für Bookings vor dem Stichtag wird der rohe (ungeclippte)
  Slot verwendet.

**Rationale:** Ohne Stichtag würde die Live-Berechnung retroaktiv alle
existierenden ShortDay-Einträge respektieren und die angezeigten Ist-Stunden
in allen Stundenkonten der Vergangenheit ändern (User-Feedback 2026-07-04:
„historisch würde sonst die Stunden neu berechnen und sämtliche
Stundenkonten wären verändert"). Persistierte `billing_period`-Snapshots
(Schema 12) bleiben ohnehin unangetastet, aber Live-Balance-Views auf
historische Wochen würden abweichen.

**Konsum-Punkt:** In **jedem** der vier Aggregat-Ketten aus D-51-06 vor
dem Clip-Aufruf: `if booking_date < active_from { use raw slot } else
{ slot.clip_to(cutoff) }`.

**Ort im Repo:** `ToggleService::get_toggle_value(
"shortday_slot_clipping_active_from", ...)`. Der Toggle wird via bestehende
Toggle-DAO/Migration seed. Admin-Editor in Settings analog zu HCFG-02
(`shifty-dioxus/src/i18n/de.rs:1130`-Muster).

**Verworfen:**
- Toggle an/aus (analog `paid_limit_hard_enforcement` v1.6) — zu grob, keine
  Übergangsphase möglich.
- An den existierenden HCFG-Stichtag gebunden — Feiertags- und
  ShortDay-Rollout haben verschiedene Semantiken; separate Konfiguration
  bleibt zukunftssicherer. Konsolidierung optional in Folge-Milestone.

### D-51-08 — Chain D: Rust-Layer-Clipping (nicht SQL-Erweiterung)

Der Balance/Ist-Stunden-Kanal (Chain D, `dao_impl_sqlite/src/
shiftplan_report.rs`) wird nicht durch SQL-JOIN + `MIN()` erweitert,
sondern durch **Rust-Layer-Refaktorierung**:

- DAO liefert Rohdaten pro Slot (statt SQL-`SUM(...)` in der Query).
- `ShiftplanReportServiceImpl` (`service_impl/src/shiftplan_report.rs`)
  aggregiert die Roh-Slots per `Slot::clip_to` + D-51-07-Stichtag-Gate.

**Rationale:** Eine kanonische Clip-Funktion für alle vier Ketten
(A'/B/C/D), Stichtag-Gate lebt an einer Stelle, testbar in Rust.
Snapshot-Immunität (D-03: keine Rückrechnung) wird durch das Stichtag-Gate
sowieso gewahrt; die persistierten `billing_period`-Snapshots werden nie
neu berechnet.

**Verworfen:**
- **Option A (SQL-Change):** `LEFT JOIN special_day + MIN(slot.time_to,
  COALESCE(special_day.time_of_day, slot.time_to))`. Nachteil: Clip-
  Semantik in SQL dupliziert, Stichtag-Gate in SQL, deutlich schwerer
  testbar, Snapshot-Immunität schwerer zu argumentieren.
- **Option C (Chain D vorerst nicht anfassen):** Balance-Konto zeigt
  weiter volle Slot-Dauer. Verworfen, weil der User explizit
  Minus-Stunden auf dem Balance-Konto sammeln möchte (D-05).

**Perf-Note:** Row-Traffic pro Slot steigt (statt server-side SUM), aber
SQLite ist lokal — Balance-Historien über mehrere Jahre bleiben
verschmerzbar. Falls Perf zum Problem wird, spätere Optimierung möglich
(Materialisierung, gestufter Cache), out-of-scope für v2.4.

### D-51-09 — SlotTO-Feld-Design: `effective_to` am `ShiftplanSlotTO`-Wrapper

Das Frontend erhält die geclippte Zeit über ein **neues Feld am
`ShiftplanSlotTO`-Wrapper** (nicht am `SlotTO` selbst):

- **Neu:** `ShiftplanSlotTO { slot: SlotTO, bookings, current_paid_count,
  effective_to: time::Time }` (`rest-types/src/lib.rs:1069-1080`).
- **`SlotTO.to` bleibt roh.** Grund: `SlotTO` ist bidirektional
  (`POST /slot`, `PUT /slot/{id}` in `rest/src/slot.rs:100, 124`) — würde
  `to` mutiert, käme das beim nächsten FE-Edit-Save zurück in die DB und
  korrumpiert die Slot-Definition.
- **Default:** `effective_to = slot.to` wenn kein Cutoff greift; sonst
  `effective_to = cutoff`.
- **Slots komplett hinter Cutoff (`slot.start >= cutoff`):** werden aus
  `ShiftplanDayTO.slots` weggelassen — nicht im DTO enthalten. Kein
  extra `visible: bool`-Feld.

**Präzedenz:**
- `ShiftplanSlotTO.current_paid_count: u8` (rest-types:1079) — gleicher
  Grund: Wrapper trägt Anzeige-Daten, `SlotTO` bleibt persistenz-treu.
- `AbsencePeriodTO.derived_days: f32` (rest-types:1793).
- `VacationBalanceTO.computed_entitled_days: Option<f32>` (rest-types:2167).

**Verworfen:**
- `SlotTO.to` mutieren (Read-Path clippt) — bricht bidirektionale
  Save-Semantik.
- Zusätzliches Feld `SlotTO.effective_to` — koppelt Anzeige an Persistenz;
  Wrapper ist der saubere Ort.

## Locked Constraints (cross-cutting)

- **Fat Backend, Thin Client** (siehe PROJECT.md → Architektur-Prinzipien):
  FE bekommt fertige geclippte Werte im DTO, rechnet nichts selbst.
- **Snapshot-Schema-Version bleibt 12.** Kein `BillingPeriodValueType`
  angefasst, keine Bump-Pflicht nach
  `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Regel (`shifty-backend/CLAUDE.md`).
- **Keine DB-Migration** (Ausnahme: Toggle-Seed für
  `shortday_slot_clipping_active_from` folgt HCFG-02-Präzedenz aus v1.7),
  keine neue Cargo-Dep.
- **Soll-Stunden unberührt.** Nur Ist-Seite der Balance.
- **Nur zukünftig** — Stichtag-Gate aus D-51-07 schützt historische
  Live-Balance-Views; persistierte Snapshots bleiben ohnehin unangetastet.
- **Vier Aggregat-Ketten** (D-51-06): jede Kette respektiert das
  Stichtag-Gate. Chain D wird Rust-Layer-refaktoriert (D-51-08), nicht
  SQL-erweitert.

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

## Open Research — CLOSED

Q-01 wurde durch `51-RESEARCH.md` (2026-07-04) beantwortet. Die Antworten
sind in D-51-06/08/09 verankert.

<details>
<summary>Q-01 (historisch, geschlossen)</summary>

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

</details>

## Vermutliche Wave-Struktur (final in plan-phase)

Aktualisiert nach `51-RESEARCH.md`-Discovery (vier Aggregat-Ketten, Chain D
Rust-Layer, effective_to am Wrapper, Stichtag-Gate D-51-07).

- **Wave 1 — Kanonische Clip-Fn + Stichtag-Toggle** (SHC-01, SHC-06)
  - `Slot::clip_to(cutoff: time::Time) -> Option<Slot>` in
    `service/src/slot.rs:12+` (Signatur aus RESEARCH §Slot Struct).
    Unit-Tests für alle vier D-04-Fälle.
  - Toggle-Seed `shortday_slot_clipping_active_from` (Migration analog
    `holiday_auto_credit` v1.7).
  - Wiederverwendbarer Helper für Stichtag-Gate + Cutoff-Lookup pro Woche
    (Muster: `reporting.rs:188-198`) — als kleine Utility, an allen vier
    Ketten konsumiert.

- **Wave 2 — BE-Aggregat-Ketten** (SHC-02, SHC-05)
  - **Chain B** — `service_impl/src/shiftplan.rs:42-66` — Bug-Fix von
    Filter zu Clip; automatisch WeekView + PDF (Chain B teilt Aggregat
    mit `pdf_render.rs`).
  - **Chain A'** — `service_impl/src/block.rs:87-96, 237-269` — Slot-Clip
    vor Merge; deckt iCal + insufficient-booked ab.
  - **Chain C** — `service_impl/src/booking_information.rs:388-409,
    506-525, 680-697` — Filter zu Clip.
  - **Chain D** — `dao_impl_sqlite/src/shiftplan_report.rs:77, 114, 147`
    + `service_impl/src/shiftplan_report.rs` — DAO liefert Rohdaten,
    Rust-Layer clippt + gated (D-51-08).
  - Jede Kette: Stichtag-Gate aus D-51-07 vor Clip.

- **Wave 3 — DTO + FE + PDF-Konsum** (SHC-03, SHC-04)
  - `ShiftplanSlotTO { …, effective_to: time::Time }` in
    `rest-types/src/lib.rs:1069-1080` (D-51-09).
  - `From<&ShiftplanSlot> for ShiftplanSlotTO`
    (`rest-types/src/lib.rs:1115`) schreibt `effective_to`.
  - FE-Loader `shifty-dioxus/src/loader.rs:101, 154` — `slot.slot.to` →
    `slot.slot.effective_to` bzw. Wrapper-Feld.
  - PDF-Renderer automatisch korrekt: `pdf_render.rs` konsumiert das
    geclippte `service::shiftplan::ShiftplanWeek` (Chain B), keine
    Änderung an `pdf_render.rs` selbst.

- **Wave 4 — Admin-Settings UI** (SHC-06 FE-Anteil)
  - Neuer Settings-Card-Eintrag (`shifty-dioxus/src/page/settings.rs`)
    für `shortday_slot_clipping_active_from` — Muster: HCFG-02
    Card-3-Datepicker (siehe `.planning/milestones/v1.7-ROADMAP.md` und
    aktuellen Settings-Screen). WASM-Datepicker-Caveat D-25-06 beachten.
  - i18n de/en/cs analog HCFG-02 (`shifty-dioxus/src/i18n/*.rs:1120+`).

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
