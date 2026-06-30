# Phase 35: Slot-Werte nur für eine Woche ändern - Pattern Map

**Mapped:** 2026-06-30
**Files analyzed:** 12
**Analogs found:** 12 / 12

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `service_impl/src/shiftplan_edit.rs` | service | CRUD + transaction | same file (modify_slot lines 51-143) | exact |
| `service/src/shiftplan_edit.rs` | service-trait | request-response | same file (ShiftplanEditService trait) | exact |
| `rest/src/shiftplan_edit.rs` | controller | request-response | same file (edit_slot handler lines 42-64) | exact |
| `service_impl/src/test/shiftplan_edit.rs` | test | CRUD | same file (test_modify_slot_carries_max_paid_employees lines 1157-1209) | exact |
| `shifty-dioxus/src/state/slot_edit.rs` | state/model | — | same file (SlotEdit struct lines 88-114) | exact |
| `shifty-dioxus/src/service/slot_edit.rs` | service | request-response | same file (save_slot_edit lines 52-74) | exact |
| `shifty-dioxus/src/component/slot_edit.rs` | component | request-response | same file (SlotEditInner lines 48-270) | exact |
| `shifty-dioxus/src/api.rs` | utility | request-response | same file (update_slot lines 153-165) | exact |
| `shifty-dioxus/src/loader.rs` | utility | request-response | same file (save_slot lines 705-713) | exact |
| `shifty-dioxus/src/i18n/mod.rs` | config | — | same file (SlotEdit* keys lines 218-227) | exact |
| `shifty-dioxus/src/i18n/de.rs` | config | — | existing slot-edit translation block | exact |
| `shifty-dioxus/src/i18n/{en,cs}.rs` | config | — | existing slot-edit translation blocks | exact |

---

## Pattern Assignments

### `service_impl/src/shiftplan_edit.rs` — new `modify_slot_single_week` method

**Analog:** same file, `modify_slot` (lines 51-143) and `remove_slot` (lines 145-197)

**Imports pattern** (lines 1-24) — no new imports needed; all in scope already:
```rust
use service::{
    permission::{Authentication, SHIFTPLANNER_PRIVILEGE},
    shiftplan_edit::{BookingCreateResult, CopyWeekResult, ShiftplanEditService},
    slot::{Slot, SlotService},
    ServiceError,
};
use uuid::Uuid;
// time crate already in scope (used in modify_slot:78)
```

**Auth + transaction bracket** (lines 59-62) — copy verbatim:
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
self.permission_service
    .check_permission("shiftplan.edit", context)
    .await?;
```

**Version conflict check** (lines 64-75) — copy verbatim:
```rust
let mut stored_slot = self
    .slot_service
    .get_slot(&slot.id, Authentication::Full, tx.clone().into())
    .await?;

if stored_slot.version != slot.version {
    return Err(ServiceError::EntityConflicts(
        slot.id,
        stored_slot.version,
        slot.version,
    ));
}
```

**Date arithmetic** (lines 77-79) — extend with two new lines:
```rust
let new_slot_valid_from =
    time::Date::from_iso_week_date(change_year as i32, change_week, time::Weekday::Monday)?;
let old_slot_valid_to = new_slot_valid_from - time::Duration::days(1); // Sunday KW-1
// NEW for single-week:
let seg2_valid_to   = new_slot_valid_from + time::Duration::days(6);   // Sunday KW
let seg3_valid_from = new_slot_valid_from + time::Duration::days(7);   // Monday KW+1
```

**Bookings + snapshot capture** (lines 80-90) — add snapshot BEFORE any mutation:
```rust
let bookings = self
    .booking_service
    .get_for_slot_id_since(slot.id, change_year, change_week, Authentication::Full, Some(tx.clone()))
    .await?;
let original_valid_to = stored_slot.valid_to;
let original_snapshot = stored_slot.clone(); // NEW: must be before any mutation of stored_slot
```

**Segment 1 shrink** (lines 92-102) — copy verbatim:
```rust
stored_slot.valid_to = Some(old_slot_valid_to);
if stored_slot.valid_to.unwrap() < stored_slot.valid_from {
    self.slot_service
        .delete_slot(&stored_slot.id, Authentication::Full, tx.clone().into())
        .await?;
} else {
    self.slot_service
        .update_slot(&stored_slot, Authentication::Full, tx.clone().into())
        .await?;
}
```

**Segment 2 create** (lines 104-117) — modified: `valid_to = Some(seg2_valid_to)`, NOT `original_valid_to`:
```rust
let mut seg2 = stored_slot;
seg2.valid_from = new_slot_valid_from;
seg2.valid_to = Some(seg2_valid_to);   // CLOSED at Sunday KW (differs from modify_slot)
seg2.id = Uuid::nil();
seg2.version = Uuid::nil();
seg2.min_resources = slot.min_resources;
seg2.max_paid_employees = slot.max_paid_employees;
seg2.from = slot.from;
seg2.to = slot.to;
let seg2_slot = self
    .slot_service
    .create_slot(&seg2, Authentication::Full, tx.clone().into())
    .await?;
```

**Segment 3 create** — NEW (no analog in modify_slot):
```rust
let mut seg3 = original_snapshot;
seg3.valid_from = seg3_valid_from;
seg3.valid_to = original_valid_to;   // original bound (None = unbounded)
seg3.id = Uuid::nil();
seg3.version = Uuid::nil();
// seg3.min_resources, .max_paid_employees, .from, .to stay from original_snapshot
let seg3_slot = self
    .slot_service
    .create_slot(&seg3, Authentication::Full, tx.clone().into())
    .await?;
```

**Booking re-point loop** (lines 119-139) — split by week instead of routing all to one slot:
```rust
for booking in bookings.iter() {
    self.booking_service
        .delete(booking.id, Authentication::Full, tx.clone().into())
        .await?;

    // SPLIT: exception week → seg2, subsequent weeks → seg3
    let target_slot_id = if booking.year == change_year
        && booking.calendar_week == change_week as i32
    {
        seg2_slot.id
    } else {
        seg3_slot.id
    };

    let mut new_booking = booking.clone();
    new_booking.id = Uuid::nil();
    new_booking.version = Uuid::nil();
    new_booking.slot_id = target_slot_id;
    new_booking.year = booking.year;
    new_booking.calendar_week = booking.calendar_week;
    new_booking.created = None;
    new_booking.created_by = None;
    self.booking_service
        .create(&new_booking, Authentication::Full, tx.clone().into())
        .await?;
}
```

**Single commit** (line 141) — copy verbatim:
```rust
self.transaction_dao.commit(tx).await?;
Ok(seg2_slot)   // return the exception-week slot (the edit target)
```

---

### `service/src/shiftplan_edit.rs` — add trait method

**Analog:** `modify_slot` trait signature (lines 41-48)

**New method to add inside `#[automock]` trait** (copy modify_slot signature, change name):
```rust
async fn modify_slot_single_week(
    &self,
    slot: &Slot,
    change_year: u32,
    change_week: u8,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Slot, ServiceError>;
```

The `#[automock(...)]` attribute on the trait (line 35) automatically generates `MockShiftplanEditService::expect_modify_slot_single_week()` — no extra annotation needed.

---

### `rest/src/shiftplan_edit.rs` — new handler + route

**Analog:** `edit_slot` handler (lines 42-64) and route registration (lines 22-26)

**Route registration** (add to `generate_route`, lines 20-40):
```rust
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/slot/{year}/{week}", put(edit_slot::<RestState>))
        .route("/slot/{year}/{week}/single-week", put(edit_slot_single_week::<RestState>))  // NEW
        .route("/slot/{slot_id}/{year}/{week}", delete(delete_slot::<RestState>))
        // ... existing routes unchanged ...
}
```

**Handler** (copy edit_slot exactly, change method call):
```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/slot/{year}/{week}/single-week",
    tags = ["ShiftplanEdit"],
    params(
        ("year" = u32, Path, description = "Year"),
        ("week" = u8, Path, description = "ISO calendar week"),
    ),
    request_body = SlotTO,
    responses(
        (status = 200, description = "Slot modified for single week", body = SlotTO),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Version conflict"),
    ),
)]
pub async fn edit_slot_single_week<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((year, week)): Path<(u32, u8)>,
    Json(slot): Json<SlotTO>,
) -> Response {
    error_handler(
        (async {
            let slot = SlotTO::from(
                &rest_state
                    .shiftplan_edit_service()
                    .modify_slot_single_week(&(&slot).into(), year, week, context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&slot).unwrap()))
                .unwrap())
        })
        .await,
    )
}
```

Add `edit_slot_single_week` to `ShiftplanEditApiDoc` paths (lines 210-229).

---

### `service_impl/src/test/shiftplan_edit.rs` — D-35-05 tests

**Analog:** `test_modify_slot_carries_max_paid_employees` (lines 1157-1209) + DI scaffold (lines 150-295)

**DI scaffold** (lines 207-295) — use `build_dependencies(true, true)` then override per test:
```rust
// Pattern: build_dependencies sets up defaults; override only what the test needs
let mut deps = build_dependencies(true, true);

// Override slot_service for Segment-1 path:
deps.slot_service.checkpoint();   // clear default get_slot mock if needed
deps.slot_service
    .expect_get_slot()
    .returning(|_, _, _| Ok(monday_slot()));   // stored slot with known version

// Override to expect update_slot (Segment 1 shrink, not delete):
deps.slot_service
    .expect_update_slot()
    .returning(|_, _, _| Ok(()));

// Provide bookings for the test scenario:
deps.booking_service
    .expect_get_for_slot_id_since()
    .returning(|_, _, _, _, _| Ok(Arc::from(vec![booking_in_week_26(), booking_in_week_27()])));

// Expect delete for each re-pointed booking:
deps.booking_service
    .expect_delete()
    .times(2)
    .returning(|_, _, _| Ok(()));

// Assert target slot_id via create mock:
let mut seg2_id = Uuid::nil();
deps.slot_service
    .expect_create_slot()
    .returning(|slot, _, _| {
        let mut result = slot.clone();
        result.id = uuid!("AAAAAAAA-0000-0000-0000-000000000001");
        Ok(result)
    });
```

**Test structure** (copy from line 1157):
```rust
#[tokio::test]
async fn test_modify_slot_single_week_bookings_split_correctly() {
    let mut deps = build_dependencies(true, true);
    // ... mock setup ...
    let service = deps.build_service();
    let input = Slot { ..monday_slot() };
    let result = service
        .modify_slot_single_week(&input, 2026, 26, ().auth(), None)
        .await
        .expect("modify_slot_single_week should succeed");
    // Assertions on segment structure and booking targets
}
```

**Key: NoneTypeExt for auth** (line 47 — already imported):
```rust
use crate::test::error_test::{test_forbidden, NoneTypeExt};
// Usage: ().auth()  →  Authentication::Full with () context
```

---

### `shifty-dioxus/src/state/slot_edit.rs` — add `single_week` field

**Analog:** existing `SlotEdit` struct (lines 88-114) and `current_paid_count` field (added in Phase 23)

**Add to `SlotEdit`** (after `current_paid_count` field, line 100):
```rust
pub struct SlotEdit {
    pub slot_edit_type: SlotEditType,
    pub slot: Rc<SlotEditItem>,
    pub visible: bool,
    pub year: u32,
    pub week: u8,
    pub has_errors: bool,
    pub current_paid_count: u8,  // existing
    pub single_week: bool,       // NEW: "only this week" mode (false = "from this week")
}
```

**Update `SlotEdit::new_edit()`** (lines 102-113) — add `single_week: false`:
```rust
impl SlotEdit {
    pub fn new_edit() -> Self {
        SlotEdit {
            // ... existing fields ...
            current_paid_count: 0,
            single_week: false,   // NEW: default to "from this week" (existing behavior)
        }
    }
}
```

---

### `shifty-dioxus/src/service/slot_edit.rs` — route SaveSlot to correct API

**Analog:** `save_slot_edit` function (lines 52-74) and `SlotEditAction` enum (lines 19-26)

**Add action variant** (line 19):
```rust
pub enum SlotEditAction {
    NewSlot(u32, u8, Option<Uuid>),
    UpdateSlot(SlotEditItem),
    SaveSlot,
    Cancel,
    DeleteSlot(Uuid, u32, u8),
    LoadSlot(Uuid, u32, u8, u8),
    SetSingleWeek(bool),  // NEW
}
```

**Modify `save_slot_edit`** (lines 52-74):
```rust
pub async fn save_slot_edit() -> Result<(), ShiftyError> {
    let mut store = SLOT_EDIT_STORE.write();
    match store.slot_edit_type {
        SlotEditType::Edit => {
            if store.single_week {
                // NEW: calls PUT /shiftplan-edit/slot/{year}/{week}/single-week
                loader::save_slot_single_week(
                    CONFIG.read().clone(),
                    store.slot.clone(),
                    store.year,
                    store.week,
                )
                .await?;
            } else {
                // EXISTING: PUT /shiftplan-edit/slot/{year}/{week}
                loader::save_slot(
                    CONFIG.read().clone(),
                    store.slot.clone(),
                    store.year,
                    store.week,
                )
                .await?;
            }
        }
        SlotEditType::New => {
            if !loader::create_slot(CONFIG.read().clone(), store.slot.clone()).await? {
                store.has_errors = true;
                return Ok(());
            }
        }
    }
    store.visible = false;
    trigger_shiftplan_refresh();
    Ok(())
}
```

**Handle new action in dispatcher** (lines 106-128):
```rust
SlotEditAction::SetSingleWeek(val) => {
    SLOT_EDIT_STORE.write().single_week = val;
    Ok(())
}
```

---

### `shifty-dioxus/src/component/slot_edit.rs` — mode toggle UI

**Analog:** existing form fields in `SlotEditInner` (lines 48-270); radio group pattern similar to weekday SelectInput (lines 154-175); `time_disabled` bool logic (line 121)

**Add to `SlotEditProps`** (lines 23-38):
```rust
pub struct SlotEditProps {
    // ... existing fields ...
    pub single_week: bool,           // NEW
    pub on_set_single_week: EventHandler<bool>,  // NEW
}
```

**i18n bindings** (add after line 95):
```rust
let only_this_week_label: ImStr = i18n.t(Key::SlotEditOnlyThisWeek).as_ref().into();
let from_this_week_label: ImStr = i18n.t(Key::SlotEditFromThisWeek).as_ref().into();
```

**Radio toggle RSX** (add inside the `div { class: "flex flex-col gap-3"` block, visible only in Edit mode):
```rust
if props.slot_edit_type == SlotEditType::Edit {
    div { class: "flex flex-col gap-1",
        label { class: "flex items-center gap-2 cursor-pointer",
            input {
                r#type: "radio",
                name: "slot_edit_mode",
                checked: !props.single_week,
                onchange: {
                    let h = props.on_set_single_week.clone();
                    move |_| h.call(false)
                },
            }
            "{from_this_week_label}"
        }
        label { class: "flex items-center gap-2 cursor-pointer",
            input {
                r#type: "radio",
                name: "slot_edit_mode",
                checked: props.single_week,
                onchange: {
                    let h = props.on_set_single_week.clone();
                    move |_| h.call(true)
                },
            }
            "{only_this_week_label}"
        }
    }
}
```

**Wire in `SlotEdit` wrapper** (lines 272-290):
```rust
#[component]
pub fn SlotEdit() -> Element {
    let slot_edit = SLOT_EDIT_STORE.read().to_owned();
    let slot_service = use_coroutine_handle::<SlotEditAction>();
    rsx! {
        SlotEditInner {
            // ... existing props ...
            single_week: slot_edit.single_week,
            on_set_single_week: move |val| slot_service.send(SlotEditAction::SetSingleWeek(val)),
        }
    }
}
```

**SSR test pattern** (lines 292-448) — add test asserting radio renders:
```rust
#[test]
fn slot_edit_shows_mode_toggle_in_edit_mode() {
    fn app() -> Element {
        pin_de_locale();
        rsx! { SlotEditInner { ..props_with(None, 0) } }
    }
    let html = render(app);
    assert!(html.contains("nur diese Woche") || html.contains("only this week"),
        "mode toggle must be present in edit mode: {html}");
}
```

---

### `shifty-dioxus/src/api.rs` — new `update_slot_single_week`

**Analog:** `update_slot` (lines 153-165) — copy exactly, change URL:
```rust
pub async fn update_slot_single_week(
    config: Config,
    slot: SlotTO,
    year: u32,
    week: u8,
) -> Result<(), reqwest::Error> {
    let url = format!(
        "{}/shiftplan-edit/slot/{}/{}/single-week",
        config.backend, year, week
    );
    let client = reqwest::Client::new();
    let response = client.put(url).json(&slot).send().await?;
    response.error_for_status_ref()?;
    info!("Updated slot (single week)");
    Ok(())
}
```

---

### `shifty-dioxus/src/loader.rs` — new `save_slot_single_week`

**Analog:** `save_slot` (lines 705-713) — copy exactly, change API call:
```rust
pub async fn save_slot_single_week(
    config: Config,
    slot: Rc<SlotEditItem>,
    year: u32,
    week: u8,
) -> Result<(), ShiftyError> {
    api::update_slot_single_week(config, slot.as_ref().into(), year, week).await?;
    Ok(())
}
```

---

### `shifty-dioxus/src/i18n/mod.rs` — new Key variants

**Analog:** slot-edit key block (lines 218-227):
```rust
// Add to the Key enum, in the Slot edit section:
SlotEditOnlyThisWeek,   // "nur diese Woche" / "only this week" / "jen tento týden"
SlotEditFromThisWeek,   // "ab dieser Woche" / "from this week" / "od tohoto týdne"
```

---

### `shifty-dioxus/src/i18n/{de,en,cs}.rs` — translations

**Analog:** existing slot-edit translation block (de.rs lines 322-338 per RESEARCH.md sources)

Pattern for each locale — add matching arms in the `t` match:
```rust
// de.rs
Key::SlotEditOnlyThisWeek => "nur diese Woche",
Key::SlotEditFromThisWeek  => "ab dieser Woche",

// en.rs
Key::SlotEditOnlyThisWeek => "only this week",
Key::SlotEditFromThisWeek  => "from this week",

// cs.rs
Key::SlotEditOnlyThisWeek => "jen tento týden",
Key::SlotEditFromThisWeek  => "od tohoto týdne",
```

---

## Shared Patterns

### Transaction bracket
**Source:** `service_impl/src/shiftplan_edit.rs` lines 59, 141
**Apply to:** `modify_slot_single_week` implementation
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
// ... ALL operations pass tx.clone() ...
self.transaction_dao.commit(tx).await?;
```
Rule: one `use_transaction`, one `commit`, no intermediate commits.

### Permission gate
**Source:** `service_impl/src/shiftplan_edit.rs` lines 60-62
**Apply to:** `modify_slot_single_week` — must be first call after `use_transaction`:
```rust
self.permission_service
    .check_permission("shiftplan.edit", context)
    .await?;
```

### `error_handler` REST wrapper
**Source:** `rest/src/shiftplan_edit.rs` lines 49-63 (edit_slot)
**Apply to:** `edit_slot_single_week` — always wrap async block with `error_handler(...)`:
```rust
error_handler(
    (async { /* ... */ Ok(Response::builder().status(200)...) }).await,
)
```

### Soft-delete booking re-point
**Source:** `service_impl/src/shiftplan_edit.rs` lines 119-139
**Apply to:** booking loop in `modify_slot_single_week`
- `booking_service.delete(id, ...)` → soft-delete (sets `deleted`; DAO filters with `AND deleted IS NULL`)
- `booking_service.create(&new_booking, ...)` → new row with `id = Uuid::nil()`, `version = Uuid::nil()`, `created_by = None`
- No hard-delete; double-count impossible by construction.

### Dioxus SSR test pattern
**Source:** `shifty-dioxus/src/component/slot_edit.rs` lines 292-448
**Apply to:** new mode-toggle tests
```rust
fn render(comp: fn() -> Element) -> String {
    let mut vdom = VirtualDom::new(comp);
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}
fn pin_de_locale() {
    use_hook(|| { *I18N.write() = generate(Locale::De); });
}
```

### OpenAPI annotation
**Source:** `rest/src/shiftplan_edit.rs` lines 122-134 (`book_slot_with_conflict_check` example)
**Apply to:** `edit_slot_single_week` handler — add `#[utoipa::path(...)]` and add to `ShiftplanEditApiDoc`

### i18n three-locale gate
**Source:** `shifty-dioxus/src/i18n/mod.rs` test at line 1283 (`i18n_slot_paid_capacity_keys_present_in_all_locales`)
**Apply to:** new keys `SlotEditOnlyThisWeek` / `SlotEditFromThisWeek` — verify all three locales in test or manually confirm all arms are present to avoid compile error.

---

## Critical Pitfalls (from RESEARCH.md — reproduce in plan actions)

1. **Snapshot before mutation:** `let original_snapshot = stored_slot.clone()` MUST precede `stored_slot.valid_to = Some(old_slot_valid_to)` (line 92). If after, Segment 3 inherits wrong `valid_to`.
2. **Segment 2 closed:** `seg2.valid_to = Some(seg2_valid_to)` — NOT `original_valid_to`. If unbounded, Segment 3 overlaps and `create_slot` conflict-checks will fail.
3. **Type cast for booking partition:** `booking.calendar_week == change_week as i32` — `calendar_week` is `i32`, `change_week` is `u8`.
4. **Clippy gate:** Run `cargo clippy --workspace -- -D warnings` before every commit. `modify_slot_single_week` is a separate method so no unused-variable risk from the clone.
5. **No `cargo sqlx prepare`:** Zero new SQL queries — all calls reuse existing service methods.

---

## No Analog Found

All files have close analogs. No new data models, no new DAO methods, no new SQL.

---

## Metadata

**Analog search scope:** `service_impl/src/`, `service/src/`, `rest/src/`, `shifty-dioxus/src/`
**Files scanned:** 12 primary + referenced support files
**Pattern extraction date:** 2026-06-30
