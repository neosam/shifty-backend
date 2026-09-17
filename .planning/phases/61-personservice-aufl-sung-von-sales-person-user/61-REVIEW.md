---
phase: 61-personservice-aufl-sung-von-sales-person-user
reviewed: 2026-09-17T09:53:20Z
depth: standard
files_reviewed: 34
files_reviewed_list:
  - dao/src/lib.rs
  - dao/src/person.rs
  - dao_impl_sqlite/src/lib.rs
  - dao_impl_sqlite/src/person.rs
  - dao_impl_sqlite/src/sales_person.rs
  - migrations/sqlite/20260917000000_drop-sales-person-user.sql
  - rest/src/lib.rs
  - rest/src/sales_person.rs
  - service/src/lib.rs
  - service/src/person.rs
  - service_impl/src/lib.rs
  - service_impl/src/person.rs
  - service_impl/src/sales_person.rs
  - service_impl/src/test/mod.rs
  - service_impl/src/test/person_candidates.rs
  - service_impl/src/test/person_creation.rs
  - service_impl/src/test/person_merge.rs
  - service_impl/src/test/person_self_gate.rs
  - shifty_bin/src/main.rs
  - shifty_bin/src/person_link_migration_test.rs
  - docs/arc42/05-building-block-view.md
  - docs/architecture/02-service-tiers.md
  - docs/architecture/02-service-tiers_de.md
  - docs/architecture/03-data-model.md
  - docs/architecture/03-data-model_de.md
  - docs/architecture/04-auth.md
  - docs/architecture/04-auth_de.md
  - docs/architecture/diagrams/db-schema-er.mmd
  - docs/architecture/diagrams/service-graph-runtime.mmd
  - docs/features/F01-employee-management.md
  - docs/features/F01-employee-management_de.md
  - docs/features/F12-auth-session.md
  - docs/features/F12-auth-session_de.md
  - AUTHENTICATION.md
findings:
  critical: 2
  warning: 7
  info: 7
  total: 16
status: issues_found
---

# Phase 61: Code-Review-Bericht

**Reviewed:** 2026-09-17T09:53:20Z
**Depth:** standard
**Files Reviewed:** 34
**Status:** issues_found

## Summary

Geprüft wurde die Auflösung der 1:1-Verknüpfungstabelle `sales_person_user` und
ihre Ablösung durch eine gemeinsame `person.id` auf `sales_person.person_id` und
`user.person_id`.

Gate-Läufe (zur Einordnung, nicht als Korrektheitsnachweis):
`cargo clippy --workspace --all-targets -- -D warnings` → exit 0,
`cargo test --workspace` → exit 0.

**Das sicherheitskritische Kernstück ist korrekt.** Der Selbst-Zugriffs-Gate
entscheidet tatsächlich im SQL (`JOIN user u ON u.person_id = sp.person_id`), und
die SQL-Drei-Werte-Logik schließt `NULL = NULL` verlässlich aus — ein fail-open
auf fremde Buchungs- und Lohndaten konnte ich in keinem der vier Lesepfade
(`find_by_user`, `find_sales_person_by_user_id`, `get_assigned_user`,
`get_all_user_assignments`) konstruieren. Die Anti-Joins in `PersonDaoImpl`
hängen an `NOT NULL`-Spalten (`user.name`, `sales_person.id`) und sind damit
ebenfalls tragfähig. Die Konflikt-Vorprüfungen in `assign_to_user` laufen
nachweislich vor jedem Schreibvorgang; Merge/Split hinterlassen in allen von mir
durchgespielten Szenarien keine trägerlose `person`-Zeile.

**Zwei Befunde blockieren trotzdem.** (1) Der Nachzieh-Backfill der
DROP-Migration deckt nur eine der beiden Richtungen ab, in denen im
Deploy-Fenster zwischen Phase 60 und 61 ein Auseinanderlaufen entstehen kann —
die häufigste Fensteraktion überhaupt (HR verknüpft einen Login) führt dazu,
dass die Selbst-Assertion in Block B abbricht und der Server nicht startet. Der
Kommentar, der diesen Fall für unmöglich erklärt, ist nachweislich falsch, und
ein Test zementiert den Abbruch als Sollverhalten. (2) `BasicDaoImpl::clear_all`
löscht sämtliche `person`-Zeilen, lässt die `user`-Zeilen aber stehen — und
zerstört damit genau die Identitätsbindung, die D-61-01 in dieser Phase
eingeführt hat.

Dazu kommen sieben Warnungen, davon drei mit Außenwirkung auf die HTTP-Semantik
(409 statt 404 / 500 statt 409) und eine funktionale Lücke im
Kandidaten-Lesepfad, die Phase 63 direkt trifft.

## Critical Issues

### CR-01: Nachzieh-Backfill deckt nur eine Richtung ab — ein im Deploy-Fenster gesetzter Link lässt die Migration abbrechen und blockiert den Server-Start

**File:** `migrations/sqlite/20260917000000_drop-sales-person-user.sql:51-69` (Block A3/A4), `:115-124` (Block B)

**Issue:**
Der Dateikopf begründet Block A genau so: „Block A zieht Zeilen nach, die
zwischen dem Phase-60- und dem Phase-61-Deploy entstanden sind" (Zeile 6-8).
Block A ist für dieses Fenster aber unvollständig, und Block B bricht dann ab.

Der Kommentar vor der Assertion behauptet (Zeile 111-114):

> „Ein Widerspruch entsteht nicht durch Block A (dessen Idempotency key
> ueberspringt belegte Zeilen), sondern durch manuelle Eingriffe oder Restores"

Das ist falsch. Zwei über den **normalen Applikationspfad** erreichbare Zustände
erzeugen den Widerspruch:

**Pfad 1 — beide Seiten tragen schon eine Person (der häufigste Fall).**
Nach dem Phase-60-Backfill trägt *jede* bestehende `sales_person`- und *jede*
bestehende `user`-Zeile eine eigene Person. Verknüpft HR im Fenster einen
bestehenden Login mit einer bestehenden Sales Person, schreibt der damals noch
aktive alte `assign_to_user` ausschließlich eine `sales_person_user`-Zeile —
`sp.person_id` und `u.person_id` bleiben verschieden.
A1-A4 überspringen beide Zeilen (Idempotency key ist `person_id IS NULL`),
A5 legt nichts nach. Block B findet `sp.person_id != u.person_id` → `CHECK
constraint failed` → die gesamte Migration rollt zurück → der Server startet
nicht.

**Pfad 2 — Block A erzeugt den Widerspruch selbst.**
Bestehender Benutzer `U` (hat `P_u` aus Phase 60), im Fenster neu angelegte
Sales Person `S` (`person_id IS NULL`, weil D-60-10 das ausdrücklich zulässt),
im Fenster verknüpft. Dann gilt:
- A2 überspringt `U` (`name IN (SELECT user_id FROM sales_person_user)`)
- A4 überspringt `U` (`person_id IS NULL` trifft nicht zu)
- **A3 vergibt `S` per `randomblob(16)` eine fabrikneue, von `P_u` verschiedene Person**

Block A stellt hier also aktiv die Inkonsistenz her, an der Block B anschließend
abbricht. Die Gegenrichtung (SP hat Person, User nicht) ist über die korrelierte
Subquery in A4 korrekt gelöst — genau diese Asymmetrie zeigt, dass A3 aus der
Phase-60-Migration übernommen wurde, ohne die geänderte Vorbedingung
nachzuziehen: zum Zeitpunkt von Phase 60 konnte noch kein Benutzer eine Person
tragen (die Spalte entstand in derselben Migration), ab Phase 61 ist das der
Regelfall.

`shifty_bin/src/person_link_migration_test.rs:1465-1509`
(`selbst_assertion_bricht_bei_widerspruechlicher_verknuepfung_ab`) seedet exakt
diesen Zustand und zementiert den Abbruch als Sollverhalten — der Test schützt
also aktuell den Defekt statt ihn zu finden.

Konsequenz: irreversible Migration ohne Down-Pfad, Server-Start-Abbruch, manuelle
SQL-Chirurgie an der Produktions-DB nötig. In keiner der in dieser Phase
aktualisierten Doku-Dateien (`03-data-model*.md`, `F12-auth-session*.md`) steht
ein Runbook für diesen Ausgang.

**Fix:**
Block A muss dieselbe Gewinner-Logik anwenden, die D-61-03 für den Laufzeitpfad
festlegt (die Sales Person behält ihre Identität, der Benutzer wird umgehängt,
die absorbierte Person wird gelöscht). Zwei zusätzliche Anweisungen, beide vor
Block B:

```sql
-- A3 (korrigiert): traegt der verknuepfte Benutzer bereits eine Person, wird
-- DIESE uebernommen statt eine neue zu wuerfeln. Sonst wie bisher.
UPDATE sales_person
SET person_id = COALESCE(
    (SELECT u.person_id
     FROM sales_person_user spu
     JOIN user u ON u.name = spu.user_id
     WHERE spu.sales_person_id = sales_person.id),
    randomblob(16))
WHERE person_id IS NULL
  AND id IN (SELECT sales_person_id FROM sales_person_user);

-- A4b (neu): auseinandergelaufene Bestandspaare verschmelzen — der Benutzer
-- wandert auf die Person der Sales Person (D-61-03: die Sales Person ist die
-- Seite mit historischem Gewicht, billing_period_sales_person haengt an ihr).
UPDATE user
SET person_id = (
    SELECT sp.person_id
    FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    WHERE spu.user_id = user.name
)
WHERE name IN (SELECT user_id FROM sales_person_user)
  AND person_id IS NOT NULL
  AND person_id IS NOT (
    SELECT sp.person_id
    FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    WHERE spu.user_id = user.name
  );

-- A4c (neu): die dabei absorbierten person-Zeilen aufraeumen.
DELETE FROM person
WHERE NOT EXISTS (SELECT 1 FROM sales_person sp WHERE sp.person_id = person.id)
  AND NOT EXISTS (SELECT 1 FROM user         u  WHERE u.person_id  = person.id);
```

Block B bleibt unverändert als letzte Absicherung stehen. Zusätzlich:
- `selbst_assertion_bricht_bei_widerspruechlicher_verknuepfung_ab` muss auf einen
  Zustand umgestellt werden, der nach A4b tatsächlich noch widersprüchlich ist
  (z. B. zwei `user`-Zeilen, die auf dieselbe Person zeigen sollen).
- Zwei neue Regressionstests: „Bestandspaar im Fenster verknüpft" und
  „Bestands-User + neue Sales Person im Fenster verknüpft" müssen nach dem
  Nachzug auf **derselben** Person landen und die Migration durchlaufen.
- Den falschen Kommentar in Zeile 111-114 korrigieren.

---

### CR-02: `clear_all` löscht alle `person`-Zeilen, lässt aber die `user`-Zeilen mit verwaisten `person_id`-Verweisen stehen

**File:** `dao_impl_sqlite/src/lib.rs:317-336`

**Issue:**
Die Anweisungskette löscht `sales_person` und anschließend **alle**
`person`-Zeilen, fasst `user` aber nicht an:

```
DELETE FROM sales_person;
DELETE FROM person;
```

`user.person_id` trägt keine `FOREIGN KEY`-Deklaration (siehe
`20260916000000_create-person-backfill.sql:40` — `ALTER TABLE ... ADD COLUMN`
kann in SQLite keine FK setzen), SQLite kann das also nicht abfangen. Nach
`clear_all` zeigt jede überlebende `user`-Zeile — inklusive der beim Start
angelegten `DEVUSER` und `admin` (`shifty_bin/src/main.rs:1635-1659`) — auf eine
nicht mehr existierende Person.

Das bricht genau die Invariante, die diese Phase eingeführt hat. `docs/architecture/03-data-model.md`
(neuer Abschnitt „The permanently checkable invariant") formuliert sie als
„every `sales_person` row and every `user` row carries exactly one `person_id`,
and every `person` row has at least one carrier" und schreibt sie ausdrücklich
dem Code zu: „Anyone writing a third creation path has to honour this by hand —
the database will not catch the omission." `clear_all` ist dieser dritte Pfad und
hält sie nicht ein.

Erreichbar über `rest/src/dev.rs:107-115` (`clear_data`) und über die
Test-Harnesses `shifty_bin/src/integration_test.rs:292` sowie
`integration_test/dev_seed.rs:66,83`. Wirkung: `PersonService::get(id)` liefert
für jeden bestehenden Benutzer `None`, und die Personenverwaltung aus Phase 62/63
sieht identitätslose Benutzer. Der begleitende Kommentar (Zeile 318-323)
begründet nur die Reihenfolge gegenüber `sales_person` und übersieht `user`
vollständig.

**Fix:**
Nur trägerlose Personen löschen — das ist zugleich exakt die Invariante aus der
Doku:

```sql
DELETE FROM booking;
DELETE FROM sales_person_unavailable;
DELETE FROM extra_hours;
DELETE FROM employee_work_details;
DELETE FROM sales_person;
-- Nur Personen ohne Traeger. Ein pauschales `DELETE FROM person` wuerde die
-- user-Zeilen mit verwaisten person_id-Verweisen zuruecklassen (D-61-01).
DELETE FROM person
WHERE NOT EXISTS (SELECT 1 FROM sales_person sp WHERE sp.person_id = person.id)
  AND NOT EXISTS (SELECT 1 FROM user         u  WHERE u.person_id  = person.id);
DELETE FROM special_day;
```

Plus ein Test in `person_creation.rs`: nach `clear_all` muss für jede
`user`-Zeile eine passende `person`-Zeile existieren.

## Warnings

### WR-01: `assign_to_user` meldet „nicht gefunden" und einen Datendefekt beide als 409 „already exists"

**File:** `dao_impl_sqlite/src/sales_person.rs:265-271`, `:329-331`

**Issue:**
Drei fachlich verschiedene Ausgänge kollabieren auf `DaoError::EntityAlreadyExists`,
das `SalesPersonServiceImpl::set_user` (`service_impl/src/sales_person.rs:347-351`)
unverändert nach `ServiceError::EntityAlreadyExists` → **HTTP 409** übersetzt:

1. Die Sales Person existiert gar nicht (`fetch_optional` → `None`) → Zeile 271
2. Die Sales Person existiert, trägt aber `person_id IS NULL` (Legacy-/Defektzustand) → Zeile 271
3. Der übergebene Login existiert nicht → `rows_affected() == 0` → Zeile 330

Fall 1 und 3 sind 404-Fälle; die Doku
(`docs/features/F01-employee-management.md`, Tabelle §5) führt für
`POST /sales-person/{id}/user` weiterhin 404 auf, das der Code nie mehr erzeugen
kann. Fall 2 ist ein Serverzustandsdefekt, den ein Client durch Wiederholen nicht
auflösen kann — als 409 gemeldet lädt er aber genau dazu ein. Ein HR-Client kann
„Login-Name getippt/existiert nicht" nicht mehr von „schon verknüpft"
unterscheiden.

**Fix:** Die Fälle im DAO trennen und im Service gezielt übersetzen:

```rust
// dao/src/lib.rs — bestehende Variante nutzen
let Some(row) = query!("SELECT person_id FROM sales_person WHERE id = ?", sales_person_id_vec)
    .fetch_optional(tx.tx.lock().await.as_mut()).await.map_db_error()?
else {
    return Err(DaoError::EntityNotFound);            // Fall 1 → 404
};
let Some(target_person) = row.person_id else {
    return Err(DaoError::DatabaseQueryError(/* Invariantenbruch D-61-01 */)); // Fall 2 → 500
};
```

und für Fall 3 vor dem `UPDATE` explizit auf Existenz der `user`-Zeile prüfen
(`person_of_user` kommt bereits aus einem `fetch_optional` — den `Option<Row>`
statt nur `.and_then(|row| row.person_id)` auswerten) und `EntityNotFound`
zurückgeben.

---

### WR-02: `users_without_sales_person` filtert nicht auf Soft-Delete — ein Benutzer mit nur soft-gelöschter Sales Person fällt aus beiden Kandidatenlisten

**File:** `dao_impl_sqlite/src/person.rs:173-189`

**Issue:**
Die beiden Kandidaten-Lesepfade sind asymmetrisch:

- `sales_persons_without_user` (Zeile 163) filtert `sp.deleted IS NULL`
- `users_without_sales_person` (Zeile 181) filtert nur `sp.id IS NULL`

Für einen Benutzer `U`, dessen einzige Sales Person soft-gelöscht ist, gilt
damit: die Sales Person ist kein „Login hinzufügen"-Kandidat (deleted-Filter),
und `U` ist kein „Person hinzufügen"-Kandidat (der Join-Partner existiert ja, wenn
auch gelöscht). `U` erscheint in **keiner** der beiden Listen und lässt sich über
den Phase-63-Picker nie wieder mit einer Person versehen.

Der Doc-Kommentar (Zeile 65-66 in `dao/src/person.rs`) weicht der Frage aus
(„`user` hat keine Soft-Delete-Spalte") — die relevante Spalte ist aber
`sales_person.deleted`, nicht `user.deleted`. `docs/features/F01-employee-management.md`
übernimmt die Formulierung ungeprüft. Kein Test in
`service_impl/src/test/person_candidates.rs` deckt den Fall ab; K-3 testet nur
die Gegenrichtung.

**Fix:** Entscheidung treffen und beidseitig festzurren. Wenn eine
soft-gelöschte Sales Person den Benutzer nicht mehr binden soll:

```sql
SELECT u.name AS "name!: String"
FROM user u
LEFT JOIN sales_person sp
       ON sp.person_id = u.person_id
      AND sp.deleted IS NULL
WHERE sp.id IS NULL
```

(Der Filter gehört in die `ON`-Klausel, nicht in `WHERE` — sonst wird der
LEFT JOIN zum INNER JOIN.) Plus Test `user_mit_nur_soft_geloeschter_sales_person_ist_kandidat`.
Wenn die jetzige Semantik gewollt ist: Doc-Kommentar und F01-Tabelle explizit
darauf festlegen und einen Test schreiben, der sie festhält.

---

### WR-03: `PermissionDaoImpl::create_user` öffnet eine eigene Pool-Transaktion — nicht atomar mit der Transaktion des Aufrufers, und ein Write-Lock-Risiko

**File:** `dao_impl_sqlite/src/lib.rs:94-137`

**Issue:**
Die Methode ruft `self.pool.begin()` und arbeitet damit auf einer **zweiten**
Verbindung, während `UserInvitationServiceImpl::validate_and_consume_token`
(`service_impl/src/user_invitation.rs:95-137`) bereits eine eigene Transaktion
offen hält und sie nach dem `create_user`-Aufruf committet. Zwei Folgen:

1. **Keine Atomarität mit dem Aufrufer.** `user` + `person` sind bereits
   committet, wenn die äußere Transaktion danach zurückrollt. Für `user` war das
   vorher schon so (Einzelstatement auf dem Pool), jetzt betrifft es zusätzlich
   die neue `person`-Zeile. Die Doku
   (`docs/features/F12-auth-session.md`, Bullet „Login auto-creation assigns
   `person_id`") nennt die interne Transaktion, erwähnt diese Einschränkung aber
   nicht.
2. **Latentes Write-Lock-Problem.** sqlx setzt für SQLite standardmäßig
   `journal_mode = WAL`; dort ist genau ein Schreiber erlaubt. Heute ist der Pfad
   sicher, weil die äußere Transaktion vor dem `create_user`-Aufruf noch nichts
   geschrieben hat (`find_by_token` läuft ohne `tx` auf dem Pool) und ein
   `BEGIN DEFERRED` noch kein Lock hält. Sobald ein künftiger Aufrufer vor
   `create_user` in seiner Transaktion schreibt, blockiert das innere `INSERT`
   bis zum `busy_timeout` und scheitert dann mit `SQLITE_BUSY`.

**Fix:** Bevorzugt die Trait-Signatur um `tx: Self::Transaction` erweitern
(`dao/src/permission.rs:27`) und beide Statements auf der Transaktion des
Aufrufers fahren — dann ist die Anlage sowohl atomar als auch lock-frei. Die
Begründung im Kommentar („die Trait-Signatur bleibt dabei unveraendert; die
Transaktion ist eine reine Interna dieser Methode") ist der eigentliche Grund für
das Problem. Wenn die Signatur nicht angefasst werden soll: die Einschränkung als
Doc-Kommentar über `create_user` festhalten („darf nicht aus einer Transaktion
heraus aufgerufen werden, die bereits geschrieben hat") und in
`F12-auth-session*.md` spiegeln.

---

### WR-04: `discard_assigned_user` liefert `EntityAlreadyExists`, `set_user` übersetzt aber nur den Fehler von `assign_to_user` — identische Bedingung, unterschiedlicher HTTP-Status

**File:** `service_impl/src/sales_person.rs:327-352`, `dao_impl_sqlite/src/sales_person.rs:432-433`

**Issue:**
`discard_assigned_user` gibt bei `rows_affected() != 1` ebenfalls
`DaoError::EntityAlreadyExists` zurück (Zeile 432-433). Im Service läuft dieser
Aufruf aber über ein blankes `?` (Zeile 330):

```rust
self.sales_person_dao
    .discard_assigned_user(sales_person_id, tx.clone())
    .await?;                                  // → ServiceError::DatabaseQueryError → 500
```

während derselbe `DaoError` aus `assign_to_user` zehn Zeilen weiter gezielt nach
`ServiceError::EntityAlreadyExists` → 409 übersetzt wird. `DELETE
/sales-person/{id}/user` ruft `set_user(id, None, ...)` — dort läuft
ausschließlich der Split, ein Konflikt erscheint also garantiert als 500. Der
Kommentar über dem `match` begründet die „chirurgische Uebersetzung" exakt mit
dem Argument, das für den Split genauso gilt.

**Fix:** Den Split in dieselbe Übersetzung nehmen:

```rust
match self.sales_person_dao.discard_assigned_user(sales_person_id, tx.clone()).await {
    Ok(()) => {}
    Err(dao::DaoError::EntityAlreadyExists) => {
        return Err(ServiceError::EntityAlreadyExists(sales_person_id));
    }
    Err(other) => return Err(other.into()),
}
```

Zusätzlich prüfen, ob `EntityAlreadyExists` für den Split-Fall überhaupt die
richtige Variante ist — der Zustand heißt dort „der Benutzer ist zwischenzeitlich
weggelaufen", nicht „existiert bereits". Ein Test für `set_user(id, None)` mit
Race-Verlierer fehlt komplett.

---

### WR-05: Die Selbst-Assertion in Block B nutzt INNER JOINs und übersieht damit genau die Zeilen, für die Phase 60 ihre Prüfung nachgeschärft hatte

**File:** `migrations/sqlite/20260917000000_drop-sales-person-user.sql:115-124`

**Issue:**

```sql
FROM sales_person_user spu
JOIN sales_person sp ON sp.id = spu.sales_person_id
JOIN user u ON u.name = spu.user_id
WHERE sp.person_id IS NULL OR u.person_id IS NULL OR sp.person_id != u.person_id
```

Beide Joins sind INNER. Eine `sales_person_user`-Zeile, deren Elternzeile fehlt,
fällt aus dem Ergebnis heraus, die Assertion geht grün durch, und Block C wirft
die Verknüpfung still weg. Das ist derselbe Fehler, den das Phase-60-Review als
CR-01 gefunden hat: `20260916000000_create-person-backfill.sql:154-166` erklärt
im Klartext, warum die Zählarithmetik allein nicht reicht, und ergänzt deshalb
`COUNT(*) FROM sales_person WHERE person_id IS NULL = 0` und dasselbe für `user`.
Phase 61 hat diese beiden globalen Terme nicht übernommen.

Praktisch schützt heute die FK-Deklaration auf `sales_person_user` plus
`PRAGMA foreign_keys = ON` (sqlx) — aber `.planning/STATE.md:59` führt die
Pre-Flight-Query auf genau verwaiste `sales_person_user`-Zeilen weiterhin als
**offen** („bewusst auf den Milestone-Check vertagt"). Die Absicherung, die den
Fall erschlagen würde, ist also ungeprüft, und die Migration ist irreversibel.

**Fix:** Die beiden Deckungsterme aus Phase 60 in die Assertion übernehmen:

```sql
INSERT INTO __phase61_drop_assertion (ok)
SELECT CASE
    WHEN (SELECT COUNT(*) FROM sales_person WHERE person_id IS NULL) = 0
     AND (SELECT COUNT(*) FROM user         WHERE person_id IS NULL) = 0
     AND NOT EXISTS (
        SELECT 1 FROM sales_person_user spu
        LEFT JOIN sales_person sp ON sp.id   = spu.sales_person_id
        LEFT JOIN user         u  ON u.name  = spu.user_id
        WHERE sp.id IS NULL OR u.name IS NULL
           OR sp.person_id IS NULL OR u.person_id IS NULL
           OR sp.person_id != u.person_id)
    THEN 1 ELSE 0
END;
```

---

### WR-06: `AUTHENTICATION.md` dokumentiert die aufgelöste Verknüpfungstabelle weiter als aktuelles Schema

**File:** `AUTHENTICATION.md:86`

**Issue:**
Die Datei führt weiterhin ein `CREATE TABLE sales_person_user (...)` als
Schema-Beschreibung. Ein `grep -rn "sales_person_user" docs/` ist nach dieser
Phase sauber, `AUTHENTICATION.md` liegt aber im Repo-Root und wurde übersehen.
Damit widerspricht ein Dokument im Repository direkt
`docs/architecture/03-data-model.md` („The 1:1 link table no longer exists").
Das Docs-Freshness-Gate aus `CLAUDE.md` verlangt, dass eine Migration in
`migrations/sqlite/*.sql` die zugehörige Doku im selben Commit mitzieht.

**Fix:** Den Abschnitt auf `person_id` umschreiben und auf
`docs/architecture/04-auth.md#self-access-gate-for-sales-person-data`
verweisen — oder, falls `AUTHENTICATION.md` durch `docs/architecture/04-auth*.md`
abgelöst ist, die Datei löschen und einen Verweis hinterlassen.

---

### WR-07: Die Rollback-Zusage P-61-B von `set_user` gilt nur, wenn der Aufrufer `tx = None` übergibt

**File:** `service_impl/src/sales_person.rs:317-352`

**Issue:**
Der Kommentar bei Zeile 349-350 sagt unbedingt zu:

> „Kein Commit: die Transaktion wird verworfen und damit auch das zuvor gelaufene
> Loesen zurueckgerollt (P-61-B)."

Die Signatur nimmt aber `tx: Option<Self::Transaction>`, und `use_transaction`
gibt eine übergebene Transaktion unverändert durch
(`dao_impl_sqlite/src/lib.rs:368-375`). Übergibt ein interner Aufrufer `Some(tx)`
und committet seine Transaktion trotz des `Err`, ist der Split (`discard`)
festgeschrieben, der Merge aber abgelehnt: die vorherige Verknüpfung ist
verloren, obwohl der Aufruf als Konflikt zurückgewiesen wurde. Heute ruft nur
`rest/src/sales_person.rs:298,330` mit `None`, der Test
`relink_konflikt_rollt_den_split_zurueck` deckt ebenfalls nur diesen Pfad ab —
die Zusage im Kommentar ist also breiter als das, was der Code garantiert.

**Fix:** Entweder die Einschränkung explizit machen —

```rust
// P-61-B gilt nur, solange der Aufrufer `tx = None` uebergibt und der Service
// die Transaktion selbst besitzt. Bei durchgereichter Transaktion muss der
// Aufrufer auf `Err` hin zwingend selbst zurueckrollen.
```

— oder bei durchgereichter Transaktion vor dem Split einen SAVEPOINT setzen und
im Fehlerfall gezielt dorthin zurückrollen.

## Info

### IN-01: `PersonDaoImpl.pool` wird nie gelesen

**File:** `dao_impl_sqlite/src/person.rs:92-99`

**Issue:** Alle vier Methoden arbeiten ausschließlich auf der übergebenen
`tx`; das `pub pool`-Feld hat keinen Leser. Es entgeht `dead_code` nur, weil es
`pub` ist.
**Fix:** Feld entfernen und `new()` den Pool verwerfen lassen, oder — wie in
`SalesPersonDaoImpl` — als `_pool` benennen, damit die Absicht sichtbar ist.

---

### IN-02: `RestStateDef::PersonService` / `person_service()` wird von niemandem aufgerufen

**File:** `rest/src/lib.rs:470-476`, `:521`

**Issue:** Ein neuer associated type plus Trait-Methode auf einem öffentlichen
Trait, die jeden Implementor zur Mitarbeit zwingen, ohne dass es einen Konsumenten
gibt. Der Kommentar begründet das damit, dass „das Feld einen Leser hat" — der
Leser ist aber nur der Accessor selbst, der wiederum ungelesen bleibt. Die
`-D warnings`-Hürde ließe sich billiger nehmen.
**Fix:** In Phase 61 `#[allow(dead_code)]` auf dem `RestStateImpl`-Feld setzen und
Trait-Erweiterung plus Accessor erst in Phase 62 zusammen mit dem ersten
Endpoint einführen.

---

### IN-03: Zeilenreferenz `sales_person.rs:316-342` in F01 ist nach dieser Phase veraltet

**File:** `docs/features/F01-employee-management.md:86`, `docs/features/F01-employee-management_de.md` (gleiche Stelle)

**Issue:** `set_user` ist durch den neuen `match`-Block auf 316-357 gewachsen; die
zitierte Spanne endet mitten in der Methode.
**Fix:** Auf `sales_person.rs:316-357` aktualisieren — oder, robuster, auf den
Methodennamen statt auf Zeilennummern verweisen.

---

### IN-04: Deutscher Text in der englischen arc42-Datei

**File:** `docs/arc42/05-building-block-view.md:119`

**Issue:** Die Tabellenzelle lautet „`person` (Identitäts-Elterntabelle, v2.7
Phase 60)" in einem sonst durchgehend englischen Dokument.
**Fix:** „`person` (identity parent table, v2.7 Phase 60)".

---

### IN-05: Unerreichbarer Fallback-Zweig `PERSON_LINK_PROCESS`

**File:** `dao_impl_sqlite/src/sales_person.rs:307-311`, Konstante `:18`

**Issue:** `process.is_empty()` kann nie zutreffen — der einzige Aufrufer
(`service_impl/src/sales_person.rs:342`) übergibt die Konstante
`SALES_PERSON_SERVICE_PROCESS`. Toter Zweig plus eine Konstante, die nur ihn
bedient.
**Fix:** Zweig und Konstante entfernen und `process` direkt einsetzen; wenn der
Schutz gewollt ist, ihn als `debug_assert!(!process.is_empty())` formulieren.

---

### IN-06: `find_by_user` und `find_sales_person_by_user_id` sind zeichengleiche Duplikate

**File:** `dao_impl_sqlite/src/sales_person.rs:121-135` und `:475-489`

**Issue:** Beide Methoden führen inzwischen exakt dieselbe Query aus. Die Phase
hat die Duplikation bei der Umstellung mitgezogen (zwei identische Stellen, die
beim nächsten Umbau beide gefunden werden müssen) und sie mit einem Test
festgeschrieben (`person_self_gate.rs`, G-7) statt sie aufzulösen.
**Fix:** Eine Methode auf die andere delegieren lassen; G-7 wird damit trivial
wahr und kann entfallen.

---

### IN-07: HR-Gate von `PersonService::get` und `unlinked_users` ist ungetestet

**File:** `service_impl/src/test/person_candidates.rs:81-95`

**Issue:** `build_perm(hr)` gibt bei `hr == true` unabhängig vom Context `Ok(())`
zurück, der Forbidden-Pfad wird also nur über `build_perm(false)` erreicht — und
dort nur für `create` (K-8) und `unlinked_sales_persons` (K-9). Für `get` und
`unlinked_users` existiert kein Gate-Test, obwohl alle vier Methoden laut
Modulkopf gleichwertig HR-gated sein sollen (T-61-05).
**Fix:** Zwei Tests ergänzen: `get_ohne_hr_privileg_ist_forbidden` und
`unlinked_users_gated`, jeweils nach dem Muster von K-9.

---

_Reviewed: 2026-09-17T09:53:20Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
