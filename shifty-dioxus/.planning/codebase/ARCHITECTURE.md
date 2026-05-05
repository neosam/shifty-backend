<!-- refreshed: 2026-05-05 -->
# Architecture

**Analysis Date:** 2026-05-05

## System Overview

```text
┌─────────────────────────────────────────────────────────────────────┐
│                         Pages (Route Handlers)                       │
│   ┌─────────────┐  ┌─────────────┐  ┌──────────────┐  ┌──────────┐ │
│   │  ShiftPlan  │  │ Employees   │  │BillingPeriod│  │MyShifts  │ │
│   │`src/page/`  │  │  Details    │  │   Details   │  │`src/page/`
│   └──────┬──────┘  └──────┬──────┘  └──────┬───────┘  └────┬─────┘ │
└──────────┼────────────────┼──────────────────┼──────────────┼───────┘
           │                │                  │              │
           ▼                ▼                  ▼              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Components (Reusable UI Elements)                 │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│   │ WeekView     │  │ EmployeeView │  │ TopBar, Footer, Dialog   │  │
│   │ `src/component/`                  │ `src/component/atoms/`   │  │
│   │ (65KB file)  │  │(40KB file)   │  │ (Btn, PersonChip, etc)  │  │
│   └──────┬───────┘  └──────┬───────┘  └──────────────────────────┘  │
│          │                 │                     │                   │
│          └─────────────────┼─────────────────────┘                   │
└──────────────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              Services (Business Logic & State Management)             │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────┐ │
│   │ employee     │  │ billing_     │  │ config       │  │ i18n    │ │
│   │ _service     │  │ period       │  │ _service     │  │_service │ │
│   │ `src/service/`  │ _service     │  │ (loads app   │  │ (locale)│ │
│   │ (GlobalSignal)  │(GlobalSignal)│  │  config)     │  │         │ │
│   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └────┬────┘ │
│          │ EMPLOYEE_STORE │ BILLING_PERIOD_STORE             │       │
│          │ ExtraHours     │ BillingPeriods                    │       │
│          │ custom_defs    │ selected_period                   │       │
│          └────────────────┴────────────────────────────────────┘       │
└──────────────────────────────────┬──────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    State (Data Structures)                           │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│   │ Employee     │  │ Shiftplan    │  │ Config       │              │
│   │ SalesPerson  │  │ Booking      │  │ AuthInfo     │              │
│   │ `src/state/` │  │ Slot         │  │ `src/state/` │              │
│   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘              │
└──────────┼────────────────┼──────────────────┼────────────────────┘
           │                │                  │
           └────────────────┼──────────────────┘
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     API / Loader                                     │
│   ┌──────────────┐  ┌──────────────┐                                │
│   │ api.rs       │  │ loader.rs    │                                │
│   │ (reqwest     │  │ (Converts    │                                │
│   │  HTTP calls) │  │  TO → State) │                                │
│   └──────┬───────┘  └──────┬───────┘                                │
└──────────┼────────────────┼──────────────────────────────────────────┘
           │                │
           └────────────────┴──────────────────────┐
                                                   ▼
                                 ┌─────────────────────────────┐
                                 │ Shifty Backend API (Axum)   │
                                 │ http://localhost:3000       │
                                 └─────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| Pages | Full page components (route handlers, coordinate multiple services) | `src/page/` |
| Components | Reusable UI elements (forms, tables, dialogs, views) | `src/component/` |
| Atoms | Primitive UI components (buttons, chips, nav buttons) | `src/component/atoms/` |
| Services | Business logic, coroutine handlers, GlobalSignal management | `src/service/` |
| State | Domain data structures, view models, type definitions | `src/state/` |
| API | HTTP communication with backend via reqwest | `src/api.rs` |
| Loader | Data transformation (REST TO → Frontend State) | `src/loader.rs` |
| Router | Route definitions and navigation | `src/router.rs` |
| Auth | Authentication and authorization context | `src/auth.rs` |
| i18n | Multi-language support (English, German, Czech) | `src/i18n/` |

## Pattern Overview

**Overall:** Component-Service-State Pattern (CSSP)

**Key Characteristics:**
- **Unidirectional data flow**: State ← Services ← Components ← Pages
- **GlobalSignal-based state management**: Dioxus signals for shared global state
- **Coroutine-based services**: Async message handling via `UnboundedReceiver<Action>`
- **Type-driven architecture**: Strong typing from REST types through state to components
- **Separation of concerns**: UI (Components), Business Logic (Services), Data (State, API)

## Layers

**Page Layer:**
- Purpose: Route-driven full-page components that coordinate services and compose components
- Location: `src/page/`
- Contains: Page components like `ShiftPlan`, `EmployeeDetails`, `BillingPeriodDetails`
- Depends on: Components, Services (via coroutine handles), State (via GlobalSignals)
- Used by: Router (`src/router.rs`) as route targets

**Component Layer:**
- Purpose: Reusable UI building blocks (forms, tables, modals, views)
- Location: `src/component/` and `src/component/atoms/`
- Contains: RSX-based React-like components with event handlers
- Depends on: State structures, Services, i18n, CSS (Tailwind)
- Used by: Pages and other components

**Service Layer:**
- Purpose: Async business logic, API coordination, cross-cutting concerns
- Location: `src/service/`
- Contains: Coroutine-based services managing GlobalSignals (e.g., `EMPLOYEE_STORE`, `CONFIG`)
- Depends on: API layer, State structures, Error handling
- Used by: Components/Pages via `use_coroutine_handle` and `GlobalSignal.read()`

**State Layer:**
- Purpose: Data structures, domain models, type definitions
- Location: `src/state/`
- Contains: Structs like `Employee`, `SalesPerson`, `Shiftplan`, `Booking`
- Depends on: rest-types (shared with backend)
- Used by: Components, Services, API layer

**API/Loader Layer:**
- Purpose: HTTP communication and data transformation
- Location: `src/api.rs`, `src/loader.rs`
- Contains: REST client functions (reqwest) and type converters (TO → State)
- Depends on: rest-types, State structures, Error handling
- Used by: Services to fetch/mutate data

## Data Flow

### Primary Request Path (e.g., Loading Employee Data)

1. **Page Triggered** (`src/page/employee_details.rs:50`)
   - User navigates to employee page
   - Component calls `use_resource(move || loader::load_sales_persons(config))`

2. **Loader Invoked** (`src/loader.rs:28-36`)
   - `load_sales_persons()` fetches data via API
   - Converts `SalesPersonTO` → `SalesPerson` (domain model)
   - Returns `Rc<[SalesPerson]>` to component

3. **Service Action Dispatched** (`src/page/shiftplan.rs:89-100`)
   - Component calls `use_coroutine_handle::<EmployeeAction>()`
   - Sends action: `EmployeeAction::LoadEmployeeDataUntilNow { sales_person_id }`
   - Service receives in `UnboundedReceiver`

4. **Service Processes** (`src/service/employee.rs:74-100`)
   - Service matches action, calls loader function
   - Updates `EMPLOYEE_STORE: GlobalSignal<EmployeeStore>`
   - Dispatches error to `ERROR_STORE` on failure

5. **Component Re-renders** (Dioxus reactivity)
   - Component re-reads `EMPLOYEE_STORE.read()`
   - RSX re-renders with new data

### Authentication Flow

1. **App Initialization** (`src/app.rs:12-40`)
   - App component calls `use_coroutine(service::config::config_service)`
   - `config_service` loads `Config` from `/assets/config.json`
   - Calls `auth::load_auth_info()`

2. **Auth Service** (`src/service/auth.rs`)
   - Fetches `/auth-info` endpoint
   - Updates `AUTH: GlobalSignal<AuthStore>` with authenticated status

3. **Auth Component** (`src/auth.rs:12-24`)
   - Reads `AUTH.read().auth_info`
   - Conditionally renders authenticated (Router) or unauthenticated (LoginPage)

### i18n/Localization Flow

1. **Service Initialization** (`src/app.rs:17`)
   - App calls `use_coroutine(service::i18n::i18n_service)`

2. **Locale Detection** (`src/service/i18n.rs`)
   - Detects browser locale from `navigator.language`
   - Sets `I18N: GlobalSignal<Locale>`

3. **Component Translation** (`src/page/billing_period_details.rs:37-38`)
   - Component reads: `I18N.read().t(Key::InvalidBillingPeriodId)`
   - Returns translated string from `src/i18n/{en,de,cs}.rs`

**State Management:**
- **GlobalSignals**: `CONFIG`, `EMPLOYEE_STORE`, `BILLING_PERIOD_STORE`, `I18N`, `AUTH`
- **Coroutine-based**: Services receive `UnboundedReceiver<Action>` for async message handling
- **Resource-based**: Components use `use_resource()` for async data loading with caching

## Key Abstractions

**GlobalSignal Pattern (Dioxus Reactive State):**
- Purpose: Shared, globally accessible state without prop drilling
- Examples: `CONFIG.read()`, `EMPLOYEE_STORE.write()` in `src/service/*.rs`
- Pattern: Define as `pub static STORE: GlobalSignal<T> = Signal::global(|| T::default())`
- Update via: `.write()` returns mutable reference, `.read()` returns immutable snapshot

**Coroutine Service Pattern:**
- Purpose: Long-lived async message handlers for side effects
- Examples: `config_service`, `employee_service`, `billing_period_service`
- Pattern: `async fn service(mut rx: UnboundedReceiver<Action>)` launched via `use_coroutine()`
- Message flow: Components send via `coroutine_handle.send(Action)` → Service processes
- Service updates GlobalSignals on completion, errors sent to `ERROR_STORE`

**Loader Functions:**
- Purpose: Bridge API (REST TO types) and State (domain types)
- Examples: `loader::load_sales_persons()`, `loader::load_bookings()`
- Pattern: `async fn load_x() -> Result<Rc<[X]>, ShiftyError>` with type conversion
- Used in: `use_resource()` closures in pages/components

**ColumnViewSlot Generic Pattern** (`src/component/week_view.rs:41-75`):
- Purpose: Reusable time-slot rendering with generic custom data
- Pattern: `pub struct ColumnViewSlot<CustomData: Identifiable>` with associated handlers
- Example: `ColumnViewItem<()>` for slots without extra data, generic for other uses

## Entry Points

**Application Entry** (`src/main.rs:25-29`):
- Location: `src/main.rs`
- Triggers: Browser loads WASM module
- Responsibilities: Initialize logger, launch Dioxus app via `launch(app::App)`

**App Component** (`src/app.rs:12-64`):
- Location: `src/app.rs`
- Triggers: Application initialization
- Responsibilities: 
  - Initialize all services via `use_coroutine()` calls (config, theme, i18n, auth, etc.)
  - Load configuration from `/assets/config.json`
  - Render root layout (TopBar, Footer, Router, Auth guard)

**Router** (`src/router.rs:19-51`):
- Location: `src/router.rs`
- Defines: 13 routes via `#[derive(Routable)]` enum `Route`
- Examples: `#[route("/shiftplan/:year/:week")]` → `ShiftPlanDeep { year, week }`
- Renders: Page component matching current route

**Page Entry Points** (Examples):
- `src/page/shiftplan.rs:88` → `ShiftPlan()` component handles shift planning UI
- `src/page/billing_period_details.rs:31` → `BillingPeriodDetails()` handles billing period details
- `src/page/employee_details.rs` → Displays employee records and working hours

## Architectural Constraints

- **Threading:** Single-threaded event loop (Dioxus/WASM model). All async work via `spawn()` and coroutines. No worker threads.
- **Global state:** Multiple `GlobalSignal` singletons: `CONFIG`, `EMPLOYEE_STORE`, `BILLING_PERIOD_STORE`, `I18N`, `AUTH`, `ERROR_STORE`, `BOOKING_CONFLICTS_STORE`. Mutation requires `.write()` call.
- **Circular imports:** Potential between pages and services (pages call services, services update global state read by pages). Mitigated by using coroutine handles via `use_coroutine_handle<T>()` instead of direct function calls.
- **API Communication:** All backend calls via `reqwest` in `src/api.rs`. Blocking on `.await` in async contexts (coroutines, resources).
- **Reactive Updates:** Changes to `GlobalSignal` values trigger component re-renders via Dioxus reactivity. Updates must use `.write()` inside services.
- **Cross-service Communication:** Via shared `GlobalSignal` state (eventual consistency), not direct message passing between services.

## Anti-Patterns

### Blocking API Calls in Component Render

**What happens:** Components directly call async API functions without `use_resource()` or `spawn()`

**Why it's wrong:** Causes render blocking, poor UX, multiple simultaneous requests, race conditions

**Do this instead:** Use `use_resource()` for async data loading (`src/page/employee_details.rs:50`) or `spawn()` inside event handlers with `.send()` to coroutine handles

### Writing to GlobalSignal from Components

**What happens:** Components directly modify state via `.write()` instead of sending to services

**Why it's wrong:** Breaks service-layer control flow, makes debugging hard, inconsistent with loader functions

**Do this instead:** Components send action to service via `coroutine_handle.send(Action)`, service processes and updates state (`src/page/shiftplan.rs:140-160`)

### Forgetting LocaleChange in i18n Updates

**What happens:** Translations added to English file only, not German/Czech locales

**Why it's wrong:** Missing translations in non-English locales, UI shows missing keys or wrong language

**Do this instead:** Always add translation key to `Key` enum in `src/i18n/mod.rs` and implement in all three locale files: `en.rs`, `de.rs`, `cs.rs`

### Using `transform: scale` in WeekView Instead of CSS `zoom`

**What happens:** Zoom implementation uses `transform: scale()` causing scrollbar gaps

**Why it's wrong:** Scale doesn't affect layout, creates visual artifacts in fixed-column scroll views

**Do this instead:** Use CSS `zoom` property directly in styles (`src/component/week_view.rs` uses `zoom` for viewport adjustment)

## Error Handling

**Strategy:** Explicit Result types with `ShiftyError`, error propagation to global `ERROR_STORE`

**Patterns:**
- API calls return `Result<T, reqwest::Error>` wrapped in `ShiftyError`
- Services catch errors in coroutine message handlers, write to `ERROR_STORE`
- Components read `ERROR_STORE.read().error` for display
- User-facing errors shown in error overlay or modal components

## Cross-Cutting Concerns

**Logging:** Configured via `dioxus_logger::init(Level::INFO)` in `src/main.rs:27`. Services log via `tracing::info!()` macro.

**Validation:** Type validation at API boundaries (serde deserialization), domain validation in state/service layer (e.g., date range checks).

**Authentication:** Global `AUTH: GlobalSignal<AuthStore>` checked in `Auth` component (`src/auth.rs`) to gate Router access. Backend validates session via `/auth-info` endpoint.

**Timezone Handling:** Uses `time` crate for date operations. Dates from backend assumed UTC, client-side formatting per locale settings.

---

*Architecture analysis: 2026-05-05*
