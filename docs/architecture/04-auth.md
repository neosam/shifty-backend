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

## Self-access gate for sales-person data

Most employee-facing reads and writes are gated as **HR ∨ self**: either the
caller holds `HR_PRIVILEGE`, or the caller *is* the sales person the request
is about. The "self" half is
`SalesPersonService::verify_user_is_sales_person(sales_person_id, ctx, tx)`
(`service_impl/src/sales_person.rs`). Callers combine the two halves with
`tokio::join!` and `hr.or(sp)?` — an HR user therefore bypasses the self gate
through their privilege, and only a non-HR caller actually depends on it
(e.g. `service_impl/src/absence.rs:109-119`).

`verify_user_is_sales_person` resolves the sales person's login through
`SalesPersonDao::get_assigned_user` and compares it against
`PermissionService::current_user_id`. If either side is `None`, the call
returns `ServiceError::Forbidden` — fail-closed.

**The match itself happens in SQL, over `person_id`** (v2.7, Phase 61
[D-61-08]). Since the 1:1 link table was dissolved by
`migrations/sqlite/20260917000000_drop-sales-person-user.sql`, all four
reading link paths (`find_by_user`, `find_sales_person_by_user_id`,
`get_assigned_user`, `get_all_user_assignments` in
`dao_impl_sqlite/src/sales_person.rs`) resolve through one join:

```sql
SELECT u.name AS user_id
FROM sales_person sp
JOIN user u ON u.person_id = sp.person_id
WHERE sp.id = ?
```

**Convention — the identity comparison belongs in SQL, not in Rust.** This is
a permanent rule, not a phase note. Both `sales_person.person_id` and
`user.person_id` are nullable at the schema level, so a Rust-side comparison
of the two values would be an `Option<Uuid>` comparison, and `None == None`
evaluates to `true`. Two *unlinked* rows would then match each other and the
gate would go **fail-open** — a stranger reading someone else's bookings and
pay data. SQL's three-valued logic never lets `NULL = NULL` be true, so the
join simply yields no row and the gate closes. Any future rework of these
paths must keep the decision inside the query.

Regression guards: `service_impl/src/test/person_self_gate.rs` runs against a
real in-memory SQLite database (deliberately **not** a DAO mock — a mock
executes no SQL and could not test this property at all). The three core
cases are `self_may_access_own_sales_person` (self may),
`foreign_sales_person_rejected` (a stranger may not) and
`unlinked_user_rejected` (an unlinked user may not); the fail-open case above
is locked by `null_person_id_does_not_match_null_person_id`.

Note that **none** of the four paths filters `deleted IS NULL`, and none may
start to: a soft-deleted sales person keeps self-access to their own history
(`soft_geloeschte_sales_person_behaelt_selbstzugriff`).

## User write paths on the shared transaction (v2.7, Phase 63)

Creating and deleting a login user became **composable** in Phase 63. Both
write paths moved from the pool onto the transaction handed in by the caller:

```rust
// dao/src/permission.rs
type Transaction: crate::Transaction;

async fn create_user(&self, user: &UserEntity, person_id: Option<Uuid>,
                     process: &str, tx: Self::Transaction) -> Result<(), DaoError>;
async fn delete_user(&self, username: &str, tx: Self::Transaction) -> Result<(), DaoError>;
```

Two things changed, and one deliberately did not.

**1. The transaction is now a parameter, not an internal detail.** Before,
`create_user` opened a method-internal `pool.begin()` and `delete_user` wrote
straight onto the pool — both invisible to the calling service and therefore
not roll-back-able from the outside. The person administration needs the user
write to sit in the *same* transaction as the sales-person write (so that
"exactly the selected objects exist afterwards" can hold) and in the same
transaction as the cleanup of the person that just lost its last carrier. The
caller now decides about commit and rollback. See
[`05-transactions.md`](./05-transactions.md).

**2. `create_user` can adopt an existing person identity.** `person_id:
None` keeps the old behaviour — a fresh `person` row is created alongside
[D-61-01]. `person_id: Some(id)` hangs the user onto the **existing** person
and creates no second row. That is what lets one dialog create a user *and* a
sales person on the same identity.

**3. What did not change: the surface of `PermissionService`.** The service
trait carries **no** `Transaction` associated type and **no** `tx` parameter —
and it must not grow one. Roughly twenty services hold `PermissionService` as
a dependency; an associated type there would be a workspace-wide ripple
without benefit, and every one of those services would have to name a
transaction type it does not use. The associated type therefore sits one
layer lower, on `dao::PermissionDao`. `PermissionServiceImpl` gained a third
dependency (`TransactionDao`) to open a transaction for itself and commit it
at the end of `create_user` / `delete_user`; from the outside the service
behaves exactly as before, and the admin gate is still its first statement.

**Documented limitation — deleting a user with an open session.**
`session.user_id` is the only foreign-key relation pointing at `user(name)`
**without** a cascade (migration `20241116224840_add-session.sql`). Roles and
open invitations are taken along by their cascades; an open session is not.
Deleting such a user therefore runs into a foreign-key error and surfaces at
the endpoint as a generic **500** rather than a speaking rejection. Known,
pre-existing, and deliberately not fixed inside Phase 63 — a fix is a schema
change and belongs in its own change.

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
