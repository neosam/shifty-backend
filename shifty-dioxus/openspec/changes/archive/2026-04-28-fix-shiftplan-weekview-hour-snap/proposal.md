## Why

After the CSS-Grid redesign of the shiftplan WeekView (commit `0f47d8e`), a vertical scrollbar appears whenever any slot starts or ends on a non-whole hour (e.g., 19:30 or 9:30). The grid body is sized to the fractional range `(day_end - day_start) * SCALING`, but the time column still renders one whole-hour label per ceil-hour, so the last label spills past the body and triggers `overflow-auto`. Before the redesign a `+ SCALING` buffer absorbed this mismatch; the buffer was removed but the label range was not adjusted to match.

In addition to the visible scrollbar, the current behavior produces semantically wrong labels: a slot ending at 19:30 is shown under a "19:00–20:00" label, and a slot starting at 9:30 sits under an unlabeled half-hour gap above the first "10:00–11:00" label.

## What Changes

- Snap the WeekView's effective day range to whole-hour boundaries (`day_start.floor()` / `day_end.ceil()`) before computing the body height, time-column labels, and slot positions, so the body, the labels, and the day columns all share the same hour range.
- Remove the now-unnecessary `+ SCALING / 2.0` height buffer in `DayView` (`week_view.rs:553`), which exists today only to compensate for the fractional-end mismatch.
- The visible labeled time range will widen by up to one hour at each end when slots have fractional bounds; partial first/last hours show as empty grid backgrounds inside the correctly-labeled hour cell.

No behavior change for shiftplans whose slots all start and end on whole hours.

## Capabilities

### New Capabilities
*(none)*

### Modified Capabilities
- `shiftplan-page`: the week-grid rendering requirements gain a rule that the time column, body height, and day columns all use a whole-hour-aligned range derived from the slot bounds.

## Impact

- **Code**: `shifty-dioxus/src/component/week_view.rs` — `WeekView` body-height / time-column computation, `DayView` height computation, and the slot positioning that uses `day_start` as anchor.
- **No API or data-model change**: `Slot::from` / `Slot::to` remain `time::Time` with minute precision; users keep being able to enter half-hour or quarter-hour boundaries.
- **Visual change**: shiftplans with fractional slot bounds will show whole-hour-labeled rows that may extend up to 30 minutes above the first slot or below the last slot. Plans with only whole-hour slots are pixel-identical to today.
- **Tests**: existing snapshot/render tests that assume the current pixel heights for fractional ranges may need to be updated.
