---
created: 2026-07-02T12:09:44.871Z
title: Special-Day Duplikat-Warnung nach Create ausblenden, erst bei Änderung wieder
area: ui
files:
  - shifty-dioxus/src/page/settings.rs:522-523 (sd_is_duplicate computed)
  - shifty-dioxus/src/page/settings.rs:833 (inline "existiert bereits"-Hinweis render)
  - shifty-dioxus/src/page/settings.rs:596-601 (Create-Success-Arm, retention)
  - shifty-dioxus/src/page/settings.rs:770,795,815 (on_change der 3 Formularfelder)
---

## Problem

Seit Phase 42 (D-42-01) bleiben die Formularfelder nach erfolgreichem Special-Day-Anlegen
gefüllt (Typ/Datum/Zeit), damit der „Anlegen"-Button aktiv bleibt. Nebeneffekt: der gerade
angelegte Feiertag matcht nun sich selbst → der Inline-Hinweis „existiert bereits"
(`sd_is_duplicate`, gerendert bei `settings.rs:833`) erscheint **sofort nach dem Create**.

Das ist verwirrend — der User hat den Eintrag gerade selbst angelegt und bekommt direkt eine
„existiert bereits"-Warnung. In Phase 42 (D-42-03) wurde das bewusst als harmlos akzeptiert
(der Hinweis bleibt informativ, ist nicht an `button.disabled` gekoppelt), aber UX-seitig
soll die Warnung nach dem Anlegen **unterdrückt** und **erst wieder eingeblendet** werden,
sobald der User ein Formularfeld ändert.

## Solution

Ein „gerade-angelegt"-Suppress-Flag einführen (analog zum bestehenden pure-fn-Muster):
- Neues Signal, z.B. `sd_dup_hint_suppressed: bool`, im Create-Success-Arm (`~596-601`, wo
  `special_day_form_after_create` die Felder retained) auf `true` setzen.
- In den `on_change`-Handlern der drei Felder (`sd_date_str` ~770, `sd_type` ~795,
  `sd_time_str` ~815) das Flag wieder auf `false` setzen → Warnung erscheint erst nach der
  ersten echten Änderung wieder.
- Render-Gate bei `settings.rs:833` von `if sd_is_duplicate` auf
  `if sd_is_duplicate && !sd_dup_hint_suppressed` ändern.

Test-Strategie konsistent mit D-42-05: die Suppress-Regel als reine Funktion/Prädikat
extrahieren (z.B. `should_show_duplicate_hint(is_duplicate, suppressed) -> bool`) und
unit-testen (nach Create → false; nach Feld-Änderung → wieder is_duplicate). Controlled-Select
(D-06/D-08) unberührt lassen — nur Sichtbarkeit des Hinweises steuern, keine Felder leeren.

Klein, FE-only, kein Backend/Snapshot/Migration. Kandidat für einen schnellen Folge-Fix
(`/gsd-quick` oder als SDF-Requirement im nächsten Milestone).
