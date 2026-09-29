---
phase: 64-planungs-soll-f-r-dynamische-vertr-ge
plan: 04
subsystem: qa
tags: [uat, manual-verification, runtime-evidence, checkpoint, testdata]

requires:
  - phase: 64-planungs-soll-f-r-dynamische-vertr-ge
    plan: 01
    provides: "planning_target_hours an beiden ShortEmployeeReport-Konstruktionsstellen, ausgeliefert von GET /report und GET /report/week/{year}/{cw}"
  - phase: 64-planungs-soll-f-r-dynamische-vertr-ge
    plan: 02
    provides: "drei Anzeigestellen in working_hours_mini_overview.rs plus der Quellcode-Invariantentest fuer PSOLL-02"
  - phase: 64-planungs-soll-f-r-dynamische-vertr-ge
    plan: 03
    provides: "geschlossenes Docs-Freshness-Gate — keine Doku-Arbeit mehr in diesem Plan"
provides:
  - ".planning/phases/64-.../64-UAT.md — Protokoll mit Gate-Ausgaben, PSOLL-02-Trefferliste, Testdaten-Inventar und auf der Leitung gemessenen Laufzeit-Belegen"
  - "Laufzeit-Nachweis an BEIDEN Konstruktionsstellen gegen die laufende Dev-Instanz (nicht nur im Test)"
affects: []

tech-stack:
  added: []
  patterns:
    - "Laufzeit-Beleg ueber die REST-Schnittstelle als Vorarbeit zu einer Browser-Sichtpruefung: die Zahl wird erst auf der Leitung gemessen, damit im Browser nur noch die Render-Kette offen ist"
    - "Reduktions-Gegenprobe durch Anlegen und Wieder-Loeschen eines einzelnen Abwesenheitseintrags — misst die Wirksamkeit, nicht nur die Existenz des Wertes"

key-files:
  created:
    - .planning/phases/64-planungs-soll-f-r-dynamische-vertr-ge/64-UAT.md
  modified: []

key-decisions:
  - "Zwei Arbeitszeitvertraege und ein Urlaubstag wurden ueber die REST-API im Dev-Bestand angelegt, weil der Bestand KEINEN einzigen Vertrag enthielt und PSOLL-01 damit unbeobachtbar gewesen waere — Eingriff vollstaendig protokolliert, Ruecknahme-Kommandos im UAT-Protokoll"
  - "Die Reduktion wurde zusaetzlich zur reinen Existenz der Zahl scharf gemessen (4h Urlaub -> Planungs-Soll 20.0 auf 16.0, Nachbarwoche unveraendert, Loeschung setzt zurueck) — das ist der eigentliche Kern der Phase und war im Plan nur als Nebenbemerkung vorgesehen"
  - "Szenario 2 wurde in seiner Beweiskraft heruntergestuft statt es als erfuellt auszugeben: ein Vorher/Nachher-Vergleich am gewachsenen Bestand ist unmoeglich, weil die Vergleichsperson vorher gar keinen Vertrag hatte"

requirements-completed: []

coverage:
  - id: D1
    description: "Ein dynamischer Vertrag liefert auf der laufenden Dev-Instanz ein Planungs-Soll > 0, waehrend das bilanzrelevante Soll 0 bleibt"
    requirement: "PSOLL-01"
    verification:
      - kind: other
        ref: "GET /report/week/2026/39 -> Rosa Verkauf: expected_hours 0.0, dynamic_hours 20.0, planning_target_hours 16.0; GET /report?year=2026&until_week=39 -> planning_target_hours 764.0 bei expected_hours 0.0"
        status: pass
      - kind: manual
        ref: "64-UAT.md Szenario 1 — Browser-Sichtpruefung in beiden Layouts und in der Summenzeile"
        status: pending
    human_judgment: true
  - id: D2
    description: "Die Reduktion um Abwesenheiten greift im dynamischen Zweig nachweislich"
    requirement: "PSOLL-01"
    verification:
      - kind: other
        ref: "4h Vacation am 2026-09-21 senkt KW39 von 20.0 auf 16.0; KW38 bleibt 20.0; Loeschung des Eintrags setzt 768.0 (Jahresstelle) wieder her"
        status: pass
    human_judgment: false
  - id: D3
    description: "Fuer den festen Vertrag ist planning_target_hours byte-identisch mit expected_hours"
    requirement: "PSOLL-04"
    verification:
      - kind: other
        ref: "Béla Verknüpft in fuenf Messungen: 20.0 = 20.0 (Woche), 768.0 = 768.0 (Jahr)"
        status: pass
      - kind: manual
        ref: "64-UAT.md Szenario 2 — Browser-Sichtpruefung"
        status: pending
    human_judgment: true
  - id: D4
    description: "Die Produktionsregion der Uebersichtskomponente liest das Planungs-Soll an genau drei Stellen, ohne Rechenausdruck und ohne Altfeld-Fallback"
    requirement: "PSOLL-02"
    verification:
      - kind: other
        ref: "Trefferliste in 64-UAT.md Teil 1 — Z. 123, 186, 238 in der Produktionsregion (Grenze #[cfg(test)] Z. 333); kein Treffer auf .expected_hours / .dynamic_hours dort"
        status: pass
      - kind: manual
        ref: "64-UAT.md Szenario 3 — Bestaetigung der Trefferliste pro Zeile durch den Nutzer"
        status: pending
    human_judgment: true
  - id: D5
    description: "Das Stundenkonto der dynamischen Person bleibt unbewegt"
    requirement: "PSOLL-04"
    verification:
      - kind: other
        ref: "Rosa Verkauf balance_hours 0.0 und overall_hours 0.0 in allen fuenf Messungen — vor, nach und nach Ruecknahme des Urlaubseintrags"
        status: pass
      - kind: manual
        ref: "64-UAT.md Szenario 4 — Browser-Sichtpruefung Stundenkonto"
        status: pending
    human_judgment: true

duration: 40min
completed: null
status: awaiting_uat
---

# Phase 64 Plan 04: UAT Summary (AWAITING HUMAN)

**Alle Gates sind neu gefahren und gruen, und das Planungs-Soll ist erstmals an der laufenden Dev-Instanz gemessen statt nur im Test behauptet — inklusive der scharfen Reduktions-Gegenprobe; die vier Browser-Sichtpruefungen und die D-64-03-Antwort stehen aus und sind NICHT vom Agenten abgehakt worden.**

> **Dieser Plan ist NICHT abgeschlossen.** `status: awaiting_uat`,
> `requirements-completed: []`. Kein Szenario in `64-UAT.md` steht auf `passed`.
> PSOLL-01 und PSOLL-02 duerfen erst nach der menschlichen Antwort als erfuellt
> markiert werden.

## Performance

- **Duration:** ~40 min (Vorflug; UAT selbst offen)
- **Tasks:** 1 von 3 (Task 2 ist der blockierende Checkpoint, Task 3 haengt daran)
- **Files modified:** 1 (1 neu, 0 geaendert) — ausschliesslich Planungsartefakt

## Task Commits

1. **Task 1: Vorflug — Gates, Trefferliste, Testdaten, Laufzeit-Messungen** — `3b60dff4` (docs)
2. **Task 2: UAT-Checkpoint** — offen, blockierend
3. **Task 3: Ergebnis protokollieren** — offen, haengt an Task 2

## Gate-Ergebnisse (echte Ausgaben, ausgezaehlt statt behauptet)

| Gate | Ergebnis |
|---|---|
| `cargo build --workspace && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings` | **EXIT 0** |
| Summe ueber alle `test result:`-Zeilen im Backend-Log | **1173 passed, 0 failed** |
| davon `service_impl` lib | `test result: ok. 963 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.79s` |
| davon Integration | `test result: ok. 104 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 69.69s` |
| Zeilen mit `^warning` / `^error` im Backend-Log | **0** |
| `cargo test --manifest-path shifty-dioxus/Cargo.toml` | `test result: ok. 860 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s` |
| `cargo clippy --manifest-path shifty-dioxus/Cargo.toml --all-targets -- -D warnings` | **EXIT 0**, 0 Warnungen |
| `cargo build --manifest-path shifty-dioxus/Cargo.toml --target wasm32-unknown-unknown` | **gruen**, `Finished dev profile ... in 15.05s` |
| beide Phase-64-Frontend-Tests namentlich im Log | `..._reads_planning_target_only ... ok`, `..._dynamic_contract_shows_planning_target ... ok` |

963 + 104 (Backend) und 860 (Frontend) decken sich exakt mit den in 64-01 und
64-02 protokollierten Zahlen — keine Drift zwischen den Wellen.

### Quellcode-Gates

| Gate | Erwartung | Gemessen |
|---|---|---|
| `available_hours: report.expected_hours` in `booking_information.rs` | 2 | **2** |
| `planning_target_hours` in `booking_information.rs` | 0 | **0** |
| `planning_target_hours` in `billing_period_report.rs` | 0 | **0** |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13` | 1 Treffer | **`billing_period_report.rs:140`** |
| Feldzugriffe in der Produktionsregion der Uebersicht | 3 | **3** (Z. 123, 186, 238) |
| `git diff --name-only -- '*.rs' '*.toml' '*.sql' docs/` | leer | **leer** |

## Die abgelesenen Zahlen

Beide Konstruktionsstellen sind ueber je einen Endpunkt direkt messbar. Die
Uebersicht unter dem Schichtplan liest **Stelle A**
(`load_working_hours_minified_for_week` → `GET /report/week/{year}/{week}`); ihre
Balance-Spalte kommt aus **Stelle B** (`get_balance_until_week` →
`GET /report?year=&until_week=`).

### Dynamischer Vertrag (Rosa Verkauf) — vorher / nachher

| Messung | `expected_hours` | `dynamic_hours` | `planning_target_hours` |
|---|---|---|---|
| **vorher** (ohne Vertragszeile, Ausgangszustand des Dev-Bestands) | 0.0 | 0.0 | **0.0** |
| Stelle A, KW 38 2026 (Vertrag, ohne Abwesenheit) | 0.0 | 20.0 | **20.0** |
| Stelle A, KW 39 2026 (Vertrag, 4h Urlaub) | 0.0 | 20.0 | **16.0** |
| Stelle B, Jahr bis KW 39, ohne Urlaub | 0.0 | 768.0 | **768.0** |
| Stelle B, Jahr bis KW 39, mit 4h Urlaub | 0.0 | 768.0 | **764.0** |

Das bilanzrelevante `expected_hours` bleibt in **jeder** Zeile 0.0 — genau wie
vor der Phase. Das Planungs-Soll traegt daneben erstmals eine Zahl.

### Fester Vertrag (Béla Verknüpft) — vorher / nachher

| Messung | `expected_hours` | `planning_target_hours` | identisch? |
|---|---|---|---|
| **vorher** (ohne Vertragszeile) | 0.0 | **0.0** | ja |
| Stelle A, KW 38 2026 | 20.0 | **20.0** | ja |
| Stelle A, KW 39 2026 | 20.0 | **20.0** | ja |
| Stelle B, Jahr bis KW 39 (drei Messungen) | 768.0 | **768.0** | ja |

**Wichtige Einschraenkung zur Beweiskraft:** Béla hatte vor diesem Plan keinen
Vertrag. Ein echter Vorher/Nachher-Vergleich an gewachsenen Daten ist damit
unmoeglich. Was gemessen ist, ist die staerkere strukturelle Aussage
(Zeichengleichheit in jeder Messung) — nicht die im Plan formulierte
historische ("dieselbe Zahl wie vor der Phase"). Das ist in `64-UAT.md`
Szenario 2 ausdruecklich so vermerkt und wird dem Nutzer so vorgelegt.

### Die Reduktion, scharf gemessen

Nicht nur „eine Zahl > 0", sondern der Kern der Phase: die Reduktionsterme
greifen jetzt im dynamischen Zweig, wo sie vorher am Bilanz-Gate
`expected_hours <= 0.0` abgeprallt waeren.

| Schritt | Stelle A KW 39 | Stelle A KW 38 | Stelle B Jahr |
|---|---|---|---|
| ohne Urlaubseintrag | 20.0 | 20.0 | 768.0 |
| 4h `Vacation` am Mo 2026-09-21 angelegt | **16.0** | 20.0 (unberuehrt) | **764.0** |
| Eintrag wieder geloescht | — | — | **768.0** (zurueck) |

Die Reduktion ist betragsgenau (−4.0h), wochenscharf (Nachbarwoche unberuehrt)
und reversibel.

### Stundenkonto der dynamischen Person

`balance_hours` und `overall_hours` von Rosa Verkauf sind in **allen fuenf**
Messungen 0.0 — vor dem Urlaubseintrag, danach und nach dessen Ruecknahme.

## Befunde

### B-64-04-1 — Der Dev-Bestand enthielt keinen einzigen Arbeitszeitvertrag

- **Beobachtet:** `GET /working-hours/for-sales-person/{id}` liefert fuer alle
  drei vorhandenen Personen `[]`. `GET /report/week/2026/39` liefert `[]`.
  `GET /report?year=2026&until_week=39` liefert fuer beide Personen
  `expected_hours 0.0, dynamic_hours 0.0, planning_target_hours 0.0`.
- **Stelle:** Dev-Datenbestand, nicht der Code. Das Gate
  `has_contract_row && dynamic_hours > 0.0` greift ohne Vertragszeile korrekt
  nicht — die 0.0 ist das **richtige** Ergebnis fuer diese Datenlage.
- **Was stattdessen gelten muss:** Die Phase ist gegen einen vertragslosen
  Bestand nicht beobachtbar. Fuer die Sichtpruefung braucht es Daten.
- **Gap-Form:** kein Code-Defekt. Behandlung: Testdaten hergestellt (siehe
  unten) und der Eingriff protokolliert.

### B-64-04-2 — `balance_hours` des festen Vertrags sprang einmalig um −12.0

- **Beobachtet:** `GET /report?year=2026&until_week=39` fuer Béla lieferte
  unmittelbar nach Vertragsanlage `balance_hours = -768.0`, ab der naechsten
  Abfrage dauerhaft `-780.0`. Der Wert bewegte sich danach bei Anlage und
  Loeschung des Urlaubseintrags kein einziges Mal mehr. `expected_hours`
  (768.0) und `planning_target_hours` (768.0) blieben in **allen** fuenf
  Messungen unveraendert.
- **Vermutete Stelle:** unklar. 39 Wochen × 20h = 780; `expected_hours` steht
  bei 768 = 780 − 12, die 12h sind die Feiertagsreduktion. `balance_hours`
  rechnet gegen das **un**reduzierte Soll — das ist die in 64-01 und 64-03
  dokumentierte, **vorbestehende** unterschiedliche Gate-Fuehrung von Bilanz-
  und Anzeigepfad. Warum die erste Messung stattdessen −768 lieferte, liess
  sich nicht klaeren: ein `carryover`-REST-Endpunkt existiert nicht und ein
  `sqlite3`-CLI stand nicht zur Verfuegung. Vermutung (unbelegt): eine erst
  beim zweiten Zugriff gefuellte Carryover-Ableitung auf einem Sekunden zuvor
  entstandenen Vertrag.
- **Was stattdessen gelten muss:** derselbe Request mit denselben Parametern
  sollte denselben `balance_hours`-Wert liefern, unabhaengig davon, wie oft er
  vorher gestellt wurde.
- **Beruehrt die Phase?** Nein. Phase 64 schreibt `balance_hours` an keiner
  Stelle, und die beiden Felder dieser Phase waren in jeder Messung stabil.
- **Empfehlung:** als Beobachtung ausserhalb von Phase 64 vormerken. Tritt nur
  auf frisch angelegten Testdaten ohne jede Buchung auf, nicht gegen gewachsenen
  Bestand reproduziert.

### DEF-6 bestaetigt (vorbestehend, nicht Scope)

`http://localhost:8080/shiftplan/2026/39` per direkter URL-Eingabe liefert
**HTTP 404**; `http://localhost:8080/` liefert 200. Das ist der in STATE.md
gefuehrte vorbestehende Routen-Defekt. Fuer das UAT muss der Nutzer deshalb von
der App-Wurzel navigieren. **Nicht** in dieser Phase gefixt.

## Eingriff in den Dev-Bestand (T-64-08)

Ausschliesslich ueber die laufende REST-API, kein direkter DB-Zugriff. **Noch
NICHT zurueckgenommen** — die Daten werden fuer das UAT gebraucht; die
Ruecknahme ist Task 3.

| # | Entitaet | ID | Inhalt |
|---|---|---|---|
| 1 | `employee_work_details` | `fd311b06-dec4-43bb-959d-1e5b8a615f0c` | Rosa Verkauf — **is_dynamic = true**, 20h/Woche, Mo–Fr, 4h/Tag, 2026 KW1 Mo – KW52 So |
| 2 | `employee_work_details` | `da1528e7-1aa2-4da0-b7cb-4df0fa99f708` | Béla Verknüpft — **is_dynamic = false**, sonst identisch |
| 3 | `extra_hours` | `dd304c75-c57f-40fd-8001-90803d88e097` | Rosa — `Vacation`, 4.0h, 2026-09-21 (Mo, KW 39) |

Zwei weitere `extra_hours`-Zeilen (`ba5d7a35-…` aus dem Datumsformat-Versuch,
`5105df79-…` aus der Reduktions-Gegenprobe) sind bereits geloescht.

**Ruecknahme (Task 3, nach dem UAT):**

```bash
curl -X DELETE http://localhost:3000/extra-hours/dd304c75-c57f-40fd-8001-90803d88e097
curl -X DELETE http://localhost:3000/working-hours/fd311b06-dec4-43bb-959d-1e5b8a615f0c
curl -X DELETE http://localhost:3000/working-hours/da1528e7-1aa2-4da0-b7cb-4df0fa99f708
```

Alle drei sind **Soft-Deletes**: die Zeilen bleiben mit gesetztem `deleted`
stehen und sind fuer die Anwendung unsichtbar. Harte Loeschung ist ueber die API
nicht moeglich; der Bestand enthielt schon vorher soft-geloeschte Artefakte aus
dem Phase-63-UAT.

## Was NICHT bewiesen ist

Ausdruecklich und ohne Beschoenigung:

1. **Die Zahl ist nicht im Browser gesehen worden.** Nur der Wert auf der
   Leitung ist gemessen. Die Kette Backend → WASM → DOM ist offen — genau der
   Grund, warum `64-VALIDATION.md` PSOLL-01 als manual-only fuehrt.
2. **Kein Layout-Vergleich.** Dass Karten-Layout, Tabellenzeile und Summenzeile
   dieselbe Zahl zeigen, ist durch die SSR-Tests aus 64-02 abgedeckt, aber nicht
   an der laufenden App beobachtet.
3. **Szenario 2 in seiner Plan-Formulierung ist nicht erbringbar** (kein
   Vorher-Wert, siehe B-64-04-1). Ersatz ist die gemessene Zeichengleichheit.
4. **PSOLL-02 ist nicht menschlich abgehakt.** Die Trefferliste liegt vor und
   ist eingeordnet; die Bestaetigung pro Zeile fehlt.
5. **Die D-64-03-Frage ist unbeantwortet.**

## Deviations from Plan

**Keine Abweichung im Sinne der Regeln 1–4.** Zwei Praezisierungen gegenueber
dem Plantext, beide innerhalb des ausdruecklich vorgesehenen Handlungsraums:

- Der Plan sah fuer den Fall „kein dynamischer Vertrag im Dev-Bestand" vor,
  „einen Vertrag im Dev-Bestand als dynamisch zu konfigurieren". Es war
  **kein** Vertrag vorhanden, der umkonfiguriert werden koennte — es mussten
  zwei neu angelegt werden. Sachlich derselbe Eingriff, groesserer Umfang,
  vollstaendig protokolliert.
- Die Reduktions-Gegenprobe (Urlaubseintrag anlegen, messen, loeschen, erneut
  messen) stand im Plan nur als Nebenbemerkung in Szenario 1. Sie wurde als
  eigenstaendige Messung durchgefuehrt, weil sie den Kern der Phase belegt und
  auf der Leitung billig zu haben war.

Kein Produktionscode angefasst: `git diff --name-only -- '*.rs' '*.toml' '*.sql'
docs/` ist leer. Das Prohibition-Gate des Plans ist eingehalten.

## Threat Flags

Keine neue Angriffsflaeche.

- **T-64-08** (Repudiation, Dev-Datenbestand) — **teilweise mitigiert.** Der
  Eingriff ist im UAT-Protokoll und hier mit IDs und Ruecknahme-Kommandos
  vermerkt. Die Ruecknahme selbst ist Task 3 und damit **noch offen**.
- **T-64-09** (Repudiation, `64-UAT.md`) — mitigiert. Beide Befunde stehen in
  Gap-Form (Beobachtung, vermutete Stelle, Soll-Verhalten), keiner als Fix.
- **T-64-SC** — entfaellt, keine Paket-Installation, kein `Cargo.toml`-Diff.

## Known Stubs

Keine.

## Self-Check: PASSED

- `.planning/phases/64-planungs-soll-f-r-dynamische-vertr-ge/64-UAT.md` — FOUND
- Commit `3b60dff4` — FOUND
- Code-Diff aus diesem Plan — leer, wie gefordert
- Kein Szenario in `64-UAT.md` auf `passed` gesetzt — bestaetigt (vier × `pending`)

## Next Steps

1. Der Nutzer geht die vier Szenarien in `64-UAT.md` Teil 5 durch und beantwortet
   die D-64-03-Frage.
2. Task 3 traegt die Ergebnisse ein und nimmt die drei Testdatensaetze zurueck.
3. Erst danach darf der Orchestrator `requirements-completed: [PSOLL-01, PSOLL-02]`
   setzen, `status` auf `complete` heben und die Phase schliessen.

---
*Phase: 64-planungs-soll-f-r-dynamische-vertr-ge*
*Status: awaiting_uat — NICHT abgeschlossen*
