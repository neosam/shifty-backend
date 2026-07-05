# Feature: Extra Hours — Legacy Zeit-Erfassung & Custom-Kategorien

> **Kurzform:** Single-Day-Zeitzeilen für Überstunden, Urlaub, Krankheit,
> Feiertag, Unverfügbarkeit, unbezahlte Freistellung, Freiwilligenarbeit und
> beliebige betriebs-definierte Zusatzkategorien — das ursprüngliche
> Zeit-Erfassungs-Aggregat, das für Absenz-Kategorien (Vacation/SickLeave/
> UnpaidLeave) mit v1.0 vom neuen Range-basierten Absence-System (F05)
> **koexistiert** wird.

**Cluster-ID:** F04
**Status:** produktiv (Legacy für Absence-Kategorien, weiterhin führend für
Overtime / Volunteer / Custom)
**Erstmalig eingeführt:** initiale HR-Iteration (Migration `20240618125847_paid-sales-persons.sql`)
**Zuständige Crates:**
- `service::extra_hours`, `service::custom_extra_hours`
- `service_impl::extra_hours`, `service_impl::custom_extra_hours`
- `dao::extra_hours`, `dao::custom_extra_hours` (plus die konkrete
  SQLite-Implementierung in `dao_impl_sqlite`)
- `rest::extra_hours`, `rest::custom_extra_hours`
- `rest_types::{ExtraHoursTO, ExtraHoursCategoryTO, CustomExtraHoursTO,
  ConvertExtraHoursRequestTO}`
- Frontend: `shifty-dioxus/src/page/custom_extra_hours_management.rs`

---

## 1. Was ist das? (Fachlich)

Extra Hours sind **Einzeltag-Zeitzeilen**, mit denen HR (oder ein Mitarbeitender
für sich selbst) Zeiten außerhalb des normal geplanten Schichtplans festhält:

- Überstunden (`ExtraWork`)
- Urlaub (`Vacation`) — Legacy: einzelne Tage; ab v1.0 üblicherweise als
  Absence-Range (F05)
- Krankheit (`SickLeave`) — dito
- Feiertag (`Holiday`)
- Unverfügbarkeit (`Unavailable`) — reine Verfügbarkeits-Markierung, ohne
  Stunden-Auswirkung auf die Bilanz
- Unbezahlte Freistellung (`UnpaidLeave`) — dito Legacy → Absence-Range;
  senkt die Erwartung
- Freiwilligenarbeit (`VolunteerWork`) — dokumentiert, aber nicht bilanzwirksam
- Custom-Kategorien (`CustomExtraHours(id)`) — vom Betrieb selbst definiert

**Custom Extra Hours** sind ein zweiter, orthogonaler Katalog: HR kann beliebig
viele eigene Buchungskategorien anlegen (z.B. "Fortbildung", "Betriebsrat",
"Notdienst-Bereitschaft"), sie einer Menge von Sales Persons zuweisen und pro
Kategorie entscheiden, ob sie die Stunden-Bilanz beeinflusst (`modifies_balance`).
Die eigentlichen Zeitzeilen liegen weiterhin in `extra_hours`, referenzieren
über `custom_extra_hours_id` aber eine Zeile aus `custom_extra_hours`.

**Beispiel-Workflow aus User-Sicht:**

1. HR öffnet die Employee-Detail-Seite und trägt "05.05.2025 — 4h Überstunden"
   ein (`ExtraWork`) — landet direkt im Report als positive Balance.
2. Ein Mitarbeitender bucht selbst "12.05.2025 — 8h Urlaub" (`Vacation`) —
   Legacy-Pfad; ab v1.0 legt das UI stattdessen einen `AbsencePeriod` an.
3. HR legt in "Custom Extra Hours Management" (siehe Frontend) eine Kategorie
   "Fortbildung" an, `modifies_balance=true`, zugewiesen an alle Verkäufer*innen.
   Ab jetzt ist "Fortbildung" als Kategorie beim Anlegen einer Extra-Hours-Zeile
   verfügbar und zählt wie Arbeitsstunden.

## 2. Fachliche Regeln

- **Kategorien-Enum** ist gepinnt in `service::extra_hours::ExtraHoursCategory`
  (`service/src/extra_hours.rs:41-50`) und im DAO-Layer als
  `ExtraHoursCategoryEntity` (`dao/src/extra_hours.rs:9-18`). Neue Fixed-Enum-
  Werte sind ein Breaking-Change; für neue Kategorien ist der Custom-Pfad
  vorgesehen.
- **`as_report_type()`** klassifiziert jede Kategorie in einen ReportType
  (`service/src/extra_hours.rs:51-73`):
  - `ExtraWork` → `WorkingHours` (zählt als gearbeitet)
  - `Vacation`, `SickLeave`, `Holiday`, `UnpaidLeave` → `AbsenceHours`
    (zählen als abwesend, senken erwartete Stunden entsprechend Kategorie-
    Semantik)
  - `Unavailable` → `None` (keine Bilanz-Wirkung, nur Verfügbarkeit)
  - `VolunteerWork` → `Documented` (weder Balance noch Erwartung)
  - `CustomExtraHours(…)` → `WorkingHours` falls `modifies_balance=true`,
    sonst `None` (fällt bei nicht geladenem `LazyLoad` auf `None` zurück)
- **`availability()`** entscheidet, ob diese Zeile die Sales Person am Tag als
  verfügbar oder blockiert markiert (`service/src/extra_hours.rs:75-96`).
- **`UnpaidLeave` (Unbezahlte Freistellung) — Sonderrolle:**
  - `as_report_type() == AbsenceHours` und `availability() == Unavailable`
    (verifiziert via Tests in `service/src/extra_hours.rs:255-268`).
  - **Senkt die Erwartung**: Der Reporting-Pfad filtert `UnpaidLeave` explizit
    aus (`service_impl/src/reporting.rs:562`, `:974`), sodass die für diese
    Zeit erwarteten Stunden aus der Wochenerwartung herausgerechnet werden —
    ein UnpaidLeave-Tag ist damit weder gearbeitet noch geschuldet, die
    Balance bleibt neutral.
- **Custom-Kategorien-Effekt hängt an `modifies_balance`:** Nur wenn die
  Definition geladen und `modifies_balance=true` ist, zählt die Zeile in die
  Balance — sonst wird sie ignoriert (auch bei Availability). Die
  Lazy-Load-Semantik ist damit "safe by default": ungeladene Custom-Kategorien
  wirken nicht.
- **Autor- und Selbstbedienung:** HR (`HR_PRIVILEGE`) darf für jede Sales
  Person schreiben; ein Sales-Person-Account darf für sich selbst schreiben.
  Das ist ein OR-Gate (`service_impl/src/extra_hours.rs:118-127` und
  `:187-196`, `:248-257`, `:322-345`).
- **Update = soft-delete + insert:** Ein Update legt eine neue physische Zeile
  mit gleichem `logical_id` an und markiert die alte als gelöscht
  (`service_impl/src/extra_hours.rs:273-309`). Die stabile ID nach außen ist
  `logical_id`, die physische `id` ändert sich pro Version. Die Migration
  `20260428101456_add-logical-id-to-extra-hours.sql` erzwingt per Partial-Index
  genau eine aktive Zeile pro `logical_id`.
- **Version-Conflict:** Update vergleicht `request.version` gegen die aktuell
  aktive Zeile; bei Mismatch → `ServiceError::EntityConflicts`
  (`service_impl/src/extra_hours.rs:265-271`).
- **`sales_person_id` ist unveränderlich:** Ein Update, das die Sales Person
  wechselt, wird abgewiesen (`service_impl/src/extra_hours.rs:259-263` →
  `ValidationFailureItem::ModificationNotAllowed`).
- **Delete = Soft-Delete:** Setzt `deleted = NOW()` auf der aktiven Zeile
  (`service_impl/src/extra_hours.rs:315-359`).
- **Bulk-Soft-Delete für Cutover:** `soft_delete_bulk` ist ein spezieller
  Massenpfad, der ausschließlich vom Cutover-Prozess (F05 / Phase 4) benutzt
  wird. Er verlangt `CUTOVER_ADMIN_PRIVILEGE`, prüft **vor** jeder DAO-Arbeit
  und übernimmt die Transaktion vom Aufrufer (kein Commit hier —
  `service_impl/src/extra_hours.rs:372-399`). Idempotent auf DAO-Ebene: bereits
  gelöschte Zeilen werden übersprungen (`dao/src/extra_hours.rs:87-99`).
- **Cutover-Konvergenz mit Absence (F05):** Nach Cutover werden für neue
  Absence-Kategorien (`Vacation`, `SickLeave`, `UnpaidLeave`) primär
  `AbsencePeriod`-Rows geschrieben; alte `extra_hours`-Zeilen dieser Kategorien
  bleiben lesbar. Die frühere Schreibsperre in `create()` wurde in Phase 8.4
  bewusst entfernt (`service_impl/src/extra_hours.rs:198-204`), damit
  Koexistenz-Modell M-01 möglich ist.
- **Konvertierungspfad:** REST-Endpoint `POST /extra-hours/{id}/convert-to-absence`
  (siehe §5) delegiert an den `AbsenceConversionService` — dieser markiert die
  Extra-Hours-Zeile intern als gelöscht und legt eine `AbsencePeriod`-Zeile an.
- **Custom Extra Hours Constraints:**
  - `HR_PRIVILEGE` für alle CUD-Operationen und `get_all`/`get_by_id`
    (`service_impl/src/custom_extra_hours.rs:38-79`, `:113-216`).
  - `get_by_sales_person_id` erlaubt HR **oder** die betroffene Sales Person
    (`service_impl/src/custom_extra_hours.rs:81-111`).
  - Create verlangt leere `id`, `version`, kein `created`, kein `deleted`;
    Delete = Soft-Delete via `deleted = NOW()`; Update prüft Version-Conflict.
  - Assignment (Zuweisung zu Sales Persons) läuft über
    `custom_extra_hours_sales_person`; das Array `assigned_sales_person_ids`
    ist Teil des Aggregats (siehe Datenmodell). **[Zu prüfen]**, wie
    Assignments beim Update konkret an die Link-Tabelle gemappt werden — die
    Trait-Signatur behandelt sie als `Arc<[Uuid]>`, die Persistenz liegt im
    SQLite-DAO.

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `extra_hours` | Einzeltag-Zeitzeile pro Sales Person | `id` (physisch), `logical_id`, `sales_person_id`, `amount`, `category`, `custom_extra_hours_id`, `description`, `date_time`, `created`, `deleted`, `update_process`, `update_timestamp`, `update_version` |
| `custom_extra_hours` | Katalog vom Betrieb definierter Zusatzkategorien | `id`, `name`, `description`, `modifies_balance`, `created`, `deleted`, `update_version`, `update_process` |
| `custom_extra_hours_sales_person` | N:M Zuweisung Custom-Kategorie ↔ Sales Person | `sales_person_id`, `custom_extra_hours_id` (Compound-PK), `created`, `deleted`, `update_process` |

### Migrations

Chronologisch, so wie die Tabelle historisch gewachsen ist:

- **`20240618125847_paid-sales-persons.sql`** — legt die Basistabelle
  `extra_hours` an (mit `id`, `sales_person_id`, `amount`, `category`,
  `description`, `date_time`, `created`, `deleted`, `update_*`), FK auf
  `sales_person`.
- **`20250413073750_add-custom-extra-hours-table.sql`** — führt Custom-
  Kategorien ein: `custom_extra_hours` + Verknüpfungstabelle
  `custom_extra_hours_sales_person`.
- **`20250418200122_insert-custom-column-to-extra-hours.sql`** — fügt
  `extra_hours.custom_extra_hours_id BLOB NOT NULL DEFAULT X'00…00'` hinzu.
  Der Nil-UUID-Default markiert "keine Custom-Kategorie" und ist der Grund,
  warum die DAO-Repräsentation `Custom(Uuid)` als expliziter Enum-Wert
  serialisiert wird — nicht als `Option`.
- **`20260428101456_add-logical-id-to-extra-hours.sql`** — führt
  `logical_id` ein: nullable-Add, Backfill (`logical_id = id`), CREATE-neu
  mit NOT-NULL-Rebuild, Partial-Unique-Index
  `idx_extra_hours_logical_id_active ON extra_hours(logical_id) WHERE
  deleted IS NULL`. Ab hier folgt Update dem "soft-delete + insert-neu"-
  Muster; die stabile API-ID ist `logical_id`, nicht mehr die physische
  Row-ID.

Zusätzlich relevant für die **Cutover-Interaktion mit F05**:

- **`20260502170000_create-absence-period.sql`** — führt `absence_period` ein
  (strikt additiv, `extra_hours` unangetastet).
- **`20260503000000_create-absence-migration-quarantine.sql`** — Quarantäne-
  Tabelle für Legacy-`extra_hours`-Zeilen, die nicht eindeutig migriert
  werden konnten (FK auf `extra_hours.id`).
- **`20260503000001_create-absence-period-migration-source.sql`** — Mapping-
  Tabelle: `extra_hours_id → absence_period_id`, Idempotenz-Key ist die
  Extra-Hours-Physical-ID.

### Beziehungen

```
sales_person 1─┬─* extra_hours ──(0..1)── custom_extra_hours  (via custom_extra_hours_id)
               │
               *
               │
       custom_extra_hours_sales_person  (N:M zwischen sales_person & custom_extra_hours)

absence_period_migration_source: (extra_hours.id) ─→ absence_period.id     [Cutover-Mapping]
absence_migration_quarantine:    (extra_hours.id) ─→ Quarantäne             [Cutover-Failed-Rows]
```

## 4. Service-API

### Traits

`service::extra_hours::ExtraHoursService` (`service/src/extra_hours.rs:187-248`):

```rust
#[async_trait]
pub trait ExtraHoursService {
    type Context;
    type Transaction: dao::Transaction;

    async fn find_by_sales_person_id_and_year(&self, sp: Uuid, year: u32, until_week: u8, ctx, tx) -> Arc<[ExtraHours]>;
    async fn find_by_sales_person_id_and_year_range(&self, sp: Uuid, from: ShiftyDate, to: ShiftyDate, ctx, tx) -> Arc<[ExtraHours]>;
    async fn find_by_week(&self, year: u32, week: u8, ctx, tx) -> Arc<[ExtraHours]>;
    async fn create(&self, entity: &ExtraHours, ctx, tx) -> ExtraHours;
    async fn update(&self, entity: &ExtraHours, ctx, tx) -> ExtraHours;
    async fn delete(&self, id: Uuid, ctx, tx) -> ();
    async fn soft_delete_bulk(&self, ids: Arc<[Uuid]>, update_process: &str, ctx, tx) -> ();
}
```

`service::custom_extra_hours::CustomExtraHoursService` (`service/src/custom_extra_hours.rs:60-105`):
`get_all`, `get_by_id`, `get_by_sales_person_id`, `create`, `update`, `delete`.

### Auth-Gates

| Methode | Gate |
| --- | --- |
| `ExtraHoursService::find_by_sales_person_id_and_year(_range)` | HR **oder** self (`service_impl/src/extra_hours.rs:118-127`) |
| `ExtraHoursService::find_by_week` | `check_only_full_authentication` — reiner Interner Pfad (Reporting/Scheduler) (`service_impl/src/extra_hours.rs:162-164`) |
| `ExtraHoursService::create` | HR **oder** self für die Ziel-Sales-Person |
| `ExtraHoursService::update` | HR **oder** self für die betroffene Zeile |
| `ExtraHoursService::delete` | HR **oder** self (doppelt geprüft, einmal via `SALES_PRIVILEGE`, einmal via `verify_user_is_sales_person`) |
| `ExtraHoursService::soft_delete_bulk` | `CUTOVER_ADMIN_PRIVILEGE` **vor** jeder DAO-Arbeit; nur der Cutover-Commit-Pfad ruft das |
| `CustomExtraHoursService::get_all` / `get_by_id` / `create` / `update` / `delete` | HR |
| `CustomExtraHoursService::get_by_sales_person_id` | HR **oder** self |

### TX-Verhalten

- Alle Methoden akzeptieren `Option<Self::Transaction>` und ziehen bei `None`
  eine eigene per `use_transaction`.
- `create`, `update`, `delete` committen selbst.
- `soft_delete_bulk` committet **nicht** — die Cutover-Kette hält die
  Transaktion und committet erst am Ende
  (`service_impl/src/extra_hours.rs:395-398`).
- `update` läuft atomar: Soft-Delete der alten Zeile + Insert der neuen Zeile
  in derselben Transaktion.

### Dependencies

`ExtraHoursServiceImpl` — Business-Logic-Tier (konsumiert einen anderen
Domain-Service):

- DAOs: `ExtraHoursDao`, `TransactionDao`
- Basic-Services: `PermissionService`, `SalesPersonService`
- Business-Logic-Service (Lazy-Load-Auflösung): `CustomExtraHoursService`
  — wird intern für das Nachladen der Custom-Definition mit
  `Authentication::Full` aufgerufen (`service_impl/src/extra_hours.rs:51-54`);
  dieser Full-Context-Bypass ist explizit dokumentiert und für interne
  Aggregat-Konsumenten der Toggle- und Custom-Kategorie-Reads vorgesehen
  (vgl. Memory "ToggleService Full-Context-Bypass").
- Infrastruktur: `ClockService`, `UuidService`.

`CustomExtraHoursServiceImpl` — Basic-Tier (nur DAO + Permission +
Transaction + `SalesPersonService` für den self-Check; kein weiterer
Domain-Service):

- DAO: `CustomExtraHoursDao`
- Basic-Services: `PermissionService`, `SalesPersonService`
- Infrastruktur: `ClockService`, `UuidService`, `TransactionDao`

## 5. REST-Endpoints

### Extra Hours

Router: `rest/src/extra_hours.rs:22-35`, gemountet unter `/extra-hours`
(`rest/src/lib.rs:667`).

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/extra-hours/by-sales-person/{id}?year=…&until_week=…` | Alle Zeilen einer Sales Person bis KW `until_week` im Jahr `year` | — | `Vec<ExtraHoursTO>` | 401, 404 |
| `POST` | `/extra-hours` | Neue Zeile anlegen | `ExtraHoursTO` | `ExtraHoursTO` (Status 201) | 400 (Validation), 403 |
| `PUT` | `/extra-hours/{id}` | Zeile updaten (Logical-ID im Pfad); versioniert | `ExtraHoursTO` | `ExtraHoursTO` | 400, 403, 404, 409 |
| `DELETE` | `/extra-hours/{id}` | Soft-Delete (Logical-ID) | — | 204 | 404 |
| `POST` | `/extra-hours/{id}/convert-to-absence` | Legacy-Zeile in `AbsencePeriod` konvertieren | `ConvertExtraHoursRequestTO` | `AbsencePeriodTO` | 403 (HR), 404 (Soft-Deleted/Unknown), 422 (`DateOrderWrong`, `OverlappingPeriod`) |

DTOs siehe `rest_types` (`rest-types/src/lib.rs:797-870, 1859-…`).

### Custom Extra Hours

Router: `rest/src/custom_extra_hours.rs:17-28`, gemountet unter
`/custom-extra-hours` (`rest/src/lib.rs:644`).

| Methode | Pfad | Beschreibung | DTO In | DTO Out |
| --- | --- | --- | --- | --- |
| `GET` | `/custom-extra-hours` | Alle Custom-Kategorien | — | `Vec<CustomExtraHoursTO>` |
| `GET` | `/custom-extra-hours/{id}` | Einzelne Kategorie | — | `CustomExtraHoursTO` |
| `GET` | `/custom-extra-hours/by-sales-person/{sales_person_id}` | Nur die der Sales Person zugewiesenen | — | `Vec<CustomExtraHoursTO>` |
| `POST` | `/custom-extra-hours` | Anlegen | `CustomExtraHoursTO` | `CustomExtraHoursTO` (201) |
| `PUT` | `/custom-extra-hours/{id}` | Update | `CustomExtraHoursTO` | `CustomExtraHoursTO` |
| `DELETE` | `/custom-extra-hours/{id}` | Soft-Delete | — | 204 |

**Hinweis Doku-Drift:** Die utoipa-Annotation für DELETE nennt `/custom-extra-hours/{id}`
(`rest/src/custom_extra_hours.rs:212`), was dem Router-Mount zusammen ergibt.
Der Handler nutzt `Path<Uuid>`; der Effektpfad ist unverändert
`DELETE /custom-extra-hours/{id}`.

## 6. Frontend-Integration

- **Pages:** `shifty-dioxus/src/page/custom_extra_hours_management.rs` — HR-Seite
  zum Verwalten der Custom-Kategorien (Anlegen/Editieren/Löschen). Extra-Hours-
  Einzelzeilen werden aktuell nicht auf einer eigenen Page verwaltet, sondern
  aus den Employee-Details-Seiten heraus.
- **API-Client:** `shifty-dioxus/src/api.rs` — `get_custom_extra_hours_by_sales_person`,
  `post_custom_extra_hours`, `put_custom_extra_hours`, `delete_custom_extra_hours`
  (Aufrufsites in `custom_extra_hours_management.rs:73-140`).
- **State-Objekte:** `shifty-dioxus/src/state/employee.rs` —
  `CustomExtraHoursDefinition` als Frontend-View-Model (Signatur:
  `custom_extra_hours_management.rs:12`).
- **i18n-Keys** (`custom_extra_hours_management.rs:49-58`):
  `CustomExtraHoursManagement`, `Name`, `Description`, `ModifiesBalance`,
  `Actions`, `AddNew`, `Save`, `Cancel`, `Edit`, `Delete`.
- **Proxy** (`shifty-dioxus/Dioxus.toml:57-58, 71-72`):
  - `/custom-extra-hours` → `http://localhost:3000/custom-extra-hours`
  - `/extra-hours` → `http://localhost:3000/extra-hours`
- **Bekannte Frontend-Lücken:**
  - Assignment "Custom-Kategorie ↔ Sales Person" ist im aktuellen UI mit
    `assigned_sales_person_ids: vec![]` hartcodiert
    (`custom_extra_hours_management.rs:190,197`); der Kommentar dort dokumentiert
    das explizit als offenes Feature.
  - `Load` wird nach jeder mutierenden Aktion nachgetriggert
    (`custom_extra_hours_management.rs:205, 330`), um den State zu
    synchronisieren.

## 7. Randfälle

Zentrale Randfall-Referenz: [`../domain/edge-cases.md#2-absence--extra-hours`](../domain/edge-cases.md#2-absence--extra-hours)
sowie [Section 8 "Soft-Delete-Konsistenz"](../domain/edge-cases.md#8-soft-delete-konsistenz).

Feature-spezifisch:

- **Cutover-Split Vacation/SickLeave/UnpaidLeave (F04 × F05):** Nach Cutover
  können für dieselbe Person und denselben Zeitraum **beide** Datenquellen
  existieren — alte Zeilen in `extra_hours` (nicht gelöscht, sondern
  konvertiert oder stehen gelassen) und neue in `absence_period`. Jeder
  Report-/Balance-Pfad muss **beide** Quellen aggregieren, sonst kippt die
  Bilanz. Das ist der prominenteste Randfall des Clusters — siehe
  `../domain/edge-cases.md#21-cutover-historie`.
- **`UnpaidLeave` senkt die Erwartung:** Reporting muss `UnpaidLeave`-Zeilen
  explizit ausfiltern und die Wochenerwartung entsprechend reduzieren
  (`service_impl/src/reporting.rs:562`, `:974`). Wer die Kategorie in einem
  neuen Aggregat vergisst, rechnet zu viel Erwartung — d.h. die Sales Person
  erscheint mit einem Minus in der Balance, das gar nicht existiert.
- **Custom-Kategorie ungeladen → keine Bilanz-Wirkung:** Wenn
  `load_custom_extra_hours_definitions` fehlschlägt (Definition gelöscht /
  nicht gefunden), fällt `LazyLoad.get()` auf `None`, und beide Semantik-
  Funktionen (`as_report_type`, `availability`) liefern `None`. Zeile ist
  effektiv unsichtbar für die Bilanz. Der Log-Warn-Pfad markiert das als
  Integrity-Issue (`service_impl/src/extra_hours.rs:60-72`), stoppt aber die
  Abfrage nicht.
- **`Unavailable`-Zeilen brauchen kein Amount, um zu wirken:** Sie sind reine
  Verfügbarkeits-Marker; ihre `amount` beeinflusst nichts in der Balance.
  Trotzdem wird `amount` mitgeschrieben und in Reports mit `Documented`-
  Semantik durchgereicht.
- **Snapshot-Drift bei Löschen/Update:** Wird eine `extra_hours`-Zeile
  gelöscht, deren Beitrag bereits in einem persistierten `billing_period`-
  Snapshot enthalten ist, driftet die Live-Ansicht gegen den Snapshot. Ohne
  Version-Bump von `CURRENT_SNAPSHOT_SCHEMA_VERSION` ist der Diff nicht als
  echter Delete identifizierbar. Siehe `../domain/edge-cases.md#23-legacy-extra-hours--delete-semantik`.
- **`logical_id`-Reuse verboten:** Der Partial-Unique-Index
  `idx_extra_hours_logical_id_active` erzwingt "eine aktive Zeile pro
  logical_id". Wer im Test bei einer Neuanlage eine `logical_id` einer
  soft-deleted Zeile setzt, kollidiert nicht — bei einer aktiven **schon**.
- **`convert-to-absence` verlangt gültiges Range:** Der REST-Endpoint mappt
  `DateOrderWrong` / `OverlappingPeriod` auf 422; die eigentliche
  Konvertierungs-Semantik lebt in `AbsenceConversionService` (F05).

## 8. Tests

- **Unit / Service-Tests:**
  - `service_impl/src/test/extra_hours.rs` (748 Zeilen) deckt die
    Update-Semantik "soft-delete + insert" ab, den OR-Permission-Flow (HR vs.
    self), Version-Conflict, den Reject bei geändertem `sales_person_id`,
    NotFound bei unbekannter/gelöschter Zeile, und den Phase-4-Bulk-Delete-
    Pfad (Happy-Path + Elevation-of-Privilege-Guard, der explizit
    `MockExtraHoursDao::expect_soft_delete_bulk().times(0)` einpint, bevor
    das Permission-Gate ablehnt).
  - `service_impl/src/test/custom_extra_hours.rs` (620 Zeilen) deckt CRUD +
    die Sales-Person-Zuweisungs-Filter für `get_by_sales_person_id` ab.
  - `service::extra_hours`-interne Tests
    (`service/src/extra_hours.rs:254-268`) pinnen die `UnpaidLeave`-
    Klassifikation.
  - DAO-Trait-Default-Tests
    (`dao/src/custom_extra_hours.rs:172-221`) belegen `find_all` /
    `find_by_id` / `find_by_sales_person_id` inkl. Soft-Delete-Filter.
- **Integration:** In-Mem-SQLite-Runs des DAO-Impl liegen in
  `dao_impl_sqlite/src/…` (verkabelt über die üblichen
  `sqlx::sqlite::SqlitePool::connect(":memory:")`-Harnesses).
- **Bekannte Lücken:**
  - **[Zu prüfen]** ob es einen dedizierten Test gibt, der die
    Reporting-Aggregation über `extra_hours` **und** `absence_period` an
    einer Cutover-überspannenden Periode misst; das wäre der
    hoch-Wert-Regressionsguard aus Randfall §7.
  - **[Zu prüfen]** ob der `convert-to-absence`-Endpoint einen
    Ende-zu-Ende-Roundtrip-Test hat, der auf die `AbsenceConversionService`-
    Impl draufgeht (nicht nur Mock-Layer).

## 9. Historie & Kontext

- **Initial (2024-06):** Migration `20240618125847_paid-sales-persons.sql`
  legt `extra_hours` und `working_hours` gemeinsam an — der ursprüngliche
  Zeit-Erfassungs-Baustein für HR & Reporting.
- **2025-04 — Custom Extra Hours eingeführt:**
  - `20250413073750_add-custom-extra-hours-table.sql` (Katalog +
    N:M-Zuweisung),
  - `20250418200122_insert-custom-column-to-extra-hours.sql` (Foreign-Key-
    Spalte in `extra_hours`).
  - Motivation: Betriebs-definierte Kategorien ohne Enum-Extension.
- **v1.0 / Cutover (2026-05, Phase 4):** Range-basiertes Absence-Aggregat
  (`absence_period`) übernimmt die Kategorien Vacation/SickLeave/UnpaidLeave
  im Neu-Fall. Bestehende `extra_hours`-Zeilen dieser Kategorien werden
  entweder migriert (`absence_period_migration_source`), in Quarantäne
  gelegt (`absence_migration_quarantine`) oder — im Koexistenz-Modell — als
  historische Zeilen belassen. `soft_delete_bulk` ist der Massenpfad, mit
  dem der Cutover-Commit gemappte Legacy-Zeilen final aus dem Live-Read
  ausblendet.
- **v1.3 / Phase 8.4:** Die Schreibsperre für die deprecated-Kategorien
  wurde entfernt (`service_impl/src/extra_hours.rs:198-204`) — Koexistenz
  M-01 ist die endgültige Modellentscheidung, nicht "Absence löst
  Extra-Hours ab". Neue Zeilen dieser Kategorien sind wieder ohne
  Feature-Gate anlegbar, u.a. für Korrekturen historischer Daten.
- **Phase 51 / Toggle-Bypass:** Die interne Nutzung von
  `Authentication::Full` beim Nachladen der Custom-Definition ist
  konsistent mit dem in Phase 51 dokumentierten Bypass für
  Internal-Aggregate-Konsumenten (Memory "ToggleService Full-Context-Bypass").
- **Verweise auf `.planning/phases/…`** für den Cutover-Kontext:
  `.planning/phases/04-*` (Migration & Cutover) und `.planning/phases/08-*`
  (Koexistenz-Nachjustierung). [Zu prüfen] konkrete Phase-IDs im aktuellen
  Milestone-Cleanup-Stand.

---

**Fazit:** `extra_hours` ist die dauerhaft führende Datenquelle für Overtime,
Volunteer und Custom-Kategorien; für Vacation/SickLeave/UnpaidLeave ist es
Legacy-Koexistent zu `absence_period` (F05) — jeder Report muss beide Quellen
lesen, sonst kippt die Balance.

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
