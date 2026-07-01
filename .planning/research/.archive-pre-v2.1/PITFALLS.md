# Pitfalls Research

**Domain:** Shifty v1.9 — Axum/SQLite RBAC backend + Dioxus/WASM frontend, adding
impersonation (read+write), Dioxus async race fix, vacation-bar math, and
Urlaub-Nicht-Verfuegbar discourage marker.
**Researched:** 2026-06-29
**Confidence:** HIGH (based on direct codebase inspection of all relevant call sites:
`rest/src/session.rs`, `rest/src/impersonate.rs`, `service_impl/src/permission.rs`,
`shifty-dioxus/src/service/weekly_summary.rs`, `shifty-dioxus/src/page/shiftplan.rs`,
`shifty-dioxus/src/page/absences.rs` `PersonVacationCard`, the two todo files,
and `CLAUDE.md` service-tier conventions)

> Orchestrator note: Pitfalls 1-2 are load-bearing security design decisions for
> impersonation. They MUST be resolved before any write path is wired up. Pitfalls
> 4-5 are a confirmed bug class (stale-week race) that must be fixed atomically across
> all three affected loaders. Read P1, P2, P4 first.

---

## Critical Pitfalls

### Pitfall 1: Audit Trail Loss — Admin Identity Erased on Every Write

**What goes wrong:**
`context_extractor` (both `oidc` and `mock_auth` variants in `rest/src/session.rs`)
calls `resolve_session_user_id` which returns `impersonate_user_id` when set:

```rust
fn resolve_session_user_id(session: &service::session::Session) -> Option<Arc<str>> {
    if let Some(ref impersonate_user_id) = session.impersonate_user_id {
        Some(impersonate_user_id.clone())   // real admin ID is dropped here
    } else {
        Some(session.user_id.clone())
    }
}
```

This `Option<Arc<str>>` is inserted into Axum request extensions. Downstream, every
`Authentication::Context(user_id)` in every service only ever sees the impersonated
user ID. When a write happens — create booking, create absence, modify working
hours — the service's `process`/`created_by`/actor field records the impersonated
user. The real admin's identity is nowhere in the service call stack. Post-hoc
forensics cannot distinguish "Alice did this" from "Admin Bob did this acting as
Alice."

`Context = Option<Arc<str>>` is a single-field type. The impersonation layer
collapses both identities into one at the HTTP boundary. The service layer was never
designed to carry both.

**Why it happens:**
The type alias `pub type Context = Option<Arc<str>>` in `rest/src/session.rs` has one
slot. The impersonation mechanism was designed for read-only before v1.9. For
read-only use, losing the real actor is acceptable (no write audit needed). For
write-capable impersonation it is a correctness violation.

**How to avoid:**
Before wiring the write path, change `Context` from `Option<Arc<str>>` to a struct:

```rust
pub struct SessionContext {
    pub effective_user_id: Arc<str>,   // what PermissionService checks
    pub real_user_id: Arc<str>,        // what audit records use
    pub is_impersonating: bool,
}
pub type Context = Option<SessionContext>;
```

Update `resolve_session_user_id` to populate both fields. Update
`PermissionService::check_permission` and `check_user` to use `effective_user_id`.
Update audit/DAO call sites (any `process` or `created_by` string) to use
`real_user_id`. The impersonate endpoints already read the raw session to get
`session.user_id` (real user) — that pattern is the model.

If the struct change is judged too invasive for v1.9, the minimum safe alternative
is: store `(real_user_id, impersonate_user_id)` as a two-tuple in the session's
in-memory extension, and thread `real_user_id` through to every DAO write as the
`process` string. Do not proceed to write-capable impersonation without one of these
two approaches in the phase plan.

**Warning signs:**
- `created_by` / `process` fields in the DAO contain the impersonated user ID for
  admin-initiated writes. No record says "admin=Bob acting-as=Alice".
- `permission_service.current_user_id(context)` returns the impersonated user in
  ALL service call sites (confirmed by tracing the `Authentication::Context` path in
  `service_impl/src/permission.rs`).
- `check_user("alice", context)` passes for admin Bob impersonating Alice — correct
  for impersonation, but all audit records say Alice, never Bob.

**Phase to address:**
Impersonation backend design phase. The audit contract must be locked before any
write-path handler is implemented; retrofitting the Context type post-write is
expensive.

---

### Pitfall 2: Admin-Gate on New Endpoints Checks Impersonated User, Not Real Admin

**What goes wrong:**
The three existing impersonate endpoints (`start_impersonate`, `stop_impersonate`,
`get_impersonate_status` in `rest/src/impersonate.rs`) correctly read the raw
session from the cookie and check `session.user_id` (real user) against "admin":

```rust
let real_user_context: Authentication<Option<Arc<str>>> =
    Authentication::Context(Some(session.user_id.clone()));
rest_state.permission_service()
    .check_permission("admin", real_user_context).await?;
```

Any NEW admin-only endpoint written during v1.9 that uses the `context_extractor`-
injected identity instead will check the IMPERSONATED user's privileges. An admin
impersonating a non-admin employee will get 403 on their own admin tools.

Concrete scenario: a new "admin audit log" or "impersonation history" endpoint guards
via `permission_service.check_permission("admin", context)` where `context` comes
from `Extension::<Option<Arc<str>>>` (the `context_extractor` path). During
impersonation this checks the non-admin impersonated user — 403.

**Why it happens:**
Two code paths diverge silently:
1. `context_extractor` middleware: resolves effective identity (impersonated user).
2. Raw session read from cookie: resolves real identity.

All regular handlers use path 1. The three impersonate handlers use path 2. New
developers copy "regular handler" boilerplate without knowing about the divergence.
There is no compile-time guard distinguishing the two.

**How to avoid:**
Add a typed extractor `RealUserAuth` that extracts `session.user_id` from the cookie,
independent of impersonation. All admin-management endpoints must use this extractor.
Add a clippy lint or a doc comment in `rest/src/lib.rs` above `context_extractor`
explicitly stating: "All permission gates for admin-management operations must use
the real session user, not the effective user resolved here. See impersonate.rs for
the pattern."

Alternatively, if the Context struct from Pitfall 1 is adopted, `check_permission`
for admin gates can accept a flag that uses `real_user_id` instead of
`effective_user_id`.

**Warning signs:**
- New admin endpoint returns 403 when admin is in an impersonation session.
- The test suite does not include: "admin while impersonating a non-admin can still
  call this admin-only endpoint."
- Any handler added in v1.9 that uses `Extension::<Option<Arc<str>>>` and calls
  `check_permission("admin", ...)` on that value is a candidate for this bug.

**Phase to address:**
Impersonation design phase. The two-path contract must be documented before any
new handler is added.

---

### Pitfall 3: Frontend Stores Retain Impersonated Data After Session Ends

**What goes wrong:**
Dioxus global stores (`VACATION_BALANCE_STORE`, `VACATION_TEAM_STORE`,
`WEEKLY_SUMMARY_STORE`, absence list, `current_sales_person`, etc.) are populated
from API calls made while the admin is impersonating Alice. When the admin calls
`DELETE /impersonate` and the backend clears `session.impersonate_user_id`, the
frontend stores still contain Alice's data. The admin now sees the correct identity
in the banner (gone), but all the cached data reflects Alice until each coroutine
fires a reload action.

Depending on store invalidation logic, this stale state may persist for the entire
page lifetime. The admin books a slot thinking they are booking for themselves but
the `current_sales_person` store still holds Alice's `SalesPerson` record.

**Why it happens:**
Global Dioxus stores have no invalidation budget tied to "which user is active." A
store load is triggered by explicit `Action` sends. When impersonation ends, nothing
broadcasts "clear all user-scoped stores and reload for the real user."

**How to avoid:**
On successful `stop_impersonate` API response, broadcast reset/reload actions to
every store that holds user-scoped data. Centralize this in an `ImpersonationService`
coroutine that handles `StopImpersonation` by:
1. Sending `LoadSelf(real_sales_person_id, year)` to `VacationBalanceAction`.
2. Sending `LoadCurrentSalesPerson` (or equivalent reset) to the sales person store.
3. Sending `WeeklySummaryAction::LoadWeek(current_year, current_week)`.
4. Clearing the `current_sales_person` signal and allowing it to re-resolve from the
   new `/my/sales-person` call.

Write a test: render as impersonated Alice, stop impersonation, assert stores show
real admin's data (or the admin's own `None` if they have no SalesPerson record).

**Warning signs:**
- After stopping impersonation, the vacation-balance card still shows Alice's numbers
  until manual page reload.
- The shift-plan "add me to slot" action books the WRONG person (Alice's sales person
  ID still in `current_sales_person`).
- The impersonation banner disappears but the page title still says Alice's name.

**Phase to address:**
Impersonation frontend phase. The `ImpersonateTO` global state and stop flow must
be designed with explicit store tear-down.

---

### Pitfall 4: Unconditional Store Overwrite After `await` — Stale-Week Race

**What goes wrong:**
`load_summary_for_week(year, week)` in
`shifty-dioxus/src/service/weekly_summary.rs` does:

```rust
async fn load_summary_for_week(year: u32, week: u8) -> Result<(), ShiftyError> {
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = false;
    let weekly_summary =
        loader::load_summary_for_week(CONFIG.read().clone(), year, week).await?;
    (*WEEKLY_SUMMARY_STORE.write()).weekly_summary = Rc::new([weekly_summary]);
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = true;
    Ok(())
}
```

The coroutine `weekly_summary_service` processes `LoadWeek` actions sequentially
(one `rx.next().await` at a time), but within each action the `loader::...await`
suspends the coroutine. If the user switches weeks again before the HTTP response
arrives, a second `LoadWeek(new_year, new_week)` action is queued. The first
response arrives, the store is written with stale data, and the second response
either overwrites it correctly (race win) or the first wins (race loss). The todo
confirms this is a visible bug in production.

**Why it happens:**
The store carries no `(year, week)` identity key. After `await` resumes, there is no
check "is this result still for the week currently selected?" The global signal is
written unconditionally.

**How to avoid:**
Add a `generation: u64` counter to `WeeklySummaryStore`. Before the `await`, read
and capture the current generation. After the `await`, compare: if the current
generation differs from the captured value, discard the result without writing.
Increment the counter atomically when `LoadWeek` is received.

Alternatively, store the `(requested_year, requested_week)` in the store and compare
after the `await`. The generation counter is preferred because it handles the A-B-A
case (navigate week 3 → week 4 → week 3 again: same `(year, week)`, different
logical load intent — generation counter correctly discards the middle result).

**Warning signs:**
- Week headers show data for a week other than the selected one.
- Fast keyboard navigation through weeks leaves the summary cards in an intermediate
  state that does not match the week shown in the plan grid.
- Any `GlobalSignal` loader that does `.write()` unconditionally after an async call
  is a candidate for this bug (use `grep -n "write().*data_loaded\|weekly_summary"
  shifty-dioxus/src/service/*.rs` to find them).

**Phase to address:**
Stale-Daten-Race fix phase. Fix atomically in this phase; do not defer.

---

### Pitfall 5: Same Race in Sibling Loaders — BookingConflict and UnavailableDays

**What goes wrong:**
The unconditional-write-after-await pattern documented in Pitfall 4 is present in at
least two sibling loaders, as called out in the todo file:

- `BookingConflictAction::LoadWeek` (see `shiftplan.rs` lines 304-305) feeds a
  separate store for conflict highlighting.
- `reload_unavailable_days` closure (see `shiftplan.rs` lines 350-368) writes to the
  `unavailable_days` signal that drives the grey-out of Nicht-Verfuegbar days.

Fixing only `WEEKLY_SUMMARY_STORE` leaves booking-conflict highlights and unavailable
markers still subject to the race.

**Why it happens:**
The store-write pattern was the idiomatic Dioxus approach before the race was
noticed. Each service was written independently; there is no shared abstraction that
enforces the generation-guard.

**How to avoid:**
When fixing the weekly-summary race, audit EVERY loader that follows the pattern
"fire coroutine action, await HTTP, write GlobalSignal" within the shiftplan page
and its services. Run:

```
grep -n "write()" shifty-dioxus/src/service/*.rs shifty-dioxus/src/page/shiftplan.rs
```

Confirm each write site is guarded. Fix all instances in the same phase as Pitfall 4
so the three fixes ship atomically. Do not partial-fix.

**Warning signs:**
- After fixing the weekly-summary race, booking-conflict highlights or unavailable-day
  markers still lag or show previous-week data.
- The week-change handler fires three `LoadWeek`-type sends (lines 304-307 in
  `shiftplan.rs`) — all three must be guarded.

**Phase to address:**
Same Stale-Daten-Race fix phase as Pitfall 4. All three stores, one phase.

---

### Pitfall 6: Vacation Bar Counts Only `used_days`; Number Shows `used + planned` Subtracted

**What goes wrong:**
`PersonVacationCard` in `absences.rs` renders the bar as:

```rust
let total = props.balance.entitled_days + (props.balance.carryover_days as f32);
let used_pct: u32 = if total > 0.01 {
    ((props.balance.used_days / total) * 100.0).clamp(0.0, 100.0) as u32
} else { 0 };
let bar_style = format!("width:{}%", used_pct);
```

The bar shows `used_days / total`, clamped to 100%. The number displayed next to the
bar is `balance.remaining_days`, which the backend computes as
`entitled + carryover - used - planned`.

Example: entitled=20, used=5, planned=10 days.
- Bar: 5/20 = 25% — "quarter used."
- Number: 20-5-10 = 5 remaining.
An HR user reading both together would mentally infer "75% left, but only 5 days
remaining." The bar does not account for the 10 planned days already committed, so
it always understates how consumed the vacation budget is. At 100% bar the balance
can still be negative (overdraft hidden by clamp).

**Why it happens:**
The bar was written before `planned_days` was tracked. `remaining_days` was added
later to account for planned absences; the bar percentage was not updated to match.

**How to avoid:**
Change the percentage to:
```rust
let committed_pct = ((props.balance.used_days + props.balance.planned_days) / total)
    * 100.0;
let bar_pct = committed_pct.clamp(0.0, 100.0) as u32;
```
Use `bar_pct` for the CSS `width` to avoid negative widths. Add a separate overdraft
indicator — a red badge or overflow element — when `committed_pct > 100.0` (i.e.,
`used + planned > entitled + carryover`). Do NOT silently clamp and suppress the
overdraft; HR needs to see it at a glance.

**Warning signs:**
- Employee with 0 used days but 15 planned days (out of 20 entitled) sees a bar at
  0% but a number of 5 remaining — bar and number are visually contradictory.
- An employee in overdraft sees a bar at 100% with a negative remaining number — the
  bar does not signal the problem.
- The `low` threshold logic (`remaining_days <= 3.0`) correctly uses `text-warn` for
  the number, but the bar does not show overdraft for a different visual reason.

**Phase to address:**
Urlaubs-Balken-Konsistenz phase. The change is isolated to `PersonVacationCard` in
`absences.rs`. Regression: test `planned_days == 0` (bar unchanged), `total == 0`
(no division), and `used + planned > total` (overdraft indicator).

---

## High-Impact Pitfalls

### Pitfall 7: Absence Category Mismatch — Urlaub-Nicht-Verfuegbar Fires for Wrong Types

**What goes wrong:**
The Urlaub-Nicht-Verfuegbar feature marks days as "discouraged" in the shiftplan
grid based on the employee's own absence ranges. If the implementation filters by
`AbsenceCategory::Vacation` only, it misses absence types that should also block
scheduling (e.g. `SickLeave`). Conversely, if it uses ALL absence categories without
an explicit whitelist, it may mark days the planner wants to leave schedulable. The
feature description says "Vacation, ggf. weitere Kategorien" — the set is explicitly
TBD and must be settled before implementation.

**Why it happens:**
Feature descriptions prototype on "vacation" as the primary case. The Rust
`AbsenceCategory` enum has multiple variants. A catch-all `filter(|a| true)` silently
catches variants that were not evaluated in the design. A too-narrow
`filter(|a| a.category == AbsenceCategory::Vacation)` silently misses others.

**How to avoid:**
Define the allowed categories as an explicit `const` whitelist (not an inline
closure). Write a unit test that for each `AbsenceCategory` variant asserts whether
it produces a discourage marker. Gate the implementation on the DECISION being
recorded in the phase CONTEXT doc as `D-NN`. If the set may grow, make it a
runtime config rather than a const — but lock that decision in the phase plan.

**Warning signs:**
- SickLeave days are marked as Nicht-Verfuegbar without an explicit design decision.
- A new `AbsenceCategory` variant added in a future milestone silently does (or does
  not) produce discourage markers depending on whether the filter uses `match` with
  exhaustive arms or a predicate that defaults one way.

**Phase to address:**
Urlaub-Nicht-Verfuegbar phase. The category whitelist must be in the DISCUSS/CONTEXT
before planning starts.

---

### Pitfall 8: Impersonation Session Survives Browser Close — Indefinite Impersonation

**What goes wrong:**
The `app_session` cookie has a 1-year expiry. The session row persists
`impersonate_user_id` until `DELETE /impersonate` is called. If the admin closes
the browser tab while impersonating and reopens the application the next day, they
are still impersonating Alice. All reads and writes silently occur as Alice until
the admin notices the banner (if they see it — Pitfall 9).

**Why it happens:**
Session management was designed for login persistence. Impersonation piggybacks on
the same session row without a separate expiry field.

**How to avoid:**
Add `impersonate_expires_at: Option<OffsetDateTime>` to the session DAO table.
`start_impersonate` writes `now + 4h` (or a configurable timeout). `context_extractor`
calls `verify_user_session` which should clear (or report as expired) an impersonation
whose `impersonate_expires_at` is in the past. If a timestamp column is out of v1.9
scope, at minimum log a structured warning when `context_extractor` finds an
`impersonate_user_id` that was set more than N hours ago.

**Warning signs:**
- Admin reloads the app the next day and the impersonation banner appears with no
  recent `start_impersonate` call in the logs.
- Session rows in the DB have non-null `impersonate_user_id` with no associated
  `stop` event.

**Phase to address:**
Impersonation backend design phase. Session row schema change can be a migration
alongside the main impersonation tables if those do not yet exist.

---

### Pitfall 9: Impersonation Banner Missing After Page Reload — Silent Admin Actions

**What goes wrong:**
The frontend shows the "Du agierst als X — Impersonation beenden" banner only if
the Dioxus store holding `ImpersonateTO` reflects `impersonating: true`. If this
store is initialized to `ImpersonateTO { impersonating: false, user_id: None }` on
app mount (default) and is only updated by the `start_impersonate` success callback,
then after a page reload the store reverts to the default. The admin performs actions
thinking they are themselves but the session still has Alice's identity active.

**Why it happens:**
Dioxus global stores start from their `Signal::global(|| ...)` default. A page
reload reinitializes the WASM module and all stores. The impersonation state lives in
the backend session, not in the frontend signal.

**How to avoid:**
On application mount, call `GET /impersonate` to fetch current impersonation status
before any user-facing data load. Store the result in the global impersonation signal.
Render the banner when `impersonating == true`, regardless of how that state was set.
This call must execute before the first data loads (booking, absence, vacation) so the
admin always knows their identity when they see the data.

**Warning signs:**
- Hard-refresh while impersonating shows no banner and no indication of the
  impersonation state.
- The `ImpersonateTO` signal defaults to `{ impersonating: false }` without an initial
  HTTP check.

**Phase to address:**
Impersonation frontend phase. The `GET /impersonate` call must be in the app init
sequence, not triggered only by user action.

---

### Pitfall 10: Admin Locked Out of Admin Endpoints While Impersonating

**What goes wrong:**
While impersonating a non-admin, `context_extractor` sets effective context to the
impersonated user. Every service call gated by `check_permission("admin", context)`
via the middleware-resolved context will return 403. The admin cannot use the
permission-management page, cannot view all-users list, cannot assign roles — all
while they are impersonating. This is actually CORRECT behavior (admin acts as the
user, not as admin), but it will surprise the admin and look like a bug.

**Why it happens:**
Single-identity context cannot simultaneously express both roles. The design choice
was made when impersonation was read-only. The admin who needs to do admin work
while impersonating has no path.

**How to avoid:**
For v1.9: document this as an explicit known limitation in the phase DISCUSS. The
banner should say "While impersonating, your admin privileges are suspended. Stop
impersonation to use admin tools." Do NOT resolve this by granting admin privilege
bleed-through to the impersonated session. If future milestones need "admin tools
while impersonating" (e.g. to fix a permission for the impersonated user live), that
requires the dual-identity Context struct from Pitfall 1.

**Warning signs:**
- Admin impersonating Alice tries to assign a role to Alice and gets 403.
- The UAT scenario "admin fixes a permission issue while looking at the user's view"
  is impossible without stopping impersonation.

**Phase to address:**
Impersonation design phase — document as known limitation, not a bug to fix.

---

### Pitfall 11: Urlaub Data Fetched for Stale Sales Person Before `current_sales_person` Resolves

**What goes wrong:**
The `reload_unavailable_days` closure in `shiftplan.rs` reads `*current_sales_person.read()`
to decide whose absence periods to fetch:

```rust
if let Some(sales_person) = &*current_sales_person.read() {
    loader::load_unavailable_sales_person_days_for_week(..., sales_person.id, ...).await
}
```

`current_sales_person` is loaded by `loader::load_current_sales_person` inside the
coroutine init block. On first page load, if the week-change signal fires before the
coroutine init completes, `current_sales_person` is still `None` and the
absence-based discourage lookup is skipped silently. The Nicht-Verfuegbar markers do
not appear on first load.

The same issue exists for the new absence-range API call that the
Urlaub-Nicht-Verfuegbar feature will add: if the feature fetches absence ranges for
the current week, it must wait for `current_sales_person` to be non-None.

**Why it happens:**
The shiftplan coroutine initializes `current_sales_person` mid-stream after other
async calls. The week-change handler does not know whether `current_sales_person` has
been resolved yet.

**How to avoid:**
Ensure the absence-range fetch for the new feature is either:
(a) placed after the `current_sales_person` assignment in the coroutine init sequence,
    or
(b) triggered reactively by a signal that only fires when `current_sales_person`
    becomes non-None.

Add a test: simulate first page load with a mock that delays `current_sales_person`
resolution — verify the absence markers appear once the person loads, not missing
permanently.

**Warning signs:**
- Urlaub-Nicht-Verfuegbar markers do not appear on the first page load; appear only
  after a manual week switch.
- `reload_unavailable_days` is called while `current_sales_person` is `None` — the
  call is silently skipped with no error, no retry.

**Phase to address:**
Urlaub-Nicht-Verfuegbar phase.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Impersonation via cookie session without dual-identity Context struct | Avoids Context type refactor across all service call sites | Audit trail gap: all writes attributed to impersonated user; real admin identity invisible in DB records | Never for write-capable impersonation; acceptable for read-only only |
| Vacation bar `clamp(0, 100)` without overdraft indicator | No negative CSS width bug | HR cannot detect overdraft from card at a glance; bar and remaining number are visually contradictory | Never — the Balken-Konsistenz phase requires an overdraft indicator |
| Fetch absence data for Urlaub-Nicht-Verfuegbar via a new independent HTTP call per week change | Simpler, no backend batch change needed | One extra HTTP round-trip per week navigation; absence data slightly out of sync with the week payload batch | Acceptable for v1.9 if latency is unnoticeable; revisit in a later cleanup phase |
| Fix generation guard only on WEEKLY_SUMMARY_STORE, skip sibling loaders | Minimal diff for the stated bug | BookingConflict and unavailable-days loaders remain racy | Never — fix all three in the same phase |
| Impersonation banner state set only in start-success callback, not fetched on mount | No extra API call on startup | Silent impersonation after page reload; admin acts as wrong user | Never for write-capable impersonation |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `context_extractor` + new admin endpoint | Reading user from `Extension::<Option<Arc<str>>>` and calling `check_permission("admin", ...)` — this checks the impersonated user during impersonation | Read the raw session from the cookie directly and extract `session.user_id` (the real user), as done in all three handlers in `rest/src/impersonate.rs` |
| `session_service.start_impersonate` + permission check | Passing `context` from `context_extractor` to `check_permission("admin", ...)` — this uses the effective identity, not the real admin | Construct `Authentication::Context(Some(session.user_id.clone()))` from the raw session; this is already the correct pattern in `start_impersonate` |
| Dioxus GlobalSignal + async coroutine | Writing to signal unconditionally after `await` with no staleness check | Capture generation token before `await`; compare after `await`; discard result if generation changed |
| Vacation bar percentage + overdraft | `clamp(0, 100)` on `used_days/total` | Use `(used_days + planned_days) / total`; clamp width to 100 for CSS; render separate overdraft badge when `committed > total` |
| Absence fetch for current user + week change | Fetching absence ranges before `current_sales_person` has resolved | Gate absence fetch on `current_sales_person` being `Some`; retry when the signal fires non-None |
| Stop impersonation + Dioxus stores | Calling `DELETE /impersonate` then assuming stores auto-refresh | Explicitly send reload actions to every user-scoped store in the stop-impersonation success handler |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Using `context_extractor` identity for admin gate on impersonation-adjacent endpoints | Admin blocked by 403 on their own tools while impersonating, or worse: impersonated user gains admin access if check logic is inverted | Always use `session.user_id` (raw session read) for admin gates. Encapsulate in a `RealUserAuth` extractor |
| Not expiring impersonation server-side | Session with stale `impersonate_user_id` persists indefinitely; reads/writes silently attributed to wrong user across sessions | Add `impersonate_expires_at` to session table; enforce on `verify_user_session` or in `context_extractor` |
| Allowing admin to impersonate another admin | Admin A writes appear as Admin B; audit poisoning; if B has *more* privileges than A in some domain, escalation possible | In `start_impersonate`, verify target user does NOT have `admin` privilege; return 403 if they do |
| Impersonation state sourced only from frontend memory | Page reload loses banner; admin acts as impersonated user without visible indicator | Fetch `GET /impersonate` on every app mount; banner driven by backend state |
| Write path under impersonation not tested for audit attribution | Ships with no test coverage for the audit gap; audit trail is silently wrong in production | Add integration test: perform a write while impersonating; assert the `process`/`created_by` field contains the real admin ID (or the impersonated ID + a separate audit column for real actor, depending on chosen design) |

---

## "Looks Done But Isn't" Checklist

- [ ] **Impersonation write-path audit:** `created_by` / `process` in DAO writes performed
  under impersonation contains the real admin user ID, not the impersonated user ID.
  Verify in `booking_dao`, `absence_dao`, any other write DAO exercised in the
  impersonation phase.
- [ ] **Stop-impersonation store reload:** After `DELETE /impersonate` succeeds, ALL
  user-scoped Dioxus stores (vacation balance, weekly summary, `current_sales_person`,
  absence list) refresh to show the real admin's data. Not just the banner disappearing.
- [ ] **Banner on hard reload:** Reload the browser while impersonating — banner is
  visible without any user interaction. Requires `GET /impersonate` call on app mount.
- [ ] **Admin gate while impersonating:** Admin impersonating a non-admin calls a
  admin-only endpoint via the `context_extractor` path — correctly gets 403. Admin
  calls `DELETE /impersonate` — correctly gets 200. Both via the same session.
- [ ] **Vacation bar overdraft:** Create a scenario where `used_days + planned_days >
  entitled_days + carryover_days`. Verify an overdraft indicator appears (not just
  a bar at 100% with a negative number).
- [ ] **Week-race fix covers all three loaders:** After the generation-guard fix,
  rapid multi-week navigation shows correct data in (1) WEEKLY_SUMMARY_STORE headers,
  (2) booking-conflict highlights, (3) unavailable-day markers. All three, not just (1).
- [ ] **Absence category gate:** Unit test lists every `AbsenceCategory` variant and
  asserts whether it produces a discourage marker. No untested variant.
- [ ] **Urlaub marker on first load:** Navigate to shiftplan with an active vacation
  absence — markers appear on first render, not only after a week switch.
- [ ] **i18n for new text:** Impersonation banner, stop-impersonation button, overdraft
  label — all three locales (de/en/cs). The `Locale::De`/`Locale::En`-in-`de.rs` bug
  is a recurring issue (memory note); add per-locale reference tests for new keys.

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Audit trail loss shipped (P1) | HIGH | Context type refactor across all services + DAO audit-column backfill. Cannot be patched at REST layer only. Historical write records under impersonation are permanently mis-attributed. |
| Admin-gate wrong identity (P2) | MEDIUM | Narrow fix to affected handlers; add `RealUserAuth` extractor; no DB migration needed. Regression-test immediately. |
| Frontend stores stale after stop (P3) | LOW | Broadcast reload actions on stop. Page reload is a manual workaround until fix ships. |
| Stale-week race in sibling loaders (P5, if P4 fixed but P5 missed) | LOW | Isolated generation-guard fix per loader; no API change. |
| Vacation bar overdraft hidden (P6) | LOW | CSS + percentage formula change only; no backend change; no migration. |
| Wrong absence categories for Nicht-Verfuegbar (P7) | LOW | Change the whitelist constant; FE redeploy only. |
| Impersonation session indefinitely persists (P8) | MEDIUM | Add `impersonate_expires_at` migration; deploy; existing stale sessions need a one-time cleanup query. |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Audit trail loss (P1) | Impersonation backend design phase | Integration test: write booking/absence while impersonating; assert `process`/`created_by` contains real admin ID |
| Admin gate wrong identity (P2) | Impersonation backend design phase | Test: admin impersonating non-admin calls admin endpoint via middleware context — expect 403; calls `DELETE /impersonate` — expect 200 |
| FE stores stale after stop (P3) | Impersonation frontend phase | Test: start impersonation, assert store shows Alice's data; stop, assert stores show admin's data |
| Stale-week race — weekly summary (P4) | Stale-Daten-Race fix phase | Rapid `LoadWeek` sequences; assert store always holds last-requested week data |
| Stale-week race — sibling loaders (P5) | Stale-Daten-Race fix phase (same) | Same test class applied to `BookingConflictStore` and `unavailable_days` signal |
| Vacation bar vs number mismatch (P6) | Urlaubs-Balken-Konsistenz phase | Unit test `PersonVacationCard`: used=5, planned=10, entitled=20 — bar width >= 75%; overdraft case (used+planned=25, entitled=20) — overdraft indicator visible |
| Absence category mismatch (P7) | Urlaub-Nicht-Verfuegbar phase | Unit test: all `AbsenceCategory` variants enumerated with expected discourage=true/false mapping |
| Impersonation session lifetime (P8) | Impersonation backend design phase | Test: session with `impersonate_expires_at` in the past — `context_extractor` treats as no impersonation |
| Banner missing on reload (P9) | Impersonation frontend phase | Browser test: reload while impersonating — banner visible without user interaction |
| Admin locked out while impersonating (P10) | Impersonation design phase | Document as known limitation in DISCUSS; confirm in UAT |
| Stale `current_sales_person` for absence fetch (P11) | Urlaub-Nicht-Verfuegbar phase | Test: `current_sales_person = None` on mount — absence markers appear once person resolves, not missing permanently |

---

## Sources

- Direct read: `rest/src/session.rs` (`resolve_session_user_id`, `context_extractor` both variants)
- Direct read: `rest/src/impersonate.rs` (admin gate uses `session.user_id`, not effective context)
- Direct read: `service_impl/src/permission.rs` (`check_permission`, `check_user` — all paths through effective-user context)
- Direct read: `service/src/permission.rs` (`Authentication<Context>` type definition, `Context = Option<Arc<str>>`)
- Direct read: `shifty-dioxus/src/service/weekly_summary.rs` (unconditional write-after-await confirmed)
- Direct read: `shifty-dioxus/src/page/shiftplan.rs` lines 295-378 (sibling loader sends; `reload_unavailable_days` pattern)
- Direct read: `shifty-dioxus/src/page/absences.rs` `PersonVacationCard` (bar math: `used_days/total` clamped; number: `remaining_days` which includes planned)
- Direct read: `shifty-dioxus/src/state/vacation_balance.rs` (`used_days`, `planned_days`, `remaining_days` field definitions)
- Direct read: `shifty-dioxus/src/service/vacation_balance.rs` (store structure; no (year,week) key on VACATION_BALANCE_STORE)
- `.planning/todos/pending/2026-06-29-impersonate-feature-f-r-admins.md` (feature intent, security requirements, preferred mechanism)
- `.planning/todos/pending/2026-06-29-wochen-summary-karten-gegen-stale-daten-bei-schnellem-wochen.md` (confirmed root cause, sibling-loader warning, generation-counter suggestion)
- `CLAUDE.md` (service tier convention, clippy gate, audit trail context)
- `.planning/PROJECT.md` (v1.9 milestone scope; banner-not-dialog convention from v1.9 context)

---
*Pitfalls research for: Shifty v1.9 — impersonation RBAC write-path, Dioxus async race, vacation-bar math, absence discourage marker*
*Researched: 2026-06-29*
