# Authentication & Authorization

## Two auth modes

Shifty supports two auth modes, selected via a feature flag:

- **`mock_auth`** (dev) — a hard-wired admin user is injected on every
  request. No login, no role check in the session layer.
- **`oidc`** (prod) — OpenID Connect login against an external IdP.
  The session token is validated, the user + roles are taken from the IdP.

The feature flag decides at compile time which session layer gets wired
into `shifty_bin`.

## `Authentication<Context>`

The auth context that services receive is generic:

```rust
pub enum Authentication<C> {
    Authenticated(C),   // Concrete user context (user ID, roles)
    Full,               // All-rights bypass (internal only)
}
```

The context `C` is a trait that adapters (session layer for prod,
mock session for dev, test harness) implement.

## The `Full` bypass

`Authentication::Full` is the **all-rights bypass** for internal
aggregate calls. Behavior in `service_impl/src/permission.rs:28,41,63,80,90`:

```rust
match auth {
    Authentication::Full => Ok(()),   // All checks succeed immediately
    Authentication::Authenticated(ctx) => { /* real check */ }
}
```

### When to use `Full`

**Only** for internal aggregate reads by Business-Logic Services:

- `ReportingService` reads `SalesPersonService::get`, `BookingService::list`,
  `ExtraHoursService::list`, `AbsenceService::list`, `CarryoverService::get`
  with `Full`, because the user context has already been checked in the
  REST handler and the internal reads should not each re-verify everything
  on their own.
- Example references:
  - `service_impl/src/scheduler.rs:60,68` —
    `update_carryover_all_employees(year-1, Full)` (batch job has no
    user context).
  - `service_impl/src/extra_hours.rs:51-54` —
    `custom_extra_hours.get_by_id(key, Full)` (internal definition lookup).
  - `service_impl/src/sales_person_shiftplan.rs:65,92` — internal
    sales-person reads.

### When NEVER to use `Full`

- **In REST handlers.** The user context from the session must be passed
  through. `Full` in a REST handler would be a catastrophic
  auth bypass.
- **Counter-example:** `service_impl/src/pdf_shiftplan.rs:21`
  explicitly documents: "(D-49-07); never internally escalate to
  `Authentication::Full`." The export chain passes the user auth all
  the way down to the data layer.

### The Full bypass for toggle reads (Phase 51)

**[Verified via memory + test file]** The `ToggleService` had a guard
that blocked Full reads with a user-ID guard
(`service_impl/src/test/toggle.rs:547-556`). That was a bug because
Reporting and BookingInformation read the toggle with `Full`. The
gap closure in Phase 51 ensures that Full reads for toggles bypass the
guard.

Consequence for new services: **read ops must accept `Full`**,
otherwise internal aggregates will break.

## Roles & privileges

The role definitions evolve across multiple migrations:

- `20240426150045_user-roles.sql` — base roles.
- `20240614075633_shiftplanner-role.sql` — shiftplanner role
  added.
- `20241118165756_add-role-shiftplan-edit.sql` — finer read/edit
  split for shiftplan.

**[To verify]** — the exact enumeration of all roles and which
privileges they carry. See
[`../features/F12-auth-session.md`](../features/F12-auth-session.md).

## Session management

- **Session table:** migration `20241116224840_add-session.sql`,
  constraint tightening `20241118180147_make-session-id-not-null.sql`.
- **Session per login:** on login a session row is created,
  session ID lives in a cookie. On logout / expiry it is invalidated.
- **[To verify]** exact refresh behavior on token expiry.

## User invitation

There is an invitation flow for new users:

- Migration `20251016154210_add-user-invitation-table.sql`.
- Extensions: `20251017044013_add-session-tracking-to-user-invitation.sql`,
  `20251020000000_add-session-revoked-at-to-user-invitation.sql`.

Details in [`../features/F10-templates-communication.md`](../features/F10-templates-communication.md).

## Related edge cases

- Full-bypass misuse → [`../domain/edge-cases.md#6-authentifizierung--autorisierung`](../domain/edge-cases.md#6-authentifizierung--autorisierung)
- Token expiry, mid-session role changes → same section.
- Invitation link redeemed multiple times → same section.
