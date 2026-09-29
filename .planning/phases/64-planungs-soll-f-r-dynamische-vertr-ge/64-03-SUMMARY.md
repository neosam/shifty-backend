---
phase: 64-planungs-soll-f-r-dynamische-vertr-ge
plan: 03
subsystem: docs
tags: [documentation, docs-freshness-gate, bilingual, reporting, planning-target]

requires:
  - phase: 64-planungs-soll-f-r-dynamische-vertr-ge
    plan: 01
    provides: "planning_target_hours an ShortEmployeeReport/TO, Gate an beiden Konstruktionsstellen, fuenf Tests"
  - phase: 64-planungs-soll-f-r-dynamische-vertr-ge
    plan: 02
    provides: "drei umgestellte Anzeigestellen plus die Flaechen-Zuordnung, welche Ansicht welches Soll zeigt"
  - phase: 63-personen-detailansicht
    plan: 10
    provides: "Muster des zentralen Doku-Plans am Phasenende inkl. Suche nach FALSCH gewordenen Aussagen"
provides:
  - "docs/features/F07-reporting-balance.md + _de.md — Abschnitt 2.7 (Planungs-Soll), Konstruktionsstellen in 3, Struct-Auszug in 4.1, Antwortfeld in 5, Abschnitt 6.5 (Flaechen-Zuordnung), Testtabelle in 8.1, Historien-Eintrag Phase 64 in 9"
  - "docs/domain/time-accounting.md + _de.md — Gegenueberstellung beider Soll-Begriffe, Sichtbarkeit, Hinweis auf die zwei Rechenstellen"
  - "Korrektur dreier vorbestehend falscher Doku-Aussagen (siehe Abschnitt 'Falsch gewordene Aussagen')"
affects: [64-04 UAT, Milestone-Close v2.7]

tech-stack:
  added: []
  patterns:
    - "Zentraler Doku-Plan am Phasenende: das Docs-Freshness-Gate wird einmal fuer alle Plaene der Phase geschlossen, EN und DE paarweise im selben Commit"
    - "Doku-Begruendung als Refactoring-Schutz: die Herleitung eines abweichenden Gates steht ausgeschrieben in der Doku, nicht nur im Codekommentar"

key-files:
  created:
    - .planning/phases/64-planungs-soll-f-r-dynamische-vertr-ge/64-03-SUMMARY.md
  modified:
    - docs/features/F07-reporting-balance.md
    - docs/features/F07-reporting-balance_de.md
    - docs/domain/time-accounting.md
    - docs/domain/time-accounting_de.md

key-decisions:
  - "Die neue Frontend-Sektion wurde als 6.5 hinter 6.4 eingeordnet statt als 6.2 vorne — dadurch bleibt die bestehende Nummerierung von 6.2/6.3/6.4 in beiden Sprachfassungen unangetastet"
  - "Die Gate-Begruendung steht vollstaendig nur in F07 2.7; time-accounting verweist darauf statt sie zu wiederholen (eine Quelle, kein Drift-Kandidat)"
  - "Drei vorbestehend falsche/veraltete Aussagen mitkorrigiert, weil sie exakt in den Sektionen stehen, die dieser Plan ohnehin anfasst, und weil die Snapshot-Version-Aussage eine geforderte Plan-Wahrheit direkt widerlegte"
  - "F01-Aussage 'is_dynamic => Soll = Ist' NICHT angefasst — sie ist ueber die Bilanz-Semantik weiterhin korrekt (Begruendung unten)"

requirements-completed: []

coverage:
  - id: D1
    description: "Die Doku unterscheidet bilanzrelevantes Soll und Planungs-Soll so, dass ein spaeterer Leser sie nicht verwechselt — Gegenueberstellung in beiden Dokumentenfamilien und beiden Sprachen"
    verification:
      - kind: other
        ref: "F07 2.7 + docs/domain/time-accounting 'Two kinds of Soll' / 'Zweierlei Soll' — je eine Tabelle mit Zweck / Verhalten bei dynamischem Vertrag / Balance-Relevanz / Sichtbarkeit"
        status: pass
    human_judgment: false
  - id: D2
    description: "Beide ShortEmployeeReport-Konstruktionsstellen sind benannt, samt Paritaets-Pflicht fuer neue Felder und der dokumentierten Ausnahme has_pending_rebooking"
    verification:
      - kind: other
        ref: "F07 Abschnitt 3, Unterabschnitt 'The two ShortEmployeeReport construction sites' / 'Die zwei ShortEmployeeReport-Konstruktionsstellen'"
        status: pass
    human_judgment: false
  - id: D3
    description: "Der Grund fuer das abweichende Gate steht in der Doku, inklusive der Warnung, es nicht zu 'vereinheitlichen'"
    verification:
      - kind: other
        ref: "F07 2.7 — beide Gate-Haelften einzeln begruendet, Zeilenverweise auf reporting.rs:1045/1121/1476/2305-2307"
        status: pass
    human_judgment: false
  - id: D4
    description: "Beide Sprachfassungen jeder Dokumentenfamilie sind im selben Commit geaendert und inhaltlich deckungsgleich"
    verification:
      - kind: other
        ref: "Commit 926c8e3f (F07 EN+DE), Commit 2cf46b6b (time-accounting EN+DE); Ueberschriften-Diff zeigt ausschliesslich Uebersetzungen, keine Strukturunterschiede"
        status: pass
    human_judgment: false
  - id: D5
    description: "Kein Code-Diff in diesem Plan"
    verification:
      - kind: other
        ref: "git diff --name-only -- '*.rs' '*.toml' '*.sql' ist leer"
        status: pass
    human_judgment: false

duration: 40min
completed: 2026-09-29
status: complete
---

# Phase 64 Plan 03: Doku — Planungs-Soll zentral dokumentiert Summary

**Das Docs-Freshness-Gate der Phase ist zentral geschlossen: die vier von der CLAUDE.md-Trigger-Tabelle geforderten Dateien trennen jetzt in beiden Sprachen das bilanzrelevante Soll vom Planungs-Soll, konservieren die Begruendung des abweichenden Reduktions-Gates und benennen die zwei Konstruktionsstellen samt Paritaets-Pflicht — und drei vorbestehend falsche Aussagen sind dabei aufgefallen und korrigiert.**

## Performance

- **Duration:** ~40 min
- **Tasks:** 2 von 2
- **Files modified:** 4 (0 neu, 4 geaendert), 605 Zeilen ergaenzt / 22 ersetzt

## Task Commits

1. **Task 1: F07 — Reporting-Feature-Doku, EN und DE** — `926c8e3f` (docs)
2. **Task 2: time-accounting — Domaenen-Doku, EN und DE** — `2cf46b6b` (docs)

## Sprachparitaet — Sektion fuer Sektion paarweise belegt

Beide Dateien einer Familie liegen jeweils **im selben Commit**. Die Sektions-Nummerierung
ist in EN und DE identisch; ein Diff der Ueberschriften-Listen beider F07-Fassungen zeigt
ausschliesslich Uebersetzungsunterschiede und keine Strukturabweichung.

| Sektion | EN (`F07-reporting-balance.md`) | DE (`F07-reporting-balance_de.md`) | Commit |
|---|---|---|---|
| Kopf, Zeilenzahl `service_impl::reporting` | `3261 lines` | `3261 Zeilen` | `926c8e3f` |
| **2.7 NEU** — Planungs-Soll vs. bilanzrelevantes Soll | „Planning target vs balance-relevant expected" | „Planungs-Soll vs. bilanzrelevantes Soll" | `926c8e3f` |
| 3 — Herkunft der Erwartung + **zwei Konstruktionsstellen NEU** | „The two `ShortEmployeeReport` construction sites" | „Die zwei `ShortEmployeeReport`-Konstruktionsstellen" | `926c8e3f` |
| 4.1 — **Struct-Auszug NEU** | „`ShortEmployeeReport` — the two target fields" | „`ShortEmployeeReport` — die zwei Soll-Felder" | `926c8e3f` |
| 5 — DTO-Liste, neues Antwortfeld | `ShortEmployeeReportTO`-Absatz erweitert | derselbe Absatz erweitert | `926c8e3f` |
| 6.1 — Pages, `page/shiftplan.rs` ergaenzt | neuer Bullet | neuer Bullet | `926c8e3f` |
| **6.5 NEU** — Flaechen-Zuordnung | „Which surface shows which \"Soll\"" | „Welche Flaeche zeigt welches „Soll"" | `926c8e3f` |
| 8.1 — Tests, neue Datei + Testtabelle | 5-Zeilen-Tabelle | 5-Zeilen-Tabelle | `926c8e3f` |
| 9 — Historie, Eintrag Phase 64 | neuer Bullet mit zwei Fallen | neuer Bullet mit zwei Fallen | `926c8e3f` |
| 9 — „Verhaeltnis zu Billing Period", Snapshot-Version | 12 → **13** korrigiert + Warum-kein-Bump | 12 → **13** korrigiert + Warum-kein-Bump | `926c8e3f` |

| Sektion | EN (`time-accounting.md`) | DE (`time-accounting_de.md`) | Commit |
|---|---|---|---|
| „What is `expected`?" — **Unterabschnitt NEU** | „Two kinds of \"Soll\"" | „Zweierlei „Soll"" | `2cf46b6b` |
| „Where the calculation happens" — zwei Rechenstellen + abweichendes Gate | Absatz ergaenzt, Zeilenzahl korrigiert | Absatz ergaenzt, Zeilenzahl korrigiert | `2cf46b6b` |
| „Where the calculation becomes visible" — Planungs-Soll-Flaeche | neuer Bullet | neuer Bullet | `2cf46b6b` |

`grep -c planning_target_hours`: F07 EN **11**, F07 DE **11**; time-accounting EN **1**,
DE **1** (in der Domaenen-Doku steht der technische Feldname absichtlich nur einmal, in
der Gegenueberstellungs-Tabelle — der Rest ist Fachsprache).

## Falsch gewordene Aussagen — was die Suche gefunden hat

Der Plan verlangt ausdruecklich die Suche nach Aussagen, die durch diese Phase nicht
unvollstaendig, sondern **FALSCH** geworden sind. Befund: **drei falsche Aussagen, alle
drei vorbestehend** (keine wurde erst von Phase 64 falsch gemacht), und **eine
Aussage, die beim Pruefen korrekt blieb**.

### Korrigiert (weil sie exakt in den angefassten Sektionen stehen)

**1. F07 §9 behauptete Snapshot-Schema-Version 12 — FALSCH, es ist 13.**
Wortlaut vorher: *„`snapshot_schema_version` (currently **12**, see
`billing_period_report.rs:117`)"*. Verifiziert:
`service_impl/src/billing_period_report.rs:140` traegt
`pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13;`. **Beide Angaben waren falsch** —
Wert *und* Zeilennummer. Die Aussage widersprach zusaetzlich §2.6 **derselben Datei**, die
den Bump 12 → 13 in Phase 57 (D-57-10) korrekt beschreibt, und `docs/features/F08-billing-period.md`
sowie `docs/domain/billing-period.md` stehen korrekt auf 13. In beiden Sprachfassungen
korrigiert. Diese Korrektur war nicht optional: der Plan fordert die Aussage
„Version bleibt bei 13" — sie waere in einer Datei gelandet, die zwei Absaetze weiter
„12" behauptet.

**2. `service_impl/src/reporting.rs` ist nicht 2205, sondern 3261 Zeilen lang — an vier Stellen.**
Die Zahl stand in allen vier von diesem Plan angefassten Dateien
(F07-Kopf EN/DE, time-accounting „Wo die Rechnung passiert" EN/DE). Vorbestehende Drift,
durch 64-01 weiter vergroessert. In allen vier korrigiert.

**3. `reporting_additive_merge.rs` hat 1575, nicht 1456 Zeilen; `reporting_phase2_fixtures.rs` 148, nicht 141.**
Beides in F07 §8.1, also genau der Sektion, in der die neue Testdatei eingetragen wurde.
In beiden Sprachfassungen korrigiert. Die Fixture-Datei wurde von 64-01 tatsaechlich
angefasst (Doc-Kommentar erweitert), ihr Zeilenzahl-Delta gehoert damit zu dieser Phase.

### Geprueft und NICHT angefasst (mit Begruendung)

**`docs/features/F01-employee-management.md:163` + `_de.md:164` und
`docs/employee-management.md:186` + `_de.md:190`:** *„`is_dynamic` — statt fixer
Wochenstunden zaehlen die tatsaechlich gearbeiteten Stunden als vereinbart (Soll = Ist)"*.

Diese Aussage ist **weiterhin korrekt** und wurde deshalb bewusst nicht angefasst. Sie
beschreibt die **Bilanz**-Semantik des Vertrags-Flags, und daran hat Phase 64 nichts
geaendert: `expected_hours` ist bei `is_dynamic == true` unveraendert 0, die Balance
unveraendert Soll = Ist. Das Planungs-Soll ist keine Vertragseigenschaft, sondern eine
abgeleitete Anzeige-Groesse am Report — es gehoert in F07, nicht in die
Vertragsfeld-Liste von F01. Ein Zusatz dort waere der Anfang genau der Verwechslung, die
§2.7 verhindern soll. **Bewusst out of scope, kein Drift-Befund.**

**Ebenfalls geprueft, kein Treffer:** `docs/domain/edge-cases*` (§1 Stundenkonto, §1.5
Balance-Perimeter) enthalten keine Aussage ueber die Soll-Anzeige, nur ueber
Balance-Beitraege — durch diese Phase unberuehrt.
`docs/architecture/06-frontend*` fuehrt Routen, keine Feldsemantik.
`docs/features/F14-rebooking*` nennt `ShortEmployeeReport` nur wegen der
Rebooking-Alert-Felder.

## Zusaetzlich geschlossene Luecke (nicht Drift, aber ueberfaellig)

F07 §6.1 kannte `page/shiftplan.rs` **gar nicht** — die Seite, unter der die
Wochen-Uebersicht haengt, war in der Frontend-Sektion des Reporting-Features nicht
gelistet, obwohl sie `GET /report/week/{year}/{calendar_week}` konsumiert. Vor Phase 64
war das nur unvollstaendig; ab Phase 64 wird die Luecke gefaehrlich, weil genau diese
Seite die einzige Flaeche mit dem neuen Soll-Begriff ist. Jetzt gelistet, mit Ladepfad
(`load_working_hours_minified_for_week` → `build_working_hours_mini` →
`WorkingHoursMini`) und Verweis auf §6.5.

## Jede Behauptung gegen den Code verifiziert

Nichts wurde aus den SUMMARYs uebernommen, ohne es im Code nachzusehen. Die Belege:

| Doku-Behauptung | Verifikation |
|---|---|
| Feld an `ShortEmployeeReport` | `service/src/reporting.rs:164` |
| Feld + `#[serde(default)]` am TO, Mapping im `From` | `rest-types/src/lib.rs:392` / `:414`; Struct-Range `380-402` |
| Gate `has_contract_row && dynamic_hours > 0.0` an Stelle A | `service_impl/src/reporting.rs:1121` |
| dasselbe Gate zeichengleich an Stelle B | `service_impl/src/reporting.rs:1476` |
| Bilanz-Gate `!has_contract_row \|\| planned_hours <= 0.0` | `service_impl/src/reporting.rs:1045` |
| Element 0 wird bei `is_dynamic` auf 0.0 gezwungen, Element 1 nicht | `weight_for_week`, `service_impl/src/reporting.rs:2305-2307` |
| Rohwert-Asymmetrie Stelle A vs. B | `:917` (`availability() == Unavailable`) vs. `:1450` (`as_report_type() == AbsenceHours`) |
| `assemble_weeks` von `get_week` UND `get_year` geteilt | `:769` Definition, Aufrufe `:2082` (`get_week` ab `:2020`) und `:2170` (`get_year` ab `:2101`) |
| `has_pending_rebooking` nur an Stelle B berechnet | `:1712` (Berechnung) vs. `:1193` (`false`/`None` an Stelle A) |
| `available_hours` liest weiter `expected_hours`, zweimal | `service_impl/src/booking_information.rs:644` und `:818` |
| Snapshot-Version 13, Writer ohne das Feld | `billing_period_report.rs:140`; `grep -c planning_target_hours billing_period_report.rs` = **0** |
| genau drei Anzeigestellen im Frontend | `working_hours_mini_overview.rs:123` (Karte), `:238` (Tabellenzeile), `:186` (Fussleiste) |
| Mini-Uebersicht haengt an `page/shiftplan.rs` | `shifty-dioxus/src/page/shiftplan.rs:20` (use) und `:1724` (Render) |
| zweiter TO-Konsument setzt `expected_working_hours: 0.0` hart | `shifty-dioxus/src/state/employee.rs:297` |
| fuenf Tests in der neuen Datei | `service_impl/src/test/reporting_planning_target.rs:269,298,333,390,527` (671 Zeilen) |

**Eine Abweichung gegenueber dem 64-02-SUMMARY:** dessen Zeilenangaben fuer zwei der drei
Anzeigestellen (`~233` fuer die Tabellenzeile, `~176` fuer die Fussleiste) treffen nicht
exakt; im Code stehen sie auf `238` und `186`. Die Doku traegt die verifizierten Zahlen.

## Decisions Made

Keine fachlichen Entscheidungen — dieser Plan hat keine. Vier Umsetzungsentscheidungen,
die der Plan offen liess:

- **Die neue Frontend-Sektion ist 6.5, nicht 6.2.** Der Plan sagt nur „in Abschnitt 6".
  Vorne einzufuegen haette 6.2/6.3/6.4 in beiden Sprachfassungen umnummeriert und jeden
  bestehenden Querverweis auf diese Nummern gebrochen. Hinten anhaengen ist additiv.
- **Die Gate-Begruendung steht ausgeschrieben nur in F07 §2.7.** `time-accounting`
  verweist darauf, statt sie zu paraphrasieren — der Plan verlangt das ausdruecklich
  („mit Verweis auf den neuen F07-Unterabschnitt statt Wiederholung der Begruendung").
  Praktischer Nebeneffekt: eine spaetere Praezisierung muss nur an einer Stelle passieren,
  also entsteht dort kein neuer Drift-Kandidat.
- **Der DE-Link auf F07 zeigt auf `F07-reporting-balance.md`, nicht `_de.md`.** Das ist
  die durchgaengige Konvention in `time-accounting_de.md` (alle sechs bestehenden
  `../features/`-Links zeigen auf die EN-Datei). Einen abweichenden Link zu setzen haette
  eine zweite Konvention eingefuehrt.
- **Die 0-Stunden-Randfall-Aussage aus 64-01 wurde uebernommen**, weil sie dort
  dokumentiert war (der Plan macht das von einer Bedingung abhaengig) — und zwar an der
  Stelle, an der sie das Weglassen der zweiten Gate-Haelfte verbietet.

## Deviations from Plan

**Keine Deviation-Regel angewandt.** Die drei Aussagen-Korrekturen oben sind kein
Deviation-Fall, sondern die im `<output>`-Block des Plans ausdruecklich geforderte Suche
nach falsch gewordenen Aussagen — sie sind Liefergegenstand, nicht Abweichung.

## Issues Encountered

Eine mechanische Panne, sofort behoben: die neue Frontend-Sektion landete beim ersten
Einfuegen in der EN-Datei versehentlich zwischen 6.1 und 6.2 und wurde als 6.5 hinter 6.4
verschoben; die Leerzeilen an beiden Nahtstellen wurden danach explizit geprueft. Die
DE-Datei wurde von Anfang an an der richtigen Stelle geschrieben. Endzustand beider
Dateien: identische Sektionsfolge.

## Gate-Ergebnisse

| Gate | Ergebnis |
|---|---|
| `cargo build --workspace` | **gruen** (`build=0`) |
| `cargo test --workspace` | **gruen** (`test=0`) — 963 (service_impl lib) + 104 Integration + 11 + uebrige, 0 failed; identisch zur 64-02-Baseline |
| `cargo clippy --workspace -- -D warnings` | **gruen** (`clippy=0`) |
| `git diff --name-only -- '*.rs' '*.toml' '*.sql'` | **leer** — kein Code-, kein Config-, kein Migrations-Diff |
| F07 EN+DE beide im Diff | ja, im selben Commit `926c8e3f` |
| time-accounting EN+DE beide im Diff | ja, im selben Commit `2cf46b6b` |
| `grep -c planning_target_hours` F07 EN / DE | 11 / 11 |
| `grep -c planning_target_hours` time-accounting EN / DE | 1 / 1 |
| Ueberschriften-Struktur EN vs. DE (F07) | deckungsgleich — Diff zeigt nur Uebersetzungen |
| `.mmd`-Diagramme beruehrt | nein (kein Migrations- oder DI-Trigger in dieser Phase) |

## Threat Flags

Keine neue Angriffsflaeche — der Plan aendert ausschliesslich Markdown.
`T-64-06` (Repudiation, „Doku trennt die Soll-Begriffe nicht") ist mitigiert: die
Begruendung des abweichenden Gates steht ausgeschrieben in beiden Sprachfassungen, mit
expliziter Warnung gegen das „Vereinheitlichen".
`T-64-07` (Sprachparitaet) ist mitigiert: je Familie ein Commit mit beiden Dateien,
oben paarweise belegt. `T-64-SC` entfaellt mangels Paket-Installation.

## Known Stubs

Keine.

## User Setup Required

Keine.

## Next Phase Readiness

64-04 (UAT, blockierender Checkpoint) kann direkt starten. Die Doku beschreibt jetzt
genau die Flaeche, die im UAT begutachtet wird — Uebersicht unterhalb des Schichtplans,
Karten- und Tabellen-Layout plus Summenzeile — und benennt ausdruecklich die Flaechen,
die sich NICHT aendern duerfen (Stundenkonto, Wochen-Histogramm, Mitarbeiterliste,
jede Balance-Zahl). Weiterhin zu beachten aus dem 64-02-SUMMARY: die Dev-Server auf
:3000 und :8080 tragen einen Build von **vor** 64-01 und muessen vor dem UAT neu
gestartet werden.

Fuer den Milestone-Close v2.7 vorgemerkt (nicht in dieser Phase zu fixen):
die vorbestehende Rohwert-Asymmetrie zwischen `Availability::Unavailable` (Stelle A) und
`ReportType::AbsenceHours` (Stelle B) ist jetzt in F07 §2.7 dokumentiert, aber nicht
behoben — sie betrifft die Bilanzpfade und braucht eine eigene Phase. Ebenso
dokumentiert, aber offen: CI laeuft `cargo clippy --workspace -- -D warnings` **ohne**
`--all-targets` (`.github/workflows/rust.yml`, `nix build`), weshalb Lib-Test-Targets
dort nie gelintet werden — so blieb eine `needless_lifetimes`-Warnung seit Phase 62
unbemerkt (in 64-01 als Rule-3-Fix beseitigt, die CI-Luecke selbst besteht weiter).

## Anmerkung zu den STATE-Zaehlern

Die bekannten Tooling-Defekte wurden umgangen: `roadmap.update-plan-progress 64` und
`state.advance-plan` wurden **nicht** aufgerufen, beide Dateien von Hand editiert. Der
ROADMAP-Diff enthaelt genau eine geaenderte Zeile (`2/4` → `3/4` in Zeile 703, nicht die
UAT-Tabelle in Zeile 114).

Beim `progress`-Block trat die vorbestehende Zaehler-Anomalie aus Phase 60 zutage:
`completed_plans` zaehlt SUMMARY-Dateien, `total_plans` zaehlt PLAN-Dateien, und Phase 60
hat 3 PLANs bei 4 SUMMARYs. Mit `total_plans: 35` waere nach diesem Plan
`completed_plans: 35` und damit 100 % herausgekommen, obwohl 64-04 noch offen ist. Daher
`total_plans: 36 / completed_plans: 35 / percent: 97` — der Zaehlerstand bleibt ehrlich,
die zugrundeliegende Anomalie ist unveraendert und steht bereits als Deferred Item
(„ROADMAP-Progress-Tabelle, Phase 60") fuer den Milestone-Close.

## Self-Check: PASSED

- `docs/features/F07-reporting-balance.md` — FOUND
- `docs/features/F07-reporting-balance_de.md` — FOUND
- `docs/domain/time-accounting.md` — FOUND
- `docs/domain/time-accounting_de.md` — FOUND
- `.planning/phases/64-planungs-soll-f-r-dynamische-vertr-ge/64-03-SUMMARY.md` — FOUND
- Commit `926c8e3f` — FOUND
- Commit `2cf46b6b` — FOUND

---
*Phase: 64-planungs-soll-f-r-dynamische-vertr-ge*
*Completed: 2026-09-29*
