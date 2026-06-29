# Architecture Research — v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation

**Domain:** Integration of four features into the existing Shifty layered Rust backend + Dioxus frontend
**Researched:** 2026-06-29
**Confidence:** HIGH (all integration points located by reading actual source; line numbers are real and current)

This is an **integration / build-order** research document for an existing codebase, not a greenfield design.
The layered architecture (REST → Service-trait → DAO-trait → SQLite on the backend; Component/Service/State/Page on
the frontend) is taken as fixed and is **not** redesigned. The job is to map each of the four v1.9 features
onto the existing layers, identify every new-vs-modified component, and order the phases against their dependencies.

---

## Existing Architecture (do not redesign)

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│  shifty-dioxus/ (WASM frontend)                                                        │
│   Pages (src/page/shiftplan.rs, absences.rs) → Components (week_view.rs, …)            │
│   Services (src/service/weekly_summary.rs, booking_conflict.rs, absence.rs, …)         │
│   State  (src/state/vacation_balance.rs, weekly_overview.rs, …)                        │
│   Loader/API  (src/loader.rs, src/api.rs)  ← rest-types DTOs (shared crate)            │
└───────────────────────────────────────────────────────────────────────────────────────┘
                         │ HTTP/REST (JSON, cookie-session)
┌───────────────────────────────────────────────────────────────────────────────────────┐
│  Backend workspace                                                                     │
│  rest/ (Axum handlers + session.rs middleware)                                         │
│    └─ context_extractor: resolves impersonate_user_id → effective Context              │
│    └─ impersonate.rs: /admin/impersonate POST/DELETE/GET  ← ALREADY EXISTS             │
│    └─ booking_information.rs, shiftplan.rs, absence.rs, vacation_balance.rs, …         │
│  service/ (traits incl. PermissionService, BookingInformationService, SessionService)   │
│  service_impl/ (booking_information.rs, permission.rs, …)                              │
│  dao/ + dao_impl_sqlite/                                                               │
│  SQLite                                                                                │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### Auth/Context propagation path (crucial for impersonation analysis)

```
HTTP request
  └─ CookieManagerLayer → app_session cookie
       └─ context_extractor middleware (rest/src/session.rs)
            └─ SessionService::verify_user_session(session_id)
                 └─ resolve_session_user_id(session):
                       if session.impersonate_user_id.is_some()
                           → impersonated user id   ← SUBSTITUTION HAPPENS HERE
                       else
                           → real user id
            └─ Extension<Context> = Option<Arc<str>>   (effective user id, ONE value)
  └─ REST handler receives Extension(context)
       └─ builds Authentication<Context>::Context(context)
            └─ passes to service methods
                 └─ PermissionService::check_permission(privilege, authentication)
                      └─ matches Authentication::Context(ctx) → UserService::current_user(ctx)
                           └─ checks EFFECTIVE user's privileges in DB
```

The critical property: **every service call downstream of context_extractor sees only the effective user**.
The real admin identity is NOT present in `Authentication<Context>` once it leaves the impersonate endpoints.

---

## Feature 1: Admin-Impersonation READ + WRITE

### What already exists (no BE work needed)

| Component | File | State |
|-----------|------|-------|
| `start_impersonate` endpoint | `rest/src/impersonate.rs` | DONE — checks admin on REAL user, sets `session.impersonate_user_id` |
| `stop_impersonate` endpoint | `rest/src/impersonate.rs` | DONE — checks admin on REAL user, clears `session.impersonate_user_id` |
| `get_impersonate_status` endpoint | `rest/src/impersonate.rs` | DONE — returns `ImpersonateTO { impersonating, user_id }` |
| Session substitution in middleware | `rest/src/session.rs:54-60` | DONE — `resolve_session_user_id()` substitutes the impersonated ID into the `Context` extension |
| `SessionService::start_impersonate` / `stop_impersonate` | `service/src/session.rs:50-55` | DONE — trait + impl exist |
| `ImpersonateTO` DTO | `rest-types/src/lib.rs` | DONE — `{ impersonating: bool, user_id: Option<Arc<str>> }` |

The entire backend impersonation stack is already implemented and working. All regular service calls
downstream of `context_extractor` transparently run as the impersonated user. WRITE operations
(bookings, absences, etc.) use the impersonated user's effective identity — this is the intended
"act as" behavior.

### No-privilege-escalation seam — confirmed correct

The `start_impersonate`, `stop_impersonate`, and `get_impersonate_status` handlers all perform:
```rust
let real_user_context = Authentication::Context(Some(session.user_id.clone()));
//                                                    ^^^^^^^^^^^^^^^^ REAL user, not impersonated
rest_state.permission_service().check_permission("admin", real_user_context).await?;
```
They bypass `context_extractor`'s substitution by directly constructing the context from
`session.user_id`, not `resolve_session_user_id(session)`. This means:
- Admin-check always uses the real calling admin's identity.
- While impersonating a non-admin user, all other endpoints see the target user's privileges.
- The admin cannot abuse impersonation to gain privileges the target does not have; `check_permission`
  reads the effective user's DB roles, and the effective user IS the impersonated person.

### Audit seam (minimal viable for v1.9)

The impersonate endpoints already run through Axum's tracing middleware. The minimal v1.9 audit
approach: add explicit `tracing::info!("admin {} started impersonation as {}", session.user_id, target_user_id)`
and `tracing::info!("admin {} stopped impersonation", session.user_id)` lines in `rest/src/impersonate.rs`.
This produces a structured log entry in the server log with both the real admin identity and the target.
Full DB-persisted audit log is out of scope for v1.9.

### What is missing — FE only

| Component | New/Modified | File | Notes |
|-----------|-------------|------|-------|
| `ImpersonateStore` global signal | NEW | `shifty-dioxus/src/service/impersonate.rs` (new file) | Stores `{ impersonating: bool, impersonated_user: Option<Arc<str>> }` |
| `impersonate` service coroutine | NEW | same file | Actions: `Check`, `Start(user_id)`, `Stop`; calls `api::get_impersonate_status`, `api::start_impersonate`, `api::stop_impersonate` |
| `api::get_impersonate_status` | NEW | `shifty-dioxus/src/api.rs` | `GET /admin/impersonate` → `ImpersonateTO` |
| `api::start_impersonate(user_id)` | NEW | `shifty-dioxus/src/api.rs` | `POST /admin/impersonate/{user_id}` |
| `api::stop_impersonate` | NEW | `shifty-dioxus/src/api.rs` | `DELETE /admin/impersonate` |
| `ImpersonationBanner` component | NEW | `shifty-dioxus/src/component/impersonation_banner.rs` (new file) | Inline banner: "Acting as: {user}" + Stop button; only renders when `impersonating == true` |
| Admin impersonate UI (selector) | NEW | embedded in admin/settings page or modal | Shows user list, start button; only rendered when the user is an admin (see below) |
| App-level banner mount | MODIFIED | `shifty-dioxus/src/app.rs` | Load impersonation status on startup; render `ImpersonationBanner` in the global layout |
| i18n keys | NEW | `src/i18n/en.rs`, `de.rs`, `cs.rs` | "Impersonating as:", "Stop Impersonation", start-dialog i18n |

### How the FE knows if the current real user is an admin

**Problem**: When impersonating a non-admin user, `GET /auth-info` returns the IMPERSONATED user's
name and privileges (because `context_extractor` has already substituted the identity). So the FE
cannot use `auth-info` privileges to decide whether to show the impersonation UI.

**Solution**: Call `GET /admin/impersonate` on page load. This endpoint:
- Returns `200 { impersonating, user_id }` if the REAL session user is admin.
- Returns `403` if the real user is not admin.

The FE uses the `200`/`403` distinction to gate the impersonation UI. This call is independent of
`auth-info`. Once the FE receives `200`, it knows:
1. The real user is an admin.
2. Whether an impersonation is currently active.

The `ImpersonationBanner` should be shown whenever `impersonating == true`. The "Start Impersonation"
UI should be shown whenever `GET /admin/impersonate` returns `200` (admin, regardless of active impersonation).

---

## Feature 2: Urlaub → Nicht-Verfügbar (discourage marker)

### Root cause confirmed

In `shifty-dioxus/src/page/shiftplan.rs:1120-1123`:
```rust
discourage_weekdays: unavailable_days
    .read()
    .iter()
    .map(|unavailable_day| unavailable_day.day_of_week)
    .collect(),
```
`discourage_weekdays` is computed **exclusively** from `sales_person_unavailable` entries
(recurring weekday-based unavailability). Absence date ranges (Vacation, SickLeave, UnpaidLeave)
from `absence_period` are not included.

`WeekView` component receives `discourage_weekdays: Vec<DayOfWeek>` — a weekday set, not a date set.
The current model is weekday-based. Absences are date-based.

### No new backend endpoint needed

The `GET /absence-period` REST endpoint already exists and is used by the absences page. The
shiftplan page needs to load the current user's absences for the displayed year. The backend already
gates this correctly: non-HR users can only see their own absences; HR/admin can see all. When an
admin is impersonating a user, the effective context IS the impersonated user — so loading absences
returns the impersonated user's absence data naturally.

Existing data relationship in `service_impl/src/booking_information.rs` is informative:
`BookingInformationServiceImpl` already uses `period_overlaps_week(from, to, week_monday, week_sunday)`
(line 77) to check whether an absence date range hits a calendar week. This exact helper can be
reused or copied as a pure function in the frontend for the date→weekday conversion.

### Integration design: frontend join

For the shiftplan page's displayed week `(year, week)`:
1. Load the current person's absence periods (`GET /absence-period?sales_person_id=...&year=...`).
2. For each absence period `[from_date, to_date]`, and for each day-of-week Mon–Sun, compute
   whether the concrete date for that weekday in `(year, week)` falls within `[from_date, to_date]`.
3. Add those weekdays to `discourage_weekdays` alongside the existing `unavailable_days` weekdays.

Concrete conversion (pure Rust logic, no await):
```rust
fn absence_periods_to_discourage_days(
    absences: &[AbsencePeriod],
    year: u32,
    week: u8,
) -> Vec<DayOfWeek> {
    use time::{Date, Weekday};
    let days = [
        (Weekday::Monday,    DayOfWeek::Monday),
        (Weekday::Tuesday,   DayOfWeek::Tuesday),
        // ...
    ];
    let mut result = vec![];
    for (tw, dow) in &days {
        if let Ok(date) = Date::from_iso_week_date(year as i32, week, *tw) {
            if absences.iter().any(|a| a.from_date <= date && date <= a.to_date) {
                result.push(*dow);
            }
        }
    }
    result
}
```

### Scope decision (inform discuss-phase)

The todo is scoped to **own absences of the editing person**. The shiftplan already has
`current_sales_person` signal (the person being worked on). Apply only to that person's absences.
For admin/shiftplanner acting on behalf of another person (or via impersonation), the backend
permission gate on `/absence-period` will return the correct data based on the effective context.

All three absence categories (Vacation, SickLeave, UnpaidLeave) should produce the discourage
marker (mirrors the `BookingOnAbsenceDay` warning that already fires for all three).

### What is missing — FE only

| Component | New/Modified | File | Notes |
|-----------|-------------|------|-------|
| `reload_absence_days` closure | NEW | `shifty-dioxus/src/page/shiftplan.rs` | Mirrors existing `reload_unavailable_days` (lines 350–368); loads absences when `current_sales_person` changes; same trigger: on person-change event |
| `person_absences` signal | NEW | `shifty-dioxus/src/page/shiftplan.rs` | `Signal<Rc<[AbsencePeriod]>>` stored in the shiftplan coroutine's local state |
| `absence_periods_to_discourage_days` helper | NEW | `shifty-dioxus/src/page/shiftplan.rs` or `src/service/` helper module | Pure function; unit-testable with `cargo test` |
| `discourage_weekdays` computation | MODIFIED | `shiftplan.rs:1120-1124` | Merge `unavailable_days` weekdays + absence-derived weekdays |
| `api::get_absence_periods_for_person` | MODIFIED or NEW | `shifty-dioxus/src/api.rs` | May already exist for absences page; add or expose for shiftplan use |

---

## Feature 3: Stale-Daten-Race (week-race guard)

### Root cause confirmed

In `shifty-dioxus/src/service/weekly_summary.rs:37-42`:
```rust
async fn load_summary_for_week(year: u32, week: u8) -> Result<(), ShiftyError> {
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = false;
    let weekly_summary = loader::load_summary_for_week(CONFIG.read().clone(), year, week).await?;
    (*WEEKLY_SUMMARY_STORE.write()).weekly_summary = Rc::new([weekly_summary]);   // ← UNCONDITIONAL
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = true;
    Ok(())
}
```
The store write is unconditional. A stale response from an earlier week can overwrite a newer
response because the coroutine processes actions sequentially but cannot cancel in-flight awaits.

Same pattern exists in `booking_conflict.rs:20-24` (also unconditional write).
`reload_unavailable_days` in `shiftplan.rs:350-368` is a closure-based async, same risk.

### Recommended guard: `(year, week)` tag in store

Rationale: cleaner than a generation counter because it's semantically meaningful (the stored data
IS for a specific week) and allows the render to also guard (show loading instead of stale data).

**Pattern for `WeeklySummaryStore`:**

```rust
pub struct WeeklySummaryStore {
    pub weekly_summary: Rc<[WeeklySummary]>,
    pub data_loaded: bool,
    pub loaded_year: u32,    // NEW
    pub loaded_week: u8,     // NEW
}
```

```rust
async fn load_summary_for_week(year: u32, week: u8) -> Result<(), ShiftyError> {
    // Mark as loading for THIS (year, week)
    {
        let mut store = WEEKLY_SUMMARY_STORE.write();
        store.data_loaded = false;
        store.loaded_year = year;
        store.loaded_week = week;
    }
    let weekly_summary = loader::load_summary_for_week(CONFIG.read().clone(), year, week).await?;
    // Guard: only write if the store still expects THIS (year, week)
    {
        let mut store = WEEKLY_SUMMARY_STORE.write();
        if store.loaded_year == year && store.loaded_week == week {
            store.weekly_summary = Rc::new([weekly_summary]);
            store.data_loaded = true;
        }
        // else: stale result, discard
    }
    Ok(())
}
```

The "last `LoadWeek` wins in setting `loaded_year/loaded_week`" property makes stale writes
self-discard: when LoadWeek(2026, 24) arrives after LoadWeek(2026, 25) has already set
`loaded_week = 25`, the guard `loaded_week == 24` is false → result dropped.

**Render guard in `shiftplan.rs`:**
In the section that reads `weekly_summary.weekly_summary` (lines 1127+), add a check:
```rust
if weekly_summary.data_loaded
    && weekly_summary.loaded_year == *year.read()
    && weekly_summary.loaded_week == *week.read()
```
A mismatch shows a loading/empty state rather than stale data.

**Apply the same pattern to `booking_conflict.rs`:**
`BOOKING_CONFLICTS_STORE` has the same unconditional write risk. Extend `BookingConflictAction`
and `BOOKING_CONFLICTS_STORE` with the same tag. The `reload_unavailable_days` closure
(shiftplan.rs:350-368) can use a simpler guard: capture `(year_snapshot, week_snapshot)` before
the await, and after the await compare against the current signal values before writing to the signal.

### What is modified — FE only

| Component | New/Modified | File | Notes |
|-----------|-------------|------|-------|
| `WeeklySummaryStore` struct | MODIFIED | `shifty-dioxus/src/service/weekly_summary.rs:14-22` | Add `loaded_year: u32`, `loaded_week: u8` fields |
| `load_summary_for_week` | MODIFIED | same file, line 37 | Add tag-set + guard write |
| `WeeklySummaryStore` default | MODIFIED | same file | Initialize `loaded_year: 0, loaded_week: 0` |
| Summary card render | MODIFIED | `shifty-dioxus/src/page/shiftplan.rs:1127` | Add `loaded_year == year && loaded_week == week` guard |
| `BOOKING_CONFLICTS_STORE` | MODIFIED | `shifty-dioxus/src/service/booking_conflict.rs:13` | Add same `(year, week)` tag |
| `load_booking_conflict_week` | MODIFIED | same file, line 20 | Same guard pattern |
| `reload_unavailable_days` | MODIFIED | `shifty-dioxus/src/page/shiftplan.rs:350-368` | Capture `(year, week)` before await; check after |

---

## Feature 4: Urlaubs-Balken-Konsistenz

### Root cause confirmed

In `shifty-dioxus/src/page/absences.rs:866-871`:
```rust
let used_pct: u32 = if total > 0.01 {
    ((props.balance.used_days / total) * 100.0).clamp(0.0, 100.0) as u32
} else { 0 };
```
The bar fills based on `used_days` ONLY. The `remaining_days` display, however, is computed by
the backend as `entitled + carryover - used - planned`, which includes `planned_days`. So the bar
(visual) and the number (textual) are inconsistent when `planned_days > 0`.

All required data is **already in the `VacationBalance` state** (`state/vacation_balance.rs:17`):
- `used_days: f32` — confirmed days off, already booked
- `planned_days: f32` — future absence-period entries not yet past
- `remaining_days: f32` — already correctly accounts for both

### Fix: FE-only, single expression change

Change the bar percentage computation to:
```rust
let committed_days = props.balance.used_days + props.balance.planned_days;
let used_pct: u32 = if total > 0.01 {
    ((committed_days / total) * 100.0).clamp(0.0, 100.0) as u32
} else { 0 };
```

Optional enhancement: show `used` and `planned` as two visually distinct bar segments (different
colors or a striped segment for planned) to communicate "used vs upcoming". This is a UI decision
for the discuss-phase. The minimal viable fix is the single computation change above.

**Overflow case**: When `used_days + planned_days > entitled_days + carryover_days`, `remaining_days`
is negative. The bar `clamp(0.0, 100.0)` already handles this; the existing low-indicator
(`remaining_days <= 3.0` → `text-warn`) already fires for the overconsumption case.
The todo specifically requires: "Überzug sichtbar" — the bar should visually show 100% fill
(and the warn color) when the person is over-entitled. The current clamp to 100% plus the warn
class handles this already if the computation above is applied.

### What is modified — FE only

| Component | New/Modified | File | Notes |
|-----------|-------------|------|-------|
| `VacationPerPersonCard` component | MODIFIED | `shifty-dioxus/src/page/absences.rs:866` | Change `used_days / total` to `(used_days + planned_days) / total` |
| Optional: bar segmentation | MODIFIED | same component | Split bar into used (solid) + planned (striped) segments — discuss-phase decision |

No state, DTO, service, or backend changes required.

---

## Cross-Feature Integration Table

| Feature | BE modified | FE modified | New FE files | New BE files | Dependencies |
|---------|-------------|-------------|--------------|--------------|--------------|
| Admin-Impersonation | None (complete) | `app.rs`, `api.rs`, `service/mod.rs` | `service/impersonate.rs`, `component/impersonation_banner.rs` | None (complete) | None |
| Discourage marker | None | `page/shiftplan.rs`, `api.rs` | optional helper module | None | `api::get_absence_periods` |
| Week-race guard | None | `service/weekly_summary.rs`, `service/booking_conflict.rs`, `page/shiftplan.rs` | None | None | None |
| Vacation bar | None | `page/absences.rs` | None | None | None |

All four features are **independent** — no cross-feature compile dependency. Each can be phased separately.

---

## Recommended Build Order

Dependencies govern order within a feature; across features, they are all independent.
Ordering below is by risk (trivial first, network-involved last) and FE compile-dependency.

```
Phase A  Urlaubs-Balken-Konsistenz (FE-only, ~1 day)
   absences.rs: change bar computation + optional two-segment display + tests.
   Gate: cargo test (frontend), WASM build.
   Compile dep: none. Zero risk of regressions elsewhere.

Phase B  Stale-Daten-Race guard (FE-only, ~1 day)
   weekly_summary.rs: add tag fields, guard write.
   booking_conflict.rs: same pattern.
   shiftplan.rs: render guard + reload_unavailable_days guard.
   Gate: cargo test (frontend, service_tests.rs covers weekly_summary), WASM build.
   Compile dep: none. Purely additive to existing store struct.

Phase C  Urlaub→Nicht-Verfügbar (FE-only, ~2 days)
   api.rs: expose absence-period load for shiftplan context.
   shiftplan.rs: reload_absence_days closure + person_absences signal +
                 absence_periods_to_discourage_days helper +
                 merge with unavailable_days in discourage_weekdays.
   Gate: cargo test (helper unit tests), WASM build, manual shiftplan smoke.
   Compile dep: none beyond existing absence API. Order after B so the shiftplan
                page file has already been touched (reduces merge conflicts).

Phase D  Admin-Impersonation FE (FE-only, ~2-3 days)
   api.rs: 3 new calls (get_status, start, stop).
   service/impersonate.rs (new): ImpersonateStore + coroutine.
   component/impersonation_banner.rs (new): banner + stop button.
   Admin selector UI: user list + start button (location TBD in discuss-phase).
   app.rs: mount banner + init coroutine.
   i18n: 3 locales.
   Gate: cargo test + WASM build + manual admin-path smoke.
   Compile dep: rest-types ImpersonateTO already exists (no new DTO needed).
```

### Ordering rationale

- A and B are purely mechanical, zero-risk FE changes on data already in state/store.
  Ship them first to reduce noise in later phases.
- C is also FE-only but involves a new network call and a new signal; order it after B
  so the shiftplan page is already "clean" (race guard in place) before adding more
  async load paths.
- D comes last because it has the most new FE surface (new file, new coroutine, new
  component, i18n), and the backend is already complete so no sequencing risk there.
- All four phases can be verified independently; none gate the others.

---

## Anti-Patterns Specific to This Change

### Anti-Pattern 1: Adding a new BE endpoint for the discourage-marker absence data

**What people do:** create `GET /shiftplan-absence-days?year=N&week=M&person_id=P` to return
weekday bits from the backend.
**Why wrong:** the data already exists at `GET /absence-period`. Computing weekday-from-date
is trivial Rust (a few lines). Adding a new endpoint creates BE maintenance burden for zero benefit.
**Instead:** frontend join: load absences via the existing endpoint, compute weekday hits locally
using `Date::from_iso_week_date`.

### Anti-Pattern 2: Using `auth-info` to gate the impersonation UI

**What people do:** check `privileges.contains("admin")` from `GET /auth-info` to decide whether
to show the admin impersonation controls.
**Why wrong:** when impersonating a non-admin user, `auth-info` returns the IMPERSONATED user's
privileges — which may not include "admin". The impersonation UI would disappear mid-session.
**Instead:** call `GET /admin/impersonate`. A 200 response proves the real user is admin.
A 403 response proves they are not. This endpoint always checks the real session user.

### Anti-Pattern 3: Encoding the race guard as an atomic counter in global scope

**What people do:** `static LOAD_GEN: AtomicU32 = AtomicU32::new(0)` and increment on each load.
**Why wrong:** in Dioxus WASM (single-threaded), atomics add no synchronization benefit; the
counter is semantically meaningless to the render code ("gen 5" doesn't say which week is loaded).
**Instead:** `(loaded_year, loaded_week)` in the store is self-documenting, lets the render guard
state declaratively, and is trivially comparable.

### Anti-Pattern 4: Showing the vacation bar at 100% capped with no overconsumption signal

**What people do:** clamp `(used + planned) / total` to 100% but don't change the bar or warn color.
**Why wrong:** the remaining-days number shows a negative value, but the bar looks full-but-green.
**Instead:** the existing `remaining_days <= 3.0 → text-warn / bg-warn` already fires for negatives
(since -5 <= 3). Just apply the `committed_days = used + planned` computation and the low-threshold
warn path handles overflow correctly at no extra code cost.

### Anti-Pattern 5: Splitting impersonation stop/start across service tiers

**What people do:** add an `ImpersonationService` at the service tier that wraps session mutation.
**Why wrong:** impersonation is a session management concern, not a domain business rule.
`SessionService` already owns the session lifecycle. Adding another service creates an unnecessary
DI coupling level.
**Instead:** the REST-layer impersonate handlers call `SessionService::start_impersonate` /
`stop_impersonate` directly. The backend is already structured this way — do not refactor it.

---

## Data Flow Diagrams

### Impersonation session resolution (existing, confirmed correct)

```
Admin browser          Axum REST layer                 Service layer
     │                      │                               │
     │ POST /admin/impers./{user} ──────────────────────────│
     │                      │  session.user_id (REAL admin) │
     │                      │  check_permission("admin", …) │ ← REAL user checked
     │                      │  SessionService::start_impers.│
     │                      │  session.impersonate_user_id = target
     │  200 OK              │                               │
     │ ─────────────────────│                               │
     │ GET /booking-inform. │                               │
     │                      │ context_extractor:            │
     │                      │  resolve_session_user_id      │
     │                      │  → impersonate_user_id (target) ─────────────────────┐
     │                      │ Extension<Context> = target   │                       │
     │                      │  ─────────────────────────────│                       │
     │                      │                    BookingInformationService          │
     │                      │                    check_permission(…, target_ctx)    │
     │                      │                    ← target's privileges used         │
     │  response for target │                               │                       │
     │ ─────────────────────│                               │                       │
     │                                                                              │
     NOTE: REAL admin identity never appears in service calls after this point ─────┘
```

### Absence → discourage weekday join (new, FE-only)

```
shiftplan.rs coroutine (on person change)
    ├─ existing: GET /sales-person-unavailable → unavailable_days → weekday set
    └─ NEW:      GET /absence-period?person_id=... → person_absences
                     │
                     └─ absence_periods_to_discourage_days(absences, year, week)
                              │  for each weekday Mon-Sun:
                              │    compute concrete date from (year, week, weekday)
                              │    check if any absence.from <= date <= absence.to
                              └─ returns Vec<DayOfWeek>

WeekView { discourage_weekdays: unavailable_weekdays ∪ absence_weekdays, … }
```

### Week-race guard (new)

```
User clicks week button
    └─ cr.send(WeeklySummaryAction::LoadWeek(year=2026, week=25))
         └─ load_summary_for_week(2026, 25)
              ├─ WRITE store: { data_loaded: false, loaded_year: 2026, loaded_week: 25 }
              ├─ await loader::load_summary_for_week(…)   ← user may click again here
              │    User clicks: LoadWeek(2026, 26)
              │      └─ WRITE store: { data_loaded: false, loaded_year: 2026, loaded_week: 26 }
              │      └─ await loader (in queue, runs after 25 completes)
              └─ READ store: loaded_week == 26 ≠ 25 → DISCARD (stale)
              
              LoadWeek(2026, 26) completes:
              └─ READ store: loaded_week == 26 == 26 → WRITE summary, data_loaded = true
              
Render: weekly_summary.loaded_week == *week.read() → show data (no stale flash)
```

---

## Sources

- `rest/src/impersonate.rs` — read 2026-06-29, all three handler implementations confirmed
- `rest/src/session.rs` — read 2026-06-29, `resolve_session_user_id` and `context_extractor` confirmed
- `service/src/permission.rs` — read 2026-06-29, `Authentication<Context>` enum + `check_permission` logic
- `service_impl/src/permission.rs` — read 2026-06-29, real-user substitution confirmed absent (processes effective context only)
- `service_impl/src/booking_information.rs` — read 2026-06-29, `period_overlaps_week` helper (line 77), absence loading pattern (line 198)
- `shifty-dioxus/src/service/weekly_summary.rs` — read 2026-06-29, unconditional store write confirmed (line 37-42)
- `shifty-dioxus/src/service/booking_conflict.rs` — read 2026-06-29, same pattern confirmed
- `shifty-dioxus/src/page/shiftplan.rs:1120-1123` — read 2026-06-29, `discourage_weekdays` source confirmed
- `shifty-dioxus/src/page/shiftplan.rs:350-368` — read 2026-06-29, `reload_unavailable_days` pattern
- `shifty-dioxus/src/page/absences.rs:866-871` — read 2026-06-29, bar computation confirmed `used_days` only
- `shifty-dioxus/src/state/vacation_balance.rs` — read 2026-06-29, all balance fields confirmed present
- `.planning/todos/pending/2026-06-29-wochen-summary-karten-gegen-stale-daten-bei-schnellem-wochen.md` — root cause analysis
- `.planning/todos/pending/2026-06-29-eigener-urlaub-markiert-nicht-als-nicht-verfuegbar-im-schich.md` — root cause analysis
- `.planning/PROJECT.md` — v1.9 milestone scope

---
*Architecture / integration research for: v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation*
*Researched: 2026-06-29 — Confidence HIGH*
