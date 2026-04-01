## ADDED Requirements

### Requirement: Session entity supports impersonation
The `SessionEntity` and `Session` structs SHALL include an `impersonate_user_id: Option<Arc<str>>` field. The database sessions table SHALL have a nullable `impersonate_user_id TEXT` column.

#### Scenario: New session has no impersonation
- **WHEN** a new session is created
- **THEN** `impersonate_user_id` SHALL be `None`

#### Scenario: Existing sessions after migration
- **WHEN** the migration runs on an existing database
- **THEN** all existing sessions SHALL have `impersonate_user_id = NULL`

### Requirement: Context extractor uses impersonated identity
The `context_extractor` middleware SHALL use `impersonate_user_id` as the Context identity when it is set on the session, falling back to `user_id` when it is not set.

#### Scenario: Session without impersonation
- **WHEN** a request arrives with a session where `impersonate_user_id` is `None`
- **THEN** the Context SHALL be `Some(session.user_id)`

#### Scenario: Session with active impersonation
- **WHEN** a request arrives with a session where `impersonate_user_id` is `Some("target")`
- **THEN** the Context SHALL be `Some("target")`
- **THEN** all downstream services SHALL see "target" as the current user

### Requirement: Start impersonation endpoint
The system SHALL provide a `POST /admin/impersonate/{user_id}` endpoint that allows an admin to start impersonating another user.

#### Scenario: Admin starts impersonation
- **WHEN** an admin calls `POST /admin/impersonate/targetuser`
- **THEN** the system SHALL set `impersonate_user_id = "targetuser"` on the admin's session
- **THEN** the system SHALL return HTTP 200

#### Scenario: Non-admin attempts impersonation
- **WHEN** a non-admin user calls `POST /admin/impersonate/targetuser`
- **THEN** the system SHALL return HTTP 403 Forbidden

#### Scenario: Impersonate non-existent user
- **WHEN** an admin calls `POST /admin/impersonate/nonexistent`
- **THEN** the system SHALL return HTTP 404 Not Found

#### Scenario: Admin privilege checked against real identity
- **WHEN** an admin is already impersonating a non-admin user and calls `POST /admin/impersonate/otheruser`
- **THEN** the system SHALL check admin privilege against the session's `user_id` (not `impersonate_user_id`)
- **THEN** the impersonation target SHALL be updated to "otheruser"

### Requirement: Stop impersonation endpoint
The system SHALL provide a `DELETE /admin/impersonate` endpoint that allows an admin to stop impersonating.

#### Scenario: Admin stops impersonation
- **WHEN** an admin calls `DELETE /admin/impersonate` while impersonating
- **THEN** the system SHALL set `impersonate_user_id = None` on the session
- **THEN** the system SHALL return HTTP 200
- **THEN** subsequent requests SHALL use the admin's real identity

#### Scenario: Admin stops impersonation when not impersonating
- **WHEN** an admin calls `DELETE /admin/impersonate` while not impersonating
- **THEN** the system SHALL return HTTP 200 (idempotent)

### Requirement: Impersonation status endpoint
The system SHALL provide a `GET /admin/impersonate` endpoint that returns the current impersonation status.

#### Scenario: Currently impersonating
- **WHEN** an admin calls `GET /admin/impersonate` while impersonating "targetuser"
- **THEN** the system SHALL return HTTP 200 with the impersonated user information

#### Scenario: Not impersonating
- **WHEN** an admin calls `GET /admin/impersonate` while not impersonating
- **THEN** the system SHALL return HTTP 200 indicating no active impersonation

### Requirement: DAO support for impersonation
The `SessionDao` trait SHALL provide a method to update the `impersonate_user_id` field on a session.

#### Scenario: Set impersonation on session
- **WHEN** `update_impersonate(session_id, Some("target"))` is called
- **THEN** the session's `impersonate_user_id` SHALL be set to "target" in the database

#### Scenario: Clear impersonation on session
- **WHEN** `update_impersonate(session_id, None)` is called
- **THEN** the session's `impersonate_user_id` SHALL be set to NULL in the database
