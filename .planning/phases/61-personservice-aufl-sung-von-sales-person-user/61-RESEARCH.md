# Phase 61: PersonService + Auflösung von `sales_person_user` - Research

**Researched:** 2026-09-17
**Domain:** Rust-Backend-Refactor (DAO-Layer-Umbau + neuer Basic-Tier-Service), SQLite-Migration, Autorisierungspfad-Umschreibung
**Confidence:** HIGH (alle Kernaussagen durch Lesen des tatsächlichen Codes in dieser Session verifiziert; SQLite-Constraint-Semantik per Websearch gegengeprüft; keine externen Libraries im Spiel)

## Summary

Phase 61 ist ein reiner Zwei-Schichten-Umbau (Migration + DAO), der durch den
gefrorenen `SalesPersonService`-Vertrag (SC3) strukturell klein bleibt: die rund
50 Aufrufstellen in `service_impl/` rufen ausschließlich die sechs stabilen
Trait-Methoden, nicht die Tabelle. Die eigentliche Arbeit verteilt sich auf vier
Bausteine: (1) ein neuer Basic-Tier-`PersonDao`/`PersonService` ohne
Domain-Service-Abhängigkeit, (2) die Umschreibung von sechs DAO-Methoden in
`dao_impl_sqlite/src/sales_person.rs` von `sales_person_user`-Joins auf
`person_id`-Joins, wovon vier reine SQL-Umformungen sind und zwei
(`assign_to_user`/`discard_assigned_user`) echte neue Merge-Geschäftslogik
brauchen, (3) die Vergabe von `person_id` bei jeder Neuanlage (D-61-01) an zwei
Stellen mit unterschiedlicher Transaktions-Verfügbarkeit, und (4) eine
DROP-Migration mit Nachzieh-Backfill und Selbst-Assertion nach dem Muster von
D-60-07.

Der einzige sicherheitsrelevante Teil ist die Umschreibung von
`find_sales_person_by_user_id`/`find_by_user` — beide heute identische Joins
über `sales_person_user`. Die sichere Ersetzung ist ein einziger SQL-Join über
`person_id`, dessen Sicherheit direkt aus SQLs Drei-Werte-Logik folgt (`NULL =
NULL` matcht nie), während ein äquivalenter Rust-Vergleich zweier separat
geladener `Option<Uuid>` fail-open ginge (`None == None` ist in Rust `true`).
Die zweite fachlich neue Komponente ist der Merge-Algorithmus für
`set_user(Some(user))`: kein CRUD, sondern das Verschmelzen zweier
`person`-Zeilen mit einer expliziten Konflikt-Bedingung (D-61-04), die exakt
den heute schon funktionierenden Fall erhält und nur den heutigen 500er durch
einen sauberen 409 ersetzt. Beide UNIQUE-Indizes aus Phase 60
(`sales_person_person_id_unique_idx`, `user_person_id_unique_idx`) werden dabei
zu keinem Zeitpunkt transient verletzt, weil der Merge niemals zwei Zeilen
derselben Tabelle gleichzeitig denselben `person_id`-Wert zuweist — SQLite prüft
UNIQUE-Indizes ohnehin sofort nach jeder Anweisung, nicht erst beim Commit
(anders als `FOREIGN KEY`, das per `PRAGMA defer_foreign_keys` aufschiebbar
wäre).

**Primary recommendation:** Migration + PersonDao/PersonService zuerst (kein
Domain-Service-Dep, direkt SQL-generierte `person_id`s analog zum
Phase-60-Backfill-Stil), dann die vier reinen Lese-Umschreibungen, dann die
Merge-/Unlink-Logik mit einer Vorher/Nachher-Permission-Matrix, die **während**
`sales_person_user` noch lebt erstellt und erst in der letzten (DROP-)Plan-Welle
von ihrer alten Hälfte befreit wird, und zuletzt die DROP-Migration.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Person-Identitäts-Verwaltung (`PersonDao`/`PersonService`) | API/Backend (Basic-Service) | Database/Storage | Reines CRUD über `person`, analog `SalesPersonService`; keine Business-Logic-Konsumenten in dieser Phase |
| Self-Access-Autorisierungsprüfung (`find_sales_person_by_user_id`, `find_by_user`) | API/Backend (DAO-Layer) | Database/Storage | Sicherheitsentscheidung liegt im SQL-Join, nicht in Rust-Vergleichslogik — siehe Pitfall 4 |
| Verknüpfen/Lösen (Merge-/Split-Logik) | API/Backend (DAO oder Service, Claude's Discretion) | Database/Storage | Neue Geschäftslogik ohne 1:1-Präzedenz; Konflikt-Erkennung braucht Lese-vor-Schreib innerhalb einer Transaktion |
| Schema + Backfill + DROP | Database/Storage | — | Migrationsdatei, kein Applikationscode |
| „Unverknüpfte Kandidaten"-Lesepfade | API/Backend (Basic-Service) | Database/Storage | Reine Outer-Join-Lesepfade, kein REST-Endpoint in dieser Phase (Vorarbeit für Phase 63) |

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PERS-05 | Ein Mitarbeiter sieht und bucht nach der Umstellung genau dieselben eigenen Daten wie vorher — die Selbst-Zugriffs-Prüfung arbeitet über die neue Person-Verknüpfung statt über `sales_person_user`. | §„Frage 2" (SQL-Rewrite von `find_sales_person_by_user_id`/`find_by_user`) + §„Frage 3" (Vorher/Nachher-Permission-Matrix-Mechanik) beantworten exakt, WIE die Umschreibung sicher ist und WIE sie beweisbar identisch bleibt. |

</phase_requirements>

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Person-Vergabe & Verknüpfungs-Semantik**

- **D-61-01:** Jede Sales Person und jeder Benutzer trägt ab dieser Phase immer
  genau eine Person. `person_id` wird bei jeder Neuanlage mitvergeben — in der
  Login-Auto-Anlage (`dao_impl_sqlite/src/lib.rs:96`) und in
  `create_sales_person` (`dao_impl_sqlite/src/sales_person.rs:120-138`). Damit
  löst Phase 61 den in D-60-10 bewusst offen gelassenen Zustand auf: nach
  dieser Phase ist `person_id` faktisch nie mehr `NULL`. Die Spalten bleiben
  schema-seitig nullable — die Invariante wird vom Code getragen, nicht vom
  Schema. Reversibility: reversible-with-effort.

- **D-61-02:** „Unverknüpft" heißt nicht `person_id IS NULL`, sondern: die
  eigene Person trägt die Gegenseite nicht. Die Lesepfade für „noch
  unverknüpfte Kandidaten" werden als Outer-Join `sales_person` ↔ `person` ↔
  `user` formuliert, **nicht** als `WHERE person_id IS NULL`.

- **D-61-03:** Verknüpfen ist ein Merge zweier Personen, kein Anlegen.
  `set_user(sales_person_id, Some(user))` zieht beide Seiten auf **eine**
  Person; die absorbierte `person`-Zeile wird gelöscht. Sicher, weil `person`
  außer Identität und Audit-Spalten nichts trägt (D-60-04) und nach dem Merge
  von nichts mehr referenziert wird.

- **D-61-04:** 409 nur bei echter Mehrdeutigkeit. Der Konflikt-Fehler greift
  genau dann, wenn die verschmolzene Person danach mehr als einen Benutzer oder
  mehr als eine Sales Person trüge — praktisch: der Ziel-Benutzer hängt bereits
  an einer **anderen** Sales Person, oder die Sales Person hängt bereits an
  einem **anderen** Benutzer. Komplementäres Verschmelzen (eine Person trägt
  nur die Sales Person, die andere nur den Benutzer) ist **erlaubt**. Der
  heutige `set_user`-Pfad löscht die alte `sales_person_user`-Zeile und legt die
  neue an; bei freiem Benutzer geht das durch, bei belegtem Benutzer schlägt
  der `UNIQUE(user_id)`-Constraint zu und der Aufrufer sieht einen 500er.
  D-61-04 erhält den funktionierenden Fall exakt und ersetzt den 500er durch
  einen sauberen 409. Die Alternative (409 immer, sobald beide Seiten schon
  eine Person haben) wurde verworfen — sie wäre ein Funktionsverlust und
  machte PADM-04 (Phase 63) für alle Bestandsdaten unmöglich. Reversibility:
  reversible.

- **D-61-05:** Beim Lösen behält die Sales Person die bestehende Person, der
  Benutzer bekommt eine frisch angelegte. Die Sales Person ist die Seite mit
  historischem Gewicht (Buchungen, `extra_hours`,
  write-once-Abrechnungs-Snapshots hängen alle an `sales_person.id`) — ihre
  Person-Identität bleibt deshalb stabil. Folge: Pitfall 5 Fall 1
  („unlink to zero") kann strukturell nicht eintreten.

**Auflösung von `sales_person_user`**

- **D-61-06:** Die DROP-Migration enthält einen Nachzieh-Backfill und eine
  Selbst-Assertion, bevor sie die Tabelle löscht: jede noch existierende
  `sales_person_user`-Zeile muss nach dem Nachzug ein `person_id`-Gegenstück auf
  beiden Seiten haben, sonst bricht die Migration ab (Form analog D-60-07).
  Phase 60 ist noch nicht produktiv deployt — 60 und 61 gehen zusammen live,
  der Zwischenzustand tritt in Produktion nie auf. Der Guard bleibt trotzdem
  drin, bekommt aber **keine eigene Fixture-Testmatrix** — ein Test, der den
  Zwischenzustand herstellt und den Nachzug nachweist, genügt.

- **D-61-07:** `sales_person_user` wird in dieser Phase per Migration gedroppt
  (Success Criterion 2). Rollback-Pfad ist das Datei-Backup der SQLite-Datei
  vor dem Deploy, nicht eine Down-Migration — dieselbe Zusage wie in D-60-09.

**Autorisierungspfad**

- **D-61-08:** Der Self-Gate-Vergleich läuft als ein einziger SQL-Join über
  `person_id`, nie als zwei separat in Rust geladene `Option<Uuid>`. Betrifft
  `find_sales_person_by_user_id` und `find_by_user`
  (`dao_impl_sqlite/src/sales_person.rs:102-119` und `:226-243`, beide heute
  `JOIN sales_person_user`). Grund: ein Rust-seitiger `Option<Uuid>`-Vergleich
  matcht `None == None` und öffnet damit fail-open. SQL matcht `NULL = NULL`
  nie — der Join ist die sichere Form.

- **D-61-09:** Die Vorher/Nachher-Permission-Matrix ist ein Pflicht-Artefakt
  mit den drei Fällen aus Success Criterion 1 — „self darf", „fremder
  Sales-Person-Datensatz darf nicht", „Benutzer ohne Verknüpfung darf nicht" —
  jeweils vor und nach dem Schema-Swap, mit byte-identischem Pass/Fail-Ergebnis.
  Nicht als Prosa im SUMMARY, sondern als Test, der beide Seiten des Swaps
  belegt.

**Gesperrt — in dieser Phase nicht neu aufmachen**

- `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**. `person` ist rein
  administrativ; keine der vier Bump-Klauseln aus `CLAUDE.md` greift.
- `sales_person.id` und `user.name` werden nicht umgeschlüsselt.
- `person` bekommt keine Namensspalten und keine `deleted`-Spalte.
- Der Index auf `sales_person.person_id` bleibt **vollständig**, nicht
  partiell — kein `WHERE deleted IS NULL` nachrüsten (D-60-01, one-way).
- HR-/Admin-Accounts ohne Sales Person und Freiwillige ohne Login bleiben
  gleichwertige Ausprägungen (D-60-03). Nirgends automatisch beides anlegen.
- `PersonService` ist **Basic-Tier**: nur `PersonDao` + `PermissionService` +
  Clock/Uuid + `TransactionDao`, **kein** anderer Domain-Service.
  DI-Konstruktion in `shifty_bin/src/main.rs` im Basic-Block, strikt vor der
  Business-Logic-Schicht.
- Die öffentlichen Signaturen von `SalesPersonService` (`set_user`,
  `get_assigned_user`, `get_all_user_assignments`, `get_sales_person_for_user`,
  `get_sales_person_current_user`, `verify_user_is_sales_person`) bleiben
  **unverändert**. Kein Endpoint, kein TO, kein Frontend-Anteil.

### Claude's Discretion

- `ServiceError`-Variante für den Verknüpfungs-Konflikt und ihr HTTP-Mapping
  auf 409. Einzige harte Vorgabe: unterscheidbar von
  `ServiceError::EntityConflicts` (heutiger Versions-Konflikt).
- API-Zuschnitt von `PersonDao` und `PersonService` — genau so groß, wie
  Phase 61 plus die „unverknüpfte Kandidaten"-Lesepfade es verlangen. Keine
  Vorratshaltung für Phase 62/63.
- Wo Merge- und Konflikt-Logik sitzt (DAO-Ebene vs. `PersonService`) —
  freigestellt, solange `SalesPersonService`-Signaturen + die ~50
  Aufrufstellen unberührt bleiben (SC3) und `PersonService` Basic-Tier bleibt
  (SC5).
- Migrations-Dateiname und Zeitstempel-Präfix, Index-Namen, konkrete Form der
  Selbst-Assertion aus D-61-06, Ablage der Tests.
- Ob `BasicDaoImpl::clear_all` zusätzlich ein `DELETE FROM person` bekommt,
  nachdem die `sales_person_user`-Zeile dort entfällt.
- Wellen-Schnitt und Plan-Anzahl.

### Deferred Ideas (OUT OF SCOPE)

- Zusammenführen zweier Personen im UI (zurückkehrender Mitarbeiter) — die
  Merge-Mechanik entsteht in dieser Phase (D-61-03), eine Bedienoberfläche
  dafür nicht.
- Kandidaten-Picker-UI für PADM-04 — die Lesepfade entstehen hier (D-61-02),
  die Oberfläche ist Phase 63.
- Person löschen inkl. `deleted`-Spalte auf `person` — frühestens Phase 63.
- Echte Personenstammdaten (E-Mail, Telefon, Geburtsdatum) an `person` —
  PERS-F-01, Future.
- `person_id` schema-seitig auf `NOT NULL` ziehen — SQLite bräuchte ein
  Table-Rebuild. Kein v2.7-Scope.
- `FOREIGN KEY`-Deklaration auf den `person_id`-Kanten — nicht Phase-61-Scope.

</user_constraints>

## Project Constraints (from CLAUDE.md)

- **Service-Tier-Konvention:** Basic Services konsumieren ausschließlich DAOs +
  `PermissionService` + `TransactionDao` (+ `ClockService`/`UuidService` bei
  Entity-Erzeugung). Sobald ein anderer Domain-Service als Dependency
  auftaucht, ist der Service Business-Logic. `PersonService` muss Basic
  bleiben — daraus folgt direkt, dass `PersonService` **nicht** von
  `SalesPersonService` konsumiert werden darf und umgekehrt (sonst Zyklus über
  `PermissionService`, das jeder Basic-Service braucht).
- **Transaction Management:** jede Service-Methode nimmt `Option<Transaction>`,
  holt sich via `transaction_dao.use_transaction(tx)` eine konkrete
  Transaktion und committet selbst.
- **Database Migrations:** `sqlx migrate add <name>`, SQLx Compile-Time-Checking
  braucht eine lokal migrierte DB.
- **Billing Period Snapshot Schema Versioning:** `CURRENT_SNAPSHOT_SCHEMA_VERSION`
  bleibt 13 — keine der vier Bump-Klauseln greift, da `person` keinen
  `BillingPeriodValueType` trägt und keine Computation ändert.
- **Clippy-Gate:** `cargo clippy --workspace -- -D warnings` ist Pflicht bei
  jedem Commit, unabhängig von `cargo test`.
- **Docs-Freshness-Gate:** `shifty_bin/src/main.rs`-Änderungen (DI) triggern
  `docs/architecture/02-service-tiers.md` + `diagrams/service-graph-runtime.mmd`;
  `service/**`/`dao/**`-Trait-Änderungen triggern die passende
  `docs/features/F*.md`; `service_impl/src/permission.rs` triggert
  `docs/architecture/04-auth.md` + `docs/features/F12-auth-session.md`.
  Beide Sprachen (EN + DE) im selben Commit.

## Standard Stack

Keine neuen externen Dependencies. Phase 61 arbeitet ausschließlich mit dem
bereits vorhandenen Stack:

| Baustein | Version | Zweck | Beleg |
|----------|---------|-------|-------|
| `sqlx` (Feature `sqlite`) | 0.8.6 | DAO-Queries, Migrationen | `dao_impl_sqlite/Cargo.toml`, verifiziert in 60-RESEARCH.md |
| `libsqlite3-sys` (gebündelt) | 0.30.1 | SQLite-Engine | `Cargo.lock:1807-1810` [VERIFIED: Cargo.lock] — unterstützt `RETURNING` (seit SQLite 3.35), falls gebraucht |
| `uuid` | bereits Workspace-Dep | Person-IDs falls Rust-seitig generiert | durchgängig im Repo verwendet |
| `time` | bereits Workspace-Dep | `PrimitiveDateTime`/`Iso8601::DATE_TIME` für `person.created` | `dao_impl_sqlite/src/sales_person.rs`, `billing_period.rs` |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Rust-seitig generierte `person_id` (via injiziertem `UuidService` in den DAOs) | SQL-generierte `randomblob(16)` direkt in der INSERT-Anweisung | `randomblob(16)` ist bereits das etablierte Muster aus der Phase-60-Migration (dieselbe Nicht-RFC-4122-Eigenschaft, dokumentiert und getestet in `person_backfill.rs::person_id_parses_as_uuid`); vermeidet eine neue Dependency von `SalesPersonDaoImpl`/`PermissionDaoImpl` auf `UuidService` |

**Installation:** entfällt — keine neuen Packages.

**Version verification:** entfällt — keine neuen Packages, siehe Package
Legitimacy Audit.

## Package Legitimacy Audit

Diese Phase installiert **keine** externen Packages. Kein Legitimacy-Check
nötig.

**Packages removed due to [SLOP] verdict:** keine (kein Package installiert).
**Packages flagged as suspicious [SUS]:** keine.

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│ REST (unverändert — kein Endpoint, kein TO, kein Handler-Diff)   │
│   rest/src/sales_person.rs: GET/POST/DELETE /{id}/user           │
└───────────────────────────┬─────────────────────────────────────┘
                             │ ruft weiterhin dieselben 6 Trait-Methoden
┌───────────────────────────▼─────────────────────────────────────┐
│ SalesPersonService (Basic, Vertrag EINGEFROREN — SC3)            │
│   set_user() ─────────────┐                                      │
│   verify_user_is_sales_person()  (Self-vs-HR-Gate)               │
│   get_assigned_user() / get_all_user_assignments() / ...         │
└───────────────────────────┼──────────────────────────────────────┘
                             │ delegiert 1:1 an DAO (Service-Layer unberührt)
┌───────────────────────────▼──────────────────────────────────────┐
│ SalesPersonDao (Trait-Signaturen unverändert, SQL-Innenleben neu) │
│                                                                    │
│  find_by_user/find_sales_person_by_user_id                        │
│    ALT: JOIN sales_person_user spu ON sp.id=spu.sales_person_id   │
│    NEU: JOIN user u ON u.person_id = sp.person_id                 │
│         (SQL-Join — NULL=NULL matcht nie → fail-closed)           │
│                                                                    │
│  get_assigned_user/get_all_user_assignments                       │
│    ALT: SELECT ... FROM sales_person_user                         │
│    NEU: SELECT ... FROM sales_person sp JOIN user u                │
│         ON u.person_id = sp.person_id                             │
│                                                                    │
│  assign_to_user/discard_assigned_user (ECHTE neue Logik)          │
│    Merge zweier person-Zeilen (D-61-03) mit Konflikt-Check        │
│    (D-61-04) bzw. Split mit frischer person-Zeile (D-61-05)       │
└───────────────────────────┬──────────────────────────────────────┘
                             │ neu: konsumiert PersonDao NICHT über
                             │ SalesPersonService — Merge-SQL liegt
                             │ direkt im DAO (kein Service-Zyklus)
┌───────────────────────────▼──────────────────────────────────────┐
│ PersonDao/PersonService (NEU, Basic-Tier, eigener DI-Zweig)        │
│   nur CRUD über `person` + „unverknüpfte Kandidaten"-Lesepfade    │
│   Deps: PersonDao, PermissionService, ClockService, UuidService,  │
│         TransactionDao — KEIN SalesPersonService, KEIN UserService │
└───────────────────────────┬──────────────────────────────────────┘
                             │
┌───────────────────────────▼──────────────────────────────────────┐
│ SQLite: person (Phase 60) · sales_person.person_id (UNIQUE)       │
│         user.person_id (UNIQUE) · sales_person_user (bis DROP)    │
└─────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
dao/src/person.rs                      # NEU: PersonDao-Trait, PersonEntity
dao_impl_sqlite/src/person.rs          # NEU: PersonDaoImpl
service/src/person.rs                  # NEU: PersonService-Trait
service_impl/src/person.rs             # NEU: PersonServiceImpl (gen_service_impl!)
dao_impl_sqlite/src/sales_person.rs    # GEÄNDERT: 6 Methoden umgeschrieben
dao_impl_sqlite/src/lib.rs             # GEÄNDERT: create_user (Person-Anlage),
                                        #           clear_all (sales_person_user raus)
service_impl/src/sales_person.rs       # UNVERÄNDERT (Service-Layer delegiert 1:1)
service/src/lib.rs                     # GEÄNDERT: neue ServiceError-Variante
migrations/sqlite/2026091700000X_...sql  # NEU: Nachzug + Assertion + DROP
service_impl/src/test/person_dao.rs    # NEU: DAO-Integrationstests (Merge/Split/Self-Gate)
service_impl/src/test/person_backfill.rs # ERGÄNZT: DROP-Nachweis-Tests (oder eigene Datei)
```

### Pattern 1: Basic-Service ohne Domain-Service-Dependency

**Was:** `PersonService` folgt exakt dem Muster von `WeekStatusService`
(v2.1 Phase 40) und `RebookingBatchService` (v2.6 Phase 54) — reines CRUD +
Permission-Gate, DI-Konstruktion im Basic-Block **vor** jedem
Business-Logic-Konsumenten.

**When to use:** Immer, wenn ein neuer Service nur ein einzelnes Aggregat
verwaltet und potenziell von mehreren Basic-Services als „Nachbar" gebraucht
wird, ohne dass einer den anderen kennen darf.

**Example (DI-Konstruktion, Muster aus `shifty_bin/src/main.rs:1287-1293`):**
```rust
// Source: shifty_bin/src/main.rs:1282-1293 (WeekStatusService, wörtliches
// Vorbild für PersonService — gleicher Dependency-Schnitt)
let week_status_service = Arc::new(WeekStatusService {
    week_status_dao: Arc::new(WeekStatusDao::new(pool.clone())),
    permission_service: permission_service.clone(),
    clock_service: clock_service.clone(),
    uuid_service: uuid_service.clone(),
    transaction_dao: transaction_dao.clone(),
});
```
Für `PersonService` identisch, nur mit `person_dao` statt `week_status_dao`.
Platzierung: irgendwo im Basic-Block (`shifty_bin/src/main.rs:1000-1060`),
**vor** jedem künftigen `PersonDirectoryService` (Phase 62) — aber Phase 61
selbst braucht `PersonService` nur als `RestStateImpl`-Feld ohne notwendigen
`RestStateDef`-Accessor (Präzedenz: `VoluntaryRebookingScheduler` hat auch
keinen Accessor, solange kein REST-Handler ihn braucht — siehe
`docs/architecture/02-service-tiers.md:79-84`). Ein Accessor schadet nicht,
ist aber nicht durch SC5 gefordert.

### Pattern 2: Sicherer Self-Gate-Join statt Rust-Options-Vergleich

**Was:** Die Autorisierungsprüfung, ob der aktuelle Benutzer „seine eigene"
Sales Person anfasst, wird über einen SQL-`JOIN` entschieden statt über einen
Rust-`==`-Vergleich zweier `Option<Uuid>`.

**When to use:** Immer, wenn eine Sicherheitsentscheidung von einer
möglicherweise-`NULL`-Spalte abhängt.

**Example:**
```sql
-- Source: dao_impl_sqlite/src/sales_person.rs:107-111 (ALT, wird ersetzt)
SELECT sp.id, sp.name, sp.background_color, sp.is_paid, sp.inactive,
       sp.deleted, sp.update_version
FROM sales_person sp
JOIN sales_person_user spu ON sp.id = spu.sales_person_id
WHERE spu.user_id = ?
```
```sql
-- NEU (D-61-08) — identisches Output-Schema, SalesPersonDb/TryFrom bleibt
-- unverändert, weil dieselben sp.*-Spalten selektiert werden.
SELECT sp.id, sp.name, sp.background_color, sp.is_paid, sp.inactive,
       sp.deleted, sp.update_version
FROM sales_person sp
JOIN user u ON u.person_id = sp.person_id
WHERE u.name = ?
```
Wenn `sp.person_id` oder `u.person_id` `NULL` ist (sollte nach D-61-01 nicht
mehr vorkommen, ist aber während der Übergangsphase theoretisch möglich), matcht
der `JOIN` diese Zeile **nicht** — SQLs Drei-Werte-Logik lässt `NULL = NULL` nie
wahr werden. Eine äquivalente Rust-Implementierung
(`sp_person_id == user_person_id` mit beiden als `Option<Uuid>`) würde bei
`None == None` `true` liefern und damit fail-open gehen — exakt das, was
D-61-08 verbietet. Identisches Muster für `find_by_user`
(`dao_impl_sqlite/src/sales_person.rs:226-235`, heute Byte-für-Byte dieselbe
Query wie `find_sales_person_by_user_id`).

Für `get_assigned_user` (liefert den Username zu einer Sales-Person-ID) und
`get_all_user_assignments` (liefert die volle Zuordnungstabelle) gilt dieselbe
Umformung, nur ohne Autorisierungsrelevanz (reine Lookup-Queries):

```sql
-- get_assigned_user, NEU
SELECT u.name AS user_id
FROM sales_person sp
JOIN user u ON u.person_id = sp.person_id
WHERE sp.id = ?
```
```sql
-- get_all_user_assignments, NEU
SELECT sp.id AS sales_person_id, u.name AS user_id
FROM sales_person sp
JOIN user u ON u.person_id = sp.person_id
```

### Pattern 3: Merge-Algorithmus für `set_user(Some(user))` (D-61-03/D-61-04)

**Was:** Verknüpfen ist kein Insert in eine Join-Tabelle mehr, sondern das
Verschmelzen zweier `person`-Zeilen mit einer expliziten
Mehrdeutigkeits-Prüfung.

**Ausgangslage (verifiziert):** Nach D-61-01 sind `sales_person.person_id`
(=`P_s`) und `user.person_id` (=`P_u`) für jede lebende Zeile nicht-`NULL`.
Beide Spalten tragen je einen vollständigen `UNIQUE`-Index
(`sales_person_person_id_unique_idx`, `user_person_id_unique_idx` — Namen aus
`migrations/sqlite/20260916000000_create-person-backfill.sql:46-50`). Es gibt
**keine** `FOREIGN KEY`-Deklaration auf `person_id` (Deferred-Punkt in
60-CONTEXT.md) — ein `DELETE FROM person` wird also nicht durch eine
FK-Constraint blockiert, sondern muss applikationsseitig sicherstellen, dass
niemand mehr auf die gelöschte Zeile zeigt.

**Algorithmus** (innerhalb der bestehenden Service-Transaktion von
`set_user`, kein zusätzliches `BEGIN`):

```sql
-- Schritt 1: aktuelle Personen beider Seiten lesen
SELECT person_id FROM sales_person WHERE id = ?;          -- P_s
SELECT person_id FROM user WHERE name = ?;                -- P_u

-- Schritt 2 (nur falls P_s <> P_u — sonst ist schon verlinkt, No-Op):
-- Konflikt A: hat P_s bereits einen ANDEREN Benutzer?
SELECT name FROM user WHERE person_id = ? AND name != ?;  -- P_s, target_user
-- Konflikt B: hat P_u bereits eine ANDERE Sales Person?
SELECT id FROM sales_person WHERE person_id = ? AND id != ?; -- P_u, target_sp

-- Liefert Konflikt A ODER B eine Zeile → 409 (D-61-04), keine Schreibung.

-- Schritt 3 (kein Konflikt): Benutzer auf P_s ummelden. WHERE-Guard auf den
-- zuvor gelesenen P_u ist Defense-in-Depth gegen eine Race innerhalb
-- derselben Transaktion (rows_affected prüfen, analog dem
-- state='pending'-Guard in dao_impl_sqlite/src/rebooking_batch.rs:350-369).
UPDATE user SET person_id = ? WHERE name = ? AND person_id = ?; -- P_s, target_user, P_u

-- Schritt 4: die jetzt verwaiste Person löschen. Die beiden NOT EXISTS-Guards
-- sind reine Absicherung — nach Schritt 3 zeigt garantiert nichts mehr auf P_u.
DELETE FROM person
WHERE id = ?
  AND NOT EXISTS (SELECT 1 FROM sales_person WHERE person_id = person.id)
  AND NOT EXISTS (SELECT 1 FROM user WHERE person_id = person.id);
```

**Warum keine transiente Constraint-Verletzung möglich ist:** Der Merge setzt
zu keinem Zeitpunkt zwei Zeilen **derselben** Tabelle auf denselben
`person_id`-Wert — Schritt 3 ändert genau eine `user`-Zeile, und nach
Konflikt-Check B gibt es keine zweite `sales_person`-Zeile mit `person_id =
P_u`. Die beiden UNIQUE-Indizes wirken pro Tabelle, nicht tabellenübergreifend
— dass eine `sales_person`- und eine `user`-Zeile denselben `person_id`-Wert
tragen, ist genau die Definition von „verknüpft", keine Verletzung.
**[CITED: SQLite-Doku-Konsens per Websearch]** SQLite prüft `UNIQUE`-Indizes
immer sofort nach der auslösenden Anweisung, nie deferred bis zum Commit —
anders als `FOREIGN KEY`, das über `PRAGMA defer_foreign_keys` aufschiebbar
wäre. Für diesen Algorithmus ist das irrelevant, weil ohnehin nie eine
transiente Doppelbelegung entsteht — es ist aber wichtig zu wissen, dass ein
Ordnungsfehler in einer künftigen Änderung **sofort** an der auslösenden
Anweisung scheitern würde, nicht erst am Ende der Transaktion.

**Wo der Merge sitzt:** Claude's Discretion (CONTEXT.md). Empfehlung: im DAO
(`SalesPersonDaoImpl::assign_to_user`), weil (a) die Konflikt-Prüfung reine
SQL-Lesungen sind, kein Cross-Service-Aufruf, und (b) `SalesPersonService`
dadurch **nicht** zu `PersonService` als Dependency greifen muss — das erhält
sowohl SC3 (Service-Vertrag unverändert) als auch die Basic-Tier-Klassifikation
von `SalesPersonService` selbst (die per CLAUDE.md-Faustregel kippen würde,
sobald ein Domain-Service als Dependency auftaucht).

### Pattern 4: Split-Algorithmus für `set_user(None)` (D-61-05)

```sql
-- Schritt 1: aktuelle Person lesen
SELECT person_id FROM sales_person WHERE id = ?; -- P

-- Schritt 2: aktuell verknüpften Benutzer finden (falls keiner → No-Op,
-- identisch zum heutigen discard_assigned_user-Verhalten bei fehlender Zeile)
SELECT name FROM user WHERE person_id = ?; -- P

-- Schritt 3 (nur falls ein Benutzer gefunden wurde): frische Person anlegen
-- und den Benutzer umhängen. sales_person.person_id bleibt UNVERÄNDERT (P).
INSERT INTO person (id, created, update_process, update_version)
VALUES (randomblob(16), strftime('%Y-%m-%dT%H:%M:%S.000000000Z','now'),
        ?, randomblob(16));
UPDATE user SET person_id = last_insert_rowid() ...
-- ACHTUNG: person.id ist ein BLOB-PK, kein INTEGER-Rowid-Alias —
-- last_insert_rowid() funktioniert hier NICHT. Entweder die neue ID vorher
-- in Rust generieren (randomblob-Äquivalent: uuid::Uuid::new_v4() oder ein
-- roher 16-Byte-Zufallswert) und in BEIDE Statements einsetzen, oder
-- `INSERT ... RETURNING id` verwenden (SQLite ≥3.35, von
-- libsqlite3-sys 0.30.1 unterstützt, aber im Repo bislang ungenutztes
-- Muster — neu einführen ist in Ordnung, aber explizit vermerken).
```

**Warum das Pitfall-5-Fall-1 („unlink to zero") strukturell ausschließt:**
Nach Schritt 3 trägt `P` weiterhin die Sales Person (unverändert), und die neue
Person trägt sofort den Benutzer. Zu keinem Zeitpunkt existiert eine
`person`-Zeile ohne Träger — die Reihenfolge „erst neue Person anlegen, dann
Benutzer umhängen" ist zwingend, damit der Benutzer nie transient ohne gültige
`person_id` dasteht (die Spalte ist zwar nullable, aber D-61-01 will genau das
vermeiden).

### Anti-Patterns to Avoid

- **Rust-seitiger `Option<Uuid>`-Vergleich für den Self-Gate:** `sp_person_id
  == user_person_id` mit beiden als `Option<Uuid>` in Rust geladen — matcht
  `None == None` als `true` und öffnet fail-open (D-61-08, Pitfall 4).
- **`WHERE person_id IS NULL` als Definition von „unverknüpft":** nach D-61-01
  ist `person_id` faktisch nie `NULL` — die Definition muss über den fehlenden
  Join-Partner laufen (D-61-02).
- **`PersonService` als Dependency von `SalesPersonService` (oder
  umgekehrt):** bricht sowohl die Basic-Tier-Klassifikation von
  `SalesPersonService` als auch SC5 für `PersonService`. Die Merge-Logik gehört
  ins DAO oder in einen dedizierten, isolierten Codepfad — nicht in einen
  Cross-Service-Aufruf.
- **Read-then-INSERT ohne DB-Autorität** für die Konflikt-Erkennung: der
  Konflikt-Check MUSS die tatsächlichen `UNIQUE`-Constraints als
  Rückversicherung respektieren (`rows_affected`-Prüfung nach dem `UPDATE`,
  analog `dao_impl_sqlite/src/rebooking_batch.rs:350-369`), nicht sich blind
  auf die vorherige `SELECT`-Prüfung verlassen.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Person-ID-Generierung | eigenes UUID-Format oder Sequenz | `randomblob(16)` in SQL (Phase-60-Präzedenz) oder `uuid::Uuid::new_v4()` in Rust, konsistent mit dem bereits akzeptierten „keine RFC-4122-UUID"-Fakt aus `person_backfill.rs::person_id_parses_as_uuid` | Neuerfindung würde eine zweite, inkonsistente ID-Konvention neben der Phase-60-Migration einführen |
| Konflikt-Ablehnung beim Merge | eigene In-Memory-Locking-Logik über mehrere Requests | die bestehenden UNIQUE-Indizes + `rows_affected`-Guard innerhalb der Service-Transaktion, Muster aus `rebooking_batch.rs` | Diese Technik ist im Repo bereits etabliert (D-54-DM-01, D-55-A) und deckt die Race-Bedingung ab, ohne neue Infrastruktur |
| 409-Fehler-Mapping | Handler-lokales Status-Mapping | zentraler `error_handler` in `rest/src/lib.rs:145+` | Jeder andere Konflikt-Pfad im Repo (`EntityAlreadyExists`, `EntityConflicts`) läuft bereits zentral — ein Handler-lokaler Sonderfall wäre Inkonsistenz |

**Key insight:** Diese Phase hat fast keine neuen Infrastruktur-Bedürfnisse —
jedes Muster (Basic-Service-DI, UNIQUE-Index-basierte Konflikterkennung,
zentrales Error-Mapping, SQL-generierte IDs) hat eine direkte, wörtliche
Präzedenz in diesem Repo aus Phase 40/54/55/60. Die Aufgabe ist Disziplin bei
der Anwendung, nicht Design neuer Mechanik.

## Common Pitfalls

### Pitfall 1: Rust-`Option<Uuid>`-Vergleich statt SQL-Join beim Self-Gate

**What goes wrong:** Ein DAO-Rewrite, der `sp.person_id` und `u.person_id`
getrennt lädt und in Rust vergleicht, matcht `None == None` als `true` und gibt
Fremdzugriff frei.
**Why it happens:** Es ist der naheliegendste 1:1-Rewrite-Impuls („lade beide
Werte, vergleiche sie") und kompiliert klaglos.
**How to avoid:** Ausschließlich als SQL-`JOIN` implementieren (Pattern 2
oben). Kein Zwischenschritt, der `person_id` als `Option<Uuid>` in Rust hält
und vergleicht.
**Warning signs:** Ein `if sp_person == user_person` (oder ähnlich) irgendwo
im DAO- oder Service-Code für `find_sales_person_by_user_id`/`find_by_user`.

### Pitfall 2: Byte-Identity-Test überlebt die DROP-Migration nicht sauber

**What goes wrong:** Der Vorher/Nachher-Permission-Matrix-Test (D-61-09) muss
zwangsläufig die alte, `sales_person_user`-basierte Query zitieren, um „vorher"
zu belegen. Bleibt dieser Testcode nach der DROP-Migration im Repo stehen,
verletzt er Success Criterion 2 (grep findet `sales_person_user` nur noch in
historischen Migrationen).
**Why it happens:** Das natürlichste Vorgehen ist, den Vorher/Nachher-Test
einmal zu schreiben und liegen zu lassen — aber SC2 verlangt einen sauberen
Endzustand.
**How to avoid:** Siehe Research-Frage 3 unten — der Test entsteht in der
DAO-Rewrite-Welle (während `sales_person_user` noch existiert), sein
„Vorher"-Anteil wird in der DROP-Welle explizit entfernt, sein
„Nachher"-Anteil bleibt als dauerhafter Regressionstest stehen.
**Warning signs:** `grep -rn "sales_person_user" service_impl/src/test/`
findet nach Abschluss der Phase noch Treffer außerhalb von
Migrations-Referenzkommentaren.

### Pitfall 3: `last_insert_rowid()` funktioniert nicht für `person.id`

**What goes wrong:** `person.id` ist `blob(16) PRIMARY KEY`, kein
`INTEGER PRIMARY KEY`-Rowid-Alias. `last_insert_rowid()` liefert die interne
SQLite-Rowid, nicht den BLOB-Wert — ein Code-Pfad, der sich darauf verlässt,
bekommt eine falsche oder unbrauchbare ID.
**Why it happens:** `last_insert_rowid()` ist der naheliegende Reflex nach
einem `INSERT`, funktioniert aber nur bei `INTEGER PRIMARY KEY`-Tabellen.
**How to avoid:** Die neue `person.id` **vor** dem `INSERT` in Rust generieren
und explizit in beide folgenden Statements einsetzen — oder `INSERT ...
RETURNING id` verwenden (SQLite ≥3.35, von der gebündelten
`libsqlite3-sys 0.30.1` unterstützt, aber ein im Repo bislang ungenutztes
Muster).
**Warning signs:** `last_insert_rowid()` irgendwo im Zusammenhang mit
`person`-Inserts.

### Pitfall 4: `PermissionDao::create_user`-Atomarität wird stillschweigend aufgegeben

**What goes wrong:** `create_user` (Login-Auto-Anlage,
`dao_impl_sqlite/src/lib.rs:93-104`) läuft heute auf `self.pool` **ohne**
Transaktionsparameter. Wird die Person-Anlage naiv als zweiter, unabhängiger
`INSERT` direkt danach hinzugefügt (zwei separate `self.pool`-Statements ohne
gemeinsame Transaktion), kann ein Absturz zwischen beiden Inserts einen
`user` ohne `person_id` hinterlassen — genau der Zustand, den D-61-01
verhindern will.
**Why it happens:** Die Trait-Signatur `create_user(&self, user: &UserEntity,
process: &str) -> Result<(), DaoError>` nimmt bewusst keine Transaktion (Login
ist heute ein einzelner Insert) — das ist eine echte strukturelle Asymmetrie
zu `SalesPersonDao::create`, das bereits eine `Self::Transaction` bekommt.
**How to avoid:** `PermissionDaoImpl::create_user` öffnet **intern** eine
lokale Transaktion (`self.pool.begin()...commit()`), die beide Inserts
(`person`, dann `user`) umschließt — ohne die Trait-Signatur zu ändern (keine
Auswirkung auf die beiden Aufrufer `PermissionServiceImpl::create_user` und
`UserInvitationServiceImpl`). `SalesPersonDaoImpl::create` braucht diesen
Kunstgriff nicht — es bekommt bereits eine `tx` vom Aufrufer und kann die
`person`-Insert einfach als zweites Statement auf derselben `tx` ausführen.
**Warning signs:** Zwei unabhängige `self.pool`-Aufrufe ohne umschließende
Transaktion im geänderten `create_user`.

### Pitfall 5: `PersonEntity`/`PersonDao`-Zuschnitt wird zu groß

**What goes wrong:** Der Impuls, `PersonDao` gleich für Phase 62/63 mit
vorzubereiten (z. B. ein volles Aggregat-Read mit Rollen/Einladungen), zieht
eine Domain-Service-Abhängigkeit nach sich und verletzt SC5 (Basic-Tier).
**Why it happens:** „Wir bauen das doch eh gleich" ist ein natürlicher Reflex
bei einem neuen Service in einer Migrations-Phase.
**How to avoid:** `PersonDao`/`PersonService` bleiben strikt auf: CRUD über
`person`, Merge-/Split-Hilfsoperationen (falls dort statt im DAO
implementiert), „unverknüpfte Kandidaten"-Lesepfade. Explizit in CONTEXT.md
als Claude's Discretion mit der Einschränkung „keine Vorratshaltung für
Phase 62/63" markiert.
**Warning signs:** `PersonService` bekommt ein Feld vom Typ
`Arc<dyn SalesPersonService<...>>` oder ähnliches.

## Runtime State Inventory

> Diese Phase ist ein Rename/Refactor mit Datenmigration (Auflösung einer
> Verknüpfungstabelle) — die Inventur ist Pflicht.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | `sales_person_user`-Zeilen (Produktions-DB, noch nicht deployt — siehe STATE.md „v2.7 Deploy-Gates" / CR-02) tragen die 1:1-Verknüpfung, die nach der DROP-Migration ausschließlich über `person_id` ausgedrückt wird. Die Phase-60-Migration hat bereits einen Backfill durchgeführt; Phase 61 braucht **zusätzlich** einen Nachzieh-Backfill für Zeilen, die zwischen Phase-60- und Phase-61-Deploy entstanden sind (D-61-06) — in Produktion praktisch irrelevant, da 60+61 zusammen live gehen, aber in Entwicklungs-DBs real durchlaufen. | Code-Änderung (DAO-Rewrite) + einmalige Daten-Migration (DROP-Migration mit Nachzieh-Backfill) |
| Live service config | Keine — Phase 61 berührt keine extern konfigurierten Services (kein n8n, kein Toggle-System betroffen). | keine |
| OS-registered state | Keine. | keine |
| Secrets/env vars | Keine — keine neuen Env-Vars, keine SOPS-Keys betroffen. | keine |
| Build artifacts | `.sqlx/query-*.json` (Offline-Query-Cache, `SQLX_OFFLINE=true` in CI). Jede geänderte oder neue `sqlx::query!`/`query_as!`-Anweisung braucht einen frischen Cache-Eintrag, sonst schlägt der CI-Build fehl (stale-cache-Miss, kein normaler Rust-Compile-Fehler). | `cargo sqlx prepare --workspace` (in `nix develop`, gegen eine lokal migrierte DB) nach jeder Query-Änderung laufen lassen und die neuen `.sqlx/query-*.json`-Dateien committen (Präzedenz: 60-RESEARCH.md dokumentiert exakt diesen Schritt) |

**Nichts gefunden in:** Live service config, OS-registered state,
Secrets/env vars — explizit verifiziert, keine Treffer bei der Durchsicht der
betroffenen Dateien.

## Research Findings — Die sieben offenen Fragen

### Frage 1: Merge-/Konflikt-Algorithmus und UNIQUE-Index-Sicherheit

Beantwortet in „Architecture Patterns" → Pattern 3 (Merge) und Pattern 4
(Split) oben. Kernaussagen:

- Index-Namen (Phase 60, verifiziert in
  `migrations/sqlite/20260916000000_create-person-backfill.sql:46-50`):
  `sales_person_person_id_unique_idx` und `user_person_id_unique_idx`, beide
  **vollständig** (kein `WHERE`), beide `UNIQUE` (nicht `PRIMARY KEY`).
- Der Merge (D-61-03/D-61-04) verletzt zu keinem Zeitpunkt einen der beiden
  Indizes transient, weil er niemals zwei Zeilen derselben Tabelle
  gleichzeitig auf denselben `person_id`-Wert setzt — nur eine `user`-Zeile
  wird pro Merge umgehängt, und die Ziel-`person_id` gehört bereits exklusiv
  der Sales Person.
- **[CITED: SQLite-Semantik, per Websearch bestätigt]** SQLite prüft
  `UNIQUE`-Constraints immer **sofort** nach der auslösenden Anweisung, nicht
  deferred bis zum Commit — im Gegensatz zu `FOREIGN KEY`, das über `PRAGMA
  defer_foreign_keys` aufschiebbar ist. Für den hier entworfenen Algorithmus
  ändert das nichts (keine transiente Verletzung geplant), ist aber die
  Antwort auf die explizite Forschungsfrage.
- Die Konflikt-Erkennung selbst (Schritt 2 in Pattern 3) ist **kein**
  Constraint-Check, sondern eine Anwendungslogik-Prüfung (dritte Zeile
  betroffen, nicht die zwei direkt beteiligten) — sie muss per `SELECT`
  innerhalb derselben Transaktion erfolgen, abgesichert durch einen
  `rows_affected`-Guard auf dem anschließenden `UPDATE` als Verteidigung gegen
  eine (bei HR-gated Admin-Aktionen unwahrscheinliche, aber nicht
  unmögliche) Race.

### Frage 2: Self-Gate-SQL-Rewrite

Beantwortet in „Architecture Patterns" → Pattern 2 oben. Kernaussage: Die
Query-Form bleibt `SELECT sp.id, sp.name, sp.background_color, sp.is_paid,
sp.inactive, sp.deleted, sp.update_version FROM sales_person sp JOIN ... WHERE
...` — exakt dieselben Spalten wie heute, nur der `JOIN`-Partner wechselt von
`sales_person_user` auf `user` (Bedingung `u.person_id = sp.person_id`
statt `spu.sales_person_id = sp.id`). `SalesPersonDb`/`impl TryFrom<&SalesPersonDb>
for SalesPersonEntity` (`dao_impl_sqlite/src/sales_person.rs:23-49`) bleibt
**vollständig unverändert**, weil `sqlx::query_as!` nur die selektierten
Spalten prüft, nicht die Join-Struktur der Query. `find_by_user` und
`find_sales_person_by_user_id` sind heute Byte-für-Byte identische Queries
(verifiziert: Zeilen 107-111 und 231-235) — dieser Umstand bleibt nach dem
Rewrite bestehen, ist aber vorbestehende Duplikation, keine neue.

### Frage 3: Byte-identische Vorher/Nachher-Permission-Matrix über einen One-Way-Swap

**Der entscheidende strukturelle Fakt:** `sales_person_user` bleibt laut
D-61-06/D-61-07 bis zur **letzten** Migration innerhalb von Phase 61 am Leben
— die DROP-Migration ist die letzte Handlung der Phase, nicht die erste. Das
bedeutet: **innerhalb** der Phase, für eine Übergangszeit von mindestens einer
Plan-Welle, koexistieren `sales_person_user` UND die vollständig funktionsfähige
`person_id`-basierte DAO-Implementierung in derselben Codebase und derselben
Test-Fixture-DB.

Daraus folgt ein konkreter, ausführbarer Mechanismus für D-61-09 — **kein**
Golden-Snapshot-Literal wie beim v2.5-Refactor (dort gab es keine Möglichkeit,
alte und neue Query gleichzeitig laufen zu lassen, weil der Code komplett
ersetzt wurde), sondern ein **Side-by-Side-Vergleich in einer einzigen
Test-Session gegen eine einzige Fixture-DB**:

1. **In der DAO-Rewrite-Welle** (bevor die DROP-Migration existiert): ein
   Integrationstest (echtes In-Memory-SQLite + `sqlx::migrate!`, Muster aus
   `service_impl/src/test/person_backfill.rs`) seedet die drei SC1-Fälle
   (self / fremd / unverknüpft) sowohl mit `sales_person_user`-Zeilen als auch
   mit den entsprechenden `person_id`-Werten. Der Test führt **zwei** Queries
   pro Fall aus: die **alte** Query (roh per `sqlx::query`/`query_as` direkt
   im Testcode, referenziert `sales_person_user` explizit — nicht über den
   DAO-Trait, weil der zu diesem Zeitpunkt bereits umgeschrieben ist) und die
   **neue** Query (über den echten, bereits umgeschriebenen `SalesPersonDao`).
   Für alle drei Fälle wird `assert_eq!(alt_ergebnis.is_some(),
   neu_ergebnis.is_some())` (bzw. die passende Accept/Reject-Semantik für
   `verify_user_is_sales_person`) geprüft. Das ist der D-61-09-Nachweis,
   erbracht **während** beide Implementierungen nebeneinander existieren —
   nicht gegen zwei verschiedene Datenbank-Zustände, sondern gegen **eine**
   Fixture-DB mit beiden Datenmodellen gleichzeitig befüllt.
2. **In der DROP-Welle** (letzter Plan der Phase): sobald `sales_person_user`
   gedroppt wird, MUSS der „Alt-Query"-Anteil dieses Tests entfernt werden —
   sonst verletzt der verbleibende String `"sales_person_user"` im Testcode
   Success Criterion 2 (grep-Sauberkeit außerhalb historischer Migrationen).
   Der „Neu-Query"-Anteil (die drei Accept/Reject-Assertions gegen die
   `person_id`-basierte Implementierung) bleibt als **dauerhafter**
   Regressionstest stehen — er ist der permanente Existenzbeweis für PERS-05,
   nicht nur ein Migrations-Artefakt.

**Konsequenz für die Plan-Struktur:** Die Plan-Welle, die die sechs
DAO-Methoden umschreibt, MUSS den Vorher/Nachher-Vergleich als eigenen Test
enthalten (nicht erst die DROP-Welle) — genau dort ist er beweisbar, weil
danach die „Vorher"-Seite nicht mehr existiert. Die DROP-Welle enthält als
eigene Aufgabe: „lösche den Alt-Query-Testcode, behalte die
Accept/Reject-Assertions". Dieses Detail gehört explizit in den Plan, sonst
läuft die Phase Gefahr, SC2 (grep-Sauberkeit) am Ende zu verfehlen, weil der
D-61-09-Beweis versehentlich als Dauerzustand im Repo verbleibt.

**Wo ablegen:** Neue Testdatei, z. B.
`service_impl/src/test/person_self_gate.rs` (Integrationstest, echtes SQLite,
kein Mock — der `SalesPersonServiceImpl`-Unit-Test in
`service_impl/src/test/sales_person.rs` mockt `SalesPersonDao` komplett und
kann die SQL-Join-Korrektheit strukturell **nicht** prüfen). Ablage im
Discretion-Bereich, aber die Notwendigkeit einer **echten** DB-Integration
(kein Mock) ist keine Diskretion — ein gemockter DAO würde die eigentliche
Sicherheitseigenschaft (SQL-Join statt Rust-Vergleich) gar nicht testen
können.

### Frage 4: DROP-Migration — Nachzieh-Backfill, Selbst-Assertion, DROP TABLE

**DROP TABLE in einer sqlx-Migrations-Transaktion:** funktioniert
uneingeschränkt auf SQLite — anders als `ALTER TABLE ... ADD COLUMN` (das
Phase 60 wegen fehlender `IF NOT EXISTS`-Klausel zum Marker-Split-Kunstgriff
zwang), unterstützt SQLite `DROP TABLE IF EXISTS` direkt und vollständig
transaktional. Kein Analogon zu Phase 60s Idempotenz-Problem nötig — `DROP
TABLE IF EXISTS sales_person_user;` ist von sich aus ein zweites Mal
gefahrlos ausführbar (falls die Migration je erneut angewendet würde, was
sqlx' Ledger ohnehin verhindert).

**Form der Migration** (ein File, drei Blöcke, nach dem Muster von
`20260916000000_create-person-backfill.sql`):

```sql
-- Block A: Nachzieh-Backfill (D-61-06) — Zeilen, die zwischen Phase-60- und
-- Phase-61-Deploy entstanden sind und noch kein person_id tragen, aber über
-- sales_person_user verknüpft sind. Idempotency key: person_id IS NULL,
-- identisch zum Muster aus Phase 60.
UPDATE sales_person SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND id NOT IN (SELECT sales_person_id FROM sales_person_user);
UPDATE user SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND name NOT IN (SELECT user_id FROM sales_person_user);
-- verknüpfte Paare: sales_person „gewinnt" die Zufallsgenerierung, user kopiert
-- (identische Reihenfolge-Logik wie Phase 60, Blöcke 3c/3d)
UPDATE sales_person SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND id IN (SELECT sales_person_id FROM sales_person_user);
UPDATE user SET person_id = (
    SELECT sp.person_id FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    WHERE spu.user_id = user.name
)
WHERE person_id IS NULL
  AND name IN (SELECT user_id FROM sales_person_user);
INSERT INTO person (id, created, update_process, update_version)
SELECT DISTINCT ids.person_id, strftime(...), 'phase-61-drop-nachzug', randomblob(16)
FROM (
    SELECT person_id FROM sales_person WHERE person_id IS NOT NULL
    UNION
    SELECT person_id FROM user WHERE person_id IS NOT NULL
) ids
WHERE ids.person_id NOT IN (SELECT id FROM person);

-- Block B: Selbst-Assertion (D-61-06, Form analog D-60-07) — jede noch
-- existierende sales_person_user-Zeile muss auf beiden Seiten ein
-- übereinstimmendes person_id-Gegenstück haben.
CREATE TEMP TABLE IF NOT EXISTS __phase61_drop_assertion (
    ok INTEGER NOT NULL CHECK (ok = 1)
);
INSERT INTO __phase61_drop_assertion (ok)
SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    JOIN user u ON u.name = spu.user_id
    WHERE sp.person_id IS NULL OR u.person_id IS NULL
       OR sp.person_id != u.person_id
) THEN 1 ELSE 0 END;
DROP TABLE __phase61_drop_assertion;

-- Block C: die eigentliche Auflösung (D-61-07)
DROP TABLE IF EXISTS sales_person_user;
```

**D-61-06 verlangt explizit KEINE eigene Fixture-Testmatrix** für Block A/B —
nur „ein Test, der den Zwischenzustand herstellt und nachweist, dass der
Nachzug greift". Präzedenz: `person_backfill.rs::dangling_link_aborts_backfill`
zeigt exakt das benötigte Testmuster (Fixture mit Zwischenzustand, Assertion
schlägt fehl bzw. greift korrekt) — dieselbe Technik (Marker-Split via
`sqlx::raw_sql` auf den isolierten Migrationsabschnitt, oder — da diese
Migration keine `ALTER TABLE`-Idempotenz-Problematik hat — sogar einfacher: die
komplette Datei kann testweise gegen eine Fixture-DB mit Zwischenzustand
gefahren werden, ohne Marker-Split, weil `DROP TABLE IF EXISTS` (anders als
`ALTER TABLE ADD COLUMN`) für einen zweiten Lauf kein Problem darstellt.

**Auswirkung auf `sqlx::query!`-Compile-Time-Checking:** Nach dem `DROP TABLE`
schlägt **jede** verbliebene `sqlx::query!`/`query_as!`-Anweisung, die
`sales_person_user` referenziert, beim nächsten Makro-Recheck fehl — aber
**nicht** als normaler `cargo build`-Fehler, sondern nur, sobald die lokale
Dev-DB neu migriert und der Makro-Cache neu validiert wird (Pitfall 4 aus
PITFALLS.md, „Warning signs"). Mit `SQLX_OFFLINE=true` in CI bedeutet das
konkret: **vor** dem Commit muss `cargo sqlx prepare --workspace` gegen eine
frisch migrierte lokale DB laufen, damit die `.sqlx/query-*.json`-Dateien die
neuen Queries widerspiegeln — sonst baut CI entweder mit veralteten
gecachten Queries (die die Tabelle noch referenzieren, obwohl der Code sie
nicht mehr aufruft — harmlos, aber Altlast) oder schlägt fehl, falls eine
vergessene Query tatsächlich noch `sales_person_user` referenziert. Die
Reihenfolge muss sein: alle sechs DAO-Methoden umschreiben → lokale DB
migrieren → `cargo sqlx prepare --workspace` → DROP-Migration schreiben →
DB erneut migrieren → `cargo sqlx prepare --workspace` erneut (jetzt ohne
jede `sales_person_user`-Referenz im Cache).

### Frage 5: `person_id`-Vergabe bei Neuanlage (D-61-01) — die Transaktions-Asymmetrie

**Verifiziert:**
- `dao_impl_sqlite/src/lib.rs:93-104`
  (`PermissionDaoImpl::create_user`) läuft auf `self.pool` **ohne**
  Transaktionsparameter — Trait-Signatur
  `create_user(&self, user: &UserEntity, process: &str) -> Result<(), DaoError>`.
- `dao_impl_sqlite/src/sales_person.rs:120-138`
  (`SalesPersonDaoImpl::create`) bekommt bereits eine
  `tx: Self::Transaction` vom Aufrufer (`SalesPersonServiceImpl::create`,
  `service_impl/src/sales_person.rs:189-195`, das seinerseits über
  `transaction_dao.use_transaction(tx)` eine Transaktion hält).

**Was das impliziert:**
- Für `SalesPersonDaoImpl::create`: die `person`-Zeile einfach als
  zusätzliches `INSERT` auf derselben `tx` einfügen, direkt vor oder nach dem
  bestehenden `INSERT INTO sales_person`. Atomarität ist automatisch gegeben —
  keine neue Transaktion nötig, kein neuer Fehlerfall.
- Für `PermissionDaoImpl::create_user`: es gibt **keine** äußere Transaktion,
  in die sich ein zweites `INSERT` einhängen könnte. Zwei naive,
  unabhängige `self.pool`-Statements (erst `person`, dann `user`) wären
  **nicht atomar** — ein Absturz zwischen beiden hinterließe entweder eine
  verwaiste `person`-Zeile (harmlos, nur Housekeeping-Ballast) oder — schlimmer
  — einen `user` ohne `person_id`, falls die Reihenfolge umgekehrt wäre. Die
  saubere Lösung: `PermissionDaoImpl::create_user` öffnet **intern** eine
  lokale Transaktion (`self.pool.begin()`), fügt `person` und `user` darin ein,
  committet. Das ändert die Trait-Signatur **nicht** (kein neuer Parameter),
  betrifft also weder `PermissionServiceImpl::create_user` noch
  `UserInvitationServiceImpl` (die beiden einzigen Aufrufer).
- **ID-Generierung ohne Service-Layer-Zugriff:** Weder `SalesPersonDaoImpl`
  noch `PermissionDaoImpl` haben aktuell Zugriff auf `UuidService`/
  `ClockService` (diese leben ausschließlich im Service-Layer). Zwei Optionen:
  (a) `person.id`/`person.created` per SQL generieren (`randomblob(16)`,
  `strftime(...)`, identisch zum Phase-60-Migrationsmuster — **empfohlen**,
  weil konsistenzwahrend und ohne neue Dependency), oder (b) die ID im
  Service-Layer generieren (dort ist `UuidService` bereits injiziert) und als
  zusätzlichen Parameter an die DAO-Methode durchreichen. Da `create`/
  `create_user` nicht zu den in SC3 eingefrorenen Signaturen gehören, ist
  Option (b) technisch zulässig, aber Option (a) braucht **keine**
  Signaturänderung der aufrufenden Service-Methoden und ist damit der
  kleinere Diff — Empfehlung: (a).

### Frage 6: „Unverknüpfte Kandidaten"-Lesepfade (D-61-02)

Kein REST-Endpoint in dieser Phase (reine Backend-Vorarbeit für Phase 63).
Zwei Lesepfade auf `PersonDao`/`PersonService`:

```sql
-- Sales Persons, deren Person keinen Benutzer trägt (Kandidaten für
-- "Person hinzufügen" von der Benutzer-Seite aus). deleted-Filter, weil
-- soft-gelöschte Sales Persons keine sinnvollen Verknüpfungs-Kandidaten sind
-- (D-60-01 lässt ihre person_id bewusst bestehen, das ist ein anderer Zweck).
SELECT sp.id, sp.name, sp.background_color, sp.is_paid, sp.inactive,
       sp.deleted, sp.update_version
FROM sales_person sp
LEFT JOIN user u ON u.person_id = sp.person_id
WHERE u.name IS NULL AND sp.deleted IS NULL;

-- Benutzer, deren Person keine Sales Person trägt (Kandidaten für "Login
-- hinzufügen" von der Sales-Person-Seite aus). Kein deleted-Filter — user hat
-- keine Soft-Delete-Spalte.
SELECT u.name
FROM user u
LEFT JOIN sales_person sp ON sp.person_id = u.person_id
WHERE sp.id IS NULL;
```

Beide sind klassische `LEFT JOIN ... WHERE <rechte-Seite> IS NULL`-Anti-Joins
— die korrekte SQL-Form für „hat keinen Join-Partner", unabhängig davon, ob
`person_id` selbst `NULL` ist (nach D-61-01 ist es das faktisch nie mehr). Das
ist exakt D-61-02s Punkt: die Abfrage prüft die **Abwesenheit eines
Join-Partners**, nicht die Abwesenheit eines Werts.

### Frage 7: Validation Architecture

Siehe eigener Abschnitt unten.

## Code Examples

### Bestehender `set_user`-Service-Pfad (bleibt strukturell erhalten)

```rust
// Source: service_impl/src/sales_person.rs:316-342 — UNVERÄNDERT im Diff,
// nur die darunterliegenden DAO-Methoden (discard_assigned_user,
// assign_to_user) bekommen neues SQL-Innenleben.
async fn set_user(
    &self,
    sales_person_id: Uuid,
    user_id: Option<Arc<str>>,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<(), ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    self.permission_service
        .check_permission(HR_PRIVILEGE, context)
        .await?;
    self.sales_person_dao
        .discard_assigned_user(sales_person_id, tx.clone())
        .await?;
    if let Some(user) = user_id {
        self.sales_person_dao
            .assign_to_user(sales_person_id, user.as_ref(), SALES_PERSON_SERVICE_PROCESS, tx.clone())
            .await?;
    }
    self.transaction_dao.commit(tx).await?;
    Ok(())
}
```
Wichtig: `discard_assigned_user` wird **immer** zuerst aufgerufen, auch beim
Verknüpfen (nicht nur beim Lösen) — der heutige Code trennt zuerst
bedingungslos, bevor er (optional) neu verknüpft. Das DAO-Rewrite muss dieses
Verhalten respektieren: `discard_assigned_user(sales_person_id)` wird
grundsätzlich vor jedem `assign_to_user`-Aufruf ausgeführt. Für D-61-05
(Split) heißt das konkret: **jeder** `set_user`-Aufruf mit `Some(user)` löst
zuerst ein implizites „Lösen" der bisherigen Verknüpfung der Sales Person aus
(falls eine bestand), bevor die neue Verknüpfung (Merge) beginnt — der Planer
muss entscheiden, ob `discard_assigned_user` in diesem Fall tatsächlich den
D-61-05-Split-Pfad durchläuft (alter Benutzer bekommt frische Person) oder ob
das ein separater, einfacherer Teilschritt ist. **[ASSUMED]** — dieser
Sequenzierungs-Punkt ist im CONTEXT.md nicht explizit adressiert und sollte
vom Planer als offene Design-Entscheidung markiert werden (siehe Open
Questions unten).

### Zentrales 409-Mapping (Präzedenz für die neue Konflikt-Variante)

```rust
// Source: rest/src/lib.rs:175-198 — beide bereits existierenden
// Konflikt-Varianten laufen zentral, kein Handler-lokales Mapping.
Err(RestError::ServiceError(service::ServiceError::EntityAlreadyExists(id))) => {
    Response::builder().status(409).body(Body::new(id.to_string())).unwrap()
}
Err(RestError::ServiceError(err @ service::ServiceError::EntityConflicts(_, _, _))) => {
    Response::builder().status(409).body(Body::new(err.to_string())).unwrap()
}
```
**Empfehlung für die neue Variante:** `ServiceError::EntityAlreadyExists(Uuid)`
wiederverwenden (trägt die ID der bereits-verknüpften Gegenseite) — dieser
Pfad ist **bereits** 409-gemappt, **bereits** unterscheidbar von
`EntityConflicts` (SC4s einzige harte Vorgabe), und hat direkte Präzedenz
(`rebooking_batch.rs`: UNIQUE-Slot-Kollision → `DaoError::EntityAlreadyExists`
→ `ServiceError::EntityAlreadyExists` → 409). Alternative: eine neue,
semantisch präzisere Variante (z. B. `PersonLinkConflict(Uuid, Uuid)` mit
beiden beteiligten IDs) einführen — mehr Boilerplate, aber aussagekräftigere
Fehlermeldung. Beides erfüllt SC4; Claude's Discretion laut CONTEXT.md.
**Hinweis zur OpenAPI-Doku:** `POST /{id}/user`
(`rest/src/sales_person.rs:268-303`, `set_sales_person_user`) dokumentiert
aktuell **keinen** 409 in seiner `#[utoipa::path(responses(...))]`-Liste (nur
204/404/500) — die zuvor kolportierte Aussage „409 ist dort bereits
dokumentiert" bezieht sich auf den **anderen** Endpoint `PUT /{id}`
(`update_sales_person`, Zeile 170), nicht auf `POST /{id}/user`. Der Plan
sollte die `responses()`-Liste von `set_sales_person_user` um `(status = 409,
description = "...")` ergänzen — reine Doku-Vollständigkeit, kein
Verhaltensänderung, da `error_handler` das Mapping ohnehin zentral übernimmt.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust eingebaut) + `sqlx::migrate!` für Integrationstests |
| Config file | keins — Konvention über Dateikonvention in `service_impl/src/test/` |
| Quick run command | `cargo test --package service_impl --lib person` (nach Anlage des neuen Testmoduls) |
| Full suite command | `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PERS-05 | Self darf eigene Sales Person sehen/ändern | integration | `cargo test --package service_impl --lib person_self_gate::self_may_access_own_sales_person` | ❌ Wave „DAO-Rewrite" |
| PERS-05 | Fremder Sales-Person-Datensatz bleibt verwehrt | integration | `cargo test --package service_impl --lib person_self_gate::foreign_sales_person_rejected` | ❌ Wave „DAO-Rewrite" |
| PERS-05 | Benutzer ohne Verknüpfung bleibt verwehrt | integration | `cargo test --package service_impl --lib person_self_gate::unlinked_user_rejected` | ❌ Wave „DAO-Rewrite" |
| SC2 (grep-Sauberkeit) | `sales_person_user` nur noch in historischen Migrationen | manual/CI-Grep | `grep -rn "sales_person_user" dao/src dao_impl_sqlite/src service_impl/src` (erwartetes Ergebnis: leer bzw. nur Migrations-Referenzkommentare) | ❌ Wave „DROP" |
| SC3 (Vertrag eingefroren) | ~50 Aufrufstellen unberührt | diff-review + `cargo test --workspace` | `git diff --stat service_impl/` gegen `service/src/sales_person.rs`/`dao/src/sales_person.rs`-Signaturen prüfen | manuell |
| SC4 (Merge-Konflikt) | 409 bei echter Mehrdeutigkeit, Erfolg bei komplementärem Merge | integration | `cargo test --package service_impl --lib person_merge` | ❌ Wave „DAO-Rewrite" |
| SC5 (Basic-Tier) | `PersonService` konsumiert keinen Domain-Service | Code-Review + Compile-Check | manuelle Durchsicht `service_impl/src/person.rs` + `PersonServiceDeps`-Trait | ❌ Wave „PersonDao/Service" |
| D-61-06 (Nachzieh-Backfill) | Zwischenzustand wird nachgezogen | integration | `cargo test --package service_impl --lib ...::dangling_sales_person_user_row_is_backfilled_before_drop` | ❌ Wave „DROP" |

### Sampling Rate

- **Per task commit:** `cargo test --package service_impl --lib person` +
  `cargo test --package dao_impl_sqlite` (falls dort Tests entstehen)
- **Per wave merge:** `cargo test --workspace` + `cargo clippy --workspace --
  -D warnings`
- **Phase gate:** Volle Suite grün + `grep -rn "sales_person_user"
  dao/src dao_impl_sqlite/src service_impl/src` liefert nur noch
  Migrations-Referenzkommentare, bevor `/gsd-verify-work` läuft.

### Wave 0 Gaps

- [ ] `dao/src/person.rs` + `dao_impl_sqlite/src/person.rs` — `PersonDao`-Trait
  + Impl existieren noch nicht.
- [ ] `service/src/person.rs` + `service_impl/src/person.rs` —
  `PersonService`-Trait + Impl existieren noch nicht.
- [ ] `service_impl/src/test/person_self_gate.rs` (oder ähnlich benannt) —
  Integrationstest-Infrastruktur für den Vorher/Nachher-Vergleich existiert
  noch nicht; kann `service_impl/src/test/person_backfill.rs`s
  Harness-Funktionen (`setup_pool`, Fixture-Helfer) als Vorbild wiederverwenden.
- [ ] Neue Migration `2026091700000X_drop-sales-person-user.sql` — existiert
  noch nicht.

*(Kein bestehendes Test-Framework fehlt — nur die neuen Test-Dateien selbst.)*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-------------------|
| V2 Authentication | nein | unverändert — Session/OIDC-Layer nicht betroffen |
| V3 Session Management | nein | unverändert |
| V4 Access Control | **ja** | Der Self-vs-HR-Gate (`verify_user_is_sales_person`, transitiv über
  `find_sales_person_by_user_id`/`get_assigned_user`) ist die **einzige**
  echte Autorisierungsprüfung dieses Milestones, die umgeschrieben wird —
  Standard-Control: DB-seitiger `JOIN` statt applikationsseitigem
  `Option`-Vergleich (siehe Pattern 2) |
| V5 Input Validation | nein | keine neuen Eingabe-Oberflächen (kein REST-Endpoint) |
| V6 Cryptography | nein | keine Krypto-Berührung |

### Known Threat Patterns for {stack}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|----------------------|
| Fail-open durch `None == None` bei `Option<Uuid>`-Vergleich | Elevation of Privilege | SQL-`JOIN` statt Rust-Comparison (D-61-08, Pattern 2) — `NULL = NULL` matcht in SQL nie |
| Stille Identitäts-Umhängung beim Verknüpfen (fremde Historie „erbt" ein anderer Benutzer) | Tampering / Information Disclosure | Explizite Konflikt-Ablehnung (D-61-04, Pattern 3) statt stillem Überschreiben; 409 statt 200 bei Mehrdeutigkeit |
| Race zwischen Konflikt-Check und Merge-Write | Tampering | `rows_affected`-Guard auf dem `UPDATE` als Verteidigung, analog `rebooking_batch.rs` |

## Sources

### Primary (HIGH confidence — dieser Session gelesener Code)

- `dao_impl_sqlite/src/sales_person.rs` — alle sechs betroffenen Methoden,
  vollständig gelesen.
- `dao/src/sales_person.rs` — `SalesPersonDao`-Trait, vollständig gelesen.
- `service_impl/src/sales_person.rs` — vollständig gelesen, insbesondere
  `set_user` (:316-342) und `verify_user_is_sales_person` (:386-406).
- `dao_impl_sqlite/src/lib.rs` — `PermissionDaoImpl::create_user` (:93-104),
  `BasicDaoImpl::clear_all` (:280-299).
- `service/src/lib.rs` — vollständiger `ServiceError`-Enum-Katalog.
- `migrations/sqlite/20260916000000_create-person-backfill.sql` — vollständig
  gelesen (Index-Namen, Backfill-Logik, Selbst-Assertion-Muster).
- `migrations/sqlite/20240506114107_add_sales_person.sql` — Ursprungsschema
  von `sales_person_user`.
- `service_impl/src/test/person_backfill.rs` — vollständig gelesen
  (Test-Harness, Fixture-Muster, Marker-Split-Technik).
- `dao_impl_sqlite/src/rebooking_batch.rs:330-450` — Claim-INSERT/Conditional-
  UPDATE-Konfliktmuster (`is_unique_violation`, `rows_affected`-Guard).
- `rest/src/rebooking.rs`, `rest/src/lib.rs:145-210` — zentrales
  Error-Mapping, 409-Präzedenz.
- `rest/src/sales_person.rs:150-336` — bestehende OpenAPI-Annotationen der
  drei `/{id}/user`-Endpoints.
- `shifty_bin/src/main.rs:995-1060, 1275-1300` — DI-Konstruktionsmuster für
  Basic-Services (`SalesPersonService`, `WeekStatusService`).
- `docs/architecture/02-service-tiers.md`, `docs/features/F12-auth-session.md`
  — aktueller Doku-Stand für das Docs-Freshness-Gate.
- `.planning/phases/61-.../61-CONTEXT.md`, `.planning/phases/60-.../60-CONTEXT.md`,
  `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/STATE.md`,
  `.planning/research/PITFALLS.md`, `.planning/research/ARCHITECTURE.md` —
  vollständig gelesen.

### Secondary (MEDIUM confidence)

- `Cargo.lock` — `libsqlite3-sys 0.30.1` verifiziert (RETURNING-Unterstützung
  daraus abgeleitet, nicht direkt im Repo getestet).

### Tertiary (LOW confidence / per Websearch bestätigt)

- SQLite-`UNIQUE`-Constraint-Timing (immediate, nicht deferred) — per
  Websearch bestätigt (Community-/Doku-Konsens, keine offizielle SQLite-Doku-
  Seite direkt zitiert). Für die vorgeschlagene Design-Entscheidung nicht
  entscheidungsrelevant (kein transienter Konflikt geplant), aber explizit als
  Antwort auf die Forschungsfrage dokumentiert.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|-----------------|
| A1 | Der Merge sollte im DAO statt im Service sitzen, um `SalesPersonService`s Basic-Tier-Klassifikation zu erhalten. | Pattern 3 | Falls der Planer die Logik doch im Service platziert, muss explizit begründet werden, warum das keine Business-Logic-Reklassifikation von `SalesPersonService` auslöst — sonst widerspricht es CLAUDE.md §Service-Tier-Konventionen |
| A2 | `discard_assigned_user` wird bei jedem `set_user(Some(user))`-Aufruf weiterhin zuerst durchlaufen (bestehendes Verhalten), was bedeutet, dass ein Re-Link denselben Split-Pfad wie ein reines Unlink auslösen könnte. | Code Examples → „Bestehender set_user-Service-Pfad" | Falls falsch, könnte ein Re-Link unnötig eine zusätzliche `person`-Zeile erzeugen und wieder löschen — funktional korrekt, aber ineffizient; sollte der Planer explizit adressieren |
| A3 | Die Konflikt-`ServiceError`-Variante sollte `EntityAlreadyExists(Uuid)` wiederverwenden statt eine neue Variante einzuführen. | Code Examples → 409-Mapping | Reine Stil-/Boilerplate-Frage, keine funktionale Konsequenz — beide Optionen erfüllen SC4 |
| A4 | `person.id`/`person.created` sollten per SQL (`randomblob`/`strftime`) statt per injiziertem `UuidService`/`ClockService` generiert werden. | Frage 5 | Falls der Planer stattdessen Service-Layer-Generierung wählt, ändern sich DAO-Methodensignaturen für `create`/`create_user` (zulässig, da nicht in SC3 eingefroren) — reine Implementierungsentscheidung |

## Open Questions

1. **Sequenzierung von `discard_assigned_user` innerhalb eines Re-Links**
   - What we know: Der heutige Service-Pfad ruft bei jedem `set_user(Some(user))`
     zuerst bedingungslos `discard_assigned_user` auf, dann (falls `Some`)
     `assign_to_user`.
   - What's unclear: Ob das im neuen Modell heißt, dass ein Re-Link (Sales
     Person war bereits mit Benutzer A verknüpft, wird jetzt mit Benutzer B
     verknüpft) zuerst den D-61-05-Split-Pfad für Benutzer A durchläuft (neue
     Person für A), bevor der D-61-03-Merge für B beginnt — oder ob das ein
     eigener, direkterer Pfad sein sollte.
   - Recommendation: Im Plan explizit als Testfall („Re-Link: Sales Person von
     Benutzer A auf Benutzer B umhängen") aufnehmen und die gewählte Semantik
     dokumentieren — funktional ist beides denkbar korrekt, aber die
     Redundanz (Split gefolgt von Merge) sollte eine bewusste, keine
     zufällige Entscheidung sein.

2. **`RestStateDef`-Accessor für `PersonService`**
   - What we know: SC5 verlangt nur DI-Konstruktion im Basic-Block, keinen
     REST-Accessor.
   - What's unclear: Ob der Planer trotzdem präventiv einen Accessor anlegt
     (kostet nichts, erleichtert Phase 62), oder bewusst wartet.
   - Recommendation: Claude's Discretion — beide Wege sind SC5-konform.

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — keine neuen Dependencies, alles bereits verifizierter Workspace-Code
- Architecture: HIGH — jedes Muster (Basic-Tier-DI, Konflikt-Erkennung, Error-Mapping) ist wörtlich aus bestehendem Code abgeleitet
- Pitfalls: HIGH — jede Pitfall-Aussage ist entweder direkt am Code verifiziert (Transaktions-Asymmetrie, `last_insert_rowid()`-Falle) oder aus PITFALLS.md übernommen und gegen den aktuellen Code gegengeprüft
- Merge-/Split-Algorithmus (Frage 1/3): MEDIUM-HIGH — der Algorithmus selbst ist neue Synthese ohne 1:1-Repo-Präzedenz (wie von der Roadmap selbst geflaggt: „echte neue Geschäftslogik ... eigene Test-Matrix entwerfen"), aber jede Einzelaussage (Index-Namen, UNIQUE-Timing, Transaktionsverhalten) ist verifiziert

**Research date:** 2026-09-17
**Valid until:** stabil bis zur Ausführung dieser Phase — keine externen, zeitabhängigen Fakten (keine Library-Versionen, die sich kurzfristig ändern könnten)
