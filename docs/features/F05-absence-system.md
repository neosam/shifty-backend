# Feature: Absence System (Range-basierte Abwesenheiten)

> **Kurzform:** Urlaub, Krank, unbezahlter Urlaub und ähnliche Abwesenheiten
> werden als **Zeiträume** (`from_date`–`to_date`) modelliert. Die Ist-Stunden
> werden zur Reporting-Zeit aus dem am jeweiligen Tag gültigen Vertrag
> abgeleitet — statt aus fixen Tages-Postings in `extra_hours`.

**Cluster-ID:** F05
**Status:** produktiv
**Erstmalig eingeführt:** v1.0 (2026-05-03, Phasen 1–4 — siehe
`docs/absence-feature-frontend.md` Kopf)
**Zuständige Crates:** `service::absence`, `service::absence_conversion`,
`service_impl::absence`, `service_impl::absence_conversion`, `dao::absence`,
`dao_impl_sqlite::absence`, `rest::absence`, `shifty-dioxus::page::absences`

---

## 1. Was ist das? (Fachlich)

Vor v1.0 wurde jede Form von Abwesenheit als **einzelner Tageseintrag mit
Stundenbetrag** in `extra_hours` erfasst (Kategorien `Vacation`, `SickLeave`,
`UnpaidLeave`, plus freiwilliger Dienst). Das brachte drei Probleme mit sich
(siehe `docs/absence-feature-frontend.md:6–13`):

1. Vertragsänderungen (z. B. Wechsel von 40 h auf 30 h pro Woche) verändern die
   Ist-Stunden vergangener Urlaubstage — die Buchhaltung musste nacharbeiten.
2. Für denselben Tag entstand eine Doppel-Eintragung: einmal `extra_hours`
   (Stunden-Betrag) und einmal `sales_person_unavailable` (Schichtplan-Sicht).
3. Feiertage mussten manuell aus Urlaubs-Postings herausgerechnet werden.

Das **Absence-System** modelliert einen Zeitraum genau einmal (Tabelle
`absence_period`). Die Stunden pro Tag werden **erst beim Report** aus dem am
jeweiligen Tag aktiven Vertrag abgeleitet
(`service::absence::AbsenceService::derive_hours_for_range`, definiert in
`service/src/absence.rs:250–258`, Impl in
`service_impl/src/absence.rs:387–556`). Feiertage bekommen 0 h, ohne
gesonderte Buchhaltungs-Zeile.

**Beispiel-Workflow aus User-Sicht:**

1. Mitarbeiter*in öffnet `/absences`, klickt "Neue Abwesenheit", wählt
   Kategorie (Urlaub / Krank / Unbezahlt), Von-Datum, Bis-Datum und
   optional Halbtag.
2. Backend prüft Range-Reihenfolge, Self-Overlap innerhalb derselben
   Kategorie und Permission (HR ∨ self), persistiert den Datensatz.
3. Backend berechnet Forward-Warnings: existierende Bookings oder manuelle
   `sales_person_unavailable`-Einträge im neuen Range werden als
   **nicht-blockierender Hinweis** ausgeliefert (Wrapper
   `AbsencePeriodCreateResultTO`).
4. Frontend zeigt die neue Range in der Übersicht und rendert die Warnings
   als Banner-Liste unter dem Dialog — der Datensatz ist bereits gespeichert.
5. Reporting-Fluss (F06) fragt später
   `derive_hours_for_range(from, to, sales_person_id)` und bekommt eine
   `BTreeMap<Date, ResolvedAbsence>` mit den pro Tag gültigen Stunden.

## 2. Fachliche Regeln

Alle Regeln aus `docs/absence-feature-frontend.md:16–30` verifiziert gegen
den Code:

- **Granularität:** Ganztag oder Halbtag pro Periode einheitlich
  (`DayFraction::{Full, Half}`, `service/src/absence.rs:61–66`,
  Migration `20260517120000` bringt die Spalte). Halbtage werden in
  `derive_hours_for_range` mit Faktor 0.5 verrechnet
  (`service_impl/src/absence.rs:538–541`).
- **Range-Semantik `[from_date, to_date]` — inklusiv beidseitig** (D-05).
  DB-CHECK `to_date >= from_date` in
  `migrations/sqlite/20260502170000_create-absence-period.sql:28`. Der Service
  wickelt Ranges via `shifty_utils::DateRange::new` — Inversion → typisierter
  Fehler `ServiceError::DateOrderWrong(from, to)`
  (`service_impl/src/absence.rs:189–190`).
- **Kategorien:** Genau drei (`Vacation`, `SickLeave`, `UnpaidLeave`). Der
  DAO-Enum `AbsenceCategoryEntity` ist bewusst kleiner als
  `ExtraHoursCategoryEntity`, damit der Compiler ungültige Kategorien
  ausschließt (`dao/src/absence.rs:9–21`).
- **Self-Overlap same-category ist verboten:** Der Create-Pfad ruft
  `find_overlapping(sales_person_id, category, range, None, tx)`
  (`service_impl/src/absence.rs:193–207`) und antwortet
  `ValidationError([OverlappingPeriod(logical_id)])`. Der Update-Pfad ruft
  dieselbe DAO-Methode mit `exclude_logical_id = Some(logical_id)`, damit
  die Row nicht mit sich selbst kollidiert
  (`service_impl/src/absence.rs:281–290`).
- **Cross-Category-Overlap ist erlaubt und wird per Priorität aufgelöst:**
  `SickLeave > Vacation > UnpaidLeave` (BUrlG §9-konform,
  `service_impl/src/absence.rs:65–74`, D-Phase2-03). Anwendung im
  Reporting-Fluss über `derive_hours_for_range` → `max_by_key(priority)`
  (`service_impl/src/absence.rs:507–512`).
- **Berechtigung:** Für alle Read-/Write-Operationen gilt **HR ∨ self**
  (D-10 Option A). Umgesetzt via
  `tokio::join!(check_permission(HR), verify_user_is_sales_person(sp_id))`
  gefolgt von `hr.or(sp)?`
  (`service_impl/src/absence.rs:110–119`, analog in `find_by_id`, `create`,
  `update`, `delete`, `find_overlapping_for_booking`,
  `derive_hours_for_range`). Ausnahme: `find_all` erwartet **HR only**
  (`service_impl/src/absence.rs:94–96`) — der By-Sales-Person-Pfad ist der
  Selbst-Sicht-Endpoint.
- **Booking-Konflikt ist NICHT blockierend:** Weder `create` noch `update`
  brechen bei überlappenden Bookings ab; sie liefern Forward-Warnings vom
  Typ `Warning::AbsenceOverlapsBooking` /
  `Warning::AbsenceOverlapsManualUnavailable`
  (`service_impl/src/absence.rs:837–927`, D-Phase3-16 "kein Auto-Cleanup").
- **Sales-Person-ID ist auf Update nicht änderbar:** Modification-Guard in
  `service_impl/src/absence.rs:265–269`, sonst `ValidationError`.
- **Optimistic Locking:** `version` (Uuid). PUT liefert `409 EntityConflicts`
  bei Stale-Version (`service_impl/src/absence.rs:270–276`).
- **Update rotiert die physische Row (Tombstone + Insert):** `logical_id`
  bleibt stabil, externe Referenzen überleben Updates (D-07,
  `service_impl/src/absence.rs:297–331`). Analog `extra_hours`-Pattern.
- **Soft-Delete:** `delete` setzt nur `deleted`
  (`service_impl/src/absence.rs:354–385`). Kein physisches Row-Drop —
  Audit-Trail bleibt bestehen.
- **Wochen-Deckelung im Reporting-Aggregator:** Pro ISO-Woche werden
  maximal `workdays_per_week` Urlaubstage gezählt, auch wenn der Vertrag
  an mehr Wochentagen verfügbar ist (siehe langer Kommentar
  `service_impl/src/absence.rs:462–472`). Bug-Motivation:
  `vacation-hours-overcounted`.
- **Feiertage kürzen den Anspruch:** In `derive_hours_for_range` werden
  Tage mit `SpecialDayType::Holiday` übersprungen — kein Eintrag in der
  Map (`service_impl/src/absence.rs:437–458, 501–503`).
- **REST-`path-id wins` auf PUT:** Body-`id` wird überschrieben mit dem
  Path-Segment (`rest/src/absence.rs:373`).

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `absence_period` | Persistierter Range pro `(sales_person, kategorie)` | `id`, `logical_id`, `sales_person_id`, `category`, `from_date`, `to_date`, `description`, `created`, `deleted`, `update_version`, `day_fraction` |
| `absence_period_migration_source` | Backlink `extra_hours_id → absence_period_id`, damit Convert-Vorgänge nachvollziehbar sind | `extra_hours_id`, `absence_period_id`, `migrated_at` |

Schema-Auszug (`migrations/sqlite/20260502170000_create-absence-period.sql:14–43`):

```sql
CREATE TABLE absence_period (
    id              BLOB(16) NOT NULL PRIMARY KEY,
    logical_id      BLOB(16) NOT NULL,
    sales_person_id BLOB(16) NOT NULL,
    category        TEXT NOT NULL,          -- 'Vacation'|'SickLeave'|'UnpaidLeave'
    from_date       TEXT NOT NULL,
    to_date         TEXT NOT NULL,
    description     TEXT,
    created         TEXT NOT NULL,
    deleted         TEXT,                   -- Soft-Delete
    update_timestamp TEXT,
    update_process  TEXT NOT NULL,
    update_version  BLOB(16) NOT NULL,
    CHECK (to_date >= from_date),
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);
```

Indexe:

- `idx_absence_period_logical_id_active` — UNIQUE, WHERE `deleted IS NULL`.
  Garantiert, dass pro `logical_id` immer nur eine lebende Row existiert
  (Tombstone-Pattern).
- `idx_absence_period_sales_person_from` — für den by-sales-person-Read und
  `find_overlapping_for_booking`.
- `idx_absence_period_self_overlap` — für den Self-Overlap-Check im
  Create/Update-Pfad.

### Migrations

Chronologisch:

- `20260502170000_create-absence-period.sql` — Basistabelle + drei Indexe
  (Phase 1, Recovery aus Phase-1-Worktree-Verlust).
- `20260503000000_create-absence-migration-quarantine.sql` — Cutover-
  Quarantäne-Tabelle (später wieder entfernt).
- `20260503000001_create-absence-period-migration-source.sql` — Backlink
  von `extra_hours` zu neu erzeugten `absence_period`-Rows.
- `20260517120000_add-day-fraction-to-absence-period.sql` — Additiv:
  `day_fraction TEXT NOT NULL DEFAULT 'full' CHECK (day_fraction IN
  ('full', 'half'))` (Phase 8.3, No-Drift für Bestandsdaten).
- `20260611000000_drop-absence-migration-quarantine.sql` — Quarantäne-
  Tabelle nach abgeschlossenem Cutover entfernt.
- `20260611000002_delete-absence-range-source-active-seed.sql` — Toggle-
  Seed `absence_range_source_active` entfernt (M-03: kein Quellen-Schalter
  mehr, siehe `service_impl/src/test/reporting_additive_merge.rs:5`).

### Beziehungen

- `absence_period.sales_person_id → sales_person.id` (FK).
- `absence_period_migration_source.absence_period_id → absence_period.id`
  (Backlink zu Convert-Vorgang; Write in
  `service_impl/src/absence_conversion.rs:151–160`).
- Kein FK auf `booking` oder `sales_person_unavailable` — Konflikte sind
  Warnings, keine harten Verweise.

## 4. Service-API

Absence ist **Business-Logic-Tier** (siehe CLAUDE.md
"Service-Tier-Konventionen"): `AbsenceServiceImpl` konsumiert neben DAOs
und `PermissionService` auch `SalesPersonService`, `SpecialDayService`,
`EmployeeWorkDetailsService`, `BookingService`,
`SalesPersonUnavailableService` und `SlotService`
(`service_impl/src/absence.rs:46–63`). Das ist bewusst — Absence kombiniert
Sales-Person + Slot + Booking und braucht die Cross-Aggregate-Sicht für
Forward-Warnings.

### Trait `AbsenceService`

Definition: `service/src/absence.rs:181–302`. Die wichtigsten Methoden:

```rust
pub trait AbsenceService {
    type Context;
    type Transaction: dao::Transaction;

    // Read
    async fn find_all(&self, ctx, tx) -> Result<Arc<[AbsencePeriod]>, ServiceError>;
    async fn find_by_sales_person(&self, sp_id, ctx, tx) -> ...;
    async fn find_by_id(&self, id, ctx, tx) -> Result<AbsencePeriod, ServiceError>;
    async fn find_overlapping_for_booking(&self, sp_id, range, ctx, tx) -> ...;

    // Write
    async fn create(&self, req, ctx, tx) -> Result<AbsencePeriodCreateResult, ServiceError>;
    async fn update(&self, req, ctx, tx) -> Result<AbsencePeriodCreateResult, ServiceError>;
    async fn delete(&self, id, ctx, tx) -> Result<(), ServiceError>;

    // Reporting-Bridge
    async fn derive_hours_for_range(&self, from, to, sp_id, ctx, tx)
        -> Result<BTreeMap<Date, ResolvedAbsence>, ServiceError>;

    // Legacy-Marker-Anzeige (extra_hours → Anzeige-Tage / Convert-Range-Vorschlag)
    async fn derive_days_for_hourly_markers(&self, sp_id, markers, ctx, tx) -> ...;
    async fn suggest_convert_ranges_for_markers(&self, sp_id, markers, ctx, tx) -> ...;
}
```

### `AbsenceCategory` und `DayFraction`

Domain-Enums in `service/src/absence.rs:27–51` bzw. `61–84`, jeweils mit
`From<&…Entity>`-Konvertern. `DayFraction::default() == Full` (Zeile 63);
Bestandsdaten kommen so ohne Backfill aus.

### `ResolvedAbsence` und `AbsencePeriodCreateResult`

- `ResolvedAbsence { category, hours, days }` — pro Tag bereits conflict-
  resolved (`service/src/absence.rs:160–165`). `hours = days * hours_per_day`,
  Halbtag / Wochen-Deckelung sind eingerechnet.
- `AbsencePeriodCreateResult { absence, warnings: Arc<[Warning]> }` — Wrapper
  für Create/Update-Antworten (`service/src/absence.rs:174–177`).

### Auth-Gates

| Methode | Gate |
| --- | --- |
| `find_all` | HR only |
| `find_by_sales_person` / `find_by_id` / `find_overlapping_for_booking` / `derive_hours_for_range` / `derive_days_for_hourly_markers` / `suggest_convert_ranges_for_markers` | HR ∨ self |
| `create` / `update` / `delete` | HR ∨ self |

Verifiziert im Test `test_create_other_sales_person_without_hr_is_forbidden`
(`service_impl/src/test/absence.rs:403–425`), analog für find_all-non-HR
(`test_find_all_non_hr_is_forbidden`, Zeile 804).

### TX-Verhalten

Alle Methoden öffnen bei `tx = None` selbst eine Transaktion via
`transaction_dao.use_transaction(tx).await?` und committen erst nach
erfolgreicher Business-Logik. Der Forward-Warning-Loop läuft **nach** dem
DAO-Persist und **vor** `commit` — falls ein Warning-Lookup fehlschlägt,
wird die neue Absence-Row zusammen mit den bereits geschriebenen Änderungen
zurückgerollt (`service_impl/src/absence.rs:220–235, 333–347`).

Der Update-Pfad ist Composite (Tombstone alte Row + Insert neue Row +
Warning-Loop) und läuft **atomar** in einer einzigen Transaktion
(`service_impl/src/absence.rs:297–347`).

### `AbsenceConversionService`

Zweiter Trait im Cluster (`service/src/absence_conversion.rs:26–48`).
Konvertiert eine lebende `extra_hours`-Row (Kategorie ∈
{Vacation, SickLeave, UnpaidLeave}) atomar in eine `absence_period`:

1. HR-Privileg prüfen (**HR only**, kein Self-Bypass — D-05).
2. `extra_hours`-Row via `find_by_logical_id` laden.
3. Range-Validierung + Overlap-Check gegen bestehende `absence_period`.
4. `absence_dao.create(...)` (Write 1).
5. `migration_source_dao.upsert_migration_source(...)` — Backlink (Write 2).
6. `extra_hours_service.soft_delete_bulk(...)` — **über die physische
   `entity.id`, nicht die `logical_id`** (Kommentar
   `service_impl/src/absence_conversion.rs:164–171`, CR-01: sonst
   Doppelzählung, weil versionierte Rows physisch unterschiedliche IDs
   haben).

Alle drei Writes laufen in einer gemeinsamen Transaktion. Kein Snapshot-
Bump nötig (D-16), weil Reporting seit 8.4 additiv aus beiden Quellen
summiert.

### `absence_conversion` als Konverter zur Reporting-Tagesreihe

Anmerkung zur Aufgabenbeschreibung: Der eigentliche Konverter zwischen
Absence-Range und Reporting-Tagesreihe ist **nicht** `absence_conversion.rs`,
sondern `AbsenceService::derive_hours_for_range`
(`service_impl/src/absence.rs:387–556`). `absence_conversion.rs` bezieht
sich ausschließlich auf den einmaligen Datenumzug einer Legacy-`extra_hours`-
Row in eine neue Absence-Periode und ist eng verwandt mit dem historischen
`CutoverService` aus v1.0 (siehe Kommentar-Kopf
`service_impl/src/absence_conversion.rs:1–9`). Beide Wege enden in
`absence_period`-Rows, die dann vom Reporting einheitlich behandelt werden.

### Dependencies

- DAOs: `AbsenceDao`, `MigrationSourceDao` (nur im Conversion-Service),
  `ExtraHoursDao` (nur im Conversion-Service), `TransactionDao`.
- Services (nur `AbsenceService`, Basic-Konsumenten):
  `PermissionService`, `SalesPersonService`, `ClockService`,
  `UuidService`, `SpecialDayService`, `EmployeeWorkDetailsService`,
  `BookingService`, `SalesPersonUnavailableService`, `SlotService`.
- Services (nur `AbsenceConversionService`): `ExtraHoursService`,
  `PermissionService`.

## 5. REST-Endpoints

Route-Basispfad `/absence-period`, montiert in `rest/src/lib.rs:656`.

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `POST` | `/absence-period` | Neue Periode anlegen | `AbsencePeriodTO` | `201 AbsencePeriodCreateResultTO` | 403 (Auth), 422 (Range invers, Self-Overlap, id/version/created/deleted preset) |
| `GET`  | `/absence-period` | Alle Perioden **+ lebende Legacy-Marker** aller Personen (HR-Sicht) | — | `200 AbsenceListWithProjectionTO` | 403 |
| `GET`  | `/absence-period/{id}` | Einzelne Periode | — | `200 AbsencePeriodTO` | 403, 404 |
| `PUT`  | `/absence-period/{id}` | Periode ändern (`path-id wins`) | `AbsencePeriodTO` | `200 AbsencePeriodCreateResultTO` | 403, 404, 409 (Version), 422 |
| `DELETE` | `/absence-period/{id}` | Soft-Delete | — | `204` | 403, 404 |
| `GET`  | `/absence-period/by-sales-person/{sales_person_id}` | Perioden + Marker einer Person | — | `200 AbsenceListWithProjectionTO` | 403 |

Handler in `rest/src/absence.rs:163–174` (Router), `188–210`
(`create_absence_period`), `222–314` (`get_all_absence_periods`), `328–346`
(`get_absence_period`), `363–386` (`update_absence_period`), `400–413`
(`delete_absence_period`), `426–521` (`get_absence_periods_for_sales_person`).

**Verwandter Convert-Endpoint** (in Cluster F04 dokumentiert, hier nur
verlinkt): `POST /extra-hours/{id}/convert-to-absence` in
`rest/src/extra_hours.rs:32,203`. Body `ConvertExtraHoursRequestTO`,
dispatcht in `AbsenceConversionService::convert_extra_hours_to_absence`.

**Deprecation-Sonderfall:** Ein `POST /extra-hours` mit Kategorie
`Vacation`/`SickLeave`/`UnpaidLeave` liefert nach Cutover-Flip `403
ExtraHoursCategoryDeprecatedErrorTO` (`rest/src/lib.rs:284–295`, Body:
`{"error":"extra_hours_category_deprecated","category":"vacation","message":"Use POST /absence-period for this category"}`).
Frontend erkennt den Fehler an `error == "extra_hours_category_deprecated"`.

**Wrapper-Details:** `AbsencePeriodCreateResultTO` trägt `.absence`
(persistierte Periode) und `.warnings` (Forward-Warnings —
`AbsenceOverlapsBooking` und `AbsenceOverlapsManualUnavailable`,
`rest-types/src/lib.rs:1942–1970`). `AbsenceListWithProjectionTO` bündelt
`absence_periods` und `hourly_markers` — Marker sind lebende Legacy-
`extra_hours`-Rows der drei Absence-Kategorien, angereichert um
`derived_days`, `suggested_end` und `is_full_week`
(`rest-types/src/lib.rs:1872–1916`, Handler
`rest/src/absence.rs:250–307`).

DTOs zusammengefasst in `rest-types/src/lib.rs:1719–2040`.

## 6. Frontend-Integration

- **Page:** `shifty-dioxus/src/page/absences.rs` (~4085 Zeilen). Route
  `/absences`. HR- vs Employee-Branch via `auth.has_privilege("hr")`
  (`shifty-dioxus/src/page/absences.rs:1–17`).
- **Services:** `shifty-dioxus/src/service/absence.rs` (CRUD-Coroutine,
  `ABSENCE_STORE`, `ABSENCE_MODAL_EVENT`, `ABSENCE_REFRESH`,
  `ABSENCE_HOURLY_STORE`) und
  `shifty-dioxus/src/service/absence_marker.rs` (Legacy-Marker-Store).
- **State:** `shifty-dioxus/src/state/absence_period.rs`
  (`AbsencePeriod`, `AbsenceCategory`, `DayFraction`, `ExtraHoursMarker`).
- **Zusatz-Komponenten:** `AbsenceModal`, `AbsenceConvertModal`,
  `ExtraHoursModal`, `WarningList`/`WarningsList`, `CategoryBadge`,
  `StatusPill`, `VacationEntitlementCard`, `VacationPerPersonList`,
  `AbsenceList`, `AbsenceFilterBar`, `StatsGrid`, `DeleteConfirmDialog`
  (`shifty-dioxus/src/page/absences.rs:5–52`).
- **Warnings:** Werden als nicht-blockierende Liste unter dem Modal
  gerendert — passt zum User-Wunsch "Inline-Warnungen statt Bestätigungs-
  Dialog" (Memory `feedback_warnings_inline_not_dialog.md`).
- **i18n-Keys:** Kategorie-Labels (`vacation`/`sickleave`/`unpaidleave`),
  Warning-Texte, Deprecation-Hinweis für Legacy-Marker — jeweils in `De`,
  `En`, `Cs`.
- **Proxy:** `shifty-dioxus/Dioxus.toml:98` mappt
  `/absence-period` → `http://localhost:3000/absence-period`. Der Convert-
  Endpoint läuft mit über den bestehenden `/extra-hours`-Proxy (F04). Ohne
  diesen Eintrag würde `dx serve` 404 zurückgeben (Memory
  `feedback_dioxus_proxy_for_new_backend_endpoints.md`).
- **Referenz-Doku:** Der ausführliche Integrations-Brief liegt in
  `docs/absence-feature-frontend.md` (v1.0 Frontend-Migration).

## 7. Randfälle

Für die zentrale Randfall-Referenz siehe
[`../domain/edge-cases.md`](../domain/edge-cases.md), Sektion
[§2 Absence & Extra Hours](../domain/edge-cases.md#2-absence--extra-hours).

- **Range spannt Billing-Period-Grenze:** Reporting fragt pro Billing-
  Period seinen Range gegen `derive_hours_for_range` — die Map deckt nur
  die abgefragten Tage ab. Beide Perioden bekommen ihren Anteil, keine
  Doppelzählung. **[Zu prüfen]** ob der Aggregator `billing_period_report`
  die Absence pro Billing-Range clippt (siehe
  `docs/domain/edge-cases.md#22-range-randfälle`).
- **Range spannt Jahreswechsel (Carryover):** Der Anteil vor dem 31.12.
  muss in den Carryover einfließen. Wenn Carryover VOR dem Absence-Insert
  gerechnet wurde, fehlt der Anteil. In der Praxis passiert Carryover
  jährlich einmalig; nachträgliche Absence-Änderungen im Vorjahr müssen
  einen Carryover-Refresh triggern
  ([`edge-cases.md#22`](../domain/edge-cases.md#2-absence--extra-hours)).
- **Overlap zwei Absences derselben Person:** Same-category ist verboten
  (422). Cross-category ist erlaubt und wird per Priorität `SickLeave >
  Vacation > UnpaidLeave` aufgelöst
  (`service_impl/src/absence.rs:65–74, 507–512`).
- **Absence vs Booking im gleichen Range:** Erzeugt einen nicht-
  blockierenden `Warning::AbsenceOverlapsBooking` beim Anlegen der
  Absence (Forward-Warning, `service_impl/src/absence.rs:837–893`) bzw.
  einen `Warning::BookingOnAbsenceDay` beim Anlegen eines Bookings über
  `POST /shiftplan-edit/booking` (Reverse-Warning, siehe
  `docs/absence-feature-frontend.md:60–69`). **Kein Auto-Cleanup**
  (D-Phase3-16). Reporting muss aufpassen, dass Absence-Tag und Booking-
  Tag nicht doppelt gutgeschrieben werden (Reporting-Detail in F06).
- **Absence auf Feiertag:** In `derive_hours_for_range` wird der Tag
  komplett übersprungen (kein Map-Eintrag,
  `service_impl/src/absence.rs:501–503, 437–458`). Der Feiertag bekommt
  seine eigene Gutschrift über den Holiday-Auto-Credit-Pfad (HCFG-02).
- **Absence auf Nicht-Arbeitstag:** Der Vertrag definiert
  `has_day_of_week(weekday)` und `workdays_per_week`. Fällt ein Range-Tag
  auf einen Wochentag, an dem die Person nicht arbeitet, entsteht kein
  Map-Eintrag (`service_impl/src/absence.rs:498–500`).
- **Vertragsänderung während der Range:** Wechselt der Vertrag mitten in
  einer Absence, wird pro Tag der am jeweiligen Tag aktive Vertrag
  gewählt (`service_impl/src/absence.rs:480–493`). Alte Perioden bleiben
  damit prospektiv unverändert (siehe
  `docs/absence-feature-frontend.md:22`).
- **Wochen-Deckelung > Verfügbarkeit:** `workdays_per_week` (z. B. 2)
  begrenzt die Wochensumme, auch wenn `has_day_of_week` an mehr Tagen
  `true` liefert (`service_impl/src/absence.rs:462–472, 523–533`).
  Regression-Test: `service_impl/src/test/absence_derive_hours_range.rs`
  Zeilen 701–815.
- **Halbtag an der Deckelungsgrenze:** Der Halbtag-Anteil (0.5) wird
  zusätzlich auf `remaining` gedeckelt
  (`service_impl/src/absence.rs:538–543`), damit Wochen mit teilweise
  ausgeschöpftem Kontingent nicht überzählen.
- **Update ändert `sales_person_id`:** Wird abgelehnt mit
  `ValidationError(ModificationNotAllowed)`
  (`service_impl/src/absence.rs:265–269`).
- **Convert einer Legacy-`extra_hours`-Row auf bereits belegtem Range:**
  Der Overlap-Check in `absence_conversion.rs:112–126` verhindert das mit
  `ValidationError(OverlappingPeriod)`.
- **Soft-Delete-Race:** Zwei parallele PUT auf dieselbe Row → einer sieht
  Stale-Version → `409 EntityConflicts`
  (`service_impl/src/absence.rs:270–276`).

## 8. Tests

- **Unit / Mock-Tests Absence:** `service_impl/src/test/absence.rs`
  (1323 Zeilen). Deckt Create-Happy-Path, Range-Inversion, Self-Overlap
  gleich- und verschieden-kategorial, ID/Version-Preset-Guards, Update-
  Happy-Path (Tombstone+Insert), Overlap-Exkludierung mit `Some(logical_id)`,
  Unknown-ID, Sales-Person-ID-Immutability, Stale-Version, Delete,
  Find-by-ID/Sales-Person/All, Permission-Verletzungen. Ab Zeile 864
  Forward-Warning-Tests (Booking, ManualUnavailable). Ab Zeile 1097
  Convert-Range-Vorschläge (Suggest UV-01/UV-02, Halbtag, Wochenende,
  Feiertag).
- **Unit-Tests Derive-Hours:** `service_impl/src/test/absence_derive_hours_range.rs`
  (1178 Zeilen). Deckt Basisfall, Feiertag=0, Vertragsänderung,
  Halbtag-Varianten (Full-Day-Vertrag, 2-Tages-Range, SickLeave), Lump-
  Sum-Range mit Feiertag, Overcounting-Regression und User-Szenarien
  ("Mo–Mi bei 2 Workdays cap = 2 Tage / 10 h").
- **Unit-Tests Conversion:** `service_impl/src/test/absence_conversion.rs`
  (649 Zeilen). Happy-Path, physische-ID-Soft-Delete (CR-01-Regression),
  Range-Inversion, Overlap-Reject, HR-Gate, plus Integrations-Test gegen
  in-memory SQLite ab Zeile 428.
- **REST-Tests:** Pure Summier-Logik `derived_days_from_map` inline in
  `rest/src/absence.rs:548–643` (Wochen-Deckelung, Halbtag, Out-of-Range,
  fehlende Map). Snapshot-Locking der OpenAPI-Surface läuft über die
  `insta`-Snapshots seit Phase 4 (siehe
  `docs/absence-feature-frontend.md:167`).
- **Frontend-Tests:** Snapshot-Tests im `#[cfg(test)]`-Block am Ende von
  `shifty-dioxus/src/page/absences.rs` (Plan-05 Task 3).
- **DAO-Round-Trip:** `dao_impl_sqlite/src/absence.rs:28–68` mappt
  `AbsencePeriodDb → AbsencePeriodEntity` und wird via SQLx-Prepare +
  In-Memory-Pool im Integration-Test (Conversion-Impl) mitgeprüft.

**Bekannte Lücken:**

- Explizite End-to-End-Absicherung für "Range spannt Jahreswechsel →
  Carryover-Refresh" ist im Absence-Cluster **nicht** abgedeckt (Carryover
  hat seine eigenen Tests, aber kein gemeinsamer Test triggert nach einer
  Absence-Änderung im Vorjahr eine Carryover-Neuberechnung). **[Zu prüfen]**
- Cross-Konfigurations-Test "Booking + Absence am gleichen Tag → Reporting
  zählt einmal" liegt im Reporting-Cluster (F06).

## 9. Historie & Kontext

- **v1.0 (2026-05-03) — Phasen 1–4:** Grüne Wiese. Neuer Range-Aggregat,
  neuer REST-Layer, Reporting-Integration mit Snapshot-Bump 2→3, Cross-
  Source-Warnings, Cutover-Service mit Heuristik-Migration + Drift-Gate.
  Frontend-Umstellung auf `/absence-period` (siehe
  `docs/absence-feature-frontend.md:1–8, 32–43`).
- **Cutover-Historie zu Legacy-`extra_hours` (F04):** Vor dem Cutover
  schrieben Frontends weiter auf `extra_hours` mit Kategorien
  `Vacation`/`SickLeave`/`UnpaidLeave`. Der einmalige `CutoverService`
  migrierte lebende Rows heuristisch in Ranges (Drift-Gate < 0.01 h pro
  `(sales_person, kategorie, jahr)`). Nach dem Flip liefert `POST
  /extra-hours` für diese drei Kategorien `403
  ExtraHoursCategoryDeprecatedErrorTO` (`rest/src/lib.rs:284–295`). Die
  Convert-per-Row-Variante lebt weiter im
  `AbsenceConversionService` (Phase 8.5, `service_impl/src/absence_conversion.rs:1–9`).
- **Phase 8.3 (Halbtag-Support):** Additive Migration
  `20260517120000_add-day-fraction-to-absence-period.sql`. Default `full`
  garantiert No-Drift für Bestandsdaten (Kommentar
  `dao/src/absence.rs:23–33`).
- **Phase 8.5/8.6 (Convert-Service-Extraktion):** Cutover-Maschinerie in
  einen schlanken BL-Tier-Service `AbsenceConversionService` extrahiert;
  der historische `CutoverService` konnte damit gelöscht werden ohne den
  neuen Service anzupassen (`service_impl/src/absence_conversion.rs:1–9`).
- **Toggle-Rollout D-51-07 (Phase 51, ShortDay-Slot-Kürzung) und
  HCFG-02 (v1.7, `holiday_auto_credit`):** Zwei nachfolgende Stichtag-
  Toggles, die den Reporting-Aggregator gegen historische Perioden
  absichern. Semantik (siehe Memory
  `feedback_stichtag_rollout_legacy_semantics.md`): **pro Konsumkette wird
  im Gate-aus-Zweig die alte Semantik vor Feature-Einführung
  rekonstruiert** — nicht "None → raw" annehmen. Für Absences relevant,
  weil `derive_hours_for_range` in Reporting-Aggregaten (Chain C
  BookingInformation, Chain D ShiftplanReport) mitläuft: der Toggle
  entscheidet nicht über die Absence-Range selbst, aber über wie die
  Absence-Tage mit ShortDay-Slot-Kürzung interagieren
  (`service_impl/src/shortday_gate.rs:1–71`). Der frühere Seed
  `absence_range_source_active` wurde in Migration
  `20260611000002_delete-absence-range-source-active-seed.sql` entfernt —
  seit M-03 ist Reporting additiv aus beiden Quellen ohne Quellen-
  Schalter (`service_impl/src/test/reporting_additive_merge.rs:5`).
- **Planning-Artefakte:** `.planning/milestones/v1.0-ROADMAP.md`,
  `.planning/phases/01-absence-domain-foundation/`,
  `.planning/phases/02-.../`, `.planning/phases/03-.../` und
  `.planning/phases/04-.../` (Deep-Kontext-Reads für D-Decisions).
- **ToggleService Full-Context-Bypass** (Memory
  `reference_toggle_service_full_context_bypass.md`): Interne Aggregate,
  die Absences konsumieren, rufen den Toggle mit `Authentication::Full` —
  seit Phase 51 Gap-Closure werden diese als all-rights-Bypass behandelt,
  damit die Reporting-Pipeline nicht an fehlenden HR-Rechten scheitert.

---

**Fazit:** Absences sind die Range-basierte Nachfolge-Aggregation für
Urlaub / Krank / Unbezahlt und ersetzen die Legacy-Tages-Postings aus
`extra_hours`. Die Stunden werden **nicht persistiert**, sondern zur
Reporting-Zeit über `derive_hours_for_range` aus dem am jeweiligen Tag
gültigen Vertrag abgeleitet — mit Wochen-Deckelung, Priorität `SickLeave >
Vacation > UnpaidLeave`, Feiertag=0 und HR ∨ self-Gate.

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
