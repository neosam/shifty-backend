## 1. Snap day bounds in WeekView

- [x] 1.1 In `shifty-dioxus/src/component/week_view.rs::WeekView`, derive `let day_start = props_day_start.floor();` and `let day_end = props_day_end.ceil();` immediately after destructuring the props (around the existing line `let body_height = (day_end - day_start) * SCALING;` at line 1205) and use these snapped values for every downstream computation in the function.
- [x] 1.2 Verify that `body_height`, `time_col_style`, `day_col_style`, the `hour_start..hour_end` time-column loop, and any `DayView { day_start, day_end, .. }` props all consume the snapped `day_start` / `day_end` (no remaining reference to the unsnapped raw values).
- [x] 1.3 Confirm via `cargo check` that the slot positioning expression `(slot.from_hour() - props.day_start) * SCALING` (line 1032 / line 561) compiles unchanged and produces the correct top-offset for fractional slots given the new whole-hour `day_start`.

## 2. Remove the leftover height buffer in DayView

- [x] 2.1 In `shifty-dioxus/src/component/week_view.rs::DayView` (line ~553), change `height: (props.day_end - props.day_start) as f32 * SCALING + SCALING / 2.0,` to `height: (props.day_end - props.day_start) as f32 * SCALING,` so the day-column height equals the body height exactly.
- [x] 2.2 Search the file for any other `+ SCALING / 2.0` occurrences and confirm none of them are guarding against the same mismatch (TimeView already lacks the buffer; DayAggregateView is unaffected).

## 3. Tests

- [x] 3.1 Add or update a unit test in `shifty-dioxus/src/component/week_view.rs` (or its test sibling) that constructs a `WeekViewProps` with fractional `day_start = 9.5` and `day_end = 19.5` and asserts: (a) the rendered body's inline `height` style equals `(20 - 9) * SCALING` px, (b) the time column emits exactly `(20 - 9)` whole-hour labels, (c) a slot starting at `9:30` is positioned at `top = 0.5 * SCALING`.
- [x] 3.2 Run `cargo test` from `shifty-dioxus/` and update any existing tests that asserted the old fractional pixel heights.
- [x] 3.3 Run `cargo fmt` and `cargo clippy` and resolve any new warnings introduced by the change.

## 4. Manual verification in the browser

- [x] 4.1 Start the dev stack (Tailwind watcher + `dx serve --hot-reload`) and open `http://localhost:8080/shiftplan/`. Pick a week containing at least one slot with a half-hour bound (or create one via the slot-edit dialog, e.g. Saturday 09:00–14:30). Confirm the WeekView wrapper has no vertical scrollbar at the default zoom.
- [x] 4.2 Inspect via DevTools that for the same fractional plan, the wrapper's `clientHeight >= scrollHeight`, the time column's last label reads e.g. `14:00–15:00`, and the slot ending at 14:30 occupies the top half of that cell.
- [x] 4.3 Switch to a week with only whole-hour slots and confirm the rendering is visually identical to the previous behavior.
- [x] 4.4 Sanity-check the print preview (`Ctrl+P`) on a fractional plan: no clipping at the bottom, all slots visible.

## 5. Wrap up

- [x] 5.1 Update `openspec/changes/fix-shiftplan-weekview-hour-snap/tasks.md` checkboxes as work proceeds.
- [x] 5.2 When all tasks are checked, run `openspec verify --change fix-shiftplan-weekview-hour-snap` (or the equivalent `/opsx:verify`) before archiving.
