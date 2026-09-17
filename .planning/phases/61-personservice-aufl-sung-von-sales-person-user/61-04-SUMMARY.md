---
phase: 61-personservice-aufl-sung-von-sales-person-user
plan: 04
subsystem: database
tags: [rust, sqlx, sqlite, dao, merge, split, conflict-409, person, transaction]

# Dependency graph
requires:
  - phase: 60-person-datenmodell-backfill
    provides: "person-Tabelle, nullable person_id auf sales_person + user, zwei vollstaendige UNIQUE-Indizes"
  - phase: 61-01
    provides: "PersonDao/PersonService (Basic-Tier) — Lesepfade auf der person-Identitaet"
  - phase: 61-02
    provides: "person_id-Vergabe bei Neuanlage (SalesPersonDaoImpl::create, PermissionDaoImpl::create_user)"
  - phase: 61-03
    provides: "alle vier lesenden Verknuepfungs-Pfade auf JOIN u.person_id = sp.person_id"
provides:
  - "SalesPersonDaoImpl::assign_to_user als Merge zweier Personen mit Loeschung der absorbierten Zeile (D-61-03)"
  - "Konflikt-Ablehnung ohne Schreibvorgang bei echter Mehrdeutigkeit (D-61-04)"
  - "SalesPersonDaoImpl::discard_assigned_user als Split mit frisch angelegter Person fuer den Benutzer (D-61-05)"
  - "Re-Link-Semantik P-61-B (Split(A) + Merge(B)) inkl. Rollback im Konfliktfall"
  - "ServiceError::EntityAlreadyExists aus set_user -> HTTP 409, unterscheidbar von EntityConflicts (SC4)"
  - "service_impl::test::person_merge — 11 Integrationstests gegen echtes In-Memory-SQLite"
affects: [61-05, 61-06, 62-personenverwaltung-liste, 63-personen-detail]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Merge zweier Identitaeten: Read-both, Konflikt-SELECTs vor jedem Write, conditional UPDATE, Aufraeum-DELETE mit NOT-EXISTS-Guards"
    - "Split einer Identitaet: erst neue Zeile anlegen, dann umhaengen — nie umgekehrt (kein transienter identitaetsloser Zustand)"
    - "person_id IS ?-Guard (null-sicher) plus rows_affected()==1 als Race-Detektor innerhalb einer Transaktion"
    - "is_unique_violation als zweite Verteidigungslinie: Race-Verlierer wird 409 statt 500"
    - "Fail-closed bei fehlender Identitaet statt Fremdschluessel auf NULL zu setzen"

key-files:
  created:
    - service_impl/src/test/person_merge.rs
  modified:
    - dao_impl_sqlite/src/sales_person.rs
    - service_impl/src/sales_person.rs
    - service_impl/src/test/mod.rs
    - rest/src/sales_person.rs
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md
    - docs/features/F12-auth-session.md
    - docs/features/F12-auth-session_de.md
    - .sqlx/ (9 Cache-Eintraege neu, 2 entfallen)

key-decisions:
  - "Die Fehleruebersetzung in set_user wurde von Task 3 nach Task 1 vorgezogen — ohne sie war Task 1s eigenes Akzeptanzkriterium M-6 unerreichbar und der Commit nicht gruen"
  - "assign_to_user ist fail-closed, wenn die Sales Person keine Identitaet traegt: statt user.person_id auf NULL zu setzen (T-61-14) wird EntityAlreadyExists geliefert"
  - "process gewinnt als update_process, PERSON_LINK_PROCESS greift nur bei leerem process (Plan gab die Wahl frei)"
  - "Kein Handler-lokales Status-Mapping in rest/: der zentrale error_handler mappt EntityAlreadyExists bereits auf 409"
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 13 — keine value_type-Aenderung"

patterns-established:
  - "Konflikt-Erkennung als Anwendungslogik VOR dem Write, UNIQUE-Index als Rueckversicherung dahinter"
  - "Rollback-Nachweis als Test: mehrstufige Service-Operation scheitert am zweiten Schritt, Ausgangszustand muss unveraendert sein"

requirements-completed: [PERS-05]

coverage:
  - id: M1
    description: "Verknuepfen zweier unverknuepfter Objekte ergibt genau EINE Person; die absorbierte person-Zeile ist geloescht (D-61-03)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_merge.rs#merge_zweier_freier_objekte_ergibt_eine_person"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_merge.rs#merge_ist_idempotent_wenn_bereits_verknuepft"
        status: pass
    human_judgment: false
  - id: M2
    description: "Echte Mehrdeutigkeit wird ohne Schreibvorgang abgelehnt, komplementaeres Verschmelzen geht durch (D-61-04, T-61-11/T-61-12)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_merge.rs#konflikt_ziel_benutzer_haengt_an_anderer_sales_person"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_merge.rs#konflikt_sales_person_haengt_an_anderem_benutzer"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_merge.rs#komplementaeres_verschmelzen_ist_erlaubt"
        status: pass
    human_judgment: false
  - id: M3
    description: "Der Konflikt erreicht den Aufrufer als ServiceError::EntityAlreadyExists und ist von EntityConflicts unterscheidbar (SC4, T-61-15)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_merge.rs#konflikt_wird_als_409_sichtbar"
        status: pass
      - kind: other
        ref: "rest/src/lib.rs:175-179 mappt EntityAlreadyExists zentral auf HTTP 409"
        status: pass
    human_judgment: false
  - id: M4
    description: "Loesen laesst die Sales Person identitaetsstabil, gibt dem Benutzer eine frische Person und hinterlaesst nie eine traegerlose person-Zeile (D-61-05, T-61-14)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_merge.rs#split_keeps_sales_person_identity"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_merge.rs#split_ohne_verknuepfung_ist_noop"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_merge.rs#split_erzeugt_keine_traegerlose_person"
        status: pass
    human_judgment: false
  - id: M5
    description: "Re-Link A->B laeuft als Split(A) + Merge(B); scheitert der Merge am 409, ist der Split mit zurueckgerollt (P-61-B)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_merge.rs#relink_von_benutzer_a_auf_benutzer_b"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_merge.rs#relink_konflikt_rollt_den_split_zurueck"
        status: pass
    human_judgment: false
  - id: M6
    description: "SC3 haelt: keine Signaturaenderung an dao/service, keine der ~50 Aufrufstellen beruehrt; POST /{id}/user dokumentiert 409"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "git diff 45bcb26d..HEAD -- dao/src/sales_person.rs service/src/sales_person.rs ist leer"
        status: pass
      - kind: other
        ref: "git diff --stat 45bcb26d..HEAD -- service_impl/ nennt nur sales_person.rs, test/mod.rs, test/person_merge.rs"
        status: pass
      - kind: other
        ref: "rest/src/sales_person.rs:284 traegt (status = 409, ...) im utoipa::path von set_sales_person_user"
        status: pass
    human_judgment: false
  - id: M7
    description: "Projekt-Gates: Build, Tests, Clippy, Server-Boot"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "cargo test --workspace (904 + 66 + 100 Tests) gruen"
        status: pass
      - kind: other
        ref: "cargo clippy --workspace --all-targets -- -D warnings Exit-Code 0"
        status: pass
      - kind: manual_procedural
        ref: "timeout 60 cargo run — Server bindet 127.0.0.1:3000, Migrationen laufen, Cron feuert"
        status: pass
    human_judgment: false

# Metrics
duration: 40min
completed: 2026-09-17
status: complete
---

# Phase 61 Plan 04: Verknuepfen als Merge, Loesen als Split, Konflikt als 409 Summary

**Verknuepfen verschmilzt zwei Personen zu einer und loescht die absorbierte Zeile, echte Mehrdeutigkeit wird ohne jeden Schreibvorgang als 409 abgelehnt, und Loesen gibt dem Benutzer eine frische Person, waehrend die Sales Person identitaetsstabil bleibt.**

## Performance

- **Duration:** ~40 min
- **Started:** 2026-09-17T08:15:00Z
- **Completed:** 2026-09-17T08:56:00Z
- **Tasks:** 3
- **Files modified:** 9 Quell-/Doku-Dateien (1 neu, 8 geaendert) + 9 neue und 2 entfallene `.sqlx`-Cache-Eintraege

## Accomplishments

- `SalesPersonDaoImpl::assign_to_user` ist ein **Merge zweier Personen**: der Benutzer wandert auf die `person_id` der Sales Person, seine bisherige `person`-Zeile wird geloescht (D-61-03). Die Sales Person behaelt ihre Identitaet — sie ist die Seite mit historischem Gewicht (Buchungen, `extra_hours`, write-once-Snapshots in `billing_period_sales_person`).
- **D-61-04 ist implementiert und getestet:** zwei Konflikt-`SELECT`s laufen **vor** jedem Schreibvorgang. Nur wenn eine *dritte* Zeile betroffen waere, gibt es `DaoError::EntityAlreadyExists` — und dann ohne jede Schreibspur. Der haeufige Bestandsfall „beide Seiten tragen je eine eigene Person" (Ergebnis des Phase-60-Backfills) geht ausdruecklich durch.
- `SalesPersonDaoImpl::discard_assigned_user` ist ein **Split**: erst wird die neue `person`-Zeile angelegt, dann der Benutzer umgehaengt (D-61-05). Die Reihenfolge ist der Schutz gegen T-61-14 — andersherum waere der Benutzer transient ohne gueltige `person_id` und damit ohne Selbstzugriff.
- Die Re-Link-Semantik **P-61-B** ist als Test festgeschrieben: `set_user(S, Some(B))` laeuft als Split(A) + Merge(B), Benutzer A behaelt eine eigene frische Person. Scheitert der Merge am 409, wird die Service-Transaktion nie committet und der Split rollt mit zurueck — das ist als eigener Test nachgewiesen.
- Der Konflikt erreicht den Aufrufer als **HTTP 409** und ist von `ServiceError::EntityConflicts` unterscheidbar (SC4). Das Status-Mapping bleibt zentral im `error_handler`; `POST /{id}/user` dokumentiert den Code jetzt in seiner `responses(...)`-Liste.
- 11 Integrationstests gegen echtes In-Memory-SQLite, darunter die beiden Kernnachweise `komplementaeres_verschmelzen_ist_erlaubt` (Abgrenzung zur verworfenen Alternative) und `relink_konflikt_rollt_den_split_zurueck` (Rollback-Nachweis).
- **SC3 haelt vollstaendig:** `dao/src/sales_person.rs` und `service/src/sales_person.rs` stehen unveraendert im Diff; in `service_impl/` sind ausser `sales_person.rs` und den Testmodulen keine Dateien beruehrt — die rund 50 Aufrufstellen sind unangetastet.

## Task Commits

Jede Task wurde atomar committet:

1. **Task 1: assign_to_user als Merge zweier Personen mit Konflikt-Ablehnung** — `80da5e78` (feat)
2. **Task 2: discard_assigned_user als Split mit frischer Person fuer den Benutzer** — `cb797edb` (feat)
3. **Task 3: 409 in der OpenAPI-Beschreibung von POST /{id}/user** — `03a41df4` (docs)

## Files Created/Modified

**Neu:**

- `service_impl/src/test/person_merge.rs` — 11 Integrationstests (9 DAO-Ebene, 2 Service-Ebene) inkl. Harness mit echtem `SalesPersonDaoImpl` und `MockPermissionService`.

**Geaendert:**

- `dao_impl_sqlite/src/sales_person.rs` — `assign_to_user` (vierstufiger Merge), `discard_assigned_user` (Split), die beiden dateilokalen Konstanten `PERSON_LINK_PROCESS`/`PERSON_SPLIT_PROCESS` und die lokale `is_unique_violation`-Hilfsfunktion.
- `service_impl/src/sales_person.rs` — **ausschliesslich** der Rumpf von `set_user`: `match` statt `?` am `assign_to_user`-Aufruf.
- `service_impl/src/test/mod.rs` — Testmodul registriert.
- `rest/src/sales_person.rs` — eine Zeile im `#[utoipa::path]`-Attribut von `set_sales_person_user`.
- `docs/features/F01-employee-management{,_de}.md` — Merge-/Split-/Konflikt-/Re-Link-Semantik in den Geschaeftsregeln, REST-Tabelle auf `404/409/500`.
- `docs/features/F12-auth-session{,_de}.md` — die Aussage „die Verknuepfungstabelle wird weiterhin geschrieben" war ab Task 1 falsch und ist korrigiert.
- `.sqlx/` — Offline-Cache regeneriert (`cargo sqlx prepare --workspace -- --all-targets`).

## Decisions Made

- **`process` gewinnt als `update_process`.** Der Plan liess die Wahl frei zwischen dem uebergebenen `process` und der dateilokalen Konstante. Entschieden wurde fuer `process`, weil er den fachlichen Ausloeser benennt (`sales-person-service`); `PERSON_LINK_PROCESS` greift nur, wenn `process` leer ist. Die Wahl ist im Methodenkommentar dokumentiert, wie vom Plan verlangt.
- **`discard_assigned_user` bekommt keine Wahl.** Die Trait-Signatur hat keinen `process`-Parameter (SC3), deshalb ist `PERSON_SPLIT_PROCESS` dort der einzige Marker — sowohl fuer die neue `person`-Zeile als auch fuer die umgehaengte `user`-Zeile.
- **Kein Handler-lokales Status-Mapping.** `rest/src/lib.rs:175-179` mappt `ServiceError::EntityAlreadyExists` bereits zentral auf 409. Die REST-Aenderung ist deshalb reine OpenAPI-Vollstaendigkeit, eine einzige Zeile.
- **`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13.** Diese Phase aendert keinen persistierten `value_type` und keine Snapshot-Berechnung; ein Bump waere falsch.
- **Kein separater RED-Commit.** Wie in 61-01: ein RED-Commit wuerde die harten Projekt-Gates aus `CLAUDE.md` verletzen. Der Zyklus lief vollstaendig im Working Tree (Belege unten).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fehleruebersetzung in `set_user` von Task 3 nach Task 1 vorgezogen**

- **Found during:** Task 1, GREEN-Lauf
- **Issue:** Der Plan legt den Test `konflikt_wird_als_409_sichtbar` **und** das Akzeptanzkriterium „ueber `set_user` wird daraus `ServiceError::EntityAlreadyExists(_)`" in Task 1, die dafuer noetige Uebersetzung aber in Task 3. Task 1 war damit nicht gruen committebar: der Test scheiterte reproduzierbar mit `DatabaseQueryError(EntityAlreadyExists)`, weil `ServiceError` den `DaoError` per `#[from]` pauschal nach 500 konvertiert. Ein roter Commit verletzt die harten Gates aus `CLAUDE.md`.
- **Fix:** Die in Task 3 beschriebene `match`-Uebersetzung wurde **wortgetreu** (gleicher Ort, gleiches Muster aus `rebooking_batch.rs`, gleicher Umfang) in Task 1 umgesetzt. Task 3 liefert damit nur noch die OpenAPI-Ergaenzung und die Doku.
- **Files modified:** `service_impl/src/sales_person.rs` (nur der Rumpf von `set_user`, wie vom Planer sanktioniert)
- **Verification:** `konflikt_wird_als_409_sichtbar` gruen; `git diff --stat service_impl/` nennt keine weitere Datei ausser `sales_person.rs` und den Testmodulen.
- **Committed in:** `80da5e78`

**2. [Rule 2 - Sicherheit/Korrektheit] `assign_to_user` ist fail-closed, wenn die Sales Person keine Identitaet traegt**

- **Found during:** Task 1
- **Issue:** Der Plan schreibt fuer den Fall „eine der beiden Zeilen fehlt", man solle den `UPDATE` mit `rows_affected() == 0` auflaufen lassen. Mit der woertlichen Umsetzung passiert das aber **nicht**: der `UPDATE` setzt `user.person_id = NULL` und trifft genau eine Zeile — der Benutzer verliert seine Identitaet und damit (D-61-08) seinen Selbstzugriff, und eine traegerlose `person`-Zeile bleibt zurueck. Das ist genau die Klasse von T-61-14.
- **Fix:** Fehlt die `sales_person`-Zeile oder traegt sie kein `person_id`, liefert die Methode `DaoError::EntityAlreadyExists` **ohne** Schreibvorgang — derselbe Ausgang, den der Plan mit `rows_affected() == 0` beabsichtigt, nur erreichbar. Der Fall ist nach dem Phase-60-Backfill und 61-02 ohnehin nicht mehr erzeugbar.
- **Files modified:** `dao_impl_sqlite/src/sales_person.rs`
- **Verification:** `split_erzeugt_keine_traegerlose_person` und `count_traegerlose_personen(...) == 0` in allen Split-Tests.
- **Committed in:** `80da5e78`

**3. [Rule 3 - Blocking] `.sqlx`-Offline-Cache regeneriert**

- **Found during:** Task 1 und Task 2
- **Issue:** Das Repo baut ohne `.env` im sqlx-Offline-Modus. Neue und geaenderte `query!`-Aufrufe ohne Cache-Eintrag machen Build und CI (`nix build`) rot.
- **Fix:** Wegwerf-SQLite im Scratchpad angelegt, `sqlx migrate run`, dann `cargo sqlx prepare --workspace -- --all-targets` nach jeder Query-Aenderung.
- **Files modified:** `.sqlx/` — insgesamt 9 neue Eintraege, 2 entfallene (die beiden Statements auf der abgeloesten Verknuepfungstabelle).
- **Verification:** `cargo build`/`cargo test --workspace` gruen ohne gesetztes `DATABASE_URL`.
- **Committed in:** `80da5e78`, `cb797edb`

**4. [Rule 2 - CLAUDE.md-Direktive] Docs-Freshness-Gate bedient**

- **Found during:** Task 1–3
- **Issue:** Die Trigger-Tabelle in `CLAUDE.md` nennt `dao_impl_sqlite/**` nicht namentlich, aber `docs/features/F12-auth-session{,_de}.md` behauptete woertlich, die Verknuepfungstabelle werde von `assign_to_user`/`discard_assigned_user` „weiterhin geschrieben". Ab Task 1 war das falsch. `F01` beschrieb die 1:1-Garantie noch ueber die `UNIQUE`-Constraints der alten Tabelle.
- **Fix:** Chirurgische Korrektur der jeweils falsch gewordenen Saetze plus neue Bullets fuer Merge (D-61-03), Konflikt-Semantik (D-61-04), Split (D-61-05) und Re-Link (P-61-B). Jede Aenderung in `foo.md` **und** `foo_de.md`, jeweils im selben Commit wie der ausloesende Code-Diff. Der grosse Doku-Umbau (Tabellen- und ER-Diagramm-Eintraege der abgeloesten Tabelle) gehoert zu 61-05 und wurde bewusst **nicht** vorgezogen.
- **Files modified:** `docs/features/F01-employee-management{,_de}.md`, `docs/features/F12-auth-session{,_de}.md`
- **Verification:** Beide Sprachfassungen tragen dieselben Abschnitte; `grep` auf die korrigierten Saetze liefert in beiden Dateien denselben Stand.
- **Committed in:** `80da5e78`, `cb797edb`, `03a41df4`

**5. [Rule 2 - CLAUDE.md-Direktive] Kein separater RED-Commit im TDD-Zyklus**

- **Found during:** Task 1 und Task 2
- **Issue:** Der GSD-TDD-Ablauf sieht einen eigenen `test(...)`-Commit fuer die rote Phase vor. Ein solcher Commit haette `cargo test` rot hinterlassen und damit die harten Gates aus `CLAUDE.md` verletzt.
- **Fix:** Der RED/GREEN-Zyklus lief vollstaendig im Working Tree und ist belegt: Task 1 meldete im RED-Lauf **6 von 6** Tests rot (u. a. `UNIQUE constraint failed` beim zweiten Merge und eine abweichende `person_id` beim ersten), Task 2 meldete `split_keeps_sales_person_identity` und `relink_von_benutzer_a_auf_benutzer_b` rot. Danach je ein gruener `feat(...)`-Commit.
- **Files modified:** keine zusaetzlichen — betrifft nur die Commit-Granularitaet.
- **Verification:** RED-Laeufe oben dokumentiert; GREEN 6/6 bzw. 11/11.
- **Committed in:** `80da5e78`, `cb797edb`

---

**Total deviations:** 5 auto-fixed (2 blocking, 2 CLAUDE.md-Direktiven, 1 Korrektheit/Sicherheit)
**Impact on plan:** Kein Scope Creep. Abweichung 1 ist eine reine Task-Umsortierung innerhalb des vom Planer ausdruecklich sanktionierten Rahmens (Rumpf von `set_user`); Abweichung 2 macht die vom Plan beabsichtigte Wirkung ueberhaupt erst erreichbar. Die Plan-Substanz ist unveraendert umgesetzt.

## TDD Gate Compliance

Der Plan traegt `type: execute` mit `tdd="true"` auf Task 1 und Task 2. Die RED-Phase ist fuer beide Tasks nachgewiesen (siehe Deviation 5), die GREEN-Phase durch die gruenen `feat(...)`-Commits. Ein REFACTOR-Schritt war nicht noetig — beide Methoden wurden in ihrer Endform geschrieben.

## Issues Encountered

- **`cargo` nicht auf dem PATH.** Alle Gates liefen ueber `nix develop --command bash -c '…'`.
- **Kein `rustfmt` in der Dev-Shell.** `cargo fmt --all -- --check` ist ein CI-Schritt (`.github/workflows/rust.yml:21`), die Nix-Shell liefert aber weder `cargo-fmt` noch `rustfmt`. Ersatzweise wurde auf die Formatregeln hin geschrieben und gegen die 100-Zeichen-Grenze geprueft; die einzige laengere Zeile ist eine SQL-String-Fortsetzung (rustfmt formatiert String-Literale nicht) mit direktem Vorbild in `person_self_gate.rs`. Die 106-Zeichen-Zeile in `rest/src/sales_person.rs` liegt in einem `#[utoipa::path]`-Attribut, das rustfmt nicht anfasst — dieselbe Datei traegt bereits vier solcher Zeilen.
- **Zwei Split-Tests waren schon vor der Implementierung gruen.** `split_erzeugt_keine_traegerlose_person` und `relink_konflikt_rollt_den_split_zurueck` bestehen auch gegen den alten No-Op-`discard_assigned_user` — sie pruefen eine Abwesenheits- bzw. eine Rollback-Eigenschaft. Die diskriminierenden RED-Tests fuer Task 2 waren `split_keeps_sales_person_identity` und `relink_von_benutzer_a_auf_benutzer_b`; beide waren rot und wurden gruen.
- **`cargo run`-Smoke.** Mit Wegwerf-DB und 60 s Timeout gefahren: Migrationen laufen, Server bindet `127.0.0.1:3000`, der Cron feuert und aktualisiert Carryover. Keine Regression durch die beiden umgeschriebenen DAO-Methoden.

## Known Stubs

Keine. Alle in diesem Plan angefassten Pfade sind vollstaendig verdrahtet und getestet.

## Threat Flags

Keine neue Sicherheitsflaeche ausserhalb des `<threat_model>` des Plans. Es kommt kein Endpunkt, kein Auth-Pfad, kein Dateizugriff und keine Schema-Aenderung hinzu; die einzige REST-Aenderung ist eine OpenAPI-Beschreibungszeile.

## User Setup Required

None — keine externe Service-Konfiguration noetig. Die Phase installiert kein Package (Threat T-61-SC, `accept`).

## Next Phase Readiness

- Schreib- **und** Lesepfade laufen jetzt vollstaendig ueber `person_id`. In `dao_impl_sqlite/src/sales_person.rs` gibt es **null** Vorkommen der abgeloesten Verknuepfungstabelle — Plan 61-05 kann die Migration `20260917000000_drop-sales-person-user.sql` ziehen, ohne dass ein Schreibpfad ins Leere laeuft.
- **Offen fuer 61-05:** `dao_impl_sqlite/src/lib.rs:321` (`BasicDao::clear_all`) und der LEGACY-VERGLEICH-Block in `service_impl/src/test/person_self_gate.rs` referenzieren die Tabelle weiterhin — beide sind ausdruecklich 61-05 zugeordnet. Ebenso die verbleibenden Doku-Stellen in `docs/architecture/03-data-model{,_de}.md`, `docs/arc42/05-building-block-view.md` und den Tabellen-/ER-Abschnitten von `F01`.
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**.

## Self-Check: PASSED

- `service_impl/src/test/person_merge.rs` existiert (641 Zeilen).
- Alle drei Commit-Hashes sind im Repo auffindbar: `80da5e78`, `cb797edb`, `03a41df4`.
- `git diff 45bcb26d..HEAD -- dao/src/sales_person.rs service/src/sales_person.rs` ist leer (SC3).
- `grep -c 'sales_person_user' dao_impl_sqlite/src/sales_person.rs` liefert `0`.
- `grep -c 'last_insert_rowid' dao_impl_sqlite/src/sales_person.rs` liefert `0`.
- `grep -c 'CREATE UNIQUE INDEX' dao_impl_sqlite/src/sales_person.rs` liefert `0`.
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` steht weiterhin auf `13`.

---
*Phase: 61-personservice-aufl-sung-von-sales-person-user*
*Completed: 2026-09-17*
