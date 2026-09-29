---
phase: 64-planungs-soll-f-r-dynamische-vertr-ge
plan: 02
subsystem: frontend
tags: [rust, dioxus, ssr-test, source-invariant, tdd, mutation-check]

requires:
  - phase: 64-planungs-soll-f-r-dynamische-vertr-ge
    plan: 01
    provides: "rest_types::ShortEmployeeReportTO::planning_target_hours — ausgeliefert von GET /report und GET /report/week/{year}/{calendar_week}"
  - phase: 62-personen-verzeichnis
    provides: "PADM-06-Praezedenz — Fat-Backend-Durchreichung ohne Frontend-Ableitung (Doc-Kommentar an load_person_directory)"
provides:
  - "shifty-dioxus WorkingHoursMini::planning_target_hours: f32 (plus Default-Eintrag)"
  - "build_working_hours_mini reicht das Feld 1:1 aus dem DTO durch"
  - "drei umgestellte Anzeigestellen in working_hours_mini_overview.rs (Karten-target, Tabellen-target, Tabellen-total_target)"
  - "working_hours_mini_overview_dynamic_contract_shows_planning_target — SSR-Render-Test beider Layouts plus Summenzeile"
  - "working_hours_mini_overview_reads_planning_target_only — dauerhafter Quellcode-Invariantentest fuer PSOLL-02"
affects: [64-03 Doku, 64-04 UAT]

tech-stack:
  added: []
  patterns:
    - "Quellcode-Invariantentest mit drei unabhaengigen Assertion-Bloecken (Zugriffszahl / Verbot der Altfelder / Operator-Nachbarschaft), mutationsgeprueft"
    - "Vorbereitende Anpassung bestehender Test-Literale, sodass unveraenderte Assertions zum Neutralitaetsbeweis der Umstellung werden"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/state/employee_work_details.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/component/working_hours_mini_overview.rs

key-decisions:
  - "D-64-01 umgesetzt wie gepinnt: alle DREI Anzeigestellen wechseln, jede mit eigener Begruendung im Quellcode-Kommentar; kein Suchen-und-Ersetzen"
  - "Die elf bestehenden WorkingHoursMini-Literale bekommen das Planungs-Soll gleich dem bilanzrelevanten Soll — sie modellieren feste Vertraege, und dadurch bleiben ihre Assertions unveraendert gueltig und werden zum Neutralitaetsbeweis"
  - "Der Invariantentest prueft die Operator-Nachbarschaft nur unmittelbar hinter dem Feldnamen (trim nur Space/Tab, kein Newline) — damit trifft ein Kommentar in der Folgezeile den Test nicht"
  - "D-64-03 eingehalten: kein visueller Zusatzhinweis, kein neuer i18n-Key"

patterns-established:
  - "Mutations-Check als Teil des TDD-Belegs: eine Invariante gilt erst als abgesichert, wenn sie gegen eine wiedereingefuehrte Verletzung nachweislich faellt"

requirements-completed: [PSOLL-01, PSOLL-02]

coverage:
  - id: D1
    description: "Ein dynamischer Vertrag (bilanzrelevantes Soll 0, Planungs-Soll 20) zeigt im Karten-Layout 20.0 statt 0.0"
    requirement: "PSOLL-01"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/component/working_hours_mini_overview.rs#working_hours_mini_overview_dynamic_contract_shows_planning_target"
        status: pass
    human_judgment: false
  - id: D2
    description: "Dieselben zwei Zeilen zeigen im Tabellen-Layout 20.0h und 10.0h pro Zeile und 30.0h in der Summenzeile; keine Soll-Zelle rendert die Null"
    requirement: "PSOLL-01"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/component/working_hours_mini_overview.rs#working_hours_mini_overview_dynamic_contract_shows_planning_target"
        status: pass
    human_judgment: false
  - id: D3
    description: "Die Produktionsregion der Komponente liest das Planungs-Soll an genau drei Stellen, liest kein Altfeld mehr und rechnet nicht am Feld"
    requirement: "PSOLL-02"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/component/working_hours_mini_overview.rs#working_hours_mini_overview_reads_planning_target_only"
        status: pass
      - kind: other
        ref: "Mutations-Check: Fallback-Mutation und Arithmetik-Mutation lassen den Test nachweislich fallen (Protokoll unten)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Fuer feste Vertraege aendert sich die Anzeige nicht — keine bestehende Render-Assertion musste angefasst werden"
    requirement: "PSOLL-04"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/component/working_hours_mini_overview.rs (alle vorbestehenden Render-Tests)"
        status: pass
    human_judgment: false

duration: 35min
completed: 2026-09-29
status: complete
---

# Phase 64 Plan 02: Planungs-Soll im Frontend Summary

**Das Planungs-Soll reicht jetzt bis in die Anzeige: `WorkingHoursMini` traegt das Feld, `build_working_hours_mini` gibt es 1:1 weiter, und alle drei Anzeigestellen der Schichtplan-Uebersicht lesen es — abgesichert durch einen Render-Test fuer den dynamischen Fall und einen Quellcode-Invariantentest, der gegen zwei wiedereingefuehrte Rueckfaelle nachweislich faellt.**

## Performance

- **Duration:** ~35 min
- **Tasks:** 2 von 2
- **Files modified:** 3 (0 neu, 3 geaendert)

## Task Commits

1. **Task 1: Feld am Zustand, Durchreichung im Ladepfad, Test-Literale nachgezogen** — `479d3766` (feat)
2. **Task 2 RED: die beiden neuen Tests** — `35a8bbf6` (test)
3. **Task 2 GREEN: die drei Anzeigestellen umgestellt** — `64f55556` (feat)

## Die drei Anzeigestellen — je Stelle einzeln entschieden

Der Origin-Todo verlangt ausdruecklich eine bewusste Entscheidung pro Stelle und kein
pauschales Suchen-und-Ersetzen. Es wurden drei getrennte Edits gemacht, mit drei
unterschiedlich formulierten, ersetzten (nicht kopierten) Kommentarbloecken.

**Stelle 1 — Karten-Layout, `target` (Datei-Zeile 123 nach der Umstellung).**
Das ist die Zahl, die ein Planer beim Verteilen von Schichten ansieht, und genau die
Stelle, die der Origin-Todo meldet ("bei dynamischen Vertraegen steht dort 0"). Sie
wechselt, weil sie der Gegenstand der Meldung ist. Ihr Kommentar traegt zusaetzlich die
Konsequenz fuer die Fortschrittsanzeige, weil `progress_bar_class`, `hours_text_class`
und `progress_bar_percent` unmittelbar darunter aus derselben Bindung lesen.

**Stelle 2 — Tabellen-Layout, `target` pro Zeile (Datei-Zeile ~233).**
Dieselbe Information, dieselbe Rolle, nur dichter dargestellt. Ein Wechsel nur im
Karten-Layout wuerde bedeuten, dass dieselbe Person in der Tabelle 0 und in den Karten
das Planungs-Soll zeigt; ein solcher gemischter Zustand ist schlechter als der heutige
einheitliche Fehler. Ihr Kommentar begruendet genau das und wiederholt die
Fortschrittsanzeige-Passage NICHT.

**Stelle 3 — Tabellen-Layout, `total_target` in der Summenzeile (Datei-Zeile ~176).**
Die Fusszeile summiert die Spalte darueber. Sobald Stelle 2 wechselt und diese nicht,
widerspricht die Fusszeile ihrer eigenen Spalte. Sie wechselt, weil Stelle 2 wechselt;
eine eigenstaendige Begruendung hat sie nicht und braucht sie nicht. Ihr Kommentar
verweist deshalb auf den Kommentar an der Zeilen-Stelle statt ihn zu wiederholen —
und haelt zusaetzlich fest, dass die Spaltensumme ueber Personen Darstellung ist und
keine Ableitungsregel fuer den Wert einer einzelnen Person, PSOLL-02 also unberuehrt
bleibt.

Der heute dort stehende Hinweis auf den HSP-03-Band-Guard bleibt an Stelle 1 und 2
inhaltlich erhalten: er erklaert weiterhin, warum der Rohwert des dynamischen Solls
kein taugliches Anzeige-Ziel ist.

## PSOLL-02: der Invariantentest und sein Mutations-Check

`working_hours_mini_overview_reads_planning_target_only` isoliert die Produktionsregion
ueber `source.split("#[cfg(test)]").next()` — dasselbe Muster wie der bestehende
`..._no_legacy_classes_in_source`. Drei unabhaengige Assertion-Bloecke, jeder gegen eine
andere Rueckfall-Form:

| Block | Gegen welche Rueckfall-Form |
|---|---|
| genau drei Zugriffe auf das Planungs-Soll-Feld | ein zusaetzlicher Zweig / eine vierte Leseart |
| kein Zugriff auf `.expected_hours` und `.dynamic_hours` | ein `else`-Fallback auf ein Altfeld |
| kein `+ - * /` unmittelbar hinter dem Feldnamen | eine Umrechnung / Ableitung am Wert |

**Mutations-Check durchgefuehrt und vollstaendig zurueckgenommen.** Die Lehre aus
64-01 (der Paritaetstest waere ohne die dritte Assertion im RED-Lauf gruen gewesen)
wurde hier explizit angewandt: der Test wurde nicht nur gegen "heute gruen" geprueft,
sondern gegen zwei kuenstlich wiedereingefuehrte Verletzungen.

| Mutation | eingesetzt an Stelle 1 | Testergebnis |
|---|---|---|
| Fallback | `let target = if working_hour.planning_target_hours > 0.0 { working_hour.planning_target_hours } else { working_hour.expected_hours };` | **FAILED** — `... found 4` / `left: 4, right: 3` (Block 1 schlug zuerst zu; Block 2 waere ebenfalls verletzt gewesen) |
| Arithmetik | `let target = working_hour.planning_target_hours - working_hour.balance_hours;` | **FAILED** — ``arithmetic operator `-` directly after the planning-target field — the frontend must read it, not compute with it`` (Block 3; Zugriffszahl blieb 3 und die Altfeld-Sperre unberuehrt, die Mutation waere von Block 1 und 2 allein NICHT gefunden worden) |

Die Datei wurde vor jeder Mutation nach `scratchpad/pristine.rs` gesichert und danach
daraus zurueckgespielt; `diff pristine.rs <Datei>` ist leer, und
`grep -c 'planning_target_hours > 0.0\|planning_target_hours -'` ergibt 0. Die
committete Datei ist mutationsfrei.

**Eine Praezisierung an Block 3:** getrimmt wird nur Space und Tab, kein Newline. Damit
gilt "unmittelbar hinter dem Feldnamen" woertlich, und ein Kommentar in der Folgezeile,
der mit `// - ...` beginnt, loest den Test nicht faelschlich aus. Der `.sum()`-Aufruf
der Summenzeile ist von Block 3 nicht betroffen (auf den Feldnamen folgt dort `)`).

## Der RED-Lauf, in je einem Satz

- `working_hours_mini_overview_dynamic_contract_shows_planning_target`:
  `card layout must show the planning target (20.0), not the balance-relevant Soll` —
  das gerenderte HTML enthielt `12.0 / 0.0h`, also genau die vom Origin-Todo gemeldete Null.
- `working_hours_mini_overview_reads_planning_target_only`:
  `production region must read the planning target at exactly the three display sites
  (cards target, table row target, table footer total), found 0` — `left: 0, right: 3`.

Beide waren fachlich rot, nicht durch einen Kompilierfehler: Task 1 hatte das Feld
bereits angelegt, sodass die Testdatei sauber kompilierte.

## Testzahlen vor und nach dem Plan

| Zeitpunkt | Frontend-Tests |
|---|---|
| vor dem Plan (Baseline aus 64-01) | 858 passed, 0 failed |
| nach Task 1 (nur Feld + Durchreichung) | 858 passed, 0 failed — unveraendert |
| nach Task 2 (GREEN) | **860 passed**, 0 failed |

Gewachsen um genau zwei Tests. **Keine bestehende Assertion wurde geaendert** — die
elf `WorkingHoursMini`-Literale bekamen in Task 1 lediglich das neue Feld mit demselben
Wert wie `expected_hours`. Das ist genau der Neutralitaetsbeweis: alle vorbestehenden
Render-Tests modellieren feste Vertraege, und sie bleiben nach der Umstellung Wort fuer
Wort gueltig.

## D-64-02-Pruefung: gibt es eine zweite Soll-Anzeige im Frontend?

**Ergebnis: Nein — aus `ShortEmployeeReportTO` zeigt kein anderer Pfad ein Soll an.
D-64-02 ist damit ohne Zusatzarbeit erledigt, wie die ROADMAP erwartet hat. Kein BEFUND.**

Belege:

1. Es gibt im Frontend genau zwei Konsumenten von `ShortEmployeeReportTO`:
   `build_working_hours_mini` (`loader.rs:435`) und
   `impl From<&ShortEmployeeReportTO> for Employee` (`state/employee.rs:297`).
2. Der zweite Konsument setzt `expected_working_hours: 0.0` **hart** und liest das
   Soll-Feld des Reports ueberhaupt nicht — er uebernimmt nur `balance_hours`,
   `volunteer_hours` und die Rebooking-Flags. Er kann folglich nicht stillschweigend
   auseinanderlaufen.
3. `WorkingHoursMini` wird nirgends sonst konstruiert (Grep ueber `shifty-dioxus/src`:
   nur `loader.rs:448` und die Testliterale der Komponente).

**Zur Abgrenzung, wichtig fuer Plan 64-03:** die Jahres-/Mitarbeiter-Ansichten zeigen
sehr wohl ein Soll — aber aus einem **anderen DTO**, naemlich `WorkingHoursReportTO`
bzw. `EmployeeReportTO`, die in 64-01 gar nicht angefasst wurden:

| Stelle | Quelle | zeigt weiterhin |
|---|---|---|
| `component/employee_view.rs:805,867,869` (Wochenliste, Wochen-Detail, Differenz) | `WorkingHoursReportTO.expected_hours` | bilanzrelevantes Soll |
| `component/employee_weekly_histogram.rs:119,126,167` (Referenzlinie + Balkenfarbe) | `WorkingHoursReportTO.expected_hours` | bilanzrelevantes Soll |
| `component/employees_list.rs:51` (`target_hours_for`, letzte Woche) | `WorkingHoursReportTO.expected_hours` | bilanzrelevantes Soll |
| `state/employee.rs:368` (`expected_working_hours`) | `EmployeeReportTO.expected_hours` | bilanzrelevantes Soll |

Das ist kein Widerspruch, sondern die gewollte Trennung: diese Flaechen sind die
Bilanz-/Stundenkonto-Flaechen. Sie koennen das Planungs-Soll auch technisch nicht
anzeigen, weil es an ihren DTOs schlicht nicht existiert.

## Fuer Plan 64-03 (Doku): welche Flaeche zeigt was

**Zeigt ab jetzt das Planungs-Soll (`planning_target_hours`):**

- Schichtplan-Uebersicht unterhalb des Plans, **Karten-Layout** — Soll-Zahl in der
  `Ist / Soll`-Zeile, Fortschrittsbalken-Fuellung und Farbschwelle.
- Schichtplan-Uebersicht, **Tabellen-Layout** — Soll-Spalte pro Zeile,
  Auslastungs-Balken und Prozentwert.
- Schichtplan-Uebersicht, **Tabellen-Summenzeile** — Soll-Summe.

Quelle jeweils: `GET /report/week/{year}/{calendar_week}` ueber
`load_working_hours_minified_for_week` → `build_working_hours_mini`.

**Zeigt weiterhin das bilanzrelevante Soll (`expected_hours`):**

- Mitarbeiter-Detailansicht: Jahres-/Wochenliste, Wochen-Detailkarte, Differenzspalte.
- Wochen-Histogramm: Referenzlinie und Balkenfarbe.
- Mitarbeiterliste: `target_hours_for` (Soll der letzten Woche).
- Stundenkonto/Balance in jeder Form — die Bilanz ist vom Planungs-Soll unberuehrt.

Merksatz fuer die F07-Frontend-Sektion: **das Planungs-Soll erscheint ausschliesslich
in der Wochen-Uebersicht unter dem Schichtplan; jede Bilanz-Flaeche zeigt unveraendert
das bilanzrelevante Soll.**

## Files Created/Modified

- `shifty-dioxus/src/state/employee_work_details.rs` — Feld `planning_target_hours: f32` an
  `WorkingHoursMini` (direkt hinter `expected_hours`) mit Doc-Kommentar zu Phase 64 /
  PSOLL-01 / PSOLL-02; Eintrag `0.0` im handgeschriebenen `Default`-Impl.
- `shifty-dioxus/src/loader.rs` — `build_working_hours_mini` reicht das Feld 1:1 durch,
  mit Kommentar-Verweis auf die Fat-Backend-Konvention (PADM-06-Praezedenz, der
  Doc-Kommentar an `load_person_directory` in derselben Datei).
- `shifty-dioxus/src/component/working_hours_mini_overview.rs` — drei umgestellte
  Anzeigestellen mit drei ersetzten Kommentarbloecken, elf ergaenzte Test-Literale,
  zwei neue Tests plus der Helfer `dynamic_contract_rows`.

## Decisions Made

Keine neuen Entscheidungen — D-64-01 und D-64-03 waren gepinnt und wurden exakt so
umgesetzt. Drei Umsetzungsdetails, die der Plan offen liess:

- **Block 3 des Invariantentests trimmt nur Space und Tab, nicht Newline.** Mit
  `trim_start()` haette ein Kommentar der Form `// - ...` in der Folgezeile den Test
  ausgeloest. Die enge Variante bildet "unmittelbar hinter dem Feldnamen" woertlich ab.
- **Die Render-Assertions greifen auf `>20.0h<` statt auf `20.0h`.** Ein blosses
  `!html.contains("0.0h")` waere unbrauchbar gewesen, weil `20.0h` die Zeichenkette
  `0.0h` als Teilkette enthaelt. Der Zellen-Anker `>…<` trennt sauber und trifft die
  Balance-Zellen (`>+0.0h<`) nicht.
- **Der Testhelfer `dynamic_contract_rows` liegt im Testmodul, nicht bei `make_row`.**
  `make_row` setzt Planungs-Soll gleich bilanzrelevantem Soll (fester Vertrag) und
  bleibt damit unveraendert die Vorlage aller Alt-Tests; der dynamische Fall braucht
  ein auseinanderfallendes Paar und bekommt deshalb einen eigenen Helfer.

## Deviations from Plan

**Keine.** Der Plan wurde exakt wie geschrieben ausgefuehrt; es war keine
Deviation-Regel anzuwenden.

## Issues Encountered

Keine. Der RED-Lauf war beim ersten Versuch fachlich rot, der GREEN-Lauf beim ersten
Versuch vollstaendig gruen, und keine Erwartung musste korrigiert werden.

## Gate-Ergebnisse

| Gate | Ergebnis |
|---|---|
| `cargo test --manifest-path shifty-dioxus/Cargo.toml` | gruen — **860 passed**, 0 failed (Baseline 858 + 2) |
| `cargo clippy --manifest-path shifty-dioxus/Cargo.toml --all-targets -- -D warnings` | gruen |
| `cargo build --manifest-path shifty-dioxus/Cargo.toml --target wasm32-unknown-unknown` | gruen |
| Produktionsregion: Zugriffe auf `.planning_target_hours` | **3** (gefordert: 3) |
| Produktionsregion: Zugriffe auf `.expected_hours` | **0** |
| Mutations-Check Fallback / Arithmetik | beide **FAILED** wie gewuenscht, vollstaendig zurueckgenommen |
| `cargo build --workspace` | gruen |
| `cargo test --workspace` | gruen — 963 (service_impl lib) + 104 Integration + 11, 0 failed (identisch zu 64-01) |
| `cargo clippy --workspace -- -D warnings` | gruen |
| `git diff -- service_impl/ service/ rest-types/ rest/ shifty-dioxus/src/i18n` | leer — kein Backend-, kein i18n-Diff |
| `git diff -- '*Cargo.toml'` | leer — keine neue Abhaengigkeit (T-64-SC) |
| `docs/` beruehrt | nein — 64-03 fuehrt die Doku zentral |

## Threat Flags

Keine neue Angriffsflaeche. `T-64-04` (Information Disclosure) und `T-64-05`
(Repudiation) bleiben `accept` wie im Plan begruendet: es wurde kein Request, keine
Route, keine Eingabe und kein Speicher hinzugefuegt; die Sichtbarkeit bestimmt
unveraendert der bestehende Ladepfad. `T-64-SC` entfaellt mangels Paket-Installation.

## Known Stubs

Keine.

## User Setup Required

Keine. Zu beachten fuer die UAT in 64-04: die laufenden Dev-Server auf :3000 und :8080
tragen einen Build von **vor** 64-01 und muessen vorher neu gestartet werden.

## Next Phase Readiness

64-03 (Doku) kann direkt aufsetzen — die Flaechen-Zuordnung oben ist genau die
Information, die die Frontend-Sektion von `F07-reporting-balance` (EN und DE) und
`docs/domain/time-accounting` braucht. Zusaetzlich aus 64-01 vorgemerkt: die
Rohwert-Asymmetrie zwischen `Availability::Unavailable` (Konstruktionsstelle A) und
`ReportType::AbsenceHours` (Stelle B) ist vorbestehend und betrifft auch die
Bilanzpfade.

64-04 (UAT) hat jetzt alles, was es braucht: ein dynamischer Vertrag zeigt in der
Uebersicht unter dem Schichtplan ein Soll > 0, in beiden Layouts und in der
Summenzeile, und ein fester Vertrag zeigt unveraendert dieselbe Zahl wie vorher.

---
*Phase: 64-planungs-soll-f-r-dynamische-vertr-ge*
*Completed: 2026-09-29*

## Self-Check: PASSED

- `shifty-dioxus/src/state/employee_work_details.rs` — FOUND
- `shifty-dioxus/src/loader.rs` — FOUND
- `shifty-dioxus/src/component/working_hours_mini_overview.rs` — FOUND
- `.planning/phases/64-planungs-soll-f-r-dynamische-vertr-ge/64-02-SUMMARY.md` — FOUND
- Commit `479d3766` — FOUND
- Commit `35a8bbf6` — FOUND
- Commit `64f55556` — FOUND
