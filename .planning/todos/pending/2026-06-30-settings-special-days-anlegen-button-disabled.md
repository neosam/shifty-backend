---
created: 2026-06-30T20:40:41.560Z
title: Settings Special-Days — Anlegen-Button bleibt nach erstem Feiertag disabled (Typ-Dropdown-Desync)
area: frontend
files:
  - shifty-dioxus/src/page/settings.rs:332
  - shifty-dioxus/src/page/settings.rs:359
  - shifty-dioxus/src/page/settings.rs:380
  - shifty-dioxus/src/page/settings.rs:417
  - shifty-dioxus/src/page/settings.rs:614
---

## Problem

In der **Einstellungen → Special-Days-Karte** (Phase 33): Wenn man **mehrere Feiertage
hintereinander** eintragen will, bleibt der **Anlegen-Button nach dem ersten Anlegen
disabled**. Workaround des Users: das **Typ-Dropdown** muss erst weg von „Feiertag" und
wieder zurück auf „Feiertag" geschaltet werden, damit der Button wieder aktiv wird.

**Live reproduziert vom User (2026-06-30).** Hängt mit dem beim Phase-33-Close deferred
visuellen Smoke „Add-Button-Disabled-Rendering" zusammen.

### Vermutete Ursache
Der Enable-Zustand des Buttons hängt am `sd_type`-Signal (`settings.rs:332`, Default `None`;
Enable-Prädikat `:359`; Create-Handler verlangt `Some(day_type)` `:380`). Nach einem
erfolgreichen `create_special_day` (`:417`) wird das Formular vermutlich zurückgesetzt
(`sd_type` → `None`), aber das `<select>`-Dropdown **zeigt weiterhin „Feiertag"** an
(Controlled-vs-Uncontrolled-Desync) → Button disabled, obwohl visuell „Feiertag" gewählt ist.
Erst das Umschalten feuert `onchange` (`:614`: "holiday" → `Some(Holiday)`) und re-aktiviert.
Ggf. zusätzlich das D-25-06-Datepicker-Signal-Caveat im Spiel (Datum-Input-Wert vs.
Dioxus-Signal nach Reset).

## Solution

1. Reset-Pfad nach erfolgreichem Create prüfen: `sd_type` (und Datum) konsistent zurücksetzen
   **und** das `<select>` als **controlled** an `sd_type` binden (`value`/`selected` aus dem
   Signal ableiten), damit Anzeige und Signal nie auseinanderlaufen.
2. Alternativ: nach Create `sd_type` NICHT auf `None` setzen, sondern auf den zuletzt
   gewählten Typ (Default `Holiday`) belassen — dann ist der Button für den nächsten Eintrag
   sofort aktiv (nur das Datum muss neu).
3. D-25-06 mitdenken: falls das Datum-Feld nach Reset einen stale Signal-Wert hat, dort
   ebenfalls den Reset/Re-Trigger absichern.
4. SSR-/Komponenten-Test ergänzen (mehrfaches Anlegen ohne Dropdown-Toggle), damit es nicht
   re-regrediert.

Verwandt: [[2026-06-30-special-days-ui-bearbeiten-einstellungen]] (anderes Thema: Edit), und
der WASM-Datepicker-Caveat D-25-06.
