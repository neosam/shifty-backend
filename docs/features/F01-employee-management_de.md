# Feature: Employee Management

> **Kurzform:** Anlage und Pflege der Mitarbeiter-Stammdaten (Sales Persons),
> ihrer Arbeitsverträge (Employee Work Details), ihrer wiederkehrenden
> Verfügbarkeits-Sperren (Unavailable) und ihrer Zuordnung zu Schichtplänen.

**Cluster-ID:** F01
**Status:** produktiv
**Erstmalig eingeführt:** v0.1 (Basistabellen 2024-05), sukzessiv erweitert bis v2.4
**Zuständige Crates:**
`service::{sales_person, sales_person_shiftplan, sales_person_unavailable, employee_work_details, person, person_directory}`,
`service_impl::{sales_person, sales_person_shiftplan, sales_person_unavailable, employee_work_details, person, person_directory}`,
`dao::{sales_person, sales_person_shiftplan, sales_person_unavailable, employee_work_details, person, person_directory}`,
`dao_impl_sqlite::{sales_person, sales_person_shiftplan, sales_person_unavailable, employee_work_details, person, person_directory}`,
`rest::{sales_person, sales_person_shiftplan, employee_work_details, person_directory}`

---

## 1. Was ist das? (Fachlich)

Employee Management ist der Fuß des gesamten Shifty-Datenmodells: fast jede
weitere Domäne (Bookings, Absences, Extra Hours, Reporting, Billing Periods)
hängt an einer `sales_person.id`. Dieses Cluster hält vier eng verzahnte
Aggregate:

1. **Sales Person** — die Stammdaten des Angestellten (Name, Kennfarbe,
   `is_paid`, `inactive`, optionale Verknüpfung zu einem Benutzer-Login).
2. **Employee Work Details** (der historisch gewachsene Name des
   **Arbeitsvertrags**) — Wochenstunden, Arbeitstage, Urlaubsanspruch,
   Dynamik-/Deckelungs-Flags und eine feste "freiwillige Zusage" pro Woche,
   gültig für einen Zeitraum `[from_year/kw/dow, to_year/kw/dow]`.
3. **Sales Person Unavailable** — pro (Sales Person, Jahr, Kalenderwoche,
   Wochentag) ein Eintrag, an dem der Angestellte **nicht** eingeplant werden
   möchte (wiederkehrende Verfügbarkeits-Sperre auf Wochentagsebene, nicht
   das Absence-System für ganze Urlaubs-/Krank-Perioden — das lebt in F02).
4. **Sales Person Shiftplan** — welche Schichtpläne der Angestellte bebuchen
   darf, mit Berechtigungslevel (`available` oder `planner_only`).

Aus User-Sicht (siehe `docs/employee-management_de.md` und
`docs/employee-management.md` für die Endanwender-Anleitung mit Screenshots)
ist der Onboarding-Ablauf:

1. **Verkäufer anlegen** über *Benutzerverwaltung → Verkäufer → Neuen Verkäufer
   erstellen* (`shifty-dioxus/src/page/sales_person_details.rs`). Der HR-User
   trägt Name, Kennfarbe, `is_paid`/`inactive`, optional den verknüpften
   Login-Benutzer und die Schichtplan-Zuweisungen ein.
2. **Arbeitsvertrag hinterlegen** über die Mitarbeiter-Detailseite
   (`shifty-dioxus/src/page/employee_details.rs`) im Bereich *Arbeitsverträge*
   → `ContractModal`. Ohne Vertrag hat der Mitarbeiter keine Soll-Stunden,
   und die Bilanz bleibt leer.
3. **Optional Unavailable-Wochentage** eintragen — der Sales-User macht das
   selbst über die Wochenansicht des Schichtplaners.
4. **Optional Extra Hours** manuell erfassen (Cluster F02 — Extra Hours /
   Absences, nicht hier).

Deaktivierung ("Diese Person ist inaktiv") ist der bevorzugte Weg statt
echtem Löschen; Löschen macht der Backend-Service zwar als Soft-Delete
verfügbar, wird im UI aber bewusst nicht angeboten, um Bilanz-Historie und
FK-Konsistenz mit Bookings/Extra Hours zu erhalten.

## 2. Fachliche Regeln

### 2.1 Sales Person

- **Sichtbarkeit `is_paid` ist datenschutzsensibel.** Nicht-HR-User sehen
  `is_paid = None` (`SalesPersonServiceImpl::get_all`,
  `service_impl/src/sales_person.rs:62-70`). In `get()` wird das Feld nur
  ausgeliefert, wenn der aufrufende User HR ist **oder** der Zugreifende
  selbst der verknüpfte User des Sales-Person-Datensatzes ist
  (`sales_person.rs:120-143`).
- **Löschen = Soft-Delete.** `delete()` setzt `deleted` auf den aktuellen
  Zeitstempel und rotiert `version` (`sales_person.rs:253-279`). Das
  `deleted`-Feld ist **nicht via update() änderbar**
  (`ValidationFailureItem::ModificationNotAllowed("deleted")`, `sales_person.rs:228-235`).
- **Optimistic Locking über `version`.** Update verlangt, dass die
  mitgesendete `version` mit dem persistierten Wert übereinstimmt; sonst
  `ServiceError::EntityConflicts` (`sales_person.rs:220-226`).
- **Create-Guards.** `id != Uuid::nil()` → `IdSetOnCreate`; `version !=
  Uuid::nil()` → `VersionSetOnCreate` (`sales_person.rs:177-183`). Sowohl
  `id` als auch `version` werden vom Service über `UuidService` erzeugt.
- **User-Zuweisung ist 1:1.** Durchgesetzt wird sie von den beiden
  `UNIQUE`-Indizes auf `person_id` (`sales_person_person_id_unique_idx`,
  `user_person_id_unique_idx`,
  `migrations/sqlite/20260916000000_create-person-backfill.sql`) in der
  Datenbank. Das Ersetzen läuft in einer TX via `discard_assigned_user` +
  `assign_to_user` (`sales_person.rs:316-342`), also atomar für den Umzug einer
  Zuordnung.
- **Verknüpfen ist ein Merge zweier Personen** (v2.7, Phase 61 [D-61-03]).
  `SalesPersonDaoImpl::assign_to_user` hängt den Benutzer auf die `person_id`
  der Sales Person um und löscht die absorbierte `person`-Zeile. Die Sales
  Person behält ihre Identität — sie ist die Seite mit historischem Gewicht
  (Buchungen, `extra_hours`, write-once-Snapshots in
  `billing_period_sales_person`).
- **Abgelehnt wird nur echte Mehrdeutigkeit** (v2.7, Phase 61 [D-61-04]). Vor
  jedem Schreibvorgang fährt `assign_to_user` zwei Konflikt-`SELECT`s: hängt
  bereits ein anderer Benutzer an der Person der Sales Person, und hängt
  bereits eine andere Sales Person an der Person des Benutzers? Ein Treffer
  liefert `DaoError::EntityAlreadyExists` — **ohne** irgendetwas zu schreiben;
  `SalesPersonServiceImpl::set_user` übersetzt das in
  `ServiceError::EntityAlreadyExists` → HTTP 409. Der häufige Bestandsfall nach
  dem Backfill — beide Seiten tragen *je eine eigene* Person — ist
  ausdrücklich **kein** Konflikt und geht durch.
- **Lösen ist ein Split** (v2.7, Phase 61 [D-61-05]).
  `discard_assigned_user` lässt die Sales Person auf ihrer Person stehen und
  gibt dem Benutzer eine **frisch angelegte**. Die Reihenfolge ist zwingend:
  erst die neue `person`-Zeile anlegen, dann den Benutzer umhängen — andersherum
  wäre der Benutzer transient ohne gültige `person_id` und damit ohne
  Selbstzugriff. Eine trägerlose `person`-Zeile bleibt zu keinem Zeitpunkt
  zurück.
- **Ein Re-Link A→B läuft als Split(A) + Merge(B)** (v2.7, Phase 61 [P-61-B]).
  `set_user` ruft `discard_assigned_user` bedingungslos vor `assign_to_user`,
  also behält Benutzer A eine eigene frische Person und bleibt als reiner
  Benutzer bestehen. Beide Schritte liegen in **einer** Service-Transaktion:
  scheitert der Merge mit 409, wird der Split mit zurückgerollt und der
  Ausgangszustand bleibt unverändert.
- **Alle vier lesenden Verknüpfungs-Pfade lösen über `person_id` auf** (v2.7,
  Phase 61 [D-61-08]). `SalesPersonDaoImpl::find_by_user`,
  `find_sales_person_by_user_id`, `get_assigned_user` und
  `get_all_user_assignments` joinen
  `sales_person sp JOIN user u ON u.person_id = sp.person_id` statt über die
  Verknüpfungstabelle zu gehen. Die Autorisierungsentscheidung liegt bewusst
  **vollständig im SQL**: ein äquivalenter Rust-Vergleich zweier
  `Option<Uuid>` würde `None == None` als `true` werten und fail-open gehen,
  während SQLs Drei-Werte-Logik `NULL = NULL` nie wahr werden lässt. Das ist
  eine **dauerhafte Konvention**, keine Phasen-Notiz — siehe
  [`../architecture/04-auth_de.md` §Selbst-Zugriffs-Gate auf Sales-Person-Daten](../architecture/04-auth_de.md#selbst-zugriffs-gate-auf-sales-person-daten).
  Belegt von `service_impl::test::person_self_gate` (11 Tests gegen eine echte
  In-Memory-SQLite-Datenbank). Die dort früher gefahrene
  Vorher/Nachher-Permission-Matrix [D-61-09] war ein *befristetes* Artefakt:
  sie brauchte beide Datenmodelle in derselben Fixture-DB, und seit
  `migrations/sqlite/20260917000000_drop-sales-person-user.sql` ist die
  „Vorher"-Hälfte nicht mehr herstellbar. Sie wurde in Phase 61 abgebaut; die
  Accept/Reject-Assertions gegen die `person_id`-Implementierung bleiben als
  Dauer-Regressionstests stehen.
- **Keiner der vier Pfade filtert `deleted IS NULL` — und keiner darf damit
  anfangen.** `verify_user_is_sales_person` hängt an `get_assigned_user`; ein
  später nachgerüsteter Soft-Delete-Filter würde einer soft-gelöschten Sales
  Person stillschweigend den Selbstzugriff entziehen. Festgezurrt von
  `person_self_gate::soft_geloeschte_sales_person_behaelt_selbstzugriff`.

### 2.2 Employee Work Details (Arbeitsvertrag)

- **Zeitraum in KW-Auflösung** über
  `(from_year, from_calendar_week, from_day_of_week)` bis
  `(to_year, to_calendar_week, to_day_of_week)`. Der "aktive" Vertrag für
  eine Woche wird durch Tupel-Vergleich bestimmt:
  `(from_year, from_calendar_week) <= (year, calendar_week) <= (to_year, to_calendar_week)`
  (`service_impl/src/employee_work_details.rs:114-118`). **[Zu prüfen]** ob
  `from_day_of_week` / `to_day_of_week` in dieser Vergleichsregel bewusst
  ignoriert werden — die aktuelle Implementierung stellt nur auf KW-Genauigkeit ab.
- **Wochenstunden-Verteilung.** `expected_hours` ist die Wochensumme;
  `workdays_per_week` ist der Divisor für `hours_per_day()`; die
  gecheckten Wochentag-Flags (`monday`..`sunday`) definieren, welche Tage
  überhaupt Kandidaten sind (`potential_weekday_list`,
  `service/src/employee_work_details.rs:78-102`).
- **Feiertagsstunden.** `holiday_hours = expected_hours / potential_days_per_week`
  — d.h. ein Feiertag auf einem potentiellen Arbeitstag entlastet um diesen
  Anteil (`employee_work_details.rs:112-114`).
- **`is_dynamic` (v1.5)** — statt fixer Wochenstunden zählen die
  tatsächlich gearbeiteten Stunden als vereinbart (Soll = Ist). Migration
  `20251029192107_add-column-is-dynamic-to-employee-work-details.sql`.
- **`cap_planned_hours_to_expected` (v2.x)** — Deckelt die
  Balance-relevanten Stunden pro Woche auf `expected_hours`; alles darüber
  gilt als Ehrenamt (siehe `docs/employee-management_de.md` §6). Migration
  `20260426120000_add-cap-flag-to-employee-work-details.sql`.
- **`committed_voluntary` (v2.4)** — Wochenweise fest zugesagte
  freiwillige Stunden zusätzlich zum bezahlten Soll; wirkt in der
  Jahresübersicht und der Kapazitätsplanung. Migration
  `20260623120000_add-committed-voluntary-to-employee-work-details.sql`.
- **Urlaubs-Proration bei Vertrags-Rändern (D-28-04, Phase 28).**
  `vacation_days_for_year(year)` prorata-t am Jahresstart und Jahresende:
  Anfang → `vac * (ordinal(from) - 1) / days_in_year(year)` (strikt vor
  dem Start; ein 1.1.-Start subtrahiert 0),
  Ende → `vac * (1 - ordinal(to) / days_in_year(year))`
  (`employee_work_details.rs:158-194`). Regression-Suite ab
  `employee_work_details.rs:286-408`.
- **Update ist partiell und ändert `from_*` NICHT.** `update()` schreibt
  nur `to_*`, `expected_hours`, `vacation_days`, `workdays_per_week`,
  `is_dynamic`, `cap_planned_hours_to_expected`, `committed_voluntary`
  und `version` zurück
  (`service_impl/src/employee_work_details.rs:240-253`). Wochentags-Flags
  und Startpunkt bleiben, wie sie beim Create waren — Änderung dieser
  Felder verlangt einen neuen Vertrag.
- **Create-Guards.** `id != nil` → `IdSetOnCreate`; `version != nil` →
  `VersionSetOnCreate` (`employee_work_details.rs:197-208`). `created`
  wird vom Service gesetzt.
- **Delete = Soft-Delete.** `delete()` setzt `deleted` und behält den
  Datensatz für die Historie (`employee_work_details.rs:262-288`).

### 2.3 Sales Person Unavailable

- **Ein Datensatz pro Wochentag pro KW.** `create()` prüft, ob es für
  `(sales_person_id, year, calendar_week, day_of_week)` bereits eine
  ungelöschte Zeile gibt und lehnt sonst mit
  `ServiceError::EntityAlreadyExists` ab
  (`service_impl/src/sales_person_unavailable.rs:155-168`).
- **Create-Guards.** `id`, `version`, `deleted` und `created` müssen leer
  sein, sonst `IdSetOnCreate` / `VersionSetOnCreate` /
  `DeletedSetOnCreate` / `CreatedSetOnCreate`
  (`sales_person_unavailable.rs:143-154`).
- **Delete = Soft-Delete.** Setzt `deleted` und dreht `version`
  (`sales_person_unavailable.rs:190-230`).
- **Auth-Gate.** Alle Read-/Write-Ops akzeptieren **entweder** einen
  Shiftplanner **oder** den Sales-User selbst (via
  `SalesPersonService::verify_user_is_sales_person`,
  `sales_person_unavailable.rs:41-50`, `73-82`, `132-141`, `203-212`). Die
  wochenweite Sammelabfrage `get_by_week()` ist **nur** für Shiftplanner
  (`sales_person_unavailable.rs:109-111`).

### 2.4 Sales Person Shiftplan

- **Berechtigungslevel je Zuweisung.** `permission_level` ∈
  {`available`, `planner_only`} — CHECK-Constraint in der Migration
  `20260402000000_add-permission-level-to-sales-person-shiftplan.sql:1`.
- **"Leerlassen = überall erlaubt".** Wenn eine Sales Person **keinerlei**
  Shiftplan-Zuweisung hat, gilt sie in **jeder** Schichtplan-Instanz als
  bebuchbar (`get_bookable_sales_persons` / `is_eligible`,
  `service_impl/src/sales_person_shiftplan.rs:100-118` und `137-140`).
- **`planner_only` sperrt Selbst-Buchung.** Die Person taucht in
  `get_bookable_sales_persons` nur auf, wenn der aufrufende User
  Shiftplanner ist (`sales_person_shiftplan.rs:114-115` und
  `147-154`).
- **Inaktive Sales Persons werden nie aufgelistet.** Filter in
  `get_bookable_sales_persons` (`sales_person_shiftplan.rs:96-99`).
- **Vollständiges Replace bei set.** `set_shiftplans_for_sales_person`
  schreibt die Zuweisungsliste komplett neu, in derselben TX, nachdem die
  Existenz der Sales Person geprüft wurde
  (`sales_person_shiftplan.rs:52-75`).

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `person` (v2.7, Phase 60) | Identitäts-Elterntabelle über Sales Person und Login-User — eine Zeile pro realem Menschen | `id` (BLOB(16) PK), `created`, `update_process`, `update_version`. Das ist der vollständige Satz: **keine `deleted`-Spalte** [D-60-04] und **keine Namensspalten** (Namen bleiben an `sales_person.name` / `user.name`) |
| `sales_person` | Stammdaten des Angestellten | `id`, `name`, `background_color`, `is_paid`, `inactive`, `deleted`, `update_version`, `person_id` (v2.7 — nullable, `UNIQUE` über `sales_person_person_id_unique_idx`) |
| `employee_work_details` (früher `working_hours`) | Arbeitsvertrag mit Zeitraum, Wochenstunden, Urlaub | `id`, `sales_person_id`, `expected_hours`, `from_/to_year/calendar_week/day_of_week`, `workdays_per_week`, `monday`..`sunday`, `vacation_days`, `is_dynamic`, `cap_planned_hours_to_expected`, `committed_voluntary`, `created`, `deleted`, `update_version` |
| `sales_person_unavailable` | Wiederkehrende Verfügbarkeits-Sperren auf Wochentagsebene | `id`, `sales_person_id`, `year`, `calendar_week`, `day_of_week`, `created`, `deleted` |
| `sales_person_shiftplan` | M:N Sales-Person ↔ Shiftplan mit Berechtigungslevel | `sales_person_id`, `shiftplan_id`, `permission_level` ∈ {`available`,`planner_only`} |

> **Aufgelöst in v2.7 Phase 61:** Die frühere 1:1-Verknüpfungstabelle zwischen
> Sales Person und Login-User existiert nicht mehr.
> `migrations/sqlite/20260917000000_drop-sales-person-user.sql` zieht jede Zeile
> nach, die noch kein `person_id` trägt, prüft anschließend, dass keine
> verbleibende Verknüpfung ihr Gegenstück verlieren würde, und löst die Tabelle
> dann auf [D-61-06, D-61-07]. Die 1:1-Garantie tragen seither allein die beiden
> vollständigen Unique-Indizes `sales_person_person_id_unique_idx` und
> `user_person_id_unique_idx`.

### `PersonDao` — Zugriff auf die Identitäts-Elterntabelle (v2.7, Phase 61)

`dao::person::PersonDao` / `dao_impl_sqlite::person::PersonDaoImpl` ist der
einzige Lese-/Schreibpfad auf die Tabelle `person`. Das Trait bleibt bewusst
bei genau vier Methoden — keine Vorratshaltung für spätere Phasen:

| Methode | Zweck |
| --- | --- |
| `find_by_id(id, tx)` | Einzelne Identität lesen, `Option<PersonEntity>`. |
| `create(entity, process, tx)` | Insert. `id`, `created` und `version` erzeugt der Service, nicht das DAO; es wird keine SQLite-Rowid zurückgelesen — `person.id` ist ein `blob(16) PRIMARY KEY` und kein Rowid-Alias. |
| `sales_persons_without_user(tx)` | Sales Persons, deren Person keinen Login-User trägt — Kandidaten für „Login hinzufügen". Filtert `sp.deleted IS NULL`. |
| `users_without_sales_person(tx)` | Login-User, deren Person keine *lebende* Sales Person trägt — Kandidaten für „Person hinzufügen". Die Sales-Person-Seite wird **in der `ON`-Klausel** auf nicht-gelöscht eingeschränkt (`AND sp.deleted IS NULL`), damit ein User, dessen einzige Sales Person soft-gelöscht wurde, wieder Kandidat ist statt aus beiden Kandidatenlisten zu fallen [D-62-05 / WR-02]. Die Tabelle `user` selbst hat weiterhin keine Soft-Delete-Spalte. |

`PersonEntity` trägt `id`, `created`, `version` und sonst nichts — derselbe
reduzierte Zuschnitt wie die Tabelle selbst [D-60-04].

**Beide Kandidaten-Lesepfade sind LEFT-JOIN-Anti-Joins über `person_id` und
keine Leer-Prüfungen auf dieser Spalte [D-61-02].** Nach dem Phase-60-Backfill
trägt jede `sales_person`- und jede `user`-Zeile eine Person; „unverknüpft"
heißt also *es gibt keinen Join-Partner auf der Gegenseite*, nicht *der Wert
fehlt*. Der Regressionsschutz dafür ist
`service_impl::test::person_candidates::kandidat_trotz_gesetzter_person_id`:
jede Fixture-Zeile trägt ein nicht-NULL `person_id`, und beide
Kandidaten-Mengen sind trotzdem nicht leer.

**Ein Soft-Delete-Filter auf der optionalen Seite eines Outer Joins gehört in
die `ON`-Klausel, niemals in die `WHERE`-Klausel [D-62-05 / WR-02].** In
`WHERE` degeneriert der LEFT JOIN zum INNER JOIN, und der Anti-Join verliert
genau die Zeilen, für die es ihn gibt. Genau das hielt der Phase-61-Review als
Befund WR-02 zu `users_without_sales_person` fest: ein User, dessen einzige
Sales Person soft-gelöscht war, hatte weiterhin einen *formalen* Join-Partner
und fiel deshalb aus **beiden** Kandidatenlisten — die Sales Person war wegen
des Lösch-Filters kein „Login hinzufügen"-Kandidat, und der User war wegen des
Join-Partners kein „Person hinzufügen"-Kandidat. Er war nie wieder
verknüpfbar. Der Wachhund für den Fix ist
`service_impl::test::person_candidates::user_mit_nur_soft_geloeschter_sales_person_ist_kandidat`:
er seedet genau diese Konstellation und assertet, dass der User Kandidat ist.
Dieselbe Regel trägt den Verzeichnis-Join weiter unten; beide Vorkommen in
`dao_impl_sqlite` tragen einen Zeilenkommentar mit D-62-05 / WR-02, damit ein
Zurückschieben des Filters nach `WHERE` sofort als Widerspruch auffällt.

### `PersonDirectoryDao` — der Lesepfad des Personen-Verzeichnisses (v2.7, Phase 62)

`dao/src/person_directory.rs` + `dao_impl_sqlite/src/person_directory.rs`.
Bewusst ein eigenes Trait und **keine** fünfte Methode auf `PersonDao`: jenes
Trait ist auf genau vier Methoden festgezurrt, und das Verzeichnis ist eine
andere Lesart derselben Tabellen — `PersonDao` liefert Identität und
Kandidaten-Anti-Joins, dieser DAO liefert das Aggregat über alle drei
Ausprägungen.

| Methode | Zweck |
| --- | --- |
| `list_directory_entries(tx)` | Alle Verzeichniszeilen in genau einem Statement — ein Drei-Wege-Outer-Join `person` ↔ `sales_person` ↔ `user` [D-62-02]. Unsortiert; die Sortierung gehört in den Business-Logic-Service, damit sie testbar und locale-unabhängig bleibt. |
| `all_user_roles(tx)` | Alle Rollen-Zuordnungen aller Benutzer als **eine** Bulk-Query. `PermissionDao::roles_for_user` bleibt unangetastet — es hat seinen Platz im Auth-Pfad, aber ein Aufruf pro Verzeichniszeile wäre genau das N+1-Muster, das diese Phase abbaut. |

`PersonDirectoryRow` ist flach. Die Verkäufer-Seite (`sales_person_id`,
`sales_person_name`, `background_color`, `is_paid`, `inactive`) ist genau dann
leer, wenn die Person keine *lebende* Sales Person trägt; `username` ist genau
dann leer, wenn die Person kein Benutzerkonto trägt. Mindestens eine der beiden
Seiten ist immer gefüllt.

**Zwei Join-Regeln tragen die gesamte Semantik:**

- `AND sp.deleted IS NULL` steht in der **`ON`-Klausel**, niemals in `WHERE`.
  In `WHERE` degeneriert der LEFT JOIN zum INNER JOIN, und jede Person ohne
  Sales Person fiele aus dem Verzeichnis — dieselbe Falle wie WR-02 im
  Kandidaten-Lesepfad. Regressionsschutz:
  `service_impl::test::person_directory_dao::soft_geloeschte_sales_person_laesst_benutzer_sichtbar`.
- `WHERE sp.id IS NOT NULL OR u.name IS NOT NULL` schließt trägerlose
  `person`-Zeilen aus — eine Person ohne lebende Sales Person und ohne
  Benutzerkonto hat nichts anzuzeigen. Regressionsschutz:
  `service_impl::test::person_directory_dao::person_ohne_traeger_erscheint_nicht`.

Soft-gelöschte Sales Persons erscheinen nicht, inaktive schon [D-62-04]:
„Inaktiv" ist ein Anzeige-Zustand mit eigenem Badge, „gelöscht" ist Abwesenheit.

### Migrations

Chronologische Historie (nur die für dieses Cluster relevanten Files aus
`migrations/sqlite/`):

- `20240506114107_add_sales_person.sql` — Basistabelle `sales_person` plus die
  1:1-Verknüpfungstabelle (mit `UNIQUE`-Constraints). Die Verknüpfungstabelle
  ist in v2.7 Phase 61 aufgelöst worden, siehe unten; die Migrationsdatei selbst
  bleibt byte-identisch, weil ihre Checksumme Teil des sqlx-Migrations-Ledgers
  ist.
- `20240614075633_shiftplanner-role.sql` — legt Rolle + Privilege
  `shiftplanner` an. Kein direktes Feld an einer Cluster-Tabelle, aber
  Auth-Voraussetzung für viele Endpoints hier.
- `20240618035521_sales-person-color.sql` — `background_color`
  (Default `#FFF`, NOT NULL).
- `20240618125847_paid-sales-persons.sql` — `is_paid` an `sales_person`
  + Ur-Tabelle `working_hours` (später umbenannt) + `extra_hours` (letzteres
  gehört fachlich zu Cluster F02).
- `20240731043118_add-table-sales-person-unavailable.sql` — Tabelle für
  Wochentags-Sperren.
- `20241023062246_add-weekdays-and-vacation-to-working-days.sql` — großer
  Umbau: `working_hours` → `employee_work_details`, Wochentag-Flags,
  `from_/to_day_of_week`, `vacation_days`, Drop von `days_per_week`.
- `20241118165756_add-role-shiftplan-edit.sql` — indirekt relevant für
  Bookings, nicht für dieses Cluster.
- `20241215063132_add_employee-yearly-carryover.sql` /
  `20241231065409_add_employee-yearly-vacation-carryover.sql` — Carryover-
  Persistierung; **[Zu prüfen]** ob die Carryover-Tabellen fachlich noch
  hier oder in Reporting (F04/F05) gehören. Sie referenzieren
  `sales_person(id)`, gehören strukturell aber zum Balance-Feature.
- `20251029192107_add-column-is-dynamic-to-employee-work-details.sql` —
  Flag `is_dynamic` (v1.5).
- `20260331000000_add-sales-person-shiftplan.sql` /
  `20260402000000_add-permission-level-to-sales-person-shiftplan.sql` —
  Zuordnungstabelle plus `permission_level` mit CHECK-Constraint
  (v2.x-Serie).
- `20260426120000_add-cap-flag-to-employee-work-details.sql` — Flag
  `cap_planned_hours_to_expected`.
- `20260623120000_add-committed-voluntary-to-employee-work-details.sql` —
  Feld `committed_voluntary` (v2.4).
- `20260916000000_create-person-backfill.sql` (v2.7, Phase 60) — legt die
  Identitäts-Elterntabelle `person` an, ergänzt die nullable Spalte
  `sales_person.person_id` (dazu `user.person_id`, siehe
  [F12](./F12-auth-session_de.md)) und setzt die 1:1-Garantie über den
  **vollständigen** Unique-Index `sales_person_person_id_unique_idx` durch —
  bewusst ohne `WHERE deleted IS NULL`-Filter, damit eine soft-gelöschte Sales
  Person ihre Identität dauerhaft behält [D-60-01, one-way]. Ein einmaliger,
  idempotenter Backfill gibt jeder Bestandszeile eine Identität (einschließlich
  der soft-gelöschten) und verifiziert sich über eine Zählprobe innerhalb der
  Migrations-Transaktion selbst. Reine Schema- + Backfill-Arbeit — kein DAO-
  und kein Service-Eingriff in dieser Phase.

### Beziehungen

Alle Tabellen des Clusters hängen per Foreign Key an `sales_person(id)`
(Ausnahme: `sales_person_shiftplan` hängt zusätzlich an `shiftplan(id)`).
SQLite prüft FKs nur, wenn `PRAGMA foreign_keys=ON` aktiv ist — **und das ist
es**: sqlx schaltet das Pragma per Default auf jeder Verbindung ein
(`sqlx-sqlite-0.8.6/src/options/mod.rs:185`), obwohl der Projektcode es
selbst nirgends setzt. Ein Grep über `*.rs`/`*.sql` findet deshalb nichts und
führt hier in die Irre; zur Laufzeit liefert `PRAGMA foreign_keys` eine `1`,
und ein verwaister Insert wird mit `FOREIGN KEY constraint failed` abgelehnt.

Was die Prüfung nicht abdeckt, ist die Soft-Delete-Semantik
(`WHERE deleted IS NULL`): eine soft-gelöschte Elternzeile ist weiterhin eine
Zeile, der Fremdschlüssel bleibt also erfüllt, während Live-Queries einen ins
Leere zeigenden Verweis sehen (siehe `docs/domain/edge-cases.md` §8). Diese
Grenze ist der Grund, warum die FK-Durchsetzung hier weniger bringt, als es
aussieht.

Die Identitäts-Elterntabelle `person` (v2.7, Phase 60) sitzt über beiden
Trägern. Beide Kanten sind optional und eindeutig: eine Person darf einen
Benutzer **oder** eine Sales Person **oder** beides haben, und es wird
nirgends automatisch die fehlende Gegenseite angelegt [D-60-03]. Details in
[`../architecture/03-data-model_de.md`](../architecture/03-data-model_de.md).

```
person ─┬─ 0..1 ─ user          (user.person_id — nullable, UNIQUE)
        └─ 0..1 ─ sales_person  (sales_person.person_id — nullable, UNIQUE)

                                        sales_person ──┬──< employee_work_details
                                                       ├──< sales_person_unavailable
                                                       └──< sales_person_shiftplan >── shiftplan

(user ↔ sales_person läuft seit v2.7 Phase 61 ausschließlich über die
 gemeinsame Person — eine Verknüpfungstabelle gibt es nicht mehr.)
```

## 4. Service-API

Alle Services sind nach der Konvention in `shifty-backend/CLAUDE.md`
klassifiziert:

| Service | Tier | Fremd-Domain-Services? |
| --- | --- | --- |
| `SalesPersonService` | **Basic** | nein — nur `PermissionService`, `ClockService`, `UuidService`, `TransactionDao` |
| `EmployeeWorkDetailsService` | Grenzfall / **Business-Logic-lite** | konsumiert `SalesPersonService` als Auth-Helfer (`verify_user_is_sales_person`, `find_by_sales_person_id`) |
| `SalesPersonUnavailableService` | Grenzfall / **Business-Logic-lite** | dito, für Auth-Helfer |
| `SalesPersonShiftplanService` | **Business-Logic** | konsumiert `SalesPersonService` (Existence + Aggregat-Listing für `get_bookable_sales_persons`) |
| `PersonService` (v2.7, Phase 61) | **Basic** | nein — nur `PersonDao`, `PermissionService`, `ClockService`, `UuidService`, `TransactionDao` |
| `PersonDirectoryService` (v2.7, Phase 62) | **Business-Logic** | nein — nur `PersonDirectoryDao`, `PermissionService`, `TransactionDao`. Business-Logic, weil er zwei Aggregate zusammenführt und mit `PersonKind` eine Cross-Entity-Invariante pflegt, nicht weil er einen anderen Service konsumiert. |

Die drei Grenzfall-Services nutzen `SalesPersonService` **nur** zur
Autorisierung ("ist der Aufrufer selbst dieser Sales-Person-Datensatz?")
bzw. zur Existenzprüfung — der Cross-Aggregat-Anteil ist minimal.

### 4.1 `SalesPersonService`

Trait: `service/src/sales_person.rs:49-130`.

Signatur der Kern-Ops (verkürzt):

```rust
async fn get_all(&self, ctx, tx) -> Result<Arc<[SalesPerson]>, ServiceError>;
async fn get_all_paid(&self, ctx, tx) -> Result<Arc<[SalesPerson]>, ServiceError>;
async fn get(&self, id, ctx, tx) -> Result<SalesPerson, ServiceError>;
async fn exists(&self, id, ctx, tx) -> Result<bool, ServiceError>;
async fn create(&self, item: &SalesPerson, ctx, tx) -> Result<SalesPerson, ServiceError>;
async fn update(&self, item: &SalesPerson, ctx, tx) -> Result<SalesPerson, ServiceError>;
async fn delete(&self, id, ctx, tx) -> Result<(), ServiceError>;
async fn get_assigned_user(&self, id, ctx, tx) -> Result<Option<Arc<str>>, ServiceError>;
async fn get_all_user_assignments(&self, ctx, tx) -> Result<HashMap<Uuid, Arc<str>>, ServiceError>;
async fn set_user(&self, id, user_id: Option<Arc<str>>, ctx, tx) -> Result<(), ServiceError>;
async fn get_sales_person_for_user(&self, user_id, ctx, tx) -> Result<Option<SalesPerson>, ServiceError>;
async fn get_sales_person_current_user(&self, ctx, tx) -> Result<Option<SalesPerson>, ServiceError>;
async fn verify_user_is_sales_person(&self, id, ctx, tx) -> Result<(), ServiceError>;
```

**Auth-Gates** (`service_impl/src/sales_person.rs`):

| Methode | Erlaubte Rollen | Zeile |
| --- | --- | --- |
| `get_all` | Shiftplanner ODER Sales ODER HR (Sales/Shiftplanner sehen `is_paid=None`) | `:44-71` |
| `get_all_paid` | HR | `:82-84` |
| `get` | Shiftplanner ODER Sales ODER HR (`is_paid` maskiert außer für HR oder den verknüpften User selbst) | `:103-143` |
| `exists` | keiner (nur TX-Wrapper) | `:149-164` |
| `create` / `update` / `delete` | HR | `:173-175`, `:207-210`, `:259-262` |
| `get_assigned_user`, `get_all_user_assignments` | HR / Shiftplanner | `:288-290`, `:305-307` |
| `set_user` | HR | `:323-326` |
| `get_sales_person_for_user` | HR | `:351-353` |
| `get_sales_person_current_user` | jeder mit User-Kontext (nutzt intern `Authentication::Full`) | `:369-384` |
| `verify_user_is_sales_person` | jeder mit User-Kontext (nur eigene Zuordnung) | `:387-406` |

**TX-Verhalten.** Klassisches `use_transaction`-Muster: wenn `tx=None`,
öffnet der Service selbst eine TX via `TransactionDao::use_transaction`
und committet vor dem Return. Interne Nested-Calls (`get_assigned_user`
aus `get()`, `verify_user_is_sales_person` aus mehreren Services) laufen
mit `Authentication::Full` und teilen sich die äußere TX — vgl.
`docs/domain/edge-cases.md` §6.1 (Full-Bypass).

**Dependencies.** `SalesPersonDao`, `PermissionService`, `ClockService`,
`UuidService`, `TransactionDao` — reines Basic-Service-Set.

### 4.2 `EmployeeWorkDetailsService`

Trait: `service/src/employee_work_details.rs:234-284`.

```rust
async fn all(&self, ctx, tx) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError>;
async fn find_by_sales_person_id(&self, sp_id, ctx, tx) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError>;
async fn find_for_week(&self, sp_id, calendar_week, year, ctx, tx) -> Result<EmployeeWorkDetails, ServiceError>;
async fn all_for_week(&self, calendar_week, year, ctx, tx) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError>;
async fn create(&self, entity, ctx, tx) -> Result<EmployeeWorkDetails, ServiceError>;
async fn update(&self, entity, ctx, tx) -> Result<EmployeeWorkDetails, ServiceError>;
async fn delete(&self, id, ctx, tx) -> Result<EmployeeWorkDetails, ServiceError>;
```

**Auth-Gates** (`service_impl/src/employee_work_details.rs`):

| Methode | Erlaubte Rollen | Zeile |
| --- | --- | --- |
| `all` | HR | `:45-47` |
| `find_by_sales_person_id` | HR ODER Self | `:68-77` |
| `find_for_week` | HR ODER Self | `:99-108` |
| `all_for_week` | Shiftplanner sieht alle; sonst nur die Zeilen der eigenen Sales-Person | `:139-177` |
| `create` / `update` / `delete` | HR | `:190-192`, `:224-226`, `:269-271` |

**TX-Verhalten.** Standard-Pattern; `update()` liest die vorhandene Zeile,
prüft `version`-Konsistenz (Optimistic Lock, `:233-239`) und schreibt die
partiell veränderbaren Felder zurück. **Wichtig:** Update **überschreibt
den Startzeitraum NICHT** und lässt die Wochentag-Flags stehen (siehe
§2.2). Wer den Start ändern will, muss den Vertrag beenden und einen
neuen anlegen.

**Dependencies.** `EmployeeWorkDetailsDao`, `SalesPersonService`
(Auth-Helfer), `PermissionService`, `ClockService`, `UuidService`,
`TransactionDao`. Die `SalesPersonService`-Dependency macht diesen
Service technisch nicht mehr rein basic; sie ist aber ausschließlich für
`verify_user_is_sales_person`- und
`get_sales_person_current_user`-Aufrufe da, nicht für Cross-Aggregat-
Business-Logik.

### 4.3 `SalesPersonUnavailableService`

Trait: `service/src/sales_person_unavailable.rs:55-94`.

```rust
async fn get_all_for_sales_person(&self, sp_id, ctx, tx) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError>;
async fn get_by_week_for_sales_person(&self, sp_id, year, calendar_week, ctx, tx) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError>;
async fn get_by_week(&self, year, calendar_week, ctx, tx) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError>;
async fn create(&self, entity, ctx, tx) -> Result<SalesPersonUnavailable, ServiceError>;
async fn delete(&self, id, ctx, tx) -> Result<(), ServiceError>;
```

**Auth-Gates** (`service_impl/src/sales_person_unavailable.rs`):

| Methode | Erlaubte Rollen | Zeile |
| --- | --- | --- |
| `get_all_for_sales_person` | Shiftplanner ODER Self | `:41-50` |
| `get_by_week_for_sales_person` | Shiftplanner ODER Self | `:73-82` |
| `get_by_week` | Shiftplanner | `:109-111` |
| `create` | Shiftplanner ODER Self | `:132-141` |
| `delete` | Shiftplanner ODER Self (bezogen auf die Sales-Person des Eintrags) | `:203-212` |

**Business-Regel** (siehe §2.3): Doppel-Insert für `(sp_id, year, cw,
dow)` scheitert mit `EntityAlreadyExists`.

**Dependencies.** `SalesPersonUnavailableDao`, `SalesPersonService`
(Auth-Helfer), `PermissionService`, `ClockService`, `UuidService`,
`TransactionDao`.

### 4.4 `SalesPersonShiftplanService`

Trait: `service/src/sales_person_shiftplan.rs:11-44`.

```rust
async fn get_shiftplans_for_sales_person(&self, sp_id, ctx, tx) -> Result<Vec<(Uuid, String)>, ServiceError>;
async fn set_shiftplans_for_sales_person(&self, sp_id, assignments: &[(Uuid, String)], ctx, tx) -> Result<(), ServiceError>;
async fn get_bookable_sales_persons(&self, shiftplan_id, ctx, tx) -> Result<Arc<[SalesPerson]>, ServiceError>;
async fn is_eligible(&self, sp_id, shiftplan_id, ctx, tx) -> Result<bool, ServiceError>;
```

**Auth-Gates** (`service_impl/src/sales_person_shiftplan.rs`):

| Methode | Erlaubte Rollen | Zeile |
| --- | --- | --- |
| `get_shiftplans_for_sales_person` | Shiftplanner | `:40-43` |
| `set_shiftplans_for_sales_person` | Shiftplanner | `:60-62` |
| `get_bookable_sales_persons` | jeder mit Kontext (Shiftplanner sieht auch `planner_only`) | `:84-88` |
| `is_eligible` | jeder mit Kontext (Shiftplanner-Bit für `planner_only`) | `:150-154` |

**TX-Verhalten.** `set_shiftplans_for_sales_person` prüft zunächst die
Existenz der Sales Person (via `SalesPersonService::exists` mit
`Authentication::Full`) und schreibt dann die gesamte Zuweisungsliste in
einer TX (`sales_person_shiftplan.rs:63-73`). Kein Zwischenzustand
mit "halb ersetzt" (`docs/domain/edge-cases.md` §7.2 — Re-Point-Atomarität).

**Dependencies.** `SalesPersonShiftplanDao`, `SalesPersonService`,
`PermissionService`, `TransactionDao`. Damit klar in der Business-Logic-Tier,
weil er ein zweites Aggregat (`SalesPersonService::get_all`) für die
Bookable-Berechnung heranzieht.

### 4.5 `PersonService` (v2.7, Phase 61)

Trait: `service/src/person.rs`. Implementierung: `service_impl/src/person.rs`.

```rust
async fn create(&self, ctx, tx) -> Result<Person, ServiceError>;
async fn get(&self, id, ctx, tx) -> Result<Option<Person>, ServiceError>;
async fn unlinked_sales_persons(&self, ctx, tx) -> Result<Arc<[SalesPerson]>, ServiceError>;
async fn unlinked_users(&self, ctx, tx) -> Result<Arc<[Arc<str>]>, ServiceError>;
```

`create` nimmt keine Nutzdaten entgegen — `person` trägt außer der Identität
nichts. `id` und `version` kommen aus dem `UuidService`, `created` aus dem
`ClockService` (dieselbe defensive Form wie
`RebookingBatchServiceImpl::create`).

**Auth-Gates** (`service_impl/src/person.rs`):

| Methode | Erlaubte Rollen |
| --- | --- |
| `create` | HR |
| `get` | HR |
| `unlinked_sales_persons` | HR |
| `unlinked_users` | HR |

`HR_PRIVILEGE` ist in jeder Methode die erste Anweisung. Begründung: die
beiden Kandidaten-Lesepfade geben Sales-Person- und Login-Namen preis
(Threat T-61-05), und der bestehende Lesepfad `get_all_user_assignments` im
Sales-Person-Aggregat ist heute ebenfalls HR-gated. Phase 62/63 darf das mit
der Personenverwaltungs-UI auf `admin` **verschärfen** — lockern darf es das
nicht.

**Tier.** Basic. Der `gen_service_impl!`-Block bindet genau fünf
Dependencies und bewusst keinen weiteren Domain-Service: eine
Domain-Service-Dependency würde hier über `PermissionService` einen DI-Zyklus
schließen und die Gate-Reihenfolge unbestimmt machen (Threat T-61-07,
Roadmap-Constraint 5). Die Akzeptanzkriterien von Plan 61-01 greppen die
Datei genau darauf.

**Verdrahtung.** Konstruiert im *Basic*-Service-Block von
`shifty_bin/src/main.rs`, strikt vor der Business-Logic-Schicht, und über
`RestStateDef::person_service` erreichbar. In Phase 61 gibt es keine
REST-Route — der Accessor existiert, damit das `RestStateImpl`-Feld einen
Leser hat (`cargo clippy --workspace -- -D warnings` ist hartes Gate) und als
Vorarbeit für Phase 62.

### 4.6 `PersonDirectoryService` (v2.7, Phase 62)

Trait: `service/src/person_directory.rs`.
Implementierung: `service_impl/src/person_directory.rs`.

```rust
async fn list(&self, ctx, tx) -> Result<Arc<[PersonEntry]>, ServiceError>;
```

Das Aggregat hinter der einen Personen-Liste, die die bisherigen Tabs der
Benutzerverwaltung ablöst. `PersonEntry` trägt `person_id`, `kind`,
`display_name`, `username`, `sales_person_id`, `background_color`, `is_paid`,
`inactive` und `roles`.

**Fat Backend [PADM-06, D-62-03].** Anzeigename und Ausprägung werden genau
einmal berechnet — hier, im Service. Der Anzeigename ist der Verkäufername,
wo vorhanden, sonst der Benutzername; er ist nie `Option` und nie leer. Diese
Ableitung ist bewusst *kein* SQL-`COALESCE` und *kein* Frontend-Fallback: eine
zweite Quelle derselben Wahrheit ist genau der Weg, auf dem beide
auseinanderlaufen. `PersonKind` (`Linked` / `SalesPersonOnly` / `UserOnly`)
ist ein expliziter Backend-Fakt [D-62-01] und wird nie clientseitig aus leeren
`Option`s abgeleitet.

Die Sortierung ist case-insensitiv nach Anzeigename und liegt ebenfalls im
Service, nicht im SQL: SQLite vergleicht `TEXT` per Voreinstellung
case-sensitiv, eine Sortierung in der Abfrage würde also `Zacharias` vor `mia`
einordnen.

**Auth-Gates** (`service_impl/src/person_directory.rs`):

| Methode | Erlaubte Rollen |
| --- | --- |
| `list` | admin |

`check_permission("admin", …)` ist die **erste Anweisung** der Methode, vor
jedem Datenbankzugriff [D-62-08, PADM-08, Threats T-62-01 / T-62-05]. Das ist
genau die Verschärfung, die der Modulkopf von `PersonService` (§4.5)
angekündigt hat: das Verzeichnis ist die Personenverwaltung des
Administrators, und es gibt Benutzernamen und Rollenzuordnungen preis. Das
Privileg ist das String-Literal `"admin"` — dasselbe, das
`PermissionServiceImpl::get_all_users` verlangt; eine
`ADMIN_PRIVILEGE`-Konstante existiert im Repo bewusst nicht.

Bei fehlendem Privileg wird der Fehler **propagiert**. Es gibt keine Redaktion
auf leere Felder wie bei `VoluntaryStatsService` — eine redigierte Antwort
wäre von einem leeren Verzeichnis ununterscheidbar. Absicherung:
`service_impl::test::person_directory::list_ohne_admin_privileg_ist_forbidden`.
Der Permission-Mock in jener Testdatei akzeptiert nur das Literal `"admin"`,
eine HR-gegatete Implementierung fällt also in jedem Test der Datei um.

**Bewusste Verhaltensänderung — HR ohne Admin verliert die
Sales-Person-Verwaltung [PADM-08, D-62-08].** Heute sind `create` / `update` /
`delete` im Sales-Person-Aggregat auf `HR_PRIVILEGE` gegated
(`service_impl/src/sales_person.rs`), während die Benutzerverwaltung bereits
`"admin"` verlangt (`service_impl/src/permission.rs`). Die zusammengelegte
Seite kann nicht zwei Gates tragen, also trägt sie das strengere: ein
HR-Nutzer ohne Admin-Rolle verliert den Zugang zur Sales-Person-Verwaltung.
Das hat der Nutzer am **2026-09-16** ausdrücklich so entschieden, nachdem die
Konsequenz benannt worden war — siehe `.planning/REQUIREMENTS.md`
§„Bewusste Änderung gegenüber heute". Dass HR abschnittsweise Teile der
Personenverwaltung zurückbekommt, ist als PADM-F-02 (Future) vorgemerkt und
kein Defekt. Beide Seiten des Gates sind über HTTP nachgewiesen, nicht nur auf
Service-Ebene:
`shifty_bin::integration_test::person_directory::admin_bekommt_200` und
`shifty_bin::integration_test::person_directory::hr_ohne_admin_bekommt_403`.
Der Negativtest assertet zusätzlich, dass der Antwort-Body keinen der
geseedeten Namen trägt — ein `200` mit leerer oder redigierter Liste wäre damit
kein stiller Bestehensfall.

**Tier.** Business-Logic. Der `gen_service_impl!`-Block bindet genau drei
Dependencies — `PersonDirectoryDao`, `PermissionService`, `TransactionDao` —
und bewusst keinen weiteren Domain-Service. Er ist trotzdem Business-Logic und
nicht Basic: er führt die beiden Träger-Aggregate einer Person zu einer Sicht
zusammen und pflegt über sie hinweg die `PersonKind`-Invariante.
`shifty_bin/src/main.rs` konstruiert ihn im Business-Logic-Block, also nach den
Basic Services und damit strikt nach `person_service` — kein `OnceLock`, keine
Forward-Declaration. Die Tier-Regel, die dieser Fall schärft, steht in
`docs/architecture/02-service-tiers_de.md`.

**REST.** `GET /person-directory` antwortet mit `200` und `[PersonEntryTO]`,
mit `403` für Nicht-Admins und sonst mit `500`; Handler
`rest::person_directory::get_person_directory`. Der Handler trägt **keine**
zweite Rechteprüfung — die Trust Boundary ist `PersonDirectoryService::list`,
und zwei Gate-Stellen wären zwei Wahrheiten, von denen die schwächere die
Lücke ist (T-62-13). Die OpenAPI-Registrierung ist `PersonDirectoryApiDoc`,
eingehängt in den `ApiDoc`-Merge in `rest/src/lib.rs` (`CLAUDE.md` §REST API).
Der Präfix ist ein Vertrag über vier Stellen: Router-Nest, `ApiDoc`-Nest,
Frontend-URL (`shifty-dioxus/src/api.rs`) und der `[[web.proxy]]`-Eintrag in
`shifty-dioxus/Dioxus.toml`. **Der Dev-Proxy-Eintrag gehört zum Pfad**, nicht
in eine spätere Aufräumrunde — ein fehlender hat dieses Repo bereits dreimal
einen ausgelieferten 404 im Dev-Server gekostet (T-62-14). Details in §5.4.

**Abgrenzung.** Phase 62 liefert die Liste, sonst nichts. Eine gemeinsame
Detailansicht der zusammengeführten Person und die Mutationen je Ausprägung
(Nachschalten einer fehlenden Ausprägung, Kandidaten-Picker,
ausprägungs-spezifische Lösch-Bestätigung) sind Phase 63
[PADM-02/03/04/05/07]. Eine Zeile in Phase 62 verlinkt deshalb weiterhin auf
die bestehenden Detailseiten — `Route::SalesPersonDetails` für verknüpfte
Einträge und reine Verkäufer, `Route::UserDetails` für reine Benutzerkonten
[D-62-07].

**Randfälle.**

- Eine Verzeichniszeile ohne jeden Träger ist `unreachable!()` — das SQL
  schließt diesen Fall bereits aus (62-01). Ein Panic hier ist darum ein
  Datenintegritäts-Alarm über eine verletzte
  `person`/`sales_person`/`user`-Invariante [D-61-01, D-61-05], akzeptiert als
  Threat T-62-10.
- Soft-gelöschte Verkäufer erscheinen nicht, inaktive schon, mit
  `inactive == Some(true)` [D-62-04].
- Rollen kommen aus einer einzigen Bulk-Query und werden im Service nach
  Benutzernamen gruppiert [D-62-06]. Eine Person ohne Benutzerkonto bekommt
  eine leere Rollenliste, nie `None`.

**Transport.** `impl From<&PersonEntry> for PersonEntryTO` und
`impl From<&PersonKind> for PersonKindTO` liegen in `rest-types/src/lib.rs`
hinter `#[cfg(feature = "service-impl")]`. Das Mapping ist rein feldweise; es
rechnet nichts nach, insbesondere nicht den Anzeigenamen.

## 5. REST-Endpoints

Alle Handler leben in `rest/src/sales_person.rs`,
`rest/src/sales_person_shiftplan.rs`, `rest/src/employee_work_details.rs`
und werden in `rest/src/lib.rs:640-682` gemountet.

### 5.1 Sales Person (`/sales-person`)

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/sales-person` | Liste aller Sales Persons | — | `[SalesPersonTO]` | 401/500 |
| `GET` | `/sales-person/{id}` | Einzelne Sales Person | — | `SalesPersonTO` | 404/500 |
| `POST` | `/sales-person` | Anlegen | `SalesPersonTO` (`id`, `version` leer) | `SalesPersonTO` | 403/422/500 |
| `PUT` | `/sales-person/{id}` | Update | `SalesPersonTO` | `SalesPersonTO` | 400 (ID mismatch), 404, 409, 403, 422, 500 |
| `DELETE` | `/sales-person/{id}` | Soft-Delete | — | 204 | 404/500 |
| `GET` | `/sales-person/{id}/user` | Verknüpften Login-User lesen | — | `Option<String>` | 404/500 |
| `POST` | `/sales-person/{id}/user` | Login-User zuweisen (ersetzt) | `String` (username) | 204 | 404/409/500 |
| `DELETE` | `/sales-person/{id}/user` | Login-User entfernen | — | 204 | 404/409/500 |
| `GET` | `/sales-person/by-user/{username}` | Sales Person zu Username | — | `SalesPersonTO` | 404/500 |
| `GET` | `/sales-person/current` | Sales Person für aktuellen Login-User | — | `Option<SalesPersonTO>` | 500 |
| `GET` | `/sales-person/{id}/ical` | iCal-Export der nächsten Wochen (nutzt `BlockService`) | — | `text/calendar` | 404/500 |
| `GET` | `/sales-person/{id}/unavailable` | Alle Wochentag-Sperren (mit optionalen Query-Params `year`, `calendar_week`) | — | `[SalesPersonUnavailableTO]` | 500 |
| `POST` | `/sales-person/unavailable` | Wochentag-Sperre anlegen | `SalesPersonUnavailableTO` | `SalesPersonUnavailableTO` | 404/409/500 |
| `DELETE` | `/sales-person/unavailable/{id}` | Wochentag-Sperre löschen | — | 204 | 404/500 |

Definitionen der DTOs in `rest-types/src/lib.rs`:

- `SalesPersonTO` — `rest-types/src/lib.rs:197-212`. Feld `version` wird
  als `"$version"` serialisiert (kollisionsfreier Snapshot-Marker).
  `is_paid: Option<bool>` → `None` bedeutet **maskiert**, nicht
  "unbekannt".
- `SalesPersonUnavailableTO` — `rest-types/src/lib.rs:890-905`.
- `DayOfWeekTO` — Enum-DTO für den Wochentag, konvertiert in
  `shifty_utils::DayOfWeek`.

### 5.2 Sales Person Shiftplan (`/sales-person-shiftplan`)

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/sales-person-shiftplan/{id}/shiftplans` | Zuweisungen einer Sales Person | — | `Vec<ShiftplanAssignmentTO>` | 403/500 |
| `PUT` | `/sales-person-shiftplan/{id}/shiftplans` | Zuweisungen komplett setzen (Replace) | `Vec<ShiftplanAssignmentTO>` | 200 | 403/404/500 |
| `GET` | `/sales-person-shiftplan/by-shiftplan/{shiftplan_id}` | Bebuchbare Sales Persons für einen Plan | — | `Vec<SalesPersonTO>` | 500 |

- `ShiftplanAssignmentTO` — `rest-types/src/lib.rs:1635-1644`. `permission_level`
  defaultet serverseitig auf `"available"`, wenn nicht gesetzt.

### 5.3 Employee Work Details (`/employee-work-details` und Legacy `/working-hours`)

Beide Pfade sind unter dieselbe Router-Factory gemountet
(`rest/src/lib.rs:651-655`), d.h. Alt-Clients auf `/working-hours` und
neue Clients auf `/employee-work-details` funktionieren identisch. Die
Handler in `rest/src/employee_work_details.rs` haben **keine
`#[utoipa::path]`-Annotationen** — sie erscheinen nicht in der OpenAPI-
Doku. **[Zu prüfen]** ob das absichtlich ist (kein `EmployeeWorkDetailsApiDoc`
existiert).

| Methode | Pfad | Beschreibung | DTO In | DTO Out |
| --- | --- | --- | --- | --- |
| `POST` | `.../` | Neuen Vertrag anlegen | `EmployeeWorkDetailsTO` | `EmployeeWorkDetailsTO` |
| `PUT` | `.../{id}` | Vertrag ändern (partiell, siehe §2.2) | `EmployeeWorkDetailsTO` | `EmployeeWorkDetailsTO` |
| `DELETE` | `.../{id}` | Soft-Delete | — | 200 |
| `GET` | `.../for-week/{sales_person_id}/{year}/{calendar_week}` | Aktiven Vertrag für eine Woche | — | `EmployeeWorkDetailsTO` |
| `GET` | `.../for-sales-person/{sales_person_id}` | Alle Verträge einer Person | — | `[EmployeeWorkDetailsTO]` |

- `EmployeeWorkDetailsTO` — `rest-types/src/lib.rs:681-721`. Die
  abgeleiteten Convenience-Felder `days_per_week`, `hours_per_day`,
  `hours_per_holiday` werden vom Backend berechnet und mitgeschickt.

**Kein `EmployeeWorkDetailsApiDoc`** ist im `ApiDoc`-Merge in
`rest/src/lib.rs:568-598` registriert — nur `SalesPersonApiDoc` und
`SalesPersonShiftplanApiDoc`. **[Zu prüfen]** ob das ein bekannter Gap ist,
den man beim nächsten OpenAPI-Sweep schließen möchte.

### 5.4 Personen-Verzeichnis (`/person-directory`, v2.7, Phase 62)

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/person-directory` | Das gesamte Personen-Verzeichnis — verknüpfte Personen, reine Benutzerkonten und reine Verkäufer nebeneinander, case-insensitiv nach Anzeigename sortiert | — | `[PersonEntryTO]` | 403 (kein Admin) / 500 |

Handler in `rest/src/person_directory.rs`, gemountet in `rest/src/lib.rs` als
`.nest("/person-directory", person_directory::generate_route())`. Anders als
die Employee-Work-Details-Handler weiter oben ist dieser Endpoint in der
OpenAPI-Doku **registriert**: `PersonDirectoryApiDoc` ist Teil des
`ApiDoc`-Merge, und der Handler trägt eine `#[utoipa::path]`-Annotation. Das
Admin-Gate liegt im Service (§4.6), nicht im Handler.

- `PersonEntryTO` / `PersonKindTO` — `rest-types/src/lib.rs`, beide mit
  `ToSchema`. `display_name` ist ein einfacher `String` und nie leer; `kind`
  ist ein expliziter Enum-Wert und nichts, was der Client aus leeren
  `Option`s ableitet [D-62-01, PADM-06].

## 6. Frontend-Integration

### Pages (`shifty-dioxus/src/page/`)

- **`employees.rs`** — Master/Detail-Shell für den *Mitarbeiter*-Menüpunkt.
  Nur Layout + Auswahl-Placeholder; keine eigene API-Logik.
- **`employee_details.rs`** — HR-Sicht auf einen Mitarbeiter mit
  Bilanz, Verträgen, Extra Hours, Absence-Navigation. Ruft
  `EmployeeAction::LoadEmployeeDataUntilNow`,
  `EmployeeWorkDetailsAction::{NewWorkingHours, Load, Save, Update}` und
  `EmployeeAction::{Refresh, DeleteExtraHours}`. Öffnet `ContractModal`
  und `ExtraHoursModal`. Die master/detail-Route lässt die Komponente
  gemountet über Employee-Wechsel — Signal-Spiegelung von `employee_id`
  ist Pflicht (Regression-Test in `employee_details.rs:242-314` erinnert
  daran).
- **`my_employee_details.rs`** — Self-Service-Sicht für Sales-User auf
  die **eigenen** Daten. Nutzt `EmployeeAction::LoadCurrentEmployeeDataUntilNow`
  und rendert `ContractModal` in `ReadOnly`-Mode.
- **`sales_person_details.rs`** — HR-Formular zum Anlegen/Bearbeiten der
  Sales-Person-Stammdaten, inklusive User-Verknüpfung und Shiftplan-
  Zuweisungen. Nutzt `UserManagementAction::{LoadSalesPerson,
  LoadShiftplanAssignments, LoadShiftplanCatalog, UpdateSalesPerson,
  UpdateSalesPersonUser, RemoveSalesPersonUser, UpdateShiftplanAssignments,
  SaveSalesPersonAndNavigate}`.

### Services & State

- `shifty-dioxus/src/service/employee.rs` — Employee-Daten-Loader
  (`EmployeeAction`, `EMPLOYEE_STORE`).
- `shifty-dioxus/src/service/employee_work_details.rs` — Contract-
  Formular-State (`EmployeeWorkDetailsAction`).
- `shifty-dioxus/src/service/user_management.rs` — Sales-Person-
  Stammdaten-Formular + Shiftplan-Zuweisung.
- `shifty-dioxus/src/state/employee.rs` — `EMPLOYEE_STORE`-Struktur,
  `ExtraHours`-Bags.
- `shifty-dioxus/src/state/shiftplan.rs` — `SalesPerson`,
  `ShiftplanAssignment` (Frontend-Kopie der DTOs).

### Proxy-Konfig (`shifty-dioxus/Dioxus.toml`)

Vorhandene Einträge für dieses Cluster:

- `http://localhost:3000/sales-person` (Zeile 66)
- `http://localhost:3000/working-hours` (Zeile 74) — **kein Eintrag für
  `/employee-work-details`**. Solange nur der Legacy-Pfad
  angesprochen wird, ist das ok; wer neuen FE-Code gegen den kanonischen
  Pfad schreibt, muss den Proxy nachziehen (Randfall §11 in
  `docs/domain/edge-cases.md`).
- `http://localhost:3000/sales-person-shiftplan` (Zeile 96)
- `http://localhost:3000/person-directory` (Zeile 118) — im **selben Commit**
  wie die Route selbst ergänzt (Phase 62, T-62-14). Ohne ihn beantwortet der
  Dev-Server den Verzeichnis-Aufruf mit 404, während die Produktion
  funktioniert; dieser Fehlermodus wurde in diesem Repo bereits dreimal
  ausgeliefert.

### i18n

Sales-Person-Formular-Keys u.a. `SalesPersonDetails`, `BasicInformation`,
`ShiftplanColor`, `ColorPreview`, `Settings`, `ThisPersonReceivesPayment`,
`ThisPersonIsInactive`, `UserAccount`, `ConnectUserAccount`,
`ShiftplanAssignments`, `ShiftplanAssignmentsInfo`,
`PermissionLevelAvailable`, `PermissionLevelPlannerOnly`,
`SaveChanges`, `Cancel`, `LoadingSalesPersonDetails` — siehe
`shifty-dioxus/src/page/sales_person_details.rs`. Alle Keys müssen in
En/De/Cs vorhanden sein (siehe `docs/domain/edge-cases.md` §13).

## 7. Randfälle

Für die zentrale Referenz siehe
[`../domain/edge-cases.md`](../domain/edge-cases.md). Feature-spezifische
Kanten dieses Clusters:

- **`from`-Wechsel im laufenden Vertrag geht nicht.** `update()` schreibt
  nur die `to_*`-Felder + Kern-Parameter zurück; wer einen Startzeitpunkt
  verschieben will, muss den bestehenden Vertrag beenden und einen neuen
  anlegen. Siehe §2.2. Vgl. auch §1.2 in edge-cases (Contract-Wechsel).
- **Contract-Lücke zwischen zwei Verträgen.** `find_for_week` liefert
  `EntityNotFoundGeneric`, wenn die Woche zwischen zwei Verträgen liegt
  (`service_impl/src/employee_work_details.rs:119-126`). Reporting-Konsumenten
  müssen diesen Fall behandeln — siehe edge-cases §1.2.
- **Vertrags-Ende-Proration bei Nicht-31.12.-Enden.**
  `vacation_days_for_year` subtrahiert am Jahresende
  `vac * (1 - ordinal(to) / days_in_year)`; ein 31.12.-Ende subtrahiert 0.
  Regression-Guards ab `service/src/employee_work_details.rs:286-408`.
- **Sales-Person-`is_paid`-Maskierung.** Wer nicht HR ist und **nicht**
  selbst der verknüpfte User ist, bekommt `is_paid = None` — nicht
  `false`. Falsche Interpretation im Frontend als "unbezahlt" wäre ein
  Info-Leak-Downstream. Siehe §2.1 und `sales_person.rs:120-143`.
- **Delete = Soft-Delete.** Alle Delete-Endpoints setzen nur `deleted`.
  Nachfolgende `create()`-Aufrufe können **denselben Namen** wieder
  anlegen (kein `UNIQUE(name)`), aber das FK-Sub-Graph (Bookings, Extra
  Hours, Absences) hängt weiter am ursprünglichen Datensatz. Siehe
  edge-cases §8.
- **User-Zuweisungs-Race.** Seit v2.7 Phase 61 trägt
  `user_person_id_unique_idx` die 1:1-Garantie. Ein Ersetzen läuft
  `discard_assigned_user` (Split) + `assign_to_user` (Merge) in derselben TX
  [P-61-B]. Zwei parallele HR-Requests, die denselben Login-User zwei
  verschiedenen Sales Persons zuordnen, erzeugen keinen generischen DAO-Fehler
  mehr: `assign_to_user` erkennt echte Mehrdeutigkeit **vor** jedem
  Schreibvorgang und liefert `DaoError::EntityAlreadyExists`, das `set_user` in
  `ServiceError::EntityAlreadyExists` → **HTTP 409** übersetzt [D-61-04]. Der
  Unique-Index bleibt die zweite Verteidigungslinie für den Race-Verlierer.
- **`sales_person.background_color`.** Freies Hex-Feld — kein Format-
  Check im Backend. Ungültige Werte (`"rot"`, leerer String) landen roh
  in der DB und rendern im Frontend als Style-`background-color:` — das
  Style-Attribut ignoriert Garbage still. **[Zu prüfen]** ob eine
  Validation lohnt.
- **Wochentag-Sperren beim Jahreswechsel.** `sales_person_unavailable`
  speichert `(year, calendar_week)`. KW 1 kann laut ISO 8601 im Dezember
  des Vorjahres liegen — Consumer müssen die Semantik von
  `(iso_week_year, iso_week)` konsistent bedienen. Siehe edge-cases §4.
- **Shiftplan-Zuweisung: "leer = überall".** Das ist Absicht (Onboarding-
  Convenience), führt aber dazu, dass eine frisch angelegte Sales Person
  in **jedem** Shiftplan als bebuchbar auftaucht, bis sie eine erste
  Zuweisung bekommt. Wer das restriktiver möchte, muss beim Anlegen
  bewusst eine Nicht-Zuweisung setzen.
- **`planner_only`-Semantik.** Sales-User können sich selbst **nicht**
  in solche Pläne buchen — die Person taucht in `get_bookable_sales_persons`
  für Non-Shiftplanner erst gar nicht auf. Der Buchungs-Service (F03)
  MUSS auf `is_eligible` prüfen, damit direkte API-Aufrufe kein Loch
  öffnen.
- **`get_all` liefert auch soft-deletete Personen.** Die DAO-Methode
  `all` in `dao_impl_sqlite/src/sales_person.rs` filtert **[Zu prüfen]**
  auf `deleted IS NULL`; im Frontend zeigt die User-Management-Liste
  auch inaktive, aber nicht gelöschte Personen. Wer eine
  Sales-Person-Liste "wirklich alle je angelegten" braucht, muss ein
  Sonder-Reading einbauen — es gibt derzeit keinen solchen Endpoint.
- **Eine soft-gelöschte Sales Person behält ihre Person-Identität dauerhaft.**
  Der Unique-Index auf `sales_person.person_id` ist vollständig, nicht
  partiell — die Löschung gibt die Identität **nicht** frei [D-60-01]. Folge:
  ein Mitarbeiter, der geht und zurückkommt, erscheint als *zweite* Person,
  weil die neue Sales-Person-Zeile eine neue Identität bekommt. Das ist
  gewollt — gäbe die Löschung die Identität frei, könnten zwei verschiedene
  Menschen über die Zeit dieselbe belegen und write-once-Snapshots in
  `billing_period_sales_person` würden auf die falsche Person auflösen. Ein
  Zusammenführen zweier Personen ist kein v2.7-Scope.
- **Eine Sales Person ohne Login ist eine dauerhaft unterstützte Ausprägung.**
  Freiwillige ohne Benutzerkonto sind genauso gültig wie HR-/Admin-Benutzer
  ohne Sales Person (siehe [F12](./F12-auth-session_de.md)). `person_id` bleibt
  bewusst nullable, und es wird **nirgends** automatisch ein Benutzer zu einer
  Sales Person angelegt [D-60-03].
- **Die Vergabe bei Neuanlage** (v2.7, Phase 61 [D-61-01]):
  `SalesPersonDaoImpl::create` legt seitdem **zuerst** eine eigene
  `person`-Zeile an und schreibt deren ID in `sales_person.person_id`. Beide
  Statements laufen auf der vom Aufrufer übergebenen Transaktion — die
  Identitäts-Vergabe ist damit atomar mit der Sales-Person-Anlage, ein
  Rollback nimmt beide Zeilen mit. Die 16-Byte-ID entsteht in Rust
  (`Uuid::new_v4()`); ein Readback der SQLite-Zeilennummer wäre falsch, weil
  `person.id` ein `blob(16) PRIMARY KEY` und kein Rowid-Alias ist. `created`
  entsteht per `strftime('%Y-%m-%dT%H:%M:%S.000000000Z','now')` — identisches
  Format wie im Phase-60-Backfill, damit `Iso8601::DATE_TIME` es parst; die
  DAO-Ebene braucht so keinen `ClockService`. Die Trait-Signatur von
  `SalesPersonDao::create` ist unverändert geblieben. Nachweise:
  `service_impl::test::person_creation` (N-1 bis N-4).

## 8. Tests

Unit-Tests liegen unter `service_impl/src/test/`:

- **`sales_person.rs`** (26 Tests, `service_impl/src/test/sales_person.rs`).
  Abgedeckt sind alle Auth-Kombinationen von `get_all`/`get`
  (Shiftplanner / Sales / HR / no permission), Create-Validation
  (`IdSetOnCreate`, `VersionSetOnCreate`), Update-Guards (No-Permission,
  Not-Found, Conflict, `deleted` nicht änderbar, Name-/Farbe-Update-Roundtrip),
  Soft-Delete-Erfolg und -Fehler sowie `exists`.
- **`employee_work_details.rs`** (nur 2 Tests, `service_impl/src/test/employee_work_details.rs`).
  Deckt gezielt die späten Feld-Erweiterungen ab: `update` propagiert
  `committed_voluntary` und `cap_planned_hours_to_expected` an den DAO —
  das sind Regression-Guards für die v2.x-Migrations. **Bekannte Lücke:**
  Es gibt **keine** Unit-Tests für `create()`-Validation, `find_for_week`
  über Vertragsgrenzen, `all_for_week`-Filterung nach Sales-User, oder
  Auth-Deny-Pfade. Die Vertrags-Proration hat dagegen eine dichte
  Regression-Suite direkt im Service-Trait
  (`service/src/employee_work_details.rs:286-408`, 5 Tests, Phase 28).
- **`sales_person_unavailable.rs`** (16 Tests). `get_all_*`, `get_by_week*`
  und Create/Delete jeweils in den Varianten Shiftplanner / Self /
  no-permission plus Create-Validation (`IdSetOnCreate`,
  `VersionSetOnCreate`, `AlreadyExists`) und Delete-Not-Found.
- **`sales_person_shiftplan.rs`** (21 Tests). Deckt `get`/`set`
  (inkl. Clear = Replace mit leerer Liste), Bookable-Berechnung
  ("keine Zuweisungen = überall", inaktive Personen filtern, mixed
  `available`/`planner_only`, Include/Exclude `planner_only` je nach
  Aufrufer-Rolle), `is_eligible` in allen Kombinationen sowie
  Auth-Deny-Pfade.

Integration-Tests (In-Memory-SQLite) laufen im gleichen Verzeichnis; sie
teilen sich `service_impl/src/test/helpers` (Mock-Auth, Time-Fake).

**Bekannte Lücken:**

- Employee-Work-Details `create`/`find_for_week`/`all_for_week` /
  Auth-Deny-Pfade (siehe oben).
- `SalesPersonService::get_all_paid` — kein dedizierter Test.
- REST-Handler haben derzeit keine `oneshot`-Tower-Tests für dieses
  Cluster (im Gegensatz zu `extra_hours`/`feature_flag`, wo das üblich ist).

## 9. Historie & Kontext

- **v0.1 (Mai 2024) — Grundgerüst.** `sales_person` plus die
  1:1-Verknüpfungstabelle (Mig `20240506114107`; die Verknüpfungstabelle wurde
  in v2.7 Phase 61 aufgelöst). Ur-Tabelle `working_hours` folgt im Juni
  (`20240618125847`), Kennfarbe (`20240618035521`) und Bezahlt-Flag im
  selben Sprint. `shiftplanner`-Rolle kommt im Juni (`20240614075633`).
- **v0.2 (Juli 2024) — Verfügbarkeits-Sperren.**
  `sales_person_unavailable` (`20240731043118`) — der erste
  Wochentags-genaue Kalender-Baustein.
- **v0.5 (Okt 2024) — Vertrags-Umbau.** `working_hours` →
  `employee_work_details` (`20241023062246`), Wochentags-Flags,
  `from_/to_day_of_week`, `vacation_days`. Damit lief das Balance-Feature
  auf eine tragfähige Datenbasis um.
- **v1.5 (Okt 2025) — Dynamische Verträge.** `is_dynamic`
  (`20251029192107`) — für Angestellte, deren Soll = Ist gerechnet wird.
- **v2.0-Serie (März–Juni 2026) — Shiftplan-Zuweisung + Fein-Tuning.**
  Neue Tabelle `sales_person_shiftplan` (`20260331000000`), Erweiterung
  um `permission_level` (`20260402000000`), Vertrags-Deckelungs-Flag
  `cap_planned_hours_to_expected` (`20260426120000`) und feste
  Freiwilligen-Zusage `committed_voluntary` (`20260623120000`,
  v2.4).
- **Phase 28 (2026, D-28-04) — Urlaubs-Proration-Fix.** Off-by-one bei
  1.1.-Startdatum korrigiert; Regression-Guards fest im Trait-Modul
  (`service/src/employee_work_details.rs:286-408`). Details im Kontext-
  Read `.planning/phases/28-*/CONTEXT.md`. **[Zu prüfen]** genauer Pfad,
  falls Phase inzwischen archiviert.
- **Phase 51 (Toggle Full-Context-Bypass, 2026).** Nicht direkt an diesem
  Cluster, aber relevant: `SalesPersonService` konsumiert intern
  `Authentication::Full` in `get_sales_person_current_user` und
  `verify_user_is_sales_person`. Nachdem der Toggle-Service den
  Full-Bypass hatte, mussten Aggregat-Aufrufer sichergestellt weiter mit
  Full lesen können.
- **v2.7 (Sept. 2026) — Person-Identität.** *Phase 60* führte die
  Identitäts-Elterntabelle `person` sowie die nullable Spalten
  `sales_person.person_id` und `user.person_id` ein, jeweils eindeutig gemacht
  durch einen **vollständigen** (nicht partiellen) Index, und zog jede
  bestehende Zeile per Backfill nach — soft-gelöschte Sales Persons
  eingeschlossen (`20260916000000_create-person-backfill.sql`; nur Schema +
  Backfill, keine Verhaltensänderung). *Phase 61* löste die
  1:1-Verknüpfungstabelle auf
  (`20260917000000_drop-sales-person-user.sql`, nach einem Nachzieh-Backfill
  und einer Selbst-Assertion), stellte alle lesenden und schreibenden
  Verknüpfungs-Pfade auf `person_id` um, formte Verknüpfen zu einem **Merge**
  und Lösen zu einem **Split** mit HTTP 409 bei echter Mehrdeutigkeit und
  ergänzte den Basic-Tier-`PersonService` (§4.5) als Grundlage für die
  Personenverwaltung in Phase 62/63. Die Selbst-Zugriffs-Prüfung entscheidet
  seither im SQL-Join — siehe
  [`../architecture/04-auth_de.md`](../architecture/04-auth_de.md).

Zusatz-Doku für Endanwender:

- `docs/employee-management.md` (EN, primär)
- `docs/employee-management_de.md` (DE, mit Screenshots) — enthält
  fachlich detaillierte Formel-Erklärungen für Bilanz, Deckelung und
  Ehrenamt, die hier nicht dupliziert sind.

---

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
