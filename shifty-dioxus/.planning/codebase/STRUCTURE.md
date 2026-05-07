# Codebase Structure

**Analysis Date:** 2026-05-05

## Directory Layout

```
shifty-dioxus/
├── src/                          # Rust/WASM source code
│   ├── main.rs                   # Application entry point
│   ├── app.rs                    # Root app component, service initialization
│   ├── router.rs                 # Route definitions and navigation
│   ├── auth.rs                   # Authentication component and guards
│   ├── api.rs                    # REST API client functions (40KB)
│   ├── loader.rs                 # Data loaders and type converters (27KB)
│   ├── base_types.rs             # Utility types (ImStr, format_hours)
│   ├── js.rs                     # JavaScript interop via wasm-bindgen
│   ├── error.rs                  # Error types and handling
│   │
│   ├── page/                     # Full-page components (route handlers)
│   │   ├── mod.rs                # Page module exports
│   │   ├── home.rs               # Landing page
│   │   ├── shiftplan.rs          # Shift planning view (complex)
│   │   ├── weekly_overview.rs    # Weekly summary view
│   │   ├── employees.rs          # Employee list page
│   │   ├── employee_details.rs   # Single employee details
│   │   ├── my_employee_details.rs # Current user's details
│   │   ├── my_shifts.rs          # Current user's shifts
│   │   ├── billing_periods.rs    # Billing period list
│   │   ├── billing_period_details.rs # Single billing period details
│   │   ├── sales_person_details.rs # Sales person (employee) management
│   │   ├── user_management.rs    # User administration
│   │   ├── user_details.rs       # Single user details
│   │   ├── custom_extra_hours_management.rs # Extra hours configuration
│   │   ├── text_template_management.rs # Report template management
│   │   ├── not_authenticated.rs  # Login page
│   │   └── blog.rs               # Blog/info pages
│   │
│   ├── component/                # Reusable UI components
│   │   ├── mod.rs                # Component module exports
│   │   ├── atoms/                # Primitive components
│   │   │   ├── mod.rs            # Atoms exports
│   │   │   ├── btn.rs            # Button component (variants: primary, danger, ghost)
│   │   │   ├── nav_btn.rs        # Navigation button with icon
│   │   │   ├── person_chip.rs    # Employee/person display chip
│   │   │   ├── media_query.rs    # Responsive CSS media query hook
│   │   │   └── tuple_row.rs      # Two-column row layout
│   │   │
│   │   ├── form/                 # Form components
│   │   │   └── mod.rs            # Form module exports
│   │   │
│   │   ├── week_view.rs          # Calendar week grid (65KB, critical)
│   │   ├── employee_view.rs      # Employee detail view (40KB, complex)
│   │   ├── employees_list.rs     # Sidebar employee list
│   │   ├── employees_shell.rs    # Employee list layout wrapper
│   │   ├── top_bar.rs            # Navigation header (42KB)
│   │   ├── footer.rs             # Application footer
│   │   ├── working_hours_mini_overview.rs # Hours summary widget
│   │   ├── working_hours_overview_layout_toggle.rs # Layout switcher
│   │   ├── employee_weekly_histogram.rs # Chart of weekly hours
│   │   ├── weekly_overview_chart.rs # Summary chart view
│   │   ├── booking_log_table.rs  # Booking history table (20KB)
│   │   ├── day_aggregate_view.rs # Day summary view
│   │   ├── extra_hours_modal.rs  # Modal for adding extra hours (22KB)
│   │   ├── add_extra_hours_form.rs # Extra hours input form
│   │   ├── add_extra_hours_choice.rs # Extra hours category picker
│   │   ├── add_extra_days_form.rs # Vacation/leave input form
│   │   ├── employee_short.rs     # Compact employee display
│   │   ├── employee_work_details_form.rs # Hours contract form (20KB)
│   │   ├── contract_modal.rs     # Modal for contract editing (20KB)
│   │   ├── slot_edit.rs          # Shift slot editor
│   │   ├── shiftplan_tab_bar.rs  # Shift plan view mode tabs
│   │   ├── user_management_tab_bar.rs # User admin tabs
│   │   ├── dialog.rs             # Generic modal dialog (21KB)
│   │   ├── dropdown_base.rs      # Dropdown menu component
│   │   ├── overlay.rs            # Overlay/modal background
│   │   ├── tooltip.rs            # Tooltip component
│   │   ├── base_components.rs    # Common component helpers
│   │   ├── error_view.rs         # Error display component
│   │   └── mod.rs                # Component exports and re-exports
│   │
│   ├── service/                  # Async business logic & state management
│   │   ├── mod.rs                # Service module exports
│   │   ├── config.rs             # App configuration loading (GlobalSignal: CONFIG)
│   │   ├── auth.rs               # Authentication state (GlobalSignal: AUTH)
│   │   ├── employee.rs           # Employee data (GlobalSignal: EMPLOYEE_STORE)
│   │   ├── billing_period.rs     # Billing periods (GlobalSignal: BILLING_PERIOD_STORE)
│   │   ├── employee_work_details.rs # Work hours contracts
│   │   ├── booking_conflict.rs   # Booking conflicts detection
│   │   ├── booking_log.rs        # Booking history
│   │   ├── weekly_summary.rs     # Weekly totals and summaries
│   │   ├── working_hours_mini.rs # Mini hours display state
│   │   ├── text_template.rs      # Report templates management
│   │   ├── slot_edit.rs          # Slot editing state and actions
│   │   ├── i18n.rs               # Localization (GlobalSignal: I18N)
│   │   ├── theme.rs              # Dark/light mode (GlobalSignal: THEME)
│   │   ├── dropdown.rs           # Dropdown state management
│   │   ├── tooltip.rs            # Tooltip state management
│   │   ├── ui_prefs.rs           # UI preferences (layout, zoom)
│   │   └── error.rs              # Error state (GlobalSignal: ERROR_STORE)
│   │
│   ├── state/                    # Data structures and domain models
│   │   ├── mod.rs                # State module exports
│   │   ├── config.rs             # App Config struct
│   │   ├── auth_info.rs          # AuthInfo and User identity
│   │   ├── shiftplan.rs          # Shiftplan, Slot, Booking, SalesPerson
│   │   ├── employee.rs           # Employee, ExtraHours, WorkingHours
│   │   ├── employee_work_details.rs # Contract and hours details
│   │   ├── sales_person_available.rs # Availability/unavailability
│   │   ├── booking_log.rs        # Booking history entry
│   │   ├── weekly_overview.rs    # WeeklySummary structure
│   │   ├── week.rs               # Week calculations
│   │   ├── text_template.rs      # TextTemplate definition
│   │   ├── dropdown.rs           # DropdownEntry structures
│   │   ├── slot_edit.rs          # SlotEditItem state
│   │   ├── tooltip.rs            # Tooltip structures
│   │   └── user_management.rs    # User, ShiftplanAssignment
│   │
│   ├── i18n/                     # Internationalization (multi-language)
│   │   ├── mod.rs                # Key enum, Locale, LocaleDef trait
│   │   ├── i18n.rs               # I18n struct and methods
│   │   ├── en.rs                 # English translations
│   │   ├── de.rs                 # German translations
│   │   └── cs.rs                 # Czech translations
│   │
│   └── tests/                    # Test utilities and fixtures
│       └── mod.rs                # Test module
│
├── rest-types/                   # Shared types with backend
│   └── lib.rs                    # REST API type definitions (Transfer Objects)
│
├── assets/                       # Static assets
│   ├── config.json               # Runtime configuration
│   └── tailwind.css              # Compiled Tailwind CSS
│
├── Cargo.toml                    # Rust dependencies
├── Cargo.lock                    # Dependency lock file
├── Dioxus.toml                   # Dioxus framework configuration
├── tailwind.config.js            # Tailwind CSS configuration
├── index.html                    # HTML entry point
└── input.css                     # Tailwind CSS input

```

## Directory Purposes

**`src/`:**
- Purpose: All Rust/WASM source code
- Contains: Components, services, state, API, routing, i18n
- Key files: `main.rs` (entry), `router.rs` (routes), `app.rs` (root component)

**`src/page/`:**
- Purpose: Full-page components that handle specific routes
- Contains: Page handlers composed of smaller components and services
- Key files: `shiftplan.rs` (shift planning), `employee_details.rs`, `billing_period_details.rs`
- Pattern: Each file ≈ one route, imports components and services, manages coroutine lifecycle

**`src/component/`:**
- Purpose: Reusable UI building blocks
- Contains: Form inputs, tables, dialogs, layout components, wrappers
- Subdirectories: `atoms/` (primitive buttons, chips), `form/` (form-specific)
- Size: `week_view.rs` (65KB) and `employee_view.rs` (40KB) are largest/most complex

**`src/component/atoms/`:**
- Purpose: Smallest, most reusable components (buttons, chips, row layouts)
- Contains: `Btn` (button variants), `PersonChip` (employee display), `NavBtn`, `TupleRow`
- Pattern: No internal state, pure presentation via props

**`src/service/`:**
- Purpose: Business logic, coroutine-based message handlers, GlobalSignal management
- Contains: Service functions and their associated GlobalSignal stores
- Pattern: Each service file defines an enum for Actions, a GlobalSignal for state, and an async handler function
- Example: `config.rs` defines `CONFIG: GlobalSignal<Config>` and `async fn config_service(rx)`

**`src/state/`:**
- Purpose: Data structures (domain models, view models, type definitions)
- Contains: Rust structs corresponding to REST API types and derived domain models
- Examples: `SalesPerson`, `Employee`, `Shiftplan`, `Booking`, `Slot`
- Dependencies: Uses `rest-types` for shared types with backend

**`src/i18n/`:**
- Purpose: Multi-language support (English, German, Czech)
- Contains: `Key` enum (all translation keys), trait `LocaleDef` (formatting), locale files
- Pattern: Define key in `mod.rs`, implement in `en.rs`, `de.rs`, `cs.rs`
- Services: `i18n_service` manages `I18N: GlobalSignal<Locale>` for current locale

**`rest-types/`:**
- Purpose: Shared API type definitions (Transfer Objects = TO suffix)
- Contains: Types like `SalesPersonTO`, `EmployeeReportTO`, `BookingTO`
- Status: Compiled from backend OpenAPI schema
- Used by: `api.rs` for deserialization, `loader.rs` for type conversion

**`assets/`:**
- Purpose: Static files served at runtime
- Contains: `config.json` (runtime configuration), `tailwind.css` (compiled styles)
- `config.json` structure: `{ "backend": "http://localhost:3000", "application_title": "Shifty", ... }`

## Key File Locations

**Entry Points:**
- `src/main.rs` - Application bootstrap (logger init, Dioxus launch)
- `src/app.rs` - Root component (service initialization, root layout)
- `src/router.rs` - Route definitions (13 routes via `Route` enum)
- `Dioxus.toml` - Framework config, backend proxy URL

**Configuration:**
- `Cargo.toml` - Rust dependencies (dioxus, reqwest, serde, tailwind)
- `tailwind.config.js` - Tailwind CSS custom colors, fonts, utilities
- `assets/config.json` - Runtime app config (loaded at startup)

**Core Logic:**
- `src/api.rs` - HTTP client (40KB, ~200 API functions)
- `src/loader.rs` - Data loaders with type conversion (27KB)
- `src/service/*.rs` - Business logic and state management (12 service modules)

**Testing:**
- `src/tests/` - Test module (utilities and fixtures)
- No unit tests found in codebase; test via integration testing or manual QA

**Styling:**
- `input.css` - Tailwind input (custom utilities like `scale-down-50`, `scale-down-75`)
- `assets/tailwind.css` - Compiled output (generated by tailwind CLI)
- `tailwind.config.js` - Custom config (print styles, custom colors)

## Naming Conventions

**Files:**
- Snake_case for Rust source files: `employee_details.rs`, `booking_log.rs`
- Component file = component name: `WeekView` struct in `week_view.rs`, `TopBar` in `top_bar.rs`
- Service file = domain entity: `employee.rs` handles `EMPLOYEE_STORE` and `EmployeeAction`
- State file = domain entity: `shiftplan.rs` defines `Shiftplan`, `Booking`, `Slot` structs

**Directories:**
- Lowercase plural for grouped code: `src/page/`, `src/component/`, `src/service/`, `src/state/`
- Subdirs by category: `atoms/` for primitives, `form/` for input components

**Functions:**
- Async loaders: `load_x()` returns `Result<T, ShiftyError>`
- Service handlers: `x_service(mut rx: UnboundedReceiver<Action>)` with `async`
- API calls: `get_x()`, `post_x()`, `put_x()`, `delete_x()`
- Components: PascalCase, no suffix: `WeekView {}`, `EmployeeDetails {}`

**Variables:**
- GlobalSignals: UPPERCASE: `CONFIG`, `EMPLOYEE_STORE`, `I18N`, `ERROR_STORE`
- Component state: CamelCase with `use_signal`: `let mut selected_employee = use_signal(...)`
- Imports: Use full path or re-export via `mod.rs`: `use crate::service::config::CONFIG`

**Types:**
- Domain structs: PascalCase (e.g., `SalesPerson`, `ExtraHours`)
- Enums: PascalCase variants (e.g., `BillingPeriodAction::LoadBillingPeriod`)
- REST types: Suffix with `TO` (Transfer Object): `SalesPersonTO`, `EmployeeReportTO`
- Signals/Stores: Suffix with `Store` or bare: `EMPLOYEE_STORE`, `I18N`

## Where to Add New Code

**New Page (Route):**
- File: `src/page/new_feature.rs`
- Structure:
  ```rust
  use crate::{component::*, service::*, state::*, i18n::Key, ...};
  
  #[component]
  pub fn NewFeature(props: Props) -> Element {
      let config = CONFIG.read().clone();
      let service_handle = use_coroutine_handle::<ServiceAction>();
      
      rsx! { ... }
  }
  ```
- Register in: `src/router.rs` (add route enum variant)
- Export in: `src/page/mod.rs` (add pub use statement)

**New Component (Reusable UI):**
- File: `src/component/new_component.rs` (or `src/component/atoms/` if primitive)
- Structure:
  ```rust
  #[derive(PartialEq, Clone, Props)]
  pub struct NewComponentProps { ... }
  
  pub fn NewComponent(props: NewComponentProps) -> Element {
      rsx! { ... }
  }
  ```
- Export in: `src/component/mod.rs`
- Uses: Accept state/callbacks from parent, no direct service access

**New Service (Business Logic):**
- Files: `src/service/new_domain.rs` + add to `src/service/mod.rs`
- Structure:
  ```rust
  #[derive(Clone, PartialEq)]
  pub struct NewStore { ... }
  
  pub static NEW_STORE: GlobalSignal<NewStore> = Signal::global(|| NewStore::default());
  
  #[derive(Debug)]
  pub enum NewAction { ... }
  
  pub async fn new_service(mut rx: UnboundedReceiver<NewAction>) {
      while let Some(action) = rx.next().await {
          match action { ... }
      }
  }
  ```
- Initialize in: `src/app.rs` (add `use_coroutine(service::new_domain::new_service)`)

**New Data Type (State):**
- File: `src/state/new_model.rs` (or extend existing if related)
- Structure:
  ```rust
  #[derive(Clone, Debug, PartialEq)]
  pub struct NewModel { ... }
  ```
- Export in: `src/state/mod.rs`

**New API Endpoint:**
- Add function to: `src/api.rs`
- Pattern:
  ```rust
  pub async fn get_x(config: Config) -> Result<Rc<[XTO]>, reqwest::Error> {
      let url = format!("{}/endpoint", config.backend);
      let response = reqwest::get(url).await?;
      response.error_for_status_ref()?;
      Ok(response.json().await?)
  }
  ```
- Coordinate: Wrap call in loader function in `src/loader.rs` for type conversion

**New i18n Key:**
- Add to: `src/i18n/mod.rs` (Key enum)
- Implement in: `src/i18n/en.rs`, `src/i18n/de.rs`, `src/i18n/cs.rs`
- Usage: `I18N.read().t(Key::NewKeyName)`

## Special Directories

**`src/component/atoms/`:**
- Purpose: Primitive, reusable UI components
- Generated: No (hand-written)
- Committed: Yes
- Examples: `Btn`, `PersonChip`, `NavBtn` (all under 10KB each)

**`assets/`:**
- Purpose: Static files served at runtime
- Generated: `tailwind.css` (compiled from input.css by tailwind CLI)
- Committed: Only `config.json` should be committed (not tailwind.css)

**`dist/`:**
- Purpose: Build output (WASM bundle, HTML)
- Generated: Yes (by `dx build`)
- Committed: No (in .gitignore)

**`src/tests/`:**
- Purpose: Test utilities and integration test fixtures
- Generated: No (hand-written)
- Committed: Yes

**`rest-types/`:**
- Purpose: Shared types with backend (OpenAPI-generated)
- Generated: Yes (from backend OpenAPI schema)
- Committed: Yes (compiled Rust code)

---

*Structure analysis: 2026-05-05*
