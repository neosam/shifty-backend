# Feature: Employee Management

> **Kurzform:** Anlage und Pflege der Mitarbeiter-Stammdaten (Sales Persons),
> ihrer Arbeitsverträge (Employee Work Details), ihrer wiederkehrenden
> Verfügbarkeits-Sperren (Unavailable) und ihrer Zuordnung zu Schichtplänen.

**Cluster-ID:** F01
**Status:** produktiv
**Erstmalig eingeführt:** v0.1 (Basistabellen 2024-05), sukzessiv erweitert bis v2.4
**Zuständige Crates:**
`service::{sales_person, sales_person_shiftplan, sales_person_unavailable, employee_work_details}`,
`service_impl::{sales_person, sales_person_shiftplan, sales_person_unavailable, employee_work_details}`,
`dao::{sales_person, sales_person_shiftplan, sales_person_unavailable, employee_work_details}`,
`dao_impl_sqlite::{sales_person, sales_person_shiftplan, sales_person_unavailable, employee_work_details}`,
`rest::{sales_person, sales_person_shiftplan, employee_work_details}`

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
- **User-Zuweisung ist 1:1.** `sales_person_user` hat
  `UNIQUE(sales_person_id)` **und** `UNIQUE(user_id)`
  (`migrations/sqlite/20240506114107_add_sales_person.sql:20-21`). Das
  Ersetzen läuft in einer TX via `discard_assigned_user` + `assign_to_user`
  (`sales_person.rs:316-342`), also atomar für den Umzug einer Zuordnung.

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
| `sales_person` | Stammdaten des Angestellten | `id`, `name`, `background_color`, `is_paid`, `inactive`, `deleted`, `update_version` |
| `sales_person_user` | 1:1 Verknüpfung Sales-Person ↔ Login-User | `sales_person_id`, `user_id` (beide `UNIQUE`), FK auf `sales_person(id)` und `user(name)` |
| `employee_work_details` (früher `working_hours`) | Arbeitsvertrag mit Zeitraum, Wochenstunden, Urlaub | `id`, `sales_person_id`, `expected_hours`, `from_/to_year/calendar_week/day_of_week`, `workdays_per_week`, `monday`..`sunday`, `vacation_days`, `is_dynamic`, `cap_planned_hours_to_expected`, `committed_voluntary`, `created`, `deleted`, `update_version` |
| `sales_person_unavailable` | Wiederkehrende Verfügbarkeits-Sperren auf Wochentagsebene | `id`, `sales_person_id`, `year`, `calendar_week`, `day_of_week`, `created`, `deleted` |
| `sales_person_shiftplan` | M:N Sales-Person ↔ Shiftplan mit Berechtigungslevel | `sales_person_id`, `shiftplan_id`, `permission_level` ∈ {`available`,`planner_only`} |

### Migrations

Chronologische Historie (nur die für dieses Cluster relevanten Files aus
`migrations/sqlite/`):

- `20240506114107_add_sales_person.sql` — Basistabellen `sales_person` +
  `sales_person_user` (mit `UNIQUE`-Constraints für die 1:1-Kopplung).
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

### Beziehungen

Alle Tabellen des Clusters hängen per Foreign Key an `sales_person(id)`
(Ausnahme: `sales_person_shiftplan` hängt zusätzlich an `shiftplan(id)`).
SQLite prüft FKs nur, wenn `PRAGMA foreign_keys=ON` aktiv ist — die
Soft-Delete-Semantik (`WHERE deleted IS NULL`) macht die FK-Prüfung ohnehin
nur bedingt aussagekräftig (siehe `docs/domain/edge-cases.md` §8).

```
user ────────────< sales_person_user >──── sales_person ──┬──< employee_work_details
                                                          ├──< sales_person_unavailable
                                                          └──< sales_person_shiftplan >── shiftplan
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
| `POST` | `/sales-person/{id}/user` | Login-User zuweisen (ersetzt) | `String` (username) | 204 | 404/500 |
| `DELETE` | `/sales-person/{id}/user` | Login-User entfernen | — | 204 | 404/500 |
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
- **User-Zuweisungs-Race.** `sales_person_user` hat
  `UNIQUE(user_id)`; ein Ersetzen läuft `discard_assigned_user` +
  `assign_to_user` in derselben TX. Zwei parallele HR-Requests, die
  denselben Login-User zwei verschiedenen Sales Persons zuordnen, führen
  bei zweitem Commit zu einem SQLite-Unique-Constraint-Fehler — die
  Fehlermeldung ist derzeit **[Zu prüfen]** ein generischer DAO-Fehler,
  nicht Business-lesbar.
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

- **v0.1 (Mai 2024) — Grundgerüst.** `sales_person` + `sales_person_user`
  (Mig `20240506114107`). Ur-Tabelle `working_hours` folgt im Juni
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

Zusatz-Doku für Endanwender:

- `docs/employee-management.md` (EN, primär)
- `docs/employee-management_de.md` (DE, mit Screenshots) — enthält
  fachlich detaillierte Formel-Erklärungen für Bilanz, Deckelung und
  Ehrenamt, die hier nicht dupliziert sind.

---

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
