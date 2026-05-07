## 1. PersonChip atom

- [x] 1.1 In `src/component/atoms/person_chip.rs`, append `whitespace-nowrap` to the `SHAPE_CLASSES` constant (line 27).
- [x] 1.2 Add a unit test in the existing `tests` module asserting `build_class(true, false, false)` contains `whitespace-nowrap`.
- [x] 1.3 Add a unit test asserting `build_class(false, false, false)` (no-color path) also contains `whitespace-nowrap`.
- [x] 1.4 Add a unit test asserting `build_class(true, true, false)` (bold variant) contains both `whitespace-nowrap` and `font-semibold`.
- [x] 1.5 Run `cargo test -p shifty-dioxus person_chip` and confirm all tests pass.
- [x] 1.6 Audit other `PersonChip` consumers visually (toolbar, my-shifts day rows, employee detail header, sidebar list, slot-edit modal) by grepping `PersonChip {` and skimming the surrounding layout — confirm `whitespace-nowrap` is the desired behavior in each spot. Note any exception in the change folder if found; otherwise no action.

## 2. WeekCellSlot three-zone layout

- [x] 2.1 In `src/component/week_view.rs::WeekCellSlot` (≈ lines 1029–1167), update the outer `<div>`'s class string: append `overflow-hidden` to the existing `format!("absolute left-0 right-0 border-t border-border {} {}", bg_class, ...)`.
- [x] 2.2 Remove `padding: 6px 32px 6px 8px` from the outer `<div>`'s inline style (the inner chip area now owns the inset; the outer needs only `top` and `height`).
- [x] 2.3 Move the min-resources `span` out of the wrap-flex div and into a sibling absolutely-positioned `span` with `style: "position: absolute; top: 6px; left: 8px; pointer-events: none; line-height: 18px;"`. Keep the existing `font-mono text-small font-bold {mr_class}` classes.
- [x] 2.4 Replace the existing `div { class: "flex flex-wrap items-start gap-1", … }` with a new chip-area `<div>`: `class: "flex flex-wrap content-start gap-1 overflow-y-auto overflow-x-hidden"`, `style: "position: absolute; inset: 6px 32px 6px 38px;"`. Inside it, render only the per-booking `WeekCellChip` loop (no min-resources span, no action button).
- [x] 2.5 Confirm the action button branch (`CellButton::Add | Remove | Dropdown | None`) remains a sibling of the chip area (not nested inside it). Existing absolute positioning via `cell_button_classes` should already place it at `top: 6px; right: 6px`; verify nothing changed.

## 3. Slot SSR tests

- [x] 3.1 Locate the existing `WeekCellSlot` SSR tests (search `mod tests` in `week_view.rs` or a sibling `week_view_tests.rs`). If none exist for `WeekCellSlot`, create a new `tests` module at the bottom of `week_view.rs` following the pattern from `person_chip.rs::tests` (uses `dioxus_ssr::render` with a `VirtualDom`).
- [x] 3.2 Add an SSR test rendering a `WeekCellSlot` with two bookings and `min_resources = 3`. Assert: outer HTML contains `overflow-hidden`; outer HTML contains `bg-warn-soft`; chip-area HTML contains `flex-wrap`, `content-start`, `overflow-y-auto`, `overflow-x-hidden`; outer HTML contains the inline `inset: 6px 32px 6px 38px`.
- [x] 3.3 Add an SSR test asserting the chip-area `<div>` contains exactly N booking elements when `slot.bookings.len() == N` (use a count of 3 or more).
- [x] 3.4 Add an SSR test asserting the min-resources span is positioned via `top: 6px; left: 8px` AND its style contains `pointer-events: none`.
- [x] 3.5 Add an SSR test asserting the +/- button (when rendered) appears in HTML AFTER the chip-area `<div>`'s closing tag (i.e., as a sibling, not a descendant). A simple substring-order assertion on the rendered HTML is sufficient.
- [x] 3.6 Run `cargo test -p shifty-dioxus week_view` and confirm all tests pass.

## 4. Visual verification

- [x] 4.1 Start the dev environment per `shifty-dioxus/CLAUDE.md` (Tailwind watcher in one terminal, `dx serve --hot-reload` in another).
- [x] 4.2 Open the shiftplan page; pick a slot with few bookings and confirm visual parity with pre-change (n/m pill top-left, chips center, +/- top-right, no internal scrollbar visible).
- [x] 4.3 Pick or create a slot whose bookings exceed visible capacity (e.g. a 1-hour slot with 6+ bookings). Confirm: chip area scrolls vertically; n/m pill stays visible at top-left; +/- button stays visible at top-right; nothing paints into the next slot.
- [x] 4.4 Toggle the `discourage` state (warn-soft background) and confirm tinting still applies to the outer slot box.
- [x] 4.5 Switch a tab to one whose `WeekViewButtonTypes::Dropdown` is active (if reachable from the UI) or otherwise inspect a slot rendered with the dropdown variant, and confirm the `…` trigger sits at top-right and remains clickable.
- [x] 4.6 Switch the locale to `de` and `cs` and confirm the n/m pill format and chip rendering are unchanged.

## 5. Final checks

- [x] 5.1 `cargo fmt` in `shifty-dioxus/`.
- [x] 5.2 `cargo clippy -p shifty-dioxus --no-deps` — fix any new warnings introduced by the change.
- [x] 5.3 `cargo test -p shifty-dioxus` — full suite green.
- [x] 5.4 `openspec validate shiftplan-slot-fixed-height-overflow --strict` — confirm the change passes validation before archiving.
