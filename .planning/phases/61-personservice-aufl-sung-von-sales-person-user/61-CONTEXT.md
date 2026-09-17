# Phase 61: PersonService + Auflösung von `sales_person_user` - Context

**Gathered:** 2026-09-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Diese Phase zieht alle Lese- und Schreibpfade, die heute über die
Verknüpfungstabelle `sales_person_user` laufen, auf `person_id` um, legt
`PersonDao`/`PersonService` als Basic-Tier-Bausteine an, gibt `person_id` künftig
bei jeder Neuanlage mit, implementiert Verknüpfen/Lösen mit expliziter
Konflikt-Ablehnung und löscht die alte Tabelle per Migration.

**Backend-only.** Kein Endpoint neu, kein TO geändert, kein Frontend-Anteil. Die
öffentlichen Signaturen von `SalesPersonService` bleiben unverändert, die rund 50
Aufrufstellen in `service_impl/` stehen unberührt im Diff — die Auflösung ist per
Konstruktion eine Zwei-Schichten-Änderung (DAO + Migration).

**Sicherheitsrelevanz:** Diese Phase enthält die einzige Umschreibung einer
echten Autorisierungsprüfung im Milestone (`find_sales_person_by_user_id`,
`service_impl/src/sales_person.rs:386-406`). Eine Vorher/Nachher-Permission-Matrix
ist Pflicht-Artefakt, kein Nice-to-have.

**Requirements:** PERS-05

</domain>

<decisions>
## Implementation Decisions

### Person-Vergabe & Verknüpfungs-Semantik

- **D-61-01:** **Jede Sales Person und jeder Benutzer trägt ab dieser Phase immer
  genau eine Person.** `person_id` wird bei jeder Neuanlage mitvergeben — in der
  Login-Auto-Anlage (`dao_impl_sqlite/src/lib.rs:96`,
  `INSERT INTO user (name, update_process)`) und in `create_sales_person`
  (`dao_impl_sqlite/src/sales_person.rs:120-138`). Damit löst Phase 61 den in
  **D-60-10** bewusst offen gelassenen Zustand auf: nach dieser Phase ist
  `person_id` faktisch nie mehr `NULL`. Die Spalten bleiben schema-seitig nullable
  (Phase 60 hat sie so angelegt und SQLite kann sie nicht nachträglich auf
  `NOT NULL` ziehen) — die Invariante wird vom Code getragen, nicht vom Schema.
  — **Reversibility:** reversible-with-effort — die Rückkehr zu „person_id bleibt
  NULL bis zur Verknüpfung" würde bereits vergebene Person-Zeilen wieder
  einsammeln müssen.

- **D-61-02:** **„Unverknüpft" heißt nicht `person_id IS NULL`, sondern: die
  eigene Person trägt die Gegenseite nicht.** Direkte Folge aus D-61-01. Die
  Lesepfade für „noch unverknüpfte Kandidaten" (Roadmap: entstehen in dieser Phase
  als Backend-Vorarbeit für den Picker in Phase 63) werden entsprechend als
  Outer-Join `sales_person` ↔ `person` ↔ `user` formuliert, **nicht** als
  `WHERE person_id IS NULL`. Eine `NULL`-basierte Definition würde Bestandsdaten
  (alle mit Person aus dem Backfill) und Neuanlagen unterschiedlich behandeln.

- **D-61-03:** **Verknüpfen ist ein Merge zweier Personen, kein Anlegen.**
  `set_user(sales_person_id, Some(user))` zieht beide Seiten auf **eine** Person;
  die absorbierte `person`-Zeile wird gelöscht. Sicher, weil `person` außer
  Identität und Audit-Spalten nichts trägt (**D-60-04**) und nach dem Merge von
  nichts mehr referenziert wird — die zwölf Tabellen mit Sales-Person-Bezug
  hängen an `sales_person.id`, die write-once-Abrechnungs-Snapshots an
  `billing_period_sales_person.sales_person_id`, beide unverändert.

  Beobachtbar erfüllt das Success Criterion 4 („Verknüpfen zweier noch
  unverknüpfter Objekte legt genau eine gemeinsame Person an") — nur der
  Mechanismus ist Verschmelzen statt Neuanlegen. Der Ergebniszustand ist identisch:
  genau eine gemeinsame Person.

- **D-61-04:** **409 nur bei echter Mehrdeutigkeit.** Der Konflikt-Fehler greift
  genau dann, wenn die verschmolzene Person danach mehr als einen Benutzer oder
  mehr als eine Sales Person trüge — praktisch: der Ziel-Benutzer hängt bereits an
  einer **anderen** Sales Person, oder die Sales Person hängt bereits an einem
  **anderen** Benutzer. Komplementäres Verschmelzen (eine Person trägt nur die
  Sales Person, die andere nur den Benutzer) ist **erlaubt**.

  Begründung (User-Entscheidung 2026-09-17): Genau das funktioniert heute. Der
  heutige `set_user`-Pfad (`service_impl/src/sales_person.rs:316-342`) löscht die
  alte `sales_person_user`-Zeile und legt die neue an; bei freiem Benutzer geht das
  durch, bei belegtem Benutzer schlägt der `UNIQUE(user_id)`-Constraint zu und der
  Aufrufer sieht einen 500er. D-61-04 erhält den funktionierenden Fall **exakt** und
  ersetzt den 500er durch einen sauberen, unterscheidbaren 409.

  Die Alternative — 409 immer, sobald beide Seiten schon eine Person haben — wurde
  **verworfen**: sie wäre ein Funktionsverlust gegenüber heute (nach dem Backfill
  hat jedes Bestandsobjekt eine Person, also ließe sich nichts Bestehendes je wieder
  verknüpfen) und machte **PADM-04** („zwei bereits getrennt bestehende Objekte
  können ausnahmsweise nachträglich verknüpft werden", Phase 63) für alle
  Bestandsdaten unmöglich.
  — **Reversibility:** reversible — reine Service-Logik, keine Schema-Wirkung.

- **D-61-05:** **Beim Lösen behält die Sales Person die bestehende Person, der
  Benutzer bekommt eine frisch angelegte.** `set_user(sales_person_id, None)`
  trennt heute nur die Verknüpfungszeile und lässt beide Objekte stehen; im
  Person-Modell muss eine Seite eine neue Identität bekommen. Die Sales Person ist
  die Seite mit historischem Gewicht (Buchungen, `extra_hours`,
  write-once-Abrechnungs-Snapshots hängen alle an `sales_person.id`) — ihre
  Person-Identität bleibt deshalb stabil.

  Folge: **Pitfall 5, Fall 1 („unlink to zero" → verwaiste `person`-Zeile ohne
  Träger) kann strukturell nicht eintreten.** Jede Person hat nach jeder Operation
  mindestens einen Träger. Das nimmt Phase 62 die dort erwartete Produktfrage nach
  Ghost-Einträgen ab.

### Auflösung von `sales_person_user`

- **D-61-06:** **Die DROP-Migration enthält einen Nachzieh-Backfill und eine
  Selbst-Assertion**, bevor sie die Tabelle löscht: jede noch existierende
  `sales_person_user`-Zeile muss nach dem Nachzug ein `person_id`-Gegenstück auf
  beiden Seiten haben, sonst bricht die Migration ab (Form analog **D-60-07**:
  Wegwerf-Tabelle mit `CHECK`-Constraint, am Ende gedroppt).

  Hintergrund: Zeilen, die zwischen dem Phase-60- und dem Phase-61-Deploy
  entstehen, hätten laut D-60-10 noch kein `person_id`; ihre Verknüpfung lebte
  ausschließlich in `sales_person_user` und ginge beim DROP still verloren.

  **Phase 60 ist noch nicht produktiv deployt** (User, 2026-09-17) — 60 und 61
  gehen zusammen live, der Zwischenzustand tritt in Produktion nie auf. Der Guard
  bleibt trotzdem drin (billig, und die Entwicklungs-Datenbanken haben den Zustand
  real durchlaufen), aber er bekommt **keine eigene Fixture-Testmatrix**. Ein Test,
  der den Zwischenzustand herstellt und nachweist, dass der Nachzug greift, genügt;
  eine Negativ-Matrix über alle Ausprägungen ist nicht nötig.

- **D-61-07:** **`sales_person_user` wird in dieser Phase per Migration gedroppt**
  (Success Criterion 2). Rollback-Pfad ist das Datei-Backup der SQLite-Datei vor
  dem Deploy, nicht eine Down-Migration — dieselbe Zusage wie in **D-60-09**.

### Autorisierungspfad

- **D-61-08:** **Der Self-Gate-Vergleich läuft als ein einziger SQL-Join über
  `person_id`, nie als zwei separat in Rust geladene `Option<Uuid>`.** Betrifft
  `find_sales_person_by_user_id` und `find_by_user`
  (`dao_impl_sqlite/src/sales_person.rs:102-119` und `:226-243`, beide heute
  `JOIN sales_person_user`). Grund: ein Rust-seitiger `Option<Uuid>`-Vergleich
  matcht `None == None` und öffnet damit fail-open (Roadmap-Constraint 1,
  Pitfall 4). SQL matcht `NULL = NULL` nie — der Join ist die sichere Form.

- **D-61-09:** **Die Vorher/Nachher-Permission-Matrix ist ein Pflicht-Artefakt**
  mit den drei Fällen aus Success Criterion 1 — „self darf", „fremder
  Sales-Person-Datensatz darf nicht", „Benutzer ohne Verknüpfung darf nicht" —
  jeweils vor und nach dem Schema-Swap, mit byte-identischem Pass/Fail-Ergebnis.
  Nicht als Prosa im SUMMARY, sondern als Test, der beide Seiten des Swaps belegt.

### Gesperrt — in dieser Phase nicht neu aufmachen

- `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**. `person` ist rein
  administrativ; keine der vier Bump-Klauseln aus `CLAUDE.md` greift. Phase 61
  ändert keinen persistierten `value_type`, keine Computation, kein Input-Set.
  **Nicht „vorsichtshalber" bumpen.**
- `sales_person.id` und `user.name` werden nicht umgeschlüsselt.
- `person` bekommt keine Namensspalten und keine `deleted`-Spalte (D-60-04,
  PERS-F-01 ist Future).
- Der Index auf `sales_person.person_id` bleibt **vollständig**, nicht partiell —
  kein `WHERE deleted IS NULL` nachrüsten (**D-60-01**, one-way).
- HR-/Admin-Accounts ohne Sales Person und Freiwillige ohne Login bleiben
  gleichwertige Ausprägungen (**D-60-03**). **Nirgends automatisch beides
  anlegen** — D-61-01 vergibt eine *Person*, niemals zusätzlich einen Benutzer
  oder eine Sales Person.
- `PersonService` ist **Basic-Tier**: nur `PersonDao` + `PermissionService` +
  Clock/Uuid + `TransactionDao`, **kein** anderer Domain-Service. DI-Konstruktion
  in `shifty_bin/src/main.rs` im Basic-Block, strikt vor der
  Business-Logic-Schicht (Roadmap-Constraint 5, `CLAUDE.md` Tier-Regel).
- Die öffentlichen Signaturen von `SalesPersonService` (`set_user`,
  `get_assigned_user`, `get_all_user_assignments`, `get_sales_person_for_user`,
  `get_sales_person_current_user`, `verify_user_is_sales_person`) bleiben
  **unverändert**. Kein Endpoint, kein TO, kein Frontend-Anteil.

### Claude's Discretion

- **`ServiceError`-Variante für den Verknüpfungs-Konflikt** und ihr HTTP-Mapping
  auf 409. Einzige harte Vorgabe: sie muss von `ServiceError::EntityConflicts`
  (heute der Versions-Konflikt) unterscheidbar sein — Success Criterion 4 verlangt
  einen „unterscheidbaren Konflikt-Fehler". Präzedenz für sauberes 409-Mapping:
  `rest/src/rebooking.rs:14-18` (zentraler Error-Handler, kein Handler-lokales
  Status-Mapping).
- **API-Zuschnitt von `PersonDao` und `PersonService`** — genau so groß, wie
  Phase 61 plus die in der Roadmap zugesagten „unverknüpfte Kandidaten"-Lesepfade
  es verlangen. Keine Vorratshaltung für Phase 62/63.
- **Wo Merge- und Konflikt-Logik sitzen** (DAO-Ebene vs. `PersonService`) —
  freigestellt, solange die `SalesPersonService`-Signaturen und die rund 50
  Aufrufstellen unberührt bleiben (SC3) und `PersonService` Basic-Tier bleibt (SC5).
- Migrations-Dateiname und Zeitstempel-Präfix, Index-Namen, konkrete Form der
  Selbst-Assertion aus D-61-06, Ablage der Tests.
- Ob `BasicDaoImpl::clear_all` (`dao_impl_sqlite/src/lib.rs:282-297`) zusätzlich
  ein `DELETE FROM person` bekommt, nachdem die `sales_person_user`-Zeile dort
  entfällt.
- Wellen-Schnitt und Plan-Anzahl.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Gesperrte Entscheidungen & Scope
- `.planning/ROADMAP.md` §„Milestone v2.7" → „Gesperrte Entscheidungen",
  „Harte Constraints aus der Research" (Punkte **1**, **5** und **7** betreffen
  Phase 61), „Docs-Freshness-Gate" (Zeile Phase 61), „Phase Details → Phase 61".
- `.planning/REQUIREMENTS.md` — PERS-05; außerdem **PADM-04** (Phase 63), dessen
  Machbarkeit direkt an D-61-04 hängt.
- `.planning/seeds/personenverwaltung-zusammenlegen.md` — Variante A
  (Elterntabelle + nullable FKs), die drei Ausprägungen, „Namen bleiben wo sie
  sind", die No-Bump-Begründung. Nicht neu aufmachen.
- `.planning/phases/60-person-datenmodell-backfill/60-CONTEXT.md` — **D-60-01**
  (vollständiger Index, one-way), **D-60-03** (drei gleichwertige Ausprägungen),
  **D-60-04** (`person` ohne `deleted`), **D-60-07** (Selbst-Assertion als
  Migrations-Muster), **D-60-09** (Datei-Backup statt Down-Migration), **D-60-10**
  (Vergabe beim Anlegen ist explizit Phase-61-Scope).

### Fallen mit direktem Phase-61-Bezug
- `.planning/research/PITFALLS.md` §Pitfall 4 — die Autorisierungsprüfung, die
  durch die aufgelöste Tabelle joint; enthält die fail-open-Analyse und die
  Vorgabe „Join im SQL, nie zwei `Option<Uuid>` in Rust".
- `.planning/research/PITFALLS.md` §Pitfall 5 — Link/Unlink erzeugt verwaiste
  oder umgehängte `Person`-Zeilen; Fall 1 wird durch D-61-05 strukturell
  ausgeschlossen, Fall 2 durch D-61-04 beantwortet.
- `.planning/research/PITFALLS.md` §„Security Mistakes" (Zeilen zu Pitfall 4 und
  5) und §„Looks Done But Isn't".
- `.planning/research/ARCHITECTURE.md` — Build-Order, Tier-Klassifikation,
  Blast Radius mit `file:line`.

### Bestehender Code, der umgeschrieben wird
- `dao_impl_sqlite/src/sales_person.rs` — die sechs Methoden mit
  `sales_person_user`-Zugriff: `find_by_user` (`:102-119`), `assign_to_user` (`:159-173`),
  `discard_assigned_user` (`:174-189`), `get_assigned_user` (`:190-205`),
  `get_all_user_assignments` (`:206-225`), `find_sales_person_by_user_id`
  (`:226-243`).
- `dao_impl_sqlite/src/lib.rs:286` — `DELETE FROM sales_person_user` in
  `BasicDaoImpl::clear_all`.
- `service_impl/src/sales_person.rs:316-342` — heutiger `set_user`-Pfad
  (`discard` + `assign`, HR-Gate).
- `service_impl/src/sales_person.rs:386-406` — `verify_user_is_sales_person`, das
  Self-vs-HR-Gate.
- `dao/src/sales_person.rs` — `SalesPersonDao`-Trait; die sechs betroffenen
  Methodensignaturen bleiben unverändert.
- `service/src/sales_person.rs` — `SalesPersonService`-Trait; eingefroren (SC3).

### Bestehendes Schema
- `migrations/sqlite/20240506114107_add_sales_person.sql:14-22` —
  `sales_person_user` mit den beiden `UNIQUE`-Constraints, die in Phase 60 als
  `UNIQUE(person_id)` wiederhergestellt wurden.
- `migrations/sqlite/20260916000000_create-person-backfill.sql` — die
  Phase-60-Migration: `person`, beide `person_id`-Spalten, Backfill,
  Selbst-Assertion.

### Konventionen & Doku-Gate
- `CLAUDE.md` §„Service-Tier-Konventionen: Basic vs. Business-Logic Services",
  §„Database Migrations", §„Billing Period Snapshot Schema Versioning",
  §„Docs-Freshness-Gate".
- Docs-Trigger dieser Phase (EN **und** DE im selben Commit):
  `docs/architecture/02-service-tiers.md` +
  `docs/architecture/diagrams/service-graph-runtime.mmd` (neuer Basic-Service in
  der DI), `docs/architecture/04-auth.md` + `docs/features/F12-auth-session.md`
  (umgeschriebener Self-Gate-Pfad). Zusätzlich fällig, weil Trigger-Dateien
  berührt werden: `docs/architecture/03-data-model.md` +
  `docs/architecture/diagrams/db-schema-er.mmd` (Tabelle verschwindet) und die
  passende `docs/features/F*.md` zur Mitarbeiterverwaltung.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `service_impl/src/test/person_backfill.rs` — die Phase-60-Testinfrastruktur:
  In-Memory-SQLite mit `sqlx::migrate!`, rohe SQL-Fixtures für `sales_person`,
  `user` und `sales_person_user`, `stored_ddl()`-Helfer zum Prüfen von
  Tabellendefinitionen. Direktes Zuhause für den Nachzieh-Backfill-Test (D-61-06)
  und den DROP-Nachweis (SC2).
- `service_impl/src/test/rebooking_batch_state.rs:45` — Präzedenz für
  Integrationstests gegen eine echte In-Memory-SQLite.
- `rest/src/rebooking.rs:14-18` — zentrales Status-Mapping (403/409/422) über den
  Error-Handler statt Handler-lokal; Muster für die neue Konflikt-Variante.
- `migrations/sqlite/20260707000001_add-source-column-to-extra-hours.sql` — Muster
  für `ALTER TABLE` mit Begründungskommentar im Kopf (Phase + Decision-ID).

### Established Patterns
- **DAO-Queries sind `sqlx::query!`/`query_as!`-Makros** mit
  Compile-Time-Checking. Nach der DROP-Migration muss die lokale Dev-DB aktuell
  sein, sonst schlagen die Makros fehl — und umgekehrt: eine stehen gebliebene
  Query auf `sales_person_user` fällt erst beim Re-Check gegen eine migrierte DB
  auf, nicht als gewöhnlicher Rust-Compile-Fehler (Pitfall 4, „Warning signs").
- **`gen_service_impl!`-Makro** für DI; Services nehmen `Option<Transaction>` und
  committen selbst (`CLAUDE.md` §Transaction Management).
- **Soft-Delete-Filter `WHERE deleted IS NULL`** ist Konvention für Live-Queries.
  Für die Person-Joins gilt sie differenziert: eine soft-gelöschte Sales Person
  behält ihre Person (D-60-01), taucht aber in Live-Listen nicht auf.
- **Migrationen laufen je Datei in einer Transaktion** — Nachzieh-Backfill,
  Assertion und DROP gehören in **eine** Datei, nicht auf mehrere verteilt, die je
  halb greifen könnten.

### Integration Points
- `shifty_bin/src/main.rs` — DI-Verdrahtung; `PersonDao`/`PersonService` kommen in
  den Basic-Block, strikt vor der Business-Logic-Schicht. Trigger für
  `docs/architecture/02-service-tiers.md` (+ `_de`).
- `shifty_bin/src/main.rs:1651` und `shifty_bin/src/integration_test.rs:283` — die
  beiden `sqlx::migrate!`-Aufrufe, die die neue Migration mitziehen.
- `dao_impl_sqlite/src/lib.rs:96` (Login-Auto-Anlage) und
  `dao_impl_sqlite/src/sales_person.rs:120-138` (`create`) — die beiden Stellen,
  die ab jetzt `person_id` mitvergeben (D-61-01).
- `rest/src/sales_person.rs:27-29, 247-330` — die drei
  `/{id}/user`-Endpoints (`get`/`post`/`delete`). Verträge bleiben unverändert;
  nur der neue 409-Pfad kommt in der OpenAPI-Annotation dazu (`:170` zeigt, dass
  409 dort bereits dokumentiert ist).

</code_context>

<specifics>
## Specific Ideas

- **Der Maßstab dieser Phase ist Nicht-Regression, nicht neue Funktion.** Der
  Nachweis ist die Vorher/Nachher-Matrix (D-61-09), nicht ein „es kompiliert und
  die Tests sind grün". Success Criterion 3 macht das explizit prüfbar: die rund
  50 Aufrufstellen in `service_impl/` müssen **unberührt im Diff stehen** — ein
  Diff, der sie anfasst, hat die Zwei-Schichten-Zusage gebrochen.
- **Phase 60 und 61 gehen zusammen live** (User, 2026-09-17). Der Zwischenzustand
  „Schema da, Code liest noch über die alte Tabelle" existiert nur in
  Entwicklungs-Datenbanken.
- **Heute geht das Verknüpfen zweier freier Objekte durch, das eines belegten
  Benutzers gibt einen 500er.** Diese Ausgangslage ist der Maßstab für D-61-04:
  der erste Fall muss weiter funktionieren, der zweite wird ein 409.
- **Der Scout-Befund bestätigt den engen Zuschnitt:** `sales_person_user` kommt in
  Rust nur in zwei Dateien vor (`dao_impl_sqlite/src/sales_person.rs`,
  `dao_impl_sqlite/src/lib.rs`), in den Migrationen in zweien, plus Erwähnungen in
  den Phase-60-Tests. Success Criterion 2 („grep findet den Namen nur noch in
  historischen Migrationen") ist damit erreichbar, ohne den Test-Code zu
  verbiegen — Kommentare und Fixtures in `person_backfill.rs` beschreiben den
  Zustand *vor* dem Drop und sind Teil der Historie, nicht der laufenden
  Wahrheit. Der Planer entscheidet, ob diese Tests mitziehen oder als
  Migrations-Regressionstests bestehen bleiben.

</specifics>

<deferred>
## Deferred Ideas

- **Zusammenführen zweier Personen im UI** (zurückkehrender Mitarbeiter, der als
  zweiter Eintrag angelegt wurde) — die Merge-Mechanik entsteht in dieser Phase
  (D-61-03), eine Bedienoberfläche dafür nicht. Kein v2.7-Scope; Kandidat für die
  Personenverwaltung, falls der Bedarf auftritt.
- **Kandidaten-Picker-UI** für PADM-04 — die Lesepfade entstehen hier (D-61-02),
  die Oberfläche ist Phase 63.
- **Person löschen** inkl. `deleted`-Spalte auf `person` — frühestens Phase 63.
- **Echte Personenstammdaten** (E-Mail, Telefon, Geburtsdatum) an `person` —
  PERS-F-01, Future.
- **`person_id` schema-seitig auf `NOT NULL` ziehen** — SQLite kann das nicht per
  `ALTER TABLE`; es bräuchte ein Table-Rebuild. Die Invariante aus D-61-01 wird
  vorerst vom Code getragen. Kein v2.7-Scope.
- **`FOREIGN KEY`-Deklaration auf den `person_id`-Kanten** — offene Frage aus
  Phase 60 (der Korrektur-Vermerk zu D-60-06 hat geklärt, dass FK-Enforcement
  aktiv *ist*, aber `person_id` trägt keine Deklaration). Nicht Phase-61-Scope.

### Reviewed Todos (not folded)

- **„admin-Rolle automatisch alle Privilegien (Wildcard oder Trigger)"**
  (`.planning/todos/pending/2026-05-08-admin-rolle-bekommt-alle-privilegien.md`) —
  Keyword-Treffer über „auth"/„user"; betrifft das Rollen-/Privilegien-Modell,
  nicht die Person-Verknüpfung.
- **„Cutover-UI als Admin-Feature"**
  (`.planning/todos/pending/2026-05-08-cutover-ui-admin-feature.md`) — Frontend-Feature
  ohne Bezug zu dieser Backend-Phase.
- **„Schmale Admin-Rolle mit User-Impersonation (,act as')"**
  (`.planning/todos/pending/2026-07-03-schmale-admin-rolle-mit-user-impersonation.md`) —
  berührt den Auth-Kontext, würde aber genau das Self-Gate verändern, dessen
  Nicht-Regression diese Phase nachweisen muss. Bewusst getrennt halten.

</deferred>

---

*Phase: 61-PersonService + Auflösung von `sales_person_user`*
*Context gathered: 2026-09-17*
