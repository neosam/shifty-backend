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

## Update 2026-07-01 — v1.11-Fix greift NICHT, entschiedene Lösung = Option 2

**Reproduziert weiterhin** nach dem v2.0.0-Release/Deploy (User live bestätigt, 2026-07-01):
Nach dem Anlegen zeigt das Typ-Dropdown weiter „Feiertag", aber der Button ist disabled; erst
Dropdown weg-und-zurück auf „Feiertag" reaktiviert.

**Warum der v1.11-Fix (Phase 36-02, Option 1) nicht wirkt:** Der Code macht das Vorgesehene —
das `<select>` ist controlled (`settings.rs:646`, `value` aus `sd_type_to_select_value(sd_type)`),
und nach dem Create wird zurückgesetzt (`settings.rs:458-459`: `sd_date_str.set("")` +
`sd_type.set(None)`). ABER Dioxus/WASM schiebt den controlled `value` nicht zuverlässig ins
native `<select>` → Anzeige („Feiertag") und Signal (`None`) laufen auseinander → Button disabled.
**Gleiche Klasse wie der Datepicker-Caveat D-25-06.** Controlled-Select ist hier praktisch wirkungslos.

**Entschiedene Lösung (User, 2026-07-01) = Option 2, verschärft:** Nach erfolgreichem Create
**gar nichts zurücksetzen — weder intern noch im View.** Alle Formular-Daten bleiben stehen
(`sd_type`, `sd_date_str`, ggf. `sd_time`), damit man direkt weitermachen kann (nur ändern, was
man will). Das umgeht den Desync komplett (keine programmatische Select-Änderung mehr), der
Button bleibt aktiv, und man sieht das zuletzt eingetragene Datum.

Konkret: den Reset-Block `settings.rs:458-459` (und einen etwaigen Zeit-Reset) entfernen; nichts
mehr auf `None`/leer setzen. SSR-/Komponenten-Test: mehrfaches Anlegen ohne Dropdown-Toggle,
Daten bleiben nach jedem Create erhalten.

**Ziel:** in **v2.1** aufgenommen (User-Entscheidung 2026-07-01). v2.1 läuft autonom über Nacht,
darum reitet dieser kleine, isolierte Settings-Bugfix als eigene Phase mit (statt separater
Hotfix). Umfang klein: Reset-Block `settings.rs:458-459` entfernen + Test fürs mehrfache Anlegen.

Verwandt: [[2026-06-30-special-days-ui-bearbeiten-einstellungen]] (anderes Thema: Edit), und
der WASM-Datepicker-Caveat D-25-06.
