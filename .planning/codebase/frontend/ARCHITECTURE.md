<!-- refreshed: 2026-05-07 -->
# Architecture

**Analysis Date:** 2026-05-07

## System Overview

```text
┌────────────────────────────────────────────────────────────────┐
│                       Browser (WASM)                           │
│  ┌────────────┐   ┌────────────┐   ┌────────────────────────┐  │
│  │   Pages    │──▶│ Components │──▶│   Atoms / Form Inputs  │  │
│  │ `src/page` │   │`src/comp..`│   │ `src/component/atoms`  │  │
│  └─────┬──────┘   └─────┬──────┘   └────────────────────────┘  │
│        │ read/write           ▲                                │
│        ▼                      │ subscribe                      │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              GlobalSignal Store (Dioxus)                 │  │
│  │   AUTH, CONFIG, I18N, EMPLOYEE_STORE, ERROR_STORE, …     │  │
│  │   `src/service/*::*_STORE` / `*::*` static signals       │  │
│  └─────────────────────┬────────────────────────────────────┘  │
│                        │ mutate via coroutines                 │
│                        ▼                                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           Service Layer (coroutines / actions)           │  │
│  │  `src/service/*.rs`  pub async fn *_service(rx)          │  │
│  └─────────────────────┬────────────────────────────────────┘  │
│                        │                                       │
│                        ▼                                       │
│  ┌─────────────────────────────┐  ┌────────────────────────┐   │
│  │   Loader (TO → state map)   │  │   API client (reqwest) │   │
│  │   `src/loader.rs`           │  │   `src/api.rs`         │   │
│  └─────────────────────┬───────┘  └─────────┬──────────────┘   │
│                        └─────────┬──────────┘                  │
│                                  ▼                             │
└──────────────────────────────────│─────────────────────────────┘
                                   │ HTTP / JSON (reqwest)
                                   ▼
┌────────────────────────────────────────────────────────────────┐
│  shifty-backend  (Axum, port 3000) — proxied via Dioxus.toml   │
│  shared DTOs:  `rest-types`  (path-dep, also at backend root)  │
└────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| App root | Mount global coroutines, gate Auth, render Router | `src/app.rs` |
| Router | Map URL paths to page components | `src/router.rs` |
| Auth gate | Switch authenticated vs. unauthenticated tree | `src/auth.rs` |
| Pages | Compose components, drive flow per route | `src/page/*.rs` |
| Components | Reusable UI (week_view, dialog, employee_view, …) | `src/component/*.rs` |
| Atoms | Smallest reusable building blocks (Btn, NavBtn, PersonChip, TupleRow) | `src/component/atoms/*.rs` |
| Form atoms | TextInput, TextareaInput, SelectInput, FormCheckbox, Field | `src/component/form/*.rs` |
| Services | Background coroutines that own a slice of `GlobalSignal` state | `src/service/*.rs` |
| State | Plain data structs and `From<*TO>` impls | `src/state/*.rs` |
| Loader | Wraps `api::*` calls, converts TOs → state types, sorts/joins | `src/loader.rs` |
| API client | Raw `reqwest` calls against the backend | `src/api.rs` |
| i18n | `I18n<Key, Locale>` translation map + locale-aware date format | `src/i18n/` |
| Error wrapper | `ShiftyError` (reqwest, time, conflict) + `error_handler` | `src/error.rs` |
| JS interop | Current date/week, clipboard (clipboard API + execCommand fallback) | `src/js.rs` |
| Base types | `ImStr` (`Rc<str>` newtype), `format_hours` helper | `src/base_types.rs` |

## Pattern Overview

**Overall:** Coroutine-driven Flux/Elm-lite. Each domain area has a single
`*_service` async function consumed via `use_coroutine` in `App()`. Pages
dispatch typed `*Action` enums; the coroutine mutates a `GlobalSignal` store;
Dioxus reactively re-renders any component that read it.

**Key Characteristics:**
- Single-threaded event loop (WASM single thread).
- No component-local "stores" — all cross-page state is global signals
  declared with `Signal::global(|| …)` inside `src/service/*.rs`.
- Pages own ephemeral, route-local UI state via `use_signal` and
  `use_resource`; they never write directly to `GlobalSignal`s of other
  domains — they go through the corresponding coroutine handle.
- DTOs (`rest_types::*TO`) never leak past the loader / service boundary into
  components. Components consume the local `state::*` mirror types.
- All shared text is `Rc<str>` (aliased as `ImStr` in `src/base_types.rs`)
  so prop equality is cheap and re-renders are minimised.

## Layers

**Page layer (`src/page/`):**
- Purpose: One file per route, owns the per-route layout and dispatches
  domain actions.
- Location: `src/page/`
- Contains: `#[component]` functions whose names are re-exported from
  `src/page/mod.rs` and into `src/router.rs`.
- Depends on: `component::*`, `service::*`, `state::*`, `loader`, `i18n`,
  `router::Route`, `service::auth::AUTH`, `service::config::CONFIG`,
  `service::i18n::I18N`.
- Used by: `src/router.rs` only.

**Component layer (`src/component/`):**
- Purpose: Reusable UI fragments with props.
- Location: `src/component/`, with sub-modules `atoms/` and `form/`.
- Contains: `#[component]` functions with `Props` structs. Atoms have no
  domain knowledge; higher-level components (e.g. `EmployeeView`,
  `WeekView`) read domain `GlobalSignal`s directly.
- Depends on: `state::*` (for prop types), `service::*` (for reactive
  reads + dispatching actions), atoms.
- Used by: pages and other components.

**Service layer (`src/service/`):**
- Purpose: Own a `GlobalSignal` per domain area + a coroutine that
  processes typed action messages.
- Location: `src/service/`
- Contains: One module per domain; each defines:
  - `pub static <NAME>_STORE: GlobalSignal<…>` (or `<NAME>` for singletons)
  - `pub enum <Name>Action { … }`
  - `pub async fn <name>_service(mut rx: UnboundedReceiver<<Name>Action>)`
  - private/public `async fn` helpers used by both the coroutine and
    other services.
- Depends on: `loader`, `api`, `state::*`, `super::config::CONFIG`,
  `super::error::{ErrorStore, ERROR_STORE}`, `super::i18n::I18N`,
  `super::auth::AUTH`.
- Used by: pages (`use_coroutine_handle::<Action>()` + read of `*_STORE`).

**Loader (`src/loader.rs`):**
- Purpose: Orchestrate one or more `api::*` calls, convert TOs into
  `state::*` types, and apply cross-cutting decoration (e.g. attach
  `SalesPerson` colour to a `Booking`, filter slots affected by special
  days, sort by date).
- Location: single file, ~865 lines.
- Depends on: `api`, `rest_types`, `state::*`, `error::ShiftyError`.
- Used by: services and a handful of pages directly (e.g.
  `loader::load_shiftplan_catalog` in `src/page/shiftplan.rs`).

**API client (`src/api.rs`):**
- Purpose: Thin async wrappers around `reqwest::get`/`Client::{post,put,delete}`
  returning `Rc<[T]>` / `T` of `rest_types::*TO`.
- Location: single file, ~1269 lines.
- Depends on: `rest_types`, `reqwest`, `web_sys::window` (only in
  `load_config` to read `window.location` for the bootstrap URL),
  `tracing`, `uuid`.
- Used by: `loader.rs` and a few `service::*` modules directly when no
  TO transformation is needed.

**State layer (`src/state/`):**
- Purpose: Domain types used by components and services. Plain `Clone +
  PartialEq` structs; no `dioxus::*` imports.
- Pattern: every TO has a `From<&FooTO> for Foo` impl.
- Notable types: `AuthInfo`, `Config`, `Shiftplan`, `Slot`, `Weekday`,
  `DayAggregate`, `Employee`, `WorkingHoursMini`, `Week`,
  `BookingLog`, `WeeklySummary`, `SalesPersonUnavailable`,
  `SlotEditItem`, `TextTemplate`, `User`, `ShiftplanAssignment`.

## Data Flow

### Primary Request Path — "user clicks Add booking"

1. `WeekView` cell click handler in `src/component/week_view.rs` invokes the
   `on_add_booking` callback prop wired from `src/page/shiftplan.rs`.
2. The page-level handler (the `cr` coroutine in
   `src/page/shiftplan.rs:267+`) sends
   `ShiftPlanAction::AddUserToSlot { … }` via the `UnboundedSender` it
   owns.
3. The page-local coroutine async-loop matches the variant and calls
   `loader::register_user_to_slot(...)` (`src/loader.rs:279`).
4. The loader calls `api::add_booking(...)` (`src/api.rs`) which issues a
   `POST {backend}/booking` via `reqwest`.
5. On success the page calls `shift_plan_context.restart()` (`use_resource`)
   which re-runs `loader::load_shift_plan(...)` (`src/loader.rs:151`),
   chaining into `api::get_shiftplan_week` (`src/api.rs`).
6. The new `Shiftplan` value flows into the resource signal. Components
   reading it (`WeekView`, `WorkingHoursMiniOverview`, …) re-render.
7. Side-effect refreshes also ride along: `WORKING_HOURS_MINI`,
   `BOOKING_CONFLICTS_STORE`, `WEEKLY_SUMMARY_STORE` are refreshed via
   their respective `use_coroutine_handle::<…Action>()` sends from inside
   the `update_shiftplan` closure.

### Bootstrap Flow

1. `main()` calls `dioxus_logger::init` and then `launch(app::App)`
   (`src/main.rs:25`).
2. `App()` registers all background coroutines (`src/app.rs:13-26`),
   most importantly `service::config::config_service` which immediately
   invokes `load_config()` (fetch `/assets/config.json`) and then
   `auth::load_auth_info()` (GET `/auth-info`).
3. While `CONFIG.backend` is empty the app renders a `"Loading
   application configuration."` placeholder (`src/app.rs:60`).
4. Once `CONFIG` is populated, `<Auth>` (`src/auth.rs`) gates the tree on
   `AUTH.read().auth_info` and `AUTH.read().loading_done`.
5. Authenticated → `Router::<Route> {}` mounts; unauthenticated →
   `TopBar` + `NotAuthenticated` page (`src/page/not_authenticated.rs`).

### i18n Flow

1. `service::i18n::i18n_service` (`src/service/i18n.rs:8`) reads
   `navigator.language` (first 2 chars), maps it via
   `i18n::Locale::from_str` (`src/i18n/mod.rs:20`) and writes the
   generated `I18nType` into the `I18N` global signal.
2. `i18n::generate(locale)` (`src/i18n/mod.rs:408`) builds an
   `I18n<Key, Locale>` and dispatches to `en::add_i18n_en`,
   `de::add_i18n_de`, or `cs::add_i18n_cs` to populate the locale map.
3. Components read `I18N.read().clone()` once per render, then call
   `i18n.t(Key::Foo)` (returns `Rc<str>`) or `i18n.t_m(key, map)` for
   `{name}` placeholder interpolation.
4. Date formatting goes through `i18n.format_date(&date)`, dispatched to
   `Locale::format_date` (`src/i18n/mod.rs:34`), which emits ISO
   (`[year]-[month]-[day]`), German (`[day].[month].[year]`) or Czech
   (`[day]. [month]. [year]`) order.
5. Fallback: missing key in current locale falls back to `Locale::En`,
   then to the `"??"` literal (`src/i18n/i18n.rs:21,47`). The
   `i18n` enum tests in `src/i18n/mod.rs:422+` enforce that key sets
   stay in sync across locales for selected key groups.

**State Management:**
- Global, observable, mutated only by the owning service: `GlobalSignal`
  in `src/service/*.rs` (`AUTH`, `CONFIG`, `I18N`, `THEME_MODE`,
  `EMPLOYEE_STORE`, `EMPLOYEES_LIST_REFRESH`, `BOOKING_LOG_STORE`,
  `BOOKING_CONFLICTS_STORE`, `WORKING_HOURS_MINI`,
  `WEEKLY_SUMMARY_STORE`, `BILLING_PERIOD_STORE`,
  `TEXT_TEMPLATE_STORE`, `SHIFTPLAN_REFRESH`, `DROPDOWN`, `TOOLTIP`,
  `ERROR_STORE`).
- Page-local: `use_signal` for ephemeral inputs (drafts, filters, mode
  toggles).
- Async-derived: `use_resource(move || loader::…)` for reactive
  fetches; restart with `resource.restart()` after mutations.
- Cross-component signalling: the `_REFRESH` `GlobalSignal<u64>` token
  pattern — bumped by services after a write, read inside other
  `use_resource` closures to force re-fetch (e.g.
  `EMPLOYEES_LIST_REFRESH`, `SHIFTPLAN_REFRESH`).

## Key Abstractions

**`I18n<Key, Locale>`:**
- Purpose: Generic, runtime-built translation map.
- Examples: `src/i18n/i18n.rs`, `src/i18n/{en,de,cs}.rs`.
- Pattern: HashMap<Locale, HashMap<Key, Rc<str>>> with fallback locale
  and `"??"` sentinel; `t_m(key, map)` does `{placeholder}` substitution.

**`ImStr`:**
- Purpose: `Rc<str>` newtype with `Display`, `IntoAttributeValue`, and
  `From<{String, &str, Rc<str>}>` impls. Cheap clone, used for every
  user-facing string passed across props.
- Examples: `src/base_types.rs:9`.

**`*Action` enums + `*_service` coroutines:**
- Purpose: Typed message bus per domain. Pages call
  `use_coroutine_handle::<<Name>Action>().send(<Name>Action::Variant{…})`
  and the coroutine in `App()` owns the corresponding `GlobalSignal`.
- Examples: `EmployeeAction` + `employee_service`
  (`src/service/employee.rs:74,191`); `BookingLogAction` +
  `booking_log_service` (`src/service/booking_log.rs:15,19`);
  `DropdownAction` + `dropdown_service` (`src/service/dropdown.rs`).

**Loader → state conversion:**
- Purpose: Centralise TO → state mapping plus side-joins so components
  never see `*TO`.
- Examples: `loader::load_bookings` (`src/loader.rs:76`) attaches
  `SalesPerson` colour and label; `loader::load_slots`
  (`src/loader.rs:104`) filters out slots blocked by `SpecialDayTO`.

**`use_resource` pattern:**
- Purpose: Declarative async data fetch tied to a render.
- Examples: `let sales_persons = use_resource(move ||
  loader::load_sales_persons(config.clone()))` in
  `src/page/shiftplan.rs:180`.

## Entry Points

**Browser bootstrap:**
- Location: `src/main.rs`
- Triggers: `dioxus::launch(app::App)` from the WASM trampoline emitted
  by `dx serve` / `dx build`.
- Responsibilities: Init `dioxus_logger` and hand off to `App()`.

**Application root:**
- Location: `src/app.rs`
- Triggers: First render after `launch`.
- Responsibilities: Register every background coroutine (config, auth via
  config, theme, dropdown, tooltip, i18n, working_hours_mini,
  user_management, booking_conflict, booking_log, weekly_summary,
  employee_work_details, employee, slot_edit, billing_period); set
  `document.title` from `CONFIG`; gate the tree on `<Auth>`; mount
  `Router::<Route>`.

**Routing:**
- Location: `src/router.rs`
- Triggers: URL match against the `#[derive(Routable)] enum Route`.
- Responsibilities: 16 routes covering home, shiftplan
  (`/shiftplan/`, `/shiftplan/:year/:week`), weekly overview,
  employees + details, my-shifts, billing periods, user/sales-person
  management, custom extra hours, text templates.

## Architectural Constraints

- **Single-threaded:** WASM main thread only. All `service::*` coroutines
  are cooperatively scheduled via Dioxus's runtime.
- **Global state:** Every cross-page mutable store is a `GlobalSignal`
  declared in `src/service/*.rs` (look for `Signal::global(...)`). They
  are written *only* by the owning service module.
- **No backend session token in code:** auth is cookie-based; the
  browser handles `Set-Cookie` from the backend, frontend just calls
  `GET /auth-info` to read identity.
- **Backend URL injection:** `assets/config.json` (loaded at runtime by
  `api::load_config`, `src/api.rs:34`) supplies `backend`,
  `application_title`, `is_prod`, `env_short_description`,
  `show_vacation`. The bundle does not hardcode the backend URL.
- **`Dioxus.toml` dev proxy:** during `dx serve`, the proxy entries in
  `Dioxus.toml:45+` forward each known REST path
  (`/booking`, `/sales-person`, `/auth-info`, `/extra-hours`, …) to
  `http://localhost:3000`, so `Config.backend` may be empty/relative
  in dev.
- **TO ↔ state isolation:** `rest_types::*TO` are imported only in
  `src/api.rs`, `src/loader.rs`, `src/state/*.rs` (for `From` impls),
  and a handful of services. Components and pages must use `state::*`
  types in props.
- **i18n key parity:** Every key listed in `src/i18n/mod.rs::Key` is
  expected to have a string in all three locales. Test groups in
  `src/i18n/mod.rs:422+` lock parity for the most recently added groups.

## Anti-Patterns

### Calling `api::*` directly from a component

**What happens:** Bypassing the loader and calling `crate::api::foo(...)`
from inside a `#[component]` skips the TO → state mapping and the
side-joins (sales-person colour merge, special-day filter, sort).

**Why it's wrong:** Components end up handling `*TO` types, which leaks
backend wire format into the view layer and bypasses the conventional
error path (`ShiftyError` + `ERROR_STORE`).

**Do this instead:** Use the loader (`src/loader.rs`) wrapper, or send
a `<Name>Action` to the owning service. See
`src/page/shiftplan.rs:140` (`loader::load_shiftplan_catalog`) for the
canonical pattern.

### Mutating a foreign-domain `GlobalSignal`

**What happens:** A page or component writes
`*OTHER_STORE.write() = …` for a store owned by another service.

**Why it's wrong:** Breaks the single-writer invariant of the service
layer; race-prone with the coroutine that also writes the same signal;
makes provenance of state changes untraceable.

**Do this instead:** Send the corresponding `*Action` via
`use_coroutine_handle::<*Action>()`. See
`src/service/employee.rs:191` for the action loop, and
`src/page/shiftplan.rs` for many `service.send(...)` call sites.

### Using `String` (or owned `String` props) for cross-component text

**What happens:** Passing `String` instead of `ImStr` / `Rc<str>` in
component props causes deep clones on every prop comparison and
re-render.

**Why it's wrong:** Defeats the cheap-clone invariant the codebase relies
on for performance (every translated label and every TO field is
already `Rc<str>`).

**Do this instead:** Use `ImStr` (`src/base_types.rs:9`) or `Rc<str>`
directly. The atoms (`src/component/atoms/btn.rs`) and form inputs
(`src/component/form/inputs.rs`) take `ImStr` everywhere.

### Reading `I18N` / `CONFIG` / `AUTH` mid-render via direct `.read()` chains

**What happens:** Re-`read()`ing the same global multiple times across
nested closures within a single component, or holding the read guard
across an `await`.

**Why it's wrong:** Duplicates work and may interleave with writes from
the service; can deadlock the signal.

**Do this instead:** Snapshot at the top of the component:
`let i18n = I18N.read().clone();`, `let config = CONFIG.read().clone();`,
`let auth_info = AUTH.read().auth_info.clone();`. Used everywhere — see
`src/page/shiftplan.rs:89-91` for the canonical opening lines.

## Error Handling

**Strategy:** All cross-layer errors use `error::ShiftyError`
(`src/error.rs:4`):
- `Reqwest(reqwest::Error)` — HTTP failures.
- `TimeComponentRange(time::error::ComponentRange)` — ISO date math.
- `Conflict(String)` — HTTP 409 optimistic-lock with a translated
  message.

**Patterns:**
- Services that catch errors write to `ERROR_STORE`
  (`src/service/error.rs:11`) instead of propagating up. Example:
  `service::employee::employee_service`'s closing `Err(err) => { … }`
  arm (`src/service/employee.rs:267`).
- Pages prefer `error::result_handler(res)` (`src/error.rs:35`) which
  unwraps to `Option<T>` while side-effecting an `eprintln!` and an
  auth-aware reload on `401` (`src/error.rs:22`).
- 409 from optimistic-lock paths is translated via
  `Key::ExtraHoursConflictNotice` and stored on `ERROR_STORE` so an
  error banner / overlay can pick it up (see `extra_hours_modal` and
  `service::employee::employee_service` `EmployeeAction::UpdateExtraHours`
  arm at `src/service/employee.rs:206`).

## Cross-Cutting Concerns

**Logging:** `tracing` macros in WASM are bridged by `dioxus-logger`
(`src/main.rs:27`). Convention: `info!` for happy-path lifecycle,
`tracing::warn!` for skipped records (e.g. slot without
`shiftplan_id`, `src/loader.rs:117`), `error!` for unexpected DOM /
JS-interop failures.

**Validation:** Lightweight, page-side (e.g. `Uuid::parse_str` on URL
parameters in `src/page/billing_period_details.rs:32`); heavier
validation lives in the backend.

**Authentication:** Read-only on the frontend. `AuthInfo` carries
`user`, `privileges: Rc<[Rc<str>]>`, `authenticated: bool`. Pages
gate features with `auth_info.has_privilege("hr"|"admin"|"sales"|
"shiftplanner"|"shiftplan.edit")` (`src/state/auth_info.rs:24`,
`src/component/top_bar.rs:32`). Logout is a backend redirect.

**Theming:** `service::theme` (`src/service/theme.rs`) persists the
selected mode in `localStorage` under `"shifty-theme"` and resolves
`System` against `prefers-color-scheme`. UI prefs share the same
`localStorage`-only pattern (`src/service/ui_prefs.rs`,
`shifty.ui.workingHoursLayout`).

**Clipboard / current-time JS interop:** centralised in `src/js.rs`
(`get_current_year`, `get_current_week`, `current_datetime`,
`copy_to_clipboard` with `execCommand` fallback).

---

*Architecture analysis: 2026-05-07*
