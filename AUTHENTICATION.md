# Authentication System Documentation - Shifty Backend

## Overview

The Shifty backend implements a robust authentication and authorization system with support for both development (mock) and production (OIDC) environments. The system uses Role-Based Access Control (RBAC) with users, roles, and privileges.

## Architecture

### Authentication Modes

The system supports two authentication modes controlled by feature flags:

1. **Mock Authentication** (`mock_auth` feature) - Development mode
2. **OIDC Authentication** (`oidc` feature) - Production mode

### Context Types

```rust
// Development mode
#[cfg(feature = "mock_auth")]
pub type Context = MockContext;

// Production mode  
#[cfg(feature = "oidc")]
pub type Context = Option<Arc<str>>; // Username
```

## Database Schema

### Core Authentication Tables

```sql
-- Users table
CREATE TABLE user (
    name TEXT NOT NULL PRIMARY KEY,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);

-- Roles table
CREATE TABLE role (
    name TEXT NOT NULL PRIMARY KEY,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);

-- Privileges table
CREATE TABLE privilege (
    name TEXT NOT NULL PRIMARY KEY,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);

-- User-Role relationship
CREATE TABLE user_role (
    user_name TEXT NOT NULL,
    role_name TEXT NOT NULL,
    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    FOREIGN KEY (user_name) REFERENCES user(name) ON DELETE CASCADE,
    FOREIGN KEY (role_name) REFERENCES role(name) ON DELETE CASCADE,
    UNIQUE (user_name, role_name)
);

-- Role-Privilege relationship
CREATE TABLE role_privilege (
    role_name TEXT NOT NULL,
    privilege_name TEXT NOT NULL,
    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    FOREIGN KEY (role_name) REFERENCES role(name) ON DELETE CASCADE,
    FOREIGN KEY (privilege_name) REFERENCES privilege(name) ON DELETE CASCADE,
    UNIQUE (role_name, privilege_name)
);

-- Session management
CREATE TABLE session (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    expires INTEGER NOT NULL,
    created INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user(name)
);

-- Link sales persons to users
CREATE TABLE sales_person_user (
    sales_person_id blob(16) NOT NULL,
    user_id TEXT NOT NULL,
    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    PRIMARY KEY (sales_person_id, user_id),
    UNIQUE(sales_person_id),
    UNIQUE(user_id),
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id),
    FOREIGN KEY (user_id) REFERENCES user(name)
);
```

### Default Privileges

The system defines three core privileges:

```rust
pub const SALES_PRIVILEGE: &str = "sales";
pub const HR_PRIVILEGE: &str = "hr";
pub const SHIFTPLANNER_PRIVILEGE: &str = "shiftplanner";
```

## Development Mode (Mock Authentication)

In development mode, authentication is bypassed for easier testing:

```rust
// service_impl/src/lib.rs
pub struct UserServiceDev;

#[async_trait]
impl service::user_service::UserService for UserServiceDev {
    type Context = MockContext;

    async fn current_user(
        &self,
        _context: Self::Context,
    ) -> Result<Arc<str>, service::ServiceError> {
        Ok("DEVUSER".into())
        // Uncomment to test unauthorized response
        //Err(service::ServiceError::Unauthorized)
    }
}
```

### Auto-creation of Admin User

On startup, the system automatically creates admin users in development:

```rust
// shifty_bin/src/main.rs
async fn create_admin_user(pool: Arc<SqlitePool>, username: &str) {
    let permission_dao = PermissionDaoImpl::new(pool.clone());
    let users = permission_dao.all_users().await.expect("Expected users");
    let contains_admin_user = users.iter().any(|user| user.name.as_ref() == username);
    
    if !contains_admin_user {
        permission_dao
            .create_user(
                &dao::UserEntity {
                    name: username.into(),
                },
                "dev-first-start",
            )
            .await
            .expect(&format!("Expected being able to create the {}", username));
            
        permission_dao
            .add_user_role(username, "admin", "dev-first-start")
            .await
            .expect(&format!("Expected being able to make {} an admin", username));
    }
}

// In main()
create_admin_user(pool.clone(), "DEVUSER").await;
create_admin_user(pool.clone(), "admin").await;
```

## Production Mode (OIDC Authentication)

### Configuration

OIDC is configured via environment variables:

```bash
# Required environment variables
APP_URL=https://your-app.com        # Application base URL
ISSUER=https://your-oidc-provider.com/realms/your-realm
CLIENT_ID=your-client-id
CLIENT_SECRET=your-client-secret    # Optional for public clients
```

### OIDC Setup Code

```rust
// rest/src/lib.rs
pub struct OidcConfig {
    pub app_url: String,
    pub issuer: String,
    pub client_id: String,
    pub client_secret: Option<String>,
}

pub fn oidc_config() -> OidcConfig {
    let app_url = std::env::var("APP_URL").expect("APP_URL env variable");
    let issuer = std::env::var("ISSUER").expect("ISSUER env variable");
    let client_id = std::env::var("CLIENT_ID").expect("CLIENT_ID env variable");
    let client_secret = std::env::var("CLIENT_SECRET").ok();
    
    OidcConfig {
        app_url,
        issuer,
        client_id,
        client_secret: client_secret.unwrap_or_default().into(),
    }
}
```

### OIDC Middleware Stack

```rust
// rest/src/lib.rs
#[cfg(feature = "oidc")]
let app = {
    use axum_oidc::{EmptyAdditionalClaims, OidcAuthLayer};
    
    let oidc_config = oidc_config();
    let session_store = MemoryStore::default();
    
    // Session management layer
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_same_site(SameSite::Strict)
        .with_expiry(Expiry::OnInactivity(Duration::minutes(50)));
    
    // OIDC authentication layer
    let oidc_auth_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
            e.into_response()
        }))
        .layer(
            OidcAuthLayer::<EmptyAdditionalClaims>::discover_client(
                Uri::from_maybe_shared(oidc_config.app_url).expect("valid APP_URL"),
                oidc_config.issuer,
                oidc_config.client_id,
                oidc_config.client_secret,
                vec![],
            )
            .await
            .unwrap(),
        );
    
    app.layer(oidc_auth_service).layer(session_layer)
};
```

### Session Registration

```rust
// rest/src/session.rs
#[cfg(feature = "oidc")]
pub async fn register_session<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    claims: Option<OidcClaims<EmptyAdditionalClaims>>,
    request: Request,
    next: Next,
) -> Response {
    let cookies = request
        .extensions()
        .get::<Cookies>()
        .expect("Cookies extension not set");
    
    if let Some(oidc_claims) = claims {
        let username = oidc_claims
            .preferred_username()
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| "NoUsername".to_string());
            
        let session = rest_state
            .session_service()
            .new_session_for_user(&username)
            .await
            .unwrap();
            
        let session_id = session.id.to_string();
        let now = OffsetDateTime::now_utc();
        let expires = now + time::Duration::days(365);
        
        let cookie = Cookie::build(Cookie::new("app_session", session_id))
            .path("/")
            .expires(expires)
            .http_only(true)
            .same_site(tower_cookies::cookie::SameSite::Strict)
            .secure(true);
            
        cookies.add(cookie.into());
    }
    next.run(request).await
}
```

## REST API Endpoints

### Authentication Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/authenticate` | GET | Login endpoint (redirects to OIDC provider) |
| `/logout` | GET | Logout endpoint (OIDC RP-initiated logout) |

### Permission Management Endpoints

All endpoints are prefixed with `/api/permission/`:

| Endpoint | Method | Description | Request Body |
|----------|--------|-------------|--------------|
| `/user` | GET | List all users | - |
| `/user` | POST | Create user | `UserTO { name: string }` |
| `/user/` | DELETE | Delete user | `string` (username) |
| `/role` | GET | List all roles | - |
| `/role` | POST | Create role | `RoleTO { name: string }` |
| `/role` | DELETE | Delete role | `string` (role name) |
| `/user/{user}/roles` | GET | Get roles for user | - |
| `/privilege/` | GET | List all privileges | - |
| `/user-role` | POST | Assign role to user | `UserRole { user: string, role: string }` |
| `/user-role` | DELETE | Remove role from user | `UserRole { user: string, role: string }` |
| `/role-privilege/` | POST | Assign privilege to role | `RolePrivilege { role: string, privilege: string }` |
| `/role-privilege/` | DELETE | Remove privilege from role | `RolePrivilege { role: string, privilege: string }` |

### Example REST Handler

```rust
// rest/src/permission.rs
#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/user",
    tags = ["Permission"],
    request_body = UserTO,
    responses(
        (status = 201, description = "User created successfully"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn add_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(user): Json<UserTO>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .create_user(user.name.as_str(), context.into())
                .await?;
            Ok(Response::builder()
                .status(201)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}
```

## Authorization Flow

### 1. Context Extraction Middleware

```rust
// rest/src/session.rs
#[cfg(feature = "oidc")]
pub async fn context_extractor<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    mut request: Request,
    next: Next,
) -> Response {
    let cookies = request
        .extensions()
        .get::<Cookies>()
        .expect("Cookies extension not set");
    
    if let Some(cookie) = cookies.get("app_session") {
        let session_id = cookie.value();
        if let Some(session) = rest_state
            .session_service()
            .verify_user_session(session_id)
            .await
            .unwrap()
        {
            request.extensions_mut().insert(Some(session.user_id));
        } else {
            request.extensions_mut().insert(None::<Arc<str>>);
        }
    } else {
        request.extensions_mut().insert(None::<Arc<str>>);
    };
    next.run(request).await
}
```

### 2. Authentication Enforcement

```rust
// rest/src/session.rs
#[cfg(feature = "oidc")]
pub async fn forbid_unauthenticated<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    request: Request,
    next: Next,
) -> Response {
    if request.extensions().get::<Context>().is_some()
        && request.extensions().get::<Context>().unwrap().is_some()
        || request.uri().path().ends_with("/ical")
        || request.uri().path().ends_with("/authenticate")
    {
        next.run(request).await
    } else {
        Response::builder()
            .status(401)
            .body("Unauthorized".into())
            .unwrap()
    }
}
```

### 3. Permission Checking in Services

```rust
// Example from a service implementation
impl SomeService {
    async fn protected_operation(
        &self,
        context: Authentication<Context>,
    ) -> Result<(), ServiceError> {
        // Check if user has required privilege
        self.permission_service
            .check_permission("shiftplanner", context.clone())
            .await?;
        
        // Proceed with operation
        // ...
    }
}
```

## Authentication Type

The `Authentication` enum provides flexibility in permission checking:

```rust
// service/src/permission.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Authentication<Context: Clone + PartialEq + Eq + Send + Sync + Debug + 'static> {
    Full,           // Full system access (internal operations)
    Context(Context), // User context from authentication
}
```

## Permission Service Interface

```rust
#[async_trait]
pub trait PermissionService {
    type Context: Clone + PartialEq + Eq + Debug + Send + Sync + 'static;
    
    // Get current user ID
    async fn current_user_id(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Option<Arc<str>>, ServiceError>;
    
    // Check if user has specific privilege
    async fn check_permission(
        &self,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
    
    // Check if request is fully authenticated (not anonymous)
    async fn check_only_full_authentication(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
    
    // Check if current user matches specified user
    async fn check_user(
        &self,
        user: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
    
    // Get all privileges for current user
    async fn get_privileges_for_current_user(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[Privilege]>, ServiceError>;
}
```

## Usage Examples

### Creating a New User with Admin Role

```bash
# Using the REST API
# 1. Create user
curl -X POST http://localhost:3000/api/permission/user \
  -H "Content-Type: application/json" \
  -d '{"name": "john.doe"}'

# 2. Assign admin role
curl -X POST http://localhost:3000/api/permission/user-role \
  -H "Content-Type: application/json" \
  -d '{"user": "john.doe", "role": "admin"}'
```

### Checking Permissions in Service Code

```rust
pub async fn delete_important_resource(
    &self,
    resource_id: Uuid,
    context: Authentication<Context>,
) -> Result<(), ServiceError> {
    // Ensure user has admin privileges
    self.permission_service
        .check_permission("admin", context.clone())
        .await?;
    
    // Proceed with deletion
    self.resource_dao.delete(resource_id).await?;
    Ok(())
}
```

### Getting Current User Information

```rust
pub async fn get_current_user_info(
    &self,
    context: Authentication<Context>,
) -> Result<UserInfo, ServiceError> {
    let user_id = self.permission_service
        .current_user_id(context.clone())
        .await?
        .ok_or(ServiceError::Unauthorized)?;
    
    let privileges = self.permission_service
        .get_privileges_for_current_user(context)
        .await?;
    
    Ok(UserInfo {
        username: user_id,
        privileges,
    })
}
```

## Middleware Configuration

The authentication middleware is applied in a specific order:

```rust
let app = Router::new()
    .route("/api/...", /* routes */)
    .layer(middleware::from_fn_with_state(
        rest_state.clone(),
        forbid_unauthenticated::<RestState>, // Block unauthenticated requests
    ))
    .layer(middleware::from_fn_with_state(
        rest_state.clone(),
        context_extractor::<RestState>,      // Extract user context
    ))
    .layer(CookieManagerLayer::new())        // Cookie management
    // OIDC layers added here in production mode
    ;
```

## Troubleshooting

### Common Issues

1. **401 Unauthorized in Development**
   - Ensure `mock_auth` feature is enabled
   - Check that admin users are created on startup

2. **OIDC Configuration Errors**
   - Verify all environment variables are set correctly
   - Ensure ISSUER URL is reachable
   - Check CLIENT_ID and CLIENT_SECRET match OIDC provider configuration

3. **Session Expiry**
   - Sessions expire after 50 minutes of inactivity (OIDC)
   - App session cookies expire after 365 days

4. **Permission Denied (403)**
   - Verify user has required role
   - Check role has necessary privileges
   - Use `/api/permission/user/{user}/roles` to inspect user's roles

### Debug Logging

Enable trace logging to debug authentication issues:

```rust
// For local development
#[cfg(feature = "local_logging")]
let subscriber = tracing_subscriber::FmtSubscriber::builder()
    .with_max_level(tracing::Level::TRACE)
    .pretty()
    .with_file(true)
    .finish();
```

## Security Considerations

1. **HTTPS Required**: OIDC authentication requires HTTPS in production
2. **Secure Cookies**: Session cookies are marked as secure, httpOnly, and SameSite=Strict
3. **CSRF Protection**: SameSite cookie attribute provides CSRF protection
4. **Session Management**: Sessions stored server-side with client receiving only session ID
5. **Role-Based Access**: Fine-grained permissions through role-privilege system

## Migration from Mock to OIDC

When moving from development to production:

1. Disable `mock_auth` feature flag
2. Enable `oidc` feature flag
3. Set required environment variables
4. Configure OIDC provider with correct redirect URIs
5. Create initial admin users through OIDC or database migration
6. Map OIDC users to application users via the user table