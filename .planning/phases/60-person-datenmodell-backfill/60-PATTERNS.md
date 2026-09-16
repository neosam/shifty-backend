# Phase 60: Person-Datenmodell + Backfill - Pattern Map

**Mapped:** 2026-09-16
**Files analyzed:** 5 (backend-only, siehe scope_hint — kein DAO/Service/REST/Frontend)
**Analogs found:** 5 / 5

## File Classification

| Neue/geänderte Datei | Rolle | Data Flow | Nächster Analog | Match-Qualität |
|---|---|---|---|---|
| `migrations/sqlite/<ts>_create-person-backfill.sql` (neu) | migration | batch (DDL + einmaliger SQL-Backfill) | `migrations/sqlite/20260707000000_create-rebooking-batch.sql` (Struktur/Header) + `20240506114107_add_sales_person.sql` (bestehendes Ziel-Schema) | exact (Struktur) / exact (Backfill-Idempotenz-Konvention aus `20260503000001`) |
| `service_impl/src/test/person_backfill.rs` (neu) | test (Integrationstest) | batch / CRUD-Verification | `service_impl/src/test/rebooking_batch_state.rs` (Pool-Harness) + `service_impl/src/test/absence_conversion.rs` (Fixture-Insert-Stil) | role-match (kein exaktes Vorbild für DAO-loses Testmodul, aber gleiche Harness-Bausteine) |
| `service_impl/src/test/mod.rs` (geändert) | config (Modul-Registry) | — | `service_impl/src/test/mod.rs` selbst (bestehende Zeilen `pub mod rebooking_batch_state;` etc.) | exact |
| `docs/architecture/03-data-model.md` + `_de.md` | docs | — | Bestehende Datei, Abschnitt „Core aggregates" | exact |
| `docs/architecture/diagrams/db-schema-er.mmd` | docs (Mermaid ER) | — | Bestehender `sales_person`-Block im selben File | exact |
| `docs/features/F01-employee-management.md` + `_de.md` | docs | — | Abschnitt „3. Data model" (Zeilen ~180–245 laut RESEARCH.md) | exact |

**Explizit kein Analog gesucht** (D-60-10 / scope_hint): DAO-Trait, Service-Impl, REST-Handler, TOs — diese Phase erzeugt keine.

---

## Pattern Assignments

### `migrations/sqlite/<ts>_create-person-backfill.sql` (migration, batch)

**Analoga:** `migrations/sqlite/20260707000000_create-rebooking-batch.sql` (Header-Konvention, `CREATE TABLE IF NOT EXISTS`, Audit-Spalten, `CREATE UNIQUE INDEX`),
`migrations/sqlite/20260707000001_add-source-column-to-extra-hours.sql` (`ALTER TABLE … ADD COLUMN` mit Begründungskommentar),
`migrations/sqlite/20260503000001_create-absence-period-migration-source.sql` (Idempotenz-Kommentar-Konvention),
`migrations/sqlite/20240506114107_add_sales_person.sql` (Ziel-Schema: `sales_person`, `sales_person_user` mit beiden `UNIQUE`-Constraints).

**Header-/Kommentar-Konvention** (`20260707000000...sql:1-7`):
```sql
-- Phase 54 (VOL-STAT / VOL-ACCT — Data-Model-Basis).
-- Rebooking-Batch Parent + Entry Child; Konsumenten (F3 manual / F4 cron /
-- F5 alert) folgen ab Phase 55/56. Der UNIQUE-Partial-Index enforced
-- (sales_person_id, iso_year, iso_week) global ueber alle kinds
-- (D-54-DM-01: Claim-on-Suggest — ...).
-- Soft-Delete-Konvention (WHERE deleted IS NULL, kein FK ON DELETE CASCADE).
```
→ Für Phase 60 analog: Header muss **explizit D-60-01** referenzieren (Grund für den vollständigen, nicht-partiellen Index — genau der Fall, den dieser Kommentar-Stil sonst für partielle Indizes dokumentiert).

**`CREATE TABLE IF NOT EXISTS` + Audit-Spalten (2026er Muster ohne `deleted`)** (`20260707000000...sql:9-22`, reduziert um `deleted` gemäß D-60-04 und um `sales_person_id`/Domain-Spalten):
```sql
CREATE TABLE IF NOT EXISTS rebooking_batch (
    id BLOB NOT NULL PRIMARY KEY,
    ...
    created TEXT NOT NULL,
    deleted TEXT,
    update_process TEXT NOT NULL,
    update_version BLOB NOT NULL
);
```
Für `person` (D-60-04: **kein** `deleted`): `id BLOB(16) NOT NULL PRIMARY KEY, created TEXT NOT NULL, update_process TEXT NOT NULL, update_version BLOB NOT NULL`. Der ID-Spaltentyp folgt dem älteren `sales_person`-Stil `blob(16)` (`20240506114107...sql:4`) statt dem neueren nackten `BLOB` (`rebooking_batch`) — beide sind in SQLite äquivalent (dynamisches Typsystem), RESEARCH.md Frage 4 verifiziert nur das Byte-Format (16 rohe Bytes), nicht die Spalten-Typ-Schreibweise; `blob(16)` ist näher an `sales_person.id`, dem direkten FK-Ziel.

**`ALTER TABLE … ADD COLUMN` mit Begründungskommentar** (`20260707000001...sql:1-6`, vollständig):
```sql
-- Phase 54 (D-54-DM-02): Rebooking-Marker auf extra_hours.
-- Bestehende Rows bekommen per DEFAULT 'manual' (kein Backfill-Sweep noetig).
-- Werte-Konvention: 'manual' (Standard) | 'rebooking' (Marker fuer F1/F2-Filter).
-- Konsumenten (F1-Ist / F2-Soll Aggregatoren) filtern ab Phase 54 auf
-- source='manual'; Rebooking-Schreiber folgen ab Phase 55.
ALTER TABLE extra_hours
    ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';
```
→ Unterschied zu Phase 60: `person_id` bekommt **kein** `DEFAULT`, ist `NULL`-fähig (D-60-03) und braucht **einen SQL-Backfill danach**, nicht nur einen statischen Default — das ist der Kern-Unterschied zu diesem Analog.

**Idempotenz-Kommentar-Konvention, wörtlich** (`20260503000001...sql:2`):
```sql
-- Idempotency key: extra_hours_id PK. Re-run skips already-mapped rows.
```
→ Für Phase 60 (D-60-08), analoger Kommentar über jedem Backfill-Block:
```sql
-- Idempotency key: person_id IS NULL. Re-run skips already-linked rows.
```

**Bestehendes Ziel-Schema, vollständig** (`20240506114107...sql:1-24`):
```sql
CREATE TABLE sales_person (
    id blob(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    inactive BOOLEAN NOT NULL,
    deleted TEXT,
    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    update_version blob(16) NOT NULL
);

CREATE TABLE sales_person_user (
    sales_person_id blob(16) NOT NULL,
    user_id TEXT NOT NULL,
    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    PRIMARY KEY (sales_person_id, user_id),
    UNIQUE(sales_person_id),
    UNIQUE(user_id),
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
    FOREIGN KEY (user_id) REFERENCES user(name)
)
```
`user` (`20240426150045...sql:2-6`, kein Soft-Delete):
```sql
CREATE TABLE user (
    name TEXT NOT NULL PRIMARY KEY,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);
```
→ `ALTER TABLE sales_person ADD COLUMN person_id BLOB(16);` und `ALTER TABLE user ADD COLUMN person_id BLOB(16);` zielen auf diese beiden Tabellen.

**Explizite Abweichung vom dominanten Index-Muster (D-60-01)** — Kontrast Analog vs. Phase 60:
```sql
-- Dominantes Repo-Muster (rebooking_batch_week_unique_idx, 20260707000000...sql:42-44):
CREATE UNIQUE INDEX IF NOT EXISTS rebooking_batch_week_unique_idx
    ON rebooking_batch (sales_person_id, iso_year, iso_week)
    WHERE deleted IS NULL;   -- PARTIELL

-- Phase 60 (D-60-01) — bewusst OHNE WHERE-Klausel, mit erklärendem Kommentar:
-- D-60-01: bewusst KEIN "WHERE deleted IS NULL" — soft-geloeschte Sales
-- Persons behalten ihre Person-Identitaet dauerhaft (Produktentscheidung,
-- siehe 60-CONTEXT.md D-60-01). NICHT beim naechsten Aufraeum-Pass "reparieren".
CREATE UNIQUE INDEX idx_sales_person_person_id
    ON sales_person (person_id);   -- VOLLSTAENDIG, nicht partiell
```

**Vollständiges Backfill-SQL** (aus RESEARCH.md `## Code Examples`, dort bereits fertig hergeleitet — 5 Blöcke 3a-3e, wörtlich übernehmen, siehe RESEARCH.md Zeilen "Block 3: Backfill"). Kernidee: unverknüpfte Seiten bekommen eigenen `randomblob(16)`, die User-Seite eines verknüpften Paars **kopiert** den auf der Sales-Person-Seite erzeugten Wert über eine korrelierte Subquery (kein zweiter `randomblob()`-Aufruf), abschließendes `INSERT INTO person ... SELECT DISTINCT ... UNION ...` dedupliziert.

**Selbst-Assertion (D-60-07)**, aus RESEARCH.md Pattern 3, wörtlich übernehmbar:
```sql
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

**Fehlerbehandlung:** keine explizite REST-/Service-Fehlerbehandlung nötig (reines SQL). Die "Fehlerbehandlung" dieser Migration ist ausschließlich das Transaktions-Rollback-Verhalten von `sqlx::migrate!` (RESEARCH.md Pattern 2, am sqlx-0.8.6-Quelltext verifiziert) — kein Code zu kopieren, nur zu wissen.

---

### `service_impl/src/test/person_backfill.rs` (test, integration)

**Analoga:** `service_impl/src/test/rebooking_batch_state.rs` (Pool-Harness gegen echte In-Memory-SQLite), `service_impl/src/test/absence_conversion.rs` (Fixture-Insert per rohem SQL).

**Pool-Setup-Muster** (`rebooking_batch_state.rs:35-45`, vollständig):
```rust
async fn setup_pool() -> Arc<sqlx::SqlitePool> {
    let pool = Arc::new(
        sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Could not connect to in-memory SQLite"),
    );
    sqlx::migrate!("./../migrations/sqlite")
        .run(pool.as_ref())
        .await
        .expect("Could not run migrations");
    pool
}
```
→ Für `person_backfill.rs` identisch übernehmen (Pfad-Präfix `./../migrations/sqlite` ist relativ zu `service_impl/`, unverändert).

**Wichtige Abweichung vom Analog:** `rebooking_batch_state.rs` baut zusätzlich `IntegrationDeps`/`RebookingBatchServiceDeps` auf, weil dort ein echter Service getestet wird. `person_backfill.rs` braucht laut D-60-10 **keinen** Service/DAO — nur `pool.as_ref()` + rohes `sqlx::query`/`sqlx::raw_sql`. Diesen Teil (Zeilen 47-59 des Analogs mit `IntegrationDeps`/`MockPermissionService`/`build_perm`) **nicht** übernehmen.

**Fixture-Insert-Stil per rohem SQL** (`absence_conversion.rs:558-568`, `sales_person`-Seed, wörtlich):
```rust
sqlx::query(
    "INSERT INTO sales_person (id, name, inactive, deleted, update_process, update_version) \
     VALUES (?1, ?2, 0, NULL, 'seed', ?3)"
)
.bind(sales_person_id.as_bytes().to_vec())
.bind("Test Person")
.bind(Uuid::new_v4().as_bytes().to_vec())
.execute(pool.as_ref())
.await
.expect("seed sales_person");
```
→ Für PERS-03/SC3 (soft-gelöschte Sales Person) den `deleted`-Bind auf einen konkreten Timestamp statt `NULL` setzen, wie RESEARCH.md im Abschnitt "Fixture-Insert-Stil für Tests" bereits vorführt:
```rust
.bind("2025-01-01T00:00:00")   // deleted <> NULL fuer den PERS-03-Fixture-Fall
```

**Apply-twice-Mechanik** (aus RESEARCH.md `## Mechanische Fragen — Frage 3`, dort bereits vollständig hergeleitet, **kein** Repo-Analog vorhanden — hier wörtlich das Muster):
```rust
let migration_sql = include_str!(
    "../../../migrations/sqlite/20260916HHMMSS_create-person-backfill.sql"
);
sqlx::raw_sql(migration_sql)
    .execute(pool.as_ref())
    .await
    .expect("zweiter Lauf darf nicht fehlschlagen");
```
Wichtig: `sqlx::migrate!().run()` ein zweites Mal funktioniert **nicht** (Ledger überspringt bereits angewandte Migration) — `sqlx::raw_sql()` gegen den Datei-Inhalt ist der einzig korrekte Weg (RESEARCH.md Anti-Patterns).

**DB-Level-Ablehnungstest (PERS-04/I-6):** kein Analog im Repo (FK-basierte Tests existieren, aber D-60-06 verbietet genau dieses Muster). Direkt zwei `INSERT`/`UPDATE`-Statements mit identischem `person_id`-Wert gegen `sales_person` fahren und `.execute(...).await` als `Err(...)` erwarten (SQLite gibt `SQLITE_CONSTRAINT_UNIQUE` zurück, was `sqlx::Error::Database` wird) — Analog-Stil für die `Err`-Assertion selbst gibt es in keinem gelesenen Testfile, folgt aber direkt aus Standard-`sqlx`-Fehlerbehandlung (`.is_err()` bzw. `matches!`).

**Testmodul-Kopfkommentar-Stil** (`rebooking_batch_state.rs:1-18`) — Konvention: Phase-Referenz + Requirement-IDs + kurze Testziel-Beschreibung im Doc-Comment über den Imports; für `person_backfill.rs` analog mit PERS-01..06-Referenzen und den vier Testfall-Bereichen aus der Verification Map (I-1..I-12) einleiten.

---

### `service_impl/src/test/mod.rs` (config, Modul-Registrierung)

**Analog:** die Datei selbst, bestehende Registrierungszeilen für Testmodule ähnlicher Provenienz (z. B. Zeile 40 `pub mod rebooking_batch_state;`, Zeile 52 `pub mod rebooking_backfill;`).

**Pattern** (`mod.rs:39-40`, wörtlich):
```rust
#[cfg(test)]
pub mod rebooking_batch;
#[cfg(test)]
pub mod rebooking_batch_state;
```
→ Neue Zeile alphabetisch/thematisch einsortieren (Vorschlag: nach `pdf_shiftplan`/vor `permission_test`, oder ans Ende — Liste ist nicht strikt alphabetisch sortiert, siehe bestehende Reihenfolge):
```rust
#[cfg(test)]
pub mod person_backfill;
```

---

### Docs (`docs/architecture/03-data-model.md` + `_de.md`, `db-schema-er.mmd`, `docs/features/F01-employee-management.md` + `_de.md`)

**Analog:** bestehende Struktur derselben Dateien.

`03-data-model.md` Abschnitt „Core aggregates" (Zeilen 15-27, vollständig gelesen) listet Aggregate als Bullet-Liste mit Fettdruck-Namen, z. B.:
```markdown
- **Employee** — Sales Person + Work Details (Contract) + Unavailable + Vacation Offset.
```
→ `person` ist kein neues eigenständiges Aggregat, sondern eine Elterntabelle, die das Employee-Aggregat und Auth&Session-Aggregat verknüpft — Planer sollte entweder die `Employee`-Zeile erweitern oder einen neuen Satz "Person (v2.7, Phase 60) — Identitäts-Elterntabelle für Sales Person + User." ergänzen. Beide EN/DE-Varianten synchron halten (CLAUDE.md-Pflicht).

`db-schema-er.mmd`: bestehender `sales_person`-Eintrag (Zeile ~376-378 referenziert `sales_person` in Kommentaren) — Planer muss dort einen `person ||--o{ sales_person` / `person ||--o{ user`-Beziehungsblock im selben Mermaid-ER-Stil wie die übrigen Tabellen ergänzen (Format an bestehenden Einträgen im selben File orientieren, nicht neu erfinden).

`docs/features/F01-employee-management.md` Abschnitt „3. Data model" (laut RESEARCH.md Zeilen 180-245) ist der natürliche Ort für die neue `person`-Beschreibung samt der D-60-01/D-60-03-Produktentscheidungen (nullable, non-partial Index, kein Delete-Reflex) — inhaltlich siehe CONTEXT.md `<decisions>`, dort bereits vollständig als Prosa formuliert und direkt in Doku-Sprache übertragbar.

---

## Shared Patterns

### Migrations-Header-Konvention (Phase + Decision-ID)
**Quelle:** `migrations/sqlite/20260707000000_create-rebooking-batch.sql:1-7`
**Anwenden auf:** die neue Migrationsdatei — Header muss Phase 60, PERS-01..06 und explizit D-60-01 (nicht-partieller Index) referenzieren, analog wie der rebooking_batch-Header D-54-DM-01 referenziert.

### Idempotenz-Guard-Konvention (`WHERE <state> IS NULL`)
**Quelle:** `migrations/sqlite/20260503000001_create-absence-period-migration-source.sql:2` (Kommentar-Wortlaut) + RESEARCH.md Pattern 4
**Anwenden auf:** jeden der 5 Backfill-Blöcke in der neuen Migration — jeder Block bekommt einen Kommentar im exakt selben Stil wie das Repo-Original, nur mit `person_id IS NULL` statt `extra_hours_id PK`.

### Fixture-Insert per rohem SQL (kein `query_as!`)
**Quelle:** `service_impl/src/test/absence_conversion.rs:558-590`
**Anwenden auf:** `person_backfill.rs` — alle Fixtures (soft-gelöschte Sales Person, HR-User ohne Sales Person, Freiwilliger ohne User, verknüpftes Paar) werden per `sqlx::query("INSERT INTO ...").bind(...)` gesetzt, **nicht** über `query_as!`-Makros (RESEARCH.md Frage 7: kein Compile-Time-Schema-Wissen über `person` nötig, spart `.sqlx/`-Cache-Neugenerierung).

### In-Memory-SQLite-Pool-Harness
**Quelle:** `service_impl/src/test/rebooking_batch_state.rs:35-45`
**Anwenden auf:** `person_backfill.rs` — identisches `setup_pool()` (ohne den nachgelagerten `IntegrationDeps`-Teil, der hier nicht gebraucht wird).

---

## No Analog Found

| Datei/Aspekt | Rolle | Data Flow | Begründung |
|---|---|---|---|
| DB-Level-Doppel-Link-Ablehnungstest (Assertion-Stil `Err(...)` bei UNIQUE-Verletzung) | test | integration (negativ) | Kein bestehender Test im Repo prüft gezielt eine UNIQUE-Constraint-Verletzung auf Schreibebene (bestehende Tests prüfen meist Happy-Path oder Service-Level-Fehler); Planer muss Standard-`sqlx`-`Result::is_err()`-Assertion direkt aus `## Mechanische Fragen` / RESEARCH.md herleiten, kein Repo-Vorbild nötig, da trivial. |
| Backfill-Korrelationslogik (3a-3e, verknüpfte Paare teilen `person_id`) | migration (SQL) | batch | RESEARCH.md selbst bestätigt: „ohne Repo-Präzedenz für exakt diese Sequenz" — vollständig in RESEARCH.md `## Code Examples` hergeleitet, dort 1:1 übernehmbar. |

## Metadata

**Analog-Suchbereich:** `migrations/sqlite/*.sql` (8 Dateien gelesen/gegrept laut RESEARCH.md „Sources"), `service_impl/src/test/*.rs` (3 Dateien gelesen: `rebooking_batch_state.rs`, `absence_conversion.rs`, `mod.rs`), `docs/architecture/`, `docs/features/F01-employee-management.md`.
**Dateien gescannt:** 5 Migrationsdateien vollständig gelesen (`20260707000000`, `20260707000001`, `20260503000001`, `20240506114107`, `20240426150045`), 2 Testdateien teilweise gelesen (Header/Harness-Bereiche), `service_impl/src/test/mod.rs` vollständig gelesen.
**Pattern-Extraktion:** 2026-09-16

---
*Phase: 60-Person-Datenmodell + Backfill*
*Pattern mapping: 2026-09-16*
