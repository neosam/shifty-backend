## Context

The backend uses two authentication modes controlled by feature flags:
- **`mock_auth`**: No sessions, `Context = MockContext` (empty struct), `current_user()` always returns "DEVUSER"
- **`oidc`**: Session-based, `Context = Option<Arc<str>>`, `current_user()` extracts username from context

This split means `MockContext` appears in 7 files, and there are parallel implementations for `UserServiceDev` / `UserServiceImpl`, two `context_extractor` functions, and two `forbid_unauthenticated` functions. Impersonate requires session management, which mock_auth lacks.

Key files:
- `service/src/permission.rs` — defines `MockContext` and `Authentication<Context>`
- `service_impl/src/lib.rs` — `UserServiceDev` (mock) and `UserServiceImpl` (OIDC)
- `rest/src/session.rs` — feature-gated `Context` type alias, `context_extractor`, `forbid_unauthenticated`
- `shifty_bin/src/main.rs` — feature-gated type aliases and DI wiring
- `dao/src/session.rs` — `SessionEntity` (no `impersonate_user_id` field)

## Goals / Non-Goals

**Goals:**
- Unify `Context` to `Option<Arc<str>>` for both feature flags
- Add session management to mock_auth mode (auto-create session for "DEVUSER")
- Add impersonate capability: admin can override session identity
- Both feature flags share the same impersonate mechanism

**Non-Goals:**
- Audit trail for impersonated actions
- Removing the feature flag system entirely (OIDC login flow, middleware layers remain feature-gated)
- Frontend changes (out of scope for backend change)

## Decisions

### 1. Unify Context type to `Option<Arc<str>>`

Remove `MockContext` struct entirely. Both modes use `Option<Arc<str>>` as Context.

**Rationale:** Eliminates parallel implementations and makes session-based features (like impersonate) work identically in both modes. The `mock_auth` vs `oidc` distinction shrinks to just "how does the initial session get created?"

**Alternative considered:** Keep `MockContext` and implement impersonate separately for mock_auth (e.g., via HTTP header). Rejected because it creates two code paths for the same feature and more maintenance burden.

**Changes:**
- `service/src/permission.rs`: Delete `MockContext` struct
- `rest/src/session.rs`: `pub type Context = Option<Arc<str>>` unconditionally
- `service_impl/src/lib.rs`: Remove `UserServiceDev`, only keep `UserServiceImpl`
- `shifty_bin/src/main.rs`: Remove feature-gated `UserService` and `Context` type aliases, use unified types

### 2. Mock auth auto-session creation

In mock_auth mode, `context_extractor` checks for an `app_session` cookie. If none exists, it auto-creates a session for "DEVUSER" and sets the cookie. Subsequent requests use the session normally.

**Rationale:** Mimics the OIDC login flow without requiring an identity provider. The developer experience stays seamless (no manual login step).

**Changes to `rest/src/session.rs` mock_auth `context_extractor`:**
- Read `app_session` cookie (requires `Cookies` extractor and `RestState` for session service)
- If cookie exists: verify session, extract user_id (same as OIDC path)
- If no cookie: create session for "DEVUSER", set cookie, use that session
- Check for `impersonate_user_id` on session (shared logic with OIDC)

**Mock auth `forbid_unauthenticated`:** Remains a no-op since auto-session guarantees authentication.

### 3. Session-based impersonate via `impersonate_user_id` field

Add `impersonate_user_id: Option<Arc<str>>` to `SessionEntity` and `Session`. The `context_extractor` checks this field and uses it as the Context identity when present.

**Rationale:** Cleanest integration point — only the context extractor changes, the entire service layer remains untouched. No new middleware, no new context types.

**Alternative considered:** Separate impersonate session (create a new session as the target user, swap cookies). Rejected because it requires tracking the original session separately and complicates the "stop impersonate" flow.

### 4. Impersonate endpoints check real session `user_id`

The `POST /admin/impersonate/{user_id}` and `DELETE /admin/impersonate` endpoints bypass the normal Context flow. They read the session directly from the cookie and verify admin privilege against `session.user_id` (the real admin identity), not the potentially impersonated identity.

**Rationale:** Once impersonating, the normal Context is the target user who may not be an admin. The impersonate endpoints need access to the real identity to verify authorization.

**Implementation approach:** These endpoints receive the raw session (via cookie + session service lookup) and perform their own admin check using `Authentication::Context(session.user_id)` against the permission service.

### 5. Database migration

Single migration: `ALTER TABLE sessions ADD COLUMN impersonate_user_id TEXT NULL`

No data migration needed — existing sessions get `NULL` which means "not impersonating."

### 6. DAO changes

Add to `SessionDao` trait:
- `update_impersonate(&self, session_id: &str, impersonate_user_id: Option<&str>) -> Result<(), DaoError>`

Add `impersonate_user_id: Option<Arc<str>>` to `SessionEntity`.

### 7. REST API design

```
POST   /admin/impersonate/{user_id}
  - Reads app_session cookie → gets real session
  - Checks admin privilege on session.user_id
  - Calls session DAO to set impersonate_user_id
  - Returns 200 with impersonated user info

DELETE /admin/impersonate
  - Reads app_session cookie → gets real session
  - Checks admin privilege on session.user_id
  - Calls session DAO to clear impersonate_user_id
  - Returns 200

GET    /admin/impersonate
  - Reads app_session cookie → gets real session
  - Returns current impersonate status (who am I impersonating, if anyone?)
```

## Risks / Trade-offs

**[Risk] Mock auth behavior change may break existing dev workflows**
→ Mitigation: Auto-session creation is transparent. The only visible change is a new `app_session` cookie in dev mode. All existing endpoints continue working.

**[Risk] Impersonating a non-existent user**
→ Mitigation: Validate that the target user exists before setting impersonate. Return 404 if not found.

**[Risk] SQLx compile-time query checking requires database update**
→ Mitigation: Run migration before building. Document in task list.

**[Trade-off] `MockContext` removal touches test code**
→ Tests use `type Context = ()` via `NoneTypeExt` trait, which is independent of `MockContext`. Only `test_user_service_dev` in `permission_test.rs` directly uses `MockContext` and needs updating.
