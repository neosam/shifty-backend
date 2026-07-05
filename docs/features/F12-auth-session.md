# Feature: Auth & Session — OIDC/Mock, Permissions, Impersonation

> **In short:** Secures all Shifty endpoints — in production via OIDC, in
> dev via mock Session; every request receives a user `Context` against
> which roles and privileges are checked. Admins can temporarily assume
> the identity of other users for support purposes (Impersonation).

**Cluster ID:** F12
**Status:** in production
**First introduced:** base role set 04/2024 (`20240426150045_user-roles.sql`),
Sessions 11/2024 (`20241116224840_add-session.sql`), Impersonation 04/2026
(`20260401000000_add-impersonate-to-session.sql`)
**Responsible crates:** `service::permission`, `service::session`,
`service::user_service`, `service_impl::permission`, `service_impl::session`,
`service_impl::lib::UserServiceImpl`, `dao::permission`, `dao::session`,
`rest::session`, `rest::permission`, `rest::impersonate`, `rest::dev`,
`shifty-dioxus::service::auth`, `shifty-dioxus::page::not_authenticated`

---

## 1. What is it? (Business perspective)

Auth & Session is Shifty's central access layer. For every HTTP request
it decides: *Who is the caller, is there a valid Session, and may they
see or change the requested aggregate?*

- In **production**, Shifty runs against an external OIDC provider
  (Keycloak/Authelia). After a successful login, the backend writes a
  Session ID as an HTTP-only cookie and persists the Session in SQLite.
- In **dev/test**, a mock variant exists: as soon as a request arrives
  without a Session cookie, a `DEVUSER` is automatically created and a
  Session cookie is set — login-free work in the browser
  (`rest/src/session.rs:180-260`).
- **Roles and privileges** determine which functions are open to a user:
  `admin`, `hr`, `sales`, `shiftplanner`, `shiftplan.edit`.
- **Impersonation** is the support mode: an admin may temporarily work
  "as" another user (typical use case: view another user's reports or
  reproduce bugs). The original admin remains identifiable for audit
  logs (`RealUser` extension).

**Example workflow from an admin's perspective:**

1. Admin opens `/user-management`, searches user "anna" in the Users tab.
2. Clicks "Work as anna" — the backend sets `session.impersonate_user_id = "anna"`.
3. From now on the admin sees Shifty as anna would (her reports, her
   balance). All write requests are logged with `real_user = ADMIN,
   acting_as = anna`.
4. Admin clicks "Back to my identity" — `impersonate_user_id = NULL`,
   the normal admin view is restored.

---

## 2. Business Rules

- **Rule A (session cookie):** the cookie `app_session` (HTTP-only,
  SameSite=Strict, Secure) carries the UUID of the Session; server-side
  it is resolved against the `session` table
  (`rest/src/session.rs:137-176`).
- **Rule B (OIDC gate):** in the OIDC build, `forbid_unauthenticated`
  (`rest/src/session.rs:270-293`) blocks all requests without a resolved
  `Context`; only `/authenticate` and `/*/ical` are open.
- **Rule C (mock bypass):** in the mock build,
  `forbid_unauthenticated` lets every request through
  (`rest/src/session.rs:262-269`) — a Session is created automatically
  for `DEVUSER` when needed.
- **Rule D (session lifetime):** `expires = created + 3600·24·365`
  (365 days, hardcoded in `service_impl/src/session.rs:29`). The cookie
  expiry matches (`+ time::Duration::days(365)` in
  `rest/src/session.rs:116, 217, 241`).
- **Rule E (role enumeration):** the base roles `admin`, `sales`, `hr`
  are set in bootstrap (`20240426150045_user-roles.sql:100-112`),
  `shiftplanner` was added in 06/2024
  (`20240614075633_shiftplanner-role.sql`),
  `shiftplan.edit` in 11/2024
  (`20241118165756_add-role-shiftplan-edit.sql`).
- **Rule F (privilege derivation):** a user has a privilege if one of
  their assigned roles carries that privilege (join in
  `permission_dao.has_privilege`, consumed by `check_permission`, see
  `service_impl/src/permission.rs:35-55`).
- **Rule G (admin may do anything):** all admin operations (create
  roles, create users, assign user roles) are gated with
  `check_permission("admin", …)`
  (`service_impl/src/permission.rs:126, 140, 162, 177, 191, 199, 213, 231, 239, 255, 267, 279, 291`).
- **Rule H (`hr` privilege for user existence):** `user_exists` requires
  `hr`, not `admin` (`service_impl/src/permission.rs:150`) — HR roles
  may check whether a user is known without being admin.
- **Rule I (`Authentication::Full` bypass):** internal callers with no
  real user context (Scheduler, aggregation services, dev seed) may pass
  `Authentication::Full`; all `PermissionService` methods with a Full
  branch return `Ok(())` or `Ok(None)` immediately
  (`service_impl/src/permission.rs:28, 41, 63, 80, 90`).
- **Rule J (Impersonate is admin-only):** all three impersonate
  endpoints check `admin` against the *real* `session.user_id`, not
  against the effective Context
  (`rest/src/impersonate.rs:67-72, 136-141, 192-198`).
- **Rule K (audit for impersonated writes):** while a Session
  impersonates, every mutating HTTP verb (POST/PUT/PATCH/DELETE) is
  logged with `real_user`, `acting_as`, `method`, `path` via
  `tracing::info!` (`rest/src/session.rs:40-87`).

---

## 3. Data Model

### Tables

| Table | Purpose | Important columns |
| --- | --- | --- |
| `user` | User master (username = PK) | `name`, `update_timestamp`, `update_process` |
| `role` | Roles (name = PK) | `name`, `update_process` |
| `privilege` | Privileges (name = PK) | `name`, `update_process` |
| `user_role` | N:M user↔role | `user_name`, `role_name` (UNIQUE) |
| `role_privilege` | N:M role↔privilege | `role_name`, `privilege_name` (UNIQUE) |
| `session` | Active Sessions | `id` (PK, UUID), `user_id` (FK→user), `expires`, `created`, `impersonate_user_id` |

### Migrations (chronological)

- `20240426150045_user-roles.sql` — base tables `user`, `role`,
  `privilege`, `user_role`, `role_privilege` with update triggers;
  bootstrap of roles `admin/sales/hr` and the identically named
  privileges. Also the helper view `V_UUID_V4`.
- `20240614075633_shiftplanner-role.sql` — adds role + privilege
  `shiftplanner`; `admin` inherits it automatically via `role_privilege`.
- `20241116224840_add-session.sql` — `session` table (initially with
  `id` nullable — see follow-up migration).
- `20241118165756_add-role-shiftplan-edit.sql` — adds role + privilege
  `shiftplan.edit`; `admin` inherits it.
- `20241118180147_make-session-id-not-null.sql` — rebuild of `session`
  table with `id TEXT NOT NULL PRIMARY KEY`. Data is carried over via
  SELECT/INSERT.
- `20260401000000_add-impersonate-to-session.sql` — `ALTER TABLE session
  ADD COLUMN impersonate_user_id TEXT NULL`. Enables support
  Impersonation without rebuilding Sessions.

Further migrations for user invitation live in cluster **F13 (User
Invitation)** and are deliberately not listed here.

### Relationships

```
user  ─┬─< user_role >─┐
       │               │
       └──< session    └─> role ─< role_privilege >─ privilege
```

All FKs are `ON DELETE CASCADE` in the join tables, so deleting a user
also removes their role assignments. The `session` FK on `user` is
**not** CASCADE — deleting a user with an active Session would fail.
**[To verify]** whether this ever happens in production.

## 4. Service API

### 4.1 `Authentication<Context>` enum

`service/src/permission.rs:49-60`:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Authentication<Context: Clone + PartialEq + Eq + Send + Sync + Debug + 'static> {
    Full,
    Context(Context),
}
```

Two variants:

- **`Authentication::Full`** — "trusted internal caller". No user, no
  role check, all permission gates are skipped. Used by Schedulers
  (`service_impl/src/scheduler.rs:60`, `pdf_export_scheduler.rs`
  lines 220/251/279/293/302/317/335/344/373/381/391/418/436),
  aggregation services (`service_impl/src/extra_hours.rs:51,54`) and
  the dev seeder (`rest/src/dev.rs:128,149,191`).
- **`Authentication::Context(ctx)`** — carries the real user context.
  For the REST layer `Context = Option<Arc<str>>`
  (`rest/src/session.rs:12`) — the resolved username from the Session
  cookie or `None` on missing auth. Via `From<Context>`
  (`service/src/permission.rs:54-60`), every `Context` is automatically
  wrapped into `Context(ctx)`, which explains the `context.into()`
  pattern in all REST handlers.

### 4.2 `PermissionService`

`service/src/permission.rs:64-169`. `Context: Clone + PartialEq + Eq + Debug + Send + Sync + 'static`.

```rust
async fn check_permission(&self, privilege: &str, ctx: Authentication<Self::Context>)
    -> Result<(), ServiceError>;
async fn check_only_full_authentication(&self, ctx: Authentication<Self::Context>)
    -> Result<(), ServiceError>;
async fn check_user(&self, user: &str, ctx: Authentication<Self::Context>)
    -> Result<(), ServiceError>;
async fn current_user_id(&self, ctx: Authentication<Self::Context>)
    -> Result<Option<Arc<str>>, ServiceError>;
async fn get_privileges_for_current_user(&self, ctx: Authentication<Self::Context>)
    -> Result<Arc<[Privilege]>, ServiceError>;
async fn get_roles_for_user(&self, user: &str, ctx: Authentication<Self::Context>)
    -> Result<Arc<[Role]>, ServiceError>;
// + CRUD for user/role/privilege/user_role/role_privilege
```

Important semantic details:

- **`check_permission("X", Full) → Ok(())`** (line 41). Bypass.
- **`check_permission("X", Context(ctx))`** resolves the user via
  `UserService::current_user(ctx)`, loads privileges via DAO and returns
  `Forbidden` if not found (lines 43-53).
- **`check_only_full_authentication`** is the *inverse* gate: allows
  **only** Full and rejects any user context (lines 75-83). Used for
  endpoints that must never be reachable from outside. **[To verify]**
  — this method currently seems to have no productive caller; a grep
  over `service_impl/` and `rest/` returns no hits other than the trait
  definition itself.
- **`get_privileges_for_current_user(Full)` → `[Privilege { name: "god-mode" }]`**
  (lines 90-92). Symbolic marker so that the dev seed path in the
  frontend does not see an empty privileges list.

### 4.3 `SessionService`

`service/src/session.rs:44-56`. No `Context`-parameterized auth check —
all methods take raw IDs and are called **before** the auth layer,
because Sessions are the *foundation* of auth, not its consumer.

```rust
async fn new_session_for_user(&self, user_id: &str) -> Result<Session, ServiceError>;
async fn invalidate_user_session(&self, id: &str) -> Result<(), ServiceError>;
async fn verify_user_session(&self, id: &str) -> Result<Option<Session>, ServiceError>;
async fn start_impersonate(&self, session_id: Arc<str>, target_user_id: Arc<str>)
    -> Result<(), ServiceError>;
async fn stop_impersonate(&self, session_id: Arc<str>) -> Result<(), ServiceError>;
```

New Sessions get a UUID (`service_impl/src/session.rs:32-36`) and last
365 days.

### 4.4 `UserService`

`service/src/user_service.rs:11-15`. Minimal trait:

```rust
async fn current_user(&self, context: Self::Context) -> Result<Arc<str>, ServiceError>;
```

The productive implementation is trivial
(`service_impl/src/lib.rs:54-66`): `Context = Option<Arc<str>>`; if
`Some(name)` is set, it is returned, otherwise `Unauthorized`. The
actual login therefore already happens earlier in the Session
middleware; `UserService` is just the translation point from the REST
Context to the concrete username.

### 4.5 Auth gates — overview

| Method | Gate |
| --- | --- |
| `check_permission` | `Full` → Ok; otherwise DAO `has_privilege` |
| `check_user` | `Full` → Ok; otherwise compare `current_user == user` |
| `get_roles_for_user` | `admin` |
| `create_user`/`delete_user`/`get_all_users` | `admin` |
| `user_exists` | `hr` |
| `create_role`/`delete_role`/`get_all_roles` | `admin` |
| `create_privilege`/`delete_privilege`/`get_all_privileges` | `admin` |
| `add_user_role`/`delete_user_role` | `admin` |
| `add_role_privilege`/`delete_role_privilege` | `admin` |

### 4.6 TX behavior

Neither `PermissionService` nor `SessionService` uses `Transaction`
parameters. All ops are single-statement DAO calls; the transactional
consistency of roles/privileges is not critical, because changes are
rare and atomic per SQL statement. Impersonate is an `UPDATE`, not
atomic with further ops.

### 4.7 Dependencies

- **`PermissionServiceImpl`** (`service_impl/src/permission.rs:10-15`):
  `PermissionDao`, `UserService`. Basic service — no domain services.
- **`SessionServiceImpl`** (`service_impl/src/session.rs:14-20`):
  `SessionDao`, `UuidService`, `ClockService`. Basic service.
- **`UserServiceImpl`** (`service_impl/src/lib.rs:54`): no deps.

## 5. REST Endpoints

### 5.1 Auth info / login

| Method | Path | Description | DTO Out | Notes |
| --- | --- | --- | --- | --- |
| `GET` | `/authenticate` | Login entry — 302 to `/` (OIDC handles the redirect earlier) | — | `rest/src/lib.rs:507` |
| `GET` | `/logout` | OIDC logout (redirect to the IdP) | — | Only `oidc` feature, `rest/src/lib.rs:523` |
| `GET` | `/auth-info` | Current user + privileges | `AuthInfoTO { user, privileges }` | `rest/src/lib.rs:537-564` |

### 5.2 Permission CRUD (`/permission`)

Route setup: `rest/src/permission.rs:18-35`. All endpoints delegate to
`PermissionService` and implicitly call `check_permission("admin", …)`.

| Method | Path | DTO | Important errors |
| --- | --- | --- | --- |
| `GET` | `/user` | `[UserTO]` | 401, 403 |
| `POST` | `/user` | `UserTO` | 400, 403 |
| `DELETE` | `/user/` | body `String` | 403, 404 |
| `GET` | `/role` | `[RoleTO]` | 403 |
| `POST` | `/role` | `RoleTO` | 403 |
| `DELETE` | `/role` | body `String` | 403, 404 |
| `GET` | `/user/{user}/roles` | `[RoleTO]` | 403, 404 |
| `GET` | `/privilege/` | `[PrivilegeTO]` | 403 |
| `POST` | `/user-role` | `UserRole` | 403 |
| `DELETE` | `/user-role` | `UserRole` | 403 |
| `POST` | `/role-privilege/` | `RolePrivilege` | 403 |
| `DELETE` | `/role-privilege/` | `RolePrivilege` | 403 |

### 5.3 Impersonate (`/admin/impersonate`)

Route setup: `rest/src/impersonate.rs:27-32`. Important
peculiarity: the admin check always runs against the *real*
`session.user_id` (D-32-02, comment `rest/src/lib.rs:688-699`).

| Method | Path | Body/Path | Description |
| --- | --- | --- | --- |
| `GET` | `/` | — | Status: `ImpersonateTO { impersonating, user_id }` |
| `POST` | `/{user_id}` | Path | Start Impersonation, `session.impersonate_user_id = user_id` |
| `DELETE` | `/` | — | End Impersonation, column to NULL |

DTO: `ImpersonateTO { impersonating: bool, user_id: Option<Arc<str>> }`
(`rest-types/src/lib.rs:1707-1712`).

Audit log lines are emitted **after** a successful service call
(`rest/src/impersonate.rs:92-96, 149-153`) so that failed calls do not
produce false positives in the audit (D-32-01/WR-01, WR-02).

### 5.4 Dev endpoints (`/dev`, `mock_auth` feature only)

Only mounted in the dev build (`rest/src/lib.rs:685-686`). Aggregated,
it uses `Authentication::Full` as a bypass (`rest/src/dev.rs:128`).

| Method | Path | Description |
| --- | --- | --- |
| `POST` | `/dev/seed` | Creates Anna/Max/Lisa/Tom/Sarah + WorkDetails + Bookings + SpecialDays |
| `POST` | `/dev/clear` | `basic_dao.clear_all()` — **destructive** |

### 5.5 Middleware stack

Setup `rest/src/lib.rs:700-720` (Tower wraps in *reverse* order — the
last `.layer()` runs as the outermost):

```text
[Cookies]
  └─ [context_extractor] (sets Context + optional RealUser)
        └─ [audit_impersonated_writes] (logs POST/PUT/PATCH/DELETE on impersonation)
              └─ [forbid_unauthenticated] (OIDC only: 401 if Context is empty)
                    └─ Handler
```

The mounting comment in `rest/src/session.rs:52-62` explicitly explains
the layer order and is relevant when refactoring the stack.

## 6. Frontend Integration

- **Pages:** `shifty-dioxus/src/page/not_authenticated.rs` (24 lines —
  welcome screen with a link to `/authenticate`, reached only in the
  OIDC build, because in mock mode `context_extractor` immediately
  creates a `DEVUSER`); `user_management.rs` (878 lines — tab layout
  Users/SalesPersons, add/delete user dialogs, "Work as …" button for
  Impersonation from line 42); `user_details.rs` (333 lines — role
  assignment + invitation list).
- **Auth guard:** `shifty-dioxus/src/auth.rs` (24 lines) — `<Auth>`
  component with `authenticated`/`unauthenticated` slots, chosen based
  on `AUTH.auth_info` and `loading_done`. While loading, it shows
  "Fetching auth information…".
- **Auth service:** `shifty-dioxus/src/service/auth.rs` — `AUTH: GlobalSignal<AuthStore>`
  holds `AuthInfo { user, privileges }`. `load_auth_info()` calls
  `api::fetch_auth_info` → `GET /auth-info` and fills the store.
- **Impersonate service:** `shifty-dioxus/src/service/impersonate.rs` —
  own coroutine with `ImpersonateAction`, consumed by
  `user_management.rs:42`.
- **State:** `shifty-dioxus/src/state/*` — `AuthInfo` as a domain type.
- **i18n keys:** `WelcomeTitle`, `PleaseLogin` (Not Authenticated),
  `UserManagement`, `BackToUserManagement`.
- **Proxy:** `Dioxus.toml` proxies `/permission`, `/auth-info`,
  `/authenticate`, `/admin/impersonate`. **[To verify]** — not directly
  listed in the feature context, but convention (see MEMORY.md entry
  "Dioxus.toml proxy").

## 7. Edge cases

For the central edge case reference see
[`../domain/edge-cases.md#6-authentifizierung--autorisierung`](../domain/edge-cases.md#6-authentifizierung--autorisierung).

- **Full-Bypass abuse:** any consumer that hands `Authentication::Full`
  to a service circumvents **all** role checks. This is *by design*
  for internal aggregates (see the toggle-bypass rule in the MEMORY
  entry "ToggleService Full-Context Bypass"), but a single misplaced
  `Full` call from an HTTP handler would be a privilege escalation.
  Rule: REST handlers *always* pass `context.into()` — only Schedulers,
  cron and startup migrations may construct `Full`.
- **Token expiry:** Sessions live 365 days. When a Session expires,
  `verify_user_session` returns `None` — in OIDC mode this results in
  401; in mock mode a new `DEVUSER` Session is freshly created
  (`rest/src/session.rs:210-233`).
- **Role change mid-session:** if an admin revokes a role from a user
  during their active Session, the effect is immediate — every
  `check_permission` request checks *live* against the DB. There is no
  privilege caching in the backend. The frontend store `AUTH` however
  caches until the next reload; the UI can therefore show things based
  on stale privileges that the backend then rejects at the actual call.
- **Impersonate + admin-only endpoints:** if an admin impersonates a
  non-admin and calls a `check_permission("admin")`-gated route, the
  call fails with 403 — the effective Context is the non-admin
  (`rest/src/lib.rs:694-696`, D-32-02a). Only `/admin/impersonate/*`
  remains accessible, because there the check is explicitly against the
  real `session.user_id`.
- **Session without user:** `session.user_id` references `user(name)`
  via FK. If a user is deleted while their Session runs, this leads to
  a consistency break (no `ON DELETE CASCADE`). **[To verify]** whether
  `permission_service.delete_user` cleans up this user's Sessions —
  neither the trait nor the SQLite impl seems to do so.
- **`impersonate_user_id` references deleted user:**
  `start_impersonate` checks existence via `user_exists` with
  `Authentication::Full` (`rest/src/impersonate.rs:75-84`), but a later
  deletion of the target user is not cascaded.
- **Not authenticated in OIDC:** `forbid_unauthenticated` lets
  `/authenticate` and `/*/ical` through. Everything else → 401. In the
  mock build this never happens, because `context_extractor` always
  creates a Context.

## 8. Tests

- **Unit `service_impl/src/test/permission_test.rs`** (472 lines) —
  full coverage of `check_permission`, `create_user`, `delete_user`,
  `create_role`, `add_role_privilege`, `delete_role_privilege`,
  roles/privileges listings. The mock setup uses `NoneTypeExt::auth()`
  (`error_test.rs:130-137`), which wraps context `()` in
  `Authentication::Context(())` — which tests the *non*-Full paths. The
  Full path is covered implicitly via dev seed integration in the live
  backend.
- **Unit `service_impl/src/test/session.rs`** (120 lines) —
  `test_start_impersonate`, `test_stop_impersonate`, plus Session CRUD.
  Mocks `SessionDao` with `expect_update_impersonate`.
- **Unit `rest/src/session.rs` (module `tests`)** — tests for
  `real_user_extension` (present/absent) and
  `should_audit_impersonated_write` across all HTTP verbs
  (POST/PUT/PATCH/DELETE audit; GET/HEAD/OPTIONS not; non-impersonating
  never).
- **Integration `service_impl/src/test/permission_test.rs::test_user_service_impl_*`**
  — tests `UserServiceImpl` directly for Some/None Context.
- **Known gaps:**
  - No automated end-to-end test for the Impersonate audit log layer
    (only the pure function `should_audit_impersonated_write` is
    covered).
  - No test that `check_only_full_authentication` actually returns 403
    on a real user Context. **[To verify]** whether any consumer even
    exists — otherwise the method can be removed.
  - No test that concurrent role changes during a running Session take
    effect correctly (live DB semantics).

## 9. History & Context

- **04/2024** — base auth (`20240426150045_user-roles.sql`): three
  roles `admin/sales/hr`, three identically named privileges. Back
  then still without Sessions — auth ran purely via OIDC cookies
  (`SessionManagerLayer` from the `axum-oidc` crate).
- **06/2024** — `shiftplanner` role added; first differentiation
  between HR (time tracking) and planning.
- **11/2024** — `session` table introduced (background: iCal feeds and
  custom cookies needed their own Session store separate from the
  OIDC middleware Session). Two migrations because the first `id`
  definition was mistakenly nullable.
- **11/2024** — `shiftplan.edit` role: splits "planner sees"
  (`shiftplanner`) from "planner writes" (`shiftplan.edit`). Motivation:
  branch management sees but does not change themselves.
- **04/2026 (Phase 32 — Impersonation)** — `impersonate_user_id` in
  the `session` table. Context see D-32-01 (audit log for writes) and
  D-32-02 (two-path admin gate). Frontend part D-32-07/IMP-01
  (`user_management.rs:42`).
- **References:** `.planning/phases/32-impersonation/` for context
  reads; cluster **F13 (User Invitation)** for the token-based invite
  login flow that exists alongside regular OIDC auth and is
  deliberately mounted *behind* the `forbid_unauthenticated` layer
  (`rest/src/lib.rs:752-759`) so it can be consumed without an
  existing Session.

---

**Conclusion:** F12 secures Shifty with a clear two-mode pattern (OIDC
prod, mock dev) and a deliberately narrow `Authentication<Context>`
API, whose `Full` variant is the only bypass for internal aggregates.
Impersonation extends this without a break by preserving the real
admin identity as a `RealUser` extension and hard-auditing it for
writes.

*Last verification against code:* see git blame of this file.
