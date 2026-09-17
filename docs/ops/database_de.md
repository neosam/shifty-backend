# Datenbank — Migrations, Cache, Betrieb

## SQLite — warum und wann nicht

Shifty läuft produktiv auf SQLite. Vorteile:

- Deploy als einzelne Datei — kein separater DB-Server.
- Test-Isolation trivial (In-Memory-DB).
- Reproduzierbare Migrations mit `sqlx`.

Grenzen:

- **Single-Writer.** Zwei parallele Schreib-Requests werden serialisiert.
- **Kein echtes Concurrent-Read-Write** ohne WAL-Mode. Wenn Load steigt,
  kann Nachdenken über Postgres nötig werden — aber die DAO-Schicht ist
  Trait-basiert, ein Umzug wäre ein DAO-Impl-Austausch, keine
  Service-Änderung.

## Migrations

**Speicherort:** `shifty-backend/migrations/sqlite/`.
**Format:** `YYYYMMDDHHMMSS_<name>.sql` — chronologisch sortiert.

### Neue Migration anlegen

```bash
nix develop
sqlx migrate add --source migrations/sqlite <name-in-kebab-case>
```

Das erzeugt eine leere `.sql`-Datei mit korrektem Timestamp.

### Migration ausführen

**Inkrementell (Dev + Prod):**

```bash
sqlx migrate run --source migrations/sqlite
```

Führt alle noch nicht angewandten Migrations aus.

**Destruktiv (nur Dev, wenn DB neu):**

```bash
sqlx database reset --source migrations/sqlite
```

⚠️ **Löscht die DB komplett.** Nie in Prod. Für lokale Dev-DB ok, aber
Konfirmation vom User einholen, bevor du das ausführst (Memory-Regel:
destruktive DB-Ops brauchen immer explizite Bestätigung).

## sqlx-Offline-Cache (`.sqlx/`)

CI läuft mit `SQLX_OFFLINE=true`. SQLx greift dann auf den Query-Cache
im `.sqlx/`-Verzeichnis zurück, statt eine echte DB zu brauchen.

**Regel:** Nach jeder neuen `query!`/`query_as!`-Verwendung MUSS

```bash
cargo sqlx prepare --workspace
```

laufen. Das aktualisiert den Cache. Der Cache muss mitcommittet werden.

**Wenn du das vergisst:**

- Lokaler inkrementeller Build ist grün (Cache noch da).
- Clean-Build failt.
- `cargo test --doc` failt.
- CI failt.

Phase 33 hat das schmerzlich mit "wieso ist CI rot" gefunden.

## Backup

**[Zu prüfen]** — Backup-Strategie in Prod. Bei SQLite typisch:
`.backup`-Kommando via `sqlite3`-CLI oder Datei-Copy bei stillstehender
DB (nur wenn keine WAL-Files aktiv).

Für kritische Deployments: Regelmäßiges Kopieren der `.sqlite`-Datei
(inklusive `-wal` und `-shm`) mit anschließender Konsistenzprüfung.

## Foreign Keys

SQLite prüft FKs nur, wenn `PRAGMA foreign_keys = ON` gesetzt ist.
**Verifiziert (2026-09-17, Plan 60-04): es ist an.** Shiftys eigener Code
setzt das Pragma nirgends, sqlx aber schon —
`SqliteConnectOptions::default()` trägt `foreign_keys = ON` in die
Pragma-Menge ein, die auf jede Verbindung angewendet wird
(`sqlx-sqlite-0.8.6/src/options/mod.rs:185`), und
`shifty_bin/src/main.rs:1645` verbindet über eine schlichte
`SqlitePool::connect("sqlite:./localdb.sqlite3")`-URL, die das nicht
überschreibt. Ein Grep nach `foreign_keys` im Repo liefert deshalb nichts und
ist **kein** Beleg dafür, dass die Durchsetzung aus wäre.

Nachprüfen an einer laufenden Datei:

```sql
PRAGMA foreign_keys;              -- muss auf einer App-Verbindung 1 liefern
PRAGMA foreign_key_check;         -- muss null Zeilen liefern
```

Betriebliche Konsequenz: Ein Delete, das eine Kindzeile verwaisen ließe,
scheitert mit `FOREIGN KEY constraint failed` (SQLite-Code 787), statt still
eine verwaiste Zeile zu erzeugen — relevant z. B. beim Admin-Benutzer-Delete,
das hart aus `user` löscht.

Der verbleibende blinde Fleck ist der Soft Delete: eine als `deleted`
markierte Zeile existiert weiterhin, die FK-Prüfung bleibt also erfüllt,
während jede Live-Query mit `WHERE deleted IS NULL` einen ins Leere zeigenden
Verweis sieht. Diese Klasse von Invarianten schützt die FK-Durchsetzung nicht
— ein Migration-Review sollte weiterhin prüfen, dass neue Tabellen sie
explizit ausdrücken.

## Views

Manche Aggregat-Reads gehen über Views:

- `bookings_view` — Basis-View, mehrfach neu erzeugt bei Feld-
  Änderungen (2024-07-28 initial, 2025-01-15 User-Tracking,
  2026-03-30 Multi-Plan).

**Regel:** Wenn du eine Spalte auf einer Tabelle änderst, prüfe, ob
Views draufliegen. Die müssen dann neu erzeugt werden (in einer
Migration).

## Soft-Delete-Konvention

Fast jede Tabelle hat `deleted` (nullable timestamp). Reader filtern
`WHERE deleted IS NULL`. Das ist Konvention, kein Constraint.

Bei neuen Queries im Review: **`deleted IS NULL` prüfen**.

Siehe [`../domain/edge-cases.md#8-soft-delete-konsistenz`](../domain/edge-cases.md#8-soft-delete-konsistenz).

## `PRAGMA journal_mode = WAL`

**[Zu prüfen]** — ob WAL-Mode aktiv ist. WAL erhöht Concurrent-Read-
While-Write-Fähigkeit. Ohne WAL blockieren lange Reads Writer.

## Debugging & Ad-hoc-Queries

Für schnelle Inspektion:

```bash
sqlite3 <pfad-zur-db.sqlite>
> .schema sales_person
> SELECT * FROM sales_person WHERE deleted IS NULL LIMIT 5;
> .quit
```

**Vorsicht:** Direkte Writes gegen die DB umgehen Business-Logic und
Snapshot-Semantik. Wenn du für Datenfix schreiben MUSST: expliziter
Kommentar im Change-Log, TX benutzen, hinterher Reporting-Konsistenz
prüfen.
