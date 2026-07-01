# Feature Landscape — v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation

**Domain:** HR + Shift-Planning SaaS (Milestone v1.9)
**Researched:** 2026-06-29
**Overall confidence:** MEDIUM (cross-checked across Deputy, When I Work, Dayforce,
Small Improvements, PropelAuth, yaro-labs; all consistent on core conventions)

---

## Scope Note

This file covers the four v1.9 features only. Existing features (absence CRUD,
vacation balance computation, shiftplan booking + warnings, paid-capacity enforcement,
holidays) are NOT re-researched.

---

## Feature A: Urlaub → Nicht-Verfügbar (Absence-as-Grid-Discourage)

### Background

Current state: Absence dates only surface as a `BookingOnAbsenceDay` warning when a
shift is actually booked. The grid itself does not proactively mark absence days as
discouraged. The `discourage` mechanism already exists, driven by
`sales_person_unavailable` (recurring weekday rules). Absence date ranges are a
separate data path that currently does not feed the discourage signal.

Industry norm: every major scheduling tool (Deputy, When I Work) proactively marks
absence/leave days in the grid before any booking attempt. Showing only a post-hoc
warning is below the norm.

### Table Stakes

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Approved absence dates appear as visually distinct cells in the scheduling grid before booking | Deputy: solid red; WhenIWork: grey bar — proactive display is universal | Low-Med | Reuse existing `discourage` render path; extend data source to include absence date ranges |
| Vacation absences are included (minimum required category) | Vacation is the most common; user reported this as missing | Low | Mandatory baseline |
| SickLeave absences are included | Sick employees also cannot work | Low | Natural extension; no reason to exclude |
| UnpaidLeave absences are included | Same rationale; absence data model already has the category | Low | Include by default; cost is zero |
| The visual treatment is identical to the existing `sales_person_unavailable` discourage cell | Consistency — scheduler recognizes the pattern from existing use | Low | Reuse the same Tailwind classes / cell rendering already in `week_view.rs` |
| Absence-based discourage is date-specific (concrete NaiveDate), not day-of-week | Vacation is a date range, not a recurring weekly pattern | Med | Current `discourage_weekdays` model is weekday-based; extend or add parallel `discourage_dates: HashSet<NaiveDate>` |

### Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Hover tooltip showing absence type on discourage cell | Zero-click info: scheduler sees "Vacation 2026-07-01..07-05" without navigating away | Low | Small Tailwind tooltip on the discourage cell div |
| Visually distinguish absence-sourced discourage from recurring-rule discourage | Scheduler immediately knows if it is "every Monday rule" vs a specific vacation | Med | Different color or indicator glyph; adds a second visual variant to the discourage system |
| Pending (unapproved) absence shown as lighter/striped vs approved (solid) | Deputy/WhenIWork both distinguish pending vs approved | Med | Shifty has no absence-approval workflow today; defer until approval model exists |

### Anti-Features

Features to explicitly NOT build in v1.9.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Hard-blocking shift booking on absence days | That is a separate policy toggle analogous to `paid_limit_hard_enforcement`; it is not what the todo asks for | Keep soft discourage; the existing `BookingOnAbsenceDay` warning already fires at booking time |
| Showing another employee's absence in the grid cell of a different employee | Absence is per-person; cross-person display creates confusion | Each person's own absence dates are discouraged only in their own row |
| Full absence-approval workflow in v1.9 | Large orthogonal feature | Keep existing absence model; approval is a future milestone |
| Loading all absences for the whole year upfront | Performance cost for large teams | Load only the absences for the displayed week, same scope as existing data loads |

### Feature Dependencies

```
Absence CRUD API (v1.0, done)
  └──provides──> absence date ranges per person per year

booking_information.rs::get_weekly_summary (done, v1.7)
  └──already returns──> absence data per week per person
  └──check──> whether absence dates are already in BookingInformationTO / week payload
              reachable by the frontend for the displayed week

Existing discourage_weekdays render (week_view.rs:975-1065)
  └──extend to──> also check discourage_dates set for the specific NaiveDate of each cell

shiftplan.rs:1120-1123 (discourage_weekdays construction site)
  └──extend to──> add discourage_dates: HashSet<NaiveDate> built from loaded absence ranges
```

**Key question to resolve in discuss-phase:** Does the current frontend weekly data
load already include the absence date ranges for the viewed week, or does an additional
fetch / field in the API payload need to be added? `booking_information.rs:70-99` is
the candidate; if absence dates already flow through `BookingInformationTO` this is a
frontend-only change.

---

## Feature B: Urlaubs-Balken-Konsistenz (Vacation Bar Consistency)

### Background

Current state: the bar in `PersonVacationCard` (absences page) shows
`used_days / (entitled + carryover)`, clamped 0-100%. The number next to it shows
`remaining = entitled + carryover − used − planned`. They measure different things.
Example: entitled+carryover=18, used=6 → bar=33%; planned=13 → remaining=−1. A user
sees "−1 remaining" next to a "33% full" bar. This is actively misleading.

Industry norm (Dayforce, WhenIWork, UX case studies): bar and number must measure the
same quantity. Overdraft must be visible, not silently clamped.

### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Bar and number measure the same quantity | Every HR tool surveyed follows this rule; mismatched indicators break trust | Low | Pure formula change; all data already in `VacationBalance` frontend state |
| Bar shows `(used + planned) / (entitled + carryover)` | This matches `remaining = total − used − planned`; consistent with the displayed number | Low | One-line formula change in `absences.rs:865-871` |
| Overdraft (remaining < 0) is visible in the bar, not hidden by a 100% clamp | Dayforce uses an "Exceeded" column; tools use warning color + full bar; clamping hides a real problem | Low | Remove the `f64::min(1.0)` clamp; when `used + planned > total`, render bar full in `bg-warn`/`bg-error` |
| Warning color fires on overdraft | Already fires when `remaining_days <= 3.0`; will naturally fire for negative values if clamp is removed | Low | No change needed if clamp is removed; verify the condition covers negative values |

### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Two-segment bar: used (solid) + planned (lighter / distinct color) | Immediately shows how much is confirmed taken vs upcoming scheduled — more information per pixel | Med | Two adjacent `div` elements with `used_pct` and `planned_pct` widths; total width still `(used+planned)/total` |
| Overdraft overflow overflow visual: bar continues past 100% mark in warning color | Visually striking for HR scanning many people; makes overdraft unmissable | Med | CSS trick: `overflow: visible` on container + absolute-positioned overflow segment |

### Anti-Features

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Separate bars for used and planned side-by-side | Clutters the tight `PersonVacationCard` row layout | Single bar with two segments via adjacent divs |
| Adding pending-vs-approved distinction within planned_days | Shifty has no absence approval workflow; `planned_days` is a single figure from the backend | Use backend's single `planned_days` figure; no subdivision needed |
| Bar animation on load | Distracting in a list of many people | Static fill |
| Changing the backend API | All fields are already present in `VacationBalance` (`used_days`, `planned_days`, `remaining_days`, `entitled_days`, `carryover_days`) | Frontend-only change |

### Feature Dependencies

```
Vacation balance API (v1.5, done)
  └──provides──> used_days, planned_days, remaining_days, entitled_days, carryover_days

VacationBalance frontend state (done)
  └──already holds──> all needed fields

PersonVacationCard component (absences.rs:843-898)
  └──change──> formula + remove clamp + conditional warning color
```

**This is a pure frontend change. No backend work needed.**

---

## Feature C: Stale-Daten-Race (Week-Summary Stale Guard)

### Background

Current state: rapid week-switching can show stale data because an earlier async
response for week N-1 arrives after the user has navigated to week N, overwriting the
correct week-N data. The visible symptom is summary cards showing last week's numbers
under this week's header.

### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Summary cards always show data for the currently-selected week | Basic correctness; any async UI framework handles this with a generation counter or cancel-on-stale | Low-Med | Dioxus 0.6 async model does not expose HTTP abort cleanly; generation token is the correct approach |
| Rapid navigation does not produce partial/mixed state (last week's N + this week's M) | Mixed state is worse than a loading state | Low | Generation token: discard response if `(year, week)` no longer matches current signal at response-write time |

### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Debounced week navigation: wait ~100-200ms after last click before firing the fetch | Prevents N requests for N rapid arrow-key presses | Low | `use_signal` + timeout-based debounce in the event handler |
| Keep previous week's data visible (reduced opacity) while next week is loading | Avoids blank flash; data is stale but visually present | Low | Don't clear state on navigation; only update on response commit |

### Anti-Features

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Canceling in-flight HTTP requests (AbortController equivalent) | Dioxus 0.6 async coroutine model does not expose this pattern cleanly; request still runs server-side anyway | Generation token: discard stale responses on the receive side |
| Loading spinner on every week change | Flickers on fast navigation; hurts perceived performance | Show spinner only after a timeout threshold (e.g. 300ms); or just discard stale silently |
| Re-architecting the data loading pattern for all pages | Out of scope; fix the week-summary path only | Surgical fix in the affected coroutine / signal |

### Feature Dependencies

```
Existing week-selector signal (done)
  └──wrap with──> generation token Signal<(year, week)>

Existing summary card data-fetch coroutine
  └──add guard──> capture (year, week) at dispatch; compare at response-write; discard if mismatch
```

**This is a pure frontend change. No backend work needed.**

---

## Feature D: Admin-Impersonation (Read + Write)

### Background

Current state: no impersonation exists. Admins must log in as another user (sharing
credentials) to reproduce a bug or see another employee's view. The todo asks for a
proper impersonation feature: admin acts as user, writes are allowed, audit trail
preserved, and the admin sees a banner with an exit button.

Industry norm (PropelAuth, Small Improvements, Deskera, Yaro Labs): impersonation is
standard in HR SaaS support/admin tooling. All surveyed tools use a persistent top
banner, full read+write, no privilege escalation, and audit logs carrying real identity.

### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Persistent "Acting as [Name] — Stop Impersonation" banner on every page during impersonation | Industry-standard; every tool surveyed has this. Non-dismissible. Yellow/amber color convention. | Med | Global Dioxus component in app root layout; reads impersonation state signal |
| Banner has a one-click exit mechanism | Canonical exit pattern across all tools; no multi-step confirmation needed | Low | Button fires DELETE /impersonation or clears the impersonation context signal |
| Start impersonation from the admin/HR person list | Canonical entry point ("Manage → Impersonate" on person row/card) | Low | Add action item to existing person list in admin/HR view |
| Privilege escalation prevention: impersonated user's permissions apply, not admin's | Core security invariant. PropelAuth + Small Improvements: admin privileges do NOT bleed through | Med | PermissionService must resolve roles of the impersonated user, not real caller; check fires on every request, not only at session start |
| Real admin identity preserved in audit log for all impersonated actions | Industry norm: other admins can see who really acted. Tag every mutating request with `impersonation_context { real_admin_id }` | Med | Axum request extension or `Authentication<Context>` carries dual identity; logging layer reads real actor |
| Admin-only gate on start/stop impersonation endpoints | No other role can enter impersonation mode | Low | `require_privilege(admin)` check on both endpoints |
| i18n de/en/cs for banner text and start/stop labels | Project convention; Shifty supports three locales | Low | ~6 new locale strings |
| Cannot impersonate oneself | Trivial correctness guard | Low | `target_id != real_admin_id` check at start |

### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Visual "impersonation mode" color tint or border on page (beyond banner) | Makes the mode unmistakable even when scrolled past the banner | Med | CSS body class or Tailwind ring on main container; optional |
| Impersonation session auto-timeout (15-60 min) | Security best practice in enterprise tools; yaro-labs recommends 15-60 min | Med | Defer to v2.0+; manual exit sufficient for v1.9 |
| Audit log UI visible to other admins (searchable list of past sessions) | Compliance feature for regulated industries | High | Defer; backend logging in structured logs is sufficient for v1.9 |

### Anti-Features

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Read-only impersonation as the v1.9 default | The todo explicitly states read + write; read-only would not cover the "support reproduces and fixes an issue" use case | Full read+write; restrict only irreversible destructive actions if desired |
| Blocking ALL write operations during impersonation | Defeats the stated purpose | Allow all normal writes; consider blocking only account deletion or password change if those endpoints exist |
| Token-swapping / JWT re-issuance for impersonation | Loses the real admin identity for audit purposes; hard to revoke; complex in OIDC production context | Server-side impersonation context: extend `Authentication<Context>` with `impersonated_sales_person_id: Option<Uuid>` set in a backend session/endpoint, not in the token |
| Impersonating another admin | Admins acting as admins is an escalation risk and typically disallowed | Gate: can only impersonate non-admin users; or require a separate confirmation |
| Real-time notification to the impersonated user | No tool surveyed notifies the target; it would cause support friction and confusion | Audit log visible to other admins; no user notification |
| Separate "view as user" (read) and "act as user" (write) endpoints | Unnecessary complexity for v1.9; the unified approach with a clear banner is sufficient | Single impersonation mode that allows both reads and writes, controlled by the normal role-based permission check of the impersonated user |

### Security Invariants (non-negotiable)

1. The impersonated user's privilege set governs all permission checks during the session — admin privileges do NOT carry through.
2. Every mutating request during impersonation is logged with the REAL admin identity as the actor, not the impersonated user's identity.
3. Cannot impersonate a user with higher effective privileges than the impersonated user's own role would grant.
4. Permission check on impersonation start AND on each subsequent request (privilege revocation must take effect immediately, not only at session start).
5. Impersonation state is server-side (not in a JWT/cookie the browser controls alone), so the admin cannot self-escalate by manipulating client state.

### Feature Dependencies

```
Authentication<Context> (exists: service/src/permission.rs)
  └──extend with──> impersonated_sales_person_id: Option<Uuid>
                    (set only when real caller has admin privilege)

PermissionService (exists: service_impl/src/permission.rs)
  └──change──> when impersonation active, resolve roles from impersonated user
  └──keep──> real caller identity for audit/logging path

REST auth layer (exists: rest/src/lib.rs)
  └──add──> read impersonation context from server-side store
  └──build──> Authentication<Context> with dual identity

New backend endpoints:
  POST /impersonation  { target_sales_person_id: Uuid }
    └──gate: require admin privilege on real caller
    └──store: session-scoped impersonation state (in-memory or DB row)
    └──return: updated session token or session cookie
  DELETE /impersonation
    └──gate: require active impersonation session
    └──clear: impersonation state

Admin/HR person list page (Dioxus, exists)
  └──add──> "Impersonate" action item per person row (admin-only, hidden for others)

App root layout (Dioxus)
  └──add──> impersonation banner component
  └──reads──> global impersonation signal (set from API response on start/stop)
  └──renders──> banner only when impersonated_sales_person != None
```

---

## MVP Recommendation for v1.9

Ship in this order (dependency-free → dependent, simple → complex):

1. **Feature B — Urlaubs-Balken-Konsistenz**: pure frontend, formula + clamp removal.
   Delivers a visible correctness fix fastest. 1-2 hours.
2. **Feature C — Stale-Daten-Race**: pure frontend, generation-token guard.
   Small, low-risk, self-contained. 2-4 hours.
3. **Feature A — Urlaub → Nicht-Verfügbar**: frontend-primary (+ possible light
   backend check). Medium complexity. Vacation mandatory; SickLeave/Unpaid natural
   inclusions at zero extra cost.
4. **Feature D — Admin-Impersonation**: largest scope; backend auth changes + new
   endpoints + frontend banner. Build last so simpler fixes ship independently.

**Defer from v1.9 (confirmed anti-features / scope traps):**
- Absence approval workflow (pending vs approved distinction in grid)
- Impersonation session auto-timeout
- Audit log UI for admins
- Two-segment bar (build single corrected bar first; upgrade to split as follow-on)
- Overflow visual for overdraft bar (implement if single-segment bar looks insufficient)

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Impersonation UX conventions (banner, audit, privilege) | MEDIUM | Consistent across 4+ sources; specifics of Shifty's auth model require code-level verify |
| Vacation bar conventions (formula, overdraft) | MEDIUM | Dayforce + WhenIWork + case study all agree; implementation is frontend-only |
| Absence-as-grid-discourage conventions | MEDIUM | Deputy + WhenIWork both confirm proactive grid marking; frontend integration path needs code verify |
| Stale-race guard pattern | MEDIUM | Standard async UI pattern; Dioxus 0.6 specifics need code verify for generation-token placement |

---

## Sources

- [Yaro Labs — Building a Safe User Impersonation Tool for SaaS](https://yaro-labs.com/blog/user-impersonation-tool-saas)
- [Small Improvements — User Impersonation](https://intercomdocs.small-improvements.com/en/articles/9146194-user-impersonation)
- [PropelAuth — User Impersonation Docs](https://docs.propelauth.com/overview/user-management/user-impersonation)
- [Zarana Solanki — Secure User Impersonation in Multi-Tenant Apps](https://medium.com/@codebyzarana/building-a-secure-user-impersonation-feature-for-multi-tenant-enterprise-applications-21e79476240c)
- [When I Work — Interpreting Availability on the Schedule](https://help.wheniwork.com/articles/interpreting-availability-on-the-schedule-computer/)
- [Deputy — Leave Management Software](https://www.deputy.com/features/leave-management)
- [Deputy — Manager Awareness of Leave](https://help.deputy.com/hc/en-au/articles/4658289483023-Manager-s-awareness-of-leave)
- [Dayforce — Your Balances (Leave balance visualization)](https://help.dayforce.com/r/documents/Employee-Guide/Your-Balances)
- [Paul Naylor — UX Case Study: Time Off Management App](https://medium.com/@pnaylor09/a-ux-case-study-on-designing-a-time-off-management-web-app-8b3151fa397d)
- OWASP CD-SEC-02: Account Impersonation — privilege escalation prevention

---
*Feature research for: v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation*
*Researched: 2026-06-29*
