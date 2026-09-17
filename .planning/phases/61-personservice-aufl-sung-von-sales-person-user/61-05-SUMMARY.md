---
phase: 61-personservice-aufl-sung-von-sales-person-user
plan: 05
subsystem: database
tags: [rust, sqlx, sqlite, migration, drop-table, backfill, self-assertion, person, docs]

# Dependency graph
requires:
  - phase: 60-person-datenmodell-backfill
    provides: "person-Tabelle, nullable person_id auf sales_person + user, zwei vollstaendige UNIQUE-Indizes, idempotenter Backfill"
  - phase: 61-01
    provides: "PersonDao/PersonService (Basic-Tier)"
  - phase: 61-02
    provides: "person_id-Vergabe bei Neuanlage in beiden Pfaden"
  - phase: 61-03
    provides: "alle vier lesenden Verknuepfungs-Pfade auf JOIN u.person_id = sp.person_id; markierte LEGACY-VERGLEICH-Region als Abbau-Vertrag"
  - phase: 61-04
    provides: "assign_to_user als Merge, discard_assigned_user als Split — beide Schreibpfade ohne die Verknuepfungstabelle"
provides:
  - "migrations/sqlite/20260917000000_drop-sales-person-user.sql — Nachzieh-Backfill (D-61-06), Selbst-Assertion, Aufloesung (D-61-07)"
  - "shifty_bin::person_link_migration_test — 23 Migrations-Regressionstests (18 aus Phase 60 umgezogen, 5 neu)"
  - "Teil-Migrationsketten-Harness via gefiltertem sqlx::migrate::Migrator (P-61-C)"
  - "sauberer SC2-Endzustand: kein Vorkommen des alten Tabellennamens in dao/, dao_impl_sqlite/, service_impl/ und .sqlx"
  - "BasicDaoImpl::clear_all raeumt person mit auf"
affects: [61-06, 62-personenverwaltung-liste, 63-personen-detail]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Teil-Migrationskette im Test: sqlx::migrate! einlesen, nach Version filtern, eigenen Migrator mit ..Migrator::DEFAULT bauen — so bleiben Invarianten eines frueheren Schema-Standes pruefbar, nachdem eine spaetere Migration sie aufgeloest hat"
    - "Migrations-Historien-Tests wandern ins Crate des Migrations-Runners (shifty_bin), nicht in die DAO-Schicht"
    - "DROP-Migration in drei Bloecken: Nachzieh-Backfill mit Idempotenz-Key -> TEMP-Table-CHECK-Selbst-Assertion -> DROP; der CHECK verhindert, dass der DROP eine Verknuepfung still wegwirft"
    - "DROP TABLE IF EXISTS braucht keinen Marker-Split-Kunstgriff (anders als ALTER TABLE ADD COLUMN in Phase 60)"
    - "RED-Zwischenstand einer Migration als naive Fassung (nur Block C) — beweist, dass Nachzug und Assertion tragend sind und nicht dekorativ"

key-files:
  created:
    - shifty_bin/src/person_link_migration_test.rs
    - migrations/sqlite/20260917000000_drop-sales-person-user.sql
  modified:
    - shifty_bin/src/main.rs
    - dao_impl_sqlite/src/lib.rs
    - service_impl/src/sales_person.rs
    - service_impl/src/test/mod.rs
    - service_impl/src/test/person_self_gate.rs
    - docs/architecture/03-data-model.md
    - docs/architecture/03-data-model_de.md
    - docs/architecture/diagrams/db-schema-er.mmd
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md
    - docs/features/F12-auth-session.md
    - docs/features/F12-auth-session_de.md
    - docs/arc42/05-building-block-view.md
    - .sqlx/ (1 Eintrag neu, 1 entfallen — clear_all)
  deleted:
    - service_impl/src/test/person_backfill.rs

key-decisions:
  - "P-61-C umgesetzt: das Phase-60-Testmodul wandert vollstaendig nach shifty_bin und faehrt dort gegen eine Kette, die vor DROP_MIGRATION_VERSION endet — kein Nachbau der aufgeloesten Tabelle in Testcode"
  - "clear_all bekommt DELETE FROM person NACH DELETE FROM sales_person (die Entscheidung war laut CONTEXT.md freigestellt; ohne sie bliebe ein Bestand verwaister person-Zeilen)"
  - "Die lokale Variable sales_person_username in verify_user_is_sales_person heisst jetzt assigned_username — sonst haette der SC2-Grep am Substring gehangen (siehe Deviation 1)"
  - "Der Kopfkommentar der neuen Migration vermeidet die Zeichenfolge 'deleted IS NULL' in Prosa — genau die Falle, vor der schon das Phase-60-Testmodul gewarnt hat (siehe Deviation 2)"
  - "Die historischen Migrationsdateien bleiben byte-identisch; der inzwischen veraltete Verweis auf service_impl::test::person_backfill in der Phase-60-Migration wird BEWUSST nicht korrigiert (sqlx-Ledger-Checksumme, T-61-18)"
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 13 — kein persistierter value_type beruehrt"

patterns-established:
  - "Gefilterter Migrator als Test-Harness fuer Invarianten eines abgeloesten Schema-Standes"
  - "Zweistufiger Nachweis einer DROP-Migration: Existenz-Nachweis gegen die volle Kette, Erhaltungs-Nachweis gegen die Teil-Kette plus manuellen Lauf"

requirements-completed: [PERS-05]

coverage:
  - id: P1
    description: "D-61-06: ein verknuepftes Paar mit beidseitigem person_id IS NULL (Zwischenzustand D-60-10) wird nachgezogen und traegt danach auf beiden Seiten denselben Wert bei genau einer person-Zeile"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "shifty_bin/src/person_link_migration_test.rs#nachzug_backfillt_verknuepftes_paar_ohne_person_id"
        status: pass
    human_judgment: false
  - id: P2
    description: "D-61-06 / T-61-17: zwei nie verknuepfte Objekte bekommen je eine EIGENE Person — der Nachzug zieht nichts faelschlich zusammen (das waere Fremdzugriff ueber den Self-Gate)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "shifty_bin/src/person_link_migration_test.rs#nachzug_backfillt_unverknuepfte_zeilen"
        status: pass
    human_judgment: false
  - id: P3
    description: "T-61-16: eine Verknuepfung mit zwei verschiedenen nicht-NULL person_id-Werten bricht den Lauf mit einem CHECK-constraint-Fehler ab; Block C laeuft nicht, die Tabelle existiert danach noch"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "shifty_bin/src/person_link_migration_test.rs#selbst_assertion_bricht_bei_widerspruechlicher_verknuepfung_ab"
        status: pass
    human_judgment: false
  - id: P4
    description: "D-61-06: nach vollstaendigem Phase-60-Backfill ist der Nachzug ein beweisbarer No-Op (Zeilenzahl UND jede Zuordnung identisch) — der Regelfall im gemeinsamen Deploy"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "shifty_bin/src/person_link_migration_test.rs#nachzug_ist_no_op_ohne_altzeilen"
        status: pass
    human_judgment: false
  - id: P5
    description: "SC2 / D-61-07: nach dem vollen sqlx::migrate!-Lauf fuehrt sqlite_master keinen Eintrag mehr fuer die aufgeloeste Tabelle; jede sales_person- und user-Zeile inkl. ihrer Identitaet ueberlebt den DROP unveraendert"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "shifty_bin/src/person_link_migration_test.rs#tabelle_ist_nach_vollem_migrationslauf_verschwunden"
        status: pass
      - kind: other
        ref: "Dev-DB localdb.sqlite3 real migriert: sales_person_user weg, sp-ohne-person=0, user-ohne-person=0"
        status: pass
    human_judgment: false
  - id: P6
    description: "SC2-Grep: kein Vorkommen des alten Tabellennamens in dao/src, dao_impl_sqlite/src, service_impl/src — auch nicht als Substring in einem Bezeichner und nicht in Kommentaren"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "grep -rn 'sales_person_user' dao/src dao_impl_sqlite/src service_impl/src — Exit-Code 1"
        status: pass
      - kind: other
        ref: "grep -rln 'sales_person_user' .sqlx — Exit-Code 1"
        status: pass
      - kind: other
        ref: "grep -rln 'sales_person_user' docs/ — keine Datei"
        status: pass
    human_judgment: false
  - id: P7
    description: "Alle 18 Phase-60-Migrations-Nachweise laufen unveraendert weiter, gegen eine Kette die vor DROP_MIGRATION_VERSION endet (P-61-C)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "cargo test -p shifty_bin --bin shifty_bin person_link_migration — 23 passed (18 Phase 60 + 5 Phase 61)"
        status: pass
    human_judgment: false
  - id: P8
    description: "Der Alt-Query-Anteil der D-61-09-Matrix ist abgebaut; die 11 Accept/Reject-Assertions gegen die person_id-Implementierung stehen als Dauer-Regression"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "grep -c 'LEGACY-VERGLEICH' service_impl/src/test/person_self_gate.rs == 0"
        status: pass
      - kind: integration
        ref: "cargo test -p service_impl --lib person — 143 passed, davon 11 in person_self_gate"
        status: pass
    human_judgment: false
  - id: P9
    description: "T-61-18: die historischen Migrationsdateien bleiben byte-identisch (sqlx-Ledger-Checksummen)"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "jj diff --stat --from 7800f38d -- 20240506114107_add_sales_person.sql 20260916000000_create-person-backfill.sql — 0 files changed"
        status: pass
    human_judgment: false
  - id: P10
    description: "SC3: keine Trait-Signatur beruehrt; Projekt-Gates gruen inkl. der CI-Feature-Konfiguration; CURRENT_SNAPSHOT_SCHEMA_VERSION unveraendert 13"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "jj diff --stat --from 7800f38d -- service/src/sales_person.rs dao/src/sales_person.rs — 0 files changed"
        status: pass
      - kind: other
        ref: "cargo build --workspace / cargo test --workspace / cargo clippy --workspace -- -D warnings / clippy --all-targets — alle Exit-Code 0"
        status: pass
      - kind: other
        ref: "cargo build -p shifty_bin --no-default-features --features oidc,json_logging — Exit-Code 0"
        status: pass
      - kind: manual_procedural
        ref: "timeout 45 cargo run gegen die migrierte Dev-DB — Migrationen laufen, Server bindet 127.0.0.1:3000, Cron feuert"
        status: pass
      - kind: other
        ref: "grep -n 'pub const CURRENT_SNAPSHOT_SCHEMA_VERSION' service_impl/src/billing_period_report.rs == 13"
        status: pass
    human_judgment: false

# Metrics
duration: 25min
completed: 2026-09-17
status: complete
---

# Phase 61 Plan 05: Aufloesung der Verknuepfungstabelle Summary

**Die 1:1-Verknuepfungstabelle ist per Migration aufgeloest — mit einem Nachzieh-Backfill davor und einer Selbst-Assertion dazwischen, die den `DROP` blockiert, falls dabei auch nur eine Verknuepfung ihr Gegenstueck verlieren wuerde; die Phase-60-Nachweise laufen unveraendert weiter, jetzt gegen eine Teil-Migrationskette in `shifty_bin`.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-09-17T09:02:15Z
- **Completed:** 2026-09-17T09:27:00Z
- **Tasks:** 3
- **Files modified:** 15 Quell-/Doku-Dateien (2 neu, 12 geaendert, 1 geloescht) + 1 ausgetauschter `.sqlx`-Cache-Eintrag

## Accomplishments

- **`migrations/sqlite/20260917000000_drop-sales-person-user.sql`** loest die Tabelle in drei Bloecken auf:
  - **Block A** zieht Zeilen nach, die zwischen dem Phase-60- und dem Phase-61-Deploy entstanden sein koennten (D-60-10 laesst `person_id IS NULL` ausdruecklich zu). Idempotenz-Key durchgehend `person_id IS NULL`, Gewinner-Logik identisch zu den Phase-60-Bloecken 3a-3d: die Sales Person generiert, der Benutzer **kopiert** ueber eine korrelierte Subquery. Bewusst **kein** Soft-Delete-Filter — soft-geloeschte Sales Persons brauchen ihre Identitaet dauerhaft, weil write-once-Snapshots in `billing_period_sales_person` an ihnen haengen.
  - **Block B** prueft sich selbst: `CREATE TEMP TABLE … CHECK (ok = 1)` plus ein `INSERT … SELECT CASE WHEN NOT EXISTS (…)`. Bliebe nach Block A eine Verknuepfung ohne uebereinstimmendes `person_id`-Gegenstueck, verletzt der `INSERT` den Constraint, sqlx rollt die Migrations-Transaktion zurueck — und Block C laeuft **nie**. Das ist die Mitigation zu T-61-16: Datenverlust beim `DROP` ist strukturell ausgeschlossen, nicht bloss unwahrscheinlich.
  - **Block C** ist ein einzeiliges `DROP TABLE IF EXISTS`. Anders als `ALTER TABLE … ADD COLUMN` in Phase 60 ist das von sich aus mehrfach ausfuehrbar — der dortige Marker-Split-Kunstgriff war hier nicht noetig.
- **Die Phase-60-Nachweise sind vollstaendig erhalten** (P-61-C). Das Modul ist als `shifty_bin/src/person_link_migration_test.rs` umgezogen — in das Crate, in dem der Migrations-Runner produktiv laeuft. Sein Harness baut die Kette ueber einen **gefilterten `Migrator`** nur bis vor `DROP_MIGRATION_VERSION` auf; dadurch bleiben alle 18 Invarianten (Zaehlprobe, Apply-twice-No-Op, dangling-link-Abbruch, UNIQUE-Ablehnung, Index-Vollstaendigkeit, `person`-Tabellenform) herstellbar, obwohl die volle Kette die Tabelle inzwischen aufloest. **Kein Nachbau der Tabelle in Testcode** — die Fixtures entstehen ueber die echten historischen Migrationen.
- **5 neue Tests** decken D-61-06 und SC2 ab, darunter der Konfliktfall (zwei verschiedene `person_id`-Werte an einem Paar → `CHECK constraint failed`, Tabelle danach noch da) und der No-Op-Nachweis fuer den Regelfall des gemeinsamen Deploys.
- **Der SC2-Endzustand ist sauber** — und zwar schaerfer als „keine Query mehr": `grep -rn` ueber `dao/src`, `dao_impl_sqlite/src` und `service_impl/src` liefert **null** Treffer, auch nicht als Substring in einem Bezeichner und nicht in Kommentaren. Dasselbe gilt fuer `.sqlx/` und fuer `docs/`.
- **Die Dev-DB ist real migriert.** `localdb.sqlite3` (2 Benutzer, 2 Personen, 0 Verknuepfungszeilen) traegt die neue Migration: Tabelle weg, `sales_person`-ohne-Person = 0, `user`-ohne-Person = 0. Der `cargo run`-Smoke bootet dagegen sauber.
- **Alle Gates gruen**, inklusive der CI-Konfiguration `--no-default-features --features oidc,json_logging`, die der `DROP` haette brechen koennen.

## Task Commits

Jede Task wurde atomar committet:

1. **Task 1: Migrations-Regressionstests nach shifty_bin verlagern** — `95bf9456` (refactor)
2. **Task 2: DROP-Migration mit Nachzieh-Backfill und Selbst-Assertion** — `dc5c1d1d` (feat)
3. **Task 3: Dev-DB neu migrieren, Offline-Cache erneuern, Phasen-Gates fahren** — kein eigener Commit, siehe Deviation 3

## Files Created/Modified

**Neu:**

- `migrations/sqlite/20260917000000_drop-sales-person-user.sql` — Block A/B/C mit Kopfkommentar nach dem Phase-60-Muster (Phase, D-61-06, D-61-07, Rollback-Pfad = Datei-Backup, kein Down-Migration-Pfad).
- `shifty_bin/src/person_link_migration_test.rs` — 23 Tests (18 unveraendert aus Phase 60 umgezogen, 5 neu), Harness mit gefiltertem `Migrator`, `DROP_MIGRATION_VERSION`, zusaetzlichem `setup_full_pool` fuer den SC2-Nachweis.

**Geaendert:**

- `shifty_bin/src/main.rs` — `#[cfg(test)] mod person_link_migration_test;` neben `mod integration_test;`.
- `dao_impl_sqlite/src/lib.rs` — `BasicDaoImpl::clear_all`: `DELETE` der aufgeloesten Tabelle entfernt, `DELETE FROM person;` **nach** `DELETE FROM sales_person;` ergaenzt (T-61-20). Sonst nichts angefasst.
- `service_impl/src/sales_person.rs` — eine lokale Variable in `verify_user_is_sales_person` umbenannt (siehe Deviation 1). Keine Signatur, keine Logik.
- `service_impl/src/test/person_self_gate.rs` — LEGACY-VERGLEICH-Region komplett entfernt (142 Zeilen: `seed_legacy_links`, `alt_find_sales_person_id_by_user_id`, `alt_get_assigned_user`, `byte_identity_old_vs_new_query`), Modul-Doc auf den Endzustand gezogen. Die 11 Dauer-Regressionstests stehen unveraendert.
- `service_impl/src/test/mod.rs` — `person_backfill` ausgetragen.
- Doku (je `foo.md` **und** `foo_de.md`, jeweils im selben Commit wie der Code-Diff): `docs/architecture/03-data-model{,_de}.md`, `docs/architecture/diagrams/db-schema-er.mmd`, `docs/features/F01-employee-management{,_de}.md`, `docs/features/F12-auth-session{,_de}.md`, `docs/arc42/05-building-block-view.md`.
- `.sqlx/` — ein Eintrag ausgetauscht (`clear_all` hat einen neuen Query-String).

**Geloescht:**

- `service_impl/src/test/person_backfill.rs` (1234 Zeilen) — vollstaendig nach `shifty_bin` umgezogen, kein Nachweis verloren.

## Decisions Made

- **Testmodul ganz umziehen statt aufteilen.** Der Plan (P-61-C) gab das vor, und die Alternative waere schlechter gewesen: eine Aufteilung haette die Fixture-Helfer dupliziert und die Invarianten I-1..I-14 auf zwei Crates verteilt. Der gefilterte `Migrator` kostet acht Zeilen und haelt den Rest des Moduls byte-identisch.
- **`..Migrator::DEFAULT` statt Feld-fuer-Feld-Konstruktion.** `Migrator` hat in sqlx 0.8.6 vier oeffentliche Felder (`migrations`, `ignore_missing`, `locking`, `no_tx`). Die Struct-Update-Syntax sorgt dafuer, dass ein fuenftes Feld in einer kuenftigen sqlx-Version den Build nicht bricht.
- **`DELETE FROM person;` in `clear_all` ergaenzt.** CONTEXT.md stellte das frei; der Plan entschied dafuer, und das ist richtig: ohne die Zeile bliebe nach `clear_all` ein wachsender Bestand verwaister `person`-Zeilen zurueck und verfaelschte die Zaehlproben kuenftiger Tests. Die Reihenfolge (nach `sales_person`) ist bedeutungslos, solange `person_id` keinen Fremdschluessel traegt — sie steht trotzdem hinten, damit die Absicht lesbar bleibt.
- **Die Phase-60-Migration bleibt unangetastet**, obwohl ihr Kommentar jetzt auf ein Modul verweist, das es nicht mehr gibt (`service_impl::test::person_backfill`). Ihre Checksumme ist Teil des sqlx-Ledgers; eine Korrektur wuerde den Produktions-Start blockieren (T-61-18). Der korrekte Ort fuer den Hinweis ist dieses SUMMARY und die Doku, nicht die versiegelte Migrationsdatei.
- **Kein separater RED-Commit**, wie in 61-01 bis 61-04. Der Zyklus lief im Working Tree und ist belegt (siehe TDD Gate Compliance).
- **`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13.** Keine der vier Bump-Klauseln aus `CLAUDE.md` greift: kein neuer, entfernter oder umbenannter `value_type`, keine geaenderte Berechnung, kein geaendertes Input-Set. `person` ist rein administrativ.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Lokale Variable `sales_person_username` umbenannt, damit das SC2-Grep-Gate erreichbar ist**

- **Found during:** Task 2, Acceptance-Pruefung
- **Issue:** Das harte Gate des Plans lautet woertlich `grep -rn "sales_person_user" dao/src dao_impl_sqlite/src service_impl/src` **liefert keine Treffer**. Es lieferte zwei — in `service_impl/src/sales_person.rs:409,417`, wo die lokale Variable `sales_person_username` den gesuchten Namen als **Substring** enthaelt. Inhaltlich war das ein Fehlalarm (keine Query, keine Tabellenreferenz), aber das Gate ist literal formuliert und nicht anders erfuellbar.
- **Fix:** Die Variable heisst jetzt `assigned_username` — praeziser ohnehin, denn der Wert ist der Login des zugeordneten Benutzers und kein „Sales-Person-Name". Ein Kommentar verweist auf die Migrationsdatei. Nur der Rumpf von `verify_user_is_sales_person`, keine Signatur, keine Logikaenderung.
- **Files modified:** `service_impl/src/sales_person.rs` (+9/-2)
- **Verification:** `grep -rn` Exit-Code 1; `cargo test -p service_impl --lib person` 143 passed.
- **Committed in:** `dc5c1d1d`

**2. [Rule 3 - Blocking] Kopfkommentar der Migration umformuliert, damit das „kein Soft-Delete-Filter"-Gate erreichbar ist**

- **Found during:** Task 2, Acceptance-Pruefung
- **Issue:** Das Kriterium verlangt `grep -c 'deleted IS NULL' <Migration>` == `0`. Der erste Entwurf traf es mit `1` — nicht durch einen echten Filter, sondern durch den erklaerenden Satz „Hier ist bewusst KEIN `WHERE deleted IS NULL`-Filter gesetzt". Exakt die Falle, vor der das Phase-60-Testmodul in `stored_ddl` schon warnt („deren Kopfkommentar erklaert die Abweichung in Prosa und enthaelt dabei Woerter, an denen eine naive Textsuche haengenbleiben wuerde").
- **Fix:** Umformuliert zu „kein Soft-Delete-Filter auf der `deleted`-Spalte". Die Aussage bleibt vollstaendig erhalten, die Textsuche wird wieder aussagekraeftig.
- **Files modified:** `migrations/sqlite/20260917000000_drop-sales-person-user.sql`
- **Verification:** `grep -c 'deleted IS NULL' …` == `0`; die vier `UPDATE`s tragen nachweislich keinen Filter.
- **Committed in:** `dc5c1d1d`

**3. [Rule 3 - Blocking] `.sqlx`-Regeneration von Task 3 nach Task 2 vorgezogen — Task 3 hat deshalb keinen eigenen Commit**

- **Found during:** Task 2
- **Issue:** Task 3 sieht die Cache-Regeneration als eigenen Schritt vor. `clear_all` ist aber ein `query!`-Makro, und sein Query-String aendert sich in Task 2 — ohne frischen Cache-Eintrag baut das Repo im Offline-Modus nicht (`error: set DATABASE_URL to use query macros online`). Ein roter Task-2-Commit verletzt die harten Gates aus `CLAUDE.md`.
- **Fix:** Wegwerf-SQLite im Scratchpad mit der **vollen** neuen Kette migriert, `cargo sqlx prepare --workspace -- --all-targets` in Task 2 gefahren und den Cache dort mitcommittet. Task 3 hat den Schritt anschliessend gegen die **echte** Dev-DB `localdb.sqlite3` wiederholt — Ergebnis byte-identisch, `jj status` meldet keine Aenderung. Task 3 bleibt damit inhaltlich vollstaendig (Dev-DB migriert, Cache gegen sie validiert, alle Gates gefahren), produziert aber keinen eigenen Commit, weil es nichts mehr zu schreiben gab.
- **Files modified:** `.sqlx/query-f834532…json` (neu), `.sqlx/query-dbceca0…json` (entfallen) — netto 0 Dateien, reine Rename-Operation.
- **Verification:** `cargo build --workspace` gruen ohne gesetztes `DATABASE_URL`; `grep -rln "sales_person_user" .sqlx` leer.
- **Committed in:** `dc5c1d1d`

**4. [Rule 2 - CLAUDE.md-Direktive] Docs-Freshness-Gate bedient — sieben Dateien, beide Sprachen**

- **Found during:** Task 2
- **Issue:** Die Trigger-Tabelle in `CLAUDE.md` nennt `migrations/sqlite/*.sql` ausdruecklich und verlangt dann `docs/architecture/03-data-model.md` **plus** `docs/architecture/diagrams/db-schema-er.mmd` **plus** die passenden `docs/features/F*.md` im selben Commit — in beiden Sprachfassungen. Der Plan erwaehnt keine Doku-Arbeit. Sieben Dateien beschrieben die aufgeloeste Tabelle als bestehend, das ER-Diagramm fuehrte sie als Entitaet mit zwei Beziehungen, und `F01` §Randfaelle beschrieb den User-Zuweisungs-Race noch ueber deren `UNIQUE(user_id)` — eine Aussage, die schon seit 61-04 falsch war (dort greift der 409-Pfad D-61-04).
- **Fix:** ER-Diagramm: Entitaet und beide `maps-to`-Beziehungen entfernt, Kommentar auf den Ist-Zustand gezogen. `03-data-model{,_de}`: FK-Empirie als historische Beobachtung markiert (sie bleibt der einzige Beleg, dass FK-Durchsetzung in diesem Repo ueberhaupt aktiv ist), Index-Durchsetzung von „zusaetzlich" auf „allein", Abgrenzungs-Sektion auf den erledigten Zustand inkl. D-61-06/D-61-07. `F01{,_de}`: Tabellenzeile durch einen Aufloesungs-Hinweis ersetzt, ASCII-Beziehungsdiagramm korrigiert, Migrations-Historie und §9 annotiert, Randfall „User-Zuweisungs-Race" auf Merge/Split/409 aktualisiert. `F12{,_de}`: neue Migration in der Migrations-Liste ergaenzt, §5 auf „einzige Wahrheit" gezogen. `arc42/05`: Tabellenliste des Employee-Clusters korrigiert (`person` statt der aufgeloesten Tabelle).
- **Files modified:** `docs/architecture/03-data-model{,_de}.md`, `docs/architecture/diagrams/db-schema-er.mmd`, `docs/features/F01-employee-management{,_de}.md`, `docs/features/F12-auth-session{,_de}.md`, `docs/arc42/05-building-block-view.md`
- **Verification:** `grep -rln "sales_person_user" docs/` liefert keine Datei; beide Sprachfassungen tragen dieselben Abschnitte.
- **Committed in:** `dc5c1d1d`

---

**Total deviations:** 4 auto-fixed (3 blocking, 1 CLAUDE.md-Direktive)
**Impact on plan:** Kein Scope Creep, kein Rule-4-Fall. Zwei Abweichungen sind literal formulierte Gates des Plans, die ohne die Anpassung nicht erreichbar waren; die dritte ist eine Task-Umsortierung innerhalb der harten Projekt-Gates; die vierte ist eine explizite `CLAUDE.md`-Pflicht, die der Plan nicht erwaehnt. Die Plan-Substanz ist unveraendert umgesetzt.

## TDD Gate Compliance

Task 2 traegt `tdd="true"`. Der Zyklus ist nachgewiesen:

- **RED:** Die fuenf Tests wurden vor der Migration geschrieben, und die Migrationsdatei entstand zunaechst als **naive Fassung mit nur Block C** (`DROP TABLE IF EXISTS`, ohne Nachzug und ohne Assertion). Ergebnis: **20 passed / 3 failed** — `nachzug_backfillt_verknuepftes_paar_ohne_person_id` („die Sales-Person-Seite muss nach dem Nachzug eine person_id tragen"), `nachzug_backfillt_unverknuepfte_zeilen` („eine unverknuepfte Sales Person braucht eine eigene Person") und `selbst_assertion_bricht_bei_widerspruechlicher_verknuepfung_ab` („ein widerspruechliches Paar darf nicht als Erfolg durchgehen: ()"). Das ist ein **diskriminierender** RED: er beweist, dass Block A und Block B tragend sind und nicht dekorativ.
- **GREEN:** Nach Ergaenzung von Block A und B: **23 passed / 0 failed**.
- **REFACTOR:** nicht noetig — die Migration wurde in ihrer Endform geschrieben.

Kein separater `test(...)`-Commit: ein solcher haette `cargo test` rot hinterlassen und damit die harten Gates aus `CLAUDE.md` verletzt (dieselbe Lage wie in 61-01 bis 61-04).

## Issues Encountered

- **`cargo` nicht auf dem PATH.** Alle Gates liefen ueber `nix develop --command bash -c '…'` (liefert `cargo`, `clippy`, `sqlx-cli`, `sqlite3`).
- **Kein `rustfmt` in der Dev-Shell.** `cargo fmt --all -- --check` ist ein CI-Schritt, die Nix-Shell liefert das Werkzeug aber nicht. Ein rustfmt aus dem Nix-Store widerspricht ausserdem der Repo-Formatierung und meldet Diffs in rund 200 unberuehrten Dateien; ein repo-weiter „Fix" waere deshalb schaedlich. Der neue Code wurde **von Hand formatkonform** geschrieben (100-Zeichen-Grenze, Einrueckung, Trailing Commas) und gegen die umgebenden Dateien abgeglichen. `cargo clippy --workspace --all-targets -- -D warnings` ist gruen.
- **Die Phase-60-Migration verweist jetzt auf ein Modul, das es nicht mehr gibt.** Ihr Kommentar zu D-60-08 nennt `service_impl::test::person_backfill` als den Test, der den Apply-twice-Nachweis fuehrt. Der Test existiert weiter, heisst aber `shifty_bin::person_link_migration_test`. Die Datei bleibt **bewusst** unveraendert (sqlx-Ledger-Checksumme, T-61-18) — siehe Decisions.
- **Der `cargo run`-Smoke hat wenig Datenbasis.** `localdb.sqlite3` enthaelt 0 Sales Persons, 2 Benutzer und enthielt 0 Verknuepfungszeilen; der Nachzug in Block A war dort erwartungsgemaess ein No-Op. Der Wert des Smokes liegt darin, dass die Migration gegen eine **reale, gewachsene** Datei laeuft und der Server danach bootet. Die inhaltliche Beweislast traegt `person_link_migration_test` gegen In-Memory-SQLite. Eine Sicherungskopie der Dev-DB liegt im Scratchpad (`localdb.sqlite3.vor-61-05.bak`); die Datei ist gitignored.

## Known Stubs

Keine. Alle in diesem Plan angefassten Pfade sind vollstaendig verdrahtet und getestet.

## Threat Flags

Keine neue Sicherheitsflaeche ausserhalb des `<threat_model>` des Plans. Es kommt kein Endpunkt, kein Auth-Pfad und kein Dateizugriff hinzu. Die Schema-Aenderung ist ein **Entfernen** an einer Trust-Boundary; sie ist von T-61-16 (Tampering/Datenverlust, `mitigate`) abgedeckt und durch Block B plus den Test `selbst_assertion_bricht_bei_widerspruechlicher_verknuepfung_ab` mitigiert.

## User Setup Required

None fuer die lokale Entwicklung — die Dev-DB ist bereits migriert.

**Vor dem Produktions-Deploy** bleibt das bestehende Gate aus Phase 60 in Kraft und wird durch diesen Plan **schaerfer**:

- **Datei-Backup der Produktions-SQLite-Datei ziehen.** Das ist der einzige Rollback-Pfad (D-60-09 / D-61-07); eine Down-Migration gibt es nicht und soll es nicht geben.
- **CR-02-Pre-Flight-Query** (bereits in `STATE.md` als Deploy-Gate gefuehrt) gegen die Produktions-DB laufen lassen. Sie muss **vor** dem Deploy laufen, weil die Tabelle danach nicht mehr existiert.
- Die neue Migration bricht den Serverstart ab, wenn nach dem Nachzug eine Verknuepfung ohne uebereinstimmendes `person_id`-Gegenstueck bliebe. Das ist gewollt — die Alternative waere stiller Datenverlust. Die Fehlermeldung ist wie in Phase 60 ein nicht-diagnostisches `CHECK constraint failed`; wer sie sieht, findet den Ausloeser mit der CR-02-Query im Datei-Backup.

## Next Phase Readiness

- **Phase 61 ist abgeschlossen**, sobald dieser Plan als letzter der Phase verifiziert ist. Das alte Datenmodell existiert nirgends mehr: nicht im Schema, nicht in `dao/`, `dao_impl_sqlite/`, `service_impl/`, nicht im `.sqlx`-Cache und nicht in `docs/`.
- **Phase 62 (Personenverwaltung-Liste)** kann auf einem eindeutigen Datenmodell aufsetzen: `person` ist die Identitaet, `sales_person.person_id` und `user.person_id` sind die beiden optionalen Auspraegungen, und beide UNIQUE-Indizes garantieren die 1:1-Kopplung DB-seitig.
- **`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13.** v2.7 bumpt nicht.
- **Offen (unveraendert, kein Blocker):** das CR-02-Deploy-Gate aus Phase 60, siehe *User Setup Required*.

## Self-Check: PASSED

- `migrations/sqlite/20260917000000_drop-sales-person-user.sql` existiert und enthaelt `__phase61_drop_assertion`, `CHECK (ok = 1)` und `DROP TABLE IF EXISTS`.
- `shifty_bin/src/person_link_migration_test.rs` existiert, enthaelt `DROP_MIGRATION_VERSION` und `Migrator::DEFAULT` und traegt 23 `#[tokio::test]`.
- `service_impl/src/test/person_backfill.rs` existiert nicht mehr; `grep -c 'person_backfill' service_impl/src/test/mod.rs` == `0`.
- Beide Commits sind im Repo auffindbar: `95bf9456`, `dc5c1d1d` (`jj log`).
- `grep -rn "sales_person_user" dao/src dao_impl_sqlite/src service_impl/src` — Exit-Code 1.
- `grep -rln "sales_person_user" .sqlx` und `… docs/` — jeweils keine Datei.
- `grep -c 'LEGACY-VERGLEICH' service_impl/src/test/person_self_gate.rs` == `0`; 11 `#[tokio::test]` erhalten.
- `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo clippy --workspace --all-targets -- -D warnings` und `cargo build -p shifty_bin --no-default-features --features oidc,json_logging` — alle Exit-Code 0.
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` steht weiterhin auf `13`.

---
*Phase: 61-personservice-aufl-sung-von-sales-person-user*
*Completed: 2026-09-17*
