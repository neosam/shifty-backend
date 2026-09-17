---
phase: 61-personservice-aufl-sung-von-sales-person-user
plan: 02
subsystem: database
tags: [rust, sqlx, sqlite, dao, transaction, atomicity, person, permission-dao]

# Dependency graph
requires:
  - phase: 60-person-datenmodell-backfill
    provides: "person-Tabelle, nullable person_id auf sales_person + user, zwei vollstaendige UNIQUE-Indizes"
  - phase: 61-01
    provides: "PersonDao/PersonEntity, PersonService (Basic-Tier) — Lesepfade, auf denen dieser Plan aufsetzt"
provides:
  - "SalesPersonDaoImpl::create vergibt person_id auf der uebergebenen Transaktion (D-61-01)"
  - "PermissionDaoImpl::create_user vergibt person_id atomar via methoden-interner Transaktion (D-61-01, Pitfall 4)"
  - "service_impl::test::person_creation — 8 Integrationstests gegen echtes In-Memory-SQLite"
affects: [61-03, 61-04, 61-05, 62-personenverwaltung-liste, 63-personen-detail]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Identitaets-Vergabe piggybacked auf der vom Aufrufer uebergebenen Transaktion — Atomaritaet ohne Signaturaenderung"
    - "Methoden-interne Transaktion (self.pool.begin()) fuer DAO-Methoden ohne Transaktionsparameter, wenn zwei Statements atomar sein muessen"
    - "16-Byte-Schluessel in Rust erzeugen statt Rowid-Readback, wenn der PK ein blob(16) und kein Rowid-Alias ist"
    - "Atomaritaets-Nachweis per absichtlichem PK-Konflikt am zweiten Statement + Zaehlprobe auf der ersten Tabelle"

key-files:
  created:
    - service_impl/src/test/person_creation.rs
  modified:
    - dao_impl_sqlite/src/sales_person.rs
    - dao_impl_sqlite/src/lib.rs
    - service_impl/src/test/mod.rs
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md
    - docs/features/F12-auth-session.md
    - docs/features/F12-auth-session_de.md

key-decisions:
  - "person.created entsteht per strftime in SQL (Phase-60-Backfill-Format) statt per ClockService — die DAO-Ebene bleibt dependency-frei"
  - "person.update_version bleibt randomblob(16) in SQL; nur die ID wird in Rust erzeugt, weil nur sie in beide Statements muss"
  - "Der Atomaritaets-Nachweis nutzt den user.name-Primaerschluessel als Fehlerquelle — kein Fault-Injection-Mock noetig"
  - "Die drei Guard-Tests (N-3, N-4, N-7) sind schon vor der Implementierung gruen; sie sichern D-60-03 und die Transaktions-Kopplung gegen spaetere Regressionen"

patterns-established:
  - "Neuanlage-Vergabe: ID in Rust erzeugen, person-Insert VOR dem Traeger-Insert, beide auf derselben Transaktion"
  - "DAO-Methoden ohne tx-Parameter, die zwei Tabellen anfassen, oeffnen eine lokale Transaktion statt zwei Pool-Statements"

requirements-completed: [PERS-05]

coverage:
  - id: D1
    description: "Jede ueber SalesPersonDaoImpl::create angelegte Sales Person traegt sofort ein nicht-NULL person_id mit passender person-Zeile (D-61-01)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_creation.rs#create_sales_person_vergibt_person_id"
        status: pass
    human_judgment: false
  - id: D2
    description: "Zwei create-Aufrufe erzeugen zwei verschiedene Personen — sales_person_person_id_unique_idx haelt (T-61-09)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_creation.rs#create_sales_person_vergibt_je_zeile_eine_eigene_person"
        status: pass
    human_judgment: false
  - id: D3
    description: "Die Person-Anlage haengt an der uebergebenen Transaktion — ein Rollback laesst weder sales_person- noch person-Zeile zurueck"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_creation.rs#create_sales_person_rollback_laesst_keine_person_zurueck"
        status: pass
    human_judgment: false
  - id: D4
    description: "Jeder ueber PermissionDaoImpl::create_user angelegte Benutzer traegt sofort ein nicht-NULL person_id mit passender person-Zeile (D-61-01)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_creation.rs#create_user_vergibt_person_id"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_creation.rs#create_user_vergibt_je_benutzer_eine_eigene_person"
        status: pass
      - kind: manual_procedural
        ref: "Frisch gebootetes Backend (mock_auth): DEVUSER + admin tragen zwei verschiedene nicht-NULL person_id, zwei passende person-Zeilen mit update_process='dev-first-start'"
        status: pass
    human_judgment: false
  - id: D5
    description: "Die Person-Anlage in create_user ist atomar mit der user-Anlage — ein Abbruch hinterlaesst keinen user ohne person_id und keine traegerlose person (Research Pitfall 4, T-61-06)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_creation.rs#create_user_ist_atomar"
        status: pass
    human_judgment: false
  - id: D6
    description: "Die Gegenseite wird nirgends automatisch mit angelegt (D-60-03, T-61-10)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_creation.rs#create_sales_person_legt_keinen_user_an"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_creation.rs#create_user_legt_keine_sales_person_an"
        status: pass
      - kind: manual_procedural
        ref: "Frisch gebootetes Backend: SELECT COUNT(*) FROM sales_person == 0 trotz zwei auto-angelegter Benutzer"
        status: pass
    human_judgment: false
  - id: D7
    description: "Keine Trait-Signatur und kein Aufrufer wurde geaendert (SC3)"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "git diff --stat dao/src/sales_person.rs dao/src/permission.rs service/src/sales_person.rs service_impl/src/permission.rs service_impl/src/user_invitation.rs — leer"
        status: pass
    human_judgment: false
  - id: D8
    description: "Projekt-Gates gruen; CURRENT_SNAPSHOT_SCHEMA_VERSION unveraendert 13"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "cargo build --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace (881 Tests in service_impl)"
        status: pass
      - kind: other
        ref: "grep -n 'CURRENT_SNAPSHOT_SCHEMA_VERSION' service_impl/src/billing_period_report.rs == 13"
        status: pass
    human_judgment: false

# Metrics
duration: 20min
completed: 2026-09-17
status: complete
---

# Phase 61 Plan 02: `person_id`-Vergabe bei Neuanlage Summary

**Beide Neuanlage-Pfade — `SalesPersonDaoImpl::create` und die Login-Auto-Anlage `PermissionDaoImpl::create_user` — vergeben ab sofort atomar eine eigene Person-Identitaet, ohne dass eine Trait-Signatur oder ein Aufrufer angefasst wurde.**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-09-17T07:58:00Z
- **Completed:** 2026-09-17T08:18:00Z
- **Tasks:** 2
- **Files modified:** 8 (1 neu, 7 geaendert) + 3 neue / 2 entfernte `.sqlx`-Cache-Eintraege

## Accomplishments

- `SalesPersonDaoImpl::create` legt die `person`-Zeile **vor** dem `sales_person`-Insert an und schreibt deren ID in `sales_person.person_id`. Beide Statements laufen auf der vom Aufrufer uebergebenen `tx` — Atomaritaet kostet hier keine zusaetzliche Transaktion.
- `PermissionDaoImpl::create_user` oeffnet eine **methoden-interne** Transaktion (`self.pool.begin()`), schreibt `person` und `user` darin und committet. Die Trait-Signatur bleibt exakt `create_user(&self, user: &UserEntity, process: &str) -> Result<(), DaoError>`.
- In beiden Pfaden entsteht die 16-Byte-ID in Rust (`Uuid::new_v4()`) — kein Rowid-Readback (Research Pitfall 3), weil `person.id` ein `blob(16) PRIMARY KEY` und kein Rowid-Alias ist.
- `person.created` entsteht per `strftime('%Y-%m-%dT%H:%M:%S.000000000Z','now')` — identisches Format wie im Phase-60-Backfill, damit `Iso8601::DATE_TIME` es parst. Die DAO-Ebene braucht damit keinen `ClockService`.
- 8 Integrationstests gegen echtes In-Memory-SQLite (`service_impl::test::person_creation`), darunter der Pitfall-4-Atomaritaets-Nachweis `create_user_ist_atomar`.
- Live belegt: ein frisch gebootetes Backend legt `DEVUSER` und `admin` mit zwei **verschiedenen** nicht-NULL `person_id` und zwei passenden `person`-Zeilen an — bei `COUNT(*) FROM sales_person == 0`.

## Task Commits

Jede Task wurde atomar committet:

1. **Task 1: `SalesPersonDaoImpl::create` legt die person-Zeile auf derselben Transaktion an** — `64a7ee86` (feat)
2. **Task 2: `PermissionDaoImpl::create_user` legt person und user in einer internen Transaktion an** — `d85150ac` (feat)

**Plan metadata:** siehe Abschluss-Commit `docs(61-02)`.

## Files Created/Modified

**Neu:**

- `service_impl/src/test/person_creation.rs` — 8 Integrationstests (N-1..N-8), eigener Harness (`setup_pool`, `tx_of`, `commit`, `rollback`), Muster aus `person_candidates.rs`.

**Geaendert:**

- `dao_impl_sqlite/src/sales_person.rs` — ausschliesslich `create`; die sechs Verknuepfungs-Methoden bleiben unberuehrt (gehoeren 61-03/61-04).
- `dao_impl_sqlite/src/lib.rs` — ausschliesslich `PermissionDaoImpl::create_user`; `BasicDaoImpl::clear_all` bleibt unberuehrt (gehoert 61-05).
- `service_impl/src/test/mod.rs` — Testmodul registriert.
- `docs/features/F01-employee-management{,_de}.md` — §7: „Vergabe bei Neuanlage"; die veraltete Aussage „Zeilen nach der Phase-60-Migration bekommen noch gar kein `person_id`" ist ersetzt.
- `docs/features/F12-auth-session{,_de}.md` — §3: „Login-Auto-Anlage vergibt `person_id` mit", inkl. Begruendung fuer die interne Transaktion; dieselbe veraltete Aussage ersetzt.
- `.sqlx/` — 3 neue, 2 entfernte Offline-Cache-Eintraege (`cargo sqlx prepare --workspace -- --all-targets`).

## Decisions Made

- **`created` in SQL, ID in Rust.** Der Plan liess beides offen. Die ID **muss** in Rust entstehen (sie geht in zwei Statements), `created` nicht — und `strftime` haelt die DAO-Ebene frei von einer `ClockService`-Dependency. Das gewaehlte Literal ist exakt das aus dem Phase-60-Backfill, dessen Parsbarkeit `person_backfill::person_created_is_iso8601_parsable` festzurrt.
- **`update_version` bleibt `randomblob(16)`.** Nur die ID muss ueber beide Statements hinweg identisch sein; fuer die Version gibt es keinen Grund, sie aus Rust zu reichen.
- **Der Atomaritaets-Nachweis braucht keinen Fault-Injection-Mock.** Der Primaerschluessel auf `user.name` ist eine natuerliche, deterministische Fehlerquelle am *zweiten* Statement — genau die Stelle, an der eine nicht-atomare Implementierung eine traegerlose `person`-Zeile hinterliesse.
- **Drei Tests sind Guards, keine RED-Tests.** `create_sales_person_legt_keinen_user_an` (N-3), `create_sales_person_rollback_laesst_keine_person_zurueck` (N-4) und `create_user_legt_keine_sales_person_an` (N-7) waren schon vor der Implementierung gruen. Das ist beabsichtigt: sie sichern D-60-03 und die Transaktions-Kopplung gegen spaetere Regressionen ab — insbesondere gegen die Versuchung, in 61-04 die Gegenseite „hilfsbereit" mit anzulegen.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - CLAUDE.md-Direktive] Kein separater RED-Commit im TDD-Zyklus**

- **Found during:** Task 1 und Task 2
- **Issue:** Der GSD-TDD-Ablauf sieht einen eigenen `test(...)`-Commit fuer die rote Phase vor. Ein solcher Commit haette hier die harten Projekt-Gates aus `CLAUDE.md` (`cargo build`/`cargo test`/`cargo clippy -D warnings` gruen bei jedem Commit) verletzt — dieselbe Lage wie in Plan 61-01.
- **Fix:** RED/GREEN im Working Tree gefahren und die rote Phase **belegt**, danach je Task ein gruener `feat(...)`-Commit.
  - Task 1 RED: `create_sales_person_vergibt_person_id` und `create_sales_person_vergibt_je_zeile_eine_eigene_person` FAILED („D-61-01: neu angelegte Sales Person muss ein person_id tragen"), 2 passed / 2 failed.
  - Task 2 RED: `create_user_vergibt_person_id`, `create_user_vergibt_je_benutzer_eine_eigene_person` und `create_user_ist_atomar` FAILED, 5 passed / 3 failed.
- **Files modified:** keine zusaetzlichen — betrifft nur die Commit-Granularitaet.
- **Committed in:** `64a7ee86`, `d85150ac`

**2. [Rule 3 - Blocking] `.sqlx`-Offline-Cache fuer die geaenderten Queries erzeugt**

- **Found during:** Task 1 (erneut Task 2)
- **Issue:** Das Repo baut ohne `.env` im sqlx-Offline-Modus. Die geaenderten `query!`-Aufrufe hatten keinen Cache-Eintrag → `error: set DATABASE_URL to use query macros online`. Ohne Regeneration waeren Build und `nix build`/CI rot — exakt die in `61-01-SUMMARY.md` dokumentierte Falle.
- **Fix:** Wegwerf-SQLite im Scratchpad, `sqlx migrate run`, `cargo sqlx prepare --workspace -- --all-targets`. Netto: 3 neue, 2 entfernte Eintraege (die beiden alten INSERT-Varianten ohne `person_id`). Der `INSERT INTO person`-Eintrag wird von beiden Pfaden geteilt — identischer Query-String, identischer Hash.
- **Files modified:** `.sqlx/query-{aba7282b,f9080fc1,c8a4714c}….json` (neu), `.sqlx/query-{15ecb6bb,3b28d77f}….json` (entfernt)
- **Verification:** `cargo build --workspace` und `cargo test --workspace` gruen ohne gesetztes `DATABASE_URL`.
- **Committed in:** `64a7ee86`, `d85150ac`

**3. [Rule 2 - CLAUDE.md-Direktive] Docs-Freshness-Gate bedient — und aktiv veraltete Aussagen korrigiert**

- **Found during:** Task 1 und Task 2
- **Issue:** Der Plan erwaehnt keine Doku-Arbeit. `docs/features/F01-employee-management{,_de}.md` und `docs/features/F12-auth-session{,_de}.md` behaupteten beide woertlich, Zeilen nach der Phase-60-Migration traegen „noch gar kein `person_id`" und die Vergabe sei Phase-61-Scope. Genau das hat dieser Plan umgesetzt — die Saetze waren ab Task 1 bzw. Task 2 sachlich falsch, nicht nur unvollstaendig.
- **Fix:** F01 §7 um „Vergabe bei Neuanlage" (Transaktions-Kopplung, Rust-ID, `strftime`-Format, unveraenderte Signatur) erweitert; F12 §3 um „Login-Auto-Anlage vergibt `person_id` mit" inkl. der Begruendung, warum die interne Transaktion zwingend ist. Jede Aenderung in `foo.md` **und** `foo_de.md`.
- **Files modified:** `docs/features/F01-employee-management{,_de}.md`, `docs/features/F12-auth-session{,_de}.md`
- **Verification:** Beide Sprachfassungen tragen dieselben Abschnitte; Doku-Aenderung liegt jeweils im selben Commit wie der ausloesende Code-Diff.
- **Committed in:** `64a7ee86`, `d85150ac`

---

**Total deviations:** 3 auto-fixed (2 CLAUDE.md-Direktiven, 1 blocking)
**Impact on plan:** Kein Scope Creep. Alle drei Abweichungen sind harte Projekt-Gates; die Plan-Substanz ist unveraendert umgesetzt. Kein Rule-4-Fall.

## Issues Encountered

- **`cargo` nicht auf dem PATH.** Alle Gates liefen ueber `nix develop --command bash -c '…'` (liefert `cargo`, `clippy`, `sqlx-cli`, `sqlite3`) — wie in `61-01-SUMMARY.md` beschrieben.
- **`cargo run`-Smoke gegen die bestehende Dev-DB ist blind fuer diesen Plan.** `shifty_bin/src/main.rs:1690` verbindet hart gegen `sqlite:./localdb.sqlite3` (`DATABASE_URL` wirkt nur zur Compile-Zeit fuer die sqlx-Makros). Da die Dev-DB `admin`/`DEVUSER` bereits enthielt, lief der `create_user`-Pfad gar nicht. Fuer den echten Nachweis wurde `localdb.sqlite3` (+ `-shm`/`-wal`) temporaer beiseitegelegt, das Backend gegen eine leere Datei gebootet, das Ergebnis geprueft und die Dev-DB anschliessend unveraendert zurueckgestellt (`jj status` danach sauber).
- **Der Connect-String hat kein `mode=rwc`.** Ein fehlendes `localdb.sqlite3` fuehrt zu `SqliteError code 14 "unable to open database file"` statt zu einer Neuanlage. Fuer den Smoke musste die Datei per `touch` vorab existieren. Das ist Bestandsverhalten, kein Regress dieses Plans — aber eine Stolperstelle fuer jeden, der den Smoke wiederholt.

## User Setup Required

None — keine externe Service-Konfiguration noetig. Die Phase installiert kein Package (Threat T-61-SC, `accept`).

## Next Phase Readiness

- **Plan 61-03** (Selbst-Zugriffs-Pruefung auf `person_id`-Joins umschreiben) kann starten: jede Zeile — Bestand wie Neuanlage — traegt jetzt eine Person. Der Zwischenzustand, vor dem der Plan-Objective warnte („Integrationstests legen Objekte an, die anschliessend niemand mehr findet"), ist geschlossen.
- **Plan 61-04** (Merge/Split) setzt auf denselben Invarianten auf; N-3/N-7 stehen als Regressionsschutz gegen das automatische Anlegen der Gegenseite.
- **Offen (unveraendert, kein Blocker):** `sales_person_user` besteht weiter und ist bis 61-05 die laufende Wahrheit — `SalesPersonDaoImpl::find_by_user` joint nach wie vor darueber. `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**.

## Self-Check: PASSED

- `service_impl/src/test/person_creation.rs` existiert.
- Commit `64a7ee86` und `d85150ac` sind im Repo auffindbar (`jj log`).
- `cargo test --workspace` gruen, `cargo clippy --workspace --all-targets -- -D warnings` gruen.

---
*Phase: 61-personservice-aufl-sung-von-sales-person-user*
*Completed: 2026-09-17*
