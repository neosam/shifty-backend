---
title: Kurzer Tag — Slot-Kürzung Semantik-Entscheidungen
date: 2026-07-04
context: explore session — Feature-Semantik für die dynamische Slot-Kürzung an Kurzen Tagen fixiert, bevor discuss-phase/plan-phase startet
---

# Kurzer Tag — Slot-Kürzung Semantik-Entscheidungen

## Trigger

`/gsd-explore`-Session am 2026-07-04. Feature-Idee: An Kurzen Tagen sollen
Slots am Cutoff enden — Slot 14:00–15:00 bei Cutoff 14:30 wird zu
14:00–14:30, gebuchte Stunden zählen 0,5 statt 1.

Zweck dieses Notes: **die harten Semantik-Entscheidungen anchor** für
spätere `discuss-phase`, damit dort nicht neu diskutiert wird.

Siehe Seed [[shortday-slot-clipping]].

## Decisions

### D-01 — Cutoff-Modell: single time per date

Ein Kurzer Tag = **eine** Uhrzeit an genau **einem** Datum. Keine Split-
Öffnungszeiten (Vormittag/Nachmittag). Das existierende `special_day`-
Modell bleibt unverändert; es wird nicht erweitert.

**Konsequenz:** Die Clip-Funktion nimmt (Slot, Time) und gibt entweder
`None` (Slot komplett raus), den unveränderten Slot oder einen gekürzten
Slot zurück.

### D-02 — Kürzung ist dynamisch (view-layer), nicht persistiert

Die `slot`-Tabelle bleibt unangetastet. Rendering + Reporting wenden den
Cutoff on-the-fly an, indem sie den Special-Day für den betreffenden Tag
lookupen und die Clip-Funktion pro Slot aufrufen.

**Vorteile:**
- Vollständig reversibel: ShortDay löschen → Slots wieder in voller Länge
- Keine Migration
- Kein Snapshot-Schema-Bump (`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12)

**Nachteil:** Clip-Logik muss an mehreren Call-Sites konsumiert werden.
Mitigation: eine kanonische Helper-Funktion.

### D-03 — Nur zukünftig, keine Rückrechnung

Wenn ein ShortDay auf einem *zukünftigen* Datum angelegt wird und dort
schon Bookings existieren, werden die Ist-Stunden dieser Bookings ab
diesem Moment reduziert dargestellt — durch die dynamische Berechnung,
ohne explizite Migration.

Bookings auf *vergangenen* Tagen bleiben unberührt. Historische Snapshots
sind unter Schema-Version 12 erzeugt und dürfen unter denselben Regeln
gelesen werden — der ShortDay-Clip ändert die Berechnung ja nicht
retroaktiv, sondern gilt ab dem Zeitpunkt des Live-Reads. Für persistierte
`billing_period`-Snapshots ändert sich nichts.

### D-04 — Cutoff-Regeln pro Slot

| Slot-Verhältnis zum Cutoff | Verhalten |
|----------------------------|-----------|
| `slot.end <= cutoff` | unverändert |
| `slot.end == cutoff` (exakt) | unverändert (kein Sonderfall) |
| `slot.start >= cutoff` | Slot komplett raus — nicht gerendert, zählt 0 h |
| `slot.start < cutoff < slot.end` | gekürzt: `[slot.start, cutoff]` — gerendert + gezählt |

### D-05 — Wirkt nur auf Ist-Stunden, nicht auf Soll

Der Vertrag des Mitarbeiters (erwartete Stunden pro Tag/Woche) bleibt
unverändert. An Kurzen Tagen sammelt ein Mitarbeiter, dessen gebuchter
Slot gekürzt wird, entsprechend **Minusstunden** im Balance-Konto.

Alternative (Soll schrumpft mit) wurde bewusst verworfen — der User
möchte, dass Kurze Tage die Balance-Rechnung beeinflussen, nicht die
Vertragserwartung.

### D-06 — Verkürzte Slots werden auch verkürzt dargestellt

Im Frontend (WeekView) wird ein gekürzter Slot in seiner tatsächlichen
Länge gerendert (also kürzer). Ob zusätzliche visuelle Markierung
(z. B. anderer Rahmen, Icon) gewünscht ist — offen, in `discuss-phase`
zu klären.

## Offene Fragen für discuss-phase

- Wo lebt die kanonische Clip-Funktion? Kandidaten: `shifty-utils`,
  Methode auf `Slot` in `rest-types`, oder als reine Funktion in
  `service_impl`.
- Betrifft es Booking-Validation? Also: kann jemand einen Slot buchen,
  der **komplett** hinter dem Cutoff liegt, oder wird das schon beim
  Booking-Create abgelehnt?
- Visuelle Markierung gekürzter Slots im WeekView? Reine Längen-Anpassung
  oder zusätzlich Icon/Farbe?
- iCal-Export und PDF-Export (v2.3-Renderer): müssen dieselben
  Clip-Regeln anwenden — gleiche Helper-Funktion oder eigene Impl?
- FE-Konsum: rest-types-Slot-Struct so erweitern, dass die effektive Zeit
  mitgegeben wird, oder rechnet das FE selbst?

## Verweise

- Seed: [[shortday-slot-clipping]]
- Codebase-Mapping-Frage: `.planning/research/questions.md`
