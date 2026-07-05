# Feature: Booking (F03)

> **Short form:** A Booking assigns a Sales Person to a Slot in a specific
> calendar week/year — the central write operation in the shift plan
> editor. A Booking log provides the read-only audit trail per week.

**Cluster ID:** F03
**Status:** production
**First introduced:** Booking table 2024-05 (migration `20240507063704`),
user tracking 2025-01 (migrations `20250115000000/01`), conflict-aware
persist path in Phase 3 (v2.x), paid-limit hard enforcement in Phase D-24,
week-lock gate in Phase 40, Absence conflict flag in v2.2.1.
**Responsible crates:** `service::booking`, `service::booking_information`,
`service::booking_log`, `service_impl::booking`, `service_impl::booking_information`,
`service_impl::booking_log`, `dao::booking`, `dao::booking_log`,
`dao_impl_sqlite::booking`, `dao_impl_sqlite::booking_log`, `rest::booking`,
`rest::booking_information`, `rest::booking_log`, `rest-types::BookingTO` /
`BookingLogTO` / `BookingConflictTO` / `WeeklySummaryTO`.

Related: `service::shiftplan_edit` (conflict-aware persist path, week lock)
and `service::warning` (emission channel for the conflicts). See F02 (Slot &
Shiftplan) and F05 (Absence).

---

## 1. What is it? (Business context)

A **Booking** is the actual assignment "Person X works on Slot Y in
calendar week N / year J". It is the narrow, technically cheap object that
gets actually manipulated in the editor — all higher-level metrics
(week summary, reports, Balance, warnings) derive from this one
table.

A **Booking log** is the read-only, denormalized weekly view onto the
same rows including visibility of soft deletes and
user tracking (who created it, who canceled it). It serves the
Shiftplanner as an audit trail and research tool in the frontend.

**Example workflow from a user's perspective:**

1. Shiftplanner opens the shift plan for week 27/2026.
2. Clicks on an empty Slot → selection of a Sales Person.
3. The conflict-aware POST to `/shiftplan-edit/booking` persists the
   entry and returns warnings if applicable (Booking on Vacation day, on a
   "I am not available" day, paid limit exceeded).
4. Warnings are displayed inline in the editor — not as a confirmation
   dialog (user preference).
5. To cancel: click on the entry → `DELETE /booking/{id}` runs
   via `ShiftplanEditService::delete_booking` (including week-lock gate).
6. Via the "Booking log" tab the planner sees the full
   history of the week including the canceled Bookings.

## 2. Business rules

- **Uniqueness per week:** A tuple `(sales_person_id, slot_id,
  calendar_week, year)` may only exist actively once.
  `BookingServiceImpl::create` checks this via
  `booking_dao.find_by_booking_data` and raises `ValidationFailureItem::Duplicate`
  (`service_impl/src/booking.rs:241–253`).
- **Foreign-key validity:** `sales_person_id` and `slot_id` must
  exist; violation → `IdDoesNotExist`
  (`service_impl/src/booking.rs:216–239`).
- **Calendar week range:** `calendar_week` must be `1..=53`
  (`service_impl/src/booking.rs:210–215`).
- **No client IDs / versions / timestamps:** `id`, `version` and
  `created` must be empty on create (`IdSetOnCreate`, `VersionSetOnCreate`,
  `InvalidValue("created")`, `service_impl/src/booking.rs:191–201`).
- **Shiftplan eligibility:** if the Slot is assigned to a shift plan,
  the service checks via `SalesPersonShiftplanService::is_eligible`;
  without permission → `Forbidden` (`service_impl/src/booking.rs:260–277`).
- **User tracking (2025-01):** `created_by`/`deleted_by` are filled
  server-side from the authenticated context. On internal
  `Authentication::Full` the service falls back to `booking.created_by`
  from the caller payload; the last fallback is the sentinel `"system"`
  (`service_impl/src/booking.rs:281–300` and `423–434`).
- **Week-lock gate:** all write paths — including `delete_booking` — run
  through `ShiftplanEditService::assert_week_not_locked` (`shiftplan_edit.rs:598–604`).
  Plain `shiftplanner` rights are **not** enough to bypass the gate — only
  `shiftplan.edit` (D-40-02).
- **Paid limit (D-24):** if the Slot has a `max_paid_employees`, the
  conflict-aware path checks:
  1. With the toggle `paid_limit_hard_enforcement` active and non-Shiftplanner:
     hard `ServiceError::PaidLimitExceeded { current, max }`
     (`shiftplan_edit.rs:618–652`).
  2. Otherwise post-persist soft warning
     `Warning::PaidEmployeeLimitExceeded` (`shiftplan_edit.rs:758–779`).
- **`min_resources` signal:** does not itself carry a Booking gate; is
  used in reporting/traffic light (F02 Slot & Shiftplan) as the target
  value for "Slot understaffed". **[To verify]** whether a frontend
  warning in the editor is coupled to the backend — in the Booking
  service itself `min_resources` is not part of the validation.
- **Absence conflict (v2.2.1):** `book_slot_with_conflict_check` creates
  one `Warning::BookingOnAbsenceDay` per overlapping AbsencePeriod
  (`shiftplan_edit.rs:708–725`); half-day Absences are silently
  tolerated (D-08.3-05).
- **Manual-unavailable conflict:** analogously, one
  `Warning::BookingOnUnavailableDay` is emitted per weekday
  (`shiftplan_edit.rs:726–739`).
- **Copy-week is somewhat idempotent:** `copy_week` discards from the
  source all Bookings whose `(sales_person_id, slot_id)` already exist
  in the target week before copying (`service_impl/src/booking.rs:351–370`)
  — duplicate bookings cannot arise this way.
- **Permission matrix (short form):**
  - Read (`get_all`, `get`, `get_for_week`, `get_for_slot_id_since`):
    `SHIFTPLANNER_PRIVILEGE` ∨ `SALES_PRIVILEGE`.
  - Write (`create`, `delete`): Shiftplanner **or** the authenticated
    user is the assigned Sales Person (`check_booking_permission`,
    `service_impl/src/booking.rs:34–68`).
  - `copy_week`: strictly Shiftplanner.
  - Conflict-aware persist (`shiftplan_edit` path): Shiftplanner ∨ self,
    additionally week lock.
  - Booking log (`booking_log`): strictly Shiftplanner.

## 3. Data model

### Tables & views

| Object | Purpose | Important columns |
| --- | --- | --- |
| `booking` (table) | Active and soft-deleted Bookings | `id BLOB(16) PK`, `sales_person_id BLOB(16)`, `slot_id BLOB(16)`, `calendar_week INT`, `year INT`, `created TEXT`, `deleted TEXT NULL`, `created_by TEXT NULL`, `deleted_by TEXT NULL`, `update_timestamp TEXT`, `update_process TEXT`, `update_version BLOB(16)` |
| `bookings_view` (view) | UUID-formatted view with join on `sales_person` and `slot` — basis of the Booking log | `booking_hex`, `sales_person_hex`, `slot_hex`, `name`, `year`, `calendar_week`, `day_of_week`, `time_from`, `time_to`, `created`, `deleted`, `created_by`, `deleted_by` |

When reading the active dataset, `booking_dao` always filters
`WHERE deleted IS NULL` (`dao_impl_sqlite/src/booking.rs:62–66`). The
Booking-Log DAO deliberately reads **including** soft deletes from
`bookings_view`, in order to make canceled Bookings visible
(`dao_impl_sqlite/src/booking_log.rs:131`).

### Migrations

Chronological list:

- `migrations/sqlite/20240507063704_add-booking.sql` — base table `booking`.
- `migrations/sqlite/20240728155625_add-bookings-view.sql` — read-only view
  `bookings_view` with UUID formatter + Slot/SalesPerson join.
- `migrations/sqlite/20250115000000_add-user-tracking-to-booking.sql` — adds
  `created_by TEXT NULL`, `deleted_by TEXT NULL`. Previous data
  stays as NULL (deliberately, `dao_impl_sqlite/src/booking_log.rs:35–39`).
- `migrations/sqlite/20250115000001_update-bookings-view-add-user-tracking.sql`
  — `DROP VIEW`/`CREATE VIEW` carries the new columns into
  `bookings_view`.

### Relationships

`booking` is child to two aggregates:

```
sales_person ─┐
              ├──< booking >── slot ── shiftplan
special_day ──┘  (view join)
```

- FK reference on `sales_person.id` and `slot.id` (not formally
  enforced in SQLite, see comment in `20250115000000_...sql`).
- `created_by` / `deleted_by` logically reference `user.name` — per
  migration comment deliberately coupled only at application level.
- The `bookings_view` join denormalizes person + Slot into one row,
  so that the Booking log gets by without backend joins.

## 4. Service API

Two Basic services and one Business-Logic service:

- **Basic:** `BookingService` (aggregate manager of the `booking` table),
  `BookingLogService` (read aggregate on `bookings_view`).
- **Business-Logic:** `BookingInformationService` (aggregates `Slot` +
  `SalesPerson` + Absence + working hours into conflict lists and
  weekly summaries).

For the write paths with cross-source warnings additionally
`ShiftplanEditService::book_slot_with_conflict_check` /
`copy_week_with_conflict_check` / `delete_booking` is involved (see F02).

### Trait `BookingService` (Basic)

`service/src/booking.rs:62–114`

```rust
#[async_trait]
pub trait BookingService {
    type Context: …;
    type Transaction: dao::Transaction;

    async fn get_all(&self, ctx, tx) -> Result<Arc<[Booking]>, ServiceError>;
    async fn get(&self, id: Uuid, ctx, tx) -> Result<Booking, ServiceError>;
    async fn get_for_week(&self, cw: u8, year: u32, ctx, tx) -> …;
    async fn get_for_slot_id_since(&self, slot_id: Uuid, year: u32, cw: u8, ctx, tx) -> …;
    async fn create(&self, booking: &Booking, ctx, tx) -> Result<Booking, ServiceError>;
    async fn copy_week(&self, from_cw: u8, from_year: u32, to_cw: u8, to_year: u32, ctx, tx) -> …;
    async fn delete(&self, id: Uuid, ctx, tx) -> Result<(), ServiceError>;
}
```

Auth gates (see `service_impl/src/booking.rs`):

| Method | Permission |
| --- | --- |
| `get_all` / `get` / `get_for_week` / `get_for_slot_id_since` | `SHIFTPLANNER` ∨ `SALES` |
| `create` | Shiftplanner ∨ authenticated user is the Sales Person (`check_booking_permission`, lines 34–68). |
| `copy_week` | `SHIFTPLANNER` (strict). |
| `delete` | Shiftplanner ∨ self, additionally eligibility check against `SalesPersonShiftplanService`. |

TX behavior:

- Every method opens on `tx=None` its own transaction via
  `TransactionDao::use_transaction` and commits at the end.
- `create` fully validates before any write; rollback if one of
  the validations or the DAO insert fails.
- `copy_week` runs in **one** outer TX and calls `create` for each
  source Booking; any duplicate/eligibility violation aborts the whole
  copy (no partial rollback, no partial commit).
- Cross-service reads (`sales_person_service.exists`, `slot_service.exists`,
  `slot_service.get_slot`, `sales_person_shiftplan_service.is_eligible`)
  run under `Authentication::Full` within the same TX, so that they
  see the same read snapshot.

Dependencies (`service_impl/src/booking.rs:20–31`):

- DAOs: `BookingDao`, `TransactionDao`.
- Other services: `PermissionService`, `ClockService`, `UuidService`,
  `SalesPersonService`, `SlotService`, `SalesPersonShiftplanService`.

Strictly speaking, the service is a Basic service (aggregate manager
for `booking`), but due to eligibility and user-trace requirements it
pulls in several other domain services. These dependencies are
unidirectional — no consuming domain service points back to Booking,
so that the tier convention (see `CLAUDE.md`) is preserved.

### Trait `BookingInformationService` (Business-Logic)

`service/src/booking_information.rs:87–113`

```rust
async fn get_booking_conflicts_for_week(year, week, ctx, tx) -> Arc<[BookingInformation]>;
async fn get_weekly_summary(year, ctx, tx)                     -> Arc<[WeeklySummary]>;
async fn get_summery_for_week(year, week, ctx, tx)             -> WeeklySummary;
```

Auth gates:

- `get_booking_conflicts_for_week`: strictly `SHIFTPLANNER`
  (`service_impl/src/booking_information.rs:155–157`).
- `get_weekly_summary`, `get_summery_for_week`: `SHIFTPLANNER` ∨ `SALES`;
  additionally `is_shiftplanner` is computed to output the
  `working_hours_per_sales_person` detail list **only** for planners
  (`booking_information.rs:274–278`).

**Full bypass propagated:** this aggregating layer calls its
inner collaborators (`BookingService::get_for_week`,
`SalesPersonService::get_all`, `SlotService::get_slots_for_week_all_plans`,
`AbsenceService::find_all`/`find_overlapping_for_booking`,
`SalesPersonUnavailableService::get_by_week`,
`SpecialDayService::get_by_week`, `ReportingService::get_week`,
`ShiftplanReportService::extract_shiftplan_report_for_week`,
`ToggleService::…`) consistently with `Authentication::Full`
(e.g. `booking_information.rs:160–169, 205–214, 283–294, 300–303, 519–528`).
The outer permission was checked at the entrance; internal reads are
aggregate details. This is the standard path where `ToggleService`
actually uses its `Authentication::Full` bypass (see MEMORY:
"ToggleService Full-Context-Bypass").

TX behavior:

- `get_weekly_summary` runs in an outer TX over 1..=52/53(+3) weeks of
  the year. All loads are pulled once before the week loop
  (load-once pattern: `all_work_details`, `all_absences`, `volunteer_ids`,
  `active_from` toggle) — see comment "Pitfall 4" and
  `booking_information.rs:290–310`.
- For the Absence conflict view (`get_booking_conflicts_for_week`)
  exactly one Absence lookup is done per affected Sales Person, then
  filtered in-memory per Booking (`booking_information.rs:187–213`).

Dependencies (`booking_information.rs:111–135`): `ShiftplanReportService`,
`SlotService`, `BookingService`, `SalesPersonService`,
`SalesPersonUnavailableService`, `ReportingService`, `SpecialDayService`,
`ToggleService`, `EmployeeWorkDetailsService`, `AbsenceService`,
`PermissionService`, `ClockService`, `UuidService`, `TransactionDao`.

### Trait `BookingLogService` (Basic, read-only)

`service/src/booking_log.rs:26–37`

```rust
async fn get_booking_logs_for_week(year, cw, ctx, tx) -> Arc<[BookingLog]>;
```

Auth gate: strictly `SHIFTPLANNER_PRIVILEGE`
(`service_impl/src/booking_log.rs:33–35`). The service is a pure
read mapper from `BookingLogEntity` to `BookingLog` including soft deletes
and user tracking.

Dependencies: `BookingLogDao`, `PermissionService`, `TransactionDao`.

## 5. REST endpoints

Mount points (`rest/src/lib.rs:641–649`): `/booking`, `/booking-information`,
`/booking-log`, `/shiftplan-edit/booking` (conflict-aware persist,
technically located in the F02 cluster).

| Method | Path | Description | DTO In | DTO Out | Important errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/booking/` | All active Bookings | — | `Vec<BookingTO>` | 401/403 |
| `GET` | `/booking/week/{year}/{cw}` | Bookings of a week | — | `Vec<BookingTO>` | 401/403 |
| `GET` | `/booking/{id}` | Single Booking | — | `BookingTO` | 404 |
| `POST` | `/booking/` | Legacy create (without conflict aggregation) | `BookingTO` | `BookingTO` | 400 (validation), 403 (eligibility), 409 (Duplicate) |
| `DELETE` | `/booking/{id}` | Deletion via `ShiftplanEditService::delete_booking` (week-lock gate) | — | 200 | 403 (week lock / eligibility), 404 |
| `POST` | `/booking/copy?from_year&from_week&to_year&to_week` | Non-conflict-aware week copy | — | 200 | 400, 403 |
| `POST` | `/shiftplan-edit/booking` | Conflict-aware persist with warnings | `BookingTO` | `BookingCreateResultTO` | 409 `PaidLimitExceeded` (D-24-08), week lock |
| `GET` | `/booking-information/conflicts/for-week/{year}/{week}` | Conflicts (unavailable + Absence overlap) | — | `Vec<BookingConflictTO>` | 403 |
| `GET` | `/booking-information/weekly-resource-report/{year}` | Yearly rollout across all CWs | — | `Vec<WeeklySummaryTO>` | 403 |
| `GET` | `/booking-information/weekly-resource-report/year/{week}` | Single week | — | `WeeklySummaryTO` | 403 |
| `GET` | `/booking-log/{year}/{week}` | Audit trail rows of the week | — | `Vec<BookingLogTO>` | 403 |

**[To verify]** The second weekly report path is technically named
`/booking-information/weekly-resource-report/year/{week}` — the
path parameter for the year is missing in the current route
(`rest/src/booking_information.rs:25–27`).
The handler expects `(year, week)`, but the year is bound as a string
`"year"` via the query/path match. Looks like a copy-paste artifact
— should be verified against the actual URL usage in the frontend.

DTOs (`rest-types/src/lib.rs`):

- `BookingTO` (line 103–122): identical to the domain `Booking`,
  including `created_by`/`deleted_by`, `$version` rename.
- `BookingLogTO` (line 158–177): denormalized (`name`, `time_from`,
  `time_to`, `day_of_week`) — no Slot/person object.
- `BookingConflictTO` (line 937–953): `booking` + `slot` + `sales_person`.
- `WeeklySummaryTO` (line 989–1036): weekly aggregate with paid,
  volunteer, committed voluntary hours (CVC series) and per-day capacities.

REST handler files:

- `rest/src/booking.rs` — GET/POST/DELETE for the plain Booking table
  (DELETE deliberately calls `ShiftplanEditService::delete_booking`, so
  the week-lock gate applies, `booking.rs:158–173`).
- `rest/src/booking_information.rs` — conflicts + weekly report.
- `rest/src/booking_log.rs` — audit trail (fully annotated with `#[utoipa::path]`).

## 6. Frontend integration

- **Pages:** `shifty-dioxus/src/page/shiftplan.rs` is the central editor.
  It integrates Booking conflicts (`BOOKING_CONFLICTS_STORE`,
  `booking_conflict.rs`), Booking log (`BOOKING_LOG_STORE`,
  `booking_log.rs`) and calls the conflict-aware persist endpoint
  (`api::book_slot_with_conflict_check`).
- **Services:** `shifty-dioxus/src/service/booking_conflict.rs`,
  `shifty-dioxus/src/service/booking_log.rs`.
- **Components:** `shifty-dioxus/src/component/booking_log_table.rs`,
  `warning_list.rs` (warnings as inline banner instead of dialog, see
  MEMORY feedback).
- **API wrapper:** `shifty-dioxus/src/api.rs`
  - `book_slot_with_conflict_check` → `POST /shiftplan-edit/booking`
    (line 202–235).
  - `remove_booking` → `DELETE /booking/{id}` (line 237–241).
  - `get_booking_conflicts_for_week` → `GET /booking-information/conflicts/for-week/{year}/{week}` (line 900–914).
  - `get_booking_log` → `GET /booking-log/{year}/{week}` (line 915–927).
- **i18n keys:** including `ConflictBookingsHeader` (editor panel title);
  filter labels for the Booking log are maintained per locale in
  `shifty-dioxus/src/i18n/{en,de,cs}.rs`.
- **Proxy:** `shifty-dioxus/Dioxus.toml` maps `/booking`,
  `/booking-information`, `/booking-log` and `/shiftplan-edit`
  to `http://localhost:3000/*`. New Booking endpoints **always** add
  here — otherwise 404 in `dx-serve` dev mode (MEMORY feedback,
  Phase 28/49).

## 7. Edge cases

All general auth/TX/time edges in
[`../domain/edge-cases.md`](../domain/edge-cases.md). Booking-specific:

- **Slot-split double count (Phase 23):** When splitting a Slot via
  `modify_slot`/`modify_slot_single_week`, Bookings move to the new
  segment(s); without atomicity + explicit test guard a double count
  would arise in reports/Balance. The re-point therefore runs in
  **one** transaction (see F02) — regression test see
  `service_impl/src/test/booking.rs`.
- **Deleting a booked Slot:** `modify_slot`/`remove_slot` implicitly
  deletes bound Bookings and records for the audit row `deleted_by`
  from the caller context. If the call runs under
  `Authentication::Full` (e.g. from a system job), the
  `"system"` fallback kicks in in `BookingServiceImpl::delete`
  (`service_impl/src/booking.rs:432–434`). The Booking log continues to
  display these rows.
- **Absence conflict (v2.2.1):** Bookings that fall into an active
  AbsencePeriod are **not** a hard error on legacy `POST
  /booking`. On the conflict-aware path a warning
  (`BookingOnAbsenceDay`) is emitted; half-day Absences (D-08.3-05) are
  silently tolerated (`shiftplan_edit.rs:716–718`).
- **User-tracking consistency:** `created_by`/`deleted_by` are `NULL`
  only for rows written before migration `20250115000000`.
  Live paths carry either the authenticated user, the
  caller-provided `booking.created_by`, or the sentinel
  `"system"` (`service_impl/src/booking.rs:281–300` and `423–434`).
  DAO comment: `dao_impl_sqlite/src/booking_log.rs:35–39`.
- **`copy_week` skips existing target Bookings:** no
  double insert, no error; just silent skip
  (`service_impl/src/booking.rs:351–360`). **[To verify]** whether this
  is desired from a UX perspective or whether an "X was not copied"
  feedback is missing.
- **Paid-limit hard enforcement** only kicks in with active toggle
  `paid_limit_hard_enforcement` **and** non-Shiftplanner
  (`shiftplan_edit.rs:618–652`). Otherwise only soft warning after persist.
- **Weekly report runs `weeks_in_year + 3`:** `get_weekly_summary`
  overshoots the year by three weeks into the following year, to
  consistently show year-boundary weeks (`booking_information.rs:311–316`).
  Important for frontend consumers that only expect one year.
- **`is_shiftplanner` gate for detail rows:** a pure sales user sees
  **no** `working_hours_per_sales_person` in `WeeklySummary`
  (`booking_information.rs:449–465` / `582–598`). Frontend consumers
  must be able to handle an empty list.
- **Volunteer-Absence whole-week-out (VFA-01, D-26):** if an
  Absence period falls into a calendar week, the person is completely
  excluded from Band 1 + Band 2 of that week — not proportionally per
  day (`booking_information.rs:317–343`, `374–402`). Deliberate design,
  not bug.
- **Special-Day filter:** in `get_weekly_summary`/`get_summery_for_week`
  Slots falling on a `Holiday` are hard-filtered out of the
  capacity calculation; for `ShortDay`, `shortday_gate::clip_slot_for_week`
  clips or drops depending on the cutoff-date toggle
  (`booking_information.rs:414–439`, `551–574`). Behavior
  on the legacy branch is safeguarded by Chain C regression tests.
- **Booking log deliberately reads soft deletes too:** anyone expecting
  rows for "canceled" there must filter by `deleted != NULL` (frontend
  does this via `booking_log_status_filter`,
  `shiftplan-dioxus/src/page/shiftplan.rs:281`).

## 8. Tests

- **Unit / roundtrip:** `service_impl/src/test/booking.rs` (1203 LOC) —
  covers CRUD, permission splits (Shiftplanner vs. sales user),
  validation failures (`test_create_with_id`, `test_create_with_version`,
  `test_create_with_created_fail`, `test_create_sales_person_does_not_exist`,
  `test_create_booking_data_already_exists`, `test_create_slot_does_not_exist`,
  `test_delete_no_permission`, …) as well as copy-week.
- **Weekly conflicts & summary:** `service_impl/src/test/booking_information.rs`
  (675 LOC) — rules D-01, D-04, D-05, CVC-04/06 (Band 1 committed voluntary,
  Band 2 surplus), cap-gate cases, multi-person/multi-day.
- **Chain C regression (Phase 51):** `service_impl/src/test/booking_information_chain_c.rs`
  (650 LOC) — legacy semantics per branch of the cutoff-date toggle against
  historical weeks (see MEMORY "Stichtag-Rollout Legacy-Semantik pro Chain").
- **VFA-01 Volunteer Absence:** `service_impl/src/test/booking_information_vfa.rs`
  (368 LOC) — category-agnostic whole-week-out rule.
- **Booking log:** `service_impl/src/test/booking_log.rs` (87 LOC) —
  read mapper, permission gate.
- **Conflict-aware persist path + week lock + paid limit:** tests
  live in `service_impl/src/test/shiftplan_edit*.rs`
  (F02, see feature doc there) and cover `PaidLimitExceeded`,
  `BookingOnAbsenceDay`, `BookingOnUnavailableDay`,
  `assert_week_not_locked`. **[To verify]** explicit test coverage
  of the user-tracking fallback chain (`current_user → payload → "system"`).
- **DAO level:** `dao_impl_sqlite` integration tests cover the
  `WHERE deleted IS NULL` filter and the `bookings_view` read paths.
  **[To verify]** whether there is a dedicated test simulating
  migration `20250115000000` against old data (NULL `created_by`).

## 9. History & context

- **2024-05:** `booking` table introduced as the base write surface of
  the editor. Model: `sales_person × slot × (year, calendar_week)`.
- **2024-07:** `bookings_view` introduced as a denormalized, UUID-formatted
  view — basis for the later audit log.
- **2025-01:** user-tracking fields `created_by`, `deleted_by`
  retrofitted; parallel view recreate. Motivation: traceable
  audit trail in the Booking log.
- **v2.x Phase 3 ("BOOK-02"):** conflict-aware persist path in
  `ShiftplanEditService::book_slot_with_conflict_check` — the new
  standard path of the editor. Legacy `POST /booking` remains for
  API compatibility (D-Phase3-18).
- **v2.x Phase 5 (D-04/06/07/08/15/16):** paid-employee limit as
  soft warning.
- **v2.x Phase 8.3:** half-day Absence + Booking → silently tolerated
  (D-08.3-05).
- **v2.x Phase D-24:** hard enforcement of the paid limit behind toggle
  `paid_limit_hard_enforcement`, Shiftplanner bypass (D-24-02).
- **v2.x Phase 40 (WST-04):** week-lock gate wrapped around all write paths;
  `delete_booking` moves into `ShiftplanEditService`, so that the
  DELETE REST handler also runs through the gate.
- **v2.2.1:** `get_booking_conflicts_for_week` enriches the conflict list
  with active AbsencePeriods; warnings emission (Absence + manual unavailable)
  on the conflict-aware persist path.
- **v2.4 Phase 51 (Chain C, D-51-06/07):** cutoff-date toggle for
  ShortDay clip; legacy semantics per branch frozen in Chain C
  regression tests.
- **Toggle bypass fix (Phase 51 gap closure):** `ToggleService` reads
  treat `Authentication::Full` as all-rights (see
  `service_impl/src/toggle.rs`), so that the internal `Full` callers in
  `booking_information.rs` and `reporting.rs` read consistently — reason
  why these docs explicitly mention the Full bypass.

Further context reads in `.planning/phases/…` (e.g. Phase 3, 5, D-24,
40, 51).

---

*Last verified against code:* see `git log`/`jj log` of this file.

---

**Conclusion:** Booking is the narrow, hard write axis of the editor —
`BookingService` (Basic) validates and persists, `BookingInformationService`
(Business-Logic, Full bypass inward) turns that into aggregated
weekly figures, and `BookingLogService` provides the audit view on
`bookings_view` including soft deletes. Anyone working on this must
keep the conflict-aware persist path in `shiftplan_edit`, the week lock, and the
paid-limit/Absence/unavailable warnings in mind — the legacy `POST /booking`
only knows the simple duplicate/eligibility check.
