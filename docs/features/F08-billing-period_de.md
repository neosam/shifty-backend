# Feature: Billing Period & Snapshot-Versioning

> **Kurzform:** Eine Abrechnungsperiode friert für einen definierten Zeitraum
> pro Mitarbeiter Balance, Erwartung, geleistete Stunden, Urlaubs- und
> Krankheitswerte als **Snapshot** in der DB ein — versehen mit einer
> Schema-Version, damit spätere Formeländerungen Alt-Snapshots nicht
> stillschweigend "brechen".

**Cluster-ID:** F08
**Status:** produktiv (aktive Snapshot-Schema-Version **12**, Stand 2026-07)
**Erstmalig eingeführt:** 2025-08 (Migration `20250813051848_add-table-billing-period.sql`);
Versionierungs-Kolumne 2026-04 (`20260426000000_add-snapshot-schema-version-to-billing-period.sql`)
**Zuständige Crates:**
- `service::billing_period`, `service::billing_period_report`
- `service_impl::billing_period`, `service_impl::billing_period_report`
- `dao::billing_period`, `dao::billing_period_sales_person`
- `dao_impl_sqlite::billing_period`, `dao_impl_sqlite::billing_period_sales_person`
- `rest::billing_period`
- Frontend: `shifty-dioxus/src/page/billing_periods.rs`, `.../billing_period_details.rs`, `.../service/billing_period.rs`

---

## 1. Was ist das? (Fachlich)

Eine **Abrechnungsperiode** (Billing Period) ist ein zusammenhängender
Zeitraum — typischerweise ein Monat, Quartal oder Halbjahr —, an dessen
Ende die HR-Abteilung für **jeden bezahlten Mitarbeiter** eine Momentaufnahme
seiner Zeit-Kennzahlen einfriert:

- **Balance** (Stundenkonto) zum Periodenende,
- **Expected Hours** (Soll) im Zeitraum,
- **Overall Hours** (Gesamt geleistet),
- **Extra Work** (Überstunden-Kategorie),
- **Vacation Hours / Sick Leave / Unpaid Leave / Holiday / Volunteer**,
- **Vacation Days** (genommen) und **Vacation Entitlement** (Anspruch),
- optional beliebige **CustomExtraHours** je nach konfigurierten Kategorien.

Jede dieser Kennzahlen wird in **vier Sichten** gespeichert (siehe §3):
`value_delta` (nur Periode), `value_ytd_from` (YTD bis Periodenanfang),
`value_ytd_to` (YTD bis Periodenende) und `value_full_year` (Vollkalenderjahr).

Die HR sieht damit auf einen Blick, was ein Mitarbeiter im Abrechnungsmonat
erarbeitet hat **und** wie er im Jahresverlauf steht — auch dann noch, wenn
Monate später Buchungen rückwirkend angepasst werden.

**Beispiel-Workflow aus HR-Sicht:**

1. HR öffnet die Seite **Abrechnungsperioden**
   (`shifty-dioxus/src/page/billing_periods.rs`).
2. Klick auf „Neue Abrechnungsperiode anlegen", End-Datum wählen
   (z.B. `2026-06-30`).
3. Backend rechnet für **jeden bezahlten** Mitarbeiter die Kennzahlen
   für den Zeitraum `[letzte Periode + 1 Tag … End-Datum]` und persistiert
   pro Kennzahl **eine Zeile** in `billing_period_sales_person`.
4. Die persistierte `billing_period`-Zeile trägt eine
   `snapshot_schema_version` — die aktuelle Version der Rechenregeln.
5. Später öffnet HR die Detail-Seite
   (`shifty-dioxus/src/page/billing_period_details.rs`) und sieht die
   damals eingefrorenen Werte — auch wenn Buchungen im Zeitraum inzwischen
   geändert wurden.
6. Optional generiert HR aus einem `text_template` einen Custom-Report
   (Tera oder MiniJinja) über die Snapshot-Daten.

## 2. Fachliche Regeln

- **Regel — Write-once:** Ein Snapshot wird **einmal** geschrieben
  (`create_billing_period` → `insert_billing_period_sales_person` in
  `service_impl/src/billing_period.rs:181-190`). Es gibt keinen
  „Update-Snapshot"-Pfad. Wer den Snapshot ändern will, löscht ihn und
  erzeugt einen neuen — und das **nur** für die letzte Periode.
- **Regel — Nur die letzte Periode ist löschbar:** `delete_billing_period`
  wirft `ServiceError::NotLatestBillingPeriod` wenn `id` nicht die
  aktuellste ist (`billing_period.rs:242-246`). Damit bleibt die zeitliche
  Kette lückenfrei.
- **Regel — Nur HR darf schreiben/löschen:** `delete_billing_period` und
  `clear_all_billing_periods` prüfen `HR_PRIVILEGE`
  (`billing_period.rs:226,275`). `generate_custom_report` ebenso
  (`billing_period_report.rs:437-439`).
  **[Zu prüfen]** `create_billing_period` und der öffnende
  REST-Handler `POST /billing-period` haben **keinen expliziten
  Permission-Check** vor dem Aufruf — der Check hängt aktuell an der
  `HR_PRIVILEGE`-Prüfung, die tiefer in `ReportingService` /
  `EmployeeWorkDetailsService` beim Lesen greift. Ein direkter Gate am
  Entry-Point wäre robuster.
- **Regel — Nur bezahlte Personen im Snapshot:**
  `build_new_billing_period` filtert `!sales_person.is_paid.unwrap_or(false)`
  (`billing_period_report.rs:371-373`). Freiwillige Helfer erscheinen
  nicht in der Abrechnung. Kommentar dort erklärt, dass dies **kein
  Schema-Version-Bump** ist (Personen-Set-Änderung, kein
  `value_type`-Change).
- **Regel — Perioden sind lückenlos aneinander gereiht:** Start-Datum
  einer neuen Periode = `letzte_periode.end_date.next_day()`
  (`billing_period_report.rs:349-356`). Erste Periode startet mit
  UNIX-Epoch-Tag `1970-01-01` (der Kommentar in `service/…/billing_period_report.rs:23`
  spricht von `2020-01-01`; **[Zu prüfen]** — der Code nutzt UNIX-Epoch).
- **Regel — End-Datum muss nach der letzten Periode liegen:**
  Der Doc-Comment am Trait
  (`service/src/billing_period_report.rs:22-24`) verspricht einen Fehler,
  wenn `end_date < letztes_end_date` — der Code selbst hat aber keinen
  expliziten Guard. Kommt das End-Datum unrealistisch daher, produziert
  `next_day()` einen Start nach dem End-Datum → leerer bzw. negativer
  Zeitraum. **[Zu prüfen]** Ob ein Guard fehlt oder ob
  `ShiftyDate`/Reporting das anderswo abfangen.
- **Invariante — Write-Version = Read-Version:** Die
  `snapshot_schema_version`, mit der eine Zeile geschrieben wurde,
  muss beim späteren Interpretieren geprüft werden. Details in §7.
- **Invariante — Enum-Vollständigkeit:** Jeder Arm von
  `BillingPeriodValueType` muss durch `as_str()` / `from_str()`
  round-trippen (`service/src/billing_period.rs:52-97`).
  Ein Locking-Test (`test_billing_period_value_type_surface_locked`,
  `service_impl/src/test/billing_period_snapshot_locking.rs:44-70`)
  erzwingt beim Compile, dass jede neue Variante bewusst behandelt wird.

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `billing_period` | Kopfzeile pro Abrechnungsperiode | `id`, `from_date_time`, `to_date_time`, **`snapshot_schema_version`**, `created`, `created_by`, `deleted`, `deleted_by`, `update_version`, `update_process` |
| `billing_period_sales_person` | Eine Zeile pro (Periode × Person × `value_type`) | `id`, `billing_period_id`, `sales_person_id`, **`value_type`**, `value_delta`, `value_ytd_from`, `value_ytd_to`, `value_full_year`, `created_at`, `deleted_at`, `update_version` |

`billing_period_sales_person` hat den Unique-Index
`(billing_period_id, sales_person_id, value_type)`
(`20250813051848_add-table-billing-period.sql:36`), damit dieselbe
Kennzahl nicht doppelt für dieselbe Person derselben Periode landet.

### Migrations

- `2025-08-13` **`20250813051848_add-table-billing-period.sql`** —
  Basistabellen `billing_period` + `billing_period_sales_person` mit
  FKs auf `sales_person(id)`.
- `2026-04-26` **`20260426000000_add-snapshot-schema-version-to-billing-period.sql`**
  — additive Kolumne `snapshot_schema_version INTEGER NOT NULL DEFAULT 1`.
  Bestehende Zeilen bekommen den Default `1`, sodass Validatoren sie als
  „ganz alter Snapshot, Semantik nicht garantiert" erkennen.

### `value_type`-Enum

Die textuelle Repräsentation in der DB-Spalte `value_type` (siehe
`service/src/billing_period.rs:52-97`):

| `value_type` (String) | Rust-Variante | Bedeutung |
| --- | --- | --- |
| `balance` | `Balance` | Stundenkonto (Ist − Soll + zählende Extras) |
| `overall` | `Overall` | Geleistete Gesamt-Stunden inkl. Extras |
| `expected_hours` | `ExpectedHours` | Vertragliches Soll im Zeitraum |
| `extra_work` | `ExtraWork` | Persistierte Extra-Arbeitsstunden |
| `vacation_hours` | `VacationHours` | Urlaubsstunden (extra_hours + absence_period-derived) |
| `sick_leave` | `SickLeave` | Krankheitsstunden |
| `unpaid_leave` | `UnpaidLeave` | Unbezahlter Urlaub (ab v3) |
| `holiday` | `Holiday` | Feiertagsstunden |
| `volunteer` | `Volunteer` | Ehrenamtliche Stunden (nur wenn ≠ 0) |
| `vacation_days` | `VacationDays` | Genommene Urlaubstage |
| `vacation_entitlement` | `VacationEntitlement` | Kalender-anteiliger Anspruch |
| `custom_extra_hours:<name>` | `CustomExtraHours(name)` | Freie Kategorien pro Betrieb |

### Beziehungen

```
billing_period (1) ────────< (N) billing_period_sales_person
    │                                     │
    │                                     └── sales_person_id ──> sales_person(id)
    └── snapshot_schema_version (u32, Stempel zum Zeitpunkt des Writes)
```

Pro Person entstehen typischerweise **10–12 Zeilen** (eine pro persistiertem
`value_type`) — plus je eine Zeile pro `custom_extra_hours:<name>`.

## 4. Service-API

Die Cluster-Services folgen der **Basic vs. Business-Logic**-Konvention
(siehe `shifty-backend/CLAUDE.md`, Sektion „Service-Tier-Konventionen"):

- `BillingPeriodService` ist **Basic-Tier** — CRUD auf dem Aggregat, kein
  Konsum anderer Domain-Services (außer dem Read-only `SalesPersonService`
  fürs Person-Set beim Lesen).
- `BillingPeriodReportService` ist **Business-Logic-Tier** — orchestriert
  `ReportingService`, `EmployeeWorkDetailsService`, `SalesPersonService`
  und schreibt schließlich **via** `BillingPeriodService` in die DB.

### 4.1 `BillingPeriodService` (Basic)

Trait: `service::billing_period::BillingPeriodService`
(`service/src/billing_period.rs:219-270`).

```rust
#[async_trait]
pub trait BillingPeriodService {
    type Context: …;
    type Transaction: dao::Transaction;

    async fn get_billing_period_overview(
        &self, ctx: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<Arc<[BillingPeriod]>, ServiceError>;

    async fn get_billing_period_by_id(
        &self, id: Uuid, ctx: …, tx: …,
    ) -> Result<BillingPeriod, ServiceError>;

    async fn create_billing_period(
        &self, entity: &BillingPeriod, process: &str, ctx: …, tx: …,
    ) -> Result<BillingPeriod, ServiceError>;

    async fn get_latest_billing_period_end_date(
        &self, ctx: …, tx: …,
    ) -> Result<Option<ShiftyDate>, ServiceError>;

    async fn delete_billing_period(
        &self, id: Uuid, ctx: …, tx: …,
    ) -> Result<(), ServiceError>;

    async fn clear_all_billing_periods(
        &self, ctx: …, tx: …,
    ) -> Result<(), ServiceError>;
}
```

**Auth-Gates:**

| Methode | Permission | Ort |
| --- | --- | --- |
| `get_billing_period_overview` | keiner (ignoriert Context) | `billing_period.rs:92-104` |
| `get_billing_period_by_id` | ruft `SalesPersonService::get_all` — das trägt die Permission-Prüfung | `billing_period.rs:107-146` |
| `create_billing_period` | **kein direkter Check** — Aufrufer (`BillingPeriodReportService`) verlässt sich auf Downstream-Checks | `billing_period.rs:149-198` |
| `get_latest_billing_period_end_date` | keiner | `billing_period.rs:201-216` |
| `delete_billing_period` | `HR_PRIVILEGE` | `billing_period.rs:225-227` |
| `clear_all_billing_periods` | `HR_PRIVILEGE` | `billing_period.rs:274-276` |

**[Zu prüfen]** Ob `create_billing_period` einen expliziten
`HR_PRIVILEGE`-Guard braucht — aktuell nutzt es das Auth-Subject nur, um
`created_by` zu füllen.

**TX-Verhalten:**

- Alle Methoden öffnen bei `tx=None` selbst eine TX
  (`use_transaction(tx)`) und committen am Ende.
- `create_billing_period` schreibt zuerst den Header, dann in einer Schleife
  je Person je `value_type` eine Zeile — alles unter **einer** TX
  (`billing_period.rs:156-192`). Rollback bei Fehler = konsistenter Zustand.
- `delete_billing_period` cascadet: erst
  `billing_period_sales_person`-Zeilen, dann Header
  (`billing_period.rs:255-262`).

**Dependencies:**

- DAOs: `BillingPeriodDao`, `BillingPeriodSalesPersonDao`,
  `TransactionDao`.
- Services: `SalesPersonService` (Basic, für Person-Aufzählung beim
  Lesen), `PermissionService` (für `HR_PRIVILEGE`-Checks + `current_user_id`
  fürs Audit).
- Utility: `UuidService`, `ClockService`.

Konstruiert per `gen_service_impl!`-Makro
(`service_impl/src/billing_period.rs:26-36`).

### 4.2 `BillingPeriodReportService` (Business-Logic)

Trait: `service::billing_period_report::BillingPeriodReportService`
(`service/src/billing_period_report.rs:10-54`).

```rust
#[async_trait]
pub trait BillingPeriodReportService {
    type Context: …; type Transaction: dao::Transaction;

    async fn build_new_billing_period(
        &self, end_date: ShiftyDate, ctx: …, tx: …,
    ) -> Result<BillingPeriod, ServiceError>;

    async fn build_and_persist_billing_period_report(
        &self, end_date: ShiftyDate, ctx: …, tx: …,
    ) -> Result<Uuid, ServiceError>;

    async fn generate_custom_report(
        &self, template_id: Uuid, billing_period_id: Uuid, ctx: …, tx: …,
    ) -> Result<Arc<str>, ServiceError>;
}
```

**Kernpfad — `build_billing_period_report_for_sales_person`**
(`billing_period_report.rs:134-331`): pro Person **vier**
`ReportingService::get_report_for_employee_range`-Aufrufe:

1. **Report-Start**: `[Jahresanfang(start_date.year) … start_date-1]`
   → Basis für `value_ytd_from`.
2. **Report-End**: `[Jahresanfang(end_date.year) … end_date]`
   → Basis für `value_ytd_to`.
3. **Report-Full-Year**: `[Jahresanfang(end_date.year) … Jahresende]`
   → Basis für `value_full_year`.
4. **Report-Delta**: `[start_date … end_date]`, Flag `false`
   (vermutlich „nur Periode, kein Carryover"; **[Zu prüfen]**
   Semantik des Flags in `ReportingService::get_report_for_employee_range`)
   → Basis für `value_delta`.

Diese vier Zahlen werden pro `BillingPeriodValueType` in eine
`BillingPeriodValue` verwoben. `Volunteer` wird nur eingesetzt, wenn
`report_delta.volunteer_hours != 0.0` (`billing_period_report.rs:283`).
`CustomExtraHours` sind namens-basiert; die YTD-Werte werden per
`.find(|ch| ch.name == custom_hours.name)` aus den drei anderen Reports
zusammengesucht (`billing_period_report.rs:294-320`).

**`build_new_billing_period`** (`billing_period_report.rs:341-399`):

- Ermittelt `start_date` = `letzte_periode.end_date.next_day()` bzw.
  UNIX-Epoch, wenn keine Periode existiert.
- Iteriert `SalesPersonService::get_all`, filtert `is_paid == true`.
- Baut pro Person via `build_billing_period_report_for_sales_person`
  einen `BillingPeriodSalesPerson`.
- Erzeugt in-memory `BillingPeriod { id: Uuid::nil(), snapshot_schema_version: CURRENT_SNAPSHOT_SCHEMA_VERSION, … }`
  — noch **nicht** persistiert.

**`build_and_persist_billing_period_report`**
(`billing_period_report.rs:401-425`):

- Ruft `build_new_billing_period`.
- Ruft `BillingPeriodService::create_billing_period(&billing_period, "BillingPeriodReportService", ctx, tx)`.
- Committet TX.
- Gibt `Uuid::nil()` **[Achtung: Bug-Verdacht]** zurück —
  `billing_period_id` wird vor `create_billing_period` gelesen, das den
  neuen `Uuid` erst dort erzeugt. **[Zu prüfen]** ob der zurückgegebene
  UUID der echte oder `Uuid::nil()` ist — Line `billing_period_report.rs:412`
  liest `billing_period.id`, welches in `build_new_billing_period` als
  `Uuid::nil()` gesetzt wird (`billing_period_report.rs:387`). Der REST-
  Handler `create_billing_period` serialisiert das aber in den Response-Body
  (`rest/src/billing_period.rs:124-133`), das FE ignoriert es und lädt die
  Liste neu (`shifty-dioxus/src/service/billing_period.rs:60-62`), daher
  bricht das den User-Flow nicht.

**`generate_custom_report`** (`billing_period_report.rs:427-550`):

- Prüft `HR_PRIVILEGE` explizit.
- Lädt `TextTemplate` + `BillingPeriod`.
- Reichert die Snapshot-Daten mit `sales_person.name` und
  `is_paid`/`is_dynamic` aus `EmployeeWorkDetailsService::all` an.
- Rendert je nach `TemplateEngine`:
  - **Tera**: `Tera::default().add_raw_template().render()`.
  - **MiniJinja**: `minijinja::Environment::new().render_str()`.
- NaN/Inf werden vorher zu `0.0` sanitisiert (`billing_period_report.rs:478-481`).

**Dependencies:**

- Services: `BillingPeriodService` (schreibt via ihm), `ReportingService`
  (rechnet die vier Report-Sichten), `SalesPersonService`,
  `EmployeeWorkDetailsService`, `TextTemplateService`, `PermissionService`.
- Utility: `UuidService`, `ClockService`, `TransactionDao`.

**Auth-Gates:**

| Methode | Permission | Anmerkung |
| --- | --- | --- |
| `build_new_billing_period` | indirekt via `ReportingService` / `SalesPersonService` | Kein direkter Check am Entry-Point. |
| `build_and_persist_billing_period_report` | indirekt (wie oben) | Der schreibende Aufruf würde vom REST-Layer geöffnet, ohne Gate. **[Zu prüfen]** ob HR-Gate ergänzt werden sollte. |
| `generate_custom_report` | `HR_PRIVILEGE` | `billing_period_report.rs:437-439` |

## 5. REST-Endpoints

Alle unter Prefix `/billing-period` (siehe `rest/src/lib.rs:642`).
Handler in `rest/src/billing_period.rs`.

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/billing-period` | Liste (nur Header, keine `sales_persons`) | — | `Vec<BillingPeriodTO>` | 401, 500 |
| `GET` | `/billing-period/{id}` | Detail mit allen `BillingPeriodSalesPersonTO` | — | `BillingPeriodTO` | 401, 404 |
| `POST` | `/billing-period` | Neue Periode anlegen (baut Snapshot) | `CreateBillingPeriodRequestTO { end_date }` | `Uuid` | 400, 401, 403, 500 |
| `DELETE` | `/billing-period` | **Alle** Perioden soft-löschen (Reset) | — | 204 | 401, 403 |
| `DELETE` | `/billing-period/{id}` | Einzelne Periode löschen (nur wenn letzte) | — | 204 | 403, 404, 409 (`NotLatestBillingPeriod`) |
| `POST` | `/billing-period/{id}/custom-report/{template_id}` | Text-Report rendern | — | `String` (`text/plain`) | 401, 403, 404, 500 |

DTOs siehe `rest-types::lib.rs:1401-1494`:

- `BillingPeriodTO` — enthält `snapshot_schema_version: u32`
  (`rest-types/src/lib.rs:1460`), Frontend sieht die Version.
- `BillingPeriodSalesPersonTO` — `values: BTreeMap<String, BillingPeriodValueTO>`
  mit dem `value_type`-String als Key.
- `BillingPeriodValueTO` — flach: `value_delta`, `value_ytd_from`,
  `value_ytd_to`, `value_full_year`.
- `CreateBillingPeriodRequestTO { end_date: time::Date }`.

**OpenAPI:** `BillingPeriodApiDoc` (`rest/src/billing_period.rs:250-267`)
sammelt alle Handler unter Tag `billing_period`.

## 6. Frontend-Integration

- **Pages:**
  - `shifty-dioxus/src/page/billing_periods.rs` — Übersicht, Create-Dialog,
    Delete-mit-Confirm (siehe MEMORY-Feedback „Warnungen inline statt
    Dialog" — hier wird ein Confirm-Dialog explizit genutzt weil
    Löschung endgültig ist).
  - `shifty-dioxus/src/page/billing_period_details.rs` — Detail-Ansicht
    mit Filter (`show_paid`, `show_active`, `filter_text`), sortierte
    Werte-Tabelle, Custom-Report-Selection.
- **Service:** `shifty-dioxus/src/service/billing_period.rs` — Coroutine
  konsumiert `BillingPeriodAction::{LoadBillingPeriods, LoadBillingPeriod, CreateBillingPeriod, DeleteBillingPeriod}`,
  hält Store `BILLING_PERIOD_STORE { billing_periods, selected_billing_period }`.
- **API:** `shifty-dioxus/src/api.rs` mit `get_billing_periods`,
  `get_billing_period`, `post_billing_period`, `delete_billing_period`,
  `generate_custom_report`.
- **i18n-Keys** (`shifty-dioxus/src/i18n/mod.rs:257-379`):
  `BillingPeriods`, `BillingPeriodDetails`, `CreateNewBillingPeriod`,
  `BillingPeriod`, `LoadingBillingPeriods`, `LoadingBillingPeriodDetails`,
  `CreateBillingPeriod`, `NoSalesPersonsInBillingPeriod`,
  `InvalidBillingPeriodId`, `SelectEndDateForNewBillingPeriod`,
  `DeleteBillingPeriod`, `ConfirmDeleteBillingPeriod`,
  `DeleteBillingPeriodError` — jeweils in `en.rs` / `de.rs` / `cs.rs`
  vorhanden.
- **Proxy:** `shifty-dioxus/Dioxus.toml:46`
  `backend = "http://localhost:3000/billing-period"` — ohne diesen Eintrag
  gibt der dx-serve-Dev-Server 404 (siehe MEMORY-Feedback „Dioxus.toml
  Proxy für neue Backend-Endpoints").

## 7. Snapshot-Versioning — Der harte Kern

Diese Sektion ist die **eigentliche Existenzberechtigung** des Clusters
und die zentrale Referenz für alle, die die Berechnung ändern.

### 7.1 Der Vertrag

**Feld:** `billing_period.snapshot_schema_version INTEGER NOT NULL DEFAULT 1`
(Migration `20260426000000_add-snapshot-schema-version-to-billing-period.sql`).
Round-trip als `u32` durch `BillingPeriodEntity.snapshot_schema_version`
(`dao/src/billing_period.rs:11`) und `BillingPeriod.snapshot_schema_version`
(`service/src/billing_period.rs:23`) bis in `BillingPeriodTO`
(`rest-types/src/lib.rs:1460`).

**Konstante (Single Source of Truth):**

```rust
// service_impl/src/billing_period_report.rs:117
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;
```

**Writer:** `build_new_billing_period` stempelt die Version auf jeden
frisch gebauten Snapshot:

```rust
// service_impl/src/billing_period_report.rs:386-396
let billing_period = BillingPeriod {
    id: Uuid::nil(),
    start_date,
    end_date,
    snapshot_schema_version: CURRENT_SNAPSHOT_SCHEMA_VERSION,   // <── Zeile 390
    sales_persons: sales_person_reports.into(),
    // …
};
```

Von dort persistiert `BillingPeriodService::create_billing_period` den Wert
in die DB (`service_impl/src/billing_period.rs:169`,
`dao_impl_sqlite/src/billing_period.rs`).

### 7.2 Bump-Regeln (verbindlich)

**Bumpe `CURRENT_SNAPSHOT_SCHEMA_VERSION` um genau 1, wenn du:**

1. **einen neuen persistierten `value_type` hinzufügst.**
   Beispiel: v3 (`UnpaidLeave` als 12. Enum-Variante) —
   `billing_period_report.rs:41-48`.
2. **einen bestehenden `value_type` entfernst oder umbenennst.**
   (Historisch bisher nie passiert — die Enum-Historie ist rein additiv.)
3. **die Berechnung eines existierenden `value_type` änderst** — andere
   Formel, andere Inputs, anderes Filtering.
   Beispiel: v4 (`day_fraction::Half` halbiert Soll-Stundenzahl in
   `derive_hours_for_range`, betrifft `VacationHours`/`SickLeave`/
   `UnpaidLeave` + transitiv `Balance`/`ExpectedHours` — Doc-Comment
   `billing_period_report.rs:46-49`).
4. **den Input-Set änderst**, den die Berechnung liest.
   Beispiel: v5 („additiver Merge" — Vacation/SickLeave/UnpaidLeave lesen
   ab jetzt **beide** Quellen: lebende `extra_hours` **plus**
   `absence_period`-derived; Doc-Comment `billing_period_report.rs:50-55`).

### 7.3 Bumpe **NICHT** wenn du:

- neue REST-Endpoints hinzufügst, die nur lesen,
- Frontend-Ansichten änderst,
- neue Felder auf **anderen** Tabellen ergänzt (die kein Snapshot
  produzieren),
- den Writer refaktorierst, ohne dass der berechnete Output pro
  `value_type` sich ändert (Verifizierung: alle bestehenden Tests grün +
  ein Diff-Vergleich Alt/Neu für dieselben Inputs = `0.0`),
- eine **Personen-Set-Änderung** machst (z.B. den `is_paid`-Filter in
  `build_new_billing_period` einbaust). Der Kommentar
  `billing_period_report.rs:365-370` spricht das explizit aus:
  *„KEIN value_type-Change → KEIN CURRENT_SNAPSHOT_SCHEMA_VERSION-Bump."*

### 7.4 Historie der Bumps (verifiziert im Code)

Aus dem großen Doc-Comment `billing_period_report.rs:38-117`:

| Version | Anlass | Betroffene `value_type`s |
| --- | --- | --- |
| v1 | Baseline (initiales Snapshot-Modell) | — |
| v2 | Zwischen-Bump (Details in Historie verschwunden) | — |
| **v3** | Phase 2 — neuer `value_type` `UnpaidLeave` + AbsencePeriod-derived Vacation/Sick/Unpaid | `UnpaidLeave`, `VacationHours`, `SickLeave` |
| **v4** | Phase 8.3 — `day_fraction::Half` halbiert Soll pro Tag | Vacation/Sick/Unpaid (hours + days) |
| **v5** | Phase 8.4 — additiver Merge: extra_hours + absence_period statt Flag-Branch | Vacation/Sick/Unpaid |
| **v6** | Phase 8.4 Gap 2 (WR-01) — absence_period-derived Kategorien reduzieren symmetrisch Balance/ExpectedHours | `Balance`, `ExpectedHours` (+ transitiv) |
| **v7** | Bugfix vacation-hours-overcounted — Wochen-Deckelung `workdays_per_week` | `Vacation*`, `Balance`, `ExpectedHours`. Nie deployed. |
| **v8** | Bugfix report-ehrenamt-gesamtstunden — Cap-Überlauf leakte in `overall/balance`; jetzt Wochen-gedeckelt | `Overall`, `Balance`, `ExpectedHours` |
| **v9** | quick-260624-ujk — Shiftplan-Stunden ohne Vertragszeile werden als `volunteer` gezählt statt neutralisiert | `Volunteer`, transitiv `Balance`/`ExpectedHours` |
| **v10** | UV-05 / D-18-07 — konvertierte hours-based Absenzen fließen in per-week Category-Felder | `VacationDays` (+ Sick/Unpaid days) |
| **v11** | Phase 25 (HOL-01/02, HCFG-01) — derive-on-read Feiertags-Auto-Credit via Toggle | `Holiday`, transitiv `Balance`/`ExpectedHours` |
| **v12** | Phase 28 (VAC-OFFSET-01 / D-28-05) — off-by-one Fix in `vacation_days_for_year`; 1.1.-Start zieht 0 Tage ab statt ~1/365 | `VacationEntitlement` (**nicht** `VacationDays`) |

Phase 15 (committed_voluntary Zwei-Band) wurde explizit **nicht** gebumpt,
weil Achse-B-only, kein persistierter `value_type` betroffen
(`billing_period_report.rs:74`).

Phase 17 (Personen-Set-Filter `is_paid`) wurde explizit **nicht** gebumpt
(`billing_period_report.rs:365-370`).

**Milestone v2.6 Phase 54 — Non-Bump-Bestätigung.**
`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **12**. Rationale: Phase 54
(Voluntary-Stats-Datenmodell, siehe Feature [F14](./F14-rebooking.md))
ergänzt nur die Marker-Spalte `extra_hours.source` (Werte: `manual` \|
`rebooking`) und zwei neue Tabellen `rebooking_batch` /
`rebooking_batch_entry` — weder wird ein neuer persistierter
`BillingPeriodValueType` eingeführt noch eine bestehende Berechnung
verändert. Voluntary-Stats selbst ist eine **live berechnete HR-only-
Read-View**, kein persistierter Snapshot: keine
`billing_period_sales_person`-Zeile, kein Versioning, kein Writer
fasst `billing_period_report.rs` an. Die Snapshot-Bump-Entscheidung
**12 → 13** ist auf Phase 56 verschoben (`REB-AUTO-05`, F4-Cron),
sobald der erste `Rebooking`-Source-Writer die Balance-Kette bespielt
und Reader-Filter (`source = 'manual'`) semantisch tragend werden —
siehe `REQUIREMENTS.md`.

### 7.5 Randfall — Validator liest v11-Snapshot mit v12-Code

Konkreter Fall aus dem v12-Doc-Comment (`billing_period_report.rs:108-116`):

- Alter Snapshot: `snapshot_schema_version = 11`. In seinen
  `VacationEntitlement`-Zeilen steht der Wert, den die alte Formel (mit
  Off-by-One) für den 1.1.-Vertragsstart geliefert hat.
- Aktueller Code: `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12`. Für denselben
  Vertrag liefert die neue Formel einen minimal höheren Wert (~1/365
  weniger abgezogen).
- **Ein naiver Re-Compute-Validator** würde eine Abweichung sehen und
  „Datenbug!" schreien.
- **Korrektes Validator-Verhalten** (per Kommentar-Regel):
  1. Lies `bp.snapshot_schema_version`.
  2. Falls `< CURRENT_SNAPSHOT_SCHEMA_VERSION` → markiere als „older
     schema" und **skippe** die betroffene Re-Validierung (hier:
     `VacationEntitlement`).
  3. `VacationDays` **darf** trotzdem re-validiert werden, weil vom v12-
     Change nicht berührt.

Analog gilt: v10 → v11 skippt Holiday-Hours (`billing_period_report.rs:101-107`);
v9 → v10 skippt Vacation/Sick/Unpaid Days
(`billing_period_report.rs:93-100`); usw.

**[Zu prüfen]** In welcher Datei die Validator-Logik tatsächlich lebt.
Der Doc-Comment referenziert sie, aber der aktuelle Cluster enthält
noch keine sichtbare Validator-Routine. Möglicherweise Teil einer
späteren Phase (SNAP-02+).

### 7.6 Bezug zur zentralen Randfall-Doku

Siehe [`../domain/edge-cases.md#3-billing-period--snapshots`](../domain/edge-cases.md#3-billing-period--snapshots)
für weitere Rand-Szenarien:

- Race zwischen Booking-Änderung und Snapshot-Erzeugung
  (TX-Konsistenz von `build_and_persist_billing_period_report`).
- Feature-Toggle ändert Semantik: Toggle-Change **muss** Version-Bump
  auslösen (siehe §9 der edge-cases).
- Report ohne Snapshot: Live-Rechnung greift — UI zeigt aktuell nicht
  kenntlich, dass kein eingefrorener Wert vorliegt
  (**[Zu prüfen]** DTO-Feld `is_snapshot`).

### 7.7 Gate-Test (Locking)

`service_impl/src/test/billing_period_snapshot_locking.rs` schützt vor
stiller Drift:

```rust
// Zeile 27-38
#[test]
fn test_snapshot_schema_version_pinned() {
    assert_eq!(
        CURRENT_SNAPSHOT_SCHEMA_VERSION, 12,
        "CURRENT_SNAPSHOT_SCHEMA_VERSION muss 12 sein nach Phase 28 …"
    );
}
```

Wer die Konstante bumpt, **muss** auch diesen Assert mitheben und die
Message auf den neuen Grund umschreiben. Der zweite Test
`test_billing_period_value_type_surface_locked` (`:44-70`) ist ein
`match`-Locking auf `BillingPeriodValueType`: eine neue Enum-Variante
gibt `non-exhaustive patterns` und zwingt den Autor, bewusst zu
entscheiden, ob ein Bump nötig ist.

## 8. Tests

- **Unit — Service-Basic:**
  `service_impl/src/test/billing_period.rs` (427 LoC). Mockt DAOs +
  Downstream-Services (`MockDeps`), deckt CRUD-Wege inkl.
  `NotLatestBillingPeriod`-Guard, HR-Gate auf Delete/ClearAll,
  Cascade-Delete `billing_period_sales_person` → `billing_period`.
- **Unit — Report-Business-Logic:**
  `service_impl/src/test/billing_period_report.rs` (1368 LoC).
  Deckt `build_new_billing_period` (Person-Filter, Perioden-Ketten),
  `build_and_persist_billing_period_report` (Persistierungs-Pfad,
  `snapshot_schema_version` = aktuelle Konstante) und
  `generate_custom_report` (Tera/MiniJinja + Sanitize von NaN/Inf).
- **Locking-Regressionen:**
  `service_impl/src/test/billing_period_snapshot_locking.rs` (70 LoC) —
  siehe §7.7.
- **Roundtrip:**
  `service::billing_period::tests` enthält
  `volunteer_row_round_trips_through_from_entities`
  (`service/src/billing_period.rs:186-216`) als Absicherung, dass eine
  persistierte `volunteer`-Row nicht stumm gedroppt wird.
- **DAO-Trait-Default-Methoden:** `dao::billing_period::tests` deckt
  `all_ordered_desc` inkl. Soft-Delete-Filter
  (`dao/src/billing_period.rs:99-238`).
- **Bekannte Lücken:**
  - **[Zu prüfen]** Es gibt aktuell keinen sichtbaren Test, der einen
    Validator gegen mehrere Snapshot-Versionen fährt (v11-vs-v12 Skip-
    Verhalten). Falls die Validator-Logik existiert, gehört ein
    parametrisierter Test dazu.
  - **[Zu prüfen]** Kein Test für den `end_date < letztes_end_date`-
    Guard, weil der Guard selbst evtl. fehlt.
  - **[Zu prüfen]** Kein e2e-Backend-Roundtrip im Browser dokumentiert —
    MEMORY-Feedback „Backend-Roundtrip e2e prüfen" wäre hier
    ausdrücklich anwendbar für den Delete-Latest-Only-Pfad.

## 9. Historie & Kontext

- **2025-08 (Milestone v1.0-Bereich):** Basis-Feature via
  `20250813051848_add-table-billing-period.sql` — Snapshot-Konzept ohne
  Versionierung. Bump-Regeln existieren nur konzeptionell.
- **2026-04:** Migration
  `20260426000000_add-snapshot-schema-version-to-billing-period.sql`
  führt das `snapshot_schema_version`-Feld ein. Der zugehörige OpenSpec-
  Change lebt/lebte unter
  `openspec/changes/billing-period-snapshot-versioning/` (siehe CLAUDE.md-
  Verweis; **[Zu prüfen]** aktueller Status — Verzeichnis war zum
  Verifikationszeitpunkt leer bzw. archiviert).
- **Kontinuierliche Bumps v3–v12** entlang der Feature-Phasen 2, 8.3, 8.4,
  ~debug/vacation-hours-overcounted, ~debug/report-ehrenamt-gesamtstunden,
  quick-260624-ujk, Phase 18 UV-05/D-18-07, Phase 25 HOL-01/02/HCFG-01,
  Phase 28 VAC-OFFSET-01/D-28-05 — siehe §7.4 und die Doc-Comment-Historie
  in `service_impl/src/billing_period_report.rs:38-117`.
- **Cross-Cluster-Abhängigkeit:** Jede Änderung an `ReportingService`
  (`service/src/reporting.rs` und `service_impl/src/reporting.rs`),
  `EmployeeWorkDetailsService::vacation_days_for_year`,
  `derive_hours_for_range` in absence_period-Logik, oder an einem
  Feature-Toggle, das Reporting-Semantik triggert — MUSS auf
  Snapshot-Impact geprüft und, falls betroffen, gebumpt werden.
- **PR-Review-Muster:** Bei jeder PR, die Files unter
  `service_impl/src/reporting.rs`, `service_impl/src/booking_information.rs`
  oder `service_impl/src/absence_period.rs` (bzw. deren Traits) anfasst,
  gehört ein aktiver Grep auf `CURRENT_SNAPSHOT_SCHEMA_VERSION` und die
  bewusste Entscheidung „Bump oder Kommentar warum nicht" ins Review-
  Checklisten-Muster.

---

**Fazit:** Der Cluster friert HR-relevante Zeit-Kennzahlen pro Mitarbeiter
periodenweise unveränderlich ein und stempelt jeden Snapshot mit einer
Schema-Version, damit spätere Formel-Änderungen alte Snapshots nicht
stillschweigend entwerten. Wer die Berechnung eines persistierten
`value_type` verändert, bumpt `CURRENT_SNAPSHOT_SCHEMA_VERSION` **um genau 1**
und hebt den Assert in `billing_period_snapshot_locking.rs` mit — sonst
kann kein Validator jemals Schema-Drift von echten Datenbugs unterscheiden.

---

*Letzte Verifikation gegen Code:* 2026-07-05 gegen
`CURRENT_SNAPSHOT_SCHEMA_VERSION = 12` (siehe git blame dieser Datei).
