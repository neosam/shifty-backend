---
created: 2026-06-29T14:30:00.000Z
title: Urlaubs-Balken pro Person konsistent zu Resturlaub-Zahl machen
area: absence
files:
  - shifty-dioxus/src/page/absences.rs:843-898
  - shifty-dioxus/src/state/vacation_balance.rs:10-27
  - service/src/vacation_balance.rs:43
---

## Problem

In der Absences-Seite unter „Pro Person · sortiert nach verbleibenden Tagen"
(`VacationPerPersonHeader`) widersprechen sich der Balken und die Zahl daneben —
sie messen zwei verschiedene Dinge:

- **Zahl (rechts):** `remaining_days = entitled + carryover − used − planned`
  (Backend-Formel, dokumentiert in `service/src/vacation_balance.rs:43`).
- **Balken:** `used_pct = used_days / (entitled + carryover)`, geclamped auf 0..100 %
  (`PersonVacationCard`, `shifty-dioxus/src/page/absences.rs:865-871`). Der Balken
  zählt also NUR `used_days` und ignoriert `planned_days` (zukünftig eingetragener,
  noch nicht genommener Urlaub).

Folge: Eine Person kann **−1 Resturlaub** anzeigen (Anspruch überzogen, weil
used + planned > Anspruch), während der Balken nur bei ~⅓ steht (erst ein Drittel
tatsächlich *genommen*). Beispiel: entitled+carryover = 18, used = 6 → Balken 33 %;
planned = 13 → remaining = 18 − 6 − 13 = −1. Optisch liest sich „−1 bei ⅓" falsch.
Zusätzlich läuft der Balken nie über 100 % (clamp), selbst wenn `used` allein den
Anspruch übersteigt.

## Solution

Balken und Zahl konsistent machen. Optionen (TBD, eine wählen):

1. **Balken auf `(used + planned) / total` umstellen** — dann passt der Füllstand
   zur Resturlaub-Zahl. Einfachste Variante.
2. **Zwei-Segment-Balken:** genommen (`used`) vs. geplant (`planned`) als zwei
   Abschnitte, damit beide Größen sichtbar bleiben. Informativer, mehr Aufwand.
3. **Überzug sichtbar machen:** bei `remaining_days < 0` den Balken voll + in
   Warnfarbe rendern (aktuell springt die Farbe nur über `remaining_days <= 3.0`
   auf `text-warn`/`bg-warn` — greift bei −1 zwar schon, erklärt aber den ⅓-Stand
   nicht).

Empfehlung: Variante 1 oder 2. Bei der Gelegenheit prüfen, ob die Clamp-Obergrenze
weg soll, damit Überzug erkennbar ist. Nur Frontend-Änderung (kein Backend), die
Felder `used_days`/`planned_days`/`remaining_days` liegen bereits in `VacationBalance`
(`shifty-dioxus/src/state/vacation_balance.rs`).
