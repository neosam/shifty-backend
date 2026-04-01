## ADDED Requirements

### Requirement: Unified context type across feature flags
The authentication context type SHALL be `Option<Arc<str>>` for both `mock_auth` and `oidc` feature flags. The `MockContext` struct SHALL be removed.

#### Scenario: Mock auth mode uses unified context type
- **WHEN** the application is compiled with `mock_auth` feature flag
- **THEN** the `Context` type alias SHALL resolve to `Option<Arc<str>>`

#### Scenario: OIDC mode uses unified context type
- **WHEN** the application is compiled with `oidc` feature flag
- **THEN** the `Context` type alias SHALL resolve to `Option<Arc<str>>` (unchanged)

### Requirement: Mock auth session auto-creation
In `mock_auth` mode, the system SHALL automatically create a session for the default user "DEVUSER" when no valid session cookie exists.

#### Scenario: First request without session cookie
- **WHEN** a request arrives in `mock_auth` mode without an `app_session` cookie
- **THEN** the system SHALL create a new session for user "DEVUSER"
- **THEN** the system SHALL set the `app_session` cookie on the response
- **THEN** the request SHALL proceed with `Context = Some("DEVUSER")`

#### Scenario: Subsequent request with valid session cookie
- **WHEN** a request arrives in `mock_auth` mode with a valid `app_session` cookie
- **THEN** the system SHALL look up the session and extract the `user_id`
- **THEN** the request SHALL proceed with `Context = Some(user_id)`

### Requirement: Unified user service
There SHALL be a single `UserServiceImpl` that handles both feature flag modes. The `UserServiceDev` struct SHALL be removed.

#### Scenario: Current user resolution
- **WHEN** `current_user()` is called with `Some(user_id)`
- **THEN** it SHALL return the `user_id`

#### Scenario: Unauthenticated context
- **WHEN** `current_user()` is called with `None`
- **THEN** it SHALL return `ServiceError::Unauthorized`
