# Coding Conventions

**Analysis Date:** 2026-05-05

## Naming Patterns

**Files:**
- Component files use snake_case: `week_view.rs`, `booking_log_table.rs`, `slot_edit.rs`
- Service files use snake_case: `billing_period.rs`, `booking_log.rs`, `employee.rs`
- Page files use snake_case: `shiftplan.rs`, `employees.rs`, `home.rs`
- Test module files use snake_case with `_tests` suffix: `i18n_tests.rs`, `error_tests.rs`, `service_tests.rs`

**Functions:**
- Functions use snake_case: `load_billing_periods()`, `clear_selected_billing_period()`, `format_date()`
- Async service coroutines use `_service` suffix: `billing_period_service()`, `i18n_service()`, `working_hours_mini_service()`
- Component constructors (with `#[component]` macro) use PascalCase: `ShiftPlan`, `WeekView`, `PersonChip`, `Btn`
- Helper/builder functions use camelCase for internal helpers: `build_class()`, `variant_classes()`, `disabled_classes()`

**Variables:**
- Signal/mutable state uses snake_case: `mut week`, `mut monday`, `selected_template`
- Props structs use PascalCase: `ShiftPlanProps`, `ColumnViewSlotProps`, `BtnProps`
- Enum variants for actions use PascalCase: `BillingPeriodAction::LoadBillingPeriods`, `ShiftPlanAction::AddUserToSlot`

**Types & Structs:**
- All struct types use PascalCase: `BillingPeriodStore`, `ColumnViewItem`, `TextTemplate`, `EmployeeStore`
- Enum types use PascalCase: `BtnVariant`, `ColumnViewContent`, `ShiftPlanAction`, `Key` (i18n keys)
- Type aliases and newtype wrappers use PascalCase: `Rc<str>`, `ImStr` (immutable string wrapper)

**i18n Keys:**
- Enumerated translation keys in `src/i18n/mod.rs` use PascalCase with screaming_snake_case compound words: `Key::Save`, `Key::Cancel`, `Key::ShiftplanCalendarWeek`, `Key::BookingLogTitle`, `Key::ShiftplanDeleteConfirmTitle`
- Locale enum variants use PascalCase: `Locale::En`, `Locale::De`, `Locale::Cs`

## Code Style

**Formatting:**
- Rust code is implicitly formatted with `cargo fmt` (default Rustfmt configuration)
- No explicit `rustfmt.toml` file exists; uses Rust standard conventions
- 4-space indentation (Rust default)

**Linting:**
- Clippy can be run with `cargo clippy` but is not enforced in CI
- The GitHub workflow (`.github/workflows/rust.yml`) has formatting checks commented out, indicating linting is optional

**Import Organization:**
Order of imports in source files:
1. Standard library imports: `use std::rc::Rc;`
2. External crate imports: `use dioxus::prelude::*;`, `use serde::{...};`, `use futures_util::StreamExt;`
3. Tracing/logging: `use tracing::info;`
4. UUIDs and time: `use uuid::Uuid;`, `use time::Date;`
5. Crate internal imports: `use crate::api;`, `use crate::component::{...};`, `use crate::service::{...};`
6. Re-exports and conditional imports: `#[cfg(test)] mod tests;`

**Path Aliases:**
- No explicit path aliases via `paths` in `Cargo.toml`
- All imports use full crate paths with `crate::` prefix: `crate::service::i18n::I18N`, `crate::component::TopBar`

## Error Handling

**Patterns:**
- Error type uses `thiserror::Error` derive for custom enums in `src/error.rs`
- Main error type: `ShiftyError` enum with variants for `Reqwest`, `TimeComponentRange`, `Conflict`
- Functions return `Result<T, ShiftyError>` or `Result<T, reqwest::Error>` depending on context
- Async API functions (in `src/api.rs`) return `Result<T, reqwest::Error>`
- Service functions return `Result<(), ShiftyError>` when side-effect only
- Error handlers use pattern matching: `match e { ShiftyError::Reqwest(e) => {...}, ... }`
- Helper functions:
  - `error_handler(e: ShiftyError)` — logs errors and handles HTTP 401 (unauthorized) by reloading page
  - `result_handler<T>(res: Result<T, ShiftyError>) -> Option<T>` — converts errors to `None`, logs via `error_handler()`

**Error Propagation:**
- `?` operator used extensively in async service functions for early return
- Services wrap API errors in `ShiftyError::Reqwest`
- Conflict errors (HTTP 409) wrapped with user-facing translated message

## Logging

**Framework:** Dioxus-logger with Tracing

**Initialization:**
- Initialized in `src/main.rs`: `dioxus_logger::init(Level::INFO).expect("failed to init logger");`
- Log level set to `INFO` for production builds

**Patterns:**
- Logging points use `info!()` macro from `tracing` crate
- Service functions log action entry: `info!("BillingPeriodAction: {:?}", &action);`
- API calls logged at entry and completion: `info!("Fetching slots")` then `info!("Fetched")`
- Error logging via `error_handler()` which uses `eprintln!()`: `eprintln!("Error: {}", e);`
- No DEBUG or TRACE level logs observed; all user-facing logs use INFO

## Comments

**When to Comment:**
- Module-level documentation comments (`//!`) used in atom components (`src/component/atoms/mod.rs`)
- Documentation comments (`///`) used for public types and enums explaining purpose
- Inline comments rare; code generally self-documenting
- Example from `src/component/atoms/btn.rs`:
  ```rust
  //! `Btn` — the primary action button used across the redesigned pages.
  //!
  //! Variants map to design tokens defined in `input.css`:
  //! - [`Primary`](BtnVariant::Primary): accent on accent-ink (call-to-action)
  ```

**Doc Comments:**
- Used on enum variants to explain mapping to design tokens
- Used on `Props` structs to explain component behavior
- JSDoc-style comments for TypeScript-adjacent code (minimal in this Rust WASM project)

## Function Design

**Size:**
- Service coroutines range 20-30 lines typically
- Component functions range 50-200 lines depending on complexity
- Event handlers are small (1-5 lines)
- Helper functions like `build_class()`, `variant_classes()` are 3-10 lines

**Parameters:**
- Props structs always use the `#[derive(Props)]` macro with field-level `#[props(...)]` annotations
- Service functions accept state via `GlobalSignal` reads: `CONFIG.read().clone()`
- Coroutine handlers accept `UnboundedReceiver<ActionEnum>` as parameter
- Event handlers typically receive no arguments or a single typed argument
- Async functions use `async/await` syntax throughout

**Return Values:**
- API functions return `Result<T, reqwest::Error>` or `Result<T, ShiftyError>`
- Service functions return `Result<(), ShiftyError>` for side-effect operations
- Component rendering functions return `Element` (Dioxus type)
- Helper functions return concrete types: `String` for class building, `Rc<str>` for strings

## Module Design

**Exports:**
- Barrel files (`mod.rs`) re-export public items for ergonomic imports
- Example in `src/component/atoms/mod.rs`:
  ```rust
  pub use btn::{Btn, BtnVariant};
  pub use nav_btn::NavBtn;
  pub use person_chip::PersonChip;
  ```
- Services export `GlobalSignal` stores and action enums: `pub static I18N: GlobalSignal<I18nType>`, `pub enum BillingPeriodAction`

**Barrel Files:**
- Used extensively in `src/component/`, `src/service/`, `src/page/` to aggregate exports
- Allows consumers to write `use crate::component::Btn;` instead of `use crate::component::atoms::Btn;`
- Each barrel file documents what it contains

## Dioxus-Specific Patterns

**Component Definition:**
- All components use `#[component]` macro attribute
- Props structs always use `#[derive(Props, Clone, PartialEq)]`
- Component functions take `props: ComponentProps` parameter and return `Element`

**Signals and State:**
- `GlobalSignal<T>` used for app-wide shared state: `I18N`, `CONFIG`, `AUTH`, `BILLING_PERIOD_STORE`
- Component-local state uses `use_signal()`: `let mut week = use_signal(|| props.week.unwrap_or_default());`
- Signals are explicitly read/written via `.read()`, `.write()`, `.with()` methods

**Coroutines:**
- Service layer coroutines receive actions via `UnboundedReceiver<ActionEnum>`
- Started with `use_coroutine_handle::<ActionEnum>()` in components
- Pattern:
  ```rust
  let billing_period_service = use_coroutine_handle::<BillingPeriodAction>();
  // Later: billing_period_service.send(BillingPeriodAction::LoadBillingPeriods);
  ```

**RSX Patterns:**
- Standard Dioxus RSX syntax: `rsx! { div { class: "..." } }`
- Event handlers: `onclick: move |evt| { evt.prevent_default(); handler.call(()); }`
- Conditional rendering: `if condition { rsx! { ... } } else { rsx! { ... } }`
- Fragment rendering: multiple elements without wrapper

**Tailwind Integration:**
- Tailwind CSS applied via `class:` attribute in RSX
- Design token aliases used: `bg-surface`, `text-ink`, `border-border-strong`, `bg-accent`
- State-dependent classes built with `format!()`: `format!("class1 {}", if cond { "class2" } else { "class3" })`
- Custom zoom classes for scaling: `scale-down-50`, `scale-down-75`, `scale-down-100` (not `transform: scale()`)
- Responsive utilities: `md:grid-cols-2`, `print:bg-white`

## i18n Convention

**Pattern:**
- All user-facing text must have an entry in `src/i18n/mod.rs` under the `Key` enum
- Translations provided for all three locales: English, German, Czech
- Each locale has its own module: `src/i18n/en.rs`, `src/i18n/de.rs`, `src/i18n/cs.rs`
- Translation lookup: `i18n.t(Key::Save)` returns `Rc<str>`

**Critical Issue:**
- German locale previously had bug using `Locale::En` instead of `Locale::De` (now fixed)
- When adding new translations, ensure all three locale files are updated

## Type Safety & Generics

**Use of Generics:**
- Components use generic type parameters for reusability: `ColumnViewSlot<CustomData>` where `CustomData: Identifiable + PartialEq + Clone + 'static`
- Trait bounds commonly used: `Rc<T>`, `Option<T>`, `Result<T, E>`
- Collection types: `Rc<[T]>` for owned slices (cheaper cloning than `Vec`)

**Immutable String Wrapper:**
- Custom type `ImStr` used throughout for immutable string storage
- More efficient than repeated `String` allocations

---

*Convention analysis: 2026-05-05*
