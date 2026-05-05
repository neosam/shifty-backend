# External Integrations

**Analysis Date:** 2026-05-05

## APIs & External Services

**Shifty Backend REST API:**
- Primary integration: HTTP REST API at `http://localhost:3000` (configurable)
- Client: reqwest 0.12.15 async HTTP client
- Location: `src/api.rs` (40KB file with ~200 async functions)
- Development proxy: Dioxus.toml defines 23 backend endpoint proxies

**Backend Endpoint Categories:**

**Authentication & User Management:**
- `GET /auth-info` - Fetch current user and privileges (`fetch_auth_info()`)
- `POST /authenticate` - Login endpoint proxy
- `POST /user-invitation` - Generate user invitations (`create_invitation()`)

**Employee & Sales Person Management:**
- `GET /sales-person` - List all sales persons (`get_sales_persons()`)
- `GET /sales-person/{id}` - Get specific sales person details
- `POST /sales-person` - Create new sales person
- `PUT /sales-person` - Update sales person
- `DELETE /sales-person/{id}` - Delete sales person
- `GET /permission` - Fetch user permissions

**Shift Planning:**
- `GET /shiftplan-catalog` - List all shift plans (`get_all_shiftplans()`)
- `POST /shiftplan-catalog` - Create shift plan (`create_shiftplan()`)
- `GET /shiftplan-info/{id}` - Get shift plan details
- `GET /shiftplan-edit/{id}` - Get editable shift plan
- `POST /shiftplan-edit/{id}` - Update shift plan
- `GET /sales-person-shiftplan/{id}` - Get sales person shift plan assignment

**Slots & Bookings:**
- `GET /slot/week/{year}/{week}/{shiftplan_id}` - Fetch slots for week (`get_slots()`)
- `POST /booking` - Create booking assignment
- `GET /booking` - List bookings
- `PUT /booking` - Update booking
- `DELETE /booking/{id}` - Delete booking
- `GET /booking-information` - Get booking summaries
- `GET /booking-log` - Fetch booking change history

**Working Hours & Extra Hours:**
- `GET /extra-hours/{type}` - Fetch extra hours (vacation, sick leave, etc.)
- `POST /extra-hours` - Create extra hours entry
- `PUT /extra-hours/{id}` - Update extra hours
- `DELETE /extra-hours/{id}` - Delete extra hours
- `POST /custom-extra-hours` - Create custom extra hours
- `GET /working-hours/{sales_person_id}/{year}/{week}` - Get working hours for period

**Reports & Summaries:**
- `GET /report` - Generate work reports
- `GET /block-report` - Get block/shift reports
- `GET /billing-period` - List billing periods
- `POST /billing-period` - Create billing period
- `GET /employee-report` - Employee work details report

**Configuration & Templates:**
- `GET /blocks` - List block definitions
- `POST /blocks` - Create block
- `GET /special-days` - List special days/holidays
- `POST /special-days` - Create special day
- `GET /text-templates` - List email/SMS templates
- `POST /text-templates` - Create template
- `PUT /text-templates/{id}` - Update template
- `GET /week-message` - Get week-specific messages

**Version & Health:**
- `GET /version` - API version endpoint

## Shared Type System

**rest-types workspace dependency** (`rest-types/` directory):
- Shared Rust types for serialization/deserialization between frontend and backend
- Located: `rest-types/Cargo.toml` (version 1.0.5-dev)
- Key types (imported in `src/api.rs`):
  - `ShiftplanTO` - Shift plan data transfer object
  - `SalesPersonTO` - Employee/sales person
  - `SlotTO` - Time slot definition
  - `BookingTO` - Shift assignment
  - `ExtraHoursTO` - Vacation, sick leave, overtime
  - `BillingPeriodTO` - Billing period for reports
  - `BlockTO` - Block/shift type definition
  - `UserTO`, `RoleTO` - User and role info
  - `WeeklySummaryTO` - Aggregated weekly hours
  - `EmployeeReportTO`, `EmployeeWorkDetailsTO` - Report data
  - `TextTemplateTO` - Email/message templates
  - `WeekMessageTO` - Week-specific messages
  - `SpecialDayTO` - Holiday/special day definitions

- Serialization: serde (1.0.219) with rc feature for Arc/Rc types
- Optional feature: `service-impl` (disabled by default) provides From traits for backend service objects

## Data Storage

**Browser Local Storage:**
- Implementation: Web Storage API via `web-sys::window().local_storage()`
- Location: `src/service/ui_prefs.rs`
- Use case: UI preferences (working hours layout preference: cards/table)
- Key: `shifty.ui.workingHoursLayout`
- Read: `get_working_hours_layout()` → returns enum (Cards/Table)
- Write: `set_working_hours_layout(layout)` → persists to localStorage

**No persistent database:**
- Frontend is stateless — all data flows from backend REST API
- Session state managed via Dioxus component state and Rc<T> borrowed data

**No file storage:**
- File uploads not detected in codebase
- Shift plans and data stored entirely on backend

**No caching layer:**
- In-memory caching via Dioxus component signals (Dioxus 0.6.1)
- No Redis, Memcached, or other external cache

## Authentication & Identity

**Auth Provider:** Custom (backend-driven)

**Flow:**
1. Frontend fetches `/auth-info` endpoint on app load
2. Backend returns `AuthInfo` struct with:
   - `user` (username)
   - `privileges` (list of permission strings)
   - `authenticated` (boolean)
3. Location: `src/state/auth_info.rs`, `src/api.rs:fetch_auth_info()`

**Privilege Checking:**
- Method: `AuthInfo::has_privilege(privilege: &str) -> bool`
- Example privileges: Admin, SalesPerson, Manager, etc. (backend-defined)
- Components check privileges before rendering restricted UI

**Session Management:**
- No explicit session tokens detected in frontend code
- Backend handles session/authentication via HTTP headers or cookies
- Frontend proxies requests through development server (Dioxus.toml)

## Monitoring & Observability

**Error Tracking:** None detected
- No Sentry, Rollbar, or similar integration

**Logging:**
- Framework: tracing 0.1.41 + dioxus-logger 0.6.2
- Pattern: `info!()`, `debug!()` macros in async functions
- Example: `info!("Fetching slots")` in `get_slots()` before API call
- Output: Browser console (via dioxus-logger integration)
- No centralized log collection

**Browser Developer Tools:**
- WASM panic messages logged to console
- Network tab shows all proxied API requests

## CI/CD & Deployment

**Hosting:** Not configured in frontend code
- Production build: `dx build` outputs to `dist/` directory
- Deployment: Static files served by web server of choice
- Expected: Reverse proxy with backend API proxying (Nginx, Apache, etc.)

**CI Pipeline:** Not detected in frontend codebase
- No GitHub Actions, GitLab CI, or CI config files in shifty-dioxus/
- Check shifty-backend/ or parent shifty/ repository for CI setup

**Build Artifacts:**
- `dist/` - Production WASM bundle and assets (not committed)
- `assets/tailwind.css` - Generated CSS from Tailwind CLI (not committed)

## Environment Configuration

**Frontend Runtime Config** (`assets/config.json`):
```json
{
  "backend": "http://backend-url:3000",
  "application_title": "Shifty",
  "is_prod": true,
  "env_short_description": "PROD",
  "show_vacation": true
}
```
- Loaded by `load_config()` in `src/api.rs` via HTTP GET
- Allows environment-specific backend URL configuration
- Must be deployed alongside built assets

**No .env files:**
- All configuration loaded from runtime config.json (web-based)
- No environment variables injected at build time

**Backend URL Sources:**
1. Development: Proxied via Dioxus dev server (http://localhost:3000) → Dioxus.toml
2. Production: Resolved from `assets/config.json` → backend URL
3. Example: `let url = format!("{}/slot/week/{year}/{week}/{shiftplan_id}", config.backend)`

## Webhooks & Callbacks

**Incoming Webhooks:** None detected

**Outgoing Webhooks:**
- No external service webhooks (Stripe, Slack, email providers, etc.)
- Backend may handle webhook generation for emails/templates

**Form Submissions:**
- All user actions → REST API POST/PUT/DELETE calls
- Example: Create booking → `POST /booking` with BookingTO payload
- No form action attributes — all JS-driven

## State Management

**Dioxus Signals & Hooks:**
- Component state: Dioxus 0.6.1 reactive signals
- Example pattern in `src/component/`: Components manage local state with `use_state()` hook
- Data loading: `use_coroutine()` for async API calls (reqwest HTTP requests)

**Application State:**
- `Config` - Backend URL and environment config (loaded once on app startup)
- `AuthInfo` - Current user and permissions (fetched on app load)
- User selections/filters - Stored in component signal state

**Caching Strategy:**
- `Rc<[T]>` (reference-counted slices) for shared read-only data from API responses
- Example: `Rc<[SalesPerson]>` for employee list, `Rc<[SlotTO]>` for weekly slots
- Data refetched on navigation/week change (no persistent cache)

---

*Integration audit: 2026-05-05*
