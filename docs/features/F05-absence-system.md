# Feature: Absence System (Range-based Absences)

> **In short:** Vacation, sick leave, unpaid leave, and similar absences are
> modelled as **date ranges** (`from_date`–`to_date`). The actual hours are
> derived at reporting time from the contract active on each individual day —
> instead of from fixed per-day postings in `extra_hours`.

**Cluster ID:** F05
**Status:** production
**First introduced:** v1.0 (2026-05-03, phases 1–4 — see
`docs/absence-feature-frontend.md` header)
**Responsible crates:** `service::absence`, `service::absence_conversion`,
`service_impl::absence`, `service_impl::absence_conversion`, `dao::absence`,
`dao_impl_sqlite::absence`, `rest::absence`, `shifty-dioxus::page::absences`

---

## 1. What is this? (Domain view)

Before v1.0, every form of absence was recorded as a **single-day entry with
an hour amount** in `extra_hours` (categories `Vacation`, `SickLeave`,
`UnpaidLeave`, plus volunteer work). This caused three problems (see
`docs/absence-feature-frontend.md:6–13`):

1. Contract changes (e.g. switching from 40 h to 30 h per week) altered the
   actual hours of past vacation days — accounting had to rework them.
2. For the same day, a double entry arose: once in `extra_hours` (hour
   amount) and once in `sales_person_unavailable` (shift plan view).
3. Public holidays had to be manually subtracted from vacation postings.

The **Absence System** models a range exactly once (table `absence_period`).
The hours per day are derived **only at reporting time** from the contract
active on that day (`service::absence::AbsenceService::derive_hours_for_range`,
defined in `service/src/absence.rs:250–258`, implementation in
`service_impl/src/absence.rs:387–556`). Public holidays get 0 h, without a
separate accounting entry.

**Example workflow from the user perspective:**

1. An employee opens `/absences`, clicks "New Absence", picks a category
   (Vacation / Sick Leave / Unpaid Leave), from-date, to-date, and
   optionally half day.
2. The backend validates range order, self-overlap within the same category,
   and permissions (HR ∨ self), then persists the record.
3. The backend computes forward warnings: existing bookings or manual
   `sales_person_unavailable` entries within the new range are returned as
   a **non-blocking hint** (wrapper `AbsencePeriodCreateResultTO`).
4. The frontend displays the new range in the overview and renders the
   warnings as a banner list beneath the dialog — the record is already
   saved.
5. Later, the reporting flow (F06) calls
   `derive_hours_for_range(from, to, sales_person_id)` and receives a
   `BTreeMap<Date, ResolvedAbsence>` with the hours applicable per day.

## 2. Domain rules

All rules from `docs/absence-feature-frontend.md:16–30` verified against
the code:

- **Granularity:** Full day or half day, uniform per period
  (`DayFraction::{Full, Half}`, `service/src/absence.rs:61–66`,
  migration `20260517120000` adds the column). Half days are applied with
  factor 0.5 in `derive_hours_for_range`
  (`service_impl/src/absence.rs:538–541`).
- **Range semantics `[from_date, to_date]` — inclusive on both sides** (D-05).
  DB CHECK `to_date >= from_date` in
  `migrations/sqlite/20260502170000_create-absence-period.sql:28`. The service
  processes ranges via `shifty_utils::DateRange::new` — inversion → typed
  error `ServiceError::DateOrderWrong(from, to)`
  (`service_impl/src/absence.rs:189–190`).
- **Categories:** Exactly three (`Vacation`, `SickLeave`, `UnpaidLeave`). The
  DAO enum `AbsenceCategoryEntity` is deliberately smaller than
  `ExtraHoursCategoryEntity` so the compiler rules out invalid categories
  (`dao/src/absence.rs:9–21`).
- **Self-overlap same-category is forbidden:** The create path calls
  `find_overlapping(sales_person_id, category, range, None, tx)`
  (`service_impl/src/absence.rs:193–207`) and responds with
  `ValidationError([OverlappingPeriod(logical_id)])`. The update path calls
  the same DAO method with `exclude_logical_id = Some(logical_id)` so the
  row does not collide with itself
  (`service_impl/src/absence.rs:281–290`).
- **Cross-category overlap is allowed and resolved by priority:**
  `SickLeave > Vacation > UnpaidLeave` (BUrlG §9-compliant,
  `service_impl/src/absence.rs:65–74`, D-Phase2-03). Applied in the
  reporting flow via `derive_hours_for_range` → `max_by_key(priority)`
  (`service_impl/src/absence.rs:507–512`).
- **Permissions:** For all read/write operations, **HR ∨ self** applies
  (D-10 Option A). Implemented via
  `tokio::join!(check_permission(HR), verify_user_is_sales_person(sp_id))`
  followed by `hr.or(sp)?`
  (`service_impl/src/absence.rs:110–119`, analogous in `find_by_id`, `create`,
  `update`, `delete`, `find_overlapping_for_booking`,
  `derive_hours_for_range`). Exception: `find_all` requires **HR only**
  (`service_impl/src/absence.rs:94–96`) — the by-sales-person path is the
  self-view endpoint.
- **Booking conflict is NOT blocking:** Neither `create` nor `update`
  aborts on overlapping bookings; they return forward warnings of type
  `Warning::AbsenceOverlapsBooking` /
  `Warning::AbsenceOverlapsManualUnavailable`
  (`service_impl/src/absence.rs:837–927`, D-Phase3-16 "no auto-cleanup").
- **Sales Person ID is not modifiable on update:** modification guard in
  `service_impl/src/absence.rs:265–269`, otherwise `ValidationError`.
- **Optimistic locking:** `version` (Uuid). PUT returns `409 EntityConflicts`
  on stale version (`service_impl/src/absence.rs:270–276`).
- **Update rotates the physical row (tombstone + insert):** `logical_id`
  stays stable; external references survive updates (D-07,
  `service_impl/src/absence.rs:297–331`). Analogous to the `extra_hours`
  pattern.
- **Soft delete:** `delete` only sets `deleted`
  (`service_impl/src/absence.rs:354–385`). No physical row drop —
  audit trail is preserved.
- **Weekly cap in the reporting aggregator:** Per ISO week, at most
  `workdays_per_week` vacation days are counted, even if the contract
  covers more weekdays (see the long comment in
  `service_impl/src/absence.rs:462–472`). Bug motivation:
  `vacation-hours-overcounted`.
- **Public holidays reduce entitlement:** In `derive_hours_for_range`,
  days with `SpecialDayType::Holiday` are skipped — no entry in the
  map (`service_impl/src/absence.rs:437–458, 501–503`).
- **REST `path-id wins` on PUT:** Body `id` is overwritten with the
  path segment (`rest/src/absence.rs:373`).

## 3. Data model

### Tables

| Table | Purpose | Key columns |
| --- | --- | --- |
| `absence_period` | Persisted range per `(sales_person, category)` | `id`, `logical_id`, `sales_person_id`, `category`, `from_date`, `to_date`, `description`, `created`, `deleted`, `update_version`, `day_fraction` |
| `absence_period_migration_source` | Back-link `extra_hours_id → absence_period_id`, so conversion operations remain traceable | `extra_hours_id`, `absence_period_id`, `migrated_at` |

Schema excerpt (`migrations/sqlite/20260502170000_create-absence-period.sql:14–43`):

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

Indexes:

- `idx_absence_period_logical_id_active` — UNIQUE, WHERE `deleted IS NULL`.
  Guarantees that at most one live row exists per `logical_id`
  (tombstone pattern).
- `idx_absence_period_sales_person_from` — for the by-sales-person read and
  `find_overlapping_for_booking`.
- `idx_absence_period_self_overlap` — for the self-overlap check in the
  create/update path.

### Migrations

Chronologically:

- `20260502170000_create-absence-period.sql` — base table + three indexes
  (phase 1, recovery from the phase 1 worktree loss).
- `20260503000000_create-absence-migration-quarantine.sql` — cutover
  quarantine table (later removed again).
- `20260503000001_create-absence-period-migration-source.sql` — back-link
  from `extra_hours` to newly created `absence_period` rows.
- `20260517120000_add-day-fraction-to-absence-period.sql` — additive:
  `day_fraction TEXT NOT NULL DEFAULT 'full' CHECK (day_fraction IN
  ('full', 'half'))` (phase 8.3, no-drift for existing data).
- `20260611000000_drop-absence-migration-quarantine.sql` — quarantine
  table removed after the cutover was completed.
- `20260611000002_delete-absence-range-source-active-seed.sql` — Toggle
  seed `absence_range_source_active` removed (M-03: no more source
  switch, see `service_impl/src/test/reporting_additive_merge.rs:5`).

### Relationships

- `absence_period.sales_person_id → sales_person.id` (FK).
- `absence_period_migration_source.absence_period_id → absence_period.id`
  (back-link to conversion operation; write in
  `service_impl/src/absence_conversion.rs:151–160`).
- No FK on `booking` or `sales_person_unavailable` — conflicts are
  warnings, not hard references.

## 4. Service API

Absence is **Business-Logic-Tier** (see CLAUDE.md
"Service tier conventions"): `AbsenceServiceImpl` consumes, in addition to
DAOs and `PermissionService`, also `SalesPersonService`, `SpecialDayService`,
`EmployeeWorkDetailsService`, `BookingService`,
`SalesPersonUnavailableService`, and `SlotService`
(`service_impl/src/absence.rs:46–63`). This is deliberate — Absence combines
Sales Person + Slot + Booking and needs the cross-aggregate view for
forward warnings.

### Trait `AbsenceService`

Definition: `service/src/absence.rs:181–302`. The most important methods:

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

### `AbsenceCategory` and `DayFraction`

Domain enums in `service/src/absence.rs:27–51` and `61–84` respectively,
each with `From<&…Entity>` converters. `DayFraction::default() == Full`
(line 63); existing data therefore works without a backfill.

### `ResolvedAbsence` and `AbsencePeriodCreateResult`

- `ResolvedAbsence { category, hours, days }` — per day already
  conflict-resolved (`service/src/absence.rs:160–165`).
  `hours = days * hours_per_day`, half-day / weekly cap are already
  factored in.
- `AbsencePeriodCreateResult { absence, warnings: Arc<[Warning]> }` — wrapper
  for create/update responses (`service/src/absence.rs:174–177`).

### Auth gates

| Method | Gate |
| --- | --- |
| `find_all` | HR only |
| `find_by_sales_person` / `find_by_id` / `find_overlapping_for_booking` / `derive_hours_for_range` / `derive_days_for_hourly_markers` / `suggest_convert_ranges_for_markers` | HR ∨ self |
| `create` / `update` / `delete` | HR ∨ self |

Verified in the test `test_create_other_sales_person_without_hr_is_forbidden`
(`service_impl/src/test/absence.rs:403–425`), analogous for find_all-non-HR
(`test_find_all_non_hr_is_forbidden`, line 804).

### Transaction behaviour

All methods open their own transaction on `tx = None` via
`transaction_dao.use_transaction(tx).await?` and commit only after the
business logic succeeds. The forward-warning loop runs **after** the
DAO persist and **before** `commit` — if a warning lookup fails, the new
absence row is rolled back together with the changes already written
(`service_impl/src/absence.rs:220–235, 333–347`).

The update path is composite (tombstone old row + insert new row +
warning loop) and runs **atomically** in a single transaction
(`service_impl/src/absence.rs:297–347`).

### `AbsenceConversionService`

Second trait in the cluster (`service/src/absence_conversion.rs:26–48`).
Converts a live `extra_hours` row (category ∈
{Vacation, SickLeave, UnpaidLeave}) atomically into an `absence_period`:

1. Check HR privilege (**HR only**, no self-bypass — D-05).
2. Load the `extra_hours` row via `find_by_logical_id`.
3. Range validation + overlap check against existing `absence_period`.
4. `absence_dao.create(...)` (write 1).
5. `migration_source_dao.upsert_migration_source(...)` — back-link (write 2).
6. `extra_hours_service.soft_delete_bulk(...)` — **via the physical
   `entity.id`, not the `logical_id`** (comment
   `service_impl/src/absence_conversion.rs:164–171`, CR-01: otherwise
   double counting, because versioned rows have physically different
   IDs).

All three writes run in a shared transaction. No snapshot bump needed
(D-16), because reporting has been additively summing from both sources
since 8.4.

### `absence_conversion` as converter to the reporting daily series

Note on the task description: the actual converter between absence range
and the reporting daily series is **not** `absence_conversion.rs`, but
`AbsenceService::derive_hours_for_range`
(`service_impl/src/absence.rs:387–556`). `absence_conversion.rs` refers
exclusively to the one-time data move of a legacy `extra_hours` row into
a new Absence period and is closely related to the historical
`CutoverService` from v1.0 (see comment header
`service_impl/src/absence_conversion.rs:1–9`). Both paths end in
`absence_period` rows, which reporting then handles uniformly.

### Dependencies

- DAOs: `AbsenceDao`, `MigrationSourceDao` (only in the conversion service),
  `ExtraHoursDao` (only in the conversion service), `TransactionDao`.
- Services (only `AbsenceService`, Basic consumers):
  `PermissionService`, `SalesPersonService`, `ClockService`,
  `UuidService`, `SpecialDayService`, `EmployeeWorkDetailsService`,
  `BookingService`, `SalesPersonUnavailableService`, `SlotService`.
- Services (only `AbsenceConversionService`): `ExtraHoursService`,
  `PermissionService`.

## 5. REST endpoints

Route base path `/absence-period`, mounted in `rest/src/lib.rs:656`.

| Method | Path | Description | DTO in | DTO out | Key errors |
| --- | --- | --- | --- | --- | --- |
| `POST` | `/absence-period` | Create new period | `AbsencePeriodTO` | `201 AbsencePeriodCreateResultTO` | 403 (auth), 422 (range inverted, self-overlap, id/version/created/deleted preset) |
| `GET`  | `/absence-period` | All periods **+ live legacy markers** for all persons (HR view) | — | `200 AbsenceListWithProjectionTO` | 403 |
| `GET`  | `/absence-period/{id}` | Single period | — | `200 AbsencePeriodTO` | 403, 404 |
| `PUT`  | `/absence-period/{id}` | Modify period (`path-id wins`) | `AbsencePeriodTO` | `200 AbsencePeriodCreateResultTO` | 403, 404, 409 (version), 422 |
| `DELETE` | `/absence-period/{id}` | Soft delete | — | `204` | 403, 404 |
| `GET`  | `/absence-period/by-sales-person/{sales_person_id}` | Periods + markers for one person | — | `200 AbsenceListWithProjectionTO` | 403 |

Handlers in `rest/src/absence.rs:163–174` (router), `188–210`
(`create_absence_period`), `222–314` (`get_all_absence_periods`), `328–346`
(`get_absence_period`), `363–386` (`update_absence_period`), `400–413`
(`delete_absence_period`), `426–521` (`get_absence_periods_for_sales_person`).

**Related convert endpoint** (documented in cluster F04, only linked here):
`POST /extra-hours/{id}/convert-to-absence` in
`rest/src/extra_hours.rs:32,203`. Body `ConvertExtraHoursRequestTO`,
dispatches into `AbsenceConversionService::convert_extra_hours_to_absence`.

**Deprecation special case:** A `POST /extra-hours` with category
`Vacation`/`SickLeave`/`UnpaidLeave` returns, after the cutover flip, `403
ExtraHoursCategoryDeprecatedErrorTO` (`rest/src/lib.rs:284–295`, body:
`{"error":"extra_hours_category_deprecated","category":"vacation","message":"Use POST /absence-period for this category"}`).
The frontend recognises this error by `error == "extra_hours_category_deprecated"`.

**Wrapper details:** `AbsencePeriodCreateResultTO` carries `.absence`
(persisted period) and `.warnings` (forward warnings —
`AbsenceOverlapsBooking` and `AbsenceOverlapsManualUnavailable`,
`rest-types/src/lib.rs:1942–1970`). `AbsenceListWithProjectionTO` bundles
`absence_periods` and `hourly_markers` — markers are live legacy
`extra_hours` rows of the three Absence categories, enriched with
`derived_days`, `suggested_end`, and `is_full_week`
(`rest-types/src/lib.rs:1872–1916`, handler
`rest/src/absence.rs:250–307`).

DTOs summarised in `rest-types/src/lib.rs:1719–2040`.

## 6. Frontend integration

- **Page:** `shifty-dioxus/src/page/absences.rs` (~4085 lines). Route
  `/absences`. HR vs. employee branch via `auth.has_privilege("hr")`
  (`shifty-dioxus/src/page/absences.rs:1–17`).
- **Services:** `shifty-dioxus/src/service/absence.rs` (CRUD coroutine,
  `ABSENCE_STORE`, `ABSENCE_MODAL_EVENT`, `ABSENCE_REFRESH`,
  `ABSENCE_HOURLY_STORE`) and
  `shifty-dioxus/src/service/absence_marker.rs` (legacy marker store).
- **State:** `shifty-dioxus/src/state/absence_period.rs`
  (`AbsencePeriod`, `AbsenceCategory`, `DayFraction`, `ExtraHoursMarker`).
- **Additional components:** `AbsenceModal`, `AbsenceConvertModal`,
  `ExtraHoursModal`, `WarningList`/`WarningsList`, `CategoryBadge`,
  `StatusPill`, `VacationEntitlementCard`, `VacationPerPersonList`,
  `AbsenceList`, `AbsenceFilterBar`, `StatsGrid`, `DeleteConfirmDialog`
  (`shifty-dioxus/src/page/absences.rs:5–52`).
- **Warnings:** Rendered as a non-blocking list beneath the modal —
  matches the user preference "inline warnings instead of confirmation
  dialog" (memory `feedback_warnings_inline_not_dialog.md`).
- **i18n keys:** Category labels (`vacation`/`sickleave`/`unpaidleave`),
  warning texts, deprecation hint for legacy markers — each in `De`,
  `En`, `Cs`.
- **Proxy:** `shifty-dioxus/Dioxus.toml:98` maps
  `/absence-period` → `http://localhost:3000/absence-period`. The convert
  endpoint runs via the existing `/extra-hours` proxy (F04). Without
  this entry, `dx serve` would return 404 (memory
  `feedback_dioxus_proxy_for_new_backend_endpoints.md`).
- **Reference doc:** The detailed integration brief lives in
  `docs/absence-feature-frontend.md` (v1.0 frontend migration).

## 7. Edge cases

For the central edge-case reference see
[`../domain/edge-cases.md`](../domain/edge-cases.md), section
[§2 Absence & Extra Hours](../domain/edge-cases.md#2-absence--extra-hours).

- **Range spans a Billing Period boundary:** Reporting queries per Billing
  Period its range against `derive_hours_for_range` — the map covers only
  the queried days. Both periods get their share, no double counting.
  **[To verify]** whether the aggregator `billing_period_report` clips
  the Absence per Billing range (see
  `docs/domain/edge-cases.md#22-range-randfälle`).
- **Range spans year change (Carryover):** The share before Dec 31 must
  flow into the Carryover. If Carryover was computed BEFORE the Absence
  insert, the share is missing. In practice, Carryover runs once a year;
  retroactive Absence changes in the previous year must trigger a
  Carryover refresh
  ([`edge-cases.md#22`](../domain/edge-cases.md#2-absence--extra-hours)).
- **Overlap of two Absences for the same person:** Same-category is
  forbidden (422). Cross-category is allowed and resolved by priority
  `SickLeave > Vacation > UnpaidLeave`
  (`service_impl/src/absence.rs:65–74, 507–512`).
- **Absence vs. Booking in the same range:** Generates a non-blocking
  `Warning::AbsenceOverlapsBooking` when creating the Absence (forward
  warning, `service_impl/src/absence.rs:837–893`) and a
  `Warning::BookingOnAbsenceDay` when creating a Booking via
  `POST /shiftplan-edit/booking` (reverse warning, see
  `docs/absence-feature-frontend.md:60–69`). **No auto-cleanup**
  (D-Phase3-16). Reporting must be careful not to credit both the
  Absence day and the Booking day (reporting detail in F06).
- **Absence on a public holiday:** In `derive_hours_for_range`, the day
  is skipped entirely (no map entry,
  `service_impl/src/absence.rs:501–503, 437–458`). The public holiday
  gets its own credit via the Holiday auto-credit path (HCFG-02).
- **Absence on a non-working day:** The contract defines
  `has_day_of_week(weekday)` and `workdays_per_week`. If a range day
  falls on a weekday on which the person does not work, no map entry
  is produced (`service_impl/src/absence.rs:498–500`).
- **Contract change during the range:** If the contract changes mid-Absence,
  the contract active on each individual day is selected
  (`service_impl/src/absence.rs:480–493`). Past periods therefore remain
  prospectively unchanged (see
  `docs/absence-feature-frontend.md:22`).
- **Weekly cap > availability:** `workdays_per_week` (e.g. 2) caps the
  weekly sum even if `has_day_of_week` returns `true` on more days
  (`service_impl/src/absence.rs:462–472, 523–533`). Regression test:
  `service_impl/src/test/absence_derive_hours_range.rs`
  lines 701–815.
- **Half day at the cap boundary:** The half-day share (0.5) is
  additionally capped at `remaining`
  (`service_impl/src/absence.rs:538–543`), so weeks with a partially
  exhausted allowance do not over-count.
- **Update changes `sales_person_id`:** Rejected with
  `ValidationError(ModificationNotAllowed)`
  (`service_impl/src/absence.rs:265–269`).
- **Convert a legacy `extra_hours` row onto an already-occupied range:**
  The overlap check in `absence_conversion.rs:112–126` prevents this
  with `ValidationError(OverlappingPeriod)`.
- **Soft-delete race:** Two parallel PUTs on the same row → one sees a
  stale version → `409 EntityConflicts`
  (`service_impl/src/absence.rs:270–276`).

## 8. Tests

- **Unit / mock tests Absence:** `service_impl/src/test/absence.rs`
  (1323 lines). Covers create happy path, range inversion, self-overlap
  same- and cross-category, ID/version preset guards, update happy path
  (tombstone+insert), overlap exclusion with `Some(logical_id)`,
  unknown ID, sales-person-ID immutability, stale version, delete,
  find-by-ID/sales-person/all, permission violations. From line 864
  forward-warning tests (Booking, ManualUnavailable). From line 1097
  convert-range suggestions (Suggest UV-01/UV-02, half day, weekend,
  public holiday).
- **Unit tests derive-hours:** `service_impl/src/test/absence_derive_hours_range.rs`
  (1178 lines). Covers base case, public holiday = 0, contract change,
  half-day variants (full-day contract, 2-day range, SickLeave), lump-sum
  range with public holiday, overcounting regression, and user scenarios
  ("Mon–Wed with 2 workdays cap = 2 days / 10 h").
- **Unit tests conversion:** `service_impl/src/test/absence_conversion.rs`
  (649 lines). Happy path, physical-ID soft-delete (CR-01 regression),
  range inversion, overlap reject, HR gate, plus integration test against
  in-memory SQLite from line 428.
- **REST tests:** Pure summation logic `derived_days_from_map` inline in
  `rest/src/absence.rs:548–643` (weekly cap, half day, out-of-range,
  missing map). Snapshot locking of the OpenAPI surface runs via the
  `insta` snapshots since phase 4 (see
  `docs/absence-feature-frontend.md:167`).
- **Frontend tests:** Snapshot tests in the `#[cfg(test)]` block at the
  end of `shifty-dioxus/src/page/absences.rs` (Plan-05 Task 3).
- **DAO round-trip:** `dao_impl_sqlite/src/absence.rs:28–68` maps
  `AbsencePeriodDb → AbsencePeriodEntity` and is covered via SQLx-Prepare
  + in-memory pool in the integration test (Conversion impl).

**Known gaps:**

- Explicit end-to-end coverage for "range spans year change →
  Carryover refresh" is **not** covered in the Absence cluster
  (Carryover has its own tests, but no shared test triggers a
  Carryover recomputation after an Absence change in the previous year).
  **[To verify]**
- The cross-configuration test "Booking + Absence on the same day →
  reporting counts once" lives in the reporting cluster (F06).

## 9. History & context

- **v1.0 (2026-05-03) — phases 1–4:** Green field. New range aggregate,
  new REST layer, reporting integration with snapshot bump 2→3, cross-source
  warnings, cutover service with heuristic migration + drift gate.
  Frontend switch to `/absence-period` (see
  `docs/absence-feature-frontend.md:1–8, 32–43`).
- **Cutover history to legacy `extra_hours` (F04):** Before the cutover,
  frontends kept writing to `extra_hours` with categories
  `Vacation`/`SickLeave`/`UnpaidLeave`. The one-off `CutoverService`
  heuristically migrated live rows into ranges (drift gate < 0.01 h per
  `(sales_person, category, year)`). After the flip, `POST /extra-hours`
  returns `403 ExtraHoursCategoryDeprecatedErrorTO` for these three
  categories (`rest/src/lib.rs:284–295`). The convert-per-row variant
  lives on in the `AbsenceConversionService` (phase 8.5,
  `service_impl/src/absence_conversion.rs:1–9`).
- **Phase 8.3 (half-day support):** Additive migration
  `20260517120000_add-day-fraction-to-absence-period.sql`. Default `full`
  guarantees no-drift for existing data (comment
  `dao/src/absence.rs:23–33`).
- **Phase 8.5/8.6 (convert-service extraction):** The cutover machinery
  was extracted into a slim business-logic-tier service
  `AbsenceConversionService`; the historical `CutoverService` could
  therefore be deleted without touching the new service
  (`service_impl/src/absence_conversion.rs:1–9`).
- **Toggle rollout D-51-07 (phase 51, ShortDay slot shortening) and
  HCFG-02 (v1.7, `holiday_auto_credit`):** Two subsequent cutover-date
  toggles that protect the reporting aggregator against historical
  periods. Semantics (see memory
  `feedback_stichtag_rollout_legacy_semantics.md`): **per consumption
  chain, the old semantics before the feature was introduced are
  reconstructed in the gate-off branch** — do not assume "None → raw".
  Relevant for Absences because `derive_hours_for_range` participates in
  reporting aggregates (Chain C BookingInformation, Chain D
  ShiftplanReport): the Toggle does not decide over the Absence range
  itself, but over how the Absence days interact with ShortDay slot
  shortening (`service_impl/src/shortday_gate.rs:1–71`). The former seed
  `absence_range_source_active` was removed in migration
  `20260611000002_delete-absence-range-source-active-seed.sql` —
  since M-03, reporting is additive from both sources without a source
  switch (`service_impl/src/test/reporting_additive_merge.rs:5`).
- **Planning artefacts:** `.planning/milestones/v1.0-ROADMAP.md`,
  `.planning/phases/01-absence-domain-foundation/`,
  `.planning/phases/02-.../`, `.planning/phases/03-.../`, and
  `.planning/phases/04-.../` (deep-context reads for D-Decisions).
- **ToggleService Full-Context bypass** (memory
  `reference_toggle_service_full_context_bypass.md`): Internal aggregates
  that consume Absences call the Toggle with `Authentication::Full` —
  since the phase-51 gap closure these are treated as an all-rights
  bypass, so the reporting pipeline does not fail on missing HR
  privileges.

---

**Summary:** Absences are the range-based successor aggregate for
vacation / sick leave / unpaid leave and replace the legacy per-day
postings from `extra_hours`. The hours are **not persisted**, but are
derived at reporting time via `derive_hours_for_range` from the contract
active on each individual day — with weekly cap, priority `SickLeave >
Vacation > UnpaidLeave`, public holiday = 0, and HR ∨ self gate.

*Last verification against code:* see git blame of this file.
