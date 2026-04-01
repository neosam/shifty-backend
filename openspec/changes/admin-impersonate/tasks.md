## 1. Scaffolding — Unified Auth Context

- [x] 1.1 Remove `MockContext` struct from `service/src/permission.rs` and all imports referencing it
- [x] 1.2 Change `Context` type alias in `rest/src/session.rs` to `Option<Arc<str>>` unconditionally (remove feature-gated split)
- [x] 1.3 Remove `UserServiceDev` from `service_impl/src/lib.rs`, keep only `UserServiceImpl`
- [x] 1.4 Update `shifty_bin/src/main.rs` to remove feature-gated `UserService` and `Context` type aliases, use `UserServiceImpl` and `Option<Arc<str>>` for both modes
- [x] 1.5 Update mock_auth `context_extractor` in `rest/src/session.rs` to accept `State<RestState>` and create auto-sessions for "DEVUSER" (stub with `todo!()` for session creation logic)
- [x] 1.6 Add `impersonate_user_id: Option<Arc<str>>` field to `SessionEntity` in `dao/src/session.rs` and `Session` in `service/src/session.rs`, update `From` conversions
- [x] 1.7 Add `update_impersonate` method stub to `SessionDao` trait in `dao/src/session.rs`
- [x] 1.8 Add `start_impersonate` and `stop_impersonate` method stubs to `SessionService` trait in `service/src/session.rs`
- [x] 1.9 Create database migration adding `impersonate_user_id TEXT NULL` column to sessions table
- [x] 1.10 Stub `update_impersonate` in `dao_impl_sqlite` with `todo!()`
- [x] 1.11 Stub REST endpoints for `POST /admin/impersonate/{user_id}`, `DELETE /admin/impersonate`, `GET /admin/impersonate` with `todo!()`
- [x] 1.12 Add `ImpersonateTO` DTO to `rest-types` with `ToSchema` derive for OpenAPI response
- [x] 1.13 Verify project compiles with `cargo build`

## 2. Red — Write Tests

- [x] 2.1 Write tests for `UserServiceImpl::current_user()` — replace existing `test_user_service_dev` test in `permission_test.rs` with tests for `Some(user_id)` and `None` cases
- [x] 2.2 Write unit tests for `SessionService::start_impersonate` — verify it calls `update_impersonate` on DAO with correct args and checks admin privilege
- [x] 2.3 Write unit tests for `SessionService::stop_impersonate` — verify it clears `impersonate_user_id` and checks admin privilege
- [x] 2.4 Write tests for `SessionDao::update_impersonate` — verify it sets and clears `impersonate_user_id` in SQLite
- [x] 2.5 Write tests for context_extractor behavior: session with `impersonate_user_id` set should use impersonated identity
- [x] 2.6 Verify all new tests compile but fail (`cargo test` — expect failures)

## 3. Green — Implement Unified Auth Context

- [x] 3.1 Implement mock_auth `context_extractor` with auto-session creation for "DEVUSER" and session cookie handling
- [x] 3.2 Update OIDC `context_extractor` to check `impersonate_user_id` on session and use it as Context when present
- [x] 3.3 Update mock_auth `context_extractor` to also check `impersonate_user_id` (shared logic)
- [x] 3.4 Update `SessionEntity` `TryFrom` database row conversion in `dao_impl_sqlite` to include `impersonate_user_id`
- [x] 3.5 Implement `update_impersonate` in `dao_impl_sqlite` with SQL query
- [x] 3.6 Run migration, update SQLx offline query data
- [x] 3.7 Verify unified auth context tests pass

## 4. Green — Implement Impersonate Feature

- [x] 4.1 Implement `start_impersonate` in `SessionService` — verify admin privilege on real user_id, validate target user exists, call DAO
- [x] 4.2 Implement `stop_impersonate` in `SessionService` — verify admin privilege on real user_id, call DAO to clear
- [x] 4.3 Implement `POST /admin/impersonate/{user_id}` REST endpoint — read session from cookie, call service, add `#[utoipa::path]` annotation
- [x] 4.4 Implement `DELETE /admin/impersonate` REST endpoint — read session from cookie, call service, add `#[utoipa::path]` annotation
- [x] 4.5 Implement `GET /admin/impersonate` REST endpoint — read session from cookie, return status, add `#[utoipa::path]` annotation
- [x] 4.6 Wire impersonate routes into the Axum router in `rest/src/lib.rs`
- [x] 4.7 Verify all impersonate tests pass

## 5. Verification

- [x] 5.1 Run full test suite with `cargo test`
- [x] 5.2 Run `cargo build` for both feature flag modes (mock_auth and oidc)
- [x] 5.3 Run application with `cargo run` and verify auto-session works in mock_auth mode
