---
created: 2026-06-26T17:30:19.468Z
title: Einzelnen Slot nur für eine KW ändern (statt „ab KW")
area: shiftplan
files:
  - shifty-backend/shifty-dioxus/src/component/slot_edit.rs
  - shifty-backend/shifty-dioxus/src/state/slot_edit.rs
  - shifty-backend/rest-types/src/lib.rs (SlotTO, valid_from/valid_to)
---

## Problem

Die Struktur des Schichtplans (Slots) lässt sich aktuell nur **„ab einer KW"**
verändern: Eine Änderung an einem Slot gilt ab der gewählten Kalenderwoche und für
alle Folgewochen (Versionierung über `valid_from`/`valid_to`, vgl. Hinweis im
`SlotEditInner`-Dialog zu `valid_to`).

Es gibt aber Fälle, in denen man **nur einen einzelnen Slot in genau einer (z.B. der
aktuellen) Woche** anpassen möchte — eine einmalige Ausnahme, ohne die wiederkehrende
Struktur ab dieser KW dauerhaft zu verändern. Dafür fehlt bisher eine Lösung.

## Solution

TBD — mehrere Ansätze denkbar, die Frage ist, welcher fachlich/technisch am besten passt:

- **A) Einmalige Slot-Override pro Woche:** Ein Slot bekommt für eine konkrete KW eine
  Override-Instanz, die nur diese Woche überschreibt; die wiederkehrende Struktur bleibt
  unverändert. (Neues Datenmodell-Konzept „week-specific override".)
- **B) Split + sofortiges Re-Merge:** Intern „ab dieser KW" ändern und „ab Folge-KW"
  wieder auf den alten Stand zurücksetzen (zwei valid_from-Schnitte), sodass nur die
  eine Woche abweicht. Nutzt das bestehende Versionierungs-Modell, erzeugt aber mehr
  Slot-Versionen.
- **C) UI-Wahl „nur diese Woche" vs. „ab dieser Woche":** Im Slot-Editor eine explizite
  Auswahl anbieten; der gewählte Modus steuert, welcher der obigen Mechanismen greift.

Offene Punkte: Datenmodell-Auswirkungen (Slot-Versionierung), Reporting/Balance-
Konsistenz, Buchungen die an der geänderten Slot-Version hängen, UX im Editor.
→ Vor Umsetzung Ansätze gegeneinander abwägen (eigene Discuss-/Spec-Phase sinnvoll).
