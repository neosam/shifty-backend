---
phase: 64-planungs-soll-f-r-dynamische-vertr-ge
plan: 04
type: uat
status: awaiting_human
created: 2026-09-29
requirements: [PSOLL-01, PSOLL-02]
---

# Phase 64 — UAT-Protokoll

> ## ⚠ Nachtrag 64-05 — die zu pruefende Zahl hat sich geaendert
>
> Das UAT von 64-04 hat ein **semantisches Loch** gefunden: 64-01 hat das
> Planungs-Soll als *Vertrags*-Groesse gebaut, es ist aber eine **Zusage**-Groesse
> [D-64-05, ROADMAP „Gesperrte Entscheidungen"]. Die ehrenamtliche Zusage
> (`employee_work_details.committed_voluntary`) fliesst seit **Plan 64-05** ein,
> reduziert um Abwesenheiten und Feiertage — proportional, nicht
> alles-oder-nichts [D-64-07].
>
> **Folge fuer dieses Protokoll:**
> - Alle in **Teil 3** protokollierten Messungen sind unter der **alten**
>   Semantik entstanden. Sie bleiben als Reihe „vor 64-05" stehen — der
>   Vergleich ist der eigentliche Beleg und darf nicht geloescht werden.
> - Die Erwartungszahlen von Szenario 1, 2 und 4 sind **nachgefuehrt** (Teil 5).
> - Es gibt ein **fuenftes Szenario** fuer die neue Klasse
>   (`expected_hours == 0` mit Zusage > 0) — der Kern des gefundenen Gaps.
> - Es gibt eine **zweite Zusatzfrage** (negatives Planungs-Soll).
> - Die beiden UAT-Vertraege wurden mit `committed_voluntary = 0` angelegt.
>   **Gegen diesen Bestand ist die neue Semantik nicht beobachtbar** — genau wie
>   B-64-04-1 fuer die alte. Teil 2 traegt dafuer einen neuen Eingriffs-Schritt.
>
> **Offener Blocker B-64-05-1:** Die Dev-Server waren beim Nachfuehren dieses
> Protokolls **gestoppt** (vom Nutzer beendet). Die neuen Zahlen in Teil 3 und
> Teil 5 sind deshalb **hergeleitet, nicht gemessen** — jede ist mit ihrer
> Arithmetik ausgeschrieben. Der Agent hat sie ausdruecklich **nicht** als
> Messung ausgegeben. Vor dem Augenpaar-Durchgang muss Schritt 2b gefahren und
> die Messreihe „nach 64-05" mit echten Ausgaben gefuellt werden.

> Einloesung der zwei Nachweise, die `64-VALIDATION.md` § Manual-Only als
> nicht-automatisierbar fuehrt (PSOLL-01 Sichtpruefung, PSOLL-02 Review pro
> Trefferstelle), plus die Augenprobe am Stundenkonto (PSOLL-04).

**Wichtig:** Kein Szenario in diesem Protokoll darf vom ausfuehrenden Agenten
selbst auf `passed` gesetzt werden. Szenario 1, 2, 4 und 5 verlangen ein
Augenpaar am laufenden Browser, Szenario 3 eine menschliche Bestaetigung der
Trefferliste. Der Agent liefert die Vorarbeit und die auf der Leitung gemessenen
Zahlen.

---

## Teil 0 — Automatisierte Gates (vom Agenten gefahren, echte Ausgaben)

| Gate | Kommando | Ergebnis |
|---|---|---|
| Backend Build + Test + Clippy | `cargo build --workspace && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings` | **EXIT 0** |
| Frontend Test + Clippy + WASM | `cargo test --manifest-path shifty-dioxus/Cargo.toml && cargo clippy ... --all-targets -- -D warnings && cargo build ... --target wasm32-unknown-unknown` | **EXIT 0** |

Gemessene Zahlen aus den Logs (nicht behauptet, ausgezaehlt):

```
Backend, Summe ueber alle test-result-Zeilen:  passed=1173  failed=0
  davon service_impl lib:  test result: ok. 963 passed; 0 failed; 0 ignored; ...; finished in 9.79s
  davon Integration:       test result: ok. 104 passed; 0 failed; 0 ignored; ...; finished in 69.69s
  Zeilen mit "^warning" oder "^error" im Backend-Log: 0

Frontend:
  test result: ok. 860 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
  test component::working_hours_mini_overview::tests::working_hours_mini_overview_reads_planning_target_only ... ok
  test component::working_hours_mini_overview::tests::working_hours_mini_overview_dynamic_contract_shows_planning_target ... ok
  Zeilen mit "^warning" im Frontend-Log: 0
  WASM: Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.05s
```

Backend 963 + 104 und Frontend 860 decken sich exakt mit den in 64-01 und 64-02
protokollierten Zahlen — keine Drift.

### Quellcode-Gates der Phase

| Gate | Erwartung | Gemessen |
|---|---|---|
| `grep -c "available_hours: report.expected_hours" service_impl/src/booking_information.rs` | 2 | **2** |
| `grep -c "planning_target_hours" service_impl/src/booking_information.rs` | 0 | **0** |
| `grep -c "planning_target_hours" service_impl/src/billing_period_report.rs` | 0 | **0** |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13` | 1 Treffer | **`service_impl/src/billing_period_report.rs:140`** |
| Feldzugriffe in der Produktionsregion von `working_hours_mini_overview.rs` | 3 | **3** (Zeilen 123, 186, 238) |
| Code-Diff aus diesem Plan (`git diff --name-only -- '*.rs' '*.toml' '*.sql' docs/`) | leer | **leer** |

---

## Teil 1 — Trefferliste fuer das PSOLL-02-Review (Gegenstand von Szenario 3)

Kommando:

```
grep -nE 'planning_target_hours|expected_hours|dynamic_hours' \
  shifty-dioxus/src/component/working_hours_mini_overview.rs
```

Die Testmodul-Grenze liegt bei `#[cfg(test)]` in **Zeile 333**; Datei-Gesamtlaenge
1166 Zeilen. Alles unterhalb 333 ist Testmodul.

### Produktionsregion (Zeile 1–332) — genau drei Treffer

| Zeile | Quelltext | Einordnung |
|---|---|---|
| 123 | `let target = working_hour.planning_target_hours;` | **reiner Feldzugriff** — Karten-Layout, Anzeigestelle 1. Rechts vom `=` steht nur der Feldzugriff und ein `;`. Kein Operator, kein `if`, kein `unwrap_or`. |
| 186 | `let total_target: f32 = props.rows.iter().map(\|r\| r.planning_target_hours).sum();` | **reiner Feldzugriff** in einer Summe ueber Personen — Summenzeile, Anzeigestelle 3. Auf den Feldnamen folgt `)`. Die Summe ist Darstellung ueber Zeilen, keine Ableitungsregel fuer den Wert einer einzelnen Person. |
| 238 | `let target = working_hour.planning_target_hours;` | **reiner Feldzugriff** — Tabellen-Layout pro Zeile, Anzeigestelle 2. Identische Form wie Zeile 123. |

**Kein Treffer auf `.expected_hours` oder `.dynamic_hours` in der Produktionsregion.**
Damit existiert dort kein Fallback auf ein Altfeld.

Die unmittelbar vorangehenden Zeilen 118–122, 183–185 und 234–237 sind
Kommentarbloecke (je eine eigene Begruendung pro Anzeigestelle, aus 64-02) — sie
enthalten die Feldnamen nicht und tauchen deshalb in der Trefferliste nicht auf.

### Testmodul (Zeile 333–1166) — 40 Treffer, alle Literale oder Test-Assertions

| Zeilen | Einordnung |
|---|---|
| 407–409 | `make_row`-Helfer: drei Struct-Feld-**Literale** (`expected_hours: target`, `planning_target_hours: target`, `dynamic_hours: target`) |
| 500–502, 532–534, 648–650, 674–676, 705–707, 758–760, 768–770, 844–846, 876–878, 1004–1006 | `WorkingHoursMini`-**Literale** in bestehenden Render-Tests; alle drei Felder tragen denselben Wert (Modell eines festen Vertrags) — genau der Neutralitaetsbeweis aus 64-02 |
| 1050–1052, 1060–1062 | Literale im neuen Test `working_hours_mini_overview_dynamic_contract_shows_planning_target`: `expected_hours: 0.0`, `planning_target_hours: 20.0` bzw. `10.0`, `dynamic_hours: 40.0` — der dynamische Fall |
| 1140 | `let field = ".planning_target_hours";` — **String-Literal** des Invariantentests |
| 1150 | `for legacy in [".expected_hours", ".dynamic_hours"]` — **String-Literale** der Altfeld-Sperre |

**Kein einziger Treffer der Datei steht in einem Rechenausdruck.**

---

## Teil 2 — Testdaten-Eingriff in den Dev-Bestand (BEFUND, dokumentiert)

**Befund B-64-04-1: Der Dev-Bestand enthielt keinen einzigen Arbeitszeitvertrag.**

Vor dem Eingriff gemessen:

```
GET /sales-person                                      -> 3 Personen
  d907198e-9620-4840-abd7-b0dea0cf5e57  Rosa Verkauf
  100e013e-2ed4-4da3-8c70-a37110d1da2e  Béla Verknüpft
  483cdedc-3943-4d60-bcab-ec5166864ad4  testo  (is_paid=false)

GET /working-hours/for-sales-person/d907198e-...       -> []
GET /working-hours/for-sales-person/100e013e-...       -> []

GET /report?year=2026&until_week=39
  Rosa: expected_hours 0.0, dynamic_hours 0.0, planning_target_hours 0.0
  Béla: expected_hours 0.0, dynamic_hours 0.0, planning_target_hours 0.0
GET /report/week/2026/39                               -> []
```

Ohne Vertragszeile greift das Gate `has_contract_row && dynamic_hours > 0.0`
korrekt nicht, und das Planungs-Soll ist per Konstruktion 0.0. Die Phase waere
gegen diesen Bestand **nicht demonstrierbar** gewesen — die Zahl > 0 haette
nirgends erscheinen koennen, auch bei fehlerfreier Implementierung nicht.

Deshalb hat der Agent ueber die laufende Anwendung (nur REST-API, kein direkter
DB-Zugriff) drei Testdatensaetze angelegt. **Die Sichtpruefung laeuft gegen eigens
hergestellte Daten, nicht gegen gewachsenen Bestand.**

| # | Entitaet | ID | Inhalt |
|---|---|---|---|
| 1 | `employee_work_details` | `fd311b06-dec4-43bb-959d-1e5b8a615f0c` | Rosa Verkauf — **is_dynamic = true**, 20h/Woche, 5 Werktage Mo–Fr, 4h/Tag, 2026 KW1 Mo bis KW52 So |
| 2 | `employee_work_details` | `da1528e7-1aa2-4da0-b7cb-4df0fa99f708` | Béla Verknüpft — **is_dynamic = false**, sonst identisch |
| 3 | `extra_hours` | `dd304c75-c57f-40fd-8001-90803d88e097` | Rosa — Kategorie `Vacation`, 4.0h, 2026-09-21 (Montag, KW 39) |

Zwei weitere `extra_hours`-Zeilen (`ba5d7a35-d098-4dc0-9ab3-254f2ca2c707` — beim
Ausprobieren des Datumsformats doppelt entstanden, und `5105df79-5cfd-4224-87ee-7b1c23ebf8b8`
— beim Reproduktionstest unten) wurden bereits geloescht und sind soft-deleted.

### Ruecknahme (gehoert zu Task 3, NACH dem UAT auszufuehren)

```bash
curl -X DELETE http://localhost:3000/extra-hours/dd304c75-c57f-40fd-8001-90803d88e097
curl -X DELETE http://localhost:3000/working-hours/fd311b06-dec4-43bb-959d-1e5b8a615f0c
curl -X DELETE http://localhost:3000/working-hours/da1528e7-1aa2-4da0-b7cb-4df0fa99f708
```

Alle drei Loeschungen sind **Soft-Deletes**: die Zeilen bleiben mit gesetztem
`deleted` in der Datenbank stehen und sind fuer die Anwendung unsichtbar. Ein
vollstaendiges Zuruecksetzen auf den Zustand von vorher (harte Loeschung) ist
ueber die API nicht moeglich und war auch vorher nicht der Fall — der Bestand
enthielt bereits soft-geloeschte Artefakte aus dem Phase-63-UAT.

### Schritt 2b (NEU, Plan 64-05) — ehrenamtliche Zusage setzen

**Befund B-64-05-2: Beide UAT-Vertraege wurden mit `committed_voluntary = 0`
angelegt.** Gegen diesen Bestand ist die neue Semantik **nicht beobachtbar** —
die Zahl wuerde sich gegenueber Teil 3 nicht bewegen, auch bei fehlerfreier
Implementierung nicht. Das ist dieselbe Falle wie B-64-04-1, eine Ebene weiter.

**Erst messen, dann setzen** (nicht annehmen — der Ist-Wert ist zu protokollieren):

```bash
# Ist-Wert von committed_voluntary VOR dem Eingriff protokollieren
curl -s http://localhost:3000/working-hours/for-sales-person/d907198e-9620-4840-abd7-b0dea0cf5e57 | jq '.[] | {id, expected_hours, is_dynamic, committed_voluntary}'
curl -s http://localhost:3000/working-hours/for-sales-person/100e013e-2ed4-4da3-8c70-a37110d1da2e | jq '.[] | {id, expected_hours, is_dynamic, committed_voluntary}'
```

| Vertrag | Person | `committed_voluntary` vor dem Eingriff | Ziel |
|---|---|---|---|
| `fd311b06-dec4-43bb-959d-1e5b8a615f0c` | Rosa Verkauf (dynamisch) | _(zu messen — erwartet `0.0`)_ | **5.0** |
| `da1528e7-1aa2-4da0-b7cb-4df0fa99f708` | Béla Verknüpft (fest) | _(zu messen — erwartet `0.0`)_ | **5.0** |

Béla bekommt die Zusage ebenfalls, damit der **feste** Fall die Differenz zum
bilanzrelevanten Soll zeigt — bei Zusage 0 waren die beiden Zahlen identisch,
und genau diese Identitaet gilt jetzt nur noch bei Zusage 0.

Setzen per `PUT /working-hours/{id}`. **Den vollstaendigen Body aus dem GET
uebernehmen und nur `committed_voluntary` aendern** — ein Teil-Body wuerde die
uebrigen Felder auf ihre Defaults ziehen:

```bash
# Muster; ID, Datumsgrenzen und Wochentage aus dem GET uebernehmen
curl -s http://localhost:3000/working-hours/for-sales-person/d907198e-9620-4840-abd7-b0dea0cf5e57 \
  | jq '.[0] | .committed_voluntary = 5.0' \
  | curl -X PUT -H 'Content-Type: application/json' --data-binary @- \
      http://localhost:3000/working-hours/fd311b06-dec4-43bb-959d-1e5b8a615f0c
# analog fuer da1528e7-1aa2-4da0-b7cb-4df0fa99f708
```

**Ruecknahme von Schritt 2b** (zusaetzlich zu den drei Loeschungen oben, NACH
dem UAT): dieselben beiden `PUT`s mit `.committed_voluntary = 0.0`. Falls
Szenario 5 gefahren wurde, gehoert dessen Ruecknahme
(`.expected_hours = 20.0` an `fd311b06-…`) ebenfalls dazu.

---

## Teil 3 — Auf der Leitung gemessene Laufzeit-Belege (Vorarbeit, ersetzt kein Augenpaar)

> **Die folgenden drei Tabellen sind die Messreihe „vor 64-05"** — entstanden
> unter der Vertrags-Semantik, mit `committed_voluntary = 0` an beiden
> Vertraegen. Sie bleiben unveraendert stehen. Die Reihe „nach 64-05" steht
> darunter.

Beide `ShortEmployeeReport`-Konstruktionsstellen sind ueber je einen Endpunkt
direkt messbar. Die Uebersicht unter dem Schichtplan liest **Stelle A**
(`load_working_hours_minified_for_week` -> `GET /report/week/{year}/{week}`);
die dort angezeigte Balance-Spalte kommt aus **Stelle B**
(`get_balance_until_week` -> `GET /report?year=&until_week=`).

### Stelle A — `GET /report/week/2026/39` (Woche mit dem 4h-Urlaub)

| Person | Vertrag | `expected_hours` | `dynamic_hours` | `planning_target_hours` | `balance_hours` |
|---|---|---|---|---|---|
| Rosa Verkauf | **dynamisch** | 0.0 | 20.0 | **16.0** | 0.0 |
| Béla Verknüpft | fest | 20.0 | 20.0 | **20.0** | −20.0 |

### Stelle A — `GET /report/week/2026/38` (Kontrollwoche ohne Urlaub)

| Person | Vertrag | `expected_hours` | `dynamic_hours` | `planning_target_hours` |
|---|---|---|---|---|
| Rosa Verkauf | **dynamisch** | 0.0 | 20.0 | **20.0** |
| Béla Verknüpft | fest | 20.0 | 20.0 | **20.0** |

### Stelle B — `GET /report?year=2026&until_week=39`

| Person | Vertrag | `expected_hours` | `dynamic_hours` | `planning_target_hours` | `balance_hours` |
|---|---|---|---|---|---|
| Rosa Verkauf | **dynamisch** | 0.0 | 768.0 | **764.0** | 0.0 |
| Béla Verknüpft | fest | 768.0 | 768.0 | **768.0** | −780.0 |

### Messreihe „nach 64-05" (Zusage 5.0h/Woche an beiden Vertraegen)

> **Status: HERGELEITET, NOCH NICHT GEMESSEN** (Blocker B-64-05-1 — die
> Dev-Server waren gestoppt). Jede Zahl ist mit ihrer Arithmetik ausgeschrieben,
> damit die Messung sie **bestaetigen oder widerlegen** kann. Weicht eine
> Messung ab, wird die Differenz offengelegt, bevor irgendeine Erwartung
> angepasst wird.

Rechenweg, einmal fuer alle Zeilen (Vertrag 20h/Woche, Mo–Fr, 4h/Tag, volle
Vertragswochen ⇒ `relation = 1.0`):

```
committed_voluntary_for_target =  5.0
dynamic_hours                  = 20.0
planning_base_for_target       = 20.0 + 5.0 = 25.0
target_reduction_scale         = 25.0 / 20.0 = 1.25          [D-64-07]
planning_target_hours          = 25.0 - Reduktion * 1.25
```

#### Stelle A — `GET /report/week/2026/39` (Woche mit dem 4h-Urlaub bei Rosa)

| Person | Vertrag | `expected_hours` | `dynamic_hours` | `planning_target_hours` | Rechenweg | vor 64-05 |
|---|---|---|---|---|---|---|
| Rosa Verkauf | **dynamisch** | 0.0 | 20.0 | **20.0** | `25 − 4 × 1.25 = 25 − 5` | 16.0 |
| Béla Verknüpft | fest | 20.0 | 20.0 | **25.0** | `25 − 0` | 20.0 |

Béla ist hier der interessante Fall: `planning_target_hours` (25.0) ist jetzt
**nicht mehr** gleich `expected_hours` (20.0) — die Differenz ist genau die
unreduzierte Zusage von 5.0. Das ist PSOLL-04 in seiner neuen Fassung.

#### Stelle A — `GET /report/week/2026/38` (Kontrollwoche ohne Urlaub)

| Person | Vertrag | `expected_hours` | `dynamic_hours` | `planning_target_hours` | Rechenweg | vor 64-05 |
|---|---|---|---|---|---|---|
| Rosa Verkauf | **dynamisch** | 0.0 | 20.0 | **25.0** | `25 − 0` | 20.0 |
| Béla Verknüpft | fest | 20.0 | 20.0 | **25.0** | `25 − 0` | 20.0 |

#### Stelle B — `GET /report?year=2026&until_week=39`

39 Vertragswochen (KW 1–39), Feiertags-Reduktion insgesamt 12.0h (aus der
Vor-Reihe: `768 = 780 − 12`), bei Rosa zusaetzlich 4.0h manueller Urlaub:

| Person | `expected_hours` | `planning_target_hours` | Rechenweg | vor 64-05 |
|---|---|---|---|---|
| Rosa Verkauf (dynamisch) | 0.0 | **955.0** | `39 × 25 − (12 + 4) × 1.25 = 975 − 20` | 764.0 |
| Béla Verknüpft (fest) | 768.0 | **960.0** | `39 × 25 − 12 × 1.25 = 975 − 15` | 768.0 |

**Was die Messung ausserdem bestaetigen muss:** `expected_hours`,
`dynamic_hours`, `balance_hours` und `overall_hours` sind in **allen** Zeilen
unveraendert gegenueber der Vor-Reihe. Bewegt sich eine dieser Zahlen, ist das
ein Befund — Plan 64-05 darf keine Bilanzgroesse angefasst haben.

### Was diese Zahlen belegen — und was nicht

**Belegt (auf der Leitung, nicht im Test):**

1. **PSOLL-01 auf der API-Ebene.** Rosa hat einen dynamischen Vertrag: das
   bilanzrelevante `expected_hours` ist 0.0 — wie vor der Phase —, waehrend
   `planning_target_hours` 20.0 (KW 38) bzw. 16.0 (KW 39) traegt. **Das ist genau
   die Zahl > 0, wo frueher 0 stand.**
2. **Die Reduktion ist scharf, nicht nur die Zahl.** 4h Urlaub am Mo 2026-09-21
   senken das Planungs-Soll der KW 39 von 20.0 auf 16.0 — exakt um den
   Urlaubsbetrag —, waehrend die Nachbarwoche 38 unveraendert 20.0 zeigt. Ueber
   das Jahr (Stelle B) faellt es von 768.0 auf 764.0. Das ist der Kern der Phase:
   die Reduktionsterme greifen jetzt auch im dynamischen Zweig, wo sie vorher am
   Bilanz-Gate `expected_hours <= 0.0` abgeprallt waeren.
   Gegenprobe gemacht: Loeschen des Urlaubseintrags setzte das Planungs-Soll
   sofort wieder auf 768.0 (Stelle B) zurueck.
3. **Zeichengleichheit fuer feste Vertraege.** Bei Béla ist
   `planning_target_hours` in jeder einzelnen Messung byte-identisch mit
   `expected_hours` (20.0 = 20.0; 768.0 = 768.0). Der feste Pfad hat sich nicht
   bewegt.
4. **Das Stundenkonto der dynamischen Person bleibt unberuehrt.** Rosas
   `balance_hours` ist in allen Messungen 0.0 — vor dem Urlaubseintrag, danach,
   und nach dessen Loeschung. Auch `overall_hours` bleibt 0.0.

**NICHT belegt — dafuer ist der Checkpoint da:**

- Dass die Zahl im **Browser** ankommt. Die Render-Kette Backend → WASM →
  DOM ist hier nicht geprueft; nur der Wert auf der Leitung ist es. Die
  SSR-Tests aus 64-02 decken die Komponente ab, nicht die laufende WASM-App.
- Dass in **beiden Layouts** (Karten und Tabelle) und in der **Summenzeile**
  dieselbe Zahl steht.
- Szenario 2 in seiner urspruenglichen Form (**„dieselbe Zahl wie vor der
  Phase"**) ist gegen diesen Bestand **nicht** erbringbar: Béla hatte vor
  diesem Plan gar keinen Vertrag, es gibt also keinen Vorher-Wert zum
  Vergleichen. Was stattdessen belegt ist, ist die staerkere strukturelle
  Aussage: `planning_target_hours == expected_hours` fuer den festen Vertrag,
  in jeder Messung. Der Nutzer bestaetigt im Browser die schwaechere, aber
  sichtbare Form: die Soll-Zahl beim festen Vertrag sieht plausibel und
  unauffaellig aus.

---

## Teil 4 — Offene Beobachtung (kein Phase-64-Befund, aber ungeklaert)

**B-64-04-2: `balance_hours` des festen Vertrags sprang einmalig um −12.0 und blieb dann stabil.**

Chronologie der Messungen an `GET /report?year=2026&until_week=39` fuer Béla:

| Zeitpunkt | `expected_hours` | `planning_target_hours` | `balance_hours` |
|---|---|---|---|
| unmittelbar nach Anlage beider Vertraege | 768.0 | 768.0 | **−768.0** |
| nach Anlage des Rosa-Urlaubs | 768.0 | 768.0 | **−780.0** |
| zweite Abfrage, identische Parameter | 768.0 | 768.0 | −780.0 |
| nach Loeschung des Rosa-Urlaubs | 768.0 | 768.0 | −780.0 |
| nach erneuter Anlage des Rosa-Urlaubs | 768.0 | 768.0 | −780.0 |

Einordnung, so weit sie belastbar ist:

- Der Sprung ist **nicht durch die Schreibvorgaenge an Rosa ausgeloest** — er
  hat sich bei Anlage und Loeschung des Urlaubs danach kein einziges Mal mehr
  bewegt.
- 39 Wochen × 20h = 780h. `expected_hours` steht bei 768 = 780 − 12; die 12h
  sind die feiertagsbedingte Reduktion. `balance_hours` −780 rechnet also gegen
  das **un**reduzierte Soll, `expected_hours` gegen das reduzierte. Das ist die
  bereits in 64-01 und 64-03 dokumentierte, **vorbestehende** unterschiedliche
  Gate-Fuehrung von Bilanz- und Anzeigepfad — nicht etwas, das diese Phase
  eingefuehrt haette.
- Warum die **erste** Messung stattdessen −768 lieferte, ist **ungeklaert**.
  Vermutung (nicht belegt): eine erst beim zweiten Zugriff gefuellte
  Carryover-/Balance-Ableitung auf einem Vertrag, der Sekunden vorher entstand.
  Ein `carryover`-REST-Endpunkt existiert nicht, und ein `sqlite3`-CLI stand
  nicht zur Verfuegung — der Verdacht liess sich nicht nachweisen.
- **Beruehrt dieses Feld die Phase?** Nein. `planning_target_hours` und
  `expected_hours` waren in **jeder** der fuenf Messungen unveraendert. Phase 64
  schreibt `balance_hours` an keiner Stelle.

**Empfehlung:** als Beobachtung ausserhalb von Phase 64 vormerken, nicht in
diesem Plan verfolgen. Sie tritt auf frisch angelegten Testdaten ohne jede
Buchung auf und ist gegen gewachsenen Bestand nicht reproduziert.

---

## Teil 5 — Die fuenf Szenarien

**Einstieg:** `http://localhost:8080/` im Browser oeffnen und von dort in den
Schichtplan navigieren, dann auf **Jahr 2026, KW 39** stellen.

> Direkte URL-Eingabe `http://localhost:8080/shiftplan/2026/39` liefert **404**
> — das ist DEF-6, ein in STATE.md vermerkter vorbestehender Routen-Defekt und
> **kein** Befund dieser Phase. Deshalb ueber die App navigieren.

Die Uebersicht liegt **unterhalb** des Schichtplan-Rasters. Der Umschalter
zwischen Karten- und Tabellen-Layout steht rechtsbuendig direkt darueber.

---

### Szenario 1 — dynamischer Vertrag zeigt eine Zahl > 0

**Person:** Rosa Verkauf · **Jahr/KW:** 2026 / 39

**Erwartet nach 64-05** (hergeleitet, Messung steht aus — Blocker B-64-05-1;
`planning_target_hours = 20.0`, Rechenweg `25 − 4 × 1.25`):

- Karten-Layout: Soll **20.0h** (nicht 0.0h). Ist-Wert 0.0h.
- Tabellen-Layout, Zeile Rosa: dieselbe **20.0h**.
- Summenzeile der Tabelle: **45.0h** (20.0 Rosa + 25.0 Béla).
- Optionale Gegenprobe KW 38: dort **25.0h** statt 20.0 — die 4h Urlaub vom
  Mo 2026-09-21 fehlen in KW 39, hochskaliert mit 1.25 also 5.0h.

**Zusaetzlich zu bestaetigen (der eigentliche Punkt von 64-05):** Die
ehrenamtliche Zusage muss in der Zahl **sichtbar** sein. Ohne Zusage zeigte die
KW 38 fuer Rosa 20.0h; mit 5.0h Zusage sind es 25.0h. Steht dort weiterhin
20.0h, ist die Zusage nicht eingeflossen und D-64-05 nicht eingeloest.

**Vorher zu erledigen:** Schritt 2b aus Teil 2 (Zusage setzen), sonst ist die
neue Semantik nicht beobachtbar.

**Erwartet vor 64-05** (historisch, gemessen): 16.0h / 16.0h / 36.0h / KW38 20.0h.

**Status:** `pending`
**Beobachtung:** _(vom Nutzer einzutragen)_

---

### Szenario 2 — fester Vertrag bleibt unauffaellig

**Person:** Béla Verknüpft · **Jahr/KW:** 2026 / 39

**Erwartet nach 64-05** (hergeleitet, Messung steht aus — Blocker B-64-05-1;
`expected_hours = 20.0`, `planning_target_hours = 25.0`):

- Soll **25.0h**, in beiden Layouts identisch.
- Die Zahl ist jetzt bewusst **groesser** als das bilanzrelevante Soll (20.0h) —
  um genau die unreduzierte Zusage von 5.0h. Das ist **keine** Auffaelligkeit,
  sondern PSOLL-04 in seiner neuen Fassung: Byte-Identitaet gilt nur noch bei
  Zusage 0.
- Gegenprobe im Stundenkonto: dort steht weiterhin **20.0h** — die Zusage
  erscheint ausschliesslich auf der Planungsflaeche.

**Einschraenkung, die der Nutzer kennen muss:** Béla hatte vor diesem Plan
keinen Vertrag; ein echter Vorher/Nachher-Vergleich am gewachsenen Bestand ist
damit nicht moeglich (siehe Teil 2, B-64-04-1). Die Byte-Identitaet bei
Zusage 0 ist auf der Leitung gemessen (Reihe „vor 64-05") und in 64-01 sowie in
Block 3 von `planning_target_fixed_contract_differs_by_commitment` automatisiert
abgesichert.

**Status:** `pending`
**Beobachtung:** _(vom Nutzer einzutragen)_

---

### Szenario 3 — PSOLL-02-Review ueber die Trefferliste

**Gegenstand:** die Tabelle in Teil 1.

**Zu bestaetigen:** Die drei Treffer der Produktionsregion (Zeilen 123, 186, 238)
sind reine Feldzugriffe — kein Rechenausdruck, keine Fallback-Bedingung, kein
`unwrap_or`. Kein Treffer auf `.expected_hours` oder `.dynamic_hours` in der
Produktionsregion. Alle uebrigen Treffer liegen im Testmodul ab Zeile 333.

**Nacherhebung nach 64-05** (gemessen, nicht angenommen — Plan 64-05 hat die
Anker-Kommentare an allen drei Stellen und das Feld-Doc-Comment in
`state/employee_work_details.rs` auf die Zusage-Semantik nachgezogen):

| Gate | Erwartung | Gemessen nach 64-05 |
|---|---|---|
| Testmodul-Grenze `#[cfg(test)]` | — | **Zeile 333** (unveraendert) |
| `planning_target_hours`-Zugriffe in der Produktionsregion | 3 | **3** (Zeilen 123, 186, 238 — unveraendert) |
| `.expected_hours` / `.dynamic_hours` in der Produktionsregion | 0 | **0** |
| `planning_target_hours` im Ladepfad (`loader.rs:456`) | 1:1-Durchreichung | **`planning_target_hours: report.planning_target_hours`** — keine Rechnung |
| Frontend-Tests | 860 | **860 passed / 0 failed** (Baseline unveraendert) |

Die Kommentare nennen die Altfelder bewusst **ausgeschrieben** („das
bilanzrelevante Soll") statt als Feldpfad — der Invariantentest grept die
Produktionsregion und zaehlt Kommentare mit.

**Status:** `pending`
**Beobachtung:** _(vom Nutzer einzutragen)_

---

### Szenario 4 — Stundenkonto der dynamischen Person unbewegt

**Person:** Rosa Verkauf · **Ansicht:** Stundenkonto / Mitarbeiter-Report

**Erwartet nach 64-05** (hergeleitet, Messung steht aus — Blocker B-64-05-1):
Balance **0.0h**, Soll dort weiterhin **0.0h**, Ist 0.0h. Die 20.0h aus
Szenario 1 tauchen hier ausdruecklich **nicht** auf — das ist der Entwurf der
Phase: Planungs-Soll auf der Planungsflaeche, bilanzrelevantes Soll auf der
Bilanzflaeche.

**Der 64-05-Zusatz:** Die ehrenamtliche Zusage von 5.0h/Woche darf **nirgends**
im Stundenkonto erscheinen — weder im Soll, noch in der Balance, noch im Ist.
Die Trennung von bezahltem Soll und Ehrenamt gilt fuer die Bilanz unveraendert;
nur die Planungsflaeche fuehrt beide zusammen [D-64-05]. Eine Bewegung hier
waere ein Befund.

**Erwartet vor 64-05** (historisch, gemessen): identisch 0.0 / 0.0 / 0.0.

**Status:** `pending`
**Beobachtung:** _(vom Nutzer einzutragen)_

---

### Szenario 5 (NEU, Plan 64-05) — 0-Stunden-Vertrag mit Zusage zeigt eine Zahl > 0

**Der Kern des in 64-04 gefundenen Gaps.** Das 64-01-Gate
(`has_contract_row && dynamic_hours > 0.0`) lieferte fuer eine Person mit
`expected_hours == 0` und echter Zusage **0.0** — obwohl sie zugesagt hat. Diese
Klasse ist in keinem der vier bisherigen Szenarien abgedeckt.

**Person:** Rosa Verkauf · **Jahr/KW:** 2026 / **38** (Kontrollwoche ohne Urlaub)

**Vorbereitung — nach Szenario 1, 2 und 4, nicht davor** (dieser Schritt
zerstoert die Datenlage der vorigen Szenarien):

```bash
# Rosas Vertrag temporaer auf 0 Vertragsstunden setzen, Zusage bleibt 5.0
curl -s http://localhost:3000/working-hours/for-sales-person/d907198e-9620-4840-abd7-b0dea0cf5e57 \
  | jq '.[0] | .expected_hours = 0.0 | .committed_voluntary = 5.0' \
  | curl -X PUT -H 'Content-Type: application/json' --data-binary @- \
      http://localhost:3000/working-hours/fd311b06-dec4-43bb-959d-1e5b8a615f0c
```

**Erwartet** (hergeleitet, Messung steht aus — Blocker B-64-05-1):

```
dynamic_hours            = 0.0 * 1.0 = 0.0
committed_voluntary      = 5.0
planning_base_for_target = 0.0 + 5.0 = 5.0
target_reduction_scale   = 1.0   (weil dynamic_hours == 0)
Reduktion in KW 38       = 0.0
planning_target_hours    = 5.0
```

- Karten- und Tabellen-Layout, Zeile Rosa: Soll **5.0h** — **nicht 0.0h**.
- `expected_hours` bleibt **0.0h**; das Stundenkonto bewegt sich nicht.
- **Das 64-01-Gate lieferte hier 0.0h.** Steht dort 0.0h, ist der Gap nicht
  geschlossen.

**Ruecknahme** (`.expected_hours` zurueck auf 20.0):

```bash
curl -s http://localhost:3000/working-hours/for-sales-person/d907198e-9620-4840-abd7-b0dea0cf5e57 \
  | jq '.[0] | .expected_hours = 20.0' \
  | curl -X PUT -H 'Content-Type: application/json' --data-binary @- \
      http://localhost:3000/working-hours/fd311b06-dec4-43bb-959d-1e5b8a615f0c
```

**Status:** `pending`
**Beobachtung:** _(vom Nutzer einzutragen)_

---

## Zusatzfrage D-64-03

Die Uebersicht zeigt fuer dynamische Vertraege jetzt eine Zahl, die das
Stundenkonto nicht bewegt. Es wurde bewusst **kein** visueller Hinweis darauf
eingebaut — kein Tooltip, kein Icon, kein neuer i18n-Key —, weil dieselbe Zahl
fuer feste Vertraege schon immer an dieser Stelle stand.

**Frage:** Reicht das so, oder soll ein Hinweis als Folgeaufgabe erfasst werden?

**Antwort:** `pending`

---

## Zusatzfrage D-64-05-Q1 (NEU, Plan 64-05) — negatives Planungs-Soll

Seit Plan 64-05 ist ein **negatives** Planungs-Soll ausdruecklich *assertiertes*
Verhalten, kein Unfall. Es entsteht, wenn die Reduktionen (manuelle
Abwesenheits-Stunden, abgeleitete Abwesenheiten, Feiertage) groesser sind als die
Planungs-Basis. Konkret gemessen im Test
`planning_target_not_clamped_when_reductions_exceed_base`: ein 0-Stunden-Vertrag
mit 5.0h Zusage und einer manuellen Urlaubs-Zeile von 20.0h ergibt ein
Planungs-Soll von **−10.0h** (im Test mit Zusage 10.0).

**Warum nicht geklemmt wird:** Der Bilanzpfad klemmt ebenfalls nicht. Ein
`.max(0.0)` nur am Planungs-Soll wuerde die Konsistenz zwischen den beiden
Groessen brechen — wer klemmt, muss beide klemmen oder keine.

**Frage:** Ist ein negatives Planungs-Soll fuer die planende Person lesbar, oder
soll die Anzeige es abfangen (z. B. bei 0 anzeigen und den Ueberhang separat
kennzeichnen)? Das ist eine **Produktfrage**, keine technische — die Zahl ist
fachlich korrekt, aber „−10.0h einplanen" ist als Anweisung sinnlos.

**Antwort:** `pending`

---

## Ergebnis

**Status:** `awaiting_human` — kein Szenario abgeschlossen. Die Vorarbeit
(Gates, Trefferliste, Testdaten, Laufzeit-Messungen) ist vollstaendig; die
**fuenf** Sichtpruefungen und die Antworten auf D-64-03 und D-64-05-Q1 stehen
aus.

**Zusaetzlich offen nach Plan 64-05:**

- **B-64-05-1** — die Messreihe „nach 64-05" in Teil 3 ist hergeleitet, nicht
  gemessen (Dev-Server gestoppt). Vor dem Augenpaar-Durchgang: Server starten,
  Schritt 2b fahren, Zahlen mit echten Ausgaben ersetzen.
- **B-64-05-2** — Schritt 2b (Zusage 5.0h an beiden Vertraegen) ist noch nicht
  ausgefuehrt. Ohne ihn ist die neue Semantik nicht beobachtbar.
- Die Ruecknahme umfasst jetzt zusaetzlich die beiden `PUT`s aus Schritt 2b und,
  falls Szenario 5 gefahren wurde, dessen `expected_hours`-Ruecksetzung.
