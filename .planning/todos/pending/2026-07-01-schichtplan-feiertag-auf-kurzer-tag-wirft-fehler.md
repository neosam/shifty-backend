---
created: 2026-07-01T04:35:00.000Z
title: Schichtplan — Tag von "Feiertag" auf "Kurzer Tag" umstellen wirft Fehlermeldung
area: shiftplan
resolves_phase: 43
files:
  - shifty-dioxus/
---

## Problem

Im Schichtplan: Wenn ein Tag zuerst auf "Feiertag" (Special Day) gesetzt wird und
anschließend auf "Kurzer Tag" umgestellt wird, führt das zu einer Fehlermeldung.

Vermutlich ein Konflikt beim Wechsel des Special-Day-Typs auf demselben Datum:
Es existiert bereits ein Special-Day-Eintrag für den Tag (Typ Feiertag), und der
Umstellen-Pfad legt einen neuen an bzw. verletzt eine Unique-/Konflikt-Bedingung,
statt den bestehenden Eintrag zu aktualisieren (update vs. insert).

Reproduktion:
1. Im Schichtplan einen Tag auf "Feiertag" setzen.
2. Denselben Tag auf "Kurzer Tag" umstellen.
3. → Fehlermeldung.

## Solution

TBD — Fehler reproduzieren und die genaue Meldung/den Statuscode erfassen (Backend-Log
+ Netzwerk-Response prüfen). Wahrscheinlich muss der Umstell-Pfad den bestehenden
Special-Day-Eintrag für das Datum aktualisieren (oder erst löschen, dann neu anlegen)
statt einen zweiten Eintrag zu erzeugen. Verwandt mit dem Dropdown-Desync-Bug
[[2026-06-30-settings-special-days-anlegen-button-disabled]] und der Special-Days-UI
[[2026-06-30-special-days-ui-bearbeiten-einstellungen]] prüfen.
