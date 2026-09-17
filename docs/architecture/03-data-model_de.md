# Datenmodell

## Physisches Schema (ER)

Das aktuelle ER-Modell wird automatisch aus den `migrations/sqlite/`-
Dateien generiert und ist in
[`diagrams/db-schema-er.mmd`](./diagrams/db-schema-er.mmd) als
Mermaid-Diagramm hinterlegt.

## Logisches Modell (Aggregate)

Die Aggregat-Sicht — welche Objekte fachlich zusammengehören und wie
Business-Regeln die Grenzen ziehen — ist in
[`diagrams/domain-aggregates.mmd`](./diagrams/domain-aggregates.mmd).

Kern-Aggregate:

- **Employee** — Sales Person + Work Details (Contract) + Unavailable + Vacation Offset.
- **Shiftplan** — Slots + Catalog + Special Days.
- **Booking** — Booking + Booking Log.
- **Absence** (v1.0+) — range-basierte Abwesenheiten.
- **Legacy Extra Hours** (vor Cutover) — Single-Day-Zeilen + Custom-Kategorien.
- **Accounting** — Carryover + Vacation Balance + Report.
- **Billing Period** — Snapshot mit `snapshot_schema_version`.
- **Rebooking** (v2.6, Phase 54) — audit-fähige Batches, die
  Freiwillig-Stunden gedeckelter Mitarbeiter*innen ausgleichen; siehe
  Feature [F14](../features/F14-rebooking.md).
- **Week Metadata** — Week Status + Week Message.
- **Auth & Session** — User + Role + Session + Invitation.
- **System** — Feature Flag + Toggle + Scheduler.
- **Person** (v2.7, Phase 60) — Identitäts-Klammer über dem
  Employee-Aggregat und dem Auth-&-Session-Aggregat: eine `person`-Zeile pro
  realem Menschen, optional referenziert von `sales_person.person_id` und
  `user.person_id`. Sie ersetzt das Employee-Aggregat nicht — sie spannt sich
  darüber.

## Konventionen im Schema

### Soft-Delete

Fast jede Tabelle hat eine `deleted` (nullable timestamp) Spalte. Reader
filtern **immer** `WHERE deleted IS NULL`. Das ist eine Konvention, kein
DB-Constraint — d.h. Reviewer müssen prüfen, dass jeder neue
`query!`/`query_as!` diesen Filter setzt.

### Time-Spalten

- **`from` / `to` (Date):** Halb-offene Ranges? Geschlossene? Konvention
  variiert pro Tabelle. **[Zu prüfen]** in `absence`, `employee_work_details`,
  `sales_person`.
- **Timestamps:** Speichern SQLite-native. Zeitzone: **[Zu prüfen]**
  (UTC vs Local-Berlin).

### Foreign Keys

SQLite prüft FKs nur, wenn `PRAGMA foreign_keys = ON` aktiv ist. **In Shifty
ist die Durchsetzung aktiv** — allerdings nicht, weil das Projekt das Pragma
setzt. sqlx schaltet es selbst ein: `SqliteConnectOptions::default()` trägt
`foreign_keys = ON` in die Pragma-Menge ein, die auf jede Verbindung
angewendet wird (`sqlx-sqlite-0.8.6/src/options/mod.rs:185`), und
`shifty_bin/src/main.rs:1645` verbindet über eine schlichte
`SqlitePool::connect("sqlite:./localdb.sqlite3")`-URL, ohne das zu
überschreiben.

Genau deshalb führt ein Quelltext-Grep hier in die Irre: die Suche nach
`foreign_keys` über alle `*.rs` und `*.sql` liefert tatsächlich null Treffer —
das Pragma kommt vom Treiber, nicht aus der Codebase. Empirisch nachgefahren
gegen die Produktions-Datei und noch einmal im Test-Harness: `PRAGMA
foreign_keys` liefert `1`, und ein Insert mit unauflösbarem Elternverweis wird
mit `FOREIGN KEY constraint failed` (SQLite-Code 787) abgelehnt.

Als Beispiel dient hier ein **weiterhin existierender** Fremdschlüssel:
`sales_person_unavailable.sales_person_id` → `sales_person(id)`
(`migrations/sqlite/20240731043118_add-table-sales-person-unavailable.sql:14`).
`PRAGMA foreign_key_list(sales_person_unavailable)` führt ihn auf, und ein
Insert mit zufälliger `sales_person_id` scheitert an
`FOREIGN KEY constraint failed`. Nachgefahren am 2026-09-17 gegen eine
Wegwerf-Datenbank, die mit der **vollen** Migrationskette migriert wurde.

(Eine frühere Fassung dieses Abschnitts führte denselben Beleg an der
1:1-Verknüpfungstabelle zwischen Sales Person und Login-Benutzer. Diese
Tabelle ist in Phase 61 aufgelöst worden — siehe *Abgrenzung* unten —, deshalb
wurde das Beispiel ersetzt und nicht als Behauptung über eine nicht mehr
existierende Tabelle stehen gelassen.)

Was die FK-Prüfung **nicht** leistet: Soft Delete. Eine nur als `deleted`
markierte Elternzeile existiert weiterhin als Zeile, der Fremdschlüssel bleibt
also erfüllt, während jede Live-Query mit `WHERE deleted IS NULL` einen ins
Leere zeigenden Verweis sieht. Für diese Klasse von Invarianten ist ein
Fremdschlüssel das falsche Werkzeug — siehe `docs/domain/edge-cases.md` §8.

Was über referentielle Integrität hinaus wirklich halten muss, muss als
`UNIQUE`-Index oder `CHECK` formuliert werden — siehe den Phase-60-Abschnitt
weiter unten.

> **Korrektur (2026-09-17, Plan 60-04):** Eine frühere Fassung dieses
> Abschnitts behauptete, die FK-Deklarationen seien „dekorative Dokumentation,
> kein durchgesetztes Constraint [D-60-06]". Diese Schlussfolgerung war falsch;
> sie stützte sich allein auf den Quelltext-Grep und übersah den
> sqlx-Default. D-60-06 — die FK-Durchsetzung in Phase 60 nicht anzufassen —
> bleibt als Scope-Entscheidung bestehen, ihre Prämisse über den Ist-Zustand
> nicht.

### Views

Manche Aggregat-Reads gehen über Views (`bookings_view` — Migration
`20240728155625_add-bookings-view.sql`, erweitert um User-Tracking in
`20250115000001_update-bookings-view-add-user-tracking.sql`). Views sind
komfortabel, aber ändern sich mit ihren zugrunde liegenden Tabellen —
Migration-Ordnung beachten.

### Enum-Spalten

Kategorien wie `ExtraHoursCategory` werden als `TEXT` mit Enum-Namen
persistiert (z.B. `"ExtraWork"`, `"Vacation"`). Neue Varianten brauchen
sowohl Code-Change (Enum-Erweiterung + Konvertierung) als auch
DB-Migration, wenn ein Constraint / Check die Werte einschränkt.

**[Zu prüfen]** — ob es explizite CHECK-Constraints auf Category-Spalten
gibt.

## Migration-Historie (High-Level)

Chronologisch geordnet — Details siehe die einzelnen Feature-Dokus:

| Zeitraum | Meilenstein |
| --- | --- |
| 2024-04 | User & Rollen, initiale RBAC. |
| 2024-05 – 2024-07 | Slot + Sales Person + Booking + Booking-View. |
| 2024-08 | `min_resources`-Spalte. |
| 2024-10 | Special-Days-Tabelle, Weekday + Vacation auf Working Days. |
| 2024-11 | Session-Tabelle, Constraint-Verschärfung. |
| 2024-12 | Yearly Carryover (Hours + Vacation). |
| 2025-01 | User-Tracking auf Bookings + View, Week-Message. |
| 2025-04 | Custom Extra Hours. |
| 2025-08 | Billing Period, Text Template. |
| 2025-10 | User Invitation, Session-Tracking. |
| 2026+ | Absence-System-Cutover, Multi-Plan-Support. |
| 2026-07 | v2.6 Phase 54 — Tabellen `rebooking_batch` + `rebooking_batch_entry`, Marker-Spalte `extra_hours.source`, Voluntary-Rebooking-Toggle-Seed. |
| 2026-09 | v2.7 Phase 60 — `20260916000000_create-person-backfill.sql`: Identitäts-Elterntabelle `person`, nullable `person_id` an `sales_person` + `user`, zwei vollständige UNIQUE-Indizes, idempotenter selbst-verifizierender Backfill. |

Die vollständige Liste steht in `migrations/sqlite/` — Dateinamen sind
sortiert nach Zeitstempel-Prefix.

## Wo das Datenmodell "spricht"

- **Basic-Service-DAOs** kennen ihre Tabelle direkt.
- **Business-Logic-Services** kennen keine DAOs außer die transitiv
  über konsumierte Basic-Services.
- **Reporting** liest über mehrere Aggregate hinweg und ist damit die
  Stelle, an der Schema-Änderungen am schnellsten Kontakt zur
  Balance-Rechnung bekommen.

## Phase 54 (v2.6) — Rebooking-Datenmodell-Ergänzungen

Migration-Set `20260707000000..02` legt das Datenmodell-Fundament
für Feature [F14](../features/F14-rebooking.md).

### `rebooking_batch` (Parent)

Audit-fähiger Batch gepaarter `extra_hours`-Zeilen, die
Freiwillig-Stunden eines/einer gedeckelten Mitarbeiter*in in einer
ISO-Woche ausgleichen.

| Spalte | Typ | Anmerkung |
| --- | --- | --- |
| `id` | BLOB(16) PK | UUID v4. |
| `sales_person_id` | BLOB(16) | Mitarbeiter, auf den der Batch bucht. |
| `iso_year`, `iso_week` | INT | ISO-Jahr + ISO-Woche des Reconciliation-Fensters. |
| `kind` | TEXT | `Manual` \| `HrSuggestion` \| `AutoCron` \| `AutoCronBackfill`. |
| `state` | TEXT | `Pending` \| `Approved` \| `Rejected` \| `SkippedLocked`. |
| `created`, `approved`, `approved_by` | TEXT | Audit-Zeitstempel + Username; `approved*` NULL bis `state = Approved`. |
| `deleted` | TEXT nullable | Soft-Delete-Marker. |
| `update_process`, `update_version` | Audit-Spalten |

**Constraint [D-54-DM-01]** — partieller UNIQUE-Index
`rebooking_batch_week_unique_idx` auf
`(sales_person_id, iso_year, iso_week) WHERE deleted IS NULL` —
*global über alle `kind`s*. Claim-on-Suggest: sobald HR einen
Pending-Batch für Woche X öffnet, kann der Phase-56-F4-Cron nicht
mit einem zweiten `AutoCron`-Batch für dieselbe Woche reinlaufen.
Zwei Performance-Indices existieren daneben
(`rebooking_batch_state_idx` auf `state`,
`rebooking_batch_entry_sp_idx` auf `sales_person_id` in der
Child-Tabelle).

### `rebooking_batch_entry` (Child)

Slot-Payload pro Batch, eine Zeile pro umgebuchtem Stundenpaket.

| Spalte | Typ | Anmerkung |
| --- | --- | --- |
| `id` | BLOB(16) PK | |
| `batch_id` | BLOB(16) FK → `rebooking_batch(id)` | Kein CASCADE — Soft-Delete-Muster. |
| `sales_person_id` | BLOB(16) | Denormalisiert für Query-Performance. |
| `hours` | REAL | Absolute Stundenzahl, die umgebucht wird. |
| `balance_before` | REAL | Balance-Snapshot zum Vorschlagszeitpunkt (Audit). |
| `voluntary_actual`, `voluntary_committed` | REAL | Snapshot von F1-Zähler + F2-pro-rata-Soll zum Vorschlagszeitpunkt. |
| `extra_hours_out_id`, `extra_hours_in_id` | BLOB(16) nullable | FKs in `extra_hours` — atomar gesetzt beim Übergang state → Approved (Phase-55-Writer). |
| `created`, `deleted`, `update_process`, `update_version` | Audit-Spalten |

### `extra_hours.source` Marker-Spalte ([D-54-DM-02])

Additive Spalte `source TEXT NOT NULL DEFAULT 'manual'`. Zwei aktive
Domain-Werte:

- **`manual`** — jede Zeile aus HR-CRUD, Absence-Convert, Dev-Seed und
  REST-TO-Writern. Bestandszeilen migrieren per Column-DEFAULT.
- **`rebooking`** — reserviert für F3/F4/F5-Writer (Phase 55+). In
  Phase 54 setzt kein Writer diesen Wert.

Reader-Konsequenz: Aggregate, die unter zukünftigen Rebooking-Paaren
balance-neutral bleiben müssen, filtern `source = 'manual'`. Erster
Live-Konsument ist `voluntary_ist_total_for_year(..)` in
`service_impl/src/reporting.rs`.

### Toggle-Seed `voluntary_rebooking_auto_active_from`

Migration `20260707000002_seed-voluntary-rebooking-toggle.sql`
seedet idempotent `INSERT OR IGNORE INTO toggle` mit
`enabled = 0, value = NULL`. In Phase 54 inaktiv — der F4-Cron in
Phase 56 liest den ISO-Datums-Wert, um die AutoCron-Writer-Kette zu
aktivieren.

## Phase 60 (v2.7) — Person-Identitäts-Elterntabelle

Migration `20260916000000_create-person-backfill.sql` legt das
Identitäts-Fundament der v2.7-Personenverwaltung für die Features
[F01](../features/F01-employee-management.md) und
[F12](../features/F12-auth-session.md). Phase 60 ist **reine** Schema- +
Backfill-Arbeit — kein DAO, kein Service, kein Endpoint, keine sichtbare
Verhaltensänderung.

### `person` (Identitäts-Elterntabelle)

Eine Zeile pro realem Menschen. `person` ist die Klammer über das
**Employee**-Aggregat (`sales_person`) und das **Auth & Session**-Aggregat
(`user`).

| Spalte | Typ | Anmerkung |
| --- | --- | --- |
| `id` | BLOB(16) PK | 16 Zufallsbytes aus `randomblob(16)`, roh gespeichert und über `Uuid::from_slice` zurückgelesen. Bewusst **keine** RFC-4122-UUID — SQLite setzt weder Version-Nibble noch Variant-Bits, und nichts im System interpretiert den Wert als versionierte UUID. |
| `created` | TEXT | Anlage-Zeitstempel. |
| `update_process` | TEXT | Herkunftsmarker — Backfill-Zeilen tragen `phase-60-person-backfill`. |
| `update_version` | BLOB(16) | Audit-Spalte. |

Diese vier Spalten sind der **vollständige** Satz:

- **Keine `deleted`-Spalte [D-60-04].** In v2.7 wird keine Person gelöscht,
  und eine ungenutzte Soft-Delete-Spalte lädt zu einer halb implementierten
  Semantik ein. Falls eine spätere Phase ein „Person löschen" braucht, kommt
  die Spalte dort dazu. Das ist eine bewusste Ausnahme von der oben
  beschriebenen Soft-Delete-Konvention; festgezurrt vom Schema-Test
  `person_table_has_no_deleted_column`.
- **Keine Namensspalten.** Namen bleiben, wo sie sind — `user.name` und
  `sales_person.name`. Echte Personenstammdaten (E-Mail, Telefon,
  Geburtsdatum) sind Future-Scope (PERS-F-01), nicht v2.7.

### `sales_person.person_id` und `user.person_id`

Beide Spalten sind `BLOB(16)`, **nullable, ohne DEFAULT**.

Die Optionalität ist eine **fachliche Zusage, kein Übergangszustand
[D-60-03]**: eine Person darf einen Benutzer **oder** eine Sales Person
**oder** beides haben. Reine HR-/Admin-Accounts ohne Sales Person und
Freiwillige ohne Login sind gleichwertige, dauerhaft unterstützte
Ausprägungen. Es wird nirgends automatisch die fehlende Gegenseite angelegt.
Der Unique-Index weiter unten verbietet ausschließlich **zwei** Träger
derselben Seite an einer Person — „null Träger" ist der Normalfall zweier der
drei Ausprägungen.

### Eindeutigkeit — `sales_person_person_id_unique_idx`, `user_person_id_unique_idx`

Beide Indizes sind **vollständig, nicht partiell**: keiner trägt eine
`WHERE deleted IS NULL`-Filterklausel.

Das weicht bewusst vom dominanten Repo-Muster ab, das unter
*Konventionen im Schema* beschrieben ist (vgl. den partiellen
`rebooking_batch_week_unique_idx`). Begründung [D-60-01]: eine soft-gelöschte
Sales Person **behält** ihre Person-Identität dauerhaft. Gäbe die Löschung die
Identität frei, könnten zwei verschiedene Menschen über die Zeit dieselbe
Person-Identität belegen, und write-once-Snapshots in
`billing_period_sales_person` von vor Jahren würden auf die falsche Person
auflösen. Bewusst in Kauf genommener Preis: ein zurückkehrender Mitarbeiter
ist zunächst zwei Personen; ein Zusammenführen ist Future-Scope, nicht v2.7.

**Diese Entscheidung ist one-way.** Nach Produktiv-Betrieb würde eine
Umstellung auf einen partiellen Index bereits vergebene Identitäten neu binden
und historische Abrechnungs-Snapshots neu zuordnen. Ein späterer Aufräum-Pass
darf diese beiden Indizes **nicht** an die Partial-Index-Konvention angleichen.
Die Regressions-Guards `sales_person_index_is_not_partial` und
`user_index_is_not_partial` lesen die gespeicherte DDL aus `sqlite_master` und
schlagen fehl, sobald die Filterklausel auftaucht. `user` hat gar keine
Soft-Delete-Spalte, dort stellt sich die Frage nicht [D-60-02].

### Durchsetzung: Index, nicht Fremdschlüssel

Die 1:1-Garantie, die früher die beiden `UNIQUE`-Constraints der
Verknüpfungstabelle trugen, sitzt seit Phase 61 **allein** in diesen beiden
Indizes. Sie wird **von der Datenbank** durchgesetzt, nicht von einer
Applikationsprüfung [D-60-05] — SQLite verbietet `UNIQUE` bei
`ALTER TABLE … ADD COLUMN`, daher der separate `CREATE UNIQUE INDEX`.

Der Nachweis läuft über den Index, **nicht** über einen Fremdschlüssel — aus
einem anderen Grund als ursprünglich festgehalten. `person_id` trägt gar keine
`FOREIGN KEY`-Deklaration, SQLite hat auf dieser Kante also nichts
durchzusetzen; die Eindeutigkeitsgarantie muss ohnehin aus dem Index kommen.
Die FK-Durchsetzung als solche *ist* in diesem Repo aktiv (sqlx setzt das
Pragma pro Verbindung — siehe *Konventionen im Schema → Foreign Keys* weiter
oben). Eine FK-Deklaration auf `person_id` zu ergänzen war nicht
Phase-60-Scope [D-60-06].

### Backfill

Ein einmaliger, idempotenter SQL-Backfill in derselben Migrationsdatei (und
damit in einer Transaktion):

- Er trägt bewusst **keinen** Soft-Delete-Filter — jede historische
  `sales_person`-Zeile bekommt eine Identität, auch die soft-gelöschten
  (PERS-03). Sonst würden write-once-Snapshots in
  `billing_period_sales_person` später ins Leere auflösen.
- Idempotenz-Schlüssel ist die noch leere `person_id`-Spalte
  (`WHERE person_id IS NULL`); das `person`-Insert schließt bereits vorhandene
  Zeilen aus. Ein zweiter Lauf ändert weder Zeilenzahlen noch eine einzige
  Zuordnung (PERS-02).
- Bei einem verknüpften Paar würfelt nur die Sales-Person-Seite ein
  `randomblob(16)`; die Benutzer-Seite **kopiert** den Wert über eine
  korrelierte Subquery zurück, damit beide Seiten eines Paars auf dieselbe
  Person zeigen.
- Am Ende **verifiziert sich die Migration selbst** [D-60-07]: die Zählprobe
  `COUNT(person) = COUNT(sales_person) + COUNT(user) − COUNT(Verknüpfungszeilen)`
  läuft als harte Assertion innerhalb der Migrations-Transaktion (eine
  Wegwerf-`TEMP`-Tabelle mit `CHECK`-Constraint, weil SQLite `RAISE()` nur in
  Triggern kennt). Geht sie nicht auf, rollt die gesamte Migration zurück und
  der Server startet nicht — das ist besser, als einen halb ausgeführten
  Backfill zu committen.
  **Diese Probe ist historisch.** Sie galt zum Zeitpunkt des Backfills und
  steht weiterhin in jener Migrationsdatei, ist gegen eine aktuelle Datenbank
  aber **nicht mehr auswertbar**: ihr letzter Term zählt Zeilen der
  Verknüpfungstabelle, und die ist in Phase 61 aufgelöst worden. Für einen
  Datenaudit ist sie damit unbrauchbar — dafür dient die Invariante unten.
  (Der Phase-60-Nachweis ist erhalten: die Regressionssuite
  `shifty_bin/src/person_link_migration_test.rs` stellt den Schema-Stand vor
  Phase 61 über einen `Migrator` her, der vor der DROP-Migration endet; dort
  bleibt die Zählprobe prüfbar.)
- Backfill-Zeilen bleiben dauerhaft an `update_process =
  'phase-60-person-backfill'` erkennbar.
- Es gibt keine Down-Migration und keine Backup-Tabelle in der Datenbank
  [D-60-09]. Der Wiederherstellungspfad ist die vor dem Deploy gezogene Kopie
  der SQLite-Datei.

### Die dauerhaft prüfbare Invariante (ab Phase 61)

Ohne die Verknüpfungstabelle lautet die Aussage, die sich gegen jede laufende
Datenbank auditieren lässt:

```sql
-- beide müssen 0 sein
SELECT COUNT(*) FROM sales_person WHERE person_id IS NULL;
SELECT COUNT(*) FROM user         WHERE person_id IS NULL;

-- muss 0 sein: keine Person ohne Träger
SELECT COUNT(*) FROM person p
WHERE NOT EXISTS (SELECT 1 FROM sales_person sp WHERE sp.person_id = p.id)
  AND NOT EXISTS (SELECT 1 FROM user         u  WHERE u.person_id  = p.id);
```

In Worten: **jede `sales_person`- und jede `user`-Zeile trägt genau ein
`person_id`, und jede `person`-Zeile hat mindestens einen Träger.** Das „genau
eins" der ersten Hälfte tragen die beiden Unique-Indizes plus die
Nicht-NULL-Eigenschaft; das „mindestens ein Träger" der zweiten Hälfte halten
die Merge- und Split-Pfade aufrecht (`assign_to_user` löscht die absorbierte
`person`-Zeile, und `discard_assigned_user` legt die neue an, **bevor** der
Benutzer umgehängt wird).

Zu beachten ist die Asymmetrie zum Schema: `person_id` ist **nullable** und
trägt keine `FOREIGN KEY`-Deklaration — und das bleibt so. SQLite kann eine
Spalte ohne Table-Rebuild nicht auf `NOT NULL` verschärfen, und ein Rebuild von
`sales_person` müsste jede referenzierende Tabelle neu verdrahten. Die
„nie NULL"-Hälfte der Invariante trägt seit Phase 61 deshalb **der Code**
[D-61-01], nicht das Schema: `SalesPersonDaoImpl::create` schreibt die eigene
`person`-Zeile innerhalb der Transaktion des Aufrufers,
`PermissionDaoImpl::create_user` tut dasselbe in einer methoden-internen, und
die DROP-Migration hat jede vorbestehende Zeile nachgezogen [D-61-06]. Wer
einen dritten Anlage-Pfad schreibt, muss das von Hand einhalten — die
Datenbank fängt das Versäumnis nicht ab.

### Abgrenzung

- **Die 1:1-Verknüpfungstabelle existiert nicht mehr.** Während Phase 60 war
  sie noch da — jene Phase nutzte sie als die Wahrheit, gegen die ihr Backfill
  geprüft wurde —, und ihre Auflösung war Aufgabe von Phase 61. Der folgende
  Absatz ist die Historie dieser Scope-Teilung, keine Beschreibung des
  aktuellen Schemas. **Nachtrag (Phase 61, erledigt):** Alle
  lesenden *und* schreibenden Verknüpfungs-Pfade in
  `dao_impl_sqlite/src/sales_person.rs` laufen über
  `u.person_id = sp.person_id` [D-61-08, D-61-03, D-61-05], und
  `migrations/sqlite/20260917000000_drop-sales-person-user.sql` hat die
  Tabelle aufgelöst [D-61-07]. Diese Migration **zieht** zuerst jede Zeile
  nach, die noch kein `person_id` trägt (gleiche Gewinner-Logik wie Phase 60:
  die Sales Person generiert, der Benutzer kopiert), und **prüft sich dann
  selbst** — bliebe danach eine Verknüpfung ohne übereinstimmendes
  `person_id`-Gegenstück auf beiden Seiten, rollt der `CHECK`-Constraint die
  gesamte Migration zurück und der `DROP` läuft nie [D-61-06]. Es gibt keine
  Down-Migration; der Wiederherstellungspfad ist die vor dem Deploy gezogene
  Dateikopie.
- Zeilen, die **nach** der Migration entstehen (Login-Auto-Anlage,
  `create_sales_person`), bekommen in Phase 60 **kein** `person_id` — die
  Vergabe beim Anlegen ist Phase-61-Scope. Kein Trigger, kein DAO-Eingriff in
  dieser Phase [D-60-10]. Die Zählprobe gilt für den Zustand *unmittelbar nach*
  der Migration.
- `sales_person.id` und `user.name` werden nicht umgeschlüsselt; beide bleiben
  bitgleich (PERS-06).

### Snapshot-Vertrag

`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13** — kein Bump. `person` ist rein
administrativ: kein neuer persistierter `value_type`, keine geänderte
Computation, kein geändertes Input-Set, und
`billing_period_sales_person.sales_person_id` ist unverändert. Keine der vier
Bump-Klauseln aus `CLAUDE.md` greift, und „vorsichtshalber bumpen" wäre hier
ausdrücklich falsch — es ließe jeden älteren Snapshot so aussehen, als sei er
unter anderen Regeln geschrieben worden.

## Verwandte Randfälle

- Soft-Delete-Konsistenz → [`../domain/edge-cases.md#8-soft-delete-konsistenz`](../domain/edge-cases.md#8-soft-delete-konsistenz)
- `.sqlx`-Cache und Migrations → [`../domain/edge-cases.md#10-migrations--sqlx-offline-cache`](../domain/edge-cases.md#10-migrations--sqlx-offline-cache)
