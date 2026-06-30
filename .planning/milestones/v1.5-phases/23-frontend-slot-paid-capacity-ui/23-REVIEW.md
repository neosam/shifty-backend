---
phase: 23-frontend-slot-paid-capacity-ui
reviewed: 2026-06-27T00:00:00Z
depth: deep
files_reviewed: 9
files_reviewed_list:
  - shifty-dioxus/src/component/week_view.rs
  - shifty-dioxus/src/component/slot_edit.rs
  - shifty-dioxus/src/state/slot_edit.rs
  - shifty-dioxus/src/service/slot_edit.rs
  - shifty-dioxus/src/page/shiftplan.rs
  - shifty-dioxus/src/i18n/mod.rs
  - shifty-dioxus/src/i18n/en.rs
  - shifty-dioxus/src/i18n/de.rs
  - shifty-dioxus/src/i18n/cs.rs
findings:
  critical: 0
  warning: 1
  info: 2
  total: 3
status: issues_found
---

# Phase 23: Code Review Report

**Reviewed:** 2026-06-27
**Depth:** deep
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Reviewed the uncommitted working-copy changes implementing the frontend Slot
Paid-Capacity UI: the 3-arg `cell_background_class` overage branch in `week_view.rs`,
the `max_paid_employees` editor field + non-blocking overage banner in `slot_edit.rs`,
the `current_paid_count` threading through state / service / page, and the three new
i18n keys across all locales.

Overall the implementation is correct and well-targeted. The core logic I was asked to
scrutinize all holds up:

- **Option<u8> parsing** (`slot_edit.rs:243-252`): empty -> `None`, valid -> `Some(value)`,
  non-empty-non-u8 -> silently ignored (no `unwrap`/`expect`, no panic). Correct.
- **Overage-priority ordering** (`week_view.rs:977-989`): `discourage || paid_overage`
  returns `bg-bad-soft` and outranks `missing` -> `bg-warn-soft`, matching the D-23-04
  priority table exactly. The merged red arm is behavior-identical (declared out-of-scope).
- **`read_unchecked`** (`shiftplan.rs:662`): used inside an event-handler closure where a
  reactive subscription would be meaningless; the missing-slot path degrades to `0` via
  `unwrap_or(0)` / `_ => 0`. Correct and defensively coded.
- **i18n interpolation**: `t_m_rc` does literal `{key}` -> value replacement; the call site
  supplies exactly `{current}` and `{limit}`, and all three locale strings use exactly those
  two placeholders. No stray `{max}`/unreplaced tokens. The coverage test guards all three
  keys in all three locales.
- **XSS/escaping**: all interpolations (`{overage_str}`, `{max_paid_value}`) are Dioxus text
  nodes (auto-escaped); no `dangerous_inner_html`. Interpolated content is numeric
  (`u8::to_string`) anyway. No injection surface.
- **No new `unwrap`/`expect`/panic** introduced by this phase (the `.unwrap().unwrap()` at
  `shiftplan.rs:659` is pre-existing and unchanged, mirrored in the Remove-slot closure).

All referenced test helpers (`make_slot`, `render`, `render_with_tooltip`, `Field`'s `hint`
prop, `SlotEditItem::empty`) and state fields were verified to exist. One real behavioral
edge (silent revert of out-of-range numeric input) is raised as a WARNING; two minor
observations as INFO.

## Warnings

### WR-01: Out-of-range numeric input is silently swallowed with no user feedback

**File:** `shifty-dioxus/src/component/slot_edit.rs:242-253`
**Issue:** The `max_paid_employees` field is a *controlled* input (`value: "{max_paid_value}"`)
whose `oninput` parses into `u8`. When the user types a value that is non-empty but not a
valid `u8` — most plausibly a number `> 255` (e.g. typing "256", or appending a digit to
"25" to make "256") — `raw.parse::<u8>()` returns `Err`, `on_update_slot` is **not** called,
and the keystroke is silently dropped. Because the input is controlled, the displayed value
then snaps back to the last accepted value on the next render. From the user's perspective
the field appears to "eat" keystrokes with no explanation. The inline comment
(`// parse failure (non-empty, non-u8): silently ignore.`) confirms this is intentional, but
silent input rejection is a usability defect, not just a style choice. There is also no
`max` attribute on the `<input type="number">` to hint the 255 ceiling to the browser.
**Fix:** Add a `max="255"` attribute so the browser surfaces the bound, and/or clamp instead
of dropping (parse as a wider int and saturate to `u8::MAX`), so the field never silently
ignores a keystroke:
```rust
input {
    class: FORM_INPUT_CLASSES,
    r#type: "number",
    min: "0",
    max: "255",
    value: "{max_paid_value}",
    oninput: {
        let slot = props.slot.clone();
        move |event: Event<FormData>| {
            let raw = event.value();
            let mut updated = slot.as_ref().clone();
            if raw.is_empty() {
                updated.max_paid_employees = None;
                props.on_update_slot.call(updated);
            } else if let Ok(value) = raw.parse::<u32>() {
                updated.max_paid_employees = Some(value.min(u8::MAX as u32) as u8);
                props.on_update_slot.call(updated);
            }
        }
    },
}
```
(Realistically a paid-employee count never approaches 255, so impact is low — hence WARNING
not BLOCKER — but the silent-drop behavior is worth a guard or an explicit `max`.)

## Info

### IN-01: Overage feedback uses two different color tokens (warn vs. bad) across surfaces

**File:** `shifty-dioxus/src/component/slot_edit.rs:259` and `shifty-dioxus/src/component/week_view.rs:982`
**Issue:** The same logical condition — paid count exceeds the limit — renders **orange**
(`bg-warn-soft`) as the editor banner but **red** (`bg-bad-soft`) as the week-view cell tint.
This is defensible (the editor banner is explicitly a non-blocking warning per D-23-02, while
the cell uses the shared red overage state) and is test-locked on both sides, so it is not a
defect. Flagging only so a future reader does not "fix" one to match the other and break a
test. No change required unless the UI-SPEC intends a single color.

### IN-02: Overage string is computed every render even when the banner is hidden

**File:** `shifty-dioxus/src/component/slot_edit.rs:73-91`
**Issue:** `overage_str` (an `i18n.t_m_rc` interpolation that allocates two `String`s and runs
two `str::replace` passes) is computed unconditionally at the top of `SlotEditInner`, but it is
only rendered when `show_overage` is true. The wasted work is trivial (a few small allocations
on a low-frequency editor render) and performance is explicitly out of v1 review scope; noted
only as a readability/locality observation. If desired, move the `t_m_rc` call inside the
`if show_overage { ... }` block so the interpolation runs only when shown.
**Fix:** Optional — inline the `overage_str` computation under the `if show_overage` guard.

---

_Reviewed: 2026-06-27_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
