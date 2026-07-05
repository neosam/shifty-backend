# Feature: Week Metadata (Special Days, Week Status, Week Message, Warnings)

> **In short:** Four loosely coupled sub-features that attach additional facts
> to an ISO (year, week): holidays / short days (Special Days), a planning
> state (Week Status), an informational text (Week Message), and derived
> cross-source anomalies (Warnings). Together they provide planners and
> employees with week-level context beyond the pure slot/booking data core.

**Cluster ID:** F09
**Status:** in production
**First introduced:** Special Days v0.x (migration 2024-10), Week Message v0.x (migration 2025-01), Warnings v1.6 (Phase 3), Week Status v2.x (Phase 39, migration 2026-07)
**Responsible crates:**
- `service::special_days`, `service::week_status`, `service::week_message`, `service::warning`
- `service_impl::special_days`, `service_impl::week_status`, `service_impl::week_message` (Warning is a pure data type)
- `dao::special_day`, `dao::week_status`, `dao::week_message` (no `warning` DAO)
- `rest::special_day`, `rest::week_status`, `rest::week_message` (Warnings travel as part of other wrapper responses)

---

## 1. What is it? (Business perspective)

The shift-planning core (Slots + Bookings) answers the question *"Who works
when?"*. F09 adds three orthogonal dimensions for a concrete
ISO (year, week):

- **Special Days** — holidays (`Holiday`) and short days (`ShortDay`) that
  influence expected hours and slot clipping. Example: 1 May as a Holiday
  → a slot on the holiday does not count toward the contractual expectation
  and, depending on the toggle, triggers auto-credit (chapter 4).
- **Week Status** — release state of a calendar week in the planning
  workflow: `Unset` (implicit, row missing) → `InPlanning` → `Planned` →
  `Locked`. Purely informative; no hard lock at the booking level (see edge
  cases).
- **Week Message** — free-form informational text per (year, week), e.g.
  "Note, extended opening until 10 pm". Rendered prominently above the
  shift plan.
- **Warnings** — non-persisted conflict or rule hints aggregated from
  multiple sources (Booking on Absence day, Absence overlaps Booking,
  paid-employee limit exceeded). They only come back as part of successful
  wrapper responses (200/201) of the writing endpoints — never as an error
  path.

**Example workflow from a user's perspective:**

1. The planner opens calendar week 20/2025 in the shift plan.
2. The frontend loads Special Days (`/special-days/for-week/2025/20`),
   Week Status (`/week-status/by-year-and-week/2025/20`), and Week Message
   (`/week-message/by-year-and-week/2025/20`) in parallel.
3. Thursday is marked as "Ascension Day" (Holiday) → the holiday tile is
   colored, and the slots on that day are weighted differently for the
   balance calculation.
4. The planner sets Week Status to `Planned` and enters an info message
   ("Please be on time, high demand").
5. When creating a new Booking that overlaps an existing vacation period
   of the employee, the backend returns the booking plus a
   `BookingOnAbsenceDay` warning; the frontend shows it as a banner (not
   a blocking dialog).

## 2. Business Rules

### Special Days

- **Categories:** `Holiday` (full day) and `ShortDay` (shortened day with
  mandatory `time_of_day`).
  Verified: `service/src/special_days.rs:13-16`, validation
  `service_impl/src/special_days.rs:131-139`.
- **Uniqueness per (year, calendar_week, day_of_week):** business rule —
  when creating, an already active entry on the same day is **replaced in
  place** (preserve `id`, preserve `created`), no duplicate error, no PUT
  endpoint. `service_impl/src/special_days.rs:170-195` (Same-Date-Replacement
  SDF-01).
- **Type / Time coupling:** `Holiday` **must not** carry a `time_of_day` —
  normalized in the service if needed (`special_days.rs:156-159`). `ShortDay`
  **must** carry a `time_of_day`, otherwise 400 (`ValidationError`).
- **calendar_week range:** 1..=`time::util::weeks_in_year(year)`.
  `service_impl/src/special_days.rs:140-149`.
- **Permission:** `create` / `delete` require `SHIFTPLANNER_PRIVILEGE`;
  reads are open. `service_impl/src/special_days.rs:122, 216`.
- **ISO week-year vs. calendar year:** `get_by_year` returns all Special
  Days whose actual **date** falls into the calendar year — including
  entries that originate from ISO week-year `year - 1` (e.g. New Year
  entries). `service_impl/src/special_days.rs:77-116` (SDF-03 post-ship).
- **Effect on Balance / Slots:**
  - `Holiday` → auto-credit of holiday hours in reporting, provided the
    toggle `holiday_auto_credit` is active (`service_impl/src/reporting.rs:151-243`)
    and holiday hours only in Absence context (`service_impl/src/absence.rs:447,
    755`).
  - `ShortDay` → slot clipping via `shortday_gate::should_clip` +
    `Slot::clip_to(cutoff)`, controlled by cutover date toggle (D-51-07).
    `service_impl/src/shortday_gate.rs:1-40, 204`.

### Week Status

- **Four domain values, three persisted:** `Unset` lives only in the
  service / frontend; the DAO enum `WeekStatusKind` knows only
  `InPlanning | Planned | Locked`. Row absence == `Unset` (D-39-04).
  Explicitly `Unset` (not `None`) to avoid Option shadowing (D-39-03).
  `service/src/week_status.rs:12-18`, `dao/src/week_status.rs:8-13`.
- **Free transitions:** every transition is allowed; there is no state
  machine with guards (D-39-02). `set_week_status` upserts without
  transition validation. `service_impl/src/week_status.rs:94-125`.
- **Read open, write protected:** `get_week_status` has **no** permission
  gate (T-39-03); `set_week_status` requires `SHIFTPLANNER_PRIVILEGE`
  (gate before every DAO access, D-39-01/T-39-01).
  `service_impl/src/week_status.rs:44-75`.
- **Transactional atomicity:** `find` + `write` run in the same TX
  (TOCTOU-free, T-39-04). `service_impl/src/week_status.rs:78-128`.
- **`Unset` semantics on write:** soft-delete the active row if present,
  otherwise no-op. `service_impl/src/week_status.rs:86-92`.
- **No cascade to bookings:** `Locked` does **not** block bookings at the
  DAO / service level — the lock is frontend-side (`shiftplan.rs:304`: only
  non-editors are blocked behind `Locked`). **[To verify]** whether this
  is an intentional convention or a gap in the backend.

### Week Message

- **Free text, one entry per week.** UNIQUE constraint `(year,
  calendar_week)` — **plain UNIQUE**, not partial (see edge cases).
  Migration `20250123000000_add-week-message-table.sql:12`.
- **Permission:** `create` / `update` / `delete` require
  `SHIFTPLANNER_PRIVILEGE`; reads are open.
  `service_impl/src/week_message.rs:78, 118, 143`.
- **No content validation:** no length cap, no sanitizing in the backend
  — the message is pure passthrough. **[To verify]** whether a length cap
  would make sense from a business perspective.
- **`id`/`version` nil-guard on create:** `IdSetOnCreate` /
  `VersionSetOnCreate` on non-nil.
  `service_impl/src/week_message.rs:87-91`.

### Warnings

- **Success, not error:** Warnings travel in the 200/201 path as `warnings: Vec<WarningTO>`
  inside wrapper response DTOs (not as `ServiceError`, not as
  `ValidationFailureItem`/422). `service/src/warning.rs:1-10`,
  `rest-types/src/lib.rs:1919-1993`.
- **Granularity:** one warning per affected booking day (D-Phase3-15),
  **no deduplication** across bookings (Copy-Week accumulates).
  `service/src/shiftplan_edit.rs:25-32`.
- **Five variants** (`service/src/warning.rs:23-73`):
  - `BookingOnAbsenceDay` — reverse warning BOOK-02, booking on an
    Absence day.
  - `BookingOnUnavailableDay` — booking on a manually blocked day
    (`sales_person_unavailable`).
  - `AbsenceOverlapsBooking` — forward warning BOOK-01, new Absence
    overlaps an existing booking.
  - `AbsenceOverlapsManualUnavailable` — Absence covers a manual
    unavailability, **no auto-cleanup** (D-Phase3-16).
  - `PaidEmployeeLimitExceeded` — Phase 5 (D-08): slot
    `max_paid_employees` strictly exceeded. The booking is persisted
    anyway (D-07); NULL limit does not trigger (D-15).
- **Deferred:** `ManualUnavailableOnAbsenceDay` as a 6th variant is
  deferred (D-Phase3-17). No code path today.

## 3. Data Model

### Tables

| Table | Purpose | Important columns |
| --- | --- | --- |
| `special_day` | Holiday / short day per ISO (year, week, day-of-week) | `id`, `year`, `calendar_week`, `day_of_week`, `day_type` (`TEXT`), `time_of_day`, `created`, `deleted`, `update_process`, `update_version` |
| `week_status` | Release state of an ISO (year, week) | `id`, `year`, `calendar_week`, `status` (`TEXT`), `created`, `deleted`, `update_process`, `update_version`. **Partial UNIQUE index** `idx_week_status_active WHERE deleted IS NULL` |
| `week_message` | Free-form info per ISO (year, week) | `id`, `year`, `calendar_week`, `message`, `created`, `deleted`, `update_process`, `update_version`. **Plain UNIQUE** `(year, calendar_week)` |

Warnings have **no table** — they are synthesized in the service layer
from Booking, Absence, and `sales_person_unavailable` data.

### Migrations

- `20241020064536_add-special-day-table.sql` — base table Special Day
  (October 2024, v0.x).
- `20250123000000_add-week-message-table.sql` — Week Message with plain
  UNIQUE (January 2025).
- `20260702000000_create-week-status.sql` — Week Status with partial
  UNIQUE (July 2026, Phase 39). The comment in the migration explicitly
  highlights the difference from Week Messages ("RESEARCH Pitfall P-6").

No separate Warning DDL — Warnings are read-only aggregates.

### Relationships

- Special Day, Week Status, Week Message key **only** on ISO (year, week),
  not on `sales_person_id`. No FKs to other aggregates.
- Warnings reference `booking_id`, `absence_id`, `unavailable_id`,
  `slot_id` at runtime — pure `Uuid` pointers in the payload, no DB
  constraint.

## 4. Service API

### Traits

- `service::special_days::SpecialDayService`
  - `get_by_week(year, calendar_week, ctx) -> Arc<[SpecialDay]>`
  - `get_by_year(year, ctx) -> Arc<[SpecialDay]>` — week-year →
    calendar-year filter, SDF-03.
  - `create(&SpecialDay, ctx) -> SpecialDay` — same-date-replace,
    ID/version nil-guard.
  - `delete(uuid, ctx) -> SpecialDay` — soft-delete.
  - **No** `Option<Transaction>` — this service was implemented in the
    pre-transaction era (**[To verify]** whether intentionally).
- `service::week_status::WeekStatusService` (`week_status.rs:32-57`)
  - `get_week_status(year, calendar_week, ctx, tx) -> WeekStatus`
  - `set_week_status(year, calendar_week, status, ctx, tx) -> WeekStatus`
- `service::week_message::WeekMessageService` (`week_message.rs:56-102`)
  - `get_by_id`, `get_by_year_and_week`, `get_by_year`, `create`, `update`,
    `delete` — all with `Option<Transaction>`.
- `service::warning::Warning` (`warning.rs`) — **not a trait**, pure data
  enum. Produced by:
  - `service::shiftplan_edit::ShiftplanEditService::book_slot_with_conflict_check`
    → `BookingCreateResult { booking, warnings }`.
  - `service::shiftplan_edit::ShiftplanEditService::copy_week` →
    `CopyWeekResult { copied, warnings }`.
  - `service::absence::AbsenceService::create` → `AbsencePeriodCreateResult`.

### Auth Gates

| Sub-feature | Read | Write |
| --- | --- | --- |
| Special Days | open (any role) | `SHIFTPLANNER_PRIVILEGE` |
| Week Status | open (T-39-03) | `SHIFTPLANNER_PRIVILEGE` (D-39-01/T-39-01) |
| Week Message | open | `SHIFTPLANNER_PRIVILEGE` |
| Warnings | n/a — read side effect of other endpoints | produced on the write path of their host endpoints |

### TX Behavior

- **Special Day:** opens **no** TX; DAO calls run in isolation (legacy
  signature without `Option<Transaction>`, see `dao/src/special_day.rs:29-39`).
  Same-date-replace performs `find_by_week` + `update` in separate SQL
  statements — theoretically a TOCTOU window, in practice mitigated by
  the SQLite single-writer (see
  `edge-cases.md#7-transaktionen--atomarität`).
- **Week Status / Week Message:** open a TX even if `tx = None`, `commit`
  at the end. `find` + `write` in the same TX (Week Status atomic by
  design, T-39-04; `service_impl/src/week_status.rs:78-128`).

### Dependencies

All three basic services consume only DAOs + support services — **no**
domain cross-coupling:

- `SpecialDayServiceImpl` → `SpecialDayDao`, `PermissionService`,
  `ClockService`, `UuidService`.
- `WeekStatusServiceImpl` → `WeekStatusDao`, `PermissionService`,
  `ClockService`, `UuidService`, `TransactionDao`.
- `WeekMessageServiceImpl` → `WeekMessageDao`, `PermissionService`,
  `ClockService`, `UuidService`, `TransactionDao`.

Thus all three belong to the **Basic Services** tier (see
`CLAUDE.md` → Service Tier Conventions).

Warnings are wire format; production happens in Business-Logic services
(`ShiftplanEditService`, `AbsenceService`) that combine multiple basics
(`booking_dao`, `absence_service`, `sales_person_unavailable_service`,
`slot_service`).

## 5. REST Endpoints

Base paths per `rest/src/lib.rs:669-675`:
- `/special-days` (nested)
- `/week-status` (nested, Phase 39)
- `/week-message` (nested)

### Special Days (`rest/src/special_day.rs`)

| Method | Path | Description | DTO In | DTO Out | Important errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/special-days/for-week/{year}/{calendar_week}` | List for a week | — | `[SpecialDayTO]` | 500 |
| `GET` | `/special-days/for-year/{year}` | List for a calendar year (SDF-03 filter) | — | `[SpecialDayTO]` | 500 |
| `POST` | `/special-days` | Create or same-date-replace | `SpecialDayTO` | `SpecialDayTO` | 400, 403 |
| `DELETE` | `/special-days/{id}` | Soft-delete | — | 204 | 404, 403 |

### Week Status (`rest/src/week_status.rs`)

| Method | Path | Description | DTO In | DTO Out | Important errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/week-status/by-year-and-week/{year}/{week}` | Status; `Unset` if no row | — | `WeekStatusTO` | — |
| `PUT` | `/week-status/by-year-and-week/{year}/{week}` | Upsert (also for `Unset` = soft-delete) | `WeekStatusTO` | `WeekStatusTO` | 403 |

Design decision D-39-06: GET and PUT on the same week path, **no**
id endpoint (`rest/src/week_status.rs:15-28`).

### Week Message (`rest/src/week_message.rs`)

| Method | Path | Description | DTO In | DTO Out | Important errors |
| --- | --- | --- | --- | --- | --- |
| `POST` | `/week-message` | Create | `WeekMessageTO` | `WeekMessageTO` | 400, 403 |
| `GET` | `/week-message/{id}` | By id | — | `WeekMessageTO` | 404 |
| `PUT` | `/week-message/{id}` | Update (path id overrides body id) | `WeekMessageTO` | `WeekMessageTO` | 400, 403, 404 |
| `DELETE` | `/week-message/{id}` | Soft-delete | — | 204 | 403, 404 |
| `GET` | `/week-message/by-year/{year}` | All messages of a year | — | `[WeekMessageTO]` | — |
| `GET` | `/week-message/by-year-and-week/{year}/{week}` | Message of a week | — | `WeekMessageTO` | 404 |

### Warnings

No dedicated endpoint. Wire form `WarningTO` (tag enum, 5 variants) in
`rest-types/src/lib.rs:1919-2054` travels as a field in, among others:

- `BookingCreateResultTO` (`POST /shiftplan-edit/booking`)
- `CopyWeekResultTO`
- `AbsencePeriodCreateResultTO`

DTOs for all sub-features see `rest-types::lib` — `SpecialDayTO`
(:1190-1241), `WeekMessageTO` (:1310-1354), `WeekStatusTO` /
`WeekStatusKindTO` (:1361-1396), `WarningTO` (:1942-2054).

## 6. Frontend Integration

- **Pages:** `shifty-dioxus/src/page/shiftplan.rs` — main consumer of all
  three sub-features; `shifty-dioxus/src/page/settings.rs` for Special
  Days maintenance.
- **Components:**
  - `component/warning_list.rs` — shared `WarningList` component, renders
    all `WarningTO` variants as **inline banners** (not as blocking
    dialogs, see `feedback_warnings_inline_not_dialog`).
  - `component/week_status_dropdown.rs` + `component/atoms/week_status_badge.rs`
    — UI element for status setting and display.
  - `component/top_bar.rs` — embeds the Week Status badge.
- **Services:** `shifty-dioxus/src/service/week_status.rs` (coroutine
  `WeekStatusAction::Load` / `Set`), `service/absence.rs` (warning
  propagation in the Absence flow).
- **State:** `state/week_status.rs` (store); warnings are held in a
  page-local signal.
- **i18n keys:** `i18n/{en,de,cs}.rs` — including
  `BookingWarningDialogHeaderSingular` / `-Plural`; the backend only
  delivers structured warning data, translation happens in the frontend
  (`warning.rs:63-64`).
- **Proxy (`shifty-dioxus/Dioxus.toml`):**
  - Line 60: `/special-days` → `http://localhost:3000/special-days`
  - Line 88: `/week-message` → `http://localhost:3000/week-message`
  - Line 90: `/week-status` → `http://localhost:3000/week-status`
  Missing proxy entry = 404 in `dx serve` dev mode (see
  `feedback_dioxus_proxy_for_new_backend_endpoints`).

## 7. Edge cases

Central edge case reference: [`../domain/edge-cases.md`](../domain/edge-cases.md).
Relevant sections: [§4 Time & Timezone](../domain/edge-cases.md#4-zeit--zeitzone)
and [§1 Hour Account](../domain/edge-cases.md#1-stundenkonto) (there
especially §1.1 "New holiday in a closed year" and §1.4 "Special Days &
Holidays").

- **Holiday on a weekend:** a `Holiday` entry on Saturday/Sunday is
  tolerated as a matter of business logic (no rejection in the service),
  but only affects reporting if the employee would have contract hours on
  that weekday — auto-credit uses `EmployeeWorkDetails::holiday_hours()`
  (`reporting.rs:230-233`). **[To verify]** whether the UI gives a
  warning here.
- **Retroactive Special Day entry into a closed year:** the balance
  calculation changes live, carryover remains static → drift. Convention
  per `edge-cases.md#1-stundenkonto` §1.1: **do not do it**, unless
  carryover is manually recomputed.
- **Week crossing year boundary (ISO week-year ≠ calendar year):** a
  holiday on 1 January of a year is stored internally under
  `(year=previous, week=53, day=Mo/Tu)`. `get_by_year` resolves this via
  two-year load + calendar filter (`special_days.rs:77-116`). The UI /
  Excel should never blindly sort by `special_day.year`.
- **`Locked` without backend enforcement:** `Locked` status does not lock
  bookings at the DAO / service level, only on the UI side for
  non-editors (`shiftplan.rs:304`). A client with `SHIFTPLANNER_PRIVILEGE`
  and API access can continue booking in a `Locked` week. **[To verify]**
  whether a backend lock is desired (Fat Backend principle, see
  `feedback_fat_backend_thin_client`).
- **Week Message UNIQUE collision:** because `week_message` has **no**
  partial index, a repeated insert for (year, week) collides **even
  against soft-deleted rows** — the DAO error cascades to 500. The DAO
  behavior is **[To verify]** in `dao_impl_sqlite/src/week_message.rs`.
- **Special Day replace under race:** two parallel POSTs on the same day
  can produce a duplicate insert in separate SQL statements
  (`find_by_week` + `update`). SQLite single-writer mitigates this in
  production, but it is not a hard constraint.
- **`Unset` as client payload:** `PUT /week-status/.../unset` soft-deletes
  the row. The frontend may send `Unset` to clear the status — not DELETE.
  `service_impl/src/week_status.rs:86-92`.
- **Warning volume with `copy_week`:** since there is no dedup across
  bookings (D-Phase3-15), a copy-week operation can produce dozens of
  warnings. The frontend renders them as a scrollable list, no pagination.
- **`PaidEmployeeLimitExceeded` with `NULL` limit:** the warning does not
  trigger if the slot has no `max_paid_employees` set (D-15,
  `warning.rs:56-72`). A slot without a limit is unbounded.

## 8. Tests

### Unit / Integration

- **Special Days:** `service_impl/src/test/special_days.rs` (843 lines).
  Covers, among others:
  - `test_get_by_year_returns_new_year_day_under_calendar_year` — SDF-03
    week-year → calendar-year.
  - `test_create_replaces_same_date_entry` — same-date-replace SDF-01.
  - `test_create_switches_holiday_to_shortday` /
    `test_create_switches_shortday_to_holiday` — atomic type switch.
  - `test_holiday_shortday_roundtrip_atomic` — round trip.
  - `test_create_rejects_shortday_without_time` — type/time coupling.
  - `test_create_rejects_calendar_week_out_of_range` — week bounds.
  - `test_create_rejects_nonnil_id` / `_version` — nil guards.
  - `test_create_forbidden_without_shiftplanner`,
    `test_delete_forbidden_without_shiftplanner` — auth gates.
- **Week Status:** `service_impl/src/test/week_status.rs` (446 lines).
  Among others:
  - `test_set_permission_denied_no_dao_write` — gate before DAO (T-39-01).
  - `test_set_unset_soft_deletes_existing` /
    `test_set_unset_noop_when_absent` — `Unset` semantics.
  - `test_set_creates_when_absent` / `test_set_updates_when_present` —
    upsert.
  - `test_transitions_free` — D-39-02.
  - `test_get_returns_unset_when_absent` / `test_get_maps_kind` —
    row-absence semantics.
- **Warnings:** covered implicitly via tests in
  `service_impl/src/test/absence.rs`, `test/shiftplan_edit.rs`,
  `test/slot.rs`, `test/booking_log.rs` — cross-source conflicts are
  asserted there (verified by grep).

### Known gaps

- **No dedicated `week_message` test file** (grep finds nothing under
  `service_impl/src/test/`). Coverage only indirectly via REST smoke.
  **[To verify]**.
- **No test for `PaidEmployeeLimitExceeded` warning suppression on
  `NULL` limit** documented in this cluster doc — presumably present in
  the Phase 5 tests, **[To verify]**.
- **No BE test for `Locked` + booking attempt** (see edge case above).

## 9. History & Context

- **Special Days** — October 2024, migration
  `20241020064536_add-special-day-table.sql`. Post-ship fixes SDF-01
  (same-date-replace) and SDF-03 (week-year filter) in v2.x
  (`service_impl/src/special_days.rs:76, 106`). Server-side validation
  D-33-06/07 was added in Phase 33 (`special_days.rs:127-152`).
- **Week Message** — January 2025, migration
  `20250123000000_add-week-message-table.sql`. Plain UNIQUE is legacy;
  Week Status uses the "proper" partial UNIQUE form (see migration
  comment 2026-07 "RESEARCH Pitfall P-6").
- **Warnings** — Phase 3 (v1.6, 2025), cross-source conflict warnings in
  the Absence / Booking / Unavailable triangle; 5th variant
  `PaidEmployeeLimitExceeded` added in Phase 5 (`warning.rs:54-72`).
  6th variant `ManualUnavailableOnAbsenceDay` intentionally deferred
  (D-Phase3-17).
- **Week Status** — Phase 39 (v2.x, July 2026), migration
  `20260702000000_create-week-status.sql`. Design decisions: `Unset`
  variant (D-39-03/04), free transitions (D-39-02), read open / write
  gated (T-39-01/03), TX atomicity (T-39-04), unified week path (D-39-06).
- **Not F09, but related:** `shortday_gate` (Phase 51, D-51-07) consumes
  Special Days and the toggle `SHORTDAY_ACTIVE_FROM` to roll out slot
  clipping on the cutover date (`service_impl/src/shortday_gate.rs:1-40`).

---

**Conclusion:** F09 is a collection of four independent, loosely coupled
sub-features around ISO (year, week) metadata; each sub-feature is
understandable on its own, but they share the auth model (read open,
write gated by `SHIFTPLANNER_PRIVILEGE`) and the week path in the
frontend. The relevant sharp edges lie at ISO week-year ↔ calendar year,
the missing backend enforcement for `Locked`, and the plain-UNIQUE trap
in `week_message` — everything else is proven standard CRUD.

*Last verification against code:* see git blame of this file.
