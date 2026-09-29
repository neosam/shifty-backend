# Stundenkonto — Wie die Balance rechnet

Diese Datei erklärt fachlich, wie das Stundenkonto (auch: Balance) in
Shifty berechnet wird. Für den technischen Deep-Dive siehe
[F07 Reporting & Balance](../features/F07-reporting-balance.md). Für
Randfälle siehe [`edge-cases.md`](./edge-cases.md).

## Die Grundformel

```
balance = worked − expected + carryover
```

Für einen gegebenen Sales Person und einen Zeitraum:

- **`worked`** — tatsächlich verbuchte Stunden.
- **`expected`** — vertraglich erwartete Stunden.
- **`carryover`** — Vorjahres-Saldo, der in dieses Jahr rollt.

Das Ergebnis ist ein positives oder negatives Delta:

- **+5 Stunden:** Fünf Stunden Überstunden.
- **−3 Stunden:** Drei Stunden Minus (nicht genug gearbeitet für die
  Erwartung).

## Was zählt als `worked`?

Ist-Stunden setzen sich zusammen aus:

1. **Bookings** — geplante Schichten (Sales Person × Slot × Datum).
   Jede Booking trägt die Dauer des zugewiesenen Slots bei.
2. **Extra Hours mit Kategorie `ExtraWork`** — Überstunden außerhalb
   des Shiftplans (Sonderdienste, ungeplante Arbeit).
3. **Absences mit "positiver" Kategorie** — Urlaub, Krank, Feiertag,
   Unavailable, VolunteerWork werden als "gearbeitet" behandelt (aus
   Sicht der Balance, damit sie die Erwartung erfüllen).
4. **Custom Extra Hours** — abhängig von der Definition der Kategorie.

**Kategorie-Übersicht:**

| Kategorie | Trägt zu `worked` bei? | Senkt `expected`? | Bemerkung |
| --- | --- | --- | --- |
| Shiftplan (Booking) | Ja | Nein | Regulärer Fall |
| ExtraWork | Ja | Nein | Überstunden |
| Vacation | Ja | Nein | Urlaub gilt als "gearbeitet" |
| SickLeave | Ja | Nein | Krankheit gilt als "gearbeitet" |
| Holiday | Ja | Nein | Feiertag-Auto-Credit |
| Unavailable | Ja | Nein | Verfügbarkeits-Sperre |
| VolunteerWork | Ja | Nein | Ehrenamt |
| **UnpaidLeave** | **Nein** | **Ja** | Sonderfall: senkt Erwartung |
| CustomExtraHours | Definiert | Definiert | Pro Custom-Kategorie |

**Der Sonderfall `UnpaidLeave`:** Statt zur Ist-Seite beizutragen, wird
die Erwartung um die Dauer reduziert. Effekt: Ein Tag unbezahlter
Urlaub bringt die Balance nicht in Minus, sondern verringert die
"was hätte gearbeitet werden müssen"-Zahl.

### Übertrag freiwilliger Stunden auf das bezahlte Konto (Rebooking)

Freiwillige Stunden können auf das bezahlte Stundenkonto **übertragen**
werden. Ein Übertrag über `h` Stunden erhöht die gearbeiteten (bezahlten)
Stunden um `h` und senkt die freiwilligen Stunden um dieselbe Menge — die
Gesamtzahl der geleisteten Stunden bleibt gleich, nur das Konto, auf dem
sie verbucht sind, ändert sich.

Die Freiwilligen-Zusage aus dem Vertrag (`committed_voluntary`) wird von
einem Übertrag **nicht** berührt. Sie ist die Sollgröße und bleibt
unangetastet — ein Übertrag bewegt ausschließlich die Ist-Seite.

Zwei Untergrenzen begrenzen, wie weit ein Übertrag gehen darf, und sie
gehören zu zwei verschiedenen Akteuren:

- **Die Automatik** (Wochen-Job) überträgt höchstens so viel, dass die
  Balance `0` erreicht, und nur den Überschuss über die Zusage. Sie hebt
  die Balance nie ins Plus, und sie lässt die Person nie unter ihre
  Zusage fallen.
- **Ein von HR bestätigter Vorschlag** *darf* die Zusage unterschreiten —
  genau dafür gibt es den Vorschlag —, aber er darf die freiwilligen
  Stunden im Jahr nie unter `0` drücken. Man kann nicht insgesamt −3
  Stunden freiwillig gearbeitet haben.

Beispiel: 5 h bezahlt + 5 h freiwillig zugesagt pro Woche über zwei
Wochen; 3 h gearbeitet in Woche eins, 17 h in Woche zwei. Das Konto steht
auf 8 h bezahlt / 12 h freiwillig, Balance −2. Ein Übertrag von 2 h
gleicht es auf 10 h / 10 h aus, Balance 0. Selbst bei 23 h statt 20 h
gearbeitet bliebe der Übertrag bei 2 h, nicht 5 — der Balance-Deckel
bindet, und die 3 Überschuss-Stunden bleiben freiwillig.

Für die Mechanik — wer das Paar schreibt, wann die Automatik läuft, wie
der Deckel gerechnet wird — siehe
[F14 §3 / §9.1](../features/F14-rebooking.md). Zu beachten: freiwillige
Stunden zählen in der Wochensicht in die Balance, im Jahresreport aber
nicht; dieser vorbestehende Unterschied zwischen den beiden
Aggregations-Pfaden ist in
[F07 §2.6](../features/F07-reporting-balance.md) dokumentiert.

### Ist/Soll-whole-week-out-Symmetrie für Freiwilligen-Stunden (v2.6.1)

Sowohl die wöchentliche Freiwilligen-Anzeige
(`WeeklySummary.committed_voluntary_hours` in
`service_impl/src/booking_information.rs`) als auch das Range-basierte
Freiwilligen-Soll (`committed_voluntary_target_in_range` in
`service_impl/src/reporting.rs`) behandeln jede Kalenderwoche mit
mindestens einem Absence-Tag desselben Freiwilligen als **komplett
raus** — der Freiwilligen-Beitrag dieser Woche fällt auf **beiden**
Seiten (Ist und Soll) auf `0`. Die Regel ist kategorie-agnostisch:
Vacation, SickLeave und UnpaidLeave lösen sie identisch aus. Im
selben Rollout klammert auch der `contract_weeks`-Nenner hinter
`ist_per_contract_week` Absence-Wochen aus, damit der Durchschnitt
nicht durch Wochen verdünnt wird, die für Freiwilligenarbeit gar
nicht verfügbar waren.

Die Weekly-Anzeige hatte diese Regel bereits seit v2.6.0 (D-26-03 /
VFA-01); v2.6.1 gleicht das Range-Soll und den
Contract-Weeks-Nenner an (D-54.5-01 / D-54.5-02). Siehe
[F14](../features/F14-rebooking.md) für das Range-Soll und F03
(booking_information) für die Weekly-Anzeige.

## Was ist `expected`?

Erwartung ergibt sich aus:

1. **Contract-Zeilen** (`employee_work_details`) — Wochenstunden verteilt
   auf Wochentage.
2. **Special Days** — Feiertage oder betriebliche Sondertage senken
   die Erwartung an dem Tag.
3. **UnpaidLeave** (siehe oben) — senkt die Erwartung im gebuchten
   Zeitraum.

**Grundschema pro Tag:**

```
expected(day) = if working_day(contract, day) then
                    hours_per_day(contract)
                    − special_day_reduction(day)
                    − unpaid_leave_reduction(sales_person, day)
                else
                    0
```

**Aggregation auf Zeitraum:**

```
expected(from..to) = Σ expected(day) für day in from..to
```

### Zweierlei „Soll": bilanzrelevantes Soll vs. Planungs-Soll

Seit Phase 64 trägt der Wochen-Kurzreport **zwei** Soll-Größen. Sie
entstehen aus denselben drei Quellen (Vertrag, Abwesenheiten,
Feiertage), beantworten aber verschiedene Fragen — und sie verhalten
sich bei genau einer Vertragsart unterschiedlich: beim dynamischen.

| | `expected_hours` — *bilanzrelevantes Soll* | `planning_target_hours` — *Planungs-Soll* |
| --- | --- | --- |
| Wofür der Wert da ist | das `expected`-Bein der Grundformel oben | die Zahl, gegen die ein Planer die verteilten Schichten hält |
| Dynamischer Vertrag | absichtlich **`0`** | absichtlich **nicht `0`** |
| Geht in die Balance ein? | **ja** — es *ist* die Erwartung der Bilanz | **nein**, nie; auch nicht in Carryover, Billing-Snapshot oder Verfügbarkeit |
| Wo der Wert sichtbar wird | die Stundenkonto-Flächen (Employee Details, Wochen-Histogramm, Mitarbeiterliste) | ausschließlich die Übersicht unterhalb des Schichtplans |

**Der eine Satz zum Merken:** Bei dynamischen Verträgen ist das
bilanzrelevante Soll absichtlich Null, weil *Soll = Ist* gilt; und das
Planungs-Soll ist absichtlich nicht Null, weil man ohne Zielwert nicht
planen kann.

Nicht-dynamische Verträge sind unberührt: dort tragen beide Größen
dieselbe Zahl, konstruktionsbedingt und nicht zufällig. Die Begründung —
warum das Planungs-Soll ein eigenes Reduktions-Gate braucht und warum
dieses nicht wieder mit dem Bilanz-Gate „vereinheitlicht" werden darf —
steht in [F07 §2.7](../features/F07-reporting-balance.md). Hier nicht
nachbauen; sie gehört an genau eine Stelle.

## Was ist `carryover`?

**Carryover** ist der eingefrorene Jahresend-Saldo aus dem Vorjahr.

Beispiel: Am 31.12.2025 hatte der Sales Person `+8` Balance-Stunden.
Dieser Wert wird als `carryover(2025)` persistiert. Wenn 2026 die
Balance ausgerechnet wird, ist der Startwert nicht 0, sondern die 8
Stunden vom Vorjahresende.

**Warum das Muster?** Ohne Carryover müsste jeder Report seit
Betrieb-Beginn alles rekalkulieren. Mit Carryover reicht das
laufende Jahr.

**Wann Carryover geschrieben wird:** Der Scheduler
(`service_impl/src/scheduler.rs:60,68`) ruft
`update_carryover_all_employees(year-1, Full)` und
`update_carryover_all_employees(year, Full)` zeitgetrieben. Beide
Jahre werden aktualisiert — das aktuelle wird nachjustiert, wenn
rückwirkende Änderungen eingehen.

## Zeit-Skalen

Die Balance kann für unterschiedliche Zeitfenster berechnet werden:

- **Pro Tag** — Basis-Einheit.
- **Pro Kalenderwoche** — Standardansicht (Block-Report).
- **Pro Monat** — HR-Sicht.
- **Pro Jahr** — Ganzjahresbilanz.
- **Ad-hoc-Range** — beliebiges `[from, to]`.

Aggregation ist additiv über die Tage.

## Weekly Cap

**[Verifiziert per F07-Doku]** Es gibt einen "weekly cap"-Mechanismus:
Die Balance in einer Woche wird begrenzt, damit Extremwerte in einer
einzelnen Woche das Bild nicht verzerren. Details:
`apply_weekly_cap` in `reporting.rs` — siehe F07.

## Vacation Balance separat

Der Urlaubs-Saldo ist eine parallele Rechnung, mit derselben
Carryover-Idee, aber auf Urlaubstagen statt Stunden:

```
vacation_balance = entitled + carryover(year−1) − (used + planned) + offset
```

Details: [F06](../features/F06-vacation-management.md).

## Wo die Rechnung passiert

Zentral in `service_impl/src/reporting.rs` (3261 Zeilen), aggregiert
über `sales_person`, `booking`, `extra_hours`, `absence`,
`carryover`, `special_days`.

Interne Reads laufen mit `Authentication::Full` — der REST-Handler hat
den User-Auth bereits geprüft, und das Reporting braucht die Rohdaten
ohne pro-Read-Permission-Guard.

Die Pro-Woche-Größen des Kurzreports entstehen nicht an einer Stelle,
sondern an **zwei** — `assemble_weeks` (geteilt von Einzelwochen- und
Jahres-Read) und `get_reports_for_all_employees`. Das **Planungs-Soll**
entsteht an beiden, genau wie jede andere Kurzreport-Größe; nur seine
Reduktions-Bedingung unterscheidet sich bewusst von der der
Bilanz-Formel. Begründung und Konsequenzen:
[F07 §2.7 und §3](../features/F07-reporting-balance.md).

## Wo die Rechnung sichtbar wird

- **Weekly Overview** — Frontend-Page mit Blocken pro Sales Person.
- **My Shifts** — Sales-Person-Eigensicht.
- **Employee Details** — HR-Sicht mit Zeitraum-Selektor.
- **Billing Period Details** — die eingefrorene Version.
- **Übersicht unterhalb des Schichtplans** — die einzige Fläche, die das
  **Planungs-Soll** statt des bilanzrelevanten Solls zeigt: die
  `Ist / Soll`-Zahl, die Auslastungsbalken und die Soll-Summe der
  Fußzeile. Alles, was das Stundenkonto zeigt — Employee Details,
  Wochen-Histogramm, Mitarbeiterliste, jede Balance-Zahl — zeigt
  weiterhin das bilanzrelevante Soll.

## Warum das kompliziert ist

Weil viele Randbedingungen zusammenkommen:

- Contract-Wechsel mitten in der Woche.
- Special Days am Wochenende (ändern nichts, wenn kein Werktag).
- Absences über Jahreswechsel (müssen den Carryover-Timepoint kreuzen).
- Toggle-Rollouts (Stichtag-basiert — vor/nach Stichtag verschiedene
  Rechnungswege).
- Snapshot-Frozen-vs-Live-Diff (Billing Period vs Live-Reporting).

Vor jeder Änderung an der Balance-Rechnung: **lies
[`edge-cases.md#1-stundenkonto`](./edge-cases.md#1-stundenkonto)**.
