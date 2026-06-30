# Phase 23: Frontend — Slot Paid-Capacity UI - Pattern Map

**Mapped:** 2026-06-26
**Files analyzed:** 8 modified (0 new files)
**Analogs found:** 8 / 8 (every change is an in-file extension of an existing analog)

> This is a pure **frontend** phase in `shifty-dioxus` (Dioxus 0.6 / Rust-WASM). No backend / DTO change.
> Every "analog" here is an *in-place* extension of the file being modified — the closest pattern to copy
> always lives in the same file (or its sibling). All line numbers verified against source on 2026-06-26.
> Paths are relative to `shifty-backend/shifty-dioxus/` unless noted.

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog (same-file unless noted) | Match Quality |
|---------------|------|-----------|------------------------------------------|---------------|
| `src/component/slot_edit.rs` (`SlotEditInner`) | component (form) | request-response (edit form) | `min_resources` Field `:185-202` + `has_errors` banner `:204-206` (same file) | exact |
| `src/component/week_view.rs` (`cell_background_class` + `WeekCellSlot`) | component (decision fn) | transform (state→class) | `cell_background_class` `:955-968`, call site `:1037` (same file) | exact |
| `src/state/slot_edit.rs` (`SlotEdit`) | state (store struct) | transform (dialog container) | `SlotEdit` struct `:88-108`; Pitfall 2 (NOT `SlotEditItem`) | exact |
| `src/service/slot_edit.rs` (`SlotEditAction::LoadSlot` + `load_slot_edit`) | service (coroutine) | event-driven (action dispatch) | `LoadSlot` enum arm `:25` + handler `:87-97` + dispatch `:109` | exact |
| `src/page/shiftplan.rs` (dropdown closure) | page (composition) | event-driven (UI → action) | "Edit slot" closure `:656-667`; `shift_plan_context` `:157`, read `:448`/`:1039` | exact |
| `src/i18n/mod.rs` (`Key` enum + coverage test) | config (i18n) | static lookup | `BookingWarningPaidLimitExceeded` `:565`; coverage test `:1150-1175` | exact |
| `src/i18n/{en,de,cs}.rs` | config (i18n) | static lookup | `BookingWarningPaidLimitExceeded` add_text (`de.rs:979-983`) | exact |
| SSR test modules (in `slot_edit.rs` + `week_view.rs`) | test | request-response (render assert) | `warning_list.rs:178-285` canonical SSR harness | role-match (cross-file template) |

---

## Pattern Assignments

### `src/component/slot_edit.rs` — `SlotEditInner` (component, form)

**Analog:** same file. Three sub-patterns to copy.

**(A) Props struct — add `current_paid_count`** (`:23-35`):
```rust
#[derive(Clone, PartialEq, Debug, Props)]
pub struct SlotEditProps {
    pub visible: bool,
    pub slot: Rc<SlotEditItem>,
    pub slot_edit_type: SlotEditType,
    pub year: u32,
    pub week: u8,
    pub has_errors: bool,
    // ADD: pub current_paid_count: u8,
    pub on_save: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
    pub on_update_slot: EventHandler<SlotEditItem>,
}
```

**(B) Core pattern — `min_resources` Field input (template for new `max_paid_employees` Field)** (`:185-202`, VERIFIED verbatim):
```rust
Field { label: min_persons_label.clone(),
    input {
        class: FORM_INPUT_CLASSES,
        r#type: "number",
        min: "0",
        value: "{min_resources_value}",
        oninput: {
            let slot = props.slot.clone();
            move |event: Event<FormData>| {
                if let Ok(value) = event.value().parse::<i32>() {
                    let mut updated = slot.as_ref().clone();
                    updated.min_resources = value as u8;
                    props.on_update_slot.call(updated);
                }
            }
        },
    }
}
```
**Adaptation for `Option<u8>`** (the ONLY structural difference): empty string → `None`, parseable digit → `Some(u8)`, parse-failure → silently ignore (no update). Value display: `props.slot.max_paid_employees.map(|n| n.to_string()).unwrap_or_default()` (mirror of `min_resources_value` at `:89`). Field gets a `hint: Some(...)` prop (see Shared Pattern: Field hint).

**(C) Inline banner — established `has_errors` banner pattern** (`:204-206`, VERIFIED verbatim):
```rust
if props.has_errors {
    p { class: "text-bad text-small font-normal", "{error_str}" }
}
```
The D-23-02 overage banner is the SAME structural pattern (conditional `div`/`p` after the Field). Per UI-SPEC the banner uses `border-l-[3px] border-warn bg-warn-soft rounded-md p-2.5 text-body text-ink` (warn, not bad — it is non-blocking) and is placed AFTER the new `max_paid_employees` Field, BEFORE the `has_errors` paragraph. Visibility: `props.slot.max_paid_employees.map_or(false, |n| props.current_paid_count > n)`.

**i18n / interpolation reference** (`:67-74`, VERIFIED — `t_m_rc` placeholder pattern):
```rust
let explanation_str = i18n.t_m_rc(
    Key::SlotEditExplanation,
    [
        ("year", props.year.to_string().into()),
        ("week", props.week.to_string().into()),
    ]
    .into(),
);
```
Use the same shape for `MaxPaidEmployeesOverageHint` with `("current", …)` and `("limit", …)`.

**Wrapper plumbing** (`SlotEdit`, `:212-229`): the wrapper reads `SLOT_EDIT_STORE` and forwards each field as a prop. Add `current_paid_count: slot_edit.current_paid_count` to the `SlotEditInner { … }` call (mirrors `year: slot_edit.year`).

**Existing legacy-class guard test** (`:255-277`) scans non-test source for forbidden classes (`bg-red-`, `text-red-`, …). New code must use semantic tokens only (`bg-warn-soft`, `text-bad`) — never literal palette classes — or this existing test fails.

---

### `src/component/week_view.rs` — `cell_background_class` + `WeekCellSlot` (component, transform)

**Analog:** same file.

**Decision function to extend** (`:955-968`, VERIFIED verbatim):
```rust
/// Returns the per-cell state-background class.
///
/// `discourage` (the editing person is unavailable on this day) takes priority
/// over `missing` and tints the cell `bad`. Missing-staff cells without
/// discourage tint `warn`.
pub(crate) fn cell_background_class(missing: bool, discourage: bool) -> &'static str {
    if discourage {
        "bg-bad-soft"
    } else if missing {
        "bg-warn-soft"
    } else {
        ""
    }
}
```
**Extension (D-23-03/D-23-04):** add `paid_overage: bool` as a third param; insert its branch BELOW `discourage`, ABOVE `missing`, returning `"bg-bad-soft"` (static literal — Pitfall 5). Update the doc comment to mention paid-overage precedence.

**Call site to update** (`:1034-1037`, VERIFIED verbatim):
```rust
let filled = slot.bookings.len();
let need = slot.min_resources as usize;
let missing = filled < need;
let bg_class = cell_background_class(missing, props.discourage);
```
**After:** compute `let paid_overage = slot.max_paid_employees.map_or(false, |n| slot.current_paid_count > n);` (UNCONDITIONAL — NOT gated on `props.is_shiftplanner`, D-23-05) and pass it as the third arg. `slot.max_paid_employees` (`state/shiftplan.rs:177`) and `slot.current_paid_count` (`:180`) are already populated by the loader.

**Unchanged badge** (`:1069-1073`, VERIFIED) — the `filled/need` `span` stays as-is (D-23-03: no number/badge added):
```rust
span {
    class: format!("font-mono text-small font-bold {}", mr_class),
    style: "position: absolute; top: 6px; left: 8px; pointer-events: none; line-height: 18px;",
    "{filled_str}"
}
```

**Existing unit tests to update** (`week_view.rs:685-713`): the 4 `cell_background_class_*` tests must each gain a `false` third arg (behavior unchanged), then add `paid_overage = true` cases. (RESEARCH.md Pitfall 5.)

---

### `src/state/slot_edit.rs` — `SlotEdit` (state, dialog container)

**Analog:** same file, `SlotEdit` struct (`:88-108`, VERIFIED verbatim):
```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotEdit {
    pub slot_edit_type: SlotEditType,
    pub slot: Rc<SlotEditItem>,
    pub visible: bool,
    pub year: u32,
    pub week: u8,
    pub has_errors: bool,
    // ADD: pub current_paid_count: u8,
}
impl SlotEdit {
    pub fn new_edit() -> Self {
        SlotEdit {
            slot_edit_type: SlotEditType::Edit,
            slot: SlotEditItem::empty().into(),
            visible: false,
            year: 0,
            week: 0,
            has_errors: false,
            // ADD: current_paid_count: 0,
        }
    }
}
```
**CRITICAL (Pitfall 2):** `current_paid_count` goes on `SlotEdit` (dialog container), NEVER on `SlotEditItem` (`:8-23`). `SlotEditItem` is the server-write payload — `From<&SlotEditItem> for SlotTO` at `:64-80`, and `SlotTO` has no `current_paid_count`. The `max_paid_employees` roundtrip is already wired both directions (`From<&SlotTO>` `:60`, `From<&SlotEditItem>` `:77`).

---

### `src/service/slot_edit.rs` — `SlotEditAction::LoadSlot` (service, coroutine)

**Analog:** same file. Three coordinated edits.

**Enum arm** (`:25`, VERIFIED): `LoadSlot(Uuid, u32, u8),` → add a 4th `u8` (`current_paid_count`).

**Handler** (`load_slot_edit`, `:87-97`, VERIFIED verbatim):
```rust
pub async fn load_slot_edit(slot_id: Uuid, year: u32, week: u8) -> Result<(), ShiftyError> {
    let slot = loader::load_slot(CONFIG.read().clone(), slot_id).await?;
    let mut store = SLOT_EDIT_STORE.write();
    store.slot_edit_type = SlotEditType::Edit;
    store.slot = slot.into();
    store.year = year;
    store.week = week;
    store.visible = true;
    store.has_errors = false;
    Ok(())
}
```
Add a `current_paid_count: u8` param; set `store.current_paid_count = current_paid_count;`. (`load_slot` returns `SlotTO` with no count — the count must come from the caller; see page closure below.)

**Dispatch** (`:109`, VERIFIED): `SlotEditAction::LoadSlot(id, year, week) => load_slot_edit(id, year, week).await,` → destructure and forward the 4th arg.

Note: `new_slot_edit` (`:32-43`) sets `store.has_errors = false` etc.; mirror it by setting `store.current_paid_count = 0` for the New path so the struct stays consistent.

---

### `src/page/shiftplan.rs` — "Edit slot" dropdown closure (page, composition)

**Analog:** same file, the existing "Edit slot" entry (`:656-667`, VERIFIED verbatim):
```rust
(
    "Edit slot",
    Box::new(move |slot_id: Option<Rc<str>>| {
        let slot_id: Uuid = slot_id.unwrap().parse().unwrap();
        slot_edit_service.send(SlotEditAction::LoadSlot(
            slot_id,
            *year.read(),
            *week.read(),
        ))
    }),
)
    .into(),
```
**Extension (RESEARCH Pattern 5):** also capture `shift_plan_context` in the closure, look up the slot by `id` in the loaded plan, read its `current_paid_count` (default `0`), and pass it as the 4th `LoadSlot` arg. `shift_plan_context` is the resource at `:157`; it is read elsewhere via `&*shift_plan_context.read_unchecked()` (`:448`, `:1039`) — use `.peek()`/`.read_unchecked()` to avoid adding a reactive subscription inside the closure. The closure must move-capture `shift_plan_context` (it is `Copy` for signal-like resources; verify capture mode against the sibling "Remove slot" closure `:668-678`).

---

### `src/i18n/mod.rs` + `en.rs` / `de.rs` / `cs.rs` (config, i18n)

**Key enum analog** (`mod.rs:565`, VERIFIED): `BookingWarningPaidLimitExceeded,` — append 3 new variants near the enum's end:
`MaxPaidEmployeesLabel`, `MaxPaidEmployeesHint`, `MaxPaidEmployeesOverageHint`.

**add_text analog** (`de.rs:979-983`, VERIFIED verbatim — placeholder-bearing translation):
```rust
i18n.add_text(
    Locale::De,
    Key::BookingWarningPaidLimitExceeded,
    "Bezahlt-Limit überschritten ({current}/{max}).",
);
```
Add equivalents in ALL THREE locale files (Pitfall 3 — German MUST use `Locale::De`, not `Locale::En`). Translations are specified verbatim in UI-SPEC § Copywriting Contract:
- `MaxPaidEmployeesLabel` — De "Max. bezahlte Mitarbeiter" / En "Max paid employees" / Cs "Max. placených zaměstnanců"
- `MaxPaidEmployeesHint` — De "Leer = kein Limit" / En "Empty = no limit" / Cs "Prázdné = bez limitu"
- `MaxPaidEmployeesOverageHint` — De "Aktuell {current} bezahlt (Limit: {limit})" / En "Currently {current} paid ({limit} allowed)" / Cs "Aktuálně {current} placených (limit: {limit})"

**Coverage test analog** (`mod.rs:1150-1175`, VERIFIED — `i18n_booking_warning_keys_present_in_all_locales`):
```rust
for locale in [Locale::En, Locale::De, Locale::Cs] {
    let i18n = generate(locale);
    for key in [ /* … keys … */ ] {
        let value = i18n.t(key);
        assert!(
            !value.is_empty() && value.as_ref() != "??",
            "missing translation for {:?} in {:?}: got `{}`",
            key, locale, value
        );
    }
}
```
Add `i18n_slot_paid_capacity_keys_present_in_all_locales` with the 3 new keys, copying this loop structure verbatim.

---

### SSR test modules (test) — `slot_edit.rs` + `week_view.rs`

**Canonical analog:** `warning_list.rs:178-285` (cross-file template — copy harness verbatim).

**Harness helpers** (`warning_list.rs:178-188`, VERIFIED verbatim):
```rust
fn render(comp: fn() -> Element) -> String {
    let mut vdom = VirtualDom::new(comp);
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

fn pin_de_locale() {
    use_hook(|| {
        *I18N.write() = generate(Locale::De);
    });
}
```

**Assertion style** (`warning_list.rs:209-222`, VERIFIED): `let html = render(app); assert!(html.contains("…"), "… got: {html}");`.

**Pitfall 4 — test `SlotEditInner`, NOT `SlotEdit`:** only the `SlotEdit` wrapper calls `use_coroutine_handle::<SlotEditAction>()` (`slot_edit.rs:215`). SSR-rendering the wrapper panics (no registered coroutine). Construct `SlotEditProps` and render `SlotEditInner` directly. Pin locale via `pin_de_locale()` since `SlotEditInner` reads `I18N` (`:47`).

`dioxus-ssr 0.6` is already a dev-dependency (`Cargo.toml:81`). `slot_edit.rs` currently has a non-SSR `mod tests` (`:231-278`) — extend it with `VirtualDom` + `dioxus_ssr` imports and the two helpers above; `week_view.rs` already has SSR tests (RESEARCH cites `:1368-1371`) to follow.

---

## Shared Patterns

### Field component with `hint` prop
**Source:** `src/component/form/field.rs` (`FieldProps` `:14-29`, `hint: Option<ImStr>` `:19`; render `:49-51`).
**Apply to:** the new `max_paid_employees` Field (D-23-01 "leer = kein Limit" hint).
```rust
// field.rs:49-51 — hint renders below input in muted micro text:
} else if let Some(hint) = props.hint.as_ref() {
    span { class: "text-micro text-ink-muted", "{hint}" }
}
```
Pass `hint: Some(i18n.t(Key::MaxPaidEmployeesHint).as_ref().into())`. The `error` prop preempts `hint` (`:48`/`:119`) — do NOT use the Field `error` prop for the overage banner (would mark the field invalid, conflicting with D-23-02 non-blocking). Use a separate `div` after the Field instead.

### Static Tailwind classes only (Pitfall 5 / project Pitfall)
**Source:** `cell_background_class` (`week_view.rs:955-968`) returns `&'static str` literals; `FORM_INPUT_CLASSES` (`slot_edit.rs:20-21`).
**Apply to:** all class strings in this phase. NEVER `format!("bg-{}", token)`. `bg-bad-soft`, `bg-warn-soft`, `border-warn`, `text-bad`, `text-warn` are already safelisted (`tailwind.config.js`); `bg-bad` is NOT used this phase. Enforced by `slot_edit.rs:255-277` legacy-class guard test (forbids literal palette classes).

### i18n `t_m_rc` placeholder interpolation
**Source:** `slot_edit.rs:67-74` (`Key::SlotEditExplanation`), `week_view.rs:1045-1052` (`Key::ShiftplanFilledOfNeed`).
**Apply to:** `MaxPaidEmployeesOverageHint` banner — `i18n.t_m_rc(Key::…, [("current", ….into()), ("limit", ….into())].into())`.

### SSR test harness
**Source:** `warning_list.rs:178-188`. **Apply to:** all new SSR tests (D-23-06). See SSR test section above.

---

## No Analog Found

None. Every modified file has an exact same-file or sibling analog. No file in this phase requires falling back to RESEARCH.md generic patterns.

---

## Metadata

**Analog search scope:** `shifty-dioxus/src/component/{slot_edit,week_view,warning_list,form/field}.rs`, `src/state/slot_edit.rs`, `src/service/slot_edit.rs`, `src/page/shiftplan.rs`, `src/i18n/{mod,en,de,cs}.rs`.
**Files scanned (read or grepped):** 11.
**Pattern extraction date:** 2026-06-26.
**Line numbers:** all verified against working-tree source on 2026-06-26 (note: `cell_background_class` spans `:955-968` incl. doc comment; the `pub(crate) fn` signature is at `:960`).
