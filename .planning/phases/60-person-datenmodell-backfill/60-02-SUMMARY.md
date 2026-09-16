---
phase: 60-person-datenmodell-backfill
plan: 02
subsystem: database
tags: [sqlite, sqlx, schema-lock, regression-guard, person, identity, uniqueness]

# Dependency graph
requires:
  - phase: 60-person-datenmodell-backfill
    plan: 01
    provides: "Migration 20260916000000_create-person-backfill.sql, Testmodul person_backfill.rs mit Harness (setup_pool, seed_*, run_backfill_section, count, person_id_of_*)"
provides:
  - "DB-Level-Ablehnungsnachweis der 1:1-Garantie auf beiden Seiten (`duplicate_person_id_rejected_by_db_sales_person`, `duplicate_person_id_rejected_by_db_user`)"
  - "Gegenprobe zur dauerhaften Optionalitaet: mehrere Zeilen ohne Person-Zuordnung koexistieren (D-60-03 / T-60-12)"
  - "Regressionsschutz gegen nachtraegliche Partialisierung beider UNIQUE-Indizes (`*_index_is_not_partial`, gelesen aus `sqlite_master`)"
  - "Spaltenzuschnitt-Lock auf `person` (genau vier Spalten, keine `deleted`-, keine Namensspalte)"
  - "Erhaltungsnachweis fuer `sales_person_user` inklusive beider UNIQUE-Klauseln"
  - "Bitgleichheitsnachweis der Legacy-Primaerschluessel plus unveraenderter Snapshot-Aufloesung (PERS-06)"
affects: [61-person-dao-selbstzugriff, 62-personenverwaltung, 63-person-auspraegungen, 60-03]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Schema-Assertion gegen die von SQLite gespeicherte DDL (`sqlite_master`) statt gegen den Quelltext der Migration — der erklaerende Kopfkommentar wuerde eine Textsuche verfaelschen"
    - "DB-Level-Ablehnungstest: direkter `UPDATE` ohne `expect`, Assertion auf `Result::is_err` + `sqlx::Error::Database` + UNIQUE-Fehlertext"
    - "Gegenprobe im selben Test: die Ablehnung wird gegen die erlaubte Koexistenz leerer Werte abgegrenzt, damit ein zu strenger Index nicht unbemerkt durchgeht"
    - "Mutations-Check als Gruen-Validierung: die Migration wird temporaer verletzt, um zu beweisen, dass die Guards wirklich rot werden"

key-files:
  created: []
  modified:
    - service_impl/src/test/person_backfill.rs

key-decisions:
  - "Die Gegenprobe zu D-60-03 liegt als Assertionsblock am Ende beider Ablehnungstests statt in einer eigenen Testfunktion — damit bleibt der Satz der in 60-02-PLAN.md namentlich gelisteten acht neuen Funktionen exakt eingehalten."
  - "I-10 (`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13) wurde bewusst NICHT als Rust-Test geschrieben — eine Konstante gegen sich selbst zu pruefen beweist nichts. Der Lock bleibt Quelltext-Grep im Phasen-Gate; die Datei taucht im Phasen-Diff nicht auf."
  - "Die UNIQUE-Assertion der beiden Index-Tests prueft den DDL-Praefix `CREATE UNIQUE INDEX`, nicht das Vorkommen des Wortes `UNIQUE` — der Indexname selbst enthaelt `unique_idx` und haette eine Teilstring-Suche vakuum-gruen gemacht (siehe Deviation 1)."
  - "Der Kommentar in `duplicate_person_id_rejected_by_db_sales_person` beschreibt die abwesende Fremdschluessel-Durchsetzung in Prosa statt den `PRAGMA`-Namen zu zitieren, damit der Prohibition-Grep auf 0 Treffer bleibt (siehe Deviation 2)."

patterns-established:
  - "Mutations-Check vor dem Commit eines reinen Regressionsschutz-Plans: jeder neue Guard wird einmal gegen eine absichtlich verletzte Migration gefahren. Ein Guard, der nie rot war, ist kein Guard."
  - "Praefix- statt Teilstring-Assertion bei DDL-Pruefungen, wenn der Objektname das gesuchte Schluesselwort enthaelt"

requirements-completed: [PERS-04, PERS-06]

coverage:
  - id: D1
    description: "Ein direkter SQL-Schreibversuch, zwei `sales_person`-Zeilen auf dieselbe `person.id` zeigen zu lassen, wird von der Datenbank abgelehnt — als `sqlx::Error::Database` mit UNIQUE-Bezug, und nichts wird halb geschrieben"
    requirement: "PERS-04"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#duplicate_person_id_rejected_by_db_sales_person"
        status: pass
      - kind: other
        ref: "Mutations-Check: UNIQUE-Index auf `user` zu `CREATE INDEX` degradiert -> `duplicate_person_id_rejected_by_db_user` FAILED"
        status: pass
    human_judgment: false
  - id: D2
    description: "Dasselbe fuer zwei `user`-Zeilen auf derselben `person.id` (D-60-02)"
    requirement: "PERS-04"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#duplicate_person_id_rejected_by_db_user"
        status: pass
    human_judgment: false
  - id: D3
    description: "Mehrere Zeilen mit leerem `person_id` koexistieren trotz des vollstaendigen UNIQUE-Index — die Optionalitaet aus D-60-03 bleibt unangetastet, nach der Migration angelegte Zeilen bekommen bewusst keine Person (D-60-10)"
    requirement: "PERS-04"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#duplicate_person_id_rejected_by_db_sales_person (Gegenprobe-Block)"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#duplicate_person_id_rejected_by_db_user (Gegenprobe-Block)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Die aus `sqlite_master` gelesene Definition beider Indizes beginnt mit `CREATE UNIQUE INDEX` und enthaelt keine Filterklausel"
    requirement: "PERS-04"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#sales_person_index_is_not_partial"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#user_index_is_not_partial"
        status: pass
      - kind: other
        ref: "Mutations-Check: `WHERE deleted IS NULL` an den sales_person-Index gehaengt -> `sales_person_index_is_not_partial` FAILED; `CREATE UNIQUE` zu `CREATE` degradiert -> `user_index_is_not_partial` FAILED"
        status: pass
    human_judgment: false
  - id: D5
    description: "`person` traegt genau `id`, `created`, `update_process`, `update_version` — keine `deleted`-Spalte, keine Namensspalte"
    requirement: "PERS-04"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#person_table_has_no_deleted_column"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#person_table_has_only_identity_and_audit_columns"
        status: pass
      - kind: other
        ref: "Mutations-Check: `deleted TEXT` an `person` angehaengt -> beide Tests FAILED"
        status: pass
    human_judgment: false
  - id: D6
    description: "`sales_person_user` existiert nach dem Migrationslauf weiter, inklusive beider UNIQUE-Klauseln auf `sales_person_id` und `user_id`"
    requirement: "PERS-06"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#sales_person_user_table_survives_migration"
        status: pass
    human_judgment: false
  - id: D7
    description: "`sales_person.id` ist nach dem Backfill bytegleich, `user.name` zeichengleich, und die bestehende `billing_period_sales_person`-Zeile loest unveraendert auf denselben Schluesselwert auf"
    requirement: "PERS-06"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#legacy_keys_are_bit_identical_after_backfill"
        status: pass
      - kind: regression
        ref: "SQLX_OFFLINE=true cargo test --workspace -> 861 + 66 + 33 + ... passed, 0 failed"
        status: pass
    human_judgment: false
  - id: D8
    description: "`CURRENT_SNAPSHOT_SCHEMA_VERSION` steht unveraendert auf 13"
    requirement: "PERS-06"
    verification:
      - kind: other
        ref: "grep -c 'CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13' service_impl/src/billing_period_report.rs == 1; Datei nicht im Phasen-Diff"
        status: pass
    human_judgment: false

# Metrics
duration: 7min
completed: 2026-09-16
status: complete
---

# Phase 60 Plan 02: Invarianten-Locks fuer die Person-Struktur Summary

**Acht Regressionstests nageln die von der Datenbank erzwungenen Person-Invarianten fest — die 1:1-Garantie als echte DB-Ablehnung auf beiden Seiten, die bewusste Vollstaendigkeit beider UNIQUE-Indizes aus `sqlite_master` gelesen, den Spaltenzuschnitt von `person` und die Bitgleichheit der Legacy-Schluessel — und jeder einzelne Guard wurde per Mutations-Check einmal nachweislich rot gefahren.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-09-16T06:04:56Z
- **Completed:** 2026-09-16T06:11:48Z
- **Tasks:** 2 (beide `auto`, `tdd="true"`)
- **Files modified:** 1 (`service_impl/src/test/person_backfill.rs`, +~560 Zeilen)

## Accomplishments

- **Die 1:1-Garantie ist jetzt eine ueberpruefte Zusage, keine Absicht mehr.** `duplicate_person_id_rejected_by_db_sales_person` und `duplicate_person_id_rejected_by_db_user` fahren den Angriff als direkten `UPDATE`, entpacken das `Result` bewusst nicht und assertieren drei Dinge: `is_err()`, `sqlx::Error::Database(_)` und einen Fehlertext mit UNIQUE-Bezug. Anschliessend wird die angegriffene Zeile erneut gelesen — die Ablehnung hat nichts halb geschrieben.
- **Der Test laeuft ausschliesslich gegen den eindeutigen Index, nie gegen einen Fremdschluessel** (D-60-06). Die Fremdschluessel-Durchsetzung ist im Repo nirgends eingeschaltet; ein FK-basierter Ablehnungstest waere lautlos gruen gewesen und haette nichts bewiesen.
- **Gegenprobe gegen einen zu strengen Index (T-60-12).** Beide Ablehnungstests legen nach dem Backfill je zwei weitere Zeilen an, die ohne Person-Zuordnung bleiben, und bestaetigen, dass beide akzeptiert werden und gleichzeitig koexistieren. Das ist die technische Grundlage von D-60-03 — reine HR-/Admin-Accounts und Freiwillige ohne Login bleiben dauerhaft anlegbar — und deckt zugleich D-60-10 ab: nach der Migration entstehende Zeilen bekommen in Phase 60 noch kein `person_id`.
- **Beide UNIQUE-Indizes sind gegen einen spaeteren „Aufraeum"-Pass verteidigt.** `sales_person_index_is_not_partial` und `user_index_is_not_partial` lesen die von SQLite tatsaechlich gespeicherte Definition aus `sqlite_master` — nicht den Quelltext der Migrationsdatei, deren Kopfkommentar die Abweichung vom dominanten Repo-Muster in Prosa erklaert und dabei genau die Zeichenketten enthaelt, an denen eine naive Textsuche haengenbleiben wuerde. Beide Tests tragen einen Kommentar, der D-60-01/D-60-02 als one-way-Produktentscheidung benennt.
- **`person` ist auf vier Spalten festgenagelt.** `person_table_has_only_identity_and_audit_columns` vergleicht den kompletten sortierten Spaltensatz gegen `id`, `created`, `update_process`, `update_version` — das schuetzt gleichzeitig gegen eine nachgeruestete Soft-Delete-Spalte (D-60-04) und gegen vorgezogene Namensspalten (PERS-F-01 ist Future).
- **PERS-06 ist bewiesen, nicht behauptet.** `legacy_keys_are_bit_identical_after_backfill` haelt `sales_person.id` als rohe Bytefolge und `user.name` als Zeichenkette vor dem Backfill fest, vergleicht danach und prueft zusaetzlich, dass die geseedete `billing_period_sales_person`-Zeile weiterhin per Join aufloest und auf denselben Schluesselwert zeigt. Eine Vorbedingungs-Assertion stellt sicher, dass der Backfill ueberhaupt gelaufen ist — sonst waere der Bitgleichheits-Vergleich trivial.
- **Mutations-Check als Gruen-Validierung.** Weil dieser Plan reinen Regressionsschutz liefert und die Implementierung bereits in 60-01 gelandet ist, gibt es keinen natuerlichen RED-Zustand. Ersatzweise wurde die Migration temporaer dreifach verletzt (`WHERE deleted IS NULL` an den sales_person-Index, `CREATE UNIQUE INDEX` → `CREATE INDEX` auf dem user-Index, `deleted TEXT` an `person`) und die Suite dagegen gefahren. Genau die betroffenen Guards wurden rot, alle anderen blieben gruen. Die Migration wurde danach bitgleich wiederhergestellt — sie taucht in keinem Commit dieses Plans auf.

## Task Commits

1. **Task 1: DB-Level-Ablehnungstests fuer die 1:1-Garantie (PERS-04 / SC4)** — `bd1c9c90` (test)
2. **Task 2: Schema- und Erhaltungs-Locks (I-7, I-9, I-11, I-12 / PERS-06)** — `087bd91e` (test)

## Files Created/Modified

- `service_impl/src/test/person_backfill.rs` (erweitert, 1036 Zeilen) — zwei neue Abschnitte (`Plan 60-02, Task 1` und `Plan 60-02, Task 2`), acht neue `#[tokio::test]`-Funktionen, drei neue Helfer (`table_columns`, `stored_ddl`, `normalized_ddl`) und ein um die Invarianten I-6, I-6b, I-7, I-7b, I-9, I-11, I-12 ergaenztes Modul-Kopfkommentar inklusive der Begruendung, warum I-10 kein Rust-Test ist.

Keine weitere Datei beruehrt: der Diff dieses Plans enthaelt nichts unter `dao/`, `dao_impl_sqlite/`, `service/`, `rest/`, `rest-types/` oder `migrations/`.

## Decisions Made

- **Gegenprobe als Assertionsblock statt eigener Testfunktion.** `60-02-PLAN.md` listet unter „Neu in diesem Plan" genau acht Funktionsnamen; eine neunte Funktion haette diese Liste gesprengt. Der Plan liess beide Formen ausdruecklich zu.
- **Praefix- statt Teilstring-Assertion bei `UNIQUE`.** Siehe Deviation 1 — die urspruengliche Fassung war nachweislich vakuum-gruen.
- **I-10 bleibt Grep-Lock.** Ein Rust-Test auf `CURRENT_SNAPSHOT_SCHEMA_VERSION == 13` wuerde die Konstante gegen sich selbst pruefen. Keine der vier Bump-Klauseln aus CLAUDE.md greift: `person` ist rein administrativ — kein neuer persistierter `value_type`, keine geaenderte Computation, kein geaendertes Input-Set, `billing_period_sales_person.sales_person_id` unveraendert.
- **Mutations-Check statt kuenstlichem RED.** Die Alternative — die Migration in einem RED-Commit zu verstuemmeln und in einem GREEN-Commit zu reparieren — haette eine bereits gelandete, verifizierte Migration ohne Not angefasst.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Die `UNIQUE`-Assertion beider Index-Tests war vakuum-gruen**

- **Found during:** Task 2 (Mutations-Check)
- **Issue:** Die erste Fassung pruefte `ddl.to_uppercase().contains("UNIQUE")`. Der Indexname lautet aber `user_person_id_unique_idx` bzw. `sales_person_person_id_unique_idx` — in Grossbuchstaben enthaelt bereits der **Name** die Zeichenkette `UNIQUE`. Der Mutations-Check bewies es: ein zu `CREATE INDEX` degradierter Index lief durch den Test hindurch (`user_index_is_not_partial ... ok`, obwohl die Eindeutigkeit weg war). Der Guard haette genau den Regressionsfall verpasst, gegen den er gebaut wurde.
- **Fix:** Neuer Helfer `normalized_ddl` (Whitespace auf einfache Leerzeichen normalisieren) plus Assertion auf `starts_with("CREATE UNIQUE INDEX")`. Beide Index-Tests tragen jetzt einen Kommentar, der die Falle benennt.
- **Files modified:** `service_impl/src/test/person_backfill.rs`
- **Verification:** Wiederholter Mutations-Check mit `CREATE UNIQUE INDEX` → `CREATE INDEX` auf dem user-Index: `user_index_is_not_partial ... FAILED`. Nach Wiederherstellung der Migration: 16/16 gruen.
- **Committed in:** `087bd91e` (Task-2-Commit)

**2. [Rule 3 - Blocking] Der Doc-Kommentar verletzte den `PRAGMA foreign_keys`-Grep-Lock**

- **Found during:** Task 1 (Verifikationslauf)
- **Issue:** Der erklaerende Kommentar zu D-60-06 zitierte den Pragma-Namen woertlich. Damit stieg `grep -rc 'PRAGMA foreign_keys' --include='*.rs' --include='*.sql' .` von 0 auf 1 Treffer — die Prohibition aus `60-02-PLAN.md` verlangt exakt 0. Die Prohibition zielt inhaltlich auf das *Einschalten* der Durchsetzung, ihre Verifikation ist aber ein woertlicher Grep; ein Kommentar haette das Phasen-Gate rot gemacht.
- **Fix:** Der Kommentar beschreibt die abwesende Fremdschluessel-Durchsetzung jetzt in Prosa („die Fremdschluessel-Durchsetzung ist in diesem Repo nirgends eingeschaltet") ohne den Pragma-Namen. Inhaltliche Aussage unveraendert.
- **Files modified:** `service_impl/src/test/person_backfill.rs`
- **Verification:** `grep -rl 'PRAGMA foreign_keys' --include='*.rs' --include='*.sql' . | wc -l` → 0. Danach 10/10 Tests und beide Clippy-Laeufe gruen.
- **Committed in:** `bd1c9c90` (Task-1-Commit)

**3. [Rule 3 - Blocking] `cargo` liegt nicht auf dem Default-PATH**

- **Found during:** Task 1 (erster Gate-Lauf)
- **Issue:** Die Rust-Toolchain liegt in der Nix-Devshell des Projekts, nicht auf dem PATH des Agenten — identisch zu Deviation 3 in `60-01-SUMMARY.md`.
- **Fix:** Alle Gates ueber `nix develop --command bash -c '...'` gefahren. Keine Repo-Aenderung.
- **Files modified:** keine
- **Committed in:** —

---

**Total deviations:** 3 auto-fixed (1 Bug, 2 Blocking)
**Impact on plan:** Deviation 1 verschaerft die Testaussage gegenueber dem Plan, statt sie zu lockern — ohne den Mutations-Check waere ein wirkungsloser Guard eingecheckt worden. Deviation 2 ist rein redaktionell. Deviation 3 betrifft die Werkzeugumgebung. Kein Scope Creep, keine Schemaaenderung, kein Applikationscode angefasst.

## TDD Gate Compliance

Beide Tasks tragen `tdd="true"`, aber der Liefergegenstand dieses Plans **sind** die Tests — die zu testende Implementierung ist bereits in Plan 60-01 gelandet und verifiziert. Ein RED-Gate im klassischen Sinn (`test(...)`-Commit mit rotem Test, danach `feat(...)`-Commit) ist hier strukturell nicht erreichbar; `60-02-PLAN.md` formuliert das explizit als Ziel („Acht zusaetzliche Testfunktionen, die unmittelbar nach dem Landen von Plan 60-01 gruen sind und ab dann als Regressionsschutz wirken"). Beide Commits sind folgerichtig `test(...)`-Commits ohne nachfolgenden `feat(...)`-Commit.

Als Ersatz fuer das fehlende RED-Signal wurde jeder neue Guard einmal gegen eine absichtlich verletzte Migration gefahren (siehe „Mutations-Check" oben). Ergebnis: vier von vier mutierten Invarianten wurden von genau dem zustaendigen Test rot gemeldet, alle uebrigen Tests blieben gruen — und der Check deckte dabei einen echten Defekt in einer der Assertions auf (Deviation 1). Ein Guard, der nie rot war, ist kein Guard.

## Prohibitions — Verifikationsstand

| Prohibition | Pruefung | Ergebnis |
|---|---|---|
| `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13 | `grep -c 'CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13' service_impl/src/billing_period_report.rs` | 1; Datei nicht im Diff |
| Kein FK-Ablehnungstest, FK-Durchsetzung bleibt aus | `grep -rl 'PRAGMA foreign_keys' --include='*.rs' --include='*.sql' . \| wc -l` | 0 |
| `sales_person_user` wird nicht gedroppt/veraendert | `grep -rc 'DROP TABLE sales_person_user' migrations/sqlite/` | 0 Treffer; Test liest die Tabelle aus `sqlite_master` |
| Kein DAO-/Service-/REST-Code (D-60-10) | `git diff --name-only 21d4e8c4 -- ':!.planning'` | nur `migrations/sqlite/20260916000000_create-person-backfill.sql` (aus 60-01), `service_impl/src/test/mod.rs` (aus 60-01), `service_impl/src/test/person_backfill.rs` |
| `person` ohne `deleted`- und Namensspalten | `person_table_has_only_identity_and_audit_columns` gegen `PRAGMA table_info(person)` | genau 4 Spalten |

Zusaetzliche Plan-Checks: `grep -c 'is_err()' service_impl/src/test/person_backfill.rs` → 3 (>= 2 gefordert); `grep -c 'sqlite_master' service_impl/src/test/person_backfill.rs` → 6 (>= 3 gefordert).

## Verification

- `SQLX_OFFLINE=true cargo test -p service_impl person_backfill` → **16 passed, 0 failed** (8 aus 60-01 + 8 neue)
- `SQLX_OFFLINE=true cargo test --workspace` → alle Suites gruen (u. a. 861 + 66 + 33 + 17 + 11 + 9 passed, 0 failed)
- `SQLX_OFFLINE=true cargo build --workspace` → exit 0
- `cargo clippy --workspace -- -D warnings` → exit 0
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0
- Mutations-Check (temporaer, danach zurueckgesetzt): 4 mutierte Invarianten → genau 4 rote Guards, Migration bitgleich wiederhergestellt

## Known Stubs

Keine. Dieser Plan liefert ausschliesslich Testcode ohne Platzhalter, ohne hartkodierte Leerwerte und ohne UI-Anbindung.

## Threat Flags

Keine neue sicherheitsrelevante Oberflaeche. Der Plan fuegt keinen Netzwerk-Endpunkt, keinen Auth-Pfad, keinen Dateizugriff und keine Schemaaenderung hinzu — er bestaetigt bestehende Grenzen mit Tests. Die im `<threat_model>` mit `mitigate` belegten Threats T-60-03, T-60-04, T-60-09, T-60-10 und T-60-12 haben jetzt je einen benannten, laufenden Test; T-60-11 bleibt Grep-Lock.

## User Setup Required

None.

## Next Phase Readiness

- **Bereit fuer 60-03** (Docs-Freshness): Die zu dokumentierenden Fakten stehen in „Prohibitions — Verifikationsstand" und in den Coverage-Eintraegen D1–D8 abrufbar. Insbesondere die Begruendung, warum beide Indizes vollstaendig sind, gehoert in `docs/architecture/03-data-model.md` — sie ist jetzt testgestuetzt.
- **Bereit fuer Phase 61:** Die 1:1-Garantie ist DB-seitig nachgewiesen, bevor `sales_person_user` als alte Quelle aufgeloest wird. `sales_person_user_table_survives_migration` ist der Marker, der beim Aufloesen in Phase 61 bewusst angefasst und angepasst werden muss — er faellt dort NICHT versehentlich um, sondern signalisiert den Uebergang.
- **Warnung fuer kuenftige Passes:** `sales_person_index_is_not_partial` und `user_index_is_not_partial` schuetzen eine one-way-Produktentscheidung. Ein Fehlschlag ist keine Stilfrage — er bedeutet, dass bereits vergebene Person-Identitaeten neu gebunden wuerden und historische `billing_period_sales_person`-Snapshots einer anderen Person zufielen.

---
*Phase: 60-person-datenmodell-backfill*
*Completed: 2026-09-16*

## Self-Check: PASSED

- `service_impl/src/test/person_backfill.rs` existiert auf der Platte (1036 Zeilen).
- Beide Task-Commits sind in der Git-Historie auffindbar: `bd1c9c90`, `087bd91e`.
- Alle acht in `60-02-PLAN.md` namentlich geforderten Testfunktionen existieren und laufen (`cargo test -p service_impl person_backfill` → 16 passed).
