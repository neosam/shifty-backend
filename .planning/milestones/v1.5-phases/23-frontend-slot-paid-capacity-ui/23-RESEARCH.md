# Phase 23: Frontend — Slot Paid-Capacity UI - Research

**Researched:** 2026-06-26
**Domain:** Dioxus 0.6 frontend (Rust/WASM) — slot editor form extension + week-view coloring logic
**Confidence:** HIGH (all findings verified directly in source code)

## Summary

This is a pure frontend phase in `shifty-dioxus`. The backend already delivers all required
fields (`SlotTO.max_paid_employees`, `ShiftplanSlotTO.current_paid_count`, `WarningTO::PaidEmployeeLimitExceeded`)
and the frontend state structs (`state/slot_edit.rs::SlotEditItem`, `state/shiftplan.rs::Slot`) already
mirror both fields end-to-end. The implementation is therefore purely additive UI work on top of an
already-wired data pipeline.

**FUI-02 (Editor):** `SlotEditItem.max_paid_employees: Option<u8>` exists and survives the
`From<&SlotTO>` / `From<&SlotEditItem>` roundtrip without loss. The editor currently ignores the field
at display time — adding one `Field` + `<input type="number">` block after `min_resources` is the
complete work, modelled on the existing `min_resources` input (`slot_edit.rs:185-202`). The only
technical nuance is the `Option<u8>` input pattern (empty → `None`, digit → `Some(u8)`) vs the
existing `parse::<i32>()` for `min_resources`.

For the D-23-02 inline hint (limit < current_paid_count), `current_paid_count` is NOT available
from `api::get_slot` (returns `SlotTO`, no count). It must be injected into `SlotEdit` state from
the shiftplan week state at edit-open time. The clean path: capture `shift_plan_context` in the
dropdown closure, look up the slot by id, and pass `current_paid_count` through `SlotEditAction::LoadSlot`.

**FUI-01 (Week-View coloring):** `cell_background_class(missing, discourage) -> &'static str` is the
single authoritative decision point (`week_view.rs:960-967`). Adding a third parameter
`paid_overage: bool` and inserting it as the highest-priority branch produces a clean, testable
extension. The `bad` and `bad-soft` color tokens are fully configured in both `tailwind.config.js`
and `input.css` (light + dark). `bg-bad-soft` is already safelisted. A new static string `"bg-bad"`
or `"bg-bad-soft"` in the function body is picked up by Tailwind's static scan without safelist entry
(confirmed from the project's `mode: "all"` config and the comment at `tailwind.config.js:1-7`).

**Tests (D-23-06):** The SSR snapshot test pattern with `dioxus_ssr` is well-established in this
codebase. `dioxus-ssr = "0.6"` is a dev-dependency. `SlotEditInner` can be SSR-tested directly (no
coroutine handle required in the pure inner component — only the `SlotEdit` wrapper uses one). I18N
must be pinned in tests via `use_hook(|| { *I18N.write() = generate(Locale::De); })`.

**Primary recommendation:** Implement in two focused tasks — (1) Editor field + inline hint plumbing,
(2) Week-view `cell_background_class` extension — both with SSR tests and 3-locale i18n keys. No new
dependencies needed.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-23-01 (Eingabefeld + NULL-Semantik):** Neues Feld für `max_paid_employees` im
`SlotEditInner`-Formular, eingefügt nach dem bestehenden `min_resources`-Feld
(`slot_edit.rs:185-202` als Vorlage). Leeres Feld = `None` = kein Limit, eine eingegebene
Zahl setzt das Limit (`Some(u8)`). Eigener i18n-Label + dezenter Hinweis „leer = kein Limit"
in allen 3 Locales.

**D-23-02 (Inline-Hinweis bei zu niedrigem Limit, nicht blockierend):** Wenn der eingegebene
`max_paid_employees`-Wert unter dem aktuellen `current_paid_count` liegt, zeigt der Editor ein
nicht-blockierendes Inline-Warn-Banner. Speichern bleibt möglich. Kein Dialog, kein Disable des
Save-Buttons.

**D-23-03 (eigene Farbe, kein Zahlen-Badge):** Bei `max_paid_employees = Some(n)` und
`current_paid_count > n` wird die Slot-Zelle in einer eigenen Farbe Richtung `bad`/rot
eingefärbt. Kein zusätzliches Badge/Zahl in der Zelle. Das bestehende `filled/need`-Badge
bleibt unverändert.

**D-23-04 (Vorrang bei gleichzeitiger Unterbesetzung):** Paid-Überschreitung hat Vorrang über
`warn-soft`-Unterbesetzungs-Hintergrund. Die `cell_background_class`-Logik muss um diesen Fall
erweitert werden, ohne die `discourage`-Behandlung zu brechen.

**D-23-05 (sichtbar für alle Rollen):** Die Warn-Färbung ist nicht auf `is_shiftplanner`
beschränkt — alle Nutzer sehen sie.

**D-23-06:** SSR-Snapshot-Tests für (a) Editor rendert `max_paid_employees`-Feld; (b) Editor
zeigt Inline-Hinweis wenn Limit < current_paid_count; (c) Week-View-Zelle trägt `bad`-Klasse
bei Overage, behält `warn-soft` bei reiner Unterbesetzung. Neue i18n-Keys in De/En/Cs.

### Claude's Discretion

- Konkrete Tailwind-Farbtokens für die Überschreitungs-Färbung (`bad`/`bad-soft` o.ä., statische
  Klassen — Pitfall 5, keine `format!()`-Arms; ggf. Safelist-Eintrag wie bei `warn`-Tokens).
- Genaue Formulierung der Editor-Labels/Hinweise und exakte Platzierung des Inline-Banners.
- Ob die Editor-Validierung (D-23-02) den `current_paid_count` aus dem Slot-State zieht oder
  ein eigenes Prop braucht.

### Deferred Ideas (OUT OF SCOPE)

- Numerische Paid-Count-Anzeige in der Zelle („paid X/Y") — vom User nicht gewünscht.
- Hartes Blockieren/Disablen des Save bei Limit-Verletzung.
</user_constraints>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| max_paid_employees editor input | Frontend (Dioxus component) | — | Pure UI addition to SlotEditInner |
| current_paid_count inline hint | Frontend (Dioxus component) | Frontend state plumbing | Value flows from shiftplan week state into SlotEdit state at open time |
| Week-view overage coloring | Frontend (Dioxus component) | — | Extends cell_background_class decision function |
| i18n keys | Frontend (i18n module) | — | 3-locale translations, Key enum enum |
| SSR tests | Frontend (dev test) | — | dioxus-ssr is already a dev dependency |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| dioxus | 0.6.1 | UI framework (Rust/WASM) | Project-pinned [VERIFIED: Cargo.toml:10] |
| dioxus-ssr | 0.6 | SSR rendering for tests | Dev-dep, established test pattern [VERIFIED: Cargo.toml:81] |
| tailwindcss | (npx, project-local) | Utility CSS | Project standard; run `npx tailwindcss -i ./input.css -o ./assets/tailwind.css` |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rest-types (shared crate) | workspace | DTO types (SlotTO, ShiftplanSlotTO) | Read-only; no changes this phase |

**No new dependencies required.** All needed libraries are already present.

---

## Architecture Patterns

### System Architecture Diagram

```
Editor open flow (FUI-02):
  [dropdown "Edit slot" click]
       ↓ slot_id (Rc<str>)
  [page closure] → look up Slot in shift_plan_context (current_paid_count)
       ↓ SlotEditAction::LoadSlot(slot_id, year, week, current_paid_count)
  [slot_edit service] → api::get_slot → SlotTO → SlotEditItem
       ↓
  [SLOT_EDIT_STORE] (SlotEdit { slot: SlotEditItem, current_paid_count: u8, ... })
       ↓
  [SlotEditInner] → renders Field(max_paid_employees) + optional inline warn banner

Week-view coloring flow (FUI-01):
  [load_shift_plan] → ShiftplanSlotTO.current_paid_count → Slot.current_paid_count
       ↓
  [WeekCellSlot] → paid_overage = slot.max_paid_employees.map_or(false, |n| slot.current_paid_count > n)
       ↓
  [cell_background_class(missing, discourage, paid_overage)] → "bg-bad-soft" / "bg-warn-soft" / ""
```

### Recommended Project Structure

No new files needed. Changes are within:
```
shifty-dioxus/src/
├── component/
│   ├── slot_edit.rs          # ADD max_paid_employees Field, inline hint, SSR tests
│   └── week_view.rs          # EXTEND cell_background_class signature + WeekCellSlot call site
├── state/
│   ├── slot_edit.rs          # ADD current_paid_count: u8 to SlotEdit struct
│   └── slot_edit.rs          # UPDATE new_slot_edit / load_slot_edit accordingly
├── service/
│   └── slot_edit.rs          # UPDATE SlotEditAction::LoadSlot to carry current_paid_count
├── page/
│   └── shiftplan.rs          # UPDATE dropdown closure to capture shift_plan_context and look up slot
└── i18n/
    ├── mod.rs                # ADD 3 new Key variants
    ├── en.rs / de.rs / cs.rs # ADD translations for new keys
```

### Pattern 1: Option<u8> Number Input (empty → None, digit → Some)

The `min_resources` input at `slot_edit.rs:185-202` uses `parse::<i32>()` which ignores empty.
For `Option<u8>`, the input value must be parsed differently:

```rust
// Source: slot_edit.rs:191-199 (existing min_resources pattern, ADAPT for Option<u8>)
oninput: {
    let slot = props.slot.clone();
    move |event: Event<FormData>| {
        let mut updated = slot.as_ref().clone();
        let raw = event.value();
        // Empty string → None (no limit); valid u8 → Some(limit)
        updated.max_paid_employees = if raw.trim().is_empty() {
            None
        } else {
            raw.parse::<u8>().ok().map(Some).flatten()
            // or simply: raw.parse::<u8>().ok()
        };
        props.on_update_slot.call(updated);
    }
},
```

Display value (the `value` attribute): render `""` when `None`, `n.to_string()` when `Some(n)`.

```rust
// Source: slot_edit.rs:89 (existing min_resources_value pattern, ADAPT)
let max_paid_value: String = props.slot.max_paid_employees
    .map(|n| n.to_string())
    .unwrap_or_default();
// use as: value: "{max_paid_value}"
```

### Pattern 2: Field with hint prop

`Field` (`component/form/field.rs`) accepts an optional `hint` prop that renders below the input
in `text-micro text-ink-muted` style. Use it for "leer = kein Limit" hint:

```rust
// Source: form/field.rs (verified — Field has hint: Option<ImStr> prop)
Field {
    label: max_paid_label.clone(),
    hint: Some(i18n.t(Key::MaxPaidEmployeesHint).as_ref().into()),
    input {
        class: FORM_INPUT_CLASSES,
        r#type: "number",
        min: "0",
        value: "{max_paid_value}",
        oninput: { /* ... */ },
    }
}
```

### Pattern 3: Inline warn banner for D-23-02

Not a `Field` error prop (which would visually be inside the Field wrapper) — use a separate
`div` after the Field, consistent with the `has_errors` banner pattern at `slot_edit.rs:204-206`:

```rust
// Source: slot_edit.rs:204-206 (existing has_errors pattern, ADAPT for inline warn)
// Place AFTER the max_paid_employees Field block:
if let (Some(limit), true) = (
    props.slot.max_paid_employees,
    props.slot.max_paid_employees
        .map_or(false, |n| props.current_paid_count > n),
) {
    div { class: "border-l-[3px] border-warn bg-warn-soft rounded-md p-2.5 text-body text-ink",
        "{i18n.t_m_rc(Key::MaxPaidEmployeesOverageHint, [...])}"
    }
}
```

Alternatively, since the `current_paid_count` is read-only info, it can also be delivered through
the `Field`'s `error` prop — but that visually disables/marks the field as invalid which conflicts
with D-23-02 (non-blocking). A separate banner `div` is more consistent with the inline-banner
pattern established in MEMORY.md.

### Pattern 4: cell_background_class extension

Current implementation (`week_view.rs:960-967`):

```rust
// Source: week_view.rs:960-967 [VERIFIED]
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

Extended implementation adding `paid_overage` with precedence over `warn-soft` but BELOW
`discourage` (D-23-04: overage wins over warn-soft understaffing; context does not say it wins
over discourage unavailability):

```rust
// PROPOSED EXTENSION for week_view.rs:960
pub(crate) fn cell_background_class(
    missing: bool,
    discourage: bool,
    paid_overage: bool,
) -> &'static str {
    if discourage {
        "bg-bad-soft"
    } else if paid_overage {
        "bg-bad-soft"   // same shade as discourage; visually distinct from warn-soft
    } else if missing {
        "bg-warn-soft"
    } else {
        ""
    }
}
```

**Note on color token:** Both `discourage` and `paid_overage` map to `"bg-bad-soft"` in the above.
This is intentional: `bg-bad-soft` (light red) is already clearly distinguishable from `bg-warn-soft`
(light orange), satisfying D-23-03's "deutlich andere Farbe als warn-soft". If the planner wants
`paid_overage` to be visually distinct FROM `discourage` as well, use `"bg-bad"` (full dark red) —
but `"bg-bad"` is NOT currently safelisted and is not in use elsewhere. If `"bg-bad"` is chosen,
add `"bg-bad"` to `tailwind.config.js` safelist. The static-string form (returned from a `&'static str`
function) would survive Tailwind purge either way (mode: "all" scans all .rs files), but adding to
safelist is the project's safety convention.

**Call site update in WeekCellSlot (`week_view.rs:1037`):**

```rust
// BEFORE (week_view.rs:1034-1037):
let filled = slot.bookings.len();
let need = slot.min_resources as usize;
let missing = filled < need;
let bg_class = cell_background_class(missing, props.discourage);

// AFTER:
let filled = slot.bookings.len();
let need = slot.min_resources as usize;
let missing = filled < need;
let paid_overage = slot.max_paid_employees
    .map_or(false, |n| slot.current_paid_count > n);
let bg_class = cell_background_class(missing, props.discourage, paid_overage);
```

D-23-05: `paid_overage` does NOT check `props.is_shiftplanner` — computed unconditionally.

### Pattern 5: current_paid_count plumbing into SlotEdit

**Problem:** `load_slot_edit` calls `api::get_slot` which returns `SlotTO` (no `current_paid_count`).
The editor needs `current_paid_count` only for displaying the D-23-02 inline hint — it is never
written back to the server.

**Solution:** Add `current_paid_count: u8` to `state/slot_edit.rs::SlotEdit` (not `SlotEditItem`
which is the server payload):

```rust
// state/slot_edit.rs:89-107 — SlotEdit struct (VERIFIED current content):
pub struct SlotEdit {
    pub slot_edit_type: SlotEditType,
    pub slot: Rc<SlotEditItem>,
    pub visible: bool,
    pub year: u32,
    pub week: u8,
    pub has_errors: bool,
    // ADD:
    pub current_paid_count: u8,
}
```

Update `SlotEditAction::LoadSlot` to carry the count:

```rust
// service/slot_edit.rs:24 (BEFORE):
LoadSlot(Uuid, u32, u8),
// AFTER:
LoadSlot(Uuid, u32, u8, u8),  // (slot_id, year, week, current_paid_count)
```

Update dropdown closure in `page/shiftplan.rs:658` to capture `shift_plan_context` and look up
the slot:

```rust
// page/shiftplan.rs:657-665 (current code):
Box::new(move |slot_id: Option<Rc<str>>| {
    let slot_id: Uuid = slot_id.unwrap().parse().unwrap();
    slot_edit_service.send(SlotEditAction::LoadSlot(
        slot_id, *year.read(), *week.read(),
    ))
}),

// NEW — also capture shift_plan_context:
Box::new(move |slot_id: Option<Rc<str>>| {
    let slot_id: Uuid = slot_id.unwrap().parse().unwrap();
    let current_paid_count = shift_plan_context
        .peek()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .and_then(|plan| plan.slots.iter().find(|s| s.id == slot_id))
        .map(|s| s.current_paid_count)
        .unwrap_or(0);
    slot_edit_service.send(SlotEditAction::LoadSlot(
        slot_id, *year.read(), *week.read(), current_paid_count,
    ))
}),
```

Then in `SlotEditProps` / `SlotEditInner`: add `current_paid_count: u8` prop, read from
`SLOT_EDIT_STORE` in the `SlotEdit` wrapper.

### Pattern 6: SSR test for SlotEditInner

`SlotEditInner` has no `use_coroutine_handle` calls (only the outer `SlotEdit` wrapper does).
It DOES read `I18N` global signal. SSR tests need to pin the locale via `use_hook`.

```rust
// Based on warning_list.rs:178-186 (VERIFIED pattern):
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

#[test]
fn slot_edit_renders_max_paid_employees_field_with_value() {
    fn app() -> Element {
        pin_de_locale();
        let mut slot = SlotEditItem::empty();
        slot.max_paid_employees = Some(5);
        rsx! {
            SlotEditInner {
                visible: true,
                slot: Rc::new(slot),
                slot_edit_type: SlotEditType::Edit,
                year: 2026,
                week: 25,
                has_errors: false,
                current_paid_count: 3,
                on_save: |_| {},
                on_cancel: |_| {},
                on_update_slot: |_| {},
            }
        }
    }
    let html = render(app);
    assert!(html.contains("5"), "max_paid value should appear in form: {html}");
}
```

**Note:** `Dialog` renders a modal backdrop — SSR renders it as HTML, so assertions on the number
input value attribute will be straightforward (`contains("value=\"5\"")`).

### Anti-Patterns to Avoid

- **Dynamic Tailwind class construction with `format!()`:** Never `format!("bg-{}", color_token)`.
  The `cell_background_class` function returns `&'static str` literals — this is exactly right.
  [VERIFIED: tailwind.config.js comment at :1-7, CLAUDE.md Pitfall 5]
- **Adding `current_paid_count` to `SlotEditItem`:** This struct is the server-write payload
  (`From<&SlotEditItem> for SlotTO`). Adding a display-only field there would cause it to be
  erroneously sent to the API. Keep it in `SlotEdit` (the dialog state container) only.
- **Checking `is_shiftplanner` for paid-overage coloring:** D-23-05 explicitly says all roles
  see the coloring. Do not gate it.
- **Disabling the Save button or showing a modal:** D-23-02 and MEMORY.md are explicit: inline
  non-blocking banner only.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Form field with label + hint | Custom div structure | `Field { hint: Some(...) }` | `form/field.rs` already renders label + hint + error layout |
| CSS color tokens | Hardcoded hex values | `bg-bad-soft`, `bg-warn-soft` (CSS variables) | Tokens handle light/dark theme automatically; vars in `input.css` |
| i18n placeholder substitution | String replace manually | `i18n.t_m_rc(Key::..., [...].into())` | Existing `I18n::t_m_rc` method; see `slot_edit.rs:67-73` for pattern |
| SSR test harness | Custom render setup | `VirtualDom::new(comp); vdom.rebuild_in_place(); dioxus_ssr::render(&vdom)` | Established codebase pattern |

---

## Open Questions Resolved

All concrete questions from the task brief were investigated directly in source code. Findings:

1. **`bad`/`bad-soft` token status:** ALREADY EXISTS. Tokens defined in `tailwind.config.js:35-36`
   and `input.css:44-45` (light) + `:67-68` (dark). `bg-bad-soft`, `text-bad`, `border-bad` are
   safelisted (`:75-81`). `bg-bad` is NOT safelisted but would be picked up by Tailwind's
   static scan since `cell_background_class` returns `&'static str`. Add to safelist if chosen.

2. **`SlotEditInner` props + `current_paid_count`:** Props struct (`slot_edit.rs:23-35`) does NOT
   currently have `current_paid_count`. It must be added as a new `pub current_paid_count: u8` prop.
   The value is not available from `api::get_slot` (returns `SlotTO` which has no `current_paid_count`).
   Must be injected from `shift_plan_context` in the page's dropdown closure.

3. **`max_paid_employees` roundtrip:** CONFIRMED. `SlotEditItem.max_paid_employees: Option<u8>`
   exists at `state/slot_edit.rs:22`. `From<&SlotTO>` copies it at `:60`, `From<&SlotEditItem>`
   copies it back at `:77`. Type is `Option<u8>` in both directions.

4. **`cell_background_class` signature:** `pub(crate) fn cell_background_class(missing: bool, discourage: bool) -> &'static str`
   at `week_view.rs:960`. Body is a 3-branch if/else. Called at `:1037`. Tests at `:686-703`.
   Extending to 3 params + adding a new branch is the complete work.

5. **Week-View `Slot` state fields:** CONFIRMED. `state/shiftplan.rs:177` `max_paid_employees: Option<u8>`,
   `:180` `current_paid_count: u8`. `load_shift_plan` at `loader.rs:172` fills `current_paid_count`
   from `ShiftplanSlotTO.current_paid_count`. `load_day_aggregate` at `loader.rs:225` does the same.

6. **i18n key pattern:** `Key` enum in `src/i18n/mod.rs:54-574`. New keys appended at bottom of enum.
   Translations added at bottom of each locale file (`en.rs`, `de.rs`, `cs.rs`) via
   `i18n.add_text(Locale::XX, Key::NewKey, "translation")`. Coverage test pattern:
   a `for key in [Key::A, Key::B, ...]` loop asserting `!value.is_empty() && value != "??"`.

7. **SSR snapshot test pattern:** VERIFIED. Pattern in `warning_list.rs:178-286` (canonical template),
   also in `week_view.rs:1368-1371`, `dialog.rs:461-464`. Helper: `fn render(comp: fn() -> Element)`.
   Assertions: `html.contains(...)`. I18N pinning: `fn pin_de_locale() { use_hook(|| { *I18N.write() = generate(Locale::De); }) }`.
   `dioxus-ssr` is a dev-dependency at `Cargo.toml:81`.

8. **`min_resources` oninput closure (template):** `slot_edit.rs:191-199`:
   ```rust
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
   ```
   Adaptation for `Option<u8>`: replace `parse::<i32>()` with a two-branch parse
   (empty → `None`, success → `Some(u8)`).

---

## Code Examples

### Existing `min_resources` input (template for new field)
```rust
// Source: slot_edit.rs:185-202 [VERIFIED]
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

### Existing `cell_background_class` (full body to extend)
```rust
// Source: week_view.rs:955-977 [VERIFIED]
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

/// Returns the min-resources `filled/need` text-color class.
pub(crate) fn min_resources_class(missing: bool) -> &'static str {
    if missing {
        "text-warn"
    } else {
        "text-ink-muted"
    }
}
```

### Existing SSR test helper (verbatim template)
```rust
// Source: warning_list.rs:178-188 [VERIFIED]
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

### Existing cell_background_class tests (must be updated to match new signature)
```rust
// Source: week_view.rs:685-713 [VERIFIED]
#[test]
fn cell_background_class_understaffed_is_warn_soft() {
    assert_eq!(cell_background_class(true, false), "bg-warn-soft");
}
#[test]
fn cell_background_class_fully_staffed_is_empty() {
    assert_eq!(cell_background_class(false, false), "");
}
#[test]
fn cell_background_class_discourage_is_bad_soft() {
    assert_eq!(cell_background_class(false, true), "bg-bad-soft");
}
#[test]
fn cell_background_class_discourage_overrides_missing() {
    assert_eq!(cell_background_class(true, true), "bg-bad-soft");
}
```
All four tests must be updated to pass the new `paid_overage` third argument as `false`
(existing behavior unchanged). New tests add `paid_overage = true` cases.

### `SlotEditItem` (complete struct — to confirm `max_paid_employees` type)
```rust
// Source: state/slot_edit.rs:8-23 [VERIFIED]
pub struct SlotEditItem {
    pub id: Uuid,
    pub day_of_week: Weekday,
    pub from: time::Time,
    pub to: time::Time,
    pub min_resources: u8,
    pub valid_from: time::Date,
    pub valid_to: Option<time::Date>,
    pub version: Uuid,
    pub shiftplan_id: Option<Uuid>,
    /// Mirror of `SlotTO.max_paid_employees` — not edited in the v1.2 UI but
    /// preserved on edit-roundtrip so the backend value is not overwritten with
    /// `None`. v1.3 FUI-02 will expose this in the slot editor (UI-SPEC Regel 2).
    pub max_paid_employees: Option<u8>,
}
```

---

## Common Pitfalls

### Pitfall 1: Dynamic Tailwind class construction
**What goes wrong:** Using `format!("bg-{}-soft", some_var)` produces class strings that
Tailwind's static scanner cannot see → purged in production build, CSS missing.
**Why it happens:** Tempting when color varies by state.
**How to avoid:** Return `&'static str` from decision functions; use `if`/`match` branches with
literal class strings. See `cell_background_class` as the canonical pattern.
**Warning signs:** Any `format!()` call that constructs a CSS class string.
[VERIFIED: tailwind.config.js:1-7 comment, existing cell_background_class pattern]

### Pitfall 2: Putting current_paid_count in SlotEditItem
**What goes wrong:** `SlotEditItem` converts `From<&SlotEditItem> for SlotTO` at `state/slot_edit.rs:64`.
If `current_paid_count` is added to `SlotEditItem`, future code might try to pass it to the API.
**Why it happens:** `SlotEditItem` feels like "the slot being edited".
**How to avoid:** Keep `current_paid_count: u8` in `SlotEdit` (dialog-state container), not in
`SlotEditItem` (server payload). The `SlotTO` DTO does not have `current_paid_count`.
[VERIFIED: rest-types/src/lib.rs:308-322]

### Pitfall 3: Missing i18n key in one locale
**What goes wrong:** German or Czech falls back to `"??"` at runtime.
**Why it happens:** Forgetting to add the key to all 3 locale files, or using wrong `Locale::` enum
variant (historical bug: `Locale::En` instead of `Locale::De` in de.rs).
**How to avoid:** Add i18n coverage test in `i18n/mod.rs` for new keys, asserting all 3 locales.
Pattern: add a `for key in [Key::NewKey1, Key::NewKey2]` loop in a test function.
[VERIFIED: existing tests at i18n/mod.rs:594-616, 698-725]

### Pitfall 4: SlotEditInner SSR test fails due to missing coroutine
**What goes wrong:** `SlotEdit` (the wrapper) registers `use_coroutine_handle::<SlotEditAction>()`.
If tests render `SlotEdit` instead of `SlotEditInner`, this panics without a registered coroutine.
**Why it happens:** Confusion between the wrapper and the inner component.
**How to avoid:** Always SSR-test `SlotEditInner` directly (it has no coroutine handles).
Use `SlotEditProps` to construct test props.
[VERIFIED: slot_edit.rs:212-228 — only SlotEdit wrapper calls use_coroutine_handle]

### Pitfall 5: `cell_background_class` signature change breaks existing tests
**What goes wrong:** Adding a third `paid_overage: bool` param breaks the 4 existing unit tests
at `week_view.rs:685-713`.
**How to avoid:** Update all 4 existing test calls to pass `paid_overage: false`, then add new
tests for `paid_overage: true` cases.
[VERIFIED: week_view.rs:686-703]

### Pitfall 6: WASM build fails after adding code
**What goes wrong:** Rust compiles fine for the host but fails for `wasm32-unknown-unknown` due to
WASM-incompatible crates (e.g., `tokio`, `std::time`, certain async runtimes).
**Why it happens:** The dev-toolchain is split; `cargo test` runs on host.
**How to avoid:** Run WASM-Build-Gate: `cargo build --target wasm32-unknown-unknown` from
`shifty-dioxus/` before committing. This is required by CLAUDE.md.
[VERIFIED: shifty-dioxus/CLAUDE.md WASM-Build-Gate section]

---

## Tailwind Color Token Reference

All tokens verified in `tailwind.config.js` and `input.css`:

| Token | Tailwind class | CSS var | Light value | Dark value | Safelisted |
|-------|---------------|---------|-------------|------------|------------|
| bad-soft | `bg-bad-soft` | `--bad-soft` | `#fde4e1` | `#3a1c18` | YES (`:75`) |
| bad | `text-bad`, `border-bad` | `--bad` | `#b8281a` | `#ef6a5b` | YES (`:78`, `:81`) |
| bg-bad | `bg-bad` | `--bad` | `#b8281a` | `#ef6a5b` | NO — add if used |
| warn-soft | `bg-warn-soft` | `--warn-soft` | `#fef0d6` | `#3a2a14` | YES (`:76`) |

**Recommendation:** Use `bg-bad-soft` for paid-overage cell background. This is visually distinct
from `bg-warn-soft` (orange) satisfying D-23-03, already safelisted, and consistent with the
existing `discourage` treatment. If the planner needs `paid_overage` to look *stronger* than
`discourage`, add `"bg-bad"` to the safelist and use it instead — but update the doc comment in
`cell_background_class` accordingly.

---

## i18n Keys to Add

Three new `Key` variants required (append to `i18n/mod.rs` enum before closing `}`):

```
// Phase 23 — Slot Paid-Capacity UI (FUI-01/FUI-02)
/// Label for the max paid employees input in the slot editor.
MaxPaidEmployeesLabel,
/// Hint below the max paid employees input ("empty = no limit").
MaxPaidEmployeesHint,
/// Inline warning shown in slot editor when entered limit < current_paid_count
/// (placeholders: {current}, {limit}).
MaxPaidEmployeesOverageHint,
```

Sample translations (Claude's discretion — planner can adjust wording):

| Key | En | De | Cs |
|-----|----|----|-----|
| `MaxPaidEmployeesLabel` | "Max paid employees" | "Max. bezahlte Mitarbeiter" | "Max. placených zaměstnanců" |
| `MaxPaidEmployeesHint` | "Empty = no limit" | "Leer = kein Limit" | "Prázdné = bez limitu" |
| `MaxPaidEmployeesOverageHint` | "Currently {current} paid ({limit} allowed)" | "Aktuell {current} bezahlt (Limit: {limit})" | "Aktuálně {current} placených (limit: {limit})" |

The inline-hint key uses `{current}` and `{limit}` placeholders, interpolated via
`i18n.t_m_rc(Key::MaxPaidEmployeesOverageHint, [("current", ...), ("limit", ...)].into())`.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust native test runner) |
| Config file | none (standard Cargo workspace) |
| Quick run command | `cargo test --package shifty-dioxus` |
| Full suite command | `cargo clippy --package shifty-dioxus -- -D warnings && cargo test --package shifty-dioxus && cargo build --target wasm32-unknown-unknown --package shifty-dioxus` |

### Phase Requirements → Test Map

| Req | Behavior | Test Type | Automated Command |
|-----|----------|-----------|-------------------|
| D-23-01 | Editor renders `max_paid_employees` field with value `Some(5)` | SSR unit | `cargo test --package shifty-dioxus slot_edit_renders_max_paid` |
| D-23-01 | Editor renders empty field for `None` | SSR unit | `cargo test --package shifty-dioxus slot_edit_renders_empty_when_none` |
| D-23-01 | i18n keys present in all 3 locales | Unit | `cargo test --package shifty-dioxus i18n_slot_paid_capacity_keys` |
| D-23-02 | Inline warn banner renders when limit < current_paid_count | SSR unit | `cargo test --package shifty-dioxus slot_edit_shows_overage_hint` |
| D-23-02 | No inline hint when limit >= current_paid_count | SSR unit | `cargo test --package shifty-dioxus slot_edit_no_overage_hint_when_ok` |
| D-23-03 | `cell_background_class` returns bad-soft on paid_overage=true | Pure unit | `cargo test --package shifty-dioxus cell_background_class_paid_overage` |
| D-23-04 | `cell_background_class` paid_overage wins over missing | Pure unit | `cargo test --package shifty-dioxus cell_background_class_paid_overage_overrides_missing` |
| D-23-04 | `cell_background_class` discourage still wins over paid_overage | Pure unit | `cargo test --package shifty-dioxus cell_background_class_discourage_overrides_overage` |
| D-23-03 | WeekCellSlot SSR carries bad-soft class when overage | SSR unit | `cargo test --package shifty-dioxus week_cell_slot_paid_overage_carries_bad_soft` |
| D-23-03 | WeekCellSlot SSR carries warn-soft when only understaffed (no overage) | SSR unit | `cargo test --package shifty-dioxus week_cell_slot_understaffed_no_overage_is_warn_soft` |

### Wave 0 Gaps

- [ ] `src/component/slot_edit.rs` — add SSR test module (VirtualDom + dioxus_ssr imports,
  `fn render()` helper, `fn pin_de_locale()` helper). No new test file needed — add to existing
  `mod tests` in `slot_edit.rs`.
- [ ] `src/i18n/mod.rs` — add `i18n_slot_paid_capacity_keys_present_in_all_locales` test
  (covering the 3 new keys).
- [ ] `week_view.rs:685-703` — update existing 4 tests to pass `paid_overage: false` as third arg.

---

## Security Domain

This phase is purely UI — no authentication, no data write beyond existing `max_paid_employees`
field (already in the write path). No new API endpoints, no new authentication surface, no new
data exposure. ASVS categories not applicable.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust (cargo) | All compilation | yes (NixOS/nix develop) | workspace | — |
| wasm32-unknown-unknown target | WASM-Build-Gate | yes (project standard) | — | — |
| npx tailwindcss | CSS compilation | yes (project standard) | — | — |
| dioxus-ssr | SSR tests | yes (Cargo.toml:81 dev-dep) | 0.6 | — |

---

## Assumptions Log

All claims in this research were verified directly from source code in this session. No `[ASSUMED]`
claims are made.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | — | — | — |

**All claims verified.** No user confirmation needed.

---

## Sources

### Primary (HIGH confidence — all verified in session via Read/Bash tools)
- `shifty-dioxus/src/component/slot_edit.rs` — full file read, props struct, min_resources input pattern, existing tests
- `shifty-dioxus/src/state/slot_edit.rs` — full file read, SlotEditItem struct, From impls, SlotEdit struct
- `shifty-dioxus/src/component/week_view.rs:955-977` — cell_background_class body and tests
- `shifty-dioxus/src/component/week_view.rs:1014-1037` — WeekCellSlotProps and call site
- `shifty-dioxus/src/state/shiftplan.rs:167-211` — Slot struct with max_paid_employees and current_paid_count
- `shifty-dioxus/src/loader.rs:154-256` — load_shift_plan and load_day_aggregate filling current_paid_count
- `shifty-dioxus/tailwind.config.js` — full file read, color tokens, safelist
- `shifty-dioxus/input.css` — full file read, CSS variable values light + dark
- `shifty-dioxus/src/i18n/mod.rs:54-574` — Key enum, generate function, test patterns
- `shifty-dioxus/src/i18n/en.rs:900-921` — BookingWarningPaidLimitExceeded and recent key pattern
- `shifty-dioxus/src/i18n/de.rs:975-990` — German translations for same keys
- `shifty-dioxus/src/i18n/cs.rs:960-978` — Czech translations for same keys
- `shifty-dioxus/src/component/warning_list.rs:172-285` — canonical SSR test pattern
- `shifty-dioxus/src/component/form/field.rs` — Field component with hint prop
- `shifty-dioxus/src/service/slot_edit.rs` — SLOT_EDIT_STORE, SlotEditAction, load_slot_edit
- `shifty-dioxus/src/page/shiftplan.rs:650-680` — dropdown entry with LoadSlot call site
- `shifty-dioxus/src/state/dropdown.rs` — DropdownEntry type (Fn(Option<Rc<str>>))
- `shifty-dioxus/Cargo.toml` — dioxus 0.6.1, dioxus-ssr 0.6 dev-dep

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified directly in Cargo.toml
- Architecture: HIGH — verified by reading all canonical files listed in CONTEXT.md
- Pitfalls: HIGH — derived from existing code patterns and verified test expectations
- Color tokens: HIGH — verified tailwind.config.js + input.css

**Research date:** 2026-06-26
**Valid until:** 2026-07-26 (stable codebase — no rapid churn expected in frontend primitives)
