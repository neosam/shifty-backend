## 1. Page Skeleton and Wrapper

- [x] 1.1 Replace the outer wrapper in `src/page/my_shifts.rs` with `main { class: "mx-auto max-w-[760px] w-full px-4 py-6 md:py-8 space-y-4", ... }` rendered after `TopBar {}`
- [x] 1.2 Replace the heading with `h1 { class: "text-xl font-semibold text-ink", "{i18n.t(Key::MyShifts)}" }`
- [x] 1.3 Remove the `expanded_weeks` and `initialized` signals, the click-handler, and the `▼ / ►` glyph — cards no longer collapse

## 2. Data Helpers

- [x] 2.1 Add a private helper `fn block_hours(block: &BlockTO) -> f32` that returns the duration of a block in hours by computing `(block.to - block.from).as_seconds_f32() / 3600.0`
- [x] 2.2 Add a private helper `fn sum_hours<'a, I: IntoIterator<Item = &'a BlockTO>>(blocks: I) -> f32` that returns the total hours
- [x] 2.3 Add a private helper `fn format_hours(hours: f32) -> String` that returns `format!("{:.1}", hours)` (zero renders as `0.0`; `-0.0` is normalized to `0.0` via the `+ 0.0` trick)
- [x] 2.4 Extract grouping into `fn group_blocks_by_week(blocks: &[BlockTO]) -> BTreeMap<(u32, u8), Vec<BlockTO>>` that filters out empty week keys; the page calls it instead of inlining the loop

## 3. Week Card Layout

- [x] 3.1 Replace the existing accordion `div { class: "border rounded-lg overflow-hidden", ... }` with a card container `section { class: "rounded-md border border-border bg-surface overflow-hidden" }` (semantic `<section>`)
- [x] 3.2 Render the card header as a flex row `div { class: "flex items-baseline justify-between px-4 py-3 border-b border-border" }` with the left span containing `"{i18n.t(Key::WeekLabel)} {week} · {date_range}"` (text-base, semibold, text-ink) and the right span containing the week-total hours (`font-mono tabular-nums text-sm text-ink`)
- [x] 3.3 Render the card body as `div { class: "px-4 py-2 divide-y divide-border" }` containing seven day rows
- [x] 3.4 Replace the `text-gray-500 p-4` "Loading..." block with `div { class: "text-ink-muted px-4 py-3", "Loading..." }`
- [x] 3.5 Replace the `text-red-600 p-4` error block with `div { class: "text-bad px-4 py-3", "Error loading shifts: {err}" }`
- [x] 3.6 Verify the all-empty case still renders `i18n.t(Key::NoShiftsFound)` styled with `text-ink-muted`

## 4. Day Rows

- [x] 4.1 For each weekday in `ALL_WEEKDAYS` (Monday → Sunday) partition the week's blocks by `day_of_week`
- [x] 4.2 Render each row as `div { style: "{DAY_ROW_STYLE}", class: "my-shifts-day-row py-2", ... }` — desktop columns inline; the mobile (`<= 720 px`) `grid-template-columns: 80px 1fr 50px;` override is injected via a `<style>` block on the page
- [x] 4.3 Render the day label cell as `span { class: "font-mono tabular-nums text-sm text-ink-soft", "{weekday_short}" }` using `Weekday::i18n_short_string`
- [x] 4.4 Render the shifts cell as a column-flex container; for non-empty days emit one `ShiftItemView` per block, for empty days emit `span { class: "text-ink-muted", "—" }`
- [x] 4.5 Render the hours cell as `span { class: "font-mono tabular-nums text-sm text-ink text-right", "{format_hours(day_total)}" }` (`0.0` for empty days)

## 5. Shift Items

- [x] 5.1 Render each shift item as an inline-flex container with class `"flex items-center gap-2"`
- [x] 5.2 Render the time range as `span { class: "font-mono tabular-nums text-sm text-ink", "{from_hh_mm}–{to_hh_mm}" }` using the `format_time_range` helper (en-dash `–`, not hyphen)
- [x] 5.3 Render the area badge using the existing `PersonChip` atom: `PersonChip { name, color: Some(...) }` when present
- [x] 5.4 When `block.sales_person.is_none()`, render `PersonChip { name: ImStr::from("-"), color: None }` so the dashed-border fallback variant kicks in

## 6. Note-Path Hook (Deferred Visual)

- [x] 6.1 Added an `Option<ImStr> note` field to `DayDisplay` with a doc comment explaining future warning-visual wiring (currently always `None`)
- [x] 6.2 Did NOT implement the `bg-warn-soft` rendering — deferred until `BlockTO`/`SlotTO` carry a note field (see `design.md` Decision 5)

## 7. Tests

- [x] 7.1 SSR test `week_card_renders_header_and_seven_day_rows`: a sample week with one Monday block renders header containing the week number, six em-dashes for the empty days, the sales-person name, and uses `bg-surface` (not `bg-gray-100`/`bg-white`)
- [x] 7.2 SSR test `day_row_with_shifts_renders_each` + `build_week_display_two_blocks_one_day_sums`: two blocks (3.5 h + 2.0 h) render both shift items and the day-total `5.5`
- [x] 7.3 SSR test `shift_item_without_sales_person_uses_dashed_chip`: a block without a sales person renders the dashed-border fallback chip (no `person-pill`, contains `border-dashed`)
- [x] 7.4 Unit test `format_hours_one_decimal`: `format_hours(0.0) == "0.0"`, `format_hours(-0.0) == "0.0"`, `format_hours(5.5) == "5.5"`. (Note: `12.25` rounds to `12.2` due to `format!`'s round-half-to-even — test uses `12.5 → "12.5"` for clarity)
- [x] 7.5 Unit tests `block_hours_for_four_and_a_half` + `block_hours_for_two_hours`: durations from 09:00–13:30 and 14:00–16:00 return `4.5` and `2.0`
- [x] 7.6 Unit test `group_blocks_by_week_omits_empty_keys`: empty input yields empty map (no zombie keys), all-empty page state therefore renders `Key::NoShiftsFound` and emits no card markup
- [x] 7.7 Unit test `group_blocks_by_week_groups_per_year_week_pair`: weeks with empty block lists never appear; non-empty weeks each emit one entry
- [x] 7.8 SSR test `source_does_not_use_legacy_gray_classes`: the non-test source contains none of `bg-gray-100`, `bg-gray-200`, `bg-white`, `text-gray-500`, `text-gray-600`, `text-gray-800`, `text-red-600`

## 8. Verification

- [x] 8.1 `cargo check` passes
- [x] 8.2 `cargo test --package shifty-dioxus` passes (234 tests, including 17 `my_shifts` tests)
- [x] 8.3 `cargo clippy --no-deps` produces no new warnings in `src/page/my_shifts.rs`
- [x] 8.4 `cargo fmt -- --check` passes
- [x] 8.5 `openspec validate "redesign-05-page-myshifts" --strict` passes
