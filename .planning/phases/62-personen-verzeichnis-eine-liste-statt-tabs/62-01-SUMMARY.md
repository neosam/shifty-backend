---
phase: 62-personen-verzeichnis-eine-liste-statt-tabs
plan: 01
subsystem: database
tags: [sqlx, sqlite, dao, outer-join, soft-delete, mockall, tdd]

# Dependency graph
requires:
  - phase: 60-person-identitaet
    provides: "person-Elterntabelle plus Backfill — jede sales_person- und jede user-Zeile traegt eine person_id"
  - phase: 61-personservice-aufl-sung-von-sales-person-user
    provides: "PersonDao mit den beiden Kandidaten-Anti-Joins; der offen gebliebene Review-Befund WR-02"
provides:
  - "dao::person_directory::PersonDirectoryDao — Trait mit list_directory_entries + all_user_roles, mockbar via #[automock]"
  - "dao::person_directory::PersonDirectoryRow — flaches Aggregat mit Option-Feldern je Traegerseite"
  - "dao::person_directory::UserRoleRow — Rollen-Zuordnung fuer die Bulk-Query"
  - "dao_impl_sqlite::person_directory::PersonDirectoryDaoImpl — Drei-Wege-Outer-Join in einem Statement"
  - "D-62-05 / WR-02 geschlossen: users_without_sales_person filtert sp.deleted in der ON-Klausel"
  - "service_impl::test::person_directory_dao — sechs DB-Integrationstests V-1..V-6"
affects: [62-02-rest-types-dto, 62-03-person-directory-service, 62-05-rest-di]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Eigenes schlankes DAO-Trait statt Erweiterung eines auf n Methoden festgezurrten Traits"
    - "Soft-Delete-Filter in der ON-Klausel des LEFT JOIN, nie in WHERE"
    - "Abwesenheits-Tests gegen einen sichtbaren Anker, damit sie nicht vakuum halten"

key-files:
  created:
    - dao/src/person_directory.rs
    - dao_impl_sqlite/src/person_directory.rs
    - service_impl/src/test/person_directory_dao.rs
  modified:
    - dao/src/person.rs
    - dao/src/lib.rs
    - dao_impl_sqlite/src/person.rs
    - dao_impl_sqlite/src/lib.rs
    - service_impl/src/test/person_candidates.rs
    - service_impl/src/test/mod.rs
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md
    - .sqlx/

key-decisions:
  - "D-62-05 / WR-02 umgesetzt: der Soft-Delete-Filter wandert in die ON-Klausel des LEFT JOIN; ein Benutzer, dessen einzige Sales Person soft-geloescht ist, ist wieder Kandidat."
  - "Das Verzeichnis bekommt ein eigenes Trait (PersonDirectoryDao) statt einer fuenften Methode auf PersonDao — jenes Trait ist auf vier Methoden festgezurrt, und das Verzeichnis ist eine andere Lesart derselben Tabellen."
  - "Sortierung findet NICHT im SQL statt; sie gehoert in den Business-Logic-Service (Plan 62-03), damit sie testbar und locale-unabhaengig bleibt."
  - "Rollen kommen als eine einzige Bulk-Query (all_user_roles); PermissionDao::roles_for_user bleibt fuer den Auth-Pfad unangetastet."
  - "K-10 (unlinked_sales_persons_liefert_kandidaten) wurde auf die erweiterte Kandidatenmenge nachgezogen — der WR-02-Fix aendert die erwartete Menge, nicht nur ihre Randfaelle."

patterns-established:
  - "ON-vs-WHERE-Disziplin: jeder Soft-Delete-Filter auf der optionalen Seite eines LEFT JOIN gehoert in die ON-Klausel; in WHERE degeneriert der Join zum INNER JOIN. Beide Vorkommen tragen einen Zeilenkommentar mit D-62-04 / WR-02."
  - "Nicht-vakuume Abwesenheits-Tests: Wer 'X erscheint nicht' behauptet, seedet zusaetzlich einen sichtbaren Anker und assertet die exakte Zeilenzahl — sonst haelt der Test auch gegen eine Implementierung, die immer leer liefert."
  - "TDD gegen eine Stub-Impl: bei DAO-Schichten, wo der RED-Lauf sonst an Compile-Fehlern statt an Assertions scheitert, zuerst eine Stub-Impl (leere Rueckgabe), RED belegen, dann das SQL einsetzen."

# PADM-01 ist bewusst NICHT abgehakt: alle sieben Plaene dieser Phase tragen es,
# und die Aussage ist nutzersichtbar ("ein Admin sieht eine Liste"). Sie ist erst
# nach 62-07 eingeloest. Siehe Deviation 4.
requirements-completed: []
requirements-advanced: [PADM-01]

coverage:
  - id: D1
    description: "users_without_sales_person liefert einen Benutzer, dessen einzige Sales Person soft-geloescht ist (D-62-05 / WR-02 geschlossen)"
    requirement: "PADM-01"
    verification:
      - kind: unit
        ref: "service_impl/src/test/person_candidates.rs#user_mit_nur_soft_geloeschter_sales_person_ist_kandidat"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/person_candidates.rs#unlinked_sales_persons_liefert_kandidaten"
        status: pass
    human_judgment: false
  - id: D2
    description: "PersonDirectoryDao-Trait mit Row-Entities, registriert und mockbar"
    requirement: "PADM-01"
    verification:
      - kind: unit
        ref: "nix develop --command bash -c 'SQLX_OFFLINE=true cargo build -p dao'"
        status: pass
    human_judgment: false
  - id: D3
    description: "list_directory_entries liefert alle drei Auspraegungen (verknuepft, reines Benutzerkonto, reiner Verkaeufer) in einem Statement"
    requirement: "PADM-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_directory_dao.rs#verzeichnis_enthaelt_alle_drei_auspraegungen"
        status: pass
    human_judgment: false
  - id: D4
    description: "Soft-geloeschte Sales Persons erscheinen nicht, nehmen aber ihren Benutzer nicht mit aus dem Verzeichnis (ON-statt-WHERE-Semantik)"
    requirement: "PADM-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_directory_dao.rs#soft_geloeschte_sales_person_erscheint_nicht"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_directory_dao.rs#soft_geloeschte_sales_person_laesst_benutzer_sichtbar"
        status: pass
    human_judgment: false
  - id: D5
    description: "Inaktive Sales Persons erscheinen mit inactive == Some(true); traegerlose person-Zeilen erscheinen nicht"
    requirement: "PADM-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_directory_dao.rs#inaktive_sales_person_erscheint"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_directory_dao.rs#person_ohne_traeger_erscheint_nicht"
        status: pass
    human_judgment: false
  - id: D6
    description: "Rollen kommen als eine einzige Bulk-Query statt eines Aufrufs pro Zeile"
    requirement: "PADM-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_directory_dao.rs#all_user_roles_liefert_alle_zuordnungen_in_einer_query"
        status: pass
    human_judgment: false

# Metrics
duration: 32min
completed: 2026-09-20
status: complete
---

# Phase 62 Plan 01: DAO-Schicht fuer das Personen-Verzeichnis Summary

**Der Drei-Wege-Outer-Join `person` ↔ `sales_person` ↔ `user` liefert alle drei Verzeichnis-Auspraegungen in einem Statement, und der seit Phase 61 offene Soft-Delete-Bug WR-02 ist mit Regressionstest geschlossen.**

## Performance

- **Duration:** 32 min
- **Started:** 2026-09-20T07:10:00Z
- **Completed:** 2026-09-20T07:42:00Z
- **Tasks:** 3/3
- **Files modified:** 12 (3 neu, 9 geaendert, dazu 3 `.sqlx`-Cache-Eintraege)

## Accomplishments

- **WR-02 geschlossen (D-62-05).** `users_without_sales_person` zog den Filter
  `sp.deleted IS NULL` in die `ON`-Klausel des `LEFT JOIN`. Vorher fiel ein
  Benutzer, dessen einzige Sales Person soft-geloescht war, aus **beiden**
  Kandidatenlisten: die Sales Person war wegen des bestehenden Loesch-Filters
  kein „Login hinzufuegen"-Kandidat, und der Benutzer hatte formal einen
  Join-Partner. Er war damit nie wieder verknuepfbar.
- **`PersonDirectoryDao` angelegt** — ein eigenes, schlankes Trait mit genau
  zwei Methoden, `#[automock]`-bar, registriert in `dao/src/lib.rs`.
  `PersonDao` bleibt unveraendert bei vier Methoden.
- **`PersonDirectoryDaoImpl` implementiert** — zwei `query_as!`-Statements,
  keine Sortierung, Soft-Delete in der `ON`-Klausel, traegerlose Personen ueber
  `WHERE sp.id IS NOT NULL OR u.name IS NOT NULL` ausgeschlossen.
- **Sieben neue Tests** (K-11 plus V-1..V-6), alle gegen echte In-Memory-SQLite
  mit `sqlx::migrate!`, kein DAO-Mock.
- **Docs-Freshness-Gate bedient:** `docs/features/F01-employee-management.md`
  und `_de.md` beschreiben jetzt die korrigierte `users_without_sales_person`-
  Semantik und die neue `PersonDirectoryDao`-Sektion.

## Task Commits

| Task | Name | Commit |
| --- | --- | --- |
| 1 | WR-02 — Soft-Delete-Filter in die ON-Klausel, mit Regressionstest | `73798102` |
| 2 | PersonDirectoryDao-Trait und Row-Entities | `497e3774` |
| 3 | SQLite-Implementierung des Verzeichnis-Joins plus DB-Integrationstests | `8b16553a` |

## TDD Gate Compliance

Task 1 und Task 3 sind als `tdd="true"` geplant. Beide RED-Laeufe sind belegt:

- **Task 1 RED:** `user_mit_nur_soft_geloeschter_sales_person_ist_kandidat`
  scheiterte an der Assertion (`… war aber nicht in ["u4-ohne-sales-person"]`),
  nicht am Compiler. Danach GREEN mit 11/11.
- **Task 3 RED:** Die sechs V-Tests liefen zuerst gegen eine Stub-Impl, die
  leere Slices zurueckgab — alle sechs fielen. Das war der eigentliche Zweck des
  Stubs: er hat bewiesen, dass auch die beiden **Abwesenheits**-Tests
  (`soft_geloeschte_sales_person_erscheint_nicht`,
  `person_ohne_traeger_erscheint_nicht`) nicht vakuum halten. Ohne den
  zusaetzlich geseedeten sichtbaren Anker haetten sie gegen eine
  Immer-leer-Implementierung gruen gemeldet. Danach GREEN mit 6/6.

Die Gate-Reihenfolge ist in den Commits nicht als getrennte `test(...)`- und
`feat(...)`-Paare sichtbar: beide Tasks sind als ein Commit pro Task angelegt
(das Plan-Commit-Protokoll verlangt genau einen Commit je Task). Die RED-Phase
ist stattdessen hier und in den Testlauf-Ausgaben dokumentiert.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blockierend] K-10 `unlinked_sales_persons_liefert_kandidaten` auf die erweiterte Kandidatenmenge nachgezogen**

- **Found during:** Task 1
- **Issue:** Der Test assertete exakte Gleichheit
  `vec![UNLINKED_USER] == user_names`. Der WR-02-Fix macht `u5` zu einem
  weiteren Kandidaten, der Test waere nach dem Fix rot geworden. Der Plan
  listete unter „Bestehende Invarianten bleiben gruen" nur K-4, K-5 und K-6 —
  K-10 war uebersehen.
- **Fix:** Sortierter Mengenvergleich gegen beide erwarteten Benutzer
  (`UNLINKED_USER` und `DELETED_SALES_PERSON_USER`), mit Begruendung im
  Assertion-Text. Die Aussage des Tests bleibt erhalten und wird sogar
  schaerfer: sie haelt jetzt fest, dass der DAO- und der Service-Lesepfad
  dieselbe — erweiterte — Menge liefern.
- **Files modified:** `service_impl/src/test/person_candidates.rs`
- **Commit:** `73798102`

**2. [Rule 2 — Projektvorgabe] Docs-Freshness-Gate bedient (CLAUDE.md)**

- **Found during:** Task 1 und Task 3
- **Issue:** `CLAUDE.md` erklaert das Docs-Gate zur harten Regel: eine Aenderung
  an `dao/**/*.rs` (Trait-Signaturen) verlangt die passende
  `docs/features/F*.md`-Sektion im selben Commit, und zwar in **beiden**
  Sprachen. Der Plan erwaehnt keinen Docs-Task. Der 61-Review hatte zusaetzlich
  festgehalten, dass `F01-employee-management.md` die falsche
  `users_without_sales_person`-Formulierung ungeprueft uebernommen hatte.
- **Fix:** In Task 1 die `users_without_sales_person`-Zeile in F01 (en+de) auf
  die `ON`-Klausel-Semantik korrigiert. In Task 3 eine neue Sektion
  `PersonDirectoryDao` unter „3. Data model" ergaenzt (en+de), inklusive der
  beiden Join-Regeln und ihrer Regressionstests.
- **Files modified:** `docs/features/F01-employee-management.md`,
  `docs/features/F01-employee-management_de.md`
- **Commits:** `73798102`, `8b16553a`

**3. [Rule 1 — Bug im Tooling] `roadmap.update-plan-progress` schrieb in die falsche ROADMAP-Tabelle**

- **Found during:** State-Update nach Task 3
- **Issue:** `gsd-tools query roadmap.update-plan-progress 62` ueberschrieb die
  Zeile für Phase 62 in der **Nyquist-Validierungs-Tabelle** (Spalten
  `Phase | ja/nein | ja/nein | Begründung`) mit Plan-Fortschrittsdaten:
  `| 62 | ja | ja | — |` wurde zu `| 62 | 1/7 | In Progress|  |`. Die
  eigentliche Plan-Checkliste weiter unten wurde korrekt aktualisiert.
- **Fix:** Die Validierungs-Tabellenzeile von Hand zurueckgesetzt. Der
  korrekte Teil der Aenderung (Checkbox `62-01-PLAN.md` auf `[x]`) bleibt.
- **Files modified:** `.planning/ROADMAP.md`
- **Note:** Bekanntes Tooling-Problem — ROADMAP-Diffs nach
  `update-plan-progress`/`phase.complete` immer pruefen.

**4. [Rule 1 — Falsche Zustandsaussage] PADM-01 nicht auf `Complete` gesetzt**

- **Found during:** State-Update nach Task 3
- **Issue:** `requirements.mark-complete PADM-01` hakte das Requirement ab und
  setzte die Traceability-Tabelle auf `Complete`. PADM-01 lautet aber „Ein
  Admin **sieht** eine Liste … mit erkennbarer Ausprägung und Suche, ohne
  Tab-Wechsel" — eine nutzersichtbare Aussage, die erst mit der gesamten Phase
  eingeloest ist. **Alle sieben** Plaene der Phase 62 fuehren PADM-01 in ihrem
  Frontmatter; nach Plan 01 (reine DAO-Schicht, kein REST, kein Frontend) waere
  die Tabelle schlicht falsch.
- **Fix:** Checkbox und Traceability-Zeile auf `Pending` zurueckgesetzt. PADM-01
  gehoert an den Abschluss der Phase (nach 62-07), nicht an diesen Plan.
- **Files modified:** `.planning/REQUIREMENTS.md` (netto unveraendert)

### Acceptance-Criterion-Abweichung (dokumentiert, nicht gefixt)

**Task 2, Kriterium `grep -c 'automock' dao/src/person_directory.rs` ergibt `1`**
— tatsaechlich ergibt es `2`. Das Kriterium ist ein Authoring-Fehler: jede Datei,
die dem als „exact analog" vorgegebenen Muster von `dao/src/person.rs` folgt,
enthaelt den String zweimal (`use mockall::automock;` **und**
`#[automock(...)]`). `dao/src/person.rs` selbst liefert ebenfalls `2`. Das
Kriterium liesse sich nur durch einen vollqualifizierten Pfad
(`#[mockall::automock(...)]`) ohne `use` erfuellen — und damit durch Abweichen
von der Konvention, die derselbe Plan verbindlich macht. Die **Absicht** des
Kriteriums („Das Trait traegt `#[automock`") ist erfuellt und nachgewiesen:
`grep -c '#\[automock' dao/src/person_directory.rs` ergibt `1`.

## Verification Results

| Gate | Kommando | Ergebnis |
| --- | --- | --- |
| K-Tests inkl. K-11 | `cargo test -p service_impl --lib test::person_candidates` | 11 passed, 0 failed |
| V-Tests | `cargo test -p service_impl --lib test::person_directory_dao` | 6 passed, 0 failed |
| Workspace | `cargo test --workspace` | alle gruen (u.a. 896 + 93 Tests), 0 failed |
| Offline-Build | `cargo build --workspace` mit `SQLX_OFFLINE=true` | ohne Fehler |
| Clippy (Hard Gate) | `cargo clippy --workspace -- -D warnings` | ohne Ausgabe |
| Offline-Cache | `git status --porcelain .sqlx` | 1 Query ersetzt, 2 neu — alle committet |

Alle Kommandos via `nix develop --command bash -c '…'`; `cargo` liegt nicht im
Default-PATH.

## Threat Mitigations

| Threat ID | Disposition | Umsetzung |
| --- | --- | --- |
| T-62-04 | mitigate | Soft-Delete-Filter in der `ON`-Klausel von `users_without_sales_person`, Zeilenkommentar mit D-62-05 / WR-02, Regressionstest K-11. |
| T-62-06 | mitigate | Derselbe Fehler im neuen Join vermieden; `soft_geloeschte_sales_person_laesst_benutzer_sichtbar` deckt die Degeneration ab. |
| T-62-07 | accept | Kein Pagination-Schutz — bewusst akzeptiert bei < 100 Personen. |
| T-62-08 | accept | `sqlx::query_as!` mit Compile-Time-Checking, keine String-Konkatenation, keine Nutzereingabe im SQL dieser Phase. |
| T-62-SC | accept | Kein einziges neues Paket installiert. `Cargo.toml` ist in keinem der drei Commits enthalten. |

## Known Stubs

Keine. Die in Task 3 zwischenzeitlich angelegte Stub-Implementierung (leere
Rueckgaben fuer den RED-Lauf) wurde vor dem Commit durch das echte SQL ersetzt
und ist in keinem Commit enthalten.

## Threat Flags

Keine neue sicherheitsrelevante Oberflaeche. Dieser Plan hat keine
REST-Oberflaeche und kein Auth-Gate — beides kommt in 62-03 (Service) und
62-05 (Endpoint). `PersonDirectoryDao` ist ein reiner Lesepfad ohne
Nutzereingabe.

## Notes for Future Phases

- **62-03 muss das Auth-Gate setzen.** Die DAO-Schicht ist bewusst ungegated;
  ohne Gate im Service waere das Verzeichnis offen. `PersonDirectoryRow`
  enthaelt Benutzernamen.
- **62-03 besitzt die Sortierung.** Das SQL liefert unsortiert (Kriterium:
  `grep -c 'ORDER BY'` ergibt `0`). Die Reihenfolge ist Service-Verantwortung.
- **62-05 muss `PersonDirectoryDaoImpl` in `shifty_bin/src/main.rs` verdrahten.**
  Der DAO ist registriert und konstruierbar (`new(pool)`), aber bisher nirgends
  instanziiert — ausserhalb der Tests hat er noch keinen Aufrufer.

## Self-Check: PASSED

Dateien:

- FOUND: `dao/src/person_directory.rs`
- FOUND: `dao_impl_sqlite/src/person_directory.rs`
- FOUND: `service_impl/src/test/person_directory_dao.rs`

Commits:

- FOUND: `73798102`
- FOUND: `497e3774`
- FOUND: `8b16553a`
