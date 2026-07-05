# Absence-System (Fachlich)

Diese Datei erklärt das range-basierte Abwesenheits-System aus Fach-Sicht.
Für die technische Referenz siehe
[F05 Absence System](../features/F05-absence-system.md).

## Was ist eine Absence?

Eine **Absence** ist eine Abwesenheitsperiode eines Sales Person mit
einem **Beginn**, einem **Ende** und einer **Kategorie** (Urlaub,
Krank, Unbezahlt, Ehrenamt, …).

**Range-Semantik:** beidseitig inklusiv — `[from, to]`. Ein Range
`from=2026-06-01`, `to=2026-06-01` ist "genau ein Tag Urlaub".

## Warum Range statt einzelne Tage?

Vor v1.0 wurden Abwesenheiten als **Extra-Hours-Zeilen pro Tag**
geführt. Das führte zu:

- **Datenexplosion:** Zwei Wochen Urlaub = 14 Zeilen.
- **Änderungs-Aufwand:** Urlaub um einen Tag verlängern = neue Zeile
  einfügen und Aggregate refreshen.
- **Semantik-Verlust:** "Diese 14 Zeilen gehören zusammen" war nur im
  Kopf des Users.

Das Absence-System macht daraus **eine Zeile**, die 14 Tage abdeckt.

## Kategorien

Absences haben dieselben Kategorien wie Extra Hours (siehe
[`glossary.md`](./glossary.md)):

- **Vacation** — Urlaub. Zählt als "gearbeitet" für Balance.
- **SickLeave** — Krank. Zählt als "gearbeitet" für Balance.
- **UnpaidLeave** — Unbezahlt. Senkt die Erwartung, addiert nichts.
- **VolunteerWork** — Ehrenamt. Zählt als "gearbeitet".
- **Unavailable** — Verfügbarkeits-Sperre.
- **Holiday** — Feiertag als individuelle Absence (selten; meist über
  Special Days).

Die Semantik für die Balance-Rechnung ist identisch mit Extra Hours —
Reporting aggregiert beide Quellen.

## Cutover-Historie

Der Wechsel von Extra-Hours-basierten Single-Day-Zeilen zu Absence-
Ranges heißt **Cutover** und passierte in Milestone v1.0.

Vor Cutover: Alle Abwesenheiten in `extra_hours`.
Ab Cutover: Neue Abwesenheiten in `absence_period`. Alte bleiben in
`extra_hours` und werden nicht migriert (Ausnahme: `absence_conversion`
für explizite Konvertierung).

**Konsequenz für Reporting:** Alle Reader-Pfade, die
Absence-relevante Kategorien aggregieren (Vacation, SickLeave,
UnpaidLeave, VolunteerWork), MÜSSEN aus **beiden** Quellen lesen und
zusammenführen — sonst fehlen historische Zeilen.

## Konflikt-Semantik

### Absence-vs-Absence (Overlap)

- **Same-Category-Overlap:** Verboten — z.B. zwei überlappende Urlaubs-
  Ranges für denselben Sales Person. Der Service lehnt den Insert ab.
- **Cross-Category-Overlap:** Erlaubt mit Priorität —
  `SickLeave > Vacation > UnpaidLeave`. Das heißt: Wenn Urlaub und
  Krank sich überlappen, wird der überlappende Zeitraum als
  Krankheit gezählt.

### Absence-vs-Booking

Nicht-blockierend. Es gibt eine **Warning** ("Da ist ein Booking
während der Absence"), aber der Insert wird nicht abgelehnt. Der User
kann entscheiden, ob die Booking gelöscht werden soll.

Fach-Motivation: In der Praxis werden Bookings oft im Voraus geplant,
und ein spontaner Kranktag soll nicht das Anlegen der Absence
verhindern.

### Absence über Nicht-Arbeitstag

Ein Urlaubs-Range, der einen Sonntag einschließt, den der Sales Person
per Contract nicht arbeitet, zählt für den Sonntag **0 Stunden**. Der
Range ist trotzdem gültig.

## Auth-Modell

- **HR** kann Absences für alle anlegen, ändern, löschen.
- **Sales Person selbst** kann eigene Absences anlegen, ändern,
  löschen (mit gewissen Einschränkungen; **[Zu prüfen]** genaue
  Regeln in F05).
- **`find_all`** ist ausschließlich HR — Sales-Person-eigene Reads sind
  gefiltert auf eigene Zeilen.

## Konvertierung: Legacy → Absence

Der `AbsenceConversionService` (`service_impl/src/absence_conversion.rs`)
ist der Weg, um Alt-Extra-Hours-Zeilen aktiv in Absence-Ranges zu
überführen. Nur ein einmaliger Datenumzug — nicht Teil des
Live-Reporting-Pfads.

## Toggle-Rollout-Kette

Das Feature-Rollout D-51-07/HCFG-02 nutzt Toggles mit Stichtag: Vor
Stichtag alte Semantik (nur Extra Hours), nach Stichtag neue Semantik
(Absence-System aktiv).

**Konvention (aus Memory):** Pro Konsumkette wird bei "Toggle aus" die
alte Semantik im Gate-aus-Zweig rekonstruiert — nicht blind "None →
raw" annehmen.

## Balance-Rechnung mit Absence

Für einen Zeitraum + Sales Person zählt Reporting:

1. Alle Bookings (immer).
2. Alle Extra-Hours-Zeilen (immer, auch nach Cutover für Legacy-Daten).
3. Alle Absences (nach Cutover).

Die Kategorien werden in ihrer Standard-Semantik behandelt (siehe
[`time-accounting.md`](./time-accounting.md)):

- Vacation/SickLeave/VolunteerWork/Unavailable/Holiday: addieren auf
  Ist-Seite.
- UnpaidLeave: senkt Erwartung.

## Randfall-Referenzen

Siehe [`edge-cases.md#2-absence--extra-hours`](./edge-cases.md#2-absence--extra-hours):

- Range über Billing-Period-Grenze (Split?).
- Range über Jahreswechsel (Carryover-Interaktion).
- Cross-Category-Overlap (Priorität).
- Absence auf Nicht-Arbeitstag.
- Absence gegen Booking-Konflikt (Warning, nicht Block).

## PR-Review-Muster

**Bei Änderungen an `reporting.rs` oder `absence.rs`:**

1. Werden **beide** Quellen (`extra_hours` + `absence_period`) für
   Absence-relevante Kategorien gelesen?
2. Ist der Toggle-Rollout-Gate-aus-Zweig für die betroffene
   Konsumkette gepflegt?
3. Gibt es Tests für Jahreswechsel-Ranges und Carryover-Interaktion?

Ohne diese Checks driftet die Cutover-Konsistenz still.
