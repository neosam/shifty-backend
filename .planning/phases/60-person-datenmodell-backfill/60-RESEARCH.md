# Phase 60: Person-Datenmodell + Backfill - Research

**Researched:** 2026-09-16
**Domain:** SQLite-Migration (Schema + idempotenter SQL-Backfill) in einem bestehenden Rust/sqlx-0.8.6-Backend
**Confidence:** HIGH — jede Kernaussage ist entweder am Repo-Code verifiziert oder am
sqlx-0.8.6-Quelltext (lokal im Cargo-Registry-Cache vorhanden) nachgelesen. Keine externe
Websuche verwendet (Research-Flag der Phase: keine tiefere Recherche nötig).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

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

- **D-60-10:** Zeilen, die **nach** der Migration entstehen (Login-Auto-Anlage in
  `dao_impl_sqlite/src/lib.rs:96`, `create_sales_person` in
  `dao_impl_sqlite/src/sales_person.rs:133`), bekommen in Phase 60 noch **kein**
  `person_id`. Die Vergabe beim Anlegen ist Phase-61-Scope. Kein DB-Trigger, kein
  DAO-Eingriff in dieser Phase — Phase 60 bleibt reine Schema-/Backfill-Phase.
  Risiko ist null, weil bis Phase 61 nichts über `person_id` liest. Success
  Criterion 1 gilt für den Zustand **unmittelbar nach** der Migration.

**Gesperrt — in dieser Phase nicht neu aufmachen:**
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

### Deferred Ideas (OUT OF SCOPE)

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

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PERS-01 | Nach der Migration ist jede bestehende Sales Person und jeder bestehende Benutzer genau einer Person zugeordnet; verknüpfte Paare teilen sich dieselbe Person. | Konkretes 4-Schritt-Backfill-SQL unten (`## Code Examples`) — verifiziert korrelationsfrei über `randomblob(16)` + korrelierte Subquery statt zweifach-generierter Zufallswerte. Zählprobe als Validierungsformel identisch zu SC1. |
| PERS-02 | Die Backfill-Migration ist wiederholbar — ein zweiter Lauf erzeugt keine zusätzlichen `person`-Zeilen und keine Doppelzuordnung. | `WHERE person_id IS NULL` / `WHERE person_id NOT IN (...)`-Guards auf jedem Schritt (verifiziertes Repo-Idiom, `20260503000001`); Apply-twice-Testmechanik über `sqlx::raw_sql()` konkret hergeleitet und an sqlx-0.8.6-Quelltext verifiziert (`## Mechanische Fragen — Frage 3`). |
| PERS-03 | Der Backfill erfasst auch soft-gelöschte Sales Persons. | Backfill-SQL filtert bewusst **nicht** auf `deleted IS NULL` — Gegenprobe zum Repo-Standardmuster explizit dokumentiert (`## Common Pitfalls`, Pitfall 2). |
| PERS-04 | Die Datenbank erzwingt weiterhin höchstens einen Benutzer / eine Sales Person pro Person. | `CREATE UNIQUE INDEX` auf beiden Spalten (SQLite-Zwang wegen `ALTER TABLE ADD COLUMN`, verifiziert); DB-Level-Ablehnungstest gegen den Index, nicht FK (`PRAGMA foreign_keys`-Grep bestätigt: 0 Treffer im ganzen Repo). |
| PERS-06 | Abrechnungs-Snapshots und Schichtplan-Daten lösen unverändert auf; `sales_person.id`/`user.name` bitgleich; `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13. | Migration ist rein additiv (`ADD COLUMN` + neue Tabelle); `CURRENT_SNAPSHOT_SCHEMA_VERSION` lokalisiert und verifiziert bei `service_impl/src/billing_period_report.rs:140` (aktueller Wert `13`, unverändert zu lassen). |

</phase_requirements>

## Summary

Phase 60 ist eine reine Schema- + Backfill-Phase ohne API-Wirkung. Jede in der
Phasenbeschreibung genannte Technik hat entweder ein direktes, lesbares Repo-Präzedenz
(`ALTER TABLE ADD COLUMN`, `CREATE UNIQUE INDEX`, Soft-Delete-Konvention,
`sqlx::migrate!`-Testharness) oder ist eine im sqlx-0.8.6-Quelltext (lokal am
Cargo-Registry-Cache verifiziert, nicht nur aus Trainingswissen behauptet) nachgelesene
Mechanik (Pro-Datei-Transaktion, `sqlx::raw_sql()`). Eine einzige Stelle hat **kein**
direktes Repo-Präzedenz und musste hier aus verifizierten SQLite-Primitiven
hergeleitet werden: die korrekte SQL-Sequenz, mit der zwei unterschiedliche Zeilen
(`sales_person` und `user` eines verknüpften Paars) **denselben** per `randomblob(16)`
erzeugten `person_id`-Wert bekommen, ohne dass beide Seiten unabhängig voneinander
zufällig würfeln. Die naive Lösung („`UPDATE … SET person_id = randomblob(16)`" auf
beiden Tabellen unabhängig) würde zwei verschiedene Werte erzeugen und SC1 sofort
brechen — diese Falle ist in `## Code Examples` explizit vorgeführt und aufgelöst.

Die Phase berührt zwei Dateien im Kern: eine neue Migrationsdatei in
`migrations/sqlite/` und einen neuen Integrationstest (Analogie zu
`service_impl/src/test/rebooking_batch_state.rs`). Kein DAO-, Service- oder
REST-Code wird in dieser Phase verändert — das bestätigt exakt die
GSD-Scope-Regel „Phase 60: Frontend nein" aus der Roadmap.

**Primary recommendation:** Migration in genau vier SQL-Blöcken bauen — (1) `person`
anlegen, (2) beide `person_id`-Spalten additiv + `UNIQUE`-Index, (3) vierteiliger
Backfill (unverknüpfte Sales Persons, unverknüpfte User, verknüpfte
Sales-Person-Seite, verknüpfte User-Seite per korrelierter Subquery + finales
`INSERT … SELECT DISTINCT` in `person`), (4) Wegwerf-Tabelle mit `CHECK`-Constraint
als Selbst-Assertion, sofort wieder gedroppt. Apply-twice-Test liest die `.sql`-Datei
zur Laufzeit ein und führt sie über `sqlx::raw_sql(sql).execute(pool)` ein zweites Mal
aus — **nicht** über `sqlx::migrate!().run()`, das die Datei wegen der
`_sqlx_migrations`-Ledger-Zeile beim zweiten Aufruf überspringen würde.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `person`-Tabelle + Backfill-Logik | Database / Storage | — | Reine SQL-Migration, keine Anwendungsschicht beteiligt (D-60-10: kein DAO-Eingriff in dieser Phase). |
| 1:1-Invariante (höchstens ein User / eine Sales Person pro Person) | Database / Storage | — | `CREATE UNIQUE INDEX` — die DB, nicht die Applikation, erzwingt PERS-04 (D-60-05/06). |
| Selbst-Verifikation des Backfills (Zählprobe) | Database / Storage | — | Läuft **innerhalb** der Migrationstransaktion (D-60-07), nicht als nachgelagerter Applikations-Check. |
| Backend-Testabdeckung (Apply-twice, Double-Link-Ablehnung, Soft-Delete-Fixture) | API / Backend (Test-Tier) | Database / Storage | Integrationstest gegen echte In-Memory-SQLite via `sqlx::migrate!`, analog `rebooking_batch_state.rs`. |
| Docs-Freshness (`03-data-model.md`, `db-schema-er.mmd`, `F01-employee-management.md`) | — | — | Kein Architektur-Tier, aber hartes CLAUDE.md-Gate, ausgelöst durch die neue `.sql`-Datei. |

## Project Constraints (from CLAUDE.md)

- **Database Migrations:** `migrations/sqlite/` per `sqlx migrate add <name>`;
  SQLx Compile-Time-Checking (`query!`/`query_as!`) braucht eine lokal
  migrierte DB — nach dieser Migration muss `sqlx database reset` (additiv:
  besser `sqlx migrate run`, `reset` ist destruktiv laut STATE.md-Constraint)
  gegen die lokale Dev-DB laufen, bevor `cargo build` mit
  Compile-Time-Query-Checks grün ist. **Diese Phase selbst nutzt aber gar
  keinen `query!`/`query_as!`-Makro** (reines SQL in der Migrationsdatei +
  `sqlx::raw_sql`/`sqlx::query` im Test), daher ist der Offline-Cache-Eintrag
  in `.sqlx/` für diese Phase nicht zwingend nötig — nur falls der
  Apply-twice-Test `query_as!` gegen `person` verwendet (nicht empfohlen, siehe
  `## Code Examples`).
- **Billing Period Snapshot Schema Versioning:** `CURRENT_SNAPSHOT_SCHEMA_VERSION`
  MUSS bei den vier genannten Bump-Klauseln erhöht werden — keine davon greift
  hier (`person` führt keinen `value_type` ein, ändert keine Computation, kein
  Input-Set, `billing_period_sales_person.sales_person_id` unverändert).
  Aktueller Wert verifiziert: **13**, `service_impl/src/billing_period_report.rs:140`.
  Bleibt unverändert.
- **Clippy-Gate:** `cargo clippy --workspace -- -D warnings` ist Pflicht vor
  jedem Commit; `cargo test` allein reicht nicht. Migrationsdateien selbst
  lösen kein Clippy aus, aber der neue Rust-Testcode (Apply-twice-Test,
  Fixture-Setup) muss warnungsfrei sein.
- **Docs-Freshness-Gate:** `migrations/sqlite/*.sql` triggert
  `docs/architecture/03-data-model.md` **und**
  `docs/architecture/diagrams/db-schema-er.mmd` **und** die passende
  `docs/features/F*.md` — jeweils EN **und** DE im selben Commit.
  Verifiziert: `docs/features/F01-employee-management.md` (Abschnitt „3. Data
  model", Zeilen 180–245) ist die bestehende Feature-Doku für
  `sales_person`/`employee_work_details` und damit der natürliche Ort für den
  neuen `person`-Abschnitt; ihr deutsches Pendant
  `docs/features/F01-employee-management_de.md` existiert ebenfalls. Beide
  Dateien existieren bereits (172/177 Zeilen für die Data-Model-Docs, 393
  Zeilen im ER-Diagramm) und müssen im selben Commit wie die Migration
  erweitert werden — nicht als Docs-Follow-up.
- **Immer Tests bei Änderungen** (User-CLAUDE.md, global): gilt hier für den
  Apply-twice-Test, den Soft-Delete-Fixture-Test und den DB-Level-Ablehnungstest
  — alle drei sind ohnehin durch PERS-01..06/SC1..5 vorgeschrieben.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|---------------|
| `sqlx` | 0.8.6 (verifiziert: `Cargo.lock:3608`, Feature-Set `["runtime-tokio", "sqlite"]` in `dao_impl_sqlite/Cargo.toml:18-20`) | Migrations-Runner (`sqlx::migrate!`), Query-Ausführung | Bereits die einzige DB-Zugriffsschicht im Repo — keine Alternative in Betracht. |

Keine neuen Dependencies für diese Phase — reine SQL-Migration + Test-Code gegen
bereits vorhandene `sqlx`/`uuid`/`tokio`-Dependencies in `service_impl` bzw.
`dao_impl_sqlite`.

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|---------------|
| `uuid` | 1.8.0 (`dao_impl_sqlite/Cargo.toml:11`) | `Uuid::from_slice`/`Uuid::new_v4` im Testcode, um Fixture-IDs zu erzeugen und zurückgelesene BLOB-Bytes zu vergleichen | Fixture-Setup im Apply-twice-/Soft-Delete-Test, analog `absence_conversion.rs:558-590`. |
| `time` | 0.3.36 mit Feature `parsing`/`macros` | Zeitstempel-Fixtures (`created`, `datetime('now')`-Vergleich) | Falls der Test konkrete `created`-Werte prüft statt nur `IS NOT NULL`. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| SQL-only-Backfill in der Migrationsdatei | Re-runnbarer Rust-Service mit Idempotenz-Key-Tabelle (v1.0-Präzedenz, `absence_period_migration_source`) | **Explizit verworfen** — CONTEXT.md D-60/Deferred: die beidseitigen `UNIQUE`-Constraints auf `sales_person_user` schließen jede Mehrdeutigkeit strukturell aus, ein Rust-Service mit Konfliktauflösung wäre hier unnötige Komplexität (PITFALLS.md „Technical Debt Patterns"-Tabelle bestätigt das explizit als „Acceptable"). |
| `CREATE UNIQUE INDEX` | `UNIQUE`-Constraint direkt in `CREATE TABLE person (...)`-Spaltendefinition | Für `person.id` selbst irrelevant (PK ist ohnehin unique); für `sales_person.person_id`/`user.person_id` **technisch unmöglich**, weil SQLite `ALTER TABLE … ADD COLUMN` kein `UNIQUE` erlaubt (D-60-05, verifiziert an SQLite-Doku-Wissen, nicht codebasiert — siehe Assumptions Log A1). |

**Installation:** keine — alle benötigten Crates sind bereits Workspace-Dependencies.

**Version verification:**
```bash
grep -n "^name = \"sqlx\"" -A2 Cargo.lock
# → version = "0.8.6" (verifiziert, siehe oben)
```

## Package Legitimacy Audit

**Nicht anwendbar.** Diese Phase installiert keine neuen externen Packages — alle
verwendeten Crates (`sqlx`, `uuid`, `time`, `tokio`) sind bereits Workspace-Dependencies
mit etablierter Nutzung im Repo. Kein `package-legitimacy check` nötig.

## Architecture Patterns

### System Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│  sqlx::migrate!("../migrations/sqlite")  (main.rs:1651 / integration_test.rs:283) │
│  läuft beim Server-/Test-Start ALLE Migrationsdateien in Reihenfolge      │
└──────────────────────────────┬─────────────────────────────────────────┘
                                 │ neue Datei: 2026...NNNN_create-person-backfill.sql
                                 ▼
                 ┌───────────────────────────────────────┐
                 │  EINE Transaktion pro Migrationsdatei   │
                 │  (sqlx-sqlite migrate.rs: begin()...)   │
                 │                                          │
                 │  1. CREATE TABLE person (...)            │
                 │  2. ALTER TABLE sales_person              │
                 │       ADD COLUMN person_id BLOB(16)       │
                 │     ALTER TABLE user                       │
                 │       ADD COLUMN person_id BLOB(16)         │
                 │     CREATE UNIQUE INDEX ... sales_person(person_id) │
                 │     CREATE UNIQUE INDEX ... user(person_id)  │
                 │  3. Backfill (4 SQL-Bloecke, siehe unten)   │
                 │  4. Selbst-Assertion (CHECK-Wegwerftabelle) │
                 │                                          │
                 │  Fehler an JEDER Stelle → ROLLBACK        │
                 │  (tx wird gedroppt statt committet)        │
                 └───────────────┬───────────────────────┘
                                 │ Erfolg
                                 ▼
                 INSERT INTO _sqlx_migrations (...) + COMMIT
                                 │
                                 ▼
        Server startet / Test läuft weiter — person_id ist ab jetzt überall gesetzt
        (außer bei NACH der Migration neu angelegten Zeilen, D-60-10 — bewusst offen)
```

### Recommended Project Structure

```
migrations/sqlite/
└── 20260916HHMMSS_create-person-backfill.sql   # eine Datei, ein Transaktions-Scope
service_impl/src/test/
└── person_backfill.rs                          # neues Testmodul (Discretion: Ablage)
    # analog rebooking_batch_state.rs: echte In-Memory-SQLite via sqlx::migrate!,
    # keine gemockte DB. 4 Testgruppen:
    #   - apply-twice (PERS-02 / SC2)
    #   - soft-deleted Sales Person mit billing_period_sales_person-Zeile (PERS-03 / SC3)
    #   - DB-Level-Doppel-Link-Ablehnung (PERS-04 / SC4)
    #   - Zählprobe / vollständige Zuordnung (PERS-01 / SC1)
```

### Pattern 1: Additive Spalte + nachgelagerter UNIQUE-Index (SQLite-Zwang)

**What:** SQLite verbietet `UNIQUE` direkt in `ALTER TABLE ... ADD COLUMN`. Die
1:1-Garantie kommt deshalb als separates `CREATE UNIQUE INDEX` NACH dem `ADD COLUMN`.

**When to use:** Immer, wenn eine bestehende Tabelle einen neuen, eindeutigen FK
bekommen soll, ohne die Tabelle neu anzulegen (was hier wegen der zwölf
referenzierenden Tabellen auf `sales_person.id` ohnehin ausgeschlossen ist).

**Example (repo-verifiziert, Stilmuster aus zwei bestehenden Migrationen kombiniert):**
```sql
-- Analog Stil von 20260707000001_add-source-column-to-extra-hours.sql (ADD COLUMN)
-- + 20260707000000_create-rebooking-batch.sql (CREATE UNIQUE INDEX danach)
ALTER TABLE sales_person ADD COLUMN person_id BLOB(16);
ALTER TABLE user         ADD COLUMN person_id BLOB(16);

-- D-60-01: bewusst KEIN "WHERE deleted IS NULL" — soft-geloeschte Sales
-- Persons behalten ihre Person-Identitaet dauerhaft (Produktentscheidung,
-- siehe 60-CONTEXT.md D-60-01). NICHT beim naechsten Aufraeum-Pass "reparieren".
CREATE UNIQUE INDEX idx_sales_person_person_id
    ON sales_person (person_id);

CREATE UNIQUE INDEX idx_user_person_id
    ON user (person_id);
```

### Pattern 2: Per-Datei-Transaktion als Rollback-Schutz

**What:** `sqlx::migrate!` führt jede `.sql`-Datei standardmäßig in genau einer
Transaktion aus (`no_tx` defaultet auf `false`, opt-out nur über eine `.sql`-Datei,
die **wörtlich** mit `-- no-transaction` beginnt — verifiziert in
`sqlx-core-0.8.6/src/migrate/source.rs:127`: `let no_tx = sql.starts_with("-- no-transaction");`).
Kein bestehendes File im Repo nutzt diesen Marker (`grep -rl "no-transaction"
migrations/sqlite/*.sql` → 0 Treffer), also läuft auch die neue Datei
transaktional.

**Verifizierter Ablauf bei einem Fehler** (`sqlx-sqlite-0.8.6/src/migrate.rs:131-162`):
```rust
// Ausschnitt aus sqlx-sqlite 0.8.6 (lokal am Cargo-Cache gelesen):
let mut tx = self.begin().await?;
let _ = tx
    .execute(&*migration.sql)          // <- die GESAMTE .sql-Datei als ein execute()
    .await
    .map_err(|e| MigrateError::ExecuteMigration(e, migration.version))?;
// ... INSERT INTO _sqlx_migrations ...
tx.commit().await?;
```
Schlägt `tx.execute(...)` fehl (z. B. durch die CHECK-Constraint-Verletzung aus
Pattern 3), gibt die Funktion früh über `?` zurück; `tx` wird nie committet, sondern
beim Drop automatisch zurückgerollt (`sqlx::Transaction`-Drop-Verhalten). **Das ist
die Grundlage, auf der D-60-07 funktioniert** — kein manuelles `BEGIN`/`ROLLBACK`
in der `.sql`-Datei nötig oder sinnvoll (würde die äußere Transaktion nur verschachteln).

**When to use:** Immer für mehrschrittige, selbst-validierende Migrationen in diesem Repo.

### Pattern 3: Selbst-abbrechende Assertion via Wegwerf-CHECK-Tabelle (D-60-07)

**What:** SQLite unterstützt `RAISE()` nur innerhalb von Trigger-Programmen, nicht
als freistehenden Statement-Typ. Der im Repo **noch nicht vorhandene**, aber aus
verifizierten SQLite-Primitiven korrekt herleitbare Trick: eine Wegwerftabelle mit
`CHECK`-Constraint anlegen, eine berechnete Zeile hineinschreiben, sofort wieder
droppen. Verletzt die Zählprobe die Bedingung, wirft `INSERT` einen
`SQLITE_CONSTRAINT_CHECK`-Fehler, der (Pattern 2) die ganze Transaktion zurückrollt.

**When to use:** Für D-60-07's Zählprobe `COUNT(person) == COUNT(sales_person) +
COUNT(user) − COUNT(sales_person_user)`, ausgeführt am Ende der Migration.

**Example:**
```sql
-- D-60-07: Selbst-Verifikation. Bricht die Migration (und damit den Server-Start)
-- ab, wenn die Zaehlprobe aus Success Criterion 1 nicht aufgeht. SQLite kennt
-- RAISE() nur in Triggern -- diese Wegwerftabelle mit CHECK ist das Aequivalent
-- fuer eine freistehende Assertion. Die Tabelle wird noch in derselben
-- Migrations-Transaktion sofort wieder gedroppt.
CREATE TEMP TABLE __phase60_backfill_assertion (
    ok INTEGER NOT NULL CHECK (ok = 1)
);

INSERT INTO __phase60_backfill_assertion (ok)
SELECT CASE
    WHEN (SELECT COUNT(*) FROM person)
       = (SELECT COUNT(*) FROM sales_person)
       + (SELECT COUNT(*) FROM user)
       - (SELECT COUNT(*) FROM sales_person_user)
    THEN 1 ELSE 0
END;

DROP TABLE __phase60_backfill_assertion;
```
`CREATE TEMP TABLE` ist hier bewusst gewählt (statt `CREATE TABLE`), damit im
unwahrscheinlichen Fall eines erfolgreichen Commits trotz eines Bugs im
`DROP TABLE`-Statement keine Wegwerf-Tabelle in der persistenten Produktions-DB
zurückbleibt — TEMP-Tabellen verschwinden ohnehin am Verbindungsende. Der
`DROP TABLE` bleibt trotzdem im Skript, damit die Assertion nicht mit dem Rest der
Migrationsdatei (falls sie später um weitere Assertions erweitert wird) kollidiert.

### Pattern 4: Idempotenz-Guard auf dem Legacy-PK (Repo-Standard)

**What:** Jedes zustandsverändernde Statement bekommt einen `WHERE`-Guard, der beim
zweiten Lauf keine Zeilen mehr trifft.

**Repo-Präzedenz (wörtlich verifiziert):**
`migrations/sqlite/20260503000001_create-absence-period-migration-source.sql:2`:
> `-- Idempotency key: extra_hours_id PK. Re-run skips already-mapped rows.`

**Anwendung auf Phase 60** — der Guard ist `person_id IS NULL` (nicht ein
Fremdschlüssel-PK wie im v1.0-Beispiel, weil `person_id` hier selbst der zu
setzende Zustand ist):
```sql
UPDATE sales_person
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND id NOT IN (SELECT sales_person_id FROM sales_person_user);
```
Beim zweiten Lauf ist `person_id` für jede Zeile bereits gesetzt → `WHERE`-Klausel
trifft null Zeilen → Statement ist ein No-Op. Das ist exakt PERS-02 / SC2.

### Anti-Patterns to Avoid

- **`WHERE deleted IS NULL` im Backfill:** Der im Repo dominante DAO-Reflex
  (CLAUDE.md: „All queries include soft delete checks") gilt für Live-Queries,
  nicht für einen einmaligen Backfill, der JEDER historischen Zeile eine
  Identität geben muss (Pitfall 2, PERS-03/SC3).
- **`person_id = randomblob(16)` unabhängig auf beiden Seiten eines verknüpften
  Paars:** Erzeugt zwei verschiedene Zufallswerte für dieselbe Person und bricht
  SC1 sofort. Siehe die korrekte Sequenz in `## Code Examples`.
- **Foreign-Key-basierter Ablehnungstest:** `PRAGMA foreign_keys` ist nirgends im
  Repo gesetzt (grep-verifiziert, 0 Treffer) — ein Test, der auf eine FK-Ablehnung
  wartet, schlägt lautlos fehl, weil SQLite die Verletzung klaglos zulässt
  (D-60-06).
- **`sqlx::migrate!().run(pool)` ein zweites Mal für den Apply-twice-Test:** Läuft
  ins Leere — die Migration steht bereits in `_sqlx_migrations` und wird
  übersprungen. Stattdessen die `.sql`-Datei selbst einlesen und über
  `sqlx::raw_sql(...)` erneut ausführen (siehe „Mechanische Fragen — Frage 3").

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|--------------|-----|
| UUID-Generierung in reinem SQL | Eigene hex/random-Konkatenation nach `V_UUID_V4`-Vorbild | `randomblob(16)` direkt als `BLOB(16)`-Spaltenwert | `V_UUID_V4` (Migration `20240426150045_user-roles.sql:90-98`) erzeugt einen **36-Zeichen-Hex-String mit Bindestrichen** — ein anderes Format als das `BLOB(16)`, das `sales_person.id`/`rebooking_batch.id` tatsächlich speichern (verifiziert: `dao_impl_sqlite/src/sales_person.rs:33` `Uuid::from_slice(sales_person.id.as_ref())` erwartet 16 rohe Bytes, kein Hex-String). `V_UUID_V4` ist zudem **im gesamten Repo unbenutzt** (grep: einzige Fundstelle ist die eigene `CREATE VIEW`-Definition) — kein Konsument existiert, es ist totes Infrastruktur-Gerüst. `randomblob(16)` (das nackte SQLite-Builtin, ohne den View) ist der korrekte, mit `Uuid::from_slice` kompatible Weg und hat als eigenständiger Aufruf **kein** Repo-Präzedenz, ist aber eine dokumentierte SQLite-Kernfunktion. |
| Konflikt-Erkennung bei verknüpften Paaren | Eigene Duplikat-Prüfung in SQL oder Rust vor dem Backfill | Die bestehenden `UNIQUE(sales_person_id)` + `UNIQUE(user_id)` auf `sales_person_user` (verifiziert `migrations/sqlite/20240506114107_add_sales_person.sql:20-21`) | Diese beiden Constraints garantieren bereits, dass die DB niemals zwei Zeilen mit demselben `sales_person_id` oder `user_id` in `sales_person_user` zulässt — jede korrelierte Subquery über diese Tabelle liefert also garantiert höchstens eine Zeile. Eine zusätzliche Eindeutigkeitsprüfung im Backfill wäre redundant. |
| Selbst-Validierung der Migration | Eigenes Rust-Assertion-Script, das nach der Migration separat läuft | Die Wegwerf-CHECK-Tabelle (Pattern 3), innerhalb derselben Transaktion | Ein nachgelagertes Rust-Script sieht die Daten erst NACH dem Commit — ein Fehler wäre dann bereits committet und ließe sich nicht mehr sauber zurückrollen, ohne die Backup-Datei einzuspielen (D-60-09 hat bewusst keine DB-interne Backup-Tabelle). |

**Key insight:** Jede Zeile dieser Migration hat eine Entsprechung, die entweder
bereits im Repo existiert oder direkt aus einer SQLite-Kernfunktion folgt — der
einzige Punkt, an dem eigenständige Herleitung nötig war (Korrelation zweier
UPDATE-Ziele auf denselben Zufallswert), ist unten vollständig vorgeführt, damit
der Planer ihn nicht selbst neu erfinden muss.

## Common Pitfalls

### Pitfall 1: Backfill ist nicht wirklich re-runnbar (PERS-02)

**What goes wrong:** Ein Backfill, der beim zweiten Lauf partiell erneut Zeilen
erzeugt oder verändert, weil ein `WHERE`-Guard fehlt oder auf der falschen Spalte
sitzt.

**Why it happens:** Die naheliegende Formulierung „lege für jede Sales Person
eine Person an" braucht einen Guard, der beim zweiten Durchlauf greift — und
zwar auf `person_id IS NULL`, nicht (wie beim v1.0-Präzedenz) auf einem
Fremdschlüssel-PK in einer separaten Mapping-Tabelle, weil es hier keine solche
Mapping-Tabelle gibt.

**How to avoid:** Jeder der vier Backfill-Blöcke (siehe `## Code Examples`) hat
einen eigenen `WHERE person_id IS NULL`-Guard; der finale `INSERT INTO person`
hat einen `WHERE person_id NOT IN (SELECT id FROM person)`-Guard. Der Apply-twice-
Test (PERS-02/SC2) ist die einzige verlässliche Verifikation — „kompiliert und
läuft durch" beweist nichts, siehe D-60-08.

**Warning signs:** Backfill-SQL ohne `WHERE person_id IS NULL`/`WHERE person_id
NOT IN (...)`; kein Test führt die Migrations-SQL zweimal aus.

**Phase to address:** Phase 60 (diese Phase).

### Pitfall 2: Soft-gelöschte Sales Persons fallen durch den `WHERE deleted IS NULL`-Reflex (PERS-03)

**What goes wrong:** `sales_person` hat eine `deleted TEXT`-Spalte (verifiziert
`migrations/sqlite/20240506114107_add_sales_person.sql:7`), `user` hat **keine**
(verifiziert `migrations/sqlite/20240426150045_user-roles.sql:2-6` — nur `name`,
`update_timestamp`, `update_process`). Der im ganzen Repo dominante DAO-Reflex
„Live-Queries filtern `deleted IS NULL`" ist für einen Backfill, der JEDER
historischen Zeile eine Identität geben muss, falsch.

**Why it happens:** CLAUDE.md schreibt „All queries include soft delete checks"
als DAO-Konvention — das gilt für Lese-Pfade zur Laufzeit, nicht für einen
einmaligen Backfill.

**How to avoid:** Keiner der vier Backfill-Blöcke filtert auf `deleted`. Ein
dedizierter Fixture-Test legt eine soft-gelöschte Sales Person mit einer
`billing_period_sales_person`-Zeile an und prüft `person_id IS NOT NULL` danach
(PERS-03/SC3, siehe `## Validation Architecture`).

**Warning signs:** Ein `WHERE deleted IS NULL` taucht irgendwo im Backfill-SQL
auf; kein Fixture-Test deckt eine soft-gelöschte Sales Person ab.

**Phase to address:** Phase 60 (diese Phase).

### Pitfall 3: Die 1:1-Invariante verschwindet mit der Verknüpfungstabelle, wenn niemand sie nachbaut (PERS-04)

**What goes wrong:** `sales_person_user` trägt heute `UNIQUE(sales_person_id)` +
`UNIQUE(user_id)` (verifiziert Zeilen 20-21 der genannten Migration) — eine
nullable FK-Spalte ohne eigenen `UNIQUE`-Index bietet diesen Schutz nicht von
selbst.

**Why it happens:** `person_id BLOB(16) NULL` sieht nach einer strikten
Verbesserung gegenüber der Join-Tabelle aus, aber SQLite lässt beliebig viele
`NULL`-Werte in einer `UNIQUE`-Spalte zu (korrekt für die drei Ausprägungen) —
verhindert aber NICHT, dass zwei verschiedene Sales-Person-Zeilen denselben
NICHT-NULL-`person_id`-Wert tragen, sofern kein `UNIQUE`-Index explizit auf der
Spalte liegt.

**How to avoid:** `CREATE UNIQUE INDEX` auf beiden Spalten, **sofort** nach dem
`ADD COLUMN`, in derselben Migrationsdatei (nicht „später nachziehen"). Ein
dedizierter Test versucht einen direkten SQL-Doppel-Link und erwartet eine
DB-Fehler-Antwort, nicht nur einen Applikations-Fehler (PERS-04/SC4).

**Warning signs:** `person_id BLOB(16) NULL` ohne begleitenden
`CREATE UNIQUE INDEX` im selben Commit; kein Test versucht einen Doppel-Link
direkt per SQL.

**Phase to address:** Phase 60 (Schema), Phase 61 (Service-Ebene — nicht Teil
dieser Phase).

## Mechanische Fragen — konkrete Antworten

### Frage 1: Welches Audit-Spalten-Muster ist für neue Tabellen dominant?

**Antwort (ausgezählt, nicht geraten):** Für **jede seit Mai 2026 genuin neu
angelegte Tabelle** (nicht `ALTER TABLE`) gilt exakt das Muster
`created TEXT NOT NULL, deleted TEXT, update_process TEXT NOT NULL,
update_version BLOB NOT NULL` — verifiziert an fünf Migrationen:

| Tabelle | Migration | Spalten (Auszug) |
|---|---|---|
| `absence_period` | `20260502170000_create-absence-period.sql` | `created`, `deleted`, `update_timestamp`, `update_process`, `update_version` |
| `rebooking_batch` | `20260707000000_create-rebooking-batch.sql` | `created`, `deleted`, `update_process`, `update_version` |
| `vacation_entitlement_offset` | `20260629000000_create-vacation-entitlement-offset.sql` | `created`, `deleted`, `update_process`, `update_version` |
| `week_status` | `20260702000000_create-week-status.sql` | `created`, `deleted`, `update_process`, `update_version` |
| `pdf_export_config` | `20260703000000_create-pdf-export-config.sql` | `update_process`, `update_version` (Single-Row, kein `created`/`deleted` nötig) |

Das ältere `update_timestamp`-Trigger-Trio (`sales_person`, `user`, `slot`,
`booking` — alle aus 2024) wird für **keine** seit Mai 2026 neu angelegte Tabelle
mehr verwendet. **Für `person` (D-60-04: kein `deleted`) folgt daraus:**
`created TEXT NOT NULL, update_process TEXT NOT NULL, update_version BLOB NOT NULL` —
das dominante moderne Muster minus der laut D-60-04 explizit ausgeschlossenen
`deleted`-Spalte.

### Frage 2: Funktioniert der CHECK-Tabellen-Trick für D-60-07 wirklich?

**Ja, vollständig am sqlx-0.8.6-Quelltext verifiziert** (nicht nur SQLite-Doku-Wissen):

1. `sqlx-core-0.8.6/src/migrate/source.rs:127`:
   `let no_tx = sql.starts_with("-- no-transaction");` — jede Migrationsdatei
   läuft transaktional, außer sie beginnt wörtlich mit diesem Marker. Kein
   File im Repo tut das (`grep -rl "no-transaction" migrations/sqlite/*.sql`
   → 0 Treffer).
2. `sqlx-sqlite-0.8.6/src/migrate.rs:131-145`:
   ```rust
   let mut tx = self.begin().await?;
   let _ = tx.execute(&*migration.sql).await
       .map_err(|e| MigrateError::ExecuteMigration(e, migration.version))?;
   ```
   Die **gesamte** `.sql`-Datei wird als ein `execute()`-Aufruf gegen die
   offene Transaktion gefahren. Schlägt irgendein Statement darin fehl (z. B.
   die `CHECK`-Verletzung), gibt `?` sofort zurück, BEVOR
   `INSERT INTO _sqlx_migrations` und `tx.commit()` erreicht werden.
3. `tx` (ein `sqlx::Transaction`) wird beim frühen `return` fallengelassen,
   ohne `commit()` aufgerufen zu haben — sqlx rollt in diesem Fall automatisch
   zurück (Standard-`Drop`-Verhalten von `sqlx::Transaction`).

Ergebnis: eine `CHECK`-Verletzung irgendwo in der Migrations-SQL rollt die
**gesamte** Migration zurück, inklusive aller vorherigen `CREATE TABLE`/`ALTER
TABLE`/Backfill-Statements in derselben Datei. Genau das verlangt D-60-07.

### Frage 3: Wie führt der Apply-twice-Test dieselbe Migration ein zweites Mal aus?

`sqlx::migrate!().run(pool)` **scheidet aus** — die Migration steht nach dem
ersten Lauf in `_sqlx_migrations` (mit Checksum) und wird beim zweiten `.run()`
automatisch übersprungen (Standardverhalten des Migrations-Ledgers, das genau
Doppelausführung verhindern soll — kontraproduktiv für einen Apply-twice-Test).

**Verifizierter Mechanismus:** `sqlx-core-0.8.6/src/raw_sql.rs` definiert
`pub fn raw_sql(sql: &str) -> RawSql<'_>` — explizit dokumentiert als API für
„one or more raw SQL statements, separated by semicolons", inklusive DDL, ohne
Prepared-Statement-Cache, und mit derselben Autocommit-Semantik wie
`sqlx::migrate!` (alle Statements laufen implizit in einer Transaktion; ein
Fehler rollt das gesamte Skript zurück — laut Doku-Kommentar im Quelltext
wörtlich: „If one statement triggers an error, the whole script aborts and
rolls back."). Die konkrete Testsequenz:

```rust
// 1) Normaler Erstlauf ueber sqlx::migrate! (wie jeder Integrationstest in diesem Repo).
let pool = setup_pool().await; // sqlx::migrate!("./../migrations/sqlite").run(...).await

// 2) Fixtures seeden (unverknuepfte Sales Person, unverknuepftes Paar, User, ...).
seed_fixtures(&pool).await;

// 3) "Vorher"-Snapshot: Zeilenzahlen + Zuordnungen aus person/sales_person/user lesen.
let before = snapshot(&pool).await;

// 4) Die MIGRATIONSDATEI SELBST ein zweites Mal ausfuehren -- NICHT ueber
//    sqlx::migrate!().run() (das wuerde sie wegen _sqlx_migrations ueberspringen),
//    sondern per sqlx::raw_sql() direkt gegen den Pool.
let migration_sql = include_str!(
    "../../../migrations/sqlite/20260916HHMMSS_create-person-backfill.sql"
);
sqlx::raw_sql(migration_sql)
    .execute(pool.as_ref())
    .await
    .expect("zweiter Lauf darf nicht fehlschlagen (Idempotenz + Selbst-Assertion muessen gruen bleiben)");

// 5) "Nachher"-Snapshot + Vergleich: Zeilenzahlen UNVERAENDERT, Zuordnungen IDENTISCH.
let after = snapshot(&pool).await;
assert_eq!(before, after);
```

**Wichtig:** Da die Migrationsdatei selbst schon `CREATE TABLE person (...)`
und `ALTER TABLE ... ADD COLUMN person_id` enthält, würde ein zweiter
`raw_sql()`-Lauf beim `CREATE TABLE`/`ALTER TABLE` selbst fehlschlagen (Tabelle/
Spalte existiert bereits) — sofern diese Statements nicht defensiv geschrieben
sind. **Konsequenz für den Migrationstext:** `CREATE TABLE IF NOT EXISTS
person (...)` (Repo-Standard, siehe `rebooking_batch`/`week_status`/
`pdf_export_config` — alle nutzen `IF NOT EXISTS`) und für die beiden
`ALTER TABLE ... ADD COLUMN`-Statements gibt es in SQLite **kein**
`IF NOT EXISTS`-Äquivalent (`ALTER TABLE ADD COLUMN` unterstützt das nicht) —
diese zwei Statements müssen deshalb in den Backfill-Testaufbau **nicht** über
den vollen Datei-Replay laufen, sondern der Apply-twice-Test darf sich exklusiv
auf den Backfill-Teil (Block 3+4 aus `## Code Examples`) beschränken, indem
dieser Teil als eigener SQL-String extrahiert wird — ODER die Migrationsdatei
wird so geschrieben, dass Schema-DDL (Block 1+2) und Backfill (Block 3+4)
**zwei separate, aber in derselben Transaktion laufende** Abschnitte sind, und
der Apply-twice-Test liest gezielt nur den Backfill-Abschnitt erneut ein
(z. B. über einen Kommentar-Marker `-- BACKFILL START` / `-- BACKFILL END`
im `.sql`-File, den der Test per String-Split herausschneidet). **Diese
Entscheidung liegt beim Planer** (nicht in CONTEXT.md gesperrt) — beide Wege
sind mit den verifizierten sqlx-Primitiven korrekt umsetzbar; der Marker-Split
ist robuster, weil er den Apply-twice-Test unabhängig von `IF NOT EXISTS`-
Nebenwirkungen auf die Schema-DDL macht.

### Frage 4: Wie ist `randomblob(16)` gegenüber dem bestehenden ID-Format geformt?

**Verifiziert, zwei unterschiedliche Formate im Repo, nicht zu verwechseln:**

1. **`V_UUID_V4`-View** (`migrations/sqlite/20240426150045_user-roles.sql:90-98`):
   erzeugt einen **36-Zeichen-Hex-String mit Bindestrichen**
   (`hex(randomblob(4)) || '-' || ...`) — Format `xxxxxxxx-xxxx-4xxx-Yxxx-xxxxxxxxxxxx`.
   **Grep-verifiziert: diese View wird im gesamten Repo nirgends konsumiert**
   (`grep -rn "V_UUID_V4"` findet nur die eigene `CREATE VIEW`-Zeile) — totes
   Infrastruktur-Gerüst, kein aktives Muster.
2. **Tatsächlich verwendetes Format** (`sales_person.id`, `rebooking_batch.id`,
   `absence_period.id`, …): rohe **16 Bytes** in einer `BLOB(16)`-Spalte, auf
   Rust-Seite `entity.id.as_bytes().to_vec()` beim Schreiben
   (`dao_impl_sqlite/src/sales_person.rs:126`) und `Uuid::from_slice(bytes)`
   beim Lesen (`dao_impl_sqlite/src/sales_person.rs:33`).

`randomblob(16)` (das nackte SQLite-Builtin, **ohne** den `V_UUID_V4`-View)
erzeugt exakt 16 zufällige Bytes — bitkompatibel mit `Uuid::from_slice()`.
**Für den Backfill ist deshalb `randomblob(16)` direkt zu verwenden, niemals
`V_UUID_V4`.** Da `person` in dieser Phase von keinem Rust-Code gelesen wird
(D-60-10: kein DAO-Eingriff), ist diese Bitkompatibilität aktuell nicht
funktional beobachtbar — sie wird erst in Phase 61 relevant, wenn ein
`PersonDao` `person.id` erstmals per `Uuid::from_slice()` einliest. **Das ist
ein guter, günstiger Vorab-Test für diese Phase:** ein Integrationstest, der
`SELECT id FROM person` liest und `Uuid::from_slice(&bytes)` erfolgreich
aufruft, verifiziert das Format bereits jetzt, ohne auf Phase 61 zu warten.

### Frage 5: Ist `PRAGMA foreign_keys` wirklich nirgends gesetzt?

**Bestätigt.** `grep -rn "PRAGMA foreign_keys" --include="*.rs" --include="*.sql" .`
liefert **0 Treffer** im gesamten Repository. D-60-06 ist damit nicht nur eine
Behauptung aus CONTEXT.md, sondern in dieser Session erneut gegengeprüft.

### Frage 6: Wo genau steht `CURRENT_SNAPSHOT_SCHEMA_VERSION`?

`service_impl/src/billing_period_report.rs:140`:
```rust
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13;
```
Verifiziert — aktueller Wert **13**, bleibt in dieser Phase unverändert (keine
der vier CLAUDE.md-Bump-Klauseln greift, siehe `## Project Constraints`).

### Frage 7: SQLx Compile-Time-Checking — was muss lokal laufen?

- `.sqlx/`-Offline-Cache **existiert und ist eingecheckt** (verifiziert: `.sqlx/`
  im Repo-Root enthält `query-*.json`-Dateien, zuletzt aktualisiert 2026-07-23).
  CI nutzt `SQLX_OFFLINE=true` (verifiziert `.github/workflows/rust.yml:11`).
- **Diese Phase braucht dafür nichts Neues:** Die Migration selbst enthält
  keinen `query!`/`query_as!`-Aufruf (reines SQL in der `.sql`-Datei). Der neue
  Testcode sollte ebenfalls **kein** `query_as!` gegen `person` verwenden
  (kein Compile-Time-Schema-Wissen über die neue Tabelle nötig), sondern
  `sqlx::query()`/`sqlx::raw_sql()` mit manuellem Row-Parsing — dieselbe
  Technik, die `absence_conversion.rs:564-580` bereits für Fixture-Inserts
  nutzt (`sqlx::query("INSERT INTO sales_person (...) VALUES (...)").bind(...)`).
  Falls der Planer sich dennoch für `query_as!` gegen `person` entscheidet, MUSS
  `cargo sqlx prepare --workspace` (in `nix develop`, gegen eine lokal migrierte
  DB) laufen und die neuen `.sqlx/query-*.json`-Dateien committet werden
  (Präzedenz: `36-01-SUMMARY.md` dokumentiert genau diesen Schritt für eine
  frühere Migration).
- **Lokale Dev-DB:** nach `sqlx migrate add <name>` und Schreiben der Migration
  muss `sqlx migrate run` (NICHT `sqlx database reset` — laut STATE.md-Constraint
  destruktiv) gegen die lokale `localdb.sqlite3` laufen, bevor `cargo build`
  mit eventuellen neuen Compile-Time-Checks grün ist.

## Code Examples

### Vollständiges Backfill-SQL (die eine Stelle ohne direktes Repo-Präzedenz, hier hergeleitet)

Die Herausforderung: für ein verknüpftes Paar (`sales_person_user`-Zeile) müssen
`sales_person.person_id` UND `user.person_id` **denselben** Wert bekommen. Zwei
unabhängige `UPDATE ... SET person_id = randomblob(16)`-Statements (eines pro
Tabelle) würden **zwei verschiedene** Zufallswerte erzeugen — SC1 bricht sofort.
Die korrekte Sequenz generiert den Zufallswert nur auf einer Seite und **kopiert**
ihn auf die andere Seite über eine korrelierte Subquery, die durch die
bestehenden `UNIQUE`-Constraints auf `sales_person_user` garantiert höchstens
eine Zeile zurückgibt:

```sql
-- ── Block 3: Backfill (idempotent, D-60-08) ─────────────────────────────

-- 3a) Unverknuepfte Sales Persons (auch soft-geloeschte, PERS-03/Pitfall 2)
--     bekommen eine EIGENE zufaellige person_id.
UPDATE sales_person
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND id NOT IN (SELECT sales_person_id FROM sales_person_user);

-- 3b) Unverknuepfte User bekommen eine EIGENE zufaellige person_id.
UPDATE user
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND name NOT IN (SELECT user_id FROM sales_person_user);

-- 3c) Verknuepfte Paare, Sales-Person-Seite: EIGENE zufaellige person_id.
--     (Diese Seite "gewinnt" die Zufallsgenerierung fuer das Paar.)
UPDATE sales_person
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND id IN (SELECT sales_person_id FROM sales_person_user);

-- 3d) Verknuepfte Paare, User-Seite: KOPIERT die gerade in 3c vergebene
--     person_id -- KEIN eigener randomblob()-Aufruf. Die korrelierte
--     Subquery liefert dank UNIQUE(user_id) auf sales_person_user
--     garantiert hoechstens eine Zeile.
UPDATE user
SET person_id = (
    SELECT sp.person_id
    FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    WHERE spu.user_id = user.name
)
WHERE person_id IS NULL
  AND name IN (SELECT user_id FROM sales_person_user);

-- 3e) Fuer JEDEN jetzt referenzierten person_id-Wert (egal ob aus 3a/3b
--     eigenstaendig oder aus 3c/3d gemeinsam) genau EINE person-Zeile anlegen.
--     UNION dedupliziert automatisch -- verknuepfte Paare liefern denselben
--     Wert aus beiden Quellen und erzeugen trotzdem nur eine Zeile.
INSERT INTO person (id, created, update_process, update_version)
SELECT DISTINCT ids.person_id, datetime('now'), 'phase-60-person-backfill', randomblob(16)
FROM (
    SELECT person_id FROM sales_person WHERE person_id IS NOT NULL
    UNION
    SELECT person_id FROM user WHERE person_id IS NOT NULL
) ids
WHERE ids.person_id NOT IN (SELECT id FROM person);
```

**Warum das idempotent ist (D-60-08):** Beim zweiten Lauf ist in 3a/3b/3c
`person_id IS NULL` für keine Zeile mehr wahr → No-Op. In 3d dasselbe. In 3e ist
`ids.person_id NOT IN (SELECT id FROM person)` für keinen Wert mehr wahr, weil
Block 3e beim ersten Lauf bereits für jeden aktuell referenzierten Wert eine
Zeile angelegt hat → No-Op.

**Warum das korrekt für verknüpfte Paare ist:** 3c setzt genau einen neuen
Zufallswert pro `sales_person`-Zeile eines Paars. 3d liest **denselben** Wert
über den Join zurück (kein zweiter `randomblob()`-Aufruf) und schreibt ihn auf
die `user`-Zeile. Beide Seiten des Paars zeigen danach garantiert auf dieselbe
`person.id` — das ist exakt SC1 „wer heute über `sales_person_user` verknüpft
ist, teilt sich danach dieselbe Person".

**Reihenfolge-Hinweis:** 3a/3b vor 3c/3d ist beliebig (überschneidungsfrei durch
die `NOT IN`/`IN`-Filter auf `sales_person_user`), aber 3d muss **nach** 3c
laufen (liest den von 3c geschriebenen Wert). 3e muss **nach** allen vieren
laufen. Innerhalb einer Migrationsdatei ist SQL-Statement-Reihenfolge ohnehin
sequenziell garantiert (kein paralleles Ausführen innerhalb einer Transaktion).

### Fixture-Insert-Stil für Tests (repo-verifiziertes Muster)

```rust
// Analog absence_conversion.rs:564-580 — rohes SQL, BLOB(16) als Vec<u8>.
sqlx::query(
    "INSERT INTO sales_person (id, name, inactive, deleted, update_process, update_version) \
     VALUES (?1, ?2, 0, ?3, 'seed', ?4)"
)
.bind(sales_person_id.as_bytes().to_vec())
.bind("Soft-geloeschte Verkaeuferin")
.bind("2025-01-01T00:00:00")   // deleted <> NULL fuer den PERS-03-Fixture-Fall
.bind(Uuid::new_v4().as_bytes().to_vec())
.execute(pool.as_ref())
.await
.expect("seed sales_person");
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|-------------------|---------------|--------|
| `update_timestamp`-Trigger-Trio für neue Tabellen (2024er Stil, `sales_person`/`user`) | `created`/`deleted`/`update_process`/`update_version` ohne Trigger (2026er Stil) | Ausgezählt ab `20260502170000` (absence_period) bis heute — keine neue Tabelle seither nutzt das Trigger-Trio | `person` sollte dem 2026er Muster folgen (minus `deleted`, D-60-04) — konsistent mit jeder anderen in diesem Milestone-Zeitraum angelegten Tabelle. |
| `V_UUID_V4`-View als Hex-String-UUID-Quelle | Direktes `Uuid::new_v4()` in Rust, `.as_bytes().to_vec()` beim Schreiben | Nie aktiv migriert — `V_UUID_V4` war wohl ein früher Ansatz, der nie konsumiert wurde | Für den SQL-only-Backfill dieser Phase ist `randomblob(16)` (nackt) der einzig kompatible Weg — nicht `V_UUID_V4`, dessen Format nicht zu `BLOB(16)` passt. |

**Deprecated/outdated:** `V_UUID_V4`-View — technisch noch vorhanden (aus der
2024er Migration), aber unbenutzt. Nicht Teil dieser Phase, ihn zu entfernen
(out of scope), aber **nicht** als Vorbild für den Backfill heranziehen.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|-----------------|
| A1 | SQLite erlaubt kein `UNIQUE` direkt in `ALTER TABLE ... ADD COLUMN` (D-60-05-Begründung). | `## Standard Stack` / Pattern 1 | Niedrig — dies ist eine seit Jahren stabile, oft dokumentierte SQLite-Einschränkung (in dieser Session nicht per Tool gegen die SQLite-Doku gegengeprüft, da die Research-Scope-Vorgabe explizit keine externe Websuche verlangt und CONTEXT.md diese Begründung bereits als Scout-Befund führt). Falls falsch, wäre die Migration ohnehin funktional identisch (Index separat statt inline) — kein Verhaltensrisiko, nur eine unnötige Umschreibung. |
| A2 | Der Marker-Split-Ansatz (`-- BACKFILL START`/`-- BACKFILL END`) für den Apply-twice-Test ist die robustere von zwei möglichen Umsetzungen (Alternative: `CREATE TABLE IF NOT EXISTS` + Re-Run der ganzen Datei). | `## Mechanische Fragen — Frage 3` | Niedrig — beide Wege sind mit den verifizierten sqlx-Primitiven korrekt; falls der Planer den vollen Datei-Replay bevorzugt, funktioniert das ebenfalls, solange Block 1+2 (`CREATE TABLE IF NOT EXISTS person`, kein `IF NOT EXISTS` für `ADD COLUMN`) entsprechend defensiv geschrieben sind — was mehr Sorgfalt erfordert als der Marker-Split. Reine Implementierungsentscheidung, nicht CONTEXT-gesperrt. |

**Bewertung:** Beide Einträge sind Implementierungsdetails mit niedrigem Risiko,
nicht Kernaussagen über Produktverhalten oder Datenintegrität — die
sicherheits-/korrektheitskritischen Aussagen (Transaktionsverhalten,
`raw_sql`-Mechanik, ID-Format, `PRAGMA foreign_keys`-Abwesenheit,
`CURRENT_SNAPSHOT_SCHEMA_VERSION`-Wert) sind alle `[VERIFIED]` am Quelltext.

## Open Questions

1. **Marker-Split vs. voller Datei-Replay für den Apply-twice-Test**
   - What we know: Beide Wege sind mit `sqlx::raw_sql()` technisch korrekt (siehe
     Assumptions Log A2).
   - What's unclear: Welcher für die konkrete Migrationsdatei-Struktur, die der
     Planer wählt, weniger Wartungsaufwand bedeutet.
   - Recommendation: Marker-Split (`-- BACKFILL START`/`END`-Kommentare), weil er
     unabhängig davon funktioniert, ob Block 1+2 `IF NOT EXISTS`-sicher sind —
     geringeres Kopplungsrisiko zwischen Schema-DDL-Stil und Testmechanik.

2. **Ablage des Apply-twice-/Fixture-Testmoduls**
   - What we know: CONTEXT.md lässt explizit offen zwischen
     `service_impl/src/test/` (Präzedenz `rebooking_batch_state.rs`) und
     `dao_impl_sqlite/`.
   - What's unclear: Ob ein `PersonDao` in Phase 60 überhaupt existiert (laut
     D-60-10 nein — kein DAO-Eingriff in dieser Phase) — das spricht dafür, den
     Test NICHT in `dao_impl_sqlite/` zu verankern (dort gäbe es noch keine
     Entity-Struct, gegen die getestet werden könnte).
   - Recommendation: `service_impl/src/test/person_backfill.rs`, rein gegen rohes
     SQL über `sqlx::SqlitePool` (wie `absence_conversion.rs`-Fixtures), OHNE
     Abhängigkeit von einem noch nicht existierenden `PersonDao` — konsistent mit
     „kein DAO-Eingriff in Phase 60".

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|--------------|-----------|---------|-----------|
| SQLite (via `sqlx-sqlite` Bundled-Feature) | Migration + Tests | ✓ | via `sqlx` 0.8.6, `sqlite`-Feature (verifiziert `dao_impl_sqlite/Cargo.toml:20`) | — |
| `sqlx-cli` | `sqlx migrate add`/`sqlx migrate run` lokal | ✓ (laut CLAUDE.md/STATE.md, über `nix develop`) | 0.8.6 (Nix-Store-Derivation verifiziert: `sqlx-cli-0.8.6.drv`) | — |
| `.sqlx/`-Offline-Cache | `SQLX_OFFLINE=true`-CI-Build | ✓ eingecheckt | zuletzt 2026-07-23 aktualisiert | Bei Bedarf `cargo sqlx prepare --workspace` neu generieren (nur falls `query_as!` gegen `person` genutzt wird, siehe Frage 7). |

**Missing dependencies with no fallback:** keine.
**Missing dependencies with fallback:** keine — alle benötigten Werkzeuge sind
bereits vorhanden und versioniert verifiziert.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust-native) + `sqlx::migrate!`-basierte Integrationstests gegen echte In-Memory-SQLite (`sqlite::memory:`) |
| Config file | keiner — Workspace-`Cargo.toml`; SQLx-Config implizit über `DATABASE_URL`/`SQLX_OFFLINE` |
| Quick run command | `SQLX_OFFLINE=true cargo test -p service_impl person_backfill` |
| Full suite command | `SQLX_OFFLINE=true cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|----------------------|--------------|
| PERS-01 (SC1) | Nach Migration hat jede `sales_person`-Zeile (inkl. soft-gelöscht) und jede `user`-Zeile ein nicht-leeres `person_id`; verknüpfte Paare zeigen auf dieselbe `person.id`; Zählprobe `COUNT(person) == COUNT(sales_person) + COUNT(user) − COUNT(sales_person_user)` hält. | integration | `SQLX_OFFLINE=true cargo test -p service_impl person_backfill::full_identity_coverage` | ❌ Wave 0 |
| PERS-02 (SC2) | Zweiter Lauf der Backfill-SQL gegen dieselbe Fixture-DB ändert weder Zeilenzahl in `person` noch eine Zuordnung. | integration (Apply-twice via `sqlx::raw_sql`) | `SQLX_OFFLINE=true cargo test -p service_impl person_backfill::apply_twice_is_noop` | ❌ Wave 0 |
| PERS-03 (SC3) | Soft-gelöschte Sales Person mit `billing_period_sales_person`-Zeile hat nach Migration ein `person_id`. | integration mit dediziertem Fixture | `SQLX_OFFLINE=true cargo test -p service_impl person_backfill::soft_deleted_sales_person_gets_person_id` | ❌ Wave 0 |
| PERS-04 (SC4) | Direkter SQL-Schreibversuch, der zwei `sales_person`- (oder zwei `user`-) Zeilen auf dieselbe `person.id` zeigen lässt, wird von der DB abgelehnt. | integration (DB-Level-Ablehnung, kein Applikations-Check) | `SQLX_OFFLINE=true cargo test -p service_impl person_backfill::duplicate_person_id_rejected_by_db` | ❌ Wave 0 |
| PERS-06 (SC5) | `sales_person.id`/`user.name` bitgleich wie vorher; komplette Backend-Testsuite bleibt grün; `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13. | regression (bestehende Suite) + grep-Lock | `SQLX_OFFLINE=true cargo test --workspace` und `grep -n "CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13" service_impl/src/billing_period_report.rs` | ✓ (bestehende Suite), grep-Check neu |

### Sampling Rate

- **Per task commit:** `SQLX_OFFLINE=true cargo test -p service_impl person_backfill`
- **Per wave merge:** `SQLX_OFFLINE=true cargo test --workspace` + `cargo clippy --workspace -- -D warnings`
- **Phase gate:** Full suite grün vor `/gsd-verify-work` — zusätzlich manuelles
  `sqlx migrate run` gegen eine Kopie der `localdb.sqlite3` mit realistischem
  Datenbestand (falls verfügbar) als Vorab-Sightcheck, weil dies eine
  produktionsnahe Migration mit irreversiblem Charakter ist (D-60-01
  „one-way").

### Wave 0 Gaps

- [ ] `migrations/sqlite/20260916HHMMSS_create-person-backfill.sql` — die Migration
      selbst (Datei existiert noch nicht).
- [ ] `service_impl/src/test/person_backfill.rs` — neues Testmodul, alle vier
      Testfälle oben; kein bestehendes File deckt das ab.
- [ ] Registrierung des neuen Testmoduls in `service_impl/src/test/mod.rs` (bzw.
      wo auch immer `rebooking_batch_state` registriert ist — Muster
      übernehmen).
- [ ] Kein Framework-Install nötig — `sqlx`/`tokio`/`uuid` sind bereits
      Dev-Dependencies.

## Security Domain

> `.planning/config.json` verifiziert (gelesen in dieser Session): kein
> `security_enforcement`-Schlüssel gesetzt → per Default aktiv. Ebenso ist
> `workflow.nyquist_validation` nicht gesetzt → per Default aktiv, daher ist die
> `## Validation Architecture`-Sektion oben verpflichtend enthalten.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|----------------|---------|---------------------|
| V2 Authentication | nein | Diese Phase berührt keinen Auth-Pfad — reine Schema-/Backfill-Migration, kein DAO-/Service-Code (D-60-10). |
| V3 Session Management | nein | Unberührt. |
| V4 Access Control | **indirekt, vorbereitend** | Die 1:1-Garantie (`CREATE UNIQUE INDEX`) ist die Datengrundlage, auf der Phase 61 die Selbst-Zugriffs-Prüfung `find_sales_person_by_user_id` umschreibt (PITFALLS Pitfall 4). Phase 60 selbst ändert keinen Zugriffspfad, legt aber die Integritätsbedingung, ohne die ein „fail-open über NULL == NULL"-Fehler in Phase 61 möglich würde. |
| V5 Input Validation | nein | Kein Nutzereingabe-Pfad in dieser Phase (Migration läuft serverseitig beim Start, kein API-Input). |
| V6 Cryptography | nein | `randomblob(16)` dient hier als UUID-Quelle, nicht als kryptografischer Zufall — dieselbe Verwendung wie das bestehende, ebenfalls nicht-kryptografische `Uuid::new_v4()`-Muster im Repo für Entity-IDs (keine Security-relevante Eigenschaft dieser IDs; sie sind niemals Secrets). |

### Known Threat Patterns for SQLite-Migrationen

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|-------------------------|
| Stille Doppel-Identität (zwei Sales Persons/Users teilen sich eine `person_id`) | Tampering (Datenintegrität) | `CREATE UNIQUE INDEX` auf beiden Spalten, DB-Level-Ablehnungstest (PERS-04/SC4) — bereits Kern dieser Phase. |
| Partiell durchgelaufener Backfill auf Produktions-DB (Prozess-Crash mitten in der Transaktion) | Tampering / Denial of Service (Server startet nicht mehr) | Per-Datei-Transaktion (sqlx-Standardverhalten, verifiziert) + Selbst-Assertion (D-60-07) — ein partieller Lauf wird automatisch zurückgerollt, nie committet. |

## Sources

### Primary (HIGH confidence — in dieser Session am Code/Quelltext verifiziert)

- `migrations/sqlite/20240506114107_add_sales_person.sql` — `sales_person`- und
  `sales_person_user`-DDL, beide `UNIQUE`-Constraints, komplett gelesen.
- `migrations/sqlite/20240426150045_user-roles.sql` — `user`-DDL (kein
  Soft-Delete), `V_UUID_V4`-View, komplett gelesen.
- `migrations/sqlite/20260707000000_create-rebooking-batch.sql`,
  `20260707000001_add-source-column-to-extra-hours.sql`,
  `20260503000001_create-absence-period-migration-source.sql`,
  `20260503000000_create-absence-migration-quarantine.sql`,
  `20260502170000_create-absence-period.sql`,
  `20260629000000_create-vacation-entitlement-offset.sql`,
  `20260702000000_create-week-status.sql`,
  `20260703000000_create-pdf-export-config.sql` — Audit-Spalten-Auszählung,
  Idempotenz-Kommentarkonvention, `IF NOT EXISTS`-Stil, Single-Row-Seed-Muster.
- `service_impl/src/test/rebooking_batch_state.rs:1-120` — Integrationstest-
  Harness gegen echte In-Memory-SQLite via `sqlx::migrate!`.
- `service_impl/src/test/absence_conversion.rs:540-610` — Fixture-Insert-Stil
  per rohem SQL für `sales_person`/`extra_hours`.
- `shifty_bin/src/main.rs:1644-1654`, `shifty_bin/src/integration_test.rs:268-286`
  — die beiden `sqlx::migrate!`-Aufrufstellen.
- `dao_impl_sqlite/src/sales_person.rs:22-45,120-138` — ID-Serialisierungsformat
  (`Uuid::as_bytes()`/`Uuid::from_slice()`), `create`-Insert-Muster.
- `dao_impl_sqlite/src/lib.rs:93-96,282-299` — `create_user`, `clear_all`
  (Multi-Statement-`query!`-Nutzung, bestätigt SQLite-spezifisches
  Multi-Statement-Verhalten).
- `service_impl/src/billing_period_report.rs:140` —
  `CURRENT_SNAPSHOT_SCHEMA_VERSION = 13`.
- `Cargo.lock:3607-3610` — `sqlx` Version 0.8.6.
- `dao_impl_sqlite/Cargo.toml:18-20` — sqlx-Feature-Set `["runtime-tokio", "sqlite"]`.
- `.github/workflows/rust.yml:11` — `SQLX_OFFLINE: "true"` in CI.
- `.sqlx/` (Verzeichnisinhalt geprüft) — eingecheckter Offline-Cache, zuletzt
  2026-07-23 aktualisiert.
- **sqlx-0.8.6-Quelltext** (lokal am Cargo-Registry-Cache gelesen, KEINE
  Websuche): `sqlx-core-0.8.6/src/migrate/source.rs:117-137` (Transaktions-
  Default-Verhalten, `no-transaction`-Marker), `sqlx-sqlite-0.8.6/src/migrate.rs:131-162`
  (Per-Datei-Transaktions-Apply-Logik, Rollback-bei-Fehler), `sqlx-core-0.8.6/src/raw_sql.rs`
  (vollständige API- und Verhaltensdokumentation von `sqlx::raw_sql()`).
- `docs/features/F01-employee-management.md` (Gliederung geprüft, Abschnitt
  „3. Data model" Zeilen 180-245) — Docs-Freshness-Ziel für diese Phase.
- `grep -rn "PRAGMA foreign_keys"` über das gesamte Repo — 0 Treffer,
  D-60-06 gegengeprüft.
- `grep -rn "V_UUID_V4"` über das gesamte Repo — nur die eigene
  `CREATE VIEW`-Definition, kein Konsument.

### Secondary (MEDIUM confidence)

- `.planning/research/PITFALLS.md` (Pitfalls 1-3) — bereits mit Repo-Zitaten
  fundierte Milestone-Vorrecherche, in dieser Session gegen den tatsächlichen
  Code re-verifiziert (keine neue Websuche nötig, da bereits codebasiert).
- `.planning/research/ARCHITECTURE.md` — Build-Order-Empfehlung
  (Migration+Backfill zuerst), in dieser Session nicht erneut gegen Code
  geprüft, da außerhalb des Phase-60-Scopes (betrifft primär Phase 61 DI).

### Tertiary (LOW confidence)

- Keine — diese Recherche verwendet ausschließlich Primary/Secondary-Quellen
  gemäß der expliziten Scope-Vorgabe „keine Websuche".

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — keine neuen Dependencies, sqlx-Version am Cargo.lock verifiziert.
- Architecture (Migrations-/Transaktionsverhalten, `raw_sql`-Mechanik): HIGH — am sqlx-0.8.6-Quelltext gelesen, nicht aus Trainingswissen behauptet.
- Backfill-SQL-Korrelationslogik (Block 3a-3e): HIGH für die Korrektheit der Herleitung (basiert auf verifizierten SQLite-Primitiven: `UPDATE`-Spaltenausdrücke werden pro Zeile ausgewertet, korrelierte Subqueries, bestehende `UNIQUE`-Constraints), aber **ohne Repo-Präzedenz für exakt diese Sequenz** — der Planer/Executor sollte sie über den Apply-twice-Test UND einen frischen `cargo test`-Lauf verifizieren, bevor sie als final gilt.
- Pitfalls: HIGH — direkt aus Repo-Code (Soft-Delete-Spalten-Asymmetrie, fehlende FK-Enforcement) abgeleitet, nicht aus generischem SQLite-Wissen.
- Audit-Spalten-Dominanz (Frage 1): HIGH — vollständig ausgezählt über alle seit Mai 2026 neu angelegten Tabellen (5 von 5 folgen dem `created`/`deleted`/`update_process`/`update_version`-Muster).

**Research date:** 2026-09-16
**Valid until:** Bis zur Ausführung dieser Phase (keine externen, sich schnell
ändernden Abhängigkeiten — sqlx-Version und Repo-Konventionen ändern sich
innerhalb eines Milestones erfahrungsgemäß nicht). Bei Verzögerung über
mehrere Wochen: `Cargo.lock`-sqlx-Version und `migrations/sqlite/`-Verzeichnis
erneut auf neue Einträge prüfen (z. B. falls Phase 58/59 zwischenzeitlich doch
promotet werden und neue Migrationen hinzufügen, die die Audit-Spalten-Zählung
in Frage 1 verschieben könnten).

---
*Research for: Phase 60 (Person-Datenmodell + Backfill), Milestone v2.7*
*Researched: 2026-09-16*
