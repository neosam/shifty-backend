# Feature: System Infrastructure (Feature Flags, Toggles, Scheduler, Clock, UUID, Shortday Gate, Config)

> **In short:** Cross-cutting services that neither maintain domain
> entities nor expose user workflows, but carry the platform: two
> switching mechanisms (Feature Flags & Toggles), a cron runner
> (Scheduler), test abstractions (Clock, UUID), a central cutover-date
> gate (`shortday_gate`), and env config.

**Cluster ID:** F13
**Status:** in production
**First introduced:** originally with the backend core; notable extensions
in milestone v1.7 (Toggle values, HCFG-02), Phase 2 (`feature_flag`),
Phase 24 (`paid_limit_hard_enforcement`), Phase 25 (`holiday_auto_credit`),
Phase 48 (PDF Scheduler) and Phase 51 (Toggle Full-Bypass +
`shortday_gate`).
**Responsible crates:** `service::{feature_flag, toggle, scheduler, clock,
uuid_service, config}`, `service_impl::{…, shortday_gate}`, `dao::{feature_flag,
toggle}`, `dao_impl_sqlite::{feature_flag, toggle}`, `rest::{feature_flag,
toggle}`.

---

## 1. What is it? (Business perspective)

Business-visible from this cluster in the UI is only a strip: the cards
on `/settings` (paid-limit enforcement, holiday auto-credit cutover date,
short-day slot-clipping cutover date). Everything else is infrastructure —
but infrastructure that *has business consequences*, because it decides
**whether** and **from which date** a new rule takes effect at all.

Six sub-services share the role:

- **Feature Flags** — boolean switches for **architecture / migration
  cutovers** (e.g. `absence_range_source_active` in Phase 2: switches the
  vacation/sick/UnpaidLeave calculation from `extra_hours` to
  `absence_period`). Not user-facing, flipped atomically via migration at
  cutover.
- **Toggles** — user-facing switches with an optional **string value**
  (typically an ISO date). Managed by admins through `/settings` and
  implement cutover-date rollouts: "from 2026-08-01 onward, slots on
  ShortDays are clipped".
- **Scheduler** — cron runner that hourly executes
  `update_carryover_all_employees` for the previous and current year, so
  that balance reports stay fast.
- **Clock / UUID** — two one-method traits that put system time and UUID
  generation behind a `mockall`-capable interface. No domain business,
  pure test abstraction.
- **Config** — reads `TIMEZONE` and `ICAL_LABEL` from env vars at
  startup.
- **Shortday Gate** (`service_impl/src/shortday_gate.rs`) — a helper
  module (not a service) that reads the
  `shortday_slot_clipping_active_from` toggle, parses the result, and
  provides the central clip algorithm for all four aggregate chains
  (Block, Shiftplan, BookingInformation, ShiftplanReport).

**Example workflow from a user's perspective (only what's visible):**

1. Admin opens `/settings`.
2. Card 1 (paid limit): button toggle between "hard" and "soft"
   enforcement — flips the boolean toggle
   `paid_limit_hard_enforcement`.
3. Card 2 / 2b: type in an ISO date and save — writes the date value to
   `toggle.value` for `holiday_auto_credit` and
   `shortday_slot_clipping_active_from` respectively.
4. All other aggregates read the toggle on every read and adjust their
   behavior date-dependently.

## 2. Business Rules

### 2.1 Feature Flags vs Toggles — the separation

This is the only rule you need to internalize to understand this
cluster. The separation is **explicit** and justified in
`migrations/sqlite/20260501000000_add-feature-flag-table.sql:1-4`:

> "Intentionally NO reuse of toggle/ToggleService — semantic separation:
> Feature Flags are architecture / migration switches, Toggles are user
> toggles."

| Axis | `feature_flag` | `toggle` |
| --- | --- | --- |
| Purpose | Cutover / migration / architecture | User-facing rollouts |
| Audience | Deployment / backend-internal | End user (admin in UI) |
| Value | only `bool` (`enabled`) | `bool` + optional `value: TEXT` |
| Grouping | none | `toggle_group` + junction table |
| Cutover-date semantics | none — flips atomically | ISO date in `value` = cutoff |
| Admin privilege | `feature_flag_admin` | `toggle_admin` |
| Read/write flow REST | only `GET /feature-flag/{key}` (read-only) | full CRUD (`/toggle`, `/toggle-group`) |
| Write path | migration (`INSERT` in seed SQL) or `set()` with admin privilege — no REST PUT | REST PUT `/toggle/{name}/enable\|disable\|value` |
| Auth bypass for reads | `Authentication::Full` bypasses user check (`feature_flag.rs:36-42`) | same, but only since Phase 51 (`toggle.rs:46-51`) |
| Typical examples | `absence_range_source_active` | `paid_limit_hard_enforcement`, `holiday_auto_credit`, `shortday_slot_clipping_active_from` |

**Consequence:** new user-facing rollout switches come as toggles. New
migration / architecture switches (that you want to remove once the
cutover is complete) come as feature flags.

**[To verify]** whether there are, besides `absence_range_source_active`
(Phase 2), other actively read feature flags — a grep over
`service_impl/src/*.rs` (excluding `feature_flag.rs` itself) currently
finds no reader. The flag thus exists as a cutover marker; the service
API is provided, but no consumer chain reads it at runtime yet.

### 2.2 Toggle semantics

- **Fail-safe default:** unknown toggle name → `is_enabled → false`,
  `get_toggle_value → None` (`ToggleDao`, see DAO impl).
- **Value semantics:** `value` is purpose-open `TEXT`. In practice: ISO
  date (`YYYY-MM-DD`) with the semantics "active from this date
  inclusive".
- **REST validation:** `PUT /toggle/{name}/value` validates the ISO date
  format before persisting (`rest/src/toggle.rs:350-368`).
- **Set-value as convenience:** `PUT` on `/value` implicitly sets
  `enabled=1` (see DAO trigger behavior in
  `dao_impl_sqlite/src/toggle.rs` — **[To verify]** concretely in the
  impl); `DELETE` on `/value` sets `enabled=0` and `value=NULL`. The
  handler comment (`rest/src/toggle.rs:337`) describes "toggle value
  set; toggle enabled".
- **Read ops accept `Authentication::Full`** (Phase 51, see chapter 7).

### 2.3 Feature Flag semantics

- **UPDATE-only DAO:** the migration MUST seed every known key. There
  is no `INSERT` by the app (`dao/src/feature_flag.rs:26-32`).
- **Fail-safe:** unknown key → `is_enabled → false`
  (`dao/src/feature_flag.rs:17`).
- **Auth read:** any authenticated user may read. `Full` passes without
  a user-ID check (`service_impl/src/feature_flag.rs:36-42`).
- **No REST write:** the only exposed REST endpoint is
  `GET /feature-flag/{key}` (`rest/src/feature_flag.rs:34-36`). Writes
  go exclusively via the service trait with
  `FEATURE_FLAG_ADMIN_PRIVILEGE` — in the backend this means "via
  migration + deployment", not "via HTTP".

### 2.4 Scheduler rules

- **Cron expression hardcoded:** `SchedulerServiceImpl::start()`
  schedules the carryover job with `"0 * * * * *"` (every minute at the
  0th second tick — effectively every minute)
  (`service_impl/src/scheduler.rs:45`). No env override in the current
  code.
- **Two runs per tick:** first `year-1`, then `year` — each with
  `Authentication::Full`, `tx=None` (`scheduler.rs:59-70`).
- **Error isolation:** both runs are wrapped in their own `if let Err(e) = ...`
  and only logged (`error!`), not propagated — the next tick tries
  again.
- **PDF export Scheduler** (`service_impl/src/pdf_export_scheduler.rs`)
  is a separate Scheduler for the PDF batch export (see F11 Export); it
  runs alongside the carryover Scheduler and is also started in
  `main.rs:1441-1445`.

### 2.5 Clock and UUID service rules

- **Clock:** UTC. `time_now()`, `date_now()`, `date_time_now()` all
  deliver a UTC-based `time::OffsetDateTime` and throw the zone away
  (`service_impl/src/clock.rs:6-14`). **[To verify]** whether that is
  ok for all consumers or whether reports already expect local time —
  this is the standard trap for the time/timezone edge case (see
  `docs/domain/edge-cases.md#4-zeit--zeitzone`).
- **UUID:** `Uuid::new_v4()` — pure V4 random UUIDs. The `usage`
  argument is discarded by the prod impl
  (`service_impl/src/uuid_service.rs:6-8`); it exists for tests so that
  mocks can deliver distinct UUIDs per call site.

### 2.6 Shortday Gate rules (Phase 51, D-51-07)

- **Central location:** `service_impl/src/shortday_gate.rs`. No
  dedicated service, no DAO — a module with pure functions plus a
  crate-local helper `read_active_from` that reads the toggle.
- **Toggle-name constant:** `TOGGLE_NAME =
  "shortday_slot_clipping_active_from"` (`shortday_gate.rs:42`).
- **Parse tolerance:** `parse_active_from(None|Some("")|Some(bad)) → None`
  (`shortday_gate.rs:51-57`). No panics on broken values.
- **`Unauthorized` tolerance:** `read_active_from` explicitly swallows
  `Unauthorized` into `Ok(None)` — for legacy setups and mock-auth tests
  (`shortday_gate.rs:105-117`). After Phase 51 the case no longer
  occurs on the prod path, because `ToggleService` lets `Full` through;
  the tolerance remains as a safety belt.
- **Mode split:** `ShortdayMode::{Modern, Legacy}` — Modern (chain
  A'/D: `block.rs`, `shiftplan_report.rs`) keeps the raw slot when the
  gate is off. Legacy (chain B/C: `shiftplan.rs`, `booking_information.rs`)
  reproduces pre-Phase-51 filtering so that historical data stays stable
  (`shortday_gate.rs:143-174`).

## 3. Data Model

### Tables

| Table | Purpose | Important columns |
| --- | --- | --- |
| `feature_flag` | Cutover switches | `key PK`, `enabled INT`, `description`, `update_timestamp`, `update_process` |
| `toggle` | User toggle | `name PK`, `enabled INT`, `description`, `value TEXT` (nullable), `update_timestamp`, `update_process` |
| `toggle_group` | Grouping | `name PK`, `description`, `update_timestamp`, `update_process` |
| `toggle_group_toggle` | Junction | `toggle_group_name`, `toggle_name`, ON DELETE CASCADE, UNIQUE constraint |
| `privilege` | extended | `toggle_admin`, `feature_flag_admin` are seeded on migration |

Note:

- No `deleted` field — Toggles and Feature Flags have no soft-delete.
  `DELETE` is a real row removal.
- `update_process` is a required field — the service fills it with
  `"toggle-service"` or `"feature-flag-service"`
  (`service_impl/src/toggle.rs:11`, `feature_flag.rs:10`).

### Migrations

- `20260105000000_app-toggles.sql` — base: `toggle`, `toggle_group`,
  `toggle_group_toggle` + `toggle_admin` privilege.
- `20260501000000_add-feature-flag-table.sql` — `feature_flag` table +
  seed for `absence_range_source_active` (Phase 2) +
  `feature_flag_admin` privilege.
- `20260627000000_seed-paid-limit-toggle.sql` — seed
  `paid_limit_hard_enforcement` (Phase 24).
- `20260628000000_toggle-value-column.sql` — `ALTER TABLE toggle ADD COLUMN
  value TEXT;` — the cutover-date era begins.
- `20260628000001_seed-holiday-auto-credit-toggle.sql` — seed
  `holiday_auto_credit` (Phase 25, HCFG-02).
- `20260704000001_seed-shortday-slot-clipping-toggle.sql` — seed
  `shortday_slot_clipping_active_from` (Phase 51, D-51-07).

### Relationships

`toggle` ↔ `toggle_group` via `toggle_group_toggle` (junction, `UNIQUE`
prevents double assignment). CASCADE DELETE on both FKs. `feature_flag`
stands alone and references nothing.

## 4. Service API

### 4.1 `FeatureFlagService` (Basic Service)

`service::feature_flag::FeatureFlagService`

```rust
#[async_trait]
pub trait FeatureFlagService {
    type Context; type Transaction;
    async fn is_enabled(&self, key: &str, context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>) -> Result<bool, ServiceError>;
    async fn set(&self, key: &str, value: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>) -> Result<(), ServiceError>;
}
```

- **`is_enabled`** — auth-only, any authenticated user; `Full` bypasses
  (`service_impl/src/feature_flag.rs:36-42`).
- **`set`** — requires `FEATURE_FLAG_ADMIN_PRIVILEGE`
  (`service_impl/src/feature_flag.rs:56-59`).
- **Deps:** `FeatureFlagDao`, `PermissionService`, `TransactionDao` —
  pure Basic-Service pattern, no other domain services.

### 4.2 `ToggleService` (Basic Service)

`service::toggle::ToggleService` — 16 methods, divided into four
groups. Core:

- **Read (auth-only, `Full` bypasses):** `is_enabled`, `get_all_toggles`,
  `get_toggle`, `get_toggle_value` (`service_impl/src/toggle.rs:26-98,
  176-202`).
- **Write (admin):** `create_toggle`, `enable_toggle`, `disable_toggle`,
  `set_toggle_value`, `delete_toggle` — all with
  `check_permission(TOGGLE_ADMIN_PRIVILEGE, …)`.
- **Group read/write (admin):** `create_toggle_group`,
  `delete_toggle_group`, `get_all_toggle_groups`, `get_toggle_group`.
  **Notable:** the group reads are admin-gated
  (`service_impl/src/toggle.rs:287-290, 304-307`) even though toggle
  reads are auth-only — groups are an admin concern.
- **Group membership (admin):** `add_toggle_to_group`,
  `remove_toggle_from_group`, `get_toggles_in_group`, `enable_group`,
  `disable_group`.
- **`enable_toggle` / `disable_toggle`** are read-modify-write in *one*
  TX: `get_toggle` → mutate `enabled` → `update_toggle` → `commit`
  (`service_impl/src/toggle.rs:131-144`).

### Auth gates

- Read-auth-only for `toggle`/`feature_flag` with `Full`-Bypass (Phase 51
  fix).
- Admin privilege is called `toggle_admin` and `feature_flag_admin`
  respectively and is applied via seed migration each.

### TX behavior

- Every method opens TX via `transaction_dao.use_transaction(tx)`,
  performs the DAO call, commits. `enable_toggle`/`disable_toggle`
  perform an atomic read-modify-write within *one* TX. No composite op
  across multiple aggregates — everything single-row scope.

### Dependencies

- `FeatureFlagService`: `FeatureFlagDao`, `PermissionService`, `TransactionDao`.
- `ToggleService`: `ToggleDao`, `PermissionService`, `TransactionDao`.
- **Both are Basic Services** (service tier convention from
  `CLAUDE.md`). They consume no domain services.

### 4.3 `SchedulerService`

`service::scheduler::SchedulerService`

```rust
async fn start(&self) -> Result<(), ServiceError>;
async fn schedule_carryover_updates(&self, cron: &'static str) -> Result<(), ServiceError>;
```

Impl detail: `SchedulerServiceImpl` holds an
`Arc<Mutex<Scheduler<Local>>>` (`tokio_cron`) as a custom field.
**Deps:** `ShiftplanEditService` (which provides
`update_carryover_all_employees(year, Auth, tx)`) —
`scheduler.rs:14-20`. This places the Scheduler in the Business-Logic
tier zone: it consumes a domain service.

The commented-out `tokio::spawn` loop in `start()`
(`service_impl/src/scheduler.rs:39-44`) shows that the Scheduler loop
itself does not put the process to sleep — `tokio_cron` manages the
tick internally. Whether that is currently intended or a leftover
refactor: **[To verify]**.

### 4.4 `ClockService` and `UuidService`

Both are synchronous one-method traits without auth, without TX,
without DAO. Purpose: test injection. The concrete impls are two lines
long and wrap `OffsetDateTime::now_utc()` and `Uuid::new_v4()`.

### 4.5 `ConfigService`

`service::config::ConfigService` — one method `get_config() ->
Result<Config, ServiceError>`. `Config` contains `timezone` and
`ical_label`. `ConfigServiceImpl` reads fresh from `std::env` on every
call (`service_impl/src/config.rs:12-22`) — no caching. Fallbacks:
`"UTC"` and `"Schicht"`.

### 4.6 `shortday_gate` module (not a service)

Public API:

- `TOGGLE_NAME: &str` — constant, so consumers don't scatter magic
  strings.
- `parse_active_from(Option<&str>) -> Option<Date>` — ISO-8601 parse
  with defensive `None` fallback.
- `should_clip(booking_date, active_from) -> bool` — inclusive at
  cutover date (`shortday_gate.rs:66-71`).
- `resolve_active_from_for_week(year, week, dow, active_from) -> bool` —
  convenience for consumers that only have `(year, week, day_of_week)`.
- `clip_slot_for_week(slot, special_days, year, week, active_from, mode) ->
  ClipOutcome` — the master helper, used by all four chains
  (`shortday_gate.rs:193-240`).

Crate-local:

- `read_active_from<S: ToggleService>(svc, ctx) -> Result<Option<Date>>` —
  tolerates `Unauthorized`.

## 5. REST Endpoints

### 5.1 `/feature-flag`

| Method | Path | Description | DTO In | DTO Out | Errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/feature-flag/{key}` | Fail-safe read; unknown key → `enabled=false`. | — | `FeatureFlagTO` | 401 |

`FeatureFlagTO`: `{ key: String, enabled: bool, description: Option<String> }`
(`rest-types/src/lib.rs:2363-2370`). The handler sets `description: None`
because the trait `is_enabled` does not deliver it
(`rest/src/feature_flag.rs:60-66`). Write access does **not** exist over
REST — intentionally (Phase 8 08-07 comment
`rest/src/feature_flag.rs:80-88`).

### 5.2 `/toggle`

| Method | Path | Description | DTO In | DTO Out | Errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/toggle` | All toggles | — | `[ToggleTO]` | 401 |
| `POST` | `/toggle` | Create toggle | `ToggleTO` | 201 | 401/403 |
| `GET` | `/toggle/{name}` | Single toggle | — | `ToggleTO` | 401/404 |
| `GET` | `/toggle/{name}/enabled` | Just boolean | — | `bool` | 401 |
| `PUT` | `/toggle/{name}/enable` | Turn on | — | 204 | 401/403/404 |
| `PUT` | `/toggle/{name}/disable` | Turn off | — | 204 | 401/403/404 |
| `DELETE` | `/toggle/{name}` | Delete | — | 204 | 401/403 |
| `GET` | `/toggle/{name}/value` | Read value (string) | — | `String` or 204 | 401 |
| `PUT` | `/toggle/{name}/value` | Set value (ISO date validated) | JSON string | 204 | 400/401/403 |
| `DELETE` | `/toggle/{name}/value` | Delete value (+disable) | — | 204 | 401/403 |

### 5.3 `/toggle-group`

Analogous CRUD endpoints for groups + `POST /toggle-group/{group}/toggle/{toggle}`
for membership. Registration: `rest/src/lib.rs:587-588`.

DTOs (`rest-types/src/lib.rs`):

- `ToggleTO { name, enabled, description, value }`
- `ToggleGroupTO { name, description }`
- `FeatureFlagTO { key, enabled, description }`
- Serde default for `description`/`value` — the frontend may omit them.

## 6. Frontend Integration

- **Page:** `shifty-dioxus/src/page/settings.rs` — the only UI site for
  this cluster. Three cards show the three current toggles:
  - Card 1 (`settings.rs:572-628`): `paid_limit_hard_enforcement`
    (Phase 24) via `loader::get_toggle_enabled` /
    `loader::set_toggle`.
  - Card 2 (`settings.rs:630-715`): `holiday_auto_credit` — ISO date
    input, `loader::get_holiday_cutoff_date` /
    `set_holiday_cutoff_date`.
  - Card 2b (`settings.rs:717-…`, Phase 51 SHC-06):
    `shortday_slot_clipping_active_from` — blueprint identical to Card 2,
    `loader::get_shortday_clipping_active_from` /
    `set_shortday_clipping_active_from`.
- **API client:** `shifty-dioxus/src/api.rs` — `set_toggle`,
  `get_toggle_enabled`, `get_toggle_value`, `set_toggle_value`,
  `clear_toggle_value`, `get_feature_flag`.
- **Loader:** `shifty-dioxus/src/loader.rs:893-957` bundles the toggle
  calls behind business names (`get_holiday_cutoff_date`,
  `get_shortday_clipping_active_from`, …). The `TOGGLE_NAME` string in
  the frontend deliberately mirrors
  `service_impl::shortday_gate::TOGGLE_NAME`
  (`loader.rs:932-935`).
- **i18n keys:** `SettingsPaidLimitToggleLabel/On/Off/Description`;
  cards for Holiday + Shortday follow the same pattern — all three
  locales (En, De, Cs) must be maintained together on changes.
- **Proxy:** `Dioxus.toml` must proxy `/toggle`, `/toggle-group`,
  `/feature-flag` to the backend. New routes need their own
  `[[web.proxy]]` entry, otherwise 404 in `dx serve` dev mode (see
  memory `feedback_dioxus_proxy_for_new_backend_endpoints`).

## 7. Edge cases

Central reference:
[`../domain/edge-cases.md#9-feature-toggles--stichtag-rollouts`](../domain/edge-cases.md#9-feature-toggles--stichtag-rollouts).

Feature-specific:

- **Edge case — toggle read under `Authentication::Full` (Phase 51
  gap closure):** internal aggregate consumers (chain C
  `booking_information.rs:310, 547`, chain D `reporting.rs`,
  `shiftplan_report.rs:118, 211, 270`, chain A' `block.rs:103, 292`)
  call `ToggleService` reads **with `Authentication::Full`** to have
  an HR bypass. Before Phase 51, `PermissionService::current_user_id(Full)`
  returned `Ok(None)` → `ToggleService` threw `Unauthorized` →
  `shortday_gate` silently swallowed to `Ok(None)` → slot clipping
  never took effect even though the toggle was set (live symptom: full
  slot hour instead of clipped 0.5 h). The fix in
  `service_impl/src/toggle.rs:46-51, 66-71, 87-92, 191-196` treats
  `Full` as an all-rights bypass. Regression guards live in
  `service_impl/src/test/toggle.rs:547-556` — the four tests must not
  call `PermissionService::current_user_id` at all (a mock without
  `expect_*` would panic). See also memory
  `reference_toggle_service_full_context_bypass`.
- **Edge case — feature flag flipped live:** the service API allows
  `set()` at runtime, but the only prod consumer
  (`absence_range_source_active`) is intended as a cutover: flip it
  atomically with a Phase 4 migration, never manually. Migration
  comment (`20260501000000_add-feature-flag-table.sql:18`): "Flip
  atomically with Phase-4 migration; do NOT flip manually."
- **Edge case — Scheduler missed tick:** the cron job runs with
  `"0 * * * * *"`, i.e. every minute. If a tick is missed (e.g. because
  an earlier run is hanging), `tokio_cron` simply jumps to the next
  slot — there is no catch-up mechanism. For carryover this is
  harmless: the next tick recomputes the current truth. Important to
  know: errors are only logged (`error!` in `scheduler.rs:63, 71`), not
  monitored — whoever needs real alerts must evaluate the logs.
- **Edge case — toggle value implicitly sets `enabled=1`:** the REST
  semantics of `PUT /toggle/{name}/value` is "set value **and**
  activate" (`rest/src/toggle.rs:337`). A value-based rollout (ISO
  date) therefore cannot be "value set but disabled". `DELETE /value`
  resets to `NULL` + `enabled=0`.
- **Edge case — `parse_active_from` on broken values:** `Some("garbage")`
  → `None` rather than panic. The consumer chain falls back to legacy
  behavior, not a 500 response (`shortday_gate.rs:51-57`, test
  coverage `shortday_gate.rs:262-265`).
- **Edge case — Clock in UTC vs report in local time:** `ClockService`
  always delivers UTC. Consumers that need local time (e.g.
  `Local`-based Scheduler in `SchedulerServiceImpl::new`) must convert
  themselves. **[To verify]** where this currently bites in practice.

## 8. Tests

- **Unit — Feature Flag:** `service_impl/src/test/feature_flag.rs`
  (196 lines). Covers auth combinations, admin gate for `set()`,
  fail-safe for unknown keys.
- **Unit — Toggle:** `service_impl/src/test/toggle.rs` (652 lines).
  Covers full CRUD + group ops + value ops. From
  `service_impl/src/test/toggle.rs:547-556` explicitly the Phase 51
  regression guards for the `Full` bypass (with the mock-without-expectation
  trick as an assertion).
- **Unit — Shortday Gate:** `service_impl/src/shortday_gate.rs:242-479`
  in the same file. Covers `parse_active_from`, `should_clip`,
  `resolve_active_from_for_week` including `Legacy` vs `Modern` mode
  against all combinations of (gate active/inactive, ShortDay yes/no,
  slot.to relative to cutoff).
- **Integration (chain level):** the four chains are tested separately
  — `service_impl/src/test/shiftplan.rs` (chain B),
  `test/booking_information_chain_c.rs` (chain C), etc. They cover
  behavior with gate on and gate off + legacy filter.
- **Integration — Scheduler:** **not directly tested.** The
  `SchedulerServiceImpl` has no test file; it is covered implicitly by
  the fact that `update_carryover_all_employees` is tested in
  `service_impl/src/test/shiftplan_edit/*`. **Known gap:** cron
  parsing, error isolation between the `year-1` and `year` runs is
  only verified by manual test.
- **Clock / UUID:** no dedicated tests — they are themselves the test
  abstraction. Consumers mock them.
- **Config:** no dedicated tests — pure env read.

## 9. History & Context

- **v1.0 era:** `toggle`, `toggle_group`, `toggle_group_toggle` as
  base infrastructure (`20260105000000`).
- **Phase 2 (2026-05):** feature-flag table as a *deliberately separate*
  mechanism for absence cutover control
  (`absence_range_source_active`). Design note in
  `openspec/changes/billing-period-snapshot-versioning/design.md`
  **[To verify whether the feature-flag separation is also explained
  there]**.
- **Phase 24 (2026-06-27):** first user-facing boolean toggle
  (`paid_limit_hard_enforcement`) — hard/soft enforcement.
- **Phase 25 / HCFG-02 (2026-06-28):** toggle-value column + first
  cutover-date toggle (`holiday_auto_credit`). The pattern
  "value=ISO date, semantics from-cutover-date" is established here and
  prototyped for consumers in
  `service_impl/src/reporting.rs:164-180`.
- **Phase 48:** PDF export Scheduler introduced
  (`pdf_export_scheduler.rs`), parallel to the carryover Scheduler.
- **Phase 51 / D-51-07 (2026-07-04):**
  - Shortday slot-clipping toggle seeded.
  - `shortday_gate` module as the central location for all four
    consumer chains.
  - **Toggle Full-Bypass in `ToggleService` reads** — the important
    gap closure. Without this fix, the chain C / chain D consumers do
    not work: see `service_impl/src/toggle.rs:32-51` and regression
    tests in `service_impl/src/test/toggle.rs:547-556`.
- **Service tier convention:** both `FeatureFlagService` and
  `ToggleService` are **Basic Services** (only DAO + Permission +
  Transaction). That is intentional — they are consumed by many
  Business-Logic services, but must not consume such services
  themselves, to avoid cycles (see `CLAUDE.md` "Service Tier
  Conventions"). The Scheduler, on the other hand, is Business-Logic
  tier (consumes `ShiftplanEditService`).
- **References to planning context:** `.planning/phases/` contains
  detailed design docs with decision codes for Phase 2 (feature-flag
  cutover), Phase 24, Phase 25, Phase 48, Phase 51 (`D-Phase2-06`,
  `D-24-06`, `D-25-06`, `D-51-06/07/09`, `HCFG-02`). Whoever needs
  context for a concrete rule: look there.

---

**Conclusion:** F13 bundles two otherwise easily confused switching
mechanisms (architectural `feature_flag` vs user-facing `toggle` with
cutover-date value) plus the necessary bits and pieces (Scheduler,
Clock, UUID, Config, Shortday Gate). The central lesson from Phase 51:
**read ops for config data must accept `Authentication::Full` as an
all-rights context**, otherwise all internal aggregate chains break
silently.

*Last verification against code:* see git blame of this file.
