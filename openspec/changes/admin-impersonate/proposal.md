## Why

Admins need the ability to impersonate other users to debug issues, verify permissions, and test the application from another user's perspective. Currently, mock auth always returns "DEVUSER" and has no session management, making it impossible to switch users during local development. This also requires unifying the authentication context between mock auth and OIDC modes to reduce code duplication and enable session-based features in both modes.

## What Changes

- Unify `Context` type to `Option<Arc<str>>` for both mock auth and OIDC modes, eliminating `MockContext`
- Add session management to mock auth mode (auto-create session for "DEVUSER" on first request)
- Remove `UserServiceDev` in favor of the unified `UserServiceImpl`
- Add `impersonate_user_id` nullable field to `SessionEntity` and the sessions database table
- Add `POST /admin/impersonate/{user_id}` endpoint (requires admin privilege, sets impersonate on session)
- Add `DELETE /admin/impersonate` endpoint (requires admin privilege, clears impersonate from session)
- Modify `context_extractor` to use `impersonate_user_id` when present, falling back to `user_id`
- Impersonate endpoints check the real `user_id` from the session (not the impersonated identity) for admin privilege verification

## Capabilities

### New Capabilities
- `unified-auth-context`: Unify authentication context type and session management across mock auth and OIDC feature flags
- `admin-impersonate`: Allow admins to impersonate other users via session-based identity override

### Modified Capabilities

## Impact

- **Database**: New migration adding `impersonate_user_id` column to sessions table
- **Crates affected**: `dao`, `dao_impl_sqlite`, `service`, `service_impl`, `rest`, `rest-types`, `shifty_bin`
- **Feature flags**: `mock_auth` behavior changes significantly (gains session management); `oidc` behavior unchanged except for impersonate addition
- **API**: Two new REST endpoints under `/admin/impersonate`
- **Breaking**: `MockContext` type removed — any code referencing it directly needs updating
- **No impact**: All existing service-layer code (bookings, shift plans, permissions, etc.) works unchanged since identity flows through `current_user()`
