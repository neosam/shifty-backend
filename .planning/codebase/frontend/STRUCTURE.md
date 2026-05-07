# Codebase Structure

**Analysis Date:** 2026-05-07

## Directory Layout

```
shifty-dioxus/
├── Cargo.toml              # Crate manifest (Dioxus 0.6.1, web+router)
├── Dioxus.toml             # dx serve config + per-endpoint backend proxy entries
├── index.html              # WASM bootstrap shell
├── input.css               # Tailwind source (design tokens, .person-pill, etc.)
├── tailwind.config.js      # Tailwind config (custom colors, scale-down classes)
├── run-tailwind.sh         # Convenience runner for `npx tailwindcss --watch`
├── default.nix / flake.nix # Nix dev shell + build
├── update_versions.sh      # Bumps shifty-dioxus + rest-types in lockstep
├── openapi.json            # Snapshot of backend OpenAPI (reference only)
├── assets/
│   ├── config.json         # Runtime config: backend, is_prod, env_short_description
│   ├── tailwind.css        # Compiled Tailwind output (generated)
│   ├── main.css            # Hand-written CSS (zoom helpers, etc.)
│   ├── shifty.webp         # Home-page hero image
│   ├── header.svg          # Layout asset
│   └── favicon.ico
├── public/                 # Static files served as-is
├── rest-types/             # Path-dep crate; mirror of backend `rest-types/`
│   ├── Cargo.toml
│   └── src/                # `*TO` DTOs with `Serialize`/`Deserialize` (no `ToSchema`)
├── src/
│   ├── main.rs             # Entry point (init logger, launch App)
│   ├── app.rs              # Root component, mounts coroutines + Auth gate + Router
│   ├── router.rs           # `enum Route` with #[derive(Routable)] mappings
│   ├── auth.rs             # `<Auth>` gate component (reads AUTH global signal)
│   ├── api.rs              # ~1269 lines — all reqwest API client functions
│   ├── loader.rs           # ~865 lines — TO → state mapping + side-joins
│   ├── error.rs            # `ShiftyError` enum + handlers
│   ├── base_types.rs       # `ImStr` (`Rc<str>` newtype) + `format_hours`
│   ├── js.rs               # JS interop: current date/week, clipboard
│   ├── i18n/
│   │   ├── mod.rs          # `Locale` enum, `Key` enum (~400 keys), date format
│   │   ├── i18n.rs         # Generic `I18n<Key, Locale>` map type
│   │   ├── en.rs           # English strings
│   │   ├── de.rs           # German strings
│   │   └── cs.rs           # Czech strings
│   ├── page/
│   │   ├── mod.rs          # Re-exports for router
│   │   ├── home.rs
│   │   ├── shiftplan.rs    # ~1377 lines — main shift planning page
│   │   ├── weekly_overview.rs
│   │   ├── employees.rs
│   │   ├── employee_details.rs
│   │   ├── my_employee_details.rs
│   │   ├── my_shifts.rs
│   │   ├── billing_periods.rs
│   │   ├── billing_period_details.rs
│   │   ├── user_management.rs
│   │   ├── user_details.rs
│   │   ├── sales_person_details.rs
│   │   ├── custom_extra_hours_management.rs
│   │   ├── text_template_management.rs
│   │   ├── not_authenticated.rs
│   │   └── blog.rs         # Unused (not referenced by router)
│   ├── component/
│   │   ├── mod.rs          # Re-exports for pages
│   │   ├── atoms/          # Smallest building blocks
│   │   │   ├── mod.rs
│   │   │   ├── btn.rs           # Btn + BtnVariant (Primary/Secondary/Ghost/Danger)
│   │   │   ├── nav_btn.rs       # 28×28 mono-glyph prev/next button
│   │   │   ├── person_chip.rs   # Pastel name pill (.person-pill)
│   │   │   ├── tuple_row.rs     # Label/value row for detail panels
│   │   │   └── media_query.rs   # `use_media_query` hook (used by Dialog Auto)
│   │   ├── form/           # Form atoms (design-tokenised)
│   │   │   ├── mod.rs
│   │   │   ├── checkbox.rs      # FormCheckbox
│   │   │   ├── field.rs         # Field (label + input wrapper)
│   │   │   └── inputs.rs        # TextInput, TextareaInput, SelectInput
│   │   ├── week_view.rs         # ~1631 lines — core shiftplan week grid (zoom CSS)
│   │   ├── top_bar.rs           # ~1166 lines — global nav with privilege gating
│   │   ├── employee_view.rs     # ~997 lines
│   │   ├── working_hours_mini_overview.rs
│   │   ├── working_hours_overview_layout_toggle.rs
│   │   ├── dialog.rs            # Modal: Center/Sheet/Bottom/Auto + ESC + scroll lock
│   │   ├── extra_hours_modal.rs
│   │   ├── contract_modal.rs
│   │   ├── employee_weekly_histogram.rs
│   │   ├── employee_short.rs
│   │   ├── employees_list.rs    # Sidebar list (subscribes to EMPLOYEES_LIST_REFRESH)
│   │   ├── employees_shell.rs
│   │   ├── employee_work_details_form.rs
│   │   ├── booking_log_table.rs
│   │   ├── day_aggregate_view.rs
│   │   ├── weekly_overview_chart.rs
│   │   ├── shiftplan_tab_bar.rs
│   │   ├── slot_edit.rs
│   │   ├── add_extra_hours_form.rs
│   │   ├── add_extra_hours_choice.rs
│   │   ├── add_extra_days_form.rs
│   │   ├── user_management_tab_bar.rs
│   │   ├── overlay.rs
│   │   ├── tooltip.rs
│   │   ├── dropdown_base.rs     # Singleton dropdown layer (reads DROPDOWN signal)
│   │   ├── error_view.rs
│   │   ├── footer.rs            # Frontend version + backend version probe
│   │   └── base_components.rs   # Header, Select/Option/SimpleSelect helpers
│   ├── service/
│   │   ├── mod.rs               # `pub mod` declarations only
│   │   ├── auth.rs              # AUTH (GlobalSignal) + load_auth_info
│   │   ├── config.rs            # CONFIG (GlobalSignal) + bootstraps load + auth chain
│   │   ├── i18n.rs              # I18N (GlobalSignal) + browser-language detect
│   │   ├── theme.rs             # THEME_MODE, RESOLVED_THEME, localStorage persist
│   │   ├── ui_prefs.rs          # `WorkingHoursLayout` (Cards|Table) localStorage
│   │   ├── error.rs             # ERROR_STORE (GlobalSignal<ErrorStore>)
│   │   ├── dropdown.rs          # DROPDOWN (singleton popup)
│   │   ├── tooltip.rs           # TOOLTIP (singleton)
│   │   ├── employee.rs          # EMPLOYEE_STORE + EMPLOYEES_LIST_REFRESH bump
│   │   ├── employee_work_details.rs
│   │   ├── booking_log.rs       # BOOKING_LOG_STORE
│   │   ├── booking_conflict.rs  # BOOKING_CONFLICTS_STORE
│   │   ├── billing_period.rs    # BILLING_PERIOD_STORE
│   │   ├── slot_edit.rs         # SHIFTPLAN_REFRESH (refresh-token signal)
│   │   ├── text_template.rs     # TEXT_TEMPLATE_STORE
│   │   ├── user_management.rs   # ~676 lines — largest service
│   │   ├── weekly_summary.rs    # WEEKLY_SUMMARY_STORE
│   │   └── working_hours_mini.rs # WORKING_HOURS_MINI
│   ├── state/
│   │   ├── mod.rs               # Re-exports of common types
│   │   ├── auth_info.rs         # AuthInfo + has_privilege
│   │   ├── config.rs            # Config (Serialize/Deserialize for config.json)
│   │   ├── shiftplan.rs         # Weekday, Booking, Slot, Shiftplan, DayAggregate, …
│   │   ├── employee.rs          # ~376 lines — Employee, ExtraHours, WorkingSchedule
│   │   ├── employee_work_details.rs
│   │   ├── weekly_overview.rs   # WeeklySummary
│   │   ├── booking_log.rs
│   │   ├── slot_edit.rs
│   │   ├── text_template.rs
│   │   ├── sales_person_available.rs
│   │   ├── user_management.rs   # User, ShiftplanAssignment
│   │   ├── week.rs              # Week struct (year + week, ISO conversions)
│   │   ├── dropdown.rs          # Dropdown + DropdownEntry
│   │   └── tooltip.rs
│   └── tests/                   # Crate-level integration tests
│       ├── mod.rs
│       ├── api_tests.rs
│       ├── error_tests.rs
│       ├── i18n_tests.rs
│       ├── integration_tests.rs
│       ├── service_tests.rs
│       ├── state_tests.rs
│       ├── utils_tests.rs
│       ├── volunteer_work_tests.rs
│       └── week_tests.rs
├── design_handoff_shifty/       # Design assets / handoff bundle (reference only)
├── shifty-design/               # Design system experiments
├── openspec/                    # OpenSpec change docs (not part of build)
└── .planning/                   # GSD planning artifacts (not part of build)
```

## Directory Purposes

**`src/page/`:**
- Purpose: One file per route. Composes layout from components, owns
  per-route ephemeral state (`use_signal`), kicks off resources
  (`use_resource`), dispatches actions to services.
- Contains: `#[component]` functions named after the page (PascalCase),
  per-page `*Action` enums where the page runs its own coroutine,
  per-page `*Props` structs.
- Key files: `shiftplan.rs` (1377 lines, the central feature),
  `user_management.rs` (840 lines), `my_shifts.rs` (669 lines),
  `weekly_overview.rs` (593 lines), `billing_period_details.rs`
  (520 lines).

**`src/component/`:**
- Purpose: Reusable UI fragments.
- Contains: `#[component]` functions, sub-trees `atoms/` and `form/`
  for the smallest building blocks. Components may read domain
  `GlobalSignal`s directly (`I18N`, `WEEKLY_SUMMARY_STORE`, …) and
  may call coroutines via `use_coroutine_handle`.
- Key files: `week_view.rs` (1631 lines, core shift grid),
  `top_bar.rs` (1166 lines, global nav with privilege gating),
  `employee_view.rs` (997 lines), `dialog.rs` (687 lines, modal
  primitive used by every dialog in the app).

**`src/component/atoms/`:**
- Purpose: Design-token-aware primitives.
- Contains: `Btn`, `NavBtn`, `PersonChip`, `TupleRow`, `use_media_query`.
- Naming hint from `mod.rs`: "Atoms — design tokens introduced in the
  `design-tokens` capability — none of them carry hardcoded colors or
  radii."

**`src/component/form/`:**
- Purpose: Canonical input atoms for redesigned dialogs.
- Contains: `TextInput`, `TextareaInput`, `SelectInput`, `FormCheckbox`,
  `Field`.

**`src/service/`:**
- Purpose: Coroutine-backed domain services. Each module owns a slice of
  global state and processes a typed `*Action` enum.
- Module shape: `pub static *_STORE: GlobalSignal<…>` +
  `pub enum *Action` + `pub async fn *_service(rx)`.
- Key files: `user_management.rs` (676 lines), `employee.rs`
  (361 lines), `theme.rs` (238 lines), `text_template.rs` (223 lines).

**`src/state/`:**
- Purpose: Plain domain types decoupled from Dioxus. `From<&FooTO>`
  conversions and small derived helpers (e.g.
  `WorkingHoursCategory::is_vacation`).
- Convention: no `dioxus::*` imports, no async; safe to test from a
  non-WASM target.

**`src/i18n/`:**
- Purpose: Translation map. The `Key` enum (~400 entries) is the
  single source of truth; each locale module
  (`en.rs`/`de.rs`/`cs.rs`) is expected to provide a string for every
  key.
- `i18n.rs` is the generic `I18n<Key, Locale>` storage type;
  `mod.rs` defines `Locale`, `Key`, `LocaleDef::format_date`, and the
  `generate(locale)` constructor.

**`src/tests/`:**
- Purpose: Crate-level integration tests, gated by
  `#[cfg(test)] mod tests;` in `src/main.rs:23`.
- Naming: `<area>_tests.rs`. Existing files cover api, error, i18n,
  service, state, utils, volunteer-work, week, plus an
  `integration_tests.rs`.

**`assets/`:**
- Purpose: Static files bundled by `dx`. `config.json` is loaded *at
  runtime* by `api::load_config` (`src/api.rs:34`) — its contents
  decide the backend URL.
- Generated: `tailwind.css` is regenerated by `npx tailwindcss …`.
- Committed: yes (config.json + main.css), but `tailwind.css` is
  often regenerated locally.

**`rest-types/` (path-dep):**
- Purpose: Frontend's local mirror of the backend's `rest-types` crate
  with the same `*TO` types so frontend and backend can share wire
  shape. Path-dep declared in `Cargo.toml:28`.
- Note: this is a separate copy living under
  `shifty-dioxus/rest-types/`, not the backend's `rest-types`. Keep
  in sync when backend DTOs change.

## Key File Locations

**Entry points:**
- `src/main.rs` — `fn main()` initialises logger and calls
  `launch(app::App)`.
- `src/app.rs` — Root component; mounts every background coroutine and
  the Auth gate.

**Configuration:**
- `Cargo.toml` — Crate config; `dioxus = { version = "0.6.1", features
  = ["web", "router"] }`.
- `Dioxus.toml` — `dx` build/serve config; per-path proxy entries
  point at `http://localhost:3000`.
- `tailwind.config.js` — Tailwind setup (custom colors `missingColor`,
  `blockedColor`, `scale-down-{50,75,100}` zoom utilities).
- `input.css` — Tailwind source + design-token CSS (`.person-pill`,
  custom variables).
- `assets/config.json` — Runtime app config (backend, env labels,
  show_vacation flag).

**Core logic:**
- `src/api.rs` — All HTTP requests.
- `src/loader.rs` — TO → state mapping.
- `src/router.rs` — Route enum.
- `src/service/config.rs` — Bootstraps the whole app via
  `config_service`.
- `src/service/auth.rs` — Auth lifecycle.
- `src/service/i18n.rs` — Locale detection.

**Testing:**
- `src/tests/` — Integration suite reachable via `cargo test`.
- Unit tests live inline in many modules (`#[cfg(test)] mod tests {…}`)
  — see `src/base_types.rs`, `src/loader.rs`, `src/i18n/mod.rs`,
  `src/service/employee.rs`, `src/service/ui_prefs.rs`.

## Naming Conventions

**Files:**
- All Rust files: `snake_case.rs`.
- One module per concept: `week_view.rs`, `add_extra_hours_form.rs`,
  `extra_hours_modal.rs`. Multi-word.

**Directories:**
- `snake_case`. Singular for layers (`page`, `component`, `service`,
  `state`) — note: it is `page/` (singular), not `pages/`. Same for
  `component/` not `components/`.

**Components:**
- `#[component] fn PascalCase(props: PascalCaseProps) -> Element`.
- Props struct: `<ComponentName>Props` colocated with the component
  function.
- Re-exported through the parent `mod.rs` (`src/component/mod.rs`,
  `src/page/mod.rs`).

**Services:**
- Module: `<area>.rs` (snake_case).
- Global signal: `<AREA>_STORE: GlobalSignal<T>` (uppercase). Singletons
  drop the `_STORE` suffix (`AUTH`, `CONFIG`, `I18N`, `THEME_MODE`,
  `RESOLVED_THEME`, `DROPDOWN`, `TOOLTIP`).
- Refresh-token signals: `<AREA>_REFRESH: GlobalSignal<u64>` (e.g.
  `EMPLOYEES_LIST_REFRESH`, `SHIFTPLAN_REFRESH`).
- Action enum: `<Area>Action` (PascalCase).
- Coroutine fn: `<area>_service(mut rx: UnboundedReceiver<<Area>Action>)`.

**State types:**
- PascalCase domain names that mirror but do not equal TO names: e.g.
  `BookingTO` ↔ `Booking`, `SalesPersonTO` ↔ `SalesPerson`,
  `ShortEmployeeReportTO` ↔ `Employee`.

**i18n keys:**
- PascalCase entries on `enum Key` grouped by feature with a comment
  banner. Example: `// Booking log`, `// Billing period management`.

## Where to Add New Code

**New page (new route):**
- Implementation: `src/page/<page_name>.rs` defining
  `pub fn PageName(...) -> Element`.
- Wire-up:
  1. `pub mod page_name;` and `pub use page_name::PageName;` in
     `src/page/mod.rs`.
  2. Add `pub use crate::page::PageName;` and a route variant in
     `src/router.rs`.
- Tests: integration coverage in `src/tests/integration_tests.rs` if
  navigation/render needs locking.

**New component:**
- Implementation: `src/component/<component_name>.rs`. If atom-level,
  put it under `src/component/atoms/` and re-export from
  `src/component/atoms/mod.rs`. If form input, under
  `src/component/form/` and re-export from `src/component/form/mod.rs`.
- Wire-up: `pub mod` in `src/component/mod.rs`, plus `pub use` if the
  component should be reachable via the parent path
  `crate::component::Foo` (most are).

**New service / global state slice:**
- Implementation: `src/service/<area>.rs` with
  `pub static <AREA>_STORE`, `pub enum <Area>Action`,
  `pub async fn <area>_service(...)`.
- Wire-up:
  1. Add `pub mod <area>;` to `src/service/mod.rs`.
  2. Add `use_coroutine(service::<area>::<area>_service);` to
     `src/app.rs` `App()` body.
- Then read `<AREA>_STORE.read().clone()` in components and dispatch
  via `use_coroutine_handle::<<Area>Action>().send(…)`.

**New state type:**
- Implementation: `src/state/<area>.rs` with `Clone + PartialEq +
  Debug` and a `From<&FooTO>` impl.
- Wire-up: `pub mod <area>;` in `src/state/mod.rs`. Re-export
  high-traffic types via `pub use` in the same `mod.rs`.

**New API endpoint binding:**
- Add `pub async fn <verb>_<resource>(config: Config, …) -> Result<T,
  reqwest::Error>` in `src/api.rs`.
- If the response needs cross-joining or sorting, wrap it in
  `src/loader.rs` returning a `Result<T, ShiftyError>` with
  `state::*` types.

**New translation key:**
- Add a variant to `enum Key` in `src/i18n/mod.rs` (under the matching
  feature comment block).
- Add `i18n.add_text(Locale::*, Key::Foo, "…");` in *all three* of
  `src/i18n/en.rs`, `src/i18n/de.rs`, `src/i18n/cs.rs` (English fallback
  is enforced; missing key prints `"??"`).
- Optionally add to a parity test group in `src/i18n/mod.rs:422+`.

**New backend endpoint proxy (dev mode):**
- Add `[[web.proxy]] backend = "http://localhost:3000/<path>"` to
  `Dioxus.toml`. Otherwise `dx serve` will not forward the path.

**Tests for shared logic:**
- Pure functions (no Dioxus runtime): inline `#[cfg(test)] mod tests`
  beside the implementation — see `src/base_types.rs:74` or
  `src/service/employee.rs:278` for the pattern.
- Cross-module / regression: `src/tests/<area>_tests.rs` with the
  module declared in `src/tests/mod.rs`.
- SSR-driven component tests: build a `VirtualDom` and call
  `rebuild_in_place` (see `src/service/employee.rs:308` for the
  pattern); use `dioxus-ssr` (dev-dep, `Cargo.toml:80`).

## Special Directories

**`assets/`:**
- Purpose: Static files bundled into `dist/`.
- Generated: `tailwind.css` (regenerated by `npx tailwindcss --watch`).
- Committed: yes — `config.json` and `main.css` ship with the
  repository; `tailwind.css` is checked in but typically rebuilt.

**`dist/`:**
- Purpose: `dx build`/`dx serve` output.
- Generated: yes.
- Committed: present in repo root listing (legacy artifacts), but it
  is a build directory — do not edit by hand.

**`rest-types/` (frontend copy):**
- Purpose: Frontend-side path-dep crate mirroring the backend's
  `rest-types`. Keep DTOs in sync when backend `rest-types/` changes.
- Generated: no (hand-maintained).
- Committed: yes.

**`openapi.json`:**
- Purpose: Reference snapshot of the backend OpenAPI document. Not
  consumed at build time; helpful when adding new `api::*` bindings.

**`design_handoff_shifty/`, `shifty-design/`:**
- Purpose: Design artifacts. Not compiled.
- Committed: yes (reference material).

**`openspec/`, `.planning/`:**
- Purpose: Spec and planning workflow artifacts (GSD).
- Committed: yes; not part of the build.

## Load-Bearing / Unusual Files

- **`src/api.rs` (1269 lines)** — the only file calling `reqwest`. Any
  new endpoint binding goes here. There is no shared `reqwest::Client`
  — most calls construct a new client per request.
- **`src/loader.rs` (865 lines)** — the only place that does TO → state
  mapping with side-joins (sales-person colour merge, special-day
  filter). Skipping it loses cross-cutting decoration.
- **`src/component/week_view.rs` (1631 lines)** — uses CSS `zoom`
  property (not `transform: scale`) for zoom — see
  `shifty-dioxus/CLAUDE.md` notes; horizontal scroll with sticky time
  column.
- **`src/i18n/mod.rs` (~553 lines incl. tests)** — the `Key` enum is
  the contract surface for translations; tests at
  `src/i18n/mod.rs:422+` lock parity and placeholder substitution.
- **`assets/config.json`** — runtime config; `backend: ""` triggers the
  `"Loading application configuration."` placeholder branch in
  `src/app.rs:60`. Per-environment values are deployed through this
  file, not via build flags.
- **`Dioxus.toml` proxy block (45+)** — without an entry, `dx serve`
  will not proxy that path; new backend resources must be added here
  for local dev.

---

*Structure analysis: 2026-05-07*
