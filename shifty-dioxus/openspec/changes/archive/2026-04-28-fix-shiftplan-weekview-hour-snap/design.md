## Context

The shiftplan WeekView (`shifty-dioxus/src/component/week_view.rs`) renders a CSS-Grid week view with a sticky time column on the left and one column per visible weekday. The vertical extent of the body is controlled by three numbers that must agree:

1. The body container's pixel height (`body_height`).
2. The number and pixel position of whole-hour labels in the sticky time column.
3. The pixel height of each day column (which absolutely-positions slots inside it).

Today, after the CSS-Grid redesign (`0f47d8e`), these three values disagree whenever any slot has a fractional minute boundary:

```rust
let body_height = (day_end - day_start) * SCALING;       // FRACTIONAL hours × 75
let hour_start  = day_start.ceil() as u8;
let hour_end    = day_end.ceil()   as u8;                // CEIL → may add an extra hour
let time_col_style = format!("...height: {}px;", body_height);
let day_col_style  = format!("height: {}px;", body_height);
// ...time-column loop renders one whole-hour label per (hour_start..hour_end), each SCALING tall
```

Concrete failure: `day_start = 9.0`, `day_end = 19.5` → `body_height = 787.5`, but the time-column loop emits 11 labels of 75 px = 825 px. The wrapper's `overflow-auto` then shows a 37.5-px scrollbar. A symmetric problem occurs at the top when `day_start` is fractional: the first whole-hour label sits below an unlabeled gap.

`day_start` and `day_end` flow in from `state::Shiftplan::day_start_hour()` / `day_end_hour()` (around `state/shiftplan.rs:240` and `:250`), which take the min/max of `slot.from_hour()` / `slot.to_hour()`. Both can be fractional because slots have minute precision (`<input type="time">` with default `step=60s`).

A `+ SCALING / 2.0` (37.5 px) buffer survives in `DayView::height` (line 553) but is not applied to the new grid body or to the time column. Pre-redesign code used `+ SCALING` (75 px) on the outer container, which masked the issue without ever truly aligning the values.

## Goals / Non-Goals

**Goals:**
- A shiftplan with any combination of fractional or whole-hour slot bounds renders without a vertical scrollbar inside the WeekView wrapper, at the default zoom level and a typical viewport.
- Whole-hour labels in the time column are semantically aligned with the rendered cells: a label "19:00–20:00" is shown only if the body actually covers the 19:00–20:00 hour.
- Slot positioning inside day columns remains pixel-correct relative to its labeled hour (a 19:00–19:30 slot occupies the top half of the "19:00–20:00" cell).
- No change in behavior for plans whose slot bounds are all on whole hours.

**Non-Goals:**
- Changing the slot data model (it stays at minute precision).
- Adding sub-hour labels (e.g., "19:00–19:30").
- Re-doing the zoom-level mechanism.
- Touching `DayAggregateView`, `DayView` outside the height computation, or other consumers of `ColumnView` that don't suffer the same mismatch.
- Changing horizontal scrolling behavior.

## Decisions

### Decision 1: Snap to whole hours at the WeekView root, not at consumer call sites

The mismatch is between three sibling computations inside `WeekView::render` (`week_view.rs` around lines 1205–1218 plus the time-column loop at 1260). The simplest single source of truth is to round `day_start` down and `day_end` up to whole-hour `f32` values immediately after they are received as props, and use those rounded values for `body_height`, `hour_start`, `hour_end`, the time-column label loop, and any `DayView::day_start` / `DayView::day_end` props passed downward.

**Rationale:**
- A single line transforms the inputs; everything downstream then uses consistent values.
- Slot positioning inside `DayView` already uses `slot.from_hour() - props.day_start` (line 1032 / 561). With the rounded `day_start`, a 9:30 slot in a `day_start = 9.0` column gets `top = (9.5 − 9.0) × 75 = 37.5 px`, which lands correctly inside the labeled "9:00–10:00" cell.
- Avoids touching `state::Shiftplan` or any spec-level data accessor.

**Alternatives considered:**
- *Keep fractional `day_start` / `day_end` and fix only `body_height` to `(hour_end − hour_start) × SCALING`*: would fix the bottom overhang for `day_end = 19.5`, but the top gap for `day_start = 9.5` remains because `hour_start = day_start.ceil()` would still be `10` and the first labeled hour starts 37.5 px below the top with the 9:30 slot rendered above the first label. Rejected.
- *Restore the pre-redesign `+ SCALING` (or `+ SCALING / 2.0`) buffer*: hides the symptom rather than aligning the model. Also produces a labeled "20:00–21:00" row that has no associated slots, which is semantically wrong. Rejected.
- *Render half-hour labels for fractional bounds*: more code paths, more visual noise, and changes the design language of the time column. Out of scope (see Non-Goals).

### Decision 2: Remove the `+ SCALING / 2.0` buffer in `DayView::height` (line 553)

With Decision 1 in place, the rounded `day_end - day_start` is already an integer multiple of 1 hour, so the day-column height equals the time-column height equals `body_height` exactly. The 37.5-px buffer that `DayView` adds (line 553) is leftover compensation for the very mismatch this change removes. Keeping it would re-introduce a 37.5-px discrepancy between time-column and day-column heights.

**Rationale:** the buffer was a hack that hid a symptom; removing it restores the invariant "all three heights are equal."

**Alternative considered:** leave the buffer and accept a 37.5-px taller day column. Rejected because that brings back the same kind of pixel-level mismatch we're fixing.

### Decision 3: Keep the rounded values as `f32`, not `u8`

`day_start` / `day_end` are passed as `f32` to `DayView` and used in slot-position arithmetic (`slot.from_hour() - props.day_start`). Casting to `u8` and back would compile but obscures the intent. Using `f32` derived from `floor()` / `ceil()` keeps the existing types and arithmetic untouched.

## Risks / Trade-offs

- **Visible empty half-hour band at the edge of a day** → for plans whose first or last slot has a fractional bound, an unfilled half-hour now appears inside the labeled hour cell (e.g., 19:30–20:00 empty at the bottom). This is intentional and matches the labeled time scale; users who find it noisy can change the slot to end on a whole hour. *Mitigation:* none — this is the correct rendering of the existing data.
- **Existing tests that assert pixel heights for fractional ranges fail** → if any unit/snapshot test asserts `787.5 px` or similar, it must be updated to the rounded value. *Mitigation:* run `cargo test` after the change; update affected tests in the same PR.
- **`DayAggregateView`, `WeekViewPrint`, and other consumers of the same components** could rely on the old `DayView` height. *Mitigation:* grep for `DayView` and `(day_end - day_start)` usages; verify that all call sites either pass integer hours already or are unaffected by Decision 2 (likely the case, since `DayAggregateView` constructs slots at `08:00–16:00` whole hours).
- **Print stylesheet** (`print:overflow-visible`) — the change does not affect print rendering; print already disables the scroll container, so the only visible change in print is the slightly taller body for fractional plans.

## Migration Plan

No data migration needed. The change is purely client-side rendering.

1. Land the change in `week_view.rs`.
2. Run `cargo test` and the dev server; verify on a plan that has at least one fractional slot bound (e.g., create a Saturday slot ending 19:30) that no scrollbar appears and the time column reads `19:00–20:00` with the slot occupying the top half of that cell.
3. Verify on a plan with only whole-hour slots that the rendering is pixel-identical to today.

Rollback: revert the commit; no schema or persistence is touched.
