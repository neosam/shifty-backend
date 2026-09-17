---
phase: 61-personservice-aufl-sung-von-sales-person-user
plan: 01
subsystem: database
tags: [rust, sqlx, sqlite, dao, service-tier, dependency-injection, person, anti-join]

# Dependency graph
requires:
  - phase: 60-person-datenmodell-backfill
    provides: "person-Tabelle, nullable person_id auf sales_person + user, zwei vollstaendige UNIQUE-Indizes, idempotenter Backfill"
provides:
  - "dao::person::PersonDao + PersonEntity (Basic-Tier-DAO-Trait, genau vier Methoden, MockPersonDao)"
  - "dao_impl_sqlite::person::PersonDaoImpl inkl. der beiden LEFT-JOIN-Anti-Join-Kandidaten-Lesepfade (D-61-02)"
  - "service::person::Person + PersonService (Transport + Trait, MockPersonService)"
  - "service_impl::person::PersonServiceImpl (Basic-Tier, fuenf Deps, alle vier Methoden HR-gated)"
  - "DI-Konstruktion im Basic-Service-Block von shifty_bin/src/main.rs, strikt vor der Business-Logic-Schicht"
  - "rest::RestStateDef::PersonService + person_service()-Accessor (Vorarbeit Phase 62, keine Route)"
  - "service_impl::test::person_candidates — 10 Integrationstests gegen echtes In-Memory-SQLite"
affects: [61-02, 61-03, 61-04, 61-05, 62-personenverwaltung-liste, 63-personen-detail]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "LEFT-JOIN-Anti-Join fuer 'hat keinen Join-Partner' statt Leer-Pruefung auf der FK-Spalte"
    - "Dateilokale Db-Row-Struct, wenn die Row-Struct des Fremdmoduls privat ist (statt dort die Sichtbarkeit zu oeffnen)"
    - "sqlx-Spalten-Overrides (AS \"name!: Typ\") gegen LEFT-JOIN-Nullability-Inferenz"

key-files:
  created:
    - dao/src/person.rs
    - dao_impl_sqlite/src/person.rs
    - service/src/person.rs
    - service_impl/src/person.rs
    - service_impl/src/test/person_candidates.rs
  modified:
    - dao/src/lib.rs
    - dao_impl_sqlite/src/lib.rs
    - service/src/lib.rs
    - service_impl/src/lib.rs
    - service_impl/src/test/mod.rs
    - shifty_bin/src/main.rs
    - rest/src/lib.rs
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md
    - docs/architecture/02-service-tiers.md
    - docs/architecture/02-service-tiers_de.md
    - docs/architecture/diagrams/service-graph-runtime.mmd

key-decisions:
  - "PersonDao bleibt bei genau vier Methoden — keine Vorratshaltung fuer Phase 62/63 (Research Pitfall 5)"
  - "Beide Kandidaten-Lesepfade sind LEFT-JOIN-Anti-Joins, keine person_id-IS-NULL-Abfragen (D-61-02), belegt durch kandidat_trotz_gesetzter_person_id"
  - "Alle vier PersonService-Methoden HR-gated (T-61-05); Phase 62/63 darf auf admin verschaerfen, nicht lockern"
  - "PersonServiceImpl bindet genau fuenf Deps, keinen Domain-Service (SC5 / T-61-07); Konstruktion im Basic-Block Zeile 1241 vor reporting_service Zeile 1284"
  - "Dateilokale CandidateSalesPersonDb statt Sichtbarkeitsaenderung an dao_impl_sqlite/src/sales_person.rs (die Datei gehoert 61-03/61-04)"
  - "pub mod person; steht nach pub mod permission; — das ist die alphabetisch korrekte Stelle (Plan sagte davor)"

patterns-established:
  - "Anti-Join-Kandidatenlesung: LEFT JOIN Gegenseite ON person_id, WHERE Gegenseiten-Spalte IS NULL"
  - "Basic-Tier-Service-Skelett: check_permission zuerst, dann use_transaction, DAO-Aufruf, commit"
  - "RestStateDef-Accessor ohne Route als dead_code-Vermeidung fuer noch nicht exponierte Services"

requirements-completed: [PERS-05]

coverage:
  - id: D1
    description: "PersonDao + PersonDaoImpl mit genau vier Methoden; person-Zeilen les- und schreibbar ohne Rowid-Readback"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_candidates.rs#create_vergibt_id_version_und_created"
        status: pass
    human_judgment: false
  - id: D2
    description: "Beide Kandidaten-Lesepfade sind Anti-Joins und liefern korrekt (unverknuepft ja, verknuepft nein, soft-geloescht nein)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_candidates.rs#sales_person_ohne_user_ist_kandidat"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_candidates.rs#verknuepfte_sales_person_ist_kein_kandidat"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_candidates.rs#soft_geloeschte_sales_person_ist_kein_kandidat"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_candidates.rs#user_ohne_sales_person_ist_kandidat"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_candidates.rs#verknuepfter_user_ist_kein_kandidat"
        status: pass
    human_judgment: false
  - id: D3
    description: "D-61-02-Nachweis: Kandidaten trotz durchgaengig gesetztem person_id — eine IS-NULL-Implementierung waere hier rot"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_candidates.rs#kandidat_trotz_gesetzter_person_id"
        status: pass
    human_judgment: false
  - id: D4
    description: "PersonService ist Basic-Tier mit fuenf Deps und HR-Gate auf allen vier Methoden (T-61-05, T-61-07)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_candidates.rs#create_ohne_hr_privileg_ist_forbidden"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_candidates.rs#unlinked_sales_persons_gated"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_candidates.rs#unlinked_sales_persons_liefert_kandidaten"
        status: pass
      - kind: other
        ref: "grep -cE 'SalesPersonService|UserService|ReportingService|BookingService' service_impl/src/person.rs == 0"
        status: pass
    human_judgment: false
  - id: D5
    description: "DI-Verdrahtung im Basic-Block vor der Business-Logic-Schicht + RestStateDef-Accessor; Workspace baut, clippy gruen, Server bootet"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "cargo build --workspace && cargo clippy --workspace -- -D warnings && cargo test --workspace (1039 Tests)"
        status: pass
      - kind: other
        ref: "awk '/PersonServiceImpl::</{print NR}' shifty_bin/src/main.rs == 1241 < reporting_service == 1284"
        status: pass
      - kind: manual_procedural
        ref: "timeout 45 cargo run — Server bootet auf 127.0.0.1:3000, Migrationen laufen, Cron feuert"
        status: pass
    human_judgment: false

# Metrics
duration: 25min
completed: 2026-09-17
status: complete
---

# Phase 61 Plan 01: PersonService + PersonDao (Basic-Tier) Summary

**`PersonDao`/`PersonService` als Basic-Tier-Quartett mit zwei LEFT-JOIN-Anti-Join-Kandidaten-Lesepfaden, HR-Gate auf allen vier Methoden und DI-Konstruktion strikt vor der Business-Logic-Schicht.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-09-17T07:31:00Z
- **Completed:** 2026-09-17T07:56:00Z
- **Tasks:** 3
- **Files modified:** 17 (5 neu, 12 geaendert) + 4 neue `.sqlx`-Cache-Eintraege

## Accomplishments

- `dao::person::PersonDao` + `PersonEntity` (nur `id`/`created`/`version`, D-60-04) mit genau vier Methoden und `MockPersonDao`.
- `PersonDaoImpl` implementiert beide „unverknuepfte Kandidaten"-Lesepfade als LEFT-JOIN-Anti-Joins (D-61-02) — ohne Rowid-Readback (Pitfall 3) und ohne jede Erwaehnung der abgeloesten Verknuepfungstabelle.
- `PersonService`/`PersonServiceImpl` als Basic-Tier-Service mit exakt fuenf Dependencies und `HR_PRIVILEGE` als erster Anweisung jeder Methode (T-61-05, T-61-07).
- DI-Konstruktion im Basic-Service-Block von `shifty_bin/src/main.rs` (Zeile 1241) strikt vor `reporting_service` (Zeile 1284) — Success Criterion 5 erfuellt.
- `RestStateDef::person_service()` als Vorarbeit fuer Phase 62 (Entscheidung P-61-A), ohne Route/Handler/TO.
- 10 Integrationstests gegen echtes In-Memory-SQLite, darunter der entscheidende D-61-02-Nachweis `kandidat_trotz_gesetzter_person_id`.

## Task Commits

Jede Task wurde atomar committet:

1. **Task 1: PersonDao-Trait, PersonEntity und PersonDaoImpl inkl. Kandidaten-Lesepfade** — `17ba8f7c` (feat)
2. **Task 2: PersonService-Trait und PersonServiceImpl (Basic-Tier)** — `225d62c1` (feat)
3. **Task 3: DI-Verdrahtung in main.rs und RestStateDef-Accessor** — `d51ce9ce` (feat)

**Plan metadata:** siehe Abschluss-Commit `docs(61-01)`.

## Files Created/Modified

**Neu:**

- `dao/src/person.rs` — `PersonEntity` + `PersonDao`-Trait (4 Methoden, `#[automock]`).
- `dao_impl_sqlite/src/person.rs` — `PersonDaoImpl`, `PersonDb`, dateilokale `CandidateSalesPersonDb`, beide Anti-Join-Queries.
- `service/src/person.rs` — `Person`-Transportstruct (beide `From`-Richtungen) + `PersonService`-Trait.
- `service_impl/src/person.rs` — `PersonServiceImpl` via `gen_service_impl!`, `PERSON_SERVICE_PROCESS`, HR-Gates.
- `service_impl/src/test/person_candidates.rs` — 10 Integrationstests (6 DAO-Ebene, 4 Service-Ebene).

**Geaendert:**

- `dao/src/lib.rs`, `dao_impl_sqlite/src/lib.rs`, `service/src/lib.rs`, `service_impl/src/lib.rs` — `pub mod person;`.
- `service_impl/src/test/mod.rs` — Testmodul registriert.
- `shifty_bin/src/main.rs` — `PersonDao`-Typalias, `PersonServiceDependencies` (7 assoziierte Typen), `person_dao`, `person_service`, `RestStateImpl`-Feld + Accessor.
- `rest/src/lib.rs` — `RestStateDef::PersonService` + `person_service()`.
- `docs/features/F01-employee-management{,_de}.md` — `PersonDao`-Sektion im Datenmodell + §4.5 `PersonService` mit Auth-Gate-Tabelle.
- `docs/architecture/02-service-tiers{,_de}.md`, `docs/architecture/diagrams/service-graph-runtime.mmd` — `PersonService` als Basic-Tier.
- `.sqlx/` — vier neue Offline-Cache-Eintraege (`cargo sqlx prepare --workspace`).

## Decisions Made

- **Dateilokale `CandidateSalesPersonDb`.** `SalesPersonDb` in `dao_impl_sqlite/src/sales_person.rs` ist privat. Der Plan erlaubt ausdruecklich eine dateilokale Kopie statt einer Sichtbarkeitsaenderung — die Datei gehoert den Plaenen 61-03/61-04, SC3 verlangt einen leeren Diff auf den Sales-Person-Signaturen.
- **sqlx-Spalten-Overrides.** Die Anti-Join-Queries nutzen `AS "spalte!: Typ"`, weil sqlx Spalten der linken Seite eines `LEFT JOIN` sonst als nullable inferieren kann. Ohne die Overrides waere das Row-Mapping `Option<...>`-lastig geworden.
- **`pub mod person;` nach `pub mod permission;`.** Der Plan schrieb „zwischen `pdf_export_config` und `permission`"; alphabetisch gehoert `person` aber hinter `permission` (`m` < `s`). Umgesetzt ist die alphabetisch korrekte Stelle.
- **TDD-Zyklus ohne separaten RED-Commit.** Siehe Deviation 1.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - CLAUDE.md-Direktive] Kein separater RED-Commit im TDD-Zyklus**

- **Found during:** Task 1 (und analog Task 2)
- **Issue:** Der GSD-TDD-Ablauf sieht einen eigenen `test(...)`-Commit fuer die rote Phase vor. In diesem Rust-Workspace referenziert ein Test auf ein noch nicht existierendes Modul — der RED-Zustand ist ein **Compile-Fehler**. Ein solcher Commit wuerde die harten Projekt-Gates aus `CLAUDE.md` (`cargo build`, `cargo test`, `cargo clippy --workspace -- -D warnings` gruen bei jedem Commit) verletzen.
- **Fix:** Der RED/GREEN-Zyklus wurde im Working Tree vollstaendig gefahren und die rote Phase **belegt** (`cargo test --package service_impl --lib person_candidates` meldete `error[E0432]: unresolved import 'dao::person'` und `'dao_impl_sqlite::person'`), danach als ein gruener `feat(...)`-Commit pro Task abgelegt.
- **Files modified:** keine zusaetzlichen — betrifft nur die Commit-Granularitaet.
- **Verification:** RED-Lauf dokumentiert oben; GREEN-Lauf 6 → 10 Tests gruen.
- **Committed in:** `17ba8f7c`, `225d62c1`

**2. [Rule 3 - Blocking] `.sqlx`-Offline-Cache fuer die vier neuen Queries erzeugt**

- **Found during:** Task 1
- **Issue:** Das Repo baut ohne `.env` im sqlx-Offline-Modus. Vier neue `query!`/`query_as!`-Aufrufe haben keinen Cache-Eintrag → der Build (und CI/`nix build`) waere rot. Genau dieser Fehler hat schon in Phase 34 CI gebrochen.
- **Fix:** Wegwerf-SQLite im Scratchpad angelegt, `sqlx migrate run`, dann `cargo sqlx prepare --workspace -- --all-targets`. Ergebnis: **4 neue** Dateien, **0** geaenderte oder geloeschte — bestehende Eintraege sind unberuehrt.
- **Files modified:** `.sqlx/query-{0da5510e,268be235,8a9ae7a5,ea9a4c19}….json`
- **Verification:** `cargo build --workspace` und `cargo test --workspace` gruen ohne gesetztes `DATABASE_URL`.
- **Committed in:** `17ba8f7c`

**3. [Rule 2 - CLAUDE.md-Direktive] Docs-Freshness-Gate bedient**

- **Found during:** Task 1–3
- **Issue:** `CLAUDE.md` erklaert das Docs-Freshness-Gate zur harten Regel: Aenderungen an `dao/**/*.rs`, `service/**/*.rs` und `shifty_bin/src/main.rs` verlangen eine Doku-Aktualisierung **im selben Commit**, in **beiden** Sprachen. Der Plan erwaehnt keine Doku-Arbeit.
- **Fix:** F01 um eine `PersonDao`-Sektion (Datenmodell) und §4.5 `PersonService` (Trait-Auszug, Auth-Gate-Tabelle, Tier-Begruendung, Verdrahtung) erweitert; `02-service-tiers` um den Basic-Tier-Eintrag; `service-graph-runtime.mmd` um den `Person`-Knoten. Jede Aenderung in `foo.md` **und** `foo_de.md`.
- **Files modified:** `docs/features/F01-employee-management{,_de}.md`, `docs/architecture/02-service-tiers{,_de}.md`, `docs/architecture/diagrams/service-graph-runtime.mmd`
- **Verification:** Beide Sprachfassungen tragen dieselben Abschnitte; Doku-Aenderung liegt jeweils im selben Commit wie der ausloesende Code-Diff.
- **Committed in:** `17ba8f7c`, `225d62c1`, `d51ce9ce`

**4. [Rule 1 - Bug] Verbotene Literale aus Kommentaren entfernt**

- **Found during:** Task 1 und Task 2 (Acceptance-Grep)
- **Issue:** Erklaerende Kommentare enthielten woertlich `last_insert_rowid`, `person_id IS NULL` und `SalesPersonService`. Die Acceptance-Kriterien greppen die Dateien auf **0 Treffer** — die Kommentare haetten die Gates gebrochen, obwohl der Code korrekt war.
- **Fix:** Die betroffenen Kommentare sachlich umformuliert (gleiche Aussage, ohne die gegrepten Literale).
- **Files modified:** `dao_impl_sqlite/src/person.rs`, `service_impl/src/person.rs`
- **Verification:** Alle drei Greps liefern jetzt `0`.
- **Committed in:** `17ba8f7c`, `225d62c1`

---

**Total deviations:** 4 auto-fixed (2 CLAUDE.md-Direktiven, 1 blocking, 1 bug)
**Impact on plan:** Kein Scope Creep. Alle vier Abweichungen sind entweder harte Projekt-Gates (`.sqlx`, Docs) oder Praezisionskorrekturen an Kommentaren; die Plan-Substanz ist unveraendert umgesetzt.

## Issues Encountered

- **`cargo` nicht auf dem PATH.** Alle Gates liefen ueber `nix develop --command bash -c '…'` (liefert `cargo`, `clippy`, `sqlx-cli`, `sqlite3`) — wie in `CLAUDE.md`/STATE beschrieben.
- **Acceptance-Kriterium fuer die SC5-Reihenfolge nicht woertlich pruefbar.** Das Kriterium vergleicht mit `awk '/ReportingServiceImpl::</{print NR}'`; die Konstruktion von `reporting_service` benutzt aber keine Turbofish-Syntax, der Ausdruck liefert daher nichts. Geprueft wurde die **Absicht**: `PersonServiceImpl::<` steht in Zeile 1241, `let reporting_service = Arc::new(service_impl::reporting::ReportingServiceImpl {` in Zeile 1284, `voluntary_stats_service` in 1310, `rebooking_reconciliation_service` in 1344 — der Basic-Block liegt vollstaendig vor der Business-Logic-Schicht.
- **`cargo run`-Smoke.** Mit einer Wegwerf-DB und 45 s Timeout gefahren: Migrationen laufen, Server bindet `127.0.0.1:3000`, der Cron feuert. Keine Regression durch die neue DI-Kante.

## User Setup Required

None — keine externe Service-Konfiguration noetig. Die Phase installiert kein Package (Threat T-61-SC, `accept`).

## Next Phase Readiness

- `PersonService` steht als Basic-Tier-Baustein bereit. Die Plaene 61-02 (`create_user`-Pfad), 61-03/61-04 (Selbst-Zugriffs-Pruefung, Merge/Split) und 61-05 (Migration + Ablösung) koennen darauf aufsetzen, ohne dass `SalesPersonService` einen Domain-Service als Dependency bekommt.
- Beide Kandidaten-Lesepfade sind fertige Backend-Vorarbeit fuer den Phase-63-Picker; der `RestStateDef`-Accessor ist die Vorarbeit fuer Phase 62.
- **Offen (unveraendert, kein Blocker dieses Plans):** `dao_impl_sqlite/src/sales_person.rs` und `service/src/sales_person.rs` sind bewusst unberuehrt (SC3). `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**.

## Self-Check: PASSED

Alle behaupteten Dateien existieren, alle drei Commit-Hashes sind im Repo auffindbar.

---
*Phase: 61-personservice-aufl-sung-von-sales-person-user*
*Completed: 2026-09-17*
