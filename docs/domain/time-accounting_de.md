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

Zentral in `service_impl/src/reporting.rs` (2205 Zeilen), aggregiert
über `sales_person`, `booking`, `extra_hours`, `absence`,
`carryover`, `special_days`.

Interne Reads laufen mit `Authentication::Full` — der REST-Handler hat
den User-Auth bereits geprüft, und das Reporting braucht die Rohdaten
ohne pro-Read-Permission-Guard.

## Wo die Rechnung sichtbar wird

- **Weekly Overview** — Frontend-Page mit Blocken pro Sales Person.
- **My Shifts** — Sales-Person-Eigensicht.
- **Employee Details** — HR-Sicht mit Zeitraum-Selektor.
- **Billing Period Details** — die eingefrorene Version.

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
