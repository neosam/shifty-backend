---
phase: 64-planungs-soll-f-r-dynamische-vertr-ge
plan: 05
subsystem: reporting
tags: [rust, reporting, planning-target, committed-voluntary, tdd, docs]
status: complete

requires:
  - phase: 64-01
    provides: "Planungs-Soll-Feld an beiden ShortEmployeeReport-Konstruktionsstellen, Gate an dynamic_hours, funf Integrationstests"
  - phase: 64-02
    provides: "drei Frontend-Anzeigestellen + Quellcode-Invariantentest (PSOLL-02)"
  - phase: 64-03
    provides: "Doku-Sektionen F07 2.7 / 6.5 und time-accounting (mit der ALTEN, jetzt korrigierten Definition)"
provides:
  - "potential_workdays_for_week — relation-Ermittlung als Pure-fn, wortwoertlich aus weight_for_week extrahiert"
  - "committed_voluntary_weight_for_week — Wochen-Betrag der ehrenamtlichen Zusage (committed_voluntary * relation)"
  - "Planungs-Soll enthaelt die ehrenamtliche Zusage, an beiden Konstruktionsstellen wortgleich (D-64-05)"
  - "Neues Gate an planning_base_for_target — 0h-Vertrag mit Zusage zeigt erstmals ein Planungs-Soll > 0"
  - "Proportionale Reduktion ueber target_reduction_scale (D-64-07)"
  - "Neun neue Tests (funf Integration, vier Pure-fn) inkl. No-Clamp-Sperre auf einen negativen Wert"
  - "Korrigierte Soll-Abgrenzung in F07 + time-accounting, EN und DE"
  - "64-UAT.md mit funf Szenarien und zwei Zusatzfragen, nichts abgehakt"
affects: [reporting, billing-period, shiftplan-uebersicht, uat-64]

tech-stack:
  added: []
  patterns:
    - "Eine Quelle fuer relation: potential_workdays_for_week speist weight_for_week UND committed_voluntary_weight_for_week — die Skalierungen koennen nicht abdriften"
    - "target_reduction_scale faellt bei Zusage 0 strukturell auf 1.0 — Byte-Identitaet wird beweisbar statt zufaellig"
    - "Gegen-Assertions im Test: der Wert ist NICHT 3.0 und NICHT 7.0, mit Nennung der jeweiligen Fehlimplementierung"

key-files:
  created: []
  modified:
    - service_impl/src/reporting.rs
    - service_impl/src/test/reporting_planning_target.rs
    - service_impl/src/test/reporting_phase2_fixtures.rs
    - shifty-dioxus/src/state/employee_work_details.rs
    - shifty-dioxus/src/component/working_hours_mini_overview.rs
    - docs/features/F07-reporting-balance.md
    - docs/features/F07-reporting-balance_de.md
    - docs/domain/time-accounting.md
    - docs/domain/time-accounting_de.md
    - .planning/phases/64-planungs-soll-f-r-dynamische-vertr-ge/64-UAT.md
    - .planning/ROADMAP.md
    - .planning/STATE.md

key-decisions:
  - "D-64-06: Die Zusage wird mit relation skaliert (werktags-basiert), nicht tages-basiert pro-rata; committed_voluntary_target_in_range (whole-week-out) wird NICHT wiederverwendet"
  - "D-64-07: Die Reduktion greift proportional ueber target_reduction_scale, weil eine reine Subtraktion nur den bezahlten Anteil kuerzen wuerde (42 statt 40)"
  - "Das Reduktions-Gate haengt an planning_base_for_target statt an dynamic_hours — das alte Gate lieferte fuer expected_hours == 0 mit Zusage > 0 den Wert 0.0"
  - "Kein .max(0.0): ein negatives Planungs-Soll ist ab jetzt assertiertes Verhalten und als Produktfrage D-64-05-Q1 gestellt"
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 13 — der Snapshot-Writer konsumiert EmployeeReport aus hours_per_week, das von 64-05 nicht angefasst ist"

patterns-established:
  - "Wortgleichheit als Beweisvoraussetzung: beide Konstruktionsstellen tragen identische Namen und identische Form, damit der Paritaetstest ein Beweis und kein Zufall ist"
  - "Gemessene statt behauptete Zahlen: jede Zahl im SUMMARY stammt aus einer echten Ausgabe; abweichende Herleitungen werden mit ihrer Arithmetik offengelegt, bevor eine Erwartung angepasst wird"

requirements-completed: []

metrics:
  duration: "~70min"
  tasks: 4
  files: 12
  completed: 2026-09-29
---

# Phase 64 Plan 05: Planungs-Soll umfasst die ehrenamtliche Zusage — Summary

Die ehrenamtliche Zusage (`committed_voluntary`) fliesst jetzt an beiden
`ShortEmployeeReport`-Konstruktionsstellen in das Planungs-Soll ein, proportional
um Abwesenheiten und Feiertage reduziert — ohne dass eine einzige Bilanzgroesse
sich bewegt.

> **Die PHASE bleibt offen.** `64-04` ist weiterhin `status: awaiting_uat`,
> `64-UAT.md` weiterhin `awaiting_human`, alle funf Szenarien und beide
> Zusatzfragen `pending`. PSOLL-01 und PSOLL-02 haengen am UAT;
> `requirements-completed` ist deshalb leer.

## Was gebaut wurde

Das im UAT von 64-04 gefundene Loch war **semantisch, kein Bug**: 64-01 hat das
Planungs-Soll als *Vertrags*-Groesse gebaut. Es ist aber eine **Zusage**-Groesse
[D-64-05] — der Zweck ist die Schichtplanung, und die planende Person muss sehen,
was die jeweilige Person fuer die Woche zusagt. Ob diese Stunden bezahlt sind,
ist dafuer unerheblich. `committed_voluntary` kam vor diesem Plan in keiner der
beiden Formeln vor.

### Task 1 — die Verteilungs-Entscheidung als Pure-fn festgenagelt (`d32beb33`)

- `potential_workdays_for_week(year, week, wh) -> (usize, u8)` — **wortwoertlich**
  aus `weight_for_week` extrahiert, alle funf Randwochen-Zweige, **kein** Guard
  fuer den vorbestehenden `0/0`-NaN. `weight_for_week` behaelt Signatur und
  Rueckgabewerte exakt.
- `committed_voluntary_weight_for_week(year, week, wh) -> f32` —
  `committed_voluntary * relation`, `relation` aus derselben Quelle wie der
  bezahlte Anteil. Das ist der Grund, warum die beiden Skalierungen **nicht**
  abdriften koennen.
- Doc-Comments korrigiert (siehe Grep-Befund unten).
- Vier Pure-fn-Tests im neuen Modul `test_committed_voluntary_weight_for_week`
  (`reporting.rs:3439`).

### Task 2 — RED (`3eaa8acf`)

Drei Fixtures mit `committed_voluntary = 10.0` (dynamisch, fest, 0h-Vertrag) und
funf Integrationstests. Die bestehenden Fixtures bleiben bei `0.0` — sie sind der
PSOLL-04-Anker. Die Harnesses wurden **additiv** erweitert
(`build_get_week_service_with_extra_hours`,
`build_all_employees_service_with_extra_hours`); die bestehenden Varianten
delegieren mit leerer Liste und aendern ihr Verhalten nicht.

### Task 3 — GREEN (`afc6dd46`)

Beide Bloecke sind identisch benannt und identisch geformt (verbatim unten).
Stelle B bindet den auf das Zieljahr geklammerten `EmployeeWorkDetails`-Row
**einmal** und liest ihn mit `weight_for_week` UND
`committed_voluntary_weight_for_week`; der Fold ist ein Dreitupel. Die
Klammer-Ausdruecke sind byte-identisch verschoben, nicht veraendert.

### Task 4 — Doku, Frontend, UAT, Buchhaltung (`07b5c59b`, `6e112069`, `2d9dba38`)

## Die beiden Bloecke verbatim — Wortgleichheit nachlesbar

**Konstruktionsstelle A** (`service_impl/src/reporting.rs:1162-1186`,
`assemble_weeks`):

```rust
                let committed_voluntary_for_target: f32 =
                    find_working_hours_for_calendar_week(working_hours, year, week)
                        .map(|wh| committed_voluntary_weight_for_week(year, week, wh))
                        .sum();
                let planning_base_for_target = dynamic_hours + committed_voluntary_for_target;
                let target_reduction_scale = if dynamic_hours > 0.0 {
                    planning_base_for_target / dynamic_hours
                } else {
                    1.0f32
                };
                let reduce_target = has_contract_row && planning_base_for_target > 0.0;
                let absence_manual_for_target = abense_hours;
                let absence_derived_for_target = absence_derived_vacation_hours
                    + absence_derived_sick_leave_hours
                    + absence_derived_unpaid_leave_hours;
                let holiday_derived_for_target = derived_holiday_for_week;
                let planning_target_hours = if reduce_target {
                    planning_base_for_target
                        - (absence_manual_for_target
                            + absence_derived_for_target
                            + holiday_derived_for_target)
                            * target_reduction_scale
                } else {
                    0.0f32
                };
```

**Konstruktionsstelle B** (`service_impl/src/reporting.rs:1562-1577`,
`get_reports_for_all_employees`) — `committed_voluntary_for_target` kommt hier aus
dem Dreitupel-Fold, die drei Reduktions-Locals stehen unmittelbar darueber:

```rust
                    let planning_base_for_target = dynamic_hours + committed_voluntary_for_target;
                    let target_reduction_scale = if dynamic_hours > 0.0 {
                        planning_base_for_target / dynamic_hours
                    } else {
                        1.0f32
                    };
                    let reduce_target = has_contract_row && planning_base_for_target > 0.0;
                    let planning_target_hours_for_week = if reduce_target {
                        planning_base_for_target
                            - (absence_manual_for_target
                                + absence_derived_for_target
                                + holiday_derived_for_target)
                                * target_reduction_scale
                    } else {
                        0.0f32
                    };
```

Ein Seite-an-Seite-Diff zeigt nur die drei Reduktions-**Quell**ausdruecke (an A
aus `employee_extra_hours` und `derive_hours_for_week_pure`, an B aus
`week_extra_hours_for_target`, `derived` und `derived_holiday`) und den Namen des
Ergebnis-Locals. Das ist die Vorbedingung dafuer, dass der Paritaetstest ein
Beweis und kein Zufall ist.

## Test-Matrix — erwartet, im RED bekommen, im GREEN gemessen

Alle GREEN-Werte sind mit temporaeren `println!`-Sonden aus dem echten Testlauf
erhoben und danach zurueckgenommen (die Datei im Commit enthaelt keine Sonden).

### Die funf neuen Integrationstests

| Test / Messpunkt | erwartet (hergeleitet) | im RED bekommen | im GREEN gemessen |
|---|---|---|---|
| `planning_target_includes_commitment_dynamic_both_sites` — A (`get_week` 2024/23) | 50.0 | **40.0** | **50.0** |
| " — B (`get_reports_for_all_employees` bis 2024/23) | 100.0 | (nicht erreicht) | **100.0** |
| `planning_target_commitment_reduced_by_absence_parity` — A KW 22 | 50.0 | **40.0** | **50.0** |
| " — A KW 23 | 20.0 | (nicht erreicht) | **20.0** |
| " — A Summe | 70.0 | — | **70.0** |
| " — B | 70.0 | (nicht erreicht) | **70.0** |
| `planning_target_fixed_contract_differs_by_commitment` — KW 22 | 50.0 / `expected` 40.0 / Diff 10.0 | **40.0** | **50.0 / 40.0 / 10.0** |
| " — KW 23 | 20.0 / `expected` 16.0 / Diff 4.0 | (nicht erreicht) | **20.0 / 16.0 / 4.0** |
| " — Block 3 (Zusage 0) | Byte-Identitaet | (nicht erreicht) | **16.0 / 16.0 / Diff 0.0** |
| `planning_target_zero_expected_hours_with_commitment` — A | 10.0 | **0.0** | **10.0** |
| " — B | 20.0 | (nicht erreicht) | **20.0** |
| `planning_target_not_clamped_when_reductions_exceed_base` — A | −10.0 | **0.0** | **−10.0** |
| " — B, Fenster bis KW 22 | 10.0 | (nicht erreicht) | **10.0** |
| " — B, Fenster bis KW 23 | (siehe Abweichung unten) | — | **0.0** |
| " — B, Wochenbeitrag KW 23 | −10.0 | — | **−10.0** |

**RED-Nachweis gefuehrt, nicht behauptet.** Jeder der funf Tests ist an einer
**Zahl** gescheitert, keiner an einem Kompilierfehler oder einem Harness-Panic.
Die funf bestehenden Tests der Datei waren im RED-Lauf gruen und sind unveraendert
geblieben (`git diff` der Datei zeigt nur Additionen ab der neuen Trennlinie, den
Header-Nachtrag und die additive Harness-Erweiterung).

**Die vier gefangenen Fehlimplementierungen, wie vom Plan verlangt gepruft:**

| Fehlimplementierung | liefert | gefangen von |
|---|---|---|
| Zusage ignoriert (Zustand vor 64-05) | 40.0 / 80.0 / 56.0 | RED-Werte 40.0 an drei Tests |
| reine Subtraktion ohne `target_reduction_scale` | 26.0 (KW 23) bzw. 76.0 (Summe) | `planning_target_commitment_reduced_by_absence_parity`, Einzelwoche-Assertion 20.0 |
| altes Gate bleibt stehen | 0.0 | RED-Wert 0.0 von `planning_target_zero_expected_hours_with_commitment` |
| `.max(0.0)`-Clamp | 0.0 | RED-Wert 0.0 von `planning_target_not_clamped_when_reductions_exceed_base`, GREEN −10.0 |

### Die vier Pure-fn-Tests (D-64-06)

| Test | erwartet | gemessen |
|---|---|---|
| `committed_voluntary_weight_full_week_is_unscaled` | 10.0 | **10.0** |
| `committed_voluntary_weight_pins_relation_scaling_d_64_06` | 4.2 (und **nicht** 3.0, **nicht** 7.0) | **4.2000003** |
| `committed_voluntary_weight_outside_contract_is_zero` | 0.0 | **0.0** |
| `committed_voluntary_weight_zero_commitment_is_zero` | 0.0 | **0.0** |

Diese vier waren beim Anlegen sofort gruen — sie sind Pins, keine Treiber (das
Verhalten ist additiv neu, es gab keinen Vorzustand, an dem sie scheitern
konnten). Ihre Beweiskraft liegt in den zwei Gegen-Assertions von
`…_pins_relation_scaling_d_64_06`, die die beiden verworfenen Kandidaten mit
Namen und Zahl benennen.

## Gemessene Randwerte, die der Plan ausdruecklich verlangt

**Abgeleitete Reduktion des 0h-Vertrags: 0.0.** Separat gemessen (temporaere
Sonde) mit derselben Fixture **und** den KW-23-Abwesenheiten (Urlaub Mo–Mi +
Krankheit Di):

```
M-T4-A       planning=10 expected=0
M-T4-ABSENCE planning=10 expected=0 vacation=0 sick=0
```

Das Planungs-Soll bleibt 10.0 — die abgeleitete Reduktion ist **0.0**, weil
`hours_per_day = expected_hours / workdays_per_week = 0/5 = 0`. Die Zahl im Test
musste also nicht angepasst werden.

**Beobachteter negativer Wert der No-Clamp-Sperre: −10.0** (`M-T5-A planning=-10`).

## Abweichung von der Herleitung im Plan — offengelegt, nicht angepasst

Der Plan sagt in `<behavior>` fuer
`planning_target_not_clamped_when_reductions_exceed_base`: „A + B → **−10.0**".
Fuer **A** stimmt das (gemessen −10.0). Fuer **B** ist es arithmetisch nicht
erreichbar:

```
Stelle B summiert die Vertragswochen des Fensters.
  KW 22: Basis 10.0, keine Reduktion                     → +10.0
  KW 23: Basis 10.0, manuelle Vacation 20.0, scale 1.0    → −10.0
  Summe (Fenster bis KW 23)                               →   0.0
```

Der Report traegt nur die Jahressumme, nicht den Wochenwert. Eine Assertion auf
`0.0` waere ausserdem ein schlechter Wachhund, weil ein neutrales Feld ebenfalls
`0.0` liefert. Der Test misst deshalb die **Zerlegung ueber zwei Fenster**:

| Fenster | gemessen | mit `.max(0.0)` waere es |
|---|---|---|
| `get_reports_for_all_employees(2024, 22)` | **10.0** | 10.0 |
| `get_reports_for_all_employees(2024, 23)` | **0.0** | 10.0 |
| Differenz = Wochenbeitrag KW 23 | **−10.0** | 0.0 |

Das ist strenger als die Plan-Formulierung und nagelt den negativen
Wochenbeitrag auch an Stelle B fest. Keine bestehende Erwartung wurde angepasst.

## Grep-Befund zu `committed_voluntary_for_calendar_week`

Kommando: `grep -rn 'committed_voluntary_for_calendar_week' --include='*.rs' .`

| Treffer | Klassifikation |
|---|---|
| `service_impl/src/reporting.rs:122` (vor Task 1) | **Definition** |
| `service_impl/src/reporting.rs:3156` | Doc-Comment des eigenen Testmoduls |
| `service_impl/src/reporting.rs:3162` | `mod test_committed_voluntary_for_calendar_week` |
| `service_impl/src/reporting.rs:3213, 3227, 3242, 3254` | vier Aufrufe **aus diesem Testmodul** |

**Befund: der im Doc-Comment angekuendigte Read-Site existiert nicht.** Der Satz
„Phase 15 (Reporting-Integration) konsumiert diesen Helper direkt" ist seit Phase
14 unerfuellt — es gibt **keinen** Produktions-Aufrufer, der Helper ist weiterhin
inert. Die Ankuendigung ist durch den gemessenen Ist-Zustand ersetzt, samt
Verweis, dass das Planungs-Soll `committed_voluntary_weight_for_week` benutzt
(Grund: keine `relation`-Skalierung). `committed_voluntary_prorata_for_week`
bleibt fachlich richtig beschrieben und hat den Zusatz bekommen, dass das
Planungs-Soll es bewusst nicht konsumiert (7-Tage- gegen Werktags-Basis).

## Frontend — gemessen, nicht angenommen (PSOLL-02)

| Gate | Erwartung | Gemessen |
|---|---|---|
| Testmodul-Grenze `#[cfg(test)]`, neu bestimmt | — | **Zeile 333** |
| `planning_target_hours`-Zugriffe in der Produktionsregion (1–332) | 3 | **3** — Zeilen **123** (Karten-`target`), **186** (Fusszeilen-`total_target`), **238** (Tabellen-`target`) |
| `.expected_hours` / `.dynamic_hours` in der Produktionsregion | 0 | **0** |
| Ladepfad | 1:1-Durchreichung | `loader.rs:456` → `planning_target_hours: report.planning_target_hours`; `state/employee_work_details.rs:17` ist die Feld-Deklaration, `:30` der `Default` — **keine Rechnung** |

**Keine Logikaenderung im Frontend.** Geaendert wurden ausschliesslich die drei
Anker-Kommentare und das Feld-Doc-Comment. Harte Nebenbedingung eingehalten: die
Kommentare nennen die Altfelder **ausgeschrieben** („das bilanzrelevante Soll")
statt als Feldpfad — der Invariantentest
`working_hours_mini_overview_reads_planning_target_only` grept die
Produktionsregion und zaehlt Kommentare mit.

## Doku-Korrektur (EN + DE, je im selben Commit)

64-03 hatte die **alte** Definition geschrieben. Jede falsche Behauptung einzeln
korrigiert:

| Stelle | war falsch | jetzt |
|---|---|---|
| F07 §2.7, Tabellenzeile „Nicht-dynamischer Vertrag" | „dieselbe Zahl, strukturell" unbedingt | auf „Zusage 0" qualifiziert; zwei neue Zeilen (Zusage > 0, 0h-Vertrag mit Zusage) plus eine Zeile „Enthaelt die ehrenamtliche Zusage" |
| F07 §2.7, „Was das Feld ist" | „**Vertrags**-Soll" | **Zusage**-Groesse [D-64-05], Zweck und Unabhaengigkeit von der Bezahlung ausgeschrieben |
| F07 §2.7, zitiertes Gate | `has_contract_row && dynamic_hours > 0.0` | neues Gate zitiert, Formelblock ergaenzt, drei tragende Eigenschaften, NaN-Nebenbefund |
| F07 §2.7, Zeichengleichheit | unbedingt | „bei Zusage 0", Beweis ueber `target_reduction_scale == 1.0`; der Fall „mit Zusage" mit exakter Differenz und Testnamen |
| F07 §2.7, kein `.max(0.0)` | nur begruendet | plus Verweis auf die Sperre, die den negativen Wert jetzt **assertiert** |
| F07 §2.7 NEU | — | D-64-06 mit der Drei-Wege-Tabelle (4.2 / 3.0 / 7.0), die Begruendung gegen `committed_voluntary_target_in_range`, die gewollte Randwochen-Abweichung zum F2-Soll, und D-64-07 mit dem 42-gegen-40-Beispiel |
| F07 §3 | „dieselben **drei** Quellen … nur das Gate" | **vier** Quellen; Basis **und** Gate **und** Skalierung unterscheiden sich. Tabelle „Wo kommt ‚Erwartung' her?" um `committed_voluntary` ergaenzt |
| F07 §4.1 | Inline-Kommentar „bei dynamischem Vertrag > 0" | Zusage-Semantik |
| F07 §5 | Bedeutungs-Halbsatz | nachgezogen, plus ausdruecklich: **keine Wire-Aenderung durch 64-05** |
| F07 §6.5 | Merksatz | Zusage-Semantik; die Tabelle der drei Anzeigestellen bleibt (weiterhin drei) |
| F07 §8.1 | funf Tests, Zeilenzahlen 671 / 148 | zehn Integrationstests + vier Pure-fn-Tests, je mit gefangener Fehlimplementierung; Zeilenzahlen **gemessen: 1087 / 199** |
| F07 §9 | zwei Fallen | **dritte** Falle: mechanisch richtig, fachlich falsch — im UAT gefunden, nicht von neun gruenen Tests; 64-05 als Gap-Closure genannt; Snapshot-Absatz **neu bewiesen** ueber die Zaehlergates, Version bleibt **13** |
| `time-accounting` (beide Sprachen) | „dieselben drei Quellen", „dort tragen beide Groessen dieselbe Zahl", „nur seine Reduktions-Bedingung unterscheidet sich" | Quellen-, 0h- und Ehrenamts-Zeile in der Tabelle; Merksatz ergaenzt; Bedingung „bei Zusage 0"; „Basis, Gate und Skalierung der Reduktion". Die ausfuehrliche Begruendung bleibt in F07 §2.7 |

Diff-Zaehler pro Paar: **1 EN / 1 DE** — beide Sprachen im selben Commit
(CLAUDE.md-Eisenregel).

## UAT nachgefuehrt, nichts abgehakt

`64-04` bleibt `status: awaiting_uat`; `64-UAT.md` bleibt `awaiting_human`; alle
Szenarien `pending`, D-64-03 `pending`.

- Kopf-Hinweis „**Nachtrag 64-05**": die zu pruefende Zahl hat sich geaendert; die
  Messreihe in Teil 3 ist unter der alten Semantik entstanden und bleibt als
  „vor 64-05" stehen.
- **Teil 2, Schritt 2b (NEU)** — Befund **B-64-05-2**: beide UAT-Vertraege wurden
  mit `committed_voluntary = 0` angelegt, gegen diesen Bestand ist die neue
  Semantik **nicht beobachtbar** (dieselbe Falle wie B-64-04-1, eine Ebene
  weiter). Der Schritt misst erst den Ist-Wert per
  `GET /working-hours/for-sales-person/{id}` und setzt dann 5.0h an Rosa **und**
  Béla (damit der feste Fall die Differenz zeigt), mit Ruecknahme-Kommandos.
- **Teil 3** um die Reihe „nach 64-05" erweitert (Stelle A KW 38 und KW 39,
  Stelle B bis KW 39), jede Zahl mit ausgeschriebenem Rechenweg.
- Szenario 1, 2, 4 mit neuen Erwartungszahlen; Szenario 1 zusaetzlich mit der
  Forderung, dass die Zusage in der Zahl **sichtbar** ist (25.0h statt 20.0h in
  KW 38). Szenario 3 mit der nach 64-05 neu erhobenen Trefferliste.
- **Szenario 5 (NEU)** fuer die neue Klasse: 0-Stunden-Vertrag mit Zusage zeigt
  5.0h statt 0.0h — der Kern des Gaps, bisher in keinem Szenario abgedeckt,
  inklusive Set- und Ruecknahme-`PUT`s und dem Hinweis, dass der Schritt **nach**
  Szenario 1/2/4 laufen muss.
- **Zusatzfrage D-64-05-Q1 (NEU)**: ein negatives Planungs-Soll ist jetzt
  assertiertes Verhalten. Als **Produktfrage** gestellt („−10.0h einplanen" ist
  als Anweisung sinnlos, die Zahl ist fachlich aber korrekt), nicht als Befund
  versteckt.

## Blocker

**B-64-05-1 — die Messreihe „nach 64-05" ist hergeleitet, nicht gemessen.** Die
Dev-Server waren beim Nachfuehren des Protokolls gestoppt (vom Nutzer beendet);
die Anweisung war ausdruecklich, sie nicht zu starten. Alle neuen Zahlen in
`64-UAT.md` Teil 3 und Teil 5 sind deshalb als **HERGELEITET** markiert und mit
ihrer Arithmetik ausgeschrieben, damit die spaetere Messung sie bestaetigen oder
widerlegen kann. Vor dem Augenpaar-Durchgang muss die Reihe mit echten Ausgaben
gefuellt werden. Das ist der einzige Punkt des Plans, der nicht erledigt werden
konnte.

**B-64-05-2 — Schritt 2b ist noch nicht ausgefuehrt** (dieselbe Ursache). Ohne
ihn ist die neue Semantik im Browser nicht beobachtbar.

## Gates (echte Ausgaben)

| Gate | Erwartung | Gemessen |
|---|---|---|
| `cargo build --workspace` | EXIT 0 | **EXIT 0** |
| `cargo test --workspace` | Baseline 1173 + 9, 0 failed | **1182 passed / 0 failed** (Summe der `test result:`-Zeilen: 11+9+33+2+3+3+17+17+972+11+104; `service_impl` lib 963→**972**, Integration **104** unveraendert) |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 Warnungen | **0** |
| Frontend `cargo test` | Baseline 860 | **860 passed / 0 failed** |
| Frontend `cargo clippy --all-targets -- -D warnings` | 0 Warnungen | **0** |
| Frontend WASM-Build | gruen | **gruen** |
| `grep -c 'available_hours: report.expected_hours'` (ohne Kommentarzeilen) | 2 | **2** |
| `grep -c 'CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13'` | 1 | **1** |
| `git diff --name-only` ueber `booking_information.rs`, `billing_period_report.rs`, `service/src/reporting.rs`, `rest-types/src/lib.rs` | leer | **leer** |
| `git diff` ueber `hours_per_week` | keine Zeile | **keine** |
| Feldzugriffe in der Produktionsregion von `working_hours_mini_overview.rs` | 3 | **3** |
| Doku-Diff-Zaehler je Paar | 1 EN / 1 DE | **1 EN / 1 DE** (F07 und time-accounting) |
| `64-UAT.md` `status: awaiting_human` | >= 1 | **1** |

**Keine bestehende Testerwartung editiert.** Nach Task 1 und Task 3 war
`git diff --stat -- service_impl/src/test/` leer bzw. zeigte ausschliesslich die
Additionen aus Task 2. Der Billing-, Carryover- und Booking-Information-Teil der
Suite ist ohne eine einzige Anpassung gruen — das ist der Beweis, dass keine
Bilanzgroesse sich bewegt hat.

## Buchhaltung (per Hand, die Verben sind defekt)

- **ROADMAP**: Progress-Zeile `3/4` → **`3/5`** (nicht 4/5 — 64-04 haengt am UAT
  und ist nicht complete), `- [ ] 64-05-PLAN.md` → `[x]`, und die Sektion
  „Gesperrte Entscheidungen" um **D-64-06** und **D-64-07** ergaenzt.
  `roadmap.update-plan-progress` und `phase.complete` **nicht** aufgerufen. Der
  `git diff` zeigt genau drei Aenderungsregionen (Zeile ~137 Einschub, Zeile
  ~732 Checkbox, Zeile ~758 Progress); die UAT-Tabelle bei ~Zeile 114 ist
  **unberuehrt**.
- **STATE.md**: `total_plans` 36 → **37**, `completed_plans` bleibt **35**,
  `percent` 97 → **95** (35/37 ≈ 94.6, konsistent und < 100). Die
  Phase-60-Zaehler-Anomalie wurde nicht „korrigiert", nur nicht verstaerkt.
  „Current Position" und `stopped_at` per Hand fortgeschrieben, mit der
  ausdruecklichen Notiz, dass 64-04 weiter am UAT haengt und PSOLL-01/02 offen
  sind. Vier Entscheidungen mit `[Phase 64]` eingetragen (D-64-06, D-64-07, das
  neue Gate, die No-Clamp-Sperre). Metrik-Zeile `Phase 64 P05` ergaenzt.
- **REQUIREMENTS.md**: unberuehrt, PSOLL-01…04 bleiben Pending.
  `requirements.mark-complete` **nicht** aufgerufen.

## Was NICHT angefasst wurde

`hours_per_week` (dritter `weight_for_week`-Aufrufer, speist `EmployeeReport` und
damit den Billing-Snapshot), `booking_information.rs` (`available_hours`),
`billing_period_report.rs`, `CURRENT_SNAPSHOT_SCHEMA_VERSION` (**bleibt 13**),
`service/src/reporting.rs`, `rest-types/src/lib.rs` (das DTO-Feld existiert
schon; die Semantik aendert sich, nicht die Wire-Form), `64-04-SUMMARY.md`,
`.planning/REQUIREMENTS.md`.

## Deviations from Plan

### Keine Rule-1/2/3-Abweichung

Es war kein Bug zu fixen, keine fehlende kritische Funktionalitaet zu ergaenzen
und kein blockierendes Problem zu loesen.

### Offengelegte Abweichung von einer Plan-Herleitung

Die `<behavior>`-Erwartung „A + B → −10.0" fuer die No-Clamp-Sperre ist an
Stelle B arithmetisch nicht erreichbar (die Summe ueber zwei Wochen ist 0.0). Der
Test misst dort statt der Summe die Zerlegung ueber zwei Fenster und nagelt den
Wochenbeitrag −10.0 fest — strenger als die Plan-Formulierung. Arithmetik oben
unter „Abweichung von der Herleitung im Plan" ausgeschrieben. **Keine bestehende
Erwartung wurde angepasst.**

### Nicht erledigt (Blocker, nicht Abweichung)

Die Laufzeit-Messreihe „nach 64-05" im UAT (B-64-05-1) und Schritt 2b
(B-64-05-2) — die Dev-Server waren gestoppt und durften nicht gestartet werden.

## Self-Check: PASSED

Dateien:

- `FOUND: service_impl/src/reporting.rs`
- `FOUND: service_impl/src/test/reporting_planning_target.rs`
- `FOUND: service_impl/src/test/reporting_phase2_fixtures.rs`
- `FOUND: shifty-dioxus/src/state/employee_work_details.rs`
- `FOUND: shifty-dioxus/src/component/working_hours_mini_overview.rs`
- `FOUND: docs/features/F07-reporting-balance.md`
- `FOUND: docs/features/F07-reporting-balance_de.md`
- `FOUND: docs/domain/time-accounting.md`
- `FOUND: docs/domain/time-accounting_de.md`
- `FOUND: .planning/phases/64-planungs-soll-f-r-dynamische-vertr-ge/64-UAT.md`

Commits:

- `FOUND: d32beb33` — `refactor(64-05): potential_workdays_for_week extrahiert, committed_voluntary_weight_for_week angelegt (D-64-06)`
- `FOUND: 3eaa8acf` — `test(64-05): RED — Zusage-Testmatrix fuer das Planungs-Soll`
- `FOUND: afc6dd46` — `feat(64-05): Planungs-Soll umfasst die ehrenamtliche Zusage (D-64-05)`
- `FOUND: 07b5c59b` — `docs(64-05): Frontend-Kommentare auf die Zusage-Semantik (D-64-05)`
- `FOUND: 6e112069` — `docs(64-05): F07 + time-accounting — Planungs-Soll ist eine Zusage-Groesse (EN+DE)`
- `FOUND: 2d9dba38` — `docs(64-05): UAT-Erwartungen auf die Zusage-Semantik nachgefuehrt`
