# Phase 24: Paid-Limit konfigurierbar & rollenbasiert durchsetzen - Pattern Map

**Mapped:** 2026-06-27
**Files analyzed:** 16 (8 backend, 8 frontend)
**Analogs found:** 16 / 16 (all have a concrete in-repo analog)

> All line numbers below were re-verified against live code on 2026-06-27.
> **Drift correction:** CONTEXT.md cites the shiftplanner gate at `sales_person_shiftplan.rs:84-88` (canonical_refs) and `:77-102` (code_context). The actual `check_permission(SHIFTPLANNER_PRIVILEGE, …).is_ok()` lines are **`sales_person_shiftplan.rs:84-88`** inside `get_bookable_sales_persons` (`:77-104`). Both refs are correct; use `:84-88` for the exact gate excerpt.
> **DI-ordering finding (not in CONTEXT):** in `shifty_bin/src/main.rs`, `toggle_service` is currently constructed at **line 1011**, AFTER `shiftplan_edit_service` at **line 905**. To wire the new dependency the planner MUST move the `toggle_dao` + `toggle_service` construction (`:1010-1015`) ABOVE the `shiftplan_edit_service` block (`:905-920`). ToggleService is Basic-tier (only DAO + permission + transaction), so moving it up is safe and matches the "Basic before Business" rule.

---

## File Classification

### Backend

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `migrations/sqlite/<new>_seed-paid-limit-toggle.sql` | migration | batch (seed INSERT) | `migrations/sqlite/20260501000000_add-feature-flag-table.sql` | exact (seeding INSERT) |
| `service/src/lib.rs` (new `ServiceError::PaidLimitExceeded`) | model (error enum) | transform | existing variants `EntityConflicts` / `ValidationError` (`:84-88`) | exact |
| `rest/src/lib.rs` (HTTP mapping) | middleware (error map) | request-response | `EntityConflicts`→409 (`:177-182`), `ValidationError`→422 (`:183-188`) | exact |
| `service_impl/src/shiftplan_edit.rs` (gen_service_impl + DI) | service | request-response | existing `gen_service_impl!` deps (`:25-41`) | exact |
| `service_impl/src/shiftplan_edit.rs` (pre-persist block) | service | event-driven (guard) | existing post-persist soft-warn count (`:529-548`) | exact (reorder) |
| `service_impl/src/shiftplan_edit.rs:407-417` (gate fix) | service | request-response | `sales_person_shiftplan.rs:84-88` shiftplanner gate | exact |
| `shifty_bin/src/main.rs` (DI wiring + reorder) | config (DI) | n/a | toggle_service block (`:1010-1015`), shiftplan_edit block (`:905-920`) | exact |
| `service_impl/src/test/shiftplan_edit.rs` (adjust HR tests) | test | n/a | existing forbidden/warning tests (`:473-485`, `:598-672`) | exact |

### Frontend (`shifty-dioxus/`)

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/page/settings.rs` (new `SettingsPage`) | page/component | request-response | `src/page/user_management.rs` `UserManagementPage` | role-match |
| `src/page/mod.rs` (export) | config (barrel) | n/a | existing `pub mod` + `pub use` block (`:1-34`) | exact |
| `src/router.rs` (`Route::Settings`) | route | n/a | existing route enum + `pub use` aliases (`:23-59`) | exact |
| `src/component/top_bar.rs` (nav entry) | component | n/a | `NavTarget::UserManagement` wiring (`:68`, `:54`, `:88-93`, `:120-128`, `:422-428`) | exact |
| `src/api.rs` + `src/loader.rs` (toggle client) | service (API client) | request-response | `api::update_shiftplan` PUT (`api.rs:119-131`) | exact |
| `src/page/shiftplan.rs:403-441` (inline block error) | page (handler) | request-response | `slot_edit.rs:264-266` error `<p>`; 403-silent path (`:431-436`) | role-match |
| `src/page/shiftplan.rs` (overage section) | page (section) | transform (client-side) | conflict section (`:869-903`); `WarningList` (`warning_list.rs:90-168`) | exact |
| `src/i18n/{mod,en,de,cs}.rs` (new keys) | config (i18n) | n/a | existing `Key` enum + locale `add_text` + `i18n_*_present_in_all_locales` test | exact |

---

## Pattern Assignments

### `migrations/sqlite/<new>_seed-paid-limit-toggle.sql` (migration, seed INSERT)

**Analog:** `migrations/sqlite/20260501000000_add-feature-flag-table.sql` (whole file)
**Schema being seeded into:** `toggle` table from `migrations/sqlite/20260105000000_app-toggles.sql:2-8`

The `toggle` table already exists; D-24-07 only needs an `INSERT` (no `CREATE TABLE`). Mirror the feature_flag seed INSERT shape, but target `toggle` and **omit any group** (D-24-01/D-24-06: no toggle group).

feature_flag seed pattern (analog, lines 14-20):
```sql
INSERT INTO feature_flag (key, enabled, description, update_process)
VALUES (
    'absence_range_source_active',
    0,
    'When ON, range-based AbsencePeriods are the source of truth ...',
    'phase-2-migration'
);
```

Target `toggle` schema (columns to fill — `app-toggles.sql:2-8`):
```sql
CREATE TABLE toggle (
    name TEXT NOT NULL PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,  -- 0 = disabled, 1 = enabled
    description TEXT,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);
```

New migration body (column name is `name`, NOT `key`; `enabled = 0` = soft = default, D-24-01a):
```sql
INSERT INTO toggle (name, enabled, description, update_process)
VALUES (
    'paid_limit_hard_enforcement',
    0,  -- 0 = soft (warnings only), default; 1 = hard (block non-shiftplanners)
    'When ON, booking over a slot/week paid-employee limit is blocked for non-shiftplanners. Default OFF (soft, warning-only).',
    'phase-24-migration'
);
```
**Note:** no `privilege` INSERT needed — `toggle_admin` already exists (`app-toggles.sql:30`). No toggle group.

---

### `service/src/lib.rs` — new `ServiceError::PaidLimitExceeded { current, max }` (error enum)

**Analog:** existing `ServiceError` variants with `thiserror` `#[error(...)]` (`service/src/lib.rs:84-88`)

```rust
#[error("Entity {0} conflicts, expected version {1} but got {2}")]
EntityConflicts(Uuid, Uuid, Uuid),

#[error("Validation error: {0:?}")]
ValidationError(Arc<[ValidationFailureItem]>),
```

Add a new variant in the same enum (insert before `InternalError` at `:126`). Struct-variant form (matches D-24-05 `{ current, max }` shape):
```rust
#[error("Paid employee limit exceeded: {current} > {max}")]
PaidLimitExceeded { current: u8, max: u8 },
```
**Type note:** `count_paid_bookings_in_slot_week` returns `u8` and `slot.max_paid_employees` is `Option<u8>` (`shiftplan_edit.rs:529`, `:646-652`) — so both fields are `u8`.

---

### `rest/src/lib.rs` — HTTP mapping for `PaidLimitExceeded` (middleware/error map)

**Analog:** the central `match` in `error_handler` — `EntityConflicts`→409 (`:177-182`) and `ValidationError`→422 (`:183-188`):

```rust
Err(RestError::ServiceError(err @ service::ServiceError::EntityConflicts(_, _, _))) => {
    Response::builder()
        .status(409)
        .body(Body::new(err.to_string()))
        .unwrap()
}
Err(RestError::ServiceError(err @ service::ServiceError::ValidationError(_))) => {
    Response::builder()
        .status(422)
        .body(Body::new(err.to_string()))
        .unwrap()
}
```

**Discretion (D-24-05 / Claude's Discretion):** choose a status that is NOT 403 (the frontend silently ignores 403, `shiftplan.rs:431-436`). Recommended **409 Conflict** (the booking conflicts with the configured capacity limit; aligns with `EntityConflicts`→409). Add the new arm next to these, before the catch-all. The frontend must match this exact status (see `shiftplan.rs` handler below).

**Reminder:** the REST handler `book_slot_with_conflict_check` (`rest/src/shiftplan_edit.rs:122-133`) carries `#[utoipa::path]` with documented response codes `201/403/422` — add the new status (e.g. `409`) to the `responses(...)` annotation for OpenAPI accuracy (CLAUDE.md OpenAPI rule).

---

### `service_impl/src/shiftplan_edit.rs` — add `ToggleService` dependency (service, DI)

**Analog:** the existing `gen_service_impl!` block (`shiftplan_edit.rs:25-41`):

```rust
gen_service_impl! {
    struct ShiftplanEditServiceImpl: ShiftplanEditService = ShiftplanEditServiceDeps {
        PermissionService: service::PermissionService<Context = Self::Context> = permission_service,
        SlotService: service::slot::SlotService<Transaction = Self::Transaction> = slot_service,
        BookingService: service::booking::BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        ...
        AbsenceService: service::absence::AbsenceService<Context = Self::Context, Transaction = Self::Transaction> = absence_service
    }
}
```

Add a `ToggleService` line (mirror the trailing `AbsenceService` entry — `ToggleService` uses `Context` + `Transaction`):
```rust
ToggleService: service::toggle::ToggleService<Context = Self::Context, Transaction = Self::Transaction> = toggle_service
```
Also add `use service::toggle::ToggleService;` to the `service::{…}` import block (`:5-19`). `ShiftplanEditService` is Business-Logic tier; `ToggleService` is Basic — consuming it is allowed (CLAUDE.md Service-Tier).

**Three wiring sites to update consistently:**
1. `service_impl/src/shiftplan_edit.rs:25-41` — `gen_service_impl!` (above).
2. `shifty_bin/src/main.rs:388-405` — add `type ToggleService = ToggleService;` to `ShiftplanEditServiceDependencies`.
3. `shifty_bin/src/main.rs:905-920` — add `toggle_service: toggle_service.clone(),` to the `ShiftplanEditServiceImpl { … }` literal **AND move the `toggle_dao`/`toggle_service` construction (`:1010-1015`) above this block** (see DI-ordering finding at top).

---

### `service_impl/src/shiftplan_edit.rs` — pre-persist hard-block (service, guard)

**Analog (reorder, not rewrite):** the existing **post-persist** soft-warning count (`:529-548`):

```rust
if let Some(max) = slot.max_paid_employees {
    let current_paid_count = self
        .count_paid_bookings_in_slot_week(
            booking.slot_id,
            booking.year,
            booking.calendar_week as u8,
            tx.clone(),
        )
        .await?;
    if current_paid_count > max {
        warnings.push(Warning::PaidEmployeeLimitExceeded { /* … */ });
    }
}
```

D-24-08 requires the count/check to run **before** `booking_service.create(...)` (`:471-474`). The pre-persist guard reads the toggle fresh per booking (`is_enabled` is auth-only), reuses `count_paid_bookings_in_slot_week` (`:646+`), and returns the new error BEFORE persisting:

```rust
// after slot lookup (:419-423), BEFORE booking_service.create (:471-474)
if let Some(max) = slot.max_paid_employees {
    let hard = self
        .toggle_service
        .is_enabled("paid_limit_hard_enforcement", Authentication::Full, tx.clone().into())
        .await?;
    if hard {
        let is_shiftplanner = self
            .permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
            .await
            .is_ok();
        if !is_shiftplanner {
            // count would-be paid bookings AFTER this booking → strict-greater (D-24-Grenzregel).
            // NOTE: count_paid_bookings_in_slot_week counts EXISTING bookings; the new booking is
            // not yet persisted. Block when (existing_paid + would-this-be-paid) > max.
            let current_paid_count = self
                .count_paid_bookings_in_slot_week(booking.slot_id, booking.year, booking.calendar_week as u8, tx.clone())
                .await?;
            // determine if the person being booked is paid (mirror count helper's is_paid source)
            // if prospective_count > max → block:
            return Err(ServiceError::PaidLimitExceeded { current: prospective_count, max });
        }
    }
}
```
**Behaviour notes from CONTEXT:**
- Strict-greater, deckungsgleich with the existing warning (`current > max`), only paid persons count (`is_paid`), unpaid never blocked (D-24-Grenzregel). Determine the booked person's `is_paid` the same way the count helper sources it (`SalesPersonService::get_all_paid`, see helper doc `:633-636`).
- No retroactive removal of existing bookings (D-07 / D-24).
- The existing soft-warning block (`:529-548`) stays for the soft path; only the **hard** path returns early before persist.
- `is_enabled` here uses `Authentication::Full` (consistent with the other inner cross-service lookups in this method, `:422`, `:441`, `:453`).

---

### `service_impl/src/shiftplan_edit.rs:407-417` — gate fix `HR ∨ self` → `Shiftplanner ∨ self` (D-24-04)

**Current code (to change):**
```rust
// Permission HR ∨ self (Pattern S2 / D-Phase3-12).
let (hr, sp) = join!(
    self.permission_service
        .check_permission(HR_PRIVILEGE, context.clone()),
    self.sales_person_service.verify_user_is_sales_person(
        booking.sales_person_id,
        context.clone(),
        tx.clone().into(),
    ),
);
hr.or(sp)?;
```

**Analog for the correct privilege:** `sales_person_shiftplan.rs:84-88` (`get_bookable_sales_persons`, already shiftplanner-gated):
```rust
let is_shiftplanner = self
    .permission_service
    .check_permission(SHIFTPLANNER_PRIVILEGE, context)
    .await
    .is_ok();
```

Swap `HR_PRIVILEGE` for `SHIFTPLANNER_PRIVILEGE` (keep the `join!` + `.or(sp)?` self-bypass shape). Update the import in the `service::{ permission::{Authentication, HR_PRIVILEGE} }` block (`shiftplan_edit.rs:11`) to bring in `SHIFTPLANNER_PRIVILEGE` (constant defined `service/src/permission.rs:11`). Admin still works via `admin-auto-grant` trigger (`migrations/sqlite/20260508120000_admin-auto-grant-privilege.sql`). Note the comment at `:407` ("Permission HR ∨ self") must be updated too.

---

### `service_impl/src/test/shiftplan_edit.rs` — adjust HR-based booking tests (D-24-04 blast radius)

**Analog (test structure already in-file):** existing tests grant permissions and call `book_slot_with_conflict_check`. Reuse the same harness:
- `test_book_slot_with_conflict_check_forbidden` (`:473-485`) — the forbidden-path test; verify it still fails for a non-shiftplanner non-self actor.
- Paid-limit warning tests `test_book_paid_into_full_slot_emits_warning` (`:598-672`), `test_book_paid_at_limit_no_warning` (`:708-768`) — these set up the paid-count scenario you can reuse to author a **new hard-block test** (toggle ON, non-shiftplanner, over limit → `Err(ServiceError::PaidLimitExceeded { .. })`; toggle ON + shiftplanner → Ok; toggle OFF → warning as today).

Any test that previously relied on `HR_PRIVILEGE` to authorize a booking must switch to granting `SHIFTPLANNER_PRIVILEGE` (or rely on self). The mock `ToggleService` (`#[automock]` on the trait, `service/src/toggle.rs:62`) must be added to the test deps and its `is_enabled` expectation set per scenario. (User rule: always add tests for changes.)

---

### `shifty-dioxus/src/page/settings.rs` — new `SettingsPage` (page/component)

**Analog:** `shifty-dioxus/src/page/user_management.rs` `UserManagementPage` (`:38-45` and import block `:10-23`).

Import + component skeleton pattern (mirror UserManagementPage):
```rust
use crate::{
    component::TopBar,
    i18n::Key,
    service::i18n::I18N,
};
use dioxus::prelude::*;

#[component]
pub fn SettingsPage() -> Element {
    let i18n = I18N.read().clone();
    // local signals for toggle state + flash/error, use_resource to load current state
    ...
}
```

Layout per UI-SPEC §Surface 1:
- Page heading: `class: "text-h2 font-semibold pb-4"`, text from `Key::Settings`.
- Card: `class: "bg-surface border border-border rounded-md p-4 flex flex-col gap-3"`.
- Toggle button states (UI-SPEC class strings) — Off: `"px-3 py-2 rounded-md border border-border text-ink text-body bg-surface hover:bg-surface-alt"`; On: `"px-3 py-2 rounded-md border border-bad text-bad text-body font-semibold bg-bad-soft"`; `aria-pressed`.
- Inline success flash: `"text-small text-ink-muted"`; inline error: `"text-bad text-small font-normal"`.

**Toggle data source:** new api/loader client (below). On click → `PUT /toggle/paid_limit_hard_enforcement/enable|disable`. Initial state via `GET /toggle/paid_limit_hard_enforcement/enabled`.

---

### `shifty-dioxus/src/api.rs` + `src/loader.rs` — toggle REST client (service)

**Analog:** `api::update_shiftplan` — a PUT with `reqwest::Client` and status check (`api.rs:119-131`):
```rust
pub async fn update_shiftplan(...) -> Result<..., reqwest::Error> {
    let url = format!("{}/shiftplan-catalog/{}", config.backend, shiftplan.id);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&shiftplan).send().await?;
    ...
}
```

New api fns (no body for enable/disable — the toggle REST routes are `PUT /toggle/{name}/enable` and `/disable`, backend `rest/src/toggle.rs:24-25`):
```rust
pub async fn set_toggle(config: Config, name: &str, enabled: bool) -> Result<(), reqwest::Error> {
    let verb = if enabled { "enable" } else { "disable" };
    let url = format!("{}/toggle/{}/{}", config.backend, name, verb);
    let client = reqwest::Client::new();
    client.put(url).send().await?.error_for_status()?;
    Ok(())
}
pub async fn get_toggle_enabled(config: Config, name: &str) -> Result<bool, reqwest::Error> {
    let url = format!("{}/toggle/{}/enabled", config.backend, name); // GET, rest/src/toggle.rs:23
    Ok(reqwest::get(url).await?.json().await?)
}
```
Wrap with thin `loader::` fns returning `Result<_, ShiftyError>` (mirror `loader.rs:286-296` `register_user_to_slot` delegation shape).

---

### `shifty-dioxus/src/router.rs` — `Route::Settings {}` (route)

**Analog:** existing route enum + `pub use` aliases (`router.rs:23-59`). Add alias (the `#[derive(Routable)]` needs an in-scope item named `Settings`):
```rust
pub use crate::page::SettingsPage as Settings;
```
And the route arm (mirror `UserManagementPage` at `:44-45`):
```rust
#[route("/settings/")]
Settings {},
```

---

### `shifty-dioxus/src/page/mod.rs` — export `SettingsPage` (barrel)

**Analog:** existing `pub mod` + `pub use` block (`page/mod.rs:1-34`). Add:
```rust
pub mod settings;
...
pub use settings::SettingsPage;
```

---

### `shifty-dioxus/src/component/top_bar.rs` — admin-gated Settings nav (component)

**Analog:** the full `NavTarget::UserManagement` wiring chain. Touch these five sites in lockstep (same edits UserManagement has):

1. **`NavVisibility` struct** (`:21-31`) — add field `pub settings: bool,`.
2. **`nav_visibility()`** (`:41-56`) — add `settings: has("admin"),` (mirror `user_management: has("admin")` at `:54`).
3. **`NavTarget` enum** (`:60-70`) — add variant `Settings,`.
4. **`is_active_for()`** (`:72-95`) — add arm `NavTarget::Settings => matches!(route, Route::Settings {}),`.
5. **`is_admin_target()`** (`:120-128`) — add `NavTarget::Settings` to the matches! set (puts it in the admin dropdown per D-24-06).
6. **`TopBarRouted` nav-items builder** (`:422-428`, mirror the `if visibility.user_management { items.push((NavTarget::UserManagement, Route::UserManagementPage {}, i18n.t(Key::UserManagement)…)) }`):
```rust
if visibility.settings {
    items.push((
        NavTarget::Settings,
        Route::Settings {},
        i18n.t(Key::Settings).to_string(),
    ));
}
```
**Reuse `Key::Settings`** (exists in all 3 locales: `mod.rs:319`, `de.rs:506` "Einstellungen", `cs.rs:476` "Nastavení", En "Settings"). The test helpers at the bottom of `top_bar.rs` (`nav_items_for_visibility` `:1123+`, `nav_visibility_admin_shows_user_management_and_templates` `:776`) will need a parallel assertion for the new `settings` field.

---

### `shifty-dioxus/src/page/shiftplan.rs:403-441` — inline hard-block error (D-24-05)

**Analog (status detection):** the existing `AddUserToSlot` match already special-cases a status (403-silent, `:431-436`):
```rust
Err(crate::error::ShiftyError::Reqwest(ref e))
    if e.status() == Some(reqwest::StatusCode::FORBIDDEN) =>
{
    // D-13: silently ignore 403, still reload
    update_shiftplan();
}
```

Add a NEW arm BEFORE the generic `Err(e)` that matches the chosen block status (e.g. `CONFLICT` if 409 chosen in `rest/src/lib.rs`), sets a slot-scoped signal instead of silently ignoring:
```rust
Err(crate::error::ShiftyError::Reqwest(ref e))
    if e.status() == Some(reqwest::StatusCode::CONFLICT) =>
{
    // D-24-05: surface inline block message, do NOT silently ignore
    block_error.set(Some(slot_id)); // slot-scoped signal
    update_shiftplan();
}
```

**Analog (render):** the slot-edit error `<p>` (`slot_edit.rs:264-266`):
```rust
if props.has_errors {
    p { class: "text-bad text-small font-normal", "{error_str}" }
}
```
Render a non-dismissible inline div at the slot using `class: "text-bad text-small font-normal mt-1"` (UI-SPEC) with text from the new `Key::BookingBlockedPaidLimit`. Clear on next successful booking action / navigation (UI-SPEC §Surface 2). This is a separate element from the existing dismissible `booking_warnings` banner (`:906-934`).

---

### `shifty-dioxus/src/page/shiftplan.rs` — persistent overage warn section (D-24-03)

**Analog (recommended — custom section):** the conflict section (`:869-903`):
```rust
if is_shiftplanner && !booking_conflicts.is_empty() {
    div { class: "mx-4 my-3 px-4 py-3 bg-bad-soft border border-bad rounded-md print:hidden",
        h2 { class: "text-h2 font-semibold pb-2 text-bad", "⚠️ {conflict_booking_entries_header}" }
        ul { class: "list-disc list-inside text-body text-ink",
            for ... { li { ... } }
        }
    }
}
```

Mirror it with **warn** tokens and **no role gate** (all roles, D-24-03 / D-23-05), inserted between the `booking_warnings` banner (`:906-934`) and the `ShiftplanTabBar` div (`:942`). UI-SPEC class strings:
- container: `"mx-4 my-3 px-4 py-3 bg-warn-soft border border-warn rounded-md print:hidden"`
- heading: `"text-h2 font-semibold pb-2 text-warn"`, text `"⚠️ {ShiftplanPaidOverageSectionHeader}"`
- list: `"list-disc list-inside text-body text-ink"`, one `li` per overage slot, text from `Key::ShiftplanPaidOverageRow` ("{slot}: {current}/{max} …").

**Data source (client-side, no backend round-trip):** iterate `state::shiftplan::Slot` for the loaded week and select `slot.current_paid_count > max` where `slot.max_paid_employees == Some(max)` (`state/shiftplan.rs:174-180`). Zero overage → render nothing (UI-SPEC empty state, like the conflict section).

**Alternative (not recommended):** reuse `WarningList` (`warning_list.rs:90-168`) by synthesizing `WarningTO::PaidEmployeeLimitExceeded` from state — but that bypasses the localized section heading. UI-SPEC §Surface 3 recommends the custom section.

---

### `shifty-dioxus/src/i18n/{mod,en,de,cs}.rs` — new keys (i18n)

**Analog:** existing `Key` enum entries (`mod.rs`: `BookingWarningPaidLimitExceeded:565`, `Settings:319`) + per-locale `add_text` (`de.rs:506`, `cs.rs:476`) + the `i18n_*_present_in_all_locales` test pattern (`mod.rs:604,683,708,…`).

Add to `Key` enum (`mod.rs`): `SettingsPaidLimitToggleLabel`, `SettingsPaidLimitToggleDescription`, `SettingsPaidLimitToggleOn`, `SettingsPaidLimitToggleOff`, `SettingsSaved`, `SettingsSaveError`, `ShiftplanPaidOverageSectionHeader`, `ShiftplanPaidOverageRow`, `BookingBlockedPaidLimit`. Add `add_text(Locale::En|De|Cs, Key::…, "…")` in each locale file (exact strings in UI-SPEC §Copywriting Contract). Add a `#[test] fn i18n_phase24_keys_present_in_all_locales()` mirroring the existing present-in-all-locales tests. **Reuse `Key::Settings`** for nav/heading (no new key). `Key::BookingWarningPaidLimitExceeded` is reused only if the WarningList-alternative is chosen.

---

## Shared Patterns

### Permission gate (`.is_ok()` bypass)
**Source:** `service_impl/src/sales_person_shiftplan.rs:84-88`
**Apply to:** the D-24-04 gate fix and the D-24-02 shiftplanner-bypass in `book_slot_with_conflict_check`.
```rust
let is_shiftplanner = self
    .permission_service
    .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
    .await
    .is_ok();
```
Privilege constants: `service/src/permission.rs:9-11` (`SALES_PRIVILEGE`, `HR_PRIVILEGE`, `SHIFTPLANNER_PRIVILEGE`).

### ServiceError → HTTP status (non-403, surfaced)
**Source:** `rest/src/lib.rs:177-188` (409/422 arms)
**Apply to:** the new `PaidLimitExceeded` mapping. MUST be a status the frontend does not silently drop (avoid 403; `shiftplan.rs:431-436` silently ignores 403). Frontend matches the same status in the `AddUserToSlot` handler.

### Transaction boilerplate
**Source:** every method in `service_impl/src/shiftplan_edit.rs` (`:405` use_transaction, `:550` commit)
**Apply to:** any new service logic. Pattern: `let tx = self.transaction_dao.use_transaction(tx).await?; … self.transaction_dao.commit(tx).await?;` (CLAUDE.md Transaction rule). The pre-persist block returns `Err` BEFORE commit, so no booking is persisted.

### Inline error text (frontend, non-dismissible)
**Source:** `shifty-dioxus/src/component/slot_edit.rs:264-266`
**Apply to:** both the Settings save error and the booking hard-block error. Class: `text-bad text-small font-normal` (+ `mt-1` at the slot).

### i18n three-locale rule
**Source:** `mod.rs` `Key` enum + `en.rs`/`de.rs`/`cs.rs` `add_text` + `i18n_*_present_in_all_locales` tests
**Apply to:** every new string. All three locales mandatory (CLAUDE.md + UI-SPEC accessibility gate).

### Nav target wiring (admin-gated)
**Source:** the `NavTarget::UserManagement` chain across `top_bar.rs` (struct field, `nav_visibility`, enum, `is_active_for`, `is_admin_target`, items-builder)
**Apply to:** `NavTarget::Settings`. All six sites must change together or the build/tests break.

---

## No Analog Found

None. Every new/modified artifact has a concrete in-repo analog. The closest thing to "new" is `SettingsPage`, which is a thin variant of `UserManagementPage` (role-match) plus the standard nav/route/barrel wiring.

---

## Metadata

**Analog search scope:** `service/`, `service_impl/`, `rest/`, `migrations/sqlite/`, `shifty_bin/`, `shifty-dioxus/src/{page,component,service,state,i18n,api.rs,loader.rs,router.rs}`
**Files scanned:** ~22 (8 backend source/migration/test, 9 frontend source, plus CLAUDE.md/UI-SPEC/CONTEXT)
**Pattern extraction date:** 2026-06-27
**Gates to honor (from CLAUDE.md):** backend `cargo clippy --workspace -- -D warnings` (hard gate, not run by `cargo test`); frontend `cargo build --target wasm32-unknown-unknown` (WASM gate, run from `shifty-dioxus/`); OpenAPI `#[utoipa::path]` on the changed REST handler; `Option<Transaction>` convention.
