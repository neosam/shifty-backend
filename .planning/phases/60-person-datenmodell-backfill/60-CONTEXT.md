# Phase 60: Person-Datenmodell + Backfill - Context

**Gathered:** 2026-09-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Diese Phase legt die Tabelle `person` an, hängt an `sales_person` und `user` je
eine nullable `person_id`-Spalte, führt einen idempotenten SQL-Backfill aus
(inklusive soft-gelöschter Sales Persons) und stellt die heute von
`sales_person_user` getragene 1:1-Garantie als DB-Constraint wieder her.

**Backend-only.** Kein Endpoint, kein TO, kein Frontend-Anteil, keine sichtbare
Verhaltensänderung. `sales_person_user` wird **nicht** gelöscht — die Tabelle
bleibt die laufende Wahrheit, gegen die der Backfill geprüft wird; ihre
Auflösung ist Phase 61.

**Requirements:** PERS-01, PERS-02, PERS-03, PERS-04, PERS-06

</domain>

<decisions>
## Implementation Decisions

### Identität & Soft-Delete-Semantik

- **D-60-01:** Eine soft-gelöschte Sales Person **behält** ihre `person_id`
  dauerhaft. Der eindeutige Index auf `sales_person.person_id` ist deshalb
  **vollständig**, nicht partiell (**kein** `WHERE deleted IS NULL`).
  Begründung (User-Entscheidung 2026-09-16): Gäbe die Löschung die Person frei,
  könnten zwei verschiedene Menschen über die Zeit dieselbe Person-Identität
  belegen, und write-once-`billing_period_sales_person`-Snapshots von vor Jahren
  würden auf die falsche Person auflösen. Bewusst in Kauf genommener Preis: ein
  zurückkehrender Mitarbeiter ist zunächst zwei Personen; ein manuelles
  Zusammenführen ist Future-Scope, nicht v2.7.
  — **Reversibility:** one-way — die Umstellung auf einen partiellen Index würde
  nach Produktiv-Betrieb bereits vergebene Identitäten neu binden und damit
  historische Abrechnungs-Snapshots neu zuordnen.
  Hinweis für die Planung: das ist genau die von `PITFALLS.md` Pitfall 3
  geforderte explizite Produktentscheidung — sie gehört verbatim in den Plan und
  als Kommentar in die Migrationsdatei, damit ein späterer „Aufräum"-Pass den im
  Repo sonst üblichen `WHERE deleted IS NULL`-Zusatz nicht nachrüstet.

- **D-60-02:** `user.person_id` bekommt einen vollständigen eindeutigen Index.
  `user` hat keine Soft-Delete-Spalte, die Frage stellt sich dort nicht.

- **D-60-03:** Beide Spalten bleiben **nullable**, und das ist eine fachliche
  Zusage, kein Übergangszustand: eine Person darf einen Benutzer **oder** eine
  Sales Person **oder** beides haben. Reine HR-/Admin-Accounts ohne Sales Person
  bleiben ausdrücklich erlaubt und sollen weiterhin anlegbar sein
  (User-Bestätigung 2026-09-16); ebenso Freiwillige ohne Login. Der eindeutige
  Index verbietet ausschließlich **zwei** Träger derselben Seite an einer Person
  — „null Träger" ist der Normalfall zweier der drei Ausprägungen. Es darf
  **nirgends** automatisch beides angelegt werden.

### Schema-Zuschnitt

- **D-60-04:** `person` trägt ausschließlich Identität und die üblichen
  Änderungs-Spalten. **Keine** `deleted`-Spalte — in v2.7 wird keine Person
  gelöscht, und eine ungenutzte Soft-Delete-Spalte lädt zu einer halb
  implementierten Semantik ein. Falls Phase 63 ein „Person löschen" braucht,
  kommt die Spalte dort dazu. Die konkrete Spaltenwahl (`created` +
  `update_process` + `update_version` nach dem Muster von `rebooking_batch`, vs.
  `update_timestamp`-Trio nach dem Muster von `sales_person`) ist
  Planer-Ermessen — siehe „Claude's Discretion".

- **D-60-05:** Die 1:1-Garantie kommt als `CREATE UNIQUE INDEX`, nicht als
  Spalten-Constraint. Technischer Zwang: SQLite erlaubt bei
  `ALTER TABLE … ADD COLUMN` kein `UNIQUE`. Die Garantie muss **von der
  Datenbank** durchgesetzt werden (PERS-04, Success Criterion 4) — eine
  Applikationsprüfung genügt nicht.

- **D-60-06:** Der DB-Level-Ablehnungstest läuft über den eindeutigen Index,
  **nicht** über Fremdschlüssel. Scout-Befund: `PRAGMA foreign_keys` wird in
  diesem Repo nirgends gesetzt, SQLite hat FK-Enforcement per Default aus — die
  `FOREIGN KEY`-Deklarationen sind damit dekorativ. Der Plan muss den Test
  entsprechend gegen den Index schreiben, und die Doku hält die Einschränkung
  fest.

### Migrations-Robustheit

- **D-60-07:** Die Migration **verifiziert sich selbst** und bricht bei
  Abweichung ab. Die Zählprobe aus Success Criterion 1
  (`COUNT(person) == COUNT(sales_person) + COUNT(user) − COUNT(sales_person_user)`)
  läuft als harte Assertion **innerhalb** der Migration, nicht nur im Test.
  Begründung: Produktion ist eine einzelne laufende SQLite-Datei; ein still
  halb ausgeführter Backfill wäre erst Monate später sichtbar. Lieber startet der
  Server nicht. Umsetzungshinweis: SQLite kennt `RAISE()` nur in Triggern — ein
  Wegwerf-Tabellchen mit `CHECK`-Constraint, das am Ende der Migration wieder
  gedroppt wird, bricht die Migrations-Transaktion sauber ab. Der Planer wählt
  die konkrete Form.

- **D-60-08:** Der Apply-twice-Nachweis (PERS-02, Success Criterion 2) ist ein
  echter Test: die Migrationsdatei wird nach `sqlx::migrate!` ein **zweites Mal**
  gegen dieselbe Fixture-DB ausgeführt, danach werden Zeilenzahlen **und**
  Zuordnungen vorher/nachher verglichen. „Läuft ohne Fehler durch" reicht nicht.
  Folge für die Migrationsdatei: sie muss als Ganzes re-executable sein — jedes
  `INSERT` mit `WHERE NOT EXISTS`-Guard auf dem Legacy-PK (`sales_person.id` /
  `user.name`), jedes `UPDATE … SET person_id` mit `WHERE person_id IS NULL`.

- **D-60-09:** **Keine** Backup-Tabelle in der Datenbank. Ein Datei-Backup der
  SQLite-Datei vor dem Deploy genügt (User-Entscheidung 2026-09-16). Begründung:
  die Migration fügt nur Spalten hinzu und überschreibt keine bestehenden Daten.
  Die v1.0-Präzedenz
  (`20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql`) wird
  hier bewusst **nicht** wiederholt — dort wurden Daten umgerechnet, hier nicht.

### Zeitliche Abgrenzung

- **D-60-10:** Zeilen, die **nach** der Migration entstehen (Login-Auto-Anlage in
  `dao_impl_sqlite/src/lib.rs:96`, `create_sales_person` in
  `dao_impl_sqlite/src/sales_person.rs:133`), bekommen in Phase 60 noch **kein**
  `person_id`. Die Vergabe beim Anlegen ist Phase-61-Scope. Kein DB-Trigger, kein
  DAO-Eingriff in dieser Phase — Phase 60 bleibt reine Schema-/Backfill-Phase.
  Risiko ist null, weil bis Phase 61 nichts über `person_id` liest. Success
  Criterion 1 gilt für den Zustand **unmittelbar nach** der Migration.

### Gesperrt — in dieser Phase nicht neu aufmachen

- `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**. `person` ist rein
  administrativ: kein neuer persistierter `value_type`, keine geänderte
  Computation, kein geändertes Input-Set,
  `billing_period_sales_person.sales_person_id` unverändert. Keine der vier
  Bump-Klauseln aus `CLAUDE.md` greift. **Nicht „vorsichtshalber" bumpen.**
- `sales_person.id` und `user.name` werden nicht umgeschlüsselt.
- `person` bekommt **keine** Namensspalten (PERS-F-01, Future).
- Backfill-Mechanik ist SQL-only mit `randomblob(16)`; die v1.0-Variante
  „re-runnbarer Rust-Service" wird hier nicht gebraucht, weil die beidseitigen
  `UNIQUE`-Constraints von `sales_person_user` jede Mehrdeutigkeit ausschließen.
- `sales_person_user` bleibt stehen.

### Claude's Discretion

- Exakter Spaltensatz der Audit-Spalten auf `person` (Muster `rebooking_batch`
  vs. Muster `sales_person`) — der Planer wählt das im Repo dominantere Muster
  für neu angelegte Tabellen.
- Dateiname und Zeitstempel-Präfix der Migration.
- Konkrete Form der Selbst-Assertion aus D-60-07.
- Ablage des Apply-twice-Tests (`service_impl/src/test/` analog
  `rebooking_batch_state.rs` vs. `dao_impl_sqlite`).
- Index-Namen.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Gesperrte Entscheidungen & Scope
- `.planning/seeds/personenverwaltung-zusammenlegen.md` — Variante A
  (Elterntabelle + nullable FKs), die drei Ausprägungen, „Namen bleiben wo sie
  sind", die No-Bump-Begründung. Nicht neu aufmachen.
- `.planning/ROADMAP.md` §„Milestone v2.7" → „Gesperrte Entscheidungen",
  „Harte Constraints aus der Research" (Punkte 2 und 7 betreffen Phase 60),
  „Phase Details → Phase 60".
- `.planning/REQUIREMENTS.md` — PERS-01, PERS-02, PERS-03, PERS-04, PERS-06.

### Fallen mit direktem Phase-60-Bezug
- `.planning/research/PITFALLS.md` §Pitfall 1 — Idempotenz des Backfills.
- `.planning/research/PITFALLS.md` §Pitfall 2 — soft-gelöschte Sales Persons
  dürfen nicht durch den gewohnten `WHERE deleted IS NULL`-Reflex herausfallen.
- `.planning/research/PITFALLS.md` §Pitfall 3 — die 1:1-Invariante verschwindet
  mit der Tabelle; enthält die Soft-Delete-Produktfrage, die D-60-01 beantwortet.
- `.planning/research/ARCHITECTURE.md` — Build-Order und Blast Radius.

### Bestehendes Schema
- `migrations/sqlite/20240506114107_add_sales_person.sql:1-22` — `sales_person`
  (mit `deleted`) und `sales_person_user` mit den beiden `UNIQUE`-Constraints,
  die wiederhergestellt werden müssen.
- `migrations/sqlite/20240426150045_user-roles.sql` — `user` ohne
  Soft-Delete-Spalte; `user_role`, `user_invitation` hängen an `user.name`.

### Konventionen & Doku-Gate
- `CLAUDE.md` §„Database Migrations", §„Billing Period Snapshot Schema
  Versioning", §„Docs-Freshness-Gate".
- Docs-Trigger dieser Phase (EN **und** DE im selben Commit):
  `docs/architecture/03-data-model.md`,
  `docs/architecture/diagrams/db-schema-er.mmd`, passende `docs/features/F*.md`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `migrations/sqlite/20260707000000_create-rebooking-batch.sql` — jüngste
  `CREATE TABLE` + `CREATE UNIQUE INDEX`-Migration; zeigt den Spaltenstil
  (`created`/`deleted`/`update_process`/`update_version`) und die
  Kommentar-Konvention (Phase + Decision-ID im Kopf).
- `migrations/sqlite/20260707000001_add-source-column-to-extra-hours.sql` —
  Muster für `ALTER TABLE … ADD COLUMN` inklusive Begründungskommentar.
- `migrations/sqlite/20260503000001_create-absence-period-migration-source.sql:2`
  — die Idempotenz-Konvention dieses Repos, wörtlich: „Idempotency key:
  extra_hours_id PK. Re-run skips already-mapped rows."
- `service_impl/src/test/rebooking_batch_state.rs:45` — Präzedenz für
  Integrationstests gegen eine echte In-Memory-SQLite mit `sqlx::migrate!`.
  Passendes Zuhause für Backfill-, Apply-twice- und Double-Link-Tests.
- `service_impl/src/test/absence_conversion.rs:564` — zeigt, wie eine
  `sales_person`-Fixture per rohem SQL angelegt wird (nützlich für die
  soft-gelöschte Fixture aus Success Criterion 3).
- `migrations/sqlite/20240426150045_user-roles.sql` enthält bereits
  `randomblob`-Nutzung im `V_UUID_V4`-View — die ID-Erzeugung im Backfill hat
  Präzedenz.

### Established Patterns
- **Partieller Unique-Index `WHERE deleted IS NULL`** ist das dominante Muster
  (`rebooking_batch_week_unique_idx`). Phase 60 weicht davon bewusst ab
  (D-60-01) — das braucht einen erklärenden Kommentar in der Migration, sonst
  liest es sich wie ein Versehen.
- **Soft-Delete-Filter in DAOs** (`WHERE deleted IS NULL`) ist laut `CLAUDE.md`
  Konvention für **Live-Queries**. Für den Backfill gilt sie ausdrücklich
  **nicht** (PERS-03).
- Migrationen laufen je Datei in einer Transaktion. Der gesamte Backfill gehört
  in **eine** Datei, nicht auf mehrere verteilt, die je halb greifen könnten.

### Integration Points
- `dao_impl_sqlite/src/lib.rs:96` — `INSERT INTO user (name, update_process)`,
  die automatische Benutzer-Anlage beim Login. Schreibt in Phase 60 weiterhin
  kein `person_id` (D-60-10).
- `dao_impl_sqlite/src/sales_person.rs:133` — `INSERT INTO sales_person (…)`.
  Dito.
- `dao/src/sales_person.rs:71` / `service_impl/src/sales_person.rs:356-401` —
  `find_sales_person_by_user_id`, die Selbst-Zugriffs-Prüfung. In Phase 60
  **unberührt**; ihre Umschreibung ist Phase-61-Scope und braucht dort eine
  eigene Vorher/Nachher-Permission-Matrix.
- `shifty_bin/src/main.rs:1651` und `shifty_bin/src/integration_test.rs:283` —
  die beiden `sqlx::migrate!`-Aufrufe, die die neue Migration mitziehen.
- SQLx Compile-Time-Checking: nach der Migration muss die lokale Dev-DB
  aktualisiert sein, sonst schlagen `query!`-Makros fehl.

</code_context>

<specifics>
## Specific Ideas

- **HR-/Admin-Accounts ohne Sales Person sind ein dauerhaft unterstützter
  Normalfall**, kein Altlast-Zustand (User, 2026-09-16, wörtlich sinngemäß: „es
  gibt HR Benutzer und die soll es auch weiterhin geben … Benutzer oder Sales
  Person bei einer Person soll optional sein"). Jede Modellierung, jeder Test und
  jede spätere UI muss das als gleichwertige Ausprägung behandeln. Insbesondere:
  **nirgends automatisch beides anlegen.**
- Produktions-Datenlage: exakte Zahlen für HR-Accounts, Freiwillige ohne Login
  und soft-gelöschte Sales Persons liegen nicht vor. Der Plan darf sich deshalb
  auf **keine** Mengenannahme stützen; alle drei Fälle brauchen eine eigene
  Fixture, und die Zählprobe aus D-60-07 ist die Absicherung gegen die
  unbekannte reale Verteilung.
- Der Betreiber zieht vor dem Deploy ein Datei-Backup der SQLite-Datei. Der
  Rollback-Pfad ist damit „Datei zurückspielen", nicht „Down-Migration".

</specifics>

<deferred>
## Deferred Ideas

- **Zusammenführen zweier Personen** (zurückkehrender Mitarbeiter, der als
  zweiter Eintrag angelegt wurde) — direkte Folge von D-60-01. Kein v2.7-Scope;
  Kandidat für die Personenverwaltung, falls der Bedarf je auftritt.
- **Person löschen** inkl. `deleted`-Spalte auf `person` — kommt frühestens mit
  den Ausprägungs-Aktionen in Phase 63, falls dort überhaupt gebraucht.
- **Echte Personenstammdaten** (E-Mail, Telefon, Geburtsdatum) an `person` —
  PERS-F-01, Future.
- **`PRAGMA foreign_keys = ON` aktivieren** — würde die FK-Deklarationen im
  gesamten Repo scharf schalten. Eigenständige Änderung mit projektweitem Blast
  Radius, ausdrücklich nicht Phase-60-Scope (siehe D-60-06).

</deferred>

---

*Phase: 60-Person-Datenmodell + Backfill*
*Context gathered: 2026-09-16*
