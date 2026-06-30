# Phase 33: Special-Days-UI in den Einstellungen - Pattern Map

**Mapped:** 2026-06-30
**Files analyzed:** 12 new/modified files
**Analogs found:** 12 / 12

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `dao/src/special_day.rs` (+`find_by_year`) | DAO trait | CRUD | `dao/src/special_day.rs` `find_by_week` | exact |
| `dao_impl_sqlite/src/special_day.rs` (+`find_by_year`) | DAO impl | CRUD | `dao_impl_sqlite/src/special_day.rs` `find_by_week` | exact |
| `service/src/special_days.rs` (+`get_by_year`) | service trait | CRUD | `service/src/special_days.rs` `get_by_week` | exact |
| `service_impl/src/special_days.rs` (+`get_by_year`) | service impl | CRUD | `service_impl/src/special_days.rs` `get_by_week` | exact |
| `rest/src/special_day.rs` (+handler+route+ApiDoc) | controller | request-response | `rest/src/special_day.rs` `get_special_days_for_week` | exact |
| `service_impl/src/test/special_days.rs` (NEW) | test | CRUD | `service_impl/src/test/slot.rs` | role-match |
| `service_impl/src/test/mod.rs` (+`pub mod special_days`) | config | — | `service_impl/src/test/mod.rs` existing entries | exact |
| `shifty-dioxus/src/api.rs` (+3 functions) | utility | request-response | `shifty-dioxus/src/api.rs` `get_special_days_for_week`, `delete_absence_period`, `create_absence_period` | exact |
| `shifty-dioxus/src/page/settings.rs` (+Card-3) | component | CRUD | `shifty-dioxus/src/page/settings.rs` Card-2 | exact |
| `shifty-dioxus/src/page/shiftplan.rs` (+per-day dropdown) | component | event-driven | `shifty-dioxus/src/page/shiftplan.rs` `field_dropdown_entries` (lines 695–738) | exact |
| `shifty-dioxus/src/i18n/mod.rs` (+17 Keys) | config | — | existing `Key` enum entries | exact |
| `shifty-dioxus/src/i18n/{de,en,cs}.rs` (+17 translations) | config | — | existing locale files | exact |

---

## Pattern Assignments

### `dao/src/special_day.rs` — add `find_by_year` to trait

**Analog:** same file, `find_by_week` method (lines 31–35)

**Existing trait (lines 27–38):**
```rust
#[automock]
#[async_trait]
pub trait SpecialDayDao {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SpecialDayEntity>, DaoError>;
    async fn find_by_week(
        &self,
        year: u32,
        calendar_week: u8,
    ) -> Result<Arc<[SpecialDayEntity]>, DaoError>;
    async fn create(&self, entity: &SpecialDayEntity, process: &str) -> Result<(), DaoError>;
    async fn update(&self, entity: &SpecialDayEntity, process: &str) -> Result<(), DaoError>;
}
```

**New method to add (after `find_by_week`):**
```rust
async fn find_by_year(&self, year: u32) -> Result<Arc<[SpecialDayEntity]>, DaoError>;
```

`#[automock]` on the trait generates `MockSpecialDayDao::expect_find_by_year()` automatically — no manual mock needed.

---

### `dao_impl_sqlite/src/special_day.rs` — add `find_by_year` impl

**Analog:** same file, `find_by_week` impl (lines 86–107)

**Clone of `find_by_week`, changing WHERE clause and adding ORDER BY:**
```rust
async fn find_by_year(&self, year: u32) -> Result<Arc<[SpecialDayEntity]>, DaoError> {
    let year = year as i64;
    Ok(query_as!(
        SpecialDayDb,
        r#"
        SELECT id, year, calendar_week, day_of_week, day_type, time_of_day, created, deleted, update_version
        FROM special_day
        WHERE year = ? AND deleted IS NULL
        ORDER BY calendar_week ASC, day_of_week ASC
        "#,
        year
    )
    .fetch_all(&*self.pool)
    .await
    .map_db_error()?
    .iter()
    .map(SpecialDayEntity::try_from)
    .collect::<Result<_, _>>()?)
}
```

Key differences from `find_by_week`: single bind param (`year` only), no `calendar_week = ?` filter, add `ORDER BY calendar_week ASC, day_of_week ASC`.

---

### `service/src/special_days.rs` — add `get_by_year` to trait

**Analog:** same file, `get_by_week` (lines 85–90)

**Existing `get_by_week` signature pattern:**
```rust
async fn get_by_week(
    &self,
    year: u32,
    calendar_week: u8,
    context: Authentication<Self::Context>,
) -> Result<Arc<[SpecialDay]>, ServiceError>;
```

**New method to add:**
```rust
async fn get_by_year(
    &self,
    year: u32,
    context: Authentication<Self::Context>,
) -> Result<Arc<[SpecialDay]>, ServiceError>;
```

The `#[automock(type Context=();)]` macro on the trait (line 81) auto-generates the mock method.

---

### `service_impl/src/special_days.rs` — add `get_by_year` impl

**Analog:** same file, `get_by_week` impl (lines 58–71)

**`get_by_week` impl to clone:**
```rust
async fn get_by_week(
    &self,
    year: u32,
    calendar_week: u8,
    _context: Authentication<Self::Context>,
) -> Result<Arc<[SpecialDay]>, ServiceError> {
    Ok(self
        .special_day_dao
        .find_by_week(year, calendar_week)
        .await?
        .iter()
        .map(SpecialDay::from)
        .collect())
}
```

**New impl (drop `calendar_week`, call `find_by_year`):**
```rust
async fn get_by_year(
    &self,
    year: u32,
    _context: Authentication<Self::Context>,
) -> Result<Arc<[SpecialDay]>, ServiceError> {
    Ok(self
        .special_day_dao
        .find_by_year(year)
        .await?
        .iter()
        .map(SpecialDay::from)
        .collect())
}
```

Note: `_context` is intentionally unused (ungegated read, same pattern as `get_by_week`).

---

### `rest/src/special_day.rs` — add `get_special_days_for_year` handler + route + ApiDoc

**Analog:** same file, `get_special_days_for_week` (lines 27–62) and `generate_route` (lines 17–25)

**Imports (lines 1–15) — unchanged, already complete.**

**`generate_route` addition (after existing `/for-week/...` route, line 21):**
```rust
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route(
            "/for-week/{year}/{calendar_week}",
            get(get_special_days_for_week::<RestState>),
        )
        .route(
            "/for-year/{year}",                          // NEW
            get(get_special_days_for_year::<RestState>), // NEW
        )
        .route("/", post(create_special_days::<RestState>))
        .route("/{id}", delete(delete_special_day::<RestState>))
}
```

**New handler (clone of `get_special_days_for_week`, single Path param):**
```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/for-year/{year}",
    tags = ["Special Days"],
    params(
        ("year" = u32, Path, description = "The year")
    ),
    responses(
        (status = 200, description = "Get special days for a year", body = [SpecialDayTO], content_type = "application/json"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_special_days_for_year<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(year): Path<u32>,
) -> Response {
    error_handler(
        (async {
            let special_days: Arc<[SpecialDayTO]> = rest_state
                .special_day_service()
                .get_by_year(year, context.into())
                .await?
                .iter()
                .map(SpecialDayTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&special_days).unwrap()))
                .unwrap())
        })
        .await,
    )
}
```

**`SpecialDayApiDoc` — add `get_special_days_for_year` to `paths(...)` (lines 127–139):**
```rust
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Special Days", description = "Special Days API")
    ),
    paths(
        get_special_days_for_week,
        get_special_days_for_year,  // ADD
        create_special_days,
        delete_special_day
    ),
    components(schemas(SpecialDayTO))
)]
pub struct SpecialDayApiDoc;
```

---

### `service_impl/src/test/special_days.rs` (NEW file)

**Analog:** `service_impl/src/test/slot.rs` (lines 1–80 shown; full file for mockall pattern)

**Imports pattern (copy from slot.rs lines 1–17, adapt for special_days):**
```rust
use std::sync::Arc;

use crate::special_days::SpecialDayServiceImpl;
use dao::special_day::{MockSpecialDayDao, SpecialDayEntity, SpecialDayTypeEntity};
use mockall::predicate::{always, eq};
use service::{
    clock::MockClockService,
    permission::SHIFTPLANNER_PRIVILEGE,
    special_days::{SpecialDay, SpecialDayType},
    uuid_service::MockUuidService,
    MockPermissionService, ServiceError,
};
use shifty_utils::DayOfWeek;
use tokio;
use uuid::{uuid, Uuid};
```

**Service factory helper (copy make-service pattern from slot.rs):**
```rust
fn make_service(
    dao: MockSpecialDayDao,
    permission: MockPermissionService,
) -> SpecialDayServiceImpl<MockSpecialDayDao, MockPermissionService, MockClockService, MockUuidService> {
    SpecialDayServiceImpl::new(
        Arc::new(dao),
        Arc::new(permission),
        Arc::new(MockClockService::new()),
        Arc::new(MockUuidService::new()),
    )
}
```

**Test pattern (mockall `expect_*` + `returning`):**
```rust
#[tokio::test]
async fn test_get_by_year_returns_entries() {
    let mut dao = MockSpecialDayDao::new();
    dao.expect_find_by_year()
        .with(eq(2026u32))
        .returning(|_| Ok(Arc::from([])));
    let svc = make_service(dao, MockPermissionService::new());
    let result = svc.get_by_year(2026, ().into()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_forbidden_without_shiftplanner() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    let svc = make_service(MockSpecialDayDao::new(), permission);
    // build a minimal SpecialDay with nil id+version
    let result = svc.create(&minimal_special_day(), ().into()).await;
    assert_eq!(result, Err(ServiceError::Forbidden));
}
```

Auth context `().into()` pattern: `service::permission::Authentication` implements `From<()>` for the mock `Context = ()`.

---

### `service_impl/src/test/mod.rs` — add `pub mod special_days`

**Analog:** existing entries in same file (lines 1–73)

**Pattern (copy any existing line, e.g., line 50):**
```rust
#[cfg(test)]
pub mod special_days;
```

Add after line 72 (`pub mod vacation_entitlement_offset`) or before `pub mod reporting_avg_weekly` to keep alphabetical/logical order.

---

### `shifty-dioxus/src/api.rs` — add 3 new functions

**Analog:** same file — `get_special_days_for_week` (lines 974–984), `delete_absence_period` (lines 659–667), `create_absence_period` (lines 608–631)

**Imports (lines 1–23) — add `SpecialDayTO` already present on line 12. No new imports needed.**

**`get_special_days_for_year` (clone of `get_special_days_for_week` lines 974–984):**
```rust
pub async fn get_special_days_for_year(
    config: Config,
    year: u32,
) -> Result<Rc<[SpecialDayTO]>, reqwest::Error> {
    let url = format!("{}/special-days/for-year/{}", config.backend, year);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    Ok(res)
}
```

**`create_special_day` (clone of `create_absence_period` lines 608–631, simplified — no 422-special-case needed unless desired):**
```rust
pub async fn create_special_day(
    config: Config,
    mut body: SpecialDayTO,
) -> Result<SpecialDayTO, reqwest::Error> {
    body.id = Uuid::nil();
    body.version = Uuid::nil();
    let url = format!("{}/special-days/", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&body).send().await?;
    response.error_for_status_ref()?;
    let result: SpecialDayTO = response.json().await?;
    Ok(result)
}
```

**`delete_special_day` (clone of `delete_absence_period` lines 659–667):**
```rust
pub async fn delete_special_day(config: Config, id: Uuid) -> Result<(), reqwest::Error> {
    let url = format!("{}/special-days/{}", config.backend, id);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    Ok(())
}
```

---

### `shifty-dioxus/src/page/settings.rs` — add Card-3 (shiftplanner-gated)

**Analog:** same file, Card-2 block (lines 99–295) and the `is_admin` guard (lines 28–38)

**Imports (lines 6–16) — unchanged, `TextInput` and `loader` already imported.**

**Inner shiftplanner guard (add after existing `is_admin` guard, before `rsx!`):**
```rust
let is_shiftplanner = AUTH
    .read()
    .auth_info
    .as_ref()
    .map(|a| a.has_privilege("shiftplanner"))
    .unwrap_or(false);
```

**Signal declarations (copy Card-2 pattern, lines 101–104):**
```rust
let mut special_day_year: Signal<u32> = use_signal(|| js::get_current_year());
let mut special_day_date_str: Signal<String> = use_signal(String::new);
let mut special_day_type: Signal<Option<SpecialDayTypeTO>> = use_signal(|| None);
let mut special_day_time_str: Signal<String> = use_signal(String::new);
let mut special_day_save_result: Signal<Option<bool>> = use_signal(|| None);
let mut special_day_saving = use_signal(|| false);
```

**`use_resource` load (clone of Card-2 `cutoff_resource`, lines 107–108):**
```rust
let config_for_year = config.clone();
let special_days_resource = use_resource(move || {
    let year = *special_day_year.read();
    get_special_days_for_year(config_for_year.clone(), year)
});
```

**`spawn` save (clone of Card-2 `on_save_cutoff`, lines 125–154):**
```rust
let on_add_special_day = move |_| {
    if *special_day_saving.read() { return; }
    let val = special_day_date_str.read().clone();
    if val.is_empty() { return; }
    let date_format = format_description!("[year]-[month]-[day]");
    let Ok(date) = time::Date::parse(&val, date_format) else {
        special_day_save_result.set(Some(false));
        return;
    };
    let (iso_year, iso_week, weekday) = date.to_iso_week_date();
    // ... build SpecialDayTO and spawn create_special_day
    special_day_saving.set(true);
    spawn(async move {
        match api::create_special_day(cfg, body).await {
            Ok(_) => { special_day_save_result.set(Some(true)); special_days_resource.restart(); }
            Err(_) => { special_day_save_result.set(Some(false)); }
        }
        special_day_saving.set(false);
    });
};
```

**Card-3 rsx block (add after Card-2 closing `}`, inside `if is_shiftplanner { ... }`):**
```rust
if is_shiftplanner {
    div { class: "bg-surface border border-border rounded-md p-4 flex flex-col gap-3 mt-4",
        div { class: "flex flex-col gap-1",
            span { class: "text-body text-ink font-semibold",
                "{i18n.t(Key::SettingsSpecialDaysSectionLabel)}"
            }
            span { class: "text-small text-ink-soft",
                "{i18n.t(Key::SettingsSpecialDaysSectionDescription)}"
            }
        }
        // Form row: date input + type select + time (conditional) + add button
        // ...
        // List: iterate special_days_resource, chronological, badges, delete buttons
    }
}
```

Full Card-3 structure mirrors Card-2 (lines 234–295): label+description row, input row, action row with `spawn`, inline feedback span.

---

### `shifty-dioxus/src/page/shiftplan.rs` — add per-day Special-Day dropdown

**Analog:** same file, `field_dropdown_entries` (lines 695–738) and `is_shiftplanner` guard (lines 102–105)

**`is_shiftplanner` already defined** (lines 102–105):
```rust
let is_shiftplanner = auth_info
    .as_ref()
    .map(|auth_info| auth_info.has_privilege("shiftplanner"))
    .unwrap_or(false);
```

**Per-day dropdown entries pattern (clone `field_dropdown_entries` lines 695–738):**
```rust
// Build per weekday: existing special day for the day (from for-week data)
let existing_sd: Option<SpecialDayTO> = special_days_for_week
    .iter()
    .find(|sd| sd.day_of_week == day_of_week)
    .cloned();
let existing_id = existing_sd.as_ref().map(|sd| sd.id);
let has_entry = existing_sd.is_some();

let day_entries: Rc<[DropdownEntry]> = {
    let cfg_h = config.clone();
    let cfg_s = config.clone();
    let cfg_d = config.clone();
    vec![
        (
            i18n.t(Key::ShiftplanDayTypeHoliday),
            Box::new(move |_| {
                spawn(async move {
                    // create Holiday, reload for-week
                });
            }),
        ).into(),
        (
            i18n.t(Key::ShiftplanDayTypeShortDay),
            Box::new(move |_| {
                // set shortday_prompt_day signal to show inline time input
                shortday_prompt_day.set(Some(day_of_week));
            }),
        ).into(),
        (
            i18n.t(Key::ShiftplanDayTypeNone),
            Box::new(move |_| {
                if let Some(id) = existing_id {
                    spawn(async move {
                        // delete_special_day, reload for-week
                    });
                }
            }),
            !has_entry,  // disabled=true hides from DropdownBase (pitfall 3)
        ).into(),
    ].into()
};
```

**ShortDay inline-prompt state (add as component-level signal):**
```rust
let mut shortday_prompt_day: Signal<Option<DayOfWeekTO>> = use_signal(|| None);
```

When `shortday_prompt_day.read() == Some(day)`, replace the dropdown trigger for that day column with an inline `<input type="time">` + Confirm/Cancel buttons. After confirm: `spawn` `create_special_day` with `ShortDay` + time, then reset signal + reload.

---

### `shifty-dioxus/src/i18n/mod.rs` — add 17 new Keys

**Analog:** same file, existing `Key` enum entries after `SettingsHolidayAutoCreditUnsetHint`

**17 new variants to append to `Key` enum:**
```rust
SettingsSpecialDaysSectionLabel,
SettingsSpecialDaysSectionDescription,
SettingsSpecialDaysYearLabel,
SettingsSpecialDaysDateLabel,
SettingsSpecialDaysTypeLabel,
SettingsSpecialDaysTypeHoliday,
SettingsSpecialDaysTypeShortDay,
SettingsSpecialDaysTimeLabel,
SettingsSpecialDaysAddBtn,
SettingsSpecialDaysEmptyBody,
SettingsSpecialDaysDuplicateHint,
SettingsSpecialDaysDeleteBtn,
SettingsSpecialDaysDeleteError,
SettingsSpecialDaysCalendarWeekAbbr,
ShiftplanDayTypeHoliday,
ShiftplanDayTypeShortDay,
ShiftplanDayTypeNone,
ShiftplanDayShortDayConfirm,
```

The `i18n` completeness test (already existing) will fail at compile time if any key is missing from any locale file — use that as the gate.

---

### `shifty-dioxus/src/i18n/{de,en,cs}.rs` — add 17 translation entries

**Analog:** same locale files, entries following `SettingsHolidayAutoCreditUnsetHint`

**Pattern (copy existing entry structure, e.g., `Key::SettingsHolidayAutoCreditSave`):**
```rust
Key::SettingsSpecialDaysSectionLabel => "Feiertage & Sondertage",   // de
Key::SettingsSpecialDaysSectionLabel => "Holidays & Special Days",   // en
Key::SettingsSpecialDaysSectionLabel => "Svátky a zvláštní dny",    // cs
```

All 17 keys × 3 locales = 51 translation entries. Exact text is Claude's Discretion (per CONTEXT.md).

---

## Shared Patterns

### shiftplanner Permission Gate
**Source:** `shifty-dioxus/src/page/shiftplan.rs` lines 102–105; `service_impl/src/special_days.rs` lines 77–78
**Apply to:** Settings Card-3 guard, Shiftplan dropdown guard (both FE); `create`/`delete` service methods (BE, already in place)

FE pattern:
```rust
let is_shiftplanner = AUTH
    .read()
    .auth_info
    .as_ref()
    .map(|a| a.has_privilege("shiftplanner"))
    .unwrap_or(false);
```

BE pattern (already in `service_impl/src/special_days.rs` lines 77–78):
```rust
self.permission_service
    .check_permission(SHIFTPLANNER_PRIVILEGE, context)
    .await?;
```

### Date→ISO-Woche Mapping
**Source:** `shifty-dioxus/src/page/settings.rs` lines 135–136 (`time::Date::parse`) + RESEARCH.md Pattern 5
**Apply to:** Settings Card-3 create-form on submit

```rust
let date_format = time::macros::format_description!("[year]-[month]-[day]");
if let Ok(date) = time::Date::parse(&date_str, date_format) {
    let (iso_year, iso_week, weekday) = date.to_iso_week_date();
    // weekday: time::Weekday → DayOfWeek (shifty-utils) → DayOfWeekTO
}
```

### spawn + saving-Guard + inline-feedback
**Source:** `shifty-dioxus/src/page/settings.rs` lines 57–80 (Card-1) and 125–155 (Card-2)
**Apply to:** All FE mutating actions (create_special_day, delete_special_day)

```rust
if *saving.read() { return; }
saving.set(true);
save_result.set(None);
let cfg = config.clone();
spawn(async move {
    match action(cfg, ...).await {
        Ok(_) => { save_result.set(Some(true)); resource.restart(); }
        Err(_) => { save_result.set(Some(false)); }
    }
    saving.set(false);
});
```

### error_handler REST wrapper
**Source:** `rest/src/special_day.rs` lines 45–62
**Apply to:** `get_special_days_for_year` handler

```rust
error_handler(
    (async {
        // ... build result
        Ok(Response::builder().status(200).body(...).unwrap())
    })
    .await,
)
```

### nil-id/version guard on POST
**Source:** `shifty-dioxus/src/api.rs` lines 613–614; `service_impl/src/special_days.rs` lines 84–89
**Apply to:** `create_special_day` in api.rs (FE defensive) + already enforced in service (BE)

```rust
body.id = Uuid::nil();
body.version = Uuid::nil();
```

### use_resource + use_effect load pattern
**Source:** `shifty-dioxus/src/page/settings.rs` lines 107–122 (Card-2)
**Apply to:** Special-days year list in Card-3

```rust
let resource = use_resource(move || loader_fn(config.clone(), param));
use_effect(move || {
    if let Some(Ok(value)) = &*resource.read_unchecked() {
        signal.set(value.clone());
    }
});
```

---

## No Analog Found

All files have close analogs. No entries in this section.

---

## Critical Notes for Planner

1. **`#[utoipa::path]` + ApiDoc both required** — must update both `paths(...)` macro in `SpecialDayApiDoc` AND add `#[utoipa::path]` on the new handler function. Missing either silently breaks Swagger UI.

2. **`DropdownBase` hides `disabled: true` entries** — `dropdown_base.rs:52` filters them out entirely. "Nichts"-option with `disabled: true` is invisible, not greyed. Design accordingly.

3. **`TextInput.on_change` uses `oninput` internally** — WASM-safe for date/time inputs. No extra workaround needed (verified in `inputs.rs:54`).

4. **`#[automock]` on DAO + Service traits** — adding `find_by_year` / `get_by_year` to the traits auto-generates mock methods. No manual mock code required.

5. **Clippy gate** — `cargo clippy --workspace -- -D warnings` must pass before every jj commit. `cargo test` alone does not run clippy.

6. **DayOfWeekTO → time::Weekday conversion** — check `rest-types/src/lib.rs` around `DayOfWeekTO` for existing `From` impls before writing a manual match. The conversion chain is `DayOfWeekTO` → `DayOfWeek` (shifty-utils) → `time::Weekday`.

---

## Metadata

**Analog search scope:** `dao/`, `dao_impl_sqlite/`, `service/`, `service_impl/`, `rest/`, `shifty-dioxus/src/`
**Files scanned:** 12 source files read directly
**Pattern extraction date:** 2026-06-30
