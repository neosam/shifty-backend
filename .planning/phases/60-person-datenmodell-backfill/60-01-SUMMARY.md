---
phase: 60-person-datenmodell-backfill
plan: 01
subsystem: database
tags: [sqlite, sqlx, migration, backfill, idempotency, person, identity]

# Dependency graph
requires:
  - phase: 54-rebooking-batch
    provides: 2026er Migrations-Konvention (CREATE TABLE IF NOT EXISTS, Audit-Spalten created/update_process/update_version, CREATE UNIQUE INDEX nach ADD COLUMN)
  - phase: 04-absence-cutover
    provides: Idempotenz-Kommentar-Konvention fuer Backfill-Migrationen ("Idempotency key: ... Re-run skips already-mapped rows")
provides:
  - "Tabelle `person` (id, created, update_process, update_version) als Identitaets-Elterntabelle"
  - "Nullable Spalte `sales_person.person_id` mit vollstaendigem UNIQUE-Index `sales_person_person_id_unique_idx`"
  - "Nullable Spalte `user.person_id` mit vollstaendigem UNIQUE-Index `user_person_id_unique_idx`"
  - "Idempotenter SQL-Backfill (Bloecke 3a-3e) zwischen den Markern `-- BACKFILL START` / `-- BACKFILL END`"
  - "Selbst-abbrechende Zaehlproben-Assertion innerhalb der Migrations-Transaktion (D-60-07)"
  - "Integrationstest-Modul `service_impl/src/test/person_backfill.rs` mit acht Testfunktionen und vier Fixture-Auspraegungen"
affects: [61-person-dao-selbstzugriff, 62-personenverwaltung, 63-person-auspraegungen, 60-02, 60-03]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Marker-Split-Apply-twice: `-- BACKFILL START`/`-- BACKFILL END` in der .sql-Datei, Test schneidet die Region per `include_str!` + `split_once` heraus und fuehrt sie per `sqlx::raw_sql` erneut aus"
    - "Selbst-abbrechende Migration: CREATE TEMP TABLE mit CHECK-Constraint als Ersatz fuer das in SQLite fehlende freistehende RAISE()"
    - "Korrelierte Wert-Kopie statt zweifachem randomblob(): eine Seite eines 1:1-Paars wuerfelt, die andere liest zurueck"

key-files:
  created:
    - migrations/sqlite/20260916000000_create-person-backfill.sql
    - service_impl/src/test/person_backfill.rs
  modified:
    - service_impl/src/test/mod.rs

key-decisions:
  - "D-60-01 bestaetigt (Checkpoint Task 1, auto-mode): Der UNIQUE-Index auf `sales_person.person_id` bleibt vollstaendig statt partiell — soft-geloeschte Sales Persons behalten ihre Person-Identitaet dauerhaft. One-way."
  - "Apply-twice-Nachweis ueber Marker-Split statt vollem Datei-Replay (RESEARCH Open Question 1): SQLite kennt fuer `ALTER TABLE ... ADD COLUMN` keine Existenz-Guard-Klausel."
  - "`run_backfill_section` gibt bei leerem Markerbereich frueh `Ok(())` zurueck — `sqlx::raw_sql` lehnt leere Skript-Strings ab; der Guard machte den sauberen RED-Zustand in Task 2 erst moeglich."
  - "Drei Tests tragen zusaetzliche Vorbedingungs-Assertions (`person_count > 0`, `!rows.is_empty()`, alle Zuordnungen gesetzt), damit sie nicht vakuum-gruen werden koennen."

patterns-established:
  - "Marker-gebundener Backfill-Abschnitt: die Migrationsdatei ist Vertragspartner des Apply-twice-Tests; Markerumbenennung schaltet den Nachweis stumm ab (im Kopfkommentar festgehalten)"
  - "Migrations-Selbstverifikation: Zaehlprobe als CHECK-TEMP-Tabelle INNERHALB der Marker, damit auch der zweite Lauf sie passieren muss"

requirements-completed: [PERS-01, PERS-02, PERS-03]

coverage:
  - id: D1
    description: "Tabelle `person` mit genau vier Spalten (id, created, update_process, update_version), ohne deleted-Spalte und ohne Namensspalten"
    requirement: "PERS-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#person_id_parses_as_uuid"
        status: pass
      - kind: other
        ref: "sqlite3 localdb.sqlite3 'PRAGMA table_info(person);' -> 4 Spalten, kein `deleted`"
        status: pass
    human_judgment: false
  - id: D2
    description: "Nach dem Backfill traegt jede sales_person-Zeile (auch soft-geloescht) und jede user-Zeile ein nicht-leeres person_id; jedes sales_person_user-Paar teilt dieselbe person.id"
    requirement: "PERS-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#full_identity_coverage"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#count_identity_holds"
        status: pass
    human_judgment: false
  - id: D3
    description: "Der Backfill-Abschnitt ist wiederholbar — ein zweiter Lauf aendert weder COUNT(person) noch eine einzige person_id-Zuordnung"
    requirement: "PERS-02"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#apply_twice_is_noop"
        status: pass
    human_judgment: false
  - id: D4
    description: "Eine soft-geloeschte Sales Person mit billing_period_sales_person-Snapshot bekommt eine person_id (kein Soft-Delete-Filter im Backfill)"
    requirement: "PERS-03"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#soft_deleted_sales_person_gets_person_id"
        status: pass
    human_judgment: false
  - id: D5
    description: "Eine verletzte Zaehlprobe bricht den Migrationslauf mit einem Datenbankfehler ab (D-60-07), statt einen halb ausgefuehrten Backfill zu committen"
    requirement: "PERS-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#migration_aborts_on_count_mismatch"
        status: pass
    human_judgment: false
  - id: D6
    description: "HR-/Admin-user ohne Sales Person und freiwillige sales_person ohne user bekommen je eine eigene Person; die Gegenseite wird NICHT automatisch mit angelegt (D-60-03)"
    requirement: "PERS-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#hr_user_without_sales_person"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#volunteer_without_user"
        status: pass
    human_judgment: false
  - id: D7
    description: "Die Migration laeuft gegen eine produktionsnahe SQLite-Datei mit realistischer Datenverteilung durch und die Zaehlprobe geht dort auf"
    requirement: "PERS-01"
    verification:
      - kind: manual_procedural
        ref: "sqlx migrate run gegen localdb.sqlite3 (Dev-DB, 2 user / 0 sales_person / 0 links) -> Applied 20260916000000, Zaehlprobe 2 = 0 + 2 - 0"
        status: pass
    human_judgment: true
    rationale: "Die lokale Dev-DB traegt keine realistische Produktionsverteilung (0 sales_person, keine verknuepften Paare). Die Mechanik ist per Fixture bewiesen, der Lauf gegen eine Kopie der echten Produktions-Datei bleibt ein manuelles Phase-Gate (60-VALIDATION.md, Manual-Only)."

# Metrics
duration: 12min
completed: 2026-09-16
status: complete
---

# Phase 60 Plan 01: Person-Datenmodell + Backfill Summary

**`person`-Elterntabelle plus nullable `person_id` auf `sales_person`/`user` mit vollstaendigen UNIQUE-Indizes und einem idempotenten, sich selbst per CHECK-Assertion verifizierenden SQL-Backfill — abgesichert durch acht Integrationstests inklusive echtem Apply-twice-Nachweis.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-09-16T05:48:20Z
- **Completed:** 2026-09-16T06:00:00Z
- **Tasks:** 3 (1 Checkpoint, 2 TDD-Tasks)
- **Files modified:** 3 (2 neu, 1 erweitert)

## Accomplishments

- **Migration `20260916000000_create-person-backfill.sql`** legt die Tabelle `person` an (vier Spalten, keine `deleted`-Spalte, keine Namensspalten), ergaenzt `sales_person.person_id` und `user.person_id` als dauerhaft nullable Spalten ohne `DEFAULT` und setzt die 1:1-Garantie als zwei **vollstaendige** UNIQUE-Indizes durch (D-60-01/D-60-02 — bewusst ohne Soft-Delete-Filter, im Kopfkommentar gegen kuenftige "Aufraeum"-Passes verteidigt).
- **Idempotenter Backfill in fuenf Bloecken.** Unverknuepfte Sales Persons und Benutzer bekommen je einen eigenen `randomblob(16)`-Wert; bei einem verknuepften Paar wuerfelt nur die Sales-Person-Seite, die Benutzer-Seite **kopiert** den Wert ueber eine korrelierte Subquery zurueck. Das abschliessende `INSERT ... SELECT DISTINCT ... UNION` legt pro referenziertem Wert genau eine `person`-Zeile an. Jeder Block traegt einen `WHERE person_id IS NULL`-Guard (bzw. den Ausschluss bereits angelegter `person`-Zeilen).
- **Selbst-abbrechende Migration (D-60-07).** Eine Wegwerf-`CREATE TEMP TABLE` mit `CHECK (ok = 1)` prueft die Zaehlprobe `COUNT(person) = COUNT(sales_person) + COUNT(user) - COUNT(sales_person_user)` innerhalb der Migrations-Transaktion. Geht sie nicht auf, scheitert der `INSERT` und sqlx rollt die gesamte Datei zurueck — verifiziert durch einen Negativtest, der eine verwaiste `person`-Zeile einschmuggelt.
- **Acht Integrationstests** gegen echte In-Memory-SQLite, ohne Service/DAO (D-60-10). Der Apply-twice-Nachweis schneidet den markierten Backfill-Abschnitt aus der Migrationsdatei heraus und fuehrt genau diesen per `sqlx::raw_sql` erneut aus — `sqlx::migrate!().run()` ein zweites Mal aufzurufen wuerde den Ledger-Skip treffen und nichts beweisen.
- **Echter TDD-Zyklus mit sauberem RED.** Task 2 lieferte Schema-DDL plus alle acht Tests bei leerem Markerbereich: alle acht scheiterten inhaltlich (3 vs. 0 fehlende Zuordnungen, 0 vs. 4 Personen, `Ok(())` statt `Err`), kein einziger strukturell. Task 3 fuellte den Markerbereich — alle acht gruen.

## Task Commits

1. **Task 1: Entscheidungs-Gate D-60-01 (vollstaendiger Unique-Index ist one-way)** — kein Commit (Checkpoint). Auto-mode hat die erste Option `bestaetigt` gewaehlt; die Entscheidung war in `60-CONTEXT.md` bereits LOCKED, der Checkpoint war die von PITFALLS §3 geforderte explizite Bestaetigung unmittelbar vor der irreversiblen Ausfuehrung.
2. **Task 2 (RED): Schema-DDL + vollstaendiges Testmodul** — `1ad756a3` (test)
3. **Task 3 (GREEN): Backfill-Bloecke und Selbst-Assertion** — `609d19ce` (feat)

_Ein REFACTOR-Commit war nicht noetig — der GREEN-Stand ist bereits die Zielform (reines SQL im Markerbereich, Clippy gruen)._

## Files Created/Modified

- `migrations/sqlite/20260916000000_create-person-backfill.sql` (neu) — Schema-DDL (`person`, zwei `ADD COLUMN`, zwei vollstaendige UNIQUE-Indizes), markierter Backfill-Abschnitt 3a-3e, Selbst-Assertion Block 4. Kopfkommentar referenziert Phase 60, PERS-01..06, D-60-01, D-60-03, D-60-04, D-60-05, D-60-08, D-60-09, D-60-10.
- `service_impl/src/test/person_backfill.rs` (neu) — Harness (`setup_pool`, `backfill_section`, `run_backfill_section`, `snapshot`), Fixture-Helfer (`seed_sales_person`, `seed_user`, `seed_link`, `seed_billing_period_row`, `seed_fixtures`), Leser (`count`, `person_id_of_sales_person`, `person_id_of_user`) und acht `#[tokio::test]`-Funktionen.
- `service_impl/src/test/mod.rs` (erweitert) — `#[cfg(test)] pub mod person_backfill;`

## Decisions Made

- **D-60-01 bestaetigt (Task-1-Checkpoint, auto-mode → erste Option `bestaetigt`).** Der vollstaendige UNIQUE-Index bleibt. Preis: ein zurueckkehrender Mitarbeiter ist zunaechst zwei Personen; Zusammenfuehren ist Future-Scope.
- **Marker-Split statt vollem Datei-Replay** fuer den Apply-twice-Nachweis (RESEARCH Open Question 1, Empfehlung uebernommen). SQLite kennt fuer `ALTER TABLE ... ADD COLUMN` kein `IF NOT EXISTS`; der volle Replay wuerde strukturell scheitern statt Idempotenz zu pruefen.
- **`update_process`-Herkunftsmarker `phase-60-person-backfill`** auf allen migrierten `person`-Zeilen (T-60-05, Repudiation) — migrierte Personen bleiben dauerhaft von spaeter regulaer angelegten unterscheidbar.
- **Vorbedingungs-Assertions gegen vakuum-gruene Tests.** `apply_twice_is_noop` prueft vor dem Vergleich, dass der erste Lauf tatsaechlich Identitaeten vergeben hat; `person_id_parses_as_uuid` prueft `!rows.is_empty()`. Ohne diese Zusaetze waeren beide Tests bei leerem Backfill trivial gruen gewesen — der D-60-08-Anspruch "laeuft durch reicht nicht" gilt auch fuer den Test selbst.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Guard fuer leeren Markerbereich in `run_backfill_section`**
- **Found during:** Task 2 (RED)
- **Issue:** `sqlx::raw_sql` lehnt einen leeren Skript-String ab. Im RED-Zustand ist der Bereich zwischen den Markern definitionsgemaess leer — die acht Tests waeren an einem `sqlx`-Fehler gescheitert (strukturell) statt an ihren inhaltlichen Assertions, was die Plan-Vorgabe "scheitern inhaltlich, nicht strukturell" verletzt haette.
- **Fix:** `run_backfill_section` gibt bei `section.trim().is_empty()` frueh `Ok(())` zurueck. Der Guard ist auch nach GREEN semantisch korrekt (ein leerer Markerbereich *ist* ein No-Op) und traegt einen erklaerenden Kommentar.
- **Files modified:** `service_impl/src/test/person_backfill.rs`
- **Verification:** Task-2-Lauf zeigte 8/8 FAILED mit ausschliesslich inhaltlichen Panic-Meldungen; Task-3-Lauf 8/8 ok.
- **Committed in:** `1ad756a3` (Task-2-Commit)

**2. [Rule 2 - Missing Critical] Vorbedingungs-Assertions in `apply_twice_is_noop` und `person_id_parses_as_uuid`**
- **Found during:** Task 2 (RED)
- **Issue:** Beide Tests waeren im RED-Zustand vakuum-gruen gewesen (Vergleich zweier leerer Snapshots bzw. Iteration ueber null Zeilen). Ein Test, der ohne Implementierung besteht, beweist nichts — und D-60-08 verlangt ausdruecklich mehr als "laeuft ohne Fehler durch".
- **Fix:** `apply_twice_is_noop` prueft vor dem Vergleich `person_count > 0` und dass jede `sales_person`-/`user`-Zeile bereits zugeordnet ist; `person_id_parses_as_uuid` prueft `!rows.is_empty()`.
- **Files modified:** `service_impl/src/test/person_backfill.rs`
- **Verification:** Beide Tests waren in Task 2 RED und in Task 3 GREEN — der Nachweis haengt jetzt nachweislich am Backfill.
- **Committed in:** `1ad756a3` (Task-2-Commit)

**3. [Rule 3 - Blocking] `cargo` ist nicht auf dem Default-PATH**
- **Found during:** Task 2 (Verifikationslauf)
- **Issue:** `cargo: command not found` — die Rust-Toolchain liegt in der Nix-Devshell des Projekts.
- **Fix:** Alle Gates (`cargo test`, `cargo clippy`, `sqlx migrate run`, `sqlite3`) ueber `nix develop --command bash -c '...'` ausgefuehrt. Keine Repo-Aenderung.
- **Files modified:** keine
- **Verification:** `cargo 1.95.0`, `sqlx-cli 0.9.0`, `sqlite3 3.51.2` in der Devshell verfuegbar.
- **Committed in:** —

**4. [Rule 1 - Bug] `roadmap.update-plan-progress` hat die falsche ROADMAP-Tabelle ueberschrieben**
- **Found during:** State-Update nach Task 3
- **Issue:** `gsd-tools query roadmap.update-plan-progress 60` ersetzte die Phase-60-Zeile der **Frontend-Relevanz**-Tabelle (`| 60 | ja | **nein** | Reine Schema- + Backfill-Phase ... |`) durch `| 60 | 1/3 | In Progress| |` — der Handler hat die erste Tabelle mit einer `60`-Zeile getroffen statt der Fortschrittstabelle. Dabei ging die Begruendung "Frontend = nein" verloren.
- **Fix:** Die Originalzeile wiederhergestellt. Der korrekte Teil des Updates (Plan-Checkbox `- [ ]` → `- [x]` fuer `60-01-PLAN.md`) bleibt erhalten.
- **Files modified:** `.planning/ROADMAP.md`
- **Verification:** `git diff .planning/ROADMAP.md` zeigt jetzt ausschliesslich die Checkbox-Aenderung.
- **Committed in:** Metadaten-Commit dieses Plans

---

**Total deviations:** 4 auto-fixed (2 blocking, 1 missing critical, 1 bug)
**Impact on plan:** Alle vier betreffen die Testmechanik, die Werkzeugumgebung bzw. GSD-Metadaten, nicht den Liefergegenstand. Deviation 2 verschaerft die Testaussage gegenueber dem Plan (statt sie zu lockern). Kein Scope Creep, keine Schemaabweichung.

## Issues Encountered

- **Der `user_update_timestamp`-AFTER-UPDATE-Trigger** (aus der 2024er `user-roles`-Migration) feuert bei Block 3b/3d mit. Da SQLite rekursive Trigger per Default aus hat und der Trigger nur `update_timestamp` schreibt, ist das folgenlos — beim zweiten Lauf greift der `WHERE person_id IS NULL`-Guard und der Trigger feuert gar nicht. Durch `apply_twice_is_noop` mit abgedeckt (der Snapshot vergleicht `person_id`, nicht `update_timestamp`).
- **Keine offenen Probleme.**

## Prohibitions — Verifikationsstand

| Prohibition | Pruefung | Ergebnis |
|---|---|---|
| `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13 | `grep -c 'CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13' service_impl/src/billing_period_report.rs` | 1 (Datei nicht im Diff) |
| Index nicht partiell | `SELECT sql FROM sqlite_master WHERE name LIKE '%person_id_unique_idx'` | beide ohne `WHERE`-Klausel |
| Kein Soft-Delete-Filter im Backfill | `grep -v '^--' <migration> \| grep -c 'deleted IS NULL'` | 0 |
| `person` ohne `deleted` | `pragma_table_info('person') WHERE name='deleted'` | 0 |
| `person` nur Identitaet + Audit | `PRAGMA table_info(person)` | 4 Spalten: id, created, update_process, update_version |
| `sales_person_user` bleibt | `sqlite_master WHERE name='sales_person_user'` | 1 |
| Keine Umschluesselung | `grep -cE 'UPDATE +sales_person +SET +id\|UPDATE +user +SET +name'` | 0 |
| Keine DB-interne Rueckfall-Tabelle | `grep -c 'backup' <migration>`; `__phase60%` in `sqlite_master` | 0 / 0 |
| Kein DAO-Eingriff, kein Trigger | Diff enthaelt keine Datei unter `dao/` oder `dao_impl_sqlite/`; `grep -v '^--' \| grep -c 'CREATE TRIGGER'` | 0 |
| `PRAGMA foreign_keys` unveraendert aus | `grep -rc 'PRAGMA foreign_keys' --include='*.rs' --include='*.sql' .` | 0 Treffer |
| `randomblob(16)` genau viermal | `grep -c 'randomblob(16)' <migration>` | 4 (3a, 3b, 3c, `update_version` in 3e) |
| Assertion-Tabelle dreimal | `grep -c '__phase60_backfill_assertion' <migration>` | 3 (anlegen, befuellen, droppen) |

## Verification

- `SQLX_OFFLINE=true cargo test -p service_impl person_backfill` → **8 passed, 0 failed**
- `SQLX_OFFLINE=true cargo test --workspace` → alle Suites gruen (u. a. 853 + 66 + 33 passed, 0 failed)
- `cargo clippy --workspace -- -D warnings` → exit 0
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0
- `sqlx migrate run` gegen `localdb.sqlite3` → `Applied 20260916000000/migrate create-person-backfill`; Zaehlprobe dort: `person=2, sales_person=0, user=2, links=0, Luecken=0`
- `person.id` in der Dev-DB: `length(id) = 16` (roh, kein Hex-String) — Vorab-Absicherung fuer `Uuid::from_slice()` in Phase 61

## User Setup Required

None — keine externe Service-Konfiguration noetig. Operativer Hinweis fuer den Produktiv-Deploy: D-60-09 verlangt vor dem Deploy eine Kopie der SQLite-Datei, weil es keine Down-Migration gibt.

## Next Phase Readiness

- **Bereit fuer 60-02** (Wave 2): Die Migration existiert, das Testmodul `person_backfill.rs` ist registriert und der Harness (`setup_pool`, `seed_*`, `count`) steht fuer die acht zusaetzlichen Schema-/Ablehnungstests von 60-02 bereit. Die dort erwarteten Symbole (`sales_person_index_is_not_partial`, `duplicate_person_id_rejected_by_db_*` usw.) kommen additiv in dieselbe Datei.
- **Bereit fuer 60-03** (Docs-Freshness): Das Docs-Gate fuer `migrations/sqlite/*.sql` ist in 60-01 bewusst NICHT bedient — es ist der Liefergegenstand von 60-03. Die dort zu dokumentierenden Fakten stehen oben in "Prohibitions — Verifikationsstand" und "Files Created/Modified" abrufbar.
- **Bereit fuer Phase 61:** `person.id` ist bitkompatibel zu `Uuid::from_slice()`; ein `PersonDao` kann direkt darauf aufsetzen. Offen und bewusst so (D-60-10): Zeilen, die NACH der Migration entstehen (Login-Auto-Anlage, `create_sales_person`), bekommen noch kein `person_id` — die Vergabe beim Anlegen ist Phase-61-Scope.
- **Warnung fuer kuenftige Passes:** Die Markerzeilen `-- BACKFILL START` / `-- BACKFILL END` und die Abwesenheit des Partial-Filters auf beiden Indizes sind Vertragsbestandteile, keine Schlampigkeit. Beides ist im Kopfkommentar der Migration dokumentiert.

---
*Phase: 60-person-datenmodell-backfill*
*Completed: 2026-09-16*

## Self-Check: PASSED

Alle in dieser SUMMARY genannten Dateien existieren auf der Platte, beide Task-Commits (`1ad756a3`, `609d19ce`) sind in der Git-Historie auffindbar.
