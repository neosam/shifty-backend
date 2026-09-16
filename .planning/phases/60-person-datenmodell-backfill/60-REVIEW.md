---
phase: 60-person-datenmodell-backfill
reviewed: 2026-09-16T07:29:17Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - migrations/sqlite/20260916000000_create-person-backfill.sql
  - service_impl/src/test/person_backfill.rs
  - service_impl/src/test/mod.rs
  - docs/architecture/03-data-model.md
  - docs/architecture/03-data-model_de.md
  - docs/architecture/diagrams/db-schema-er.mmd
  - docs/features/F01-employee-management.md
  - docs/features/F01-employee-management_de.md
  - docs/features/F12-auth-session.md
  - docs/features/F12-auth-session_de.md
findings:
  critical: 3
  warning: 6
  info: 5
  total: 14
status: issues_found
---

# Phase 60: Code Review Report

**Reviewed:** 2026-09-16T07:29:17Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

Geprüft wurden die Migration `20260916000000_create-person-backfill.sql`, die
16 Integrationstests in `service_impl/src/test/person_backfill.rs`, deren
Einhängung in `mod.rs` sowie die sieben Doku-Dateien (EN/DE-Paare + ER-Diagramm).
`cargo test -p service_impl person_backfill` läuft grün (16/16),
`cargo clippy -p service_impl --all-targets -- -D warnings` ist sauber.

Die im Review-Fokus explizit benannten Punkte sind **korrekt umgesetzt** und
wurden gegen eine echte SQLite-Instanz nachgefahren: Block 3d kopiert den Wert
tatsächlich per korrelierter Subquery (kein zweites `randomblob`), die erzeugten
Ids sind 16 rohe Bytes, der Backfill-Abschnitt ist auf gesunden Daten
nachweisbar idempotent (zweiter Lauf: identische Zuordnungen), und die beiden
Unique-Index-Tests lesen die gespeicherte DDL statt den Quelltext.

Die Befunde liegen woanders. Der Kern: **die Selbst-Assertion prüft nicht die
Invariante, die sie laut Migration, Tests und Doku beweisen soll.** Sie prüft
eine Zählarithmetik, die weder notwendig noch hinreichend für „jede Zeile hat
eine Identität" ist. Beide Fehlerrichtungen sind reproduzierbar: ein
verwaister Link lässt die Migration Erfolg melden, obwohl ein `user` ohne
`person_id` zurückbleibt (CR-01); die andere Verwaisungsrichtung ist über
einen existierenden Admin-Endpoint (`delete_user`, Hard-Delete ohne
Link-Aufräumung, FK-Durchsetzung aus) produzierbar und lässt die Migration —
und damit den Serverstart — mit `CHECK constraint failed: ok = 1` abbrechen,
ohne Down-Migration und ohne Diagnose (CR-02). Dazu kommt ein
Datenformat-Defekt: `created` wird write-once in einem Format geschrieben, das
der im Repo überall verwendete Parser ablehnt (CR-03).

Die Doku ist inhaltlich sorgfältig und EN/DE weitgehend synchron; zwei
Behauptungen deckt der Code jedoch nicht (WR-01, WR-04).

## Critical Issues

### CR-01: Selbst-Assertion meldet Erfolg, obwohl `user.person_id` NULL bleibt

**File:** `migrations/sqlite/20260916000000_create-person-backfill.sql:89-139`

**Issue:**
Block 3d setzt `user.person_id` auf das Ergebnis einer korrelierten Subquery.
Existiert die in `sales_person_user.sales_person_id` referenzierte
`sales_person`-Zeile nicht (FK-Durchsetzung ist laut D-60-06 bewusst aus, die
Deklaration ist dekorativ), liefert die Subquery `NULL` — die `WHERE`-Klausel
greift trotzdem, und die Zeile wird auf `NULL` „gesetzt". Block 3b hat diese
Zeile vorher wegen `name NOT IN (SELECT user_id FROM sales_person_user)`
übersprungen. Ergebnis: der Benutzer bekommt **keine** Identität.

Die Zählprobe in Block 4 fängt das nicht ab, weil sie mit derselben
Link-Zeile rechnet, die den Schaden verursacht hat: `1 == 1 + 1 - 1`. Sie
geht auf, die Migration committet, Success Criterion 1 ist verletzt.

Reproduziert gegen die unveränderte Migrationsdatei (SQLite 3.51):

```
1 sales_person (unverknüpft), 1 user, 1 sales_person_user mit dangling sales_person_id
→ Migration exit 0
→ SELECT COUNT(*) FROM person                              -> 1
→ SELECT COUNT(*) FROM user WHERE person_id IS NULL        -> 1   (!)
```

Kein Test deckt diesen Fall ab: alle Fixtures in `person_backfill.rs` seeden
konsistente Paare. `full_identity_coverage` würde den Fall fangen, sieht ihn
aber nie.

Die Doku wiederholt die falsche Zusage: „A one-off, idempotent backfill gives
every existing row an identity" (`docs/features/F01-employee-management.md`,
DE analog) und „Er trägt bewusst keinen Soft-Delete-Filter — jede historische
`sales_person`-Zeile bekommt eine Identität"
(`docs/architecture/03-data-model_de.md`).

**Fix:** Die Assertion muss die Invariante prüfen, nicht eine Arithmetik, die
sie nur unter der (ungeprüften) Annahme referentieller Integrität impliziert.
Zusätzlich sollte 3d Zeilen ohne auflösbare Gegenseite gar nicht erst
anfassen, damit 3b/3d zusammen vollständig abdecken:

```sql
-- 3b: auch Benutzer aufnehmen, deren Link-Zeile ins Leere zeigt
UPDATE user
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND name NOT IN (
      SELECT spu.user_id
      FROM sales_person_user spu
      JOIN sales_person sp ON sp.id = spu.sales_person_id
  );

-- 3d: symmetrisch nur die auflösbaren Paare
UPDATE user
SET person_id = (
    SELECT sp.person_id
    FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    WHERE spu.user_id = user.name
)
WHERE person_id IS NULL
  AND name IN (
      SELECT spu.user_id
      FROM sales_person_user spu
      JOIN sales_person sp ON sp.id = spu.sales_person_id
  );
```

und Block 4 auf die tatsächliche Invariante umstellen (die Zählprobe darf als
zusätzliche Bedingung bleiben, siehe aber CR-02):

```sql
INSERT INTO __phase60_backfill_assertion (ok)
SELECT CASE
    WHEN (SELECT COUNT(*) FROM sales_person WHERE person_id IS NULL) = 0
     AND (SELECT COUNT(*) FROM user WHERE person_id IS NULL) = 0
     AND (SELECT COUNT(*) FROM sales_person_user spu
          JOIN sales_person sp ON sp.id = spu.sales_person_id
          JOIN user u ON u.name = spu.user_id
          WHERE sp.person_id IS NOT u.person_id) = 0
    THEN 1 ELSE 0
END;
```

Dazu ein Regressionstest mit verwaister Link-Zeile (beide Richtungen), der
heute fehlt.

---

### CR-02: Verwaiste `sales_person_user`-Zeile blockiert den Serverstart — produzierbar über einen bestehenden Admin-Endpoint

**File:** `migrations/sqlite/20260916000000_create-person-backfill.sql:116-141`

**Issue:**
Die Gegenrichtung von CR-01 — eine Link-Zeile, deren `user_id` auf keine
`user`-Zeile mehr zeigt — bringt die Zählprobe zum Kippen und rollt die
gesamte Migration zurück:

```
1 sales_person (verknüpft), 0 user, 1 sales_person_user mit dangling user_id
→ 1 == 1 + 0 - 1  ->  falsch
→ "Runtime error: CHECK constraint failed: ok = 1 (19)", Migration exit 1
```

Dieser Zustand ist **kein theoretisches Konstrukt**. Es existiert ein
erreichbarer Pfad in der Produktion:

- `rest/src/permission.rs:90` → `PermissionService::delete_user` →
  `dao_impl_sqlite/src/lib.rs:112`: `DELETE FROM user WHERE name = ?` —
  Hard-Delete, ohne `sales_person_user` anzufassen.
- Die FK-Deklaration `FOREIGN KEY (user_id) REFERENCES user(name)` ist laut
  D-60-06 und laut der in diesem Changeset neu formulierten Doku-Sektion
  „Foreign Keys" dekorativ — sie wird nie durchgesetzt.

Konsequenz: Jede Produktions-DB, in der ein Admin jemals einen Benutzer mit
Sales-Person-Verknüpfung gelöscht hat, lässt den Server nach dem Upgrade
**nicht mehr starten**. Diagnose ist ausschließlich
`CHECK constraint failed: ok = 1` — die Fehlermeldung nennt weder Tabelle
noch Zeile noch die drei beteiligten Zählwerte. Eine Down-Migration gibt es
bewusst nicht (D-60-09); der Wiederherstellungspfad ist das Zurückspielen
der Dateikopie, was das Upgrade komplett zurückdreht.

Fail-closed ist die richtige Haltung, aber ohne Pre-Flight und ohne
Diagnostik ist es ein unnötiger Deploy-Ausfall.

**Fix:** Dreiteilig.

1. Den Fall überhaupt heilen statt daran zu scheitern — mit dem 3b-Fix aus
   CR-01 verliert die verwaiste Zeile ihre Schadwirkung auf der User-Seite;
   die Zählprobe muss dann verwaiste Links ausklammern:

```sql
-- statt COUNT(*) FROM sales_person_user: nur beidseitig auflösbare Paare zählen
(SELECT COUNT(*) FROM sales_person_user spu
   JOIN sales_person sp ON sp.id = spu.sales_person_id
   JOIN user u ON u.name = spu.user_id)
```

2. Die Migration mit einer Pre-Flight-Abfrage ergänzen, die in den
   Deploy-Runbook-Text der Doku wandert:

```sql
-- Vor dem Deploy ausführen; muss 0 liefern:
SELECT COUNT(*) FROM sales_person_user spu
WHERE NOT EXISTS (SELECT 1 FROM sales_person sp WHERE sp.id = spu.sales_person_id)
   OR NOT EXISTS (SELECT 1 FROM user u WHERE u.name = spu.user_id);
```

3. Die `03-data-model`-Sektion (EN + DE) um diesen Vorbedingungs-Hinweis
   erweitern — aktuell steht dort nur „the whole migration rolls back and the
   server does not start — better than committing a half-executed backfill",
   ohne zu sagen, welcher Datenzustand das auslöst und wie man ihn vorher
   findet.

---

### CR-03: `person.created` wird in einem Format geschrieben, das kein DAO im Repo parsen kann

**File:** `migrations/sqlite/20260916000000_create-person-backfill.sql:108`

**Issue:**
`datetime('now')` liefert `YYYY-MM-DD HH:MM:SS` mit **Leerzeichen** als
Trennzeichen. Verifiziert gegen die echte Migration:

```
sqlite> SELECT created FROM person LIMIT 1;
2026-09-16 07:24:24
```

Die repo-weite Konvention für TEXT-Zeitstempel ist ISO-8601 mit `T`, gelesen
mit `time::format_description::well_known::Iso8601::DATE_TIME` — siehe
`dao_impl_sqlite/src/billing_period.rs:31-35` und
`dao_impl_sqlite/src/user_invitation.rs:45-47` (beide mit
`OffsetDateTime::parse` und `PrimitiveDateTime`-Fallback). Der Parser der
`time`-Crate verlangt das `T` hart
(`time-0.3.x/src/parsing/iso8601.rs`: `ascii_char::<b'T'>`); ein Leerzeichen
schlägt in **beiden** Zweigen fehl.

Das ist write-once-Daten: Der `PersonDao` aus Phase 61 wird bei jeder
Backfill-Zeile scheitern, sobald er `created` nach Repo-Muster liest, und die
Reparatur braucht dann eine weitere Datenmigration über bereits produktiv
vergebene Zeilen.

Das neu geschriebene ER-Diagramm im selben Changeset behauptet zudem
ausdrücklich das Gegenteil:
`docs/architecture/diagrams/db-schema-er.mmd:10` — „TEXT = ISO-Datum/Zeit".

Kein Test prüft den Inhalt von `person.created`.

**Fix:** Im Backfill das ISO-Format erzeugen:

```sql
INSERT INTO person (id, created, update_process, update_version)
SELECT DISTINCT ids.person_id,
       strftime('%Y-%m-%dT%H:%M:%S', 'now'),
       'phase-60-person-backfill',
       randomblob(16)
FROM (...) ids
WHERE ids.person_id NOT IN (SELECT id FROM person);
```

Plus ein Test, der das Format festzurrt, damit Phase 61 nicht darauf
aufläuft:

```rust
let created = row.get::<String, _>("created");
assert!(
    time::PrimitiveDateTime::parse(&created, &time::format_description::well_known::Iso8601::DATE_TIME).is_ok(),
    "person.created muss mit Iso8601::DATE_TIME parsebar sein — war: {created}"
);
```

## Warnings

### WR-01: Doku behauptet „UUID v4", der Backfill erzeugt zufällige Bytes ohne Version/Variant

**File:** `docs/architecture/03-data-model.md:186`, `docs/architecture/03-data-model_de.md:197`
(Spaltentabelle `person`), erzeugt in
`migrations/sqlite/20260916000000_create-person-backfill.sql:62,69,77,108`

**Issue:** Beide Doku-Varianten sagen „`id` | BLOB(16) PK | UUID v4, stored as
raw 16 bytes" bzw. „UUID v4, roh als 16 Bytes gespeichert". `randomblob(16)`
erzeugt 16 gleichverteilte Zufallsbytes — die Versionsnibble (Byte 6) und die
Variant-Bits (Byte 8) sind **nicht** nach RFC 4122 gesetzt. Der Rest des Repos
erzeugt echte v4-Uuids (`Uuid::new_v4()`, View `V_UUID_V4`). Der Test
`person_id_parses_as_uuid` prüft nur Länge und `Uuid::from_slice` — beides
gilt für beliebige 16 Bytes und deckt die Behauptung nicht.

Praktische Folge: `Uuid::get_version()` liefert für diese Ids Unsinn, und
strikte Konsumenten (OpenAPI `format: uuid`-Validierung, Frontend-Validatoren)
können die Werte ablehnen.

**Fix:** Entweder die Doku auf „zufällige 16 Bytes, kompatibel zu
`Uuid::from_slice`; bewusst keine RFC-4122-Version" korrigieren (EN **und**
DE), oder die Bits in SQL setzen:

```sql
randomblob(6) ||
char(64 + (abs(random()) % 16))  -- Platzhalter: sauberer ist eine explizite
                                 -- Bitmaskierung der Bytes 7 und 9
```

Die Doku-Korrektur ist der ehrlichere und billigere Weg — dann aber in beiden
Sprachvarianten, und der Grund gehört dazu, damit Phase 61 nicht „repariert".

---

### WR-02: `migration_aborts_on_count_mismatch` akzeptiert jeden Datenbankfehler und prüft die behauptete Rollback-Eigenschaft nicht

**File:** `service_impl/src/test/person_backfill.rs:396-419`

**Issue:** Zwei Defekte in einem Test.

1. Der Test behauptet in seiner Message „die Selbst-Assertion muss den Lauf
   mit einem Datenbankfehler abbrechen", prüft aber nur `result.is_err()`.
   Jeder beliebige Fehler bestätigt ihn — ein Syntaxfehler, eine
   Constraint-Verletzung aus einem ganz anderen Block, ein umbenannter Marker,
   der eine halbe Anweisung ausschneidet. Ausgerechnet in derselben Datei
   machen `duplicate_person_id_rejected_by_db_sales_person` (Zeile 648-651)
   und `..._user` (Zeile 732-735) es richtig und prüfen den Fehlertext. Hier
   fehlt die Prüfung.
2. Die Assert-Message sagt „statt einen halb ausgefuehrten Backfill zu
   committen" — genau das passiert im Testpfad aber. `run_backfill_section`
   fährt `sqlx::raw_sql(...).execute(pool)` **ohne** Transaktion; die vier
   `UPDATE`s und das `INSERT INTO person` sind im Autocommit bereits
   geschrieben, wenn die Assertion greift. Die Rollback-Eigenschaft
   (D-60-07, in beiden `03-data-model`-Varianten und im Migrations-Kommentar
   Zeile 121-123 zugesagt) entsteht ausschließlich aus
   `sqlx-sqlite/src/migrate.rs::apply`, das die Datei in eine Transaktion
   einbettet — **kein** Test exerziert diesen Pfad.

**Fix:**

```rust
let err = result.expect_err("D-60-07: die Selbst-Assertion muss abbrechen");
assert!(
    matches!(err, sqlx::Error::Database(_)),
    "D-60-07: die Ablehnung muss von der Datenbank kommen — war: {err:?}"
);
assert!(
    err.to_string().to_lowercase().contains("check constraint failed"),
    "D-60-07: der Abbruch muss aus der CHECK-Assertion stammen, nicht aus \
     einem beliebigen anderen Fehler — Fehlertext war: {err}"
);
```

Für die Rollback-Zusage: entweder den Abschnitt im Test explizit in eine
`pool.begin()`-Transaktion fahren und nach dem `rollback()` prüfen, dass
`COUNT(person)` und alle `person_id` unverändert sind — oder die
Formulierung in Migration und Doku darauf zurücknehmen, dass die
Transaktionsklammer von sqlx kommt und hier nicht mitgetestet wird.

---

### WR-03: `CREATE TEMP TABLE` ohne `IF NOT EXISTS` bricht den vertraglich zugesagten Replay

**File:** `migrations/sqlite/20260916000000_create-person-backfill.sql:128-141`

**Issue:** Schlägt der `INSERT` in Zeile 132 fehl, stoppt `sqlx::raw_sql` an
dieser Anweisung — das `DROP TABLE` in Zeile 141 läuft nie. Die TEMP-Tabelle
ist connection-lokal und überlebt. Ein erneuter Lauf des Abschnitts auf
derselben Connection scheitert dann nicht an der Assertion, sondern an

```
Parse error: table __phase60_backfill_assertion already exists
```

Reproduziert. Damit maskiert ein Folgefehler die eigentliche Aussage, und der
in D-60-08 ausdrücklich zugesagte „Apply-twice"-Vertrag („der Abschnitt ist
idempotent und wird ein zweites Mal ausgeführt") gilt nur für den
Erfolgspfad. In der Testsuite fällt das heute nicht auf, weil
`migration_aborts_on_count_mismatch` nach dem Fehlschlag nichts mehr auf
derselben Connection fährt — das ist Glück, keine Eigenschaft.

**Fix:**

```sql
DROP TABLE IF EXISTS temp.__phase60_backfill_assertion;
CREATE TEMP TABLE __phase60_backfill_assertion (
    ok INTEGER NOT NULL CHECK (ok = 1)
);
```

---

### WR-04: Der Backfill schreibt über einen bestehenden Trigger jede `user.update_timestamp` neu — Doku sagt „unberührt"

**File:** `migrations/sqlite/20260916000000_create-person-backfill.sql:68-97`

**Issue:** `user` trägt seit `20240426150045_user-roles.sql` den Trigger
`user_update_timestamp AFTER UPDATE ON user`, der `update_timestamp` auf
`DATETIME('now')` setzt. Die Blöcke 3b und 3d sind `UPDATE`s auf `user` und
feuern ihn für **jede** Bestandszeile. Verifiziert:

```
vorher: 2020-01-01 00:00:00
nachher: 2026-09-16 07:26:06
```

Die Doku behauptet das Gegenteil: „the auth layer is untouched"
(`docs/features/F12-auth-session.md`), „Es wird nichts umgeschlüsselt, die
Auth-Schicht ist damit unberührt" (`_de.md`), „`sales_person.id` und
`user.name` werden nicht umgeschlüsselt; beide bleiben bitgleich (PERS-06)"
(`03-data-model_de.md`). `legacy_keys_are_bit_identical_after_backfill`
(Zeile 936-1036) vergleicht ausschließlich `sales_person.id` und `user.name`
und sieht die Spalte gar nicht an. Der Schaden ist gering — es ist eine
Audit-Spalte, kein Schlüssel —, aber die Doku-Zusage ist so nicht haltbar,
und ein Auditor, der Änderungszeitstempel auswertet, sieht eine
Massenänderung ohne Erklärung.

**Fix:** Den Effekt in der Doku benennen (EN + DE, im
Backfill-Aufzählungspunkt der `03-data-model`-Sektion) und im Test
festzurren, damit er bewusst bleibt:

```rust
// vor dem Backfill lesen, danach erneut
assert_ne!(ts_before, ts_after,
    "Bekannter Seiteneffekt: der user_update_timestamp-Trigger schreibt die \
     Audit-Spalte bei jedem Backfill-UPDATE neu (dokumentiert in 03-data-model)");
```

Wer ihn vermeiden will, braucht ein temporäres `DROP TRIGGER` /
`CREATE TRIGGER` um die beiden `UPDATE`s herum — das ist aber teurer als die
Doku-Ehrlichkeit.

---

### WR-05: Der Marker-Vertrag hat keinen Guard gegen Schrumpfen des Abschnitts

**File:** `service_impl/src/test/person_backfill.rs:103-127`

**Issue:** `backfill_section()` fängt **fehlende** Marker laut ab (zwei
`panic!` mit klarer Message) — das ist gut. Ungedeckt bleibt die
wahrscheinlichere Drift: jemand ergänzt einen weiteren Backfill-Block
*hinter* `-- BACKFILL END` oder verschiebt einen Marker, sodass der replayte
Abschnitt kleiner wird als der tatsächliche Backfill. Dann testet
`apply_twice_is_noop` weiterhin grün, deckt aber nicht mehr, was es zu
decken vorgibt. Der Migrations-Kommentar (Zeile 19-22) erklärt den Vertrag,
aber nichts erzwingt ihn.

Verstärkend: `run_backfill_section` gibt bei leerem Abschnitt `Ok(())`
zurück (Zeile 120-124). Der Pfad ist heute unerreichbar, führt aber einen
stillen Erfolgszweig ein, den ein zukünftiger Marker-Unfall treffen könnte.

**Fix:** Struktur-Guard direkt in `backfill_section()`:

```rust
let section = section.to_string();
for needle in [
    "UPDATE sales_person",
    "UPDATE user",
    "INSERT INTO person",
    "__phase60_backfill_assertion",
] {
    assert!(
        section.contains(needle),
        "Der Markerbereich enthält '{needle}' nicht mehr — der Apply-twice-Nachweis \
         (D-60-08) würde stillschweigend weniger prüfen als der echte Backfill"
    );
}
assert_eq!(
    section.matches("UPDATE ").count(), 4,
    "Der Backfill hat 4 UPDATE-Blöcke; weicht die Zahl ab, liegt ein Block \
     außerhalb der Marker"
);
```

Zusätzlich: statt `Ok(())` bei leerem Abschnitt besser `panic!` — ein leerer
Markerbereich ist immer ein Defekt, kein legitimer No-Op.

---

### WR-06: Zwei zugesagte Eigenschaften der `person`-Zeilen sind ungetestet

**File:** `service_impl/src/test/person_backfill.rs:566-593`

**Issue:** `person_id_parses_as_uuid` ist der einzige Test, der `person`-Zeilen
inhaltlich anfasst, und er liest nur `id`. Zwei Zusagen, die beide
Doku-Varianten als dauerhafte Eigenschaft ausweisen, hat niemand festgezurrt:

- `update_process = 'phase-60-person-backfill'` — „Backfilled rows stay
  recognisable forever" / „Backfill-Zeilen bleiben dauerhaft an
  `update_process` erkennbar" (`03-data-model.md` / `_de.md`).
- `created` — siehe CR-03; hier fehlt jede Zusicherung, weshalb der
  Format-Defekt durch 16 grüne Tests gelaufen ist.

**Fix:** Im bestehenden Test mitlesen:

```rust
let rows = sqlx::query("SELECT id, created, update_process FROM person")
    .fetch_all(pool.as_ref()).await.expect("read person rows");
assert!(!rows.is_empty(), "Vorbedingung: der Backfill muss person-Zeilen erzeugt haben");
for row in rows {
    assert_eq!(
        row.get::<String, _>("update_process"),
        "phase-60-person-backfill",
        "Herkunftsmarker muss die Backfill-Zeilen dauerhaft unterscheidbar halten"
    );
    // + created-Format-Assertion aus CR-03
}
```

## Info

### IN-01: `count_identity_holds` kann praktisch nicht fehlschlagen

**File:** `service_impl/src/test/person_backfill.rs:374-392`
**Issue:** Der Test prüft exakt dieselbe Bedingung, die Block 4 der Migration
unmittelbar davor selbst geprüft hat. Geht sie nicht auf, scheitert schon
`run_backfill_section(...).expect(...)` zwei Zeilen höher — das `assert_eq!`
ist dann unerreichbar. Als Regressionsschutz gegen ein *Entfernen* von Block 4
hat der Test einen Restwert, aber er misst nicht die Invariante, sondern die
Assertion.
**Fix:** In der Doc-Kommentarzeile ehrlich benennen („deckt den Fall ab, dass
Block 4 aus der Migration verschwindet"), oder — sinnvoller — den Test auf
die Deckungs-Invariante aus CR-01 umstellen, die Block 4 heute nicht prüft.

---

### IN-02: Toter Erfolgszweig in `run_backfill_section`

**File:** `service_impl/src/test/person_backfill.rs:120-124`
**Issue:** Der `if section.trim().is_empty() { return Ok(()) }`-Zweig ist bei
korrekten Markern unerreichbar und bringt einen stillen `Ok`-Pfad ein.
**Fix:** Siehe WR-05 — `panic!` statt `Ok(())`.

---

### IN-03: Gemischte Re-Run-Guards suggerieren eine Wiederholbarkeit, die die Datei nicht hat

**File:** `migrations/sqlite/20260916000000_create-person-backfill.sql:28,39-40,46,49`
**Issue:** `CREATE TABLE IF NOT EXISTS` und `CREATE UNIQUE INDEX IF NOT EXISTS`
stehen neben zwei ungeschützten `ALTER TABLE ... ADD COLUMN`. Ein Leser
schließt aus den Guards, die Datei sei als Ganzes wiederholbar — der
Test-Kommentar (Zeile 99-102) erklärt zu Recht das Gegenteil, die Migration
selbst nicht.
**Fix:** Eine Kommentarzeile über den `ALTER TABLE`s: „Nur der Abschnitt
zwischen den BACKFILL-Markern ist wiederholbar; `ALTER TABLE ... ADD COLUMN`
kennt in SQLite keine Existenz-Guard-Klausel — die Datei als Ganzes darf nur
einmal laufen (sqlx-Ledger)."

---

### IN-04: EN/DE-Drift bei der Beschreibung von `user.person_id`

**File:** `docs/features/F12-auth-session.md:109` vs. `docs/features/F12-auth-session_de.md:99`
**Issue:** EN schreibt „FK-style reference to the identity parent table
`person`", DE nur „Verweis auf die Identitäts-Elterntabelle `person`". Es gibt
weder eine `FOREIGN KEY`-Deklaration auf `person` noch eine Durchsetzung; die
Formulierung „FK-style" widerspricht der drei Absätze später im selben
Dokument getroffenen Aussage („Enforcement … **not** through foreign keys").
**Fix:** EN an DE angleichen: „reference to the identity parent table `person`
(no `FOREIGN KEY` declaration — see below)".

---

### IN-05: Test-Pool ohne `max_connections(1)` für `sqlite::memory:`

**File:** `service_impl/src/test/person_backfill.rs:83-94`
**Issue:** `SqlitePool::connect("sqlite::memory:")` öffnet mit
`SQLITE_OPEN_MEMORY | SQLITE_OPEN_PRIVATECACHE` — jede zusätzliche Connection
des Pools bekäme eine **eigene, leere** Datenbank, und die TEMP-Tabelle der
Assertion ist ohnehin connection-lokal. Heute greift das nicht, weil alle
Queries sequenziell awaited werden und der Pool eine Connection wiederverwendet.
Das ist das im Repo etablierte Muster (`absence_conversion.rs`,
`rebooking_batch_state.rs`, `pdf_export_config.rs`) und deshalb kein Neu-Defekt
dieser Phase — bei nebenläufigen Zugriffen wäre es aber latent flaky.
**Fix:** Optional `SqlitePoolOptions::new().max_connections(1).connect(...)`,
idealerweise repo-weit in einem gemeinsamen Test-Helper.

---

_Reviewed: 2026-09-16T07:29:17Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
