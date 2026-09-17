---
phase: 61-personservice-aufl-sung-von-sales-person-user
plan: 07
subsystem: migration
tags: [migration, sqlite, backfill, deploy-window, gap-closure, review-cr-01, review-wr-05, docs-freshness-gate]

# Dependency graph
requires:
  - phase: 61-05
    provides: "DROP-Migration 20260917000000 mit Block A/B/C und das Teil-Migrationsketten-Harness in shifty_bin/src/person_link_migration_test.rs"
  - phase: 61-06
    provides: "docs/architecture/03-data-model{,_de}.md §Die dauerhaft pruefbare Invariante und §Abgrenzung — die beiden Abschnitte, die hier nachgezogen wurden"
provides:
  - "Block A der DROP-Migration mit sieben Anweisungen: Entkopplung (A3) vor Verschmelzung (A5), Traeger-Aufraeumung (A7) nach dem person-INSERT (A6)"
  - "Block B mit aeusseren Joins plus den beiden globalen Deckungstermen aus der Phase-60-Assertion"
  - "Fuenf Migrations-Regressionstests fuer die vier ueber den Applikationspfad erreichbaren Deploy-Fenster-Zustaende plus den echten Abbruchfall"
  - "Datenmodell-Doku (EN + DE), die den Nachzug als Entkoppeln/Verschmelzen/Aufraeumen beschreibt und benennt, was die Selbst-Assertion danach noch faengt"
affects: [62-personenverwaltung-liste, 63-personen-detail]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Ein Backfill, der im Deploy-Fenster entstandene Zustaende nachzieht, muss die Gewinner-Regel des Laufzeitpfads en bloc anwenden — nicht die Regel der Vorgaenger-Migration, deren Vorbedingung inzwischen eine andere ist"
    - "Bei einem vollstaendigen (nicht partiellen) UNIQUE-Index ist die Reihenfolge Entkopplung-vor-Merge ein Korrektheitsvertrag, kein Stil: die naheliegende COALESCE-Reparatur tauscht CHECK constraint failed nur gegen UNIQUE constraint failed"
    - "Eine Selbst-Assertion ueber eine Verknuepfungstabelle braucht aeussere Joins plus globale Deckungsterme — innere Joins filtern genau die Zeilen heraus, die die Assertion finden soll"
    - "Ein Test, der einen Abbruch als Sollverhalten festschreibt, schuetzt im Zweifel den Defekt; beim Gap-Closure wird er umgedreht statt ergaenzt"
    - "In-Place-Aenderung einer bereits lokal angewendeten Migration: den SHA-384-Eintrag in _sqlx_migrations gezielt nachziehen statt die Dev-DB wegzuwerfen"

key-files:
  created: []
  modified:
    - migrations/sqlite/20260917000000_drop-sales-person-user.sql
    - shifty_bin/src/person_link_migration_test.rs
    - docs/architecture/03-data-model.md
    - docs/architecture/03-data-model_de.md

decisions:
  - "Die Migrationsdatei wird IN PLACE geaendert statt per Folgemigration repariert — ein Abbruch innerhalb von 20260917000000 laesst jede spaetere Datei nie laufen (sqlx bricht Migrator::run ab). Produktiv ist nichts angewendet (D-61-06)."
  - "Der im Review vorgeschlagene COALESCE-Fix fuer A3 wurde bewusst NICHT umgesetzt: im Re-Link-Fall traegt die verdraengte Sales Person die Person des Benutzers noch, und sales_person_person_id_unique_idx ist vollstaendig — der Vorschlag tauschte einen Abbruch gegen einen anderen."
  - "Gewinner-Richtung durchgehend D-61-03/D-61-05: die Sales Person behaelt bzw. erzeugt ihre Identitaet, der Benutzer wandert, ein verdraengter Benutzer bekommt eine frische Person."
  - "ER-Diagramm, F01 und F12 bleiben unberuehrt — geaendert wurde ausschliesslich Backfill- und Assertionslogik, kein Schema-Element und kein Laufzeitpfad."
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 13 — kein persistierter value_type beruehrt."

metrics:
  duration: "~35 min"
  completed: 2026-09-17
  tasks: 3
  files-modified: 4

status: complete
---

# Phase 61 Plan 07: Gap Closure CR-01 + WR-05 Summary

Der Nachzieh-Backfill der DROP-Migration erzeugte den Widerspruch selbst, an dem
die Selbst-Assertion anschliessend abbrach; Block A konvergiert jetzt
(Entkopplung → Verschmelzung → Traeger-Aufraeumung) und Block B faengt mit
aeusseren Joins und zwei globalen Deckungstermen genau das, was Block A nicht
reparieren kann.

## Was gebaut wurde

### Task 1 + Task 2 (ein Commit, `0cb99215`)

`migrations/sqlite/20260917000000_drop-sales-person-user.sql`:

| Anweisung | Rolle | Status |
|---|---|---|
| `-- A1)` | unverknuepfte Sales Persons bekommen eine eigene Person | unveraendert |
| `-- A2)` | unverknuepfte Benutzer bekommen eine eigene Person | unveraendert |
| `-- A3)` | **Entkopplung** — ein Benutzer, der die Person einer Sales Person traegt, mit der er nicht mehr verknuepft ist, bekommt eine frische Person (D-61-05) | **neu** |
| `-- A4)` | verknuepfte Sales Persons ohne `person_id` generieren | die alte A3 |
| `-- A5)` | **Verschmelzung** jeder lebenden Verknuepfung auf die Person der Sales Person (D-61-03) | die alte A4, verallgemeinert: `IS NOT` statt `person_id IS NULL` |
| `-- A6)` | eine `person`-Zeile je referenziertem Wert | die alte A5 |
| `-- A7)` | **Traeger-Aufraeumung** — `person`-Zeilen ohne Traeger loeschen | **neu** |

Block B nutzt jetzt aeussere Joins und die beiden globalen Deckungsterme
(`sales_person` bzw. `user` ohne `person_id`), die aus der Phase-60-Assertion
(`20260916000000_create-person-backfill.sql`, Block 4) uebernommen wurden.

`shifty_bin/src/person_link_migration_test.rs`: 23 → 27 Tests im Filter
`person_link_migration` (ein Test umgedreht, vier neu).

### Task 3 (Commit `633106fc`)

`docs/architecture/03-data-model.md` + `_de.md`: der Satz „same winner logic as
Phase 60: the sales person generates, the user copies" ist durch die
Drei-Schritt-Beschreibung ersetzt; ein neuer Absatz benennt, was die
Selbst-Assertion nach der Aenderung noch faengt; der Invarianten-Abschnitt
haelt fest, dass die Migration die Haelfte „jede `person`-Zeile hat mindestens
einen Traeger" beim Lauf selbst herstellt.

## RED-Nachweis (Task 1, woertlich)

Gegen die **unveraenderte** Migration, vor Task 2:

```
test result: FAILED. 22 passed; 5 failed; 0 ignored; 0 measured; 66 filtered out

failures:
    person_link_migration_test::nachzug_gibt_verdraengtem_benutzer_eine_frische_person
    person_link_migration_test::nachzug_verschmilzt_bei_verdraengter_sales_person
    person_link_migration_test::nachzug_verschmilzt_bestands_user_mit_neuer_sales_person
    person_link_migration_test::nachzug_verschmilzt_im_fenster_verknuepftes_bestandspaar
    person_link_migration_test::selbst_assertion_bricht_bei_verwaister_verknuepfung_ab
```

Fehlertexte:

- `nachzug_verschmilzt_im_fenster_verknuepftes_bestandspaar` (F-1, CR-01 Pfad 1):
  `Database(SqliteError { code: 275, message: "CHECK constraint failed: ok = 1" })`
- `nachzug_verschmilzt_bestands_user_mit_neuer_sales_person` (F-2, CR-01 Pfad 2):
  `Database(SqliteError { code: 275, message: "CHECK constraint failed: ok = 1" })`
- `nachzug_gibt_verdraengtem_benutzer_eine_frische_person` (F-3, D-61-05):
  `Database(SqliteError { code: 275, message: "CHECK constraint failed: ok = 1" })`
- `nachzug_verschmilzt_bei_verdraengter_sales_person` (F-4, T-61-26):
  `Database(SqliteError { code: 275, message: "CHECK constraint failed: ok = 1" })`
- `selbst_assertion_bricht_bei_verwaister_verknuepfung_ab` (F-5, WR-05):
  `expect_err` schlug fehl mit `: ()` — die Migration lieferte `Ok`, die beiden
  inneren Joins hatten die verwaiste Zeile herausgefiltert. Das ist der
  Datenverlust-Pfad T-61-27 im Original.

GREEN nach Task 2: `test result: ok. 27 passed; 0 failed`.

## Der neue Wortlaut vor der Selbst-Assertion (Akzeptanzkriterium Task 2)

Die Behauptung „Ein Widerspruch entsteht nicht durch Block A (dessen Idempotency
key ueberspringt belegte Zeilen), sondern durch manuelle Eingriffe oder
Restores" ist ersatzlos entfallen. An ihrer Stelle steht:

> Den Fenster-Widerspruch (beide Seiten tragen je eine eigene Person) LOEST
> Block A inzwischen selbst auf — Entkopplung in A3, Verschmelzung in A5,
> Aufraeumung in A7. Diese Assertion steht deshalb fuer das, was Block A NICHT
> reparieren kann: eine Verknuepfungszeile, deren Elternzeile fehlt (Herkunft:
> Review-Befunde CR-01 und WR-05 der Phase 61).
>
> Die aeusseren Joins sind zwingend: mit inneren Joins faellt genau die
> verwaiste Zeile aus dem Ergebnis, die Assertion ginge gruen durch und Block C
> wuerfe die Verknuepfung still weg. Die beiden globalen Deckungsterme sind aus
> `20260916000000_create-person-backfill.sql` Block 4 uebernommen [...]

Zusaetzlich wurde der Kopfkommentar nachgezogen: er benennt jetzt beide Klassen
des Deploy-Fensters (Zeilen ohne `person_id` **und** Bestandspaare, die im
Fenster verknuepft/umgehaengt wurden), haelt fest, dass Klasse (2) der Regelfall
ist, und verweist auf `dao_impl_sqlite/src/sales_person.rs` als Quelle derselben
Gewinner-Regel.

## Warum der Review-Vorschlag NICHT uebernommen wurde

Der Review schlaegt vor, A3 per `COALESCE` die Person des verknuepften Benutzers
uebernehmen zu lassen. Im Re-Link-Fall (Szenario F-4) traegt die verdraengte
Sales Person `SPz` diese Person noch, und `sales_person_person_id_unique_idx`
ist ein vollstaendiger Unique-Index — die Anweisung liefe in `UNIQUE constraint
failed` statt in `CHECK constraint failed`. Der Test
`nachzug_verschmilzt_bei_verdraengter_sales_person` pinnt diese Gegenrichtung
dauerhaft. Umgesetzt wurde stattdessen die Richtung aus D-61-03/D-61-05.

## Anleitung: lokale Dev-DB nach der In-Place-Aenderung entsperren

Jede lokale `localdb.sqlite3`, die die alte Fassung von `20260917000000` bereits
angewendet hat, fuehrt im Ledger `_sqlx_migrations` die **alte** SHA-384-Summe.
`cargo run` scheitert dann beim Migrationslauf mit `VersionMismatch`. Die DB
muss dafuer **nicht** weggeworfen werden:

```bash
nix develop --command bash -c '
HASH=$(sha384sum migrations/sqlite/20260917000000_drop-sales-person-user.sql | cut -d" " -f1)
sqlite3 localdb.sqlite3 "UPDATE _sqlx_migrations SET checksum = x'"'"'$HASH'"'"' WHERE version = 20260917000000;"
sqlite3 localdb.sqlite3 "SELECT lower(hex(checksum)) FROM _sqlx_migrations WHERE version = 20260917000000;"
'
```

Neuer Hash dieser Fassung:
`9f2b82793b2a97095d4e2e6263e084fb7257c4ccb0b57a3335cee5671dd0debfa90cc4f6980d13efd7db401d1d297d02`
(alter Wert war `dd1ffddc591445a8c0b1b4e179b231ca...`). Nach dem Nachzug startet
`cargo run` wieder — verifiziert: `INFO rest: Running server at 127.0.0.1:3000`,
kein `VersionMismatch`, kein `Failed to run migrations`.

`localdb.sqlite3` ist nicht versioniert; dieser Schritt beruehrt keine
committete Datei.

## Docs-Freshness-Gate: bewusste Auslassungen

Das Gate aus `CLAUDE.md` zieht bei `migrations/sqlite/*.sql` drei Ziele. Zwei
davon bleiben hier bewusst unberuehrt:

- **`docs/architecture/diagrams/db-schema-er.mmd` — unveraendert.** Task 2
  aendert kein Schema-Element: keine Tabelle, keine Spalte, kein Index, kein
  Fremdschluessel. Geaendert wurde ausschliesslich Backfill- und
  Assertionslogik *innerhalb* der Migration. Ein ER-Diagramm beschreibt das
  Ergebnis-Schema, und das ist bit-identisch zum Stand nach 61-05.
  Nachweis: `jj diff --summary docs/architecture/diagrams/db-schema-er.mmd` ist
  leer.
- **`docs/features/F01-employee-management{,_de}.md` und
  `docs/features/F12-auth-session{,_de}.md` — unveraendert.** Beide beschreiben
  den Laufzeitpfad (`assign_to_user` / `discard_assigned_user` / Self-Gate).
  Dieser Plan aendert keine Trait-Signatur, keinen DAO und keinen
  Permission-Pfad; die Migration wendet die dort bereits dokumentierte
  Gewinner-Regel lediglich en bloc an. Eine Doku-Aenderung waere hier
  Redundanz, keine Drift-Behebung.

Geaendert wurde ausschliesslich `docs/architecture/03-data-model.md` und
`_de.md` — die beiden Dateien, die den Nachzug tatsaechlich beschreiben, und
zwar synchron in beiden Sprachen.

## Deviations from Plan

### 1. [Beobachtung, keine Regel] F-4 war im RED-Lauf ebenfalls rot

Der Plan sagte, `nachzug_verschmilzt_bei_verdraengter_sales_person` (F-4) „darf
schon gruen sein" und sei primaer Regressionsschutz gegen die Fehlreparatur.
Tatsaechlich lief F-4 gegen die unveraenderte Migration ebenfalls in
`CHECK constraint failed`: die alte A4 hatte den Idempotenz-Key
`person_id IS NULL`, der verdraengte Benutzer trug aber bereits eine
`person_id`, sodass die Verschmelzung ausblieb. Das ist ein *staerkerer*
Befund als geplant (der Re-Link mit verdraengter Sales-Person-Seite ist ein
fuenfter erreichbarer Abbruch-Zustand), kein Widerspruch zum Plan. Kein
Akzeptanzkriterium wurde dadurch verletzt — die Kriterien fordern F-1/F-2/F-3/F-5
als `FAILED` und schweigen zu F-4.

### 2. [Akzeptanzkriterium praezisiert] Grep-Kriterium auf den entfernten Testnamen

Das Task-1-Kriterium lautet: „enthaelt keine **Funktion** mehr, deren Name auf
`bei_widerspruechlicher_verknuepfung_ab` endet". Die Funktion ist entfernt. Ein
naiver `grep` auf den String haette dennoch angeschlagen, weil der Doc-Kommentar
des Nachfolge-Tests den alten Namen als Herkunftshinweis zitierte — genau die
Falle, die 61-05 schon zweimal produziert hat. Der Doc-Kommentar wurde deshalb
umformuliert („ersetzt die fruehere Fassung, die genau diese Fixture aufbaute
und den Abbruch der Selbst-Assertion als Sollverhalten festhielt"); der
Produktions- bzw. Testbezeichner blieb dabei unangetastet. `grep -c` auf den
String liefert jetzt 0.

### 3. [Rule 1 - Bug] `roadmap.update-plan-progress` hat erneut die falsche Tabelle beschrieben

Der State-Update-Schritt ruft `gsd-tools query roadmap.update-plan-progress 61`.
Das Tool hat die Zeile fuer Phase 61 in der Tabelle
„Backend-/Frontend-Abdeckung pro Phase (GSD-Scope-Regel)" mit
`| 61 | 7/10 | In Progress|  |` ueberschrieben und damit die inhaltliche
Begruendungszeile zerstoert. Das ist exakt derselbe Defekt, den der Commit
`fix(roadmap): repair phase-61 row in backend/frontend coverage table` (direkter
Vorgaenger dieser Welle) schon einmal repariert hat — die Tabelle hat kein
Plan-Fortschritts-Schema, das Tool trifft sie trotzdem.

Fix: die Originalzeile wiederhergestellt; die einzige gewollte
ROADMAP-Aenderung dieser Welle ist der abgehakte Wave-7-Eintrag
`- [x] 61-07-PLAN.md`. Nachfolgende Wellen sollten
`roadmap.update-plan-progress` fuer diese Phase **nicht** ungeprueft laufen
lassen — oder den Diff danach kontrollieren.

Sonst: keine Auto-Fixes nach Regel 1-3 noetig, keine architektonische Frage
(Regel 4), keine Authentifizierungs-Gates.

## Verification

| Gate | Ergebnis |
|---|---|
| `cargo test -p shifty_bin --bin shifty_bin person_link_migration` | `ok. 27 passed; 0 failed` (vorher 23) |
| `nachzug_ist_no_op_ohne_altzeilen` | `ok` — die No-Op-Zusage des Nachzugs haelt trotz A3/A5/A7 |
| `tabelle_ist_nach_vollem_migrationslauf_verschwunden` | `ok` |
| `cargo test --workspace` | alle Suites `0 failed` (u. a. 885 + 93 + 33) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo build -p shifty_bin --no-default-features --features oidc,json_logging` | Exit 0 |
| `timeout 60 cargo run` | `INFO rest: Running server at 127.0.0.1:3000`, kein Migrationsfehler |
| `grep -c -- '-- A[1-7])' <migration>` | 7 |
| `grep -c 'LEFT JOIN' <migration>` | 2 |
| `grep -c 'COUNT(\*)' <migration>` | 2 |
| Reihenfolge A3 < A5 < A6 < A7 | Zeilen 77 / 109 / 143 / 166 — aufsteigend |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION` | unveraendert 13 (`billing_period_report.rs:140`) |
| `jj diff --stat Cargo.toml Cargo.lock` | `0 files changed` (T-61-SC) |
| Ledger-Checksumme `localdb.sqlite3` | identisch zu `sha384sum` der Datei |

`.sqlx`-Cache wurde nicht neu erzeugt: dieser Plan aendert keine
`sqlx::query!`-Makro-Aufrufe. Die Migration wird per `sqlx::migrate!` zur
Compile-Zeit eingebettet, die Tests fahren `sqlx::query`/`raw_sql` zur Laufzeit
— beides liegt ausserhalb des Offline-Caches.

## Commits

| Commit | Inhalt |
|---|---|
| `0cb99215` | `fix(61-07)`: Migration (Block A sieben Anweisungen, Block B aeussere Joins + Deckungsterme, Kommentare) **und** die fuenf Regressionstests |
| `633106fc` | `docs(61-07)`: `03-data-model.md` + `03-data-model_de.md` |

Task 1 hat plangemaess **nicht** committet — ein eigenstaendiger RED-Commit
haette die harten Gates aus `CLAUDE.md` (`cargo test` gruen vor jedem Commit)
verletzt. Der RED-Nachweis steht stattdessen oben in diesem SUMMARY.

## Known Stubs

Keine.

## Threat Flags

Keine neue sicherheitsrelevante Oberflaeche. Die vier `mitigate`-Dispositionen
des Plans (T-61-24 bis T-61-27) sind umgesetzt und je durch mindestens einen
Test belegt; T-61-28 (Ledger-Checksumme) ist per Anleitung oben geschlossen.

## Fuer die naechsten Wellen

- 61-08 bis 61-10 schliessen CR-02, WR-04 und WR-06. `AUTHENTICATION.md`
  (WR-06) fuehrt weiterhin `CREATE TABLE sales_person_user` — nach dieser Welle
  unveraendert offen.
- Der Deferral Record aus `61-07-PLAN.md` (WR-01/02/03/07, IN-01…IN-07) bleibt
  unveraendert gueltig; WR-02 gehoert nach Phase 62.

## Self-Check: PASSED

Alle vier geaenderten Dateien und das SUMMARY existieren auf der Platte; beide
Commits (`0cb99215`, `633106fc`) sind im Log auffindbar.
