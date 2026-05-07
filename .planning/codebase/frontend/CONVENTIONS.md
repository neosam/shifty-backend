# Coding Conventions — shifty-dioxus (frontend)

**Analysis Date:** 2026-05-07

## Naming Patterns

**Files:**
- `snake_case.rs` for every source file. Examples: `employee_short.rs`, `extra_hours_modal.rs`, `working_hours_overview_layout_toggle.rs`.
- One feature per file. Big feature files are split into sibling submodules under a directory-`mod.rs` pattern: `component/atoms/`, `component/form/` each have a `mod.rs` re-exporting items via `pub use`.
- Page modules sit under `src/page/` with the same name as the route they serve (`shiftplan.rs`, `billing_periods.rs`, `my_shifts.rs`).

**Functions:**
- Free functions and methods: `snake_case` (Rust default). The `#![allow(non_snake_case)]` at the top of `src/main.rs` exists *only* so Dioxus components can use `PascalCase`; non-component helpers MUST stay `snake_case`.
- Pure helpers used by tests are extracted at module scope as `pub(crate)` or `pub`: `build_class`, `variant_classes`, `disabled_classes`, `row_class`, `pill_button_class`, `block_hours`, `format_hours`, `signed_hours_diff`. See `src/component/atoms/btn.rs` lines 31-61 and `src/page/my_shifts.rs` lines 30-56.

**Variables and Signals:**
- Local bindings: `snake_case`.
- Signals follow the same convention: `let mut filter_text = use_signal(|| String::new());` (see `src/page/billing_period_details.rs:53`).
- Global stores use SCREAMING_SNAKE_CASE: `AUTH`, `CONFIG`, `I18N`, `EMPLOYEE_STORE`, `BILLING_PERIOD_STORE`, `EMPLOYEES_LIST_REFRESH`, `ERROR_STORE`. See `src/service/employee.rs:45-72`.

**Types:**
- Structs and enums: `PascalCase`. Examples: `BtnVariant`, `DialogVariant`, `WorkingHoursCategory`, `EmployeeStore`, `BillingPeriodAction`.
- Component functions: `PascalCase` to match Dioxus's React-like convention: `Btn`, `Dialog`, `EmployeeShort`, `WorkingHoursMiniOverview`, `App`. Defined with `#[component]` attribute (`src/component/atoms/btn.rs:81`) or as plain `fn Name() -> Element` (`src/app.rs:12`).
- Props structs: always `<ComponentName>Props` with `#[derive(Props, Clone, PartialEq)]`. Example: `BtnProps` in `src/component/atoms/btn.rs:63`.
- Enum variants used as actions: `<Domain>Action` with verb-style variants: `BillingPeriodAction::LoadBillingPeriods`, `EmployeeAction::DeleteExtraHours(Uuid)`, `EmployeeAction::Refresh`. See `src/service/employee.rs:74-86`.
- Transport types from the backend always end in `TO` and live in the sibling `rest-types` crate. Frontend domain types drop the `TO` suffix (`SalesPersonTO` ↔ `SalesPerson` in `src/state/shiftplan.rs:120-153`).

## Code Style

**Formatting:**
- `cargo fmt` is the source of truth. No `rustfmt.toml` overrides in the repo, so default rustfmt rules apply (4-space indent, 100-col soft limit).

**Linting:**
- `cargo clippy` per `CLAUDE.md` "Code Quality" section. No project-level `clippy.toml`.
- Self-policing convention tests use `include_str!("self.rs")` to grep for forbidden Tailwind tokens. 18 such tests exist (`grep -rn "no_legacy_classes_in_source"`). They reject `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-green-`, `bg-red-`, `border-black`, `border-gray-` to keep the design-token migration enforced. See `src/page/employees.rs:18-43` for the canonical shape; replicate this pattern in any new page or redesigned component.

**Annotations:**
- `src/main.rs` carries `#![allow(non_snake_case)]` so PascalCase components compile without warnings. Do not add this allow elsewhere.

## Import Organization

Idiomatic Rust grouping is followed throughout. The convention observed in nearly every file (e.g., `src/service/billing_period.rs:1-15`, `src/component/extra_hours_modal.rs:13-33`):

1. `std` imports.
2. Third-party crate imports (`dioxus::prelude::*`, `futures_util::StreamExt`, `tracing::info`, `uuid::Uuid`, `time::*`, `rest_types::*`).
3. Crate-local imports via `use crate::{ ... }` with a multi-line nested grouping.
4. `use super::{ ... }` last (used inside service modules to pull from sibling modules: `super::config::CONFIG`, `super::error::ERROR_STORE`).

**Path aliases:**
- None at the Cargo level (`Cargo.toml` has no `[patch]` or alias).
- The i18n key enum is consistently aliased on import: `use crate::i18n::Key as K;` in pages (e.g., `src/page/home.rs:4`). Inside services and components the full path `crate::i18n::Key` or just `Key` is used.
- `format_hours` is occasionally re-aliased to disambiguate from a local `format_hours`: `use crate::base_types::{format_hours as format_hours_norm, ImStr};` in `src/page/my_shifts.rs:7`.

## Component Patterns

**Function components with `#[component]` attribute:**
```rust
#[derive(Props, Clone, PartialEq)]
pub struct BtnProps {
    pub children: Element,

    #[props(default = BtnVariant::Secondary)]
    pub variant: BtnVariant,

    #[props(default = false)]
    pub disabled: bool,

    #[props(!optional, default = None)]
    pub icon: Option<ImStr>,

    #[props(!optional, default = None)]
    pub on_click: Option<EventHandler<()>>,
}

#[component]
pub fn Btn(props: BtnProps) -> Element {
    let class = build_class(props.variant, props.disabled);
    // …
    rsx! { button { /* … */ } }
}
```
(`src/component/atoms/btn.rs:63-106`)

**Prop conventions:**
- `#[props(default = …)]` for primitive defaults.
- `#[props(!optional, default = None)]` for `Option<T>` so Dioxus does NOT treat them as auto-optional (the `!optional` opt-out makes the prop explicitly required-with-default in the generated builder). Used for `Option<ImStr>`, `Option<EventHandler<…>>`, `Option<Element>`, `Option<u8>`. Examples: `BtnProps::icon`/`on_click`, `FieldProps::hint`/`error`/`span`, `DialogProps::subtitle`/`footer`.
- Children are typed as `Element` and consumed via `{ props.children }` inside `rsx!`.
- Event handlers are `EventHandler<T>`; invoked with `handler.call(value)`.

**Class-string builders:**
- Components separate the class-string assembly into a `pub(crate) fn build_class(...)` (or `variant_classes`, `row_class`, `weekday_pill_class`, `pill_button_class`) that returns `String` or `&'static str`. This keeps the visual-token logic pure and unit-testable without spinning up a VirtualDom. Found in `btn.rs`, `nav_btn.rs`, `person_chip.rs`, `tuple_row.rs`, `employee_short.rs`, `working_hours_overview_layout_toggle.rs`, `contract_modal.rs`, etc.
- Class strings concatenate base classes + variant classes + state classes via `String::with_capacity(N)` and `push_str`, never via `format!("…")` and never via dynamic Tailwind names. The Tailwind config (`tailwind.config.js`) only emits classes it can statically detect in source files; dynamic names like `format!("bg-{}-soft", state)` are forbidden by comment at the top of the config.

**Conditional rendering:**
- Early-return `rsx! {}` for hidden state: `if !props.open { return rsx! {}; }` (`src/component/dialog.rs:139`).
- Inline `if let Some(...)` and `match` blocks inside `rsx!`. Example: `src/component/atoms/btn.rs:100-103` shows `if let Some(icon) = props.icon.as_ref() { span { ... } }`.

## Hooks and State

**Local component state uses `use_signal`:**
```rust
let mut filter_text = use_signal(|| String::new());
let mut show_paid = use_signal(|| true);
let mut selected_template_id = use_signal(|| None::<Uuid>);
```
(`src/page/billing_period_details.rs:53-58`)

**Async data loading uses `use_resource`:**
```rust
let employees = use_resource(move || {
    let _refresh_token = *EMPLOYEES_LIST_REFRESH.read();
    loader::load_employees(config.to_owned(), *year.read(), week_until)
});
```
(`src/component/employees_list.rs:52-59`) — note the deliberate read of a refresh-token signal to subscribe the resource to invalidation events.

**Effects use `use_effect` + `spawn`** for fire-and-forget async setup:
```rust
use_effect(move || {
    spawn(async move {
        handle_text_template_action(TextTemplateAction::LoadTemplatesByType(
            "billing-period".to_string(),
        )).await;
    });
});
```
(`src/page/billing_period_details.rs:64-71`)

**Long-lived cross-component state uses `GlobalSignal`** declared at module scope:
```rust
pub static AUTH: GlobalSignal<AuthStore> = Signal::global(|| AuthStore::default());
pub static EMPLOYEE_STORE: GlobalSignal<EmployeeStore> = Signal::global(|| /* … */);
```
27 such globals exist (services own one each). Read with `STORE.read()`, write with `*STORE.write() = …;` or `STORE.write().field = …;`.

**Background services use `use_coroutine` in `App`** (`src/app.rs:13-26`):
```rust
use_coroutine(service::config::config_service);
use_coroutine(service::theme::theme_service);
use_coroutine(service::i18n::i18n_service);
// … 14 coroutines registered in App, one per domain service.
```
Each service exports an async `<name>_service(rx: UnboundedReceiver<Action>)` that loops on `rx.next().await` and dispatches enum-variant actions. The body is `match action { Variant => async_handler(...).await, … }`, with errors funneled into `ERROR_STORE` (`src/service/billing_period.rs:71-95`).

**Custom hooks** are sparingly used. The only project-defined hook is `use_media_query` in `src/component/atoms/media_query.rs:24` (returns `Signal<bool>`, listens to `window.matchMedia` change events with a `Drop`-guarded WASM-side cleanup).

**Hook rule workaround:** When a component must early-return based on `props.open`, hooks are pushed into an inner always-mounted `*Content` component to satisfy the rules-of-hooks. See `Dialog` → `DialogContent` split in `src/component/dialog.rs:138-172` (comment at line 167 documents the rationale).

## Action / Service Pattern

Each domain service follows a consistent shape (canonical example: `src/service/billing_period.rs`):

1. Define a `Store` struct with `Default` and a `pub static <NAME>_STORE: GlobalSignal<Store>`.
2. Define an `enum <Domain>Action` of all mutations and reloads.
3. Provide free `async fn` per action, returning `Result<(), ShiftyError>`.
4. Provide an async `<domain>_service(rx: UnboundedReceiver<Action>)` coroutine that matches every action and pipes errors into `ERROR_STORE`.
5. Register the coroutine inside `App` via `use_coroutine(...)`.
6. Page/component callers grab a handle via `let svc = use_coroutine_handle::<Action>();` and `svc.send(Action::Foo)`.

## Error Handling

**No `anyhow`** anywhere in `src/` (zero hits for `grep -rn "anyhow"`). The crate uses **`thiserror`** exclusively.

**Single error enum** in `src/error.rs:1-16`:
```rust
#[derive(Error, Debug)]
pub enum ShiftyError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Time ComponentRange error: {0}")]
    TimeComponentRange(#[from] time::error::ComponentRange),
    /// HTTP 409 Conflict — typically optimistic-lock failure on a versioned PUT.
    #[error("{0}")]
    Conflict(String),
}
```

**Two helper sinks** (also in `src/error.rs`):
- `pub fn error_handler(e: ShiftyError)` — logs and, for `Reqwest` 401, force-reloads the window (`location().reload()`).
- `pub fn result_handler<T>(res: Result<T, ShiftyError>) -> Option<T>` — converts `Err` to `None` after logging.

**Service layer pattern:** every service handler returns `Result<(), ShiftyError>`. The coroutine match-arm pushes errors into `ERROR_STORE` rather than letting them propagate (`src/service/billing_period.rs:86-94`). 211 references to `ShiftyError`/`error_handler`/`result_handler` across `src/`.

**409 Conflict mapping:** `EmployeeAction::UpdateExtraHours` matches on `Err(ShiftyError::Conflict(_))`, refreshes the employee data, and writes a translated `Key::ExtraHoursConflictNotice` message into `ERROR_STORE` (`src/service/employee.rs:206-222`). Use this pattern for any new optimistic-locking PUT.

**Domain panics:** `WorkingHoursCategory::from_identifier` panics on unknown identifier (`src/state/employee.rs:78-91`); the conversion `&WorkingHoursCategory → ExtraHoursCategoryTO` panics on `Shiftplan`/`VacationDays` (`src/state/employee.rs:140-157`). These represent program-bug invariants, not user-facing errors.

## Logging

**Framework:** `tracing` with `dioxus-logger` adapter, initialized in `main.rs:27` at `Level::INFO`.

**Patterns:**
- `tracing::info!(...)` for service entry/exit and API calls. 21 hits across the crate. Examples: `src/api.rs:23,30,51`, `src/service/billing_period.rs:44,47,52`.
- `eprintln!` is used inside `error_handler` for terminal-only debugging (`src/error.rs:21,27,30`). Prefer `tracing::error!` for new code.

## i18n Usage

There is **no `t!()` macro** in this codebase. Translation lookups are method calls on a frontend-built `I18n<Key, Locale>` value.

**Reading translations in components:**
```rust
let i18n = I18N.read().clone();
let title_str = i18n.t(K::WelcomeTitle);
let choose_str = i18n.t(K::PleaseChoose);
// …
rsx! { h1 { class: "text-6xl font-bold", "{title_str}" } }
```
(`src/page/home.rs:17-32`)

**Three translation methods** on `I18n` (`src/i18n/i18n.rs`):
- `t(key) -> Rc<str>` — straight lookup with fallback to `Locale::En`, then to `"??"`.
- `t_m(key, HashMap<&str, &str>) -> Rc<str>` — replaces `{name}` placeholders. Used for parametric strings like `Key::ShiftplanFilledOfNeed` ("`{filled}/{need}`") and `Key::ShiftplanDeleteConfirmBody`.
- `t_m_rc(key, HashMap<&str, ImStr>) -> Rc<str>` — variant accepting `ImStr` values.

**Locale enum** has three variants: `Locale::En`, `Locale::De`, `Locale::Cs` (`src/i18n/mod.rs:14-18`). Every `Key` MUST receive a translation in all three locale modules (`src/i18n/en.rs`, `de.rs`, `cs.rs`). Skipping one yields the literal `"??"` fallback at runtime.

**Date and week formatting** uses locale-specific format descriptors via the `LocaleDef` trait (`src/i18n/mod.rs:30-52`):
- En: `[year]-[month]-[day]`
- De: `[day].[month].[year]`
- Cs: `[day]. [month]. [year]`

Call as `i18n.format_date(&date)` or `i18n.format_week(&week)`.

**Adding a new translation key:**
1. Add the variant to `pub enum Key` in `src/i18n/mod.rs` (alphabetical-ish, grouped by feature).
2. Add `i18n.add_text(Locale::En, Key::Foo, "…")` to `src/i18n/en.rs`, mirror in `de.rs` and `cs.rs`.
3. Add a presence test inside `src/i18n/mod.rs::tests` (e.g., `i18n_employees_keys_present_in_all_locales` lines 426-449 is the template).

**Locale selection:** `i18n_service` reads `navigator.language[..2]` once at startup and writes the resulting `I18n` into `I18N` (`src/service/i18n.rs:8-20`). No runtime locale-switch UI ships today.

## Styling Conventions (Tailwind)

**Design tokens, not raw colors.** Per `tailwind.config.js`, all colors map to CSS variables defined in `input.css` (`--bg`, `--surface`, `--ink`, `--ink-soft`, `--ink-muted`, `--accent`, `--accent-ink`, `--accent-soft`, `--good`, `--warn`, `--bad`, etc.). Components compose from token-named classes:

```text
bg-surface, bg-surface-alt, bg-accent, bg-accent-soft, bg-warn, bg-good
text-ink, text-ink-soft, text-ink-muted, text-accent-ink, text-bad
border-border, border-border-strong, border-accent, border-bad
```

**Forbidden classes** (enforced by 18 self-tests; see `src/page/employees.rs:18-43`):
- `bg-gray-*`, `bg-white`, `text-gray-*`, `text-blue-*`, `text-red-*`, `text-green-*`
- `bg-blue-*`, `bg-green-*`, `bg-red-*`
- `border-black`, `border-gray-*`

When adding a redesigned page or component, copy the `no_legacy_classes_in_source` test alongside it — this is the project's lint mechanism.

**Typography scale** (token-bound in `tailwind.config.js`):
- `text-micro` — 11px, weight 600, used for uppercase labels and tags.
- `text-small` — 12px, weight 500, used for descriptions and secondary metadata.
- `text-body` — 14px, weight 400, the default reading size.
- `text-lg` — 16px, weight 600, used for modal titles only (deliberately not 18px).

`tailwind.config.js` includes a comment at line 51 documenting that `lg = 16px` is intentional.

**Dynamic class names:** forbidden. `tailwind.config.js` line 1-8 instructs: "When constructing class strings dynamically … prefer static `if`/`match` branches that yield literal class strings." Components follow this — see `Btn::variant_classes` (`src/component/atoms/btn.rs:31-38`) which uses a `match` that returns four `&'static str` literals.

**Class organization in components:**
- Long static class strings are extracted to `const NAME: &str = "...";` at module top: `BASE_CLASSES`, `SHAPE_CLASSES`, `SHARED_INPUT_CLASSES`, `LABEL_CLASSES`, `ROW_BASE`, `PILL_CONTAINER`, `ACTIVE_BTN`, `INACTIVE_BTN`. Examples: `src/component/atoms/btn.rs:28`, `src/component/form/inputs.rs:11-12`, `src/component/form/field.rs:30`.
- Variant-dependent classes are picked by helper functions returning `&'static str` or `String`.
- Inline styles are used only for runtime-computed values (hex colors, pixel sizes coming from props): `style: "background-color: {color}"`, `style: "{span_style}"`. See `src/component/atoms/person_chip.rs:79-88`.

## TO Conversion Patterns

The frontend never operates on raw `*TO` types in render code. It always converts to a domain type defined in `src/state/`. Conversions are declared with **`impl From<&XTO> for X`** (frontend types take a borrowed TO):

```rust
impl From<&SalesPersonTO> for SalesPerson {
    fn from(sales_person: &SalesPersonTO) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.as_ref().into(),
            background_color: sales_person.background_color.as_ref().into(),
            is_paid: sales_person.is_paid.unwrap_or(false),
            inactive: sales_person.inactive,
            version: sales_person.version,
        }
    }
}
impl From<&SalesPerson> for SalesPersonTO {
    fn from(sales_person: &SalesPerson) -> Self { /* … */ }
}
```
(`src/state/shiftplan.rs:129-153`)

19 such `impl From<&...TO>` implementations exist. Conventions:
- `&str`/`Rc<str>` fields land via `.as_ref().into()` to land in `Rc<str>` storage.
- Optional booleans on the TO are unwrapped to a plain `bool` on the frontend (`is_paid.unwrap_or(false)`).
- Round-trip conversions provide both directions when a form posts updates back to the backend.
- Loading code in `src/loader.rs` wraps `api::*` calls and converts `Rc<[XTO]>` → `Rc<[X]>` via `iter().map(X::from).collect()`. Example: `src/loader.rs:28-36`.

**ImStr** (`src/base_types.rs:9-22`) is the project's `Rc<str>` newtype. Used for prop types where ergonomic `From<String>`/`From<&str>` is wanted alongside `IntoAttributeValue`. Prefer `ImStr` for any prop carrying display text, prefer `Rc<str>` for state fields.

**`format_hours(value: f32, decimals: usize) -> String`** (`src/base_types.rs:66-72`) is the canonical helper for rendering hour amounts. It normalises `-0.0` (and tiny negatives that round to zero) to `0.0`. Use it everywhere instead of raw `format!("{:.*}", decimals, value)` — there are explicit unit tests in `src/base_types.rs:74-122` locking the negative-zero behavior.

## Comments

**When to comment:**
- Module-level docstrings (`//!`) explain the role and design constraints of every redesigned atom: `src/component/atoms/btn.rs:1`, `dialog.rs:1-8`, `nav_btn.rs:1-7`, `person_chip.rs:1-22`, `tuple_row.rs:1-6`, `field.rs:1-7`, `inputs.rs:1-5`, `media_query.rs:1-12`.
- Inline comments justify non-obvious invariants: rules-of-hooks workarounds (`dialog.rs:167`), color-token invariants (`person_chip.rs:1-22`), refresh-token contracts (`employee.rs:37-44`).
- Every `pub(crate) fn build_class`/`variant_classes`/etc. has a one-line `///` doc.

**JSDoc/TSDoc:** N/A (Rust). Use `///` for items, `//!` for modules.

## Function Design

**Size:** Components stay small — most are <80 lines. Page modules are larger (`src/page/shiftplan.rs` is 1377 lines, `src/component/week_view.rs` is 1631) and *should* be split as redesign progresses (see CONCERNS notes in TESTING.md).

**Parameters:**
- Components take a single `props: <ComponentName>Props`.
- Pure helpers take borrowed inputs (`&Employee`, `&BlockTO`, `&str`) and return owned values (`String`, `Rc<str>`, `f32`).
- Service functions return `Result<T, ShiftyError>`.

**Return Values:**
- `rsx! { … }` blocks return `Element`.
- Class builders return `String` when state-dependent, `&'static str` when variant is the only axis.
- Loader functions return `Result<Rc<[T]>, ShiftyError>` for collections (always `Rc<[T]>`, never `Vec<T>`, so the resulting handles can be cheaply cloned across signal reads).

## Module Design

**Re-exports via `pub use`** in every directory `mod.rs`:
- `src/component/mod.rs:32-48` re-exports the 20 most-used component types so callers can write `use crate::component::Btn;` without traversing `atoms::btn::`.
- `src/component/atoms/mod.rs:28-32` and `src/component/form/mod.rs` do the same one level down.
- `src/page/mod.rs:17-32` re-exports every page.
- `src/state/mod.rs:16-24` re-exports the most-used domain types (`AuthInfo`, `Config`, `Shiftplan`, `Slot`, `Weekday`, `User`).

**Barrel files** (`mod.rs`) only declare submodules and re-export. They contain no logic.

**Service modules** are flat single files under `src/service/<domain>.rs`. Each exports its own `Store`, `Action` enum, free `async fn` handlers, and the `<domain>_service` coroutine entry point.

---

*Convention analysis: 2026-05-07*
