---
title: Kurzer Tag — Slot-Kürzung am Cutoff
trigger_condition: v2.3 archiviert / neuer Milestone v2.4 startet
planted_date: 2026-07-04
---

# Kurzer Tag — Slot-Kürzung am Cutoff

## Trigger

Sobald v2.3 (PDF-Export) sauber archiviert ist und der nächste Milestone
(vermutlich v2.4) aufgemacht wird: **`/gsd-new-milestone`** bzw.
`/gsd-phase` mit diesem Feature als erstem Kandidaten. Kein Blocker, aber
konzeptionell abgeschlossen und planungsbereit.

## Skizze: Feature

An einem Kurzen Tag (existierendes `special_day`-Modell — genau **eine**
Cutoff-Uhrzeit auf einem konkreten Datum) sollen Slots, die den Cutoff
überlappen, gestutzt werden — sowohl im Schichtplan-Rendering als auch bei
der Stunden-Berechnung.

**Beispiel:** Kurzer Tag bis 14:30, Slot 14:00–15:00
→ gerendert + gezählt als 14:00–14:30
→ gebuchte Stunden zählen 0,5 h statt 1 h.

## Fixierte Semantik (User-bestätigt in Explore 2026-07-04)

Ausführliches Decision-Log: siehe [[shortday-slot-clipping-semantics]].

Kurzfassung:

- **Cutoff-Regeln pro Slot:**
  - Endet ≤ Cutoff → unverändert
  - Beginnt ≥ Cutoff → komplett raus (nicht gerendert, zählt nicht)
  - Überlappt Cutoff → gekürzt auf `[start, cutoff]` (gerendert + gezählt)
- **Kürzung dynamisch (view-layer)** — Slot-DB bleibt unangetastet, kein
  destruktives Rewrite. ShortDay lässt sich wieder entfernen und Slots
  erscheinen automatisch wieder in voller Länge.
- **Nur zukünftig** — Bookings auf Vergangenheits-Tagen bleiben, wie sie
  waren. Kein Snapshot-Schema-Bump nötig
  (`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt bei 12).
- **Wirkt nur auf Ist-Stunden**, nicht auf Soll-Stunden. Der Vertrag des
  Mitarbeiters ändert sich nicht — an Kurzen Tagen sammelt der Mitarbeiter
  ggf. Minusstunden im Balance-Konto, wenn er auf einen gekürzten Slot
  gebucht war.
- **Bookings überleben** die Einführung eines ShortDays; ihre gezählten
  Stunden schrumpfen automatisch, weil die Berechnung dynamisch am Slot
  hängt.

## Skizze: Betroffene Oberflächen

Bei Plan-Phase per Code-Mapping zu verifizieren (siehe offene
Research-Question in `.planning/research/questions.md`):

- Frontend WeekView-Slot-Rendering (Zellenhöhe/Label)
- Reporting-Service (Balance/Ist-Stunden)
- Booking-Information-Service (Dashboard-Aggregates)
- ggf. PDF-Export (v2.3-Renderer, gerade neu — 5-Parameter-Signatur)
- ggf. iCal-Export
- Booking-Validation: darf man einen ganz-außerhalb-Slot noch buchen?
  (nicht in Explore geklärt, offen für Discuss-Phase)

## Erste Skizze: mögliche Phasen-Struktur

- **Phase A (Backend):** Kanonische Clip-Funktion in
  `shifty-utils` oder als Method auf `Slot`/via `SlotService`;
  Reporting + Booking-Information konsumieren sie; Tests für alle drei
  Cutoff-Fälle.
- **Phase B (Frontend):** WeekView + PDF-Export nutzen dieselbe
  Clip-Semantik (rest-types oder eigene FE-Funktion), verkürzte Slots
  visuell markieren.

Alternativ als **eine vertikale MVP-Phase** über beide Enden, wenn die
Clip-Funktion klein genug bleibt.

## Verweise

- Semantik-Decisions: [[shortday-slot-clipping-semantics]]
- Codebase-Mapping-Frage: `.planning/research/questions.md`
- Existierendes Modell: `special_day` (Enum-Variante `ShortDay { until: Time }`)
